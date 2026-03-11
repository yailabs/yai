# Mesh Implementation Domain (Canonical)

`lib/mesh/` is the distributed topology domain of the unified YAI system.

Implemented minimal subdomains:

- `identity/`
- `peer_registry/`
- `membership/`
- `discovery/`
- `awareness/`
- `coordination/`
- `transport/`
- `replay/`
- `conflict/`
- `containment/`
- `enrollment/`

Boundary rules:

- mesh tracks distributed state, membership, and coordination
- protocol framing/ABI stays under `lib/protocol/`
- edge runtime process/state stays under `lib/edge/`
