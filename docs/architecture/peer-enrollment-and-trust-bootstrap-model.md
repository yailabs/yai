# Peer Enrollment and Trust Bootstrap Model

Status: active  
Owner: runtime  
Effective date: 2026-03-11

## Purpose

Define the v1 owner-centric bootstrap used to move a peer from reachable to
enrolled. This model is intentionally minimal: it provides a traceable trust
bootstrap without introducing full PKI or transitive peer trust.

## Core decisions

- Enrollment is owner-centric: peer trust is not self-declared.
- `enroll` and `attach` are separate phases.
- Owner emits an operational trust artifact after enrollment acceptance.
- `attach`, `emit`, and `status` require that owner-issued trust artifact.
- Workspace truth stays owner-side; peer trust bootstrap does not grant
  workspace authority by itself.

## Flow

1. Peer sends `yai.source.enroll` with bootstrap identity (`source_label`,
   optional `source_node_id`, optional `daemon_instance_id`, `owner_ref`).
2. Owner resolves enrollment decision (`accepted`, `pending`, `rejected`).
3. Owner persists:
   - `source_node`
   - `source_daemon_instance`
   - `source_owner_link`
   - `source_enrollment_grant`
4. On acceptance, owner returns:
   - `source_enrollment_grant_id`
   - `owner_trust_artifact_id`
   - `owner_trust_artifact_token`
5. Peer uses that trust artifact token for `attach`, `emit`, and `status`.

## New source-plane record class

- `source_enrollment_grant`

Minimum fields:

- `source_enrollment_grant_id`
- `source_node_id`
- `daemon_instance_id`
- `owner_ref`
- `enrollment_decision`
- `trust_artifact_id`
- `trust_artifact_token`
- `trust_bootstrap_model` (`owner_issued_v1`)
- `issued_at_epoch`

## v1 trust artifact semantics

- Artifact is owner-issued and scoped to the enrolled source node.
- v1 token shape is deterministic and traceable (not a full cryptographic
  credential chain).
- Purpose in v1: block unauthenticated source-plane operations after ingress
  reachability is achieved.

## Enrollment decisions

- `accepted`: peer is ready for `attach`.
- `pending`: peer is known but not attach-ready.
- `rejected`: peer is denied for current bootstrap request.

## Explicit non-goals in v1

- enterprise PKI and certificate lifecycle
- hardware attestation
- delegated trust between peers
- full revocation/rotation fabric

## References

- `docs/architecture/secure-peering-plane-model.md`
- `docs/architecture/owner-ingress-model.md`
- `docs/architecture/source-owner-ingest-model.md`
- `docs/architecture/source-plane-model.md`
