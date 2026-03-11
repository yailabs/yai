---
id: ADR-019
status: accepted
effective_date: 2026-03-11
supersedes: []
applies_to:
  phase: ER-3
  anchor: "#edge-binding-and-action-point-model"
---
# ADR-019 - Edge Binding and Action Point Model

## Context

RF-0.x and ER-1/ER-2 established subordinate edge runtime semantics, but the
runtime still needed a canonical distinction between:

- observation-only surfaces
- mediable surfaces
- first-class action points

Without this distinction, local enforcement and integration mediation become
ambiguous and hard to govern.

## Decision

YAI locks the edge binding model with explicit classes and scopes:

- `observational` bindings
- `mediable` bindings
- first-class `source_action_point` entities

Each binding now carries explicit scope fields:

- `binding_scope`
- `observation_scope`
- `mediation_scope`
- `enforcement_scope`

`action_points` are modeled as binding-attached descriptors and persisted
owner-side as `source_action_point` records during attach flow.

## Authority Rule

Binding/mapping of action points does not transfer sovereignty to edge runtime.
All mediation/enforcement remains delegated, owner-issued, scope-limited, and
revocable.

## Consequences

### Positive

- explicit controllable integration surface for ER-4
- cleaner read/query/graph integration for action mediation visibility
- less ambiguity across local integrations and third-party handoff points

### Negative

- larger binding payload/state surface
- requires follow-up on grants/validity coupling for action scopes

## Non-goals

- full mediation engine implementation
- full third-party connector framework
- autonomous edge policy authoring

## References

- `docs/architecture/edge-binding-and-action-point-model.md`
- `docs/architecture/delegated-edge-enforcement-model.md`
- `docs/architecture/workspace-runtime-binding-model.md`
