# H10 execution evidence

Only commands executed during H10 appear here. Bounded excerpts are unedited
subsets of actual stdout/stderr. Full product logs were retained during the run
at `/tmp/h10-policy-authority-product.log` and
`/tmp/h10-human-review-product.log`; `/tmp` is execution residue, not authority.

## E-H10-F01 — pre-fix forged ALLOW bypass

- run_id: `h10-engine-prefixed-forgery-01`
- execution_order: 1
- purpose: prove content-valid authority material could cross the old write boundary
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: normal test environment; isolated temporary LMDB
- pre-state: actual current EffectivePolicy posture DENY; forged evaluator input
  retained the real policy ID/digest but substituted ALLOW and resealed both hashes
- exact command: `cargo test --manifest-path engine/yai-engine/Cargo.toml wave10_policy_mutation_between_decision_and_grant_fails_in_same_transaction -- --nocapture`
- exit: 101

```text
running 1 test
thread 'store::lmdb::tests::wave10_policy_mutation_between_decision_and_grant_fails_in_same_transaction' panicked at engine/yai-engine/src/store/lmdb.rs:
content-valid forged ALLOW must fail semantic re-derivation: CanonicalCommit { transition: Transition { sequence: 12,
decision_id: "decision:e2e56a7e363f3baf48ab9a185c78a43c"
basis_id: "decision-basis:bbc5db97db894416e20448484dadf1b7"
effective_policy_id: "effective-policy:7dd713c72e1495aca400571e41e6dd1b"
operation_restriction: ExplicitAllow
final_posture: Allow
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 94 filtered out
```

Proves the original H10 bypass was real: digest/ID consistency did not establish
semantic authority.

## E-H10-Q01 — unchanged forgery reproduction after fix

- run_id: `h10-engine-final-01`
- execution_order: 2
- purpose: Decision/Grant injection and non-policy authority staleness
- cwd: repository root
- environment: normal test environment; isolated temporary LMDB
- pre-state: same forged ALLOW scenario, then a valid ALLOW followed by a
  Participant role Transition and by explicit policy replacement
- exact command: `cargo test --manifest-path engine/yai-engine/Cargo.toml wave10_policy_mutation_between_decision_and_grant_fails_in_same_transaction -- --nocapture`
- exit: 0

```text
running 1 test
h10_authority_injection: forged_decision=authority_decision_basis_mismatch forged_grant=policy_execution_grant_semantic_mismatch role_stale_grant=policy_grant_decision_not_adjacent policy_stale_grant=policy_grant_decision_not_adjacent decision_basis=decision-basis:3ae5e6ce9f0df85f5ef6e96a56a68d03 evaluated_effective_policy=effective-policy:2d736bb624f9121586f182f41e63a135 current_effective_policy=effective-policy:138b88bfb0a120db6fd28018294d5148 grant_committed=false
test store::lmdb::tests::wave10_policy_mutation_between_decision_and_grant_fails_in_same_transaction ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 96 filtered out
```

Produced refs: basis `decision-basis:3ae5e6ce…`, evaluated policy
`effective-policy:2d736bb6…`, current policy `effective-policy:138b88bf…`.
Proves hashes are not bearer authority and any intervening canonical authority
state prevents issuance from the old Decision.

## E-H10-Q02 — canonical evidence and review re-derivation

- run_id: `h10-engine-final-02`
- execution_order: 3
- purpose: forged evidence, reviewer roles, Request and final Decision
- cwd: repository root
- environment: normal test environment; isolated temporary LMDB
- pre-state: canonical provider-origin Operation and policy requiring proposer,
  reviewer, source provenance, audit reason and post observation
- exact command: `cargo test --manifest-path engine/yai-engine/Cargo.toml h10_review_writes_rederive_roles_provenance_and_final_decision -- --nocapture`
- exit: 0

```text
running 1 test
h10_review_rederivation: caller_evidence=authority_decision_basis_mismatch forged_request=authority_review_request_mismatch wrong_reviewer=review_action_binding_or_generation_mismatch forged_final=authority_decision_basis_mismatch canonical_final=true
test store::lmdb::tests::h10_review_writes_rederive_roles_provenance_and_final_decision ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 96 filtered out
```

Proves caller evidence strings, a resealed role-tampered ReviewRequest, an
ineligible low-level action and a resealed final Decision all fail at the
canonical boundary; the independently derived final Decision commits.

## E-H10-Q03 — historical P1 chain survives P2

- run_id: `h10-engine-final-03`
- execution_order: 4
- purpose: distinguish historical truth from current authorization
- cwd: repository root
- environment: normal test environment; isolated temporary LMDB
- pre-state: complete P1 ALLOW chain, then explicit binding replacement to P2 DENY
- exact command: `cargo test --manifest-path engine/yai-engine/Cargo.toml h10_historical_p1_authority_chain_replays_after_p2_and_cache_rebuild -- --nocapture`
- exit: 0

```text
running 1 test
h10_historical_replay: operation_generation=9 basis_generation=9 decision_transition=10 grant_expected_generation=10 grant_transition=11 prepare_transition=12 finalize_transition=13 replacement_transition=14 p1_basis=decision-basis:2bfff403f006572e034e75ff9d6baa65 p1_decision=decision:58053833059f6c0023abc2852ee789a7 p1_grant=grant:2d872b0c27af97949d61c0648726dfe5 p1_receipt=effect-receipt:a594b1cbfa505a23abcd493cbfc4e506 current_p2=effective-policy:2ae1335e9aa5ffe01c6f78f03735164d replay=true
test store::lmdb::tests::h10_historical_p1_authority_chain_replays_after_p2_and_cache_rebuild ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 96 filtered out
```

Produced refs are printed above. Proves exact adjacency, obligation-closed
effect history, P1 replay after P2, and no historical re-evaluation under P2.

## E-H10-P01 — ordinary governed ALLOW still closes the real carrier chain

- run_id: `h10-product-N3Hzt8-allow`
- execution_order: 5 / product command 08
- purpose: product ALLOW regression and complete causal chain
- cwd: repository root
- environment: `YAI_HOME=/tmp/yai-policy-admission.N3Hzt8/allow/home`;
  `YAI_JOURNAL=/tmp/yai-policy-admission.N3Hzt8/allow/journal.jsonl`;
  local fixture endpoint `127.0.0.1:37005`
- pre-state: Published artifact bound exactly; Ready EffectivePolicy;
  Case-bound proposer role; zero Decision/Grant/effect for the proposal
- exact command and exit: below, exit 0

```text
$ YAI_HOME=/tmp/yai-policy-admission.N3Hzt8/allow/home YAI_JOURNAL=/tmp/yai-policy-admission.N3Hzt8/allow/journal.jsonl /home/mothx/computer-science/projects/YAI/yai/target/debug/yai effect filesystem-write --case case:new12-filesystem --subject subject:llm-provider --attachment workspace --prompt 'propose one policy-governed write' --provider-id provider:allow --base-url http://127.0.0.1:37005/v1/chat/completions --model controlled-model
provider_invocation_id: invocation:model-prompt-32
provider_result_id: provider-result:model-output-33
provider_result_authority: non_authoritative_candidate_material
operation_normalization: accepted
operation_id: operation:989230c14a68c8f09a38a673ec40ad45
decision_id: decision:b19c7f4e28517382db31e39312aa16fe
decision_basis_id: decision-basis:73c2cce693ddeee915fdaf91f335a2ca
decision: allow
execution_grant_id: grant:ff795aef93abe0d6832e635275a0e74c
effect_id: effect:ff795aef93abe0d6832e635275a0e74c
effect_state: prepared_durable_before_mutation
effect_receipt_id: effect-receipt:4c27aeee07d5b2e429a65a9126c0486d
effect_outcome: Applied
effect_state: finalized
effect_chain_closure: valid
second_provider_invocation_id: invocation:model-prompt-36
second_provider_result_id: provider-result:model-output-37
second_turn_consequence: observed_reality_from_canonical_state
exit: 0
```

Proves H10 does not regress the actual Provider→Operation→Basis→Decision→Grant
→PREPARE→Receipt→FINALIZE chain and that the next turn sees consequence.

## E-H10-P02 — DENY remains Grant/effect-free

- run_id: `h10-product-N3Hzt8-deny`
- execution_order: 6 / product command 09
- purpose: fail-closed negative product path
- cwd: repository root
- environment: isolated deny `YAI_HOME`; local fixture endpoint
- pre-state: Ready exact policy with applicable DENY; resource envelope permits target
- exact command and exit: below, exit 0 (DENY is a committed product outcome)

```text
$ YAI_HOME=/tmp/yai-policy-admission.N3Hzt8/deny/home YAI_JOURNAL=/tmp/yai-policy-admission.N3Hzt8/deny/journal.jsonl /home/mothx/computer-science/projects/YAI/yai/target/debug/yai effect filesystem-write --case case:new12-filesystem --subject subject:llm-provider --attachment workspace --prompt 'propose one policy-governed write' --provider-id provider:deny --base-url http://127.0.0.1:39685/v1/chat/completions --model controlled-model
provider_invocation_id: invocation:model-prompt-32
provider_result_id: provider-result:model-output-33
operation_id: operation:989230c14a68c8f09a38a673ec40ad45
decision_id: decision:3d0e65feddde3842a11a957171adc576
decision_reason: applicable_policy_deny
decision_basis_id: decision-basis:e9d0a8e49866fda20538cf4072267b7a
decision: deny
execution_grant: none
external_effect: none
exit: 0
```

Proves semantic write hardening did not create authority or effect on DENY.

## E-H10-P03 — review eligibility, canonical audit reason and same Operation

- run_id: `h10-review-pxjdx5-approve`
- execution_order: 7 / review product sequence
- purpose: policy-driven review end to end
- cwd: repository root
- environment: isolated `YAI_HOME=/tmp/yai-human-review.pxjdx5/approve/home`;
  local provider fixture
- pre-state: Ready review policy; proposer and reviewer roles bound; no review/effect
- exit: run/show/eligible approval/resume 0; wrong reviewer 2

```text
$ YAI_HOME=/tmp/yai-human-review.pxjdx5/approve/home YAI_JOURNAL=/tmp/yai-human-review.pxjdx5/approve/journal.jsonl /home/mothx/computer-science/projects/YAI/yai/target/debug/yai case run --case case:new12-filesystem --subject subject:llm-provider --attachment workspace --prompt 'propose one human-reviewed filesystem write, then report completion' --max-invocations 3 --max-operations 2 --max-resident-items 12 --max-semantic-units 6000 --max-estimated-input-units 50000
operation_id: operation:39a37e0572a03c9bc1906aac01d005f4
decision: require_review
review_id: review:c43ca38393f2b43dc3d2ec8d2f5bc10a
execution_grant: none
external_effect: none
runtime_status: AwaitingReview
invocations: 1
operations: 0
exit: 0

$ YAI_HOME=/tmp/yai-human-review.pxjdx5/approve/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai review show review:c43ca38393f2b43dc3d2ec8d2f5bc10a --case case:new12-filesystem
review_id: review:c43ca38393f2b43dc3d2ec8d2f5bc10a
operation_id: operation:39a37e0572a03c9bc1906aac01d005f4
initial_decision_id: decision:f676e0094628e8d3b3b5078070be9d33
required_reviewer_roles: operation-reviewer
decision_basis_id: decision-basis:55d6f03fa31c32a82d4e05f15fe9ed35
effective_policy_id: effective-policy:ccb5d32c167d6ed5642d3b48ed0ba12e
effective_decision_id: none
exit: 0

$ YAI_HOME=/tmp/yai-human-review.pxjdx5/approve/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai review approve review:c43ca38393f2b43dc3d2ec8d2f5bc10a --case case:new12-filesystem --as subject:llm-provider --reason 'self approve'
reviewer_not_eligible_for_case_review
exit: 2

$ YAI_HOME=/tmp/yai-human-review.pxjdx5/approve/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai review approve review:c43ca38393f2b43dc3d2ec8d2f5bc10a --case case:new12-filesystem --as subject:policy-pack --reason 'human participant approve exact operation'
review_action: committed
review_id: review:c43ca38393f2b43dc3d2ec8d2f5bc10a
reviewer_participant: subject:policy-pack
action: approve
action_id: review-action:sha256:5ac84d392ea70a1f1547c87ba
effective_decision_id: decision:7d5f7596b1d7002b770b9b9128ca96c4
execution_grant: none_review_command_never_executes
external_effect: none
exit: 0

$ YAI_HOME=/tmp/yai-human-review.pxjdx5/approve/home YAI_JOURNAL=/tmp/yai-human-review.pxjdx5/approve/journal.jsonl /home/mothx/computer-science/projects/YAI/yai/target/debug/yai case resume --case case:new12-filesystem
operation_id: operation:39a37e0572a03c9bc1906aac01d005f4
decision_basis_id: decision-basis:bd8a04a5d9fcd9618ba2fbd820be33f8
effective_policy_id: effective-policy:ccb5d32c167d6ed5642d3b48ed0ba12e
runtime_status: Completed
invocations: 2
operations: 1
last_effect_id: effect:b52d60f972129adcb60ef93ea7e3b589
exit: 0
```

Proves the wrong reviewer cannot act, the canonical human reason satisfies the
audit obligation, approval itself has no Grant/effect, and resume advances the
same Operation. The second invocation is the consequence turn, not an
authority re-proposal.

## E-H10-P04 — policy changes during review

- run_id: `h10-review-pxjdx5-stale`
- execution_order: 8 / review product commands 06–07
- purpose: old review basis cannot cross a new EffectivePolicy
- cwd: repository root
- environment: isolated stale-review `YAI_HOME`
- pre-state: pending ReviewRequest under E1; explicit same-lineage replacement to E2
- exits: replace 0; approval 2

```text
$ YAI_HOME=/tmp/yai-human-review.pxjdx5/stale/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai case policy replace --case case:new12-filesystem --binding case-policy-binding:034e63612f01bc06bf406021f9be61a6 --artifact policy-artifact:5c583238ebc18ee76656a643b6168d999fb9f90e49548318d26589d29deed2f1 --expected-generation 18 --as participant:local-policy-operator --reason 'replace while review pending'
case_policy_replace: committed
effective_policy_id: effective-policy:a0b0b95e5bda35466aaf097f8ce27a5d
exit: 0

$ YAI_HOME=/tmp/yai-human-review.pxjdx5/stale/home /home/mothx/computer-science/projects/YAI/yai/target/debug/yai review approve review:915a9fa882f70e33490757c2865bea1d --case case:new12-filesystem --as subject:policy-pack --reason 'human participant approve exact operation'
review_policy_basis_stale
exit: 2
```

Proves no old ReviewAction/Decision/Grant can be produced after a visible basis change.

## E-H10-Q04 — dedicated hardening characterization

- run_id: `h10-qualification-final-01`
- execution_order: 9
- cwd: repository root
- environment: normal test environment
- pre-state: final H10 source
- exact command: `make smoke-policy-authority-hardening`
- exit: 0

```text
running 2 tests
h10_historical_replay: operation_generation=9 basis_generation=9 decision_transition=10 grant_expected_generation=10 grant_transition=11 prepare_transition=12 finalize_transition=13 replacement_transition=14 p1_basis=decision-basis:2bfff403f006572e034e75ff9d6baa65 p1_decision=decision:58053833059f6c0023abc2852ee789a7 p1_grant=grant:2d872b0c27af97949d61c0648726dfe5 p1_receipt=effect-receipt:a594b1cbfa505a23abcd493cbfc4e506 current_p2=effective-policy:2ae1335e9aa5ffe01c6f78f03735164d replay=true
h10_review_rederivation: caller_evidence=authority_decision_basis_mismatch forged_request=authority_review_request_mismatch wrong_reviewer=review_action_binding_or_generation_mismatch forged_final=authority_decision_basis_mismatch canonical_final=true
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 95 filtered out
running 1 test
h10_authority_injection: forged_decision=authority_decision_basis_mismatch forged_grant=policy_execution_grant_semantic_mismatch role_stale_grant=policy_grant_decision_not_adjacent policy_stale_grant=policy_grant_decision_not_adjacent decision_basis=decision-basis:3ae5e6ce9f0df85f5ef6e96a56a68d03 evaluated_effective_policy=effective-policy:2d736bb624f9121586f182f41e63a135 current_effective_policy=effective-policy:138b88bfb0a120db6fd28018294d5148 grant_committed=false
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 96 filtered out
policy_authority_hardening:canonical_write_rederivation ok
policy_authority_hardening:canonical_evidence_and_review ok
policy_authority_hardening:grant_adjacency_and_historical_replay ok
```

## E-H10-Q05 — repository suite and characterization

- run_id: `h10-qualification-final-02`
- execution_order: 10
- cwd: repository root
- environment: repository defaults; permitted local IPC for second command
- pre-state: final source and historical dirty entries unchanged
- commands/exits: `make check` → 0; `make characterization` first sandbox run
  → 2; identical permitted rerun → 0

```text
$ make check
check-no-old-roots: ok
check-required-layout: ok
check-source-placement: ok
check-source-surface-clean: ok
doc_root_canon: ok
check-doc-links: ok (28 files)
running 97 tests
test store::lmdb::tests::wave10_policy_mutation_between_decision_and_grant_fails_in_same_transaction ... ok
test store::lmdb::tests::h10_historical_p1_authority_chain_replays_after_p2_and_cache_rebuild ... ok
test store::lmdb::tests::h10_review_writes_rederive_roles_provenance_and_final_decision ... ok
test result: ok. 97 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
human_review:crash_r1_r6_recovery ok
case_runtime:agentless_26_turn_provider_model_replacement ok
governance_hardening_characterization: pass
case_policy_materialization_characterization: pass
policy_authority:allow_chain ok
policy_authority:explicit_deny_and_no_match_fail_closed ok
exit: 0

$ make characterization
running 97 tests
test result: ok. 97 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
failed to start ipc server: invalid
make: *** [Makefile:428: smoke-new11] Error 1
exit: 2

$ make characterization
running 97 tests
test result: ok. 97 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
daemon:started
ipc:status ok
ipc:info ok
daemon:shutdown ok
case_runtime:agentless_26_turn_provider_model_replacement ok
human_review:crash_r1_r6_recovery ok
policy_authority:allow_chain ok
governance_intake_characterization: pass
governance_hardening_characterization: pass
exit: 0
```

The red characterization was an execution-sandbox IPC restriction: all Rust
tests had already passed and the identical command passed once local IPC was
allowed. It was not hidden or rewritten as green.

## E-H10-Q06 — bounded 128-iteration endurance

- run_id: `h10-qualification-final-03`
- execution_order: 11
- cwd: repository root
- environment: normal test environment
- pre-state: final H10 source
- exact command: `cargo test --manifest-path engine/yai-engine/Cargo.toml hundred_iteration -- --nocapture`
- exit: 0

```text
running 2 tests
test residency::tests::hundred_iteration_case_state_memory_context_endurance ... ok
test residency::tests::hundred_iteration_planning_remains_bounded ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 95 filtered out; finished in 0.20s
```

Both test bodies execute 128 iterations. This is bounded endurance evidence,
not a performance claim.

## E-H10-Q07 — Clippy characterization

- run_id: `h10-qualification-final-04`
- execution_order: 12
- cwd: repository root
- environment: normal Cargo environment
- pre-state: final H10 source
- exact commands: `cargo clippy --manifest-path engine/Cargo.toml --workspace --all-targets`;
  `cargo clippy --manifest-path cmd/yai/Cargo.toml --all-targets`
- exit: 0

```text
warning: `yai-engine` (lib) generated 14 warnings
warning: `yai-engine` (lib test) generated 14 warnings (14 duplicates)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.44s
warning: `yai` (bin "yai") generated 17 warnings (14 duplicates)
warning: `yai` (bin "yai" test) generated 17 warnings (3 duplicates)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.92s
```

The warnings are the repository's pre-existing categories/sites; the reported
LMDB line numbers are unchanged legacy compatibility code shifted by H10
insertions. No warning points into the new admission resolver, write validators
or runtime refresh block.

## E-H10-Q08 — authority crash-boundary rollback

- run_id: `h10-qualification-final-05`
- execution_order: 13
- cwd: repository root
- environment: normal test environment; isolated temporary LMDBs
- pre-state: valid semantically re-derived Decision, ReviewRequest and Grant
- exact command: `make smoke-policy-authority-hardening`
- exit: 0

```text
running 2 tests
h10_historical_replay: operation_generation=9 basis_generation=9 decision_transition=10 grant_expected_generation=10 grant_transition=11 prepare_transition=12 finalize_transition=13 replacement_transition=14 p1_basis=decision-basis:4171992359c02baab6a6f30ec2d618be p1_decision=decision:c4dc31176d79c507cfe94ab919504a11 p1_grant=grant:802de76a9998c3471a5fa39b0ca6bcd6 p1_receipt=effect-receipt:6baead8d07a1f63b2397fa448f956e9a current_p2=effective-policy:6ffc2f0af85a14b2d4b46a6e046d41c1 replay=true crash_c1_c2_c5=true
h10_review_rederivation: caller_evidence=authority_decision_basis_mismatch forged_request=authority_review_request_mismatch wrong_reviewer=review_action_binding_or_generation_mismatch forged_final=authority_decision_basis_mismatch canonical_final=true crash_c3_c4=true
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 95 filtered out
policy_authority_hardening:canonical_write_rederivation ok
policy_authority_hardening:canonical_evidence_and_review ok
policy_authority_hardening:grant_adjacency_and_historical_replay ok
```

C1/C3/C5 inject LMDB abort after successful semantic verification but before
commit and prove no transition/current-state residue. C2 proves a committed
Decision with no Grant reopens/replays safely; C4 proves a committed ReviewAction
with the initial Decision still current reopens/replays before final re-derivation.

## E-H10-Q09 — policy pre/post evidence obligation closure

- run_id: `h10-qualification-final-06`
- execution_order: 14
- cwd: repository root
- environment: normal test environment; isolated temporary LMDBs/filesystem
- pre-state: final H10 source with explicit policy `pre_observation` and
  `post_observation` obligations
- exact command: `make smoke-policy-authority-hardening`
- exit: 0

```text
running 2 tests
h10_historical_replay: operation_generation=9 basis_generation=9 decision_transition=10 grant_expected_generation=10 grant_transition=11 prepare_transition=12 finalize_transition=13 replacement_transition=14 p1_basis=decision-basis:1fe50c4c788093c9b99b94730456c5f3 p1_decision=decision:5eebacc8d5242805d4eba96ca79f3552 p1_grant=grant:4fda7a50da36af622eb3939daa2d229c p1_receipt=effect-receipt:5773b89973953da1ffbcbb2ce7e3be93 current_p2=effective-policy:e93d303af9164067f4559782534304a9 replay=true crash_c1_c2_c5=true
h10_review_rederivation: caller_evidence=authority_decision_basis_mismatch forged_request=authority_review_request_mismatch wrong_reviewer=review_action_binding_or_generation_mismatch forged_final=authority_decision_basis_mismatch canonical_final=true crash_c3_c4=true
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 95 filtered out
running 1 test
h10_authority_injection: forged_decision=authority_decision_basis_mismatch forged_grant=policy_execution_grant_semantic_mismatch role_stale_grant=policy_grant_decision_not_adjacent policy_stale_grant=policy_grant_decision_not_adjacent decision_basis=decision-basis:19493d73a451dd234128b383af30c443 evaluated_effective_policy=effective-policy:bb18059529a27f266dbc1b3bf73814e8 current_effective_policy=effective-policy:e6e81f2bf53945c934c69da9465c46cd grant_committed=false
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 96 filtered out
policy_authority_hardening:canonical_write_rederivation ok
policy_authority_hardening:canonical_evidence_and_review ok
policy_authority_hardening:grant_adjacency_and_historical_replay ok
```

The exercised Grant contains both typed execution requirements. PREPARE binds
the matching canonical pre-observation; an intentionally mismatched receipt is
rejected as `required_post_observation_evidence_missing`; the committed receipt
then closes the exact post-observation and the historical chain validates after
P1 is replaced by P2.

## E-H10-Q10 — final repository gate

- run_id: `h10-qualification-final-07`
- execution_order: 15
- cwd: repository root
- environment: repository defaults
- pre-state: exact final H10 source and evidence package
- exact command: `make check`
- exit: 0

```text
check-no-old-roots: ok
check-required-layout: ok
check-source-placement: ok
check-source-surface-clean: ok
doc_root_canon: ok
check-doc-links: ok (28 files)
running 97 tests
test store::lmdb::tests::wave10_policy_mutation_between_decision_and_grant_fails_in_same_transaction ... ok
test store::lmdb::tests::h10_historical_p1_authority_chain_replays_after_p2_and_cache_rebuild ... ok
test store::lmdb::tests::h10_review_writes_rederive_roles_provenance_and_final_decision ... ok
test result: ok. 97 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
case_runtime:agentless_26_turn_provider_model_replacement ok
human_review:crash_r1_r6_recovery ok
governance_intake_characterization: pass
governance_hardening_characterization: pass
case_policy_materialization_characterization: pass
policy_authority:allow_chain ok
policy_authority:explicit_deny_and_no_match_fail_closed ok
policy_authority:unconfigured_pre_provider_stop ok
```

The complete gate closed on the same source subsequently staged for H10.
