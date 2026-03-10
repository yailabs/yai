#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI="$REPO/build/bin/yai"
SOCK="${YAI_RUNTIME_INGRESS:-$HOME/.yai/run/control.sock}"
WS="ws_inspect_v1"
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

(cd "$REPO" && "$YAI" >/tmp/yai_workspace_inspect_runtime.log 2>&1) &
RUNTIME_PID=$!

for _ in $(seq 1 50); do
  [[ -S "$SOCK" ]] && break
  sleep 0.1
done
[[ -S "$SOCK" ]] || { echo "workspace_inspect_surfaces_v1: FAIL (missing ingress socket)"; exit 1; }

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


def call(ws_id, command_id, argv=None, extra=None):
    if argv is None:
        argv = []
    if extra is None:
        extra = {}

    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.connect(SOCK)

    hs_payload = struct.pack(REQ_FMT, YAI_PROTOCOL_IDS_VERSION, 0, b"yai-test")
    s.sendall(build(YAI_CMD_HANDSHAKE, ws_id, hs_payload, "hs"))
    env = recv_exact(s, 96)
    _, _, _, _, cmd, _, _, _, plen, _ = struct.unpack(ENV_FMT, env)
    if cmd != YAI_CMD_HANDSHAKE:
        raise RuntimeError("bad handshake")
    recv_exact(s, plen)

    payload_obj = {
        "type": "yai.control.call.v1",
        "command_id": command_id,
        "target_plane": "runtime",
        "argv": argv,
    }
    payload_obj.update(extra)
    payload = json.dumps(payload_obj).encode("utf-8")
    s.sendall(build(YAI_CMD_CONTROL_CALL, ws_id, payload, "call"))

    env = recv_exact(s, 96)
    _, _, _, _, cmd, _, _, _, plen, _ = struct.unpack(ENV_FMT, env)
    if cmd != YAI_CMD_CONTROL_CALL:
        raise RuntimeError("bad control response")
    body = recv_exact(s, plen).decode("utf-8")
    s.close()
    return json.loads(body)

# normalize binding state before assertions
r = call("system", "yai.workspace.status")
assert r["status"] == "ok"
if r["data"]["binding_status"] != "no_active":
    _ = call("system", "yai.workspace.unset")

# create + activate
r = call(WS, "yai.workspace.create", [WS])
assert r["status"] == "ok"
r = call("system", "yai.workspace.set", [WS])
assert r["status"] == "ok"

# current/status/inspect
r = call("system", "yai.workspace.current")
assert r["status"] == "ok"
assert r["data"]["binding_status"] == "active"
assert r["data"]["workspace_id"] == WS

r = call("system", "yai.workspace.status")
assert r["status"] == "ok"
assert r["data"]["binding_status"] == "active"
assert r["data"]["active"] is True
assert "runtime_capabilities" in r["data"]
assert "data" in r["data"]["runtime_capabilities"]
assert "graph" in r["data"]["runtime_capabilities"]
assert "knowledge" in r["data"]["runtime_capabilities"]
assert "exec" in r["data"]["runtime_capabilities"]
assert r["data"]["security_level_declared"] in ("logical", "scoped", "isolated", "sandboxed")
assert r["data"]["security_level_effective"] in ("logical", "scoped", "isolated", "sandboxed")

r = call("system", "yai.workspace.inspect")
assert r["status"] == "ok"
assert r["data"]["identity"]["workspace_id"] == WS
assert "normative" in r["data"]
assert "security" in r["data"]
assert "governance" in r["data"]
assert "runtime_capabilities" in r["data"]
assert "graph_persistence" in r["data"]
assert "knowledge_transient_persistence" in r["data"]
assert r["data"]["security"]["level_declared"] in ("logical", "scoped", "isolated", "sandboxed")
assert r["data"]["security"]["capabilities"]["sandbox_ready"] is True

# policy effective surface baseline
r = call("system", "yai.workspace.policy_effective")
assert r["status"] == "ok"
assert "policy_attachments" in r["data"]
assert "policy_attachment_count" in r["data"]
assert "runtime_capabilities" in r["data"]

# domain set/get valid
r = call("system", "yai.workspace.domain_set", ["--family", "economic", "--specialization", "payments"])
assert r["status"] == "ok"
assert r["data"]["declared"]["family"] == "economic"
assert r["data"]["declared"]["specialization"] == "payments"

r = call("system", "yai.workspace.domain_get")
assert r["status"] == "ok"
assert r["data"]["declared"]["family"] == "economic"
assert r["data"]["declared"]["specialization"] == "payments"

# domain set invalid
r = call("system", "yai.workspace.domain.set", ["--family", "economic", "--specialization", "parameter-governance"])
assert r["status"] == "error"
assert r["code"] == "BAD_ARGS"

# perform one runtime resolution call to populate summaries
r = call(WS, "yai.runtime.ping", ["payment.authorize", "provider=bank", "resource=money-transfer"])
assert r["status"] in ("ok", "error")

# policy effective + debug summary available
r = call("system", "yai.workspace.policy_effective")
assert r["status"] == "ok"
assert r["data"]["workspace_id"] == WS
assert "effect_summary" in r["data"]
assert r["data"]["security_level_effective"] in ("logical", "scoped", "isolated", "sandboxed")

r = call("system", "yai.workspace.debug_resolution")
assert r["status"] == "ok"
assert r["data"]["workspace_id"] == WS
assert "effective" in r["data"]
assert "runtime_capabilities" in r["data"]
assert r["data"]["security_level_effective"] in ("logical", "scoped", "isolated", "sandboxed")

# clear + no active
r = call("system", "yai.workspace.unset")
assert r["status"] == "ok"
r = call("system", "yai.workspace.status")
assert r["status"] == "ok"
assert r["data"]["binding_status"] == "no_active"

# stale and invalid binding reflected in status
bind_path = os.path.expanduser("~/.yai/session/active_workspace.json")
os.makedirs(os.path.dirname(bind_path), exist_ok=True)
with open(bind_path, "w", encoding="utf-8") as f:
    json.dump({
        "type": "yai.workspace.binding.v1",
        "workspace_id": "ws_missing_for_status",
        "workspace_alias": "ws_missing_for_status",
        "bound_at": 0,
        "source": "explicit"
    }, f)

r = call("system", "yai.workspace.status")
assert r["status"] == "ok"
assert r["data"]["binding_status"] == "stale"

with open(bind_path, "w", encoding="utf-8") as f:
    json.dump({
        "type": "yai.workspace.binding.v1",
        "workspace_id": "bad/id",
        "workspace_alias": "bad",
        "bound_at": 0,
        "source": "explicit"
    }, f)

r = call("system", "yai.workspace.status")
assert r["status"] == "ok"
assert r["data"]["binding_status"] == "invalid"
PY

echo "workspace_inspect_surfaces_v1: ok"
