---
role: support
status: accepted
audience: governance
owner_domain: program-adr
id: ADR-013
decision_id: ADR-013
supersedes: []
superseded_by: []
implements: []
evidenced_by: [docs/program/reports/runtime-convergence-report.md]
related: []
anchor: "#source-plane-refoundation"
applies_to: 
effective_date: 2026-03-10
law_refs: 
phase: YD-1
runbook: docs/archive/legacy/program/milestone-packs/runtime-baselines/operations-foundation/mp-runtime-000-engine-attach.md
---
# ADR-013 - Distributed Acquisition Plane, Centralized Control Plane

## Context

The unified runtime has one owner runtime (`yai`) and workspace-first truth.
The program now introduces multi-machine source acquisition without introducing
runtime federation or secondary truth owners.

## Decision

YAI adopts this v1 topology model:

- distributed acquisition plane on source nodes via standalone subordinate `yai-daemon`
- centralized control plane on owner runtime `yai`

`yai` remains canonical owner/runtime source of truth for:

- workspace lifecycle and binding
- policy/authority/evidence/enforcement final outcomes
- canonical persistence and graph truth

`yai-daemon` is a subordinate edge runtime and is explicitly not an owner
runtime or independent sovereign policy/truth authority.

`exec` is the active runtime layer for owner/daemon mediation (transport,
routing handoff, and acquisition gating).

## Rationale

- Enables multi-machine acquisition in trusted-network pre-pilot conditions.
- Preserves one authority and one canonical truth path.
- Avoids federated-runtime ambiguity during v1 rollout.
- Provides a clean architecture contract for YD-2..YD-7 implementation slices.

## Consequences

### Positive

- Clear owner/daemon boundary and naming (`yai-daemon` canonical).
- No confusion between acquisition distribution and control-plane ownership.
- Deterministic upgrade path for transport/ingest verticalization.

### Negative

- Edge autonomy is intentionally limited in v1.
- Delegated edge execution is always owner-scoped.
- Federation-style capabilities are explicitly deferred.
- Requires explicit docs/runtime alignment to avoid local-only assumptions.

## Non-goals

- Runtime federation or peer-owner mesh.
- Edge-local final policy/authority/evidence/enforcement decisions.
- Edge-local canonical graph/workspace truth.
- Network security hardening design (WireGuard/VPN) in this tranche.

## Traceability

- Architecture model:
  - `docs/architecture/distributed-acquisition-plane-model.md`
- Exec role baseline:
  - `docs/architecture/protocol/source-plane.md`
- Anchoring ADR:
  - `docs/program/adr/adr-001-single-runtime.md`

## Status

Accepted and active.
