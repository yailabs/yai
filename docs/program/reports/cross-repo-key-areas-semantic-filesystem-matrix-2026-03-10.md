# Cross-Repo Key Areas Report: Semantic-to-Filesystem Matrix

Status: draft-for-operational-baseline  
Date: 2026-03-10  
Scope: `yai`, `law`, `sdk`, `cli`  
Intent: single-source narrative to explain architecture, responsibilities, and work boundaries.

## 1) Executive Baseline

Canonical chain:

`operator -> cli -> sdk -> yai`  
`law -> (export embedded contract) -> yai runtime`

Canonical macroareas:

1. Policy/Governance engine
2. Runtime core-exec chain
3. Workspace lifecycle and containment
4. Data plane (data/db/graph/knowledge/storage)
5. Operator surface (CLI)
6. Programmatic surface (SDK)
7. Formal/scientific continuity (traceability + TLA)

This report maps each macroarea to concrete repository paths and files.

## 2) Repository Roles (Non-overlapping Authority)

| Repo | Primary Authority | What it owns | What it must not own |
|---|---|---|---|
| `law` | Normative authority | policy semantics, constraints, manifests, schemas, registries, formal continuity | runtime execution host, operator UX |
| `yai` | Runtime integration authority | ingress, dispatch, workspace binding, law resolution execution, runtime families (`core/exec/data/graph/knowledge`) | normative law authorship |
| `sdk` | Programmatic compatibility authority | typed API and runtime/readiness/workspace models for apps | normative law, runtime internals |
| `cli` | Operator compatibility authority | command parsing/rendering/help/watch and lifecycle UX | runtime enforcement logic, normative law |

## 3) Macro Semantic Areas -> Filesystem Matrix

| Macro semantic area | Semantic meaning | Primary repos | Physical anchors (dirs/files) |
|---|---|---|---|
| Policy engine (normative + execution) | Define policy semantics and execute deterministic resolution/effects | `law` + `yai` (+ surfaced by `sdk`/`cli`) | `../law/manifests/`, `../law/grammar/`, `../law/registry/`, `../law/schema/`, `../law/control-families/`, `../law/specializations/`, `../law/overlays/`; `lib/law/`, `include/yai/law/`, `embedded/law/`, `tests/unit/law/`, `tests/integration/law_resolution/` |
| Runtime core-exec chain | Runtime host lifecycle, ingress, command dispatch, authority gate, execution orchestration | `yai` | `cmd/yai/main.c`, `lib/core/`, `lib/exec/`, `lib/protocol/`, `include/yai/core/`, `include/yai/exec/`, `include/yai/protocol/` |
| Workspace model | Workspace-first binding, recovery/load, containment, state transitions, governance attach/apply boundaries | `yai` (+ contracts in `law`, models in `sdk`, UX in `cli`) | `lib/core/workspace/`, `include/yai/core/workspace.h`, `tests/integration/workspace_lifecycle/`, `docs/architecture/workspace-*.md`, `../law/schema/workspace_governance_attachment.v1.schema.json`, `../sdk/include/yai_sdk/workspace.h`, `../cli/docs/guide/workspace-consumer-role.md` |
| Data plane: storage/db/data | Persistence, records, retention/archive, query surfaces and bindings | `yai` (+ surfaced in `sdk`) | `lib/data/store/duckdb_store.c`, `lib/data/store/file_store.c`, `lib/data/records/`, `lib/data/query/`, `lib/data/lifecycle/`, `include/yai/data/`, `data/` |
| Data plane: graph | Relational truth/materialization/query lineage surfaces | `yai` (+ surfaced in `sdk`) | `lib/graph/state/`, `lib/graph/query/`, `lib/graph/materialization/`, `include/yai/graph/`, `../sdk/include/yai_sdk/graph.h` |
| Data plane: knowledge | cognition/memory/provider support and semantic substrate | `yai` (+ surfaced in `sdk`) | `lib/knowledge/cognition/`, `lib/knowledge/memory/`, `lib/knowledge/providers/`, `include/yai/knowledge/`, `../sdk/include/yai_sdk/knowledge.h` |
| SDK surface | Stable C API and typed model translation (liveness/readiness/binding/governance) | `sdk` | `../sdk/include/yai_sdk/public.h`, `../sdk/include/yai_sdk/models.h`, `../sdk/src/models/runtime_models.c`, `../sdk/src/rpc/rpc_client.c`, `../sdk/src/client/client.c`, `../sdk/src/catalog/catalog.c` |
| CLI surface | Operator commands/help/rendering/watch and lifecycle wrappers | `cli` | `../cli/src/parse/parse.c`, `../cli/src/app/control_call.c`, `../cli/src/app/lifecycle.c`, `../cli/src/render/`, `../cli/src/help/help.c`, `../cli/docs/guide/commands.md` |
| Formal/scientific continuity | Invariants traceability and formal properties linked to contracts | `law` (+ verified via runtime checks in `yai`) | `../law/formal/traceability.v1.json`, `../law/formal/tla/YAI_KERNEL.tla`, `../law/foundation/invariants/`, `../law/formal/configs/`, `tests/unit/law/test_embedded_surface_matches_publish_index.py`, `tools/bin/yai-law-compat-check` |

## 4) Policy Engine Split (How it spans multiple repos)

| Layer | Repo | Responsibility | Key files |
|---|---|---|---|
| Normative definition | `law` | policy vocabulary, precedence, overlays, governable objects, attachability constraints | `../law/grammar/rules/*.md`, `../law/grammar/schema/*.json`, `../law/manifests/governance-attachability.constraints.v1.json`, `../law/registry/governable-objects.v1.json` |
| Publish/export contract | `law` | deterministic runtime-consumable embedded export | `../law/manifests/publish.index.json`, `../law/manifests/publish.layers.json`, `../law/manifests/embedded-export.manifest.json`, `../law/tools/bin/law-export-embedded` |
| Runtime ingestion | `yai` | embedded payload sync/validation and compatibility checks | `tools/bin/yai-law-embed-sync`, `tools/bin/yai-law-compat-check`, `embedded/law/` |
| Runtime resolution | `yai` | classify -> discover -> stack build -> precedence/overlay/compliance merge -> effect/evidence mapping | `lib/law/discovery/*.c`, `lib/law/resolution/*.c`, `lib/law/resolver.c`, `lib/law/mapping/*.c`, `lib/law/policy_effects.c` |
| External consumption | `sdk`/`cli` | model and expose outcomes (without owning normative semantics) | `../sdk/include/yai_sdk/policy.h`, `../sdk/include/yai_sdk/governance.h`, `../cli/docs/reference/contract.md` |

Operational message: the policy engine is intentionally split: `law` decides what is valid, `yai` decides how it is executed at runtime.

## 5) Runtime Core-Exec Area (Concrete Runtime Chain)

| Stage | Semantic step | Runtime files |
|---|---|---|
| Ingress host | unix socket + lifecycle host process | `cmd/yai/main.c` |
| Protocol/frame gate | envelope validation, handshake precondition, authority gate, dispatch entry | `lib/core/dispatch/command_dispatch.c` |
| Session dispatch | route command to runtime/session handlers | `lib/core/session/session.c`, `lib/core/session/session_reply.c` |
| Lifecycle authority | preboot/startup/runtime capabilities | `lib/core/lifecycle/*.c` |
| Exec path | agents, orchestration, runtime engine, gates, bridge/router | `lib/exec/agents/`, `lib/exec/orchestration/`, `lib/exec/runtime/`, `lib/exec/gates/`, `lib/exec/bridge/` |
| Law resolution hook | resolve control call against embedded law | `lib/law/resolver.c`, `lib/law/resolution/*.c` |

## 6) Workspace Area (Primary Boundary of the system)

| Workspace concern | Filesystem anchors |
|---|---|
| Workspace state + registry + runtime binding | `lib/core/workspace/workspace_registry.c`, `lib/core/workspace/workspace_binding.c`, `lib/core/workspace/workspace_runtime.c` |
| Recovery/load semantics | `lib/core/workspace/workspace_recovery.c`, `docs/architecture/workspace-binding-persistence-model.md` |
| Security/containment boundary | `docs/architecture/workspace-boundary-model.md`, `docs/architecture/workspace-containment-levels.md`, `docs/architecture/workspace-security-envelope-model.md` |
| Governance attach/apply semantics | `docs/architecture/workspace-governance-targeting-and-apply-model.md`, `tests/integration/workspace_lifecycle/workspace_governance_apply_semantics_v1.sh` |
| Verification battery | `tests/integration/workspace_lifecycle/*.sh` |

## 7) Data Plane Area (DB, Knowledge, Graph, Storage)

| Subarea | Meaning | Runtime anchors |
|---|---|---|
| DB/Store | physical persistence (duckdb/file), workspace-bound paths | `lib/data/store/duckdb_store.c`, `lib/data/store/file_store.c`, `include/yai/data/store.h` |
| Data lifecycle | archive/retention policies | `lib/data/lifecycle/archive.c`, `lib/data/lifecycle/retention.c`, `include/yai/data/archive.h`, `include/yai/data/retention.h` |
| Records/query | event and governance-query surfaces | `lib/data/records/event_records.c`, `lib/data/query/inspect_query.c`, `include/yai/data/query.h` |
| Graph | relational state and summary/materialization | `lib/graph/state/graph_backend.c`, `lib/graph/query/workspace_summary.c`, `lib/graph/materialization/from_runtime_records.c` |
| Knowledge | semantic memory/provider substrate | `lib/knowledge/memory/semantic_db.c`, `lib/knowledge/memory/vector_index.c`, `lib/knowledge/providers/provider_registry.c` |

## 8) Formal/TLA -> Runtime Verification Map

Question to answer: "How does TLA in law verify/certify runtime?"

Answer path (current architecture):

1. Formal properties and invariants are declared in `law`:
   - `../law/formal/tla/YAI_KERNEL.tla`
   - `../law/formal/traceability.v1.json`
   - `../law/foundation/invariants/*.md`
2. Runtime-facing law export is constrained by manifests:
   - `../law/manifests/publish.index.json`
   - `../law/manifests/embedded-export.manifest.json`
   - `../law/manifests/runtime.entrypoints.json`
3. `yai` enforces contract consistency of embedded surface:
   - `tests/unit/law/test_embedded_surface_matches_publish_index.py`
   - `tools/bin/yai-law-compat-check`
   - `tools/bin/yai-law-embed-sync`
4. Runtime execution path applies deterministic resolution at control-call time:
   - `lib/law/resolver.c`
   - `lib/law/resolution/*.c`

Important precision: TLA in `law` is currently a formal continuity layer linked via traceability and contract discipline; runtime certification is operationalized through export-contract checks + unit/integration tests + deterministic resolver behavior in `yai`.

## 9) Work Isolation Matrix (To avoid cross-repo chaos)

| If you work on... | Touch first | Usually do not touch | Why |
|---|---|---|---|
| New policy semantics/precedence/overlay rules | `law/grammar`, `law/manifests`, `law/registry` | `cli/src`, `sdk/src` | keep normative changes isolated before consumer updates |
| Runtime enforcement behavior | `yai/lib/law`, `yai/lib/core/enforcement`, `yai/lib/core/dispatch` | `law/foundation` (unless semantics change) | execution logic belongs to runtime authority |
| Workspace behavior | `yai/lib/core/workspace`, `yai/tests/integration/workspace_lifecycle` | `cli` (except output/help alignment) | workspace boundary is runtime-owned |
| Data plane internals | `yai/lib/data`, `yai/lib/graph`, `yai/lib/knowledge` | `law/docs/authoring` | implementation and storage are runtime concerns |
| Operator UX/commands | `cli/src/parse`, `cli/src/render`, `cli/docs/guide` | `yai/lib/*` business logic | CLI must consume, not reimplement runtime |
| App integration API | `sdk/include/yai_sdk`, `sdk/src/models`, `sdk/src/rpc` | `law` normative artifacts | SDK abstracts runtime model to stable client contract |

## 10) Canonical File Set for External Explanation (Minimal package)

For CTO/PM/investor/technical onboarding, use this minimum set:

1. `README.md` (each repo): `../law/README.md`, `README.md`, `../sdk/README.md`, `../cli/README.md`
2. Runtime truth: `docs/architecture/runtime-model.md`
3. Law consumption: `docs/architecture/law-consumption-model.md`
4. Cross-repo vocabulary: `docs/architecture/cross-repo-naming-and-terminology-contract.md`
5. Workspace architecture: `docs/architecture/workspace-architecture-model.md`
6. Law publish/export contracts: `../law/manifests/publish.index.json`, `../law/manifests/embedded-export.manifest.json`, `../law/manifests/runtime.entrypoints.json`
7. SDK typed runtime model: `../sdk/include/yai_sdk/models.h`
8. CLI operator role: `../cli/docs/guide/workspace-consumer-role.md`
9. Formal continuity: `../law/formal/traceability.v1.json`, `../law/formal/tla/YAI_KERNEL.tla`

## 11) Single-Truth Statement

- `law` is the canonical semantic truth.
- `yai` is the canonical runtime truth.
- `sdk` and `cli` are canonical consumer truths (programmatic/operator), not semantic authorities.
- Workspace is the primary operational boundary.
- Runtime families are `core`, `exec`, `data`, `graph`, `knowledge`.

## 12) Residual Legacy/Transition Notes (explicitly non-primary)

| Residual surface | Where | Interpretation rule |
|---|---|---|
| Historical `brain` naming in formal/runtime legacy paths | `../law/formal/*`, `../law/runtime/brain/*`, selected old symbols in runtime internals | treat as compatibility/historical artifact names; do not use as primary architecture narrative |
| Transitional seed payload | `embedded/law/transitional/domain-family-seed/` and `../law/transitional/domain-family-seed/` | opt-in bridge only, not default normative runtime path |
| Old docs with prior topology wording | parts of historical reports and archived docs | use current cross-repo terminology contract as precedence source |
