#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
YAI_HOME=${YAI_HOME:-"$ROOT/build/tmp/spine24-home"}
export YAI_HOME
YAI_BIN="$ROOT/target/debug/yai"
YAID="$ROOT/build/yaid"
SOCKET="$YAI_HOME/run/yaid.sock"
HOT="$YAI_HOME/run/hot-state.json"
PID=""

cleanup() {
  if [ -n "$PID" ]; then
    "$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null 2>&1 || true
    wait "$PID" 2>/dev/null || true
  fi
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

rm -rf "$YAI_HOME"
mkdir -p "$YAI_HOME/run"

missing=$("$YAI_BIN" hot status)
require_line "$missing" "hot_state: unavailable"
require_line "$missing" "reason: missing_snapshot"
printf 'snapshot:missing handled\n'

printf '{broken\n' > "$HOT"
corrupt=$("$YAI_BIN" hot status)
require_line "$corrupt" "hot_state: unavailable"
require_line "$corrupt" "reason: invalid_snapshot"
printf 'snapshot:corrupt handled\n'

YAI_HOME="$YAI_HOME" "$YAID" --socket "$SOCKET" --foreground >/tmp/yai-spine24-yaid.log 2>&1 &
PID=$!
sleep 1

"$YAI_BIN" daemon run-minimum-loop --socket "$SOCKET" >/dev/null

valid=$("$YAI_BIN" hot status)
require_line "$valid" "hot_state: active"
require_line "$valid" "schema: yai.hot_state.v1"
require_line "$valid" "snapshot_status: active"
require_line "$valid" "projection: fresh"
require_line "$valid" "case_session: active"
require_line "$valid" "case_context: active"
printf 'snapshot:valid handled\n'

doctor=$("$YAI_BIN" doctor)
require_line "$doctor" "Posture            NOT_INITIALIZED"
require_line "$doctor" "YAI_HOME        $YAI_HOME"
require_line "$doctor" "Local identity  not_enrolled"
printf 'doctor:legacy snapshot does not masquerade as initialization\n'

if [ -e "$HOT.tmp" ]; then
  printf 'unexpected temp snapshot left behind: %s.tmp\n' "$HOT" >&2
  exit 1
fi
printf 'snapshot:atomic rename clean\n'
