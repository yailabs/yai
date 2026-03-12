# Authority Status Model

Status classes for governance artifacts and decisions.

## Artifact status

- draft: not authoritative
- review: pending authority decision
- canonical: authoritative and active
- deprecated: superseded, pending removal
- retired: no longer active

## Decision status

- proposed
- approved
- rejected
- escalated
- superseded

All status transitions must preserve traceability.
