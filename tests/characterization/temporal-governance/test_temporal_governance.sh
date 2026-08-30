#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
YAID="$ROOT/build/yaid"
FIXTURE="$ROOT/tests/fixtures/controlled_effect_provider.py"
source "$ROOT/tests/characterization/lib/governed_case_policy.sh"
TEST_DIR="$(mktemp -d /tmp/yai-temporal-governance.XXXXXX)"
SOCKET="/tmp/yai-wave11-$$.sock"
BASE_JOURNAL="$TEST_DIR/base.jsonl"
CASE_HOME="$TEST_DIR/home"
CASE_JOURNAL="$TEST_DIR/journal.jsonl"
RESOURCE_ROOT="$TEST_DIR/resource"
DAEMON_PID=""
PROVIDER_PID=""

cleanup() {
  if [[ -n "$PROVIDER_PID" ]]; then
    kill "$PROVIDER_PID" >/dev/null 2>&1 || true
    wait "$PROVIDER_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "$DAEMON_PID" ]]; then
    "$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null 2>&1 || true
    wait "$DAEMON_PID" >/dev/null 2>&1 || true
  fi
  rm -f "$SOCKET"
  if [[ "${YAI_KEEP_TEST_DIR:-0}" != "1" ]]; then rm -rf "$TEST_DIR"; fi
}
trap cleanup EXIT INT TERM

require_text() { grep -Fq -- "$2" <<<"$1"; }
trace_product() {
  [[ "${YAI_EXECUTION_EVIDENCE:-0}" == "1" ]] || return 0
  printf '\n[product-command:%s]\n$ %s\n' "$1" "$2"
  if [[ "${YAI_EVIDENCE_COMPACT:-0}" == "1" ]]; then
    grep -E '^(case_|policy_|artifact_id|artifact_version|lifecycle|runtime_consumable|transition_id|normative_readiness|observed_wall_time|persisted_authority_floor|effective_authority_time|active_policy_bindings|policy_binding|effective_policy_id|catalog_drift|binding_validity|provider_invocation_id|provider_result_id|operation_id|decision_id|decision_basis_id|decision:|execution_grant_id|effect_id|effect_state|controlled_effect_crash_injected|invalidated_reviews|abandoned_grants|cancellation_|closure_|closed_at|usable_pending_reviews|usable_issued_grants|unresolved_effects|reconciliation:|receipt_id|provider_invocations|execution_grants|external_effect|case_close_blocked)' <<<"$3" || true
  else
    printf '%s\n' "$3"
  fi
  printf 'exit: %s\n' "$4"
}

mkdir -p "$TEST_DIR/daemon-home" "$CASE_HOME" "$RESOURCE_ROOT/allowed"
YAI_HOME="$TEST_DIR/daemon-home" "$YAID" --socket "$SOCKET" --foreground >"$TEST_DIR/yaid.log" 2>&1 &
DAEMON_PID=$!
for _ in $(seq 1 100); do [[ -S "$SOCKET" ]] && break; sleep 0.02; done
[[ -S "$SOCKET" ]]
loop_output=$("$YAI_BIN" daemon run-filesystem-loop --socket "$SOCKET")
source_journal=$(sed -n 's/.*"journal_path":"\([^"]*\)".*/\1/p' <<<"$loop_output")
cp "$ROOT/$source_journal" "$BASE_JOURNAL"
"$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null
wait "$DAEMON_PID"
DAEMON_PID=""
cp "$BASE_JOURNAL" "$CASE_JOURNAL"

yai_bootstrap_tenant_case "$YAI_BIN" "$CASE_HOME" case:new12-filesystem
YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case enter \
  --case case:new12-filesystem --subject subject:llm-provider >/dev/null
YAI_HOME="$CASE_HOME" "$YAI_BIN" case attach-filesystem \
  --case case:new12-filesystem --attachment workspace --root "$RESOURCE_ROOT" \
  --allow-prefix allowed --policy-owner subject:policy-pack --max-bytes 256 >/dev/null

YAI_POLICY_EXECUTION_EVIDENCE=0
p1=$(yai_configure_governed_filesystem_case "$YAI_BIN" "$CASE_HOME" \
  case:new12-filesystem temporal-filesystem 1 allow subject:llm-provider)
valid_status=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" case policy status --case case:new12-filesystem)
require_text "$valid_status" "policy_validity: Valid"
require_text "$valid_status" "artifact_id=$p1 version=1"
trace_product 01 "YAI_HOME=$CASE_HOME $YAI_BIN case policy status --case case:new12-filesystem" "$valid_status" 0

p2_source="$TEST_DIR/temporal-filesystem-v2.policy.json"
printf '%s\n' '{"schema":"yai.policy_source_input.v4","policy_key":"temporal-filesystem","source_version":"2","owner_ref":"organization:characterization","source_origin":{"source_system":"characterization","source_uri":"test://temporal-filesystem/2"},"validity":{"mode":"unbounded"},"rules":[{"kind":"operation_restriction","rule_id":"filesystem-posture-v2","operation_kind":"filesystem.write","resource_kind":"filesystem","effect":"allow","reason":"explicit refreshed posture"},{"kind":"authority_requirement","rule_id":"filesystem-proposer-v2","operation_kind":"filesystem.write","resource_kind":"filesystem","subject":"proposer","required_role":"operation-proposer","reason":"Case-bound proposer"},{"kind":"evidence_obligation","rule_id":"filesystem-source-v2","operation_kind":"filesystem.write","resource_kind":"filesystem","obligation":"source_provenance","reason":"canonical provider lineage"},{"kind":"evidence_obligation","rule_id":"filesystem-post-v2","operation_kind":"filesystem.write","resource_kind":"filesystem","obligation":"post_observation","reason":"observed closure"}]}' >"$p2_source"
p2_ingest=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" policy ingest "$p2_source" --tenant tenant:characterization)
p2=$(sed -n 's/^artifact_id: //p' <<<"$p2_ingest" | head -1)
p2_validate=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" policy validate "$p2" --reason "validate refresh")
p2_publish=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" policy publish "$p2" --reason "publish refresh")
trace_product 02 "YAI_HOME=$CASE_HOME $YAI_BIN policy publish $p2 --reason 'publish refresh'" "$p2_publish" 0

stale_status=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" case policy status --case case:new12-filesystem)
require_text "$stale_status" "artifact_id=$p1 version=1"
require_text "$stale_status" "policy_validity: Stale"
require_text "$stale_status" "status=superseded:current=$p2"
trace_product 03 "YAI_HOME=$CASE_HOME $YAI_BIN case policy status --case case:new12-filesystem" "$stale_status" 0
binding=$(sed -n 's/^policy_binding: binding_id=\([^ ]*\).*/\1/p' <<<"$stale_status" | head -1)
generation=$(sed -n 's/^case_generation: //p' <<<"$stale_status" | head -1)
refreshed=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" case policy replace --case case:new12-filesystem \
  --binding "$binding" --artifact "$p2" --expected-generation "$generation" \
  --reason "explicit temporal refresh")
require_text "$refreshed" "policy_validity: Valid"
require_text "$refreshed" "artifact_id=$p2 version=2"
trace_product 04 "YAI_HOME=$CASE_HOME $YAI_BIN case policy replace --case case:new12-filesystem --binding $binding --artifact $p2 --expected-generation $generation --reason 'explicit temporal refresh'" "$refreshed" 0

port_file="$TEST_DIR/provider.port"
python3 "$FIXTURE" allow >"$port_file" &
PROVIDER_PID=$!
for _ in $(seq 1 100); do [[ -s "$port_file" ]] && break; sleep 0.02; done
PROVIDER_PORT=$(tr -d '[:space:]' <"$port_file")
YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" case attach-provider \
  --case case:new12-filesystem --subject subject:llm-provider --provider-id provider:temporal \
  --base-url "http://127.0.0.1:$PROVIDER_PORT/v1/chat/completions" --model controlled-model >/dev/null
set +e
prepare_output=$(YAI_HOME="$CASE_HOME" YAI_JOURNAL="$CASE_JOURNAL" "$YAI_BIN" effect filesystem-write \
  --case case:new12-filesystem --subject subject:llm-provider --attachment workspace \
  --prompt "prepare one temporal write" --provider-id provider:temporal \
  --base-url "http://127.0.0.1:$PROVIDER_PORT/v1/chat/completions" --model controlled-model \
  --failpoint after_effect_before_finalize 2>&1)
prepare_exit=$?
set -e
kill "$PROVIDER_PID" >/dev/null 2>&1 || true
wait "$PROVIDER_PID" >/dev/null 2>&1 || true
PROVIDER_PID=""
[[ "$prepare_exit" -eq 86 ]]
require_text "$prepare_output" "effect_state: prepared_durable_before_mutation"
trace_product 05 "YAI_HOME=$CASE_HOME YAI_JOURNAL=$CASE_JOURNAL $YAI_BIN effect filesystem-write --case case:new12-filesystem --subject subject:llm-provider --attachment workspace --prompt 'prepare one temporal write' --provider-id provider:temporal --base-url http://127.0.0.1:$PROVIDER_PORT/v1/chat/completions --model controlled-model --failpoint after_effect_before_finalize" "$prepare_output" "$prepare_exit"

revoke_output=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" policy revoke "$p2" \
  --reason "withdraw before future authority")
require_text "$revoke_output" "policy_revoke: revoked"
require_text "$revoke_output" "lifecycle: revoked"
require_text "$revoke_output" "runtime_consumable: false"
trace_product 06 "YAI_HOME=$CASE_HOME $YAI_BIN policy revoke $p2 --reason 'withdraw before future authority'" "$revoke_output" 0

cancel_output=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" case cancel --case case:new12-filesystem \
  --reason "stop after prepared external attempt")
require_text "$cancel_output" "case_cancel: cancelled"
require_text "$cancel_output" "unresolved_effects: 1"
trace_product 07 "YAI_HOME=$CASE_HOME $YAI_BIN case cancel --case case:new12-filesystem --reason 'stop after prepared external attempt'" "$cancel_output" 0

set +e
unsafe_close=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" case close --case case:new12-filesystem \
  --reason "unsafe close" 2>&1)
unsafe_close_exit=$?
set -e
[[ "$unsafe_close_exit" -ne 0 ]]
require_text "$unsafe_close" "case_close_blocked: unresolved_effect:"
trace_product 08 "YAI_HOME=$CASE_HOME $YAI_BIN case close --case case:new12-filesystem --reason 'unsafe close'" "$unsafe_close" "$unsafe_close_exit"

reconcile_output=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" effect reconcile --case case:new12-filesystem --retry)
require_text "$reconcile_output" "reconciliation: EffectObserved"
trace_product 09 "YAI_HOME=$CASE_HOME $YAI_BIN effect reconcile --case case:new12-filesystem --retry" "$reconcile_output" 0
close_output=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" case close --case case:new12-filesystem \
  --reason "safe after reconciliation")
require_text "$close_output" "case_close: closed"
require_text "$close_output" "case_lifecycle: Closed"
require_text "$close_output" "unresolved_effects: 0"
trace_product 10 "YAI_HOME=$CASE_HOME $YAI_BIN case close --case case:new12-filesystem --reason 'safe after reconciliation'" "$close_output" 0

set +e
closed_effect=$(YAI_HOME="$CASE_HOME" "$YAI_BIN" effect filesystem-write --case case:new12-filesystem \
  --subject subject:llm-provider --attachment workspace --prompt "must not invoke" \
  --base-url http://127.0.0.1:1/v1/chat/completions --model controlled-model 2>&1)
closed_effect_exit=$?
set -e
[[ "$closed_effect_exit" -ne 0 ]]
require_text "$closed_effect" "provider_invocations: 0"
require_text "$closed_effect" "case_closed_new_effect_forbidden"
trace_product 11 "YAI_HOME=$CASE_HOME $YAI_BIN effect filesystem-write --case case:new12-filesystem --subject subject:llm-provider --attachment workspace --prompt 'must not invoke' --base-url http://127.0.0.1:1/v1/chat/completions --model controlled-model" "$closed_effect" "$closed_effect_exit"

cargo test --manifest-path "$ROOT/engine/yai-engine/Cargo.toml" wave11_ -- --nocapture
printf 'temporal_governance_characterization: pass\n'
printf 'case_id: case:new12-filesystem\npolicy_v1: %s\npolicy_v2: %s\n' "$p1" "$p2"
printf 'prepare_exit: %s\nunsafe_close_exit: %s\nclosed_effect_exit: %s\n' "$prepare_exit" "$unsafe_close_exit" "$closed_effect_exit"
