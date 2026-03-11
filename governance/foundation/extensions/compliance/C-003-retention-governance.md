# C-003 Retention Governance

## Definition

A retained record is governed by a retention policy identified by `retention_policy_id` and anchored to deterministic time.

Deterministic expiry in R6 is defined as:

`expires_at = created_ts + ttl_seconds`

with `created_ts` derived from event time (`expiry_anchor = event_ts`).

## Invariant

For any retained record that is not tombstoned:

`now_ts >= expires_at => record.tombstone = true AND RETENTION_EXPIRE emitted`

## Constraints

- `now_ts` MUST be event-derived logical time.
- Wall clock time MUST NOT drive expiry transitions.
- R6 uses `deletion_mode = tombstone` only.

## Binding

- Spec: `compliance/retention.policy.v1.json`
- Pack defaults: `packs/compliance/gdpr-eu/2026Q1/retention.defaults.json`
