# ADR-023 — Governed Sovereign Mesh Foundation (MF-A1)

## Status

Accepted

## Context

After RF/ER/SW waves, YAI now has:

- subordinate edge runtime semantics
- delegated scope model
- owner-side sovereign authority and canonical truth lock

The next architectural step requires mesh-native topology without sovereignty
drift into flat peer authority.

## Decision

YAI adopts the **Governed Sovereign Mesh** model:

- mesh-native topology with explicit node roles (`owner`, `peer`)
- explicit three-plane separation:
  - Mesh Discovery Plane
  - Mesh Coordination Plane
  - Sovereign Authority Plane
- peer awareness is allowed and required for coordination
- owner workspace runtime remains final authority and canonical truth plane

Boundary lock:

- awareness/visibility/coordination never imply sovereign authority transfer
- mesh participation does not create distributed canonical truth ownership

## Consequences

- MF-1 can implement discovery without redefining authority.
- MF-2 can implement peer registry/coordination as non-sovereign runtime plane.
- MF-3 can implement trust and authority mechanics with explicit guardrails.
- MT waves can integrate transport overlays without conflating network reachability
  with authority.

## Non-goals

- flat peer mesh with equal authority
- autonomous peer-to-peer canonical adjudication
- transfer of owner canonical truth to peers
