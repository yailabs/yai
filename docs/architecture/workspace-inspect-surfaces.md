# Workspace Inspect Surfaces

This document defines the first canonical inspectability surfaces for an active workspace.

## Scope

Version: `v1` (WS-3/6)

Surfaces:
- `yai.workspace.current`
- `yai.workspace.status`
- `yai.workspace.inspect`
- `yai.workspace.domain.get`
- `yai.workspace.domain.set`
- `yai.workspace.run`
- `yai.workspace.policy.effective`
- `yai.workspace.debug.resolution`

Canonical IDs for registry/CLI are:
- `yai.workspace.domain_get`
- `yai.workspace.domain_set`
- `yai.workspace.run`
- `yai.workspace.policy_effective`
- `yai.workspace.debug_resolution`

The dotted variants above remain accepted as compatibility aliases.

These are runtime-facing surfaces. They are intentionally compact and machine-consumable.

## Surface Semantics

### `current`

Short current view:
- `workspace_id`
- `workspace_alias`
- `state`
- `root_path`
- `session_binding`
- `runtime_attached`
- `binding_status`

### `status`

Operational status summary:
- active/inactive
- binding validity (`active`, `no_active`, `stale`, `invalid`)
- runtime attachment
- isolation/debug flags
- declared/effective context presence

### `inspect`

Expanded workspace view:
- identity
- session/runtime attachment
- declared/inferred/effective normative context
- last effect/authority/evidence summaries
- last resolution summary + trace reference

### `domain get/set`

Declared context handling:
- `get` returns declared context separately from inferred/effective
- `set` updates declared context only

Validation in `set` is against embedded canonical indices:
- `embedded/law/control-families/index/families.index.json`
- `embedded/law/specializations/index/specializations.index.json`

`set` fails for:
- unknown family
- unknown specialization
- specialization/family mismatch
- missing active workspace binding

### `policy effective`

First effective policy summary:
- effective family/specialization (inferred preferred, fallback declared)
- effective stack reference
- overlays summary reference
- effect/authority/evidence summaries

### `debug resolution`

First workspace-centric debug summary:
- context source and declared/inferred context
- effective stack references
- precedence outcome (summary)
- effect outcome
- last trace reference and resolution summary

## Declared / Inferred / Effective

- declared: workspace baseline hint set by user or system
- inferred: runtime result from actual resolution path
- effective: final stack used by runtime after composition

The runtime can refine declared context through inferred/effective results; `domain set` does not overwrite inferred values.

## Contract Notes

- These surfaces are CLI-ready but also internal-runtime-ready.
- They are stable enough for WS-4 scenario execution work.
- Human-friendly formatting is intentionally deferred.
