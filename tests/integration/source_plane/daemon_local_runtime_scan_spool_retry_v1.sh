#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI="$REPO/build/bin/yai"
YAI_DAEMON="$REPO/build/bin/yai-daemon"
SOCK="${TMPDIR:-/tmp}/yai-yd5-owner-$$.sock"

if [[ ! -x "$YAI" || ! -x "$YAI_DAEMON" ]]; then
  make -C "$REPO" yai yai-daemon >/dev/null
fi

OWNER_HOME="$(mktemp -d "${TMPDIR:-/tmp}/yai_yd5_owner.XXXXXX")"
DAEMON_HOME="$(mktemp -d "${TMPDIR:-/tmp}/yai_yd5_daemon.XXXXXX")"
SRC_DIR="$DAEMON_HOME/sources/inbox"
WS="yd5_source_ws"
RUNTIME_PID=""

cleanup() {
  HOME="$OWNER_HOME" YAI_RUNTIME_INGRESS="$SOCK" "$YAI" down >/dev/null 2>&1 || true
  if [[ -n "$RUNTIME_PID" ]] && kill -0 "$RUNTIME_PID" 2>/dev/null; then
    kill "$RUNTIME_PID" >/dev/null 2>&1 || true
    wait "$RUNTIME_PID" >/dev/null 2>&1 || true
  fi
  rm -f "$SOCK" >/dev/null 2>&1 || true
  if [[ "${YD5_KEEP_TMP:-0}" != "1" ]]; then
    rm -rf "$OWNER_HOME" "$DAEMON_HOME"
  else
    echo "daemon_local_runtime_scan_spool_retry_v1: kept tmp dirs owner=$OWNER_HOME daemon=$DAEMON_HOME"
  fi
}
trap cleanup EXIT

mkdir -p "$SRC_DIR" "$DAEMON_HOME/config"
cat >"$SRC_DIR/a.txt" <<'TXT'
yd5 sample payload
TXT

cat >"$DAEMON_HOME/config/source-bindings.manifest.json" <<JSON
{
  "bindings": [
    {
      "workspace_id": "$WS",
      "root_path": "$SRC_DIR",
      "scope": "workspace",
      "asset_type": "file",
      "enabled": true
    }
  ]
}
JSON

rm -f "$SOCK" >/dev/null 2>&1 || true

# Phase 1: owner unavailable -> spool must populate
YAI_DAEMON_HOME="$DAEMON_HOME" \
YAI_DAEMON_OWNER_REF="unix://$SOCK" \
YAI_DAEMON_SOURCE_LABEL="yd5-node-a" \
YAI_DAEMON_BINDINGS_MANIFEST="$DAEMON_HOME/config/source-bindings.manifest.json" \
"$YAI_DAEMON" --tick-ms 120 --max-ticks 8 >/tmp/yai_yd5_daemon_phase1.log 2>&1 || true

Q1="$(find "$DAEMON_HOME/spool/queue" -type f 2>/dev/null | wc -l | tr -d ' ')"
if [[ "$Q1" -lt 1 ]]; then
  echo "daemon_local_runtime_scan_spool_retry_v1: debug queue_count=$Q1 daemon_home=$DAEMON_HOME"
  find "$DAEMON_HOME" -maxdepth 4 2>/dev/null | sort | sed -n '1,220p'
  sed -n '1,220p' /tmp/yai_yd5_daemon_phase1.log 2>/dev/null || true
  echo "daemon_local_runtime_scan_spool_retry_v1: FAIL (queue not populated in disconnected phase)"
  exit 1
fi

# Phase 2: owner available -> retry and delivery
HOME="$OWNER_HOME" YAI_RUNTIME_INGRESS="$SOCK" "$YAI" down >/dev/null 2>&1 || true
(cd "$REPO" && HOME="$OWNER_HOME" YAI_RUNTIME_INGRESS="$SOCK" "$YAI" >/tmp/yai_yd5_owner_runtime.log 2>&1) &
RUNTIME_PID=$!

for _ in $(seq 1 80); do
  [[ -S "$SOCK" ]] && break
  sleep 0.1
done
[[ -S "$SOCK" ]] || { echo "daemon_local_runtime_scan_spool_retry_v1: FAIL (owner socket not ready)"; exit 1; }

HOME="$OWNER_HOME" YAI_RUNTIME_INGRESS="$SOCK" python3 - "$SOCK" "$WS" <<'PY'
import json
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

def call(ws_id, body, trace):
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.connect(SOCK)
    hs_payload = struct.pack(REQ_FMT, YAI_PROTOCOL_IDS_VERSION, 0, b"yai-yd5")
    s.sendall(build(YAI_CMD_HANDSHAKE, ws_id, hs_payload, "hs-yd5"))
    env = recv_exact(s, 96)
    _, _, _, _, cmd, _, _, _, plen, _ = struct.unpack(ENV_FMT, env)
    if cmd != YAI_CMD_HANDSHAKE:
        raise RuntimeError("bad handshake")
    recv_exact(s, plen)
    payload = json.dumps(body).encode("utf-8")
    s.sendall(build(YAI_CMD_CONTROL_CALL, ws_id, payload, trace))
    env = recv_exact(s, 96)
    _, _, _, _, cmd, _, _, _, plen, _ = struct.unpack(ENV_FMT, env)
    if cmd != YAI_CMD_CONTROL_CALL:
        raise RuntimeError("bad control response")
    body = recv_exact(s, plen).decode("utf-8")
    s.close()
    return json.loads(body)

def expect_ok(r, lbl):
    if r.get("status") != "ok":
        raise RuntimeError(f"{lbl} not ok: {r}")

expect_ok(call("system", {
  "type":"yai.control.call.v1",
  "command_id":"yai.workspace.create",
  "target_plane":"runtime",
  "argv":[WS]
}, "create"), "create")
expect_ok(call("system", {
  "type":"yai.control.call.v1",
  "command_id":"yai.workspace.set",
  "target_plane":"runtime",
  "argv":[WS]
}, "set"), "set")
PY

YAI_DAEMON_HOME="$DAEMON_HOME" \
YAI_DAEMON_OWNER_REF="unix://$SOCK" \
YAI_DAEMON_SOURCE_LABEL="yd5-node-a" \
YAI_DAEMON_BINDINGS_MANIFEST="$DAEMON_HOME/config/source-bindings.manifest.json" \
"$YAI_DAEMON" --tick-ms 120 --max-ticks 18 >/tmp/yai_yd5_daemon_phase2.log 2>&1

D2="$(find "$DAEMON_HOME/spool/delivered" -type f 2>/dev/null | wc -l | tr -d ' ')"
if [[ "$D2" -lt 1 ]]; then
  echo "daemon_local_runtime_scan_spool_retry_v1: FAIL (no delivered units after owner recovery)"
  exit 1
fi

if [[ ! -f "$DAEMON_HOME/state/health.v1.json" ]]; then
  echo "daemon_local_runtime_scan_spool_retry_v1: FAIL (missing daemon health state file)"
  exit 1
fi
if ! rg -n '"state":"ready"|"state":"degraded"|"state":"stopping"' "$DAEMON_HOME/state/health.v1.json" >/dev/null; then
  echo "daemon_local_runtime_scan_spool_retry_v1: FAIL (unexpected health state payload)"
  exit 1
fi

if [[ ! -f "$DAEMON_HOME/state/bindings.v1.json" ]]; then
  echo "daemon_local_runtime_scan_spool_retry_v1: FAIL (missing bindings state file)"
  exit 1
fi
if ! rg -n '"status":"active"|\"status\":\"degraded\"' "$DAEMON_HOME/state/bindings.v1.json" >/dev/null; then
  echo "daemon_local_runtime_scan_spool_retry_v1: FAIL (bindings state does not expose runtime status)"
  exit 1
fi

echo "daemon_local_runtime_scan_spool_retry_v1: ok"
