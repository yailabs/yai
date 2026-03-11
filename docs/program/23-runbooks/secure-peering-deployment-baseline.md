---
id: RB-SECURE-PEERING-BASELINE
status: active
owner: runtime
effective_date: 2026-03-11
revision: 2
---
# Secure Peering Deployment Baseline

## Purpose

Provide baseline deployment guidance for owner/peer source-plane operation
outside local trusted LAN assumptions.

## Modes

### Mode A: Trusted LAN (transitional)

- owner and peers on same trusted segment
- acceptable for internal validation only
- not customer-grade Internet posture

### Mode B: Private Overlay (recommended)

- owner and peers connected through private overlay (WireGuard/equivalent)
- owner peer ingress reachable only on overlay address
- local control ingress remains local-only

## Baseline checklist

1. Provision private overlay identities and routes.
2. Restrict owner peer ingress to overlay interfaces.
3. Keep local control ingress on host-local transport.
4. Enroll/attach peers through governed source-plane operations.
5. Monitor per-peer health/backlog and trace ingestion decisions.

## NP-4 integration note

This runbook remains the compact baseline checklist.
The full owner/peer bootstrap sequence is defined in:

- `docs/program/23-runbooks/owner-peer-overlay-bootstrap.md`
- `docs/architecture/secure-overlay-integration-model.md`

## Explicit non-goals

- No claim of full zero-trust closure.
- No automatic global discovery/mesh.
- No runtime-owned VPN orchestration in NP-1.

## Verification signals

- remote peer connects only through secure peering path
- owner accepts source-plane operations on peer ingress path
- owner canonical truth remains unchanged in role split
