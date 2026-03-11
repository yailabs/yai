# Peer Enrollment Baseline Runbook

Status: active  
Audience: operator / maintainer

## Scope

Baseline owner-centric enrollment for `yai-daemon` peers in v1.

## Preconditions

- Owner runtime is up with both ingress planes:
  - local control ingress
  - peer ingress (source-plane scoped)
- Secure overlay assumptions are in place for non-local deployment.
- Workspace already exists on owner.

## Steps

1. Start peer daemon with owner reference configured.
2. Peer calls `yai.source.enroll`.
3. Validate enroll reply contains:
   - `enrollment_decision`
   - `source_enrollment_grant_id`
   - `owner_trust_artifact_id`
   - `owner_trust_artifact_token`
4. If decision is `accepted`, proceed with `yai.source.attach`.
5. Include `owner_trust_artifact_token` in all subsequent:
   - `yai.source.attach`
   - `yai.source.emit`
   - `yai.source.status`

## Failure handling

- `enrollment_decision=pending`: stop attach; keep peer out of workspace flow.
- `enrollment_decision=rejected`: stop attach and emit.
- `*_trust_bootstrap_required` errors: peer is missing/using invalid owner
  trust artifact token.

## Notes

- Enrollment does not grant final workspace authority.
- Trust bootstrap in v1 is owner-issued and traceable, not a complete PKI.
