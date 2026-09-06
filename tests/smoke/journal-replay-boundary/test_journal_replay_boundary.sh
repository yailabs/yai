#!/bin/sh
# YAI - SPINE.35 journal replay boundary smoke
#
# Purpose:
#   Verify journal inspect diagnostics before journal-to-LMDB replay exists.
#
# Scope:
#   Exercises valid, missing, corrupt, duplicate and empty journal inspection.
#
# Non-goals:
#   Does not rebuild LMDB, write LMDB, replay graph or repair journals.
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
YAI_BIN="$ROOT/yai"
YAI_HOME=${YAI_HOME:-"/tmp/yai-spine35-home-$$"}
export YAI_HOME

cleanup() {
  rm -rf "$YAI_HOME" /tmp/yai-spine35-valid.jsonl /tmp/yai-spine35-bad.jsonl /tmp/yai-spine35-empty.jsonl /tmp/yai-spine35-duplicate.jsonl
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

cat > /tmp/yai-spine35-valid.jsonl <<'JSONL'
{"schema":"yai.store.record.v0","record_id":"rec:spine35-valid","case_ref":"case:spine35","record_kind":"receipt","subject_ref":"subject:spine35","attempt_id":"op:spine35","decision_id":"decision:spine35","receipt_id":"receipt:spine35","summary":"receipt:blocked"}
JSONL

valid=$("$YAI_BIN" journal inspect --path /tmp/yai-spine35-valid.jsonl)
require_line "$valid" "journal_status: readable"
require_line "$valid" "lines_total: 1"
require_line "$valid" "valid_entries: 1"
require_line "$valid" "invalid_entries: 0"
require_line "$valid" "unsupported_entries: 0"
require_line "$valid" "duplicate_entries: 0"
require_line "$valid" "replay_ready: yes"
require_line "$valid" "lmdb_write: no"
reject_path "$YAI_HOME/store/lmdb"

missing=$("$YAI_BIN" journal inspect --path /tmp/yai-spine35-missing.jsonl)
require_line "$missing" "journal_status: missing"
require_line "$missing" "replay_ready: no"

printf '{"record_id":"ok"}\n{broken\n' > /tmp/yai-spine35-bad.jsonl
bad=$("$YAI_BIN" journal inspect --path /tmp/yai-spine35-bad.jsonl --show-errors)
require_line "$bad" "lines_total: 2"
require_line "$bad" "invalid_entries: 1"
require_line "$bad" "unsupported_entries: 1"
require_line "$bad" "replay_ready: no"
require_line "$bad" "line: 1"
require_line "$bad" "entry_status: invalid_schema"
require_line "$bad" "line: 2"
require_line "$bad" "entry_status: invalid_json"

: > /tmp/yai-spine35-empty.jsonl
empty=$("$YAI_BIN" journal inspect --path /tmp/yai-spine35-empty.jsonl)
require_line "$empty" "journal_status: readable"
require_line "$empty" "lines_total: 0"
require_line "$empty" "valid_entries: 0"
require_line "$empty" "replay_ready: no"

cat /tmp/yai-spine35-valid.jsonl /tmp/yai-spine35-valid.jsonl > /tmp/yai-spine35-duplicate.jsonl
duplicate=$("$YAI_BIN" journal inspect --path /tmp/yai-spine35-duplicate.jsonl --show-errors)
require_line "$duplicate" "lines_total: 2"
require_line "$duplicate" "valid_entries: 1"
require_line "$duplicate" "duplicate_entries: 1"
require_line "$duplicate" "entry_status: duplicate"
require_line "$duplicate" "replay_ready: no"

printf 'journal_inspect:valid ok\n'
printf 'journal_inspect:missing ok\n'
printf 'journal_inspect:corrupt invalid_json ok\n'
printf 'journal_inspect:empty ok\n'
printf 'journal_inspect:no_lmdb_write confirmed\n'
