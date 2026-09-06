#!/bin/sh
# YAI - SPINE.36 journal replay to LMDB smoke
#
# Purpose:
#   Verify explicit journal replay materializes valid journal records into the
#   frozen LMDB record plane and remains idempotent.
#
# Scope:
#   Exercises dry-run, real replay, idempotent replay, store query and corrupt
#   journal failure.
#
# Non-goals:
#   Does not rebuild graph, facts, memory, RuntimeGraph or HNSW.
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
YAI_BIN="$ROOT/yai"
YAI_HOME=${YAI_HOME:-"/tmp/yai-spine36-home-$$"}
export YAI_HOME

cleanup() {
  rm -rf "$YAI_HOME" /tmp/yai-spine36-valid.jsonl /tmp/yai-spine36-bad.jsonl
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

reject_path() {
  path="$1"
  if [ -e "$path" ]; then
    printf 'unexpected path exists: %s\n' "$path" >&2
    exit 1
  fi
}

rm -rf "$YAI_HOME"
mkdir -p "$YAI_HOME"

cat > /tmp/yai-spine36-valid.jsonl <<'JSONL'
{"schema":"yai.store.record.v0","record_id":"rec:spine36-attempt","case_ref":"case:spine36","record_kind":"attempt","subject_ref":"subject:spine36","attempt_id":"op:spine36","decision_id":"","receipt_id":"","summary":"attempt:spine36"}
{"schema":"yai.store.record.v0","record_id":"rec:spine36-receipt","case_ref":"case:spine36","record_kind":"receipt","subject_ref":"subject:spine36","attempt_id":"op:spine36","decision_id":"decision:spine36","receipt_id":"receipt:spine36","summary":"receipt:spine36"}
JSONL

dry_run=$("$YAI_BIN" journal replay --path /tmp/yai-spine36-valid.jsonl --dry-run)
require_line "$dry_run" "journal_replay: dry_run"
require_line "$dry_run" "lines_total: 2"
require_line "$dry_run" "records_to_write: 2"
require_line "$dry_run" "would_write_lmdb: yes"
require_line "$dry_run" "replay_ready: yes"
reject_path "$YAI_HOME/store/lmdb"

write=$("$YAI_BIN" journal replay --path /tmp/yai-spine36-valid.jsonl)
require_line "$write" "journal_replay: completed"
require_line "$write" "records_seen: 2"
require_line "$write" "records_written: 2"
require_line "$write" "records_duplicate: 0"
require_line "$write" "record_store_status: ready"

summary=$("$YAI_BIN" store summary)
require_line "$summary" "record_store_status: ready"
require_line "$summary" "records_total: 2"
require_line "$summary" "records_by_case: 2"
require_line "$summary" "records_by_kind: 2"
require_line "$summary" "records_by_subject: 2"
require_line "$summary" "records_by_receipt: 1"

case_list=$("$YAI_BIN" store record list --case case:spine36 --limit 10)
require_line "$case_list" "records_total: 2"
require_line "$case_list" "record_id: rec:spine36-attempt"
require_line "$case_list" "record_id: rec:spine36-receipt"

second=$("$YAI_BIN" journal replay --path /tmp/yai-spine36-valid.jsonl)
require_line "$second" "journal_replay: completed"
require_line "$second" "records_seen: 2"
require_line "$second" "records_written: 0"
require_line "$second" "records_duplicate: 2"
require_line "$second" "idempotent: yes"

printf '{"record_id":"ok"}\n{broken\n' > /tmp/yai-spine36-bad.jsonl
bad=$("$YAI_BIN" journal replay --path /tmp/yai-spine36-bad.jsonl)
require_line "$bad" "journal_replay: failed"
require_line "$bad" "replay_ready: no"
require_line "$bad" "invalid_entries: 1"
require_line "$bad" "records_written: 0"
require_line "$bad" "reason: invalid_json"

printf 'journal_replay:dry_run ok\n'
printf 'journal_replay:write ok\n'
printf 'journal_replay:idempotent ok\n'
printf 'journal_replay:corrupt invalid_json ok\n'
printf 'store_summary:ready ok\n'
