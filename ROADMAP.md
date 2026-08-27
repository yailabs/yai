# YAI implementation refoundation roadmap

Authority: implementation delta between [the constitution](docs/constitution.md)
and [the current executable architecture](docs/architecture.md). A named
concept or stage does not automatically justify a source subsystem.

## Objective

Establish one characterized vertical in which an admitted operation changes
canonical operational state through explicit transition authority, while the
same Case continues semantically across provider/runtime replacement.

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

## Stage 2 — admission and uncertain external effects

The exact next task is `YAI.SOURCE.REFOUNDATION.3`: build one narrow typed
admission/effect vertical on the new authority. It must make the selected
product-reachable filesystem carrier consume an ExecutionGrant and then retire
the direct filesystem bypass only after equivalent behavior and recovery are
proved. Implement:

- durable `PREPARED` intent before invocation;
- expected Resource generation/pre-state and stable idempotency identity;
- typed Attempt and EffectReceipt outcomes;
- `FINALIZED` and first-class `INDETERMINATE` states;
- restart enumeration and reconciliation;
- explicit denial, failure, no-effect, and internal-only transitions;
- one reachable carrier boundary shared by CLI/daemon entry paths.

Success requires crash/lost-reply tests. A happy-path write is insufficient.
The characterized C observation/write mechanics may be reused only after the
property is assigned to the surviving product resource boundary.

## Stage 3 — first constitutional model-to-effect vertical

Deliver one narrow end-to-end Case path:

```text
qualified CaseState
→ Projection + provider-independent ContextFrame
→ provider Invocation + typed ProviderResult
→ interpreted OperationCandidate
→ Decision + ExecutionGrant
→ PREPARE / filesystem carrier / FINALIZE
→ committed Transition + materialized CaseState
→ rebuilt next Projection
```

Required proofs include denial, provider failure, crash between PREPARE and
FINALIZE, ambiguous effect reconciliation, replay equivalence, provider
replacement with the same semantic Case, and loss of provider continuation
without loss of correctness.

## Stage 4 — derived state and context efficiency

Only after transition authority is stable:

- rebuild graph/index/memory/analytics from qualified canonical inputs;
- implement Projection lineage and invalidation;
- introduce Residency as a replaceable policy with measured semantic budgets;
- introduce provider-independent ContextFrame and versioned rendering;
- add optional ProviderContinuationReference validation;
- evaluate ContextDelta only when a consumer proves equivalence to a full
  destination-frame rebuild.

The context-residency lab is research evidence, not implementation authority.
Token/KV optimization cannot become Case memory.

## Explicit non-goals

This roadmap does not introduce Space or Agent as owners, import `yai-dev`,
clone YVEX, create a directory per concept, or require ContextDelta. Later
stages must not opportunistically implement the final context model before its
prerequisites exist.

## Exit criteria for the next source task

`YAI.SOURCE.REFOUNDATION.3` is complete only when one real filesystem path has
typed Operation/Decision/ExecutionGrant admission, durable PREPARE before the
carrier boundary, exact Observation/EffectReceipt closure, FINALIZED and
INDETERMINATE outcomes, restart reconciliation, and replay-equivalent
CaseState—without making a provider result or external acknowledgement
authoritative by itself.
