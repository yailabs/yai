#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
YAID="$ROOT/build/yaid"
TMP_DIR="build/tmp/yai-spine44c-$$"
SOCKET="$TMP_DIR/run/yaid.sock"
DAEMON_PID=""

cleanup() {
  if [[ -n "$DAEMON_PID" ]]; then
    YAI_HOME="$TMP_DIR/daemon-home" "$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null 2>&1 || true
    wait "$DAEMON_PID" >/dev/null 2>&1 || true
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

require_line() {
  local haystack="$1"
  local needle="$2"
  grep -Fq -- "$needle" <<<"$haystack"
}

mkdir -p "$TMP_DIR/run" "$TMP_DIR/daemon-home"
YAI_HOME="$TMP_DIR/daemon-home" "$YAID" --socket "$SOCKET" --foreground &
DAEMON_PID=$!
sleep 1

run_review_loop() {
  local home="$1"
  mkdir -p "$home"
  YAI_HOME="$home" "$YAI_BIN" daemon run-filesystem-review-loop --socket "$SOCKET"
}

APPROVE_HOME="$TMP_DIR/home-approve"
loop_output="$(run_review_loop "$APPROVE_HOME")"
require_line "$loop_output" "outside_sandbox_status: blocked"
require_line "$loop_output" "outside_sandbox_carrier_attempted: false"
require_line "$loop_output" "outside_sandbox_execution_performed: false"
require_line "$loop_output" "outside_sandbox_receipt_status: blocked"
outside_receipt="$(YAI_HOME="$APPROVE_HOME" "$YAI_BIN" store record get rec:new12-fs-outside-write-receipt)"
require_line "$outside_receipt" "status:blocked"
require_line "$outside_receipt" "receipt_status:blocked"
require_line "$outside_receipt" "carrier_attempted:false"
require_line "$outside_receipt" "execution_performed:false"
printf "review_matrix:blocked ok\n"

require_line "$loop_output" "review_required_attempt: pending_operator"
require_line "$loop_output" "review_id: review:new12-fs-write-review"
require_line "$loop_output" "status: pending_operator"
require_line "$loop_output" "carrier_attempted: false"
require_line "$loop_output" "execution_performed: false"
printf "review_matrix:pending ok\n"

require_line "$loop_output" "next_commands:"
require_line "$loop_output" "approve: yai control approve review:new12-fs-write-review --reason \"...\""
require_line "$loop_output" "deny: yai control deny review:new12-fs-write-review --reason \"...\""
require_line "$loop_output" "defer: yai control defer review:new12-fs-write-review --reason \"...\""
require_line "$loop_output" "quarantine: yai control quarantine review:new12-fs-write-review --reason \"...\""
pending="$(YAI_HOME="$APPROVE_HOME" "$YAI_BIN" control pending --case case:new12-filesystem)"
require_line "$pending" "next_commands:"
printf "review_matrix:next_commands ok\n"

show="$(YAI_HOME="$APPROVE_HOME" "$YAI_BIN" control show review:new12-fs-write-review)"
require_line "$show" "subject:linenoise-terminal is prompt surface only"
require_line "$show" "subject:operator-reviewer is local-dev review authority"

wait_timeout="$(YAI_HOME="$APPROVE_HOME" "$YAI_BIN" control wait review:new12-fs-write-review --timeout 1)"
require_line "$wait_timeout" "status: pending_operator"
require_line "$wait_timeout" "resolved: no"
require_line "$wait_timeout" "timeout: true"
printf "review_matrix:wait_timeout ok\n"

watch_output="$(YAI_HOME="$APPROVE_HOME" "$YAI_BIN" control watch --case case:new12-filesystem --interval-ms 1 --max-events 1)"
require_line "$watch_output" "control_watch:"
require_line "$watch_output" "events_seen: 1"
printf "review_matrix:watch ok\n"

non_tty_status=0
non_tty_output="$(YAI_HOME="$APPROVE_HOME" "$YAI_BIN" control review --case case:new12-filesystem --interactive 2>&1)" || non_tty_status=$?
test "$non_tty_status" -ne 0
require_line "$non_tty_output" "interactive_review: unavailable"
require_line "$non_tty_output" "reason: not_a_tty"
printf "review_matrix:non_tty ok\n"

approve="$(YAI_HOME="$APPROVE_HOME" "$YAI_BIN" control approve review:new12-fs-write-review --reason "matrix approve")"
require_line "$approve" "action: approve"
require_line "$approve" "decision: allow_with_constraints"
require_line "$approve" "carrier_attempted: true"
require_line "$approve" "execution_performed: true"
require_line "$approve" "receipt_status: executed"
require_line "$approve" "effect_chain: typed_grant_prepare_finalize"
test -f "$APPROVE_HOME/tmp/filesystem-review-loop/sandbox/reviewed-output.txt"
printf "review_matrix:approve executed ok\n"

wait_resolved="$(YAI_HOME="$APPROVE_HOME" "$YAI_BIN" control wait review:new12-fs-write-review --timeout 1)"
require_line "$wait_resolved" "status: approved"
require_line "$wait_resolved" "resolved: yes"
require_line "$wait_resolved" "timeout: false"
require_line "$wait_resolved" "receipt_status: executed"
printf "review_matrix:wait_resolved ok\n"

DENY_HOME="$TMP_DIR/home-deny"
run_review_loop "$DENY_HOME" >/dev/null
deny="$(YAI_HOME="$DENY_HOME" "$YAI_BIN" control deny review:new12-fs-write-review --reason "matrix deny")"
require_line "$deny" "action: deny"
require_line "$deny" "decision: deny"
require_line "$deny" "carrier_attempted: false"
require_line "$deny" "execution_performed: false"
require_line "$deny" "receipt_status: blocked"
test ! -f "$DENY_HOME/tmp/filesystem-review-loop/sandbox/reviewed-output.txt"
printf "review_matrix:deny no_execution ok\n"

DEFER_HOME="$TMP_DIR/home-defer"
run_review_loop "$DEFER_HOME" >/dev/null
defer="$(YAI_HOME="$DEFER_HOME" "$YAI_BIN" control defer review:new12-fs-write-review --reason "matrix defer")"
require_line "$defer" "action: defer"
require_line "$defer" "status: deferred"
require_line "$defer" "carrier_attempted: false"
require_line "$defer" "execution_performed: false"
require_line "$defer" "receipt_status: deferred"
test ! -f "$DEFER_HOME/tmp/filesystem-review-loop/sandbox/reviewed-output.txt"
printf "review_matrix:defer no_execution ok\n"

QUARANTINE_HOME="$TMP_DIR/home-quarantine"
run_review_loop "$QUARANTINE_HOME" >/dev/null
quarantine="$(YAI_HOME="$QUARANTINE_HOME" "$YAI_BIN" control quarantine review:new12-fs-write-review --reason "matrix quarantine")"
require_line "$quarantine" "action: quarantine"
require_line "$quarantine" "status: quarantined"
require_line "$quarantine" "carrier_attempted: false"
require_line "$quarantine" "execution_performed: false"
require_line "$quarantine" "receipt_status: quarantined"
require_line "$quarantine" "quarantine_scope: case"
test ! -f "$QUARANTINE_HOME/tmp/filesystem-review-loop/sandbox/reviewed-output.txt"
printf "review_matrix:quarantine no_execution ok\n"
