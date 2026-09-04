#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
source "$ROOT/tests/characterization/lib/governed_case_policy.sh"
YAID="$ROOT/build/yaid"
TEST_DIR="$ROOT/build/tmp/characterization-provider-$$"
YAI_HOME="$TEST_DIR/home"
SOCKET="$TEST_DIR/yaid.sock"
PORT_FILE="$TEST_DIR/provider.port"
DAEMON_PID=""
PROVIDER_PID=""
export YAI_HOME

cleanup() {
  if [[ -n "$DAEMON_PID" ]]; then
    "$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null 2>&1 || true
    wait "$DAEMON_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "$PROVIDER_PID" ]]; then
    wait "$PROVIDER_PID" >/dev/null 2>&1 || true
  fi
  rm -rf "$TEST_DIR"
}
trap cleanup EXIT

require_text() {
  local value="$1"
  local expected="$2"
  grep -Fq -- "$expected" <<<"$value"
}

mkdir -p "$TEST_DIR" "$YAI_HOME"
"$YAID" --socket "$SOCKET" --foreground >"$TEST_DIR/yaid.log" 2>&1 &
DAEMON_PID=$!
sleep 1

loop_output=$("$YAI_BIN" daemon run-filesystem-loop --socket "$SOCKET")
journal=$(sed -n 's/.*"journal_path":"\([^"]*\)".*/\1/p' <<<"$loop_output")
test -s "$journal"
frames_before=$(grep -c -F '"record_kind":"participant_view_frame"' "$journal" || true)

yai_bootstrap_tenant_case "$YAI_BIN" "$YAI_HOME" case:new12-filesystem
YAI_JOURNAL="$journal" "$YAI_BIN" case enter \
  --case case:new12-filesystem --subject subject:llm-provider >/dev/null
YAI_JOURNAL="$journal" "$YAI_BIN" case attach-provider \
  --case case:new12-filesystem \
  --subject subject:llm-provider \
  --base-url http://127.0.0.1:1/v1/chat/completions \
  --model characterization-model >/dev/null

python3 "$ROOT/tests/fixtures/openai_compatible_server.py" >"$PORT_FILE" &
PROVIDER_PID=$!
for _ in $(seq 1 50); do
  [[ -s "$PORT_FILE" ]] && break
  sleep 0.02
done
test -s "$PORT_FILE"
port=$(tr -d '[:space:]' <"$PORT_FILE")

prompt_output=$(YAI_JOURNAL="$journal" \
  YAI_CASE_REF=case:new12-filesystem \
  YAI_PROVIDER_SUBJECT_REF=subject:llm-provider \
  YAI_PROVIDER_BASE_URL="http://127.0.0.1:$port/v1/chat/completions" \
  YAI_PROVIDER_MODEL=characterization-model \
  "$YAI_BIN" prompt --once "characterize provider continuity")
wait "$PROVIDER_PID"
PROVIDER_PID=""

require_text "$prompt_output" "fixture provider result"
require_text "$prompt_output" "projection_id: projection:"
require_text "$prompt_output" "context_frame_id: context-frame:"
require_text "$prompt_output" "interaction_turn: interaction-turn:"
require_text "$prompt_output" "model_interpretation: model_interpretation:observed"
require_text "$prompt_output" "continuation_disposition: not_provided"

projection_id=$(sed -n 's/^projection_id: //p' <<<"$prompt_output" | tail -1)
frame_id=$(sed -n 's/^context_frame_id: //p' <<<"$prompt_output" | tail -1)
projection_inspect=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" context inspect --projection "$projection_id")
frame_inspect=$(YAI_HOME="$YAI_HOME" "$YAI_BIN" context inspect --frame "$frame_id")
require_text "$projection_inspect" "schema: yai.projection.v7"
require_text "$projection_inspect" "participant_id: subject:llm-provider"
require_text "$projection_inspect" "provenance: entry="
require_text "$frame_inspect" "schema: yai.context_frame.v7"
require_text "$frame_inspect" "projection_id: $projection_id"

grep -F '"record_kind":"attempt"' "$journal" | grep -F 'op:model.prompt.submit' >/dev/null
grep -F '"record_kind":"effect_receipt"' "$journal" | grep -F 'model.output status:observed' >/dev/null
grep -F '"record_kind":"model_interpretation"' "$journal" | grep -F 'authority:not_authoritative_state' >/dev/null
grep -F '"record_kind":"interaction_turn"' "$journal" >/dev/null
frames_after=$(grep -c -F '"record_kind":"participant_view_frame"' "$journal" || true)
[[ "$frames_after" == "$frames_before" ]]

canonical_summary="$("$YAI_BIN" store summary)"
grep -Eq "transitions_total: [1-9][0-9]*" <<<"$canonical_summary"
grep -Eq "cases_materialized: [1-9][0-9]*" <<<"$canonical_summary"

printf 'provider_model_vertical:real_http_invocation ok\n'
printf 'provider_model_vertical:durable_continuity_residue ok\n'
printf 'provider_model_vertical:canonical_transition_authority ok\n'
printf 'provider_model_vertical:typed_projection_context_frame ok\n'
