# YAI implementation refoundation roadmap

Authority: implementation delta between [the constitution](docs/constitution.md)
and [the current executable architecture](docs/architecture.md). Ordering here
does not claim that a named concept deserves a source subsystem.

## Refoundation objective

Establish one characterized vertical in which an admitted operation changes
canonical operational state through explicit transition authority, and in
which the same Case can continue semantically after provider/runtime
replacement. Preserve proven behavior before deleting or consolidating code.

## Stage 0 — characterize before deletion

Freeze regression specifications for behavior that currently exists only in
large or duplicated surfaces:

- Rust CLI command parsing, case creation, provider prompting, review,
  journal replay, graph, facts, and direct filesystem behavior;
- C daemon boot/stop and IPC framing;
- C decision/gate/grant, carrier, receipt, observation, record, graph, and
  persistence component tests;
- filesystem fixture expectations and denial/defer/quarantine behavior;
- JSONL `yai.store.record.v0` and `yai.record.v1` compatibility;
- LMDB database contents and graph relations derived from summary tokens;
- DuckDB projection and facts output;
- provider request/response and failure behavior;
- historical E05 closure/replay and E07 case-scoped provider properties that
  are not present in current product paths.

Characterization is a prerequisite for deletion. Historical source is not
copied into the current tree.

## Stage 1 — typed transition authority

Define the minimum typed semantic objects and compatibility readers required
to stop using `Record.summary` as a schema. Establish:

- Transition identity, Case generation, chronology, cause, Scope, actor,
  Decision, Attempt, observations/receipts, and outcome;
- Record as a versioned serialization envelope;
- one canonical Committed Transition Ledger;
- atomically maintained materialized CaseState;
- explicit rebuild/checkpoint behavior for current state and derived graph,
  index, memory, and analytics;
- compatibility import for existing journal/LMDB facts with provenance and no
  false promotion of summaries into target semantics.

The implementation must choose a transaction/storage design, but the database
technology remains an implementation decision until measured against the
contract.

## Stage 2 — admission and external effects

Make every product-reachable carrier consume an ExecutionGrant. Consolidate or
remove bypasses only after characterization. Implement:

- durable `PREPARED` intent before carrier invocation;
- expected Resource generation/pre-state and stable idempotency identity;
- typed Attempt and EffectReceipt outcomes;
- `FINALIZED` and first-class `INDETERMINATE` states;
- restart enumeration and reconciliation;
- explicit denial, failure, no-effect, and internal-only transitions;
- direct filesystem command routing or deliberate removal;
- one reachable carrier boundary shared by CLI/daemon entry paths.

Success requires crash and lost-reply tests; a happy-path filesystem write is
not sufficient.

## Stage 3 — extract the current Rust operational core

The 10k-line Rust CLI currently owns protocol, storage, control, provider,
projection, graph, and analytics behavior. Extract responsibilities behind
typed contracts without reifying every documentation noun:

- command/orchestration shell;
- transition transaction boundary;
- current-state materialization and replay;
- provider adapter boundary;
- carrier/effect boundary;
- derived projection/graph/analytics readers.

Choose one product core. Retain C surfaces only where their behavior or ABI is
intentionally part of that core; otherwise preserve their tests as regression
evidence before retirement. Do not preserve duplicate state owners for
topological symmetry.

## Stage 4 — first constitutional vertical

Deliver one narrow end-to-end Case vertical:

```text
qualified CaseState
→ Projection + ContextFrame
→ provider Invocation + typed ProviderResult
→ interpreted OperationCandidate
→ Decision + ExecutionGrant
→ PREPARE / filesystem carrier / FINALIZE
→ committed Transition + materialized CaseState
→ rebuilt next Projection
```

Required proofs include denial and provider failure transitions, crash between
PREPARE and FINALIZE, ambiguous effect reconciliation, replay equivalence,
provider replacement with the same semantic Case, and loss of continuation
without loss of correctness.

## Stage 5 — derived state and context efficiency

Only after transition authority is stable:

- rebuild graph/index/memory/analytics from qualified canonical inputs;
- implement Projection lineage and invalidation;
- introduce Residency as a replaceable policy with measured semantic budgets;
- introduce provider-independent ContextFrame and versioned rendering;
- add optional ProviderContinuationReference validation;
- evaluate ContextDelta only when a concrete consumer can prove equivalence to
  a full destination-frame rebuild.

The context-residency lab is an experimental baseline, not an implementation
specification. Token/KV optimizations cannot become Case memory.

## Explicit non-goals

This roadmap does not introduce Space or Agent as owners, choose a database,
import `yai-dev`, clone YVEX, create a directory for every concept, or require
ContextDelta. It does not authorize source changes as part of the documentation
refoundation.

## Exit criteria for the next source task

The next task should complete Stage 0 and propose the smallest Stage 1
transaction vertical. It must name behavior preserved, compatibility posture,
deletion candidates, and test evidence before changing source ownership. The
full implementation-gap inventory is retained in the refoundation evidence
package identified by [Documentation authority](docs/index.md).
