---
id: ADR-014
status: accepted
deciders:
  - runtime
date: 2026-03-11
supersedes: []
superseded_by: []
---
# ADR-014: Secure Peering Plane

## Context

Source-plane owner/peer operations are now real runtime paths. Local trusted-network
execution is possible, but Internet/multi-site deployment requires explicit secure
peering semantics not provided by the application protocol alone.

## Decision

1. YAI protocol stays application-layer only and does not replace secure network transport.
2. Secure peering is mandatory for non-local/untrusted network owner<->peer operation.
3. Recommended baseline is private overlay peering (WireGuard or equivalent).
4. Owner ingress roles are separated:
   - local control ingress
   - remote peer ingress
5. Runtime federation and daemon mesh are explicitly out of scope for this phase.

## Consequences

- Runtime and SDK/CLI must model endpoint roles explicitly.
- Deployment documentation must state overlay requirement for customer-grade remote operation.
- Law/governance must express trust/provenance assumptions for remote source attachment.

## Rejected alternatives

- Treat binary protocol as sufficient transport security.
- Expose owner local control socket directly for remote peers.
- Introduce peer-to-peer/federated runtime topology in this phase.

## Follow-up

- NP-2 owner remote ingress hardening
- NP-3 peer identity bootstrap
- NP-4 secure peering operationalization
