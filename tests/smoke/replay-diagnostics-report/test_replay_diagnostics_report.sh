#!/bin/sh
# YAI - SPINE.38 replay diagnostics report smoke
#
# Purpose:
#   Verify durable replay/rebuild reports for completed, idempotent and failed
#   journal replay.
#
# Scope:
#   Exercises replay-report, report persistence, idempotent replay and corrupt
#   journal diagnostics through isolated temp journals and YAI_HOME.
#
# Non-goals:
#   Does not rebuild graph, facts, memory, RuntimeGraph or HNSW.
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
YAI_BIN="$ROOT/yai"
YAI_HOME=${YAI_HOME:-"/tmp/yai-spine38-home-$$"}
export YAI_HOME

cleanup() {
  rm -rf "$YAI_HOME" \
    /tmp/yai-spine38-valid.jsonl \
    /tmp/yai-spine38-corrupt.jsonl
}
trap cleanup EXIT INT TERM

require_line() {
  output="$1"
  expected="$2"
  if ! printf '%s\n' "$output" | grep -Fq -- "$expected"; then
    printf 'missing expected line: %s\noutput:\n%s\n' "$expected" "$output" >&2
    exit 1
  fi
}

rm -rf "$YAI_HOME"
mkdir -p "$YAI_HOME"

cat > /tmp/yai-spine38-valid.jsonl <<'JSONL'
{"schema":"yai.store.record.v0","record_id":"rec:spine38-attempt","case_ref":"case:spine38","record_kind":"attempt","subject_ref":"subject:spine38","attempt_id":"op:spine38","decision_id":"","receipt_id":"","summary":"attempt:spine38"}
{"schema":"yai.store.record.v0","record_id":"rec:spine38-receipt","case_ref":"case:spine38","record_kind":"receipt","subject_ref":"subject:spine38","attempt_id":"op:spine38","decision_id":"decision:spine38","receipt_id":"receipt:spine38","summary":"receipt:spine38"}
JSONL

first=$("$YAI_BIN" journal replay --path /tmp/yai-spine38-valid.jsonl)
require_line "$first" "journal_replay: completed"
require_line "$first" "records_written: 2"
require_line "$first" "records_duplicate: 0"
require_line "$first" "cursor_line: 2"
require_line "$first" "compatibility: compatible"
require_line "$first" "replay_report: $YAI_HOME/store/replay/reports/"

report=$("$YAI_BIN" journal replay-report --path /tmp/yai-spine38-valid.jsonl)
require_line "$report" "replay_report_schema: yai.replay_report.v1"
require_line "$report" "journal_identity: journal:"
require_line "$report" "compatibility: compatible"
require_line "$report" "cursor_line: 2"
require_line "$report" "records_written: 2"
require_line "$report" "records_duplicate: 0"
require_line "$report" "invalid_entries: 0"

second=$("$YAI_BIN" journal replay --path /tmp/yai-spine38-valid.jsonl)
require_line "$second" "journal_replay: completed"
require_line "$second" "records_written: 0"
require_line "$second" "records_duplicate: 2"
require_line "$second" "idempotent: yes"

idempotent_report=$("$YAI_BIN" journal replay-report --path /tmp/yai-spine38-valid.jsonl)
require_line "$idempotent_report" "records_written: 0"
require_line "$idempotent_report" "records_duplicate: 2"
require_line "$idempotent_report" "idempotent: yes"

printf '{"schema":"yai.store.record.v0","record_id":"rec:ok","record_kind":"receipt"}\n{broken\n' \
  > /tmp/yai-spine38-corrupt.jsonl
failed=$("$YAI_BIN" journal replay --path /tmp/yai-spine38-corrupt.jsonl)
require_line "$failed" "journal_replay: failed"
require_line "$failed" "invalid_entries: 1"
require_line "$failed" "records_written: 0"
require_line "$failed" "replay_report: $YAI_HOME/store/replay/reports/"

failed_report=$("$YAI_BIN" journal replay-report --path /tmp/yai-spine38-corrupt.jsonl)
require_line "$failed_report" "replay_report_schema: yai.replay_report.v1"
require_line "$failed_report" "replay_status: failed"
require_line "$failed_report" "invalid_entries: 1"
require_line "$failed_report" "records_written: 0"
require_line "$failed_report" "errors:"

printf 'replay_report:write ok\n'
printf 'replay_report:read ok\n'
printf 'replay_report:idempotent ok\n'
printf 'replay_report:failed ok\n'
printf 'replay_report:schema ok\n'
