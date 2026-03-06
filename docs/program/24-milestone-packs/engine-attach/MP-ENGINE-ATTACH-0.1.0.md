---
id: MP-ENGINE-ATTACH-0.1.0
status: draft
runbook: docs/program/23-runbooks/engine-attach.md
phase: "0.1.0 — phase-engine-attach-v4 (compat anchor)"
adrs:
  - docs/program/22-adr/ADR-009-engine-attachment.md
  - docs/program/22-adr/ADR-008-connection-lifecycle.md
  - docs/program/22-adr/ADR-012-audit-convergence-gates.md
spec_anchors:
  - deps/yai-law/contracts/protocol/include/transport.h
  - deps/yai-law/contracts/protocol/include/yai_protocol_ids.h
  - deps/yai-law/contracts/control/schema/exec_reply.v1.json
claims:
  - C-DOMAIN-COVERAGE-PROVIDERS
  - C-CONTEXT-PROPAGATION
evidence_commands_required:
  - tools/bin/yai-check-pins
  - tools/bin/yai-verify
  - tools/bin/yai-gate
issues:
  - "N/A: engine-attach wave sequencing"
---

# MP-ENGINE-ATTACH-0.1.0

## Metadata
- Runbook: `docs/program/23-runbooks/engine-attach.md`
- Phase: `0.1.0 — phase-engine-attach-v4 (compat anchor)`
- Status: `draft`

## Objective
Preserve ADR/audit linkage while formalizing shared-plane engine attach semantics.

## Mandatory command outcomes
- `tools/bin/yai-check-pins` -> `PASS`
- `tools/bin/yai-verify` -> `PASS`
- `tools/bin/yai-gate` -> `PASS`

Closure policy: mandatory `SKIP` is treated as `FAIL`.

## Definition of Done
- [ ] Functional probe-based readiness integrated in status semantics.
- [ ] Workspace-context dispatch verified through governed path.
- [ ] No dependency on per-workspace engine socket/process layout for qualification.
- [ ] MP links from runbook phase and matrix remain valid.

## Execution Snapshot
- Status: `PLANNED`
- Evidence bundle: `docs/program/24-milestone-packs/engine-attach/evidence/0.1.0/`
