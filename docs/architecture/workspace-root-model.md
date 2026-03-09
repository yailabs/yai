# Workspace Root Model (6/8)

## Canonical Roots

Each workspace exposes four root semantics:

- `workspace_store_root`: parent root containing workspace roots.
- `workspace_root`: physical workspace root path.
- `runtime_state_root`: runtime state location (`~/.yai/run/<ws_id>`).
- `metadata_root`: workspace metadata location (currently co-located with runtime state root).

## Anchor Modes

`root_anchor_mode` is explicit and inspectable:

- `managed_default_root`: default store root (`$HOME/.yai/workspaces`).
- `managed_custom_root`: store root overridden by `YAI_WORKSPACE_ROOT`.
- `explicit_absolute`: create requested an absolute `--root/--path`.
- `explicit_relative`: create requested a relative `--root/--path` resolved from cwd.

## Why This Matters

The root model prevents conflating:

- active binding
- filesystem location
- shell path

Prompt token shows binding, not path ownership.

## Current Implementation Notes

- Manifest keeps canonical root metadata.
- `inspect` and `current` expose root model state.
- cwd relation is exposed as `inside_workspace_root` or `outside_workspace_root`.

