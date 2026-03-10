#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI="$REPO/build/bin/yai"
SOCK="${YAI_RUNTIME_INGRESS:-$HOME/.yai/run/control.sock}"
WS="ws_brain_graph_transient_dp7_v1"
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

(cd "$REPO" && "$YAI" >/tmp/yai_workspace_brain_graph_transient_dp7.log 2>&1) &
RUNTIME_PID=$!

for _ in $(seq 1 120); do
  [[ -S "$SOCK" ]] && break
  sleep 0.1
done
[[ -S "$SOCK" ]] || { echo "workspace_brain_graph_transient_dp7_v1: FAIL (missing ingress socket)"; exit 1; }

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


def last_ndjson(path):
    with open(path, "r", encoding="utf-8") as f:
        lines = [ln.strip() for ln in f.readlines() if ln.strip()]
    assert lines, path
    return json.loads(lines[-1])


r = call(WS, "yai.workspace.create", [WS]); assert r["status"] == "ok", r
r = call("system", "yai.workspace.set", [WS]); assert r["status"] == "ok", r

r = None
for _ in range(20):
    r = call("system", "yai.workspace.domain_set", ["--family", "digital", "--specialization", "remote-publication"])
    if r["status"] == "ok":
        break
    time.sleep(0.1)
assert r is not None and r["status"] == "ok", r

r = call(WS, "yai.workspace.run", ["digital.publish", "sink=external_untrusted", "contract=missing", "artifact=bundle-v1"])
assert r["status"] in ("ok", "error"), r

for command_id in ("yai.workspace.inspect", "yai.workspace.policy_effective", "yai.workspace.debug_resolution"):
    out = call("system", command_id)
    assert out["status"] == "ok", out
    bp = out["data"].get("brain_persistence")
    assert isinstance(bp, dict), (command_id, out)
    assert bp.get("last_graph_node_ref", "").startswith("bgn-"), (command_id, bp)
    assert bp.get("last_graph_edge_ref", "").startswith("bge-"), (command_id, bp)
    assert bp.get("last_transient_state_ref", "").startswith("btc-"), (command_id, bp)
    assert bp.get("last_transient_working_set_ref", "").startswith("bws-"), (command_id, bp)
    assert bp.get("graph_truth_authoritative") is True, (command_id, bp)
    assert bp.get("transient_authoritative") is False, (command_id, bp)

base = os.path.join(HOME, ".yai", "run", WS, "runtime")
graph_nodes_log = os.path.join(base, "graph", "persistent-nodes.v1.ndjson")
graph_edges_log = os.path.join(base, "graph", "persistent-edges.v1.ndjson")
graph_index = os.path.join(base, "graph", "index.v1.json")
transient_activation_log = os.path.join(base, "transient", "activation-state.v1.ndjson")
transient_working_set_log = os.path.join(base, "transient", "working-set.v1.ndjson")
transient_index = os.path.join(base, "transient", "index.v1.json")

for path in (
    graph_nodes_log,
    graph_edges_log,
    graph_index,
    transient_activation_log,
    transient_working_set_log,
    transient_index,
):
    assert os.path.exists(path), path

with open(graph_index, "r", encoding="utf-8") as f:
    gidx = json.load(f)
with open(transient_index, "r", encoding="utf-8") as f:
    tidx = json.load(f)

assert gidx["type"] == "yai.graph.index.v1", gidx
assert gidx["workspace_id"] == WS, gidx
assert gidx["graph_truth_authoritative"] is True, gidx
assert gidx["backend_role"] == "BR-3", gidx
assert gidx["last_graph_node_ref"].startswith("bgn-"), gidx
assert gidx["last_graph_edge_ref"].startswith("bge-"), gidx
assert gidx["last_decision_ref"].startswith("dec-"), gidx
assert gidx["last_evidence_ref"].startswith("evd-"), gidx

assert tidx["type"] == "yai.transient.index.v1", tidx
assert tidx["workspace_id"] == WS, tidx
assert tidx["authoritative"] is False, tidx
assert tidx["backend_role"] == "BR-4", tidx
assert tidx["last_transient_state_ref"].startswith("btc-"), tidx
assert tidx["last_working_set_ref"].startswith("bws-"), tidx
assert tidx["last_decision_ref"].startswith("dec-"), tidx

gn = last_ndjson(graph_nodes_log)
ge = last_ndjson(graph_edges_log)
ts = last_ndjson(transient_activation_log)
ws = last_ndjson(transient_working_set_log)

assert gn["type"] == "yai.graph_node.v1", gn
assert ge["type"] == "yai.graph_edge.v1", ge
assert ts["type"] == "yai.transient_cognition_state.v1", ts
assert ts["authoritative"] is False, ts
assert ws["type"] == "yai.transient_working_set.v1", ws
assert ws["authoritative"] is False, ws

r = call("system", "yai.workspace.unset")
assert r["status"] == "ok", r
PY

echo "workspace_brain_graph_transient_dp7_v1: ok"
