#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI="$REPO/build/bin/yai"
SOCK="${YAI_RUNTIME_INGRESS:-$HOME/.yai/run/control.sock}"
WS="ws_graph_materialization_dp11_v1"
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

(cd "$REPO" && "$YAI" >/tmp/yai_workspace_graph_materialization_dp11.log 2>&1) &
RUNTIME_PID=$!

for _ in $(seq 1 120); do
  [[ -S "$SOCK" ]] && break
  sleep 0.1
done
[[ -S "$SOCK" ]] || { echo "workspace_graph_materialization_hooks_dp11_v1: FAIL (missing ingress socket)"; exit 1; }

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


def load_ndjson(path):
    with open(path, "r", encoding="utf-8") as f:
        return [json.loads(ln) for ln in f.read().splitlines() if ln.strip()]

r = call(WS, "yai.workspace.create", [WS]); assert r["status"] == "ok", r
r = call("system", "yai.workspace.set", [WS]); assert r["status"] == "ok", r

for _ in range(20):
    r = call("system", "yai.workspace.domain_set", ["--family", "digital", "--specialization", "remote-publication"])
    if r["status"] == "ok":
        break
    time.sleep(0.1)
assert r["status"] == "ok", r

r = call(WS, "yai.workspace.run", ["digital.publish", "sink=external_untrusted", "contract=missing", "artifact=bundle-v1"])
assert r["status"] in ("ok", "error"), r

base = os.path.join(HOME, ".yai", "run", WS, "runtime")
graph_nodes_log = os.path.join(base, "graph", "persistent-nodes.v1.ndjson")
graph_edges_log = os.path.join(base, "graph", "persistent-edges.v1.ndjson")
graph_index = os.path.join(base, "graph", "index.v1.json")
for path in (graph_nodes_log, graph_edges_log, graph_index):
    assert os.path.exists(path), path

# Read-only surfaces must not append graph materialization rows.
node_before = len(load_ndjson(graph_nodes_log))
edge_before = len(load_ndjson(graph_edges_log))
for command_id in ("yai.workspace.inspect", "yai.workspace.policy_effective", "yai.workspace.debug_resolution"):
    out = call("system", command_id)
    assert out["status"] == "ok", out
    bp = out["data"].get("brain_persistence")
    assert isinstance(bp, dict), (command_id, out)
    assert bp.get("graph_truth_authoritative") is True, (command_id, bp)
    assert bp.get("transient_authoritative") is False, (command_id, bp)
node_after = len(load_ndjson(graph_nodes_log))
edge_after = len(load_ndjson(graph_edges_log))
assert node_before == node_after, (node_before, node_after)
assert edge_before == edge_after, (edge_before, edge_after)

nodes = load_ndjson(graph_nodes_log)
edges = load_ndjson(graph_edges_log)
assert len(nodes) >= 7, len(nodes)
assert len(edges) >= 8, len(edges)

node_classes = {n.get("node_class") for n in nodes}
edge_classes = {e.get("edge_class") for e in edges}

expected_node_classes = {
    "workspace_node",
    "governance_object_node",
    "decision_node",
    "evidence_node",
    "authority_node",
    "artifact_node",
    "runtime_episode_node",
}
expected_edge_classes = {
    "decision_in_workspace",
    "decision_under_governance",
    "decision_under_authority",
    "decision_on_artifact",
    "evidence_for_decision",
    "artifact_governed_by",
    "workspace_uses_governance",
    "episode_yielded_decision",
}
assert expected_node_classes.issubset(node_classes), node_classes
assert expected_edge_classes.issubset(edge_classes), edge_classes

for n in nodes[-7:]:
    assert n["type"] == "yai.graph_node.v1", n
    assert n.get("source_record_ref", "") != "", n
    assert n.get("origin_domain", "") != "", n

for e in edges[-8:]:
    assert e["type"] == "yai.graph_edge.v1", e
    assert e.get("decision_ref", "").startswith("dec-"), e

with open(graph_index, "r", encoding="utf-8") as f:
    gidx = json.load(f)
assert gidx["type"] == "yai.graph.index.v1", gidx
assert gidx["workspace_id"] == WS, gidx
assert gidx["graph_truth_authoritative"] is True, gidx
assert gidx["backend_role"] == "BR-3", gidx
assert gidx["materialization_mode"] == "runtime_record_driven", gidx
assert gidx["materialization_status"] == "complete", gidx
assert "decision_node" in gidx.get("node_classes", ""), gidx
assert "evidence_for_decision" in gidx.get("edge_classes", ""), gidx

# Query layer can read graph summary without backend leakage.
out = call("system", "yai.workspace.graph.summary")
assert out["status"] == "ok", out
assert out["data"]["query_family"] == "graph", out

r = call("system", "yai.workspace.unset")
assert r["status"] == "ok", r
PY

echo "workspace_graph_materialization_hooks_dp11_v1: ok"
