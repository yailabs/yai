#!/bin/sh
# YAI - SPINE.37 replay idempotency and schema version smoke
#
# Purpose:
#   Verify journal replay metadata, cursor status, schema compatibility and
#   idempotent replay posture.
#
# Scope:
#   Exercises replay, replay-status, schema mismatch and missing-schema
#   diagnostics through isolated temp journals and YAI_HOME.
#
# Non-goals:
#   Does not rebuild graph, facts, memory, RuntimeGraph or HNSW.
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
YAI_BIN="$ROOT/yai"
YAI_HOME=${YAI_HOME:-"/tmp/yai-spine37-home-$$"}
export YAI_HOME

cleanup() {
  rm -rf "$YAI_HOME" \
    /tmp/yai-spine37-valid.jsonl \
    /tmp/yai-spine37-schema-mismatch.jsonl \
    /tmp/yai-spine37-missing-schema.jsonl \
    /tmp/yai-spine37-corrupt.jsonl
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

cat > /tmp/yai-spine37-valid.jsonl <<'JSONL'
{"schema":"yai.store.record.v0","record_id":"rec:spine37-attempt","case_ref":"case:spine37","record_kind":"attempt","subject_ref":"subject:spine37","attempt_id":"op:spine37","decision_id":"","receipt_id":"","summary":"attempt:spine37"}
{"schema":"yai.store.record.v0","record_id":"rec:spine37-receipt","case_ref":"case:spine37","record_kind":"receipt","subject_ref":"subject:spine37","attempt_id":"op:spine37","decision_id":"decision:spine37","receipt_id":"receipt:spine37","summary":"receipt:spine37"}
JSONL

dry_run=$("$YAI_BIN" journal replay --path /tmp/yai-spine37-valid.jsonl --dry-run)
require_line "$dry_run" "journal_replay: dry_run"
require_line "$dry_run" "record_schema: yai.record.v1"
require_line "$dry_run" "journal_schema: yai.store.record.v0"
require_line "$dry_run" "compatibility: compatible"
require_line "$dry_run" "cursor_line: 0"
require_line "$dry_run" "would_update_cursor: yes"

first=$("$YAI_BIN" journal replay --path /tmp/yai-spine37-valid.jsonl)
require_line "$first" "journal_replay: completed"
require_line "$first" "records_written: 2"
require_line "$first" "cursor_line: 2"
require_line "$first" "replay_status: completed"
require_line "$first" "compatibility: compatible"

status=$("$YAI_BIN" journal replay-status --path /tmp/yai-spine37-valid.jsonl)
require_line "$status" "journal_replay_status:"
require_line "$status" "record_schema: yai.record.v1"
require_line "$status" "cursor_line: 2"
require_line "$status" "replay_status: completed"
require_line "$status" "records_written: 2"
require_line "$status" "compatibility: compatible"

second=$("$YAI_BIN" journal replay --path /tmp/yai-spine37-valid.jsonl)
require_line "$second" "journal_replay: completed"
require_line "$second" "records_written: 0"
require_line "$second" "records_duplicate: 2"
require_line "$second" "cursor_line: 2"
require_line "$second" "idempotent: yes"

cat > /tmp/yai-spine37-schema-mismatch.jsonl <<'JSONL'
{"schema":"yai.record.v2","record_id":"rec:spine37-mismatch","record_kind":"receipt","case_ref":"case:spine37","payload":{}}
JSONL

mismatch=$("$YAI_BIN" journal replay --path /tmp/yai-spine37-schema-mismatch.jsonl)
require_line "$mismatch" "journal_replay: failed"
require_line "$mismatch" "record_schema: yai.record.v2"
require_line "$mismatch" "supported_schema: yai.record.v1"
require_line "$mismatch" "compatibility: schema_mismatch"
require_line "$mismatch" "records_written: 0"
require_line "$mismatch" "replay_status: incompatible"

cat > /tmp/yai-spine37-missing-schema.jsonl <<'JSONL'
{"record_id":"rec:spine37-legacy","record_kind":"receipt","case_ref":"case:spine37","payload":{}}
JSONL

missing_schema=$("$YAI_BIN" journal replay --path /tmp/yai-spine37-missing-schema.jsonl)
require_line "$missing_schema" "journal_replay: failed"
require_line "$missing_schema" "journal_schema: none"
require_line "$missing_schema" "reason: invalid_schema"
require_line "$missing_schema" "records_written: 0"

printf '{"schema":"yai.store.record.v0","record_id":"rec:ok","record_kind":"receipt"}\n{broken\n' \
  > /tmp/yai-spine37-corrupt.jsonl
corrupt=$("$YAI_BIN" journal replay --path /tmp/yai-spine37-corrupt.jsonl)
require_line "$corrupt" "journal_replay: failed"
require_line "$corrupt" "invalid_entries: 1"
require_line "$corrupt" "records_written: 0"
require_line "$corrupt" "replay_status: failed"

printf 'replay:idempotent ok\n'
printf 'replay:cursor ok\n'
printf 'replay:schema_mismatch ok\n'
printf 'replay:status completed\n'
printf 'replay:no_false_complete ok\n'
