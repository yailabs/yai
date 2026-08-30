#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
TMP_DIR="${TMPDIR:-/tmp}/yai-spine41-$$"
YAI_HOME="$TMP_DIR/home"
JOURNAL="$TMP_DIR/journal.jsonl"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

mkdir -p "$TMP_DIR" "$YAI_HOME"

cat > "$JOURNAL" <<'JSONL'
{"schema":"yai.store.record.v0","record_id":"rec:spine41-attempt","case_ref":"case:spine41","record_kind":"attempt","subject_ref":"subject:filesystem","attempt_id":"attempt:spine41-write","decision_id":"","receipt_id":"","summary":"attempt targets filesystem subject"}
{"schema":"yai.store.record.v0","record_id":"rec:spine41-decision","case_ref":"case:spine41","record_kind":"decision","subject_ref":"subject:filesystem","attempt_id":"attempt:spine41-write","decision_id":"decision:spine41-allow","receipt_id":"","summary":"decision controls attempt"}
{"schema":"yai.store.record.v0","record_id":"rec:spine41-receipt","case_ref":"case:spine41","record_kind":"effect_receipt","subject_ref":"subject:filesystem","attempt_id":"attempt:spine41-write","decision_id":"decision:spine41-allow","receipt_id":"receipt:spine41-write","summary":"receipt records effect"}
{"schema":"yai.store.record.v0","record_id":"rec:spine41-policy","case_ref":"case:spine41","record_kind":"policy_rule","subject_ref":"subject:filesystem","attempt_id":"","decision_id":"","receipt_id":"","summary":"policy constrains subject"}
{"schema":"yai.store.record.v0","record_id":"rec:spine41-divergence","case_ref":"case:spine41","record_kind":"divergence","subject_ref":"subject:filesystem","attempt_id":"attempt:spine41-write","decision_id":"decision:spine41-allow","receipt_id":"receipt:spine41-write","summary":"divergence describes mismatch"}
JSONL

YAI_HOME="$YAI_HOME" "$YAI_BIN" journal replay --path "$JOURNAL" >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" security bootstrap-local \
  --tenant tenant:spine41 --organization organization:characterization >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" case create \
  --case case:spine41 --tenant tenant:spine41 >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" case create \
  --case case:missing --tenant tenant:spine41 >/dev/null

materialize_first="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph materialize --case case:spine41)"
grep -q "graph_materialize:" <<<"$materialize_first"
grep -q "schema: yai.graph_relation.v1" <<<"$materialize_first"
grep -Eq "relations_written: [1-9][0-9]*" <<<"$materialize_first"
grep -q "runtime_graph_updated: false" <<<"$materialize_first"
printf "graph_relation:materialize ok\n"

materialize_second="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph materialize --case case:spine41)"
grep -q "relations_written: 0" <<<"$materialize_second"
grep -Eq "relations_duplicate: [1-9][0-9]*" <<<"$materialize_second"
printf "graph_relation:idempotent ok\n"

relations="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph relations --case case:spine41 --limit 30)"
grep -q "graph_relations:" <<<"$relations"
grep -Eq "relations_total: [1-9][0-9]*" <<<"$relations"
printf "graph_relation:list ok\n"

grep -q "edge_kind: decision_controls_attempt" <<<"$relations"
printf "graph_relation:decision_controls_attempt ok\n"

grep -q "edge_kind: receipt_records_effect" <<<"$relations"
printf "graph_relation:receipt_records_effect ok\n"

missing="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph relations --case case:missing --limit 20)"
grep -q "relations_total: 0" <<<"$missing"
grep -q "relations: none" <<<"$missing"

runtime_status="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph runtime-status)"
grep -q "status: active_minimal" <<<"$runtime_status"
grep -q "relation_write_path: active_minimal" <<<"$runtime_status"
printf "runtime_graph:not_updated confirmed\n"
