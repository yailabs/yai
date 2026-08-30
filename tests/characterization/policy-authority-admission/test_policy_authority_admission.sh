#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
YAID="$ROOT/build/yaid"
FIXTURE="$ROOT/tests/fixtures/controlled_effect_provider.py"
source "$ROOT/tests/characterization/lib/governed_case_policy.sh"
TEST_DIR="$(mktemp -d /tmp/yai-policy-admission.XXXXXX)"
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
  if [[ "${YAI_KEEP_TEST_DIR:-0}" != "1" ]]; then
    rm -rf "$TEST_DIR"
  else
    printf 'preserved_test_dir:%s\n' "$TEST_DIR" >&2
  fi
}
trap cleanup EXIT INT TERM

require_text() {
  grep -Fq -- "$2" <<<"$1"
}

trace_product() {
  [[ "${YAI_EXECUTION_EVIDENCE:-0}" == "1" ]] || return 0
  if [[ "${YAI_EVIDENCE_COMPACT:-0}" == "1" ]]; then
    local bounded
    bounded=$(grep -E '^(principal_id|tenant_id|policy_(ingest|validate|publish)|artifact_id|policy_lineage_id|lifecycle|runtime_consumable|case_policy_bind|binding_id|normative_readiness|policy_validity|effective_policy_id|provider_invocation_id|provider_result_id|operation_id|decision_id|decision_reason|decision_basis_id|decision|execution_grant_id|execution_grant|grant_id|grant_issued_at|grant_expires_at|prepared_effect_id|effect_id|receipt_id|effect_receipt_id|effect_lifecycle|external_effect|second_provider_invocation_id|second_provider_result_id|second_turn_consequence|provider_invocations|execution_grants):' <<<"$3" || true)
    printf '\n[product-command:%s]\n$ %s\n%s\nexit: %s\n' \
      "$1" "$2" "$bounded" "$4" >&2
  else
    printf '\n[product-command:%s]\n$ %s\n%s\nexit: %s\n' \
      "$1" "$2" "$3" "$4" >&2
  fi
}

mkdir -p "$TEST_DIR/daemon-user"
HOME="$TEST_DIR/daemon-user" "$YAID" --socket "$SOCKET" --foreground \
  >"$TEST_DIR/yaid.log" 2>&1 &
DAEMON_PID=$!
for _ in $(seq 1 100); do
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

setup_case() {
  local name="$1"
  local policy_effect="$2"
  local fixture_mode="$3"
  CASE_HOME="$TEST_DIR/$name/home"
  CASE_JOURNAL="$TEST_DIR/$name/journal.jsonl"
  RESOURCE_ROOT="$TEST_DIR/$name/resource"
  mkdir -p "$CASE_HOME" "$RESOURCE_ROOT/allowed"
  cp "$BASE_JOURNAL" "$CASE_JOURNAL"
  local port_file="$TEST_DIR/$name/provider.port"
  python3 "$FIXTURE" "$fixture_mode" >"$port_file" &
  PROVIDER_PID=$!
  for _ in $(seq 1 100); do
    [[ -s "$port_file" ]] && break
    sleep 0.02
  done
  PROVIDER_PORT=$(tr -d '[:space:]' <"$port_file")
  yai_bootstrap_tenant_case "$YAI_BIN" "$CASE_HOME" case:new12-filesystem
  YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case enter \
    --case case:new12-filesystem --subject subject:llm-provider >/dev/null
  YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case attach-provider \
    --case case:new12-filesystem --subject subject:llm-provider \
    --provider-id "provider:$name" \
    --base-url "http://127.0.0.1:$PROVIDER_PORT/v1/chat/completions" \
    --model controlled-model >/dev/null
  YAI_HOME="$CASE_HOME" "$YAI_BIN" case attach-filesystem \
    --case case:new12-filesystem --attachment workspace --root "$RESOURCE_ROOT" \
    --allow-prefix allowed --policy-owner subject:policy-pack --max-bytes 256 >/dev/null
  yai_configure_governed_filesystem_case "$YAI_BIN" "$CASE_HOME" \
    case:new12-filesystem "authority-$name" 1 "$policy_effect" \
    subject:llm-provider >/dev/null
}

run_effect() {
  YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" effect filesystem-write \
    --case case:new12-filesystem --subject subject:llm-provider --attachment workspace \
    --prompt "propose one policy-governed write" \
    --provider-id "provider:$1" \
    --base-url "http://127.0.0.1:$PROVIDER_PORT/v1/chat/completions" \
    --model controlled-model
}

setup_case allow allow allow
allow_output=$(run_effect allow)
trace_product 08 "YAI_HOME=$CASE_HOME YAI_JOURNAL=$CASE_JOURNAL $YAI_BIN effect filesystem-write --case case:new12-filesystem --subject subject:llm-provider --attachment workspace --prompt 'propose one policy-governed write' --provider-id provider:allow --base-url http://127.0.0.1:$PROVIDER_PORT/v1/chat/completions --model controlled-model" "$allow_output" 0
wait "$PROVIDER_PID"
PROVIDER_PID=""
require_text "$allow_output" "decision_basis_id: decision-basis:"
require_text "$allow_output" "decision: allow"
require_text "$allow_output" "execution_grant_decision_basis_id: decision-basis:"
require_text "$allow_output" "effect_state: finalized"
[[ "$(cat "$RESOURCE_ROOT/allowed/hello.txt")" == "hello from controlled YAI" ]]

setup_case deny deny policy_deny
deny_output=$(run_effect deny)
trace_product 09 "YAI_HOME=$CASE_HOME YAI_JOURNAL=$CASE_JOURNAL $YAI_BIN effect filesystem-write --case case:new12-filesystem --subject subject:llm-provider --attachment workspace --prompt 'propose one policy-governed write' --provider-id provider:deny --base-url http://127.0.0.1:$PROVIDER_PORT/v1/chat/completions --model controlled-model" "$deny_output" 0
wait "$PROVIDER_PID"
PROVIDER_PID=""
require_text "$deny_output" "decision: deny"
require_text "$deny_output" "execution_grant: none"
require_text "$deny_output" "applicable_policy_deny"
[[ ! -e "$RESOURCE_ROOT/allowed/hello.txt" ]]

setup_case no-match none no_match
no_match_output=$(run_effect no-match)
trace_product 10 "YAI_HOME=$CASE_HOME YAI_JOURNAL=$CASE_JOURNAL $YAI_BIN effect filesystem-write --case case:new12-filesystem --subject subject:llm-provider --attachment workspace --prompt 'propose one policy-governed write' --provider-id provider:no-match --base-url http://127.0.0.1:$PROVIDER_PORT/v1/chat/completions --model controlled-model" "$no_match_output" 0
wait "$PROVIDER_PID"
PROVIDER_PID=""
require_text "$no_match_output" "decision: deny"
require_text "$no_match_output" "no_applicable_allow_rule"
require_text "$no_match_output" "execution_grant: none"
[[ ! -e "$RESOURCE_ROOT/allowed/hello.txt" ]]

# Readiness is checked before transport. Port 1 is deliberately unreachable;
# the product must still report zero invocations instead of attempting it.
CASE_HOME="$TEST_DIR/unconfigured/home"
CASE_JOURNAL="$TEST_DIR/unconfigured/journal.jsonl"
RESOURCE_ROOT="$TEST_DIR/unconfigured/resource"
mkdir -p "$CASE_HOME" "$RESOURCE_ROOT/allowed"
cp "$BASE_JOURNAL" "$CASE_JOURNAL"
yai_bootstrap_tenant_case "$YAI_BIN" "$CASE_HOME" case:new12-filesystem
YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case enter \
  --case case:new12-filesystem --subject subject:llm-provider >/dev/null
YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case attach-provider \
  --case case:new12-filesystem --subject subject:llm-provider --provider-id provider:none \
  --base-url http://127.0.0.1:1/v1/chat/completions --model controlled-model >/dev/null
YAI_HOME="$CASE_HOME" "$YAI_BIN" case attach-filesystem \
  --case case:new12-filesystem --attachment workspace --root "$RESOURCE_ROOT" \
  --allow-prefix allowed --policy-owner subject:policy-pack --max-bytes 256 >/dev/null
set +e
unconfigured_output=$(YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" \
  "$YAI_BIN" effect filesystem-write --case case:new12-filesystem \
  --subject subject:llm-provider --attachment workspace --prompt "must not invoke" \
  --base-url http://127.0.0.1:1/v1/chat/completions --model controlled-model 2>&1)
unconfigured_exit=$?
set -e
[[ "$unconfigured_exit" -ne 0 ]]
trace_product 11 "YAI_HOME=$CASE_HOME YAI_JOURNAL=$CASE_JOURNAL $YAI_BIN effect filesystem-write --case case:new12-filesystem --subject subject:llm-provider --attachment workspace --prompt 'must not invoke' --base-url http://127.0.0.1:1/v1/chat/completions --model controlled-model" "$unconfigured_output" "$unconfigured_exit"
require_text "$unconfigured_output" "normative_readiness: Unconfigured"
require_text "$unconfigured_output" "provider_invocations: 0"

printf 'policy_authority:allow_chain ok\n'
printf 'policy_authority:explicit_deny_and_no_match_fail_closed ok\n'
printf 'policy_authority:unconfigured_pre_provider_stop ok\n'
