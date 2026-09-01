#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
TEST_DIR="$(mktemp -d /tmp/yai-workflow-kernel.XXXXXX)"
export YAI_HOME="$TEST_DIR/home"
export YAI_TEST_TENANT_ID="tenant:workflow-product"
RUNTIME_PID=""

# shellcheck source=../lib/governed_case_policy.sh
source "$ROOT/tests/characterization/lib/governed_case_policy.sh"
export YAI_POLICY_EXECUTION_EVIDENCE=0

cleanup() {
  if [[ -n "$RUNTIME_PID" ]] && kill -0 "$RUNTIME_PID" 2>/dev/null; then
    kill "$RUNTIME_PID" 2>/dev/null || true
    wait "$RUNTIME_PID" 2>/dev/null || true
  fi
  if [[ "${YAI_KEEP_TEST_DIR:-0}" != "1" ]]; then rm -rf "$TEST_DIR"; fi
}
trap cleanup EXIT INT TERM

require_text() { grep -Fq -- "$2" <<<"$1"; }
trace_product() {
  [[ "${YAI_EXECUTION_EVIDENCE:-0}" == "1" ]] || return 0
  printf '\n[product-command:%s]\n$ %s\n%s\nexit: %s\n' "$1" "$2" "$3" "$4"
}

mkdir -p "$YAI_HOME" "$TEST_DIR/resource/allowed"
mkdir -p "$TEST_DIR/resource-b/allowed"
yai_bootstrap_tenant_case "$YAI_BIN" "$YAI_HOME" \
  "case:workflow-product" "$YAI_TEST_TENANT_ID" "organization:characterization"
principal_id=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" identity whoami --json | \
  python3 -c 'import json,sys; print(next(field["value"] for field in json.load(sys.stdin)["data"]["fields"] if field["name"] == "Principal"))')

YAI_HOME="$YAI_HOME" "$YAI_BIN" case bind-participant-role \
  --case case:workflow-product --participant participant:operator \
  --role operation-proposer >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" case bind-participant-role \
  --case case:workflow-product --participant participant:operator \
  --role workflow-input >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" case principal link \
  --case case:workflow-product --principal "$principal_id" \
  --participant participant:operator >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" case attach-filesystem \
  --case case:workflow-product --attachment workspace \
  --root "$TEST_DIR/resource" --allow-prefix allowed \
  --policy-owner subject:policy-pack --max-bytes 4096 >/dev/null
yai_configure_governed_filesystem_case \
  "$YAI_BIN" "$YAI_HOME" "case:workflow-product" \
  "workflow.controlled-remediation" "1" "allow" "participant:operator" >/dev/null

define_output=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" workflow define \
  --tenant "$YAI_TEST_TENANT_ID" \
  --file "$ROOT/tests/fixtures/workflows/controlled-remediation.v1.json")
definition_id=$(sed -n 's/^workflow_definition_id: //p' <<<"$define_output" | head -1)
[[ "$definition_id" == workflow-definition:* ]]
trace_product 01 "YAI_HOME=$YAI_HOME ./yai workflow define --tenant $YAI_TEST_TENANT_ID --file tests/fixtures/workflows/controlled-remediation.v1.json" "$define_output" 0

v2_output=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" workflow define \
  --tenant "$YAI_TEST_TENANT_ID" \
  --file "$ROOT/tests/fixtures/workflows/controlled-remediation.v2.json")
definition_v2_id=$(sed -n 's/^workflow_definition_id: //p' <<<"$v2_output" | head -1)
[[ "$definition_v2_id" == workflow-definition:* ]]
[[ "$definition_v2_id" != "$definition_id" ]]
list_output=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" workflow list --tenant "$YAI_TEST_TENANT_ID")
require_text "$list_output" "workflow_definitions: 2"
require_text "$list_output" "version=1"
require_text "$list_output" "version=2"
show_output=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" workflow show "$definition_id")
require_text "$show_output" '"declared_version": "1"'
trace_product 02 "YAI_HOME=$YAI_HOME ./yai workflow list --tenant $YAI_TEST_TENANT_ID" "$list_output" 0
trace_product 03 "YAI_HOME=$YAI_HOME ./yai workflow show $definition_id" "$show_output" 0

bind_output=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" workflow bind \
  --case case:workflow-product --definition "$definition_id" \
  --executor input-actor=participant:operator \
  --executor operator=participant:operator \
  --executor analyst-model=participant:operator \
  --resource workspace=workspace)
binding_id=$(sed -n 's/^workflow_binding_id: //p' <<<"$bind_output" | head -1)
[[ "$binding_id" == case-workflow-binding:* ]]
trace_product 04 "YAI_HOME=$YAI_HOME ./yai workflow bind --case case:workflow-product --definition $definition_id --executor input-actor=participant:operator --executor operator=participant:operator --executor analyst-model=participant:operator --resource workspace=workspace" "$bind_output" 0

# A second Case adopts the exact same immutable definition but owns separate
# binding, Case history and physical resource state.
YAI_HOME="$YAI_HOME" "$YAI_BIN" case create \
  --case case:workflow-product-b --tenant "$YAI_TEST_TENANT_ID" >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" case bind-participant-role \
  --case case:workflow-product-b --participant subject:policy-pack \
  --role resource-attachment-compatibility-owner >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" case bind-participant-role \
  --case case:workflow-product-b --participant participant:operator-b \
  --role operation-proposer >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" case bind-participant-role \
  --case case:workflow-product-b --participant participant:operator-b \
  --role workflow-input >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" case principal link \
  --case case:workflow-product-b --principal "$principal_id" \
  --participant participant:operator-b >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" case attach-filesystem \
  --case case:workflow-product-b --attachment workspace-b \
  --root "$TEST_DIR/resource-b" --allow-prefix allowed \
  --policy-owner subject:policy-pack --max-bytes 4096 >/dev/null
yai_configure_governed_filesystem_case \
  "$YAI_BIN" "$YAI_HOME" "case:workflow-product-b" \
  "workflow.controlled-remediation-b" "1" "allow" \
  "participant:operator-b" >/dev/null
bind_b_output=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" workflow bind \
  --case case:workflow-product-b --definition "$definition_id" \
  --executor input-actor=participant:operator-b \
  --executor operator=participant:operator-b \
  --executor analyst-model=participant:operator-b \
  --resource workspace=workspace-b)
binding_b_id=$(sed -n 's/^workflow_binding_id: //p' <<<"$bind_b_output" | head -1)
[[ "$binding_b_id" == case-workflow-binding:* ]]
[[ "$binding_b_id" != "$binding_id" ]]

waiting=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" workflow status --case case:workflow-product)
require_text "$waiting" "node: change-ticket kind=human_input posture=WaitingHumanInput"
trace_product 05 "YAI_HOME=$YAI_HOME ./yai workflow status --case case:workflow-product" "$waiting" 0

input_output=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" workflow input \
  --case case:workflow-product --node change-ticket --value CHG-1500)
require_text "$input_output" "review_action_created: false"
YAI_HOME="$YAI_HOME" "$YAI_BIN" workflow input \
  --case case:workflow-product-b --node change-ticket --value CHG-1500-B >/dev/null
trace_product 06 "YAI_HOME=$YAI_HOME ./yai workflow input --case case:workflow-product --node change-ticket --value CHG-1500" "$input_output" 0

YAI_HOME="$YAI_HOME" "$YAI_BIN" runtime serve \
  --workers 2 --max-active-per-tenant 2 --max-queued-per-tenant 8 \
  --max-queued-total 16 >"$TEST_DIR/runtime.log" 2>&1 &
RUNTIME_PID=$!

completed=""
completed_b=""
for _ in $(seq 1 200); do
  completed=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" workflow status \
    --case case:workflow-product 2>/dev/null || true)
  completed_b=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" workflow status \
    --case case:workflow-product-b 2>/dev/null || true)
  if grep -Fq "completed: true" <<<"$completed" && \
     grep -Fq "completed: true" <<<"$completed_b"; then break; fi
  if ! kill -0 "$RUNTIME_PID" 2>/dev/null; then
    printf 'runtime exited before workflow completion\n' >&2
    sed -n '1,240p' "$TEST_DIR/runtime.log" >&2
    exit 1
  fi
  sleep 0.1
done
require_text "$completed" "completed: true"
require_text "$completed_b" "completed: true"
require_text "$completed" "satisfied: 5"
require_text "$completed" "skipped: 1"
[[ "$(<"$TEST_DIR/resource/allowed/remediation.txt")" == "controlled remediation applied" ]]
[[ "$(<"$TEST_DIR/resource-b/allowed/remediation.txt")" == "controlled remediation applied" ]]
require_text "$completed" "workflow_definition_id: $definition_id"
require_text "$completed_b" "workflow_definition_id: $definition_id"

case_status=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" case status --case case:workflow-product)
case_b_status=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" case status --case case:workflow-product-b)
require_text "$case_status" "invocations: 0"
require_text "$case_status" "operations: 1"
require_text "$case_b_status" "invocations: 0"
require_text "$case_b_status" "operations: 1"
trace_product 07 "YAI_HOME=$YAI_HOME ./yai workflow status --case case:workflow-product" "$completed" 0
trace_product 08 "YAI_HOME=$YAI_HOME ./yai workflow status --case case:workflow-product-b" "$completed_b" 0
trace_product 09 "YAI_HOME=$YAI_HOME ./yai case status --case case:workflow-product" "$case_status" 0

stop_output=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" runtime stop)
trace_product 10 "YAI_HOME=$YAI_HOME ./yai runtime stop" "$stop_output" 0
wait "$RUNTIME_PID"
RUNTIME_PID=""

runtime_output=$(sed -n '1,240p' "$TEST_DIR/runtime.log")
require_text "$runtime_output" "workflow_work_materialized: 2"
require_text "$runtime_output" "effect_state: finalized"
trace_product 11 "runtime foreground output" "$runtime_output" 0

printf 'workflow_kernel_characterization: pass\n'
printf 'workflow_definition_id: %s\n' "$definition_id"
printf 'workflow_binding_id: %s\n' "$binding_id"
printf 'workflow_definition_v2_id: %s\n' "$definition_v2_id"
printf 'second_case_workflow_binding_id: %s\n' "$binding_b_id"
printf 'shared_definition_case_count: 2\n'
printf 'definition_pinning: v1_case_unchanged_after_v2\n'
printf 'provider_invocations: 0\n'
printf 'deterministic_nodes: 2\n'
printf 'passive_nodes: 8\n'
printf 'modelwork_nodes_executed: 0\n'
