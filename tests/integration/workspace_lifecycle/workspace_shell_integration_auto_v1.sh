#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI="$REPO/build/bin/yai"
TMP_HOME="$(mktemp -d /tmp/yai-home-shell-auto.XXXXXX)"
SOCK="$TMP_HOME/.yai/run/control.sock"
WS1="ws_shell_auto_1"
WS2="ws_shell_auto_2"

if [[ ! -x "$YAI" ]]; then
  make -C "$REPO" yai >/dev/null
fi

RUNTIME_PID=""
cleanup() {
  if [[ -n "$RUNTIME_PID" ]] && kill -0 "$RUNTIME_PID" 2>/dev/null; then
    kill "$RUNTIME_PID" >/dev/null 2>&1 || true
    wait "$RUNTIME_PID" >/dev/null 2>&1 || true
  fi
  rm -rf "$TMP_HOME" || true
}
trap cleanup EXIT

mkdir -p "$TMP_HOME"
(cd "$REPO" && HOME="$TMP_HOME" YAI_RUNTIME_INGRESS="$SOCK" YAI_SHELL_INTEGRATION_MODE=managed "$YAI" >/tmp/yai_workspace_shell_auto_runtime.log 2>&1) &
RUNTIME_PID=$!

for _ in $(seq 1 50); do
  [[ -S "$SOCK" ]] && break
  sleep 0.1
done
[[ -S "$SOCK" ]] || { echo "workspace_shell_integration_auto_v1: FAIL (missing ingress socket)"; exit 1; }

HOME="$TMP_HOME" YAI_RUNTIME_INGRESS="$SOCK" YAI_SHELL_INTEGRATION_MODE=managed python3 - "$SOCK" "$WS1" "$WS2" <<'PY'
import json
import pathlib
import socket
import struct
import sys

SOCK = sys.argv[1]
WS1 = sys.argv[2]
WS2 = sys.argv[3]

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


def assert_true(cond, msg):
    if not cond:
        raise AssertionError(msg)


home = pathlib.Path.home()
zshrc = home / ".zshrc"
prompt_script = home / ".config" / "yai" / "shell" / "yai-prompt.zsh"

call(WS1, "yai.workspace.create", [WS1])
call("system", "yai.workspace.set", [WS1])

assert_true(prompt_script.exists(), "managed prompt script missing after activate")
assert_true(zshrc.exists(), "managed zshrc missing after activate")

z1 = zshrc.read_text(encoding="utf-8")
s1 = prompt_script.read_text(encoding="utf-8")

assert_true("# BEGIN YAI MANAGED SHELL INTEGRATION" in z1, "managed begin marker missing")
assert_true("# END YAI MANAGED SHELL INTEGRATION" in z1, "managed end marker missing")
assert_true(z1.count("# BEGIN YAI MANAGED SHELL INTEGRATION") == 1, "managed block duplicated")
assert_true("source \"$HOME/.config/yai/shell/yai-prompt.zsh\"" in z1, "managed source line missing")
assert_true("prompt_yai_ws" in s1, "prompt segment function missing")
assert_true(WS1 not in s1, "prompt script must not be workspace-specific")

call(WS2, "yai.workspace.create", [WS2])
call("system", "yai.workspace.set", [WS2])

z2 = zshrc.read_text(encoding="utf-8")
s2 = prompt_script.read_text(encoding="utf-8")

assert_true(z2.count("# BEGIN YAI MANAGED SHELL INTEGRATION") == 1, "managed block duplicated after switch")
assert_true(s2 == s1, "prompt script changed across workspace switch")
assert_true(WS2 not in s2, "prompt script must remain workspace-agnostic after switch")
PY

echo "workspace_shell_integration_auto_v1: ok"
