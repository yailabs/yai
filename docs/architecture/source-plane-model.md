# Source Plane Model

Status: active
Owner: runtime
Effective date: 2026-03-10

## Purpose

Define the canonical source-plane model that links `yai-daemon` (subordinate
edge runtime)
to owner runtime truth in `yai`.

This model is the baseline for YD-3 and is intentionally scoped to entities,
IDs, contracts, and persistence hooks. It does not claim full transport or
operational completion.

## Design constraints

- `yai` owner runtime remains the only workspace truth source.
- Source plane is distributed only on acquisition side.
- Control plane stays centralized owner-side.
- No runtime federation in v1.
- No final authority/enforcement/graph truth at source node.
- Edge runtime execution is delegated and owner-scoped only.
- Edge observation is asset + process + runtime-state aware (RF-0.4).
- Observation scope is distinct from mediation/enforcement scope.

## Canonical entities

### 1) `source_node`

Identity of a source machine/node.

Minimum fields:

- `source_node_id`
- source label/hostname
- trust metadata placeholder
- owner association placeholder
- state

### 2) `source_daemon_instance`

Runtime instance identity of subordinate `yai-daemon` on a source node.

Minimum fields:

- `daemon_instance_id`
- `source_node_id`
- session/runtime marker
- start timestamp
- health/status
- build/version marker

### 3) `source_binding`

First-class attach relationship between source node and owner workspace context.

Minimum fields:

- `source_binding_id`
- `source_node_id`
- `owner_workspace_id`
- binding scope/type
- attachment status
- constraints placeholder
- delegated capability envelope placeholder

### 4) `source_asset`

Observed source-side asset identity (not owner canonical artifact truth).

Minimum fields:

- `source_asset_id`
- `source_binding_id`
- source locator/path/reference
- asset type
- provenance fingerprint
- observation state

### 5) `source_acquisition_event`

Acquisition event generated from source observation/emit steps.

Minimum fields:

- `source_acquisition_event_id`
- `source_node_id`
- `source_binding_id`
- `source_asset_id`
- event type
- observed timestamp
- idempotency marker
- delivery status
- delegated action marker placeholder

### 6) `source_evidence_candidate`

Evidence candidate emitted by source plane before owner-side acceptance.

Minimum fields:

- `source_evidence_candidate_id`
- `source_acquisition_event_id`
- candidate type
- derived metadata placeholder
- owner resolution/acceptance status

### 7) `source_owner_link`

Registration/trust link semantics between source-side identity and owner plane.

Minimum fields:

- `source_owner_link_id`
- `source_node_id`
- owner reference
- registration status
- registration timestamp

### 8) `source_enrollment_grant`

Owner-issued bootstrap artifact that records enrollment decision and v1 trust
token used by peer operations after enrollment.

Minimum fields:

- `source_enrollment_grant_id`
- `source_node_id`
- `daemon_instance_id`
- `owner_ref`
- `enrollment_decision`
- `trust_artifact_id`
- `trust_artifact_token`
- `issued_at_epoch`

### 9) `workspace_peer_membership`

Owner-side orchestration relation for one peer inside one workspace.
This relation is distinct from `source_binding`: membership captures
peer coordination semantics at case level (role/scope/health/backlog/coverage).

Minimum fields:

- `workspace_peer_membership_id`
- `owner_workspace_id`
- `source_node_id`
- `source_binding_id`
- `daemon_instance_id`
- `peer_role`
- `peer_scope`
- `peer_state`
- `backlog_queued`
- `backlog_retry_due`
- `backlog_failed`
- `coverage_ref`
- `overlap_state`
- `updated_at_epoch`

### 10) `source_policy_snapshot`

Owner-issued delegated policy material distributed to a specific edge target.

Minimum fields:

- `source_policy_snapshot_id`
- `source_node_id`
- `daemon_instance_id`
- `owner_workspace_id`
- `source_enrollment_grant_id`
- `snapshot_version`
- `distribution_target_ref`
- `issued_at_epoch`

### 11) `source_capability_envelope`

Owner-issued delegated capability envelope for binding-scoped edge operation.

Minimum fields:

- `source_capability_envelope_id`
- `source_node_id`
- `daemon_instance_id`
- `source_binding_id`
- `owner_workspace_id`
- `source_enrollment_grant_id`
- `observation_scope`
- `mediation_scope`
- `enforcement_scope`
- `distribution_target_ref`
- `issued_at_epoch`

## Distinctions (non-negotiable)

- `source_node` != `source_daemon_instance`
- `source_asset` != owner canonical artifact
- `source_evidence_candidate` != owner final evidence
- `source_binding` is explicit first-class relation, not implicit context

## Canonical record classes

Runtime data-plane record classes introduced by YD-3:

- `source_node`
- `source_daemon_instance`
- `source_binding`
- `source_asset`
- `source_acquisition_event`
- `source_evidence_candidate`
- `source_owner_link`
- `source_enrollment_grant`
- `workspace_peer_membership`
- `source_ingest_outcome`
- `source_policy_snapshot`
- `source_capability_envelope`
- `mesh_node`
- `mesh_discovery_advertisement`
- `mesh_bootstrap_descriptor`
- `mesh_coordination_membership`
- `mesh_peer_awareness`
- `mesh_peer_legitimacy`
- `mesh_authority_scope`
- `mesh_transport_endpoint`
- `mesh_transport_path_state`
- `mesh_transport_channel_state`

These are appendable through `yai_data_records_*` hooks and visible in summary
counts.

MF/MT extension note:
- mesh and transport record classes are coordination/authority/transport-plane
  operational classes;
- they do not redefine owner-side canonical truth authority.

## ID model

YD-3 introduces dedicated source ID helpers:

- `sn-*` source node IDs
- `sd-*` daemon instance IDs
- `sb-*` source binding IDs
- `sa-*` source asset IDs
- `se-*` source acquisition event IDs
- `sc-*` source evidence candidate IDs
- `sl-*` source owner link IDs
- `sg-*` source enrollment grant IDs
- `spm-*` workspace peer membership IDs
- `sps-*` source policy snapshot IDs
- `sce-*` source capability envelope IDs

IDs are deterministic enough for local consistency and are shaped for future
cross-plane traceability.

## Contract surface (model-level)

Source-plane logical operations are canonicalized as:

- `enroll`
- `attach`
- `emit`
- `status`

Model-level call/reply contract type IDs are reserved for each operation:

- `yai.source.enroll.call.v1` / `yai.source.enroll.reply.v1`
- `yai.source.attach.call.v1` / `yai.source.attach.reply.v1`
- `yai.source.emit.call.v1` / `yai.source.emit.reply.v1`
- `yai.source.status.call.v1` / `yai.source.status.reply.v1`

Enrollment reply now includes owner-issued trust bootstrap fields
(`owner_trust_artifact_id`, `owner_trust_artifact_token`) for subsequent
attach/emit/status operations.

SW-2 distribution baseline:
- owner replies include delegated distribution material references
  (`source_policy_snapshot_id`, `source_capability_envelope_id`,
  `distribution_target_ref`, delegated scopes);
- attach/status flows carry and refresh target-aware delegated scope material.

## RF-0.4 Observation Lock

Source-plane observation classes are canonical:

- asset observables
- process observables
- runtime observables

The daemon can emit owner-relevant runtime/process context (health, freshness,
spool/retry pressure, policy/grant staleness, integration reachability), but
this does not grant autonomous local action authority.

These define operation intent and shape anchors for YD-4/YD-5 transport and
owner-ingest work.

Delegated edge-runtime semantics baseline:
- edge can observe and mediate local actions only under owner-issued scope;
- edge-side outcomes are non-canonical until owner acceptance.

RF-0.3 delegated edge enforcement baseline:
- modes: observe-only, post-event, preventive, escalated;
- outcomes: observe_only, allow, block, hold, execute, escalate, defer,
  deny_due_to_missing_scope, deny_due_to_expired_grant.

## Persistence slice

The YD-3 persistence baseline does not add a second store topology.
It reuses canonical data-plane append flow in owner runtime:

- class-aware append into LMDB/DuckDB-backed runtime records
- source-plane classes countable via inspect summary
- no side-channel source truth stores

## Graph linkage roadmap (for YD-6)

Source-plane graph projection classes are locked for next slice:

- source node
- daemon instance
- binding
- asset
- acquisition event
- evidence candidate
- owner link

YD-3 defines naming/contracts/record shape so YD-6 can materialize graph
without re-inventing source entities.

## Explicitly incomplete in YD-3

- full remote transport flow
- full enroll/attach wire execution
- complete source scan/spool/retry behavior
- full delegated enforcement distribution
- complete source-plane graph materialization
- complete CLI/SDK source-plane UX surfaces

## References

- `docs/architecture/distributed-acquisition-plane-model.md`
- `docs/architecture/source-plane-model-refoundation-rf01.md`
- `docs/architecture/source-owner-ingest-model.md`
- `docs/architecture/source-plane-read-model.md`
- `docs/architecture/owner-peer-registry-and-coordination-model.md`
- `docs/architecture/exec-source-plane-role.md`
- `docs/program/22-adr/ADR-013-distributed-acquisition-centralized-control.md`
