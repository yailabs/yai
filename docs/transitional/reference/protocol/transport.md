---
role: reference
status: active
audience: developer
owner_domain: reference
depends_on: [docs/reference/protocol/README.md]
---

# Transport

# Purpose
Define transport contract surfaces and integration boundaries.

# Scope
Covers transport headers, contracts, and runtime transport implementation.

# Relationships
- `surface.md`
- `binary.md`

# Canonical Role
Lookup reference for transport contracts.

# Main Body
Transport contracts are defined in `include/yai/protocol/transport_contract.h` and related transport headers. Runtime transport behavior lives under `lib/control/protocol/transport/`.

# Related Docs
- `docs/architecture/protocol/transport.md`
