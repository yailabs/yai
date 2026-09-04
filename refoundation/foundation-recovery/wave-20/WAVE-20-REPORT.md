# Wave 20 — Case-native episodic and evidence-bound semantic memory

State at report creation: `implemented_qualified_prepublication`.

Baseline: `de2ca50606cc4b38e0b45e96d66e1bd2a4cbb9b6` on `master`.
Intended semantic commit: `feat: add evidence-bound long-horizon memory`.
Live external posture: `complete_published_external_acceptance_pending` because
the six required provider/model variables were absent during qualification.

## Outcome

W20 adds deterministic Case-native Episodes, typed SemanticAssertions,
content-addressed consolidation inputs, strict provider-candidate normalization,
bounded support validation, structural contradictions, mechanical supersession,
generation-based retention, and a rebuildable hierarchy manifest. All remain
derived from canonical Transition history and recorded ProviderResults. No
episodic, semantic, hierarchy, contradiction, or working-memory owner and no
LMDB database were added.

The W19/H19 index accepts Operational, Episodic, and Semantic memory through
`MemoryRepresentationDocument v2` and emits `RetrievalSet v3`. Qualification is
still performed before fuzzy ranking, and selected content is re-resolved from
current sources. Projection and ContextFrame advance to v6 because memory
family, epistemic class, lifecycle, support provenance, and the explicit
`MemoryConsolidation` purpose change serialized meaning. v5 constants and v1/v2
retrieval readers remain present.

## Authority boundary

`Transition -> OperationalMemory -> Episode/SemanticAssertion -> representation
and index -> qualified RetrievalSet -> Projection -> ContextFrame` is one-way.
Provider output never selects epistemic class, creates an Observation, executes
a tool, or authorizes an Operation. Recorded consolidation ProviderResult bytes
are the durable source for deterministic normalization after restart; rebuild
does not invoke a provider.

## Bounds

Consolidation admits at most 16 Episodes, 32 OperationalMemory entries, 16
existing assertions, 64 output assertions, 16 supports per assertion, support
depth 16, and 256 KiB result bytes. Contradiction groups fail closed above 1024
members. Default active semantic retention is 4096 items and 128 recent
Episodes, while unresolved state and support ancestors are preferentially kept.
The combined H19 physical corpus remains capped at 50,000 documents.

## Compatibility and scope

The derived store remains `yai.derived_memory_store.v2`; ProviderQualification
remains v3; LMDB remains 37/40. H20 hardening, W21 learned reranking/compression,
and W22 governed capabilities were not started. The root `README.md` was not
modified.

## Post-publication manual-acceptance usability correction

Operator review correctly found that the first published acceptance artifact
was a Bash assertion harness rather than a natural CLI walkthrough. The
follow-up keeps exact target/artifact/profile IDs fully supported, while adding
fail-closed stable references for interactive use: Tenant-scoped provider
keys, `--principal self`, current published policy keys, the sole current
memory profile, `--episode latest`, and the latest Case Projection or
ContextFrame. Optional credential environment variables are now resolved by
`yai provider add` without exposing their values. The W20 smoke exercises the
same stable-reference route.

This correction changes no memory semantics, schema version, authority path,
owner, LMDB database, YVEX behavior, H20/W21 scope, or W22 capability surface.
