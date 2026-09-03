# H19 execution evidence

All excerpts below are bounded and copied from the identified run. No output
from different runs is combined as one causal proof. Provider secrets were not
present and are never rendered.

## H19-E01 — baseline and publication pre-state

- run ID: `h19-baseline-20260903`
- order: 1
- cwd: `/home/mothx/computer-science/projects/YAI/yai`
- environment: repository shell; no YAI_HOME mutation
- command set: `git branch --show-current`; `git rev-parse HEAD`; `git
  rev-parse origin/master`; `git status --short`; `git diff --stat`; `git diff
  --cached --stat`; `git ls-remote origin refs/heads/master`
- exit: 0
- raw material output:

```text
master
95730f1a11025491a3f0f7ccec0aba8b1d3f036b
95730f1a11025491a3f0f7ccec0aba8b1d3f036b
95730f1a11025491a3f0f7ccec0aba8b1d3f036b	refs/heads/master
```

- invariant: clean exact published W19 baseline, semantic commit
  `34582f36fbe0093b6c9d2a60d5c00233ba104236` in its direct chain.

## H19-E02 — remote YVEX freshness

- run ID: `h19-yvex-freshness-20260903`
- order: 2
- cwd: repository root; inspection clone `/tmp/yvex-h19-remote.A4MJl1/repo`
- environment: read-only YVEX inspection; no provider endpoint
- exact command: `git ls-remote https://github.com/yailabs/yvex.git
  refs/heads/models1`
- exit: 0
- raw output:

```text
e5c77060daae458f4b22664b910ed3582ec8a2a0	refs/heads/models1
```

- local pre-state: HEAD `5b3aa34be8999ad8240403e884074833d80c301d`,
  `origin/models1` `1f7ff1cd11ab8aec0976a9c8b0ee88ac5c73f010`.
- inspected at remote SHA: `docs/openai-compatibility.md`,
  `src/server/openai/core.c`, `src/server/openai/json.c`,
  `src/provider/core.c`, `tests/integration/openai.sh`.
- invariant: current remote, not a stale local reference; profile v2 supports
  chat/responses/structured function calls and refuses embeddings.

## H19-E03 — focused adversarial core and CLI

- run ID: `h19-focused-20260903-01`
- order: 3
- cwd: repository root
- environment: deterministic fixture-only; no Tenant/Case product store
- commands: `cargo test --manifest-path engine/yai-engine/Cargo.toml
  memory_index::tests::h19_ -- --test-threads=1`; `cargo test --manifest-path
  cmd/yai/Cargo.toml memory_cli::tests::h19_ -- --test-threads=1`
- exits: 0, 0
- raw summaries:

```text
test result: ok. 23 passed; 0 failed; 1 ignored; 0 measured; 228 filtered out; finished in 18.39s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 30 filtered out; finished in 0.00s
```

- invariant: H19-S01..S24 path, source, snapshot, ranking, bounds, crash,
  concurrency, authority, and encoder-response/governance contracts pass.

## H19-E04 — real registry-backed fixture product path

- run ID: unified exec session `85433`
- order: 4
- cwd: repository root
- environment: fresh `mktemp` YAI_HOME; Tenant `tenant:w19-smoke`; Case
  `case:w19-memory`; Participant `participant:model`; loopback cognition and
  deterministic test-only embedding fixtures; profile
  `memory-profile:5173ce2c67ac2f7a92f76c27a884181b`; index
  `memory-index:062b23682ce11ffaf56438b4512074cc`
- exact command: `make smoke-memory-representation`
- exit: 0
- bounded raw output:

```text
test result: ok. 249 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 91.48s
test result: ok. 33 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.06s
memory_representation_characterization: pass
corpus_profile_index: memory-profile:5173ce2c67ac2f7a92f76c27a884181b memory-index:062b23682ce11ffaf56438b4512074cc
qualified_planes: exact_operational lexical_bm25 vector_exact_cosine
ann_posture: deferred_exact_scan_within_bound
cross_case_isolation: true
drop_preserved_case_truth: true
content_identical_rebuild: true
runtime_context_used_current_w19_index: true
physical_store: yai.derived_memory_store.v2
deep_source_verify: true
concurrent_rebuilders: 32
duplicate_embedding_requests: 0
```

- invariant: one real Case product path builds, verifies, searches, drops,
  rebuilds, enters existing Projection/ContextFrame, enforces cross-Case
  isolation, and admits external encoder work once under 32 rebuild callers.

## H19-E05 — full phase profile

- run ID: unified exec session `2038`
- order: 5
- cwd: repository root
- environment: release; deterministic fixture encoder; synthetic derived
  Case `case:memory-index`; no network
- exact command: `cargo test --release --manifest-path
  engine/yai-engine/Cargo.toml memory_index::tests::memory_index_scale_characterization
  -- --ignored --nocapture --test-threads=1`
- exit: 0
- bounded raw output:

```text
memory_scale entries=1000 dimension=8 representation_ms=10 lexical_build_ms=21 fixture_embedding_build_ms=12 physical_build_ms=110 storage_bytes=4592085 physical_load_ms=18 deep_validation_us=46663 load_validation_us=813 source_qualification_us=568 source_revalidation_us=9275 bm25_us=989 exact_cosine_us=172 hybrid_cold_us=21845 hybrid_warm_us=21753 exact_reference_us=147 lexical_reference_us=1146 exact_hits=32 lexical_hits=32 qualified=1000 hybrid_hits=32 warm_hits=32 peak_memory=not_observed ann=deferred
memory_scale entries=10000 dimension=8 representation_ms=97 lexical_build_ms=218 fixture_embedding_build_ms=127 physical_build_ms=1272 storage_bytes=45982164 physical_load_ms=208 deep_validation_us=586180 load_validation_us=12606 source_qualification_us=9295 source_revalidation_us=106182 bm25_us=19374 exact_cosine_us=2451 hybrid_cold_us=236284 hybrid_warm_us=239587 exact_reference_us=2285 lexical_reference_us=13525 exact_hits=32 lexical_hits=32 qualified=10000 hybrid_hits=32 warm_hits=32 peak_memory=not_observed ann=deferred
memory_scale entries=50000 dimension=8 representation_ms=516 lexical_build_ms=1054 fixture_embedding_build_ms=639 physical_build_ms=6411 storage_bytes=226178047 physical_load_ms=937 deep_validation_us=2767752 load_validation_us=46553 source_qualification_us=32913 source_revalidation_us=486823 bm25_us=88464 exact_cosine_us=12342 hybrid_cold_us=1182221 hybrid_warm_us=1179175 exact_reference_us=16288 lexical_reference_us=110700 exact_hits=32 lexical_hits=32 qualified=50000 hybrid_hits=32 warm_hits=32 peak_memory=not_observed ann=deferred
```

- invariant: W19 39.4 s query pathology is removed; deep verification remains
  explicit and is no longer paid on the query hot path.

## H19-E06 — realistic vector dimensions

- run ID: unified exec session `2038`, second command
- order: 6
- cwd/environment: same independent release process as E05; controlled vectors
- exact command: `cargo test --release --manifest-path
  engine/yai-engine/Cargo.toml
  memory_index::tests::h19_realistic_vector_dimension_characterization --
  --ignored --nocapture --test-threads=1`
- exit: 0
- bounded raw output:

```text
memory_vector_scale entries=10000 dimension=1536 posture=admitted fixture_encode_ms=3 vector_build_ms=586 exact_query_us=18881 raw_vector_bytes=61440000 hits=32 peak_rss=VmHWM: __305808_kB
memory_vector_scale entries=50000 dimension=384 posture=admitted fixture_encode_ms=17 vector_build_ms=1226 exact_query_us=36881 raw_vector_bytes=76800000 hits=32 peak_rss=VmHWM: __525904_kB
memory_vector_scale entries=50000 dimension=768 posture=refused reason=memory_index_vector_element_budget_exceeded max_elements=25000000
test result: ok. 1 passed; 0 failed
```

- invariant: useful dimensions are characterized and unsafe document x
  dimension products are refused before allocation.

## H19-E07 — CLI registry

- run ID: `h19-registry-20260903-01`
- order: 7
- cwd: repository root
- exact command: `python3
  tests/characterization/cli-product-surface/audit_registry.py --binary ./yai`
- exit: 0
- raw output:

```json
{"handler_failures": 0, "help_failures": 0, "operation_count": 153, "registry_digest": "sha256:fdaaf6dd2cd0460c7b2ab4e4d94da18d81647298aacc53814cf1ce5b0bcd2925", "schema": "yai.cli.command_discovery.v1", "visibility_counts": {"advanced": 10, "compatibility": 16, "plumbing": 45, "product": 81, "removed": 1}}
```

- invariant: `yai.case.memory.index.verify` is registered PRODUCT/read-only,
  help reachable, structured, and handler-backed.

## H19-E08 — live acceptance dependency posture

- run ID: `h19-live-preflight-20260903`
- order: 8
- cwd: repository root
- environment: operator environment; values never printed
- exit: 0
- raw output:

```text
YAI_EXTERNAL_PROVIDER_BASE_URL=missing
YAI_EXTERNAL_PROVIDER_MODEL=missing
YAI_MEMORY_ENCODER_BASE_URL=missing
YAI_MEMORY_ENCODER_MODEL=missing
YAI_MEMORY_ENCODER_REVISION=missing
YAI_MEMORY_ENCODER_DIMENSION=missing
```

- invariant: live YVEX+DeepSeek and real loopback encoder acceptance cannot be
  truthfully run; exact script remains published in `MANUAL-ACCEPTANCE.md`.

## H19-E09 — complete repository check

- run ID: unified exec session `60228`
- order: 9
- cwd: repository root
- environment: repository default qualification environment with loopback
  socket access; central `CARGO_TARGET_DIR=target`
- exact command: `make check`
- exit: 0
- bounded raw output:

```text
test result: ok. 250 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out
test result: ok. 33 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
memory_representation_characterization: pass
physical_store: yai.derived_memory_store.v2
deep_source_verify: true
concurrent_rebuilders: 32
duplicate_embedding_requests: 0
test result: ok. 24 passed; 0 failed; 1 ignored; 0 measured; 228 filtered out
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 30 filtered out
memory_index_hardening: pass
adversarial_matrix: H19-S01..H19-S24
authority_delta: zero
lmdb_delta: zero
```

- invariant: the full lower-wave regression, W19 product smoke, and named H19
  hardening matrix pass together.

## H19-E10 — complete characterization

- run ID: unified exec session `24148`
- order: 10
- cwd: repository root
- environment: repository characterization environment with loopback socket
  access; central `CARGO_TARGET_DIR=target`
- exact command: `make characterization`
- exit: 0
- bounded raw output:

```text
test result: ok. 250 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 88.96s
test result: ok. 33 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.06s
memory_representation_characterization: pass
physical_store: yai.derived_memory_store.v2
deep_source_verify: true
concurrent_rebuilders: 32
duplicate_embedding_requests: 0
test result: ok. 24 passed; 0 failed; 1 ignored; 0 measured; 228 filtered out; finished in 18.27s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 30 filtered out; finished in 0.00s
memory_index_hardening: pass
adversarial_matrix: H19-S01..H19-S24
authority_delta: zero
lmdb_delta: zero
```

- invariant: all characterization surfaces through H18/W19 and the new H19
  path pass in one causally distinct run.

## H19-E11 — static, registry, layout, and artifact closure

- run ID: `h19-static-20260903-02`
- order: 11
- cwd: repository root
- exact commands:
  - `cargo fmt --manifest-path engine/Cargo.toml --all -- --check`
  - `cargo fmt --manifest-path cmd/yai/Cargo.toml --all -- --check`
  - `cargo clippy --manifest-path engine/Cargo.toml --workspace --all-targets --all-features`
  - `cargo clippy --manifest-path cmd/yai/Cargo.toml --all-targets --all-features`
  - `make check-layout check-docs`
  - `python3 tests/characterization/cli-product-surface/audit_registry.py --binary ./yai`
  - `bash -n refoundation/foundation-recovery/hardening-19/MANUAL-ACCEPTANCE.md`
  - `git diff --check`
  - `git diff --exit-code -- README.md`
- exit: all 0
- bounded raw output:

```text
warning: `yai-engine` (lib) generated 12 warnings
warning: `yai` (bin "yai") generated 13 warnings
check-source-placement: ok
check-doc-links: ok (30 files)
{"handler_failures": 0, "help_failures": 0, "operation_count": 153, "registry_digest": "sha256:fdaaf6dd2cd0460c7b2ab4e4d94da18d81647298aacc53814cf1ce5b0bcd2925", "schema": "yai.cli.command_discovery.v1", "visibility_counts": {"advanced": 10, "compatibility": 16, "plumbing": 45, "product": 81, "removed": 1}}
```

Both format checks, both Bash syntax checks, TSV shape validation, diff hygiene,
and the root README guard emitted no output. Clippy warnings are the unchanged
repository baseline; no H19 source is named. Generated per-manifest target
directories were removed and layout was rerun successfully.

- invariant: source formatting, the repository Clippy contract, docs/layout,
  CLI registry, exact manual syntax, evidence shape, diff hygiene, and the root
  README boundary all pass before staging.

Publication identity is deliberately recorded by the post-commit handoff, not
reconstructed inside the commit that contains this file.

## H19-E12 — staged wave boundary inspection

- run ID: `h19-staged-diff-20260903-01`
- order: 12
- cwd: repository root
- exact commands:
  - `git status --short`
  - `git diff --cached --name-status`
  - `git diff --cached --stat`
  - `git diff --cached --check`
  - `git diff --cached --exit-code -- README.md`
- exit: all 0
- observed posture: 44 files, restricted to the memory-index implementation,
  CLI registry/adapter, deterministic fixtures/smokes, Make target, and the
  `hardening-19` dossier; no unstaged source change and no root README change.
- invariant: the isolated H19 whitelist is the complete staged wave, with no
  unrelated operator work or generated target directory included.
