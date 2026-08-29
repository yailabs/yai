#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
source "$ROOT/tests/characterization/lib/governed_case_policy.sh"
YAID="$ROOT/build/yaid"
FIXTURE="$ROOT/tests/fixtures/controlled_effect_provider.py"
TEST_DIR="$(mktemp -d /tmp/yai-controlled-effect.XXXXXX)"
SOCKET="$TEST_DIR/yaid.sock"
BASE_JOURNAL="$TEST_DIR/base.jsonl"
DAEMON_PID=""
PROVIDER_PID=""

cleanup() {
  if [[ -n "$PROVIDER_PID" ]]; then
    kill "$PROVIDER_PID" >/dev/null 2>&1 || true
    wait "$PROVIDER_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "$DAEMON_PID" ]]; then
    "$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null 2>&1 || true
    wait "$DAEMON_PID" >/dev/null 2>&1 || true
  fi
  rm -rf "$TEST_DIR"
}
trap cleanup EXIT INT TERM

require_text() {
  grep -Fq -- "$2" <<<"$1"
}

mkdir -p "$TEST_DIR/daemon-user"
HOME="$TEST_DIR/daemon-user" "$YAID" --socket "$SOCKET" --foreground \
  >"$TEST_DIR/yaid.log" 2>&1 &
DAEMON_PID=$!
for _ in $(seq 1 50); do
  [[ -S "$SOCKET" ]] && break
  sleep 0.02
done
[[ -S "$SOCKET" ]]
loop_output=$("$YAI_BIN" daemon run-filesystem-loop --socket "$SOCKET")
source_journal=$(sed -n 's/.*"journal_path":"\([^"]*\)".*/\1/p' <<<"$loop_output")
cp "$ROOT/$source_journal" "$BASE_JOURNAL"
"$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null
wait "$DAEMON_PID"
DAEMON_PID=""

start_provider() {
  local scenario="$1"
  local port_file="$CASE_DIR/provider.port"
  python3 "$FIXTURE" "$scenario" >"$port_file" &
  PROVIDER_PID=$!
  for _ in $(seq 1 100); do
    [[ -s "$port_file" ]] && break
    sleep 0.02
  done
  [[ -s "$port_file" ]]
  PROVIDER_PORT=$(tr -d '[:space:]' <"$port_file")
}

wait_provider() {
  wait "$PROVIDER_PID"
  PROVIDER_PID=""
}

setup_case() {
  local name="$1"
  local scenario="$2"
  local max_bytes="${3:-128}"
  CASE_DIR="$TEST_DIR/$name"
  CASE_HOME="$CASE_DIR/home"
  CASE_JOURNAL="$CASE_DIR/journal.jsonl"
  RESOURCE_ROOT="$CASE_DIR/resource"
  mkdir -p "$CASE_HOME" "$RESOURCE_ROOT/allowed"
  cp "$BASE_JOURNAL" "$CASE_JOURNAL"
  start_provider "$scenario"
  YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case enter \
    --case case:new12-filesystem --subject subject:llm-provider >/dev/null
  YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case attach-provider \
    --case case:new12-filesystem \
    --subject subject:llm-provider \
    --base-url "http://127.0.0.1:$PROVIDER_PORT/v1/chat/completions" \
    --model controlled-model >/dev/null
  YAI_HOME="$CASE_HOME" "$YAI_BIN" case attach-filesystem \
    --case case:new12-filesystem \
    --attachment workspace \
    --root "$RESOURCE_ROOT" \
    --allow-prefix allowed \
    --policy-owner subject:policy-pack \
    --max-bytes "$max_bytes" >/dev/null
  yai_configure_governed_filesystem_case "$YAI_BIN" "$CASE_HOME" \
    case:new12-filesystem "controlled-$name" 1 allow subject:llm-provider >/dev/null
}

run_effect() {
  YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" effect filesystem-write \
    --case case:new12-filesystem \
    --subject subject:llm-provider \
    --attachment workspace \
    --prompt "propose the controlled test write" \
    --base-url "http://127.0.0.1:$PROVIDER_PORT/v1/chat/completions" \
    --model controlled-model "$@"
}

# ALLOW: provider transport, controlled effect and the second provider turn are
# all product-owned. The injected derived failure must not affect authority.
setup_case allow allow
allow_output=$(run_effect --inject-derived-failure 2>&1)
wait_provider
require_text "$allow_output" "decision: allow"
require_text "$allow_output" "effect_state: prepared_durable_before_mutation"
require_text "$allow_output" "effect_outcome: Applied"
require_text "$allow_output" "derived_update: injected_failure_canonical_state_preserved"
require_text "$allow_output" "second_turn_consequence: observed_reality_from_canonical_state"
require_text "$allow_output" "effect_chain_closure: valid"
[[ "$(cat "$RESOURCE_ROOT/allowed/hello.txt")" == "hello from controlled YAI" ]]
allow_effect=$(sed -n 's/^effect_id: //p' <<<"$allow_output" | head -1)
inspect=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" effect inspect \
  --case case:new12-filesystem --effect "$allow_effect")
require_text "$inspect" "closure: valid"
duplicate=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" effect reconcile \
  --case case:new12-filesystem --effect "$allow_effect")
require_text "$duplicate" "reconciliation: already_finalized"

# DENY: the provider can claim/propose a write but cannot produce authority.
setup_case deny deny
deny_output=$(run_effect)
wait_provider
require_text "$deny_output" "decision: deny"
require_text "$deny_output" "execution_grant: none"
require_text "$deny_output" "second_turn_consequence: committed_denial_no_effect"
[[ ! -e "$RESOURCE_ROOT/denied/hello.txt" ]]

# Candidate failures preserve ProviderResult but create neither Operation nor
# external effect.
for scenario in malformed claim_only traversal absolute wrong_attachment oversized; do
  setup_case "$scenario" "$scenario" 128
  output=$(run_effect)
  wait_provider
  require_text "$output" "provider_result_authority: non_authoritative_candidate_material"
  require_text "$output" "operation_normalization: rejected"
  require_text "$output" "external_effect: none"
  [[ ! -e "$RESOURCE_ROOT/allowed/hello.txt" ]]
done

# Lexical validation is supplemented by real parent canonicalization so a
# symlink cannot escape the attachment root.
setup_case symlink symlink_escape
mkdir -p "$CASE_DIR/outside"
ln -s "$CASE_DIR/outside" "$RESOURCE_ROOT/allowed/link"
set +e
symlink_output=$(run_effect 2>&1)
symlink_status=$?
set -e
wait_provider
[[ "$symlink_status" -eq 2 ]]
require_text "$symlink_output" "pre_effect_observation_unavailable"
[[ ! -e "$CASE_DIR/outside/escape.txt" ]]

# A deterministic carrier failure is finalized as an observed no-effect, not
# invented success, and is visible to the second provider turn.
setup_case carrier_failure carrier_failure
failure_output=$(run_effect --failpoint carrier_failure)
wait_provider
require_text "$failure_output" "effect_outcome: FailedNoEffect"
require_text "$failure_output" "second_turn_consequence: observed_reality_from_canonical_state"
[[ ! -e "$RESOURCE_ROOT/allowed/hello.txt" ]]

# Crash after PREPARE but before mutation: restart reconciliation observes the
# pre-state and an explicit retry consumes the same prepared authority once.
setup_case crash_before_prepare allow_once
set +e
run_effect --failpoint after_grant_before_prepare >"$CASE_DIR/crash.out" 2>&1
crash_status=$?
set -e
wait_provider
[[ "$crash_status" -eq 84 ]]
require_text "$(cat "$CASE_DIR/crash.out")" "execution_grant_id: grant:"
[[ ! -e "$RESOURCE_ROOT/allowed/hello.txt" ]]
set +e
unprepared_reconcile=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" effect reconcile \
  --case case:new12-filesystem 2>&1)
unprepared_status=$?
set -e
[[ "$unprepared_status" -eq 2 ]]
require_text "$unprepared_reconcile" "no matching prepared or finalized effect"

setup_case crash_before allow_once
set +e
run_effect --failpoint after_prepare_before_effect >"$CASE_DIR/crash.out" 2>&1
crash_status=$?
set -e
wait_provider
[[ "$crash_status" -eq 85 ]]
crash_effect=$(sed -n 's/^effect_id: //p' "$CASE_DIR/crash.out" | head -1)
[[ ! -e "$RESOURCE_ROOT/allowed/hello.txt" ]]
retry_output=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" effect reconcile \
  --case case:new12-filesystem --effect "$crash_effect" --retry)
require_text "$retry_output" "reconciliation: EffectObserved"
[[ "$(cat "$RESOURCE_ROOT/allowed/hello.txt")" == "hello from controlled YAI" ]]

# Crash after the rename is visible but before FINALIZE: reconciliation proves
# the intended post-state and finalizes without invoking the carrier again.
setup_case crash_after allow_once
set +e
run_effect --failpoint after_effect_before_finalize >"$CASE_DIR/crash.out" 2>&1
crash_status=$?
set -e
wait_provider
[[ "$crash_status" -eq 86 ]]
crash_effect=$(sed -n 's/^effect_id: //p' "$CASE_DIR/crash.out" | head -1)
[[ "$(cat "$RESOURCE_ROOT/allowed/hello.txt")" == "hello from controlled YAI" ]]
reconciled=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" effect reconcile \
  --case case:new12-filesystem --effect "$crash_effect")
require_text "$reconciled" "reconciliation: EffectObserved"
inspect=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" effect inspect \
  --case case:new12-filesystem --effect "$crash_effect")
require_text "$inspect" "closure: valid"

# Receipt construction is process-local until FINALIZE; a crash at that seam
# still recovers from durable PREPARE plus the real post-state.
setup_case crash_after_receipt allow_once
set +e
run_effect --failpoint after_receipt_before_finalize >"$CASE_DIR/crash.out" 2>&1
crash_status=$?
set -e
wait_provider
[[ "$crash_status" -eq 87 ]]
crash_effect=$(sed -n 's/^effect_id: //p' "$CASE_DIR/crash.out" | head -1)
[[ "$(cat "$RESOURCE_ROOT/allowed/hello.txt")" == "hello from controlled YAI" ]]
reconciled=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" effect reconcile \
  --case case:new12-filesystem --effect "$crash_effect")
require_text "$reconciled" "reconciliation: EffectObserved"

# A third-party conflicting state is neither pre-state nor intended post-state;
# reconciliation retains explicit uncertainty.
setup_case conflict allow_once
set +e
run_effect --failpoint after_prepare_before_effect >"$CASE_DIR/crash.out" 2>&1
crash_status=$?
set -e
wait_provider
[[ "$crash_status" -eq 85 ]]
crash_effect=$(sed -n 's/^effect_id: //p' "$CASE_DIR/crash.out" | head -1)
printf '%s\n' conflict >"$RESOURCE_ROOT/allowed/hello.txt"
conflict_output=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" effect reconcile \
  --case case:new12-filesystem --effect "$crash_effect")
require_text "$conflict_output" "reconciliation: Conflict"
inspect=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" effect inspect \
  --case case:new12-filesystem --effect "$crash_effect")
require_text "$inspect" "closure: unresolved"

printf 'controlled_effect:allow_deny_second_turn ok\n'
printf 'controlled_effect:normalization_security ok\n'
printf 'controlled_effect:prepare_crash_reconciliation ok\n'
printf 'controlled_effect:conflict_indeterminate ok\n'
printf 'controlled_effect:derived_failure_isolation ok\n'
