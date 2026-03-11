# Governed Sovereign Mesh Model (MF-A1)

## Purpose

Define the canonical mesh foundation for YAI after the edge/runtime and
workspace-sovereignty refoundation waves.

YAI is mesh-native in topology and sovereign in authority.

## Canonical thesis

The system allows multiple subordinate edge nodes to participate in one
workspace case as governed mesh members, while canonical authority and truth
remain owner-side.

This model explicitly rejects flat peer authority and ungoverned peer mesh.

## Three-plane mesh architecture

### 1) Mesh Discovery Plane

Purpose:

- discover owner/peer nodes
- bootstrap node visibility and role awareness
- expose topological presence and minimal capabilities

Discovery provides visibility, not authority.

### 2) Mesh Coordination Plane

Purpose:

- maintain governed mesh membership
- provide peer registry and peer-awareness surfaces
- carry scheduling/coverage/ordering coordination hints
- represent peer freshness/availability/degradation

Coordination provides operational awareness, not sovereign adjudication.

### 3) Sovereign Authority Plane

Purpose:

- retain canonical policy truth
- retain canonical graph/db/case truth
- issue trust/enrollment/delegation material
- resolve final conflict truth and final adjudication

This plane is owner workspace runtime authority and is never flattened into
peer mesh authority.

## Mesh nodes and node roles

Canonical mesh node roles:

- `owner`
- `peer`

Reserved extension roles (future, non-goal in MF-A1):

- `observer`
- `relay`
- `delegate`

Role awareness is mandatory; role equality is not assumed.

## Awareness vs authority boundaries

Allowed peer awareness:

- peer existence
- role
- health/freshness summary
- coarse coverage/scope
- overlap/conflict hints
- owner-issued coordination hints

Disallowed implications:

- awareness does not grant sovereign policy authority
- visibility does not grant broad permission
- coordination does not create distributed truth ownership
- peer existence does not imply peer enrollment authority over others

Formula lock:

- visibility is not permission
- awareness is not authority

## Governed mesh membership

Mesh membership is governed and owner-anchored:

- node appears in mesh through owner/workspace trust and membership flow
- peer is not accepted only because it is network-visible
- membership carries role/scope/trust assumptions
- owner plane remains final authority over enrollment and delegated scope

## Flat peer mesh anti-goal

The following model is explicitly out of scope and rejected:

- equal sovereign authority across peers
- discovery == trust
- peer awareness == broad permission
- coordination == distributed canonical truth
- topology distribution == owner sovereignty loss

## Cross-repo impact baseline

### `yai`

- runtime architecture must expose discovery/coordination/authority as distinct
  planes
- peer registry, summary and conflict semantics remain coordination-plane
  concerns under owner sovereignty

### `yai-sdk`

- target and locator model must become mesh-node-role aware
- inspect/query surfaces must distinguish discovery/coordination data from
  authority data

### `yai-cli`

- command surfaces for list/inspect/watch must distinguish:
  - mesh visibility
  - coordination health/freshness
  - authority-owned canonical decisions

### `yai-law`

- governance semantics must preserve owner sovereignty under mesh awareness
- trust/provenance/enrollment semantics must not collapse into flat peer trust

## Consequences for next waves

MF-A1 provides the canonical base for:

- MF-1 Discovery Foundation
- MF-2 Coordination Foundation
- MF-3 Sovereign Authority Foundation (mesh-side)
- MT transport waves and distributed qualification waves

These waves can proceed without re-deciding topology-vs-authority boundaries.

MF-1 and MF-2 concrete foundation docs:

- `docs/architecture/mesh-discovery-foundation-model.md`
- `docs/architecture/mesh-coordination-foundation-model.md`
- `docs/architecture/sovereign-mesh-authority-foundation-model.md`
