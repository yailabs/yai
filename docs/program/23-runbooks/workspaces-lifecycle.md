---
id: RB-WORKSPACES-LIFECYCLE
title: Workspace Lifecycle
status: draft
revision: 4
owner: runtime
effective_date: 2026-03-06
supersedes:
  - RB-WORKSPACES-LIFECYCLE@rev3
scope_grade: SC102

depends_on:
  - RB-ROOT-HARDENING
  - RB-ENGINE-ATTACH@rev4
  - RB-DATA-PLANE@rev2

adr_refs:
  - docs/program/22-adr/ADR-006-unified-rpc.md
  - docs/program/22-adr/ADR-007-workspace-isolation.md
  - docs/program/22-adr/ADR-009-engine-attachment.md
  - docs/program/22-adr/ADR-011-contract-baseline-lock.md
  - docs/program/22-adr/ADR-012-audit-convergence-gates.md

related:
  specs:
    - deps/yai-law/contracts/control/schema/control_plane.v1.json
    - deps/yai-law/contracts/control/schema/control_call.v1.json
    - deps/yai-law/contracts/control/schema/exec_reply.v1.json
    - deps/yai-law/contracts/protocol/include/transport.h
    - deps/yai-law/contracts/protocol/include/auth.h
    - deps/yai-law/contracts/protocol/include/yai_protocol_ids.h
    - deps/yai-law/registry/commands.v1.json
    - deps/yai-law/contracts/vault/include/yai_vault_abi.h
    - deps/yai-law/contracts/vault/schema/vault_abi.json
  runbooks:
    - docs/program/23-runbooks/root-hardening.md
    - docs/program/23-runbooks/engine-attach.md
    - docs/program/23-runbooks/data-plane.md
    - docs/program/23-runbooks/mind-redis-stm.md
  maps:
    - docs/program/23-runbooks/workspaces-lifecycle-command-map.v2.md
  evidence:
    - yai-ops/evidence/qualification/
    - yai-ops/evidence/validation/

tools:
  - tools/bin/yai-check-pins
  - tools/bin/yai-verify
  - tools/bin/yai-gate
  - tools/bin/yai-suite

tags:
  - runtime
  - workspace
  - lifecycle
  - isolation
  - audit-convergence
---

# RB-WORKSPACES-LIFECYCLE - Workspace Lifecycle (rev4)

## 1) Purpose
Define the enterprise operational model for workspace lifecycle in YAI, including physical workspace model, security posture, lifecycle governance, and integration with Engine Attach and Data Plane.

This runbook defines lifecycle and security semantics.
Command inventory remains in the dedicated command-map file.

## 2) Snapshot (as of 2026-03-06)
- Runtime topology in `yai`: Root -> Kernel -> Engine (Mind optional by scope).
- Workspace-first operation is active and is the primary governance boundary.
- Engine attach and data-plane checks are mandatory dependencies for lifecycle closure.

## 3) Scope

### In scope
- Workspace lifecycle state machine and transition governance.
- Physical workspace model and runtime artifacts.
- Security posture actually provided by current workspace model.
- Isolation/path-jail/authority/traceability constraints.
- Runtime-plane integration and closure criteria.

### Out of scope
- Full command catalog documentation.
- Multi-node orchestration and federated cross-workspace operations.
- Mandatory SC103 (Mind) behavior for SC102 closure.

## 4) Workspace model (physical and security baseline)

### 4.1 What a workspace is physically
A workspace is a governed runtime tenant boundary represented by:
- runtime root path: `~/.yai/run/<ws_id>/`
- workspace artifacts: manifest, authority/events/log roots
- workspace-scoped runtime state context (including vault-backed runtime state)

A workspace is not a Linux container boundary by itself.

### 4.2 Security provided by current workspace model
Provided:
- tenant context isolation by `ws_id`
- authority mediation by Root/Kernel (`arming`, `role`, command governance)
- path-jail constraints for workspace storage operations
- deterministic deny/degrade behavior with traceability evidence

Not provided by workspace alone:
- namespace/kernel isolation (PID, mount, network) as containers provide
- cgroup hard resource isolation
- syscall-level sandboxing (seccomp/AppArmor/SELinux) as a native workspace primitive
- strong multi-tenant hostile isolation by kernel boundary (zero-trust by kernel isolation)

### 4.3 Enterprise deployment implication
For high-assurance or hostile multi-tenant scenarios, workspace governance should be layered with infrastructure isolation (container/VM/network policy), not treated as a replacement.

## 5) Core invariants
1. Workspace isolation is mandatory and fail-closed.
2. All storage/effect operations remain mediated by governance plane.
3. Lifecycle transitions are deterministic and auditable.
4. No direct CLI/SDK storage mutation path.
5. Evidence is required for lifecycle actions touching effects-out.
6. Contract authority stays pinned to `deps/yai-law`; local semantic drift is non-compliant.

## 6) Architecture and role boundaries

### Root
- ingress, envelope checks, routing, handshake lifecycle enforcement.

### Kernel
- workspace authority, identity validation, path-jail enforcement, transition gatekeeper.

### Engine
- execution and event/data surfaces under kernel-mediated dispatch.
- lifecycle visibility required; lifecycle authority not delegated to engine.

### Mind (optional SC103)
- advisory surface only; commits remain governance-gated.

## 7) Canonical lifecycle model

### State set
- `NON_EXISTENT`
- `CREATED`
- `OPEN`
- `CLOSING`
- `CLOSED`
- `DESTROYED`

### Allowed transitions
- `NON_EXISTENT -> CREATED -> OPEN -> CLOSED -> DESTROYED`
- `OPEN -> CLOSING -> CLOSED`

### Forbidden transitions
- `OPEN -> DESTROYED` without controlled close path
- `CLOSED -> OPEN` without revalidation
- any transition bypassing identity/path-jail/authority checks

## 8) Engine Attach integration (mandatory in SC102 baseline)

Engine attach is a lifecycle concern.

Integration rules:
- `OPEN` requires runtime readiness under Root/Kernel governance and functional engine probe semantics.
- Engine socket exposure is informational; readiness is functional and governance-mediated.
- `OPEN -> CLOSING -> CLOSED` includes deterministic engine drain/stop behavior.
- Engine failure cannot leave workspace in pseudo-open state.

Operational implication:
- lifecycle acceptance is incomplete without verified engine-attach semantics.

## 9) Data Plane integration (rev2)

Workspace lifecycle consumes Data Plane contract from `RB-DATA-PLANE@rev2`.

Required alignment:
- workspace layout/manifest creation and validation
- authority/events/log roots bound to workspace path-jail
- restart/recovery validates data-plane compatibility before `OPEN`

## 10) Phase model (rev4)

<a id="phase-0-1-0-workspace-layout"></a>
### Phase 0.1.0 - Workspace layout baseline
Objective:
- establish deterministic workspace skeleton and manifest governance contract.

Exit criteria:
- layout and manifest are idempotent;
- invalid/partial layout is hard-rejected;
- trace events emitted for lifecycle creation path.

<a id="phase-0-1-1-ws-create-guardrails"></a>
### Phase 0.1.1 - Workspace create/open guardrails
Objective:
- enforce identity, auth-context, and path-jail checks for create/open flows.

Exit criteria:
- no cross-workspace side effects under negative tests;
- malformed `ws_id` and unauthorized create/open are deterministic rejects;
- evidence links generated for acceptance runs.

### Phase 0.1.2 - Engine-attached open semantics
Objective:
- bind `OPEN` semantics to engine-attach functional readiness.

Exit criteria:
- lifecycle open reports governance-ready runtime status;
- functional engine probe integrated in open status;
- degraded mode behavior explicit and audited.

### Phase 0.1.3 - Controlled close/restart and pending state safety
Objective:
- harden `CLOSING/CLOSED` with deterministic drain, pending/quarantine persistence, and restart consistency.

Exit criteria:
- pending state survives restart when policy requires;
- close/restart does not lose authority-critical state;
- recovery path emits traceable evidence.

### Phase 0.1.4 - Audit convergence closure pack
Objective:
- close lifecycle scope with reproducible evidence and traceability links.

Exit criteria:
- phase evidence linked in `yai-ops`;
- convergence matrix pointers resolve;
- no mandatory check in `SKIP` state.

## 11) Verification and evidence policy

Mandatory lanes:
- contract/pin verification,
- lifecycle transition tests,
- workspace isolation/path-jail negative tests,
- engine-attach integration checks,
- restart/recovery checks,
- evidence publication checks.

Minimum evidence set:
- command outputs,
- transition logs,
- verification reports,
- traceability pointers (`phase -> claim -> evidence artifact`).

## 12) Failure modes and controls
- identity/path-jail violation:
  - deny transition, emit evidence, keep workspace safe.
- engine readiness mismatch:
  - block `OPEN` or set deterministic degraded state; never false green.
- data-plane compatibility failure:
  - prevent transition to `OPEN`, require remediation.
- unauthorized lifecycle action:
  - deterministic deny + audit, no side effects.

## 13) Rollback and recovery
- rollback active lifecycle phase only.
- restore last verified lifecycle baseline and re-run mandatory checks.
- no silent rollback of pinned contracts.
- recovery to `OPEN` requires full revalidation: workspace + engine attach + data-plane compatibility.

## 14) Command-map boundary
This runbook defines lifecycle and security semantics.
The exhaustive command inventory remains in:

- `docs/program/23-runbooks/workspaces-lifecycle-command-map.v2.md`

## 15) Definition of Done (rev4)
- lifecycle state machine enforced and auditable.
- workspace physical/security model explicit and verified.
- engine attach semantics integrated in lifecycle acceptance.
- data-plane compatibility checks integrated before `OPEN`.
- isolation/path-jail guarantees verified under negative tests.
- evidence published and traceability links resolve in audit convergence artifacts.
