# Current executable architecture

Authority: implementation truth. Source-refoundation baseline:
`8839f65d9eb989d0b7bc4f6c94a87e1b5f1e76c0` (the isolated
`YAI.SOURCE.REFOUNDATION.1` checkpoint). This document describes the resulting
`YAI.SOURCE.REFOUNDATION.2` worktree.

This document includes current contradictions. It does not claim that the
[Constitution](constitution.md) is implemented. Target changes and sequencing
belong only in the [Roadmap](../ROADMAP.md).

## Executable summary

YAI has two unequal product processes:

```text
operator
  |
  +-- yai (Rust command/process boundary)
  |     +-- provider/case invocation
  |     +-- controlled review transition family
  |     +-- direct filesystem resource path
  |     +-- canonical Transition/CaseState authority in LMDB
  |     +-- legacy journal inspect/import/replay compatibility
  |     +-- LMDB graph materialization/query
  |     +-- DuckDB analytical derivation
  |     +-- yai-engine Rust library
  |     +-- selected yaid Unix-socket requests
  |
  +-- yaid (C daemon)
        +-- Unix-socket status/info/shutdown
        +-- prepared minimum/filesystem fixture loops
        +-- C JSONL records/journal and projections used by those loops
        +-- restartable hot-state snapshot
```

[`cmd/yai/src/main.rs`](../cmd/yai/src/main.rs) is a 1,840-line command
parser, dispatcher, common CLI support surface, and compatibility shell. The
current domain implementation is grouped by demonstrated boundary in
[`provider.rs`](../cmd/yai/src/provider.rs),
[`review.rs`](../cmd/yai/src/review.rs),
[`filesystem.rs`](../cmd/yai/src/filesystem.rs),
[`replay.rs`](../cmd/yai/src/replay.rs),
[`graph_runtime.rs`](../cmd/yai/src/graph_runtime.rs), and
[`analytics.rs`](../cmd/yai/src/analytics.rs). These modules do not imply
future subsystems; they isolate existing behavior for the next semantic
refoundation.

The normal CLI links [`yai-engine`](../engine/yai-engine/src/lib.rs) directly
as Rust. There is no product C→Rust call edge or installed Rust C ABI. The
former marker FFI crate and smoke bridge were removed.

The production C archive contains 16 sources. `yaid` adds only its entrypoint,
IPC, and core loop. Component-only C mechanics build into a separate
characterization archive and are linked only by their tests. Static archive
membership is no longer used to make those components look product-reachable.

## Demonstrated product verticals

“Complete” means that the bounded path has an entrypoint, consequence or
durable residue, operator output, and executable regression proof. It does not
mean constitutional, general, or production-ready.

| Vertical | Current path and demonstrated consequence | First architectural gap |
|---|---|---|
| Case-bound provider prompt | `yai case enter` → typed participant/provider attachment → typed Invocation and ProviderResult → non-authoritative ModelInterpretation; a real OpenAI-compatible HTTP fixture proves the path while legacy records remain readable | ContextFrame and result→Operation normalization do not exist; render/frame/thread material remains legacy-derived |
| Filesystem review fixture | typed ReviewRequested/ReviewResolved transitions materialize current review state; approval writes a real file and every branch retains legacy output compatibility | fixed identities/path; the effect still precedes final commit; no PREPARE/INDETERMINATE protocol |
| Journal compatibility | inspect/dry-run/import `yai.store.record.v0` or `yai.record.v1`, preserving unknowns opaquely in an isolated target; old replay still materializes legacy record indexes | general semantic promotion is deliberately absent; the old record plane remains compatibility data, not authority |
| Graph | typed canonical transitions and explicitly decoded legacy records → derived relations → rebuildable RuntimeGraph → bounded query | generation/version invalidation remains minimal; legacy-only cases still depend on compatibility translation |
| Analytical facts | LMDB operational records → DuckDB extraction → reports | four declared families have no extractor; schema/orchestration remains embedded in the command crate |

`yaid` startup, status/info/shutdown, restart, fixture loops, and hot snapshot
reconstruction remain covered separately. The direct filesystem command is
also characterized: it performs a real path-bounded write/read and creates no
admission or receipt residue. That behavior is preserved as a known bypass,
not endorsed as target architecture.

Partial and component paths must not be promoted into product claims:

- `yai process observe` probes a PID but persists no Observation; CLI process
  signaling does not dispatch.
- the C process carrier observes, signals, and re-observes a test-owned child;
  it is a characterized platform boundary, not a product vertical.
- the C filesystem carrier preserves pre-state, real write, post-state hash,
  and receipt mechanics under component tests only.
- `yaid` filesystem fixtures perform fixture effects before creating
  decision/receipt-shaped records and do not call the C control/carrier stack.

The provider planning registry, CaseHandle/CapabilityLease inspection views,
generic carrier registries, synthetic dispatch families, C graph/index/memory
mirrors, `net/`, and its unconsumed `proto/` fixtures are absent. None had a
product caller or unique property not already captured by the surviving
verticals/tests.

## Current state and schema authority

### Canonical LMDB authority

Rust owns one canonical semantic write path in
[`transition.rs`](../engine/yai-engine/src/transition.rs) and
[`lmdb.rs`](../engine/yai-engine/src/store/lmdb.rs). Its serialized contracts
are `yai.transition.v1` and `yai.case_state.v1`. One bounded LMDB write
transaction:

1. validates typed payload closure and global Transition identity;
2. compares the expected per-Case generation;
3. appends the immutable Transition by identity and zero-padded Case sequence;
4. reduces it into CaseState;
5. commits ledger and materialization together.

The ledger databases are `transitions_by_id` and
`case_transition_sequence`; `case_state` is the rebuildable materialization.
Per-Case sequence, not timestamp, determines reducer order. Duplicate
Transition IDs and stale generations fail deterministically. Reopening after a
commit exposes both history and state; an injected failure before commit
exposes neither. Rebuild verifies:

```text
materialized CaseState == replay(ordered canonical Transitions)
```

The initial reducer deliberately covers only current live fields: Case
lifecycle/generation, participant bindings/admitted views, one current
provider/model attachment, latest provider invocation/result/interpretation
lineage, and fixture review state. It is not a general object bag.

### Legacy compatibility

The C/Rust journal remains readable as `yai.store.record.v0`; LMDB legacy
record envelopes remain `yai.record.v1`. Rust still defines 35 legacy kinds and
C 32. [`compatibility.rs`](../engine/yai-engine/src/compatibility.rs) is the
only summary-token decoder. It classifies input as losslessly structurally
promoted, promoted with compatibility metadata, preserved opaque, or rejected
malformed. Unknown future kinds/schemas are retained without invented meaning.

`yai journal compatibility-inspect` and `compatibility-import` support inspect,
dry-run, and import into an explicitly isolated LMDB target. Compatibility
payloads never append canonical Transitions or CaseState. Old `journal replay`
and legacy record indexes survive for operator/data compatibility; they are no
longer historical authority. Provider and review commands commit canonical
state first and emit their old JSONL/record shapes afterward for existing
consumers.

### Derived/cache stores

- LMDB graph relations are rebuilt from typed Transitions plus the explicit
  legacy compatibility decoder; graph failure cannot affect canonical commit.
- RuntimeGraph is per-command and ephemeral.
- DuckDB facts are rebuildable analytics; historical extraction is routed
  through compatibility fields.
- Rust projection/memory/query/reconcile values are derived legacy views.
- the C hot-state JSON snapshot is a daemon restart cache and is not updated by
  independent Rust mutations.

No canonical reducer, graph relation, provider/review decision, memory
category, or fact field parses arbitrary `summary` text. Remaining summary
grammar exists only inside the named legacy compatibility boundary and in
presentation assertions.

## Current control and effect behavior

The strongest live control path is the fixed Rust review fixture isolated in
[`review.rs`](../cmd/yai/src/review.rs). ReviewRequested and ReviewResolved are
typed canonical payloads, and operator reads use CaseState rather than summary
parsing. On approval the CLI still validates a lexical sandbox prefix and
writes the file before committing ReviewResolved. A crash after the write but
before commit can therefore leave an effect with missing canonical knowledge.
No prepared grant, idempotency key, expected pre-state, `INDETERMINATE` state,
or restart reconciliation exists.

Deny, defer, and quarantine do not invoke the filesystem. Their typed review
transition records exactly that no carrier was attempted, but compatibility
export still creates the historical `FilesystemReceipt` shapes. The
Constitution rejects that legacy meaning while this Architecture preserves the
observable compatibility behavior.

[`filesystem.rs`](../cmd/yai/src/filesystem.rs) also owns the direct `carrier
fs-read/fs-write` mechanics. Its lexical sandbox check is not authorization;
the write consumes no Decision/ExecutionGrant and emits no durable receipt.

Surviving C control, filesystem/process carrier, receipt, and observation
components contain typed or platform properties protected by component tests.
They are built separately and are not normal product call paths.

## Current provider and context behavior

[`provider.rs`](../cmd/yai/src/provider.rs) promotes legacy Case/participant
bindings through the compatibility boundary, then commits typed participant
admission, provider attachment, Invocation, ProviderResult, and
ModelInterpretation transitions. ProviderResult carries provider/model/
invocation identity and returned content; CaseState stores only current lineage
and output size. The compatibility journal still emits the former
`EffectReceipt` record so old readers and vertical assertions remain valid.

The invocation path sends a manually framed HTTP/1.x
`/v1/chat/completions` request and extracts one `content` string. Transport
endpoint replacement is allowed without changing Case or model identity; the
current attachment retains endpoint configuration but reducer closure matches
participant/provider/model. A failed HTTP invocation leaves its typed
invocation committed and invents no ProviderResult.

There is no formal deadline, streaming/cancellation, TLS, tokenizer, native
YVEX, or continuation contract. The ParticipantViewFrame is a summary-token
record rather than the provider-independent ContextFrame defined by the
reference. The implementation has no formal Projection identity contract,
Residency decision, ContextFrame schema, ContextDelta invalidation contract,
token identity, or KV integration. Loss/replacement of a model runtime does
not erase the JSONL Case history, but provider replacement is not yet a typed
or fully tested contract.

The context-residency lab estimates logical base/delta token reduction and
does not prove KV reuse or an implemented Context Compiler.

## Physical ownership after source refoundation

| Surface | Executable role | Classification |
|---|---|---|
| `cmd/yai/src/main.rs` | parsing, dispatch, common CLI/process initiation and residual compatibility commands | product-reachable command boundary |
| `cmd/yai/src/provider.rs` | current case admission, projection rendering, HTTP provider invocation and result residue | product-reachable transition family |
| `cmd/yai/src/review.rs` + `filesystem.rs` | fixed review path and current filesystem resource mechanics | product-reachable effect evidence; constitutionally incomplete |
| `cmd/yai/src/replay.rs` | legacy inspect/dry-run/isolated import and replay reports | product-reachable state compatibility boundary |
| `cmd/yai/src/graph_runtime.rs` | graph relation materialization, rebuild and query | product-reachable derived owner |
| `cmd/yai/src/analytics.rs` | DuckDB schemas, extraction and reports | product-reachable derived owner |
| `engine/yai-engine` | canonical Transition/CaseState semantics, LMDB authority, legacy decoder, and reusable derived algorithms | product-reachable semantic/data authority |
| `cmd/yaid` + selected `system/` sources | daemon IPC, fixture loops, C journal/projection/hot snapshot | product-reachable process/platform boundary |
| separate C component archive | gates, carriers, process/observation and compatibility mechanics | component characterization; not product capability |
| tests/labs/history | current proof, research, and historical specification | evidence, never implementation authority |

There are 55 surviving headers under `include/yai`, down from 101. They are a
source compatibility surface for the product C subset and characterized
platform components; the repository does not install them. No external
consumer is known. The daemon socket JSON, JSONL record shapes, and public Rust
crate remain compatibility risks because external absence cannot be proven
from one checkout.

## Requirement/current/gap register

| Constitutional requirement | Current implementation | Gap |
|---|---|---|
| one canonical Transition Ledger with transactional CaseState | implemented in LMDB for typed payloads; provider and review are live consumers | migrate remaining current workflows; add operational checkpoint/compaction policy |
| carrier consumes an ExecutionGrant | fixed review path, direct filesystem bypass, C carriers test-only | one general admission boundary and dynamic Binding resolution |
| PREPARE/EFFECT/FINALIZE with indeterminate recovery | effect precedes non-atomic records; no recovery state | idempotency, expected pre-state, failpoints, reconciliation |
| distinct ProviderResult, Observation, EffectReceipt | ProviderResult is typed; compatibility export still uses receipt-shaped records; process observations are not canonical | typed Observation/EffectReceipt during the effect refoundation |
| Case plus materialized CaseState | implemented for the minimal live provider/review field set and rebuildable from ledger | extend only with future Transition consumers; migrate daemon fixture state |
| summary is presentation only | canonical reducers and migrated paths do not parse it; old projection/frame and analytics records use the compatibility decoder | migrate or retire remaining legacy-only producers and views |
| Projection/Residency/ContextFrame/KV separation | frame/projection identity is encoded in summaries; no residency/KV contract | typed lineage and provider-independent invocation frame |
| provider replacement preserves semantic continuity | Case/participant/provider/model invocation lineage is typed; endpoint replacement is accepted | normalized Binding lifecycle, ContextFrame, and provider replacement proof |
| derived data rebuilds from canonical state | graph rebuild consumes typed transitions and legacy compatibility; facts remain legacy-derived | generation, invalidation, and full typed analytics inputs |

The remaining gaps are intentional boundaries of this wave. The
[Roadmap](../ROADMAP.md) owns their implementation sequence.
