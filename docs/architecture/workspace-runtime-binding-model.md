# Workspace Runtime Binding Model (6/8)

## Binding Source of Truth

Binding resolution order:

1. `YAI_ACTIVE_WORKSPACE` (explicit override)
2. `~/.yai/session/active_workspace.json`

Binding status values:

- `active`
- `no_active`
- `stale`
- `invalid`

## Runtime Consumption

Runtime uses binding to resolve workspace context before control-call resolution.

Workspace commands (`current/status/inspect/domain/policy/debug/run`) consume this binding model.

## Workspace-local vs Runtime-global

Workspace-local:

- declared context
- inferred/effective summaries
- trace refs
- inspect summaries

Runtime-global (current tranche):

- law embedded corpus
- runtime process primitives
- service-level ingress

Runtime-aware, workspace-scoped:

- effective resolution snapshot
- authority/evidence summaries
- effect outcome references

