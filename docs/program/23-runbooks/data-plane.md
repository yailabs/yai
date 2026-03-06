---
id: RB-DATA-PLANE
title: Data Plane
status: draft
owner: runtime
effective_date: 2026-03-06
revision: 2
supersedes:
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
    - deps/yai-law/REGISTRY.md
    - deps/yai-law/contracts/control/schema/control_call.v1.json
    - deps/yai-law/contracts/control/schema/exec_reply.v1.json
    - deps/yai-law/contracts/control/schema/authority.v1.json
    - deps/yai-law/contracts/protocol/include/transport.h
    - deps/yai-law/contracts/protocol/include/yai_protocol_ids.h
    - deps/yai-law/contracts/vault/include/yai_vault_abi.h
    - deps/yai-law/contracts/vault/schema/vault_abi.json
    - deps/yai-law/registry/commands.v1.json
  test_plans:
    - yai-ops/evidence/qualification/test-plans/hardfail.md
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

# RB-DATA-PLANE - Data Plane (rev2)

## 1) Purpose
Define the enterprise baseline for YAI Data Plane delivery across the current platform stack:

`yai-law -> yai-sdk -> yai-cli -> yai -> yai-ops` (with `yai-infra` as governance/tooling factory).

This runbook aligns implementation sequencing, contract boundaries, and evidence closure for storage and stateful runtime operations.

## 2) Snapshot (as of 2026-03-06)
- Runtime topology in `yai`: Root -> Kernel -> Engine, Mind surfaces optional/planned by scope.
- Workspace-first operation is active and remains the containment boundary.
- Vault contract is pinned from `yai-law` and consumed by runtime/SDK/CLI.
- Audit convergence program is active; Data Plane remains partially closed in matrix status.
- Existing rev1 content is superseded by this rev2 governance and delivery model.

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
3. Contract authority is externalized to pinned `deps/yai-law` artifacts.
4. Deterministic error semantics must use canonical exec-reply/control schemas.
5. Data-plane changes are not complete until evidence is published in `yai-ops`.
6. If checks are mandatory in a closure phase, `SKIP` is treated as `FAIL`.

## 5) Responsibility model (current repo structure)
- `yai-law`: normative contracts, schemas, registries, ABI surfaces.
- `yai-sdk`: contract-constrained API surface for governed data operations.
- `yai-cli`: operator command surface only (no storage bypass).
- `yai`: runtime implementation (Root/Kernel/Engine behavior and enforcement).
- `yai-ops`: evidence bundles, qualification reports, official closure artifacts.
- `yai-infra`: shared governance checks, docs/tooling policy, reusable automation.

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

## 8) Delivery phases (rev2)

### DP-0: Contract and pin baseline lock
Objective:
- Ensure runtime, SDK, and CLI consume the same pinned data-plane contract surfaces.

Required outputs:
- Pin check green.
- Registry and schema references resolved to pinned `deps/yai-law`.
- No local redefinition of normative fields.

Exit criteria:
- `yai-check-pins` and repository verify suite green.
- Contract references aligned in runbook/docs and code anchors.

### DP-1: Workspace layout and manifest hardening
Objective:
- Enforce deterministic workspace data-plane skeleton creation and validation.

Required outputs:
- Manifest creation on workspace bootstrap.
- Layout validation on workspace open/use.
- Explicit error codes for missing/incompatible layout.

Exit criteria:
- Create/open/close lifecycle tests green.
- Negative tests for invalid paths and missing manifest green.

### DP-2: Authority store kernel integration
Objective:
- Stabilize workspace authority store operations under kernel governance.

Required outputs:
- Kernel-owned open/read/write/delete authority records.
- Deterministic record schema version checks.
- Bounded error mapping to canonical exec-reply payloads.

Exit criteria:
- Authority store smoke and regression tests green.
- No cross-workspace visibility under concurrent sessions.

### DP-3: Event store engine integration
Objective:
- Persist and query workspace events through governed dispatch.

Required outputs:
- Engine event append/read surfaces through kernel-mediated calls.
- Backpressure and failure semantics documented.
- Workspace-level event retention policy hooks.

Exit criteria:
- Event write/read/tail tests green.
- Failure injection tests show deterministic behavior and recovery.

### DP-4: CLI and SDK data command surface
Objective:
- Expose enterprise-safe operator/programmatic commands without direct storage coupling.

Required outputs:
- CLI data commands aligned to law registry semantics.
- SDK wrappers aligned to control/exec-reply contracts.
- Help/docs updated for command behavior and failure modes.

Exit criteria:
- End-to-end tests green: CLI/SDK -> Root -> Kernel -> Engine/Store.
- Backward-compatibility checks for existing workspace flows green.

### DP-5: Evidence closure and operational qualification
Objective:
- Close runbook phases with traceable evidence suitable for audit and partner review.

Required outputs:
- Evidence pack index and verification reports in `yai-ops`.
- Run references linked from milestone packs and audit convergence matrix.
- Open issues mapped to residual risk or closure actions.

Exit criteria:
- Qualification checks green.
- Claims and evidence pointers resolve without drift.

## 9) Verification matrix

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

## 10) Failure modes and controls
- Cross-tenant leakage risk:
  - Control: path-jail + workspace identity checks at kernel boundary.
- Contract drift risk:
  - Control: pin checks + registry/schema anchor validation.
- Runtime/backend mismatch risk:
  - Control: deterministic error mapping and startup validation.
- Non-reproducible closure risk:
  - Control: evidence publication in `yai-ops` with stable pointers.

## 11) Rollback policy
- Roll back the active DP phase branch only.
- Restore last verified baseline for workspace layout and stores.
- Re-run mandatory checks before reopening phase.
- Do not forward-port partial schema/runtime changes without pin realignment.

## 12) Traceability and closure mapping
- Audit convergence:
  - `docs/program/audit-convergence/EXECUTION-PLAN-v0.1.0.md`
  - `docs/program/audit-convergence/AUDIT-CONVERGENCE-MATRIX-v0.1.0.md`
- Related runbooks:
  - `docs/program/23-runbooks/root-hardening.md`
  - `docs/program/23-runbooks/workspaces-lifecycle.md`
  - `docs/program/23-runbooks/engine-attach.md`
- Evidence destination:
  - `yai-ops/evidence/qualification/`
  - `yai-ops/evidence/validation/`

## 13) Definition of Done (rev2)
- All DP phases closed with explicit evidence links.
- No unresolved drift between code behavior and pinned law contracts.
- Workspace data-plane operations are deterministic and isolated.
- CLI/SDK data commands are governance-routed, not storage-coupled.
- Audit convergence matrix updated from partial to closed for Data Plane scope.
