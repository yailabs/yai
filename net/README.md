# NET source map

Authority: current scaffold description only. Provider and resource semantics
are owned by [`docs/reference/boundaries.md`](../docs/reference/boundaries.md).

NET currently consists of C enum/string validation helpers, metadata accessors,
public headers, JSON schemas, and fixtures for stream, node, capability,
endpoint, health, lifecycle, and transport vocabulary. It performs no network
I/O, discovery, routing, live registry, transport, service execution, or YVEX/
CLORI execution.

`make build-net-c` and the `check-net-*` targets validate this scaffold. Empty
or README-only subdirectories are historical module planning, not implemented
subsystems and not future ownership commitments. NET metadata cannot approve an
Operation; an advertised capability is not an ExecutionGrant.
