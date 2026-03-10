#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI="$REPO/build/bin/yai"
SOCK="${YAI_RUNTIME_INGRESS:-$HOME/.yai/run/control.sock}"
WS="ws_data_lifecycle_dp15b_v1"
BIND_FILE="$HOME/.yai/session/active_workspace.json"

if [[ ! -x "$YAI" ]]; then
  make -C "$REPO" yai >/dev/null
fi
make -C "$REPO" law-embed-sync >/dev/null

"$YAI" down >/dev/null 2>&1 || true
rm -f "$SOCK" >/dev/null 2>&1 || true
rm -f "$BIND_FILE" >/dev/null 2>&1 || true
rm -rf "$HOME/.yai/run/$WS" >/dev/null 2>&1 || true

cleanup() {
  if [[ -n "${RUNTIME_PID:-}" ]] && kill -0 "$RUNTIME_PID" 2>/dev/null; then
    kill "$RUNTIME_PID" >/dev/null 2>&1 || true
    wait "$RUNTIME_PID" >/dev/null 2>&1 || true
  fi
  "$YAI" down --force >/dev/null 2>&1 || true
}
trap cleanup EXIT

(cd "$REPO" && \
  YAI_LIFECYCLE_KEEP_EVENTS=6 \
  YAI_LIFECYCLE_KEEP_DECISIONS=6 \
  YAI_LIFECYCLE_KEEP_EVIDENCE=6 \
  YAI_LIFECYCLE_KEEP_GRAPH_NODES=8 \
  YAI_LIFECYCLE_KEEP_GRAPH_EDGES=8 \
  YAI_LIFECYCLE_KEEP_TRANSIENT=4 \
  "$YAI" >/tmp/yai_workspace_lifecycle_dp15b.log 2>&1) &
RUNTIME_PID=$!

for _ in $(seq 1 120); do
  [[ -S "$SOCK" ]] && break
  sleep 0.1
done
[[ -S "$SOCK" ]] || { echo "workspace_data_lifecycle_execution_dp15b_v1: FAIL (missing ingress socket)"; exit 1; }

python3 - "$SOCK" "$WS" "$HOME" <<'PY'
import json
import os
import socket
import struct
import sys
import time

SOCK = sys.argv[1]
WS = sys.argv[2]
HOME = sys.argv[3]

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


r = call(WS, "yai.workspace.create", [WS]); assert r["status"] == "ok", r
r = call("system", "yai.workspace.set", [WS]); assert r["status"] == "ok", r

for _ in range(20):
    r = call("system", "yai.workspace.domain_set", ["--family", "digital", "--specialization", "remote-publication"])
    if r["status"] == "ok":
        break
    time.sleep(0.1)
assert r["status"] == "ok", r

for i in range(24):
    rr = call(WS, "yai.workspace.run", [f"digital.publish", f"sink=external_untrusted", f"contract=missing_{i}", f"artifact=bundle-{i}"])
    assert rr["status"] in ("ok", "error"), rr

status0 = call("system", "yai.workspace.lifecycle.status")
assert status0["status"] == "ok", status0

maint = call("system", "yai.workspace.lifecycle.maintain")
assert maint["status"] == "ok", maint
md = maint["data"]
assert md["type"] == "yai.workspace.lifecycle.maintenance.v1", md
assert md["stats"]["files_touched"] > 0, md
assert md["stats"]["files_compacted"] > 0, md
assert md["stats"]["records_archived"] > 0, md
assert md["stats"]["violations"] == 0, md

status1 = call("system", "yai.workspace.lifecycle.status")
assert status1["status"] == "ok", status1
sd = status1["data"]
assert sd["status"] == "initialized", sd
assert sd["maintenance"]["status"] == "ok", sd

base = os.path.join(HOME, ".yai", "run", WS)
rollup = os.path.join(base, "lifecycle", "rollup.v1.json")
snapshot = os.path.join(base, "lifecycle", "snapshot.v1.json")
indexp = os.path.join(base, "lifecycle", "maintenance.index.v1.json")
archive_events = os.path.join(base, "archive", "events", "runtime-events.v1.ndjson")
archive_graph_edges = os.path.join(base, "archive", "runtime", "graph", "persistent-edges.v1.ndjson")

for path in (rollup, snapshot, indexp, archive_events):
    assert os.path.exists(path), path

with open(rollup, "r", encoding="utf-8") as f:
    rv = json.load(f)
assert rv["type"] == "yai.lifecycle.rollup.v1", rv
assert rv["execution"]["records_archived"] > 0, rv

with open(archive_events, "r", encoding="utf-8") as f:
    lines = [ln.strip() for ln in f.readlines() if ln.strip()]
assert lines, "archive events empty"
for ln in lines[-10:]:
    obj = json.loads(ln)
    if "workspace_id" in obj:
        assert obj["workspace_id"] == WS, obj
    if "workspace_ref" in obj:
        assert obj["workspace_ref"] == WS, obj

if os.path.exists(archive_graph_edges):
    with open(archive_graph_edges, "r", encoding="utf-8") as f:
        g_lines = [ln.strip() for ln in f.readlines() if ln.strip()]
    for ln in g_lines[-10:]:
        obj = json.loads(ln)
        assert obj.get("workspace_id", WS) == WS, obj

r = call("system", "yai.workspace.unset")
assert r["status"] == "ok", r
PY

echo "workspace_data_lifecycle_execution_dp15b_v1: ok"
