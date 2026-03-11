# ADR-030 — Source and Edge Query Surfaces (QG-1)

## Status

Accepted

## Context

RF/ER/SW/MF/MT waves established runtime, authority, mesh, and transport
boundaries, but distributed operation remains opaque without structured inspect
surfaces.

## Decision

YAI adopts canonical source-and-edge query surfaces as a first-class owner-side
operational plane:

- structured inspect/query views over source, edge, mesh, delegated scope,
  transport, ingress, and overlay state
- explicit separation between raw signals, inspect views, and canonical truth
- summary-oriented models for operator and automation consumption

## Consequences

- QG-2 can project unified graph from stable inspect/query semantics.
- DX-2 can consume runtime query views without ad-hoc stitching.
- QW waves can validate behavior using deterministic read surfaces.

## Non-goals

- replacing canonical truth/adjudication with inspect summaries
- reducing query plane to raw low-level dumps
- deriving authority decisions directly from transport-only signals
