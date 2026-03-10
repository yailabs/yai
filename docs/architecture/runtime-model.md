# Runtime Model

`yai` is the runtime host of the platform and executes the internal modules `core`, `exec`, `data`, `graph`, and `knowledge`.

## Canonical runtime flow

1. Ingress request reaches runtime control surface.
2. Workspace identity/binding/context is resolved from active runtime workspace state.
3. Runtime builds classification context from the operation.
4. Embedded law is loaded and validated.
5. Discovery selects family/specialization policy context.
6. Normative stack is resolved (specialization + overlays + authority/evidence composition).
7. Final effect is handed to enforcement.
8. Decision/evidence trace shape is returned for downstream handling and workspace inspect surfaces.

## Repository boundaries

- Normative source of truth is in sibling repo `law`.
- `yai` consumes the runtime-facing export under `embedded/law/`.
- `ops` is qualification/publication bureau and not runtime normative authority.

## Scope note

This runtime model covers the unified runtime topology target where:
- `brain` is not a canonical subsystem.
- execution actors and orchestration are in `exec`.
- graph truth state is in `graph`.
- cognition/memory/provider substrate is in `knowledge`.
- persisted records/query/lifecycle are in `data`.

Workspace model details are defined in `docs/architecture/workspace-model.md`.
