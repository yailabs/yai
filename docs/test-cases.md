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
| characterization | `make characterization` | selected product verticals plus direct-bypass removal regression |
| full | `make check` | layout, documentation, build, and smoke aggregation |

## Executable verticals

The strongest current validation groups are:

| Vertical | Representative tests | What it supports | What it does not support |
|---|---|---|---|
| provider prompt | `provider-model-vertical`, `provider-semantic-continuity`, engine context/residency tests | real OpenAI-compatible fixture invocation, typed Projection/ContextFrame/Invocation/ProviderResult lineage, continuation loss and provider/model replacement | TLS/streaming, authoritative tokenization, native KV/YVEX continuation |
| agentless Case runtime | `agentless-case-runtime`, `human-review-runtime`, engine 128-iteration residency endurance | 26 real HTTP turns, 24 controlled effects, DENY/adaptation, typed human pause/resume, single-Case runner exclusion, provider/model replacement, automatic memory repair/effect reconciliation, budget stop, bounded context and crash resume without Agent/Workflow state | distributed admission, generalized operation/carrier families |
| controlled filesystem effect | `controlled-effect-vertical`, `policy-authority-admission`, engine controlled-effect/replay tests | real provider candidate → typed Operation/DecisionBasis/Decision/policy-bound Grant → durable PREPARE → observed atomic replacement → FINALIZE/RECONCILE → second typed provider view, including crashes and fail-closed authority paths | another carrier, validity/revoke or multi-Case background recovery |
| human review/filesystem | `human-review-runtime`, engine typed-review/replay tests | policy-bound REQUIRE_REVIEW, Case-role eligible Participant APPROVE/DENY/DEFER, stale-policy rejection, no-live-runner action, provider/model replacement, R1–R6 recovery and same Operation/effect chain | authenticated OS/remote identity, generic approval workflow, review for other carriers |
| governance intake/admission | `governance-intake`, `governance-hardening`, `case-policy-materialization`, `policy-authority-admission`, engine governance/admission/LMDB tests | immutable owner-scoped artifacts, exact Case binding, EffectivePolicy v2, closed-world DecisionBasis, role/evidence eligibility, stale-basis prevention and pure inspection | authenticated publisher/owner, expiry/revoke, tenant semantics, general retention policy |
| direct filesystem bypass | `direct-filesystem-bypass` | former write command is unreachable; retained compatibility command is observation-only | a second effect path or bypass authority |
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
make endurance-agentless-case-runtime
make smoke-governance-intake
make smoke-governance-hardening
make smoke-case-policy-materialization
make smoke-policy-authority-admission
```

Read the first failing target directly. Do not mask failures caused by a dirty
worktree, missing native dependency, or absent provider.

## Manual local inspection

With built binaries and an isolated `YAI_HOME`:

```sh
./yai doctor
./yai hot status
./yai store status
./yai store summary
```

Detailed public test wrappers remain under `tests/cases/`:

- [repository health](../tests/cases/00-repository-health/README.md);
- [local runtime inspection](../tests/cases/01-local-runtime-inspection/README.md);
- [filesystem-loop manual evidence](../tests/cases/02-filesystem-loop-manual/README.md);
- [model-behavior lab evidence](../tests/cases/03-model-behavior-lab/README.md).

The last two point into `labs/`. Labs are reproducible evidence packages, not
current architecture or operational requirements.

## Historical properties requiring future regression tests

E05 closure/replay and linked identity, E07 Case-scoped provider separation,
and V11 resource attachment plus pre/effect/post lineage are now recovered for
the controlled filesystem vertical. Their broader retention/process/provider
properties remain historical specifications where no current product test
demonstrates them; the recovery mapping lives in
`refoundation/source-refoundation-3/historical-property-recovery.tsv`.

## Non-claims

These validations do not prove production readiness, provider breadth, model
quality, deterministic real-model behavior, hostile concurrent namespace
confinement, or a universal external-effect protocol beyond local
`filesystem.write`. Facts, graph, projection, hot-state, and provider outputs
remain subject to the authority limits in
[Executable architecture](architecture.md).
