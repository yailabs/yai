#!/bin/sh
# YAI - SPINE.33 LMDB CLI manual validation smoke
#
# Purpose:
#   Freeze the line-oriented LMDB command surface used by operators.
#
# Scope:
#   Verifies status, summary, get and list output shapes over the existing
#   daemon-loop journal import path.
#
# Non-goals:
#   Does not add new indexes, query planning or journal replay fallback.
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
YAI_HOME=${YAI_HOME:-"/tmp/yai-spine33-home-$$"}
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
mkdir -p "$YAI_HOME/run" "$RECORD_STORE"

initial_status=$("$YAI_BIN" store status)
require_line "$initial_status" "record_store_backend: lmdb"
require_line "$initial_status" "record_store_status: not_initialized"
require_line "$initial_status" "record_store_path: $RECORD_STORE"
reject_line "$initial_status" "record_store_status: ready"

initial_summary=$("$YAI_BIN" store summary)
require_line "$initial_summary" "record_store_status: not_initialized"
require_line "$initial_summary" "records_total: 0"
require_line "$initial_summary" "records_by_case: 0"
require_line "$initial_summary" "records_by_kind: 0"
require_line "$initial_summary" "records_by_subject: 0"
require_line "$initial_summary" "records_by_receipt: 0"

prewrite_get=$("$YAI_BIN" store record get rec:missing)
require_line "$prewrite_get" "record_store_status: not_initialized"
reject_line "$prewrite_get" "record: not_found"

YAI_HOME="$YAI_HOME" "$YAID" --socket "$SOCKET" --foreground >/tmp/yai-spine33-yaid.log 2>&1 &
PID=$!
sleep 1

minimum_output=$("$YAI_BIN" daemon run-minimum-loop --socket "$SOCKET")
require_line "$minimum_output" '"status":"completed"'

ready_status=$("$YAI_BIN" store status)
require_line "$ready_status" "record_store_status: ready"
require_line "$ready_status" "schema: yai.record.v1"
require_line "$ready_status" "indexes: records_by_id,records_by_case,records_by_kind,records_by_subject,records_by_receipt"

minimum_summary=$("$YAI_BIN" store summary)
require_line "$minimum_summary" "record_store_backend: lmdb"
require_line "$minimum_summary" "record_store_status: ready"
require_line "$minimum_summary" "records_total: 8"
require_line "$minimum_summary" "records_by_case: 8"
require_line "$minimum_summary" "records_by_kind: 8"
require_line "$minimum_summary" "records_by_subject:"
require_line "$minimum_summary" "records_by_receipt:"

case_list=$("$YAI_BIN" store record list --case case:new12-daemon --limit 10)
require_line "$case_list" "filter: case"
require_line "$case_list" "filter_value: case:new12-daemon"
require_line "$case_list" "records_total: 8"
require_line "$case_list" "limit: 10"
require_line "$case_list" "records:"
require_line "$case_list" "- record_id: rec:new12-min-receipt"
require_line "$case_list" "  record_kind: receipt"
require_line "$case_list" "  case_ref: case:new12-daemon"

kind_list=$("$YAI_BIN" store record list --kind receipt --limit 1)
require_line "$kind_list" "filter: kind"
require_line "$kind_list" "filter_value: receipt"
require_line "$kind_list" "records_total: 1"
require_line "$kind_list" "limit: 1"
require_line "$kind_list" "- record_id: rec:new12-min-receipt"

record_id=$(field_value "- record_id" "$kind_list" | sed -n '1p')
if [ -z "$record_id" ]; then
  printf 'failed to derive record id from kind list:\n%s\n' "$kind_list" >&2
  exit 1
fi

record_get=$("$YAI_BIN" store record get "$record_id")
require_line "$record_get" "schema: yai.record.v1"
require_line "$record_get" "record_id: $record_id"
require_line "$record_get" "record_kind: receipt"
require_line "$record_get" "case_ref: case:new12-daemon"
require_line "$record_get" "source:"
require_line "$record_get" "  plane: journal"
require_line "$record_get" "  ref:"
require_line "$record_get" "payload:"
require_line "$record_get" "  summary:"

missing_record=$("$YAI_BIN" store record get rec:missing)
require_line "$missing_record" "record: not_found"
reject_line "$missing_record" "journal"

zero_subject=$("$YAI_BIN" store record list --subject subject:missing --limit 10)
require_line "$zero_subject" "filter: subject"
require_line "$zero_subject" "filter_value: subject:missing"
require_line "$zero_subject" "records_total: 0"
require_line "$zero_subject" "limit: 10"
require_line "$zero_subject" "records: none"

filesystem_output=$("$YAI_BIN" daemon run-filesystem-loop --socket "$SOCKET")
require_line "$filesystem_output" '"status":"completed"'

filesystem_summary=$("$YAI_BIN" store summary)
subject_count=$(field_value records_by_subject "$filesystem_summary")
receipt_count=$(field_value records_by_receipt "$filesystem_summary")
if [ -z "$subject_count" ] || [ "$subject_count" -le 0 ]; then
  printf 'expected records_by_subject to be non-zero\n%s\n' "$filesystem_summary" >&2
  exit 1
fi
if [ -z "$receipt_count" ] || [ "$receipt_count" -le 0 ]; then
  printf 'expected records_by_receipt to be non-zero\n%s\n' "$filesystem_summary" >&2
  exit 1
fi

subject_list=$("$YAI_BIN" store record list --subject subject:filesystem-sandbox --limit 10)
require_line "$subject_list" "filter: subject"
require_line "$subject_list" "filter_value: subject:filesystem-sandbox"
require_line "$subject_list" "limit: 10"
require_line "$subject_list" "records:"
require_line "$subject_list" "case_ref: case:new12-filesystem"

receipt_list=$("$YAI_BIN" store record list --receipt receipt:new12-fs-write --limit 10)
require_line "$receipt_list" "filter: receipt"
require_line "$receipt_list" "filter_value: receipt:new12-fs-write"
require_line "$receipt_list" "limit: 10"
require_line "$receipt_list" "- record_id: rec:new12-fs-write-receipt"
require_line "$receipt_list" "record_kind: filesystem_receipt"

"$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null
wait "$PID" 2>/dev/null || true
PID=""

printf 'record_store_cli_manual_validation:spine33 ok\n'
