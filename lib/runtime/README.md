# Runtime Implementation Domain (Canonical)

`lib/runtime/` is the canonical governed runtime core of YAI.

Implemented runtime subdomains:

- `authority/`
- `dispatch/`
- `enforcement/`
- `lifecycle/`
- `session/`
- `workspace/`
- `policy/`
- `grants/`
- `containment/`
- `internal/`

Boundary rules:

- `governance/` resolves policy content; `runtime/` applies runtime state
- `orchestration/` controls operational flow; `runtime/` executes and stores
  runtime core state
