---
id: MP-WORKSPACES-LIFECYCLE-0.1.0
status: draft
runbook: docs/program/23-runbooks/workspaces-lifecycle.md
phase: "0.1.0 — Workspace layout baseline"
adrs:
  - docs/program/22-adr/ADR-007-workspace-isolation.md
  - docs/program/22-adr/ADR-010-boot-entrypoint.md
  - docs/program/22-adr/ADR-012-audit-convergence-gates.md
spec_anchors:
  - deps/yai-law/contracts/control/schema/control_plane.v1.json
  - deps/yai-law/contracts/vault/include/yai_vault_abi.h
claims:
  - C-DOMAIN-COVERAGE-RESOURCE
  - C-KERNEL-HARD-BOUNDARY-CORE
evidence_commands_required:
  - tools/bin/yai-check-pins
  - tools/bin/yai-verify
  - tools/bin/yai-gate
issues:
  - "N/A: workspace-lifecycle phase sequencing"
---

# MP-WORKSPACES-LIFECYCLE-0.1.0

## Metadata
- Runbook: `docs/program/23-runbooks/workspaces-lifecycle.md`
- Phase: `0.1.0 — Workspace layout baseline`
- Status: `draft`

## Objective
Establish deterministic workspace skeleton and manifest governance contract.

## Mandatory command outcomes
- `tools/bin/yai-check-pins` -> `PASS`
- `tools/bin/yai-verify` -> `PASS`
- `tools/bin/yai-gate` -> `PASS`

Closure policy: mandatory `SKIP` is treated as `FAIL`.

## Definition of Done
- [ ] Layout and manifest are idempotent.
- [ ] Invalid/partial layout is hard rejected.
- [ ] Trace events emitted for lifecycle creation path.
- [ ] MP links from runbook phase and matrix remain valid.

## Execution Snapshot
- Status: `PLANNED`
- Evidence bundle: `docs/program/24-milestone-packs/workspaces-lifecycle/evidence/0.1.0/`
