---
id: A-001
title: Governed Execution
type: Axiom
status: Canonical
version: 2.0.0
---

# A-001 — Governed Execution

Execution is a committed transition under governance.
Execution is valid only when:
1. authority checks are satisfied in `core`, and
2. effects are performed through `exec` boundaries, and
3. evidence is attributable and inspectable.

Execution is not inference.
Execution is not proposal.
Execution is not an ungoverned side effect.

## Normative statements

- Execution MUST be represented as a governed transition with traceable inputs, decision basis, and outcomes.
- `exec` performs effects but does not authorize them.
- `brain` may propose plans and actions but does not commit execution authority.
- Any path where inference directly causes effects without authority evaluation is invalid by definition.

## Plane mapping

- `core`: authority and transition legitimacy
- `exec`: effect realization under constraint
- `brain`: cognitive proposal and adaptation support

## Consequence

A system that executes outside this structure is not YAI-conformant.
