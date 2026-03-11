# Runtime Model

`yai` is the runtime host of the platform and executes the internal modules `core`, `exec`, `data`, `graph`, and `knowledge`.

For distributed source acquisition v1, `yai` remains the centralized owner
runtime while edge acquisition is handled by standalone `yai-daemon` processes.

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
- source-plane transport mediation/handoff is an active `exec` responsibility.

Workspace model details are defined in `docs/architecture/workspace-model.md`.
Distributed acquisition topology details are defined in
`docs/architecture/distributed-acquisition-plane-model.md`.
Source-plane entity/contract model is defined in
`docs/architecture/source-plane-model.md`.
Owner ingest runtime flow is defined in
`docs/architecture/source-owner-ingest-model.md`.
Daemon local runtime behavior is defined in
`docs/architecture/daemon-local-runtime-model.md`.
Source-plane query/graph read surfaces are defined in
`docs/architecture/source-plane-read-model.md`.

Secure peering baseline (NP-1):
- local runtime control ingress and remote peer ingress are distinct concerns.
- binary protocol semantics remain application-level and ride over secure peering transport.
- recommended non-local deployment path is private overlay peering (WireGuard/equivalent).

Secure overlay operationalization (NP-4):
- private overlay integration is the canonical deployment model for customer-grade
  non-local owner/peer operation.
- owner/peer bootstrap sequence is defined in
  `docs/program/23-runbooks/owner-peer-overlay-bootstrap.md`.
