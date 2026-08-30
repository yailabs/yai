#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
TEST_DIR="$(mktemp -d /tmp/yai-governance-hardening.XXXXXX)"
export YAI_HOME="$TEST_DIR/home"

cleanup() {
  if [[ "${YAI_KEEP_TEST_DIR:-0}" != "1" ]]; then
    rm -rf "$TEST_DIR"
  fi
}
trap cleanup EXIT INT TERM

require_text() {
  grep -Fq -- "$2" <<<"$1"
}

write_policy() {
  local path="$1"
  local owner="$2"
  local required="$3"
  cat >"$path" <<JSON
{
  "schema": "yai.policy_source_input.v4",
  "policy_key": "production.filesystem",
  "source_version": "1",
  "owner_ref": "$owner",
  "source_origin": {
    "source_system": "hardening-fixture",
    "source_uri": "test://governance-hardening/$owner"
  },
  "validity": {"mode": "unbounded"},
  "rules": [{
    "kind": "review_requirement",
    "rule_id": "review-write",
    "operation_kind": "filesystem.write",
    "resource_kind": "filesystem",
    "required": $required,
    "reason": "explicit owner-scoped review posture"
  }]
}
JSON
}

mkdir -p "$YAI_HOME" "$TEST_DIR/sources"
"$YAI_BIN" security bootstrap-local --tenant tenant:hardening-a \
  --organization organization:a >/dev/null
"$YAI_BIN" security bootstrap-local --tenant tenant:hardening-b \
  --organization organization:b >/dev/null
write_policy "$TEST_DIR/sources/a.json" "organization:a" true
write_policy "$TEST_DIR/sources/b.json" "organization:b" false

for owner in a b; do
  ingest=$("$YAI_BIN" policy ingest "$TEST_DIR/sources/$owner.json" \
    --tenant "tenant:hardening-$owner")
  artifact_id=$(sed -n 's/^artifact_id: //p' <<<"$ingest" | head -1)
  [[ "$artifact_id" == policy-artifact:* ]]
  "$YAI_BIN" policy validate "$artifact_id" >/dev/null
  "$YAI_BIN" policy publish "$artifact_id" >/dev/null
  if [[ "$owner" == "a" ]]; then
    artifact_a="$artifact_id"
  else
    artifact_b="$artifact_id"
  fi
done

inspect_a=$("$YAI_BIN" policy inspect "$artifact_a")
inspect_b=$("$YAI_BIN" policy inspect "$artifact_b")
require_text "$inspect_a" "owner_ref: organization:a"
require_text "$inspect_a" "lifecycle: published"
require_text "$inspect_b" "owner_ref: organization:b"
require_text "$inspect_b" "lifecycle: published"

# Same owner/key/declared-version with changed immutable bytes is rejected and
# cannot persist the newly content-addressed source.
write_policy "$TEST_DIR/sources/a-collision.json" "organization:a" false
set +e
collision=$("$YAI_BIN" policy ingest "$TEST_DIR/sources/a-collision.json" \
  --tenant tenant:hardening-a 2>&1)
collision_code=$?
set -e
[[ "$collision_code" -ne 0 ]]
require_text "$collision" "policy_version_identity_collision"

cat >"$TEST_DIR/sources/duplicate-top.json" <<'JSON'
{"schema":"yai.policy_source_input.v4","policy_key":"a","policy_key":"b","source_version":"1","owner_ref":"organization:a","source_origin":{"source_system":"test","source_uri":"test://duplicate"},"validity":{"mode":"unbounded"},"rules":[]}
JSON
cat >"$TEST_DIR/sources/duplicate-rule.json" <<'JSON'
{"schema":"yai.policy_source_input.v4","policy_key":"a","source_version":"1","owner_ref":"organization:a","source_origin":{"source_system":"test","source_uri":"test://duplicate"},"validity":{"mode":"unbounded"},"rules":[{"kind":"review_requirement","rule_id":"r","operation_kind":"filesystem.write","required":true,"required":false,"reason":"x"}]}
JSON
for duplicate in duplicate-top duplicate-rule; do
  set +e
  output=$("$YAI_BIN" policy ingest "$TEST_DIR/sources/$duplicate.json" \
    --tenant tenant:hardening-a 2>&1)
  code=$?
  set -e
  [[ "$code" -ne 0 ]]
  require_text "$output" "duplicate_json_key"
done

listing_a=$("$YAI_BIN" policy list --tenant tenant:hardening-a)
listing_b=$("$YAI_BIN" policy list --tenant tenant:hardening-b)
listing="$listing_a
$listing_b"
require_text "$listing_a" "policy_artifacts: 1"
require_text "$listing_b" "policy_artifacts: 1"
canonical=$("$YAI_BIN" store summary)
require_text "$canonical" "transitions_total: 0"
require_text "$canonical" "cases_materialized: 0"

printf 'governance_hardening_characterization: pass\n'
printf 'published_lineages: 2\n'
printf 'version_collision: rejected\n'
printf 'duplicate_keys: rejected\n'
printf 'case_transitions: 0\n'
