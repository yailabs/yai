# Workspace Security Scopes

## Scopes
- process scope
- filesystem scope
- socket scope
- network scope
- resource scope
- privilege scope
- runtime route scope
- binding scope

## Scope status in current implementation
- Active now: filesystem, runtime_route, binding.
- Modeled/hook-ready: process, socket, network, resource, privilege.

## NP-1 secure peering interpretation
- `socket` and `network` scopes are distinct.
- local control ingress uses host-local transport.
- remote peer ingress requires explicit secure peering assumptions before
authoritative owner ingest is considered customer-grade.

## NP-4 secure overlay baseline

- recommended non-local source-plane transport is private secure overlay
  (WireGuard/equivalent).
- overlay reachability is required but not sufficient: owner-issued trust
  bootstrap is still mandatory for peer operations.
- secure overlay is deployment substrate, not replacement for runtime
  governance and source-plane attachability semantics.
