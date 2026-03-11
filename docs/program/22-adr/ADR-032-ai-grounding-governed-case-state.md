# ADR-032 — AI Grounding over Governed Case State (QG-3)

## Status

Accepted

## Context

QG-1 and QG-2 provide structured inspect surfaces and unified graph
representation. AI/agent behavior remains unsafe if it consumes raw, unscoped
runtime exhaust directly.

## Decision

YAI adopts governed case-state grounding for AI and `exec`:

- separate raw inputs from inspect surfaces and graph representation
- build task-scoped grounding contexts from owner-side governed summaries
- include provenance/freshness/validity/adjudication boundary markers
- keep AI reasoning explicitly non-sovereign

## Consequences

- Agent behavior becomes explainable and auditable against governed state.
- `exec` aligns with workspace sovereignty rather than ad-hoc prompt context.
- DX/QW waves can validate grounded-AI behavior in distributed scenarios.

## Non-goals

- AI as autonomous final adjudication authority
- direct grounding from peer-local or transport-only raw signals
- bypassing owner governance boundaries through prompts
