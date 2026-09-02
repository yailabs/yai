#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
ENGINE_MANIFEST="$ROOT/engine/Cargo.toml"
CLI_MANIFEST="$ROOT/cmd/yai/Cargo.toml"
YAI_BIN="$ROOT/target/debug/yai"
RUN_ROOT="$(mktemp -d)"
SERVER_PID=""
export CARGO_TARGET_DIR="$ROOT/target"

cleanup() {
  if [[ -n "$SERVER_PID" ]]; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
  rm -rf "$RUN_ROOT"
}
trap cleanup EXIT

run_engine_proof() {
  local test_name=$1
  local expected=$2
  local output
  output=$(cargo test --manifest-path "$ENGINE_MANIFEST" -p yai-engine \
    "$test_name" -- --exact --nocapture 2>&1)
  grep -Fq -- "$expected" <<<"$output"
  printf '%s\n' "$output" | grep -E '^h18_|test result:' | tail -2
}

run_engine_proof \
  store::lmdb::tests::hardening18_tests::h18_qualification_and_trust_current_projections_are_not_authority \
  'h18_projection_rebuild:'
run_engine_proof \
  store::lmdb::tests::hardening18_tests::h18_corrupt_qualification_and_missing_trust_sequence_fail_closed \
  'h18_governance_corruption:'
run_engine_proof \
  store::lmdb::tests::hardening18_tests::h18_qualification_time_is_bounded_expiry_is_exclusive_and_rollback_safe \
  'rollback_resurrection=false'
run_engine_proof \
  store::lmdb::tests::hardening18_tests::h18_credential_rotation_is_non_secret_and_invalidates_old_qualification \
  'secret_persisted=false'
run_engine_proof \
  store::lmdb::tests::hardening18_tests::h18_half_open_probe_admits_one_and_dead_owner_is_reclaimed \
  'contenders=64 admitted=1'
run_engine_proof \
  store::lmdb::tests::hardening18_tests::h18_independent_process_trust_probe_and_selection_are_serialized \
  'trust_processes=64'
run_engine_proof \
  store::lmdb::tests::hardening18_tests::h18_historical_selector_v1_and_attempt_boundaries_remain_exact \
  'future_unknown=fail_closed generic_429_retry_safe=false'

transport_output=$(cargo test --manifest-path "$CLI_MANIFEST" \
  provider_transport::tests -- --nocapture 2>&1)
grep -Fq 'h18_dns_rebinding:' <<<"$transport_output"
grep -Fq 'h18_http_boundary:' <<<"$transport_output"
grep -Fq 'h18_tls:' <<<"$transport_output"
printf '%s\n' "$transport_output" | grep -E '^h18_|test result:'

python3 "$ROOT/tests/fixtures/provider_governance_server.py" \
  --mode full --model h18-provider-model --requests 16 \
  >"$RUN_ROOT/server.out" 2>"$RUN_ROOT/server.err" &
SERVER_PID=$!
for _ in $(seq 1 100); do
  [[ -s "$RUN_ROOT/server.out" ]] && break
  sleep 0.05
done
PORT="$(head -1 "$RUN_ROOT/server.out")"
[[ "$PORT" =~ ^[0-9]+$ ]]

export YAI_HOME="$RUN_ROOT/yai-home"
export H18_PROVIDER_TOKEN='h18-secret-must-never-be-rendered'
"$YAI_BIN" init --tenant tenant:h18-smoke --organization organization:h18 >/dev/null
add_output=$("$YAI_BIN" provider add \
  --tenant tenant:h18-smoke \
  --provider-key fixture \
  --endpoint "http://127.0.0.1:$PORT" \
  --model h18-provider-model \
  --credential-ref env:H18_PROVIDER_TOKEN \
  --locality loopback)
TARGET_ID=$(awk '/target_id:/ {print $2}' <<<"$add_output")
[[ "$TARGET_ID" == provider-target:* ]]

qualification_before=$("$YAI_BIN" provider qualify --target "$TARGET_ID")
"$YAI_BIN" provider trust approve --target "$TARGET_ID" >/dev/null
rotation=$("$YAI_BIN" provider credential rotate "$TARGET_ID" --revision operator-rotation-1)
grep -Fq 'credential_revision: 1' <<<"$rotation"
grep -Fq 'secret_persisted: false' <<<"$rotation"
post_rotation=$("$YAI_BIN" provider show --target "$TARGET_ID")
grep -Fq 'configuration_credential_revision: 1' <<<"$post_rotation"
grep -Fq 'qualification: missing' <<<"$post_rotation"
qualification_after=$("$YAI_BIN" provider qualify --target "$TARGET_ID")
post_requalification_json=$("$YAI_BIN" provider show --target "$TARGET_ID" --json)
grep -Fq '"name":"configuration credential revision","value":"1"' \
  <<<"$post_requalification_json"
grep -Fq '"name":"qualification","value":"qualified"' \
  <<<"$post_requalification_json"

combined_output="$add_output
$qualification_before
$rotation
$post_rotation
$qualification_after
$post_requalification_json"
if grep -Fq "$H18_PROVIDER_TOKEN" <<<"$combined_output"; then
  printf 'credential_secret_leaked\n' >&2
  exit 1
fi

help_json=$("$YAI_BIN" help --json)
grep -Fq '"operation_id":"yai.provider.credential.rotate"' <<<"$help_json"

printf 'provider_governance_hardening_characterization: pass\n'
printf 'target_id: %s\n' "$TARGET_ID"
printf 'credential_revision: 1\n'
printf 'old_qualification_invalidated: true\n'
printf 'requalification_required: true\n'
printf 'secret_persisted_or_rendered: false\n'
printf 'qualification_before: %s\n' "$(sed -n 's/^qualification_id: //p' <<<"$qualification_before")"
printf 'qualification_after: %s\n' "$(sed -n 's/^qualification_id: //p' <<<"$qualification_after")"
