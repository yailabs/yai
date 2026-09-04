# W20 execution evidence

All runs used cwd
`/home/mothx/computer-science/projects/YAI/yai`. Output excerpts below are
bounded verbatim excerpts from the named run; they are not combined across
runs. Provider credentials were absent and are never recorded.

## E-W20-001 — repository and external pre-state

- run ID: `w20-preflight-20260904`
- order: 1
- command: `git branch --show-current`; `git rev-parse HEAD`;
  `git rev-parse origin/master`; `git status --short`; remote master check
- environment: ordinary repository environment; no isolated YAI_HOME
- initial Case/Tenant/Participant: none
- exit: 0
- invariant: clean master exactly at the published H19 baseline and remote
  equality before W20 edits

```text
master
de2ca50606cc4b38e0b45e96d66e1bd2a4cbb9b6
de2ca50606cc4b38e0b45e96d66e1bd2a4cbb9b6
```

## E-W20-002 — YVEX reference freshness

- run ID: `w20-yvex-reference-20260904`
- order: 2
- command: `git ls-remote <configured-yvex-origin> refs/heads/models1`
- read-only reference checkout: `/tmp/yvex-w20-readonly`
- exit: 0
- invariant: current remote ref, not a stale local/documented checkpoint

```text
c3f675d1213ce3a6d7387179bb22415775e40e37	refs/heads/models1
```

Exact inspected reference files include `docs/openai-compatibility.md` and the
server/provider compatibility sources it names. The profile is
`yvex.openai.compat.v2`: Chat Completions, Responses, `json_object`, and
bounded function calls are supported; embeddings explicitly refuse; YVEX
executes no application tool.

## E-W20-003 — focused hierarchy contracts

- run ID: `w20-focused-hierarchy-20260904`
- order: 3
- command: `cargo test --manifest-path engine/Cargo.toml memory_hierarchy::tests -- --nocapture`
- environment: deterministic in-process fixtures
- Case/Tenant/Participant: `case:test` / none / `participant:test`
- provider: fixture identities only; no dispatch
- exit: 0
- invariant: structural episode identity, epistemic separation, support
  qualification/cycles, exact deduplication, contradiction/supersession

```text
running 9 tests
test memory_hierarchy::tests::w20_s05_identity_deduplicates_repeated_provider_origin ... ok
test memory_hierarchy::tests::w20_k01_inference_conflict_is_unresolved_without_class_inflation ... ok
test memory_hierarchy::tests::w20_k02_k03_grounded_assertion_is_not_displaced_by_inference_or_claim ... ok
test memory_hierarchy::tests::w20_s08_support_cycle_rejected_but_shared_ancestor_is_valid ... ok
test memory_hierarchy::tests::w20_s04_provider_cannot_choose_epistemic_class ... ok
test memory_hierarchy::tests::w20_k04_k05_only_unique_newer_grounded_state_supersedes ... ok
test memory_hierarchy::tests::w20_e01_e02_e03_episode_identity_is_structural_and_provider_independent ... ok
test memory_hierarchy::tests::w20_s06_s07_cross_case_and_hidden_support_fail_closed ... ok
test memory_hierarchy::tests::w20_cn03_cn09_support_is_exact_and_duplicate_candidates_deduplicate ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 257 filtered out; finished in 0.00s
```

The later bound test also passed:

```text
running 1 test
test memory_hierarchy::tests::w20_support_depth_and_contradiction_storm_are_bounded ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 267 filtered out; finished in 0.02s
```

## E-W20-004 — realistic W20 hierarchy scale

- run ID: `w20-scale-release-20260904`
- order: 4
- command: `cargo test --release --manifest-path engine/Cargo.toml memory_hierarchy::tests::w20_episode_and_semantic_scale_characterization -- --ignored --nocapture`
- environment: optimized build, synthetic typed data, no provider/network
- exit: 0
- invariant: 10k Episode derivation and 50k assertion grouping are bounded on
  the qualification host; these are informational, not an SLA

```text
family	items	materialize_ms	group_ms	output_items
episode	100	0	0	100
episode	1000	0	5	1000
episode	10000	2	60	10000
semantic	1000	1	0	0
semantic	10000	11	2	0
semantic	50000	59	21	0
test memory_hierarchy::tests::w20_episode_and_semantic_scale_characterization ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 267 filtered out; finished in 0.18s
```

## E-W20-005 — W20 product smoke and full Rust suites

- run ID: `w20-smoke-product-20260904-b`
- order: 5
- command: `make smoke-episodic-semantic-memory`
- environment: fresh `mktemp` YAI_HOME; deterministic loopback cognition and
  encoder fixtures
- Tenant: `tenant:w20-smoke`
- Case: `case:w20-memory`; negative Case `case:w20-isolated`
- Participant: `participant:model`
- provider target/model: content-addressed local target /
  `memory-w20-cognition-fixture`
- encoder target/model/profile: content-addressed local target /
  `memory-w20-encoder` / fixture revision dimension 4
- exit: 0
- invariant: denied/write/replacement/provider-claim history, deterministic
  Episodes, governed consolidation, exact rebuild without provider reinference,
  typed contradiction, multi-family v3 retrieval, cross-Case isolation, index
  drop continuity, and final ContextFrame use

Rust suite excerpt:

```text
test result: ok. 264 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 92.11s

test result: ok. 34 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

Product smoke output:

```text
episodic_semantic_memory: pass
episode_schema: yai.memory_episode.v1
semantic_schema: yai.semantic_memory_assertion.v1
retrieval_schema: yai.retrieval_set.v3
consolidation_input: memory-consolidation-input:24a7c98c8993d675
consolidation_provider_result: provider-result:case:w20-memory:model-output-27
hierarchy_before_consolidation: memory-hierarchy:83de5b1fee0fcc0e
hierarchy_after_consolidation: memory-hierarchy:3e06bf484317c342
hierarchy_rebuild_exact: true
provider_reinference_on_rebuild: zero
cross_case_isolation: true
index_drop_preserved_hierarchy: true
```

The smoke's current generation, corpus/index/profile, RetrievalSet, Projection,
ContextFrame, and ProviderResult identities are produced by its product
commands; the bounded summary above retains the load-bearing IDs. Fixture
vectors qualify mechanics only, not semantic quality.

## E-W20-006 — manual artifact and dossier validation

- run ID: `w20-doc-validation-20260904`
- order: 6
- commands:
  - Bash syntax check of the exact fenced commands in `MANUAL-ACCEPTANCE.md`
  - rectangular TSV validation of every Wave 20 TSV
  - `git diff --check`
- environment: repository only
- exit: 0
- invariant: natural manual commands are syntactically valid and evidence TSVs
  are structurally valid

```text
wave20_tsv_validation: pass 37
```

## E-W20-007 — complete characterization

- run ID: `w20-characterization-20260904-c`
- order: 7
- command: `make characterization`
- environment: repository qualification environment; loopback fixtures only
- exit: 0
- invariant: all lower-Wave characterization plus H19 and W20 product smoke
  remain green after the multi-family ContextFrame compatibility correction

```text
test result: ok. 264 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out
test result: ok. 34 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
memory_index_hardening: pass
adversarial_matrix: H19-S01..H19-S24
episodic_semantic_memory: pass
hierarchy_rebuild_exact: true
provider_reinference_on_rebuild: zero
cross_case_isolation: true
index_drop_preserved_hierarchy: true
```

## E-W20-008 — complete repository check

- run ID: `w20-make-check-20260904-b`
- order: 8
- command: `make check`
- environment: repository qualification environment; loopback fixtures only
- exit: 0
- invariant: repository layout/docs, complete engine and CLI tests, every
  smoke through W20, registry audit, H19 adversarial matrix, and product
  characterization pass together

```text
check-source-placement: ok
check-source-surface-clean: ok
check-doc-links: ok (30 files)
test result: ok. 264 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out
test result: ok. 34 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
operation_count: 161
memory_index_hardening: pass
episodic_semantic_memory: pass
```

## E-W20-009 — format, Clippy, registry and artifact closure

- run ID: `w20-static-closure-20260904`
- order: 9
- commands:
  - `cargo fmt --manifest-path engine/Cargo.toml --all -- --check`
  - `cargo fmt --manifest-path cmd/yai/Cargo.toml --all -- --check`
  - `CARGO_TARGET_DIR=target cargo clippy --manifest-path engine/Cargo.toml --workspace --all-targets --all-features`
  - `CARGO_TARGET_DIR=target cargo clippy --manifest-path cmd/yai/Cargo.toml --all-targets --all-features`
  - Bash syntax validation, Python fixture compile, TSV validation,
    `git diff --check`, and root README guard
- exit: 0
- invariant: W20 adds no Clippy diagnostic, formatting drift, malformed test
  fixture, malformed TSV, whitespace error, or root README mutation; the
  repository's pre-existing default Clippy warnings remain unchanged

```text
Finished `dev` profile [unoptimized + debuginfo] target(s)
wave20_tsv_validation: pass 37
manual_acceptance_natural_commands: pass
fmt_diff_root_readme_guard: pass
```

## E-W20-010 — final repository check after retention closure

- run ID: `w20-make-check-20260904-final`
- order: 10
- command: `make check`
- environment: repository qualification environment; loopback fixtures only
- exit: 0
- invariant: the final source, including deterministic retention of assertion
  Episode ancestors, passes repository layout/docs, full engine and CLI suites,
  every smoke through W20, registry audit, and the complete H19 adversarial
  matrix

```text
test result: ok. 265 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 91.21s
test result: ok. 34 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.07s
memory_index_hardening: pass
adversarial_matrix: H19-S01..H19-S24
episodic_semantic_memory: pass
consolidation_input: memory-consolidation-input:78d9fe93249cd93e
hierarchy_after_consolidation: memory-hierarchy:ac0b09648a8558de
hierarchy_rebuild_exact: true
provider_reinference_on_rebuild: zero
cross_case_isolation: true
index_drop_preserved_hierarchy: true
```

## E-W20-011 — final characterization after retention closure

- run ID: `w20-characterization-20260904-final`
- order: 11
- command: `make characterization`
- environment: repository qualification environment; loopback fixtures only
- exit: 0
- invariant: the separately invoked final characterization passes on the same
  final source used by E-W20-010

```text
test result: ok. 265 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 91.00s
test result: ok. 34 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.07s
memory_representation_characterization: pass
physical_store: yai.derived_memory_store.v2
deep_source_verify: true
concurrent_rebuilders: 32
duplicate_embedding_requests: 0
episodic_semantic_memory: pass
consolidation_input: memory-consolidation-input:77d160e3c9141f88
hierarchy_after_consolidation: memory-hierarchy:5786fb9c507573f8
hierarchy_rebuild_exact: true
provider_reinference_on_rebuild: zero
cross_case_isolation: true
index_drop_preserved_hierarchy: true
```

## E-W20-012 — final static closure

- run ID: `w20-static-closure-20260904-final`
- order: 12
- commands:
  - `cargo fmt --manifest-path engine/Cargo.toml --all -- --check`
  - `CARGO_TARGET_DIR=/home/mothx/computer-science/projects/YAI/yai/target cargo clippy --manifest-path engine/Cargo.toml --workspace --all-targets`
  - `CARGO_TARGET_DIR=/home/mothx/computer-science/projects/YAI/yai/target cargo clippy --manifest-path cmd/yai/Cargo.toml --all-targets`
  - `make check-docs`
  - `git diff --check`
- exit: 0 for every command
- invariant: final formatting, default repository Clippy contract,
  documentation/layout, and whitespace integrity all pass; reported Clippy
  warnings are the repository's pre-existing admitted warnings

```text
check-doc-links: ok (30 files)
check-repository-identity: ok
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## Live external acceptance

At execution time all six required live provider/encoder variables were absent.
No live ProviderSelection, RetrievalSet, Projection, ContextFrame, or
ProviderResult ID is claimed. Classification:
`blocked_external_dependency` / `DEPLOYMENT_LIMITATION`. The exact runnable
operator sequence is retained in `MANUAL-ACCEPTANCE.md`.
