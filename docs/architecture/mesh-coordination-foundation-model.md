# Mesh Coordination Foundation Model (MF-2)

## Purpose

Define the canonical Mesh Coordination Plane for the Governed Sovereign Mesh.

Coordination is the plane that turns discovered mesh nodes into governed,
workspace-aware operational members, with explicit registry, awareness, and
coverage/overlap/ordering/replay coordination semantics.

## Coordination plane boundaries

In scope:

- governed mesh membership
- owner-anchored peer registry
- peer awareness metadata
- coverage/overlap/gap representation
- baseline scheduling/ordering/replay/conflict coordination signals

Out of scope:

- sovereign policy authority transfer
- final conflict adjudication transfer
- canonical truth relocation from owner runtime

## Coordination vs other mesh planes

- Discovery plane: makes nodes visible/bootstrap-ready.
- Coordination plane: organizes distributed operation among governed members.
- Sovereign authority plane: retains canonical policy/truth/final adjudication.

Formula lock:

coordination organizes distributed operation; it does not dissolve sovereign
adjudication.

## Governed mesh membership

A discovered node becomes a coordinated mesh member only when owner/workspace
membership state is established.

Minimal membership semantics:

- `owner_workspace_id`
- `source_node_id`
- `source_binding_id`
- `daemon_instance_id`
- `peer_role`
- `peer_scope`
- `peer_state`
- `coverage_ref`
- `overlap_state`
- `updated_at_epoch`

Discovery visibility alone is insufficient for coordinated membership.

## Owner-anchored peer registry

Owner-side registry is the canonical coordination substrate for runtime
operation.

Registry baseline includes:

- membership identity tuple (workspace/node/binding)
- peer role/scope/state
- freshness and activity
- backlog pressure
- coverage and overlap
- ordering/replay/conflict counters
- review-required and handling hints

## Peer awareness model (minimum)

Allowed awareness metadata across peers (scope-governed):

- peer existence in same workspace mesh
- role
- freshness/availability summary
- synthetic health/pressure
- coverage/overlap summary
- owner-issued coordination hints

Disallowed implication:

- awareness metadata does not create sovereign policy authority or canonical
  truth ownership.

## Coverage / overlap / scheduling baseline

Coordination plane must support:

- coverage distinctness and gaps
- overlap/collision risk visibility
- scheduling baseline states (`nominal`, `backlog_pressure`,
  `attention_required`)
- replay/order/conflict pressure rollups for operator/runtime decisions

## Ordering, replay, conflict relation

Coordination plane can:

- detect and represent distributed integrity pressure
- aggregate per-peer ordering/replay/overlap/conflict signals
- expose them to summary/query/graph surfaces

Coordination plane cannot:

- finalize conflict truth sovereignly
- override owner canonical adjudication

## Cross-repo impact baseline

### `yai`

- owner runtime registry/membership/awareness remain explicit coordination
  components
- read surfaces and summaries consume coordination state as first-class input

### `yai-law`

- law alignment must constrain membership and awareness scope semantics
- law alignment must preserve sovereignty boundary between coordination and
  authority/final adjudication

## Handoffs

MF-2 provides baseline for:

- MF-3 sovereign mesh authority lock
- MT-2 remote ingress coordination coupling
- QG read/graph coordination surfaces
- QW scale/replay/overlap distributed qualification

MF-3 authority follow-up:

- `docs/architecture/sovereign-mesh-authority-foundation-model.md`
