# Stub Gap Analysis

## Stub/Placeholder Components Identified
- `yai_precedence_legacy.tla`: placeholder only.
- `yai_resolution_legacy.tla`: placeholder only.
- `control/assurance/model/schema/formal_resolution_trace.v1.schema.json`: unconstrained object.
- Traceability references to non-canonical split-era paths.

## Runtime Gap Side
- Runtime policy/grants/containment were previously declared but weakly represented.
- Formal layer had no explicit invariant mapping to runtime enforcement decisions.

## Closure
- Placeholder TLA modules moved to `docs/transitional/formal-archive/formal-legacy/tla/`.
- Canonical replacements added as concrete modules (`yai_resolution`, `yai_policy_application`, `yai_grants`, `yai_containment`).
- Runtime enforcement bridge map added under `control/assurance/traceability/`.
