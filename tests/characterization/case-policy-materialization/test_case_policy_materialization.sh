#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
YAID="$ROOT/build/yaid"
TEST_DIR="$(mktemp -d /tmp/yai-case-policy.XXXXXX)"
SOCKET="/tmp/yai-wave9-$$.sock"
BASE_JOURNAL="$TEST_DIR/base.jsonl"
DAEMON_PID=""
CASE_HOME="$TEST_DIR/home"

cleanup() {
  if [[ -n "$DAEMON_PID" ]]; then
    "$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null 2>&1 || true
    wait "$DAEMON_PID" >/dev/null 2>&1 || true
  fi
  rm -f "$SOCKET"
  if [[ "${YAI_KEEP_TEST_DIR:-0}" != "1" ]]; then rm -rf "$TEST_DIR"; fi
}
trap cleanup EXIT INT TERM

require_text() { grep -Fq -- "$2" <<<"$1"; }

write_policy() {
  local path="$1" key="$2" version="$3" effect="$4" review="$5"
  cat >"$path" <<JSON
{"schema":"yai.policy_source_input.v2","policy_key":"$key","source_version":"$version","owner_ref":"organization:wave9","source_origin":{"source_system":"wave9-characterization","source_uri":"test://wave9/$key/$version"},"rules":[{"kind":"operation_restriction","rule_id":"operation-$key-$version","operation_kind":"filesystem.write","resource_kind":"filesystem","effect":"$effect","reason":"deterministic operation posture"},{"kind":"review_requirement","rule_id":"review-$key-$version","operation_kind":"filesystem.write","resource_kind":"filesystem","required":$review,"reason":"deterministic review posture"},{"kind":"evidence_obligation","rule_id":"evidence-$key-$version","operation_kind":"filesystem.write","resource_kind":"filesystem","obligation":"post_observation","reason":"observed consequence required"}]}
JSON
}

mkdir -p "$TEST_DIR/daemon-home" "$CASE_HOME" "$TEST_DIR/policies"
YAI_HOME="$TEST_DIR/daemon-home" "$YAID" --socket "$SOCKET" --foreground >"$TEST_DIR/yaid.log" 2>&1 &
DAEMON_PID=$!
for _ in $(seq 1 100); do [[ -S "$SOCKET" ]] && break; sleep 0.02; done
[[ -S "$SOCKET" ]]
loop_output=$(YAI_HOME="$TEST_DIR/daemon-home" "$YAI_BIN" daemon run-filesystem-loop --socket "$SOCKET")
source_journal=$(sed -n 's/.*"journal_path":"\([^"]*\)".*/\1/p' <<<"$loop_output")
cp "$ROOT/$source_journal" "$BASE_JOURNAL"
YAI_HOME="$TEST_DIR/daemon-home" "$YAI_BIN" daemon shutdown --socket "$SOCKET" >/dev/null
wait "$DAEMON_PID"
DAEMON_PID=""
export YAI_HOME="$CASE_HOME"
YAI_JOURNAL="$BASE_JOURNAL" "$YAI_BIN" case enter --case case:new12-filesystem --subject subject:llm-provider >/dev/null

write_policy "$TEST_DIR/policies/security-v1.json" filesystem-security 1 allow false
write_policy "$TEST_DIR/policies/security-v2.json" filesystem-security 2 deny true
write_policy "$TEST_DIR/policies/audit-v1.json" audit-obligations 1 deny true

candidate=$("$YAI_BIN" policy ingest "$TEST_DIR/policies/security-v1.json" --as participant:policy-admin)
v1=$(sed -n 's/^artifact_id: //p' <<<"$candidate" | head -1)
generation=$("$YAI_BIN" case policy status --case case:new12-filesystem | sed -n 's/^case_generation: //p')
set +e
candidate_error=$("$YAI_BIN" case policy bind --case case:new12-filesystem --artifact "$v1" --expected-generation "$generation" --as participant:operator 2>&1)
candidate_code=$?
set -e
[[ "$candidate_code" -ne 0 ]]
require_text "$candidate_error" "not_eligible"
v1_validation=$("$YAI_BIN" policy validate "$v1" --as participant:policy-admin)
v1_publication=$("$YAI_BIN" policy publish "$v1" --as participant:policy-admin)
bound=$("$YAI_BIN" case policy bind --case case:new12-filesystem --artifact "$v1" --expected-generation "$generation" --as participant:operator)
require_text "$bound" "case_policy_bind: committed"
require_text "$bound" "normative_readiness: ready"
binding=$(sed -n 's/^policy_binding: binding_id=\([^ ]*\).*/\1/p' <<<"$bound" | head -1)

audit_candidate=$("$YAI_BIN" policy ingest "$TEST_DIR/policies/audit-v1.json" --as participant:policy-admin)
audit=$(sed -n 's/^artifact_id: //p' <<<"$audit_candidate" | head -1)
audit_validation=$("$YAI_BIN" policy validate "$audit" --as participant:policy-admin)
audit_publication=$("$YAI_BIN" policy publish "$audit" --as participant:policy-admin)
generation=$(sed -n 's/^case_generation: //p' <<<"$bound" | head -1)
multi=$("$YAI_BIN" case policy bind --case case:new12-filesystem --artifact "$audit" --expected-generation "$generation" --as participant:operator)
require_text "$multi" "active_policy_bindings: 2"
require_text "$multi" "effective_resolved_conflicts: 2"
require_text "$multi" "decision_count: 0"
require_text "$multi" "execution_grant_count: 0"
require_text "$multi" "prepared_effect_count: 0"

v2_candidate=$("$YAI_BIN" policy ingest "$TEST_DIR/policies/security-v2.json" --as participant:policy-admin)
v2=$(sed -n 's/^artifact_id: //p' <<<"$v2_candidate" | head -1)
v2_validation=$("$YAI_BIN" policy validate "$v2" --as participant:policy-admin)
v2_publication=$("$YAI_BIN" policy publish "$v2" --as participant:policy-admin)
pinned=$("$YAI_BIN" case policy status --case case:new12-filesystem)
require_text "$pinned" "artifact_id=$v1 version=1"
require_text "$pinned" "status=superseded:current=$v2"
generation=$(sed -n 's/^case_generation: //p' <<<"$pinned")
replaced=$("$YAI_BIN" case policy replace --case case:new12-filesystem --binding "$binding" --artifact "$v2" --expected-generation "$generation" --as participant:operator)
require_text "$replaced" "case_policy_replace: committed"
require_text "$replaced" "artifact_id=$v2 version=2"

before=$(sed -n 's/^transition_count: //p' <<<"$replaced")
after_status=$("$YAI_BIN" case policy status --case case:new12-filesystem)
after=$(sed -n 's/^transition_count: //p' <<<"$after_status")
[[ "$before" == "$after" ]]
rebuilt=$("$YAI_BIN" case policy rebuild --case case:new12-filesystem)
require_text "$rebuilt" "canonical_transitions_before: $before"
require_text "$rebuilt" "canonical_transitions_after: $before"

if [[ "${YAI_EVIDENCE_COMPACT:-0}" == "1" ]]; then
  compact_lines='^(policy_ingest|policy_validate|policy_publish|source_id|artifact_id|artifact_version|lifecycle|runtime_consumable|case_policy_|transition_id|case_id|case_generation|transition_count|normative_readiness|active_policy_bindings|policy_binding|effective_policy_id|effective_policy_digest|materializer_version|effective_input_rules|effective_output_rules|effective_merged_rules|effective_resolved_conflicts|effective_provenance_contributions|blocking_conflicts|missing_inputs|catalog_drift|decision_count|execution_grant_count|prepared_effect_count|authority_emitted_by_case_policy|effective_policy_rebuild|canonical_transitions_before|canonical_transitions_after):'
  printf '%s\n' '--- E9-P01 ingest/validate/publish security-v1 ---'
  printf '%s\n' "$candidate" "$v1_validation" "$v1_publication" | grep -E "$compact_lines"
  printf '%s\n' '--- E9-P05 candidate bind exit/stderr ---' "$candidate_code" "$candidate_error"
  printf '%s\n' '--- E9-P02 bind security-v1 ---'
  printf '%s\n' "$bound" | grep -E "$compact_lines"
  printf '%s\n' '--- E9-P04 bind second lineage / compose ---'
  printf '%s\n' "$multi" | grep -E "$compact_lines"
  printf '%s\n' '--- E9-P06 publish P@2 / remain pinned P@1 ---'
  printf '%s\n' "$v2_publication" "$pinned" | grep -E "$compact_lines"
  printf '%s\n' '--- E9-P07 explicit replace ---'
  printf '%s\n' "$replaced" | grep -E "$compact_lines"
  printf '%s\n' '--- E9-P09 pure status and rebuild ---'
  printf '%s\n' "$after_status" "$rebuilt" | grep -E "$compact_lines"
  printf 'evidence_test_dir: %s\nevidence_yai_home: %s\n' "$TEST_DIR" "$CASE_HOME"
elif [[ "${YAI_EVIDENCE_VERBOSE:-0}" == "1" ]]; then
  printf '%s\n' '--- E9-P01 ingest security-v1 ---' "$candidate"
  printf '%s\n' '--- E9-P01 validate security-v1 ---' "$v1_validation"
  printf '%s\n' '--- E9-P01 publish security-v1 ---' "$v1_publication"
  printf '%s\n' '--- E9-P05 candidate bind exit ---' "$candidate_code"
  printf '%s\n' '--- E9-P05 candidate bind stderr ---' "$candidate_error"
  printf '%s\n' '--- E9-P02 bind security-v1 ---' "$bound"
  printf '%s\n' '--- E9-P04 ingest audit-v1 ---' "$audit_candidate"
  printf '%s\n' '--- E9-P04 validate audit-v1 ---' "$audit_validation"
  printf '%s\n' '--- E9-P04 publish audit-v1 ---' "$audit_publication"
  printf '%s\n' '--- E9-P04 bind audit-v1 / multi-artifact status ---' "$multi"
  printf '%s\n' '--- E9-P06 ingest security-v2 ---' "$v2_candidate"
  printf '%s\n' '--- E9-P06 validate security-v2 ---' "$v2_validation"
  printf '%s\n' '--- E9-P06 publish security-v2 ---' "$v2_publication"
  printf '%s\n' '--- E9-P06 pinned P@1 after P@2 publication ---' "$pinned"
  printf '%s\n' '--- E9-P07 explicit replace P@1 to P@2 ---' "$replaced"
  printf '%s\n' '--- E9-P09 pure status ---' "$after_status"
  printf '%s\n' '--- E9-P09 derived rebuild ---' "$rebuilt"
  printf 'evidence_test_dir: %s\nevidence_yai_home: %s\n' "$TEST_DIR" "$CASE_HOME"
fi

printf 'case_policy_materialization_characterization: pass\n'
printf 'case_id: case:new12-filesystem\n'
printf 'policy_v1: %s\npolicy_v2: %s\n' "$v1" "$v2"
printf 'active_bindings: 2\ncanonical_transitions: %s\n' "$before"
printf 'effective_policy_id: %s\n' "$(sed -n 's/^effective_policy_id: //p' <<<"$replaced")"
printf 'authority_emitted: false\n'
