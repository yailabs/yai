# Runtime Policy Domain

`lib/runtime/policy/` materializes effective governance outcomes into runtime
policy state.

Responsibilities:

- runtime policy state lifecycle
- effective snapshot application runtime-side
- policy state export for runtime consumers

Boundary:

- governance resolves policy stacks
- runtime policy applies resolved outcomes to runtime state
