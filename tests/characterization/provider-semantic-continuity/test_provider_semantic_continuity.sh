#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
source "$ROOT/tests/characterization/lib/governed_case_policy.sh"
YAID="$ROOT/build/yaid"
FIXTURE="$ROOT/tests/fixtures/semantic_continuity_provider.py"
TEST_DIR="$(mktemp -d /tmp/yai-semantic-continuity.XXXXXX)"
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
  rm -rf "$TEST_DIR"
}
trap cleanup EXIT INT TERM

require_text() {
  grep -Fq -- "$2" <<<"$1"
}

start_provider() {
  local mode="$1"
  local name="$2"
  local port_file="$TEST_DIR/$name.port"
  YAI_SEMANTIC_PROVIDER_LOG="$TEST_DIR/$name.log.json" \
    python3 "$FIXTURE" "$mode" >"$port_file" &
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
  yai_bootstrap_tenant_case "$YAI_BIN" "$CASE_HOME" case:new12-filesystem
  YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case enter \
    --case case:new12-filesystem --subject subject:llm-provider >/dev/null
  YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case attach-provider \
    --case case:new12-filesystem --subject subject:llm-provider \
    --provider-id "$provider_id" \
    --base-url "http://127.0.0.1:$port/v1/chat/completions" \
    --model "$model" >/dev/null
}

# Provider replacement: A proposes; B, with no A continuation, observes the
# real finalized consequence through a newly compiled frame.
start_provider proposal-a provider-a
port_a="$LAST_PROVIDER_PORT"
start_provider consequence-b provider-b
port_b="$LAST_PROVIDER_PORT"
setup_case provider-switch provider:a model-a "$port_a"
YAI_HOME="$CASE_HOME" "$YAI_BIN" case attach-filesystem \
  --case case:new12-filesystem --attachment workspace --root "$RESOURCE_ROOT" \
  --allow-prefix allowed --policy-owner subject:policy-pack --max-bytes 256 >/dev/null
yai_configure_governed_filesystem_case "$YAI_BIN" "$CASE_HOME" \
  case:new12-filesystem semantic-continuity 1 allow subject:llm-provider >/dev/null
provider_switch=$(YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" effect filesystem-write \
  --case case:new12-filesystem --subject subject:llm-provider --attachment workspace \
  --prompt "propose a continuity write" \
  --provider-id provider:a --base-url "http://127.0.0.1:$port_a/v1/chat/completions" --model model-a \
  --second-provider-id provider:b --second-base-url "http://127.0.0.1:$port_b/v1/chat/completions" --second-model model-b)
wait_providers
require_text "$provider_switch" "effect_outcome: Applied"
require_text "$provider_switch" "second_turn_consequence: observed_reality_from_canonical_state"
[[ "$(cat "$RESOURCE_ROOT/allowed/provider-switch.txt")" == "provider-independent continuity" ]]
grep -Fq '"provider_id": "provider:b"' "$TEST_DIR/provider-b.log.json"
grep -Fq '"posture": "observed_resource_state"' "$TEST_DIR/provider-b.log.json"

# Operator proof: derive, inspect provenance/retrieval, drop, rebuild, and
# recover the same deterministic memory identity without touching the ledger.
memory_rebuild=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" memory rebuild \
  --case case:new12-filesystem)
memory_list=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" memory list \
  --case case:new12-filesystem --include-superseded --limit 50)
memory_id=$(sed -n 's/^entry: \([^ ]*\) kind:resource_effect .*/\1/p' <<<"$memory_list" | head -1)
[[ -n "$memory_id" ]]
memory_show=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" memory show "$memory_id")
memory_provenance=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" memory provenance "$memory_id")
memory_retrieval=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" memory retrieve \
  --case case:new12-filesystem --participant subject:llm-provider \
  --purpose effect_consequence --resource workspace --limit 4)
memory_clear=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" memory clear \
  --case case:new12-filesystem)
memory_rebuild_again=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" memory rebuild \
  --case case:new12-filesystem)
memory_list_again=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" memory list \
  --case case:new12-filesystem --include-superseded --limit 50)
require_text "$memory_rebuild" "canonical_ledger_mutated: no"
require_text "$memory_show" "posture: finalized_observed_consequence"
require_text "$memory_provenance" "provenance_valid: yes"
require_text "$memory_retrieval" "canonical_ledger_mutated: no"
require_text "$memory_retrieval" "direct_resource_match:+100"
require_text "$memory_clear" "derived_entries_remaining: 0"
require_text "$memory_clear" "canonical_transitions_remaining:"
require_text "$memory_rebuild_again" "canonical_ledger_mutated: no"
require_text "$memory_list_again" "$memory_id"

# Model replacement under the same logical provider identity.
start_provider model-switch model-switch
port_model="$LAST_PROVIDER_PORT"
setup_case model-switch provider:stable model-a "$port_model"
YAI_HOME="$CASE_HOME" "$YAI_BIN" case attach-filesystem \
  --case case:new12-filesystem --attachment workspace --root "$RESOURCE_ROOT" \
  --allow-prefix allowed --policy-owner subject:policy-pack --max-bytes 256 >/dev/null
yai_configure_governed_filesystem_case "$YAI_BIN" "$CASE_HOME" \
  case:new12-filesystem semantic-continuity-restart 1 allow subject:llm-provider >/dev/null
model_switch=$(YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" effect filesystem-write \
  --case case:new12-filesystem --subject subject:llm-provider --attachment workspace \
  --prompt "propose a model-switch write" \
  --provider-id provider:stable --base-url "http://127.0.0.1:$port_model/v1/chat/completions" --model model-a \
  --second-provider-id provider:stable --second-base-url "http://127.0.0.1:$port_model/v1/chat/completions" --second-model model-b)
wait_providers
require_text "$model_switch" "effect_outcome: Applied"
grep -Fq '"model": "model-a"' "$TEST_DIR/model-switch.log.json"
grep -Fq '"model": "model-b"' "$TEST_DIR/model-switch.log.json"

# A generic invalid-continuation response is not proof that cognition did not
# execute, so it cannot trigger an automatic retry. A later explicit provider
# restart consumes a freshly rebuilt frame without the opaque reference.
start_provider invalid-continuation invalid-continuation
port_invalid="$LAST_PROVIDER_PORT"
setup_case continuation provider:continuation continuation-model "$port_invalid"
set +e
continuation_output=$(YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" prompt \
  --case case:new12-filesystem --subject subject:llm-provider \
  --provider-id provider:continuation \
  --base-url "http://127.0.0.1:$port_invalid/v1/chat/completions" --model continuation-model \
  --continuation-capable --provider-runtime-id runtime:old \
  --continuation-ref opaque-missing-state --once "continue after KV loss" 2>&1)
continuation_exit=$?
set -e
wait_providers
[[ "$continuation_exit" -ne 0 ]]
require_text "$continuation_output" "provider_remote_response:409"
first_frame=$(sed -n 's/.*"frame_id": "\([^"]*\)".*/\1/p' \
  "$TEST_DIR/invalid-continuation.log.json" | head -1)
[[ -n "$first_frame" ]]

start_provider fresh-restart fresh-restart
port_restart="$LAST_PROVIDER_PORT"
YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case attach-provider \
  --case case:new12-filesystem --subject subject:llm-provider \
  --provider-id provider:continuation \
  --base-url "http://127.0.0.1:$port_restart/v1/chat/completions" \
  --model continuation-model >/dev/null
restart_output=$(YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" prompt \
  --case case:new12-filesystem --subject subject:llm-provider \
  --provider-id provider:continuation \
  --base-url "http://127.0.0.1:$port_restart/v1/chat/completions" --model continuation-model \
  --once "continue after provider restart")
wait_providers
require_text "$restart_output" "Provider restart preserved semantic continuity."
require_text "$restart_output" "continuation_disposition: not_provided"
second_frame=$(sed -n 's/^context_frame_id: //p' <<<"$restart_output" | tail -1)
[[ "$first_frame" != "$second_frame" ]]

printf 'semantic_continuity:provider_replacement ok\n'
printf 'semantic_continuity:model_replacement ok\n'
printf 'semantic_continuity:unsafe_continuation_retry_refused_and_restart ok\n'
printf 'semantic_continuity:memory_inspect_drop_rebuild ok\n'
