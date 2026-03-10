---
id: RB-RUNBOOKS-README
status: active
adr_refs:
  - docs/program/22-adr/ADR-011-contract-baseline-lock.md
  - docs/program/22-adr/ADR-012-audit-convergence-gates.md
decisions:
  - docs/program/22-adr/ADR-011-contract-baseline-lock.md
  - docs/program/22-adr/ADR-012-audit-convergence-gates.md
---

# Runbooks

Runbooks translate decisions into phased, repeatable execution.

A good runbook must be deterministic:
- clear preconditions,
- explicit procedure,
- verifiable outcomes,
- rollback/failure handling.

## Typical runbooks in this repo

- `docs/program/23-runbooks/contract-baseline-lock.md`
- `docs/program/23-runbooks/specs-refactor-foundation.md`
- `docs/program/23-runbooks/root-hardening.md`
- `docs/program/23-runbooks/workspaces-lifecycle.md`
- `docs/program/23-runbooks/engine-attach.md`
- `docs/program/23-runbooks/data-plane.md`
- `docs/program/23-runbooks/data-plane-storage-classes.md`
- `docs/program/23-runbooks/data-plane-storage-topology.md`
- `docs/program/23-runbooks/workspace-data-sinks.md`
- `docs/program/23-runbooks/evidence-and-event-persistence.md`
- `docs/program/23-runbooks/governance-and-compliance-persistence.md`
- `docs/program/23-runbooks/authority-and-artifact-persistence.md`
- `docs/program/23-runbooks/brain-memory-and-graph-sinks.md`
- `docs/program/23-runbooks/data-surfaces-and-operator-query-model.md`
- `docs/program/23-runbooks/data-plane-qualification-and-closure.md`
- `docs/program/23-runbooks/enforcement-to-record-persistence.md`
- `docs/program/23-runbooks/graph-materialization-from-runtime-records.md`
- `docs/program/23-runbooks/db-first-read-path-cutover.md`
- `docs/program/23-runbooks/filesystem-governance-state-decommission.md`
- `docs/program/23-runbooks/filesystem-cleanup-and-archive-execution.md`
- `docs/program/23-runbooks/data-lifecycle-retention-and-tiering.md`
- `docs/program/23-runbooks/unified-runtime-topology-refoundation.md`
- `docs/program/23-runbooks/mind-redis-stm.md`
- `docs/program/23-runbooks/kernel-sovereignty.md`
- `docs/program/23-runbooks/operations.md`

## Program Convergence Backbone

Program-level target-state and wave ordering are defined in:
- `docs/program/audit-convergence/EXECUTION-PLAN-v0.1.0.md`
- `docs/program/audit-convergence/AUDIT-CONVERGENCE-MATRIX-v0.1.0.md`

Runbook phase closure should reference claim IDs from:
- `ops/evidence/validation/audits/claims/infra-grammar.v0.1.json`

## Template

- `docs/program/25-templates/runbooks/RB-000-template.md`

## Traceability expectation

Runbooks should link:
- upstream ADRs and law/spec anchors,
- downstream milestone packs as phases are delivered.
