# Workspace Session Binding

## Purpose

Define canonical session binding semantics for workspace activation and resolution in `yai` runtime.

This model makes active workspace state explicit and inspectable without requiring full CLI/prompt implementation.

## Active workspace semantics

Binding states:

- `active`: binding resolves to an existing workspace manifest.
- `no_active`: no binding exists.
- `stale`: binding exists but referenced workspace is missing.
- `invalid`: binding payload/id is malformed.

Terms:

- Active workspace: workspace selected as primary session context.
- Current workspace: resolved active workspace snapshot when binding is valid.

## Source of truth

Binding resolution order:

1. `YAI_ACTIVE_WORKSPACE` env override (if set)
2. per-terminal session binding file `~/.yai/session/by-tty/<tty>.json` (when TTY is available)
3. session-global binding file `~/.yai/session/active_workspace.json` (non-interactive fallback)

If env override is invalid, state is `invalid`.
If binding points to missing workspace manifest, state is `stale`.

## Persistence surface

Binding file contract:

- path (interactive): `~/.yai/session/by-tty/<tty>.json`
- path (fallback): `~/.yai/session/active_workspace.json`
- type: `yai.workspace.binding.v1`
- fields:
  - `workspace_id`
  - `workspace_alias`
  - `bound_at`
  - `source`

Workspace canonical manifest remains at:

- `~/.yai/run/<workspace_id>/manifest.json`

## Runtime operations

Implemented runtime control commands (`command_id` in control payload):

- `yai.workspace.create`
- `yai.workspace.reset`
- `yai.workspace.destroy`
- `yai.workspace.set` / `yai.workspace.switch`
- `yai.workspace.current`
- `yai.workspace.unset` (binding clear)
- `yai.workspace.clear` (runtime-state clear)
- `yai.workspace.prompt_context`

These are lightweight session/context primitives, not full user-facing CLI workflows.

## Runtime consumption

Runtime reads current workspace binding before context export operations and reports status explicitly (`active|no_active|stale|invalid`).

This allows downstream inspect/prompt UX to be deterministic and debuggable.

## Shell integration boundary

- Workspace activation/binding does not mutate user shell configuration by default.
- Managed shell file bootstrap is explicit opt-in only (`YAI_SHELL_INTEGRATION_MODE=managed`).
