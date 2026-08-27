#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
TMP_DIR="${TMPDIR:-/tmp}/yai-spine42-$$"
YAI_HOME="$TMP_DIR/home"
JOURNAL="$TMP_DIR/journal.jsonl"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

mkdir -p "$TMP_DIR" "$YAI_HOME"

cat > "$JOURNAL" <<'JSONL'
{"schema":"yai.store.record.v0","record_id":"rec:spine42-attempt","case_ref":"case:spine42","record_kind":"attempt","subject_ref":"subject:filesystem","attempt_id":"attempt:spine42-write","decision_id":"","receipt_id":"","summary":"attempt targets filesystem subject"}
{"schema":"yai.store.record.v0","record_id":"rec:spine42-decision","case_ref":"case:spine42","record_kind":"decision","subject_ref":"subject:filesystem","attempt_id":"attempt:spine42-write","decision_id":"decision:spine42-allow","receipt_id":"","summary":"decision controls attempt"}
{"schema":"yai.store.record.v0","record_id":"rec:spine42-receipt","case_ref":"case:spine42","record_kind":"effect_receipt","subject_ref":"subject:filesystem","attempt_id":"attempt:spine42-write","decision_id":"decision:spine42-allow","receipt_id":"receipt:spine42-write","summary":"receipt records effect"}
{"schema":"yai.store.record.v0","record_id":"rec:spine42-policy","case_ref":"case:spine42","record_kind":"policy_rule","subject_ref":"subject:filesystem","attempt_id":"","decision_id":"","receipt_id":"","summary":"policy constrains subject"}
JSONL

YAI_HOME="$YAI_HOME" "$YAI_BIN" journal replay --path "$JOURNAL" >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" graph materialize --case case:spine42 >/dev/null

status="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph runtime-status)"
grep -q "status: active_minimal" <<<"$status"
grep -q "working_set: per_command_ephemeral" <<<"$status"
grep -q "resident_service: planned" <<<"$status"
printf "runtime_graph:status active_minimal\n"

load="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph runtime-load --case case:spine42)"
grep -q "runtime_graph_load:" <<<"$load"
grep -q "source: graph_relations" <<<"$load"
printf "runtime_graph:load ok\n"

grep -Eq "nodes_total: [1-9][0-9]*" <<<"$load"
printf "runtime_graph:nodes nonzero\n"

grep -Eq "edges_total: [1-9][0-9]*" <<<"$load"
printf "runtime_graph:edges nonzero\n"

grep -Eq "outgoing_index_entries: [1-9][0-9]*" <<<"$load"
grep -Eq "incoming_index_entries: [1-9][0-9]*" <<<"$load"
printf "runtime_graph:adjacency ok\n"

empty="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph runtime-load --case case:missing)"
grep -q "nodes_total: 0" <<<"$empty"
grep -q "edges_total: 0" <<<"$empty"
grep -q "dirty: no" <<<"$empty"
grep -q "stale: no" <<<"$empty"
printf "runtime_graph:empty ok\n"

grep -q "durable_truth: canonical_transition_or_legacy_compatibility_input" <<<"$load"
grep -q "hnsw: future_candidate_index" <<<"$load"
grep -q "context_compiler: future_consumer" <<<"$load"
grep -q "resident: false" <<<"$load"
printf "runtime_graph:no_hnsw confirmed\n"
