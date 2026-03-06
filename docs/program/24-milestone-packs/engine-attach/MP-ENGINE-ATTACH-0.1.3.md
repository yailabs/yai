---
id: MP-ENGINE-ATTACH-0.1.3
status: draft
runbook: docs/program/23-runbooks/engine-attach.md
phase: "0.1.3 — EA-3 lifecycle integration closure"
adrs:
  - docs/program/22-adr/ADR-008-connection-lifecycle.md
  - docs/program/22-adr/ADR-009-engine-attachment.md
  - docs/program/22-adr/ADR-012-audit-convergence-gates.md
spec_anchors:
  - deps/yai-law/contracts/control/schema/control_call.v1.json
  - deps/yai-law/contracts/control/schema/exec_reply.v1.json
  - deps/yai-law/contracts/protocol/include/transport.h
claims:
  - C-CONTEXT-PROPAGATION
  - C-KERNEL-HARD-BOUNDARY-CORE
evidence_commands_required:
  - tools/bin/yai-verify
  - tools/bin/yai-suite
issues:
  - "N/A: engine-attach wave sequencing"
---

# MP-ENGINE-ATTACH-0.1.3

## Metadata
- Runbook: `docs/program/23-runbooks/engine-attach.md`
- Phase: `0.1.3 — EA-3 lifecycle integration closure`
- Status: `draft`

## Objective
Close engine attach integration with workspace lifecycle open/close/restart semantics.

## Mandatory command outcomes
- `tools/bin/yai-verify` -> `PASS`
- `tools/bin/yai-suite` -> `PASS`

Closure policy: mandatory `SKIP` is treated as `FAIL`.

## Definition of Done
- [ ] Lifecycle acceptance includes engine attach checks.
- [ ] Degraded and restart paths documented and verified.
- [ ] Integration evidence linked in closure pack.
- [ ] Traceability links remain valid.

## Execution Snapshot
- Status: `PLANNED`
- Evidence bundle: `docs/program/24-milestone-packs/engine-attach/evidence/0.1.3/`
