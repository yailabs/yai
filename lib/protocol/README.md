# Protocol Implementation Domain (Canonical)

`lib/protocol/` is the canonical exchange and contract domain for unified YAI.

Canonical subdomains:

- `rpc/`
- `binary/`
- `contracts/`
- `transport/`
- `internal/`

Boundary rules:

- Protocol transport defines wire/runtime exchange assumptions.
- Mesh transport (`lib/network/transport/`) defines topology/reachability awareness.
- Control plane and source plane contracts live under `protocol/contracts/`.
