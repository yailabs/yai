---
id: MP-ENGINE-ATTACH-0.1.4
status: draft
runbook: docs/program/23-runbooks/engine-attach.md
phase: "0.1.4 — EA-4 data-plane coupling checks"
adrs:
  - docs/program/22-adr/ADR-009-engine-attachment.md
  - docs/program/22-adr/ADR-011-contract-baseline-lock.md
  - docs/program/22-adr/ADR-012-audit-convergence-gates.md
spec_anchors:
  - deps/yai-law/contracts/vault/include/yai_vault_abi.h
  - deps/yai-law/contracts/vault/schema/vault_abi.json
  - deps/yai-law/registry/commands.v1.json
claims:
  - C-DOMAIN-COVERAGE-STORAGE
  - C-KERNEL-HARD-BOUNDARY-CORE
evidence_commands_required:
  - tools/bin/yai-verify
  - tools/bin/yai-gate
issues:
  - "N/A: engine-attach wave sequencing"
---

# MP-ENGINE-ATTACH-0.1.4

## Metadata
- Runbook: `docs/program/23-runbooks/engine-attach.md`
- Phase: `0.1.4 — EA-4 data-plane coupling checks`
- Status: `draft`

## Objective
Verify engine attach behavior remains compatible with data-plane enforcement and recovery contracts.

## Mandatory command outcomes
- `tools/bin/yai-verify` -> `PASS`
- `tools/bin/yai-gate` -> `PASS`

Closure policy: mandatory `SKIP` is treated as `FAIL`.

## Definition of Done
- [ ] No bypass around data-plane governance path.
- [ ] Restart/recovery preserves workspace/data-plane integrity guarantees.
- [ ] Coupled validation evidence published.
- [ ] Cross-runbook links remain valid.

## Execution Snapshot
- Status: `PLANNED`
- Evidence bundle: `docs/program/24-milestone-packs/engine-attach/evidence/0.1.4/`
