#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
source "$ROOT/tests/characterization/lib/governed_case_policy.sh"
YAID="$ROOT/build/yaid"
FIXTURE="$ROOT/tests/fixtures/agentless_case_runtime_provider.py"
TEST_DIR="$(mktemp -d /tmp/yai-human-review.XXXXXX)"
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

trace_review_product() {
  [[ "${YAI_EXECUTION_EVIDENCE:-0}" == "1" ]] || return 0
  printf '\n[review-product-command:%s]\n$ %s\n%s\nexit: %s\n' \
    "$1" "$2" "$3" "$4" >&2
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
  YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case enter \
    --case case:new12-filesystem --subject subject:llm-provider >/dev/null
  YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case attach-provider \
    --case case:new12-filesystem --subject subject:llm-provider \
    --provider-id "$provider_id" \
    --base-url "http://127.0.0.1:$port/v1/chat/completions" \
    --model "$model" >/dev/null
  YAI_HOME="$CASE_HOME" "$YAI_BIN" case attach-filesystem \
    --case case:new12-filesystem --attachment workspace --root "$RESOURCE_ROOT" \
    --allow-prefix allowed --policy-owner subject:policy-pack --max-bytes 256 \
    --require-review >/dev/null
  yai_configure_governed_filesystem_case "$YAI_BIN" "$CASE_HOME" \
    case:new12-filesystem "review-$name" 1 allow subject:llm-provider \
    subject:policy-pack >/dev/null
}

run_case() {
  YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case run \
    --case case:new12-filesystem --subject subject:llm-provider --attachment workspace \
    --prompt "propose one human-reviewed filesystem write, then report completion" \
    --max-invocations 3 --max-operations 2 --max-resident-items 12 \
    --max-semantic-units 6000 --max-estimated-input-units 50000 "$@"
}

resume_case() {
  YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case resume \
    --case case:new12-filesystem "$@"
}

pending_review_id() {
  YAI_HOME="$CASE_HOME" "$YAI_BIN" review pending --case case:new12-filesystem |
    sed -n 's/^review_id: //p' | head -1
}

resolve_review() {
  local action="$1"
  local review_id="$2"
  shift 2
  YAI_HOME="$CASE_HOME" "$YAI_BIN" review "$action" "$review_id" \
    --case case:new12-filesystem --as subject:policy-pack \
    --reason "human participant $action exact operation" "$@"
}

# The primary product proof: no Grant/effect before human action, review queries
# are pure, approval itself performs no effect, and resume executes the original
# Operation exactly once through the controlled carrier.
start_provider review 2 approve
setup_case approve provider:review-a model-review-a "$LAST_PROVIDER_PORT"
approve_pause=$(run_case)
trace_review_product 01 "YAI_HOME=$CASE_HOME YAI_JOURNAL=$CASE_JOURNAL $YAI_BIN case run --case case:new12-filesystem --subject subject:llm-provider --attachment workspace --prompt 'propose one human-reviewed filesystem write, then report completion' --max-invocations 3 --max-operations 2 --max-resident-items 12 --max-semantic-units 6000 --max-estimated-input-units 50000" "$approve_pause" 0
require_text "$approve_pause" "runtime_status: AwaitingReview"
require_text "$approve_pause" "execution_grant: none"
require_text "$approve_pause" "external_effect: none"
[[ ! -e "$RESOURCE_ROOT/allowed/reviewed.txt" ]]
review_id=$(pending_review_id)
[[ "$review_id" == review:* ]]
summary_before=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" store summary)
count_before=$(sed -n 's/^transitions_total: //p' <<<"$summary_before")
show_pending=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" review show "$review_id" \
  --case case:new12-filesystem)
trace_review_product 02 "YAI_HOME=$CASE_HOME $YAI_BIN review show $review_id --case case:new12-filesystem" "$show_pending" 0
require_text "$show_pending" "status: pending"
require_text "$show_pending" "operation_id: operation:"
require_text "$show_pending" "normalized_target: allowed/reviewed.txt"
require_text "$show_pending" "operator_trust_boundary: local_cli_claimed_bound_participant"
YAI_HOME="$CASE_HOME" "$YAI_BIN" review pending --case case:new12-filesystem >/dev/null
summary_after=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" store summary)
count_after=$(sed -n 's/^transitions_total: //p' <<<"$summary_after")
[[ "$count_before" == "$count_after" ]]
set +e
wrong_reviewer=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" review approve "$review_id" \
  --case case:new12-filesystem --as subject:llm-provider --reason "self approve" 2>&1)
wrong_reviewer_code=$?
set -e
[[ "$wrong_reviewer_code" -ne 0 ]]
trace_review_product 03 "YAI_HOME=$CASE_HOME $YAI_BIN review approve $review_id --case case:new12-filesystem --as subject:llm-provider --reason 'self approve'" "$wrong_reviewer" "$wrong_reviewer_code"
require_text "$wrong_reviewer" "reviewer_not_eligible_for_case_review"
set +e
wrong_case=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" review approve "$review_id" \
  --case case:wrong-review-target --as subject:policy-pack \
  --reason "wrong Case must not resolve review" 2>&1)
wrong_case_code=$?
set -e
[[ "$wrong_case_code" -ne 0 ]]
require_text "$wrong_case" "canonical CaseState missing for case:wrong-review-target"
still_pending=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" review show "$review_id" \
  --case case:new12-filesystem)
require_text "$still_pending" "status: pending"
approved=$(resolve_review approve "$review_id")
trace_review_product 04 "YAI_HOME=$CASE_HOME $YAI_BIN review approve $review_id --case case:new12-filesystem --as subject:policy-pack --reason 'human participant approve exact operation'" "$approved" 0
require_text "$approved" "review_action: committed"
require_text "$approved" "execution_grant: none_review_command_never_executes"
[[ ! -e "$RESOURCE_ROOT/allowed/reviewed.txt" ]]
duplicate=$(resolve_review approve "$review_id")
require_text "$duplicate" "review_action: already_resolved_idempotent"
approve_resume=$(resume_case)
trace_review_product 05 "YAI_HOME=$CASE_HOME YAI_JOURNAL=$CASE_JOURNAL $YAI_BIN case resume --case case:new12-filesystem" "$approve_resume" 0
wait_providers
require_text "$approve_resume" "runtime_status: Completed"
require_text "$approve_resume" "operations: 1"
[[ "$(cat "$RESOURCE_ROOT/allowed/reviewed.txt")" == "human-reviewed effect" ]]
approved_review=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" review show "$review_id" \
  --case case:new12-filesystem)
require_text "$approved_review" "status: approved"
require_text "$approved_review" "effective_decision_id: decision:"
memory=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" memory list \
  --case case:new12-filesystem --include-superseded --limit 50)
require_text "$memory" "kind:review"
require_text "$memory" "kind:resource_effect"
YAI_HOME="$CASE_HOME" "$YAI_BIN" graph materialize --case case:new12-filesystem >/dev/null
review_relations=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" graph relations \
  --case case:new12-filesystem --limit 200)
require_text "$review_relations" "edge_kind: review_request_for_operation"
require_text "$review_relations" "edge_kind: review_action_resolves_request"
require_text "$review_relations" "edge_kind: review_action_by_participant"
status_after=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" case status --case case:new12-filesystem)
require_text "$status_after" "runtime_admission_status: none"

# Human DENY is durable control history, never effect evidence.
start_provider review 2 deny
setup_case deny provider:review-deny model-review-deny "$LAST_PROVIDER_PORT"
deny_pause=$(run_case)
require_text "$deny_pause" "runtime_status: AwaitingReview"
deny_review=$(pending_review_id)
denied=$(resolve_review deny "$deny_review")
require_text "$denied" "effective_decision_id: decision:"
set +e
approve_after_deny=$(resolve_review approve "$deny_review" 2>&1)
approve_after_deny_code=$?
set -e
[[ "$approve_after_deny_code" -ne 0 ]]
require_text "$approve_after_deny" "review_already_resolved: denied"
deny_resume=$(resume_case)
wait_providers
require_text "$deny_resume" "runtime_status: Completed"
require_text "$deny_resume" "operations: 0"
[[ ! -e "$RESOURCE_ROOT/allowed/reviewed.txt" ]]
deny_memory=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" memory list \
  --case case:new12-filesystem --include-superseded --limit 50)
require_text "$deny_memory" "kind:review"
if grep -Fq "kind:resource_effect" <<<"$deny_memory"; then
  printf 'denied review created false resource-effect memory\n' >&2
  exit 1
fi

# DEFER remains unresolved, releases the run claim and does not spin the
# provider. A later approval resumes the same Operation.
start_provider review 2 defer
setup_case defer provider:review-defer model-review-defer "$LAST_PROVIDER_PORT"
defer_pause=$(run_case)
defer_review=$(pending_review_id)
deferred=$(resolve_review defer "$defer_review")
require_text "$deferred" "effective_decision_id: none_deferred"
defer_resume=$(resume_case)
require_text "$defer_resume" "runtime_status: AwaitingReview"
require_text "$defer_resume" "invocations: 1"
[[ ! -e "$RESOURCE_ROOT/allowed/reviewed.txt" ]]
resolve_review approve "$defer_review" >/dev/null
defer_final=$(resume_case)
wait_providers
require_text "$defer_final" "runtime_status: Completed"
[[ -e "$RESOURCE_ROOT/allowed/reviewed.txt" ]]

# Provider and model replacement during the human delay cannot change Case or
# Operation identity. Provider B sees the observed consequence without A/KV.
start_provider review 1 replacement-a
setup_case replacement provider:a model-a "$LAST_PROVIDER_PORT"
replacement_pause=$(run_case)
replacement_review=$(pending_review_id)
operation_before=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" review show "$replacement_review" \
  --case case:new12-filesystem | sed -n 's/^operation_id: //p')
wait_providers
resolve_review approve "$replacement_review" >/dev/null
start_provider review_resume 1 replacement-b
YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case attach-provider \
  --case case:new12-filesystem --subject subject:llm-provider \
  --provider-id provider:b \
  --base-url "http://127.0.0.1:$LAST_PROVIDER_PORT/v1/chat/completions" \
  --model model-b >/dev/null
replacement_resume=$(resume_case)
wait_providers
require_text "$replacement_resume" "runtime_status: Completed"
effect_id=$(sed -n 's/^last_effect_id: //p' <<<"$replacement_resume")
effect_chain=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" effect inspect \
  --case case:new12-filesystem --effect "$effect_id")
require_text "$effect_chain" "operation_id: $operation_before"
grep -Fq '"model": "model-b"' "$TEST_DIR/replacement-b.log.json"

# Provider material containing an invented approval/reviewer is rejected at the
# proposal boundary and cannot create review authority or an effect.
start_provider fake_approval 1 fake-approval
setup_case fake-approval provider:fake model-fake "$LAST_PROVIDER_PORT"
fake_output=$(run_case)
wait_providers
require_text "$fake_output" "runtime_status: MalformedProviderResult"
require_text "$fake_output" "operations: 0"
[[ ! -e "$RESOURCE_ROOT/allowed/reviewed.txt" ]]

# A pending review is bound to the exact EffectivePolicy. Replacing the Case
# binding makes the old human gate stale; approval cannot mint authority from
# the superseded basis.
start_provider review 1 policy-stale
setup_case stale provider:stale-policy model-stale-policy "$LAST_PROVIDER_PORT"
stale_pause=$(run_case)
require_text "$stale_pause" "runtime_status: AwaitingReview"
policy_stale_review=$(pending_review_id)
wait_providers
status_before_replace=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" case policy status \
  --case case:new12-filesystem)
prior_binding=$(sed -n 's/^policy_binding: binding_id=\([^ ]*\).*/\1/p' \
  <<<"$status_before_replace" | head -1)
generation_before_replace=$(sed -n 's/^case_generation: //p' \
  <<<"$status_before_replace" | head -1)
policy_v1="$CASE_HOME/review-stale-1.policy.json"
policy_v2="$CASE_HOME/review-stale-2.policy.json"
sed 's/"source_version":"1"/"source_version":"2"/; s#test://review-stale/1#test://review-stale/2#' \
  "$policy_v1" >"$policy_v2"
v2_ingest=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" policy ingest "$policy_v2" \
  --as participant:local-policy-operator)
v2_artifact=$(sed -n 's/^artifact_id: //p' <<<"$v2_ingest" | head -1)
YAI_HOME="$CASE_HOME" "$YAI_BIN" policy validate "$v2_artifact" \
  --as participant:local-policy-operator --reason "validate replacement" >/dev/null
YAI_HOME="$CASE_HOME" "$YAI_BIN" policy publish "$v2_artifact" \
  --as participant:local-policy-operator --reason "publish replacement" >/dev/null
replace_output=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" case policy replace \
  --case case:new12-filesystem --binding "$prior_binding" --artifact "$v2_artifact" \
  --expected-generation "$generation_before_replace" --as participant:local-policy-operator \
  --reason "replace while review pending")
trace_review_product 06 "YAI_HOME=$CASE_HOME $YAI_BIN case policy replace --case case:new12-filesystem --binding $prior_binding --artifact $v2_artifact --expected-generation $generation_before_replace --as participant:local-policy-operator --reason 'replace while review pending'" "$replace_output" 0
set +e
stale_approval=$(resolve_review approve "$policy_stale_review" 2>&1)
stale_approval_code=$?
set -e
[[ "$stale_approval_code" -ne 0 ]]
trace_review_product 07 "YAI_HOME=$CASE_HOME $YAI_BIN review approve $policy_stale_review --case case:new12-filesystem --as subject:policy-pack --reason 'human participant approve exact operation'" "$stale_approval" "$stale_approval_code"
require_text "$stale_approval" "review_invalidation: committed"
require_text "$stale_approval" "invalidation_reason: Some(PolicyBasisChanged)"
require_text "$stale_approval" "review_authority_invalidated"
[[ ! -e "$RESOURCE_ROOT/allowed/reviewed.txt" ]]

# Cross-process Case admission: runner B is rejected transactionally while A
# waits at the real provider boundary. Normal completion releases the claim.
start_provider delay_complete 1 concurrency
setup_case concurrency provider:concurrency model-concurrency "$LAST_PROVIDER_PORT"
run_case >"$TEST_DIR/concurrency-a.out" 2>&1 &
runner_a=$!
for _ in $(seq 1 100); do
  if YAI_HOME="$CASE_HOME" "$YAI_BIN" case status --case case:new12-filesystem \
      >"$TEST_DIR/concurrency-status.out" 2>/dev/null &&
      grep -Fq "runtime_admission_status: active" "$TEST_DIR/concurrency-status.out"; then
    break
  fi
  sleep 0.02
done
grep -Fq "runtime_admission_status: active" "$TEST_DIR/concurrency-status.out"
set +e
runner_b=$(resume_case 2>&1)
runner_b_code=$?
set -e
[[ "$runner_b_code" -ne 0 ]]
require_text "$runner_b" "case_runtime_admission_active"
wait "$runner_a"
wait_providers
concurrency_status=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" case status \
  --case case:new12-filesystem)
require_text "$concurrency_status" "runtime_admission_status: none"

# A process::exit failpoint leaves admission metadata behind. Dead-owner
# detection reclaims it deterministically before continuing from CaseState.
start_provider review 2 stale-claim
setup_case stale-claim provider:stale model-stale "$LAST_PROVIDER_PORT"
set +e
stale_crash=$(run_case --failpoint runtime_after_provider_result 2>&1)
stale_code=$?
set -e
[[ "$stale_code" -eq 91 ]]
stale_resume=$(resume_case)
require_text "$stale_resume" "runtime_admission: reclaimed_stale"
require_text "$stale_resume" "runtime_status: AwaitingReview"
stale_review=$(pending_review_id)
resolve_review approve "$stale_review" >/dev/null
stale_final=$(resume_case)
wait_providers
require_text "$stale_final" "runtime_status: Completed"

# Review crash matrix. Each boundary resumes from canonical state; no second
# Operation is synthesized and only R5 can leave a Grant without PREPARE.
run_review_crash_case() {
  local name="$1"
  local phase="$2"
  local expected_code="$3"
  local resolution="$4"
  start_provider review 2 "$name"
  setup_case "$name" "provider:$name" "model-$name" "$LAST_PROVIDER_PORT"
  if [[ "$phase" == "review_r1" || "$phase" == "review_r2" ]]; then
    set +e
    run_case --failpoint "$phase" >"$TEST_DIR/$name.crash" 2>&1
    local code=$?
    set -e
    [[ "$code" -eq "$expected_code" ]]
    local paused
    paused=$(resume_case)
    require_text "$paused" "runtime_status: AwaitingReview"
  else
    local paused
    paused=$(run_case)
    require_text "$paused" "runtime_status: AwaitingReview"
  fi
  local rid
  rid=$(pending_review_id)
  if [[ "$phase" == "review_r3" || "$phase" == "review_r4" || "$phase" == "review_r6" ]]; then
    set +e
    resolve_review "$resolution" "$rid" --failpoint "$phase" \
      >"$TEST_DIR/$name.crash" 2>&1
    local code=$?
    set -e
    [[ "$code" -eq "$expected_code" ]]
  else
    resolve_review "$resolution" "$rid" >/dev/null
  fi
  if [[ "$phase" == "review_r5" ]]; then
    set +e
    resume_case --failpoint review_r5 >"$TEST_DIR/$name.crash" 2>&1
    local code=$?
    set -e
    [[ "$code" -eq "$expected_code" ]]
  fi
  local final
  final=$(resume_case)
  wait_providers
  require_text "$final" "runtime_status: Completed"
  if [[ "$resolution" == "approve" ]]; then
    [[ -e "$RESOURCE_ROOT/allowed/reviewed.txt" ]]
    require_text "$final" "operations: 1"
  else
    [[ ! -e "$RESOURCE_ROOT/allowed/reviewed.txt" ]]
    require_text "$final" "operations: 0"
  fi
}

run_review_crash_case crash-r1 review_r1 94 approve
run_review_crash_case crash-r2 review_r2 95 approve
run_review_crash_case crash-r3 review_r3 103 approve
run_review_crash_case crash-r4 review_r4 104 approve
run_review_crash_case crash-r5 review_r5 84 approve
run_review_crash_case crash-r6 review_r6 106 deny

printf 'human_review:approve_deny_defer_and_query_purity ok\n'
printf 'human_review:provider_model_replacement_and_no_second_operation ok\n'
printf 'human_review:runtime_admission_concurrency_and_stale_reclaim ok\n'
printf 'human_review:crash_r1_r6_recovery ok\n'
printf 'human_review:provider_cannot_invent_human_authority ok\n'
printf 'human_review:policy_basis_change_fails_closed ok\n'
