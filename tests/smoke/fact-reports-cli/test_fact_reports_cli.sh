#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
YAID="$ROOT/build/yaid"
TMP_DIR="build/tmp/yai-spine50-$$"
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
YAI_HOME="$YAI_HOME" "$YAI_BIN" facts extract --case case:new12-filesystem --kind operational >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" facts summary --case case:new12-filesystem >/dev/null

report="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts report --case case:new12-filesystem)"
require_line "$report" "facts_report:"
require_line "$report" "schema: yai.fact.v1"
require_line "$report" "facts_are_truth: false"
require_line "$report" "  receipts:"
require_line "$report" "  decisions:"
require_line "$report" "  projections:"
require_line "$report" "  policy:"
require_line "$report" "  carriers:"
require_line "$report" "  divergence:"
require_line "$report" "  memory:"
require_line "$report" "  model:"
printf "facts_report:default ok\n"

receipts="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts report --case case:new12-filesystem --section receipts)"
require_line "$receipts" "  receipts:"
require_line "$receipts" "    total: 3"
require_line "$receipts" "    blocked: 1"
require_line "$receipts" "    not_executed: 1"
require_line "$receipts" "    observed: 1"
require_line "$receipts" "    execution_performed: 1"
require_line "$receipts" "    carrier_attempted: 1"
printf "facts_report:receipts ok\n"

decisions="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts report --case case:new12-filesystem --section decisions)"
require_line "$decisions" "  decisions:"
require_line "$decisions" "    total: 2"
require_line "$decisions" "    require_review: 1"
require_line "$decisions" "    unknown: 1"
printf "facts_report:decisions ok\n"

policy="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts report --case case:new12-filesystem --section policy)"
require_line "$policy" "  policy:"
require_line "$policy" "    total: 7"
require_regex "$policy" "    review_required: [1-9][0-9]*"
printf "facts_report:policy ok\n"

carriers="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts report --case case:new12-filesystem --section carriers)"
require_line "$carriers" "  carriers:"
require_line "$carriers" "    total: 3"
require_line "$carriers" "    filesystem: 3"
require_regex "$carriers" "    execution_performed: [1-9][0-9]*"
require_regex "$carriers" "    carrier_attempted: [1-9][0-9]*"
printf "facts_report:carriers ok\n"

divergence="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts report --case case:new12-filesystem --section divergence)"
require_line "$divergence" "  divergence:"
require_line "$divergence" "    total: 0"
require_line "$divergence" "    status: none_observed"
printf "facts_report:divergence none_observed ok\n"

memory="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts report --case case:new12-filesystem --section memory)"
require_line "$memory" "  memory:"
require_line "$memory" "    total: 1"
require_line "$memory" "    memory_is_truth: false"
require_line "$memory" "    candidates: 1"
printf "facts_report:memory not_truth ok\n"

model="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts report --case case:new12-filesystem --section model)"
require_line "$model" "  model:"
require_line "$model" "    total: 0"
require_line "$model" "    status: no_model_records"
printf "facts_report:model no_model_records ok\n"
printf "facts_report:not_truth ok\n"
