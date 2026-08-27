#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
YAI_HOME=${YAI_HOME:-"/tmp/yai-spine24a-home-$$"}
PREFIX=${PREFIX:-"/tmp/yai-spine24a-install-$$"}
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
  rm -rf "$YAI_HOME" "$PREFIX"
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

rm -rf "$YAI_HOME" "$PREFIX"
mkdir -p "$YAI_HOME/run"

paths=$(make -C "$ROOT" --no-print-directory print-install-paths PREFIX="$PREFIX" YAI_HOME="$YAI_HOME")
require_line "$paths" "PREFIX=$PREFIX"
require_line "$paths" "YAI_HOME=$YAI_HOME"
require_line "$paths" "run=$YAI_HOME/run"
require_line "$paths" "socket=$SOCKET"
printf 'command_surface:install-paths ok\n'

make -C "$ROOT" --no-print-directory check-foundation-freeze >/dev/null
printf 'command_surface:foundation-check ok\n'

"$YAI_BIN" --version >/dev/null
info=$("$YAI_BIN" info)
require_line "$info" "yai: technical YAI control command"
doctor_missing=$("$YAI_BIN" doctor)
require_line "$doctor_missing" "hot_state_status: unavailable"
printf 'command_surface:runtime-info ok\n'

missing=$("$YAI_BIN" hot status)
require_line "$missing" "hot_state: unavailable"
require_line "$missing" "reason: missing_snapshot"
printf 'command_surface:hot-state missing ok\n'

printf '{broken\n' > "$HOT"
corrupt=$("$YAI_BIN" hot status)
require_line "$corrupt" "hot_state: unavailable"
require_line "$corrupt" "reason: invalid_snapshot"
printf 'command_surface:hot-state corrupt ok\n'

"$YAID" --version >/dev/null
YAI_HOME="$YAI_HOME" "$YAID" --socket "$SOCKET" --foreground >/tmp/yai-spine24a-yaid.log 2>&1 &
PID=$!
sleep 1

daemon_status=$("$YAI_BIN" daemon status --socket "$SOCKET")
require_line "$daemon_status" '"status":"ok"'
daemon_info=$("$YAI_BIN" daemon info --socket "$SOCKET")
require_line "$daemon_info" '"message":"yaid'
printf 'command_surface:daemon-status ok\n'

"$YAI_BIN" daemon run-minimum-loop --socket "$SOCKET" >/dev/null
valid=$("$YAI_BIN" hot status)
require_line "$valid" "hot_state: active"
require_line "$valid" "snapshot: $HOT"
require_line "$valid" "schema: yai.hot_state.v1"
require_line "$valid" "case_session: active"
require_line "$valid" "projection_policy: usable"
require_line "$valid" "freshness_policy: usable"
printf 'command_surface:hot-state valid ok\n'

"$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null
wait "$PID" 2>/dev/null || true
PID=""

printf 'command_surface:spine24a ok\n'
