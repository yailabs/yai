# Distributed Acquisition Plane Model

Status: active
Owner: runtime
Effective date: 2026-03-10

## Purpose

Define the canonical source-plane topology for YAI v1:

- distributed acquisition plane
- centralized control plane

This model introduces `yai-daemon` as the subordinate edge runtime while
preserving `yai` as the only runtime owner and workspace source of truth.

## Canonical topology (v1)

- Owner runtime: `yai` (central control plane and canonical truth).
- Edge daemon: `yai-daemon` (subordinate edge runtime).
- Source nodes: machines that host subordinate edge runtimes attached to one
  owner runtime.

Topology class:

- one owner runtime
- one or more source nodes
- no runtime federation

## Role split

### `yai` (owner runtime)

- owns lifecycle and control-plane authority
- owns workspace truth and runtime binding truth
- owns final policy/authority/evidence/enforcement decisions
- owns canonical persistence and graph truth

### `yai-daemon` (subordinate edge runtime)

- runs local observation loops (assets/process-local signals in baseline scope)
- mediates source-side attach/emit/status toward owner runtime
- can mediate local action points and delegated local enforcement only when
  owner-issued scope/grants allow it
- can spool/buffer transient acquisition payloads and handle retry/reconnect
- does not own workspace truth
- does not own final policy/authority/evidence/enforcement
- does not own canonical graph truth
- does not own policy truth or conflict-resolution truth

### `exec` runtime family

`exec` is the active mediation layer for owner/daemon exchange:

- transport mediation
- routing handoff owner <-> daemon
- acquisition-plane gating
- runtime-side dispatch bridge into canonical control and persistence paths

`exec` is not treated as a legacy bridge-only area in this model.
It mediates delegated edge behavior under owner-issued constraints and does not
form a sovereign authority plane.

## Control-plane and acquisition-plane contract

### Centralized control plane

Control and final truth stay owner-side:

- command authority
- runtime readiness and acceptance checks
- workspace binding semantics
- canonical records and graph state
- policy issuance and delegated capability authority

### Distributed acquisition plane

Acquisition and delegated local execution can run edge-side, but only as
subordinate behavior under owner-issued scope and only as upstream feed into
owner truth.

- edge emits candidate/source payloads
- owner validates, governs, and persists canonical state

## Daemon local topology baseline (YD-2)

`yai-daemon` keeps local process state under a daemon-specific root and does not
reuse owner workspace runtime roots as daemon home:

- `~/.yai/daemon/config/`
- `~/.yai/daemon/state/`
- `~/.yai/daemon/log/`
- `~/.yai/daemon/spool/` (placeholder)
- `~/.yai/daemon/identity/` (placeholder)
- `~/.yai/daemon/run/`

This local topology is subordinate runtime state only; it is not canonical
workspace truth, policy truth, graph truth, or conflict truth authority.

## Non-goals (v1)

- runtime-to-runtime federation
- peer owner election
- independent edge workspace truth
- independent edge graph truth
- final authority at edge
- secure-mesh/WireGuard-native transport design in this tranche

## Invariants

1. `yai` remains the only owner runtime source of truth.
2. `yai-daemon` is a standalone subordinate edge runtime, not a second owner runtime.
3. Source-node state is advisory/transient unless accepted by owner runtime.
4. Workspace canonical truth never migrates to edge nodes.
5. Control-plane decisions are centralized even with distributed acquisition.

## Naming lock (canonical)

- edge binary: `yai-daemon`
- owner runtime: `yai`
- topology: `distributed acquisition plane / centralized control plane`
- remote participant: `source node`

Non-canonical forms:

- `yai-source`
- `source runtime` (as owner-equivalent runtime)
- `peer runtime` (for v1 topology)
- `federated runtime` (for v1 topology)

## Downstream tranche anchors

- YD-2: `yai-daemon` binary skeleton + build integration
- YD-3: source-plane records/IDs/contracts
- YD-4: owner ingest core + transport bridge
- YD-5: daemon local runtime (binding/scan/spool/retry/health)
- YD-7: exec verticalization for distributed acquisition

Canonical source-plane entity/ID/contract model:
- `docs/architecture/source-plane-model.md`
- `docs/architecture/daemon-local-runtime-model.md`
- `docs/architecture/source-plane-model-refoundation-rf01.md`
