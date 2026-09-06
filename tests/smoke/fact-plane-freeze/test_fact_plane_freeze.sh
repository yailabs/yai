#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/yai"
YAID="$ROOT/build/yaid"
TMP_DIR="${TMPDIR:-/tmp}/yai-spine51-$$"
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

status="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts status)"
require_line "$status" "status: not_initialized"
require_line "$status" "schema: yai.fact.v1"
require_line "$status" "facts_are_truth: false"
printf "fact_freeze:status ok\n"

schema="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts schema)"
for line in \
  "fact_schema: yai.fact.v1" \
  "bitemporal: yes" \
  "fact_receipt" \
  "fact_decision" \
  "fact_projection" \
  "fact_carrier_outcome" \
  "fact_divergence" \
  "fact_model_behavior" \
  "fact_policy_outcome" \
  "fact_memory_quality" \
  "transaction_time" \
  "valid_time_start" \
  "valid_time_end" \
  "known_at" \
  "revision_of" \
  "superseded_by" \
  "retracted_by" \
  "facts_are_truth: false"; do
  require_line "$schema" "$line"
done
printf "fact_freeze:schema ok\n"

init="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts init)"
require_line "$init" "facts_init:"
require_line "$init" "status: ready"
require_line "$init" "schema: yai.fact.v1"
printf "fact_freeze:init ok\n"

core="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts extract --case case:new12-filesystem --kind core)"
require_line "$core" "kind: core"
require_line "$core" "fact_receipt_written: 3"
require_line "$core" "fact_decision_written: 2"
require_line "$core" "fact_projection_written: 3"
printf "fact_freeze:extract_core ok\n"

behavior="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts extract --case case:new12-filesystem --kind behavior)"
require_line "$behavior" "kind: behavior"
require_line "$behavior" "fact_model_behavior_written: 0"
require_line "$behavior" "fact_policy_outcome_written: 7"
printf "fact_freeze:extract_behavior ok\n"

operational="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts extract --case case:new12-filesystem --kind operational)"
require_line "$operational" "kind: operational"
require_line "$operational" "fact_carrier_outcome_written: 3"
require_line "$operational" "fact_divergence_written: 0"
require_line "$operational" "fact_memory_quality_written: 1"
printf "fact_freeze:extract_operational ok\n"

all_first="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts extract --case case:new12-filesystem --kind all)"
require_line "$all_first" "kind: all"
for line in \
  "fact_receipt_written: 0" \
  "fact_decision_written: 0" \
  "fact_projection_written: 0" \
  "fact_model_behavior_written: 0" \
  "fact_policy_outcome_written: 0" \
  "fact_carrier_outcome_written: 0" \
  "fact_divergence_written: 0" \
  "fact_memory_quality_written: 0"; do
  require_line "$all_first" "$line"
done
require_regex "$all_first" "facts_duplicate: 1[0-9]"
printf "fact_freeze:extract_all ok\n"

all_second="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts extract --case case:new12-filesystem --kind all)"
for line in \
  "fact_receipt_written: 0" \
  "fact_decision_written: 0" \
  "fact_projection_written: 0" \
  "fact_model_behavior_written: 0" \
  "fact_policy_outcome_written: 0" \
  "fact_carrier_outcome_written: 0" \
  "fact_divergence_written: 0" \
  "fact_memory_quality_written: 0"; do
  require_line "$all_second" "$line"
done
require_regex "$all_second" "facts_duplicate: 1[0-9]"
printf "fact_freeze:idempotent ok\n"

summary="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts summary --case case:new12-filesystem)"
for line in \
  "facts_summary:" \
  "case_ref: case:new12-filesystem" \
  "fact_receipt: 3" \
  "fact_decision: 2" \
  "fact_projection: 3" \
  "fact_carrier_outcome: 3" \
  "fact_divergence: 0" \
  "fact_model_behavior: 0" \
  "fact_policy_outcome: 7" \
  "fact_memory_quality: 1" \
  "facts_total: 19" \
  "facts_are_truth: false" \
  "memory_is_truth: false"; do
  require_line "$summary" "$line"
done
printf "fact_freeze:summary ok\n"

report="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts report --case case:new12-filesystem)"
for line in \
  "facts_report:" \
  "facts_are_truth: false" \
  "  receipts:" \
  "  decisions:" \
  "  projections:" \
  "  policy:" \
  "  carriers:" \
  "  divergence:" \
  "    status: none_observed" \
  "  memory:" \
  "    memory_is_truth: false" \
  "  model:" \
  "    status: no_model_records"; do
  require_line "$report" "$line"
done
printf "fact_freeze:report ok\n"

for section in receipts decisions projections policy carriers divergence memory model; do
  section_report="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts report --case case:new12-filesystem --section "$section")"
  require_line "$section_report" "facts_report:"
  require_line "$section_report" "  $section:"
done

divergence="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts report --case case:new12-filesystem --section divergence)"
require_line "$divergence" "    total: 0"
require_line "$divergence" "    status: none_observed"
printf "fact_freeze:divergence none_observed ok\n"

model="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts report --case case:new12-filesystem --section model)"
require_line "$model" "    total: 0"
require_line "$model" "    status: no_model_records"
printf "fact_freeze:model no_model_records ok\n"

require_line "$summary" "facts_are_truth: false"
printf "fact_freeze:not_truth ok\n"

memory="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts report --case case:new12-filesystem --section memory)"
require_line "$memory" "    memory_is_truth: false"
printf "fact_freeze:memory_not_truth ok\n"
