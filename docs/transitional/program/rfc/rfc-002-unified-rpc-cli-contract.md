---
role: support
status: draft
audience: governance
owner_domain: program-rfc
id: RFC-002
decision_id: RFC-002
supersedes: []
superseded_by: []
implements: []
evidenced_by: [docs/program/reports/audit-convergence-report.md]
related: [docs/program/adr/adr-006-unified-rpc.md]
---
# RFC-002 - Unified RPC envelope and CLI contract alignment

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
Contract drift between protocol/runtime headers and CLI command definitions can produce green CI with incomplete semantic alignment.

## Scope
- In scope: Envelope contract, command-surface alignment, deterministic reject semantics, anti-drift controls.
- Out of scope: Workspace lifecycle internals and engine attach strategy.

## Proposed Change
Adopt one canonical RPC surface and enforce CLI-to-spec alignment through explicit baseline checks and mandatory non-skip gate policy.

## Options Compared
- Option A: Strict single-surface contract with CI anti-drift enforcement.
- Option B: Multi-surface transition period with compatibility adapters.

## Risks
- Short-term CI noise due to stricter checks. Mitigation: phased enforcement with visible failure taxonomy.
- Cross-repo sync overhead. Mitigation: pin policy and sync scripts.

## Rollout Sketch
1. Freeze baseline artifact list.
2. Add contract comparison checks in CI.
3. Enforce deterministic reject matrix for mandatory steps.

## Exit Criteria
- [ ] Baseline artifact list is explicit and versioned.
- [ ] CI detects spec/CLI drift deterministically.
- [ ] Mandatory gates fail on missing capability (no pass-on-skip).

## Traceability

- Spec anchors (if any): `../governance/contracts/protocol/include/transport.h`, `../governance/contracts/protocol/include/protocol.h`, `../governance/contracts/protocol/runtime/include/rpc_runtime.h`, `../governance/model/registry/commands.v1.json`
- Targets ADR: `docs/program/adr/adr-006-unified-rpc.md`, `docs/program/adr/adr-011-contract-runbook-lock.md`
- Downstream runbook: `docs/archive/legacy/program/milestone-packs/runtime-baselines/operations-foundation/mp-runtime-000-root-hardening.md`
- Downstream MP: `docs/archive/legacy/program/milestone-packs/root-hardening/mp-runtime-000-root-hardening-v0-1-5.md`

## References
- `docs/program/spine.md`
- `docs/program/adr/adr-006-unified-rpc.md`
- `docs/program/adr/adr-011-contract-runbook-lock.md`

# Related Docs
- `docs/program/rfc/README.md`
- Linked ADR and report artifacts
