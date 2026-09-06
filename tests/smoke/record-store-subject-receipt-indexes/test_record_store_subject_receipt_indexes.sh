#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
YAI_HOME=${YAI_HOME:-"/tmp/yai-spine32-home-$$"}
export YAI_HOME

YAI_BIN="$ROOT/yai"
YAID="$ROOT/build/yaid"
SOCKET="$YAI_HOME/run/yaid.sock"
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

extract_json_string() {
  key="$1"
  sed -n "s/.*\"$key\":\"\\([^\"]*\\)\".*/\\1/p"
}

field_value() {
  field="$1"
  printf '%s\n' "$2" | sed -n "s/^$field: //p"
}

rm -rf "$YAI_HOME"
mkdir -p "$YAI_HOME/run"

missing_subject=$("$YAI_BIN" store record list --subject subject:missing --limit 10)
require_line "$missing_subject" "record_store_status: missing"

YAI_HOME="$YAI_HOME" "$YAID" --socket "$SOCKET" --foreground >/tmp/yai-spine32-yaid.log 2>&1 &
PID=$!
sleep 1

filesystem_output=$("$YAI_BIN" daemon run-filesystem-loop --socket "$SOCKET")
require_line "$filesystem_output" '"status":"completed"'
filesystem_journal=$(printf '%s\n' "$filesystem_output" | extract_json_string journal_path)
if [ -z "$filesystem_journal" ] || [ ! -f "$filesystem_journal" ]; then
  printf 'filesystem journal missing: %s\noutput:\n%s\n' "$filesystem_journal" "$filesystem_output" >&2
  exit 1
fi

summary=$("$YAI_BIN" store summary)
require_line "$summary" "record_store_status: ready"
require_line "$summary" "records_by_subject:"
require_line "$summary" "records_by_receipt:"
subject_count=$(field_value records_by_subject "$summary")
receipt_count=$(field_value records_by_receipt "$summary")
if [ -z "$subject_count" ] || [ "$subject_count" -le 0 ]; then
  printf 'expected records_by_subject to be non-zero\n%s\n' "$summary" >&2
  exit 1
fi
if [ -z "$receipt_count" ] || [ "$receipt_count" -le 0 ]; then
  printf 'expected records_by_receipt to be non-zero\n%s\n' "$summary" >&2
  exit 1
fi

subject_list=$("$YAI_BIN" store record list --subject subject:filesystem-sandbox --limit 20)
require_line "$subject_list" "filter: subject"
require_line "$subject_list" "filter_value: subject:filesystem-sandbox"
require_line "$subject_list" "records_total:"
require_line "$subject_list" "limit: 20"
require_line "$subject_list" "- record_id: rec:new12-fs-authority-scope-fs"
require_line "$subject_list" "record_kind: authority_scope"
require_line "$subject_list" "case_ref: case:new12-filesystem"

receipt_list=$("$YAI_BIN" store record list --receipt receipt:new12-fs-write --limit 10)
require_line "$receipt_list" "filter: receipt"
require_line "$receipt_list" "filter_value: receipt:new12-fs-write"
require_line "$receipt_list" "records_total:"
require_line "$receipt_list" "limit: 10"
require_line "$receipt_list" "- record_id: rec:new12-fs-write-receipt"
require_line "$receipt_list" "record_kind: filesystem_receipt"
require_line "$receipt_list" "case_ref: case:new12-filesystem"

missing_subject_ready=$("$YAI_BIN" store record list --subject subject:missing --limit 10)
require_line "$missing_subject_ready" "filter: subject"
require_line "$missing_subject_ready" "filter_value: subject:missing"
require_line "$missing_subject_ready" "records_total: 0"
require_line "$missing_subject_ready" "limit: 10"
require_line "$missing_subject_ready" "records: none"

"$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null
wait "$PID" 2>/dev/null || true
PID=""

printf 'record_store_subject_receipt_indexes:spine32 ok\n'
