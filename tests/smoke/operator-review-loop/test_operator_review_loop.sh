#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
YAID="$ROOT/build/yaid"
TMP_DIR="build/tmp/yai-spine44a-$$"
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
require_line "$loop_output" "filesystem_review_loop: completed"
require_line "$loop_output" "outside_sandbox_attempt: blocked"
require_line "$loop_output" "outside_sandbox_carrier_attempted: false"
printf "review_loop:outside_sandbox blocked ok\n"
require_line "$loop_output" "review_required_attempt: pending_operator"
require_line "$loop_output" "review_required: yes"
require_line "$loop_output" "review_id: review:new12-fs-write-review"
require_line "$loop_output" "status: pending_operator"
require_line "$loop_output" "carrier_attempted: false"
require_line "$loop_output" "execution_performed: false"
require_line "$loop_output" "next_commands:"
require_line "$loop_output" "approve: yai control approve review:new12-fs-write-review --reason \"...\""
require_line "$loop_output" "deny: yai control deny review:new12-fs-write-review --reason \"...\""
require_line "$loop_output" "defer: yai control defer review:new12-fs-write-review --reason \"...\""
require_line "$loop_output" "quarantine: yai control quarantine review:new12-fs-write-review --reason \"...\""
printf "review_loop:pending_operator ok\n"

pending="$(YAI_HOME="$APPROVE_HOME" "$YAI_BIN" control pending --case case:new12-filesystem)"
require_line "$pending" "control_pending:"
require_line "$pending" "items_total: 1"
require_line "$pending" "review_id: review:new12-fs-write-review"
require_line "$pending" "status: pending_operator"
printf "control_pending:list ok\n"

show="$(YAI_HOME="$APPROVE_HOME" "$YAI_BIN" control show review:new12-fs-write-review)"
require_line "$show" "subject:linenoise-terminal is prompt surface"
require_line "$show" "operator reviewer authority: local-dev"

approve="$(YAI_HOME="$APPROVE_HOME" "$YAI_BIN" control approve review:new12-fs-write-review --reason "smoke approve")"
require_line "$approve" "action: approve"
require_line "$approve" "status: approved"
require_line "$approve" "decision: allow_with_constraints"
require_line "$approve" "carrier_attempted: true"
require_line "$approve" "execution_performed: true"
require_line "$approve" "receipt_status: executed"
require_line "$approve" "effect_chain: typed_grant_prepare_finalize"
require_line "$approve" "execution_grant_id: grant:"
test -f "$APPROVE_HOME/tmp/filesystem-review-loop/sandbox/reviewed-output.txt"
printf "review:approve executed ok\n"

receipt_list="$(YAI_HOME="$APPROVE_HOME" "$YAI_BIN" store record list --kind filesystem_receipt --limit 20)"
require_line "$receipt_list" "rec:new12-fs-review-receipt-approve"
review_records="$(YAI_HOME="$APPROVE_HOME" "$YAI_BIN" store record list --kind review_request --limit 20)"
require_line "$review_records" "rec:review:new12-fs-write-review"
decision_records="$(YAI_HOME="$APPROVE_HOME" "$YAI_BIN" store record list --kind review_decision --limit 20)"
require_line "$decision_records" "rec:review-decision:new12-fs-write-review-approve"
pending_records="$(YAI_HOME="$APPROVE_HOME" "$YAI_BIN" store record list --kind control_pending --limit 20)"
require_line "$pending_records" "rec:pending:new12-fs-write-review"
test -s "$APPROVE_HOME/store/control/review.jsonl"

canonical_summary="$(YAI_HOME="$APPROVE_HOME" "$YAI_BIN" store summary)"
grep -Eq "transitions_total: [1-9][0-9]*" <<<"$canonical_summary"
grep -Eq "cases_materialized: [1-9][0-9]*" <<<"$canonical_summary"
printf "review:canonical_transition_authority ok\n"

YAI_HOME="$APPROVE_HOME" "$YAI_BIN" graph materialize --case case:new12-filesystem >/dev/null
relations="$(YAI_HOME="$APPROVE_HOME" "$YAI_BIN" graph relations --case case:new12-filesystem --limit 200)"
require_line "$relations" "rec:review:new12-fs-write-review"

DENY_HOME="$TMP_DIR/home-deny"
run_review_loop "$DENY_HOME" >/dev/null
deny="$(YAI_HOME="$DENY_HOME" "$YAI_BIN" control deny review:new12-fs-write-review --reason "smoke deny")"
require_line "$deny" "status: denied"
require_line "$deny" "receipt_status: blocked"
require_line "$deny" "carrier_attempted: false"
require_line "$deny" "execution_performed: false"
test ! -f "$DENY_HOME/tmp/filesystem-review-loop/sandbox/reviewed-output.txt"
printf "review:deny blocked ok\n"

DEFER_HOME="$TMP_DIR/home-defer"
run_review_loop "$DEFER_HOME" >/dev/null
defer="$(YAI_HOME="$DEFER_HOME" "$YAI_BIN" control defer review:new12-fs-write-review --reason "smoke defer")"
require_line "$defer" "status: deferred"
require_line "$defer" "receipt_status: deferred"
require_line "$defer" "carrier_attempted: false"
require_line "$defer" "execution_performed: false"
test ! -f "$DEFER_HOME/tmp/filesystem-review-loop/sandbox/reviewed-output.txt"
printf "review:defer no_execution ok\n"

QUARANTINE_HOME="$TMP_DIR/home-quarantine"
run_review_loop "$QUARANTINE_HOME" >/dev/null
quarantine="$(YAI_HOME="$QUARANTINE_HOME" "$YAI_BIN" control quarantine review:new12-fs-write-review --reason "smoke quarantine")"
require_line "$quarantine" "status: quarantined"
require_line "$quarantine" "receipt_status: quarantined"
require_line "$quarantine" "quarantine_scope: case"
require_line "$quarantine" "carrier_attempted: false"
require_line "$quarantine" "execution_performed: false"
test ! -f "$QUARANTINE_HOME/tmp/filesystem-review-loop/sandbox/reviewed-output.txt"
printf "review:quarantine no_execution ok\n"

printf "carrier_attempted:false ok\n"
