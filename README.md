<!--
YAI

Copyright (c) 2026 Francesco Maiomascio.
All rights reserved.

This file is part of the source-available YAI repository. Use, copying,
modification, distribution, and production operation are governed by the
repository licensing documents, including LICENSE.md and docs/legal.md.
-->

# YAI

YAI is an early local runtime for governing admitted transformations of
canonical operational state. Work persists in a durable `Case`; a transition
binds typed intent, authority, and observed outcome without making a model,
provider, cache, graph, or presentation view the owner of truth.

That sentence is the constitutional direction, not a claim that the current
repository already implements it. The current executable is a mixed C/Rust
prototype with valuable verticals and known authority gaps.

## Read this repository

The documentation has three deliberately separate truth classes:

- [Constitution](docs/constitution.md) states invariants that the refounded
  implementation must satisfy.
- [Architecture](docs/architecture.md) describes only what this repository
  currently executes at HEAD.
- [Roadmap](ROADMAP.md) owns the delta between those two.

The [documentation index](docs/index.md) is the complete authority map.
[Semantics](docs/reference/semantics.md),
[state and transitions](docs/reference/state-transitions.md),
[context](docs/reference/context.md), and
[model/resource boundaries](docs/reference/boundaries.md) own the stable
reference contracts.

`work/`, `labs/`, `yai-dev`, and `refoundation/audits/` are evidence and
history. They are not required to discover the current architecture and do not
override the canon above.

## Current executable reality

The Rust `yai` CLI is currently the operational center. It performs case and
provider admission, calls an OpenAI-compatible provider over HTTP, appends
interaction records, manages LMDB records and graph relations, builds DuckDB
facts, renders derived views, and implements both a fixture-bound reviewed
filesystem write and a separate direct write path.

The C `yaid` process supplies Unix-socket status/lifecycle behavior, a
restartable hot snapshot, and two prepared fixture loops. The C control,
carrier, process, graph, memory, reconcile, index, observation, and Rust bridge
families are component-tested or scaffolded but are not general product paths
through `yaid`.

Five current vertical families have executable evidence:

1. case-bound invocation of an external/local model provider;
2. a fixed operator review and filesystem-effect fixture;
3. JSONL journal validation and replay into LMDB;
4. graph relation materialization, rebuild, and query;
5. derived DuckDB fact extraction and reporting for eight populated families.

Important limitations remain:

- JSONL history and LMDB current state can diverge and are not one atomic
  authority.
- The reviewed filesystem write happens before its result records are
  persisted; ambiguous crash outcomes are not reconciled.
- Direct `carrier fs-write` bypasses the review/control path.
- Provider output is stored as an `EffectReceipt` even though it is a
  `ProviderResult`, not proof of a YAI resource effect.
- Domain semantics are widely encoded in free-text `summary key:value` tokens.
- `ContextFrame`, `ContextDelta`, formal residency, provider continuation, and
  KV continuation are not implemented contracts.
- Most live domain behavior is concentrated in
  `cmd/yai/src/main.rs`; many C noun modules have no normal executable caller.

See [Architecture](docs/architecture.md) for the evidence-backed topology and
[Roadmap](ROADMAP.md) for the implementation sequence.

## Core boundaries

- A `Case` owns durable identity and lifecycle. `CaseState` is a materialized
  consequence of committed transitions, not a mutable Case object.
- `Scope` is the immutable effective boundary of one transition. It is not a
  daemon, store, world, or owner.
- `Space` and runtime-owning `Agent` are rejected from the canonical ontology
  until their documented falsifiers are met.
- A model or provider produces non-authoritative material. YAI may invoke a
  provider, but it does not host or own model execution.
- Projection, residency, ContextFrame, rendered bytes, tokens, and provider KV
  state have distinct identities and authority.
- Graph, index, memory, analytics, hot state, and participant views are derived
  or cached. They are not canonical history.
- An external effect requires durable preparation, bounded execution,
  observation, and finalization. Missing acknowledgement is not proof that no
  effect occurred.

## Build and inspect

From the repository root:

```sh
make info
make check-docs
make check
```

`make check` builds and runs a broad historical smoke inventory as well as the
current documentation checks. It does not turn component tests, fixtures, or
named scaffolds into supported product capability. See the
[quickstart](docs/quickstart.md) and [validation guide](docs/test-cases.md).

## Repository map

```text
cmd/          current yai and yaid entrypoints
engine/       Rust record/store/graph/derived algorithms
system/       C daemon behavior and component-tested C implementations
include/      broad C header surface; external consumers are not yet audited
proto/        schemas and fixtures, many of them scaffold rather than wire ABI
tests/        executable characterization and smoke evidence
labs/         reproducible experiments and frozen run evidence
work/         de-authorized project history, spines, waves, and discovery notes
docs/         canonical documentation and explicitly labeled research
tools/        build, layout, schema, and documentation validation
```

Directory names do not establish semantic ownership. The next implementation
refoundation must characterize behavior before moving or deleting source.

## Project posture

YAI is source-available for technical evaluation and is not production-ready
unless explicitly stated. Provider names in evidence or reference material are
not a support matrix. YAI is not a model runtime, generic agent framework,
workflow builder, cloud platform, or generic audit logger.

- [License](LICENSE.md)
- [Legal posture](docs/legal.md)
- [Contributing](CONTRIBUTING.md)
- [Security](SECURITY.md)
- [Notices](NOTICE.md)
