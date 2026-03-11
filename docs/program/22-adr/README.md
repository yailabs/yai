---
id: ADR-README
status: active
law_refs:
  - ../law/foundation/invariants/I-001-traceability.md
---
# ADR Index

Architecture Decision Records (ADRs) capture irreversible or high-impact technical choices.

An ADR should answer:
- what was decided,
- why alternatives were rejected,
- what consequences are accepted,
- which law/spec anchors govern the decision.

## Canonical ADR set

- `ADR-001-single-runtime.md`
- `ADR-002-root-entrypoint.md`
- `ADR-003-kernel-authority.md`
- `ADR-004-engine-execution.md`
- `ADR-005-mind-proposer.md`
- `ADR-006-unified-rpc.md`
- `ADR-007-workspace-isolation.md`
- `ADR-008-connection-lifecycle.md`
- `ADR-009-engine-attachment.md`
- `ADR-010-boot-entrypoint.md`
- `ADR-011-contract-baseline-lock.md`
- `ADR-012-audit-convergence-gates.md`
- `ADR-013-distributed-acquisition-centralized-control.md`
- `ADR-014-secure-peering-plane.md`
- `ADR-015-daemon-architecture-refoundation-slice.md`
- `ADR-016-global-to-edge-policy-hierarchy-lock.md`
- `ADR-017-delegated-edge-enforcement-model.md`
- `ADR-018-process-and-asset-runtime-observation-model.md`
- `ADR-019-edge-binding-and-action-point-model.md`
- `ADR-020-workspace-authority-and-truth-plane.md`
- `ADR-021-workspace-to-edge-policy-distribution.md`
- `ADR-022-edge-policy-validity-expiry-refresh.md`
- `ADR-023-governed-sovereign-mesh-foundation.md`
- `ADR-024-mesh-discovery-foundation.md`
- `ADR-025-mesh-coordination-foundation.md`
- `ADR-026-sovereign-mesh-authority-foundation.md`
- `ADR-027-secure-overlay-transport-plane.md`
- `ADR-028-owner-remote-peer-ingress.md`
- `ADR-029-overlay-integration.md`
- `ADR-030-source-and-edge-query-surfaces.md`
- `ADR-031-unified-graph-workspace-edge-runtime.md`
- `ADR-032-ai-grounding-governed-case-state.md`

## Template policy

The ADR template is canonical only in:
- `docs/program/25-templates/adr/ADR-000-template.md`

No template copy should be kept inside this ADR directory.

## Maintenance notes

When adding a new ADR:
- update this index,
- ensure law/spec references are explicit,
- add downstream runbook linkage when available.

Program-level convergence governance is defined in:
- `docs/program/audit-convergence/EXECUTION-PLAN-v0.1.0.md`
