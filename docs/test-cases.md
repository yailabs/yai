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
| characterization | `make characterization` | selected product verticals plus known bypass behavior |
| full | `make check` | layout, documentation, build, and smoke aggregation |

## Executable verticals

The strongest current validation groups are:

| Vertical | Representative tests | What it supports | What it does not support |
|---|---|---|---|
| provider prompt | `provider-model-vertical`, `model-behavior-policy-facts`, engine canonical-authority tests | real OpenAI-compatible fixture invocation, typed invocation/ProviderResult/interpretation transitions, legacy projection/facts summaries | provider-independent ContextFrame, typed terminal provider failures, continuation/KV |
| operator review/filesystem | `operator-review-loop`, `review-loop-test-matrix`, `receipt-decision-projection-facts` | hard-coded review decisions and fixture write path | durable PREPARE, general grants/carriers, atomic external effects |
| direct filesystem bypass | `direct-filesystem-bypass` | current bounded write/read and absence of durable admission/receipt residue | authorized or constitutionally valid effect execution |
| journal replay/store | `journal-replay-*`, `record-store-*`, `replay-idempotency-schema-version`, engine canonical-authority tests | JSONL compatibility replay/import, atomic LMDB Transition/CaseState commit, restart, rollback, replay and rebuild | retire remaining legacy mutable record paths after their consumers migrate |
| graph | `graph-relation-write-path`, `runtimegraph-*`, engine derived-failure tests | typed-transition and decoded-legacy relation materialization, deterministic rebuild and causal query behavior | migrate remaining legacy compatibility inputs; graph stays derived |
| facts/analytics | `duckdb-fact-plane`, `fact-reports-cli`, policy/carrier/divergence facts tests | rebuildable DuckDB extraction and reports | authoritative operational state |

Lower-level C tests exercise retained control, carrier, observation, store,
projection, and hot-state mechanics that the product daemon does not generally
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
make characterization
```

Read the first failing target directly. Do not mask failures caused by a dirty
worktree, missing native dependency, or absent provider.

## Manual local inspection

With built binaries and an isolated `YAI_HOME`:

```sh
target/debug/yai doctor
target/debug/yai hot status
target/debug/yai store status
target/debug/yai store summary
```

Detailed public test wrappers remain under `tests/cases/`:

- [repository health](../tests/cases/00-repository-health/README.md);
- [local runtime inspection](../tests/cases/01-local-runtime-inspection/README.md);
- [filesystem-loop manual evidence](../tests/cases/02-filesystem-loop-manual/README.md);
- [model-behavior lab evidence](../tests/cases/03-model-behavior-lab/README.md).

The last two point into `labs/`. Labs are reproducible evidence packages, not
current architecture or operational requirements.

## Historical properties requiring future regression tests

E05 transition closure/replay/causal reachability, E07 Case-scoped semantic
workset/provider rendering, and V11 process observation/effect linkage are
classified in `refoundation/source-refoundation-1/legacy-property-recovery.tsv`.
They constrain the next implementation task but remain historical
specifications where no current product test demonstrates them.

## Non-claims

These validations do not prove production readiness, provider breadth, model
quality, deterministic model behavior, crash-safe external effects, or a
complete constitutional vertical. Facts, graph, projection, hot-state, and
provider outputs remain subject to the authority limits in
[Executable architecture](architecture.md).
