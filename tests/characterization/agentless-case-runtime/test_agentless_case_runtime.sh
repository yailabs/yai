#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
source "$ROOT/tests/characterization/lib/governed_case_policy.sh"
YAID="$ROOT/build/yaid"
FIXTURE="$ROOT/tests/fixtures/agentless_case_runtime_provider.py"
TEST_DIR="$(mktemp -d /tmp/yai-agentless-runtime.XXXXXX)"
SOCKET="$TEST_DIR/yaid.sock"
BASE_JOURNAL="$TEST_DIR/base.jsonl"
DAEMON_PID=""
PROVIDER_PIDS=()
LAST_PROVIDER_PORT=""

cleanup() {
  for pid in "${PROVIDER_PIDS[@]}"; do
    kill "$pid" >/dev/null 2>&1 || true
    wait "$pid" >/dev/null 2>&1 || true
  done
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

start_provider() {
  local mode="$1"
  local expected="$2"
  local name="$3"
  local port_file="$TEST_DIR/$name.port"
  YAI_CASE_RUNTIME_PROVIDER_LOG="$TEST_DIR/$name.log.json" \
    python3 "$FIXTURE" "$mode" "$expected" >"$port_file" &
  local pid=$!
  PROVIDER_PIDS+=("$pid")
  for _ in $(seq 1 100); do
    [[ -s "$port_file" ]] && break
    sleep 0.02
  done
  [[ -s "$port_file" ]]
  LAST_PROVIDER_PORT=$(tr -d '[:space:]' <"$port_file")
}

wait_providers() {
  for pid in "${PROVIDER_PIDS[@]}"; do
    wait "$pid"
  done
  PROVIDER_PIDS=()
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

setup_case() {
  local name="$1"
  local provider_id="$2"
  local model="$3"
  local port="$4"
  CASE_HOME="$TEST_DIR/$name/home"
  CASE_JOURNAL="$TEST_DIR/$name/journal.jsonl"
  RESOURCE_ROOT="$TEST_DIR/$name/resource"
  mkdir -p "$CASE_HOME" "$RESOURCE_ROOT/allowed"
  cp "$BASE_JOURNAL" "$CASE_JOURNAL"
  yai_bootstrap_tenant_case "$YAI_BIN" "$CASE_HOME" case:new12-filesystem
  YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case enter \
    --case case:new12-filesystem --subject subject:llm-provider >/dev/null
  YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case attach-provider \
    --case case:new12-filesystem --subject subject:llm-provider \
    --provider-id "$provider_id" \
    --base-url "http://127.0.0.1:$port/v1/chat/completions" \
    --model "$model" >/dev/null
  YAI_HOME="$CASE_HOME" "$YAI_BIN" case attach-filesystem \
    --case case:new12-filesystem --attachment workspace --root "$RESOURCE_ROOT" \
    --allow-prefix allowed --policy-owner subject:policy-pack --max-bytes 256 >/dev/null
  yai_configure_governed_filesystem_case "$YAI_BIN" "$CASE_HOME" \
    case:new12-filesystem "runtime-$name" 1 allow subject:llm-provider >/dev/null
}

run_case() {
  local include_default_invocations=1
  local argument
  for argument in "$@"; do
    if [[ "$argument" == "--max-invocations" ]]; then
      include_default_invocations=0
    fi
  done
  local -a budget_args=(
    --max-operations 30 --max-resident-items 12
    --max-semantic-units 6000 --max-estimated-input-units 200000
  )
  if [[ "$include_default_invocations" -eq 1 ]]; then
    budget_args+=(--max-invocations 30)
  fi
  YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case run \
    --case case:new12-filesystem --subject subject:llm-provider --attachment workspace \
    --prompt "advance the bounded Case from observed reality" \
    "${budget_args[@]}" "$@"
}

# A process crash after the canonical ProviderResult is followed by provider
# and model replacement. Resume normalizes the pending A result, then Provider
# B continues from observed state for a total of 26 governed invocations.
start_provider proposal 1 long-provider-a
port_a="$LAST_PROVIDER_PORT"
setup_case long provider:a model-a "$port_a"
set +e
long_crash=$(run_case --failpoint runtime_after_provider_result 2>&1)
long_code=$?
set -e
[[ "$long_code" -eq 91 ]]
require_text "$long_crash" "case_runtime_crash_injected: runtime_after_provider_result"
wait_providers

start_provider adaptive 25 long-provider-b
port_b="$LAST_PROVIDER_PORT"
YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case attach-provider \
  --case case:new12-filesystem --subject subject:llm-provider \
  --provider-id provider:b --base-url "http://127.0.0.1:$port_b/v1/chat/completions" \
  --model model-b >/dev/null
long_output=$(YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case resume \
  --case case:new12-filesystem)
wait_providers
require_text "$long_output" "runtime_status: Completed"
require_text "$long_output" "invocations: 26"
require_text "$long_output" "operations: 24"
require_text "$long_output" "resident_items:"
require_text "$long_output" "actual_total_tokens:"
[[ "$(cat "$RESOURCE_ROOT/allowed/step-23.txt")" == "runtime step 23" ]]
[[ ! -e "$RESOURCE_ROOT/denied/blocked.txt" ]]
grep -Fq '"model": "model-a"' "$TEST_DIR/long-provider-a.log.json"
grep -Fq '"model": "model-b"' "$TEST_DIR/long-provider-b.log.json"
grep -Fq '"denied": true' "$TEST_DIR/long-provider-b.log.json"
max_entries=$(python3 -c 'import json,sys; print(max(x["entry_count"] for x in json.load(open(sys.argv[1]))))' "$TEST_DIR/long-provider-b.log.json")
[[ "$max_entries" -le 12 ]]

status=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" case status --case case:new12-filesystem)
plan_id=$(sed -n 's/^residency_plan_id: //p' <<<"$status")
plan=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" context inspect --id "$plan_id")
require_text "$plan" "artifact_kind: residency_plan"
require_text "$plan" "max_items: 12"
memory=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" memory list \
  --case case:new12-filesystem --limit 100)
require_text "$memory" "kind:resource_effect"
store_summary=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" store summary)
require_text "$store_summary" "cases_materialized: 1"

# Crash after issued Grant but before PREPARE. The provider stays available;
# resume reconstructs the pending chain from canonical history, performs the
# one controlled effect, then consumes a semantic completion turn.
start_provider proposal 2 grant-restart
port_restart="$LAST_PROVIDER_PORT"
setup_case grant-restart provider:restart model-restart "$port_restart"
set +e
grant_crash=$(run_case --max-invocations 2 --failpoint after_grant_before_prepare 2>&1)
grant_code=$?
set -e
[[ "$grant_code" -eq 84 ]]
require_text "$grant_crash" "controlled_effect_crash_injected: after_grant_before_prepare"
grant_resume=$(YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case resume \
  --case case:new12-filesystem --max-invocations 2)
wait_providers
require_text "$grant_resume" "runtime_status: Completed"
require_text "$grant_resume" "operations: 1"
[[ "$(cat "$RESOURCE_ROOT/allowed/step-00.txt")" == "runtime step 00" ]]

# Crash after the carrier made the real effect visible but before FINALIZE.
# Resume must reconcile the unresolved PREPARE before invoking the provider,
# recognize the intended digest, finalize without a duplicate write, and then
# continue from the newly observed Case reality.
start_provider proposal 2 effect-reconcile
port_reconcile="$LAST_PROVIDER_PORT"
setup_case effect-reconcile provider:reconcile model-reconcile "$port_reconcile"
set +e
effect_crash=$(run_case --max-invocations 2 \
  --failpoint after_effect_before_finalize 2>&1)
effect_code=$?
set -e
[[ "$effect_code" -eq 86 ]]
require_text "$effect_crash" "controlled_effect_crash_injected: after_effect_before_finalize"
[[ "$(cat "$RESOURCE_ROOT/allowed/step-00.txt")" == "runtime step 00" ]]
effect_resume=$(YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case resume \
  --case case:new12-filesystem --max-invocations 2)
wait_providers
require_text "$effect_resume" "reconciliation: EffectObserved"
require_text "$effect_resume" "effect_state: Some(Finalized)"
require_text "$effect_resume" "runtime_status: Completed"
require_text "$effect_resume" "operations: 1"
[[ "$(cat "$RESOURCE_ROOT/allowed/step-00.txt")" == "runtime step 00" ]]

# Crash after FINALIZE with derived memory deliberately removed. Resume repairs
# memory from canonical transitions before the next provider invocation.
start_provider proposal 2 memory-restart
port_memory="$LAST_PROVIDER_PORT"
setup_case memory-restart provider:memory model-memory "$port_memory"
set +e
memory_crash=$(run_case --max-invocations 2 \
  --failpoint runtime_after_finalized_before_memory 2>&1)
memory_code=$?
set -e
[[ "$memory_code" -eq 92 ]]
require_text "$memory_crash" "case_runtime_crash_injected: runtime_after_finalized_before_memory"
memory_resume=$(YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case resume \
  --case case:new12-filesystem --max-invocations 2)
wait_providers
require_text "$memory_resume" "runtime_status: Completed"
memory_after=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" memory list \
  --case case:new12-filesystem --limit 20)
require_text "$memory_after" "kind:resource_effect"

# Malformed provider material remains a ProviderResult but cannot create an
# Operation, Decision, Grant or external effect.
start_provider malformed 1 malformed
port_malformed="$LAST_PROVIDER_PORT"
setup_case malformed provider:malformed model-malformed "$port_malformed"
malformed_output=$(run_case --max-invocations 2)
wait_providers
require_text "$malformed_output" "runtime_status: MalformedProviderResult"
require_text "$malformed_output" "invocations: 1"
require_text "$malformed_output" "operations: 0"
[[ ! -e "$RESOURCE_ROOT/allowed/step-00.txt" ]]

# A transient HTTP failure is retried only within the provider boundary. The
# successful semantic invocation is counted once and no effect is required.
start_provider transient 2 transient
port_transient="$LAST_PROVIDER_PORT"
setup_case transient provider:transient model-transient "$port_transient"
transient_output=$(run_case --max-invocations 1 --max-provider-retries 1 2>&1)
wait_providers
require_text "$transient_output" "provider_retry: 1"
require_text "$transient_output" "runtime_status: Completed"
require_text "$transient_output" "provider_failures: 1"
require_text "$transient_output" "invocations: 1"

# Operator stop is durable run metadata, not Case state. A crash between
# iterations leaves the Case valid; stop is inspectable and explicit resume
# clears that request before continuing from canonical state.
start_provider proposal 2 operator-stop
port_stop="$LAST_PROVIDER_PORT"
setup_case operator-stop provider:stop model-stop "$port_stop"
set +e
between_crash=$(run_case --max-invocations 2 --failpoint runtime_between_iterations 2>&1)
between_code=$?
set -e
[[ "$between_code" -eq 93 ]]
require_text "$between_crash" "case_runtime_crash_injected: runtime_between_iterations"
YAI_HOME="$CASE_HOME" "$YAI_BIN" case stop --case case:new12-filesystem >/dev/null
stopped_status=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" case status --case case:new12-filesystem)
require_text "$stopped_status" "stop_requested: true"
stop_resume=$(YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case resume \
  --case case:new12-filesystem --max-invocations 2)
wait_providers
require_text "$stop_resume" "runtime_status: Completed"
require_text "$stop_resume" "stop_requested: false"

# Invocation and operation budgets stop before an extra provider call/effect
# and leave the Case valid.
start_provider proposal 1 budget
port_budget="$LAST_PROVIDER_PORT"
setup_case budget provider:budget model-budget "$port_budget"
budget_output=$(YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case run \
  --case case:new12-filesystem --subject subject:llm-provider --attachment workspace \
  --prompt "advance once, then stop at the invocation budget" \
  --max-invocations 1 --max-operations 2 --max-resident-items 12 \
  --max-semantic-units 6000 --max-estimated-input-units 20000)
wait_providers
require_text "$budget_output" "runtime_status: InvocationBudgetExhausted"
require_text "$budget_output" "invocations: 1"
require_text "$budget_output" "operations: 1"

start_provider proposal 1 operation-budget
port_operation_budget="$LAST_PROVIDER_PORT"
setup_case operation-budget provider:operation-budget model-operation-budget "$port_operation_budget"
operation_budget_output=$(YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case run \
  --case case:new12-filesystem --subject subject:llm-provider --attachment workspace \
  --prompt "stop before a second operation" \
  --max-invocations 2 --max-operations 1 --max-resident-items 12 \
  --max-semantic-units 6000 --max-estimated-input-units 20000)
wait_providers
require_text "$operation_budget_output" "runtime_status: OperationBudgetExhausted"
require_text "$operation_budget_output" "invocations: 1"
require_text "$operation_budget_output" "operations: 1"

# Impossible mandatory semantic context and provider-input cost limits fail
# before transport. Zero-request fixtures prove no HTTP request was issued.
start_provider complete 0 context-budget
port_context_budget="$LAST_PROVIDER_PORT"
setup_case context-budget provider:context-budget model-context-budget "$port_context_budget"
context_budget_output=$(YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case run \
  --case case:new12-filesystem --subject subject:llm-provider --attachment workspace \
  --prompt "mandatory context cannot fit" \
  --max-invocations 1 --max-operations 1 --max-resident-items 12 \
  --max-semantic-units 1 --max-estimated-input-units 20000)
wait_providers
require_text "$context_budget_output" "runtime_status: ContextBudgetExhausted"
require_text "$context_budget_output" "invocations: 0"

start_provider complete 0 cost-budget
port_cost_budget="$LAST_PROVIDER_PORT"
setup_case cost-budget provider:cost-budget model-cost-budget "$port_cost_budget"
cost_budget_output=$(YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case run \
  --case case:new12-filesystem --subject subject:llm-provider --attachment workspace \
  --prompt "provider input cannot fit" \
  --max-invocations 1 --max-operations 1 --max-resident-items 12 \
  --max-semantic-units 6000 --max-estimated-input-units 1)
wait_providers
require_text "$cost_budget_output" "runtime_status: CostBudgetExhausted"
require_text "$cost_budget_output" "invocations: 0"

printf 'case_runtime:agentless_26_turn_provider_model_replacement ok\n'
printf 'case_runtime:deny_adaptation_and_bounded_residency ok\n'
printf 'case_runtime:grant_effect_and_memory_restart_recovery ok\n'
printf 'case_runtime:malformed_retry_operator_stop ok\n'
printf 'case_runtime:budget_stops_before_extra_invocation ok\n'
