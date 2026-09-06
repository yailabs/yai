#!/bin/sh
# YAI - SPINE.39 journal replay freeze smoke
#
# Purpose:
#   Validate the full frozen journal replay lifecycle from inspect through
#   durable replay report without adding new replay semantics.
#
# Scope:
#   Exercises inspect, replay dry-run, real replay, replay-status,
#   replay-report, idempotency, schema mismatch, corrupt input and no journal
#   fallback for LMDB store queries.
#
# Non-goals:
#   Does not rebuild graph, facts, memory, RuntimeGraph or HNSW.
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
YAI_BIN="$ROOT/yai"
YAI_HOME=${YAI_HOME:-"/tmp/yai-spine39-home-$$"}
export YAI_HOME

cleanup() {
  rm -rf "$YAI_HOME" \
    /tmp/yai-spine39-valid.jsonl \
    /tmp/yai-spine39-not-replayed.jsonl \
    /tmp/yai-spine39-schema-mismatch.jsonl \
    /tmp/yai-spine39-corrupt.jsonl
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

cat > /tmp/yai-spine39-valid.jsonl <<'JSONL'
{"schema":"yai.store.record.v0","record_id":"rec:spine39-attempt","case_ref":"case:spine39","record_kind":"attempt","subject_ref":"subject:spine39","attempt_id":"op:spine39","decision_id":"","receipt_id":"","summary":"attempt:spine39"}
{"schema":"yai.store.record.v0","record_id":"rec:spine39-receipt","case_ref":"case:spine39","record_kind":"receipt","subject_ref":"subject:spine39","attempt_id":"op:spine39","decision_id":"decision:spine39","receipt_id":"receipt:spine39","summary":"receipt:spine39"}
JSONL

inspect=$("$YAI_BIN" journal inspect --path /tmp/yai-spine39-valid.jsonl)
require_line "$inspect" "replay_ready: yes"
require_line "$inspect" "invalid_entries: 0"

dry_run=$("$YAI_BIN" journal replay --path /tmp/yai-spine39-valid.jsonl --dry-run)
require_line "$dry_run" "journal_replay: dry_run"
require_line "$dry_run" "journal_identity: journal:"
require_line "$dry_run" "record_schema: yai.record.v1"
require_line "$dry_run" "journal_schema: yai.store.record.v0"
require_line "$dry_run" "compatibility: compatible"
require_line "$dry_run" "would_write_lmdb: yes"

first=$("$YAI_BIN" journal replay --path /tmp/yai-spine39-valid.jsonl)
require_line "$first" "journal_replay: completed"
require_line "$first" "records_written: 2"
require_line "$first" "records_duplicate: 0"
require_line "$first" "cursor_line: 2"
require_line "$first" "replay_status: completed"
require_line "$first" "replay_report: $YAI_HOME/store/replay/reports/"

summary=$("$YAI_BIN" store summary)
require_line "$summary" "record_store_status: ready"
require_line "$summary" "records_total: 2"

case_list=$("$YAI_BIN" store record list --case case:spine39 --limit 10)
require_line "$case_list" "records_total: 2"
require_line "$case_list" "record_id: rec:spine39-attempt"
require_line "$case_list" "record_id: rec:spine39-receipt"

status=$("$YAI_BIN" journal replay-status --path /tmp/yai-spine39-valid.jsonl)
require_line "$status" "journal_replay_status:"
require_line "$status" "cursor_line: 2"
require_line "$status" "replay_status: completed"
require_line "$status" "compatibility: compatible"

report=$("$YAI_BIN" journal replay-report --path /tmp/yai-spine39-valid.jsonl)
require_line "$report" "replay_report_schema: yai.replay_report.v1"
require_line "$report" "journal_identity: journal:"
require_line "$report" "cursor_line: 2"
require_line "$report" "records_written: 2"
require_line "$report" "invalid_entries: 0"

second=$("$YAI_BIN" journal replay --path /tmp/yai-spine39-valid.jsonl)
require_line "$second" "journal_replay: completed"
require_line "$second" "records_written: 0"
require_line "$second" "records_duplicate: 2"
require_line "$second" "idempotent: yes"

idempotent_report=$("$YAI_BIN" journal replay-report --path /tmp/yai-spine39-valid.jsonl)
require_line "$idempotent_report" "records_written: 0"
require_line "$idempotent_report" "records_duplicate: 2"
require_line "$idempotent_report" "idempotent: yes"

cat > /tmp/yai-spine39-not-replayed.jsonl <<'JSONL'
{"schema":"yai.store.record.v0","record_id":"rec:spine39-not-replayed","case_ref":"case:spine39-not-replayed","record_kind":"receipt","subject_ref":"subject:spine39","attempt_id":"op:spine39","decision_id":"decision:spine39","receipt_id":"receipt:spine39-not-replayed","summary":"not replayed"}
JSONL
not_replayed=$("$YAI_BIN" store record list --case case:spine39-not-replayed --limit 10)
require_line "$not_replayed" "records_total: 0"

cat > /tmp/yai-spine39-schema-mismatch.jsonl <<'JSONL'
{"schema":"yai.record.v2","record_id":"rec:spine39-mismatch","record_kind":"test","case_ref":"case:spine39","payload":{}}
JSONL
mismatch=$("$YAI_BIN" journal replay --path /tmp/yai-spine39-schema-mismatch.jsonl)
require_line "$mismatch" "journal_replay: failed"
require_line "$mismatch" "compatibility: schema_mismatch"
require_line "$mismatch" "records_written: 0"
require_line "$mismatch" "replay_status: incompatible"

mismatch_report=$("$YAI_BIN" journal replay-report --path /tmp/yai-spine39-schema-mismatch.jsonl)
require_line "$mismatch_report" "replay_report_schema: yai.replay_report.v1"
require_line "$mismatch_report" "compatibility: schema_mismatch"
require_line "$mismatch_report" "records_written: 0"

printf '{"schema":"yai.store.record.v0","record_id":"rec:spine39-ok","record_kind":"receipt"}\n{broken\n' \
  > /tmp/yai-spine39-corrupt.jsonl
corrupt=$("$YAI_BIN" journal replay --path /tmp/yai-spine39-corrupt.jsonl)
require_line "$corrupt" "journal_replay: failed"
require_line "$corrupt" "invalid_entries: 1"
require_line "$corrupt" "records_written: 0"
require_line "$corrupt" "replay_status: failed"

corrupt_report=$("$YAI_BIN" journal replay-report --path /tmp/yai-spine39-corrupt.jsonl)
require_line "$corrupt_report" "replay_report_schema: yai.replay_report.v1"
require_line "$corrupt_report" "replay_status: failed"
require_line "$corrupt_report" "invalid_entries: 1"
require_line "$corrupt_report" "records_written: 0"

printf 'replay_freeze:inspect ok\n'
printf 'replay_freeze:dry_run ok\n'
printf 'replay_freeze:replay ok\n'
printf 'replay_freeze:status ok\n'
printf 'replay_freeze:report ok\n'
printf 'replay_freeze:idempotent ok\n'
printf 'replay_freeze:schema_mismatch ok\n'
printf 'replay_freeze:corrupt_failed_report ok\n'
printf 'replay_freeze:no_journal_fallback ok\n'
