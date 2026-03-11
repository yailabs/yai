# Workspace-to-Edge Policy Distribution Model (SW-2)

## Purpose

Define how the sovereign owner workspace runtime distributes delegated operating
material to subordinate edge runtimes (`yai-daemon`) without transferring
sovereignty or canonical truth.

## Distributed material classes

Owner runtime distributes distinct artifacts:

- `source_enrollment_grant`
- `source_policy_snapshot`
- `source_capability_envelope`
- target association (`distribution_target_ref`)
- delegated scopes (`observation`, `mediation`, `enforcement`)

These are operational artifacts, not owner truth transfer.

## Targeting model

Distribution is always scoped to:

- workspace
- source node
- daemon instance
- binding (for capability envelopes)

Material must be target-aware and scope-limited.

## Runtime record model (v1 baseline)

`source_policy_snapshot` records:

- `source_policy_snapshot_id`
- `source_node_id`
- `daemon_instance_id`
- `owner_workspace_id`
- `source_enrollment_grant_id`
- `snapshot_version`
- `distribution_target_ref`

`source_capability_envelope` records:

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

## Contract flow

- `yai.source.enroll`: owner issues grant + baseline snapshot + baseline envelope.
- `yai.source.attach`: owner issues binding-scoped envelope and target association.
- `yai.source.status`: owner refreshes/records current distributed material state.

## Boundaries

- Distribution enables edge operation.
- Distribution does not move policy truth, graph truth, DB truth, or final
  adjudication away from owner runtime.
- Stale/missing distributed material reduces edge autonomy.
