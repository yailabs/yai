#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
TEST_DIR="$(mktemp -d /tmp/yai-workflow-hardening.XXXXXX)"
export YAI_HOME="$TEST_DIR/home"
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

# The explicit DAG is acyclic, but NodeSatisfied introduces the inverse edge.
YAI_HOME="$YAI_HOME" "$YAI_BIN" security bootstrap-local \
  --tenant tenant:workflow-hardening --organization organization:characterization >/dev/null
set +e
semantic_cycle_output=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" workflow define \
  --tenant tenant:workflow-hardening \
  --file "$ROOT/tests/fixtures/workflows/semantic-dependency-cycle.v1.json" 2>&1)
semantic_cycle_exit=$?
set -e
[[ "$semantic_cycle_exit" -ne 0 ]]
require_text "$semantic_cycle_output" "workflow_cycle_rejected"
trace_product 01 "./yai workflow define --tenant tenant:workflow-hardening --file tests/fixtures/workflows/semantic-dependency-cycle.v1.json" \
  "$semantic_cycle_output" "$semantic_cycle_exit"

# Crash after the canonical deterministic proposal and before Operation. The
# persisted WorkItem carries the failpoint; recovery must consume the existing
# proposal rather than create another semantic proposal or execution.
export YAI_TEST_TENANT_ID="tenant:workflow-product"
yai_bootstrap_tenant_case "$YAI_BIN" "$YAI_HOME" \
  case:h15-deterministic "$YAI_TEST_TENANT_ID" organization:characterization
principal_id=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" identity whoami --json |
  python3 -c 'import json,sys; print(next(field["value"] for field in json.load(sys.stdin)["data"]["fields"] if field["name"] == "Principal"))')
YAI_HOME="$YAI_HOME" "$YAI_BIN" case bind-participant-role \
  --case case:h15-deterministic --participant participant:operator \
  --role operation-proposer >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" case bind-participant-role \
  --case case:h15-deterministic --participant participant:operator \
  --role workflow-input >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" case principal link \
  --case case:h15-deterministic --principal "$principal_id" \
  --participant participant:operator >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" case attach-filesystem \
  --case case:h15-deterministic --attachment workspace \
  --root "$TEST_DIR/resource" --allow-prefix allowed \
  --policy-owner subject:policy-pack --max-bytes 4096 >/dev/null
yai_configure_governed_filesystem_case \
  "$YAI_BIN" "$YAI_HOME" case:h15-deterministic \
  h15.deterministic 1 allow participant:operator >/dev/null
definition_output=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" workflow define \
  --tenant "$YAI_TEST_TENANT_ID" \
  --file "$ROOT/tests/fixtures/workflows/controlled-remediation.v1.json")
definition_id=$(sed -n 's/^workflow_definition_id: //p' <<<"$definition_output" | head -1)
bind_output=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" workflow bind \
  --case case:h15-deterministic --definition "$definition_id" \
  --executor input-actor=participant:operator \
  --executor operator=participant:operator \
  --executor analyst-model=participant:operator \
  --resource workspace=workspace)
YAI_HOME="$YAI_HOME" "$YAI_BIN" workflow input \
  --case case:h15-deterministic --node change-ticket --value CHG-H15 >/dev/null

set +e
deterministic_crash_output=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" runtime serve \
  --workers 1 --max-active-per-tenant 1 --max-queued-per-tenant 4 \
  --max-queued-total 4 \
  --workflow-work-failpoint after_workflow_deterministic_proposal 2>&1)
deterministic_crash_exit=$?
set -e
[[ "$deterministic_crash_exit" -eq 90 ]]
require_text "$deterministic_crash_output" "workflow_deterministic_proposal_id: workflow-proposal:"
require_text "$deterministic_crash_output" \
  "controlled_effect_crash_injected: after_workflow_deterministic_proposal"
[[ ! -e "$TEST_DIR/resource/allowed/remediation.txt" ]]
crashed_status=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" workflow status \
  --case case:h15-deterministic)
require_text "$crashed_status" "node: apply-remediation kind=deterministic_work posture=Active"
trace_product 02 "./yai runtime serve --workflow-work-failpoint after_workflow_deterministic_proposal" \
  "$deterministic_crash_output" "$deterministic_crash_exit"
trace_product 03 "./yai workflow status --case case:h15-deterministic" "$crashed_status" 0

YAI_HOME="$YAI_HOME" "$YAI_BIN" runtime serve \
  --workers 1 --max-active-per-tenant 1 --max-queued-per-tenant 4 \
  --max-queued-total 4 >"$TEST_DIR/deterministic-recovery.log" 2>&1 &
RUNTIME_PID=$!
recovered_status=""
for _ in $(seq 1 240); do
  recovered_status=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" workflow status \
    --case case:h15-deterministic 2>/dev/null || true)
  grep -Fq "completed: true" <<<"$recovered_status" && break
  if ! kill -0 "$RUNTIME_PID" 2>/dev/null; then
    sed -n '1,260p' "$TEST_DIR/deterministic-recovery.log" >&2
    exit 1
  fi
  sleep 0.1
done
require_text "$recovered_status" "completed: true"
[[ "$(<"$TEST_DIR/resource/allowed/remediation.txt")" == "controlled remediation applied" ]]
deterministic_case_status=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" case status \
  --case case:h15-deterministic)
require_text "$deterministic_case_status" "invocations: 0"
require_text "$deterministic_case_status" "operations: 1"
deterministic_queue=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" runtime queue --all)
require_text "$deterministic_queue" "runtime_queue_items: 1"
require_text "$deterministic_queue" "state: Completed"
YAI_HOME="$YAI_HOME" "$YAI_BIN" runtime stop >/dev/null
wait "$RUNTIME_PID"
RUNTIME_PID=""
recovery_log=$(sed -n '1,280p' "$TEST_DIR/deterministic-recovery.log")
trace_product 04 "./yai workflow status --case case:h15-deterministic" "$recovered_status" 0
trace_product 05 "./yai case status --case case:h15-deterministic" "$deterministic_case_status" 0
trace_product 06 "runtime recovery foreground output" "$recovery_log" 0

# Maximum admitted node scale. The cheap passive chain creates no WorkItem and
# no provider invocation; status is pure and stable after the runtime stops.
scale_definition="$TEST_DIR/workflow-128.json"
node - "$scale_definition" <<'NODE'
const fs = require('fs');
const path = process.argv[2];
const nodes = Array.from({length: 128}, (_, index) => ({
  node_id: `wait-${String(index).padStart(3, '0')}`,
  kind: 'wait',
  predicate: {predicate: 'case_lifecycle', lifecycle: 'open'}
}));
const edges = Array.from({length: 127}, (_, index) => ({
  from: nodes[index].node_id,
  to: nodes[index + 1].node_id,
  kind: 'always'
}));
fs.writeFileSync(path, JSON.stringify({
  schema: 'yai.workflow_definition.v1',
  tenant_id: 'tenant:workflow-hardening',
  workflow_key: 'maximum-passive-chain',
  declared_version: '1',
  name: 'Maximum passive chain',
  description: '128-node bounded deterministic replay characterization',
  nodes,
  edges
}, null, 2));
NODE
YAI_HOME="$YAI_HOME" "$YAI_BIN" case create \
  --case case:h15-scale --tenant tenant:workflow-hardening >/dev/null
scale_define_output=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" workflow define \
  --tenant tenant:workflow-hardening --file "$scale_definition")
scale_definition_id=$(sed -n 's/^workflow_definition_id: //p' <<<"$scale_define_output" | head -1)
scale_bind_output=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" workflow bind \
  --case case:h15-scale --definition "$scale_definition_id")
scale_start_ms=$(date +%s%3N)
YAI_HOME="$YAI_HOME" "$YAI_BIN" runtime serve \
  --workers 1 --max-active-per-tenant 1 --max-queued-per-tenant 4 \
  --max-queued-total 4 >"$TEST_DIR/scale-runtime.log" 2>&1 &
RUNTIME_PID=$!
scale_status=""
for _ in $(seq 1 300); do
  scale_status=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" workflow status \
    --case case:h15-scale 2>/dev/null || true)
  grep -Fq "completed: true" <<<"$scale_status" && break
  if ! kill -0 "$RUNTIME_PID" 2>/dev/null; then
    sed -n '1,260p' "$TEST_DIR/scale-runtime.log" >&2
    exit 1
  fi
  sleep 0.05
done
scale_end_ms=$(date +%s%3N)
require_text "$scale_status" "completed: true"
require_text "$scale_status" "satisfied: 128"
require_text "$scale_status" "ready: 0"
YAI_HOME="$YAI_HOME" "$YAI_BIN" runtime stop >/dev/null
wait "$RUNTIME_PID"
RUNTIME_PID=""
scale_replay=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" workflow status --case case:h15-scale)
[[ "$scale_replay" == "$scale_status" ]]
scale_generation=$(sed -n 's/^case_generation: //p' <<<"$scale_status" | head -1)
scale_bytes=$(wc -c <"$scale_definition" | tr -d '[:space:]')
trace_product 07 "./yai workflow define --tenant tenant:workflow-hardening --file <128-node-definition>" \
  "$scale_define_output" 0
trace_product 08 "./yai workflow bind --case case:h15-scale --definition $scale_definition_id" \
  "$scale_bind_output" 0
trace_product 09 "./yai workflow status --case case:h15-scale" "$scale_status" 0

printf 'workflow_kernel_hardening_characterization: pass\n'
printf 'semantic_cycle_exit: %s\n' "$semantic_cycle_exit"
printf 'deterministic_crash_exit: %s\n' "$deterministic_crash_exit"
printf 'deterministic_provider_invocations: 0\n'
printf 'deterministic_operations: 1\n'
printf 'deterministic_work_items: 1\n'
printf 'scale_nodes: 128\n'
printf 'scale_edges: 127\n'
printf 'scale_definition_bytes: %s\n' "$scale_bytes"
printf 'scale_case_generation: %s\n' "$scale_generation"
printf 'scale_progression_ms: %s\n' "$((scale_end_ms - scale_start_ms))"
printf 'scale_work_items_created: 0\n'
printf 'scale_provider_invocations: 0\n'
printf 'scale_physical_effects: 0\n'
printf 'scale_replay_equal: true\n'
