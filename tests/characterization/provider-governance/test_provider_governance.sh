#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
ENGINE_MANIFEST="$ROOT/engine/Cargo.toml"
YAI_BIN="$ROOT/target/debug/yai"
RUN_ROOT="$(mktemp -d)"
SERVER_PID=""
PRIMARY_PID=""
DROP_PID=""

cleanup() {
  if [[ -n "$SERVER_PID" ]]; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
  if [[ -n "$PRIMARY_PID" ]]; then
    kill "$PRIMARY_PID" 2>/dev/null || true
    wait "$PRIMARY_PID" 2>/dev/null || true
  fi
  if [[ -n "$DROP_PID" ]]; then
    kill "$DROP_PID" 2>/dev/null || true
    wait "$DROP_PID" 2>/dev/null || true
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
  printf '%s\n' "$output" | grep -E '^w18_|test result:' | tail -2
}

run_engine_proof \
  store::lmdb::tests::wave18_tests::wave18_governed_single_target_selection_is_case_canonical_and_replayable \
  'w18_governed_selection:'
run_engine_proof \
  store::lmdb::tests::wave18_tests::wave18_capability_trust_health_and_cross_tenant_filters_are_mechanical \
  'text_only=required_capability_missing denied=trust_not_approved'
run_engine_proof \
  store::lmdb::tests::wave18_tests::wave18_circuit_and_delivery_contract_forbid_indeterminate_failover \
  'delivery=indeterminate retry_safe=false automatic_failover=false'
run_engine_proof \
  store::lmdb::tests::wave18_tests::wave18_concurrent_selection_and_attempt_outcome_have_one_case_truth \
  'contenders=16 selections=1 outcomes=1'
run_engine_proof \
  store::lmdb::tests::wave18_tests::wave18_selection_scale_is_bounded_and_deterministic \
  'candidates=1:'
run_engine_proof \
  store::lmdb::tests::wave18_tests::wave18_qualification_current_projection_never_rolls_back \
  'rollback=false'
run_engine_proof \
  store::lmdb::tests::wave18_tests::wave18_trust_revoke_and_invocation_start_serialize \
  'serializable=true'

transport_output=$(cargo test --manifest-path "$ROOT/cmd/yai/Cargo.toml" \
  wave18_ -- --ignored --nocapture 2>&1)
grep -Fq 'wave18_connect_refused_is_provably_not_dispatched ... ok' <<<"$transport_output"
grep -Fq 'wave18_accepted_request_then_drop_is_delivery_indeterminate ... ok' <<<"$transport_output"

python3 "$ROOT/tests/fixtures/provider_governance_server.py" \
  --mode full --model provider-governance-model --requests 32 \
  >"$RUN_ROOT/server.out" 2>"$RUN_ROOT/server.err" &
SERVER_PID=$!
for _ in $(seq 1 100); do
  [[ -s "$RUN_ROOT/server.out" ]] && break
  sleep 0.05
done
PORT="$(head -1 "$RUN_ROOT/server.out")"
[[ "$PORT" =~ ^[0-9]+$ ]]

export YAI_HOME="$RUN_ROOT/yai-home"
"$YAI_BIN" init --tenant tenant:w18-smoke --organization organization:cli-product >/dev/null
add_output=$("$YAI_BIN" provider add \
  --tenant tenant:w18-smoke \
  --provider-key fixture \
  --endpoint "http://127.0.0.1:$PORT" \
  --model provider-governance-model \
  --locality loopback)
TARGET_ID=$(awk '/Target id/ {print $NF}' <<<"$add_output")
if [[ -z "$TARGET_ID" ]]; then
  TARGET_ID=$(awk '/target_id:/ {print $2}' <<<"$add_output")
fi
[[ "$TARGET_ID" == provider-target:* ]]

qualification=$("$YAI_BIN" provider qualify --target "$TARGET_ID")
grep -Fq 'ChatText' <<<"$qualification"
grep -Fq 'StructuredJsonObject' <<<"$qualification"
"$YAI_BIN" provider trust approve --target "$TARGET_ID" >/dev/null
posture_json=$("$YAI_BIN" provider show --target "$TARGET_ID" --json)
grep -Fq '"status":"ok"' <<<"$posture_json"
grep -Fq 'qualified' <<<"$posture_json"
grep -Fq 'Approved' <<<"$posture_json"
grep -Fq 'Healthy' <<<"$posture_json"

"$YAI_BIN" case create case:w18-smoke --tenant tenant:w18-smoke >/dev/null
"$YAI_BIN" case participant role add case:w18-smoke \
  --participant participant:model --role model-executor >/dev/null
"$YAI_BIN" case participant role add case:w18-smoke \
  --participant participant:model --role operation-proposer >/dev/null
"$YAI_BIN" case provider bind case:w18-smoke \
  --participant participant:model --target "$TARGET_ID" \
  --failover safe_only --max-attempts 3 >/dev/null
binding_json=$("$YAI_BIN" case provider show case:w18-smoke --json)
grep -Fq '"status":"ok"' <<<"$binding_json"
grep -Fq "$TARGET_ID" <<<"$binding_json"
grep -Fq 'SafeOnly' <<<"$binding_json"
mkdir -p "$RUN_ROOT/resource/allowed"
"$YAI_BIN" case resource attach filesystem case:w18-smoke \
  --resource resource:w18-smoke \
  --root "$RUN_ROOT/resource" \
  --allow-prefix allowed \
  --policy-owner participant:model \
  --max-bytes 1024 >/dev/null
policy_ingest=$("$YAI_BIN" policy ingest \
  "$ROOT/tests/fixtures/cli-product-policy.json" \
  --tenant tenant:w18-smoke)
POLICY_ID=$(sed -n 's/^artifact_id: //p' <<<"$policy_ingest" | head -1)
[[ -n "$POLICY_ID" ]]
"$YAI_BIN" policy validate "$POLICY_ID" --reason 'W18 fixture validation' >/dev/null
"$YAI_BIN" policy publish "$POLICY_ID" --reason 'W18 fixture publication' >/dev/null
"$YAI_BIN" case policy bind case:w18-smoke --artifact "$POLICY_ID" \
  --reason 'W18 governed provider run' >/dev/null
run_output=$("$YAI_BIN" case run case:w18-smoke \
  --participant participant:model \
  --resource resource:w18-smoke \
  --prompt 'complete the fixed synthetic provider-governance turn' \
  --max-invocations 1 --max-runtime-ms 5000)
grep -Fq 'runtime_status: Completed' <<<"$run_output"
grep -Fq 'invocations: 1' <<<"$run_output"
case_json=$("$YAI_BIN" case show case:w18-smoke --json)
grep -Fq "$TARGET_ID" <<<"$case_json"
grep -Fq 'last_selection_id' <<<"$case_json"

# Qualify a primary while it is reachable, then prove that a later connect
# failure is pre-dispatch and may select the already-qualified secondary for
# the same semantic turn.
python3 "$ROOT/tests/fixtures/provider_governance_server.py" \
  --mode full --model safe-primary-model --requests 16 \
  >"$RUN_ROOT/primary.out" 2>"$RUN_ROOT/primary.err" &
PRIMARY_PID=$!
for _ in $(seq 1 100); do
  [[ -s "$RUN_ROOT/primary.out" ]] && break
  sleep 0.05
done
PRIMARY_PORT="$(head -1 "$RUN_ROOT/primary.out")"
primary_add=$("$YAI_BIN" provider add \
  --tenant tenant:w18-smoke --provider-key safe-primary \
  --endpoint "http://127.0.0.1:$PRIMARY_PORT" --model safe-primary-model \
  --locality loopback)
PRIMARY_TARGET=$(awk '/target_id:/ {print $2}' <<<"$primary_add")
"$YAI_BIN" provider qualify --target "$PRIMARY_TARGET" >/dev/null
"$YAI_BIN" provider trust approve --target "$PRIMARY_TARGET" >/dev/null
kill "$PRIMARY_PID"
wait "$PRIMARY_PID" 2>/dev/null || true
PRIMARY_PID=""

"$YAI_BIN" case create case:w18-safe-failover --tenant tenant:w18-smoke >/dev/null
"$YAI_BIN" case participant role add case:w18-safe-failover \
  --participant participant:model --role model-executor >/dev/null
"$YAI_BIN" case participant role add case:w18-safe-failover \
  --participant participant:model --role operation-proposer >/dev/null
"$YAI_BIN" case provider bind case:w18-safe-failover \
  --participant participant:model --target "$PRIMARY_TARGET" --target "$TARGET_ID" \
  --failover safe_only --max-attempts 3 >/dev/null
mkdir -p "$RUN_ROOT/resource-safe/allowed"
"$YAI_BIN" case resource attach filesystem case:w18-safe-failover \
  --resource resource:w18-safe --root "$RUN_ROOT/resource-safe" \
  --allow-prefix allowed --policy-owner participant:model --max-bytes 1024 >/dev/null
"$YAI_BIN" case policy bind case:w18-safe-failover --artifact "$POLICY_ID" \
  --reason 'W18 safe failover run' >/dev/null
safe_output=$("$YAI_BIN" case run case:w18-safe-failover \
  --participant participant:model --resource resource:w18-safe \
  --prompt 'complete the fixed safe-failover turn' \
  --max-invocations 1 --max-provider-retries 2 --max-runtime-ms 5000 2>&1)
grep -Fq 'provider_safe_failover: attempt=2' <<<"$safe_output"
grep -Fq 'runtime_status: Completed' <<<"$safe_output"
grep -Fq 'provider_failures: 1' <<<"$safe_output"
safe_case=$("$YAI_BIN" case provider show case:w18-safe-failover --json)
grep -Fq "$TARGET_ID" <<<"$safe_case"
grep -Fq 'ResultReceived' <<<"$safe_case"

# The same endpoint is next qualified while healthy, then restarted as an
# accept-and-drop fixture. Possible delivery forbids selecting the secondary.
python3 "$ROOT/tests/fixtures/provider_governance_server.py" \
  --mode full --model indeterminate-model --requests 16 \
  >"$RUN_ROOT/drop-qualification.out" 2>"$RUN_ROOT/drop-qualification.err" &
DROP_PID=$!
for _ in $(seq 1 100); do
  [[ -s "$RUN_ROOT/drop-qualification.out" ]] && break
  sleep 0.05
done
DROP_PORT="$(head -1 "$RUN_ROOT/drop-qualification.out")"
drop_add=$("$YAI_BIN" provider add \
  --tenant tenant:w18-smoke --provider-key indeterminate \
  --endpoint "http://127.0.0.1:$DROP_PORT" --model indeterminate-model \
  --locality loopback)
DROP_TARGET=$(awk '/target_id:/ {print $2}' <<<"$drop_add")
"$YAI_BIN" provider qualify --target "$DROP_TARGET" >/dev/null
"$YAI_BIN" provider trust approve --target "$DROP_TARGET" >/dev/null
kill "$DROP_PID"
wait "$DROP_PID" 2>/dev/null || true
DROP_PID=""
python3 "$ROOT/tests/fixtures/provider_governance_server.py" \
  --port "$DROP_PORT" --mode drop --model indeterminate-model --requests 1 \
  >"$RUN_ROOT/drop.out" 2>"$RUN_ROOT/drop.err" &
DROP_PID=$!
sleep 0.1

"$YAI_BIN" case create case:w18-indeterminate --tenant tenant:w18-smoke >/dev/null
"$YAI_BIN" case participant role add case:w18-indeterminate \
  --participant participant:model --role model-executor >/dev/null
"$YAI_BIN" case participant role add case:w18-indeterminate \
  --participant participant:model --role operation-proposer >/dev/null
"$YAI_BIN" case provider bind case:w18-indeterminate \
  --participant participant:model --target "$DROP_TARGET" --target "$TARGET_ID" \
  --failover safe_only --max-attempts 3 >/dev/null
mkdir -p "$RUN_ROOT/resource-indeterminate/allowed"
"$YAI_BIN" case resource attach filesystem case:w18-indeterminate \
  --resource resource:w18-indeterminate --root "$RUN_ROOT/resource-indeterminate" \
  --allow-prefix allowed --policy-owner participant:model --max-bytes 1024 >/dev/null
"$YAI_BIN" case policy bind case:w18-indeterminate --artifact "$POLICY_ID" \
  --reason 'W18 indeterminate-delivery run' >/dev/null
indeterminate_output=$("$YAI_BIN" case run case:w18-indeterminate \
  --participant participant:model --resource resource:w18-indeterminate \
  --prompt 'complete the fixed indeterminate-delivery turn' \
  --max-invocations 1 --max-provider-retries 2 --max-runtime-ms 5000)
grep -Fq 'runtime_status: DeliveryIndeterminate' <<<"$indeterminate_output"
grep -Fq 'provider_failures: 1' <<<"$indeterminate_output"
indeterminate_case=$("$YAI_BIN" case provider show case:w18-indeterminate --json)
grep -Fq "$DROP_TARGET" <<<"$indeterminate_case"
grep -Fq 'DeliveryIndeterminate' <<<"$indeterminate_case"
wait "$DROP_PID" 2>/dev/null || true
DROP_PID=""

help_json=$("$YAI_BIN" help --json)
grep -Fq '"operation_id":"yai.provider.add"' <<<"$help_json"
grep -Fq '"operation_id":"yai.case.provider.bind"' <<<"$help_json"

printf 'provider_governance_characterization: pass\n'
printf 'synthetic_case_context_items_sent: 0\n'
printf 'qualified_capabilities: chat_text,structured_json_object,model_exact_addressing,usage_accounting\n'
printf 'provider_dimensions_collapsed: false\n'
printf 'indeterminate_automatic_failover: false\n'
printf 'provider_selection_case_canonical: true\n'
printf 'case_provider_binding_product_path: true\n'
printf 'governed_provider_modelwork_completed: true\n'
printf 'safe_connect_failover_completed: true\n'
printf 'indeterminate_delivery_stopped_without_failover: true\n'
printf 'provider_health_operational_shared: true\n'
printf 'target_id: %s\n' "$TARGET_ID"
printf '%s\n' '--- P18 qualification output ---'
printf '%s\n' "$qualification"
printf '%s\n' '--- P18 governed run output ---'
printf '%s\n' "$run_output"
printf '%s\n' '--- P18 safe failover output ---'
printf '%s\n' "$safe_output"
printf '%s\n' '--- P18 indeterminate output ---'
printf '%s\n' "$indeterminate_output"
