#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI="$REPO/build/bin/yai"
SOCK="${YAI_RUNTIME_INGRESS:-$HOME/.yai/run/control.sock}"
WS="ws_negative_v1"
BIND_FILE="$HOME/.yai/session/active_workspace.json"

if [[ ! -x "$YAI" ]]; then
  make -C "$REPO" yai >/dev/null
fi

"$YAI" down >/dev/null 2>&1 || true
rm -f "$SOCK" >/dev/null 2>&1 || true
rm -f "$BIND_FILE" >/dev/null 2>&1 || true

RUNTIME_PID=""
cleanup() {
  if [[ -n "$RUNTIME_PID" ]] && kill -0 "$RUNTIME_PID" 2>/dev/null; then
    kill "$RUNTIME_PID" >/dev/null 2>&1 || true
    wait "$RUNTIME_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

"$YAI" >/tmp/yai_workspace_negative_runtime.log 2>&1 &
RUNTIME_PID=$!

for _ in $(seq 1 50); do
  [[ -S "$SOCK" ]] && break
  sleep 0.1
done
[[ -S "$SOCK" ]] || { echo "workspace_negative_paths_v1: FAIL (missing ingress socket)"; exit 1; }

python3 - "$SOCK" "$WS" <<'PY'
import json
import os
import socket
import struct
import sys

SOCK = sys.argv[1]
WS = sys.argv[2]

YAI_FRAME_MAGIC = 0x59414950
YAI_PROTOCOL_IDS_VERSION = 1
YAI_CMD_HANDSHAKE = 0x0102
YAI_CMD_CONTROL_CALL = 0x0105
ENV_FMT = "<II36s36sIHBBII"
REQ_FMT = "<II32s"

def build(cmd_id, ws_id, payload, trace):
    ws = ws_id.encode("utf-8")[:36].ljust(36, b"\0")
    tr = trace.encode("utf-8")[:36].ljust(36, b"\0")
    env = struct.pack(ENV_FMT, YAI_FRAME_MAGIC, YAI_PROTOCOL_IDS_VERSION, ws, tr, cmd_id, 2, 1, 0, len(payload), 0)
    return env + payload

def recv_exact(sock, n):
    out = b""
    while len(out) < n:
        c = sock.recv(n - len(out))
        if not c:
            raise RuntimeError("eof")
        out += c
    return out

def call(ws_id, command_id, argv=None):
    if argv is None:
        argv = []

    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.connect(SOCK)

    hs_payload = struct.pack(REQ_FMT, YAI_PROTOCOL_IDS_VERSION, 0, b"yai-test")
    s.sendall(build(YAI_CMD_HANDSHAKE, ws_id, hs_payload, "hs"))
    env = recv_exact(s, 96)
    _, _, _, _, cmd, _, _, _, plen, _ = struct.unpack(ENV_FMT, env)
    if cmd != YAI_CMD_HANDSHAKE:
        raise RuntimeError("bad handshake")
    recv_exact(s, plen)

    payload = json.dumps({
        "type": "yai.control.call.v1",
        "command_id": command_id,
        "target_plane": "runtime",
        "argv": argv,
    }).encode("utf-8")

    s.sendall(build(YAI_CMD_CONTROL_CALL, ws_id, payload, "call"))
    env = recv_exact(s, 96)
    _, _, _, _, cmd, _, _, _, plen, _ = struct.unpack(ENV_FMT, env)
    if cmd != YAI_CMD_CONTROL_CALL:
        raise RuntimeError("bad control response")
    body = recv_exact(s, plen).decode("utf-8")
    s.close()
    return json.loads(body)

# no active workspace: domain_set must fail
r = call("system", "yai.workspace.domain_set", ["--family", "economic", "--specialization", "payments"])
assert r["status"] == "error", r
assert r["code"] == "BAD_ARGS", r

# workspace exists but not active: domain_set still fails
r = call(WS, "yai.workspace.create", [WS])
assert r["status"] == "ok", r
r = call("system", "yai.workspace.domain_set", ["--family", "economic", "--specialization", "payments"])
assert r["status"] == "error", r
assert r["code"] == "BAD_ARGS", r

# activate workspace
r = call("system", "yai.workspace.activate", [WS])
assert r["status"] == "ok", r

# invalid family
r = call("system", "yai.workspace.domain_set", ["--family", "not-a-family", "--specialization", "payments"])
assert r["status"] == "error", r
assert r["reason"] == "family_not_found", r

# invalid specialization
r = call("system", "yai.workspace.domain_set", ["--family", "economic", "--specialization", "not-a-specialization"])
assert r["status"] == "error", r
assert r["reason"] == "specialization_not_found", r

# incompatible specialization/family
r = call("system", "yai.workspace.domain_set", ["--family", "economic", "--specialization", "parameter-governance"])
assert r["status"] == "error", r
assert r["reason"] == "specialization_family_mismatch", r

# valid declaration then out-of-context action still resolves with coherent outputs
r = call("system", "yai.workspace.domain_set", ["--family", "economic", "--specialization", "payments"])
assert r["status"] == "ok", r
r = call(WS, "yai.workspace.run", ["experiment.run", "provider=lab", "resource=model-card"])
assert r["status"] in ("ok", "error"), r
assert "decision" in r["data"], r

# policy/debug surfaces remain readable after mixed signal
p = call("system", "yai.workspace.policy_effective")
assert p["status"] == "ok", p
assert p["data"]["workspace_id"] == WS, p
assert p["data"]["family_effective"], p
assert p["data"]["specialization_effective"], p

q = call("system", "yai.workspace.debug_resolution")
assert q["status"] == "ok", q
assert q["data"]["declared"]["family"] == "economic", q
assert q["data"]["effective"]["stack_ref"], q

# unset workspace and verify no_active status
r = call("system", "yai.workspace.unset")
assert r["status"] == "ok", r
s = call("system", "yai.workspace.status")
assert s["status"] == "ok", s
assert s["data"]["binding_status"] == "no_active", s
PY

echo "workspace_negative_paths_v1: ok"
