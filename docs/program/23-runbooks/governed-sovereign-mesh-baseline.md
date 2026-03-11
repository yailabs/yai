# Governed Sovereign Mesh Baseline (MF-A1)

## Objective

Provide operator/maintainer baseline checks to validate that mesh behavior is
topology-distributed but authority-sovereign.

## Baseline checks

1. Verify owner + N peers are visible in mesh topology surfaces.
2. Verify peer role visibility is present (`owner` vs `peer`).
3. Verify coordination surfaces show peer health/freshness/coverage hints.
4. Verify canonical authority decisions remain owner-side.
5. Verify no peer surface claims sovereign policy/graph/conflict truth.

## Expected interpretation model

- Discovery confirms node presence and role awareness.
- Coordination confirms peer operational participation in a workspace case.
- Authority confirms final canonical decisions and truth ownership.

## Anti-drift checks

- If a peer can see another peer, that does not imply permission escalation.
- If coordination hints are exchanged, that does not imply policy sovereignty.
- If mesh connectivity grows, owner truth/authority boundary remains unchanged.

## Next-wave handoff

Use this baseline to start:

- MF-1 discovery protocol slices
- MF-2 coordination/runtime slices
- MF-3 trust/authority mesh slices
- MT transport/overlay slices
