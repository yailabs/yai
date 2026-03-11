# ER-3 Edge Binding and Action Points Baseline

Status: active  
Owner: runtime  
Effective date: 2026-03-11

## Purpose

Operational baseline for reading and validating edge bindings/action points in
`yai-daemon` and owner-side source-plane records.

## Required Binding Fields

Each binding should expose at least:

- `binding_kind` (`observational` or `mediable`)
- `binding_scope`
- `observation_scope`
- `mediation_scope`
- `enforcement_scope`
- `mediation_mode`
- `action_point_count`
- `action_points_ref`

## Action Point Baseline

When `action_point_count > 0`, daemon state should include `action_points[]`
with:

- `source_action_point_id`
- `action_kind`
- `action_ref`
- `mediation_scope`
- `enforcement_scope`
- `controllability_state`

Owner attach path must persist `source_action_point` records.

## Guardrails

- observational bindings must not imply mediation authority
- mediable bindings remain grant/scoped and owner-governed
- stale/missing delegated state must reduce autonomy, never expand it

## References

- `docs/architecture/edge-binding-and-action-point-model.md`
- `docs/program/22-adr/ADR-019-edge-binding-and-action-point-model.md`
