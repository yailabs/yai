# Test and evidence cases

Authority: current validation entrypoints and the limits of the claims they
support. Tests prove executable behavior at the frozen repository state; they
do not promote historical or experimental terminology into architecture.

## Validation layers

| Layer | Entry point | Authority |
|---|---|---|
| documentation | `make check-docs` | canonical files, authority shape, and local links |
| repository layout | `make check-layout` | expected current source/build surface |
| build | `make build` | current C/Rust sources compile and link |
| smoke | `make smoke` | bounded component and CLI behaviors |
| full | `make check` | layout, documentation, build, and smoke aggregation |

## Executable verticals

The strongest current validation groups are:

| Vertical | Representative tests | What it supports | What it does not support |
|---|---|---|---|
| provider prompt | `provider-runtime-surface`, `model-behavior-policy-facts` | OpenAI-compatible HTTP path, output recording, projection/facts summaries | provider-independent ContextFrame, typed ProviderResult, continuation/KV |
| operator review/filesystem | `operator-review-loop`, `review-loop-test-matrix`, `receipt-decision-projection-facts` | hard-coded review decisions and fixture write path | durable PREPARE, general grants/carriers, atomic external effects |
| journal replay/store | `journal-replay-*`, `record-store-*`, `replay-idempotency-schema-version` | JSONL replay, LMDB import/index behavior, compatibility schemas | one atomic canonical ledger/current-state authority |
| graph | `graph-relation-write-path`, `runtimegraph-*` | relation materialization and causal query behavior | canonical graph truth or typed replacement of summary tokens |
| facts/analytics | `duckdb-fact-plane`, `fact-reports-cli`, policy/carrier/divergence facts tests | rebuildable DuckDB extraction and reports | authoritative operational state |

Many lower-level C tests exercise control, carrier, observation, graph, memory,
and reconciliation components that the product daemon does not generally
reach. A passing component test is evidence for that component contract, not
evidence for end-to-end product integration.

## Minimal repository validation

```sh
make info
make check-docs
make check-layout
```

For a full validation run:

```sh
make check
```

Read the first failing target directly. Do not mask failures caused by a dirty
worktree, missing native dependency, or absent provider.

## Manual local inspection

With built binaries and an isolated `YAI_HOME`:

```sh
build/bin/yai doctor
build/bin/yai hot status
build/bin/yai store status
build/bin/yai store summary
```

Detailed public test wrappers remain under `tests/cases/`:

- [repository health](../tests/cases/00-repository-health/README.md);
- [local runtime inspection](../tests/cases/01-local-runtime-inspection/README.md);
- [filesystem-loop manual evidence](../tests/cases/02-filesystem-loop-manual/README.md);
- [model-behavior lab evidence](../tests/cases/03-model-behavior-lab/README.md).

The last two point into `labs/`. Labs are reproducible evidence packages, not
current architecture or operational requirements.

## Historical properties requiring future regression tests

The next source refoundation must convert valuable historical behavior into
current specifications before deletion or consolidation. Priority properties
are E05 transition closure/replay/causal reachability, E07 Case-scoped semantic
workset/provider rendering, process signal observation, and physical carrier
enforcement. Their commits and classifications are recorded in the
documentation-refoundation evidence package; historical tests are not current
product claims.

## Non-claims

These validations do not prove production readiness, provider breadth, model
quality, deterministic model behavior, crash-safe external effects, or a
complete constitutional vertical. Facts, graph, projection, hot-state, and
provider outputs remain subject to the authority limits in
[Executable architecture](architecture.md).
