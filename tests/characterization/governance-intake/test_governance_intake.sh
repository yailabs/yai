#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
TEST_DIR="$(mktemp -d /tmp/yai-governance-intake.XXXXXX)"
export YAI_HOME="$TEST_DIR/home"

cleanup() {
  if [[ "${YAI_KEEP_TEST_DIR:-0}" != "1" ]]; then
    rm -rf "$TEST_DIR"
  else
    printf 'preserved_test_dir:%s\n' "$TEST_DIR" >&2
  fi
}
trap cleanup EXIT INT TERM

require_text() {
  grep -Fq -- "$2" <<<"$1"
}

write_policy() {
  local path="$1"
  local version="$2"
  local required="$3"
  cat >"$path" <<JSON
{
  "schema": "yai.policy_source_input.v2",
  "policy_key": "organization.example.filesystem",
  "source_version": "$version",
  "owner_ref": "organization:example",
  "source_origin": {
    "source_system": "characterization-fixture",
    "source_uri": "test://governance-intake/version-$version"
  },
  "rules": [
    {
      "kind": "review_requirement",
      "rule_id": "review-v$version",
      "operation_kind": "filesystem.write",
      "resource_kind": "filesystem",
      "required": $required,
      "reason": "filesystem writes use explicit governance"
    },
    {
      "kind": "evidence_obligation",
      "rule_id": "post-observation-v$version",
      "operation_kind": "filesystem.write",
      "resource_kind": "filesystem",
      "obligation": "post_observation",
      "reason": "observed consequence is required"
    }
  ]
}
JSON
}

mkdir -p "$YAI_HOME" "$TEST_DIR/sources"
write_policy "$TEST_DIR/sources/v1.json" 1 true
write_policy "$TEST_DIR/sources/v2.json" 2 false

v1_ingest=$("$YAI_BIN" policy ingest "$TEST_DIR/sources/v1.json" \
  --as participant:policy-admin)
require_text "$v1_ingest" "policy_ingest: candidate_created"
require_text "$v1_ingest" "policy_source_schema: yai.policy_source_artifact.v2"
require_text "$v1_ingest" "source_system: characterization-fixture"
require_text "$v1_ingest" "source_uri: test://governance-intake/version-1"
require_text "$v1_ingest" "parsed_schema: yai.parsed_policy.v1"
require_text "$v1_ingest" "policy_ir_schema: yai.policy_ir.v1"
require_text "$v1_ingest" "lifecycle: candidate"
require_text "$v1_ingest" "runtime_consumable: false"
require_text "$v1_ingest" "case_binding: absent_without_explicit_case_action"
require_text "$v1_ingest" "decision_or_grant: never_emitted_by_policy_authoring"
v1_id=$(sed -n 's/^artifact_id: //p' <<<"$v1_ingest" | head -1)
v1_source=$(sed -n 's/^source_id: //p' <<<"$v1_ingest" | head -1)
[[ "$v1_id" == policy-artifact:* ]]
[[ "$v1_source" == policy-source:* ]]

duplicate=$("$YAI_BIN" policy ingest "$TEST_DIR/sources/v1.json" \
  --as participant:policy-admin)
require_text "$duplicate" "policy_ingest: existing_idempotent"
require_text "$duplicate" "source_created: false"
require_text "$duplicate" "artifact_created: false"
require_text "$duplicate" "lifecycle_events: 1"

set +e
publish_before_validate=$("$YAI_BIN" policy publish "$v1_id" \
  --as participant:policy-admin 2>&1)
publish_before_validate_code=$?
set -e
[[ "$publish_before_validate_code" -ne 0 ]]
require_text "$publish_before_validate" "must_be_validated_before_publish"

validated=$("$YAI_BIN" policy validate "$v1_id" --as participant:policy-admin)
require_text "$validated" "policy_validate: validated"
require_text "$validated" "lifecycle: validated"
require_text "$validated" "runtime_consumable: false"

published=$("$YAI_BIN" policy publish "$v1_id" --as participant:policy-admin)
require_text "$published" "policy_publish: published"
require_text "$published" "lifecycle: published"
require_text "$published" "runtime_consumable: true"

# Inspection and listing are pure: repeated reads do not append lifecycle
# history or touch the Case Transition authority.
inspect_before=$("$YAI_BIN" policy inspect "$v1_id")
events_before=$(sed -n 's/^lifecycle_events: //p' <<<"$inspect_before")
"$YAI_BIN" policy list >/dev/null
"$YAI_BIN" policy inspect "$v1_source" >/dev/null
inspect_after=$("$YAI_BIN" policy inspect "$v1_id")
events_after=$(sed -n 's/^lifecycle_events: //p' <<<"$inspect_after")
[[ "$events_before" == "$events_after" ]]
canonical=$("$YAI_BIN" store summary)
require_text "$canonical" "transitions_total: 0"
require_text "$canonical" "cases_materialized: 0"

# A new immutable version supersedes the old published version without
# rewriting P@1 or losing its source/provenance chain.
v2_ingest=$("$YAI_BIN" policy ingest "$TEST_DIR/sources/v2.json" \
  --as participant:policy-admin)
v2_id=$(sed -n 's/^artifact_id: //p' <<<"$v2_ingest" | head -1)
[[ "$v2_id" != "$v1_id" ]]
"$YAI_BIN" policy validate "$v2_id" --as participant:policy-admin >/dev/null
v2_published=$("$YAI_BIN" policy publish "$v2_id" --as participant:policy-admin)
require_text "$v2_published" "runtime_consumable: true"
v1_old=$("$YAI_BIN" policy inspect "$v1_id")
require_text "$v1_old" "artifact_version: 1"
require_text "$v1_old" "lifecycle: superseded"
require_text "$v1_old" "superseded_by: $v2_id"
require_text "$v1_old" "runtime_consumable: false"

# Unknown semantics remain inspectable but fail deterministic qualification;
# malformed and future schemas fail before an artifact exists.
cat >"$TEST_DIR/sources/unknown.json" <<'JSON'
{
  "schema": "yai.policy_source_input.v2",
  "policy_key": "organization.example.unknown",
  "source_version": "1",
  "owner_ref": "organization:example",
  "source_origin": {
    "source_system": "characterization-fixture",
    "source_uri": "test://governance-intake/unknown"
  },
  "rules": [{"kind": "imagined_rule", "meaning": "must not be guessed"}]
}
JSON
unknown_ingest=$("$YAI_BIN" policy ingest "$TEST_DIR/sources/unknown.json" \
  --as participant:policy-admin)
unknown_id=$(sed -n 's/^artifact_id: //p' <<<"$unknown_ingest" | head -1)
require_text "$unknown_ingest" "validation_status: blocked"
require_text "$unknown_ingest" "unresolved: code=unsupported_rule_kind"
set +e
unknown_validate=$("$YAI_BIN" policy validate "$unknown_id" \
  --as participant:policy-admin 2>&1)
unknown_validate_code=$?
set -e
[[ "$unknown_validate_code" -ne 0 ]]
require_text "$unknown_validate" "policy_artifact_qualification_blocked"

printf '{' >"$TEST_DIR/sources/malformed.json"
set +e
malformed=$("$YAI_BIN" policy ingest "$TEST_DIR/sources/malformed.json" \
  --as participant:policy-admin 2>&1)
malformed_code=$?
set -e
[[ "$malformed_code" -ne 0 ]]
require_text "$malformed" "policy_source_json_invalid"

printf '%s\n' '{"schema":"yai.policy_source_input.v99","policy_key":"p","source_version":"1","owner_ref":"o","rules":[{}]}' >"$TEST_DIR/sources/future.json"
set +e
future=$("$YAI_BIN" policy ingest "$TEST_DIR/sources/future.json" \
  --as participant:policy-admin 2>&1)
future_code=$?
set -e
[[ "$future_code" -ne 0 ]]
require_text "$future" "unsupported_policy_source_schema"

printf 'governance_intake_characterization: pass\n'
printf 'policy_v1: %s\n' "$v1_id"
printf 'policy_v2: %s\n' "$v2_id"
printf 'canonical_case_transitions: 0\n'
