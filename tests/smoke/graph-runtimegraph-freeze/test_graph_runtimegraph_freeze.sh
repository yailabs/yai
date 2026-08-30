#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
YAID="$ROOT/build/yaid"
TMP_DIR="build/tmp/yai-spine45-$$"
YAI_HOME="$TMP_DIR/home"
SOCKET="$TMP_DIR/run/yaid.sock"
DAEMON_PID=""

cleanup() {
  if [[ -n "$DAEMON_PID" ]]; then
    YAI_HOME="$YAI_HOME" "$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null 2>&1 || true
    wait "$DAEMON_PID" >/dev/null 2>&1 || true
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

require_line() {
  local haystack="$1"
  local needle="$2"
  grep -Fq -- "$needle" <<<"$haystack"
}

require_regex() {
  local haystack="$1"
  local pattern="$2"
  grep -Eq -- "$pattern" <<<"$haystack"
}

mkdir -p "$TMP_DIR/run" "$YAI_HOME"
YAI_HOME="$YAI_HOME" "$YAID" --socket "$SOCKET" --foreground &
DAEMON_PID=$!

for _ in $(seq 1 50); do
  [[ -S "$SOCKET" ]] && break
  sleep 0.1
done
[[ -S "$SOCKET" ]]

filesystem_output="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" daemon run-filesystem-loop --socket "$SOCKET")"
journal_path="$(sed -n 's/.*"journal_path":"\([^"]*\)".*/\1/p' <<<"$filesystem_output")"
if [[ -z "$journal_path" ]]; then
  printf '%s\n' "$filesystem_output" >&2
  exit 1
fi

YAI_HOME="$YAI_HOME" "$YAI_BIN" journal replay --path "$journal_path" >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" security bootstrap-local \
  --tenant tenant:graph-freeze --organization organization:characterization >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" case create \
  --case case:new12-filesystem --tenant tenant:graph-freeze >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" case create \
  --case case:missing --tenant tenant:graph-freeze >/dev/null

schema="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph schema)"
require_line "$schema" "graph_schema:"
require_line "$schema" "review_request"
require_line "$schema" "review_decision"
require_line "$schema" "control_pending"
require_line "$schema" "review_request_for_attempt"
require_line "$schema" "review_decision_resolves_request"
require_line "$schema" "control_pending_blocks_attempt"
require_line "$schema" "review_resolution_produces_receipt"
printf "graph_freeze:schema ok\n"

runtime_status="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph runtime-status)"
require_line "$runtime_status" "working_set: per_command_ephemeral"
require_line "$runtime_status" "durable_truth: canonical_transition_or_legacy_compatibility_input"
require_line "$runtime_status" "hnsw: future_candidate_index"
require_line "$runtime_status" "context_compiler: future_consumer"

materialize="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph materialize --case case:new12-filesystem)"
require_line "$materialize" "graph_materialize:"
require_line "$materialize" "schema: yai.graph_relation.v1"
require_regex "$materialize" "relations_written: [1-9][0-9]*"
printf "graph_freeze:materialize ok\n"

relations="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph relations --case case:new12-filesystem --limit 200)"
require_line "$relations" "graph_relations:"
require_line "$relations" "edge_kind: decision_controls_attempt"
require_line "$relations" "edge_kind: receipt_records_effect"
printf "graph_freeze:relations ok\n"

decision_ref="$(awk '/edge_kind: decision_controls_attempt/{getline; sub(/^  from_ref: /, ""); print; exit}' <<<"$relations")"
attempt_ref="$(awk '/edge_kind: decision_controls_attempt/{getline; getline; sub(/^  to_ref: /, ""); print; exit}' <<<"$relations")"
if [[ -z "$decision_ref" || -z "$attempt_ref" ]]; then
  printf '%s\n' "$relations" >&2
  exit 1
fi

runtime_load="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph runtime-load --case case:new12-filesystem)"
require_line "$runtime_load" "runtime_graph_load:"
require_regex "$runtime_load" "nodes_total: [1-9][0-9]*"
require_regex "$runtime_load" "edges_total: [1-9][0-9]*"
summary="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph runtime-summary --case case:new12-filesystem)"
require_line "$summary" "runtime_graph_summary:"
require_regex "$summary" "outgoing_index_entries: [1-9][0-9]*"
printf "graph_freeze:runtime_load ok\n"

rebuild="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph rebuild --case case:new12-filesystem --from journal --path "$journal_path")"
require_line "$rebuild" "runtime_graph_rebuild:"
require_line "$rebuild" "source_mode: journal"
require_line "$rebuild" "journal_replay_status: completed"
require_line "$rebuild" "graph_materialize_status: completed"
require_line "$rebuild" "runtime_graph_status: completed"
require_line "$rebuild" "rebuild_status: completed"
from_relations="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph rebuild --case case:new12-filesystem --from graph-relations)"
require_line "$from_relations" "source_mode: graph_relations"
require_line "$from_relations" "rebuild_status: completed"
printf "graph_freeze:runtime_rebuild ok\n"

report="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph rebuild-report --case case:new12-filesystem)"
require_line "$report" "runtime_graph_rebuild_report:"
require_line "$report" "report_schema: yai.runtime_graph_rebuild_report.v1"
require_line "$report" "rebuild_status: completed"
printf "graph_freeze:rebuild_report ok\n"

fanout="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph fanout --case case:new12-filesystem --node "$decision_ref" --limit 10)"
require_line "$fanout" "graph_fanout:"
require_line "$fanout" "edge_kind: decision_controls_attempt"
printf "graph_freeze:fanout ok\n"

fanin="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph fanin --case case:new12-filesystem --node "$attempt_ref" --limit 10)"
require_line "$fanin" "graph_fanin:"
require_line "$fanin" "edge_kind: decision_controls_attempt"
printf "graph_freeze:fanin ok\n"

neighborhood="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph neighborhood --case case:new12-filesystem --node "$decision_ref" --depth 1 --limit 20)"
require_line "$neighborhood" "graph_neighborhood:"
require_regex "$neighborhood" "nodes_total: [1-9][0-9]*"
require_regex "$neighborhood" "edges_total: [1-9][0-9]*"
printf "graph_freeze:neighborhood ok\n"

path_found="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph path --case case:new12-filesystem --from "$decision_ref" --to "$attempt_ref" --max-depth 4)"
require_line "$path_found" "path_status: found"
printf "graph_freeze:path_found ok\n"

path_not_found="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph path --case case:new12-filesystem --from "$decision_ref" --to subject:missing --max-depth 4)"
require_line "$path_not_found" "path_status: not_found"
printf "graph_freeze:path_not_found ok\n"

empty_case="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph runtime-load --case case:missing)"
require_line "$empty_case" "nodes_total: 0"
require_line "$empty_case" "edges_total: 0"
printf "graph_freeze:empty_case ok\n"

bounded="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph fanout --case case:new12-filesystem --node "$decision_ref" --limit 999)"
require_line "$bounded" "limit: 200"
require_line "$bounded" "limit_clamped: yes"
bounded_path="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph path --case case:new12-filesystem --from "$decision_ref" --to "$attempt_ref" --max-depth 99)"
require_line "$bounded_path" "max_depth: 6"
require_line "$bounded_path" "max_depth_clamped: yes"
printf "graph_freeze:bounded ok\n"
