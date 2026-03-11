# Exec Source Plane Role

Status: active
Owner: runtime
Effective date: 2026-03-10

## Purpose

Define the canonical source-plane role of `exec` for distributed acquisition
in YAI v1 and lock core/exec boundaries for owner/daemon mediation.

## Positioning

`exec` is no longer treated as a bridge-only historical area for this program
slice. In v1 source-plane architecture, `exec` is the active mediation layer
between edge acquisition (`yai-daemon`) and owner runtime truth (`yai`).

Canonical topology remains:

- distributed acquisition plane
- centralized owner control plane
- no runtime federation in v1

## Responsibilities in source-plane flow

- transport mediation for owner/daemon exchange
- source-owner routing handoff into owner runtime command paths
- acquisition-plane gates before canonical persistence/effect paths
- runtime mediation for source-originated payloads
- guardrail preflight that prevents source-plane bypass around exec

## Boundaries

`exec` does:

- mediate transport and routing in owner-side runtime
- own source-plane mediation preflight (`transport + gates + route`)
- enforce execution-time gates prior to sink/effect paths
- provide bridge/runtime substrate for source-plane integrations

`exec` does not (v1):

- become an independent authority owner
- become a federated runtime coordinator
- own canonical workspace or graph truth
- replace `core` workspace/persistence/graph truth responsibilities

## Core vs exec split (v1 lock)

`core` remains canonical for:

- workspace truth and binding semantics
- authority/enforcement final decisions
- canonical persistence and DB-first read surfaces
- owner graph truth materialization

`exec` remains canonical for:

- transport adaptation and readiness for source-plane flows
- runtime mediation for `yai.source.*` operations
- operational gating (network/resource/storage) before handoff
- bridge substrate for owner/daemon delivery path evolution

## Runtime path (YD-7)

Owner-side source operation path:

1. control call enters runtime session in `core`
2. `yai.source.*` operations are routed to `exec` mediation
3. exec preflight enforces:
   - owner-canonical topology lock
   - transport readiness
   - network/resource/storage gate readiness
4. exec source ingest handler validates and persists source-plane records
5. owner runtime returns reply with mediation metadata

This path is validated by integration test:

- `tests/integration/source_plane/source_owner_ingest_bridge_v1.sh`

## Implementation anchor points

- headers: `include/yai/exec/*.h`
- runtime layer: `lib/exec/runtime/*`
- bridge layer: `lib/exec/bridge/*`
- transport layer: `lib/exec/transport/*`
- gating layer: `lib/exec/gates/*`
- owner ingress hook: `lib/exec/runtime/source_ingest.c`

## Guardrails

- `exec` must mediate `yai.source.*` operations; do not bypass into ad-hoc core
  handlers.
- source-plane gates do not decide authority/evidence final truth.
- daemon-originated payloads do not become canonical truth outside owner runtime
  persistence/materialization.

## Program linkage

- Architecture model:
  - `docs/architecture/distributed-acquisition-plane-model.md`
  - `docs/architecture/ai-grounding-governed-case-state-model.md`
- Decision anchor:
  - `docs/program/22-adr/ADR-013-distributed-acquisition-centralized-control.md`

QG-3 grounding extension:
- `exec` consumes governed case-state context derived from owner-side query and
  unified graph surfaces before agent reasoning;
- this preserves owner sovereignty while enabling context-disciplined AI
  assistance.
