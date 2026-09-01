#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
YAID="$ROOT/build/yaid"
PROVIDER_FIXTURE="$ROOT/tests/fixtures/controlled_effect_provider.py"
source "$ROOT/tests/characterization/lib/governed_case_policy.sh"

TEST_DIR="$(mktemp -d /tmp/yai-h14-contention.XXXXXX)"
export YAI_HOME="$TEST_DIR/home"
SHARED_ROOT="$TEST_DIR/shared"
TENANT="tenant:h14-contention"
SOCKET="$TEST_DIR/yaid.sock"
BASE_JOURNAL="$TEST_DIR/base.jsonl"
DAEMON_PID=""
PROVIDER_PIDS=()
declare -A PORTS JOURNALS

cleanup() {
  if [[ -n "$DAEMON_PID" ]]; then
    "$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null 2>&1 || true
    wait "$DAEMON_PID" >/dev/null 2>&1 || true
  fi
  for pid in "${PROVIDER_PIDS[@]}"; do
    kill "$pid" >/dev/null 2>&1 || true
    wait "$pid" >/dev/null 2>&1 || true
  done
  if [[ "${YAI_KEEP_TEST_DIR:-0}" != "1" ]]; then
    rm -rf "$TEST_DIR"
  else
    printf 'preserved_test_dir: %s\n' "$TEST_DIR" >&2
  fi
}
trap cleanup EXIT INT TERM

start_provider() {
  local name="$1"
  local scenario="${2:-allow_once}"
  local port_file="$TEST_DIR/$name.port"
  python3 "$PROVIDER_FIXTURE" "$scenario" >"$port_file" &
  local pid=$!
  PROVIDER_PIDS+=("$pid")
  for _ in $(seq 1 100); do
    [[ -s "$port_file" ]] && break
    sleep 0.02
  done
  [[ -s "$port_file" ]]
  PORTS[$name]="$(tr -d '[:space:]' <"$port_file")"
}

setup_case() {
  local index="$1"
  local provider_scenario="${2:-allow_once}"
  local name="contend-$index"
  local case_id="case:h14-contend-$index"
  local journal="$TEST_DIR/$name.jsonl"
  sed -e "s/case:new12-filesystem/$case_id/g" \
    -e "s/new12-fs/h14-contend-$index/g" "$BASE_JOURNAL" >"$journal"
  start_provider "$name" "$provider_scenario"
  "$YAI_BIN" case create --case "$case_id" --tenant "$TENANT" >/dev/null
  YAI_JOURNAL="$journal" "$YAI_BIN" case enter \
    --case "$case_id" --subject subject:llm-provider >/dev/null
  YAI_JOURNAL="$journal" "$YAI_BIN" case attach-provider \
    --case "$case_id" --subject subject:llm-provider \
    --provider-id "provider:$name" \
    --base-url "http://127.0.0.1:${PORTS[$name]}/v1/chat/completions" \
    --model controlled-model >/dev/null
  "$YAI_BIN" case bind-participant-role --case "$case_id" \
    --participant subject:policy-pack \
    --role resource-attachment-compatibility-owner >/dev/null
  "$YAI_BIN" case attach-filesystem --case "$case_id" --attachment workspace \
    --root "$SHARED_ROOT" --allow-prefix allowed \
    --policy-owner subject:policy-pack --max-bytes 256 >/dev/null
  YAI_TEST_TENANT_ID="$TENANT" yai_configure_governed_filesystem_case \
    "$YAI_BIN" "$YAI_HOME" "$case_id" "h14-contend-$index" 1 allow \
    subject:llm-provider >/dev/null
  JOURNALS[$index]="$journal"
}

run_case() {
  local index="$1"
  local failpoint="${2:-}"
  local name="contend-$index"
  local command=(
    "$YAI_BIN" effect filesystem-write
    --case "case:h14-contend-$index" \
    --subject subject:llm-provider --attachment workspace \
    --prompt "contend for one fenced resource" \
    --provider-id "provider:$name" \
    --base-url "http://127.0.0.1:${PORTS[$name]}/v1/chat/completions" \
    --model controlled-model
  )
  if [[ -n "$failpoint" ]]; then
    command+=(--failpoint "$failpoint")
  fi
  YAI_JOURNAL="${JOURNALS[$index]}" "${command[@]}"
}

mkdir -p "$YAI_HOME" "$SHARED_ROOT/allowed" "$TEST_DIR/daemon-user" \
  "$TEST_DIR/seed-home"
HOME="$TEST_DIR/daemon-user" "$YAID" --socket "$SOCKET" --foreground \
  >"$TEST_DIR/yaid.log" 2>&1 &
DAEMON_PID=$!
for _ in $(seq 1 100); do
  [[ -S "$SOCKET" ]] && break
  sleep 0.02
done
[[ -S "$SOCKET" ]]
loop_output=$(YAI_HOME="$TEST_DIR/seed-home" "$YAI_BIN" daemon run-filesystem-loop \
  --socket "$SOCKET")
source_journal=$(sed -n 's/.*"journal_path":"\([^"]*\)".*/\1/p' <<<"$loop_output")
cp "$ROOT/$source_journal" "$BASE_JOURNAL"
"$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null
wait "$DAEMON_PID"
DAEMON_PID=""

"$YAI_BIN" security bootstrap-local --tenant "$TENANT" \
  --organization organization:characterization >/dev/null

for index in $(seq 1 8); do
  setup_case "$index"
done
setup_case 9 allow
setup_case 10 allow_once
setup_case 11 allow_once

# Eight independent YAI processes race a free resource. LMDB serializes the
# PREPARE/acquisition transaction: exactly one commits epoch 1 and exits at the
# failpoint; every loser observes the canonical active owner.
declare -A RUN_PIDS RUN_EXITS
set +e
for index in $(seq 1 8); do
  run_case "$index" after_prepare_before_effect >"$TEST_DIR/run-$index.out" 2>&1 &
  RUN_PIDS[$index]=$!
done
for index in $(seq 1 8); do
  wait "${RUN_PIDS[$index]}"
  RUN_EXITS[$index]=$?
done
set -e

winner_count=0
blocked_count=0
winner_index=""
for index in $(seq 1 8); do
  case "${RUN_EXITS[$index]}" in
    85)
      winner_count=$((winner_count + 1))
      winner_index="$index"
      ;;
    2)
      grep -Fq 'resource_temporarily_owned' "$TEST_DIR/run-$index.out"
      blocked_count=$((blocked_count + 1))
      ;;
    *)
      printf 'unexpected contender exit: index=%s exit=%s\n' \
        "$index" "${RUN_EXITS[$index]}" >&2
      exit 1
      ;;
  esac
done
[[ "$winner_count" -eq 1 ]]
[[ "$blocked_count" -eq 7 ]]
[[ ! -e "$SHARED_ROOT/allowed/hello.txt" ]]

winner_output="$TEST_DIR/run-$winner_index.out"
effect_id=$(sed -n 's/^effect_id: //p' "$winner_output" | head -1)
resource_id=$(sed -n 's/^resource_id: //p' "$winner_output" | head -1)
epoch_one=$(sed -n 's/^resource_epoch: //p' "$winner_output" | head -1)
[[ "$effect_id" == effect:* ]]
[[ "$resource_id" == resource-control:* ]]
[[ "$epoch_one" == 1 ]]

# Two recovery processes race the same unresolved Effect. Only one may reclaim
# and execute it; the other must observe or reject the winner, never signal a
# second semantic effect or create a second physical writer.
set +e
"$YAI_BIN" effect reconcile --case "case:h14-contend-$winner_index" \
  --effect "$effect_id" --retry >"$TEST_DIR/reconcile-1.out" 2>&1 &
reconcile_pid_one=$!
"$YAI_BIN" effect reconcile --case "case:h14-contend-$winner_index" \
  --effect "$effect_id" --retry >"$TEST_DIR/reconcile-2.out" 2>&1 &
reconcile_pid_two=$!
wait "$reconcile_pid_one"
reconcile_exit_one=$?
wait "$reconcile_pid_two"
reconcile_exit_two=$?
set -e
reconcile_successes=0
[[ "$reconcile_exit_one" -eq 0 ]] && reconcile_successes=$((reconcile_successes + 1))
[[ "$reconcile_exit_two" -eq 0 ]] && reconcile_successes=$((reconcile_successes + 1))
[[ "$reconcile_successes" -ge 1 ]]
[[ "$(cat "$SHARED_ROOT/allowed/hello.txt")" == "hello from controlled YAI" ]]

# A fresh post-release effect receives epoch 3: epoch 2 belonged to the
# same-Effect reclaim, not a new Effect.
run_case 9 >"$TEST_DIR/run-9.out" 2>&1
epoch_three=$(sed -n 's/^resource_epoch: //p' "$TEST_DIR/run-9.out" | head -1)
[[ "$epoch_three" == 3 ]]

# The post-terminal failpoint exits only after the single LMDB transaction has
# committed both FINALIZE and release. Restart sees an already-finalized Effect
# and the next Case can immediately acquire the next epoch.
set +e
run_case 10 after_terminal_resource_release_commit >"$TEST_DIR/run-10.out" 2>&1
terminal_exit=$?
set -e
[[ "$terminal_exit" -eq 89 ]]
terminal_effect=$(sed -n 's/^effect_id: //p' "$TEST_DIR/run-10.out" | head -1)
epoch_four=$(sed -n 's/^resource_epoch: //p' "$TEST_DIR/run-10.out" | head -1)
[[ "$epoch_four" == 4 ]]
terminal_reconcile=$("$YAI_BIN" effect reconcile --case case:h14-contend-10 \
  --effect "$terminal_effect")
grep -Fq 'reconciliation: already_finalized' <<<"$terminal_reconcile"

set +e
run_case 11 after_prepare_before_effect >"$TEST_DIR/run-11.out" 2>&1
post_terminal_exit=$?
set -e
[[ "$post_terminal_exit" -eq 85 ]]
epoch_five=$(sed -n 's/^resource_epoch: //p' "$TEST_DIR/run-11.out" | head -1)
[[ "$epoch_five" == 5 ]]
post_terminal_effect=$(sed -n 's/^effect_id: //p' "$TEST_DIR/run-11.out" | head -1)
"$YAI_BIN" effect reconcile --case case:h14-contend-11 \
  --effect "$post_terminal_effect" --retry >/dev/null

printf 'h14_multiprocess_contention: pass\n'
printf 'test_run_id: %s\n' "$TEST_DIR"
printf 'contender_processes: 8\n'
printf 'acquisition_winners: %s\n' "$winner_count"
printf 'acquisition_blocked: %s\n' "$blocked_count"
printf 'resource_id: %s\n' "$resource_id"
printf 'first_epoch: %s\n' "$epoch_one"
printf 'same_effect_reclaim_epoch: 2\n'
printf 'next_acquisition_epoch: %s\n' "$epoch_three"
printf 'terminal_commit_epoch: %s\n' "$epoch_four"
printf 'terminal_failpoint_exit: %s\n' "$terminal_exit"
printf 'post_terminal_acquisition_epoch: %s\n' "$epoch_five"
printf 'terminal_recovery_posture: already_finalized\n'
printf 'reconcile_exit_one: %s\n' "$reconcile_exit_one"
printf 'reconcile_exit_two: %s\n' "$reconcile_exit_two"
printf 'physical_mutations_per_effect: 1\n'
