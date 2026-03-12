# Cognition Implementation Domain (Canonical)

`lib/cognition/` is governed internal cognition/memory infrastructure.

Canonical subdomains:

- `cognition/`
- `memory/`
- `semantic/`
- `episodic/`
- `vector/`
- `internal/`

Domain responsibility:

- cognition: reasoning roles, scoring, synthesis, controlled cognitive flow
- memory: persistence model, storage bridge, retrieval interfaces
- semantic: structured meaning layer
- episodic: event-history and experience memory
- vector: vector indexing and nearest-neighbor retrieval

Boundary rules:

- `cognition/` is not `graph/` state materialization.
- `cognition/` is not `agents/` role orchestration.
- `cognition/` is runtime-governed infrastructure, not an external AI subsystem.
