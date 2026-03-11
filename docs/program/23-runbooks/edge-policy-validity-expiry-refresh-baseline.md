# Edge Policy Validity / Expiry / Refresh Baseline (SW-3)

## Objective

Validate that delegated edge material lifecycle is explicit and that daemon
behavior contracts under stale/expired/revoked states.

## Baseline checks

1. Start owner runtime and daemon.
2. Enroll + attach to materialize grant/snapshot/capability.
3. Verify daemon operational state includes:
   - delegated validity/refresh/revoke/fallback/stale_reason
   - lifecycle epoch fields
4. Trigger/observe refresh-required condition and confirm restricted fallback.
5. Trigger/observe expired or revoked condition and confirm autonomy reduction.

## Expected runtime behavior

- `valid`: delegated behavior can run within granted scope.
- `refresh_required`: edge contracts to restricted behavior (`hold/escalate`).
- `stale`: edge converges to limited behavior (typically `observe_only`).
- `expired`: local delegated enforcement is disabled.
- `revoked`: scope is removed; fallback is `disabled_by_revoke`.

## Boundary assertions

- Edge material lifecycle is operational and subordinate.
- Owner runtime remains policy/authority/truth plane.
- Local stale/expired/revoked state never increases edge authority.
