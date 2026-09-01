#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
YAID="$ROOT/build/yaid"
FIXTURE="$ROOT/tests/fixtures/controlled_effect_provider.py"
RUNTIME_FIXTURE="$ROOT/tests/fixtures/agentless_case_runtime_provider.py"
source "$ROOT/tests/characterization/lib/governed_case_policy.sh"

TEST_DIR="$(mktemp -d /tmp/yai-wave14-cross-process.XXXXXX)"
export YAI_HOME="$TEST_DIR/home"
SOCKET="$TEST_DIR/yaid.sock"
BASE_JOURNAL="$TEST_DIR/base.jsonl"
SHARED_ROOT="$TEST_DIR/shared-resource"
DAEMON_PID=""
SERVICE_PID=""
PROVIDER_PIDS=()
declare -A JOURNALS PROVIDER_PORTS

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
    printf 'preserved_test_dir: %s\n' "$TEST_DIR" >&2
  fi
}
trap cleanup EXIT INT TERM

start_provider() {
  local name="$1"
  local port_file="$TEST_DIR/$1.port"
  if [[ "$name" == scheduled ]]; then
    python3 "$RUNTIME_FIXTURE" proposal 2 >"$port_file" &
  else
    python3 "$FIXTURE" allow_once >"$port_file" &
  fi
  local pid=$!
  PROVIDER_PIDS+=("$pid")
  for _ in $(seq 1 100); do
    [[ -s "$port_file" ]] && break
    sleep 0.02
  done
  [[ -s "$port_file" ]]
  PROVIDER_PORTS[$name]="$(tr -d '[:space:]' <"$port_file")"
}

setup_case() {
  local name="$1"
  local case_id="$2"
  local journal="$TEST_DIR/$name.jsonl"
  sed -e "s/case:new12-filesystem/$case_id/g" \
    -e "s/new12-fs/w14-$name/g" "$BASE_JOURNAL" >"$journal"
  "$YAI_BIN" case create --case "$case_id" --tenant tenant:wave14 >/dev/null
  "$YAI_BIN" case bind-participant-role --case "$case_id" \
    --participant subject:policy-pack \
    --role resource-attachment-compatibility-owner >/dev/null
  YAI_JOURNAL="$journal" "$YAI_BIN" case enter \
    --case "$case_id" --subject subject:llm-provider >/dev/null
  YAI_JOURNAL="$journal" "$YAI_BIN" case attach-provider \
    --case "$case_id" --subject subject:llm-provider \
    --provider-id "provider:$name" \
    --base-url "http://127.0.0.1:${PROVIDER_PORTS[$name]}/v1/chat/completions" \
    --model controlled-model >/dev/null
  "$YAI_BIN" case attach-filesystem --case "$case_id" --attachment workspace \
    --root "$SHARED_ROOT" --allow-prefix allowed \
    --policy-owner subject:policy-pack --max-bytes 256 >/dev/null
  YAI_TEST_TENANT_ID=tenant:wave14 yai_configure_governed_filesystem_case \
    "$YAI_BIN" "$YAI_HOME" "$case_id" "wave14-$name" 1 allow \
    subject:llm-provider >/dev/null
  JOURNALS[$name]="$journal"
}

run_direct() {
  local name="$1" case_id="$2"
  shift 2
  YAI_JOURNAL="${JOURNALS[$name]}" "$YAI_BIN" effect filesystem-write \
    --case "$case_id" --subject subject:llm-provider --attachment workspace \
    --prompt "propose the fenced shared-resource write" \
    --provider-id "provider:$name" \
    --base-url "http://127.0.0.1:${PROVIDER_PORTS[$name]}/v1/chat/completions" \
    --model controlled-model "$@"
}

mkdir -p "$YAI_HOME" "$TEST_DIR/daemon-user" "$TEST_DIR/seed-home" \
  "$SHARED_ROOT/allowed"
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

"$YAI_BIN" security bootstrap-local --tenant tenant:wave14 \
  --organization organization:characterization >/dev/null
start_provider direct
start_provider scheduled
start_provider direct-peer
setup_case direct case:wave14-direct
setup_case scheduled case:wave14-scheduled
setup_case direct-peer case:wave14-direct-peer

# The independent direct process wins PREPARE and then exits. Its unresolved
# effect remains the resource owner even though its process is gone.
set +e
run_direct direct case:wave14-direct --failpoint after_prepare_before_effect \
  >"$TEST_DIR/direct-prepare.out" 2>&1
direct_exit=$?
set -e
[[ "$direct_exit" -eq 85 ]]
wait "${PROVIDER_PIDS[0]}"
grep -Fq 'resource_epoch: 1' "$TEST_DIR/direct-prepare.out"
grep -Fq 'effect_state: prepared_durable_before_mutation' "$TEST_DIR/direct-prepare.out"
[[ ! -e "$SHARED_ROOT/allowed/hello.txt" ]]
[[ ! -e "$SHARED_ROOT/allowed/step-00.txt" ]]

# A RuntimeInstance worker is a different process and cannot see the direct
# run through scheduler-local serialization. Shared LMDB resource authority
# rejects its PREPARE before physical mutation.
"$YAI_BIN" runtime serve --workers 1 --max-active-per-tenant 1 \
  --max-queued-per-tenant 2 --max-queued-total 2 \
  >"$TEST_DIR/runtime.service.log" 2>&1 &
SERVICE_PID=$!
for _ in $(seq 1 200); do
  status=$("$YAI_BIN" runtime status 2>/dev/null || true)
  grep -Fq 'state: Running' <<<"$status" && break
  sleep 0.02
done
submit=$(YAI_JOURNAL="${JOURNALS[scheduled]}" "$YAI_BIN" runtime submit \
  --tenant tenant:wave14 --case case:wave14-scheduled \
  --subject subject:llm-provider --attachment workspace \
  --prompt "attempt the same shared resource" \
  --idempotency-key request:wave14-cross-process \
  --max-invocations 2 --max-operations 2)
work_id=$(sed -n 's/^work_id: //p' <<<"$submit")
[[ "$work_id" == runtime-work:* ]]
for _ in $(seq 1 300); do
  queue=$("$YAI_BIN" runtime queue)
  state=$(awk -v wanted="$work_id" '
    $1 == "work_id:" { found = ($2 == wanted) }
    found && $1 == "state:" { print $2; exit }
  ' <<<"$queue")
  [[ "$state" == Blocked ]] && break
  sleep 0.02
done
[[ "$state" == Blocked ]]
blocked_state="$state"
grep -Fq 'resource_temporarily_owned' <<<"$queue"
[[ ! -e "$SHARED_ROOT/allowed/hello.txt" ]]
[[ ! -e "$SHARED_ROOT/allowed/step-00.txt" ]]

# A third, separately invoked direct YAI process is subject to the same shared
# owner. This proves exclusion does not depend on RuntimeInstance code at all.
set +e
run_direct direct-peer case:wave14-direct-peer \
  >"$TEST_DIR/direct-peer.out" 2>&1
direct_peer_exit=$?
set -e
[[ "$direct_peer_exit" -eq 2 ]]
wait "${PROVIDER_PIDS[2]}"
grep -Fq 'resource_temporarily_owned' "$TEST_DIR/direct-peer.out"
[[ ! -e "$SHARED_ROOT/allowed/hello.txt" ]]
[[ ! -e "$SHARED_ROOT/allowed/step-00.txt" ]]

# Only the same unresolved effect may reclaim. Reconciliation advances its
# fence epoch and commits the visible effect plus terminal resource release.
effect_id=$(sed -n 's/^effect_id: //p' "$TEST_DIR/direct-prepare.out" | head -1)
reconcile=$("$YAI_BIN" effect reconcile --case case:wave14-direct \
  --effect "$effect_id" --retry)
grep -Fq 'reconciliation: EffectObserved' <<<"$reconcile"
[[ "$(cat "$SHARED_ROOT/allowed/hello.txt")" == "hello from controlled YAI" ]]

# Resource release is a meaningful retry trigger. The parked WorkItem retains
# its identity, revalidates current authority and completes without busy spin.
for _ in $(seq 1 500); do
  queue=$("$YAI_BIN" runtime queue)
  state=$(awk -v wanted="$work_id" '
    $1 == "work_id:" { found = ($2 == wanted) }
    found && $1 == "state:" { print $2; exit }
  ' <<<"$queue")
  [[ "$state" == Completed ]] && break
  sleep 0.02
done
[[ "$state" == Completed ]]
[[ "$(cat "$SHARED_ROOT/allowed/step-00.txt")" == "runtime step 00" ]]
"$YAI_BIN" runtime stop >/dev/null
wait "$SERVICE_PID"
SERVICE_PID=""
wait "${PROVIDER_PIDS[1]}"

printf 'cross_process_resource_fencing: pass\n'
printf 'test_run_id: %s\n' "$TEST_DIR"
printf 'direct_exit: %s\n' "$direct_exit"
printf 'direct_effect_id: %s\n' "$effect_id"
sed -n -E '/^(resource_id|resource_epoch|resource_fence_id):/p' "$TEST_DIR/direct-prepare.out"
printf 'runtime_work_id: %s\n' "$work_id"
printf 'runtime_work_initial_state: %s\n' "$blocked_state"
printf 'runtime_work_final_state: %s\n' "$state"
printf 'runtime_block_reason: resource_temporarily_owned\n'
printf 'runtime_retry_trigger: terminal_resource_release\n'
printf 'direct_peer_exit: %s\n' "$direct_peer_exit"
printf 'direct_peer_block_reason: resource_temporarily_owned\n'
printf 'physical_mutations_before_reconcile: 0\n'
printf 'physical_mutations_after_reconcile: 1\n'
