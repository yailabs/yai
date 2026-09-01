#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
YAID="$ROOT/build/yaid"
PROVIDER_FIXTURE="$ROOT/tests/fixtures/agentless_case_runtime_provider.py"
TEST_DIR="$(mktemp -d /tmp/yai-workflow-modelwork.XXXXXX)"
RUNTIME_PID=""
PROVIDER_PID=""
DAEMON_PID=""
SOCKET="$TEST_DIR/yaid.sock"
BASE_JOURNAL="$TEST_DIR/base.jsonl"

# shellcheck source=../lib/governed_case_policy.sh
source "$ROOT/tests/characterization/lib/governed_case_policy.sh"
export YAI_POLICY_EXECUTION_EVIDENCE=0

cleanup() {
  if [[ -n "$RUNTIME_PID" ]] && kill -0 "$RUNTIME_PID" 2>/dev/null; then
    kill "$RUNTIME_PID" 2>/dev/null || true
    wait "$RUNTIME_PID" 2>/dev/null || true
  fi
  if [[ -n "$PROVIDER_PID" ]] && kill -0 "$PROVIDER_PID" 2>/dev/null; then
    kill "$PROVIDER_PID" 2>/dev/null || true
    wait "$PROVIDER_PID" 2>/dev/null || true
  fi
  if [[ -n "$DAEMON_PID" ]] && kill -0 "$DAEMON_PID" 2>/dev/null; then
    "$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null 2>&1 || true
    wait "$DAEMON_PID" 2>/dev/null || true
  fi
  if [[ "${YAI_KEEP_TEST_DIR:-0}" != "1" ]]; then rm -rf "$TEST_DIR"; fi
}
trap cleanup EXIT INT TERM

trace_product() {
  [[ "${YAI_EXECUTION_EVIDENCE:-0}" == "1" ]] || return 0
  printf '\n[product-command:%s]\n$ %s\n%s\nexit: %s\n' "$1" "$2" "$3" "$4"
}

start_provider() {
  local expected="$1"
  local mode="${2:-complete}"
  local port_file="$TEST_DIR/provider.port"
  YAI_CASE_RUNTIME_PROVIDER_LOG="$TEST_DIR/provider.log.json" \
    python3 "$PROVIDER_FIXTURE" "$mode" "$expected" >"$port_file" &
  PROVIDER_PID=$!
  for _ in $(seq 1 100); do
    [[ -s "$port_file" ]] && break
    sleep 0.02
  done
  [[ -s "$port_file" ]]
  PROVIDER_PORT=$(tr -d '[:space:]' <"$port_file")
}

run_runtime_until_terminal() {
  local yai_home="$1"
  local case_id="$2"
  local expected_status="$3"
  YAI_HOME="$yai_home" "$YAI_BIN" runtime serve \
    --workers 2 --max-active-per-tenant 2 --max-queued-per-tenant 8 \
    --max-queued-total 16 >"$TEST_DIR/runtime.log" 2>&1 &
  RUNTIME_PID=$!
  local status=""
  for _ in $(seq 1 200); do
    status=$(YAI_HOME="$yai_home" "$YAI_BIN" workflow status --case "$case_id" 2>/dev/null || true)
    if grep -Fq "$expected_status" <<<"$status"; then break; fi
    if ! kill -0 "$RUNTIME_PID" 2>/dev/null; then
      sed -n '1,240p' "$TEST_DIR/runtime.log" >&2
      exit 1
    fi
    sleep 0.1
  done
  WORKFLOW_STATUS="$status"
}

wait_case_runtime_status() {
  local yai_home="$1"
  local case_id="$2"
  local expected="$3"
  local status=""
  for _ in $(seq 1 200); do
    status=$(YAI_HOME="$yai_home" "$YAI_BIN" case status --case "$case_id" 2>/dev/null || true)
    if grep -Fq "$expected" <<<"$status"; then break; fi
    sleep 0.1
  done
  grep -Fq "$expected" <<<"$status"
  CASE_RUNTIME_STATUS="$status"
}

setup_model_case() {
  local yai_home="$1"
  local tenant_id="$2"
  local case_id="$3"
  local fixture="$4"
  local policy_effect="${5:-none}"
  local case_journal="$yai_home/compat.jsonl"
  export YAI_TEST_TENANT_ID="$tenant_id"
  mkdir -p "$yai_home" "$TEST_DIR/resource/allowed"
  cp "$BASE_JOURNAL" "$case_journal"
  yai_bootstrap_tenant_case "$YAI_BIN" "$yai_home" "$case_id" "$tenant_id" "organization:characterization"
  YAI_HOME="$yai_home" YAI_JOURNAL="$case_journal" "$YAI_BIN" case enter \
    --case "$case_id" --subject subject:llm-provider >/dev/null
  YAI_HOME="$yai_home" YAI_JOURNAL="$case_journal" "$YAI_BIN" case attach-provider \
    --case "$case_id" --subject subject:llm-provider --provider-id provider:fixture \
    --base-url "http://127.0.0.1:$PROVIDER_PORT/v1/chat/completions" \
    --model model:workflow-fixture >/dev/null
  YAI_HOME="$yai_home" "$YAI_BIN" case attach-filesystem \
    --case "$case_id" --attachment workspace --root "$TEST_DIR/resource" \
    --allow-prefix allowed --policy-owner subject:policy-pack --max-bytes 4096 >/dev/null
  yai_configure_governed_filesystem_case "$YAI_BIN" "$yai_home" "$case_id" \
    "workflow-model-policy" 1 "$policy_effect" subject:llm-provider >/dev/null
  local define_output definition_id
  define_output=$(YAI_HOME="$yai_home" "$YAI_BIN" workflow define --tenant "$tenant_id" --file "$fixture")
  definition_id=$(sed -n 's/^workflow_definition_id: //p' <<<"$define_output" | head -1)
  YAI_HOME="$yai_home" "$YAI_BIN" workflow bind --case "$case_id" \
    --definition "$definition_id" --executor analyst=subject:llm-provider \
    --resource workspace=workspace >/dev/null
  printf '%s' "$definition_id"
}

mkdir -p "$TEST_DIR/daemon-home"
HOME="$TEST_DIR/daemon-home" "$YAID" --socket "$SOCKET" --foreground \
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

# A cognitive ModelWork finishes from its exact execution-scoped ProviderResult.
start_provider 1
MODEL_HOME="$TEST_DIR/model-home"
model_definition=$(setup_model_case "$MODEL_HOME" tenant:workflow-model case:new12-filesystem \
  "$ROOT/tests/fixtures/workflows/model-analysis.v1.json")
run_runtime_until_terminal "$MODEL_HOME" case:new12-filesystem "completed: true"
model_status="$WORKFLOW_STATUS"
grep -Fq "completed: true" <<<"$model_status"
grep -Fq "posture=Satisfied" <<<"$model_status"
model_case_status=$(YAI_HOME="$MODEL_HOME" "$YAI_BIN" case status --case case:new12-filesystem)
grep -Fq "invocations: 1" <<<"$model_case_status"
grep -Fq "operations: 0" <<<"$model_case_status"
stop_output=$(YAI_HOME="$MODEL_HOME" "$YAI_BIN" runtime stop)
wait "$RUNTIME_PID"
RUNTIME_PID=""
wait "$PROVIDER_PID"
PROVIDER_PID=""
grep -Fq '"model": "model:workflow-fixture"' "$TEST_DIR/provider.log.json"
grep -Fq '"frame_id": "context-frame:' "$TEST_DIR/provider.log.json"
trace_product 01 "./yai workflow status --case case:new12-filesystem" "$model_status" 0
trace_product 02 "./yai case status --case case:new12-filesystem" "$model_case_status" 0
trace_product 03 "./yai runtime stop" "$stop_output" 0

# One ModelWork execution performs two governed turns. The second provider
# request must observe the first finalized effect before it can produce the
# exact target effect that satisfies the node.
start_provider 2 adaptive
ITERATIVE_HOME="$TEST_DIR/iterative-home"
iterative_definition=$(setup_model_case "$ITERATIVE_HOME" tenant:workflow-iterative \
  case:new12-filesystem \
  "$ROOT/tests/fixtures/workflows/model-iterative-effect.v1.json" allow)
run_runtime_until_terminal "$ITERATIVE_HOME" case:new12-filesystem "completed: true"
iterative_status="$WORKFLOW_STATUS"
iterative_case_status=$(YAI_HOME="$ITERATIVE_HOME" "$YAI_BIN" case status \
  --case case:new12-filesystem)
grep -Fq "completed: true" <<<"$iterative_status"
grep -Fq "posture=Satisfied" <<<"$iterative_status"
grep -Fq "invocations: 2" <<<"$iterative_case_status"
grep -Fq "operations: 2" <<<"$iterative_case_status"
[[ "$(<"$TEST_DIR/resource/allowed/step-00.txt")" == "runtime step 00" ]]
[[ "$(<"$TEST_DIR/resource/allowed/step-01.txt")" == "runtime step 01" ]]
stop_output=$(YAI_HOME="$ITERATIVE_HOME" "$YAI_BIN" runtime stop)
wait "$RUNTIME_PID"
RUNTIME_PID=""
wait "$PROVIDER_PID"
PROVIDER_PID=""
grep -Fq '"step": -1' "$TEST_DIR/provider.log.json"
grep -Fq '"step": 0' "$TEST_DIR/provider.log.json"
trace_product 04 "./yai workflow status --case case:new12-filesystem" \
  "$iterative_status" 0
trace_product 05 "./yai case status --case case:new12-filesystem" \
  "$iterative_case_status" 0
trace_product 06 "./yai runtime stop" "$stop_output" 0

# A crash after an unsatisfying first ProviderResult must resume by consuming
# that canonical result, then issue only the next required turn. The persisted
# failpoint is deliberately one-shot for the WorkItem execution.
start_provider 2 adaptive
UNSAT_CRASH_HOME="$TEST_DIR/unsatisfying-crash-home"
unsatisfying_crash_definition=$(setup_model_case "$UNSAT_CRASH_HOME" \
  tenant:workflow-iterative case:new12-filesystem \
  "$ROOT/tests/fixtures/workflows/model-iterative-effect.v1.json" allow)
YAI_HOME="$UNSAT_CRASH_HOME" "$YAI_BIN" runtime serve \
  --workers 1 --max-active-per-tenant 1 --max-queued-per-tenant 4 \
  --max-queued-total 4 \
  --workflow-work-failpoint runtime_after_provider_result \
  >"$TEST_DIR/unsatisfying-crash-runtime.log" 2>&1 &
RUNTIME_PID=$!
set +e
wait "$RUNTIME_PID"
unsatisfying_crash_exit=$?
set -e
RUNTIME_PID=""
[[ "$unsatisfying_crash_exit" == "91" ]]
unsatisfying_crash_status=$(YAI_HOME="$UNSAT_CRASH_HOME" "$YAI_BIN" workflow status \
  --case case:new12-filesystem)
grep -Fq "completed: false" <<<"$unsatisfying_crash_status"
grep -Fq "posture=Active reason=workflow_execution_active" \
  <<<"$unsatisfying_crash_status"
trace_product 07 "./yai runtime serve --workflow-work-failpoint runtime_after_provider_result" \
  "$(sed -n '1,160p' "$TEST_DIR/unsatisfying-crash-runtime.log")" \
  "$unsatisfying_crash_exit"

run_runtime_until_terminal "$UNSAT_CRASH_HOME" case:new12-filesystem "completed: true"
unsatisfying_recovered_status="$WORKFLOW_STATUS"
unsatisfying_recovered_case=$(YAI_HOME="$UNSAT_CRASH_HOME" "$YAI_BIN" case status \
  --case case:new12-filesystem)
grep -Fq "invocations: 2" <<<"$unsatisfying_recovered_case"
grep -Fq "operations: 2" <<<"$unsatisfying_recovered_case"
stop_output=$(YAI_HOME="$UNSAT_CRASH_HOME" "$YAI_BIN" runtime stop)
wait "$RUNTIME_PID"
RUNTIME_PID=""
wait "$PROVIDER_PID"
PROVIDER_PID=""
[[ "$(jq 'length' "$TEST_DIR/provider.log.json")" == "2" ]]
trace_product 08 "./yai workflow status --case case:new12-filesystem" \
  "$unsatisfying_recovered_status" 0
trace_product 09 "./yai case status --case case:new12-filesystem" \
  "$unsatisfying_recovered_case" 0

# Two byte-identical provider outputs retain distinct invocation/result
# identities. Neither textual completion satisfies an effect predicate.
start_provider 2 complete
REPEATED_HOME="$TEST_DIR/repeated-output-home"
repeated_definition=$(setup_model_case "$REPEATED_HOME" \
  tenant:workflow-repeated-output case:new12-filesystem \
  "$ROOT/tests/fixtures/workflows/repeated-identical-model-output.v1.json")
run_runtime_until_terminal "$REPEATED_HOME" case:new12-filesystem \
  "reason=workflow_execution_active"
repeated_status="$WORKFLOW_STATUS"
wait_case_runtime_status "$REPEATED_HOME" case:new12-filesystem \
  "runtime_status: InvocationBudgetExhausted"
repeated_case_status="$CASE_RUNTIME_STATUS"
grep -Fq "invocations: 2" <<<"$repeated_case_status"
grep -Fq "operations: 0" <<<"$repeated_case_status"
grep -Fq "completed: false" <<<"$repeated_status"
stop_output=$(YAI_HOME="$REPEATED_HOME" "$YAI_BIN" runtime stop)
wait "$RUNTIME_PID"
RUNTIME_PID=""
wait "$PROVIDER_PID"
PROVIDER_PID=""
[[ "$(jq 'length' "$TEST_DIR/provider.log.json")" == "2" ]]
[[ "$(jq '[.[].frame_id] | unique | length' "$TEST_DIR/provider.log.json")" == "2" ]]
trace_product 10 "./yai workflow status --case case:new12-filesystem" \
  "$repeated_status" 0
trace_product 11 "./yai case status --case case:new12-filesystem" \
  "$repeated_case_status" 0

# A model's typed completion claim cannot satisfy an effect-required node.
start_provider 1
FALSE_HOME="$TEST_DIR/false-home"
false_definition=$(setup_model_case "$FALSE_HOME" tenant:workflow-false-completion \
  case:new12-filesystem \
  "$ROOT/tests/fixtures/workflows/false-effect-completion.v1.json")
run_runtime_until_terminal "$FALSE_HOME" case:new12-filesystem \
  "reason=workflow_execution_active"
false_status="$WORKFLOW_STATUS"
grep -Fq "completed: false" <<<"$false_status"
grep -Fq "posture=Active reason=workflow_execution_active" <<<"$false_status"
wait_case_runtime_status "$FALSE_HOME" case:new12-filesystem \
  "runtime_status: InvocationBudgetExhausted"
false_case_status="$CASE_RUNTIME_STATUS"
grep -Fq "runtime_status: InvocationBudgetExhausted" <<<"$false_case_status"
grep -Fq "invocations: 1" <<<"$false_case_status"
grep -Fq "operations: 0" <<<"$false_case_status"
stop_output=$(YAI_HOME="$FALSE_HOME" "$YAI_BIN" runtime stop)
wait "$RUNTIME_PID"
RUNTIME_PID=""
wait "$PROVIDER_PID"
PROVIDER_PID=""
trace_product 12 "./yai workflow status --case case:new12-filesystem" "$false_status" 0
trace_product 13 "./yai case status --case case:new12-filesystem" "$false_case_status" 0
trace_product 14 "./yai runtime stop" "$stop_output" 0

# ProviderResult is canonical, but the RuntimeInstance dies before the
# workflow satisfaction writer runs. Restart must consume the same result and
# must not contact the now-stopped provider a second time.
start_provider 1
CRASH_HOME="$TEST_DIR/crash-home"
crash_definition=$(setup_model_case "$CRASH_HOME" tenant:workflow-model \
  case:new12-filesystem \
  "$ROOT/tests/fixtures/workflows/model-analysis.v1.json")
YAI_HOME="$CRASH_HOME" "$YAI_BIN" runtime serve \
  --workers 1 --max-active-per-tenant 1 --max-queued-per-tenant 4 \
  --max-queued-total 4 \
  --workflow-work-failpoint runtime_after_provider_result \
  >"$TEST_DIR/crash-runtime.log" 2>&1 &
RUNTIME_PID=$!
set +e
wait "$RUNTIME_PID"
crash_exit=$?
set -e
RUNTIME_PID=""
[[ "$crash_exit" == "91" ]]
wait "$PROVIDER_PID"
PROVIDER_PID=""
crash_status=$(YAI_HOME="$CRASH_HOME" "$YAI_BIN" workflow status \
  --case case:new12-filesystem)
grep -Fq "completed: false" <<<"$crash_status"
grep -Fq "reason=completion_proven_pending_canonical_satisfaction" <<<"$crash_status"
trace_product 15 "./yai runtime serve --workflow-work-failpoint runtime_after_provider_result" \
  "$(sed -n '1,160p' "$TEST_DIR/crash-runtime.log")" "$crash_exit"
trace_product 16 "./yai workflow status --case case:new12-filesystem" "$crash_status" 0

run_runtime_until_terminal "$CRASH_HOME" case:new12-filesystem "completed: true"
recovered_status="$WORKFLOW_STATUS"
recovered_case_status=$(YAI_HOME="$CRASH_HOME" "$YAI_BIN" case status \
  --case case:new12-filesystem)
recovered_queue=$(YAI_HOME="$CRASH_HOME" "$YAI_BIN" runtime queue --all)
grep -Fq "state: Completed" <<<"$recovered_queue"
grep -Fq "stop_reason: canonical_workflow_satisfaction_recovered" <<<"$recovered_queue"
grep -Fq "runtime_status: Completed" <<<"$recovered_case_status"
grep -Fq "canonical workflow satisfaction recovered without Case re-execution" \
  <<<"$recovered_case_status"
grep -Fq "runtime_admission_status: none" <<<"$recovered_case_status"
grep -Fq "invocations: 1" <<<"$recovered_case_status"
grep -Fq "operations: 0" <<<"$recovered_case_status"
stop_output=$(YAI_HOME="$CRASH_HOME" "$YAI_BIN" runtime stop)
wait "$RUNTIME_PID"
RUNTIME_PID=""
trace_product 17 "./yai workflow status --case case:new12-filesystem" "$recovered_status" 0
trace_product 18 "./yai case status --case case:new12-filesystem" "$recovered_case_status" 0
trace_product 19 "./yai runtime queue --all" "$recovered_queue" 0
trace_product 20 "./yai runtime stop" "$stop_output" 0

printf 'workflow_modelwork_characterization: pass\n'
printf 'model_definition_id: %s\n' "$model_definition"
printf 'iterative_definition_id: %s\n' "$iterative_definition"
printf 'false_completion_definition_id: %s\n' "$false_definition"
printf 'crash_recovery_definition_id: %s\n' "$crash_definition"
printf 'modelwork_provider_invocations: 1\n'
printf 'iterative_modelwork_provider_invocations: 2\n'
printf 'iterative_modelwork_operations: 2\n'
printf 'second_turn_observed_first_effect: true\n'
printf 'unsatisfying_provider_result_crash_exit: %s\n' "$unsatisfying_crash_exit"
printf 'unsatisfying_provider_result_recovery_invocations: 2\n'
printf 'unsatisfying_provider_result_duplicate_prior_turns: 0\n'
printf 'repeated_identical_output_invocations: 2\n'
printf 'repeated_identical_output_distinct_results: 2\n'
printf 'repeated_identical_output_node_satisfied: false\n'
printf 'false_completion_provider_invocations: 1\n'
printf 'false_completion_node_satisfied: false\n'
printf 'provider_result_crash_exit: %s\n' "$crash_exit"
printf 'provider_result_recovery_invocations: 1\n'
printf 'provider_result_recovery_duplicate_calls: 0\n'
