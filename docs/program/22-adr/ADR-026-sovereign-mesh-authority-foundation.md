# ADR-026 — Sovereign Mesh Authority Foundation (MF-3)

## Status

Accepted

## Context

After MF-1 (discovery) and MF-2 (coordination), mesh operation is visible and
coordinable. A final authority lock is required so visibility/membership does
not drift into flat peer authority.

## Decision

YAI adopts an explicit Sovereign Authority Plane in mesh context with:

- owner-anchored enrollment finalization
- owner-anchored trust bootstrap and legitimacy recognition
- explicit authority-scope limitation and revocation semantics
- strict separation from discovery and coordination planes

Boundary locks:

- mesh participation does not imply sovereign authority
- discovery/coordination signals do not imply canonical truth ownership
- final adjudication remains owner workspace runtime responsibility

## Consequences

- MT waves can integrate remote peering/overlay without authority ambiguity.
- DX law alignment can model legitimacy/trust/revocation coherently.
- QW trust/WAN qualification can validate degraded/revoked legitimacy cases.

## Non-goals

- peer-autonomous sovereign trust issuance
- distributed final adjudication among peers
- flattening owner sovereignty into mesh membership
