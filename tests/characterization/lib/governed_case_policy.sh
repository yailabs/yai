#!/usr/bin/env bash

# Configure the smallest explicit policy needed by the live filesystem
# characterizations. Callers must have already materialized the Case and its
# participants. All IDs remain product-generated and exact-version bound.
yai_policy_trace() {
  [[ "${YAI_POLICY_EXECUTION_EVIDENCE:-${YAI_EXECUTION_EVIDENCE:-0}}" == "1" ]] || return 0
  local order="$1"
  local command_text="$2"
  local output="$3"
  printf '\n[product-command:%s]\n$ %s\n%s\nexit: 0\n' \
    "$order" "$command_text" "$output" >&2
}

yai_configure_governed_filesystem_case() {
  local yai_bin="$1"
  local yai_home="$2"
  local case_id="$3"
  local policy_key="$4"
  local policy_version="$5"
  local effect="$6"
  local proposer="$7"
  local reviewer="${8:-}"
  local policy_source="$yai_home/${policy_key//[:\/]/_}-${policy_version}.policy.json"

  local role_output
  role_output=$(YAI_HOME="$yai_home" "$yai_bin" case bind-participant-role \
    --case "$case_id" --participant "$proposer" --role operation-proposer \
    --as participant:local-policy-operator)
  yai_policy_trace 01 "YAI_HOME=$yai_home $yai_bin case bind-participant-role --case $case_id --participant $proposer --role operation-proposer --as participant:local-policy-operator" "$role_output"
  if [[ -n "$reviewer" ]]; then
    role_output=$(YAI_HOME="$yai_home" "$yai_bin" case bind-participant-role \
      --case "$case_id" --participant "$reviewer" --role operation-reviewer \
      --as participant:local-policy-operator)
    yai_policy_trace 02 "YAI_HOME=$yai_home $yai_bin case bind-participant-role --case $case_id --participant $reviewer --role operation-reviewer --as participant:local-policy-operator" "$role_output"
  fi

  {
    printf '{"schema":"yai.policy_source_input.v4","policy_key":"%s","source_version":"%s","owner_ref":"organization:characterization","source_origin":{"source_system":"characterization","source_uri":"test://%s/%s"},"validity":{"mode":"unbounded"},"rules":[' \
      "$policy_key" "$policy_version" "$policy_key" "$policy_version"
    if [[ "$effect" != "none" ]]; then
      printf '{"kind":"operation_restriction","rule_id":"filesystem-posture","operation_kind":"filesystem.write","resource_kind":"filesystem","effect":"%s","reason":"explicit characterization posture"},' "$effect"
    fi
    printf '{"kind":"authority_requirement","rule_id":"filesystem-proposer","operation_kind":"filesystem.write","resource_kind":"filesystem","subject":"proposer","required_role":"operation-proposer","reason":"proposer must be a Case-bound participant"},'
    printf '{"kind":"evidence_obligation","rule_id":"filesystem-source","operation_kind":"filesystem.write","resource_kind":"filesystem","obligation":"source_provenance","reason":"provider lineage must be canonical"},'
    printf '{"kind":"evidence_obligation","rule_id":"filesystem-post","operation_kind":"filesystem.write","resource_kind":"filesystem","obligation":"post_observation","reason":"final consequence must be observed"}'
    if [[ -n "$reviewer" ]]; then
      printf ',{"kind":"review_requirement","rule_id":"filesystem-review","operation_kind":"filesystem.write","resource_kind":"filesystem","required":true,"reason":"eligible human review is required"}'
      printf ',{"kind":"authority_requirement","rule_id":"filesystem-reviewer","operation_kind":"filesystem.write","resource_kind":"filesystem","subject":"reviewer","required_role":"operation-reviewer","reason":"reviewer must hold the Case-bound review role"}'
      printf ',{"kind":"evidence_obligation","rule_id":"filesystem-audit-reason","operation_kind":"filesystem.write","resource_kind":"filesystem","obligation":"audit_reason","reason":"human approval must carry a rationale"}'
    fi
    printf ']}'
  } >"$policy_source"

  local ingest_output artifact_id status_output generation
  ingest_output=$(YAI_HOME="$yai_home" "$yai_bin" policy ingest "$policy_source" \
    --as participant:local-policy-operator)
  yai_policy_trace 03 "YAI_HOME=$yai_home $yai_bin policy ingest $policy_source --as participant:local-policy-operator" "$ingest_output"
  artifact_id=$(sed -n 's/^artifact_id: //p' <<<"$ingest_output" | head -1)
  [[ "$artifact_id" == policy-artifact:* ]]
  local validate_output publish_output bind_output
  validate_output=$(YAI_HOME="$yai_home" "$yai_bin" policy validate "$artifact_id" \
    --as participant:local-policy-operator --reason "deterministic validation")
  yai_policy_trace 04 "YAI_HOME=$yai_home $yai_bin policy validate $artifact_id --as participant:local-policy-operator --reason 'deterministic validation'" "$validate_output"
  publish_output=$(YAI_HOME="$yai_home" "$yai_bin" policy publish "$artifact_id" \
    --as participant:local-policy-operator --reason "publish characterization policy")
  yai_policy_trace 05 "YAI_HOME=$yai_home $yai_bin policy publish $artifact_id --as participant:local-policy-operator --reason 'publish characterization policy'" "$publish_output"
  status_output=$(YAI_HOME="$yai_home" "$yai_bin" case policy status --case "$case_id")
  yai_policy_trace 06 "YAI_HOME=$yai_home $yai_bin case policy status --case $case_id" "$status_output"
  generation=$(sed -n 's/^case_generation: //p' <<<"$status_output" | head -1)
  bind_output=$(YAI_HOME="$yai_home" "$yai_bin" case policy bind --case "$case_id" \
    --artifact "$artifact_id" --expected-generation "$generation" \
    --as participant:local-policy-operator --reason "bind explicit runtime policy")
  yai_policy_trace 07 "YAI_HOME=$yai_home $yai_bin case policy bind --case $case_id --artifact $artifact_id --expected-generation $generation --as participant:local-policy-operator --reason 'bind explicit runtime policy'" "$bind_output"
  printf '%s\n' "$artifact_id"
}
