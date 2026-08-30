# Wave 12 execution evidence

Only command output actually observed during this Wave is retained here.
Temporary workspace IDs never cross run boundaries in a causal claim.

## E12-01 — direct legacy reinspection

- run_id: `w12-legacy-20260830-a`
- execution_order: 1
- pre-state: `yai-dev` historical repository read-only
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: local Git object database; no dossier input
- exact command: `git -C ../yai-dev show --format='%H %s' --no-renames --stat <commit> -- <required paths>` followed by commit-scoped `git grep`
- actual exit: 0 for every command
- produced IDs: none
- invariant: direct source/history, not TSV navigation, determines recovery

```text
1238741f7493e24138b00951f0d3547ec165306b checkpoint 2: login canonico — user session aperta post-auth
 usr/libexec/yai/runtime/yai-substrated.c    | 63 ++++++++++++++++----
 usr/libexec/yai/session/yai-session-shell.c | 91 +++++------------------------

1efadcfe94bba0d8cedf436716d93248ee81596e checkpoint/7A: root authority semantics and contamination cleanup
 usr/libexec/yai/runtime/yai-auth.c | 71 ++++++++++++++++++++++++++++++++++++++

c88e0193909f8bdad68b2841a81a9ba815e7a55c feat(protocol): unify RPC runtime + roles + root authority enforcement
 core/src/yai_root_server.c | 157 ++++++++++++++++++---------------------------
 law/specs/protocol/roles.h |   9 +++
 tools/cli/src/cmd_root.c   |  43 ++++++++-----

1238741f...:yai-substrated.c:270: strcmp(session.authn_kind, "pre-login") == 0 &&
1238741f...:yai-substrated.c:271: !session.authn_verified &&
1238741f...:yai-substrated.c:411: session.active_case_id[0] ? session.active_case_id : "case://substrate");
1efadcfe...:substrate_session.c:189: uid_t uid = getuid();
1efadcfe...:substrate_session.c:190: gid_t gid = getgid();
1efadcfe...:substrate_session.c:410: snprintf(session->principal_id,
1efadcfe...:substrate_session.c:703: snprintf(session->active_case_id,
c88e0193...:yai_root_server.c:171: if (env.role != YAI_ROLE_OPERATOR || !env.arming) {
c88e0193...:cmd_root.c:49: We always escalate to operator + armed.
```

## E12-02 — POSIX bootstrap and Tenant isolation product run

- run_id: `w12-product-tenant-CIwtZE`
- execution_order: 2
- pre-state: new `/tmp/yai-tenant-security.CIwtZE`; empty security/Case/policy catalog
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: `YAI_EXECUTION_EVIDENCE=1`, `YAI_EVIDENCE_COMPACT=1`, real UID/eUID/GID/eGID 1000
- exact command: `YAI_EXECUTION_EVIDENCE=1 YAI_EVIDENCE_COMPACT=1 tests/characterization/tenant-security/test_tenant_security.sh`
- actual exit: 0
- produced IDs: Principal `principal:72cc156b82060120eac8f7e234dbfcef`; artifacts `policy-artifact:cc03ce26dde8e3380f032f4f0cac38bdcd2a451cf04e936c15120ae7d5410a24` and `policy-artifact:76aa2c11c0febee471dd2f199e0627476cad2a190c1867771edce62876593873`
- invariant: kernel identity is stable; same organization/source does not merge Tenants; cross bind/root aliases/`--as` spoof fail

```text
$ ... yai security bootstrap-local --tenant tenant:product-a --organization organization:shared
authenticated: true
authentication_kind: local_posix_effective_credential
real_uid: 1000
effective_uid: 1000
real_gid: 1000
effective_gid: 1000
principal_id: principal:72cc156b82060120eac8f7e234dbfcef
tenant_id: tenant:product-a
membership: owner
exit: 0

$ ... yai identity whoami
authenticated: true
principal_id: principal:72cc156b82060120eac8f7e234dbfcef
authn_method: local_posix_effective_credential
tenant_relations: 2
exit: 0

$ ... yai policy ingest .../shared.json --tenant tenant:product-a
source_digest: sha256:82ec07b22cb3c913ab12c334afc32df6664897eebedfb836360c5bb11594cc71
artifact_id: policy-artifact:cc03ce26dde8e3380f032f4f0cac38bdcd2a451cf04e936c15120ae7d5410a24
policy_lineage_id: policy-lineage:a4f24cfecfdfd066ea27f993389554319ffeb2efe44d95dfb33bb6d449e6b8fb
exit: 0

$ ... yai policy ingest .../shared.json --tenant tenant:product-b
source_digest: sha256:82ec07b22cb3c913ab12c334afc32df6664897eebedfb836360c5bb11594cc71
artifact_id: policy-artifact:76aa2c11c0febee471dd2f199e0627476cad2a190c1867771edce62876593873
policy_lineage_id: policy-lineage:fbf4b40fb41d43efffbee49d818e048a1680a5c2d3577c6497abc180b3f13c3d
exit: 0

$ ... yai case policy bind --case case:product-a --artifact policy-artifact:76aa... --expected-generation 2 ...
cross_tenant_case_policy_binding_rejected
exit: 2
$ ... yai case attach-filesystem --case case:product-b --root /tmp/yai-tenant-security.CIwtZE/root ...
cross_tenant_filesystem_root_overlap: conflicting_case=case:product-a
exit: 2
$ ... yai case attach-filesystem --case case:product-b --root /tmp/yai-tenant-security.CIwtZE/root/nested ...
cross_tenant_filesystem_root_overlap: conflicting_case=case:product-a
exit: 2
$ ... yai case cancel --case case:product-a --as participant:forged --reason 'spoof must fail'
caller_supplied_as_cannot_authenticate_principal
exit: 2
tenant_security_characterization: pass
```

## E12-03 — Tenant-bound policy authority and real effect

- run_id: `w12-product-authority-3mWNUU`
- execution_order: 3
- pre-state: fresh Tenant-scoped characterization Cases and local HTTP fixtures
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: evidence compact; deterministic fixture authority time; local filesystem carrier
- exact command: `set -o pipefail; YAI_EXECUTION_EVIDENCE=1 YAI_EVIDENCE_COMPACT=1 tests/characterization/policy-authority-admission/test_policy_authority_admission.sh 2>&1 | rg '<bounded causal fields>'`
- actual exit: 0 (pipefail active)
- produced IDs: listed below, all from the allow sub-workspace of this run
- invariant: Tenant artifact → exact binding → EffectivePolicy → Provider result → Operation → DecisionBasis → Decision → Grant → EffectReceipt; deny/no-match/unconfigured remain effect-free

```text
artifact_id: policy-artifact:cfd6e72de84caa4fb015546fd2b11a96d075c607ee52b0a7de70546943407e55
tenant_id: tenant:characterization
policy_lineage_id: policy-lineage:07e0cfe88f9abf0fcfd6ccda55c9ca47e57daa7b6e9ef81aa368ff341eba898f
effective_policy_id: effective-policy:d0ae1c04a400c281752cac365f409493
provider_invocation_id: invocation:model-prompt-32
provider_result_id: provider-result:model-output-33
operation_id: operation:d4f6f2dcc65fd1e71de6271ed81ded7c
decision_id: decision:af99747ae93a62fec3dd0d4dd599f0d7
decision_reason: policy_admission_satisfied
decision_basis_id: decision-basis:20be0a020b3cf59e26630f40a17e60d8
decision: allow
execution_grant_id: grant:5b094f9323a9550c8b9b05588caff185
effect_id: effect:5b094f9323a9550c8b9b05588caff185
effect_receipt_id: effect-receipt:49e0432f45488f1da3fcacc9af7372e3
second_provider_invocation_id: invocation:model-prompt-36
second_turn_consequence: observed_reality_from_canonical_state
exit: 0

decision_reason: applicable_policy_deny
decision: deny
execution_grant: none
external_effect: none
exit: 0
decision_reason: no_applicable_allow_rule
decision: deny
execution_grant: none
external_effect: none
exit: 0
normative_readiness: Unconfigured
provider_invocations: 0
execution_grants: 0
external_effect: none
exit: 2
```

## E12-04 — authenticated human review

- run_id: `w12-product-review-uV5ejR`
- execution_order: 4
- pre-state: fresh Tenant Case with linked owner Principal/`subject:policy-pack`, reviewer role and review policy
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: evidence compact; local HTTP/provider fixture
- exact command: `set -o pipefail; YAI_EXECUTION_EVIDENCE=1 YAI_EVIDENCE_COMPACT=1 tests/characterization/human-review-runtime/test_human_review_runtime.sh 2>&1 | rg '<bounded review fields>'`
- actual exit: 0 (pipefail active)
- produced IDs: below, from the approve sub-workspace only
- invariant: same Operation continues; caller-selected provider Participant fails; authenticated Principal resolves to linked reviewer; stale policy review fails

```text
$ ... yai case run --case case:new12-filesystem ...
operation_id: operation:a724731c990a1a8e8cb617bcd5c87235
review_id: review:3860509545418d58138d51db2ae49041
external_effect: none
exit: 0
$ ... yai review show review:3860509545418d58138d51db2ae49041 --case case:new12-filesystem
required_reviewer_roles: operation-reviewer
decision_basis_id: decision-basis:3cc834b283db12cc31806de6afb05d8e
effective_policy_id: effective-policy:c5b3b03c66df1110e709b419571bd180
exit: 0
$ ... yai review approve review:3860509545418d58138d51db2ae49041 --case case:new12-filesystem --as subject:llm-provider --reason 'self approve'
exit: 2
$ ... yai review approve review:3860509545418d58138d51db2ae49041 --case case:new12-filesystem --reason 'human participant approve exact operation'
review_action: committed
reviewer_participant: subject:policy-pack
authenticated_principal_id: principal:72cc156b82060120eac8f7e234dbfcef
external_effect: none
exit: 0
$ ... yai case resume --case case:new12-filesystem
operation_id: operation:a724731c990a1a8e8cb617bcd5c87235
decision_id: decision:84e4baee3f87eb52fe5ad238038caf10
decision_basis_id: decision-basis:5a4b098ee25fecc870ca0cdca052572a
effective_policy_id: effective-policy:c5b3b03c66df1110e709b419571bd180
execution_grant_id: grant:8d1d43a636b5446804cdd297b147cf54
effect_id: effect:8d1d43a636b5446804cdd297b147cf54
effect_receipt_id: effect-receipt:3d1b121e8ecdd7621e0965e70f532e6d
exit: 0
$ ... yai case policy replace ...
effective_policy_id: effective-policy:32c556f4803e51c4dfef64d489182646
exit: 0
$ ... yai review approve review:f54d2e46d6c3b33930d83e7b483716ba ...
exit: 2
```

## E12-05 — focused security engine tests

- run_id: `w12-engine-focused-final`
- execution_order: 5
- pre-state: source complete; test-only authenticated credential constructors
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: `CARGO_TARGET_DIR=target`
- exact command: `cargo test --manifest-path engine/yai-engine/Cargo.toml wave12_ -- --nocapture`
- actual exit: 0
- produced IDs: test-local only
- invariant: restart, low-level injection, read/admin isolation, policy namespace and root overlap gates

```text
running 3 tests
wave12_root_isolation: tenant_a=tenant:root-a tenant_b=tenant:root-b exact=cross_tenant_filesystem_root_overlap overlap=cross_tenant_filesystem_root_overlap tenant_b_resource_count=0
test store::lmdb::tests::wave12_cross_tenant_filesystem_roots_reject_exact_and_overlapping_aliases ... ok
wave12_security_domains: principal_a=principal:910f060e28e27e81d3b2dc7b86f149dc principal_b=principal:67a32afdd5217336d7c973900635f0cf tenant_a=tenant:wave12-a tenant_b=tenant:wave12-b case_a=case:wave12-a case_b=case:wave12-b link=principal-participant-link:79e9c3e525337e0b266d79314828eac1 cross_tenant_read=denied member_admin=tenant_owner_required low_level_injection=authenticated_tenant_owner_required cancellation_actor=principal:910f060e28e27e81d3b2dc7b86f149dc closure_actor=principal:910f060e28e27e81d3b2dc7b86f149dc owner_cannot_reopen_closed=true restart_tenant=tenant:wave12-a organization_projection_shared_without_cross_access=true
test store::lmdb::tests::wave12_kernel_principals_tenants_and_case_links_are_isolated_and_restart_safe ... ok
wave12_policy_isolation: shared_source_digest=sha256:0407cc4a4f2129c5af8c21796827fb145365bb30e48564b7e5f2eb1896d9d23f artifact_a=policy-artifact:0cf2818cf53e0014f20b40b416c836befe200eb1362b9840096d35a2648ba92d artifact_b=policy-artifact:81e0045f742f28ddae406016b56079b5ebcc4cc0536f4f8a9a664ad283ce5f53 lineage_a=policy-lineage:40e086930483c559fb04ba121d1ffaa15ae88c8345eca2010c5ef5926675bc4f lineage_b=policy-lineage:c5ee7addaa301a2185a813fbb665c9116690d5afb8cc4ff46f2bd424cd2164f2 cross_bind=cross_tenant_case_policy_binding_rejected cross_revoke=tenant_not_visible tenant_b_lifecycle=published
test store::lmdb::tests::wave12_policy_namespace_is_tenant_bound_and_cross_tenant_binding_fails_closed ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 104 filtered out
```

## E12-06 — repository qualification gate

- run_id: `w12-qualification-check-final`
- execution_order: 6
- pre-state: all insecure live fixture fallbacks removed; graph fixtures explicitly tenant-scoped
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: ordinary repository build/test environment
- exact command: `make check`
- actual exit: 0
- produced IDs: fixture-local only
- invariant: repository layout/docs, full Rust suite and smoke surface remain green

```text
check-no-old-roots: ok
check-required-layout: ok
check-source-placement: ok
check-source-surface-clean: ok
check-doc-links: ok (28 files)
running 107 tests
...
test store::lmdb::tests::wave12_cross_tenant_filesystem_roots_reject_exact_and_overlapping_aliases ... ok
test store::lmdb::tests::wave12_kernel_principals_tenants_and_case_links_are_isolated_and_restart_safe ... ok
test store::lmdb::tests::wave12_policy_namespace_is_tenant_bound_and_cross_tenant_binding_fails_closed ... ok
test result: ok. 107 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
...
graph_relation:materialize ok
runtime_graph_query:bounded ok
case_runtime:agentless_26_turn_provider_model_replacement ok
human_review:crash_r1_r6_recovery ok
governance_intake_characterization: pass
governance_hardening_characterization: pass
case_policy_materialization_characterization: pass
policy_authority:allow_chain ok
exit: 0
```

## E12-07 — named H10/W11/W12 qualification

- run_id: `w12-qualification-named-final`
- execution_order: 7
- pre-state: final implementation
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: deterministic test authority time and local fixtures
- exact command: `make smoke-governance-intake smoke-governance-hardening smoke-case-policy-materialization smoke-policy-authority-admission smoke-policy-authority-hardening smoke-temporal-governance smoke-human-review-runtime smoke-semantic-continuity smoke-agentless-case-runtime smoke-tenant-security endurance-agentless-case-runtime`
- actual exit: 0
- produced IDs: isolated test-local IDs
- invariant: W8-H10-W11 behavior, crash boundaries, 26 turns, endurance and Wave12 remain intact

```text
test result: ok. 107 passed; 0 failed
governance_intake_characterization: pass
governance_hardening_characterization: pass
case_policy_materialization_characterization: pass
policy_authority:allow_chain ok
running 2 tests
test store::lmdb::tests::h10_historical_p1_authority_chain_replays_after_p2_and_cache_rebuild ... ok
test store::lmdb::tests::h10_review_writes_rederive_roles_provenance_and_final_decision ... ok
test result: ok. 2 passed; 0 failed
running 1 test
test store::lmdb::tests::wave10_policy_mutation_between_decision_and_grant_fails_in_same_transaction ... ok
test result: ok. 1 passed; 0 failed
running 6 tests
test store::lmdb::tests::wave11_cancellation_and_closure_are_atomic_terminal_and_replayable ... ok
test store::lmdb::tests::wave11_policy_time_postures_revoke_stale_refresh_and_clock_floor_contract ... ok
test result: ok. 6 passed; 0 failed
temporal_governance_characterization: pass
human_review:crash_r1_r6_recovery ok
case_runtime:agentless_26_turn_provider_model_replacement ok
tenant_security_characterization: pass
cross_bind_exit: 2
root_exact_exit: 2
root_overlap_exit: 2
spoof_exit: 2
exit: 0
```

## E12-08 — final characterization

- run_id: `w12-characterization-final`
- execution_order: 8
- pre-state: graph/runtime live fixtures migrated to explicit Tenant Cases
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: local provider/socket fixtures
- exact command: `make characterization`
- actual exit: 0
- produced IDs: fixture-local only
- invariant: replay/rebuild, graph, fact, effect, provider, review and governed runtime product characterizations all pass

```text
test result: ok. 107 passed; 0 failed
daemon-loop:filesystem completed
security:tenant case bootstrapped
replay_freeze:idempotent ok
graph_freeze:bounded ok
fact_freeze:idempotent ok
provider_model_vertical:real_http_invocation ok
semantic_continuity:memory_inspect_drop_rebuild ok
controlled_effect:prepare_crash_reconciliation ok
case_runtime:agentless_26_turn_provider_model_replacement ok
human_review:crash_r1_r6_recovery ok
policy_authority:explicit_deny_and_no_match_fail_closed ok
governance_intake_characterization: pass
governance_hardening_characterization: pass
exit: 0
```

## E12-F01 — failures that closed insecure integration seams

- run_id: `w12-implementation-failures`
- execution_order: implementation-time, before E12-05..08
- pre-state: partial Wave12 implementation
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: sandbox first run, then unchanged commands with required local socket permission
- exact commands: focused Rust H10/W11 tests; `make check`; `make characterization`
- actual exits: non-zero before fixes; all unchanged relevant reproductions exit 0 in E12-06..08
- produced IDs: none relied upon
- invariant: failures were preserved and classified instead of bypassed

```text
authority_decision_time_mismatch
new_live_case_requires_security_bootstrap_and_tenant_case_create
case_not_visible
```

The first exposed a real test/write-boundary race: positive Decision paths now
derive and commit within one transaction. The second and third proved the new
barrier was active: daemon and graph live fixtures were converted to explicit
security bootstrap and Tenant Case creation. The initial AF_UNIX bind denial was
sandbox infrastructure, not product behavior; the identical elevated local
fixture passed. No permissive fallback was added.

## E12-09 — final hygiene (pre-publication)

- run_id: `w12-hygiene-final`
- execution_order: 9
- pre-state: report/evidence added, formatting complete
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: repository toolchain
- exact commands: `cargo fmt --manifest-path engine/Cargo.toml --all -- --check`; `cargo fmt --manifest-path cmd/yai/Cargo.toml -- --check`; engine and command Clippy; `git diff --check`; historical checksum command
- actual exit: 0 for every command
- produced IDs: none
- invariant: no formatting/whitespace defect, no new Wave12 Clippy warning, historical dirty patch unchanged

```text
$ cargo fmt --manifest-path engine/Cargo.toml --all -- --check
exit: 0
$ cargo fmt --manifest-path cmd/yai/Cargo.toml -- --check
exit: 0
$ cargo clippy --manifest-path engine/Cargo.toml --workspace --all-targets
warning: `yai-engine` (lib) generated 14 warnings
Finished `dev` profile
exit: 0
$ cargo clippy --manifest-path cmd/yai/Cargo.toml --all-targets
warning: `yai` (bin "yai") generated 17 warnings
Finished `dev` profile
exit: 0
$ make check-docs
doc_root_canon: ok
check-doc-links: ok (28 files)
check-repository-identity: ok
exit: 0
$ git diff --check
exit: 0
$ git diff -- <eight historical tracked paths> | sha256sum
3fdb219654405e6fd40b5c0d1b02b94c04fadef5aa57a139aa5fb8fd6db7777e  -
exit: 0
```

`git blame` ties every emitted Clippy warning to pre-Wave12 lines (including
the shifted legacy LMDB graph helpers); new/touched Wave12 logic adds no new
warning class. No commit SHA is self-recorded here.
