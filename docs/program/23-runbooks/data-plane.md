---

> Historical topology note: legacy labels (root/kernel/engine) may appear in historical artifacts.
> Canonical operator path is `cli -> sdk -> yai` on `~/.yai/run/control.sock`.
id: RB-DATA-PLANE
title: Data Plane Program
status: draft
owner: runtime
effective_date: 2026-03-09
revision: 4
supersedes:
  - RB-DATA-PLANE@rev3
  - RB-DATA-PLANE@rev2
  - RB-DATA-PLANE@rev1
depends_on:
  - RB-ROOT-HARDENING
  - RB-WORKSPACES-LIFECYCLE
  - RB-ENGINE-ATTACH
  - RB-MIND-REDIS-STM
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

# RB-DATA-PLANE - Data Plane Program (rev4)

## 1) Purpose
Define the canonical, governed persistence program for YAI.

Data Plane is not a backend choice. Data Plane is the persistent substrate for
`core`, `exec`, `brain`, and governance lifecycle surfaces.

## 2) Program framing

### Dominant model
- `cli -> sdk -> yai` is canonical operator path.
- Inside `yai`, responsibilities are stratified in `core`, `exec`, `brain`.
- `law` remains normative source; `ops` remains closure evidence sink.

### Declassed legacy center
- `mind-redis-stm.md` is component/backend-specific guidance, not DP center.
- Redis is a candidate backend role for transient cognition, not DP identity.

## 3) Non-negotiable invariants
1. No direct client write to storage backends.
2. Workspace scope and path-jail boundaries are mandatory.
3. Authority/lifecycle-gated state cannot be bypassed.
4. Deterministic reply semantics are mandatory.
5. Sink-first order is mandatory before rich query surfaces.

## 4) Canonical classes and ownership
DP storage classes and owner mapping are defined in:
- `docs/program/23-runbooks/data-plane-storage-classes.md`

Runtime anchors used by this program:
- `lib/core/workspace/*`
- `lib/core/authority/*`
- `lib/core/session/*`
- `lib/exec/runtime/*`
- `lib/exec/gates/storage_gate.c`
- `lib/law/mapping/decision_to_evidence.c`
- `lib/law/mapping/decision_to_audit.c`
- `lib/brain/memory/*`
- `lib/brain/memory/graph/*`
- `data/global/knowledge.db`

## 5) Sink-first execution strategy
Mandatory order:
1. class model
2. backend role model
3. storage topology/layout
4. sink contracts and write paths
5. implementation and cutover
6. read/query/operator surfaces
7. richer workspace↔graph↔workflow semantics

## 6) Program mapping (DP block, 9 deliveries)

### DP-1 — Refoundation of the Canonical Data Plane Model
Canonical model, terminology, boundaries, ownership baseline.

### DP-2 — Canonical Storage Classes and Backend Role Model
Class semantics, owner mapping, backend role fit, separation rules.

### DP-3 — Canonical Storage Topology and Persistence Layout
On-disk/in-store topology, persistence layout, migration-safe structure.

### DP-4 — Event and Evidence Sink Hardening
Event/evidence sink contracts, retention, deterministic failure semantics.

### DP-5 — Governance and Compliance Persistence Integration
Governance object/lifecycle persistence integrated with runtime boundaries.

### DP-6 — Authority and Artifact Metadata Store Integration
Authority state and artifact metadata persistence under canonical ownership.

### DP-7 — Brain Graph Sink and Transient Cognition Backend
Graph truth and transient cognition separation with explicit sink contracts.

### DP-8 — CLI/SDK Data Surfaces and Operator Query Model
Operator/programmatic query surfaces over canonical sinks, no bypass.

### DP-9 — Verification, Qualification and Pre-Pilot Data Closure
Cross-repo verification, qualification evidence, and pre-pilot closure pack.

## 7) Out of scope for current DP block
- distributed replication/HA/federation
- multi-node graph fabric
- rich cross-workspace federated query model
- full workflow persistence model
- full cockpit data fabric

## 8) Verification matrix baseline
Mandatory lanes:
- pin/contract checks against `law`
- workspace scope/path-jail checks
- lifecycle/boundary gate checks
- deterministic error/reply checks
- cross-link integrity checks in program docs

Evidence minimum:
- command outputs
- logs
- verification reports
- traceability pointers to runbook and claims

## 9) Failure modes and controls
- Cross-tenant leakage:
  - control: workspace boundary + path-jail enforcement.
- Contract drift:
  - control: pin checks + anchor verification.
- Backend-role drift:
  - control: storage-class/role matrix review gating.
- File-first regression:
  - control: sink contract enforcement and lifecycle gates.

## 10) Rollback policy
- Roll back active DP phase branch only.
- Restore last verified model or sink baseline.
- Re-run mandatory checks before phase reopen.

## 11) Traceability
- `docs/program/audit-convergence/EXECUTION-PLAN-v0.1.0.md`
- `docs/program/audit-convergence/AUDIT-CONVERGENCE-MATRIX-v0.1.0.md`
- `docs/program/23-runbooks/data-plane-storage-classes.md`
- `docs/program/23-runbooks/workspace-data-sinks.md`
- `docs/program/23-runbooks/evidence-and-event-persistence.md`
- `docs/program/23-runbooks/brain-memory-and-graph-sinks.md`

## 12) Definition of Done (program)
- DP-1..DP-9 closures contain explicit evidence links.
- No unresolved drift between code behavior and pinned contracts.
- Data-plane evolution remains sink-first and boundary-governed.
