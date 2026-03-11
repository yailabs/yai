# ADR-022 — Edge Policy Validity / Expiry / Refresh

## Status

Accepted

## Context

SW-2 introduced owner-to-edge policy distribution (grant/snapshot/capability
material). Without explicit lifecycle semantics, delegated scope can drift into
implicit indefinite authority.

## Decision

YAI adopts explicit delegated lifecycle semantics for edge policy material:

- validity states: `valid`, `refresh_required`, `stale`, `expired`, `revoked`
- lifecycle fields per material class: `issued_at`, `refresh_after`, `expires_at`
- explicit refresh/replacement and stale fallback behavior
- owner-authoritative revoke handling

Contraction rule is mandatory:

- as delegated material weakens, edge local autonomy must reduce, never expand

Baseline fallback modes:

- `full_delegated`
- `restricted_hold_escalate`
- `observe_only`
- `disabled_by_revoke`

## Consequences

- Disconnected operation is bounded by explicit delegated validity.
- Expiry/revoke become inspectable and auditable runtime states.
- MT-2/MT-3 and qualification waves can validate WAN/degraded behavior without
  redefining authority boundaries.

## Non-goals

- Full enterprise policy orchestration/fleet rollout.
- Any transfer of sovereign authority/truth to edge runtime.
- Peer-to-peer delegated authority propagation.
