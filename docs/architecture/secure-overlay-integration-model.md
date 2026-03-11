# Secure Overlay Integration Model (NP-4)

Status: active
Owner: runtime
Effective date: 2026-03-11

## Purpose

Operationalize secure peering for non-local owner/peer operation by treating a
private secure overlay as the canonical transport baseline.

This NP-4 slice is now anchored by MT-1 transport-plane architecture:
`docs/architecture/secure-overlay-transport-plane-model.md`.

NP-4 does not build a VPN product inside YAI. It defines how YAI is deployed on
an existing secure overlay so owner/peer source-plane traffic is customer-grade
outside trusted LAN.

## Canonical position

- Recommended model: WireGuard private overlay (or equivalent private overlay
  required by customer infrastructure).
- YAI binary protocol remains application-layer and runs inside the overlay.
- Owner runtime is not exposed as generic Internet-facing control plane.
- Owner keeps ingress split:
  - local control ingress (`control.sock`)
  - remote peer ingress (`peer.sock`) for source-plane operations only.

## Baseline topology

- one owner runtime
- N peer daemons
- all nodes joined to one private overlay address space
- peer to owner traffic only required in v1 baseline
- no peer-to-peer daemon mesh required

## Layer boundaries

### Overlay layer (deployment concern)

Responsible for:

- tunnel establishment and key distribution
- route/reachability on private address space
- firewalling and interface exposure controls

Not responsible for:

- source-plane enroll/attach/emit/status semantics
- workspace truth decisions
- governance/evidence semantics

### YAI runtime layer (application concern)

Responsible for:

- endpoint role semantics (control vs peer ingress)
- peer enrollment and owner-issued trust bootstrap
- source-plane operation handling and canonical persistence
- workspace/governance authority semantics

Not responsible for:

- full network fabric orchestration
- full fleet VPN lifecycle management across all platforms

## Endpoint model on overlay

- owner peer ingress is bound and consumed as a source-plane remote endpoint
  reachable through overlay-private addressing.
- local control ingress remains host-local for operator/CLI on owner node.
- peer daemon uses owner overlay endpoint reference for bootstrap and emit flow.

## Baseline deployment examples

### Example A: Owner server + two office peers

- owner runtime on central server
- peer A in office-1, peer B in office-2
- all joined in same WireGuard private network
- both peers enroll to owner, then attach to same workspace
- emit/status flow reaches owner peer ingress via overlay only

### Example B: Owner server + desktop peer + document peer

- owner runtime on secured central host
- desktop peer observes user-side filesystem source
- document peer observes ingestion folder source
- both peers attach to one workspace with separate source bindings
- operator reviews per-peer health/backlog from owner-side surfaces

## Security claims and non-claims

### Claims in NP-4

- secure transport baseline is explicit and deployable
- owner/peer reachability model is practical outside LAN
- protocol/transport/trust/governance boundaries are explicit
- connectivity is modeled as transport-state, not sovereign authority

### Non-claims in NP-4

- no complete zero-trust architecture closure
- no enterprise-grade PKI lifecycle completion
- no universal overlay automation for every environment
- no multi-owner federation

## Cross-repo alignment requirements

### `yai`

- keep ingress separation and peer ingress source-scoped
- enforce owner-issued trust artifact checks on post-enroll operations
- document overlay assumptions in architecture/runbooks

### `yai-sdk`

- runtime target model must support explicit owner endpoint usage for overlay
- connection docs must distinguish local control and remote peer usage
- error taxonomy must classify unreachable/invalid owner endpoint cleanly

### `yai-cli`

- command docs must explain local-vs-overlay target expectations
- troubleshooting must include overlay reachability checks before protocol checks

### `yai-law`

- source attachment governance must keep network trust assumptions explicit
- overlay presence is a deployment precondition, not governance completion

## References

- `docs/architecture/secure-peering-plane-model.md`
- `docs/architecture/owner-ingress-model.md`
- `docs/architecture/peer-enrollment-and-trust-bootstrap-model.md`
- `docs/program/23-runbooks/owner-peer-overlay-bootstrap.md`
- `docs/program/23-runbooks/secure-peering-deployment-baseline.md`
