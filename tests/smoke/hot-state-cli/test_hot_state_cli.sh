#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
YAI_HOME=${YAI_HOME:-"/tmp/yai-spine27-home-$$"}
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

rm -rf "$YAI_HOME"
mkdir -p "$YAI_HOME/run"

"$YAI_BIN" --version >/dev/null
info=$("$YAI_BIN" info)
require_line "$info" "status: SPINE.51 Fact Plane Freeze"

doctor_missing=$("$YAI_BIN" doctor)
require_line "$doctor_missing" "Posture            NOT_INITIALIZED"
require_line "$doctor_missing" "YAI_HOME        $YAI_HOME"
require_line "$doctor_missing" "Local identity  not_enrolled"

missing=$("$YAI_BIN" hot status)
require_line "$missing" "hot_state: unavailable"
require_line "$missing" "reason: missing_snapshot"
require_line "$missing" "freshness_policy: unknown"
printf 'hot_cli:missing snapshot ok\n'

printf '{broken\n' > "$HOT"
corrupt=$("$YAI_BIN" hot status)
require_line "$corrupt" "hot_state: unavailable"
require_line "$corrupt" "reason: invalid_snapshot"
require_line "$corrupt" "freshness_policy: unknown"
printf 'hot_cli:corrupt snapshot ok\n'

YAI_HOME="$YAI_HOME" "$YAID" --socket "$SOCKET" --foreground >/tmp/yai-spine27-yaid.log 2>&1 &
PID=$!
sleep 1

daemon_status=$("$YAI_BIN" daemon status --socket "$SOCKET")
require_line "$daemon_status" '"status":"ok"'

minimum_output=$("$YAI_BIN" daemon run-minimum-loop --socket "$SOCKET")
minimum_hot=$("$YAI_BIN" hot status)
require_line "$minimum_output" '"status":"completed"'
require_line "$minimum_hot" "hot_state: active"
require_line "$minimum_hot" "schema: yai.hot_state.v1"
require_line "$minimum_hot" "case_session: active"
require_line "$minimum_hot" "case_world: loaded"
require_line "$minimum_hot" "case_context: active"
require_line "$minimum_hot" "projection: fresh"
require_line "$minimum_hot" "stale_reason: none"
require_line "$minimum_hot" "freshness_policy: usable"
require_line "$minimum_hot" "last_decision: decision:new12-minimum"
require_line "$minimum_hot" "last_receipt: receipt:new12-minimum"
printf 'hot_cli:minimum hot status ok\n'

filesystem_output=$("$YAI_BIN" daemon run-filesystem-loop --socket "$SOCKET")
filesystem_journal=$(printf '%s\n' "$filesystem_output" | sed -n 's/.*"journal_path":"\([^"]*\)".*/\1/p')
if [ -z "$filesystem_journal" ]; then
  printf '%s\n' "$filesystem_output" >&2
  exit 1
fi

filesystem_hot=$("$YAI_BIN" hot status)
require_line "$filesystem_hot" "hot_state: active"
require_line "$filesystem_hot" "schema: yai.hot_state.v1"
require_line "$filesystem_hot" "case_session: active"
require_line "$filesystem_hot" "case_world: loaded"
require_line "$filesystem_hot" "case_context: active"
require_line "$filesystem_hot" "projection: fresh"
require_line "$filesystem_hot" "freshness_policy: usable"
require_line "$filesystem_hot" "last_decision: decision:new12-fs-write"
require_line "$filesystem_hot" "last_receipt: receipt:new12-fs-write"
printf 'hot_cli:filesystem hot status ok\n'

projection_model=$("$YAI_BIN" projection inspect --journal "$filesystem_journal" --consumer model)
require_line "$projection_model" "consumer: model"
require_line "$projection_model" "projection_freshness: fresh"
require_line "$projection_model" "stale_reason: none"
require_line "$projection_model" "freshness_policy: usable"
require_line "$projection_model" "freshness_source: hot_state"
printf 'hot_cli:projection freshness inspect ok\n'

doctor_valid=$("$YAI_BIN" doctor)
require_line "$doctor_valid" "Posture            NOT_INITIALIZED"
require_line "$doctor_valid" "YAI_HOME        $YAI_HOME"
require_line "$doctor_valid" "Local identity  not_enrolled"
printf 'hot_cli:doctor keeps legacy hot state nonauthoritative\n'

"$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null
wait "$PID" 2>/dev/null || true
PID=""

printf 'hot_cli:spine27 ok\n'
