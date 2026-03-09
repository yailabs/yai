#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI="$REPO/build/bin/yai"
SOCK="${YAI_RUNTIME_INGRESS:-$HOME/.yai/run/control.sock}"
WS_DEFAULT="ws_arch_default"
WS_ABS="ws_arch_abs"

if [[ ! -x "$YAI" ]]; then
  make -C "$REPO" yai >/dev/null
fi

"$YAI" down >/dev/null 2>&1 || true
rm -f "$SOCK" >/dev/null 2>&1 || true

RUNTIME_PID=""
cleanup() {
  if [[ -n "$RUNTIME_PID" ]] && kill -0 "$RUNTIME_PID" 2>/dev/null; then
    kill "$RUNTIME_PID" >/dev/null 2>&1 || true
    wait "$RUNTIME_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

(cd /tmp && "$YAI" >/tmp/yai_workspace_arch_runtime.log 2>&1) &
RUNTIME_PID=$!

for _ in $(seq 1 50); do
  [[ -S "$SOCK" ]] && break
  sleep 0.1
done
[[ -S "$SOCK" ]] || { echo "workspace_architecture_boundary_v1: FAIL (missing ingress socket)"; exit 1; }

python3 - "$SOCK" "$WS_DEFAULT" "$WS_ABS" <<'PY'
import json
import os
import socket
import struct
import sys

SOCK = sys.argv[1]
WS_DEFAULT = sys.argv[2]
WS_ABS = sys.argv[3]

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

    payload_obj = {
        "type": "yai.control.call.v1",
        "command_id": command_id,
        "target_plane": "runtime",
        "argv": argv,
    }
    payload = json.dumps(payload_obj).encode("utf-8")
    s.sendall(build(YAI_CMD_CONTROL_CALL, ws_id, payload, "call"))
    env = recv_exact(s, 96)
    _, _, _, _, cmd, _, _, _, plen, _ = struct.unpack(ENV_FMT, env)
    if cmd != YAI_CMD_CONTROL_CALL:
        raise RuntimeError("bad control response")
    body = recv_exact(s, plen).decode("utf-8")
    s.close()
    return json.loads(body)


# default-root workspace
r = call(WS_DEFAULT, "yai.workspace.create", [WS_DEFAULT])
assert r["status"] == "ok"
assert r["data"]["workspace_store_root"].endswith("/.yai/workspaces")
assert r["data"]["runtime_state_root"].endswith(f"/.yai/run/{WS_DEFAULT}")
assert r["data"]["metadata_root"].endswith(f"/.yai/run/{WS_DEFAULT}")
assert r["data"]["root_anchor_mode"] in ("managed_default_root", "managed_custom_root")

r = call("system", "yai.workspace.activate", [WS_DEFAULT])
assert r["status"] == "ok"
assert r["data"]["binding_scope"] == "session"

r = call("system", "yai.workspace.current")
assert r["status"] == "ok"
assert r["data"]["workspace_store_root"].endswith("/.yai/workspaces")
assert r["data"]["runtime_state_root"].endswith(f"/.yai/run/{WS_DEFAULT}")
assert r["data"]["shell_path_relation"] in ("inside_workspace_root", "outside_workspace_root", "workspace_root_unset")

r = call("system", "yai.workspace.inspect")
assert r["status"] == "ok"
assert "root_model" in r["data"]
assert "shell" in r["data"]
assert r["data"]["root_model"]["runtime_state_root"].endswith(f"/.yai/run/{WS_DEFAULT}")
assert r["data"]["shell"]["cwd_relation"] in ("inside_workspace_root", "outside_workspace_root", "workspace_root_unset", "cwd_unavailable")

r = call("system", "yai.workspace.status")
assert r["status"] == "ok"
assert r["data"]["workspace_root"].endswith(f"/.yai/workspaces/{WS_DEFAULT}")
assert r["data"]["shell_path_relation"] in ("inside_workspace_root", "outside_workspace_root", "workspace_root_unset", "cwd_unavailable")

# explicit absolute root workspace
abs_root = f"/tmp/{WS_ABS}"
r = call(WS_ABS, "yai.workspace.create", [WS_ABS, "--root", abs_root])
assert r["status"] == "ok"
assert r["data"]["root_path"] == abs_root
assert r["data"]["root_anchor_mode"] == "explicit_absolute"
PY

echo "workspace_architecture_boundary_v1: ok"

