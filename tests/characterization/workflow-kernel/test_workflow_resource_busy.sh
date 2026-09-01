#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
YAID="$ROOT/build/yaid"
PROVIDER_FIXTURE="$ROOT/tests/fixtures/controlled_effect_provider.py"
TEST_DIR="$(mktemp -d /tmp/yai-workflow-resource-busy.XXXXXX)"
export YAI_HOME="$TEST_DIR/home"
export YAI_TEST_TENANT_ID="tenant:wave15-busy"
export YAI_POLICY_EXECUTION_EVIDENCE=0
SHARED_ROOT="$TEST_DIR/shared"
SOCKET="$TEST_DIR/yaid.sock"
RUNTIME_PID=""
PROVIDER_PID=""
DAEMON_PID=""

# shellcheck source=../lib/governed_case_policy.sh
source "$ROOT/tests/characterization/lib/governed_case_policy.sh"

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

mkdir -p "$YAI_HOME" "$TEST_DIR/daemon-home" "$TEST_DIR/seed-home" \
  "$SHARED_ROOT/allowed"
HOME="$TEST_DIR/daemon-home" "$YAID" --socket "$SOCKET" --foreground \
  >"$TEST_DIR/yaid.log" 2>&1 &
DAEMON_PID=$!
for _ in $(seq 1 100); do [[ -S "$SOCKET" ]] && break; sleep 0.02; done
[[ -S "$SOCKET" ]]
loop_output=$(YAI_HOME="$TEST_DIR/seed-home" "$YAI_BIN" daemon run-filesystem-loop \
  --socket "$SOCKET")
source_journal=$(sed -n 's/.*"journal_path":"\([^"]*\)".*/\1/p' <<<"$loop_output")
sed -e 's/case:new12-filesystem/case:wave15-holder/g' \
  -e 's/new12-fs/w15-holder/g' "$ROOT/$source_journal" >"$TEST_DIR/holder.jsonl"
"$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null
wait "$DAEMON_PID"
DAEMON_PID=""

"$YAI_BIN" security bootstrap-local --tenant "$YAI_TEST_TENANT_ID" \
  --organization organization:characterization >/dev/null
python3 "$PROVIDER_FIXTURE" allow_once >"$TEST_DIR/provider.port" &
PROVIDER_PID=$!
for _ in $(seq 1 100); do [[ -s "$TEST_DIR/provider.port" ]] && break; sleep 0.02; done
provider_port=$(tr -d '[:space:]' <"$TEST_DIR/provider.port")

# Direct Case A owns an unresolved PREPARE on the shared root.
"$YAI_BIN" case create --case case:wave15-holder --tenant "$YAI_TEST_TENANT_ID" >/dev/null
"$YAI_BIN" case bind-participant-role --case case:wave15-holder \
  --participant subject:policy-pack --role resource-attachment-compatibility-owner >/dev/null
YAI_JOURNAL="$TEST_DIR/holder.jsonl" "$YAI_BIN" case enter \
  --case case:wave15-holder --subject subject:llm-provider >/dev/null
YAI_JOURNAL="$TEST_DIR/holder.jsonl" "$YAI_BIN" case attach-provider \
  --case case:wave15-holder --subject subject:llm-provider \
  --provider-id provider:holder \
  --base-url "http://127.0.0.1:$provider_port/v1/chat/completions" \
  --model controlled-model >/dev/null
"$YAI_BIN" case attach-filesystem --case case:wave15-holder --attachment workspace \
  --root "$SHARED_ROOT" --allow-prefix allowed --policy-owner subject:policy-pack \
  --max-bytes 4096 >/dev/null
yai_configure_governed_filesystem_case "$YAI_BIN" "$YAI_HOME" case:wave15-holder \
  wave15-holder 1 allow subject:llm-provider >/dev/null

# Workflow Case B references the same physical root through its own canonical
# attachment. Its deterministic proposal still goes through policy/fencing.
"$YAI_BIN" case create --case case:wave15-workflow-busy \
  --tenant "$YAI_TEST_TENANT_ID" >/dev/null
"$YAI_BIN" case bind-participant-role --case case:wave15-workflow-busy \
  --participant subject:policy-pack --role resource-attachment-compatibility-owner >/dev/null
"$YAI_BIN" case bind-participant-role --case case:wave15-workflow-busy \
  --participant participant:operator --role operation-proposer >/dev/null
"$YAI_BIN" case attach-filesystem --case case:wave15-workflow-busy --attachment workspace \
  --root "$SHARED_ROOT" --allow-prefix allowed --policy-owner subject:policy-pack \
  --max-bytes 4096 >/dev/null
yai_configure_governed_filesystem_case "$YAI_BIN" "$YAI_HOME" \
  case:wave15-workflow-busy wave15-workflow-busy 1 allow \
  participant:operator >/dev/null
define_output=$("$YAI_BIN" workflow define --tenant "$YAI_TEST_TENANT_ID" \
  --file "$ROOT/tests/fixtures/workflows/resource-busy-retry.v1.json")
definition_id=$(sed -n 's/^workflow_definition_id: //p' <<<"$define_output")
bind_output=$("$YAI_BIN" workflow bind --case case:wave15-workflow-busy \
  --definition "$definition_id" --executor operator=participant:operator \
  --resource workspace=workspace)
binding_id=$(sed -n 's/^workflow_binding_id: //p' <<<"$bind_output")

set +e
holder_output=$(YAI_JOURNAL="$TEST_DIR/holder.jsonl" "$YAI_BIN" effect filesystem-write \
  --case case:wave15-holder --subject subject:llm-provider --attachment workspace \
  --prompt "hold the shared resource at PREPARE" --provider-id provider:holder \
  --base-url "http://127.0.0.1:$provider_port/v1/chat/completions" \
  --model controlled-model --failpoint after_prepare_before_effect 2>&1)
holder_exit=$?
set -e
[[ "$holder_exit" == "85" ]]
wait "$PROVIDER_PID"
PROVIDER_PID=""
holder_effect=$(sed -n 's/^effect_id: //p' <<<"$holder_output" | head -1)
[[ "$holder_effect" == effect:* ]]
[[ ! -e "$SHARED_ROOT/allowed/workflow.txt" ]]
trace_product 01 "./yai effect filesystem-write ... --failpoint after_prepare_before_effect" \
  "$holder_output" "$holder_exit"

"$YAI_BIN" runtime serve --workers 1 --max-active-per-tenant 1 \
  --max-queued-per-tenant 4 --max-queued-total 4 \
  >"$TEST_DIR/runtime.log" 2>&1 &
RUNTIME_PID=$!
queue=""
for _ in $(seq 1 300); do
  queue=$("$YAI_BIN" runtime queue --all 2>/dev/null || true)
  grep -Fq "state: Blocked" <<<"$queue" && break
  sleep 0.02
done
grep -Fq "state: Blocked" <<<"$queue"
grep -Fq "resource_temporarily_owned" <<<"$queue"
blocked_status=$("$YAI_BIN" workflow status --case case:wave15-workflow-busy)
grep -Fq "completed: false" <<<"$blocked_status"
grep -Fq "posture=Active" <<<"$blocked_status"
[[ ! -e "$SHARED_ROOT/allowed/workflow.txt" ]]
trace_product 02 "./yai runtime queue --all" "$queue" 0
trace_product 03 "./yai workflow status --case case:wave15-workflow-busy" \
  "$blocked_status" 0

reconcile_output=$("$YAI_BIN" effect reconcile --case case:wave15-holder \
  --effect "$holder_effect" --retry)
grep -Fq "reconciliation: EffectObserved" <<<"$reconcile_output"
trace_product 04 "./yai effect reconcile --case case:wave15-holder --effect $holder_effect --retry" \
  "$reconcile_output" 0

completed_status=""
for _ in $(seq 1 500); do
  completed_status=$("$YAI_BIN" workflow status --case case:wave15-workflow-busy \
    2>/dev/null || true)
  grep -Fq "completed: true" <<<"$completed_status" && break
  sleep 0.02
done
grep -Fq "completed: true" <<<"$completed_status"
[[ "$(<"$SHARED_ROOT/allowed/workflow.txt")" == \
  "workflow resumed after resource release" ]]
final_queue=$("$YAI_BIN" runtime queue --all)
grep -Fq "state: Completed" <<<"$final_queue"
"$YAI_BIN" runtime stop >/dev/null
wait "$RUNTIME_PID"
RUNTIME_PID=""
trace_product 05 "./yai workflow status --case case:wave15-workflow-busy" \
  "$completed_status" 0
trace_product 06 "./yai runtime queue --all" "$final_queue" 0

printf 'workflow_resource_busy_characterization: pass\n'
printf 'workflow_definition_id: %s\n' "$definition_id"
printf 'workflow_binding_id: %s\n' "$binding_id"
printf 'holder_effect_id: %s\n' "$holder_effect"
printf 'blocked_work_posture: Blocked\n'
printf 'blocked_decision_posture: allow\n'
printf 'retry_trigger: terminal_resource_release\n'
printf 'same_workflow_execution_completed: true\n'
printf 'provider_invocations_for_workflow: 0\n'
