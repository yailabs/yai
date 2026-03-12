---
role: canonical
status: active
audience: architect
owner_domain: architecture
related: [docs/architecture/overview/system-overview.md]
---
# Repository Map

# Purpose
Map architecture sections to concrete repository surfaces.

# Scope
Connects architecture documentation to implementation and verification directories.

# Relationships
- `system-overview.md`
- `subsystem-map.md`

# Canonical Role
Primary architecture-to-repo alignment map.

# Main Body
## Alignment
- `foundation/` docs -> repo `foundation/`
- `control/assurance/` docs -> repo `control/assurance/`
- architecture runtime domains -> repo `include/yai/**`, `lib/**`, `cmd/**`
- operational validation -> repo `tests/**`

# Related Docs
- `system-overview.md`
- `subsystem-map.md`
