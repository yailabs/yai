# Workspace-to-Edge Policy Distribution Baseline (SW-2)

## Objective

Verify that owner runtime distributes delegated scope material to a daemon and
that daemon state reflects it.

## Baseline flow

1. Start owner runtime.
2. Enroll daemon node.
3. Attach daemon binding.
4. Send daemon status.
5. Inspect source-plane records and edge operational state.

## What to verify

- `source_enrollment_grant` exists.
- `source_policy_snapshot` exists with `distribution_target_ref`.
- `source_capability_envelope` exists with delegated scopes.
- `source.status` reply carries snapshot/envelope/scope fields.
- daemon local health/operational state includes distribution metadata.

## Expected boundary behavior

- Distributed material enables edge operation.
- Owner remains canonical truth and final authority plane.
- Missing/stale material must reduce edge autonomy.
