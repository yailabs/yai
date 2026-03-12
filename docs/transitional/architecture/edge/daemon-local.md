---
role: canonical
status: active
audience: architect
owner_domain: architecture
primary_for: edge-architecture
---

# Daemon Local Runtime Model (YD-5)

# Purpose
Defines canonical architecture semantics for the domain.

# Scope
Covers boundaries, responsibilities, and integration semantics for this domain section.

# Relationships
- Parent section README
- Adjacent architecture source documents

# Canonical Role
Authoritative architecture source for its scope.

# Main Body
## Scope

`yai-daemon` is a subordinate edge runtime. In v1 it is intentionally
non-sovereign: no workspace truth, no policy truth, no final policy/authority/
enforcement/conflict truth, no local graph truth.

## Runtime Planes

`yai-daemon` runs four explicit local planes:

1. `binding` plane
   Loads and activates local source bindings from
   `config/source-bindings.manifest.json`.
2. `observation` plane
   Performs polling scan on active filesystem bindings and detects new/changed
   assets through local fingerprints (`size` + `mtime` baseline).
   RF-0.4 extends this plane semantics to process/runtime observables
   (still owner-scoped and governed).
3. `spool` plane
   Persists acquisition units on disk under `spool/queue` and tracks
   `delivered` / `failed` units.
4. `delivery` plane
   Sends source-plane operations to owner runtime:
   `yai.source.enroll`, `yai.source.attach`, `yai.source.emit`,
   `yai.source.status`.

Plus delegated local runtime hooks (owner-scoped only):

5. `edge mediation` plane
   Local action mediation and delegated enforcement points may execute only
   under owner-issued policy snapshots, grants, and capability envelopes.

RF-0.3 enforcement modes:

- observe-only
- post-event local enforcement
- preventive local enforcement
- escalated enforcement

RF-0.3 local outcomes baseline:

- observe_only / allow / block / hold / execute / escalate / defer
- deny_due_to_missing_scope / deny_due_to_expired_grant

RF-0.4 observation classes baseline:

- asset observables
- process observables
- runtime observables

Observation scope remains distinct from mediation/enforcement scope.

## Local Layout

YD-5 actively uses daemon-local roots:

- `~/.yai/daemon/config`
- `~/.yai/daemon/state`
- `~/.yai/daemon/log`
- `~/.yai/daemon/spool`
  - `queue`
  - `delivered`
  - `failed`
- `~/.yai/daemon/identity`
- `~/.yai/daemon/bindings`

Runtime state files:

- `state/health.v1.json` (v3 payload with health+spool+resilience counters)
- `state/bindings.v1.json`
- `state/observed-assets.v1.tsv`
- `state/edge-runtime-state.v1.json` (ER-1 lifecycle + edge state baseline)
- `state/edge-operational-state.v1.json` (ER-2 spool/retry/health/freshness)

## ER-1 Lifecycle Baseline

`yai-daemon` lifecycle is explicit:

- bootstrap
- config_load
- identity_init
- delegated_scope_init
- runtime_start
- observation_loop
- degraded/disconnected (conditional)
- shutdown
- stopped

ER-1 also introduces explicit edge service surfaces as runtime modules:
observation, state, delegated_policy, mediation, spool_retry, health,
owner_link.

## ER-2 Resilience Baseline

ER-2 makes local durability and resilience first-class runtime behavior:

- spool queue is canonical edge-local durability surface (`spool/queue`);
- retry/redelivery follows bounded backoff and terminal-failure demotion;
- health is computed with explicit connectivity/freshness/pressure signals;
- disconnected/degraded operation keeps local continuity without creating
  owner-side truth.

ER-2 operational states include:

- connectivity: `connected|disconnected|unconfigured`
- freshness: `fresh|aging|stale|unknown`
- pressures: `spool_pressure` and `retry_pressure` as `low|medium|high`
- policy/grant placeholders: `policy_staleness_state`, `grant_validity_state`
- degradation reason baseline (e.g. `delivery_degraded`, `retry_saturated`)

## Binding/Unit/Health States

Binding state:

- `configured`
- `active`
- `degraded`
- `invalid`

Acquisition unit state:

- `discovered`
- `queued`
- `emitting`
- `delivered`
- `retry_due`
- `failed_terminal`

Daemon health state:

- `starting`
- `ready`
- `degraded`
- `disconnected`
- `stopping`

## Retry Baseline

- Failed delivery remains in spool.
- Retry uses simple exponential backoff baseline.
- Attempts are persisted with each unit.
- After max attempts, unit is moved to `spool/failed`.

This is intentionally minimal v1 behavior for deterministic edge validation.

## Sovereignty Boundary

Edge runtime can execute delegated local behavior but remains subordinate:

- global/workspace policy truth remains owner-side
- canonical graph/persistence/conflict truth remains owner-side
- daemon-local outcomes are advisory until owner acceptance/materialization

# Related Docs
- `docs/architecture/README.md`
- Domain-adjacent architecture documents

## Policy Enforcement Boundary

Delegated enforcement executes at edge runtime scope only under owner-issued
policy snapshots and grants. Final authority, conflict adjudication, and
canonical policy truth remain owner-side (`lib/runtime/enforcement/`,
`lib/governance/policy_effects/`).
