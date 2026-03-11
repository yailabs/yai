# Source and Edge Query Surfaces Model (QG-1)

## Purpose

Define canonical owner-side inspect/query surfaces for source plane, edge
runtime, mesh coordination, delegated scope, and transport/overlay state.

QG-1 makes distributed runtime behavior interrogable as structured views instead
of raw log-only introspection.

## Canonical thesis

Inspect/query surfaces are a first-class operational plane.

Formula lock:

inspectability is governed visibility into runtime state, not automatic
equivalence with final case truth.

## Query plane scope

QG-1 surfaces must expose at least:

- source nodes and edge runtime identities
- mesh peer membership and awareness metadata
- bindings and action points
- grants, policy snapshots, capability envelopes
- mediation outcomes and restriction states
- health/freshness/connectivity and spool/retry pressure
- ingress state and decision reasons
- transport/overlay/path mutation state

## Three-layer distinction

### Raw runtime signals

Low-level and potentially noisy local/transport/process signals.

### Structured inspect/query surfaces

Owner-governed, normalized read views over runtime state and summarized
operational condition.

### Canonical truth

Owner-side final adjudication and canonical case/graph truth.

QG-1 standardizes the middle layer without collapsing it into canonical truth.

## Interrogable class families

### Source/edge identity and state

- source node
- daemon instance
- edge lifecycle/health/freshness
- spool/retry/backlog pressure

### Binding, mediation, delegated scope

- source binding and typed binding role
- action points and mediation eligibility
- enrollment grant/policy snapshot/capability envelope validity state

### Mesh and coordination

- mesh node/discovery/bootstrap
- coordination membership/awareness/coverage/overlap
- legitimacy/authority-scope constraint state

### Transport and ingress

- transport endpoint/path/channel state
- owner remote ingress state/session/decision
- overlay presence/target association/path mutation

## Summary-oriented read model

QG-1 requires summary views in addition to low-level record tails:

- workspace source/edge summary
- peer and membership summary
- delegated-scope validity summary
- transport/overlay/ingress summary
- coordination pressure summary

Summaries are for operators and automation gates, not only for debugging.

## Runtime and CLI/graph handoff

QG-1 is designed for reuse by:

- QG-2 unified graph materialization/query
- QG-3 AI grounding over governed runtime state
- DX-2 CLI inspect/watch surfaces
- qualification and readiness waves (QW)

## Guardrails

- Query visibility must not imply authority elevation.
- Reachability/transport visibility must not imply trust or scope validity.
- Ingress acceptance visibility must not imply canonicalization.
- Summary views must remain workspace-owner governed.
