#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="${YAI_BIN:-$ROOT/target/debug/yai}"
PROVIDER_BASE_URL="${YAI_EXTERNAL_PROVIDER_BASE_URL:-${YVEX_BASE_URL:-http://127.0.0.1:8001/v1}}"
PROVIDER_MODEL="${YAI_EXTERNAL_PROVIDER_MODEL:-${YVEX_MODEL:-}}"
PROVIDER_TIMEOUT_MS="${YAI_EXTERNAL_PROVIDER_TIMEOUT_MS:-${YVEX_TEST_TIMEOUT_MS:-900000}}"
RUN_ID="yvex-black-box-$(date -u +%Y%m%dT%H%M%SZ)-$$"
YAI_SHA="$(git -C "$ROOT" rev-parse HEAD)"

printf 'run_id: %s\n' "$RUN_ID"
printf 'yai_sha: %s\n' "$YAI_SHA"
printf 'qualification_mode: black_box_openai_compatible_provider\n'
printf 'yvex_repository_accessed: false\n'
printf 'yvex_cli_used: false\n'
printf 'provider_endpoint: %s\n' "$PROVIDER_BASE_URL"

timeout_seconds=$(( (PROVIDER_TIMEOUT_MS + 999) / 1000 ))
models_url="${PROVIDER_BASE_URL%/}/models"
health_url="${PROVIDER_BASE_URL%/v1}/health"
health=$(curl -fsS --connect-timeout 2 --max-time 10 "$health_url" 2>&1 || true)
printf 'yvex_health: %s\n' "${health:-unavailable_optional_extension}"
if ! models=$(curl -fsS --connect-timeout 2 --max-time 10 "$models_url" 2>&1); then
  printf 'yvex_external_qualification_state: blocked_external_dependency\n'
  printf 'reason: no reachable YVEX OpenAI-compatible models endpoint at %s\n' "$models_url"
  printf 'models_probe: %s\n' "$models"
  exit 3
fi

if [[ -n "$PROVIDER_MODEL" ]]; then
  model="$PROVIDER_MODEL"
  if ! jq -e --arg model "$model" '.data[]? | select(.id == $model)' <<<"$models" >/dev/null; then
    printf 'yvex_external_qualification_state: failed\n'
    printf 'reason: explicitly requested model is not exposed exactly: %s\n' "$model"
    exit 2
  fi
else
  model=$(jq -er '.data[0].id' <<<"$models")
fi
printf 'yvex_model: %s\n' "$model"
printf 'discovered_model_id: %s\n' "$model"
printf 'provider_kind: openai_compatible\n'

TEST_DIR=$(mktemp -d /tmp/yai-yvex-qualification.XXXXXX)
cleanup() {
  if [[ "${YAI_KEEP_TEST_DIR:-0}" != "1" ]]; then
    rm -rf "$TEST_DIR"
  fi
}
trap cleanup EXIT INT TERM
export YAI_HOME="$TEST_DIR/home"
mkdir -p "$YAI_HOME"
journal="$TEST_DIR/case-yvex.jsonl"
: >"$journal"

"$YAI_BIN" security bootstrap-local \
  --tenant tenant:yvex-qualification \
  --organization organization:yvex-qualification >/dev/null
"$YAI_BIN" case create \
  --case case:yvex-qualification \
  --tenant tenant:yvex-qualification >/dev/null
"$YAI_BIN" case enter \
  --case case:yvex-qualification \
  --subject participant:model-yvex >/dev/null
"$YAI_BIN" case attach-provider \
  --case case:yvex-qualification \
  --subject participant:model-yvex \
  --provider-id provider:external-openai-compatible \
  --base-url "${PROVIDER_BASE_URL%/}/chat/completions" \
  --model "$model" >/dev/null

started=$(date +%s%3N)
set +e
output=$(timeout "${timeout_seconds}s" env \
  YAI_JOURNAL="$journal" \
  YAI_PROVIDER_API_KEY="local-qualification-placeholder" \
  "$YAI_BIN" prompt --once \
  "Describe the current Case identity, Tenant security domain, your participant/provider/model identity, visible resources, and whether you can mint Decisions, Grants, or receipts. Use only the supplied Case view; say unknown when absent." \
  --case case:yvex-qualification \
  --subject participant:model-yvex \
  --provider-id provider:external-openai-compatible \
  --base-url "${PROVIDER_BASE_URL%/}/chat/completions" \
  --model "$model" 2>&1)
exit_code=$?
set -e
ended=$(date +%s%3N)
printf '%s\n' "$output"
printf 'latency_ms: %s\n' "$((ended - started))"
if [[ "$exit_code" -ne 0 ]]; then
  printf 'yvex_external_qualification_state: failed\n'
  printf 'provider_exit: %s\n' "$exit_code"
  exit "$exit_code"
fi
grep -F 'provider_kind: openai_compatible' <<<"$output" >/dev/null
grep -E '^provider_invocation_id: invocation:' <<<"$output" >/dev/null
grep -E '^provider_result_id: provider-result:' <<<"$output" >/dev/null
grep -E '^context_frame_id: context-frame:' <<<"$output" >/dev/null
response_model=$(sed -n 's/^provider_response_model_id: //p' <<<"$output")
[[ "$response_model" == not_returned ]] && response_model=""
if [[ -n "$response_model" && "$response_model" != "$model" ]]; then
  printf 'yvex_external_qualification_state: failed\n'
  printf 'reason: provider response model identity mismatch requested=%s response=%s\n' \
    "$model" "$response_model"
  exit 2
fi
printf 'model_identity_chain: discovered=%s requested=%s response=%s\n' \
  "$model" "$model" "${response_model:-not_returned}"
printf 'yvex_external_qualification_state: passed\n'
