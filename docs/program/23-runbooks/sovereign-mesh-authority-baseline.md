# Sovereign Mesh Authority Baseline (MF-3)

## Objective

Validate that mesh discovery/coordination operation remains bounded by
owner-anchored sovereign authority.

## Baseline checks

1. Verify peer can be discovered without being enrolled.
2. Verify peer can be coordinated while still scope-limited.
3. Verify legitimacy state is explicit (`candidate|enrolled|trusted|suspended|revoked` baseline).
4. Verify suspended/revoked legitimacy constrains authority path.
5. Verify final authority/adjudication remains owner-side in all cases.

## Expected outcomes

- Discovery and coordination remain operational planes.
- Enrollment/trust legitimacy remains owner-anchored authority plane.
- Revocation/suspension semantics are visible and enforce authority contraction.

## Anti-drift assertions

- Visible peer != authoritative peer.
- Coordinated member != sovereign trust issuer.
- Participation state != canonical truth authority.
