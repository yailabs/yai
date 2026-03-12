---
role: canonical
status: active
audience: architect
owner_domain: architecture
primary_for: architecture-index
---
# Architecture Documentation

# Purpose
Provide the canonical architecture surface for the YAI platform.

## Scope
Canonical architecture source-of-truth for subsystem boundaries and platform model.

## What Belongs Here
- Architecture authority docs for overview, foundation, formal, runtime, orchestration, edge, protocol, data, graph, knowledge, mesh, workspace.

## What Does Not Belong Here
- Runbook procedures.
- Report history.
- Reference lookup tables.

## Navigation Order
1. `overview/system-overview.md`
2. `overview/subsystem-map.md`
3. `overview/repository-map.md`
4. `foundation/`
5. `control/assurance/`
6. `runtime/`
7. `orchestration/`
8. `edge/`
9. `protocol/`
10. `data/`
11. `graph/`
12. `knowledge/`
13. `mesh/`
14. `workspace/`

## Extension Rules
- New architecture docs must map directly to real code surfaces.
- Avoid parallel satellites that duplicate canonical owner docs.

# Relationships
- `foundation/`
- `control/assurance/`
- `include/yai/`
- `lib/`
- `cmd/`
- `tests/`

# Canonical Role
Primary architecture source-of-truth index.

# Main Body
Use this section as entrypoint for platform architecture authority.

# Related Docs
- `docs/README.md`
- `docs/reference/README.md`
- `docs/runbooks/README.md`
