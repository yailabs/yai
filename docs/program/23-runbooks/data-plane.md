---

> Historical topology note: this runbook may use root/kernel/engine labels from earlier waves.
> Canonical runtime ingress now is `yai` on `~/.yai/run/control.sock`.
id: RB-DATA-PLANE
title: Data Plane Program
status: draft
owner: runtime
effective_date: 2026-03-09
revision: 3
supersedes:
  - RB-DATA-PLANE@rev2
  - RB-DATA-PLANE@rev1
depends_on:
  - RB-ROOT-HARDENING
  - RB-WORKSPACES-LIFECYCLE
  - RB-ENGINE-ATTACH
adr_refs:
  - docs/program/22-adr/ADR-006-unified-rpc.md
  - docs/program/22-adr/ADR-007-workspace-isolation.md
  - docs/program/22-adr/ADR-009-engine-attachment.md
  - docs/program/22-adr/ADR-011-contract-baseline-lock.md
  - docs/program/22-adr/ADR-012-audit-convergence-gates.md
decisions:
  - docs/program/22-adr/ADR-006-unified-rpc.md
  - docs/program/22-adr/ADR-007-workspace-isolation.md
  - docs/program/22-adr/ADR-011-contract-baseline-lock.md
  - docs/program/22-adr/ADR-012-audit-convergence-gates.md
related:
  adr:
    - docs/program/22-adr/ADR-006-unified-rpc.md
    - docs/program/22-adr/ADR-007-workspace-isolation.md
    - docs/program/22-adr/ADR-009-engine-attachment.md
    - docs/program/22-adr/ADR-011-contract-baseline-lock.md
    - docs/program/22-adr/ADR-012-audit-convergence-gates.md
  specs:
    - ../law/REGISTRY.md
    - ../law/contracts/control/schema/control_call.v1.json
    - ../law/contracts/control/schema/exec_reply.v1.json
    - ../law/contracts/control/schema/authority.v1.json
    - ../law/contracts/protocol/include/transport.h
    - ../law/contracts/protocol/include/yai_protocol_ids.h
    - ../law/contracts/vault/include/yai_vault_abi.h
    - ../law/contracts/vault/schema/vault_abi.json
    - ../law/registry/commands.v1.json
  test_plans:
    - ops/evidence/qualification/test-plans/hardfail.md
  tools:
    - tools/bin/yai-check-pins
    - tools/bin/yai-verify
    - tools/bin/yai-gate
    - tools/bin/yai-suite
tags:
  - runtime
  - data-plane
  - storage
  - audit-convergence
---

# RB-DATA-PLANE - Data Plane Program (rev3)

## 1) Purpose
Define the enterprise baseline for YAI Data Plane delivery across the current platform stack:

`law -> sdk -> cli -> yai -> ops` (with `infra` as governance/tooling factory).

This runbook aligns implementation sequencing, contract boundaries, and evidence closure for storage and stateful runtime operations.

## 2) Snapshot (as of 2026-03-09)
- Runtime topology in `yai`: Root -> Kernel -> Engine, Mind surfaces optional/planned by scope.
- Workspace-first operation is active and remains the containment boundary.
- Vault contract is pinned from `law` and consumed by runtime/SDK/CLI.
- Audit convergence program is active; Data Plane remains partially closed in matrix status.
- Existing rev1/rev2 content is superseded by this rev3 program mapping.

## 3) Scope

### In scope
- Workspace-scoped storage layout and manifest discipline.
- Kernel authority store lifecycle and deterministic path governance.
- Engine event/data persistence surfaces governed by kernel authority.
- CLI/SDK data commands routed only through Root/Kernel governance.
- Evidence and qualification outputs required for phase closure.

### Out of scope
- Multi-node/distributed data-plane orchestration.
- Cross-workspace federated queries.
- Production-grade HA/replication design.
- Mind-dependent data features as mandatory closure condition.

## 4) Non-negotiable invariants
1. No direct client access to storage backends.
2. All operations must be workspace-scoped and path-jail compliant.
3. Contract authority is externalized to pinned `../law` artifacts.
4. Deterministic error semantics must use canonical exec-reply/control schemas.
5. Data-plane changes are not complete until evidence is published in `ops`.
6. If checks are mandatory in a closure phase, `SKIP` is treated as `FAIL`.

## 5) Responsibility model (current repo structure)
- `law`: normative contracts, schemas, registries, ABI surfaces.
- `sdk`: contract-constrained API surface for governed data operations.
- `cli`: operator command surface only (no storage bypass).
- `yai`: runtime implementation (Root/Kernel/Engine behavior and enforcement).
- `ops`: evidence bundles, qualification reports, official closure artifacts.
- `infra`: shared governance checks, docs/tooling policy, reusable automation.

## 6) Canonical control and data flow

### Control path (mandatory)
`CLI/SDK -> Root -> Kernel -> Engine/Store -> Kernel -> Root -> CLI/SDK`

### Data-plane authority rule
- Kernel is the authority plane for workspace isolation and privileged data ops.
- Engine executes data operations only after governed dispatch.
- Vault state is runtime operational state, not a bypass channel around governance.

## 7) Workspace storage baseline

Workspace root:

```text
~/.yai/run/<ws_id>/
├── manifest.json
├── authority/
├── events/
├── engine/
└── logs/
```

Baseline rules:
- `manifest.json` is mandatory and versioned.
- Authority and event storage must be created/validated by governed runtime flows.
- Layout creation and migration are idempotent.
- All paths must resolve under workspace root with canonical path-jail helpers.

## 8) Program mapping (DP block, 9 deliveries)

This block is the canonical program map for the next data-plane wave. It is
intentionally high-level, sequential, and cross-repo readable.

### DP-1 — Refoundation of the Canonical Data Plane Model
Objective:
- Define canonical data-plane model, boundaries, and invariants.
Outputs:
- Refounded architecture model, terminology, and interfaces baseline.

### DP-2 — Canonical Storage Classes and Backend Role Model
Objective:
- Define storage classes and ownership boundaries (runtime/kernel/engine).
Outputs:
- Backend role matrix and class-to-responsibility mapping.

### DP-3 — Canonical Storage Topology and Persistence Layout
Objective:
- Formalize persistence layout and workspace-scoped topology.
Outputs:
- Stable on-disk topology model and deterministic layout contract.

### DP-4 — Event and Evidence Sink Hardening
Objective:
- Harden event/evidence sinks with deterministic write/read guarantees.
Outputs:
- Sink controls, retention hooks, failure semantics, and verification lanes.

### DP-5 — Governance and Compliance Persistence Integration
Objective:
- Persist governance/compliance state with canonical linkage to law/runtime.
Outputs:
- Governed persistence surfaces and compatibility constraints.

### DP-6 — Authority and Artifact Metadata Store Integration
Objective:
- Integrate authority state and artifact metadata stores under kernel control.
Outputs:
- Deterministic authority/artifact metadata persistence contract.

### DP-7 — Brain Graph Sink and Transient Cognition Backend
Objective:
- Introduce bounded graph sink/transient cognition backend without scope creep.
Outputs:
- Controlled graph sink model and transient backend interfaces.

### DP-8 — CLI/SDK Data Surfaces and Operator Query Model
Objective:
- Expose operator/programmatic data surfaces aligned with runtime governance.
Outputs:
- CLI/SDK query model, response semantics, and usage guidance.

### DP-9 — Verification, Qualification and Pre-Pilot Data Closure
Objective:
- Close the block with reproducible verification and qualification evidence.
Outputs:
- Cross-repo verification pack, qualification matrix, and pre-pilot closure.

## 9) Immediate start scope (DP-1 now)

DP-1 starts immediately with these non-negotiable targets:
- canonical data-plane model document as single source of truth,
- explicit scope boundaries (in/out) for this DP block,
- invariants and terminology normalization across `law`, `yai`, `sdk`, `cli`,
- baseline verification hooks to prevent model drift before DP-2.

## 10) Verification matrix

Mandatory verification lanes per phase:
- Build and baseline runtime checks.
- Contract and pin checks.
- Workspace isolation and path-jail tests.
- Data command integration tests (CLI and SDK).
- Failure-mode checks: malformed payload, unauthorized op, storage unavailable, quota/authority constraints.

Evidence minimums:
- command outputs,
- logs,
- verification reports,
- traceability pointers to runbook phase and claim IDs.

## 11) Failure modes and controls
- Cross-tenant leakage risk:
  - Control: path-jail + workspace identity checks at kernel boundary.
- Contract drift risk:
  - Control: pin checks + registry/schema anchor validation.
- Runtime/backend mismatch risk:
  - Control: deterministic error mapping and startup validation.
- Non-reproducible closure risk:
  - Control: evidence publication in `ops` with stable pointers.

## 12) Rollback policy
- Roll back the active DP phase branch only.
- Restore last verified baseline for workspace layout and stores.
- Re-run mandatory checks before reopening phase.
- Do not forward-port partial schema/runtime changes without pin realignment.

## 13) Traceability and closure mapping
- Audit convergence:
  - `docs/program/audit-convergence/EXECUTION-PLAN-v0.1.0.md`
  - `docs/program/audit-convergence/AUDIT-CONVERGENCE-MATRIX-v0.1.0.md`
- Related runbooks:
  - `docs/program/23-runbooks/root-hardening.md`
  - `docs/program/23-runbooks/workspaces-lifecycle.md`
  - `docs/program/23-runbooks/engine-attach.md`
- Evidence destination:
  - `ops/evidence/qualification/`
  - `ops/evidence/validation/`

## 14) Definition of Done (rev3)
- All DP-1..DP-9 deliveries closed with explicit evidence links.
- No unresolved drift between code behavior and pinned law contracts.
- Workspace data-plane operations are deterministic and isolated.
- CLI/SDK data commands are governance-routed, not storage-coupled.
- Audit convergence matrix updated from partial to closed for Data Plane scope.
