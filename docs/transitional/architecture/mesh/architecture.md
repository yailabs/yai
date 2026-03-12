---
role: canonical
status: active
audience: architect
owner_domain: architecture
primary_for: mesh-architecture
---

# Mesh Architecture

# Purpose
Define topology and coordination boundaries for mesh operation.

# Scope
Covers discovery, enrollment, membership, coordination, and transport alignment.

# Relationships
- `include/yai/mesh/`
- `lib/mesh/`
- `include/yai/protocol/transport/`
- `lib/control/protocol/transport/`

# Canonical Role
Authoritative architecture source for mesh behavior.

# Main Body
Mesh topology enables distributed coordination while preserving owner-side authority for policy and canonical truth.

## Core Responsibilities
- discovery and awareness
- enrollment and membership lifecycle
- coordination and peer registry state
- transport-state integration for remote peers

# Related Docs
- `docs/architecture/edge/daemon-local.md`
- `docs/architecture/protocol/transport.md`
