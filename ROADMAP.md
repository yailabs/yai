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

## Stage 1 — typed transition authority

The next task is `YAI.SOURCE.REFOUNDATION.2`. It must begin with a compatibility
corpus and transaction design, then implement the smallest storage vertical
that stops new target semantics from depending on `Record.summary`.

Required boundary:

- define minimum typed Transition identity, Case generation, chronology,
  cause, Scope, actor, Decision, Attempt, observations/receipts, and outcome;
- retain Record as a versioned serialization envelope;
- select and implement one canonical Committed Transition Ledger with
  atomically maintained materialized CaseState;
- define rebuild/checkpoint rules for current state and derived graph, index,
  memory, and analytics;
- import existing JSONL/LMDB facts with explicit provenance and compatibility
  readers, without promoting parsed summaries into target truth;
- prove linked closure, record completeness, replay equivalence, idempotency,
  corruption/fallback behavior, and migration failure recovery.

The database technology remains an implementation decision until measured
against those requirements. Existing durable data must not be silently
discarded.

## Stage 2 — admission and uncertain external effects

After typed transition authority exists, make each product-reachable carrier
consume an ExecutionGrant and retire the direct filesystem bypass. Implement:

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
clone YVEX, create a directory per concept, or require ContextDelta. Stage 1
must not opportunistically implement the external-effect protocol or final
context model before their prerequisites exist.

## Exit criteria for the next source task

`YAI.SOURCE.REFOUNDATION.2` is complete only when one typed transactional
transition vertical coexists safely with, or migrates, the characterized
legacy corpus; replay and materialized-state equivalence are proved; and no
new target semantic field is recovered by parsing arbitrary summary text.
