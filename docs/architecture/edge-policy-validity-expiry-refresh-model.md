# Edge Policy Validity / Expiry / Refresh Model (SW-3)

## Purpose

Define the canonical lifecycle of owner-distributed delegated material used by
subordinate edge runtimes (`yai-daemon`): policy snapshots, enrollment grants,
and capability envelopes.

This model ensures delegated scope is time-bounded, revocable, inspectable, and
safe under degraded/disconnected operation.

## Canonical lifecycle states

Delegated material is modeled with explicit states:

- `valid`
- `refresh_required`
- `stale`
- `expired`
- `revoked`

Operational helper states:

- `replacement_pending`
- `revalidation_pending`

## Freshness vs validity

- **Freshness**: how recent edge material is relative to owner-side expected
  state.
- **Validity**: whether the material is still authorization-usable.
- **Expiry**: temporal end of allowed delegated usage window.
- **Revoke**: owner-side active withdrawal of delegated scope.

These dimensions are related but non-equivalent.

## Delegation contraction rule

When delegated material gets weaker (older/ambiguous/invalid), edge autonomy
must contract, never expand.

Baseline fallback modes:

- `full_delegated`
- `restricted_hold_escalate`
- `observe_only`
- `disabled_by_revoke`

## Refresh and replacement flow

Baseline flow:

1. Edge detects refresh threshold or owner marks refresh-required.
2. Edge requests/receives replacement delegated material.
3. Edge updates local lifecycle fields (`issued`, `refresh_after`, `expires`).
4. Edge transitions back to `valid` when replacement succeeds.
5. Edge contracts behavior if replacement fails or owner unreachable.

## Expiry and revoke behavior

- **Expired** delegated material removes local delegated enforcement eligibility.
- **Revoked** delegated material has precedence over local cache.
- Revocation is owner-authoritative and does not require edge-side consent.

## Runtime inspect surfaces

Daemon runtime must expose at least:

- delegated validity state
- refresh state
- revoke state
- fallback mode
- stale reason
- lifecycle epochs (`issued`, `refresh_after`, `expires`)
- revoked booleans per material class

## Boundaries

- Local cache/spool/snapshots remain subordinate operational state.
- Canonical policy truth and final adjudication remain owner-side.
- SW-3 governs delegated lifecycle, not transfer of sovereignty.
