# ADR-021 — Workspace-to-Edge Policy Distribution

## Status

Accepted

## Context

SW-1 locked workspace sovereignty and canonical truth owner-side.  
We need an explicit owner-to-edge distribution model so delegated edge behavior
is operationally real, target-aware, and auditable.

## Decision

Owner runtime distributes delegated operating material through source-plane
contracts and persisted source records:

- enrollment grants (`source_enrollment_grant`)
- policy snapshots (`source_policy_snapshot`)
- capability envelopes (`source_capability_envelope`)
- target association (`distribution_target_ref`)
- delegated scope fields (`observation/mediation/enforcement`)

Distribution is non-sovereign by definition:

- no policy truth transfer
- no graph/db truth transfer
- no final authority transfer

## Consequences

- Edge runtime can operate with explicit delegated material instead of implicit
  assumptions.
- Query/graph surfaces can inspect distribution coverage and scope assignment.
- SW-3 validity/expiry/refresh can extend this model without redesign.

## Non-goals

- Full revoke/expiry engine in this ADR (handled by SW-3).
- Peer-to-peer delegation.
- Any transfer of owner final adjudication authority.
