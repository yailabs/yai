# Source and Edge Query Surfaces Baseline (QG-1)

## Objective

Validate that owner runtime exposes structured query/inspect surfaces across
source, edge, mesh, delegated scope, ingress, and transport/overlay state.

## Baseline checks

1. Verify source/edge class summaries are returned as structured fields.
2. Verify delegated scope validity classes are query-visible.
3. Verify mesh coordination and legitimacy classes are query-visible.
4. Verify transport/ingress/overlay classes are query-visible.
5. Verify summary views remain distinct from final canonical adjudication.

## Expected outcomes

- Workspace query returns multi-family operational summaries.
- Operators can distinguish degraded/restricted/validity states.
- Query surfaces are reusable for CLI/graph/qualification automation.

## Anti-drift assertions

- Query summary != final case truth.
- Transport visibility != authority validity.
- Ingress acceptance visibility != canonicalization completion.
