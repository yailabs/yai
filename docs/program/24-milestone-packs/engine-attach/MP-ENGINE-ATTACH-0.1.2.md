---
id: MP-ENGINE-ATTACH-0.1.2
status: draft
runbook: docs/program/23-runbooks/engine-attach.md
phase: "0.1.2 — EA-2 workspace contract validation"
adrs:
  - docs/program/22-adr/ADR-007-workspace-isolation.md
  - docs/program/22-adr/ADR-009-engine-attachment.md
  - docs/program/22-adr/ADR-012-audit-convergence-gates.md
spec_anchors:
  - deps/yai-law/contracts/protocol/include/auth.h
  - deps/yai-law/contracts/control/schema/authority.v1.json
  - deps/yai-law/registry/commands.v1.json
claims:
  - C-DOMAIN-COVERAGE-RESOURCE
  - C-CONTEXT-PROPAGATION
evidence_commands_required:
  - tools/bin/yai-verify
  - tools/bin/yai-gate
issues:
  - "N/A: engine-attach wave sequencing"
---

# MP-ENGINE-ATTACH-0.1.2

## Metadata
- Runbook: `docs/program/23-runbooks/engine-attach.md`
- Phase: `0.1.2 — EA-2 workspace contract validation`
- Status: `draft`

## Objective
Enforce workspace-bound attach semantics (identity, isolation, path-jail interactions).

## Mandatory command outcomes
- `tools/bin/yai-verify` -> `PASS`
- `tools/bin/yai-gate` -> `PASS`

Closure policy: mandatory `SKIP` is treated as `FAIL`.

## Definition of Done
- [ ] Workspace mismatch is hard rejected.
- [ ] No cross-workspace side effects under negative tests.
- [ ] Workspace-open readiness checks include governed engine probe.
- [ ] Evidence links and logs archived.

## Execution Snapshot
- Status: `PLANNED`
- Evidence bundle: `docs/program/24-milestone-packs/engine-attach/evidence/0.1.2/`
