#!/usr/bin/env bash
set -euo pipefail
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)
RUN=$(mktemp -d /tmp/yai-policy-native.XXXXXX)
export YAI_HOME="$RUN/home"
cd "$ROOT"
printf 'run_id=%s cwd=%s pre_state=fresh_home provider_mode=no_provider\n' "${RUN##*/}" "$ROOT"
# Keep the bounded qualification home for replay/inspection; never a canary.
for format in json md pdf; do
  tenant="tenant:policy-$format"
  case_id="case:policy-$format"
  printf '\nFORMAT %s\n' "$format"
  ./yai init --tenant "$tenant" --organization organization:engineering
  ./yai case create "$case_id" --tenant "$tenant"
  ./yai policy extract "tests/cases/05-governance-cognitive/policy.$format" | jq -e 'select(.candidate.validation.status == "qualified") | {original_digest,original_bytes,source,validation:.candidate.validation}'
  intake=$(./yai policy ingest "tests/cases/05-governance-cognitive/policy.$format" --tenant "$tenant")
  printf '%s\n' "$intake"
  artifact=$(sed -n 's/^artifact_id: //p' <<<"$intake" | head -1)
  [[ "$artifact" == policy-artifact:* ]]
  if ./yai policy publish "$artifact"; then exit 1; fi
  ./yai policy validate "$artifact" --reason 'Inspected exact enterprise policy rules'
  ./yai policy publish "$artifact" --reason 'Explicit automated qualification publication'
  ./yai case policy bind "$case_id" --artifact "$artifact" --reason 'Exact scoped enterprise policy'
  ./yai case policy status "$case_id" | tee "$RUN/$format-status.txt"
  rg -q '^normative_readiness: ready$' "$RUN/$format-status.txt"
  sed -n 's/^effective_rule: //p' "$RUN/$format-status.txt" | jq -sS 'map(del(.contributions)) | sort_by(.kind,.operation_kind)' > "$RUN/$format-rules.json"
  ./yai case policy rebuild --case "$case_id"
  ./yai case verify "$case_id"
done
cmp "$RUN/json-rules.json" "$RUN/md-rules.json"
cmp "$RUN/json-rules.json" "$RUN/pdf-rules.json"
./yai policy extract tests/cases/05-governance-cognitive/ambiguous.md | jq -e '.extraction.unresolved | length > 0'
if ./yai policy ingest tests/cases/05-governance-cognitive/ambiguous.md --tenant tenant:policy-json; then exit 1; fi
printf 'product_policy_intake=PASS formats=JSON,Markdown,PDF publish_before_validation=REFUSED ambiguous=REFUSED model_invocations=0 retained_home=%s\n' "$YAI_HOME"
