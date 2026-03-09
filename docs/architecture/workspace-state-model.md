# Workspace State Model (6/8)

## State Strata

### Identity state

- workspace id / alias
- workspace root
- lifecycle state

### Root model state

- workspace store root
- runtime state root
- metadata root
- root anchor mode

### Binding state

- session binding
- runtime attached
- control-plane attached
- shell cwd relation to workspace root

### Normative state

- declared family/specialization/source
- inferred family/specialization/confidence
- effective stack/overlays/effect/authority/evidence summaries

### Debug state

- last resolution summary
- last resolution trace ref

## Persistence Surface

Manifest path:

- `~/.yai/run/<workspace_id>/manifest.json`

Schema:

- `yai.workspace.manifest.v1`
- `workspace-runtime.v1`

The manifest is canonical for workspace runtime state and architecture metadata.

