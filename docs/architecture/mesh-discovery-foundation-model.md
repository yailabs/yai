# Mesh Discovery Foundation Model (MF-1)

## Purpose

Define the canonical Mesh Discovery Plane for the Governed Sovereign Mesh.

Discovery is a governed visibility/bootstrap plane. It enables node
discoverability and topology awareness, but it does not grant trust,
permission, or sovereign authority.

## Discovery plane boundaries

In scope:

- node identification for mesh visibility
- node advertisement
- owner discovery and peer discovery
- bootstrap discovery flow and seed material
- scope-aware discovery visibility

Out of scope:

- enrollment completion
- trust finalization
- delegated capability issuance
- authority transfer
- distributed canonical truth ownership

## Node advertisement baseline

Canonical minimal advertisement fields:

- `mesh_node_id`
- `node_role` (`owner`|`peer`)
- `mesh_id`
- `owner_id` (owner reference for sovereign anchor)
- `protocol_version`
- `capabilities_ref` (minimal declared capability set)
- `reachability_state`
- `discovery_endpoint_ref`
- `enrollment_mode`
- `workspace_visibility_scope`
- `advertisement_status`
- `advertised_at_epoch`

## Owner discovery vs peer discovery

### Owner discovery

Allows a peer runtime to locate/recognize the owner node for workspace-anchored
bootstrap.

### Peer discovery

Allows governed visibility of peer nodes for awareness and future coordination.

Boundary lock:

- owner discovery != enrollment
- peer discovery != trust
- peer visibility != broad permission

## Bootstrap discovery flow

1. Node initializes discovery identity and role.
2. Node loads bootstrap descriptor/seed material.
3. Node announces or queries discovery surfaces.
4. Node obtains owner/peer visibility under discovery scope.
5. Node transitions into enrollment/trust/coordination planes.

## Scope-aware discovery

Discovery visibility is governed and scope-aware by:

- workspace or mesh scope
- node role
- owner policy context
- reachability context (LAN/overlay/WAN transitions)
- bootstrap visibility policy

Not all nodes must see all mesh nodes at all times.

## Canonical formulas

- discoverable does not mean trusted
- visible does not mean enrolled
- discovery visibility is not sovereign authority

## Cross-repo impact baseline

### `yai`

- must expose discovery records and bootstrap descriptors as explicit runtime
  model surfaces
- must keep discovery separated from enrollment/trust/authority decisions

### `yai-sdk`

- must represent mesh-node discovery descriptors and bootstrap discovery results
- must separate discovery surfaces from authority/grant surfaces

### `yai-cli`

- must expose list/watch/inspect semantics for discovery state and mesh nodes
- must avoid implying trust/authority from visibility alone

## Handoffs

MF-1 provides baseline for:

- MF-2 coordination model
- MF-3 trust/authority lock in mesh context
- MT transport/ingress integration
- distributed qualification waves

MF-3 authority follow-up:

- `docs/architecture/sovereign-mesh-authority-foundation-model.md`
