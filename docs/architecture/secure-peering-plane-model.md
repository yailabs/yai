# Secure Peering Plane Model (NP-1, NP-4 aligned)

Status: active
Owner: runtime
Effective date: 2026-03-11

## Purpose

Define the canonical secure peering architecture for owner `yai` and peer `yai-daemon`
when communication leaves local trusted LAN assumptions.

This document fixes one non-negotiable boundary:

- YAI binary protocol is application-layer contract.
- Secure peering plane is transport/network trust layer.
- The protocol does not replace secure peering.

## Scope

In scope:

- secure peering model for owner <-> peer over non-local networks
- owner ingress taxonomy (local control vs remote peer ingress)
- deployment vs runtime responsibility boundary
- v1.5 baseline recommendation and v1 limits

Out of scope:

- full WireGuard automation in runtime
- full mTLS stack implementation
- peer-to-peer daemon mesh
- runtime federation

## Canonical topology

- one owner runtime (`yai`) remains canonical workspace truth
- one or more source peers (`yai-daemon`) emit acquisition payloads
- all remote peer traffic is mediated by secure peering before runtime protocol exchange

## Recommended model (v1.5 baseline)

Recommended:

- private overlay peering (WireGuard or equivalent private overlay)
- owner runtime peer ingress reachable only on private overlay address space
- daemon peers connect to owner peer ingress through overlay
- application payloads use existing YAI protocol/contracts on top of secure channel

Fallback for controlled environments:

- trusted LAN/private segment with explicit risk acceptance
- still treated as transitional and not Internet-grade

NP-4 operationalization:

- secure overlay integration is now the canonical deployment baseline for
  customer-grade non-local operation.
- operator bootstrap sequence is formalized in
  `docs/program/23-runbooks/owner-peer-overlay-bootstrap.md`.

## Endpoint taxonomy

Owner endpoint classes:

1. `local-control-ingress`
- purpose: operator/CLI/SDK local control and diagnostics
- expected transport: local UDS
- exposure: local host only

2. `owner-peer-ingress`
- purpose: remote source-plane peer operations (`enroll`, `attach`, `emit`, `status`)
- expected transport: secure private overlay path
- exposure: restricted private overlay

3. `admin-diagnostics-ingress` (optional, future)
- purpose: operational diagnostics beyond local control path
- not required for NP-1

## Responsibility boundary

YAI runtime concerns:

- endpoint semantics and command gating
- owner-side ingress role separation
- peer identity/trust assumptions consumed by runtime governance
- protocol-level operation semantics and persistence authority

Deployment/network concerns:

- provisioning overlay network (WireGuard/equivalent)
- key distribution and host-level firewalling
- route/NAT/network policy management

## Trust/bootstrap assumptions (NP-1 baseline)

- peer identity bootstrap must be explicit (owner-issued or operator-approved claim path)
- owner accepts remote peer operations only under secure peering assumptions
- source-plane evidence remains candidate until owner-side canonicalization

NP-1 does not claim full remote trust chain or full PKI closure.

## Anti-goals

- no daemon-to-daemon autonomous peering
- no second workspace truth outside owner runtime
- no claim that binary protocol alone is sufficient network security
- no uncontrolled Internet-facing owner socket exposure

## Derived implementation waves

- NP-2: owner remote ingress + endpoint hardening
- NP-3: peer identity/bootstrap contract + trust gating
- NP-4: overlay integration run/ops hardening

## Cross references

- `docs/architecture/distributed-acquisition-plane-model.md`
- `docs/architecture/source-owner-ingest-model.md`
- `docs/architecture/workspace-network-scope-model.md`
- `docs/architecture/secure-overlay-integration-model.md`
- `docs/program/22-adr/ADR-014-secure-peering-plane.md`
- `docs/program/23-runbooks/secure-peering-deployment-baseline.md`
