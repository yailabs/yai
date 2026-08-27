# Current executable architecture

Authority: implementation truth. Source-refoundation baseline:
`98b6a1f7e5289d53d22897aff2ee19243f6c43cc` (the dedicated documentation
checkpoint). This document describes the resulting
`YAI.SOURCE.REFOUNDATION.1` worktree.

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
  |     +-- journal replay and LMDB import
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

[`cmd/yai/src/main.rs`](../cmd/yai/src/main.rs) is now a 1,801-line command
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
| Case-bound provider prompt | `yai case enter` → attach provider → `yai prompt`; a real OpenAI-compatible HTTP fixture proves Attempt, model-output `EffectReceipt`, ModelInterpretation, ParticipantViewFrame, and InteractionTurn durability | provider result is misclassified; endpoint/model authority is split; no typed model-result→Operation seam |
| Filesystem review fixture | prepare fixed review → approve/deny/defer/quarantine; approval writes a real file and every branch persists current record shapes | fixed identities/path; write precedes persistence; no-effect branches use receipt-shaped records |
| Journal replay | validate `yai.store.record.v0` → import into LMDB → replay metadata/report | journal and LMDB are not one atomic authority |
| Graph | LMDB records → durable relations → rebuildable RuntimeGraph → bounded query | relation identities still depend partly on summary-token parsing |
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

There is no single global state authority today.

### JSONL journal

The Rust/C journal envelope remains `yai.store.record.v0`. Rust defines 35
`RecordKind` values in
[`record.rs`](../engine/yai-engine/src/record.rs); C defines 32 and cannot parse
the three Rust review kinds. A record has identity, a small set of references,
and one free-text `summary`. JSONL preserves append order for paths that write
it and is the replay source for LMDB import.

### LMDB

[`LmdbRecordStore`](../engine/yai-engine/src/store/lmdb.rs) maintains records,
secondary indexes, replay metadata, and graph relations. One record put is an
LMDB transaction, but review persistence performs a journal append and a
separate LMDB transaction for each record. Repeated IDs preserve multiple
JSONL entries while LMDB exposes the last value. Prompt interactions append to
their selected journal and need not update LMDB. Divergence is normal workflow
behavior, not only a crash case.

### Derived/cache stores

- LMDB graph relations are rebuildable indexes from records; RuntimeGraph is
  per-command and ephemeral.
- DuckDB facts are rebuildable analytics; eight of twelve declared tables are
  populated.
- Rust graph/projection/memory/query/reconcile values are derived.
- the C hot-state JSON snapshot is a daemon restart cache and is not updated by
  independent Rust mutations.

### Implicit schemas

Current behavior still parses whitespace-delimited `key:value` tokens from
`Record.summary`. Review state, case/provider admission, graph relations,
memory categories, fact fields, and rendered frames depend on this. `summary`
is therefore an implicit schema in executable reality even though the target
contract restricts it to presentation.

## Current control and effect behavior

The strongest live control path is the fixed Rust review fixture isolated in
[`review.rs`](../cmd/yai/src/review.rs). On approval it validates a lexical
sandbox prefix, writes the file through the current filesystem mechanics, then
appends dispatch/receipt records to an in-memory vector and persists them. Each
record is appended to JSONL and separately put into LMDB. A crash after the
write or during persistence can leave an effect with missing or partial
knowledge. No prepared grant, idempotency key, expected pre-state,
`INDETERMINATE` state, or restart reconciliation exists.

Deny, defer, and quarantine do not invoke the filesystem, yet currently create
a `FilesystemReceipt` with a blocked/deferred/quarantined status. The
Constitution rejects that target meaning while this Architecture preserves the
fact.

[`filesystem.rs`](../cmd/yai/src/filesystem.rs) also owns the direct `carrier
fs-read/fs-write` mechanics. Its lexical sandbox check is not authorization;
the write consumes no Decision/ExecutionGrant and emits no durable receipt.

Surviving C control, filesystem/process carrier, receipt, and observation
components contain typed or platform properties protected by component tests.
They are built separately and are not normal product call paths.

## Current provider and context behavior

[`provider.rs`](../cmd/yai/src/provider.rs) requires case-entry and
provider-attachment records, selects a summary-derived participant view,
appends a ParticipantViewFrame, sends a manually framed HTTP/1.x
`/v1/chat/completions` request, extracts one `content` string, and appends the
interaction records. Provider output is described as non-authoritative but is
serialized as `RecordKind::EffectReceipt`.

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
| `cmd/yai/src/replay.rs` | journal inspection/import/replay reports | product-reachable state compatibility boundary |
| `cmd/yai/src/graph_runtime.rs` | graph relation materialization, rebuild and query | product-reachable derived owner |
| `cmd/yai/src/analytics.rs` | DuckDB schemas, extraction and reports | product-reachable derived owner |
| `engine/yai-engine` | Rust record/journal/LMDB and reusable derived algorithms | product-reachable implementation library |
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
| one canonical Transition Ledger with transactional CaseState | JSONL history and LMDB records can diverge | typed transactional authority and migration/replay corpus |
| carrier consumes an ExecutionGrant | fixed review path, direct filesystem bypass, C carriers test-only | one general admission boundary and dynamic Binding resolution |
| PREPARE/EFFECT/FINALIZE with indeterminate recovery | effect precedes non-atomic records; no recovery state | idempotency, expected pre-state, failpoints, reconciliation |
| distinct ProviderResult, Observation, EffectReceipt | provider text and blocked outcomes use receipt-shaped records; process observations are not persisted | typed result/observation/receipt schema |
| Case plus materialized CaseState | Case state is summary-derived across journal/LMDB and local views | one lifecycle/generation model |
| summary is presentation only | control/graph/fact/context paths parse it | versioned semantic fields and historical migration |
| Projection/Residency/ContextFrame/KV separation | frame/projection identity is encoded in summaries; no residency/KV contract | typed lineage and provider-independent invocation frame |
| provider replacement preserves semantic continuity | attachment and args/env split invocation authority | normalized Binding and invocation lineage |
| derived data rebuilds from canonical state | graph/facts mostly rebuild; current authority remains ambiguous | source generation, invalidation and equivalence contracts |

These gaps are intentionally not repaired here. The
[Roadmap](../ROADMAP.md) owns their implementation sequence.
