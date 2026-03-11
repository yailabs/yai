# Edge Binding and Action Point Model (ER-3)

Status: active  
Owner: runtime  
Effective date: 2026-03-11

## Purpose

Define canonical edge binding semantics for subordinate `yai-daemon` runtime,
including first-class action points and scope separation.

## Binding Classes

### Observational bindings

Observation-only surfaces (files, directories, datasets, process/runtime
observation). They can emit events/evidence/state but have no direct local
mediation authority.

### Mediable bindings

Surfaces that may expose delegated local action mediation points under
owner-issued scope.

## Action Points (first-class)

`source_action_point` is a canonical source-plane entity that models a
potential decision point on local/third-party operational flow, for example:

- controlled file operation hook
- process/job dispatch gate
- workflow transition handoff
- API/webhook mediation point

An action point is not the same object as asset, event, or evidence.

## Scope Separation (non-negotiable)

Per binding:

- `binding_scope`: what surface the daemon is attached to
- `observation_scope`: what can be observed
- `mediation_scope`: where edge can interpose evaluation
- `enforcement_scope`: where edge can apply delegated outcomes

These scopes are related but not interchangeable.

## Runtime Baseline in ER-3

Daemon local binding state now carries:

- `binding_kind`
- `binding_scope`
- `observation_scope`
- `mediation_scope`
- `enforcement_scope`
- `mediation_mode`
- `action_point_count`
- `action_points_ref`
- `action_points[]` descriptors

Owner attach flow persists:

- `source_binding` enriched with binding/action metadata
- `source_action_point` records derived from attach payload

## Authority Rule

Binding/action-point capability is delegated execution scope, not sovereign
authority. Owner runtime remains canonical authority for policy, graph,
conflict and final state truth.

## References

- `docs/program/22-adr/ADR-019-edge-binding-and-action-point-model.md`
- `docs/architecture/delegated-edge-enforcement-model.md`
- `docs/architecture/process-and-asset-runtime-observation-model.md`
