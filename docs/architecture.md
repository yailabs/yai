# Current executable architecture

Authority: implementation truth. Baseline:
`db183ae4c56bd16c7e6f31787ee4d90a51496d6d`.

This document describes the executable repository, including its
contradictions. It does not claim that the [Constitution](constitution.md) is
implemented. Target changes and sequencing belong only in the
[Roadmap](../ROADMAP.md).

## Executable summary

YAI currently has two unequal product processes:

```text
operator
  |
  +-- yai (Rust CLI; operational center)
  |     +-- yai-engine Rust library
  |     |     +-- JSONL journals
  |     |     +-- LMDB records and graph relations
  |     |     +-- derived graph/projection/memory/query/reconcile views
  |     +-- raw HTTP OpenAI-compatible provider invocation
  |     +-- direct Rust filesystem effects on two paths
  |     +-- external duckdb CLI for derived facts
  |     +-- Unix-socket client for selected yaid requests
  |
  +-- yaid (C daemon)
        +-- Unix-socket status/info/shutdown
        +-- prepared minimum/filesystem fixture loops
        +-- C JSONL record/journal and projection code used by those loops
        +-- restartable hot-state snapshot
```

The Rust CLI contains 10,901 lines in
[`cmd/yai/src/main.rs`](../cmd/yai/src/main.rs). Argument parsing, rendering,
provider transport, case admission, review transitions, filesystem effects,
fact orchestration, and much of the domain behavior share that file.

The normal CLI links [`yai-engine`](../engine/yai-engine/src/lib.rs) as a Rust
dependency. There is no product C→Rust call edge. The C/Rust FFI in
[`system/engine_bridge`](../system/engine_bridge/rust_engine_backend.c) is used
by a smoke binary; `engine/yai-engine-ffi` is a marker crate. Static-library
linking means `yaid` does not gain every compiled C module merely because the
module appears in the Make archive.

## Demonstrated verticals

“Complete” below means the bounded demonstrated path has an entrypoint,
consequence or durable transition, evidence/output, and executable or frozen
run proof. It does not mean constitutional, general, or production-ready.

| Vertical | Current path | What is real | First architectural gap |
|---|---|---|---|
| Case-bound provider prompt | `yai case enter` / attach provider / `yai prompt` | raw HTTP provider call and Attempt, model-output `EffectReceipt`, ModelInterpretation, ParticipantViewFrame, InteractionTurn journal records; ten successful calls exist in the frozen context-residency run | endpoint/model authority is split between attachment and args/env; provider result is misclassified; no typed model-result→Operation seam |
| Filesystem review fixture | prepare fixed review, then approve/deny/defer/quarantine | approve writes a real file; all branches append journal and LMDB records | identities/path are hard-coded; write precedes persistence; deny/defer/quarantine emit receipt-shaped records despite no carrier attempt |
| Journal replay | inspect `yai.store.record.v0`, import into LMDB, store replay metadata/report | validated durable indexed import and operator report | journal and LMDB are not one atomic authority; replay copies between ambiguous owners |
| Graph materialization/rebuild/query | LMDB records→relations→RuntimeGraph | durable LMDB relation indexes and rebuildable in-memory graph/query output | some relation identities are parsed from summary strings |
| DuckDB facts | embedded schema/extraction SQL invoked through `duckdb` CLI | eight populated derived fact families and reports | four declared families have no extractor; SQL/orchestration lives in CLI |

Partial paths remain important evidence:

- `yai carrier fs-write` performs a real path-bounded write but consumes no
  Decision/Grant and writes no durable receipt.
- `yai process observe` probes a PID but persists no Observation; CLI process
  signaling does not dispatch. The C process carrier can signal a test-owned
  child only through component tests.
- provider registry CRUD and start planning persist device data but do not
  configure the prompt invocation path; provider start is dry-run only.
- CaseHandle/CapabilityLease views are inspect-only; no carrier consumes the
  lease. Separate C implementations have no verified caller.
- `yaid` fixture loops demonstrate plumbing, not general admission. The
  filesystem loop writes fixture files before creating decision/receipt-shaped
  records and does not call the C control or carrier implementations.

## Current state and schema authority

There is no single global state authority today.

### JSONL journal

The Rust/C journal envelope is `yai.store.record.v0`. Rust defines 35
`RecordKind` values in
[`record.rs`](../engine/yai-engine/src/record.rs); C defines 32 and cannot parse
the three Rust review kinds. A record has identity and a small set of refs plus
one free-text `summary` field. JSONL preserves append order for paths that write
it and is the replay source for LMDB import.

### LMDB

[`LmdbRecordStore`](../engine/yai-engine/src/store/lmdb.rs) maintains
records-by-ID and secondary indexes plus graph relations. A record write is one
LMDB transaction, but review persistence loops over records and performs a
journal append followed by a separate LMDB transaction for each record.
Repeated record IDs preserve multiple JSONL entries while LMDB exposes the
last value.

Prompt interactions append to their selected journal and do not necessarily
update LMDB. Divergence is therefore normal workflow behavior, not only a crash
case.

### Derived and cached stores

- LMDB graph relations are derived indexes from records; RuntimeGraph is
  per-command and rebuildable.
- DuckDB facts are rebuildable analytics. Eight of twelve declared tables are
  populated.
- Rust projection, memory, query, and reconcile outputs are derived.
- The C hot-state JSON snapshot survives daemon restart but is a cache and is
  not updated by independent Rust mutations.
- provider-device JSONL is authoritative only for registry CRUD/dry-run
  planning, not actual invocation.

### Implicit schemas

Current semantic behavior depends heavily on parsing whitespace-delimited
`key:value` tokens from `summary`. Review state, case/provider admission,
thread/frame identity, graph relations, memory categories, fact fields, and
rendering all use this pattern. Examples include
`review_summary_value`, `summary_token`, and
`summary_token_value_or` in the CLI/LMDB code. `summary` is therefore an
implicit schema today, contrary to the target contract.

## Current control and effect behavior

The strongest live control path is the fixed Rust review fixture. On approval
it validates a lexical sandbox prefix, creates the parent directory, writes the
file, adds dispatch/receipt records to an in-memory vector, then calls
`persist_control_records`. That function appends each JSONL line and performs a
separate LMDB put. A crash after the write or any partial persistence can leave
an external effect with missing/partial knowledge. No prepared grant,
idempotency key, expected pre-state, `INDETERMINATE` state, or restart
reconciliation exists.

Deny, defer, and quarantine do not call the carrier, but the current path still
creates a `FilesystemReceipt` with blocked/deferred/quarantined status. The
Constitution intentionally rejects that target meaning while preserving the
current fact here.

The C filesystem, process, gate, dispatch, receipt, and observation components
contain useful typed and platform behavior and have smoke tests. They are not
called by a normal product executable. Directory/build membership is not
product reachability.

## Current model and context behavior

`yai prompt` requires case-entry and provider-attachment records, renders a
summary-only participant view, appends a ParticipantViewFrame record, sends an
HTTP/1.x `/v1/chat/completions` request using a manually framed OpenAI-compatible
payload, extracts one `content` string, and appends interaction records.

The provider output is correctly described in prose as non-authoritative, but
is serialized under `RecordKind::EffectReceipt`. The adapter has no formal
timeouts/deadlines, streaming, cancellation contract, TLS policy, tokenizer
contract, native YVEX contract, or provider continuation reference.

The persisted ParticipantViewFrame is a summary string containing frame,
thread, projection, previous-frame, counts, redaction, and freshness tokens.
It is not the provider-independent `ContextFrame` defined by the target
reference. The current implementation has:

- no formal Projection identity contract separate from rendered text;
- no Residency decision object/algorithm;
- no ContextFrame schema or lifecycle;
- no ContextDelta source/destination/invalidation contract;
- no authoritative tokenization identity;
- no KV or provider computational-continuation integration.

The context-residency lab proves real case-bound provider invocation. Its C5
condition estimates logical base/delta token reduction from artifacts and
reuses C4 latency; it explicitly does not claim KV reuse or a Context Compiler.

## Source and process ownership

| Surface | Current executable role | Classification |
|---|---|---|
| `cmd/yai` | primary operator entrypoint and de facto domain coordinator | product-reachable; requires extraction/rewrite |
| `engine/yai-engine` record/journal/LMDB/graph | live Rust persistence/index behavior | product-reachable core evidence |
| Rust projection/memory/query/reconcile | live derived algorithms | product-reachable derivation evidence |
| `cmd/yaid` + `system/daemon` | narrow Unix-socket daemon and fixture loops | product-reachable process boundary |
| C store/projection/hot | used by daemon fixtures/hot snapshot | reachable duplicate/merge evidence |
| C control/carrier/process/graph/memory/reconcile/index/observation | direct smoke consumers only | component characterization, not product capability |
| C/Rust bridge | rust-engine-r1 smoke only | tested compatibility candidate |
| `net/` | enum/string helpers, schemas, fixtures, no network I/O | scaffold/evidence |
| `proto/` | schemas/fixtures with mixed consumer status | evidence; not automatically deployed ABI |
| tests/labs | behavior proof and reproducible evidence | non-authoritative evidence |

External consumers of the broad C headers, daemon protocol, JSONL v0,
proto schemas, and Rust public API have not been enumerated. Their absence
inside this checkout is deletion evidence, not proof that no consumer exists.

The executable `yai info` output still labels the repository
`SPINE.51 Fact Plane Freeze` and calls planned provider/registry surfaces
active. That status vocabulary is stale compatibility output; it does not
override this Architecture and must be characterized before source cleanup.

## Requirement/current/gap register

| Constitutional requirement | Current implementation | Gap |
|---|---|---|
| one canonical Transition Ledger with transactional CaseState | JSONL history and LMDB current records can diverge | typed transactional authority and migration/replay corpus |
| carrier consumes an ExecutionGrant | only a fixed Rust review path; direct fs-write bypass; C carriers are test-only | one general admission boundary and dynamic Binding resolution |
| PREPARE/EFFECT/FINALIZE with indeterminate recovery | effect occurs before non-atomic records; no recovery state | idempotency, expected pre-state, failpoints, reconciliation |
| distinct ProviderResult, Observation, EffectReceipt | provider text stored as EffectReceipt; blocked outcomes also use receipts | typed result/observation/receipt schema |
| Case + materialized CaseState | Case semantics spread over summaries, C fixture types, local views, journal/LMDB | one lifecycle and generation model |
| summary is presentation only | many control/graph/fact/context paths parse it | versioned semantic fields and historical migration |
| Projection/Residency/ContextFrame/KV separation | projection/frame identity encoded in summaries; no residency/KV contract | typed lineage and provider-independent invocation frame |
| provider replacement preserves semantic continuity | provider binding and actual args/env invocation authority are split | normalized Binding and invocation lineage |
| derived data rebuilds from canonical state | graph/facts mostly rebuild; current state authority is ambiguous | source generation, invalidation, equivalence, canonical fallback |

These gaps are intentionally not repaired in prose. The
[Roadmap](../ROADMAP.md) is the implementation-facing owner.
