---
id: RB-OWNER-PEER-OVERLAY-BOOTSTRAP
status: active
owner: runtime
effective_date: 2026-03-11
revision: 1
---
# Owner/Peer Overlay Bootstrap

## Purpose

Provide a concrete bootstrap sequence for deploying owner `yai` and peer
`yai-daemon` outside LAN on a private secure overlay.

## Preconditions

- private overlay is available (WireGuard recommended baseline)
- owner and peer hosts have overlay identity and routes configured
- owner runtime binaries and peer daemon binaries are installed

## Topology baseline

- one owner node
- one or more peer nodes
- owner peer ingress reachable only through overlay-private path

## Procedure

1. Prepare owner node
- start `yai` runtime
- verify local control ingress for local operator flow
- verify peer ingress is available for source-plane remote operations

2. Prepare peer node
- start `yai-daemon` with owner reference pointing to owner overlay endpoint
- validate local daemon config paths/bindings

3. Verify overlay reachability before protocol checks
- peer can reach owner overlay address/path
- no direct public Internet exposure is used for peer flow

4. Bootstrap peer trust
- execute source enroll (`yai.source.enroll`)
- require owner-issued trust artifact fields in enroll reply
- stop if enrollment decision is not `accepted`

5. Attach peer to workspace
- execute source attach (`yai.source.attach`) with owner trust artifact token

6. Start source delivery flow
- run emit/status path (`yai.source.emit`, `yai.source.status`) with trust token
- confirm owner-side records and query surfaces reflect peer activity

## Verification checklist

- owner ingress separation is preserved (control vs peer)
- peer operations are source-plane scoped
- post-enroll operations include owner trust artifact token
- workspace/source summaries show peer contributions

## Failure handling

- overlay unreachable:
  - fix route/interface/firewall before retrying enrollment
- enrollment pending/rejected:
  - do not run attach/emit until owner acceptance
- trust bootstrap required errors:
  - refresh enroll reply and ensure token propagation in attach/emit/status

## Explicit non-goals

- full mesh orchestration automation
- full PKI lifecycle and certificate automation
- multi-owner federation management

## Related

- `docs/architecture/secure-overlay-integration-model.md`
- `docs/architecture/peer-enrollment-and-trust-bootstrap-model.md`
- `docs/program/23-runbooks/peer-enrollment-baseline.md`
