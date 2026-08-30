# Wave 11 execution evidence

Only output captured during actual commands is included. Product blocks P01-P07
share one workspace and causal run. Lines are a bounded raw selection emitted
by `YAI_EVIDENCE_COMPACT=1`; they are not reconstructed summaries.

## E11-P01 — valid exact binding

- run_id: `wave11-product-oLny9e`
- execution_order: 01
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: `YAI_HOME=/tmp/yai-temporal-governance.oLny9e/home`
- pre-state: real Case open at generation 9; P1 published and exactly bound
- authority time: observed/effective `1788098905000`; persisted floor `1788098905000`
- command: `YAI_HOME=/tmp/yai-temporal-governance.oLny9e/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai case policy status --case case:new12-filesystem`
- exit: `0`
- produced IDs: `case:new12-filesystem`, `case-policy-binding:1a09f5e17750f2fa40643087493095f6`, `effective-policy:1256a6c1725a41a1a1dd5d7ca60781d4`

Raw output:

```text
case_id: case:new12-filesystem
case_generation: 9
case_lifecycle: Open
case_cancelled: false
normative_readiness: ready
policy_validity: Valid
observed_wall_time: 1788098905000
persisted_authority_floor: 1788098905000
effective_authority_time: 1788098905000
active_policy_bindings: 1
policy_binding: binding_id=case-policy-binding:1a09f5e17750f2fa40643087493095f6 lineage_id=policy-lineage:8cc78adf7b4a8f757ba2cd3582ed7e0f716c81d7b557c7608421d6fe51a981d1 owner_ref=organization:characterization policy_key=temporal-filesystem artifact_id=policy-artifact:2f2a2446e76a6380473c9d0aec0643ab1408d124e9f63a6056e9b151ed478076 version=1 publication_event=policy-event:e1548480e9fe2c578d8f69e1376b751b2d29f3aa0a6068007529f72cd89ce73b bound_generation=9
effective_policy_id: effective-policy:1256a6c1725a41a1a1dd5d7ca60781d4
catalog_drift: lineage_id=policy-lineage:8cc78adf7b4a8f757ba2cd3582ed7e0f716c81d7b557c7608421d6fe51a981d1 status=current
binding_validity: lineage_id=policy-lineage:8cc78adf7b4a8f757ba2cd3582ed7e0f716c81d7b557c7608421d6fe51a981d1 binding_id=case-policy-binding:1a09f5e17750f2fa40643087493095f6 artifact_id=policy-artifact:2f2a2446e76a6380473c9d0aec0643ab1408d124e9f63a6056e9b151ed478076 posture=Valid reason=unbounded valid_from=none refresh_after=none expires_at=none revoke_event=none
```

Proves: readiness and temporal validity are separate, exact P1 identity is
current, and authority time/floor are inspectable without a write.

## E11-P02 — publication makes P1 stale, not upgraded

- run_id: `wave11-product-oLny9e`
- execution_order: 02-03
- cwd/environment: same as E11-P01
- pre-state: P1 remains exactly bound
- commands:
  - `YAI_HOME=/tmp/yai-temporal-governance.oLny9e/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai policy publish policy-artifact:9729f6c741b9b2dd4a66a36fd063ff6e54688d37d2e0693cf4153b1247b5d0b3 --as participant:policy-admin --reason 'publish refresh'`
  - `YAI_HOME=/tmp/yai-temporal-governance.oLny9e/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai case policy status --case case:new12-filesystem`
- exits: `0`, `0`
- produced IDs: `policy-artifact:9729f6c741b9b2dd4a66a36fd063ff6e54688d37d2e0693cf4153b1247b5d0b3`, `policy-event:8d3c47237406ab0e106839fd153f11a4d3cc525981e6dc25bcfe5426bce39489`

Raw output:

```text
policy_publish: published
policy_artifact_schema: yai.policy_artifact.v4
artifact_id: policy-artifact:9729f6c741b9b2dd4a66a36fd063ff6e54688d37d2e0693cf4153b1247b5d0b3
artifact_version: 2
policy_lineage_id: policy-lineage:8cc78adf7b4a8f757ba2cd3582ed7e0f716c81d7b557c7608421d6fe51a981d1
lifecycle: published
runtime_consumable: true
lifecycle_event: sequence=7 event_id=policy-event:8d3c47237406ab0e106839fd153f11a4d3cc525981e6dc25bcfe5426bce39489 action=Published actor=participant:policy-admin related_artifact=none
case_generation: 9
policy_validity: Stale
policy_binding: binding_id=case-policy-binding:1a09f5e17750f2fa40643087493095f6 lineage_id=policy-lineage:8cc78adf7b4a8f757ba2cd3582ed7e0f716c81d7b557c7608421d6fe51a981d1 owner_ref=organization:characterization policy_key=temporal-filesystem artifact_id=policy-artifact:2f2a2446e76a6380473c9d0aec0643ab1408d124e9f63a6056e9b151ed478076 version=1 publication_event=policy-event:e1548480e9fe2c578d8f69e1376b751b2d29f3aa0a6068007529f72cd89ce73b bound_generation=9
catalog_drift: lineage_id=policy-lineage:8cc78adf7b4a8f757ba2cd3582ed7e0f716c81d7b557c7608421d6fe51a981d1 status=superseded:current=policy-artifact:9729f6c741b9b2dd4a66a36fd063ff6e54688d37d2e0693cf4153b1247b5d0b3
binding_validity: lineage_id=policy-lineage:8cc78adf7b4a8f757ba2cd3582ed7e0f716c81d7b557c7608421d6fe51a981d1 binding_id=case-policy-binding:1a09f5e17750f2fa40643087493095f6 artifact_id=policy-artifact:2f2a2446e76a6380473c9d0aec0643ab1408d124e9f63a6056e9b151ed478076 posture=Stale reason=bound_artifact_superseded valid_from=none refresh_after=none expires_at=none revoke_event=none
```

Proves: P2 publication contracts P1 authority but does not mutate the Case
binding or generation.

## E11-P03 — explicit refresh

- run_id: `wave11-product-oLny9e`
- execution_order: 04
- pre-state: P1 stale at Case generation 9
- command: `YAI_HOME=/tmp/yai-temporal-governance.oLny9e/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai case policy replace --case case:new12-filesystem --binding case-policy-binding:1a09f5e17750f2fa40643087493095f6 --artifact policy-artifact:9729f6c741b9b2dd4a66a36fd063ff6e54688d37d2e0693cf4153b1247b5d0b3 --expected-generation 9 --as participant:operator --reason 'explicit temporal refresh'`
- exit: `0`
- produced IDs: `transition:policy-replace:case-policy-binding:67c0833084c2e5f6b0bff34b63e57d1f`, `case-policy-binding:67c0833084c2e5f6b0bff34b63e57d1f`, `effective-policy:577668d71b296bc5c5cbaa72db54375e`

Raw output:

```text
case_policy_replace: committed
transition_id: transition:policy-replace:case-policy-binding:67c0833084c2e5f6b0bff34b63e57d1f
case_generation: 10
normative_readiness: ready
policy_validity: Valid
policy_binding: binding_id=case-policy-binding:67c0833084c2e5f6b0bff34b63e57d1f lineage_id=policy-lineage:8cc78adf7b4a8f757ba2cd3582ed7e0f716c81d7b557c7608421d6fe51a981d1 owner_ref=organization:characterization policy_key=temporal-filesystem artifact_id=policy-artifact:9729f6c741b9b2dd4a66a36fd063ff6e54688d37d2e0693cf4153b1247b5d0b3 version=2 publication_event=policy-event:8d3c47237406ab0e106839fd153f11a4d3cc525981e6dc25bcfe5426bce39489 bound_generation=10
effective_policy_id: effective-policy:577668d71b296bc5c5cbaa72db54375e
binding_validity: lineage_id=policy-lineage:8cc78adf7b4a8f757ba2cd3582ed7e0f716c81d7b557c7608421d6fe51a981d1 binding_id=case-policy-binding:67c0833084c2e5f6b0bff34b63e57d1f artifact_id=policy-artifact:9729f6c741b9b2dd4a66a36fd063ff6e54688d37d2e0693cf4153b1247b5d0b3 posture=Valid reason=unbounded valid_from=none refresh_after=none expires_at=none revoke_event=none
```

Proves: refresh is an explicit exact-version Case transition.

## E11-P04 — real authority chain reaches PREPARE

- run_id: `wave11-product-oLny9e`
- execution_order: 05
- environment: same Case plus local provider fixture on the printed command's localhost port
- pre-state: P2 Ready+Valid at generation 10
- command: `YAI_HOME=/tmp/yai-temporal-governance.oLny9e/home YAI_JOURNAL=/tmp/yai-temporal-governance.oLny9e/journal.jsonl /home/mothx/computer-science/projects/YAI/yai/target/debug/yai effect filesystem-write --case case:new12-filesystem --subject subject:llm-provider --attachment workspace --prompt 'prepare one temporal write' --provider-id provider:temporal --base-url http://127.0.0.1:43675/v1/chat/completions --model controlled-model --failpoint after_effect_before_finalize`
- actual exit: `86`
- produced IDs: `invocation:model-prompt-32`, `provider-result:model-output-33`, `operation:f3615dc53c7d61c9be5ae929ea9d6d60`, `decision-basis:bb56d8442965295c9ae6e71098116972`, `decision:f1388b38f28a820b404acc11c68f76ae`, `grant:4dd8498313530b22926d558dcb2016fc`, `effect:4dd8498313530b22926d558dcb2016fc`

Raw output:

```text
provider_invocation_id: invocation:model-prompt-32
provider_result_id: provider-result:model-output-33
operation_id: operation:f3615dc53c7d61c9be5ae929ea9d6d60
decision_id: decision:f1388b38f28a820b404acc11c68f76ae
decision_basis_id: decision-basis:bb56d8442965295c9ae6e71098116972
effective_policy_id: effective-policy:577668d71b296bc5c5cbaa72db54375e
decision: allow
execution_grant_id: grant:4dd8498313530b22926d558dcb2016fc
effect_id: effect:4dd8498313530b22926d558dcb2016fc
effect_state: prepared_durable_before_mutation
controlled_effect_crash_injected: after_effect_before_finalize
```

Proves: actual v3 authority progressed through canonical Operation,
DecisionBasis, Decision, Grant and durable PREPARE before the injected crash.

## E11-P05 — revoke and cancellation after PREPARE

- run_id: `wave11-product-oLny9e`
- execution_order: 06-07
- pre-state: effect `effect:4dd8498313530b22926d558dcb2016fc` is already Prepared
- commands:
  - `YAI_HOME=/tmp/yai-temporal-governance.oLny9e/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai policy revoke policy-artifact:9729f6c741b9b2dd4a66a36fd063ff6e54688d37d2e0693cf4153b1247b5d0b3 --as participant:policy-admin --reason 'withdraw before future authority'`
  - `YAI_HOME=/tmp/yai-temporal-governance.oLny9e/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai case cancel --case case:new12-filesystem --as participant:operator --reason 'stop after prepared external attempt'`
- exits: `0`, `0`
- produced IDs: `policy-event:d91fdabe99872e5d74e140a1fb1608edbc0c34831dafd8be1cb09d6f081a9aa8`, `transition:case-cancel:case:new12-filesystem`

Raw output:

```text
policy_revoke: revoked
artifact_id: policy-artifact:9729f6c741b9b2dd4a66a36fd063ff6e54688d37d2e0693cf4153b1247b5d0b3
lifecycle: revoked
runtime_consumable: false
lifecycle_event: sequence=8 event_id=policy-event:d91fdabe99872e5d74e140a1fb1608edbc0c34831dafd8be1cb09d6f081a9aa8 action=Revoked actor=participant:policy-admin related_artifact=none
case_cancel: cancelled
invalidated_reviews: 0
abandoned_grants: 0
cancellation_commit: transition_id=transition:case-cancel:case:new12-filesystem generation=20 kind=case_cancellation_requested
case_generation: 20
case_cancelled: true
cancellation_requested_at: 1788098905000
unresolved_effects: 1
```

Proves: revoke is canonical and non-consumable; cancellation does not abandon
the already Prepared Grant or erase the unresolved effect.

## E11-P06 — unsafe close rejected, reconciliation preserved

- run_id: `wave11-product-oLny9e`
- execution_order: 08-09
- pre-state: cancelled Case, one Prepared unresolved effect
- commands:
  - `YAI_HOME=/tmp/yai-temporal-governance.oLny9e/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai case close --case case:new12-filesystem --as participant:operator --reason 'unsafe close'`
  - `YAI_HOME=/tmp/yai-temporal-governance.oLny9e/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai effect reconcile --case case:new12-filesystem --retry`
- exits: `2`, `0`
- produced IDs: no closure ID on the rejected command; reconciliation finalized `effect:4dd8498313530b22926d558dcb2016fc`

Raw output:

```text
case_close_blocked: unresolved_effect:effect:4dd8498313530b22926d558dcb2016fc
reconciliation: EffectObserved
effect_id: effect:4dd8498313530b22926d558dcb2016fc
effect_state: Some(Finalized)
```

Proves: close cannot hide physical ambiguity; reconciliation remains available
after revoke+cancellation.

## E11-P07 — safe terminal closure and no provider bypass

- run_id: `wave11-product-oLny9e`
- execution_order: 10-11
- pre-state: cancelled Case; effect reconciled/finalized
- commands:
  - `YAI_HOME=/tmp/yai-temporal-governance.oLny9e/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai case close --case case:new12-filesystem --as participant:operator --reason 'safe after reconciliation'`
  - `YAI_HOME=/tmp/yai-temporal-governance.oLny9e/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai effect filesystem-write --case case:new12-filesystem --subject subject:llm-provider --attachment workspace --prompt 'must not invoke' --base-url http://127.0.0.1:1/v1/chat/completions --model controlled-model`
- exits: `0`, `2`
- produced IDs: `transition:case-close:case:new12-filesystem`; the rejected effect produced no Operation, Decision or Grant

Raw output:

```text
case_close: closed
closure_transition: transition:case-close:case:new12-filesystem
closure_generation: 22
case_generation: 22
case_lifecycle: Closed
case_cancelled: true
closed_at: 1788098905000
usable_pending_reviews: 0
usable_issued_grants: 0
unresolved_effects: 0
case_lifecycle: Closed
case_cancelled: true
provider_invocations: 0
execution_grants: 0
external_effect: none
case_closed_new_effect_forbidden
```

Proves: close is durable terminal state and the direct effect surface stops
before an intentionally unreachable provider endpoint.

## E11-Q01 — focused temporal qualification

- run_id: `wave11-qualification-20260830-01`
- execution_order: 01
- cwd: `/home/mothx/computer-science/projects/YAI/yai/engine`
- environment: default test environment; deterministic in-process authority times
- pre-state: isolated temporary LMDB stores
- command: `cargo test -p yai-engine wave11_ -- --nocapture`
- exit: `0`

Raw output:

```text
running 6 tests
wave11_case_terminal: cancellation_transition=transition:case-cancel:case:wave11-terminal cancellation_generation=2 close_transition=transition:case-close:case:wave11-terminal close_generation=3 cancel_idempotent=true close_idempotent=true post_cancel=case_cancelled_write_barrier post_close=case_closed_write_barrier replay_closed=true
wave11_grant_expiry: grant=grant:873904470182c0534d3cd94d6d453d81 issued_at=1788097724000 expires_at=1788097754000 authority_floor=1788097754000 invalidation_transition=transition:grant-invalidated:grant:873904470182c0534d3cd94d6d453d81:12 prepare=false effects=0
wave11_prepare_cut: grant=grant:960709930fd56588a4dd54642fce1c70 prepare_generation=12 revoke_after_prepare=true cancellation_generation=13 prepared_close=case_close_blocked: unresolved_effect:effect:960709930fd56588a4dd54642fce1c70 indeterminate_generation=14 indeterminate_close=case_close_blocked: unresolved_effect:effect:960709930fd56588a4dd54642fce1c70 reconcile_generation=15 close_generation=16 receipt=effect-receipt:39d096caa199bb2e5c7cc347f6dd8811 effect_truth_preserved=true
wave11_review_invalidation: review=review:861bfe357bedd34ed1344e971b9ee2ac artifact=policy-artifact:db7507dc5af0ff13d1504f9731af25b3a6a5f34ee238eae8650f632ae7ad8c53 invalidation_transition=transition:review-invalidated:review:861bfe357bedd34ed1344e971b9ee2ac:12 approval_error=review_action_binding_or_generation_mismatch grants=0 replay=true
wave11_temporal_governance: p1=policy-artifact:e44cb4d511175380ce844acadeae7d66dc7ca4292a75c0268d5773fa864998fe p2=policy-artifact:29e4b1ec919abf180aa52022f467d63c08ad57c736e5c40e8442dec550e74fd4 future=policy-artifact:c608f06bd9f95f8cc5171905c28ba2399f6d678e69a9666d1bbc8ee66931fb6c valid=true not_yet_valid=true refresh_required=true expired=true rollback_floor=1788097774000 stale_pinned_to_p1=true retired_stale=true weakest_composition=true explicit_refresh=true revoke_event=policy-event:a819db3977579a5cd711274086ec2eecad532be00d56f9a51339973cc1889462 revoked=true
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 97 filtered out
```

Proves: deterministic time advancement/high-water rollback, finite Grant
expiry, review invalidation, PREPARE cut, closure blockers and replay.

## E11-Q02 — complete Rust regression

- run_id: `wave11-qualification-20260830-02`
- execution_order: 02
- cwd: `/home/mothx/computer-science/projects/YAI/yai/engine`
- command: `cargo test --workspace -- --nocapture`
- exit: `0`

Raw output:

```text
running 103 tests
h10_authority_injection: forged_decision=authority_decision_basis_mismatch forged_grant=policy_execution_grant_semantic_mismatch role_stale_grant=policy_grant_decision_not_adjacent policy_stale_grant=policy_grant_decision_not_adjacent ... grant_committed=false
h10_historical_replay: operation_generation=9 basis_generation=9 decision_transition=10 grant_expected_generation=10 grant_transition=11 prepare_transition=12 finalize_transition=13 replacement_transition=14 ... replay=true crash_c1_c2_c5=true
test result: ok. 103 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 33.75s
Doc-tests yai_core_engine
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Proves: Wave 2-H10 compatibility, forged-authority negatives, historical P1→P2
replay, endurance and catalog capacity remain green.

## E11-Q03 — full repository gate after fixes

- run_id: `wave11-qualification-20260830-03`
- execution_order: 03
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: local Unix sockets and localhost fixtures enabled
- pre-state: atomic Decision write and typed-review test fixes applied
- command: `make check`
- exit: `0`
- produced IDs: suite-owned temporary IDs only

Raw bounded output:

```text
check-no-old-roots: ok
check-required-layout: ok
check-source-placement: ok
check-source-surface-clean: ok
check-doc-links: ok (28 files)
running 103 tests
test result: ok. 103 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
case_runtime:agentless_26_turn_provider_model_replacement ok
case_runtime:grant_effect_and_memory_restart_recovery ok
human_review:crash_r1_r6_recovery ok
human_review:policy_basis_change_fails_closed ok
governance_intake_characterization: pass
governance_hardening_characterization: pass
case_policy_materialization_characterization: pass
policy_authority:allow_chain ok
policy_authority:explicit_deny_and_no_match_fail_closed ok
policy_authority:unconfigured_pre_provider_stop ok
```

Proves: repository layout/docs, all Rust tests and all historical smoke gates
pass together after the temporal changes.

## E11-Q04 — complete characterization

- run_id: `wave11-qualification-20260830-04`
- execution_order: 04
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: local Unix sockets and localhost fixtures enabled
- pre-state: full repository gate green
- command: `make characterization`
- exit: `0`
- produced IDs: characterization workspaces are isolated and discarded

Raw bounded output:

```text
test result: ok. 103 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
provider_model_vertical:real_http_invocation ok
controlled_effect:prepare_crash_reconciliation ok
case_runtime:agentless_26_turn_provider_model_replacement ok
human_review:approve_deny_defer_and_query_purity ok
human_review:crash_r1_r6_recovery ok
policy_authority:allow_chain ok
governance_intake_characterization: pass
governance_hardening_characterization: pass
```

Proves: provider, effect, replay, review, authority and governance
characterizations coexist under the Wave11 schemas.

## E11-Q05 — H10 hardening preserved

- run_id: `wave11-qualification-20260830-05`
- execution_order: 05
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: deterministic temporary LMDB stores
- pre-state: Wave11 authority write path active
- command: `make smoke-policy-authority-hardening`
- exit: `0`
- produced IDs: `decision-basis:43d71f01e147eaa71aec7cf08a8e4575`, `decision:62bf3fc50109ad47ed764fe5acfe0e3c`, `grant:cefeafe783dd9b5da83f395717b5ce8b`

Raw output:

```text
h10_historical_replay: operation_generation=9 basis_generation=9 decision_transition=10 grant_expected_generation=10 grant_transition=11 prepare_transition=12 finalize_transition=13 replacement_transition=14 p1_basis=decision-basis:43d71f01e147eaa71aec7cf08a8e4575 p1_decision=decision:62bf3fc50109ad47ed764fe5acfe0e3c p1_grant=grant:cefeafe783dd9b5da83f395717b5ce8b ... replay=true crash_c1_c2_c5=true
h10_review_rederivation: caller_evidence=authority_decision_basis_mismatch forged_request=authority_review_request_mismatch wrong_reviewer=review_action_binding_or_generation_mismatch forged_final=authority_decision_basis_mismatch canonical_final=true crash_c3_c4=true
h10_authority_injection: forged_decision=authority_decision_basis_mismatch forged_grant=policy_execution_grant_semantic_mismatch role_stale_grant=policy_grant_decision_not_adjacent policy_stale_grant=policy_grant_decision_not_adjacent ... grant_committed=false
policy_authority_hardening:canonical_write_rederivation ok
policy_authority_hardening:canonical_evidence_and_review ok
policy_authority_hardening:grant_adjacency_and_historical_replay ok
```

Proves: forged authority remains rejected and historical P1 truth remains
replayable after current-policy replacement.

## E11-Q06 — dedicated temporal smoke

- run_id: `wave11-qualification-20260830-06`
- execution_order: 06
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: local socket/provider fixture plus deterministic test clocks
- pre-state: prior regression and hardening gates green
- command: `make smoke-temporal-governance`
- exit: `0`
- produced IDs: product artifacts `policy-artifact:2f2a2446e76a6380473c9d0aec0643ab1408d124e9f63a6056e9b151ed478076`, `policy-artifact:9729f6c741b9b2dd4a66a36fd063ff6e54688d37d2e0693cf4153b1247b5d0b3`

Raw bounded output:

```text
wave11_grant_expiry: grant=grant:6ec4f8ba7955b417d2d7c9ea6d72e6fa issued_at=1788099386000 expires_at=1788099416000 authority_floor=1788099416000 invalidation_transition=transition:grant-invalidated:grant:6ec4f8ba7955b417d2d7c9ea6d72e6fa:12 prepare=false effects=0
wave11_prepare_cut: grant=grant:db33e5556e9948e85f4c1e0bd48eadc9 prepare_generation=12 revoke_after_prepare=true cancellation_generation=13 ... reconcile_generation=15 close_generation=16 ... effect_truth_preserved=true
wave11_temporal_governance: ... valid=true not_yet_valid=true refresh_required=true expired=true rollback_floor=1788099436000 stale_pinned_to_p1=true retired_stale=true weakest_composition=true explicit_refresh=true ... revoked=true
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 97 filtered out
temporal_governance_characterization: pass
prepare_exit: 86
unsafe_close_exit: 2
closed_effect_exit: 2
```

Proves: all temporal postures, rollback fence, finite Grant, PREPARE cut,
reconciliation and closure are executable together.

## E11-Q07 — bounded 128-iteration endurance

- run_id: `wave11-qualification-20260830-07`
- execution_order: 07
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: deterministic in-process history
- pre-state: isolated test state
- commands:
  - `cargo test --manifest-path engine/Cargo.toml -p yai-engine hundred_iteration_case_state_memory_context_endurance -- --nocapture`
  - `cargo test --manifest-path engine/Cargo.toml -p yai-engine hundred_iteration_planning_remains_bounded -- --nocapture`
- exits: `0`, `0`
- produced IDs: deterministic test-local history only

Raw output:

```text
running 1 test
test residency::tests::hundred_iteration_case_state_memory_context_endurance ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 102 filtered out
running 1 test
test residency::tests::hundred_iteration_planning_remains_bounded ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 102 filtered out
```

The historical test names say “hundred”; their source loops execute 128
iterations. Proves bounded CaseState/memory/context and residency planning.

## Captured failure history

The first focused Wave11 run produced this real failure before the test was
corrected to respect severity composition:

```text
assertion `left == right` failed
  left: Expired
 right: Stale
test store::lmdb::tests::wave11_policy_time_postures_revoke_stale_refresh_and_clock_floor_contract ... FAILED
test result: FAILED. 4 passed; 1 failed
```

This exposed a test sequencing error: the persisted clock floor had already
crossed P1 expiry, and `Expired` correctly outranks catalog `Stale`. The
unchanged implementation was then tested for stale before advancing the floor.

The first product harness run failed with `failed to start ipc server: invalid`
inside the restricted sandbox; the same socket characterization was rerun with
the required local socket permission. A later harness assertion expected
lowercase `effect_observed` while the unedited product output was
`reconciliation: EffectObserved`; the assertion and fixture cleanup were fixed,
then the unchanged product flow passed.

The first final `make check` after introducing authority time failed at the
26-turn runtime with this actual output:

```text
policy_grant_decision_semantics_stale
make: *** [Makefile:566: smoke-agentless-case-runtime] Error 2
```

An unchanged direct rerun after the first attempted correction then exposed
the narrower write-boundary race:

```text
authority_decision_time_mismatch
make: *** [Makefile:566: smoke-agentless-case-runtime] Error 2
```

Decision derivation and canonical append were moved into one RW transaction
with one authority time. The unchanged characterization then returned:

```text
case_runtime:agentless_26_turn_provider_model_replacement ok
case_runtime:deny_adaptation_and_bounded_residency ok
case_runtime:grant_effect_and_memory_restart_recovery ok
case_runtime:malformed_retry_operator_stop ok
case_runtime:budget_stops_before_extra_invocation ok
```

The next full `make check` reached the human-review smoke and exited `1`. A
traced unchanged reproduction showed the product had correctly persisted the
new Wave11 terminal result:

```text
review_invalidation: committed
invalidation_reason: Some(PolicyBasisChanged)
review_authority_invalidated
```

The old test still required `review_policy_basis_stale`. It was updated to
assert the stronger typed invalidation; its unchanged product flow then passed
all six human-review characterizations, followed by the green E11-Q03 gate.

## E11-Q08 — formatting, documentation and Clippy qualification

- run_id: `wave11-qualification-20260830-08`
- execution_order: 08
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: repository toolchains; no authority-time injection
- pre-state: final Wave11 source and dossier complete, publication pending
- commands:
  - `cargo fmt --manifest-path engine/Cargo.toml --all -- --check`
  - `cargo fmt --manifest-path cmd/yai/Cargo.toml --all -- --check`
  - `make check-docs`
  - `git diff --check`
  - `cargo clippy --manifest-path engine/Cargo.toml --workspace --all-targets`
  - `cargo clippy --manifest-path cmd/yai/Cargo.toml --workspace --all-targets`
- exits: `0`, `0`, `0`, `0`, `0`, `0`
- produced IDs: none

Raw bounded output:

```text
doc_root_canon: ok
check-doc-canonical-location: ok
check-doc-required-files: ok
check-doc-links: ok (28 files)
check-repository-identity: ok
warning: `yai-engine` (lib) generated 15 warnings
warning: `yai-engine` (lib test) generated 15 warnings (15 duplicates)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.73s
warning: `yai` (bin "yai" test) generated 17 warnings
warning: `yai` (bin "yai") generated 17 warnings (17 duplicates)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.58s
```

The format and diff checks are silent on success. Standard Clippy exits 0 with
the repository's historical warning classes; this is not presented as a
repository-wide `-D warnings` result. Wave11-introduced warnings encountered
during implementation were fixed before this capture.
