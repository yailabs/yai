# Wave 19 execution evidence

All blocks are bounded excerpts from one named run; stdout/stderr from different
runs is not merged into a causal proof. The working directory for every run was
`/home/mothx/computer-science/projects/YAI/yai`.

## W19-E01 — focused memory-index contract

- run ID: `W19-E01-20260903`
- order: 1
- environment: repository default; no provider network
- Tenant/Case/Participant/provider/model: deterministic in-test fixtures
- material pre-state: uncommitted W19 implementation; no external index state
- command: `cargo test --manifest-path engine/Cargo.toml -p yai-engine memory_index -- --nocapture`
- exit: 0
- produced identifiers: content-addressed fixture IDs asserted internally
- invariant: representation/profile identity, BM25, exact vector, hybrid,
  isolation, corruption, crash, concurrent rebuild and profile replacement

```text
running 15 tests
test memory_index::tests::concurrent_rebuild_child ... ignored
test memory_index::tests::memory_index_scale_characterization ... ignored
test memory_index::tests::derived_store_rejects_intermediate_symlink_without_external_write ... ok
test memory_index::tests::representation_and_profile_identity_are_deterministic_and_content_addressed ... ok
test memory_index::tests::exact_vector_scan_rejects_invalid_vectors_and_has_stable_ties ... ok
test memory_index::tests::representation_scrubs_sensitive_tokens_and_bounds_canonical_input ... ok
test memory_index::tests::bm25_and_hybrid_fusion_are_query_sensitive_and_visibility_safe ... ok
test memory_index::tests::hybrid_fixture_exposes_distinct_lexical_vector_and_exact_causal_ranks ... ok
test memory_index::tests::profile_replacement_never_compares_incompatible_vectors ... ok
test memory_index::tests::corruption_is_detected_and_drop_preserves_source_material ... ok
test memory_index::tests::stale_and_cross_case_candidates_fail_closed ... ok
test memory_index::tests::provider_claim_similarity_never_changes_authority_posture ... ok
test memory_index::tests::profile_namespaces_are_independent_and_deletable ... ok
test memory_index::tests::build_crash_matrix_never_publishes_partial_index ... ok
test memory_index::tests::eight_process_concurrent_rebuilds_publish_one_equivalent_manifest ... ok
test result: ok. 13 passed; 0 failed; 2 ignored
```

## W19-E02 — CLI embedding response boundary

- run ID: `W19-E02-20260903`
- order: 2
- environment: repository default; no socket
- Tenant/Case/Participant/provider/model: parser fixtures only
- material pre-state: W19 v3 provider qualification implementation
- command: `cargo test --manifest-path cmd/yai/Cargo.toml memory_cli::tests -- --nocapture`
- exit: 0
- invariant: exact count/index/model/dimension and finite-value response checking

```text
running 1 test
test command_adapters::memory_cli::tests::embedding_response_requires_exact_count_dimension_model_and_finite_values ... ok
test result: ok. 1 passed; 0 failed
```

## W19-E03 — scale characterization

- run ID: `W19-E03-20260903`
- order: 3
- environment: deterministic test encoder; 8 dimensions; release claims disabled
- Tenant/Case/Participant: synthetic Case-local documents
- provider target/model: test-only fixture encoder, not YVEX/DeepSeek
- material pre-state: no persisted product index
- command: `cargo test --manifest-path engine/yai-engine/Cargo.toml memory_index_scale_characterization -- --ignored --nocapture`
- exit: 0
- invariant: bounded 1k/10k/50k representation, BM25, exact scan, full
  qualification/fusion retrieval and storage characterization

```text
memory_scale entries=1000 representation_ms=156 lexical_build_ms=247 fixture_embedding_build_ms=190 exact_query_us=610 lexical_query_us=3756 hybrid_query_us=754754 exact_hits=32 lexical_hits=32 hybrid_hits=32 serialized_bytes=4602183 peak_memory=not_observed ann=deferred
memory_scale entries=10000 representation_ms=1529 lexical_build_ms=2553 fixture_embedding_build_ms=1915 exact_query_us=7374 lexical_query_us=48183 hybrid_query_us=7767639 exact_hits=32 lexical_hits=32 hybrid_hits=32 serialized_bytes=46091258 peak_memory=not_observed ann=deferred
memory_scale entries=50000 representation_ms=7745 lexical_build_ms=13298 fixture_embedding_build_ms=9776 exact_query_us=43163 lexical_query_us=326008 hybrid_query_us=39411645 exact_hits=32 lexical_hits=32 hybrid_hits=32 serialized_bytes=226727138 peak_memory=not_observed ann=deferred
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 228 filtered out
```

## W19-E04 — real product path with loopback fixtures

- run ID: `W19-E04-20260903`
- order: 4
- environment: fresh temporary `YAI_HOME`; local chat and embedding fixtures
- Tenant: `tenant:w19-smoke`
- Case: `case:w19-memory`
- Participant: `participant:model`
- provider target/model: content-addressed ephemeral target / `memory-cognition-fixture`
- encoder/model: content-addressed ephemeral loopback target / `memory-fixture-encoder`
- material pre-state: clean temporary YAI_HOME and empty Resource root
- corpus/profile/index: emitted by this exact run below
- retrieval: v2 identity asserted and persisted within the temporary run
- command: `tests/characterization/memory-representation/test_memory_representation.sh`
- exit: 0
- invariant: real Effect + memory derive + build/search/drop/rebuild + cross-Case
  isolation + current-index runtime context

```text
memory_representation_characterization: pass
corpus_profile_index: memory-profile:35ad9c6fba62489fa71f4d3aef02aaff memory-index:432c4f7a56bd1f8a97bc62b3a462588d
qualified_planes: exact_operational lexical_bm25 vector_exact_cosine
ann_posture: deferred_exact_scan_within_bound
cross_case_isolation: true
drop_preserved_case_truth: true
content_identical_rebuild: true
runtime_context_used_current_w19_index: true
```

## W19-E05 — complete engine and CLI suites

- run ID: `W19-E05-20260903`
- order: 5
- environment: repository test environment; loopback-required CLI unit tests ignored
- material pre-state: full W19 tree
- commands: `cargo test --manifest-path engine/Cargo.toml --workspace` then
  `cargo test --manifest-path cmd/yai/Cargo.toml`
- exits: 0, 0
- invariant: all H1–H18 executable regressions retained

```text
test result: ok. 227 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
test result: ok. 28 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
```

Further publication qualification blocks are appended only from their own
later runs; no final commit SHA is asserted here before publication.

## W19-E06 — full repository check

- run ID: `W19-E06-20260903`
- order: 6
- environment: repository qualification environment; loopback fixture sockets
  explicitly available; no external provider
- Tenant/Case/Participant/provider/model: suite-owned isolated fixtures
- material pre-state: full uncommitted W19 tree after focused qualification
- command: `make check`
- exit: 0
- invariant: layout, docs, build, every current smoke through H18 and the W19
  product path pass together

```text
test result: ok. 227 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
test result: ok. 28 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
{"handler_failures": 0, "help_failures": 0, "operation_count": 152, "registry_digest": "sha256:34cc09dbec03d809cbdb7ed2be593369c75043973b0eb418d5701bbb159ae547", "schema": "yai.cli.command_discovery.v1", "visibility_counts": {"advanced": 10, "compatibility": 16, "plumbing": 45, "product": 80, "removed": 1}}
memory_representation_characterization: pass
cross_case_isolation: true
drop_preserved_case_truth: true
content_identical_rebuild: true
runtime_context_used_current_w19_index: true
```

## W19-E07 — full characterization target

- run ID: `W19-E07-20260903`
- order: 7
- environment: repository characterization environment with explicit loopback
  socket permission; no YVEX endpoint configured
- Tenant/Case/Participant/provider/model: suite-owned isolated fixtures
- material pre-state: the same full W19 tree qualified by W19-E06
- command: `make characterization`
- exit: 0
- invariant: the independent characterization target retains all lower-wave
  behavioral evidence and includes the W19 derived-index product path

```text
test result: ok. 227 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
test result: ok. 28 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
provider_governance_hardening_characterization: pass
memory_representation_characterization: pass
qualified_planes: exact_operational lexical_bm25 vector_exact_cosine
ann_posture: deferred_exact_scan_within_bound
cross_case_isolation: true
drop_preserved_case_truth: true
content_identical_rebuild: true
runtime_context_used_current_w19_index: true
```

## W19-E08 — static, registry and documentation closure

- run ID: `W19-E08-20260903`
- order: 8
- environment: `CARGO_TARGET_DIR=target` for both Clippy commands; repository
  default for all other commands
- Tenant/Case/Participant/provider/model: not applicable
- material pre-state: qualification-complete W19 tree; nested session-generated
  Cargo targets removed
- commands, in order:
  1. `cargo fmt --manifest-path engine/Cargo.toml --all -- --check`
  2. `cargo fmt --manifest-path cmd/yai/Cargo.toml --all -- --check`
  3. `CARGO_TARGET_DIR=target cargo clippy --manifest-path engine/Cargo.toml --workspace --all-targets --all-features`
  4. `CARGO_TARGET_DIR=target cargo clippy --manifest-path cmd/yai/Cargo.toml --all-targets --all-features`
  5. `make check-layout`
  6. `make check-docs`
  7. `python3 tests/characterization/cli-product-surface/audit_registry.py --binary ./target/debug/yai`
  8. `awk '/^```bash$/{p=1;next} /^```$/{if(p) exit} p' refoundation/foundation-recovery/wave-19/MANUAL-ACCEPTANCE.md | bash -n`
  9. `awk -F '\t' 'FNR==1 { expected=NF } NF!=expected { bad=1 } END { if (bad) exit 1 }' refoundation/foundation-recovery/wave-19/*.tsv`
  10. `awk -F '\t' 'FNR==1 { expected=NF } NF!=expected { bad=1 } END { if (bad) exit 1 }' refoundation/foundation-recovery/FOUNDATION-RECOVERY-LEDGER.tsv`
  11. `git diff --check`
  12. `git diff --exit-code -- README.md`
- exits: 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
- invariant: formatting, admitted warning baseline, repository layout, canonical
  docs, complete CLI registry, exact manual Bash syntax, TSV shape, whitespace,
  and the root README boundary all pass

```text
warning: `yai-engine` (lib) generated 12 warnings
warning: `yai` (bin "yai") generated 13 warnings
check-source-placement: ok
check-source-surface-clean: ok
check-doc-links: ok (30 files)
{"handler_failures": 0, "help_failures": 0, "operation_count": 152, "registry_digest": "sha256:34cc09dbec03d809cbdb7ed2be593369c75043973b0eb418d5701bbb159ae547", "schema": "yai.cli.command_discovery.v1", "visibility_counts": {"advanced": 10, "compatibility": 16, "plumbing": 45, "product": 80, "removed": 1}}
```

## W19-E09 — derived-store scope binding regression

- run ID: `W19-E09-20260903`
- order: 9
- environment: central repository Cargo target; no provider network
- Tenant/Case/Participant/provider/model: deterministic in-test fixtures
- material pre-state: final read-side symlink hardening plus explicit
  publication and automatic-profile Tenant/Case binding
- command: `CARGO_TARGET_DIR=target cargo test --manifest-path engine/Cargo.toml -p yai-engine memory_index -- --nocapture`
- exit: 0
- invariant: a bundle cannot publish through a foreign Tenant/Case/profile lock;
  corrupted automatic-profile pointers fail closed; eight concurrent builders
  still converge

```text
running 16 tests
test memory_index::tests::publication_and_automatic_profile_selection_enforce_tenant_case_scope ... ok
test memory_index::tests::eight_process_concurrent_rebuilds_publish_one_equivalent_manifest ... ok
test result: ok. 14 passed; 0 failed; 2 ignored; 0 measured; 214 filtered out
```

## W19-E10 — definitive full repository check

- run ID: `W19-E10-20260903`
- order: 10
- environment: repository qualification environment; test-only loopback
  fixtures; no external provider
- Tenant/Case/Participant/provider/model: suite-owned isolated fixtures
- material pre-state: final W19 implementation including publication and
  automatic-profile scope binding
- command: `make check`
- exit: 0
- invariant: layout, documentation, engine/CLI suites, all H1–H18 smokes and
  the complete W19 product path pass on the definitive tree

```text
test result: ok. 228 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
test result: ok. 28 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
{"handler_failures": 0, "help_failures": 0, "operation_count": 152, "registry_digest": "sha256:34cc09dbec03d809cbdb7ed2be593369c75043973b0eb418d5701bbb159ae547", "schema": "yai.cli.command_discovery.v1", "visibility_counts": {"advanced": 10, "compatibility": 16, "plumbing": 45, "product": 80, "removed": 1}}
memory_representation_characterization: pass
cross_case_isolation: true
drop_preserved_case_truth: true
content_identical_rebuild: true
runtime_context_used_current_w19_index: true
```

## W19-E11 — definitive full characterization

- run ID: `W19-E11-20260903`
- order: 11
- environment: explicit loopback socket permission; no YVEX endpoint
- Tenant/Case/Participant/provider/model: suite-owned isolated fixtures
- material pre-state: exact final W19 source used by W19-E10
- command: `make characterization`
- exit: 0
- invariant: independent characterization retains every lower-wave contract and
  the W19 real CLI path after final scope hardening

```text
test result: ok. 228 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
test result: ok. 28 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
provider_governance_hardening_characterization: pass
memory_representation_characterization: pass
qualified_planes: exact_operational lexical_bm25 vector_exact_cosine
ann_posture: deferred_exact_scan_within_bound
cross_case_isolation: true
drop_preserved_case_truth: true
content_identical_rebuild: true
runtime_context_used_current_w19_index: true
```

## W19-E12 — definitive static and staged closure

- run ID: `W19-E12-20260903`
- order: 12
- environment: central `CARGO_TARGET_DIR=target`; repository default otherwise
- Tenant/Case/Participant/provider/model: not applicable
- material pre-state: definitive tests complete; nested suite-generated Cargo
  targets removed
- commands: both Cargo fmt checks; both full-feature Clippy contracts; layout;
  docs; CLI registry audit; manual Bash syntax; Wave/ledger TSV shape;
  `git diff --check`; root README diff
- exits: all 0
- invariant: publishable formatting, warning baseline, layout, docs, registry,
  executable manual syntax, evidence shape, whitespace and README boundary

```text
warning: `yai-engine` (lib) generated 12 warnings
warning: `yai` (bin "yai") generated 13 warnings
check-source-placement: ok
check-source-surface-clean: ok
check-doc-links: ok (30 files)
{"handler_failures": 0, "help_failures": 0, "operation_count": 152, "registry_digest": "sha256:34cc09dbec03d809cbdb7ed2be593369c75043973b0eb418d5701bbb159ae547", "schema": "yai.cli.command_discovery.v1", "visibility_counts": {"advanced": 10, "compatibility": 16, "plumbing": 45, "product": 80, "removed": 1}}
```

## W19-E13 — semantic publication

- run ID: `W19-E13-20260903`
- order: 13
- environment: repository `master`; origin `https://github.com/yailabs/yai.git`
- Tenant/Case/Participant/provider/model: not applicable
- material pre-state: clean semantic commit based on
  `b47a4261484d2cdbfd11836156b7a21da280efce`
- commands: `git commit -m "feat: add derived hybrid memory indexing"`;
  `git push origin master`; `git rev-parse HEAD`; `git rev-parse origin/master`;
  `git ls-remote origin refs/heads/master`
- exits: all 0; the first restricted `ls-remote` DNS attempt was followed by
  the recorded network-enabled read below
- produced semantic SHA: `34582f36fbe0093b6c9d2a60d5c00233ba104236`
- invariant: semantic W19 commit is published and local/tracking/remote refs are
  exactly equal before this evidence-only closure commit

```text
[master 34582f3] feat: add derived hybrid memory indexing
To https://github.com/yailabs/yai.git
   b47a426..34582f3  master -> master
34582f36fbe0093b6c9d2a60d5c00233ba104236
34582f36fbe0093b6c9d2a60d5c00233ba104236
34582f36fbe0093b6c9d2a60d5c00233ba104236 refs/heads/master
```
