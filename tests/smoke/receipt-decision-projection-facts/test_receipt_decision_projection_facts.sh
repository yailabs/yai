#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/yai"
YAID="$ROOT/build/yaid"
TMP_DIR="build/tmp/yai-spine47-$$"
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

receipt="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts extract --case case:new12-filesystem --kind receipt)"
require_line "$receipt" "facts_extract:"
require_line "$receipt" "kind: receipt"
require_line "$receipt" "table: fact_receipt"
require_regex "$receipt" "facts_written: [1-9][0-9]*"
printf "facts_extract:receipt ok\n"

decision="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts extract --case case:new12-filesystem --kind decision)"
require_line "$decision" "kind: decision"
require_line "$decision" "table: fact_decision"
require_regex "$decision" "facts_written: [1-9][0-9]*"
printf "facts_extract:decision ok\n"

projection="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts extract --case case:new12-filesystem --kind projection)"
require_line "$projection" "kind: projection"
require_line "$projection" "table: fact_projection"
require_regex "$projection" "facts_written: [1-9][0-9]*"
printf "facts_extract:projection ok\n"

core="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts extract --case case:new12-filesystem --kind core)"
require_line "$core" "kind: core"
require_line "$core" "fact_receipt_written: 0"
require_line "$core" "fact_decision_written: 0"
require_line "$core" "fact_projection_written: 0"
require_regex "$core" "facts_duplicate: [1-9][0-9]*"
printf "facts_extract:core ok\n"
printf "facts_extract:idempotent ok\n"

summary="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts summary --case case:new12-filesystem)"
require_line "$summary" "facts_summary:"
require_regex "$summary" "fact_receipt: [1-9][0-9]*"
require_regex "$summary" "fact_decision: [1-9][0-9]*"
require_regex "$summary" "fact_projection: [1-9][0-9]*"
require_regex "$summary" "facts_total: [1-9][0-9]*"
printf "facts_summary:counts ok\n"

status="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts status)"
require_line "$status" "facts_are_truth: false"
require_regex "$status" "facts_extracted: [1-9][0-9]*"
require_regex "$status" "fact_receipt: [1-9][0-9]*"
require_regex "$status" "fact_decision: [1-9][0-9]*"
require_regex "$status" "fact_projection: [1-9][0-9]*"
printf "facts:not_truth ok\n"
