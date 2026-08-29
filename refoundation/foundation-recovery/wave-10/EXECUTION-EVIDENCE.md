# Wave 10 execution evidence

Only output actually captured during Wave 10 appears below. Product logs were
captured with true command exits at `/tmp/yai-wave10-product-final.log` (699
lines) and `/tmp/yai-wave10-review-final.log` (183 lines); the
bounded excerpts below retain the material raw lines. `/tmp` logs are execution
residue, not repository authority.

## E10-P01 — policy intake, publication and exact Case binding

- run_id: `e10-product-GcDRRF-allow`
- execution_order: 03, 04, 05, 07
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: isolated `YAI_HOME=/tmp/yai-policy-admission.GcDRRF/allow/home`
- pre-state: real open Case; bound provider/resource/proposer role; zero policy
  bindings, zero Decisions, zero Grants
- exit: each command 0

```text
$ YAI_HOME=/tmp/yai-policy-admission.GcDRRF/allow/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai policy ingest /tmp/yai-policy-admission.GcDRRF/allow/home/authority-allow-1.policy.json --as participant:local-policy-operator
policy_ingest: candidate_created
source_created: true
artifact_created: true
policy_source_schema: yai.policy_source_artifact.v3
source_id: policy-source:b61a6b492351dc2817c91d8f2516ae7e98cc15a0c4fc43697cb2e2c39adc7916
source_digest: sha256:b61a6b492351dc2817c91d8f2516ae7e98cc15a0c4fc43697cb2e2c39adc7916
policy_artifact_schema: yai.policy_artifact.v3
artifact_id: policy-artifact:2715d5c826c4921829bf6a8be4e9ebd2989a16876a8f8bddecff69a437c2db28
policy_lineage_id: policy-lineage:0c05a676c545f7961eaf98ef9235a99013de02e9b3d1ff54a6e6bc73953c97dd
exit: 0

$ YAI_HOME=/tmp/yai-policy-admission.GcDRRF/allow/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai policy validate policy-artifact:2715d5c826c4921829bf6a8be4e9ebd2989a16876a8f8bddecff69a437c2db28 --as participant:local-policy-operator --reason 'deterministic validation'
policy_validate: validated
artifact_id: policy-artifact:2715d5c826c4921829bf6a8be4e9ebd2989a16876a8f8bddecff69a437c2db28
validation_status: qualified
validation_blockers: 0
lifecycle: validated
runtime_consumable: false
lifecycle_events: 2
decision_or_grant: never_emitted_by_policy_authoring
exit: 0

$ YAI_HOME=/tmp/yai-policy-admission.GcDRRF/allow/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai policy publish policy-artifact:2715d5c826c4921829bf6a8be4e9ebd2989a16876a8f8bddecff69a437c2db28 --as participant:local-policy-operator --reason 'publish characterization policy'
policy_publish: published
validation_status: qualified
validation_blockers: 0
lifecycle: published
runtime_consumable: true
lifecycle_events: 3
decision_or_grant: never_emitted_by_policy_authoring
exit: 0

$ YAI_HOME=/tmp/yai-policy-admission.GcDRRF/allow/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai case policy bind --case case:new12-filesystem --artifact policy-artifact:2715d5c826c4921829bf6a8be4e9ebd2989a16876a8f8bddecff69a437c2db28 --expected-generation 9 --as participant:local-policy-operator --reason 'bind explicit runtime policy'
case_policy_bind: committed
transition_id: transition:policy-bind:case-policy-binding:62a9ad6a0c7b318837b647223701fb0a
case_generation: 10
normative_readiness: ready
active_policy_bindings: 1
effective_policy_id: effective-policy:872f6308809d5d0db20f0764bb37556f
effective_policy_digest: sha256:872f6308809d5d0db20f0764bb37556f27f071eeeb6c4a3310c9979e738f48c8
materializer_version: yai.policy_materializer.v2
decision_count: 0
execution_grant_count: 0
prepared_effect_count: 0
authority_emitted_by_case_policy: false
exit: 0
```

Proves immutable source/artifact identity, publication eligibility, exact
binding and derived readiness while authoring/materialization alone emits no
authority.

## E10-P02 — complete policy ALLOW causal chain

- run_id: `e10-product-GcDRRF-allow`
- execution_order: 08
- pre-state: P01 binding Ready; fixture waiting for one governed write
- exact command and exit: shown below, exit 0

```text
$ YAI_HOME=/tmp/yai-policy-admission.GcDRRF/allow/home YAI_JOURNAL=/tmp/yai-policy-admission.GcDRRF/allow/journal.jsonl /home/mothx/computer-science/projects/YAI/yai/target/debug/yai effect filesystem-write --case case:new12-filesystem --subject subject:llm-provider --attachment workspace --prompt 'propose one policy-governed write' --provider-id provider:allow --base-url http://127.0.0.1:34071/v1/chat/completions --model controlled-model
provider_invocation_id: invocation:model-prompt-32
provider_result_id: provider-result:model-output-33
provider_result_authority: non_authoritative_candidate_material
operation_normalization: accepted
operation_id: operation:989230c14a68c8f09a38a673ec40ad45
decision_id: decision:a782520747e85a9afdd1d18f62725d87
decision_reason: policy_admission_satisfied
decision_basis_id: decision-basis:f580fcff1e66d2857de4863db7d2eb73
effective_policy_id: effective-policy:872f6308809d5d0db20f0764bb37556f
decision: allow
execution_grant_id: grant:7fb3b6ec823ba5b418600c02edb2b705
execution_grant_decision_basis_id: decision-basis:f580fcff1e66d2857de4863db7d2eb73
effect_id: effect:7fb3b6ec823ba5b418600c02edb2b705
effect_state: prepared_durable_before_mutation
effect_receipt_id: effect-receipt:233eda85d15bf6df7de7c267e88ccd45
effect_outcome: Applied
effect_state: finalized
effect_chain_closure: valid
second_provider_invocation_id: invocation:model-prompt-36
second_provider_result_id: provider-result:model-output-37
second_turn_consequence: observed_reality_from_canonical_state
exit: 0
```

The same raw command also reported `source_provenance=satisfied` with refs
`provider-result:model-output-33` and `invocation:model-prompt-32`, and
`post_observation=required_at_execution`. The real target contained `hello from
controlled YAI` after FINALIZE (asserted by the unchanged command script).

## E10-P03/P04 — applicable DENY and no applicable ALLOW

- run_ids: `e10-product-GcDRRF-deny`, `e10-product-GcDRRF-no-match`
- execution_order: 09 and 10
- pre-state: separate Ready Cases; mechanically permitted same target
- exit: 0 because DENY is a committed product outcome

```text
$ YAI_HOME=/tmp/yai-policy-admission.GcDRRF/deny/home YAI_JOURNAL=/tmp/yai-policy-admission.GcDRRF/deny/journal.jsonl /home/mothx/computer-science/projects/YAI/yai/target/debug/yai effect filesystem-write --case case:new12-filesystem --subject subject:llm-provider --attachment workspace --prompt 'propose one policy-governed write' --provider-id provider:deny --base-url http://127.0.0.1:43697/v1/chat/completions --model controlled-model
operation_id: operation:989230c14a68c8f09a38a673ec40ad45
decision_id: decision:815f5eafeb22259bb14f193902035b4e
decision_reason: applicable_policy_deny
decision_basis_id: decision-basis:5920568287021d0a45b2cbd4635f553e
effective_policy_id: effective-policy:cb11afa474c896960551a247b824d6cc
decision: deny
execution_grant: none
external_effect: none
second_turn_consequence: committed_denial_no_effect
exit: 0

$ YAI_HOME=/tmp/yai-policy-admission.GcDRRF/no-match/home YAI_JOURNAL=/tmp/yai-policy-admission.GcDRRF/no-match/journal.jsonl /home/mothx/computer-science/projects/YAI/yai/target/debug/yai effect filesystem-write --case case:new12-filesystem --subject subject:llm-provider --attachment workspace --prompt 'propose one policy-governed write' --provider-id provider:no-match --base-url http://127.0.0.1:43879/v1/chat/completions --model controlled-model
decision_id: decision:34fe0c685470eacd4aed8f24a824049d
decision_reason: no_applicable_allow_rule
decision_basis_id: decision-basis:3cedf6cf6d3ca519ea236f3ab73aaf23
effective_policy_id: effective-policy:eb271b58c5cac1512b24099184e669b2
decision: deny
execution_grant: none
external_effect: none
second_turn_consequence: committed_denial_no_effect
exit: 0
```

## E10-P05 — unconfigured runtime stops before provider

- run_id: `e10-product-GcDRRF-unconfigured`
- execution_order: 11
- pre-state: Case/resource/provider configured; zero policy bindings; endpoint
  deliberately unreachable
- exit: 2

```text
$ YAI_HOME=/tmp/yai-policy-admission.GcDRRF/unconfigured/home YAI_JOURNAL=/tmp/yai-policy-admission.GcDRRF/unconfigured/journal.jsonl /home/mothx/computer-science/projects/YAI/yai/target/debug/yai effect filesystem-write --case case:new12-filesystem --subject subject:llm-provider --attachment workspace --prompt 'must not invoke' --base-url http://127.0.0.1:1/v1/chat/completions --model controlled-model
normative_readiness: Unconfigured
provider_invocations: 0
execution_grants: 0
external_effect: none
normative_case_not_ready: Unconfigured
exit: 2
```

## E10-P06 — policy-bound human review and real audit reason

- run_id: `e10-review-QETsdT-approve`
- execution_order: 01–05
- pre-state: Ready review policy; proposer and reviewer roles bound; no pending
  review/effect
- exit: each successful command 0; wrong reviewer exit 2

```text
$ YAI_HOME=/tmp/yai-human-review.QETsdT/approve/home YAI_JOURNAL=/tmp/yai-human-review.QETsdT/approve/journal.jsonl /home/mothx/computer-science/projects/YAI/yai/target/debug/yai case run --case case:new12-filesystem --subject subject:llm-provider --attachment workspace --prompt 'propose one human-reviewed filesystem write, then report completion' --max-invocations 3 --max-operations 2 --max-resident-items 12 --max-semantic-units 6000 --max-estimated-input-units 50000
operation_id: operation:39a37e0572a03c9bc1906aac01d005f4
decision: require_review
review_id: review:9d5eb95a9fa03b89144049127a7e2292
execution_grant: none
external_effect: none
runtime_status: AwaitingReview
last_provider_result_id: provider-result:model-output-33
exit: 0

$ YAI_HOME=/tmp/yai-human-review.QETsdT/approve/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai review show review:9d5eb95a9fa03b89144049127a7e2292 --case case:new12-filesystem
review_schema: yai.review_request.v2
operation_id: operation:39a37e0572a03c9bc1906aac01d005f4
initial_decision_id: decision:645b8931a2f2b6ed10f86ce284653d17
required_reviewer_roles: operation-reviewer
decision_basis_id: decision-basis:139613d569e10c1d93d63a7575e8c04c
effective_policy_id: effective-policy:d7e234d5fac9c9efc8988b690eb21714
status: pending
exit: 0

$ YAI_HOME=/tmp/yai-human-review.QETsdT/approve/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai review approve review:9d5eb95a9fa03b89144049127a7e2292 --case case:new12-filesystem --as subject:llm-provider --reason 'self approve'
reviewer_not_eligible_for_case_review
exit: 2

$ YAI_HOME=/tmp/yai-human-review.QETsdT/approve/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai review approve review:9d5eb95a9fa03b89144049127a7e2292 --case case:new12-filesystem --as subject:policy-pack --reason 'human participant approve exact operation'
review_action: committed
action_id: review-action:sha256:1349651815e888ce3f736acd2
effective_decision_id: decision:5d553368aa649f0fba9269d227b1ee4e
execution_grant: none_review_command_never_executes
external_effect: none
exit: 0

$ YAI_HOME=/tmp/yai-human-review.QETsdT/approve/home YAI_JOURNAL=/tmp/yai-human-review.QETsdT/approve/journal.jsonl /home/mothx/computer-science/projects/YAI/yai/target/debug/yai case resume --case case:new12-filesystem
operation_id: operation:39a37e0572a03c9bc1906aac01d005f4
decision_id: decision:5d553368aa649f0fba9269d227b1ee4e
decision_reason: eligible_human_review_approved
decision_basis_id: decision-basis:3f11e3e13991d0642b5c30dc6da8cac3
effective_policy_id: effective-policy:d7e234d5fac9c9efc8988b690eb21714
evidence_obligations: [{"obligation":"audit_reason","status":"satisfied","evidence_refs":["review-action:sha256:1349651815e888ce3f736acd2"],"contributing_rules":[{"binding_id":"case-policy-binding:2163a1c04a281f88adaba0a121844351","artifact_id":"policy-artifact:a0669ec559ef75f9f3f2c5a900fe3471465fce3db59d6804be32885c3269d498","policy_ir_rule_id":"filesystem-audit-reason","source_id":"policy-source:84b6744aab8b10d1198e164575f6932aa133f463a2dd3a3f69ccdb9a45759030","fact_refs":["policy-fact:5454ff584cef6c15da12ff168b74f023e3bd3296c083937b9ed8013e974fabbc"],"source_locations":["$.rules[6]"]}]},{"obligation":"post_observation","status":"required_at_execution","evidence_refs":[],"contributing_rules":[{"binding_id":"case-policy-binding:2163a1c04a281f88adaba0a121844351","artifact_id":"policy-artifact:a0669ec559ef75f9f3f2c5a900fe3471465fce3db59d6804be32885c3269d498","policy_ir_rule_id":"filesystem-post","source_id":"policy-source:84b6744aab8b10d1198e164575f6932aa133f463a2dd3a3f69ccdb9a45759030","fact_refs":["policy-fact:acb10b3a4c6d83b05d9bd22c3e44b4ed52a2ad562c1585f12ad74009f987951f"],"source_locations":["$.rules[3]"]}]},{"obligation":"source_provenance","status":"satisfied","evidence_refs":["provider-result:model-output-33","invocation:model-prompt-32"],"contributing_rules":[{"binding_id":"case-policy-binding:2163a1c04a281f88adaba0a121844351","artifact_id":"policy-artifact:a0669ec559ef75f9f3f2c5a900fe3471465fce3db59d6804be32885c3269d498","policy_ir_rule_id":"filesystem-source","source_id":"policy-source:84b6744aab8b10d1198e164575f6932aa133f463a2dd3a3f69ccdb9a45759030","fact_refs":["policy-fact:ea7568ed57475c11ced28bd68f8211dad971d781278da22b30093bae932beca2"],"source_locations":["$.rules[2]"]}]}]
decision: allow
execution_grant_id: grant:354bade488637433f65bdc653b5811b8
effect_id: effect:354bade488637433f65bdc653b5811b8
effect_receipt_id: effect-receipt:996d0942077e933bc582ff53403099aa
effect_outcome: Applied
runtime_status: Completed
operations: 1
exit: 0
```

The Operation ID is identical before/after the no-runner human action. Approval
is authority evidence, produces no effect itself, and its actual action ref
satisfies `audit_reason`.

## E10-P07 — review basis becomes stale after explicit replacement

- run_id: `e10-review-QETsdT-stale`
- execution_order: 06–07
- pre-state: pending ReviewRequest under E1; no Grant/effect
- replace exit: 0; stale approval exit: 2

```text
$ YAI_HOME=/tmp/yai-human-review.QETsdT/stale/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai case policy replace --case case:new12-filesystem --binding case-policy-binding:cf8136ebc13d30defb566ed0c8eb0a5d --artifact policy-artifact:5c583238ebc18ee76656a643b6168d999fb9f90e49548318d26589d29deed2f1 --expected-generation 18 --as participant:local-policy-operator --reason 'replace while review pending'
case_policy_replace: committed
case_generation: 19
normative_readiness: ready
effective_policy_id: effective-policy:1074a987348c782b80a0b8102177c6ed
decision_count: 1
execution_grant_count: 0
prepared_effect_count: 0
exit: 0

$ YAI_HOME=/tmp/yai-human-review.QETsdT/stale/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai review approve review:27237a63cc3d27676fca7da5a9cc3f30 --case case:new12-filesystem --as subject:policy-pack --reason 'human participant approve exact operation'
review_policy_basis_stale
exit: 2
```

## Qualification evidence

Focused engine and product commands already executed before full closure:

```text
$ cargo test --manifest-path engine/yai-engine/Cargo.toml admission::tests -- --nocapture
running 5 tests
test admission::tests::audit_reason_requires_real_review_action_evidence ... ok
test admission::tests::decision_basis_and_grant_integrity_detect_tampering ... ok
test admission::tests::explicit_allow_is_policy_bound_and_resource_legacy_fields_are_inert ... ok
test admission::tests::review_eligibility_and_source_provenance_are_mechanical ... ok
test admission::tests::deny_no_match_resource_violation_and_missing_role_fail_closed ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
exit: 0

$ cargo test --manifest-path engine/yai-engine/Cargo.toml wave10_policy_mutation_between_decision_and_grant_fails_in_same_transaction -- --nocapture
running 1 test
wave10_stale_basis: decision_basis=decision-basis:690361d6b699d02318e6e487f7c3f5cc evaluated_effective_policy=effective-policy:3d7d4a02b3a04f3b638f46e16196be7f current_effective_policy=effective-policy:71a456bef393badb2ee68d57c55dd134 grant_committed=false error=policy_authority_basis_stale
test store::lmdb::tests::wave10_policy_mutation_between_decision_and_grant_fails_in_same_transaction ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 93 filtered out
exit: 0

$ tests/characterization/policy-authority-admission/test_policy_authority_admission.sh
policy_authority:allow_chain ok
policy_authority:explicit_deny_and_no_match_fail_closed ok
policy_authority:unconfigured_pre_provider_stop ok
exit: 0

$ tests/characterization/human-review-runtime/test_human_review_runtime.sh
human_review:approve_deny_defer_and_query_purity ok
human_review:provider_model_replacement_and_no_second_operation ok
human_review:runtime_admission_concurrency_and_stale_reclaim ok
human_review:crash_r1_r6_recovery ok
human_review:provider_cannot_invent_human_authority ok
human_review:policy_basis_change_fails_closed ok
exit: 0
```

## E10-Q01 — full repository check

- run_id: `e10-qualification-20260829-check`
- execution_order: Q01
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: normal repository environment; Unix-socket tests authorized
- pre-state: final Wave-10 source after product evidence capture
- exit: 0

```text
$ make check
check-no-old-roots: ok
check-required-layout: ok
check-source-placement: ok
check-source-surface-clean: ok
doc_root_canon: ok
check-doc-canonical-location: ok
check-doc-required-files: ok
check-doc-links: ok (28 files)
check-repository-identity: ok
running 95 tests
test admission::tests::decision_basis_and_grant_integrity_detect_tampering ... ok
test admission::tests::explicit_allow_is_policy_bound_and_resource_legacy_fields_are_inert ... ok
test admission::tests::review_eligibility_and_source_provenance_are_mechanical ... ok
test admission::tests::deny_no_match_resource_violation_and_missing_role_fail_closed ... ok
test admission::tests::audit_reason_requires_real_review_action_evidence ... ok
test store::lmdb::tests::wave10_policy_mutation_between_decision_and_grant_fails_in_same_transaction ... ok
test result: ok. 95 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 33.44s
case_runtime:agentless_26_turn_provider_model_replacement ok
case_runtime:deny_adaptation_and_bounded_residency ok
case_runtime:grant_effect_and_memory_restart_recovery ok
case_runtime:malformed_retry_operator_stop ok
case_runtime:budget_stops_before_extra_invocation ok
human_review:approve_deny_defer_and_query_purity ok
human_review:provider_model_replacement_and_no_second_operation ok
human_review:runtime_admission_concurrency_and_stale_reclaim ok
human_review:crash_r1_r6_recovery ok
human_review:provider_cannot_invent_human_authority ok
human_review:policy_basis_change_fails_closed ok
governance_intake_characterization: pass
governance_hardening_characterization: pass
case_policy_materialization_characterization: pass
policy_authority:allow_chain ok
policy_authority:explicit_deny_and_no_match_fail_closed ok
policy_authority:unconfigured_pre_provider_stop ok
exit: 0
```

## E10-Q02 — full characterization

- run_id: `e10-qualification-20260829-characterization`
- execution_order: Q02
- cwd/environment/pre-state: same final repository state as Q01
- exit: 0

```text
$ make characterization
running 95 tests
test result: ok. 95 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 33.35s
provider_model_vertical:real_http_invocation ok
provider_model_vertical:durable_continuity_residue ok
provider_model_vertical:canonical_transition_authority ok
provider_model_vertical:typed_projection_context_frame ok
semantic_continuity:provider_replacement ok
semantic_continuity:model_replacement ok
semantic_continuity:continuation_loss_and_restart ok
semantic_continuity:memory_inspect_drop_rebuild ok
case_runtime:agentless_26_turn_provider_model_replacement ok
human_review:crash_r1_r6_recovery ok
human_review:policy_basis_change_fails_closed ok
policy_authority:allow_chain ok
policy_authority:explicit_deny_and_no_match_fail_closed ok
policy_authority:unconfigured_pre_provider_stop ok
governance_intake_characterization: pass
governance_hardening_characterization: pass
exit: 0
```

## E10-Q03 — required individual smoke targets

- run_id: `e10-qualification-20260829-smokes`
- execution_order: Q03.1–Q03.7
- cwd/environment/pre-state: same final repository state; isolated temporary
  stores created by each characterization
- exit: every target 0

```text
$ make smoke-governance-intake
governance_intake_characterization: pass
policy_v1: policy-artifact:43da66a41a8b760da557d4267df6d13b26dee99fba1fa0db20547fc73dee7b7b
policy_v2: policy-artifact:7f2cb1391c39097bb316257c92dd035f1c999bf6fe6c819794364392c86442e5
canonical_case_transitions: 0
exit: 0

$ make smoke-governance-hardening
governance_hardening_characterization: pass
published_lineages: 2
version_collision: rejected
duplicate_keys: rejected
case_transitions: 0
exit: 0

$ make smoke-case-policy-materialization
case_policy_materialization_characterization: pass
active_bindings: 2
canonical_transitions: 9
effective_policy_id: effective-policy:d196523084e8c919a6a8f311d44bf062
authority_emitted: false
exit: 0

$ make smoke-human-review-runtime
human_review:approve_deny_defer_and_query_purity ok
human_review:provider_model_replacement_and_no_second_operation ok
human_review:runtime_admission_concurrency_and_stale_reclaim ok
human_review:crash_r1_r6_recovery ok
human_review:provider_cannot_invent_human_authority ok
human_review:policy_basis_change_fails_closed ok
exit: 0

$ make smoke-semantic-continuity
semantic_continuity:provider_replacement ok
semantic_continuity:model_replacement ok
semantic_continuity:continuation_loss_and_restart ok
semantic_continuity:memory_inspect_drop_rebuild ok
exit: 0

$ make smoke-agentless-case-runtime
case_runtime:agentless_26_turn_provider_model_replacement ok
case_runtime:deny_adaptation_and_bounded_residency ok
case_runtime:grant_effect_and_memory_restart_recovery ok
case_runtime:malformed_retry_operator_stop ok
case_runtime:budget_stops_before_extra_invocation ok
exit: 0

$ make smoke-policy-authority-admission
policy_authority:allow_chain ok
policy_authority:explicit_deny_and_no_match_fail_closed ok
policy_authority:unconfigured_pre_provider_stop ok
exit: 0
```

## E10-Q04 — endurance and replay/rebuild

- run_id: `e10-qualification-20260829-focused`
- execution_order: Q04.1–Q04.4
- cwd/environment/pre-state: final repository, Rust test temporary stores
- exit: every command 0

```text
$ cargo test --manifest-path engine/yai-engine/Cargo.toml hundred_iteration_case_state_memory_context_endurance -- --nocapture
running 1 test
test residency::tests::hundred_iteration_case_state_memory_context_endurance ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 94 filtered out; finished in 0.03s
exit: 0

$ cargo test --manifest-path engine/yai-engine/Cargo.toml hundred_iteration_planning_remains_bounded -- --nocapture
running 1 test
test residency::tests::hundred_iteration_planning_remains_bounded ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 94 filtered out; finished in 0.20s
exit: 0

$ cargo test --manifest-path engine/yai-engine/Cargo.toml wave9_exact_version_binding_replacement_replay_and_rebuild_are_deterministic -- --nocapture
running 1 test
test store::lmdb::tests::wave9_exact_version_binding_replacement_replay_and_rebuild_are_deterministic ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 94 filtered out; finished in 0.06s
exit: 0

$ cargo test --manifest-path engine/yai-engine/Cargo.toml typed_human_review_replays_without_promoting_approval_to_effect_truth -- --nocapture
running 1 test
test store::lmdb::tests::typed_human_review_replays_without_promoting_approval_to_effect_truth ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 94 filtered out; finished in 0.00s
exit: 0
```

The two endurance tests execute 128 iterations in the checked source. A
product command for deliberate immutable-catalog corruption is not justified;
the closest supported blocked-readiness qualification was run directly:

```text
$ cargo test --manifest-path engine/yai-engine/Cargo.toml wave9_missing_artifact_blocks_readiness_without_erasing_binding_history -- --nocapture
running 1 test
test store::lmdb::tests::wave9_missing_artifact_blocks_readiness_without_erasing_binding_history ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 94 filtered out; finished in 0.02s
exit: 0
```

## E10-Q05 — format, documentation, diff and clippy verdict

- run_id: `e10-qualification-20260829-hygiene`
- execution_order: Q05.1–Q05.6
- cwd/environment/pre-state: final formatted working tree

```text
$ cargo fmt --manifest-path engine/Cargo.toml --all -- --check
exit: 0

$ cargo fmt --manifest-path cmd/yai/Cargo.toml --all -- --check
exit: 0

$ make check-docs
doc_root_canon: ok
check-doc-canonical-location: ok
check-doc-required-files: ok
check-doc-links: ok (28 files)
check-repository-identity: ok
exit: 0

$ git diff --check
exit: 0
```

Repository-wide strict clippy remains an honestly non-green historical gate:

```text
$ cargo clippy --manifest-path engine/Cargo.toml --workspace --all-targets -- -D warnings
error: this function has too many arguments (8/7)
    --> yai-engine/src/effect.rs:1388:1
error: this function has too many arguments (8/7)
   --> yai-engine/src/journal.rs:231:1
error: method `from_str` can be confused for the standard trait method `std::str::FromStr::from_str`
   --> yai-engine/src/record.rs:134:5
error: could not compile `yai-engine` (lib) due to 14 previous errors
exit: 101

$ cargo clippy --manifest-path cmd/yai/Cargo.toml --all-targets -- -D warnings
error: this expression creates a reference which is immediately dereferenced by the compiler
   --> src/main.rs:917:41
error: this expression creates a reference which is immediately dereferenced by the compiler
   --> src/provider.rs:284:63
error: this let-binding has unit value
    --> src/main.rs:1423:5
error: could not compile `yai` (bin "yai" test) due to 17 previous errors
exit: 101
```

All reported warning sites existed at baseline. The actual zero-context diff
check for every reported source pattern returned no matches (`rg` exit 1), so
Wave 10 introduced no new clippy warning site. No warning was suppressed and no
unrelated historical cleanup was folded into the Wave.
