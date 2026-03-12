# Formal System

`control/assurance/` is the canonical mathematical and verification layer for the unified
YAI system.

Canonical structure:

- `audit/` current-state and migration autopsy
- `models/` root composition specs
- `modules/` decomposed TLA modules by domain semantics
- `configs/` model-checking profiles
- `artifacts/` derived state/report outputs
- `model/schema/` formal metadata schemas
- `traceability/` invariant linkage maps
- `checks/` formal quality checks
- `legacy/` historical kernel-era formal remnants (non-canonical)

Formal-enforcement rule:

- `control/assurance/` defines invariants, violation classes, and transition legality
- `runtime/enforcement/` applies and records those outcomes at runtime
