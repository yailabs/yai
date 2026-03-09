# Workspace Model

## Purpose

`workspace` is a first-class operational object in `yai`. It is the runtime anchor where session binding, normative context, and inspect/debug state converge before policy resolution is applied.

This model does not introduce dataplane storage or full UX commands. It defines the canonical state contract that future `ws` UX and runtime attachment flows will consume.

## What a workspace is

- Operational boundary for a working context.
- Runtime attachment anchor.
- Carrier of declared normative hints.
- Carrier of inferred/effective normative summaries.
- Inspect/debug surface for last resolution state.

## What a workspace is not

- Not a full OS container manager.
- Not a standalone database.
- Not a UI mode.
- Not a replacement for canonical normative source (`law`).

## Canonical dimensions

A workspace state has these dimensions:

1. `identity`
2. `lifecycle`
3. `session/runtime binding`
4. `declared normative context`
5. `inferred normative context`
6. `effective normative state`
7. `isolation/debug runtime flags`

## Canonical state model

Implemented in `include/yai/core/workspace.h` via:

- `yai_workspace_identity_t`
- `yai_workspace_lifecycle_t`
- `yai_workspace_runtime_binding_t`
- `yai_workspace_declared_context_t`
- `yai_workspace_inferred_context_t`
- `yai_workspace_effective_state_t`
- `yai_workspace_runtime_flags_t`
- `yai_workspace_manifest_v1_t`
- `yai_workspace_inspect_v1_t`

## Declared vs inferred vs effective

- Declared context: explicit workspace hint (`declared_control_family`, `declared_specialization`, `declared_profile`).
- Inferred context: runtime classification outcome from real operations (`last_inferred_family`, `last_inferred_specialization`, confidence).
- Effective context: final stack pointers/summaries used by decision output (`effective_stack_ref`, overlays/effect/authority/evidence summaries).

Workspace hints are strong defaults, not immutable truth. Runtime can confirm, refine, or override by evidence.

## Minimal persistence surface

Workspace persistence remains file-based and minimal:

- Location: `~/.yai/run/<workspace_id>/manifest.json`
- Contract: `yai.workspace.manifest.v1` (`workspace-runtime.v1`)
- Compatibility fields preserved:
  - `ws_id`, `state`, `root_path`, `layout`, `created_at`, `updated_at`
- Canonical sections added:
  - `identity`, `lifecycle`, `binding`, `declared_context`, `inferred_context`, `effective_state`, `runtime`, `inspect`

Persistent fields are identity/lifecycle/context/effective summaries and runtime flags.
Transient fields remain process-owned session internals.

## Runtime binding model

Binding is represented explicitly in manifest/runtime state:

- `session_binding`
- `runtime_attached`
- `runtime_endpoint`
- `control_plane_attached`

This creates a stable contract for later commands like `ws status`, `ws inspect`, and prompt/session indicators.

## Inspect model anchor

`yai_workspace_inspect_v1_t` is the canonical inspect payload shape for future `ws inspect` UX:

- identity + state
- binding
- declared/inferred/effective context
- isolation/debug flags
- last resolution summary

## Boundaries

- Canonical normative logic stays in `law` and is consumed through embedded contract.
- Workspace model in `yai` stores and exposes operational context, not normative authorship.

Session semantics and prompt-facing binding contract are detailed in `docs/architecture/workspace-session-binding.md`.

Architecture refoundation extensions (6/8):

- `docs/architecture/workspace-architecture-model.md`
- `docs/architecture/workspace-root-model.md`
- `docs/architecture/workspace-lifecycle-model.md`
- `docs/architecture/workspace-state-model.md`
- `docs/architecture/workspace-runtime-binding-model.md`
- `docs/architecture/workspace-shell-binding-model.md`
- `docs/architecture/workspace-boundary-model.md`
