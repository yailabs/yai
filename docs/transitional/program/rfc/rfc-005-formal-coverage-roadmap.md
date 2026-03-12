---
role: support
status: draft
audience: governance
owner_domain: program-rfc
id: RFC-005
decision_id: RFC-005
supersedes: []
superseded_by: []
implements: []
evidenced_by: [docs/program/reports/audit-convergence-report.md]
related: [docs/program/adr/adr-012-audit-convergence-gates.md]
---
# RFC-005 - Formal coverage roadmap for spec-critical domains

# Purpose
Captures governance-level request-for-comment context and decisions.

# Scope
Covers rationale, constraints, and acceptance direction for platform evolution.

# Relationships
- Related ADRs
- Associated implementation evidence and reports

# Canonical Role
Program support artifact; not a runtime architecture source-of-truth.

# Main Body
## Problem
Current formal coverage is uneven across domains. Some areas are modeled while others remain smoke/none, which limits confidence for stronger TRL claims.

## Scope
- In scope: Coverage priorities, domain gap list, staged property roadmap, evidence expectations.
- Out of scope: Rewriting the whole TLA model in one phase.

## Proposed Change
Define a prioritized formal roadmap that starts from protocol/control criticality, then addresses CLI, vault, and graph gaps with explicit property targets.

## Options Compared
- Option A: Risk-first roadmap based on invariant and boundary criticality.
- Option B: Surface-first roadmap by component ownership.

## Risks
- Over-expansion of formal scope. Mitigation: milestone-based slices with hard acceptance criteria.
- Weak adoption if disconnected from delivery gates. Mitigation: link roadmap outputs to CI and milestone packs.

## Rollout Sketch
1. Publish current modeled/smoke/none matrix.
2. Select next two domains for explicit property additions.
3. Bind roadmap outputs to release evidence expectations.

## Exit Criteria
- [ ] Coverage matrix and priority order are explicit.
- [ ] Next formal increments define target properties and artifacts.
- [ ] Proposal links to ADR and milestone evidence strategy.

## Traceability

- Spec anchors (if any): `../governance/control/assurance/spec_map.md`, `../governance/control/assurance/tla/YAI_KERNEL.tla`, `../governance/control/assurance/bindings/BINDING_PROTOCOL.md`, `../governance/control/assurance/bindings/BINDING_CLI.md`
- Targets ADR: `docs/program/adr/adr-006-unified-rpc.md`, `docs/program/adr/adr-011-contract-runbook-lock.md`
- Downstream runbook: `docs/archive/legacy/program/milestone-packs/runtime-baselines/operations-foundation/mp-runtime-000-root-hardening.md`
- Downstream MP: `docs/archive/legacy/program/milestone-packs/root-hardening/mp-runtime-000-root-hardening-v0-1-5.md`

## References
- `docs/program/spine.md`
- `../governance/control/assurance/spec_map.md`
- `../governance/control/assurance/traceability.v1.json`

# Related Docs
- `docs/program/rfc/README.md`
- Linked ADR and report artifacts
