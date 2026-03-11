# ADR-027 — Secure Overlay Transport Plane (MT-1)

## Status

Accepted

## Context

Mesh discovery/coordination/authority foundations are already defined
(MF-1/MF-2/MF-3). Remote owner/peer operation now requires a dedicated
transport-plane lock that does not collapse governance boundaries.

## Decision

YAI adopts a distinct Secure Overlay Transport Plane for remote connectivity:

- private overlay reachability and secure endpoint paths are first-class runtime
  concerns
- owner<->peer path state is modeled separately from trust/enrollment/authority
- transport observability includes availability, degradation, freshness and
  reconnect state

Boundary locks:

- reachable != enrolled
- connected != trusted
- transport-visible != authority-bearing

## Consequences

- MT-2 can harden owner remote peer ingress on top of explicit transport-plane
  semantics.
- MT-3 can integrate WireGuard/equivalent overlay without redefining authority
  boundaries.
- QW-2/QW-5 can qualify remote peering and WAN resilience with layered signals.

## Non-goals

- deriving trust or legitimacy from connectivity alone
- replacing enrollment or delegated-scope governance with endpoint reachability
- turning overlay transport into canonical truth/authority plane
