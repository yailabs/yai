#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
YAID="$ROOT/build/yaid"
FIXTURE="$ROOT/tests/fixtures/agentless_case_runtime_provider.py"
source "$ROOT/tests/characterization/lib/governed_case_policy.sh"

TEST_DIR="$(mktemp -d /tmp/yai-multi-case-runtime.XXXXXX)"
export YAI_HOME="$TEST_DIR/home"
SOCKET="$TEST_DIR/yaid.sock"
BASE_JOURNAL="$TEST_DIR/base.jsonl"
DAEMON_PID=""
SERVICE_PID=""
SERVICE_LOG=""
SERVICE_NAME=""
SERVICE_COMMAND=""
PROVIDER_PIDS=()
LAST_PROVIDER_PORT=""
LAST_WORK_ID=""
declare -A JOURNALS ROOTS

cleanup() {
  if [[ -n "$SERVICE_PID" ]]; then
    kill "$SERVICE_PID" >/dev/null 2>&1 || true
    wait "$SERVICE_PID" >/dev/null 2>&1 || true
  fi
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

trace_product() {
  [[ "${YAI_EXECUTION_EVIDENCE:-0}" == "1" ]] || return 0
  printf '\n[product-command:%s]\n$ %s\n%s\nexit: %s\n' "$1" "$2" "$3" "$4"
}

start_provider() {
  local mode="$1" expected="$2" name="$3" delay_ms="${4:-0}"
  local port_file="$TEST_DIR/$name.port"
  YAI_CASE_RUNTIME_PROVIDER_LOG="$TEST_DIR/$name.log.json" \
    YAI_PROVIDER_DELAY_MS="$delay_ms" \
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

provider_attach() {
  local case_id="$1" journal="$2" name="$3" port="$4"
  YAI_JOURNAL="$journal" "$YAI_BIN" case attach-provider \
    --case "$case_id" --subject subject:llm-provider \
    --provider-id "provider:$name" \
    --base-url "http://127.0.0.1:$port/v1/chat/completions" \
    --model "model-$name" >/dev/null
}

setup_case() {
  local name="$1" case_id="$2" tenant_id="$3" port="$4" root_path="$5" review="${6:-no}"
  local journal="$TEST_DIR/$name/journal.jsonl"
  mkdir -p "$TEST_DIR/$name" "$root_path/allowed"
  sed -e "s/case:new12-filesystem/$case_id/g" \
    -e "s/new12-fs/w13-$name/g" "$BASE_JOURNAL" >"$journal"
  "$YAI_BIN" case create --case "$case_id" --tenant "$tenant_id" >/dev/null
  "$YAI_BIN" case bind-participant-role --case "$case_id" \
    --participant subject:policy-pack \
    --role resource-attachment-compatibility-owner >/dev/null
  YAI_JOURNAL="$journal" "$YAI_BIN" case enter \
    --case "$case_id" --subject subject:llm-provider >/dev/null
  provider_attach "$case_id" "$journal" "$name" "$port"
  local attach_args=(case attach-filesystem --case "$case_id" --attachment workspace \
    --root "$root_path" --allow-prefix allowed \
    --policy-owner subject:policy-pack --max-bytes 256)
  if [[ "$review" == "yes" ]]; then attach_args+=(--require-review); fi
  "$YAI_BIN" "${attach_args[@]}" >/dev/null
  if [[ "$review" == "yes" ]]; then
    YAI_TEST_TENANT_ID="$tenant_id" yai_configure_governed_filesystem_case \
      "$YAI_BIN" "$YAI_HOME" "$case_id" "w13-$name" 1 allow \
      subject:llm-provider subject:policy-pack >/dev/null
    local principal_id whoami
    whoami=$("$YAI_BIN" identity whoami --json)
    principal_id=$(python3 -c 'import json,sys; print(next(field["value"] for field in json.load(sys.stdin)["data"]["fields"] if field["name"] == "Principal"))' <<<"$whoami")
    "$YAI_BIN" case principal link --case "$case_id" \
      --principal "$principal_id" --participant subject:policy-pack >/dev/null
  else
    YAI_TEST_TENANT_ID="$tenant_id" yai_configure_governed_filesystem_case \
      "$YAI_BIN" "$YAI_HOME" "$case_id" "w13-$name" 1 allow \
      subject:llm-provider >/dev/null
  fi
  JOURNALS[$name]="$journal"
  ROOTS[$name]="$root_path"
}

start_service() {
  local name="$1" workers="$2" active="$3" tenant_queue="$4" global_queue="$5"
  shift 5
  SERVICE_NAME="$name"
  SERVICE_LOG="$TEST_DIR/$name.service.log"
  SERVICE_COMMAND="$YAI_BIN runtime serve --workers $workers --max-active-per-tenant $active --max-queued-per-tenant $tenant_queue --max-queued-total $global_queue $*"
  "$YAI_BIN" runtime serve --workers "$workers" \
    --max-active-per-tenant "$active" \
    --max-queued-per-tenant "$tenant_queue" \
    --max-queued-total "$global_queue" "$@" >"$SERVICE_LOG" 2>&1 &
  SERVICE_PID=$!
  for _ in $(seq 1 200); do
    local status
    status=$("$YAI_BIN" runtime status 2>/dev/null || true)
    if grep -Fq 'state: Running' <<<"$status"; then return 0; fi
    kill -0 "$SERVICE_PID" 2>/dev/null || break
    sleep 0.02
  done
  sed -n '1,160p' "$SERVICE_LOG" >&2 || true
  return 1
}

stop_service() {
  local output
  output=$("$YAI_BIN" runtime stop)
  trace_product stop "$YAI_BIN runtime stop" "$output" 0
  wait "$SERVICE_PID"
  SERVICE_PID=""
  local service_output
  service_output=$(sed -n '1,200p' "$SERVICE_LOG")
  trace_product "serve-$SERVICE_NAME" "$SERVICE_COMMAND" "$service_output" 0
}

submit_work() {
  local name="$1" tenant_id="$2" case_id="$3" request_id="$4" prompt="$5"
  shift 5
  local output
  output=$(YAI_JOURNAL="${JOURNALS[$name]}" "$YAI_BIN" runtime submit \
    --tenant "$tenant_id" --case "$case_id" --subject subject:llm-provider \
    --attachment workspace --prompt "$prompt" --idempotency-key "$request_id" \
    --max-invocations 3 --max-operations 2 "$@")
  LAST_WORK_ID=$(sed -n 's/^work_id: //p' <<<"$output" | head -1)
  [[ "$LAST_WORK_ID" == runtime-work:* ]]
  trace_product submit "$YAI_BIN runtime submit --tenant $tenant_id --case $case_id --idempotency-key $request_id" "$output" 0
}

work_state() {
  local work_id="$1" queue
  queue=$("$YAI_BIN" runtime queue)
  awk -v wanted="$work_id" '
    $1 == "work_id:" { found = ($2 == wanted) }
    found && $1 == "state:" { print $2; exit }
  ' <<<"$queue"
}

wait_work_state() {
  local work_id="$1" expected="$2"
  for _ in $(seq 1 600); do
    [[ "$(work_state "$work_id")" == "$expected" ]] && return 0
    sleep 0.02
  done
  "$YAI_BIN" runtime queue >&2 || true
  return 1
}

mkdir -p "$YAI_HOME" "$TEST_DIR/daemon-user" "$TEST_DIR/seed-home"
HOME="$TEST_DIR/daemon-user" "$YAID" --socket "$SOCKET" --foreground \
  >"$TEST_DIR/yaid.log" 2>&1 &
DAEMON_PID=$!
for _ in $(seq 1 100); do
  [[ -S "$SOCKET" ]] && break
  sleep 0.02
done
[[ -S "$SOCKET" ]]
loop_output=$(YAI_HOME="$TEST_DIR/seed-home" "$YAI_BIN" daemon run-filesystem-loop --socket "$SOCKET")
source_journal=$(sed -n 's/.*"journal_path":"\([^"]*\)".*/\1/p' <<<"$loop_output")
cp "$ROOT/$source_journal" "$BASE_JOURNAL"
"$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null
wait "$DAEMON_PID"
DAEMON_PID=""

bootstrap_a=$("$YAI_BIN" security bootstrap-local \
  --tenant tenant:w13-a --organization organization:characterization)
bootstrap_b=$("$YAI_BIN" security bootstrap-local \
  --tenant tenant:w13-b --organization organization:characterization)
principal_id=$(sed -n 's/^principal_id: //p' <<<"$bootstrap_a" | head -1)
[[ "$principal_id" == principal:* ]]
trace_product bootstrap-a "$YAI_BIN security bootstrap-local --tenant tenant:w13-a --organization organization:characterization" "$bootstrap_a" 0
trace_product bootstrap-b "$YAI_BIN security bootstrap-local --tenant tenant:w13-b --organization organization:characterization" "$bootstrap_b" 0

# The first phase proves real overlap, Tenant round-robin, FIFO, active quotas,
# per-Tenant/global backpressure, same-Case serialization, and split-brain refusal.
start_provider delay_complete 2 a1 1500; port_a1="$LAST_PROVIDER_PORT"
start_provider delay_complete 1 a2 1500; port_a2="$LAST_PROVIDER_PORT"
start_provider delay_complete 2 b1 1500; port_b1="$LAST_PROVIDER_PORT"
setup_case a1 case:w13-a1 tenant:w13-a "$port_a1" "$TEST_DIR/resources/a1"
setup_case a2 case:w13-a2 tenant:w13-a "$port_a2" "$TEST_DIR/resources/a2"
setup_case b1 case:w13-b1 tenant:w13-b "$port_b1" "$TEST_DIR/resources/b1"
start_service phase1 2 1 2 3

set +e
split_brain=$("$YAI_BIN" runtime serve --workers 1 --max-active-per-tenant 1 \
  --max-queued-per-tenant 1 --max-queued-total 1 2>&1)
split_brain_exit=$?
set -e
[[ "$split_brain_exit" -ne 0 ]]
grep -Fq 'runtime_instance_active' <<<"$split_brain"
trace_product split-brain "$YAI_BIN runtime serve --workers 1 ..." "$split_brain" "$split_brain_exit"

submit_work a1 tenant:w13-a case:w13-a1 request:a1-first 'complete Case A1 after the delayed provider turn'; work_a1_first="$LAST_WORK_ID"
submit_work b1 tenant:w13-b case:w13-b1 request:b1-first 'complete Case B1 after the delayed provider turn'; work_b1_first="$LAST_WORK_ID"
wait_work_state "$work_a1_first" Running
wait_work_state "$work_b1_first" Running

set +e
cross_tenant=$(YAI_JOURNAL="${JOURNALS[a1]}" "$YAI_BIN" runtime submit \
  --tenant tenant:w13-b --case case:w13-a1 --subject subject:llm-provider \
  --attachment workspace --prompt forged --idempotency-key request:cross 2>&1)
cross_tenant_exit=$?
set -e
[[ "$cross_tenant_exit" -ne 0 ]]
grep -Fq 'runtime_work_security_domain_mismatch' <<<"$cross_tenant"
trace_product cross-tenant "$YAI_BIN runtime submit --tenant tenant:w13-b --case case:w13-a1 ..." "$cross_tenant" "$cross_tenant_exit"

submit_work a2 tenant:w13-a case:w13-a2 request:a2-first 'Tenant A second Case'; work_a2_first="$LAST_WORK_ID"
submit_work a1 tenant:w13-a case:w13-a1 request:a1-second 'same Case second intentional task'; work_a1_second="$LAST_WORK_ID"
set +e
tenant_full=$(YAI_JOURNAL="${JOURNALS[a2]}" "$YAI_BIN" runtime submit \
  --tenant tenant:w13-a --case case:w13-a2 --subject subject:llm-provider \
  --attachment workspace --prompt overflow --idempotency-key request:tenant-overflow 2>&1)
tenant_full_exit=$?
set -e
[[ "$tenant_full_exit" -ne 0 ]]
grep -Fq 'runtime_tenant_queue_capacity_exhausted' <<<"$tenant_full"
trace_product tenant-backpressure "$YAI_BIN runtime submit --tenant tenant:w13-a ..." "$tenant_full" "$tenant_full_exit"

submit_work b1 tenant:w13-b case:w13-b1 request:b1-second 'Tenant B second FIFO task'; work_b1_second="$LAST_WORK_ID"
set +e
global_full=$(YAI_JOURNAL="${JOURNALS[a2]}" "$YAI_BIN" runtime submit \
  --tenant tenant:w13-a --case case:w13-a2 --subject subject:llm-provider \
  --attachment workspace --prompt global-overflow --idempotency-key request:global-overflow 2>&1)
global_full_exit=$?
set -e
[[ "$global_full_exit" -ne 0 ]]
grep -Fq 'runtime_global_queue_capacity_exhausted' <<<"$global_full"
trace_product global-backpressure "$YAI_BIN runtime submit --tenant tenant:w13-a ..." "$global_full" "$global_full_exit"

submit_work a2 tenant:w13-a case:w13-a2 request:a2-first 'Tenant A second Case'; repeated_a2="$LAST_WORK_ID"
[[ "$repeated_a2" == "$work_a2_first" ]]
for work_id in "$work_a1_first" "$work_b1_first" "$work_a2_first" "$work_a1_second" "$work_b1_second"; do
  wait_work_state "$work_id" Completed
done
phase1_status=$("$YAI_BIN" runtime status)
trace_product phase1-status "$YAI_BIN runtime status" "$phase1_status" 0
stop_service

python3 - "$TEST_DIR/a1.log.json" "$TEST_DIR/b1.log.json" "$SERVICE_LOG" \
  "$work_a1_first" "$work_a1_second" "$work_b1_first" "$work_a2_first" <<'PY'
import json, re, sys
a, b = json.load(open(sys.argv[1])), json.load(open(sys.argv[2]))
assert a[0]["started_at_unix_ms"] < b[0]["completed_at_unix_ms"]
assert b[0]["started_at_unix_ms"] < a[0]["completed_at_unix_ms"]
events = open(sys.argv[3]).read().splitlines()
starts, stops, dispatch = {}, {}, []
for line in events:
    if line.startswith("runtime_dispatch:"):
        dispatch.append(re.search(r"work_id=([^ ]+)", line).group(1))
    if line.startswith("runtime_worker_event:"):
        work = re.search(r"work_id=([^ ]+)", line).group(1)
        stamp = int(re.search(r"timestamp_unix_ms=(\d+)", line).group(1))
        (starts if " started " in line else stops)[work] = stamp
assert starts[sys.argv[5]] >= stops[sys.argv[4]]
assert starts[sys.argv[7]] >= stops[sys.argv[4]]
assert dispatch.index(sys.argv[6]) < dispatch.index(sys.argv[7])
assert dispatch.index(sys.argv[7]) < dispatch.index(sys.argv[5])
print("concurrency_intervals:", json.dumps({"tenant_a": a[0], "tenant_b": b[0]}, sort_keys=True))
print("dispatch_order:", " ".join(dispatch))
print("same_case_max_active: 1")
print("tenant_active_limit_observed: 1")
PY

# With the Tenant active limit raised, known overlapping filesystem roots are
# still conservatively serialized by the scheduler (not fenced).
start_provider delay_complete 1 a1-overlap-phase 800; port_a1_overlap="$LAST_PROVIDER_PORT"
provider_attach case:w13-a1 "${JOURNALS[a1]}" a1-overlap-phase "$port_a1_overlap"
start_provider delay_complete 1 overlap 800; port_overlap="$LAST_PROVIDER_PORT"
mkdir -p "${ROOTS[a1]}/nested"
setup_case overlap case:w13-overlap tenant:w13-a "$port_overlap" "${ROOTS[a1]}/nested"
start_service overlap-phase 2 2 4 8
submit_work a1 tenant:w13-a case:w13-a1 request:a1-overlap-phase 'parent root work'; overlap_parent="$LAST_WORK_ID"
submit_work overlap tenant:w13-a case:w13-overlap request:overlap-child 'overlapping child root work'; overlap_child="$LAST_WORK_ID"
wait_work_state "$overlap_parent" Completed
wait_work_state "$overlap_child" Completed
grep -Fq 'serialized_due_to_resource_overlap_or_unknown_relation' "$SERVICE_LOG"
stop_service

# A review parks the WorkItem and releases its worker. Another Case completes;
# authenticated review then resumes the exact WorkItem and Operation.
start_provider review 2 review 0; port_review="$LAST_PROVIDER_PORT"
setup_case review case:w13-review tenant:w13-a "$port_review" "$TEST_DIR/resources/review" yes
start_provider delay_complete 1 b-review-peer 500; port_b_peer="$LAST_PROVIDER_PORT"
provider_attach case:w13-b1 "${JOURNALS[b1]}" b-review-peer "$port_b_peer"
start_service review-phase 2 2 4 8
submit_work review tenant:w13-a case:w13-review request:review 'propose reviewed filesystem work'; work_review="$LAST_WORK_ID"
submit_work b1 tenant:w13-b case:w13-b1 request:b-review-peer 'complete while review is parked'; work_peer="$LAST_WORK_ID"
wait_work_state "$work_review" WaitingReview
wait_work_state "$work_peer" Completed
pending_reviews=$("$YAI_BIN" review pending --case case:w13-review)
review_id=$(sed -n 's/^review_id: //p' <<<"$pending_reviews" | head -1)
[[ "$review_id" == review:* ]]
review_output=$("$YAI_BIN" review approve "$review_id" --case case:w13-review \
  --reason 'authenticated scheduler review qualification')
trace_product review "$YAI_BIN review approve $review_id --case case:w13-review --reason ..." "$review_output" 0
wait_work_state "$work_review" Completed
stop_service

# Crash the whole RuntimeInstance after a durable PREPARE. Restart reclaims the
# instance and Running WorkItem, reconciles physical truth, and consumes only
# the second provider turn.
start_provider proposal 2 recovery 0; port_recovery="$LAST_PROVIDER_PORT"
setup_case recovery case:w13-recovery tenant:w13-b "$port_recovery" "$TEST_DIR/resources/recovery"
start_service recovery-crash 2 2 4 8
submit_work recovery tenant:w13-b case:w13-recovery request:recovery \
  'write once then complete after recovery' --failpoint after_prepare_before_effect
work_recovery="$LAST_WORK_ID"
set +e
wait "$SERVICE_PID"
crash_exit=$?
set -e
SERVICE_PID=""
[[ "$crash_exit" -eq 85 ]]
grep -Fq 'controlled_effect_crash_injected: after_prepare_before_effect' "$SERVICE_LOG"
crash_output=$(sed -n '1,200p' "$SERVICE_LOG")
trace_product serve-recovery-crash "$SERVICE_COMMAND" "$crash_output" "$crash_exit"
[[ "$(work_state "$work_recovery")" == Running ]]

start_service recovery-restart 2 2 4 8
grep -Fq 'instance_admission: reclaimed_stale' "$SERVICE_LOG"
grep -Eq 'recovered_items: [1-9]' "$SERVICE_LOG"
wait_work_state "$work_recovery" Completed
[[ "$(cat "${ROOTS[recovery]}/allowed/step-00.txt")" == 'runtime step 00' ]]
stop_service

for pid in "${PROVIDER_PIDS[@]}"; do wait "$pid"; done
PROVIDER_PIDS=()

recovery_effect_id=$(sed -n 's/^effect_id: //p' "$TEST_DIR/recovery-crash.service.log" | head -1)
[[ "$recovery_effect_id" == effect:* ]]
printf 'multi_case_runtime_characterization: pass\n'
printf 'test_run_id: %s\n' "$TEST_DIR"
printf 'principal_id: %s\n' "$principal_id"
printf 'runtime_instance_id: runtime-instance:local-default\n'
printf 'workers_max_observed: 2\n'
printf 'tenant_queue_rejection_count: 1\n'
printf 'global_queue_rejection_count: 1\n'
printf 'cross_tenant_rejection_count: 1\n'
printf 'split_brain_exit: %s\n' "$split_brain_exit"
printf 'crash_exit: %s\n' "$crash_exit"
printf 'review_id: %s\n' "$review_id"
printf 'review_work_id: %s\n' "$work_review"
printf 'recovered_work_id: %s\n' "$work_recovery"
printf 'recovery_effect_id: %s\n' "$recovery_effect_id"
