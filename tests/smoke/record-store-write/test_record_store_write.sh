#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
YAI_HOME=${YAI_HOME:-"/tmp/yai-spine30-home-$$"}
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
  if ! printf '%s\n' "$output" | grep -Fq "$expected"; then
    printf 'missing expected line: %s\noutput:\n%s\n' "$expected" "$output" >&2
    exit 1
  fi
}

extract_json_string() {
  key="$1"
  sed -n "s/.*\"$key\":\"\\([^\"]*\\)\".*/\\1/p"
}

records_total() {
  printf '%s\n' "$1" | sed -n 's/^records_total: //p'
}

rm -rf "$YAI_HOME"
mkdir -p "$YAI_HOME/run"

initial=$("$YAI_BIN" store status)
require_line "$initial" "record_store_backend: lmdb"
require_line "$initial" "record_store_status: missing"
require_line "$initial" "record_store_path: $RECORD_STORE"

YAI_HOME="$YAI_HOME" "$YAID" --socket "$SOCKET" --foreground >/tmp/yai-spine30-yaid.log 2>&1 &
PID=$!
sleep 1

minimum_output=$("$YAI_BIN" daemon run-minimum-loop --socket "$SOCKET")
require_line "$minimum_output" '"status":"completed"'
minimum_journal=$(printf '%s\n' "$minimum_output" | extract_json_string journal_path)
if [ -z "$minimum_journal" ] || [ ! -f "$minimum_journal" ]; then
  printf 'minimum journal missing: %s\noutput:\n%s\n' "$minimum_journal" "$minimum_output" >&2
  exit 1
fi

minimum_status=$("$YAI_BIN" store status)
require_line "$minimum_status" "record_store_status: ready"
minimum_summary=$("$YAI_BIN" store summary)
require_line "$minimum_summary" "record_store_backend: lmdb"
require_line "$minimum_summary" "record_store_status: ready"
require_line "$minimum_summary" "records_total: 8"
require_line "$minimum_summary" "records_by_case: 8"
require_line "$minimum_summary" "records_by_kind: 8"

filesystem_output=$("$YAI_BIN" daemon run-filesystem-loop --socket "$SOCKET")
require_line "$filesystem_output" '"status":"completed"'
filesystem_journal=$(printf '%s\n' "$filesystem_output" | extract_json_string journal_path)
if [ -z "$filesystem_journal" ] || [ ! -f "$filesystem_journal" ]; then
  printf 'filesystem journal missing: %s\noutput:\n%s\n' "$filesystem_journal" "$filesystem_output" >&2
  exit 1
fi

filesystem_summary=$("$YAI_BIN" store summary)
filesystem_total=$(records_total "$filesystem_summary")
if [ -z "$filesystem_total" ] || [ "$filesystem_total" -le 8 ]; then
  printf 'expected filesystem import to increase LMDB record count\n%s\n' "$filesystem_summary" >&2
  exit 1
fi
require_line "$filesystem_summary" "record_store_status: ready"

"$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null
wait "$PID" 2>/dev/null || true
PID=""

printf 'record_store_write:spine30 ok\n'
