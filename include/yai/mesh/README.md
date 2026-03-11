# Mesh Public Headers

Canonical mesh topology headers expose real minimal surfaces for:

- identity
- peer registry
- membership
- discovery
- awareness
- coordination
- transport-awareness state
- replay/conflict/containment/enrollment state

Boundary rules:

- `mesh/` models distributed topology and coordination state
- wire contracts remain under `protocol/`
- edge-local acquisition behavior remains under `edge/`
