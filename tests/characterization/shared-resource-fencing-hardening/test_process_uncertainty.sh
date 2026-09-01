#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
YAID="$ROOT/build/yaid"
PROVIDER_FIXTURE="$ROOT/tests/fixtures/process_signal_provider.py"
TEST_DIR="$(mktemp -d /tmp/yai-h14-process-uncertainty.XXXXXX)"
export YAI_HOME="$TEST_DIR/home"
SOCKET="$TEST_DIR/yaid.sock"
JOURNAL="$TEST_DIR/case.jsonl"
TENANT="tenant:h14-process"
CASE_ID="case:h14-process"
DAEMON_PID=""
PROVIDER_PID=""
FIXTURE_PID=""

cleanup() {
  if [[ -n "$FIXTURE_PID" ]]; then
    kill -CONT "$FIXTURE_PID" >/dev/null 2>&1 || true
    kill -TERM "$FIXTURE_PID" >/dev/null 2>&1 || true
    wait "$FIXTURE_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "$PROVIDER_PID" ]]; then
    kill "$PROVIDER_PID" >/dev/null 2>&1 || true
    wait "$PROVIDER_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "$DAEMON_PID" ]]; then
    "$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null 2>&1 || true
    wait "$DAEMON_PID" >/dev/null 2>&1 || true
  fi
  if [[ "${YAI_KEEP_TEST_DIR:-0}" != "1" ]]; then
    rm -rf "$TEST_DIR"
  fi
}
trap cleanup EXIT INT TERM

mkdir -p "$YAI_HOME" "$TEST_DIR/daemon-user" "$TEST_DIR/seed-home"
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
sed -e "s/case:new12-filesystem/$CASE_ID/g" \
  -e 's/new12-fs/h14-process/g' "$ROOT/$source_journal" >"$JOURNAL"
"$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null
wait "$DAEMON_PID"
DAEMON_PID=""

sh -c 'while :; do sleep 1; done' &
FIXTURE_PID=$!

port_file="$TEST_DIR/provider.port"
python3 "$PROVIDER_FIXTURE" >"$port_file" &
PROVIDER_PID=$!
for _ in $(seq 1 100); do
  [[ -s "$port_file" ]] && break
  sleep 0.02
done
[[ -s "$port_file" ]]
provider_port=$(tr -d '[:space:]' <"$port_file")

"$YAI_BIN" security bootstrap-local --tenant "$TENANT" \
  --organization organization:characterization >/dev/null
"$YAI_BIN" case create --case "$CASE_ID" --tenant "$TENANT" >/dev/null
YAI_JOURNAL="$JOURNAL" "$YAI_BIN" case enter \
  --case "$CASE_ID" --subject subject:llm-provider >/dev/null
YAI_JOURNAL="$JOURNAL" "$YAI_BIN" case attach-provider \
  --case "$CASE_ID" --subject subject:llm-provider \
  --provider-id provider:h14-process \
  --base-url "http://127.0.0.1:$provider_port/v1/chat/completions" \
  --model controlled-process-model >/dev/null
"$YAI_BIN" case bind-participant-role --case "$CASE_ID" \
  --participant subject:policy-pack \
  --role resource-attachment-compatibility-owner >/dev/null
"$YAI_BIN" case attach-process --case "$CASE_ID" \
  --attachment process-fixture --pid "$FIXTURE_PID" \
  --policy-owner subject:policy-pack --actions suspend >/dev/null
"$YAI_BIN" case bind-participant-role --case "$CASE_ID" \
  --participant subject:llm-provider --role operation-proposer >/dev/null

policy_source="$TEST_DIR/process-policy.json"
printf '%s\n' '{"schema":"yai.policy_source_input.v4","policy_key":"h14.process.signal","source_version":"1","owner_ref":"organization:characterization","source_origin":{"source_system":"characterization","source_uri":"test://h14/process"},"validity":{"mode":"unbounded"},"rules":[{"kind":"operation_restriction","rule_id":"allow","operation_kind":"process.signal","resource_kind":"process","effect":"allow","reason":"test-owned process action"},{"kind":"authority_requirement","rule_id":"proposer","operation_kind":"process.signal","resource_kind":"process","subject":"proposer","required_role":"operation-proposer","reason":"explicit Case role"},{"kind":"evidence_obligation","rule_id":"source","operation_kind":"process.signal","resource_kind":"process","obligation":"source_provenance","reason":"canonical provider source"},{"kind":"evidence_obligation","rule_id":"pre","operation_kind":"process.signal","resource_kind":"process","obligation":"pre_observation","reason":"exact birth pre-observation"},{"kind":"evidence_obligation","rule_id":"post","operation_kind":"process.signal","resource_kind":"process","obligation":"post_observation","reason":"truthful kernel result"}]}' >"$policy_source"
artifact_output=$("$YAI_BIN" policy ingest "$policy_source" --tenant "$TENANT")
artifact_id=$(sed -n 's/^artifact_id: //p' <<<"$artifact_output" | head -1)
"$YAI_BIN" policy validate "$artifact_id" --reason deterministic >/dev/null
"$YAI_BIN" policy publish "$artifact_id" --reason characterization >/dev/null
generation=$("$YAI_BIN" case policy status --case "$CASE_ID" | \
  sed -n 's/^case_generation: //p' | head -1)
"$YAI_BIN" case policy bind --case "$CASE_ID" --artifact "$artifact_id" \
  --expected-generation "$generation" --reason bind >/dev/null

set +e
YAI_JOURNAL="$JOURNAL" "$YAI_BIN" effect process-signal \
  --case "$CASE_ID" --subject subject:llm-provider \
  --attachment process-fixture --prompt "suspend the test-owned process" \
  --provider-id provider:h14-process \
  --base-url "http://127.0.0.1:$provider_port/v1/chat/completions" \
  --model controlled-process-model \
  --failpoint after_process_signal_before_finalize \
  >"$TEST_DIR/signal.out" 2>&1
signal_exit=$?
set -e
[[ "$signal_exit" -eq 88 ]]
wait "$PROVIDER_PID"
PROVIDER_PID=""
effect_id=$(sed -n 's/^effect_id: //p' "$TEST_DIR/signal.out" | head -1)
resource_id=$(sed -n 's/^resource_id: //p' "$TEST_DIR/signal.out" | head -1)
[[ "$effect_id" == effect:* ]]
[[ "$resource_id" == resource-control:* ]]

reconcile_one=$("$YAI_BIN" effect reconcile --case "$CASE_ID" \
  --effect "$effect_id" --retry)
grep -Fq 'reconciliation: StillIndeterminate' <<<"$reconcile_one"
grep -Fq 'process_recovery_mode: observation_only' <<<"$reconcile_one"
grep -Fq 'process_signal_repeated: false' <<<"$reconcile_one"
reconcile_two=$("$YAI_BIN" effect reconcile --case "$CASE_ID" \
  --effect "$effect_id" --retry)
grep -Fq 'process_signal_repeated: false' <<<"$reconcile_two"

printf 'h14_process_uncertainty: pass\n'
printf 'test_run_id: %s\n' "$TEST_DIR"
printf 'fixture_pid: %s\n' "$FIXTURE_PID"
printf 'signal_carrier_exit: %s\n' "$signal_exit"
printf 'effect_id: %s\n' "$effect_id"
printf 'resource_id: %s\n' "$resource_id"
printf 'recovery_mode: observation_only\n'
printf 'signal_repeated_during_recovery: false\n'
printf 'effect_posture: indeterminate\n'
