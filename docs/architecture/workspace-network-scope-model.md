# Workspace Network Scope Model

Network scope is modeled in the envelope and intentionally separated from normative law/policy decisions.

- Law/policy: whether network action is allowed.
- Security envelope: confinement profile of execution network plane.

## NP-1 secure peering baseline

For source-plane owner/peer operation outside trusted LAN assumptions:

- application protocol and transport security are separate layers
- owner must distinguish local control ingress from remote peer ingress
- remote peer traffic is expected on secure private overlay path

Current runtime implementation is UDS-first at control ingress level.
Customer-grade non-local owner/peer operation is modeled as:

- explicit peer ingress usage
- private secure overlay transport baseline (WireGuard/equivalent)
- source-plane trust bootstrap on top of overlay reachability

See:

- `docs/architecture/secure-overlay-integration-model.md`
- `docs/program/23-runbooks/owner-peer-overlay-bootstrap.md`
