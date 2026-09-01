#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
TEST_DIR="$(mktemp -d /tmp/yai-tenant-security.XXXXXX)"
export YAI_HOME="$TEST_DIR/home"

cleanup() {
  if [[ "${YAI_KEEP_TEST_DIR:-0}" != "1" ]]; then rm -rf "$TEST_DIR"; fi
}
trap cleanup EXIT INT TERM

require_text() { grep -Fq -- "$2" <<<"$1"; }
trace_product() {
  [[ "${YAI_EXECUTION_EVIDENCE:-0}" == "1" ]] || return 0
  if [[ "${YAI_EVIDENCE_COMPACT:-0}" == "1" && "$4" == "0" ]]; then
    local bounded
    bounded=$(grep -E '^(security_bootstrap|authenticated|authentication_kind|authn_method|real_uid|effective_uid|real_gid|effective_gid|principal_id|tenant_id|organization_ref|membership|tenant_relations|security_events|case_created|case_id|case_generation|lifecycle|source_digest|artifact_id|policy_lineage_id|binding_id|effective_policy_id|normative_readiness|resource_attachment|error):' <<<"$3" || true)
    printf '\n[product-command:%s]\n$ %s\n%s\nexit: %s\n' "$1" "$2" "$bounded" "$4"
  else
    printf '\n[product-command:%s]\n$ %s\n%s\nexit: %s\n' "$1" "$2" "$3" "$4"
  fi
}

mkdir -p "$YAI_HOME" "$TEST_DIR/policy" "$TEST_DIR/root/allowed" \
  "$TEST_DIR/root/nested/allowed"
bootstrap_a=$("$YAI_BIN" security bootstrap-local \
  --tenant tenant:product-a --organization organization:shared)
trace_product 01 "YAI_HOME=$YAI_HOME $YAI_BIN security bootstrap-local --tenant tenant:product-a --organization organization:shared" "$bootstrap_a" 0
require_text "$bootstrap_a" "authentication_kind: local_posix_effective_credential"
principal_id=$(sed -n 's/^principal_id: //p' <<<"$bootstrap_a" | head -1)
[[ "$principal_id" == principal:* ]]

bootstrap_b=$("$YAI_BIN" security bootstrap-local \
  --tenant tenant:product-b --organization organization:shared)
trace_product 02 "YAI_HOME=$YAI_HOME $YAI_BIN security bootstrap-local --tenant tenant:product-b --organization organization:shared" "$bootstrap_b" 0
whoami=$("$YAI_BIN" identity whoami)
trace_product 03 "YAI_HOME=$YAI_HOME $YAI_BIN identity whoami" "$whoami" 0
require_text "$whoami" "Authentication local_posix_effective_credential"
require_text "$whoami" "tenant:product-a, tenant:product-b"

case_a=$("$YAI_BIN" case create --case case:product-a --tenant tenant:product-a)
case_b=$("$YAI_BIN" case create --case case:product-b --tenant tenant:product-b)
trace_product 04 "YAI_HOME=$YAI_HOME $YAI_BIN case create --case case:product-a --tenant tenant:product-a" "$case_a" 0
trace_product 05 "YAI_HOME=$YAI_HOME $YAI_BIN case create --case case:product-b --tenant tenant:product-b" "$case_b" 0
require_text "$case_a" "tenant_id: tenant:product-a"
require_text "$case_b" "tenant_id: tenant:product-b"

for case_id in case:product-a case:product-b; do
  "$YAI_BIN" case bind-participant-role --case "$case_id" \
    --participant subject:policy-pack --role resource-attachment-compatibility-owner >/dev/null
done

cat >"$TEST_DIR/policy/shared.json" <<'JSON'
{"schema":"yai.policy_source_input.v4","policy_key":"production.security","source_version":"1","owner_ref":"organization:shared","source_origin":{"source_system":"wave12-product","source_uri":"test://wave12/shared"},"validity":{"mode":"unbounded"},"rules":[{"kind":"operation_restriction","rule_id":"allow-write","operation_kind":"filesystem.write","resource_kind":"filesystem","effect":"allow","reason":"explicit Tenant-scoped admission"}]}
JSON

ingest_a=$("$YAI_BIN" policy ingest "$TEST_DIR/policy/shared.json" --tenant tenant:product-a)
ingest_b=$("$YAI_BIN" policy ingest "$TEST_DIR/policy/shared.json" --tenant tenant:product-b)
artifact_a=$(sed -n 's/^artifact_id: //p' <<<"$ingest_a" | head -1)
artifact_b=$(sed -n 's/^artifact_id: //p' <<<"$ingest_b" | head -1)
source_a=$(sed -n 's/^source_digest: //p' <<<"$ingest_a" | head -1)
source_b=$(sed -n 's/^source_digest: //p' <<<"$ingest_b" | head -1)
[[ "$source_a" == "$source_b" ]]
[[ "$artifact_a" != "$artifact_b" ]]
for artifact in "$artifact_a" "$artifact_b"; do
  "$YAI_BIN" policy validate "$artifact" --reason "Wave12 product validation" >/dev/null
  "$YAI_BIN" policy publish "$artifact" --reason "Wave12 product publication" >/dev/null
done
trace_product 06 "YAI_HOME=$YAI_HOME $YAI_BIN policy ingest $TEST_DIR/policy/shared.json --tenant tenant:product-a" "$ingest_a" 0
trace_product 07 "YAI_HOME=$YAI_HOME $YAI_BIN policy ingest $TEST_DIR/policy/shared.json --tenant tenant:product-b" "$ingest_b" 0

generation_a=$("$YAI_BIN" case policy status --case case:product-a | sed -n 's/^case_generation: //p' | head -1)
set +e
cross_bind=$("$YAI_BIN" case policy bind --case case:product-a \
  --artifact "$artifact_b" --expected-generation "$generation_a" \
  --reason "must fail across Tenant" 2>&1)
cross_bind_exit=$?
set -e
[[ "$cross_bind_exit" -ne 0 ]]
require_text "$cross_bind" "cross_tenant_case_policy_binding_rejected"
trace_product 08 "YAI_HOME=$YAI_HOME $YAI_BIN case policy bind --case case:product-a --artifact $artifact_b --expected-generation $generation_a --reason 'must fail across Tenant'" "$cross_bind" "$cross_bind_exit"

bound=$("$YAI_BIN" case policy bind --case case:product-a \
  --artifact "$artifact_a" --expected-generation "$generation_a" \
  --reason "exact Tenant policy")
trace_product 09 "YAI_HOME=$YAI_HOME $YAI_BIN case policy bind --case case:product-a --artifact $artifact_a --expected-generation $generation_a --reason 'exact Tenant policy'" "$bound" 0
require_text "$bound" "normative_readiness: ready"

attach_a=$("$YAI_BIN" case attach-filesystem --case case:product-a \
  --attachment workspace-a --root "$TEST_DIR/root" --allow-prefix allowed \
  --policy-owner subject:policy-pack --max-bytes 256)
trace_product 10 "YAI_HOME=$YAI_HOME $YAI_BIN case attach-filesystem --case case:product-a --attachment workspace-a --root $TEST_DIR/root --allow-prefix allowed --policy-owner subject:policy-pack --max-bytes 256" "$attach_a" 0
set +e
root_exact=$("$YAI_BIN" case attach-filesystem --case case:product-b \
  --attachment workspace-b --root "$TEST_DIR/root" --allow-prefix allowed \
  --policy-owner subject:policy-pack --max-bytes 256 2>&1)
root_exact_exit=$?
root_overlap=$("$YAI_BIN" case attach-filesystem --case case:product-b \
  --attachment workspace-b-nested --root "$TEST_DIR/root/nested" --allow-prefix allowed \
  --policy-owner subject:policy-pack --max-bytes 256 2>&1)
root_overlap_exit=$?
set -e
[[ "$root_exact_exit" -ne 0 && "$root_overlap_exit" -ne 0 ]]
require_text "$root_exact" "cross_tenant_filesystem_root_overlap"
require_text "$root_overlap" "cross_tenant_filesystem_root_overlap"
trace_product 11 "YAI_HOME=$YAI_HOME $YAI_BIN case attach-filesystem --case case:product-b --attachment workspace-b --root $TEST_DIR/root --allow-prefix allowed --policy-owner subject:policy-pack --max-bytes 256" "$root_exact" "$root_exact_exit"
trace_product 12 "YAI_HOME=$YAI_HOME $YAI_BIN case attach-filesystem --case case:product-b --attachment workspace-b-nested --root $TEST_DIR/root/nested --allow-prefix allowed --policy-owner subject:policy-pack --max-bytes 256" "$root_overlap" "$root_overlap_exit"

set +e
spoof=$("$YAI_BIN" case cancel --case case:product-a --as participant:forged \
  --reason "spoof must fail" 2>&1)
spoof_exit=$?
set -e
[[ "$spoof_exit" -ne 0 ]]
require_text "$spoof" "caller_supplied_as_cannot_authenticate_principal"
trace_product 13 "YAI_HOME=$YAI_HOME $YAI_BIN case cancel --case case:product-a --as participant:forged --reason 'spoof must fail'" "$spoof" "$spoof_exit"

status=$("$YAI_BIN" case policy status --case case:product-a)
require_text "$status" "case_cancelled: false"
require_text "$status" "tenant_id: tenant:product-a"
printf 'tenant_security_characterization: pass\n'
printf 'principal_id: %s\nartifact_a: %s\nartifact_b: %s\n' \
  "$principal_id" "$artifact_a" "$artifact_b"
printf 'cross_bind_exit: %s\nroot_exact_exit: %s\nroot_overlap_exit: %s\nspoof_exit: %s\n' \
  "$cross_bind_exit" "$root_exact_exit" "$root_overlap_exit" "$spoof_exit"
