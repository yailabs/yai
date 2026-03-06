---
id: MP-WORKSPACES-LIFECYCLE-0.1.1
status: in_progress
runbook: docs/program/23-runbooks/workspaces-lifecycle.md
phase: "0.1.1 — Workspace create/open guardrails"
adrs:
  - docs/program/22-adr/ADR-007-workspace-isolation.md
  - docs/program/22-adr/ADR-008-connection-lifecycle.md
  - docs/program/22-adr/ADR-012-audit-convergence-gates.md
spec_anchors:
  - deps/yai-law/contracts/protocol/include/auth.h
  - deps/yai-law/contracts/protocol/include/transport.h
claims:
  - C-DOMAIN-COVERAGE-RESOURCE
  - C-CONTEXT-PROPAGATION
evidence_commands_required:
  - tools/bin/yai-verify
  - tools/bin/yai-suite
issues:
  - "Parent: https://github.com/yai-labs/yai/issues/217"
  - "Child: https://github.com/yai-labs/yai/issues/220"
---

# MP-WORKSPACES-LIFECYCLE-0.1.1

## Metadata
- Runbook: `docs/program/23-runbooks/workspaces-lifecycle.md`
- Phase: `0.1.1 — Workspace create/open guardrails`
- Status: `draft`

## Objective
Enforce identity, auth-context, and path-jail checks for create/open flows.

## Mandatory command outcomes
- `tools/bin/yai-verify` -> `PASS`
- `tools/bin/yai-suite` -> `PASS`

Closure policy: mandatory `SKIP` is treated as `FAIL`.

## Definition of Done
- [ ] No cross-workspace side effects under negative tests.
- [ ] Malformed `ws_id` and unauthorized create/open are deterministic rejects.
- [ ] Evidence links generated for acceptance runs.
- [ ] Traceability links remain valid.

## Execution Snapshot
- Status: `IN_PROGRESS`
- Evidence bundle: `docs/program/24-milestone-packs/workspaces-lifecycle/evidence/0.1.1/`

## Traceability
- Parent issue: `#217`
- MP issue: `#220`
