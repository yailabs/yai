#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
YAID="$ROOT/build/yaid"
PROVIDER="$ROOT/tests/fixtures/agentless_case_runtime_provider.py"
source "$ROOT/tests/characterization/lib/governed_case_policy.sh"

TEST_DIR="$(mktemp -d /tmp/yai-h13-terminal-ack.XXXXXX)"
export YAI_HOME="$TEST_DIR/home"
SOCKET="$TEST_DIR/yaid.sock"
SERVICE_PID=""
PROVIDER_PID=""
PROVIDER_PIDS=()
DAEMON_PID=""
BASE_JOURNAL="$TEST_DIR/base.jsonl"
LAST_PROVIDER_PORT=""

cleanup() {
  [[ -z "$SERVICE_PID" ]] || kill "$SERVICE_PID" >/dev/null 2>&1 || true
  for provider_pid in "${PROVIDER_PIDS[@]}"; do
    kill "$provider_pid" >/dev/null 2>&1 || true
    wait "$provider_pid" >/dev/null 2>&1 || true
  done
  if [[ -n "$DAEMON_PID" ]]; then
    "$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null 2>&1 || true
    wait "$DAEMON_PID" >/dev/null 2>&1 || true
  fi
  if [[ "${YAI_KEEP_TEST_DIR:-0}" == "1" ]]; then
    printf 'preserved_test_dir:%s\n' "$TEST_DIR" >&2
  else
    rm -rf "$TEST_DIR"
  fi
}
trap cleanup EXIT INT TERM

start_provider() {
  local mode="$1" expected="$2" name="$3"
  YAI_CASE_RUNTIME_PROVIDER_LOG="$TEST_DIR/$name.provider.json" \
    python3 "$PROVIDER" "$mode" "$expected" >"$TEST_DIR/$name.provider.port" &
  PROVIDER_PID=$!
  PROVIDER_PIDS+=("$PROVIDER_PID")
  for _ in $(seq 1 100); do
    [[ -s "$TEST_DIR/$name.provider.port" ]] && break
    sleep 0.02
  done
  LAST_PROVIDER_PORT=$(tr -d '[:space:]' <"$TEST_DIR/$name.provider.port")
}

setup_case() {
  local name="$1" case_id="$2" tenant_id="$3" port="$4" review="${5:-no}"
  local journal="$TEST_DIR/$name/journal.jsonl" root="$TEST_DIR/$name/resource"
  local -a attach
  mkdir -p "$TEST_DIR/$name" "$root/allowed"
  sed -e "s/case:new12-filesystem/$case_id/g" -e "s/new12-fs/h13-$name/g" \
    "$BASE_JOURNAL" >"$journal"
  "$YAI_BIN" case create --case "$case_id" --tenant "$tenant_id" >/dev/null
  "$YAI_BIN" case bind-participant-role --case "$case_id" \
    --participant subject:policy-pack --role resource-attachment-compatibility-owner >/dev/null
  YAI_JOURNAL="$journal" "$YAI_BIN" case enter --case "$case_id" \
    --subject subject:llm-provider >/dev/null
  YAI_JOURNAL="$journal" "$YAI_BIN" case attach-provider --case "$case_id" \
    --subject subject:llm-provider --provider-id "provider:$name" \
    --base-url "http://127.0.0.1:$port/v1/chat/completions" --model "model-$name" >/dev/null
  attach=(case attach-filesystem --case "$case_id" --attachment workspace \
    --root "$root" --allow-prefix allowed --policy-owner subject:policy-pack --max-bytes 256)
  [[ "$review" != yes ]] || attach+=(--require-review)
  "$YAI_BIN" "${attach[@]}" >/dev/null
  if [[ "$review" == yes ]]; then
    YAI_TEST_TENANT_ID="$tenant_id" yai_configure_governed_filesystem_case \
      "$YAI_BIN" "$YAI_HOME" "$case_id" "h13-$name" 1 allow \
      subject:llm-provider subject:policy-pack >/dev/null
    principal_id=$($YAI_BIN identity whoami --json | \
      python3 -c 'import json,sys; print(next(field["value"] for field in json.load(sys.stdin)["data"]["fields"] if field["name"] == "Principal"))')
    "$YAI_BIN" case principal link --case "$case_id" --principal "$principal_id" \
      --participant subject:policy-pack >/dev/null
  else
    YAI_TEST_TENANT_ID="$tenant_id" yai_configure_governed_filesystem_case \
      "$YAI_BIN" "$YAI_HOME" "$case_id" "h13-$name" 1 allow \
      subject:llm-provider >/dev/null
  fi
}

wait_state() {
  local work_id="$1" expected="$2"
  for _ in $(seq 1 400); do
    queue=$($YAI_BIN runtime queue 2>/dev/null || true)
    state=$(awk -v id="$work_id" '$1=="work_id:"{f=$2==id} f&&$1=="state:"{print $2;exit}' <<<"$queue")
    [[ "$state" == "$expected" ]] && return 0
    sleep 0.02
  done
  return 1
}

mkdir -p "$YAI_HOME" "$TEST_DIR/daemon-home" "$TEST_DIR/seed-home" "$TEST_DIR/case" "$TEST_DIR/resource/allowed"
HOME="$TEST_DIR/daemon-home" "$YAID" --socket "$SOCKET" --foreground >"$TEST_DIR/yaid.log" 2>&1 &
DAEMON_PID=$!
for _ in $(seq 1 100); do [[ -S "$SOCKET" ]] && break; sleep 0.02; done
[[ -S "$SOCKET" ]]
loop_output=$(YAI_HOME="$TEST_DIR/seed-home" "$YAI_BIN" daemon run-filesystem-loop --socket "$SOCKET")
source_journal=$(sed -n 's/.*"journal_path":"\([^"]*\)".*/\1/p' <<<"$loop_output")
cp "$ROOT/$source_journal" "$BASE_JOURNAL"
sed -e 's/case:new12-filesystem/case:h13-terminal/g' \
  -e 's/new12-fs/h13-terminal/g' "$BASE_JOURNAL" >"$TEST_DIR/case/journal.jsonl"
"$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null
wait "$DAEMON_PID"
DAEMON_PID=""

"$YAI_BIN" security bootstrap-local --tenant tenant:h13 --organization organization:characterization >/dev/null
YAI_CASE_RUNTIME_PROVIDER_LOG="$TEST_DIR/provider.json" \
  python3 "$PROVIDER" delay_complete 1 >"$TEST_DIR/provider.port" &
PROVIDER_PID=$!
PROVIDER_PIDS+=("$PROVIDER_PID")
for _ in $(seq 1 100); do [[ -s "$TEST_DIR/provider.port" ]] && break; sleep 0.02; done
port=$(tr -d '[:space:]' <"$TEST_DIR/provider.port")

"$YAI_BIN" case create --case case:h13-terminal --tenant tenant:h13 >/dev/null
"$YAI_BIN" case bind-participant-role --case case:h13-terminal \
  --participant subject:policy-pack --role resource-attachment-compatibility-owner >/dev/null
YAI_JOURNAL="$TEST_DIR/case/journal.jsonl" "$YAI_BIN" case enter \
  --case case:h13-terminal --subject subject:llm-provider >/dev/null
YAI_JOURNAL="$TEST_DIR/case/journal.jsonl" "$YAI_BIN" case attach-provider \
  --case case:h13-terminal --subject subject:llm-provider \
  --provider-id provider:h13 --base-url "http://127.0.0.1:$port/v1/chat/completions" \
  --model model-h13 >/dev/null
"$YAI_BIN" case attach-filesystem --case case:h13-terminal --attachment workspace \
  --root "$TEST_DIR/resource" --allow-prefix allowed \
  --policy-owner subject:policy-pack --max-bytes 256 >/dev/null
YAI_TEST_TENANT_ID=tenant:h13 yai_configure_governed_filesystem_case \
  "$YAI_BIN" "$YAI_HOME" case:h13-terminal h13-terminal 1 allow \
  subject:llm-provider >/dev/null

"$YAI_BIN" runtime serve --workers 1 --max-active-per-tenant 1 \
  --max-queued-per-tenant 4 --max-queued-total 4 \
  --failpoint after_case_runtime_terminal_before_workitem_terminal_commit \
  >"$TEST_DIR/crash.log" 2>&1 &
SERVICE_PID=$!
for _ in $(seq 1 200); do
  status=$($YAI_BIN runtime status 2>/dev/null || true)
  grep -Fq 'state: Running' <<<"$status" && break
  sleep 0.02
done
set +e
split_output=$("$YAI_BIN" runtime serve --workers 1 --max-active-per-tenant 1 \
  --max-queued-per-tenant 4 --max-queued-total 4 2>&1)
split_exit=$?
set -e
[[ "$split_exit" -ne 0 ]]
grep -Fq 'runtime_instance_active' <<<"$split_output"

sed 's/case:h13-terminal/case:h13-wrong/g' "$TEST_DIR/case/journal.jsonl" \
  >"$TEST_DIR/case/wrong-journal.jsonl"
set +e
wrong_journal_output=$(YAI_JOURNAL="$TEST_DIR/case/wrong-journal.jsonl" \
  "$YAI_BIN" runtime submit --tenant tenant:h13 --case case:h13-terminal \
  --subject subject:llm-provider --attachment workspace --prompt forged-journal \
  --idempotency-key request:h13-wrong-journal 2>&1)
wrong_journal_exit=$?
set -e
[[ "$wrong_journal_exit" -ne 0 ]]
grep -Fq 'journal_case_identity_mismatch' <<<"$wrong_journal_output"

submit=$(YAI_JOURNAL="$TEST_DIR/case/journal.jsonl" "$YAI_BIN" runtime submit \
  --tenant tenant:h13 --case case:h13-terminal --subject subject:llm-provider \
  --attachment workspace --prompt 'complete once' --idempotency-key request:h13-terminal \
  --max-invocations 2 --max-operations 1)
work_id=$(sed -n 's/^work_id: //p' <<<"$submit")
set +e
wait "$SERVICE_PID"
crash_exit=$?
set -e
SERVICE_PID=""
[[ "$crash_exit" -eq 122 ]]
before=$($YAI_BIN runtime queue)
grep -A12 -F "work_id: $work_id" <<<"$before" | grep -Fq 'state: Running'

"$YAI_BIN" runtime serve --workers 1 --max-active-per-tenant 1 \
  --max-queued-per-tenant 4 --max-queued-total 4 >"$TEST_DIR/restart.log" 2>&1 &
SERVICE_PID=$!
for _ in $(seq 1 300); do
  queue=$($YAI_BIN runtime queue)
  state=$(awk -v id="$work_id" '$1=="work_id:"{f=$2==id} f&&$1=="state:"{print $2;exit}' <<<"$queue")
  [[ "$state" == Completed ]] && break
  kill -0 "$SERVICE_PID" 2>/dev/null || break
  sleep 0.02
done
"$YAI_BIN" runtime stop >/dev/null 2>&1 || true
wait "$SERVICE_PID" >/dev/null 2>&1 || true
SERVICE_PID=""

provider_count=$(python3 - "$TEST_DIR/provider.json" <<'PY'
import json, sys
print(len(json.load(open(sys.argv[1]))))
PY
)
queue=$($YAI_BIN runtime queue)
final_state=$(awk -v id="$work_id" '$1=="work_id:"{f=$2==id} f&&$1=="state:"{print $2;exit}' <<<"$queue")
printf 'h13_terminal_ack_reproduction: provider_count=%s final_state=%s crash_exit=%s\n' \
  "$provider_count" "$final_state" "$crash_exit"
printf 'test_run_id: %s\n' "$TEST_DIR"
printf 'work_id: %s\n' "$work_id"
printf 'checkpoint_status: completed\n'
printf 'live_owner_split_exit: %s\n' "$split_exit"
printf 'wrong_journal_exit: %s\n' "$wrong_journal_exit"
printf 'live_owner_split_error: %s\n' "$(tail -n 1 <<<"$split_output")"
printf 'wrong_journal_error: %s\n' "$(tail -n 1 <<<"$wrong_journal_output")"
printf 'dead_owner_reclaim: %s\n' "$(grep -m1 'instance_admission:' "$TEST_DIR/restart.log")"
[[ "$provider_count" -eq 1 ]]
[[ "$final_state" == Completed ]]

# A caught worker panic fails the scheduler process closed. The WorkItem stays
# Running until a new process sweeps it; no semantic outcome is fabricated.
wait "$PROVIDER_PID" >/dev/null 2>&1 || true
PROVIDER_PID=""
start_provider delay_complete 1 panic
panic_port="$LAST_PROVIDER_PORT"
setup_case panic case:h13-panic tenant:h13 "$panic_port"
"$YAI_BIN" runtime serve --workers 2 --max-active-per-tenant 1 \
  --max-queued-per-tenant 4 --max-queued-total 8 \
  --failpoint worker_panic_before_case_runtime >"$TEST_DIR/panic-crash.log" 2>&1 &
SERVICE_PID=$!
for _ in $(seq 1 200); do
  status=$($YAI_BIN runtime status 2>/dev/null || true)
  grep -Fq 'state: Running' <<<"$status" && break
  sleep 0.02
done
panic_submit=$(YAI_JOURNAL="$TEST_DIR/panic/journal.jsonl" "$YAI_BIN" runtime submit \
  --tenant tenant:h13 --case case:h13-panic --subject subject:llm-provider \
  --attachment workspace --prompt 'complete after recovered worker panic' \
  --idempotency-key request:h13-panic --max-invocations 2 --max-operations 1)
panic_work=$(sed -n 's/^work_id: //p' <<<"$panic_submit")
set +e
wait "$SERVICE_PID"
panic_exit=$?
set -e
SERVICE_PID=""
[[ "$panic_exit" -ne 0 ]]
grep -Fq 'runtime_worker_panic:' "$TEST_DIR/panic-crash.log"
wait_state "$panic_work" Running

"$YAI_BIN" runtime serve --workers 2 --max-active-per-tenant 1 \
  --max-queued-per-tenant 4 --max-queued-total 8 >"$TEST_DIR/panic-restart.log" 2>&1 &
SERVICE_PID=$!
wait_state "$panic_work" Completed
"$YAI_BIN" runtime stop >/dev/null
wait "$SERVICE_PID"
SERVICE_PID=""
panic_provider_count=$(python3 - "$TEST_DIR/panic.provider.json" <<'PY'
import json, sys
print(len(json.load(open(sys.argv[1]))))
PY
)
[[ "$panic_provider_count" -eq 1 ]]
printf 'h13_worker_panic_recovery: work_id=%s panic_exit=%s provider_count=%s final_state=Completed\n' \
  "$panic_work" "$panic_exit" "$panic_provider_count"

# AwaitingReview is reconstructed from the exact checkpoint after the
# scheduler crashes before its operational acknowledgement.
wait "$PROVIDER_PID" >/dev/null 2>&1 || true
PROVIDER_PID=""
"$YAI_BIN" security bootstrap-local --tenant tenant:h13-b \
  --organization organization:characterization >/dev/null
start_provider review 2 review
review_port="$LAST_PROVIDER_PORT"
setup_case review case:h13-review tenant:h13-b "$review_port" yes
"$YAI_BIN" runtime serve --workers 2 --max-active-per-tenant 1 \
  --max-queued-per-tenant 4 --max-queued-total 8 \
  --failpoint after_case_runtime_awaiting_review_before_workitem_state_commit \
  >"$TEST_DIR/review-crash.log" 2>&1 &
SERVICE_PID=$!
for _ in $(seq 1 200); do
  status=$($YAI_BIN runtime status 2>/dev/null || true)
  grep -Fq 'state: Running' <<<"$status" && break
  sleep 0.02
done
review_submit=$(YAI_JOURNAL="$TEST_DIR/review/journal.jsonl" "$YAI_BIN" runtime submit \
  --tenant tenant:h13-b --case case:h13-review --subject subject:llm-provider \
  --attachment workspace --prompt 'propose one reviewed write then complete' \
  --idempotency-key request:h13-review --max-invocations 3 --max-operations 2)
review_work=$(sed -n 's/^work_id: //p' <<<"$review_submit")
set +e
wait "$SERVICE_PID"
review_crash_exit=$?
set -e
SERVICE_PID=""
[[ "$review_crash_exit" -eq 123 ]]
wait_state "$review_work" Running

"$YAI_BIN" runtime serve --workers 2 --max-active-per-tenant 1 \
  --max-queued-per-tenant 4 --max-queued-total 8 >"$TEST_DIR/review-restart.log" 2>&1 &
SERVICE_PID=$!
wait_state "$review_work" WaitingReview
review_count_before=$(python3 - "$YAI_HOME/run/case-runtime" "$review_work" <<'PY'
import glob, json, os, sys
for path in glob.glob(os.path.join(sys.argv[1], "*.json")):
    value = json.load(open(path))
    if value.get("work_item_id") == sys.argv[2]:
        print(value["invocations"])
        break
PY
)
[[ "$review_count_before" -eq 1 ]]
pending=$($YAI_BIN review pending --case case:h13-review)
review_id=$(sed -n 's/^review_id: //p' <<<"$pending" | head -1)
"$YAI_BIN" review approve "$review_id" --case case:h13-review \
  --reason 'H13 authenticated crash recovery review' >/dev/null
wait_state "$review_work" Completed
"$YAI_BIN" runtime stop >/dev/null
wait "$SERVICE_PID"
SERVICE_PID=""
wait "$PROVIDER_PID" >/dev/null 2>&1 || true
PROVIDER_PID=""
review_provider_count=$(python3 - "$TEST_DIR/review.provider.json" <<'PY'
import json, sys
print(len(json.load(open(sys.argv[1]))))
PY
)
[[ "$review_provider_count" -eq 2 ]]
printf 'h13_waiting_review_recovery: work_id=%s review_id=%s crash_exit=%s provider_before=%s provider_final=%s final_state=Completed\n' \
  "$review_work" "$review_id" "$review_crash_exit" "$review_count_before" "$review_provider_count"

# The dispatch cursor is committed with the claim. Repeated restart after the
# claim therefore alternates eligible Tenants instead of resetting to A.
start_provider delay_complete 2 fair-a
fair_a_port="$LAST_PROVIDER_PORT"
setup_case fair-a case:h13-fair-a tenant:h13 "$fair_a_port"
start_provider delay_complete 2 fair-b
fair_b_port="$LAST_PROVIDER_PORT"
setup_case fair-b case:h13-fair-b tenant:h13-b "$fair_b_port"

"$YAI_BIN" runtime serve --workers 2 --max-active-per-tenant 1 \
  --max-queued-per-tenant 4 --max-queued-total 8 \
  --startup-dispatch-delay-ms 800 --failpoint after_work_running_before_case_admission \
  >"$TEST_DIR/fair-crash-a.log" 2>&1 &
SERVICE_PID=$!
for _ in $(seq 1 200); do
  status=$($YAI_BIN runtime status 2>/dev/null || true)
  grep -Fq 'state: Running' <<<"$status" && break
  sleep 0.02
done
fair_a1=$(YAI_JOURNAL="$TEST_DIR/fair-a/journal.jsonl" "$YAI_BIN" runtime submit \
  --tenant tenant:h13 --case case:h13-fair-a --subject subject:llm-provider \
  --attachment workspace --prompt 'fair A1' --idempotency-key request:h13-fair-a1 | sed -n 's/^work_id: //p')
fair_a2=$(YAI_JOURNAL="$TEST_DIR/fair-a/journal.jsonl" "$YAI_BIN" runtime submit \
  --tenant tenant:h13 --case case:h13-fair-a --subject subject:llm-provider \
  --attachment workspace --prompt 'fair A2' --idempotency-key request:h13-fair-a2 | sed -n 's/^work_id: //p')
fair_b1=$(YAI_JOURNAL="$TEST_DIR/fair-b/journal.jsonl" "$YAI_BIN" runtime submit \
  --tenant tenant:h13-b --case case:h13-fair-b --subject subject:llm-provider \
  --attachment workspace --prompt 'fair B1' --idempotency-key request:h13-fair-b1 | sed -n 's/^work_id: //p')
fair_b2=$(YAI_JOURNAL="$TEST_DIR/fair-b/journal.jsonl" "$YAI_BIN" runtime submit \
  --tenant tenant:h13-b --case case:h13-fair-b --subject subject:llm-provider \
  --attachment workspace --prompt 'fair B2' --idempotency-key request:h13-fair-b2 | sed -n 's/^work_id: //p')
set +e
wait "$SERVICE_PID"
fair_crash_a_exit=$?
set -e
SERVICE_PID=""
[[ "$fair_crash_a_exit" -eq 121 ]]
grep -Fq 'tenant=tenant:h13 ' "$TEST_DIR/fair-crash-a.log"

"$YAI_BIN" runtime serve --workers 2 --max-active-per-tenant 1 \
  --max-queued-per-tenant 4 --max-queued-total 8 \
  --failpoint after_work_running_before_case_admission \
  >"$TEST_DIR/fair-crash-b.log" 2>&1 &
SERVICE_PID=$!
set +e
wait "$SERVICE_PID"
fair_crash_b_exit=$?
set -e
SERVICE_PID=""
[[ "$fair_crash_b_exit" -eq 121 ]]
grep -Fq 'tenant=tenant:h13-b ' "$TEST_DIR/fair-crash-b.log"

"$YAI_BIN" runtime serve --workers 2 --max-active-per-tenant 1 \
  --max-queued-per-tenant 4 --max-queued-total 8 >"$TEST_DIR/fair-final.log" 2>&1 &
SERVICE_PID=$!
for fair_work in "$fair_a1" "$fair_a2" "$fair_b1" "$fair_b2"; do
  wait_state "$fair_work" Completed
done
"$YAI_BIN" runtime stop >/dev/null
wait "$SERVICE_PID"
SERVICE_PID=""
fair_a_count=$(python3 - "$TEST_DIR/fair-a.provider.json" <<'PY'
import json, sys
print(len(json.load(open(sys.argv[1]))))
PY
)
fair_b_count=$(python3 - "$TEST_DIR/fair-b.provider.json" <<'PY'
import json, sys
print(len(json.load(open(sys.argv[1]))))
PY
)
[[ "$fair_a_count" -eq 2 && "$fair_b_count" -eq 2 ]]
printf 'h13_restart_fairness: first=tenant:h13 second=tenant:h13-b crash_exits=%s,%s provider_counts=%s,%s final=all_completed\n' \
  "$fair_crash_a_exit" "$fair_crash_b_exit" "$fair_a_count" "$fair_b_count"
printf 'fairness_work_ids: %s %s %s %s\n' "$fair_a1" "$fair_a2" "$fair_b1" "$fair_b2"
printf 'h13_hardening_characterization: pass\n'
printf 'test_run_id: %s\n' "$TEST_DIR"
