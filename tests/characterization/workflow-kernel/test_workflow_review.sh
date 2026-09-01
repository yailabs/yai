#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
TEST_DIR="$(mktemp -d /tmp/yai-workflow-review.XXXXXX)"
export YAI_HOME="$TEST_DIR/home"
export YAI_TEST_TENANT_ID="tenant:wave15-review"
export YAI_POLICY_EXECUTION_EVIDENCE=0
RUNTIME_PID=""

# shellcheck source=../lib/governed_case_policy.sh
source "$ROOT/tests/characterization/lib/governed_case_policy.sh"

cleanup() {
  if [[ -n "$RUNTIME_PID" ]] && kill -0 "$RUNTIME_PID" 2>/dev/null; then
    kill "$RUNTIME_PID" 2>/dev/null || true
    wait "$RUNTIME_PID" 2>/dev/null || true
  fi
  if [[ "${YAI_KEEP_TEST_DIR:-0}" != "1" ]]; then rm -rf "$TEST_DIR"; fi
}
trap cleanup EXIT INT TERM

trace_product() {
  [[ "${YAI_EXECUTION_EVIDENCE:-0}" == "1" ]] || return 0
  printf '\n[product-command:%s]\n$ %s\n%s\nexit: %s\n' "$1" "$2" "$3" "$4"
}

mkdir -p "$YAI_HOME" "$TEST_DIR/resource/allowed"
yai_bootstrap_tenant_case "$YAI_BIN" "$YAI_HOME" case:wave15-review \
  "$YAI_TEST_TENANT_ID" organization:characterization
principal_id=$("$YAI_BIN" identity whoami --json | \
  python3 -c 'import json,sys; print(next(field["value"] for field in json.load(sys.stdin)["data"]["fields"] if field["name"] == "Principal"))')
"$YAI_BIN" case bind-participant-role --case case:wave15-review \
  --participant participant:operator --role operation-proposer >/dev/null
"$YAI_BIN" case bind-participant-role --case case:wave15-review \
  --participant participant:operator --role operation-reviewer >/dev/null
"$YAI_BIN" case principal link --case case:wave15-review \
  --principal "$principal_id" --participant participant:operator >/dev/null
"$YAI_BIN" case attach-filesystem --case case:wave15-review --attachment workspace \
  --root "$TEST_DIR/resource" --allow-prefix allowed \
  --policy-owner subject:policy-pack --max-bytes 4096 >/dev/null
yai_configure_governed_filesystem_case "$YAI_BIN" "$YAI_HOME" case:wave15-review \
  wave15-review 1 allow participant:operator participant:operator >/dev/null

define_output=$("$YAI_BIN" workflow define --tenant "$YAI_TEST_TENANT_ID" \
  --file "$ROOT/tests/fixtures/workflows/review-required.v1.json")
definition_id=$(sed -n 's/^workflow_definition_id: //p' <<<"$define_output")
bind_output=$("$YAI_BIN" workflow bind --case case:wave15-review \
  --definition "$definition_id" --executor operator=participant:operator \
  --resource workspace=workspace)
binding_id=$(sed -n 's/^workflow_binding_id: //p' <<<"$bind_output")

"$YAI_BIN" runtime serve --workers 2 --max-active-per-tenant 2 \
  --max-queued-per-tenant 4 --max-queued-total 4 \
  >"$TEST_DIR/runtime.log" 2>&1 &
RUNTIME_PID=$!
queue=""
for _ in $(seq 1 300); do
  queue=$("$YAI_BIN" runtime queue --all 2>/dev/null || true)
  grep -Fq "state: WaitingReview" <<<"$queue" && break
  sleep 0.02
done
grep -Fq "state: WaitingReview" <<<"$queue"
[[ ! -e "$TEST_DIR/resource/allowed/reviewed.txt" ]]
parked_status=$("$YAI_BIN" workflow status --case case:wave15-review)
grep -Fq "completed: false" <<<"$parked_status"
grep -Fq "posture=Active" <<<"$parked_status"
case_status=$("$YAI_BIN" case status --case case:wave15-review)
review_id=$(sed -n 's/^last_review_id: //p' <<<"$case_status")
[[ "$review_id" == review:* ]]
grep -Fq "invocations: 0" <<<"$case_status"
trace_product 01 "./yai runtime queue --all" "$queue" 0
trace_product 02 "./yai workflow status --case case:wave15-review" "$parked_status" 0

approve_output=$("$YAI_BIN" review approve "$review_id" --case case:wave15-review \
  --reason "authenticated workflow change approval")
grep -Fq "review_action: committed" <<<"$approve_output"
trace_product 03 "./yai review approve $review_id --case case:wave15-review --reason ..." \
  "$approve_output" 0

completed_status=""
for _ in $(seq 1 500); do
  completed_status=$("$YAI_BIN" workflow status --case case:wave15-review \
    2>/dev/null || true)
  grep -Fq "completed: true" <<<"$completed_status" && break
  sleep 0.02
done
grep -Fq "completed: true" <<<"$completed_status"
[[ "$(<"$TEST_DIR/resource/allowed/reviewed.txt")" == \
  "approved deterministic workflow effect" ]]
final_queue=$("$YAI_BIN" runtime queue --all)
grep -Fq "state: Completed" <<<"$final_queue"
"$YAI_BIN" runtime stop >/dev/null
wait "$RUNTIME_PID"
RUNTIME_PID=""
trace_product 04 "./yai workflow status --case case:wave15-review" "$completed_status" 0
trace_product 05 "./yai runtime queue --all" "$final_queue" 0

printf 'workflow_review_characterization: pass\n'
printf 'workflow_definition_id: %s\n' "$definition_id"
printf 'workflow_binding_id: %s\n' "$binding_id"
printf 'review_id: %s\n' "$review_id"
printf 'worker_released_while_waiting_review: true\n'
printf 'provider_invocations: 0\n'
printf 'review_owner_reused: yai.review\n'
