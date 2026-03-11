# Secure Overlay Transport Baseline (MT-1)

## Objective

Validate transport-plane readiness for remote owner/peer operation without
confusing transport health with trust, enrollment, or sovereign authority.

## Baseline checks

1. Verify owner and peer endpoints are reachable through private overlay path.
2. Verify transport state is inspectable (`available|degraded|reconnect_required` baseline).
3. Verify disconnected/recovered transitions are visible as transport events.
4. Verify transport recovery does not bypass enrollment/trust/policy gates.
5. Verify authority semantics remain owner-anchored after path restoration.

## Expected outcomes

- Secure overlay transport path state is explicit and queryable.
- Reachability troubleshooting is separated from authority troubleshooting.
- Path degradation/recovery is operationally visible without authority drift.

## Anti-drift assertions

- Reachable endpoint != enrolled peer.
- Connected channel != trusted legitimacy state.
- Transport healthy != delegated scope automatically valid.
