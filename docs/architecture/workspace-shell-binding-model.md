# Workspace Shell Binding Model (6/8)

## Principle

Shell prompt token represents active workspace binding, not cwd.

This is intentionally Git-like:

- branch indicator != path root
- workspace token != workspace root path

## Semantics

- Token present only when binding is active.
- Token alias comes from workspace identity alias.
- Token remains valid even if cwd is outside workspace root.
- Token disappears when binding is cleared.

## Diagnostics

Inspect surfaces expose:

- shell cwd
- cwd relation to workspace root (`inside_workspace_root`, `outside_workspace_root`)

This prevents path/binding confusion during operations and debugging.

