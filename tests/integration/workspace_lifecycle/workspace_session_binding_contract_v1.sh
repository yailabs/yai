#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI="$REPO/build/bin/yai"
SOCK="${YAI_RUNTIME_INGRESS:-$HOME/.yai/run/control.sock}"
BIND_FILE="$HOME/.yai/session/active_workspace.json"
WS_GOOD="ws_bind_good"
WS_SWITCH="ws_bind_switch"
WS_STALE="ws_bind_stale"

if [[ ! -x "$YAI" ]]; then
  make -C "$REPO" yai >/dev/null
fi

"$YAI" down >/dev/null 2>&1 || true
rm -f "$SOCK" >/dev/null 2>&1 || true
rm -f "$BIND_FILE" >/dev/null 2>&1 || true

RUNTIME_PID=""
cleanup() {
  rm -f "$BIND_FILE" || true
  if [[ -n "$RUNTIME_PID" ]] && kill -0 "$RUNTIME_PID" 2>/dev/null; then
    kill "$RUNTIME_PID" >/dev/null 2>&1 || true
    wait "$RUNTIME_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

"$YAI" >/tmp/yai_workspace_binding_runtime.log 2>&1 &
RUNTIME_PID=$!

for _ in $(seq 1 50); do
  [[ -S "$SOCK" ]] && break
  sleep 0.1
done
[[ -S "$SOCK" ]] || { echo "workspace_session_binding_contract_v1: FAIL (missing ingress socket)"; exit 1; }

python3 - "$SOCK" "$WS_GOOD" "$WS_SWITCH" "$WS_STALE" <<'PY'
import json
import os
import socket
import struct
import sys

SOCK = sys.argv[1]
WS_GOOD = sys.argv[2]
WS_SWITCH = sys.argv[3]
WS_STALE = sys.argv[4]

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
        raise RuntimeError("bad handshake response")
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


# 1) no active binding
r = call("system", "yai.workspace.current")
assert r["status"] == "ok"
assert r["data"]["binding_status"] == "no_active"

# 2) create + activate
r = call(WS_GOOD, "yai.workspace.create", [WS_GOOD])
assert r["status"] == "ok"

r = call("system", "yai.workspace.activate", [WS_GOOD])
assert r["status"] == "ok"
assert r["data"]["binding_status"] == "active"

# 3) current should resolve active
r = call("system", "yai.workspace.current")
assert r["status"] == "ok"
assert r["data"]["binding_status"] == "active"
assert r["data"]["workspace_id"] == WS_GOOD

# 4) prompt context should be compact and active
r = call("system", "yai.workspace.prompt_context")
assert r["status"] == "ok"
assert r["data"]["binding_status"] == "active"
assert r["data"]["workspace_id"] == WS_GOOD

# 4b) switch should simply change active workspace
r = call(WS_SWITCH, "yai.workspace.create", [WS_SWITCH])
assert r["status"] == "ok"
r = call("system", "yai.workspace.switch", [WS_SWITCH])
assert r["status"] == "ok"
r = call("system", "yai.workspace.current")
assert r["status"] == "ok"
assert r["data"]["binding_status"] == "active"
assert r["data"]["workspace_id"] == WS_SWITCH

# 5) stale: activate stale id directly by injecting binding file path through command
# runtime supports activation only for existing workspaces; simulate stale by writing binding to missing ws
bind_path = os.path.expanduser("~/.yai/session/active_workspace.json")
os.makedirs(os.path.dirname(bind_path), exist_ok=True)
with open(bind_path, "w", encoding="utf-8") as f:
    json.dump({
        "type": "yai.workspace.binding.v1",
        "workspace_id": WS_STALE,
        "workspace_alias": WS_STALE,
        "bound_at": 0,
        "source": "explicit"
    }, f)

r = call("system", "yai.workspace.current")
assert r["status"] == "ok"
assert r["data"]["binding_status"] == "stale"

# 6) invalid binding
with open(bind_path, "w", encoding="utf-8") as f:
    json.dump({
        "type": "yai.workspace.binding.v1",
        "workspace_id": "bad/id",
        "workspace_alias": "bad",
        "bound_at": 0,
        "source": "explicit"
    }, f)

r = call("system", "yai.workspace.current")
assert r["status"] == "ok"
assert r["data"]["binding_status"] == "invalid"

# 7) clear
r = call("system", "yai.workspace.unset")
assert r["status"] == "ok"
assert r["data"]["binding_status"] == "no_active"

r = call("system", "yai.workspace.current")
assert r["status"] == "ok"
assert r["data"]["binding_status"] == "no_active"
PY

echo "workspace_session_binding_contract_v1: ok"
