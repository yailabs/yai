# yai repository scope

## Role

`yai` is the runtime implementation repository and law consumer.

## In scope

- runtime ingress, lifecycle, dispatch, and enforcement realization
- internal unified runtime composition (`core`, `exec`, `data`, `graph`, `knowledge`)
- workspace-first runtime binding and active workspace capability attachment
- runtime consumption/integration of canonical law surfaces

## Out of scope (this tranche)

- canonical law authorship (owned by `law`)
- ops official qualification/collateral publishing (owned by `ops`)
- external consumer API and operator UX ownership (owned by `sdk` and `cli`)

## Dependency boundary

`ops` is never a normative source for runtime behavior.
Normative authority comes from `law`.
