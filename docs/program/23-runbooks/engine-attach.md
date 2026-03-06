---
id: RB-ENGINE-ATTACH
title: Engine Attach
status: active
owner: runtime
effective_date: 2026-03-06
revision: 4
supersedes:
  - RB-ENGINE-ATTACH@rev3
depends_on:
  - RB-ROOT-HARDENING
  - RB-WORKSPACES-LIFECYCLE@rev3
  - RB-DATA-PLANE@rev2
adr_refs:
  - docs/program/22-adr/ADR-009-engine-attachment.md
  - docs/program/22-adr/ADR-008-connection-lifecycle.md
  - docs/program/22-adr/ADR-006-unified-rpc.md
  - docs/program/22-adr/ADR-012-audit-convergence-gates.md
decisions:
  - docs/program/22-adr/ADR-009-engine-attachment.md
  - docs/program/22-adr/ADR-008-connection-lifecycle.md
related:
  adr:
    - docs/program/22-adr/ADR-009-engine-attachment.md
    - docs/program/22-adr/ADR-008-connection-lifecycle.md
    - docs/program/22-adr/ADR-006-unified-rpc.md
  specs:
    - deps/yai-law/contracts/protocol/include/transport.h
    - deps/yai-law/contracts/protocol/include/yai_protocol_ids.h
    - deps/yai-law/contracts/control/schema/exec_reply.v1.json
    - deps/yai-law/contracts/control/schema/authority.v1.json
    - deps/yai-law/registry/commands.v1.json
    - deps/yai-law/runtime/engine/README.md
    - deps/yai-law/contracts/vault/include/yai_vault_abi.h
    - deps/yai-law/contracts/vault/schema/vault_abi.json
  runbooks:
    - docs/program/23-runbooks/root-hardening.md
    - docs/program/23-runbooks/workspaces-lifecycle.md
    - docs/program/23-runbooks/data-plane.md
  test_plans:
    - yai-ops/evidence/qualification/test-plans/hardfail.md
  tools:
    - tools/bin/yai-check-pins
    - tools/bin/yai-verify
    - tools/bin/yai-gate
    - tools/bin/yai-suite
tags:
  - runtime
  - engine
  - attach
  - workspace
  - audit-convergence
---

# RB-ENGINE-ATTACH - Engine Attach (rev4)

## 1) Purpose
Define the enterprise baseline for attaching Engine as a governed runtime plane under Root/Kernel control, with deterministic readiness, lifecycle alignment, and audit-grade evidence.

This runbook governs semantics and closure criteria.
It is not a step-by-step command cookbook.

## 2) Snapshot (as of 2026-03-06)
- Engine is treated as shared runtime plane, not per-workspace process topology.
- Workspace context is carried by governed dispatch metadata.
- Readiness is functional (RPC probe), not socket-presence-driven.
- Workspace lifecycle and Data Plane consume engine attach as a baseline dependency.

## 3) Scope

### In scope
- Engine attach semantics under governed control plane.
- Readiness and status contract for lifecycle integration.
- Authority and isolation rules for engine-routed operations.
- Workspace operational model required by engine attach.
- Traceability and closure evidence for audit convergence.

### Out of scope
- Replacing Root/Kernel authority model.
- Defining full data-plane storage semantics (owned by RB-DATA-PLANE).
- Defining optional Mind orchestration policy as closure requirement.

## 4) Non-negotiable invariants
1. Root remains envelope-first router, not payload business authority.
2. Kernel remains authority gatekeeper for engine-bound operations.
3. Engine readiness must be functional and deterministic.
4. Workspace isolation cannot be inferred from process topology.
5. Engine attach completion requires evidence publication, not only local pass.

## 5) Architecture contract

### Runtime control chain
`CLI/SDK -> Root -> Kernel -> Engine -> Kernel -> Root -> CLI/SDK`

### Readiness contract
`READY` requires:
- root plane reachable,
- kernel plane reachable,
- engine functional probe successful in governed workspace context.

Socket exposure is informational and non-authoritative.

## 6) Workspace model (authoritative for this runbook)

### 6.1 What a workspace is physically
A workspace is a governed runtime tenant boundary, represented by:
- runtime path root: `~/.yai/run/<ws_id>/`
- workspace-scoped runtime artifacts (manifest, authority/events/log roots)
- workspace-scoped runtime state context (including vault-backed state and command context)

It is not a Linux container boundary by itself.

### 6.2 Security posture provided by workspace model
Provided:
- identity-scoped governance (`ws_id`)
- Root/Kernel authority mediation (`arming`, `role`, command semantics)
- path-jail constraints for workspace-bound storage operations
- deterministic deny/degrade behavior and auditable traceability

Not provided alone:
- kernel namespace isolation (PID/mount/net)
- cgroup-level hard resource isolation
- syscall sandbox as a workspace primitive

### 6.3 Workspace and engine attach interaction
- Engine accepts operations only through governed dispatch for a target workspace context.
- Cross-workspace operations are forbidden and treated as hard reject.
- Engine readiness is evaluated in workspace context through functional probe path.
- Workspace `OPEN` semantics depend on successful engine readiness checks.

### 6.4 Workspace lifecycle checkpoints relevant to engine attach
- `CREATED`: workspace materialized, not execution-ready
- `OPEN`: engine attach readiness checks passed in governed path
- `CLOSING`: engine drain/stop behavior enforced deterministically
- `CLOSED`: workspace inactive and retained for controlled reopen/recovery

Reference source of truth for full lifecycle semantics:
- `RB-WORKSPACES-LIFECYCLE@rev3`

## 7) Lifecycle binding (rev4 alignment)
Engine attach is lifecycle-bound, not standalone.

Binding rules:
- `OPEN` in workspace lifecycle requires engine functional readiness criteria.
- `CLOSING/CLOSED` transitions require deterministic engine drain/stop semantics.
- Engine failure cannot silently leave workspace in pseudo-open state.
- Restart path requires revalidation before restoring `OPEN` semantics.

Reference binding:
- `RB-WORKSPACES-LIFECYCLE@rev3`
- `RB-DATA-PLANE@rev2`

## 8) Authority and isolation model
- Envelope authority context (`arming`, `role`, `ws_id` semantics) is enforced at Root/Kernel boundaries.
- Engine executes only dispatched operations already gated by authority checks.
- Cross-workspace effects are forbidden; violations are hard reject + audit event.
- Degraded behavior must be explicit, bounded, and auditable.

## 9) Phase model (rev4)

<a id="phase-engine-attach-v4"></a>
### Phase engine-attach-v4 (compat anchor)
Objective:
- preserve ADR and audit linkage while formalizing shared-plane engine attach semantics.

Exit criteria:
- functional probe-based readiness integrated in status semantics;
- workspace-context dispatch verified through governed path;
- no dependency on per-workspace engine socket/process layout for qualification.

### Phase EA-1: Control-plane semantic hardening
Objective:
- harden deterministic engine start/stop/status behavior at kernel authority boundary.

Exit criteria:
- deterministic responses on success/failure;
- error mapping aligned to canonical contract payload semantics;
- no silent drop paths.

### Phase EA-2: Workspace contract validation
Objective:
- enforce workspace-bound attach semantics (identity, isolation, path-jail interactions).

Exit criteria:
- workspace mismatch is hard rejected;
- no cross-workspace side effects under negative tests;
- workspace-open readiness checks include governed engine probe.

### Phase EA-3: Lifecycle integration closure
Objective:
- close engine attach integration with workspace lifecycle open/close/restart semantics.

Exit criteria:
- lifecycle acceptance includes engine attach checks;
- degraded and restart paths documented and verified;
- integration evidence linked in closure pack.

### Phase EA-4: Data-plane coupling checks
Objective:
- verify engine attach behavior remains compatible with data-plane enforcement and recovery contracts.

Exit criteria:
- no bypass around data-plane governance path;
- restart/recovery preserves workspace/data-plane integrity guarantees;
- evidence of coupled validation published.

### Phase EA-5: Audit convergence closure
Objective:
- complete provider-domain closure obligations and traceability links.

Exit criteria:
- matrix pointers resolve for provider domain claims;
- mandatory checks are green (no SKIP for mandatory lanes);
- evidence pack indexed in `yai-ops`.

## 10) Verification policy

Mandatory lanes:
- contract/pin checks,
- engine functional probe checks,
- workspace isolation/path-jail negative tests,
- lifecycle integration tests,
- restart/degraded behavior tests,
- evidence publication checks.

Minimum evidence set:
- status snapshots,
- controlled probe outputs,
- failure-mode outputs,
- traceability pointers from phase to claim and artifact.

## 11) Failure modes and required behavior
- Probe fails with socket present:
  - `NOT_READY`, not false green.
- Unauthorized engine operation:
  - deterministic deny + audit event.
- Workspace mismatch in dispatch:
  - hard reject + no partial side effect.
- Cross-workspace target attempt:
  - hard reject + evidence event.
- Engine unavailable during open path:
  - deterministic degraded/open-block behavior per lifecycle policy.
- Recovery mismatch after restart:
  - block ready-state until revalidation succeeds.

## 12) Rollback policy
- Roll back active engine-attach phase only.
- Restore last verified lifecycle-compatible baseline.
- Re-run mandatory checks before reopening phase.
- Do not roll forward partial contract/runtime divergence.

## 13) Traceability and closure
- ADR anchors:
  - `docs/program/22-adr/ADR-009-engine-attachment.md`
  - `docs/program/22-adr/ADR-008-connection-lifecycle.md`
- Audit convergence:
  - `docs/program/audit-convergence/EXECUTION-PLAN-v0.1.0.md`
  - `docs/program/audit-convergence/AUDIT-CONVERGENCE-MATRIX-v0.1.0.md`
- Evidence destination:
  - `yai-ops/evidence/qualification/`
  - `yai-ops/evidence/validation/`

## 14) Definition of Done (rev4)
- Engine attach semantics are lifecycle-integrated and deterministic.
- Workspace model and isolation semantics are explicit and verified.
- Functional readiness contract is authoritative in status evaluation.
- Authority and isolation behavior are verified under negative tests.
- Data-plane compatibility checks are closed for attach/restart paths.
- Provider-domain audit-convergence links resolve with reproducible evidence.
