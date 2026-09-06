#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/yai"
YAID="$ROOT/build/yaid"
TMP_DIR="build/tmp/yai-spine49-$$"
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
YAI_HOME="$YAI_HOME" "$YAI_BIN" facts init >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" facts extract --case case:new12-filesystem --kind core >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" facts extract --case case:new12-filesystem --kind behavior >/dev/null

carrier="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts extract --case case:new12-filesystem --kind carrier_outcome)"
require_line "$carrier" "facts_extract:"
require_line "$carrier" "kind: carrier_outcome"
require_line "$carrier" "table: fact_carrier_outcome"
require_regex "$carrier" "facts_written: [1-9][0-9]*"
printf "facts_extract:carrier_outcome ok\n"

divergence="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts extract --case case:new12-filesystem --kind divergence)"
require_line "$divergence" "kind: divergence"
require_line "$divergence" "table: fact_divergence"
require_regex "$divergence" "facts_written: [0-9]+"
printf "facts_extract:divergence ok\n"

memory="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts extract --case case:new12-filesystem --kind memory_quality)"
require_line "$memory" "kind: memory_quality"
require_line "$memory" "table: fact_memory_quality"
require_regex "$memory" "facts_written: [1-9][0-9]*"
require_line "$memory" "memory_is_truth: false"
printf "facts_extract:memory_quality ok\n"

operational="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts extract --case case:new12-filesystem --kind operational)"
require_line "$operational" "kind: operational"
require_line "$operational" "fact_carrier_outcome_written: 0"
require_line "$operational" "fact_divergence_written: 0"
require_line "$operational" "fact_memory_quality_written: 0"
require_regex "$operational" "facts_duplicate: [1-9][0-9]*"
printf "facts_extract:operational ok\n"
printf "facts_extract:idempotent ok\n"

summary="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts summary --case case:new12-filesystem)"
require_line "$summary" "facts_summary:"
require_regex "$summary" "fact_carrier_outcome: [1-9][0-9]*"
require_regex "$summary" "fact_divergence: [0-9]+"
require_regex "$summary" "fact_memory_quality: [1-9][0-9]*"
require_regex "$summary" "facts_total: [1-9][0-9]*"
printf "facts_summary:operational_counts ok\n"

status="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts status)"
require_line "$status" "facts_are_truth: false"
require_regex "$status" "fact_carrier_outcome: [1-9][0-9]*"
require_regex "$status" "fact_memory_quality: [1-9][0-9]*"
printf "facts:not_truth ok\n"
printf "memory:not_truth ok\n"
