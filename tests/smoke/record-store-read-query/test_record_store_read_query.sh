#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
YAI_HOME=${YAI_HOME:-"/tmp/yai-spine31-home-$$"}
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

rm -rf "$YAI_HOME"
mkdir -p "$YAI_HOME/run"

missing_read=$("$YAI_BIN" store record get rec:missing)
require_line "$missing_read" "record_store_status: missing"

YAI_HOME="$YAI_HOME" "$YAID" --socket "$SOCKET" --foreground >/tmp/yai-spine31-yaid.log 2>&1 &
PID=$!
sleep 1

minimum_output=$("$YAI_BIN" daemon run-minimum-loop --socket "$SOCKET")
require_line "$minimum_output" '"status":"completed"'
minimum_journal=$(printf '%s\n' "$minimum_output" | extract_json_string journal_path)
if [ -z "$minimum_journal" ] || [ ! -f "$minimum_journal" ]; then
  printf 'minimum journal missing: %s\noutput:\n%s\n' "$minimum_journal" "$minimum_output" >&2
  exit 1
fi

summary=$("$YAI_BIN" store summary)
require_line "$summary" "record_store_status: ready"
require_line "$summary" "records_total: 8"

kind_list=$("$YAI_BIN" store record list --kind receipt --limit 10)
require_line "$kind_list" "filter: kind"
require_line "$kind_list" "filter_value: receipt"
require_line "$kind_list" "records_total: 1"
require_line "$kind_list" "limit: 10"
require_line "$kind_list" "- record_id: rec:new12-min-receipt"
require_line "$kind_list" "case_ref: case:new12-daemon"

case_list=$("$YAI_BIN" store record list --case case:new12-daemon --limit 10)
require_line "$case_list" "filter: case"
require_line "$case_list" "filter_value: case:new12-daemon"
require_line "$case_list" "records_total: 8"
require_line "$case_list" "limit: 10"
require_line "$case_list" "- record_id: rec:new12-min-receipt"

record_get=$("$YAI_BIN" store record get rec:new12-min-receipt)
require_line "$record_get" "schema: yai.record.v1"
require_line "$record_get" "record_id: rec:new12-min-receipt"
require_line "$record_get" "record_kind: receipt"
require_line "$record_get" "case_ref: case:new12-daemon"
require_line "$record_get" "source:"
require_line "$record_get" "  plane: journal"
require_line "$record_get" "payload:"

missing_record=$("$YAI_BIN" store record get rec:missing)
require_line "$missing_record" "record: not_found"

"$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null
wait "$PID" 2>/dev/null || true
PID=""

printf 'record_store_read_query:spine31 ok\n'
