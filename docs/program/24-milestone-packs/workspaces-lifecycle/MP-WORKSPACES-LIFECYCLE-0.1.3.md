---
id: MP-WORKSPACES-LIFECYCLE-0.1.3
status: draft
runbook: docs/program/23-runbooks/workspaces-lifecycle.md
phase: "0.1.3 — Controlled close/restart and pending-state safety"
adrs:
  - docs/program/22-adr/ADR-008-connection-lifecycle.md
  - docs/program/22-adr/ADR-011-contract-baseline-lock.md
  - docs/program/22-adr/ADR-012-audit-convergence-gates.md
spec_anchors:
  - deps/yai-law/contracts/control/schema/control_call.v1.json
  - deps/yai-law/contracts/control/schema/exec_reply.v1.json
claims:
  - C-KERNEL-HARD-BOUNDARY-CORE
  - C-DOMAIN-COVERAGE-RESOURCE
evidence_commands_required:
  - tools/bin/yai-verify
  - tools/bin/yai-suite
issues:
  - "N/A: workspace-lifecycle phase sequencing"
---

# MP-WORKSPACES-LIFECYCLE-0.1.3

## Metadata
- Runbook: `docs/program/23-runbooks/workspaces-lifecycle.md`
- Phase: `0.1.3 — Controlled close/restart and pending-state safety`
- Status: `draft`

## Objective
Harden `CLOSING/CLOSED` with deterministic drain, pending/quarantine persistence, and restart consistency.

## Mandatory command outcomes
- `tools/bin/yai-verify` -> `PASS`
- `tools/bin/yai-suite` -> `PASS`

Closure policy: mandatory `SKIP` is treated as `FAIL`.

## Definition of Done
- [ ] Pending state survives restart when policy requires.
- [ ] Close/restart does not lose authority-critical state.
- [ ] Recovery path emits traceable evidence.
- [ ] Cross-runbook links remain valid.

## Execution Snapshot
- Status: `PLANNED`
- Evidence bundle: `docs/program/24-milestone-packs/workspaces-lifecycle/evidence/0.1.3/`
