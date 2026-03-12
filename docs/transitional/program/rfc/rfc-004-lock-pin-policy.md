---
role: support
status: draft
audience: governance
owner_domain: program-rfc
id: RFC-004
decision_id: RFC-004
supersedes: []
superseded_by: []
implements: []
evidenced_by: [docs/program/reports/audit-convergence-report.md]
related: [docs/program/adr/adr-011-contract-runbook-lock.md]
---
# RFC-004 - Contract baseline lock and cross-repo pin policy

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
Cross-repo updates can become inconsistent when pins are not updated in lockstep with contract changes, reducing trust in evidence and delivery claims.

## Scope
- In scope: Baseline definition, pin update rules, CI anti-drift gates, cross-repo coordination points.
- Out of scope: Detailed runtime topology and per-command behavior.

## Proposed Change
Define a formal baseline-lock policy with explicit pin responsibilities for `yai`, `cli`, and `governance`, including required checks before milestone closure.

## Options Compared
- Option A: Strict lockstep pin policy with mandatory checks.
- Option B: Soft pin policy with best-effort sync windows.

## Risks
- Coordination cost across repos. Mitigation: scripted sync + clear owner model.
- Slower merge velocity for contract-touching changes. Mitigation: narrow baseline scopes per milestone.

## Rollout Sketch
1. Define baseline contract manifest.
2. Define owner/responsibility matrix for pin updates.
3. Enforce checks before merge on contract-touching PRs.

## Exit Criteria
- [ ] Pin policy is explicit for each consumer repo.
- [ ] Mandatory pre-merge checks are documented.
- [ ] ADR-011 acceptance criteria can reference this proposal directly.

## Traceability

- Spec anchors (if any): `../governance/control/assurance/traceability.v1.json`, `../governance/control/assurance/spec_map.md`, `../governance/foundation/invariants/I-001-traceability.md`, `../governance/foundation/invariants/I-007-compliance-context-required.md`
- Targets ADR: `docs/program/adr/adr-011-contract-runbook-lock.md`
- Downstream runbook: `docs/archive/legacy/program/milestone-packs/runtime-baselines/operations-foundation/mp-runtime-000-root-hardening.md`
- Downstream MP: `docs/archive/legacy/program/milestone-packs/root-hardening/mp-runtime-000-root-hardening-v0-1-5.md`

## References
- `docs/program/spine.md`
- `docs/program/adr/adr-011-contract-runbook-lock.md`
- `../infra/tools/bin/governance-sync`

# Related Docs
- `docs/program/rfc/README.md`
- Linked ADR and report artifacts
