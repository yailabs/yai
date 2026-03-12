---
role: support
status: draft
audience: governance
owner_domain: program-adr
id: ADR-011
decision_id: ADR-011
supersedes: []
superseded_by: []
implements: [docs/program/rfc/rfc-004-lock-pin-policy.md]
evidenced_by: [docs/program/reports/runtime-convergence-report.md]
related: []
anchor: "#phase-0-1-0-pin-baseline-freeze"
applies_to: 
effective_date: 2026-02-19
law_refs: 
phase: 0.1.0
runbook: docs/archive/legacy/program/milestone-packs/runtime-baselines/operations-foundation/mp-runtime-000-contract-runbook-lock.md
---
# ADR-011 - Contract Baseline Lock for Milestone 1

## Context

Milestone 1 exposed three recurring risks:

- Contract drift between specs and CLI/runtime behavior
- Green pipelines with skipped mandatory proof steps
- Inconsistent evidence quality for TRL-facing claims

## Decision

Milestone 1 enforces a contract baseline lock across `governance`, `yai`, and `cli`.

Controls:

1. CI parity checks between pinned specs contract and CLI/runtime behavior
2. Mandatory gates fail on missing capability (no pass-on-skip)
3. Formal/core verify updates are required when contract deltas affect policy/authority/envelope invariants
4. Cross-repo pins remain explicit and auditable

## Rationale

A lock provides a stable legal/technical floor so later runbook phases can evolve without losing evidence integrity.

## Consequences

- Positive:
  - Stronger confidence in cross-repo compatibility.
  - Better audit quality and clearer TRL narrative.
- Negative:
  - Higher short-term coordination cost for contract-touching changes.

## Traceability

- Proposals:
  - `docs/program/rfc/rfc-002-unified-rpc-cli-contract.md`
  - `docs/program/rfc/rfc-004-lock-pin-policy.md`
  - `docs/program/rfc/rfc-005-formal-coverage-roadmap.md`
- Implemented by runbooks:
  - `docs/archive/legacy/program/milestone-packs/runtime-baselines/operations-foundation/mp-runtime-000-contract-runbook-lock.md`
  - `docs/archive/legacy/program/milestone-packs/runtime-baselines/operations-foundation/mp-runtime-000-root-hardening.md` (downstream hardening)
- Milestone packs:
  - `docs/archive/legacy/program/milestone-packs/contract-baseline-lock/mp-contracts-000-contract-runbook-lock-v0-1-4.md` (planned)

## Status

Draft; intended for acceptance at Milestone 1 governance kickoff.
