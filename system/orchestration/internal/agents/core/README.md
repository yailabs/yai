# Agents Implementation Domain (Canonical)

`lib/agents/` is a governed mechanical subsystem of the unified YAI system.

Canonical subdomains:

- `runtime/`
- `dispatch/`
- `roles/`
- `safety/`
- `grounding/`
- `execution/`
- `internal/`

Boundary rules:

- orchestration remains flow-control authority
- agents execute roles and tasks within governed runtime boundaries
- governance constraints and safe primitives bound all agent operations
