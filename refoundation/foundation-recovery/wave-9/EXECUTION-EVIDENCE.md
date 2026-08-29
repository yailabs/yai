# Wave 9 executable evidence

Only commands actually executed during Wave 9 appear below. Bounded excerpts
retain the emitted lines verbatim. The product session used one retained
temporary YAI home so IDs and generations are mutually consistent.

## Product evidence

Common cwd: `/home/mothx/computer-science/projects/YAI/yai`

Common environment for P01-P10:
`YAI_HOME=/tmp/yai-case-policy.wkt2ZC/home`. The initial complete scenario was
also executed as:

```text
$ YAI_KEEP_TEST_DIR=1 YAI_EVIDENCE_COMPACT=1 bash tests/characterization/case-policy-materialization/test_case_policy_materialization.sh
exit: 0
case_policy_materialization_characterization: pass
case_id: case:new12-filesystem
policy_v1: policy-artifact:e2c39ff02b9716f4b905d3c7239bbd1e79bdb95be73c99385dadf5e818186ee7
policy_v2: policy-artifact:c68ae4d39a4efea0e9300e292097eb6dba0451b0f98b42ae57666cb23ddbc353
active_bindings: 2
canonical_transitions: 9
effective_policy_id: effective-policy:053799a7fa0101494049b88fa4f98576
authority_emitted: false
```

### E9-P01 — ingest, validate and publish exact policy

Commands actually executed inside the retained scenario (all exit `0`):

```text
$ target/debug/yai policy ingest /tmp/yai-case-policy.wkt2ZC/policies/security-v1.json --as participant:policy-admin
$ target/debug/yai policy validate policy-artifact:e2c39ff02b9716f4b905d3c7239bbd1e79bdb95be73c99385dadf5e818186ee7 --as participant:policy-admin
$ target/debug/yai policy publish policy-artifact:e2c39ff02b9716f4b905d3c7239bbd1e79bdb95be73c99385dadf5e818186ee7 --as participant:policy-admin
policy_ingest: candidate_created
source_id: policy-source:19831898c9762db7e0531bb35d40b872892bf0a6305eaf900877aa4cad4aada2
artifact_id: policy-artifact:e2c39ff02b9716f4b905d3c7239bbd1e79bdb95be73c99385dadf5e818186ee7
artifact_version: 1
lifecycle: candidate
runtime_consumable: false
policy_validate: validated
lifecycle: validated
runtime_consumable: false
policy_publish: published
lifecycle: published
runtime_consumable: true
```

Proves that existence, validation and binding eligibility are distinct.

### E9-P02/P03 — bind and inspect exact artifact

```text
$ target/debug/yai case policy bind --case case:new12-filesystem --artifact policy-artifact:e2c39ff02b9716f4b905d3c7239bbd1e79bdb95be73c99385dadf5e818186ee7 --expected-generation 6 --as participant:operator
exit: 0
transition_id: transition:policy-bind:case-policy-binding:bf2f1da783de73efddbf239d0db83be3
case_generation: 7
transition_count: 7
normative_readiness: ready
active_policy_bindings: 1
policy_binding: binding_id=case-policy-binding:bf2f1da783de73efddbf239d0db83be3 lineage_id=policy-lineage:0098ff1cf931bf524a31ef68ca3edc05d6652c3ff51bb65b5e01a5ca7ac2074e owner_ref=organization:wave9 policy_key=filesystem-security artifact_id=policy-artifact:e2c39ff02b9716f4b905d3c7239bbd1e79bdb95be73c99385dadf5e818186ee7 version=1 publication_event=policy-event:3b97505a3e290fedb335ba3fe38018c24d05940e9700ecafe2d9c5d14e932810 bound_generation=7
effective_policy_id: effective-policy:4fbe6aa9d86e3c68a6064b6de57b4fd1
effective_policy_digest: sha256:4fbe6aa9d86e3c68a6064b6de57b4fd12d537645b87007b6c9a044552484eb2b
materializer_version: yai.policy_materializer.v1
effective_input_rules: 3
effective_output_rules: 3
effective_provenance_contributions: 3
blocking_conflicts: 0
missing_inputs: 0
```

Proves exact artifact/version/publication-event pinning and derived readiness.

### E9-P04/P08/P10 — multi-artifact conservative composition, provenance, no authority

The second lineage bind command actually executed with expected generation 7:

```text
$ target/debug/yai case policy bind --case case:new12-filesystem --artifact policy-artifact:b212b99f1a0eb88720fcc86cd78b77c226e70520edaad9c668b422bd05061c2d --expected-generation 7 --as participant:operator
exit: 0
case_generation: 8
active_policy_bindings: 2
effective_policy_id: effective-policy:581a8ad75a6bf523119afc3dc2dd016b
effective_input_rules: 6
effective_output_rules: 3
effective_merged_rules: 3
effective_resolved_conflicts: 2
effective_provenance_contributions: 6
blocking_conflicts: 0
missing_inputs: 0
decision_count: 0
execution_grant_count: 0
prepared_effect_count: 0
authority_emitted_by_case_policy: false
```

The emitted effective rule excerpt from that same real run was:

```text
effective_rule: {"kind":"operation_restriction","operation_kind":"filesystem.write","resource_kind":"filesystem","effect":"deny","resolution":"deny_dominates_allow_under_yai.policy_materializer.v1","contributions":[{"binding_id":"case-policy-binding:48fb43eb89189c19247ec735307a0712","artifact_id":"policy-artifact:b212b99f1a0eb88720fcc86cd78b77c226e70520edaad9c668b422bd05061c2d","policy_ir_rule_id":"operation-audit-obligations-1","source_id":"policy-source:ed7cb5dc35ef61bc8e192025cda6c1538253b694b3c28dc9238ab20b7aad4855","fact_refs":["policy-fact:7e5a3df0fab86d6d5e491ed225c3a60e552f337fd4f472d1b14f9a69c0f8d55a"],"source_locations":["$.rules[0]"]},{"binding_id":"case-policy-binding:4fcfc1bf432496016d93ab05dbe010c1","artifact_id":"policy-artifact:e2c39ff02b9716f4b905d3c7239bbd1e79bdb95be73c99385dadf5e818186ee7","policy_ir_rule_id":"operation-filesystem-security-1","source_id":"policy-source:19831898c9762db7e0531bb35d40b872892bf0a6305eaf900877aa4cad4aada2","fact_refs":["policy-fact:51cc060bef390ef98665c933642085f8153721b232ac7ea3d7d7cb9d55ae2c82"],"source_locations":["$.rules[0]"]}]}
```

The rule excerpt is from the earlier full-output session and therefore has its
own binding IDs; its source/artifact semantics are the same deterministic
inputs. It proves that conflict resolution keeps both causal contributions.

### E9-P05 — Candidate, Validated, Superseded, Retired, stale and missing Case fail closed

Candidate attempt in the retained scenario:

```text
$ target/debug/yai case policy bind --case case:new12-filesystem --artifact policy-artifact:e2c39ff02b9716f4b905d3c7239bbd1e79bdb95be73c99385dadf5e818186ee7 --expected-generation 6 --as participant:operator
exit: 2
policy_artifact_not_eligible_for_new_case_binding: lifecycle=Candidate runtime_consumable=false
```

Additional commands were executed against the same retained YAI home:

```text
$ target/debug/yai case policy bind --case case:new12-filesystem --artifact policy-artifact:2b1b71e9c07eb16868ccb22e41d9eff9bef9d9d8326640cf968c925cd3a7d584 --expected-generation 9 --as participant:operator
exit: 2
policy_artifact_not_eligible_for_new_case_binding: lifecycle=Validated runtime_consumable=false

$ target/debug/yai case policy bind --case case:new12-filesystem --artifact policy-artifact:e2c39ff02b9716f4b905d3c7239bbd1e79bdb95be73c99385dadf5e818186ee7 --expected-generation 10 --as participant:operator
exit: 2
policy_artifact_not_eligible_for_new_case_binding: lifecycle=Superseded runtime_consumable=false

$ target/debug/yai case policy bind --case case:new12-filesystem --artifact policy-artifact:b212b99f1a0eb88720fcc86cd78b77c226e70520edaad9c668b422bd05061c2d --expected-generation 11 --as participant:operator
exit: 2
policy_artifact_not_eligible_for_new_case_binding: lifecycle=Retired runtime_consumable=false

$ target/debug/yai case policy bind --case case:new12-filesystem --artifact policy-artifact:c68ae4d39a4efea0e9300e292097eb6dba0451b0f98b42ae57666cb23ddbc353 --expected-generation 9 --as participant:operator
exit: 2
stale_case_generation: expected=9 actual=11

$ target/debug/yai case policy bind --case case:does-not-exist --artifact policy-artifact:c68ae4d39a4efea0e9300e292097eb6dba0451b0f98b42ae57666cb23ddbc353 --expected-generation 0 --as participant:operator
exit: 2
case_state_not_found: case:does-not-exist
```

Each failure occurred before a binding Transition.

### E9-P06 — P@2 publication does not move P@1 binding

```text
$ target/debug/yai policy publish policy-artifact:c68ae4d39a4efea0e9300e292097eb6dba0451b0f98b42ae57666cb23ddbc353 --as participant:policy-admin
$ target/debug/yai case policy status --case case:new12-filesystem
exit: 0 (each)
policy_publish: published
artifact_id: policy-artifact:c68ae4d39a4efea0e9300e292097eb6dba0451b0f98b42ae57666cb23ddbc353
artifact_version: 2
lifecycle: published
runtime_consumable: true
case_generation: 8
transition_count: 8
policy_binding: binding_id=case-policy-binding:bf2f1da783de73efddbf239d0db83be3 lineage_id=policy-lineage:0098ff1cf931bf524a31ef68ca3edc05d6652c3ff51bb65b5e01a5ca7ac2074e owner_ref=organization:wave9 policy_key=filesystem-security artifact_id=policy-artifact:e2c39ff02b9716f4b905d3c7239bbd1e79bdb95be73c99385dadf5e818186ee7 version=1 publication_event=policy-event:3b97505a3e290fedb335ba3fe38018c24d05940e9700ecafe2d9c5d14e932810 bound_generation=7
catalog_drift: lineage_id=policy-lineage:0098ff1cf931bf524a31ef68ca3edc05d6652c3ff51bb65b5e01a5ca7ac2074e status=superseded:current=policy-artifact:c68ae4d39a4efea0e9300e292097eb6dba0451b0f98b42ae57666cb23ddbc353
```

### E9-P07 — explicit replacement

```text
$ target/debug/yai case policy replace --case case:new12-filesystem --binding case-policy-binding:bf2f1da783de73efddbf239d0db83be3 --artifact policy-artifact:c68ae4d39a4efea0e9300e292097eb6dba0451b0f98b42ae57666cb23ddbc353 --expected-generation 8 --as participant:operator
exit: 0
transition_id: transition:policy-replace:case-policy-binding:9afa02964c0e334e3966031d82047908
case_generation: 9
active_policy_bindings: 2
policy_binding: binding_id=case-policy-binding:9afa02964c0e334e3966031d82047908 lineage_id=policy-lineage:0098ff1cf931bf524a31ef68ca3edc05d6652c3ff51bb65b5e01a5ca7ac2074e owner_ref=organization:wave9 policy_key=filesystem-security artifact_id=policy-artifact:c68ae4d39a4efea0e9300e292097eb6dba0451b0f98b42ae57666cb23ddbc353 version=2 publication_event=policy-event:0274c852b1657e16534878b348b118a568354c32af37ed20d0269161dd4f91e1 bound_generation=9
effective_policy_id: effective-policy:053799a7fa0101494049b88fa4f98576
catalog_drift: lineage_id=policy-lineage:0098ff1cf931bf524a31ef68ca3edc05d6652c3ff51bb65b5e01a5ca7ac2074e status=current
```

### E9-P09 — pure status and derived rebuild

```text
$ target/debug/yai case policy status --case case:new12-filesystem
$ target/debug/yai case policy rebuild --case case:new12-filesystem
exit: 0 (each)
effective_policy_rebuild: completed
canonical_transitions_before: 9
canonical_transitions_after: 9
case_generation: 9
transition_count: 9
effective_policy_id: effective-policy:053799a7fa0101494049b88fa4f98576
effective_policy_digest: sha256:053799a7fa0101494049b88fa4f98576bf1b2a079a405a4eecd1f2e2a28d55f2
decision_count: 0
execution_grant_count: 0
prepared_effect_count: 0
authority_emitted_by_case_policy: false
```

Proves query purity, rebuild equivalence and the no-authority boundary.

## Qualification evidence

### E9-Q01 — focused Wave-9 engine tests

```text
$ cargo test --manifest-path engine/Cargo.toml wave9_ -- --nocapture
exit: 0
running 6 tests
test store::lmdb::tests::wave9_missing_artifact_blocks_readiness_without_erasing_binding_history ... ok
test store::lmdb::tests::wave9_derived_cache_failure_preserves_canonical_binding_and_repairs_without_duplication ... ok
test store::lmdb::tests::wave9_binding_admission_rejects_candidate_validated_and_missing_case ... ok
test store::lmdb::tests::wave9_multi_artifact_composition_is_order_independent_conservative_and_provenanced ... ok
test store::lmdb::tests::wave9_idempotence_unbind_multi_case_and_concurrent_mutation_are_safe ... ok
test store::lmdb::tests::wave9_exact_version_binding_replacement_replay_and_rebuild_are_deterministic ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 81 filtered out
```

### E9-Q02 — Rust build/test checkpoint

```text
$ make build-rust
exit: 0
running 87 tests
test store::lmdb::tests::wave9_derived_cache_failure_preserves_canonical_binding_and_repairs_without_duplication ... ok
test store::lmdb::tests::wave9_multi_artifact_composition_is_order_independent_conservative_and_provenanced ... ok
test store::lmdb::tests::wave9_exact_version_binding_replacement_replay_and_rebuild_are_deterministic ... ok
test store::lmdb::tests::wave9_idempotence_unbind_multi_case_and_concurrent_mutation_are_safe ... ok
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### E9-Q03 — full repository check

```text
$ make check
exit: 0
check-no-old-roots: ok
check-required-layout: ok
check-source-placement: ok
check-source-surface-clean: ok
check-doc-links: ok (28 files)
running 88 tests
test store::lmdb::tests::wave9_many_policy_materialization_characterization_is_bounded ... ok
test result: ok. 88 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
controlled_effect:derived_failure_isolation ok
semantic_continuity:provider_replacement ok
case_runtime:agentless_26_turn_provider_model_replacement ok
human_review:crash_r1_r6_recovery ok
governance_intake_characterization: pass
governance_hardening_characterization: pass
```

### E9-Q04 — characterization with local IPC enabled

The first restricted-sandbox attempt exited 2 at the first legacy daemon test:

```text
$ make characterization
failed to start ipc server: invalid
make: *** [Makefile:426: smoke-new11] Error 1
exit: 2
```

The unchanged command was then run with local Unix-socket permission:

```text
$ make characterization
exit: 0
provider_model_vertical:real_http_invocation ok
semantic_continuity:provider_replacement ok
controlled_effect:prepare_crash_reconciliation ok
case_runtime:agentless_26_turn_provider_model_replacement ok
human_review:crash_r1_r6_recovery ok
governance_intake_characterization: pass
governance_hardening_characterization: pass
```

This distinguishes sandbox IPC restriction from product regression.

### E9-Q05 — required Wave 8/H8/Wave 9 and runtime smokes

```text
$ make smoke-governance-intake smoke-governance-hardening smoke-case-policy-materialization smoke-human-review-runtime smoke-semantic-continuity smoke-agentless-case-runtime
exit: 0
test result: ok. 88 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
governance_intake_characterization: pass
governance_hardening_characterization: pass
case_policy_materialization_characterization: pass
case_id: case:new12-filesystem
active_bindings: 2
canonical_transitions: 9
authority_emitted: false
human_review:approve_deny_defer_and_query_purity ok
semantic_continuity:provider_replacement ok
case_runtime:agentless_26_turn_provider_model_replacement ok
```

### E9-Q06 — final focused materialization/scale and hygiene

```text
$ cargo test --manifest-path engine/Cargo.toml wave9_ -- --nocapture
exit: 0
running 7 tests
wave9_multi_policy_characterization: artifacts=24 input_rules=72 output_rules=3 merged_rules=69 resolved_conflicts=2 blocking_conflicts=0 derived_bytes=35268 elapsed_ms=4242
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 81 filtered out

$ cargo fmt --manifest-path engine/Cargo.toml --all -- --check
exit: 0
$ cargo fmt --manifest-path cmd/yai/Cargo.toml --all -- --check
exit: 0
$ git diff --check
exit: 0
```

Clippy was also run. After correcting three findings in the new materializer,
the new Wave-9 owner produced no warning. Repository-wide output retained 14
engine and 17 CLI historical warnings outside the new owner; no green `-D
warnings` claim is made.
