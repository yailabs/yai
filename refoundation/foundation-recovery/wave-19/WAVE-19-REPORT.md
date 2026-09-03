# Source Refoundation Wave 19 report

## State and baseline

W19 implements derived long-horizon memory representation and hybrid retrieval
without changing canonical memory authority. The operator-published minimum was
`d314ea5f670090ffdb21eb6ebacae8367445ace4`; the clean starting `master` and
`origin/master` were both
`b47a4261484d2cdbfd11836156b7a21da280efce`. The sole intervening commit,
`b47a426 docs: reconcile roadmap after H18`, was inspected in full and was
compatible. The intended semantic commit is
`feat: add derived hybrid memory indexing`. This report deliberately does not
claim the SHA of the commit that contains it.

Live YVEX + DeepSeek acceptance is pending because both
`YAI_EXTERNAL_PROVIDER_BASE_URL` and `YAI_EXTERNAL_PROVIDER_MODEL` were absent.
Automated qualification is complete; publication will therefore use state
`complete_published_external_acceptance_pending`, not `complete_published`.

## Direct current audit

The executable pipeline was reinspected in `engine/yai-engine/src/transition.rs`,
`memory.rs`, `context.rs`, `residency.rs`, `store/lmdb.rs`, current graph code,
`provider_governance.rs`, `cmd/yai/src/provider.rs`, `case_runtime.rs`,
`memory_cli.rs`, and `cli/registry.rs`. Transition history remains canonical;
CaseState and OperationalMemory remain rebuildable materializations. Retrieval,
Projection, Residency, ContextFrame, provider rendering, and all W19 artifacts
remain derived. [current-memory-pipeline.tsv](current-memory-pipeline.tsv)
records each stage.

## Fresh legacy archaeology

The strongest complete pre-drain implementation was inspected directly at
`yai-dev` commit `8a2b09e268fe6e20b1681dab7b22eac6b8239a8c`, including every requested
retrieval file, memory/recall families, embeddings headers/implementations,
tests, and consumers. `cffb318b980456f2671a297e14a6b05f5ac68320`
provided relevant organization history; deletion commit
`2a4018147219044dfe1fad2268759b1f2a585945` established why wholesale plane
resurrection is rejected. `working.c` and `memory/rank.c` never existed in the
searched history; their headers and adjacent consumers were inspected instead.

Recovered properties are plane separation, bounded per-plane/total candidates,
deduplication, explicit embedding request/result seams, provenance intent, and
rank explanations. Rejected mechanisms are mutable global arrays,
process-local truth, eight-dimensional byte-hash pseudo-embeddings, fake-vector
L2, global episodic arrays, `case://global`, wall-clock truth decay, arbitrary
salience, unversioned embedding references, Agent ownership, and vector-to-model
bypass. Exact file/commit verdicts are in
[direct-legacy-memory-reinspection.tsv](direct-legacy-memory-reinspection.tsv).

## Owner verdicts

Representation documents, embeddings, corpus/index manifests, BM25/vector
indexes, ANN, and RetrievalSet are `derived_no_owner`. The representation
profile is an immutable Tenant-bound runtime/admin configuration value shared
across Cases; persistence alone does not earn another semantic owner. W19 adds
zero semantic owners and zero LMDB databases. The existing count remains
37/40. Case Transition history remains the memory authority.

## Representation and encoder contracts

`yai.memory_representation_document.v1` binds one exact OperationalMemoryEntry,
its full SHA-256 source digest, derivation and representation versions, semantic
kind, authority posture, lifecycle, deterministic bounded canonical text,
provenance, and visibility. Canonical text is generated from typed value fields,
not mutable CLI descriptions. Provider claims remain claims. Sensitive token
shapes are scrubbed before encoding.

`yai.memory_representation_profile.v1` binds Tenant, representation contract,
ProviderTarget, exact model, operator-declared encoder revision, dimension,
float32 little-endian representation, L2 normalization, cosine metric, and the
same-profile query policy. Automatic Case-memory encoding accepts only a fresh,
approved, loopback ProviderTarget with an exactly qualified `TextEmbedding`
dimension. It uses the W18 provider transport/governance stack; no second
provider manager exists. `yai.memory_embedding.v1` is derived and never appears
in Transition, CaseState, normal product output, or ContextFrame.

## Derived indexing and lifecycle

`yai.memory_corpus_manifest.v1` binds exact Case generation, derivation and
representation contracts, ordered memory/document identities and lifecycle.
`yai.memory_index_manifest.v1` binds the corpus/profile digests, index types,
counts, dimension, build version and checksums. Any later Case generation is
stale. Profile replacement produces an independent namespace and vectors are
never reinterpreted or compared across profiles.

The disposable layout is
`$YAI_HOME/store/derived-memory/v1/<tenant-hash>/<case-hash>/profiles/<profile-hash>`.
Directories/files are private, components are digest-derived, symlinks are
refused at every existing path component, and publication is
lock/validate/seal/rename/atomic-pointer with fsync.
Eight process builders converge on one content identity. Drop removes only the
derived profile namespace and proves Transition count is unchanged.
Stable document identities make incremental reuse conceivable, but W19
deliberately rebuilds the complete bounded corpus/profile: partial publication
and real-encoder reproducibility do not yet have a sufficient contract.

The lexical plane is real bounded BM25 with deterministic tokenization,
matched-term evidence, and stable ties. The vector plane is an exact cosine
scan and permanent correctness oracle. It rejects empty, wrong-dimensional,
non-finite, zero-norm and oversized vectors. ANN/HNSW was consciously deferred:
at controlled 50k scale exact 8d scan took 43.163 ms, while admitting a new
serialization/corruption/dependency surface was not justified. The full
qualification-plus-three-plane hybrid path took 39.412 s at 50k and is recorded
as a performance limit, not hidden behind the much cheaper exact-vector number.
The manifests already expose an explicit ANN posture and allow a later
accelerator without authority change.

## Hybrid retrieval and context integration

`yai.retrieval_query_document.v1` separates task text, bounded retrieval input,
and provider-profile vector encoding. `yai.retrieval_set.v2` records query,
corpus/profile/index identities, availability, per-plane ranks and evidence,
public qualification rejections, RRF reasons, selected identities and omission.
Its content identity binds those full public ranks/reasons/results, not merely
the selected IDs. V1 retrieval remains implemented.

Hard Case, Participant/view, generation, lifecycle, kind, Resource, and causal
qualification runs before exact/BM25/vector ranking. Hidden items do not enter
results, reasons, or sensitive counts. Memory IDs deduplicate planes. Integer
Reciprocal Rank Fusion (`k=60`) combines ranks without pretending BM25 and
cosine scores share a scale; exact Resource/causal anchors sort before fuzzy-only
candidates. ProviderClaim posture and provenance survive unchanged.

The selected v2 entries are converted to the existing `DerivedProjectionInput`
and pass through the existing Projection v5 → ResidencyPlan v1 → ContextFrame
v5 → generic OpenAI-compatible render. No DeepSeek-specific ontology or context
schema change was required. ProviderSelection is itself a canonical Transition;
therefore runtime detects the resulting staleness and may atomically refresh the
configured profile using only its separately qualified loopback encoder before
compilation. Encoder failure leaves vector unavailable and falls back to
qualified operational retrieval.

## Crash, corruption, concurrency, replacement and scale

M19-C1 through M19-C6 prove no partial current index across corpus, lexical,
embedding, vector, temp-publish and post-publish acknowledgement boundaries.
Bundle/pointer/corpus/profile/item/dimension/postings/vector corruption fails
closed. Eight independent processes rebuild one exact corpus/profile without
corruption. Profile A/B query/delete/rebuild testing proves namespace
independence. Detailed matrices are versioned beside this report.

Observed synthetic 1k/10k/50k timings and bytes are in
[memory-scale.tsv](memory-scale.tsv). They are local characterization, not
marketing or semantic-quality evidence. Fixture vectors prove mechanics only;
the 50k hybrid result establishes that later acceleration/qualification work is
warranted even though ANN admission itself remains deferred from W19.

## Product surface and compatibility

W16 registry descriptors now expose product `case memory show/search/index
status/build/rebuild/retrieval show` and advanced `index drop`; product JSON is
typed and ANSI-free. An explicit product `case participant view admit` command
commits the already-existing ParticipantAdmitted Transition so a role cannot
silently imply model visibility. Runtime summaries expose Projection and
ContextFrame IDs for inspection. Root README is untouched.

ProviderQualification advances to v3 for exact embedding evidence while v1/v2
readers remain. Transition/CaseState, Projection and ContextFrame versions are
unchanged. Lower-wave Policy, Effect, ResourceFence, Workflow, PlanPatch,
Handoff, health/trust, and safe-failover boundaries remain authoritative.

## Governed capability delta and H19 boundary

[post-wave19-foundation-delta.tsv](post-wave19-foundation-delta.tsv) verifies
that `filesystem.write` and `process.signal` are the only model-usable typed
operation families. Read/list/stat/search/process observation are absent as
model capabilities; the existing read/process observation surfaces are
compatibility/plumbing only. This gap likely deserves a future Foundation wave,
but W19 implements none of it and recommends no generic shell capability.

H19 episodic/semantic consolidation, contradiction handling, decay/retention,
learned reranking and semantic promotion were not started. W19 preserves
conflicting derived memories with exact provenance and authority posture.

## Evidence and manual acceptance

[EXECUTION-EVIDENCE.md](EXECUTION-EVIDENCE.md) and
[FAILURE-EVIDENCE.md](FAILURE-EVIDENCE.md) retain run-specific bounded output.
[MANUAL-ACCEPTANCE.md](MANUAL-ACCEPTANCE.md) is the exact clean zero-to-use-case
YVEX + operator-selected DeepSeek + separately qualified loopback encoder path.
The external dependency state is recorded in
[yvex-deepseek-manual-acceptance.tsv](yvex-deepseek-manual-acceptance.tsv).
