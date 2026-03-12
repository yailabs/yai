---
role: canonical
status: active
audience: architect
owner_domain: architecture
primary_for: protocol-architecture
---

# Secure Overlay Transport Plane Model (MT-1)

# Purpose
Defines canonical architecture semantics for the domain.

# Scope
Covers boundaries, responsibilities, and integration semantics for this domain section.

# Relationships
- Parent section README
- Adjacent architecture source documents

# Canonical Role
Authoritative architecture source for its scope.

# Main Body
## Purpose

Define the canonical transport plane for remote owner/peer connectivity in the
Governed Sovereign Mesh.

MT-1 locks secure overlay transport as a distinct architectural plane:

- separate from mesh discovery,
- separate from mesh coordination,
- separate from sovereign policy/authority/truth.

## Transport plane responsibilities

In scope:

- private overlay reachability for owner and peer nodes
- secure transport endpoint representation
- owner<->peer remote path availability/degradation state
- reconnect/freshness visibility for transport paths
- transport-state observability for runtime/query/runbook qualification

Out of scope:

- trust issuance by transport reachability
- enrollment finalization by connectivity alone
- delegated scope issuance by endpoint visibility
- canonical truth or final adjudication transfer

## Plane boundaries

Boundary locks:

- reachable != enrolled
- connected != trusted
- transport-visible != authority-bearing

Transport enables contact; it does not conclude sovereign decisions.

## Private overlay baseline

YAI assumes a private secure overlay baseline for non-LAN deployment:

- owner transport presence on private overlay
- peer transport presence on private overlay
- secure endpoint references bound to mesh node identity
- no requirement for public flat exposure of owner ingress

WireGuard (or equivalent private overlay) is deployment substrate; YAI remains
application/runtime/governance layer above it.

## Owner<->peer remote connectivity model

MT-1 baseline requires explicit representation of:

- owner remote ingress readiness
- peer remote egress/connectivity readiness
- overlay path availability/degradation
- secure channel state and reconnect requirement
- transport freshness/path age signals

This model supports variable reachability and intermittent connectivity without
changing sovereign boundaries.

## Relationship with discovery/coordination/authority

### Discovery plane

Discovery can expose nodes and endpoints but does not imply ready secure
transport or trusted enrollment.

### Coordination plane

Coordination can represent peer state/coverage/order/replay signals but does
not replace transport health.

### Sovereign authority plane

Authority, trust bootstrap, enrollment finalization, delegated scope and
canonical truth remain owner-anchored even when transport path is healthy.

## Transport state as first-class concern

Canonical transport-state categories (baseline vocabulary):

- endpoint availability
- overlay reachability
- path freshness
- secure channel state
- ingress readiness
- transport degradation state
- reconnect required

These are runtime-operational signals and must not be interpreted as authority
state.

## Subordinate edge runtime compatibility

Transport behavior must remain consistent with edge subordination:

- remote/disconnected edge operation can continue in constrained mode
- transport loss triggers degradation/reconnect paths
- transport recovery does not bypass policy/trust/authority gates
- owner-side sovereign authority remains intact during all path transitions

## Cross-repo contract baseline

### `yai`

- model secure transport records/state as runtime-observable classes
- keep transport-state separated from enrollment/trust/authority classes

### `yai-sdk`

- expose remote endpoint/transport path descriptors as first-class surfaces
- keep transport errors distinct from policy/authority/policy errors

### `yai-cli`

- inspect/list/watch transport state without implying trust/enrollment success
- keep troubleshooting layered: transport reachability before policy assumptions

### `yai-governance`

- preserve explicit boundary between transport availability and sovereign
  legitimacy/authority semantics
- prevent policy interpretation from deriving authority from connectivity alone

## Handoffs

MT-1 provides the transport-architecture lock for:

- MT-2 owner remote peer ingress hardening
- MT-3 concrete secure overlay integration (WireGuard/equivalent)
- QW-2 secure peering qualification
- QW-5 WAN resilience qualification

Owner ingress boundary details are defined in:
`docs/architecture/owner-remote-peer-ingress-model.md`.
Overlay-native deployment integration details are defined in:
`docs/architecture/overlay-integration-model.md`.

# Related Docs
- `docs/architecture/README.md`
- Domain-adjacent architecture documents
