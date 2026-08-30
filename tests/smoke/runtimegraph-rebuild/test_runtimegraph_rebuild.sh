#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
TMP_DIR="${TMPDIR:-/tmp}/yai-spine43-$$"
YAI_HOME="$TMP_DIR/home"
JOURNAL="$TMP_DIR/journal.jsonl"
BAD_JOURNAL="$TMP_DIR/bad-journal.jsonl"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

mkdir -p "$TMP_DIR" "$YAI_HOME"

cat > "$JOURNAL" <<'JSONL'
{"schema":"yai.store.record.v0","record_id":"rec:spine43-attempt","case_ref":"case:spine43","record_kind":"attempt","subject_ref":"subject:filesystem","attempt_id":"attempt:spine43-write","decision_id":"","receipt_id":"","summary":"attempt targets filesystem subject"}
{"schema":"yai.store.record.v0","record_id":"rec:spine43-decision","case_ref":"case:spine43","record_kind":"decision","subject_ref":"subject:filesystem","attempt_id":"attempt:spine43-write","decision_id":"decision:spine43-allow","receipt_id":"","summary":"decision controls attempt"}
{"schema":"yai.store.record.v0","record_id":"rec:spine43-receipt","case_ref":"case:spine43","record_kind":"effect_receipt","subject_ref":"subject:filesystem","attempt_id":"attempt:spine43-write","decision_id":"decision:spine43-allow","receipt_id":"receipt:spine43-write","summary":"receipt records effect"}
{"schema":"yai.store.record.v0","record_id":"rec:spine43-policy","case_ref":"case:spine43","record_kind":"policy_rule","subject_ref":"subject:filesystem","attempt_id":"","decision_id":"","receipt_id":"","summary":"policy constrains subject"}
JSONL

YAI_HOME="$YAI_HOME" "$YAI_BIN" security bootstrap-local \
  --tenant tenant:spine43 --organization organization:characterization >/dev/null
for case_id in case:spine43 case:missing case:bad; do
  YAI_HOME="$YAI_HOME" "$YAI_BIN" case create \
    --case "$case_id" --tenant tenant:spine43 >/dev/null
done

journal_rebuild="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph rebuild --case case:spine43 --from journal --path "$JOURNAL")"
grep -q "runtime_graph_rebuild:" <<<"$journal_rebuild"
grep -q "source_mode: journal" <<<"$journal_rebuild"
grep -q "journal_replay_status: completed" <<<"$journal_rebuild"
grep -q "graph_materialize_status: completed" <<<"$journal_rebuild"
grep -q "runtime_graph_status: completed" <<<"$journal_rebuild"
grep -q "rebuild_status: completed" <<<"$journal_rebuild"
grep -q "report_schema: yai.runtime_graph_rebuild_report.v1" <<<"$journal_rebuild"
grep -Eq "nodes_total: [1-9][0-9]*" <<<"$journal_rebuild"
grep -Eq "edges_total: [1-9][0-9]*" <<<"$journal_rebuild"
printf "runtime_graph_rebuild:journal ok\n"

report="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph rebuild-report --case case:spine43)"
grep -q "runtime_graph_rebuild_report:" <<<"$report"
grep -q "report_schema: yai.runtime_graph_rebuild_report.v1" <<<"$report"
grep -q "rebuild_status: completed" <<<"$report"
printf "runtime_graph_rebuild:report ok\n"

summary="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph runtime-summary --case case:spine43)"
grep -q "runtime_graph_summary:" <<<"$summary"
grep -Eq "nodes_total: [1-9][0-9]*" <<<"$summary"
grep -Eq "edges_total: [1-9][0-9]*" <<<"$summary"

relations_rebuild="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph rebuild --case case:spine43 --from graph-relations)"
grep -q "source_mode: graph_relations" <<<"$relations_rebuild"
grep -q "rebuild_status: completed" <<<"$relations_rebuild"
grep -Eq "nodes_total: [1-9][0-9]*" <<<"$relations_rebuild"
printf "runtime_graph_rebuild:graph_relations ok\n"

second_rebuild="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph rebuild --case case:spine43 --from journal --path "$JOURNAL")"
grep -Eq "records_duplicate: [1-9][0-9]*" <<<"$second_rebuild"
grep -Eq "relations_written: [1-9][0-9]*" <<<"$second_rebuild"
grep -q "relations_duplicate: 0" <<<"$second_rebuild"
second_summary="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph runtime-summary --case case:spine43)"
[[ "$second_summary" == "$summary" ]]
printf "runtime_graph_rebuild:idempotent ok\n"

missing_case="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph rebuild --case case:missing --from graph-relations)"
grep -q "nodes_total: 0" <<<"$missing_case"
grep -q "edges_total: 0" <<<"$missing_case"
grep -q "warnings:" <<<"$missing_case"
grep -q "no_graph_relations_for_case" <<<"$missing_case"
printf "runtime_graph_rebuild:missing_case ok\n"

printf '{"schema":"yai.store.record.v0","record_id":"rec:bad","case_ref":"case:bad","record_kind":"attempt","subject_ref":"subject:bad"}\n{broken\n' > "$BAD_JOURNAL"
bad_rebuild="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph rebuild --case case:bad --from journal --path "$BAD_JOURNAL")"
grep -q "journal_replay_status: failed" <<<"$bad_rebuild"
grep -q "rebuild_status: failed" <<<"$bad_rebuild"
grep -q "reason: invalid_json" <<<"$bad_rebuild"
grep -q "nodes_total: 0" <<<"$bad_rebuild"
grep -q "edges_total: 0" <<<"$bad_rebuild"
printf "runtime_graph_rebuild:bad_journal failed ok\n"

status="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph runtime-status)"
grep -q "resident_service: planned" <<<"$status"
printf "runtime_graph:resident_service planned\n"
