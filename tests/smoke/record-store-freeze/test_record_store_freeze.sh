#!/bin/sh
# YAI - SPINE.34 LMDB record-plane freeze smoke
#
# Purpose:
#   Verify the frozen LMDB record-plane command surface and no-fallback posture.
#
# Scope:
#   Exercises status, summary, get and list commands over the daemon journal
#   import path, then verifies that records are not synthesized from journal
#   when the LMDB environment is absent.
#
# Non-goals:
#   Does not add journal replay, graph/fact/memory behavior or new indexes.
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
YAI_HOME=${YAI_HOME:-"/tmp/yai-spine34-home-$$"}
export YAI_HOME

YAI_BIN="$ROOT/yai"
YAID="$ROOT/build/yaid"
SOCKET="$YAI_HOME/run/yaid.sock"
RECORD_STORE="$YAI_HOME/store/lmdb"
PID=""

cleanup() {
  if [ -n "$PID" ]; then
    "$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null 2>&1 || true
    wait "$PID" 2>/dev/null || true
  fi
  rm -rf "$YAI_HOME"
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

reject_line() {
  output="$1"
  rejected="$2"
  if printf '%s\n' "$output" | grep -Fq -- "$rejected"; then
    printf 'unexpected line: %s\noutput:\n%s\n' "$rejected" "$output" >&2
    exit 1
  fi
}

field_value() {
  field="$1"
  printf '%s\n' "$2" | sed -n "s/^$field: //p"
}

rm -rf "$YAI_HOME"
mkdir -p "$YAI_HOME/run"

missing_get=$("$YAI_BIN" store record get rec:new12-min-receipt)
require_line "$missing_get" "record_store_backend: lmdb"
require_line "$missing_get" "record_store_status: missing"
reject_line "$missing_get" "record: not_found"
reject_line "$missing_get" "schema: yai.record.v1"

YAI_HOME="$YAI_HOME" "$YAID" --socket "$SOCKET" --foreground >/tmp/yai-spine34-yaid.log 2>&1 &
PID=$!
sleep 1

minimum_output=$("$YAI_BIN" daemon run-minimum-loop --socket "$SOCKET")
require_line "$minimum_output" '"status":"completed"'

filesystem_output=$("$YAI_BIN" daemon run-filesystem-loop --socket "$SOCKET")
require_line "$filesystem_output" '"status":"completed"'

status=$("$YAI_BIN" store status)
require_line "$status" "record_store_backend: lmdb"
require_line "$status" "record_store_status: ready"
require_line "$status" "schema: yai.record.v1"
require_line "$status" "indexes: records_by_id,records_by_case,records_by_kind,records_by_subject,records_by_receipt"

summary=$("$YAI_BIN" store summary)
require_line "$summary" "record_store_status: ready"
records_total=$(field_value records_total "$summary")
records_by_case=$(field_value records_by_case "$summary")
records_by_kind=$(field_value records_by_kind "$summary")
records_by_subject=$(field_value records_by_subject "$summary")
records_by_receipt=$(field_value records_by_receipt "$summary")

for value in "$records_total" "$records_by_case" "$records_by_kind" "$records_by_subject" "$records_by_receipt"; do
  if [ -z "$value" ] || [ "$value" -le 0 ]; then
    printf 'expected non-zero summary counts\n%s\n' "$summary" >&2
    exit 1
  fi
done

case_list=$("$YAI_BIN" store record list --case case:new12-daemon --limit 10)
require_line "$case_list" "filter: case"
require_line "$case_list" "filter_value: case:new12-daemon"
require_line "$case_list" "- record_id: rec:new12-min-receipt"
require_line "$case_list" "record_kind: receipt"

decision_list=$("$YAI_BIN" store record list --kind decision --limit 10)
require_line "$decision_list" "filter: kind"
require_line "$decision_list" "filter_value: decision"
require_line "$decision_list" "record_kind: decision"
require_line "$decision_list" "case_ref: case:new12-daemon"

receipt_list=$("$YAI_BIN" store record list --receipt receipt:new12-fs-write --limit 10)
require_line "$receipt_list" "filter: receipt"
require_line "$receipt_list" "filter_value: receipt:new12-fs-write"
require_line "$receipt_list" "- record_id: rec:new12-fs-write-receipt"
require_line "$receipt_list" "record_kind: filesystem_receipt"

subject_list=$("$YAI_BIN" store record list --subject subject:filesystem-sandbox --limit 10)
require_line "$subject_list" "filter: subject"
require_line "$subject_list" "filter_value: subject:filesystem-sandbox"
require_line "$subject_list" "case_ref: case:new12-filesystem"

record_get=$("$YAI_BIN" store record get rec:new12-min-receipt)
require_line "$record_get" "schema: yai.record.v1"
require_line "$record_get" "record_id: rec:new12-min-receipt"
require_line "$record_get" "record_kind: receipt"
require_line "$record_get" "source:"
require_line "$record_get" "  plane: journal"
require_line "$record_get" "payload:"
require_line "$record_get" "  summary:"

missing_record=$("$YAI_BIN" store record get rec:spine34-missing)
require_line "$missing_record" "record: not_found"
reject_line "$missing_record" "journal fallback"

rm -rf "$RECORD_STORE"

post_remove_get=$("$YAI_BIN" store record get rec:new12-min-receipt)
require_line "$post_remove_get" "record_store_status: missing"
reject_line "$post_remove_get" "record_id: rec:new12-min-receipt"
reject_line "$post_remove_get" "schema: yai.record.v1"

"$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null
wait "$PID" 2>/dev/null || true
PID=""

printf 'record_store_freeze:status ok\n'
printf 'record_store_freeze:indexes ok\n'
printf 'record_store_freeze:record_get ok\n'
printf 'record_store_freeze:no_journal_fallback ok\n'
printf 'record_store_freeze:spine34 ok\n'
