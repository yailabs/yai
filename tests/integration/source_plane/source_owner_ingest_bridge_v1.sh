#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI="$REPO/build/bin/yai"
TMP_HOME="$(mktemp -d "${TMPDIR:-/tmp}/yai_source_ingest_v1.XXXXXX")"
SOCK="/tmp/yai-source-ingest-v1.sock"
WS_ROOT="$TMP_HOME/.yai/run/data"

if [[ ! -x "$YAI" ]]; then
  make -C "$REPO" yai >/dev/null
fi

mkdir -p "$TMP_HOME/.yai"
HOME="$TMP_HOME" YAI_RUNTIME_INGRESS="$SOCK" "$YAI" down >/dev/null 2>&1 || true
rm -f "$SOCK" >/dev/null 2>&1 || true

RUNTIME_PID=""
cleanup() {
  HOME="$TMP_HOME" YAI_RUNTIME_INGRESS="$SOCK" "$YAI" down >/dev/null 2>&1 || true
  if [[ -n "$RUNTIME_PID" ]] && kill -0 "$RUNTIME_PID" 2>/dev/null; then
    kill "$RUNTIME_PID" >/dev/null 2>&1 || true
    wait "$RUNTIME_PID" >/dev/null 2>&1 || true
  fi
  rm -rf "$TMP_HOME"
}
trap cleanup EXIT

(cd "$REPO" && HOME="$TMP_HOME" YAI_RUNTIME_INGRESS="$SOCK" "$YAI" >/tmp/yai_source_owner_ingest.log 2>&1) &
RUNTIME_PID=$!

for _ in $(seq 1 50); do
  if [[ -S "$SOCK" ]]; then
    break
  fi
  sleep 0.1
done

if [[ ! -S "$SOCK" ]]; then
  echo "source_owner_ingest_bridge_v1: FAIL (missing ingress socket $SOCK)"
  exit 1
fi

HOME="$TMP_HOME" YAI_RUNTIME_INGRESS="$SOCK" python3 - <<'PY'
import socket
import struct
import json
import os

SOCK = os.environ.get("YAI_RUNTIME_INGRESS", os.path.expanduser("~/.yai/run/control.sock"))
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


def call(ws_id, body, trace="call-1"):
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.connect(SOCK)
    hs_payload = struct.pack(REQ_FMT, YAI_PROTOCOL_IDS_VERSION, 0, b"yai-test")
    s.sendall(build(YAI_CMD_HANDSHAKE, ws_id, hs_payload, "hs-1"))
    env = recv_exact(s, 96)
    _, _, _, _, cmd, _, _, _, plen, _ = struct.unpack(ENV_FMT, env)
    if cmd != YAI_CMD_HANDSHAKE:
      raise RuntimeError("bad handshake response")
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


def expect_ok(reply, why):
    if reply.get("status") != "ok":
      raise RuntimeError(f"{why}: expected ok, got {reply}")


def expect_error(reply, code, reason, why):
    if reply.get("status") != "error":
      raise RuntimeError(f"{why}: expected error, got {reply}")
    if reply.get("code") != code:
      raise RuntimeError(f"{why}: expected code {code}, got {reply}")
    if reason is not None and reply.get("reason") != reason:
      raise RuntimeError(f"{why}: expected reason {reason}, got {reply}")

def expect_exec_mediation(reply, why):
    med = reply.get("data", {}).get("mediation", {})
    if med.get("layer") != "exec":
      raise RuntimeError(f"{why}: missing exec mediation layer: {reply}")
    if med.get("route") != "owner_ingest_v1":
      raise RuntimeError(f"{why}: unexpected mediation route: {reply}")
    if med.get("transport_ready") is not True:
      raise RuntimeError(f"{why}: transport not ready in mediation payload: {reply}")
    if med.get("network_gate_ready") is not True:
      raise RuntimeError(f"{why}: network gate not ready in mediation payload: {reply}")
    if med.get("resource_gate_ready") is not True:
      raise RuntimeError(f"{why}: resource gate not ready in mediation payload: {reply}")
    if med.get("storage_gate_ready") is not True:
      raise RuntimeError(f"{why}: storage gate not ready in mediation payload: {reply}")

ws = "yd4_ingest_ws"
expect_ok(call("system", {
  "type":"yai.control.call.v1",
  "command_id":"yai.workspace.create",
  "target_plane":"runtime",
  "argv":[ws]
}), "workspace.create")
expect_ok(call("system", {
  "type":"yai.control.call.v1",
  "command_id":"yai.workspace.set",
  "target_plane":"runtime",
  "argv":[ws]
}), "workspace.set")

enroll = call(ws, {
  "type":"yai.control.call.v1",
  "command_id":"yai.source.enroll",
  "target_plane":"runtime",
  "workspace_id": ws,
  "source_label":"yd4-node-a",
  "owner_ref":"uds:///tmp/yai-owner.sock"
}, "enroll")
expect_ok(enroll, "source.enroll")
expect_exec_mediation(enroll, "source.enroll")
node_id = enroll.get("data", {}).get("source_node_id")
daemon_id = enroll.get("data", {}).get("daemon_instance_id")
trust_artifact_id = enroll.get("data", {}).get("owner_trust_artifact_id")
trust_artifact_token = enroll.get("data", {}).get("owner_trust_artifact_token")
if not node_id or not daemon_id:
  raise RuntimeError(f"source.enroll missing ids: {enroll}")
if not trust_artifact_id or not trust_artifact_token:
  raise RuntimeError(f"source.enroll missing trust bootstrap artifact: {enroll}")

attach = call(ws, {
  "type":"yai.control.call.v1",
  "command_id":"yai.source.attach",
  "target_plane":"runtime",
  "workspace_id": ws,
  "source_node_id": node_id,
  "owner_trust_artifact_id": trust_artifact_id,
  "owner_trust_artifact_token": trust_artifact_token,
  "binding_scope":"workspace"
}, "attach")
expect_ok(attach, "source.attach")
expect_exec_mediation(attach, "source.attach")
binding_id = attach.get("data", {}).get("source_binding_id")
if not binding_id:
  raise RuntimeError(f"source.attach missing binding id: {attach}")

emit = call(ws, {
  "type":"yai.control.call.v1",
  "command_id":"yai.source.emit",
  "target_plane":"runtime",
  "workspace_id": ws,
  "source_node_id": node_id,
  "source_binding_id": binding_id,
  "owner_trust_artifact_id": trust_artifact_id,
  "owner_trust_artifact_token": trust_artifact_token,
  "idempotency_key":"yd4-emit-001",
  "source_assets":[
    {"type":"yai.source_asset.v1","source_asset_id":"sa-yd4-a","source_binding_id":binding_id,"locator":"file:///tmp/a.txt","asset_type":"file","provenance_fingerprint":"sha256:aaa","observation_state":"observed"}
  ],
  "source_acquisition_events":[
    {"type":"yai.source_acquisition_event.v1","source_acquisition_event_id":"se-yd4-a","source_node_id":node_id,"source_binding_id":binding_id,"source_asset_id":"sa-yd4-a","event_type":"discovered","observed_at_epoch":1773179000,"idempotency_key":"yd4-emit-001","delivery_status":"received"}
  ],
  "source_evidence_candidates":[
    {"type":"yai.source_evidence_candidate.v1","source_evidence_candidate_id":"sc-yd4-a","source_acquisition_event_id":"se-yd4-a","candidate_type":"file_observation","derived_metadata_ref":"meta://yd4/a","owner_resolution_status":"pending"}
  ]
}, "emit")
expect_ok(emit, "source.emit")
expect_exec_mediation(emit, "source.emit")
accepted = emit.get("data", {}).get("accepted", {})
if accepted.get("source_asset", 0) < 1 or accepted.get("source_acquisition_event", 0) < 1 or accepted.get("source_evidence_candidate", 0) < 1:
  raise RuntimeError(f"source.emit accepted counts invalid: {emit}")

status = call(ws, {
  "type":"yai.control.call.v1",
  "command_id":"yai.source.status",
  "target_plane":"runtime",
  "workspace_id": ws,
  "source_node_id": node_id,
  "daemon_instance_id": daemon_id,
  "owner_trust_artifact_id": trust_artifact_id,
  "owner_trust_artifact_token": trust_artifact_token,
  "health":"ready"
}, "status")
expect_ok(status, "source.status")
expect_exec_mediation(status, "source.status")

bad_emit = call(ws, {
  "type":"yai.control.call.v1",
  "command_id":"yai.source.emit",
  "target_plane":"runtime",
  "workspace_id": ws,
  "source_node_id": node_id,
  "source_binding_id": binding_id
}, "emit-bad")
expect_error(bad_emit, "INVALID_STATE", "source_emit_trust_bootstrap_required", "source.emit trust bootstrap guard")

PY

if [[ ! -d "$WS_ROOT/yd4_ingest_ws/data/lmdb" ]]; then
  echo "source_owner_ingest_bridge_v1: FAIL (missing lmdb path for yd4_ingest_ws: $WS_ROOT/yd4_ingest_ws/data/lmdb)"
  exit 1
fi

echo "source_owner_ingest_bridge_v1: ok"
