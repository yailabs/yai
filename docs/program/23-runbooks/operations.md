---
id: RB-OPERATIONS
title: Operations
status: draft
owner: runtime
effective_date: 2026-02-19
revision: 1
supersedes: []
depends_on:
  - RB-ROOT-HARDENING
  - RB-WORKSPACES-LIFECYCLE
adr_refs:
  - docs/program/22-adr/ADR-003-kernel-authority.md
  - docs/program/22-adr/ADR-006-unified-rpc.md
decisions:
  - docs/program/22-adr/ADR-003-kernel-authority.md
  - docs/program/22-adr/ADR-006-unified-rpc.md
related:
  adr: []
  specs: []
  test_plans:
    - ops/evidence/qualification/test-plans/hardfail.md
  tools:
    - tools/bin/yai-gate
    - tools/bin/yai-suite
    - tools/bin/yai-verify
tags:
  - runtime
  - ops
---

# RB-OPERATIONS — Operations

## 1) Purpose
Define the operational control loop for running, validating, and recovering the YAI runtime stack in deterministic fashion.

## 2) Preconditions
- [ ] Required binaries are built and available.
- [ ] Workspace/runtime directories are writable.
- [ ] Core verify/gate scripts are executable.

## 3) Inputs
- Runtime commands: `yai up/down/status`
- Gate commands: `tools/bin/yai-gate`, `tools/bin/yai-suite`, `tools/bin/yai-verify`
- Test plans: `ops/evidence/qualification/test-plans/hardfail.md`

## 4) Procedure
1. Start from clean runtime state.
2. Boot baseline services and verify core health.
3. Execute gate/suite levels required by target phase.
4. Capture logs/evidence and close only on deterministic pass.

## 5) Verification
- Core health checks pass (`up/status/ping`).
- Required verify/gate commands exit `0`.
- Evidence artifacts are attached for audit/review.

## 6) Failure Modes
- Symptom: runtime processes remain in partial/dirty state.
  - Fix: force cleanup, validate sockets/pids, restart baseline.
- Symptom: gate pass without required evidence.
  - Fix: mark run failed and require explicit artifact capture.

## 7) Rollback
- Stop runtime processes and clear active run state for the affected workspace.
- Revert active operational change set to last known good baseline.
- Re-run core checks before resuming.

## 8) References
- Test plans: `ops/evidence/qualification/test-plans/hardfail.md`
- Ops scripts: `tools/bin/yai-gate list`

## Traceability
- ADR refs (ops-only runbook): none mandatory.
- MPs (filled as phases ship): `docs/program/24-milestone-packs/...`

## 9) Secure Peering Baseline

- LAN/trusted-only deployments can validate source-plane behavior but are not Internet-grade.
- Customer-grade remote owner/peer operation requires secure peering baseline per
  `docs/program/23-runbooks/secure-peering-deployment-baseline.md`.
- Do not treat protocol success over unsecured remote exposure as acceptance evidence.
- NP-4 operational bootstrap is defined in
  `docs/program/23-runbooks/owner-peer-overlay-bootstrap.md`.
