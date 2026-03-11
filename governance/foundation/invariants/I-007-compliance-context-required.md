# I-007 — Compliance Context Required

Compliance context is mandatory for external effects.

## Invariant

For every external-effect transition:
- compliance context MUST validate at decision time
- authority MUST be explicit and non-null

## Required properties

- deny/error on missing or invalid compliance context
- evidence must prove context validity offline
- no fallback path may authorize external effects without context

## Ontology alignment

Compliance context validation is evaluated in `core` authority flow and enforced at the `exec` effect boundary.
