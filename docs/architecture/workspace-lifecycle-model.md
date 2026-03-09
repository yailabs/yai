# Workspace Lifecycle Model (6/8)

## Lifecycle States

Current canonical states:

- `created`
- `active`
- `attached`
- `suspended`
- `destroyed`
- `error`

## Lifecycle Events

- `create`: create workspace runtime manifest and roots.
- `activate`: set session binding to workspace.
- `clear` / `deactivate`: clear active session binding.
- `resolve`: runtime records inferred/effective summaries.
- `attach`: runtime/control-plane attachment reflected in manifest.
- `destroy`: remove runtime workspace state (manifest + runtime root).

## State Semantics

- `active` is a binding concept.
- `attached` reflects runtime/control-plane attachment.
- `created` does not imply runtime attached.

## Operational Rule

A workspace can be active while shell cwd is outside workspace root.
That is valid and explicit in inspect/status surfaces.

