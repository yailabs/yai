# Deprecation Policy

Deprecation must be explicit, staged, and reversible within a bounded window.

## Rules

- mark deprecated artifacts with replacement target and cutover deadline
- keep deprecation rationale and impact visible to operators
- require escalation for high-impact removals

## Prohibited

- silent deletion of canonical governance artifacts
- deprecation without migration path
- introducing new dependencies on already deprecated paths
