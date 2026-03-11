# Sovereign Mesh Authority Foundation Model (MF-3)

## Purpose

Define the canonical Sovereign Authority Plane for the Governed Sovereign Mesh.

MF-3 closes the mesh triangle:

- MF-1 discovery
- MF-2 coordination
- MF-3 sovereign authority

Topology is distributed; authority remains owner-anchored.

## Authority plane boundaries

In scope:

- owner-anchored enrollment finalization
- trust bootstrap authority
- peer legitimacy recognition state
- authority scope issuance/limitation
- sovereign boundary between participation and final adjudication

Out of scope:

- flat peer authority
- autonomous peer trust issuance
- distributed canonical truth ownership

## Enrollment and legitimacy model

A peer can be:

- discovered
- candidate
- enrolled
- trusted scope-limited
- stale/suspended/revoked

Boundary lock:

- discovered != enrolled
- visible != trusted
- coordinated member != sovereign authority holder

Enrollment finalization and legitimacy recognition are owner/workspace anchored.

## Trust bootstrap and authority scope

Trust bootstrap is owner-issued and revocable.

Authority-relevant material:

- enrollment trust artifact
- legitimacy state
- authority scope state (effective delegated boundary)
- suspension/revocation state

These states constrain operation and never relocate sovereign authority.

## Membership vs authority

Mesh membership grants governed participation in discovery/coordination planes.
It does not grant:

- global policy authority
- final conflict adjudication authority
- authority to enroll/trust peers autonomously
- canonical truth ownership

Formula lock:

participation in the mesh does not constitute sovereign authority within the
mesh.

## Awareness vs sovereign adjudication

- awareness remains awareness
- coordination remains coordination
- trust/enrollment finalization remains owner-anchored
- final adjudication remains workspace owner-side

Mesh remains governed and non-flat in authority.

## Provenance/trust/authority binding

Canonical owner-side binding links:

- peer identity
- enrollment/trust state
- legitimacy state
- delegated authority scope
- provenance of distributed contributions
- owner-side adjudication outcome

This keeps distributed operation explainable without authority drift.

## WAN/overlay compatibility

MF-3 supports scenarios where peers are remote and reachability changes over
time:

- visible but not enrolled peer
- enrolled but scope-limited peer
- stale/suspended/revoked peer
- disconnected peer with constrained authority path

Authority boundaries remain stable across LAN/overlay/WAN contexts.

## Cross-repo impact baseline

### `yai`

- authority-plane markers must remain explicit in runtime model
- discovery/coordination data cannot be interpreted as final authority

### `yai-law`

- law semantics must encode legitimacy, suspension/revocation, and authority
  boundary constraints
- trust/provenance governance must remain owner-anchored

## Handoffs

MF-3 provides the authority lock for:

- MT transport/overlay waves
- DX governance consumption wave
- QW WAN/trust qualification waves
