# YAI implementation refoundation roadmap

Authority: implementation delta between [the constitution](docs/constitution.md)
and [the current executable architecture](docs/architecture.md). A named
concept or stage does not automatically justify a source subsystem.

## Objective

Preserve canonical transition/effect/context authority while replacing the
remaining legacy-derived memory/retrieval plane with provenance-bound derived
knowledge that can improve Projection selection without becoming truth.

## Completed boundary — executable reality convergence

`YAI.SOURCE.REFOUNDATION.1` characterized the current product verticals before
collapsing physical ownership. It:

- separated the 16-source production C archive from component-only C tests;
- removed the marker Rust FFI crate, smoke-only bridge, net/proto scaffold,
  synthetic carrier/registry families, inspection-only Case/lease views, and
  duplicate C graph/index/memory/reconcile implementations;
- retained and characterized C filesystem/process/observation mechanics with
  unique platform value;
- extracted provider, review, direct filesystem, replay, graph, and analytics
  behavior from the former 10,901-line `main.rs`;
- added real HTTP provider and direct filesystem bypass characterization;
- preserved the five demonstrated product verticals and `yaid` lifecycle;
- left JSONL/LMDB authority, Record schemas, and effect semantics unchanged.

The removed source protected no uncharacterized product behavior. Historical
E05/E07/V11 properties not implemented today are retained as regression
requirements in the source-refoundation evidence package, not restored as old
runtime directories.

## Completed boundary — typed transition authority

`YAI.SOURCE.REFOUNDATION.2` selected the existing LMDB environment after
testing it against the required transaction semantics. It implemented:

- `yai.transition.v1`, with global identity, Case identity, per-Case sequence,
  source, optional Scope, causal refs, typed payload, provenance, and
  presentation-only summary;
- `yai.case_state.v1`, atomically reduced with every ledger append and fully
  rebuildable from ordered Transitions;
- deterministic duplicate/stale-generation rejection, rollback-before-commit,
  restart, replay equivalence, materialization rebuild, persisted-version
  rejection, and derived failure isolation;
- typed Case/participant/provider/invocation/result/interpretation and fixed
  review request/resolution payloads as the minimum current consumer set;
- typed graph derivation and deterministic graph replacement from canonical
  transitions, with historical records routed through one compatibility
  decoder;
- a corpus covering all 35 Rust and 32 C legacy kinds, drift cases, both old
  schemas, optional/malformed/unknown input, repeated IDs, and old summary
  variants;
- inspect, dry-run, and isolated compatibility import that preserves unknown
  information opaquely and never creates canonical meaning;
- live provider and review consumers while retaining old JSONL/record output
  for compatibility.

LMDB is now physical storage for both canonical ledger and current
materialization, but only the Transition ledger is historical authority. The
old journal/record databases remain compatibility input/output and operator
evidence, not a second mutable canon.

## Completed boundary — first constitutional effect vertical

`YAI.SOURCE.REFOUNDATION.3` implemented one narrow end-to-end Case path:

```text
qualified CaseState
→ bounded controlled-effect projection
→ provider Invocation + typed ProviderResult
→ interpreted OperationCandidate
→ Decision + ExecutionGrant
→ PREPARE / filesystem carrier / FINALIZE
→ committed Transition + materialized CaseState
→ rebuilt next Projection
```

The provider result remains raw candidate material until an exact
`filesystem.write` proposal contract is normalized. ALLOW/DENY is deterministic
and owned by a bound non-model policy participant. Only ALLOW produces an
integrity-bound, generation-bound, one-time Grant. The Rust carrier requires a
materialized durable PREPARE, performs real pre/post observation and atomic
replacement, and finalizes only established outcomes.

Crash injection now covers after Grant/before PREPARE, after PREPARE/before
effect, after visible effect/before observation, and after receipt construction/
before FINALIZE. Explicit restart reconciliation concludes effect observed, no
effect observed, conflict, or still indeterminate from real state. Duplicate
reconciliation does not duplicate the semantic effect. The direct Rust write
command is removed, review approval uses the same carrier, and the C daemon
filesystem fixture no longer performs or claims an effect.

The deterministic HTTP vertical performs a required second provider call. Its
view derives success or denial from typed Transition/CaseState consequence,
never from the first model's assertion. This proves the first constitutional
effect path, not a generalized carrier or final context architecture.

## Completed boundary — typed Projection and semantic continuity

`YAI.SOURCE.REFOUNDATION.4` implemented one provider-independent semantic
compiler used by ordinary prompts and controlled effects:

- `yai.projection.v1` binds Case generation, admitted participant/purpose,
  ordered typed entries, authority posture, provenance and bounded omissions;
- `yai.context_frame.v1` gives one invocation task/output contract identity
  independent of provider render, token sequence and continuation;
- `yai.rendered_input.v1` metadata identifies provider/model render and digest
  without retaining full rendered input;
- `yai.transition.v3` records provider/model/Projection/Frame/render/output-
  contract lineage and typed bounded interaction turns;
- new provider invocations no longer write or consume ParticipantViewFrame;
- optional opaque continuation is ephemeral and invalidation retries the full
  frame; its loss cannot change Case history or CaseState;
- deterministic HTTP proofs replace provider and model after a real filesystem
  effect, restart provider state, and preserve the observed consequence;
- participant visibility fails before rendering, indeterminate effects remain
  unresolved, false provider claims remain claims, and long history produces a
  bounded frame;
- the droppable semantic-context artifact store supports inspection but owns no
  canonical data.

Residency remains provisional and ContextDelta deferred because neither has an
independent current consumer. The context-residency lab remains research
evidence, not implementation authority.

## Stage 5 — provenance-bound operational memory and qualified retrieval

The exact next task is `YAI.SOURCE.REFOUNDATION.5 — Provenance-Bound Memory and
Qualified Retrieval`. It should:

- inventory the surviving legacy MemoryCandidate, graph/retrieval and DuckDB
  extractors and characterize their real consumers;
- define the smallest typed derived-memory entry with source Transition/
  Observation/Receipt provenance, Case/participant visibility, freshness and
  invalidation;
- migrate useful operational consequence/interaction retrieval away from
  summary parsing without making provider claims trusted memory;
- implement deterministic bounded query/selection with canonical-state fallback
  when graph or memory indexes are absent;
- connect the existing optional `DerivedMemoryInput` seam to one real,
  rebuildable producer and prove deleting/rebuilding it preserves CaseState and
  semantic correctness;
- remove the last active duplicate memory/projection authority after its
  behavior is migrated;
- keep learned compression, adaptive Residency, ContextDelta and analytics as
  authority out of scope unless a measured consumer supplies an independent
  lifecycle.

## Explicit non-goals

This roadmap does not introduce Space or Agent as owners, import `yai-dev`,
clone YVEX, create a directory per concept, or require ContextDelta. Later
stages must not opportunistically implement the final context model before its
prerequisites exist.

## Exit criteria for the next source task

`YAI.SOURCE.REFOUNDATION.5` is complete only when a typed, provenance-bearing,
rebuildable memory/retrieval result can influence Projection selection; loss of
every derived index leaves required semantic continuity intact; provider claims
cannot become operational memory without an admitted transition/consequence;
participant visibility is preserved through retrieval; and no summary string,
graph edge, analytics row or memory entry becomes canonical authority.
