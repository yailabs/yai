# Source Plane Model

Status: active
Owner: runtime
Effective date: 2026-03-10

## Purpose

Define the canonical source-plane model that links `yai-daemon` (edge acquisition)
to owner runtime truth in `yai`.

This model is the baseline for YD-3 and is intentionally scoped to entities,
IDs, contracts, and persistence hooks. It does not claim full transport or
operational completion.

## Design constraints

- `yai` owner runtime remains the only workspace truth source.
- Source plane is distributed only on acquisition side.
- Control plane stays centralized owner-side.
- No runtime federation in v1.
- No final authority/enforcement/graph truth at source node.

## Canonical entities

### 1) `source_node`

Identity of a source machine/node.

Minimum fields:

- `source_node_id`
- source label/hostname
- trust metadata placeholder
- owner association placeholder
- state

### 2) `source_daemon_instance`

Runtime instance identity of `yai-daemon` on a source node.

Minimum fields:

- `daemon_instance_id`
- `source_node_id`
- session/runtime marker
- start timestamp
- health/status
- build/version marker

### 3) `source_binding`

First-class attach relationship between source node and owner workspace context.

Minimum fields:

- `source_binding_id`
- `source_node_id`
- `owner_workspace_id`
- binding scope/type
- attachment status
- constraints placeholder

### 4) `source_asset`

Observed source-side asset identity (not owner canonical artifact truth).

Minimum fields:

- `source_asset_id`
- `source_binding_id`
- source locator/path/reference
- asset type
- provenance fingerprint
- observation state

### 5) `source_acquisition_event`

Acquisition event generated from source observation/emit steps.

Minimum fields:

- `source_acquisition_event_id`
- `source_node_id`
- `source_binding_id`
- `source_asset_id`
- event type
- observed timestamp
- idempotency marker
- delivery status

### 6) `source_evidence_candidate`

Evidence candidate emitted by source plane before owner-side acceptance.

Minimum fields:

- `source_evidence_candidate_id`
- `source_acquisition_event_id`
- candidate type
- derived metadata placeholder
- owner resolution/acceptance status

### 7) `source_owner_link`

Registration/trust link semantics between source-side identity and owner plane.

Minimum fields:

- `source_owner_link_id`
- `source_node_id`
- owner reference
- registration status
- registration timestamp

### 8) `source_enrollment_grant`

Owner-issued bootstrap artifact that records enrollment decision and v1 trust
token used by peer operations after enrollment.

Minimum fields:

- `source_enrollment_grant_id`
- `source_node_id`
- `daemon_instance_id`
- `owner_ref`
- `enrollment_decision`
- `trust_artifact_id`
- `trust_artifact_token`
- `issued_at_epoch`

## Distinctions (non-negotiable)

- `source_node` != `source_daemon_instance`
- `source_asset` != owner canonical artifact
- `source_evidence_candidate` != owner final evidence
- `source_binding` is explicit first-class relation, not implicit context

## Canonical record classes

Runtime data-plane record classes introduced by YD-3:

- `source_node`
- `source_daemon_instance`
- `source_binding`
- `source_asset`
- `source_acquisition_event`
- `source_evidence_candidate`
- `source_owner_link`
- `source_enrollment_grant`

These are appendable through `yai_data_records_*` hooks and visible in summary
counts.

## ID model

YD-3 introduces dedicated source ID helpers:

- `sn-*` source node IDs
- `sd-*` daemon instance IDs
- `sb-*` source binding IDs
- `sa-*` source asset IDs
- `se-*` source acquisition event IDs
- `sc-*` source evidence candidate IDs
- `sl-*` source owner link IDs
- `sg-*` source enrollment grant IDs

IDs are deterministic enough for local consistency and are shaped for future
cross-plane traceability.

## Contract surface (model-level)

Source-plane logical operations are canonicalized as:

- `enroll`
- `attach`
- `emit`
- `status`

Model-level call/reply contract type IDs are reserved for each operation:

- `yai.source.enroll.call.v1` / `yai.source.enroll.reply.v1`
- `yai.source.attach.call.v1` / `yai.source.attach.reply.v1`
- `yai.source.emit.call.v1` / `yai.source.emit.reply.v1`
- `yai.source.status.call.v1` / `yai.source.status.reply.v1`

Enrollment reply now includes owner-issued trust bootstrap fields
(`owner_trust_artifact_id`, `owner_trust_artifact_token`) for subsequent
attach/emit/status operations.

These define operation intent and shape anchors for YD-4/YD-5 transport and
owner-ingest work.

## Persistence slice

The YD-3 persistence baseline does not add a second store topology.
It reuses canonical data-plane append flow in owner runtime:

- class-aware append into LMDB/DuckDB-backed runtime records
- source-plane classes countable via inspect summary
- no side-channel source truth stores

## Graph linkage roadmap (for YD-6)

Source-plane graph projection classes are locked for next slice:

- source node
- daemon instance
- binding
- asset
- acquisition event
- evidence candidate
- owner link

YD-3 defines naming/contracts/record shape so YD-6 can materialize graph
without re-inventing source entities.

## Explicitly incomplete in YD-3

- full remote transport flow
- full enroll/attach wire execution
- complete source scan/spool/retry behavior
- complete source-plane graph materialization
- complete CLI/SDK source-plane UX surfaces

## References

- `docs/architecture/distributed-acquisition-plane-model.md`
- `docs/architecture/source-owner-ingest-model.md`
- `docs/architecture/source-plane-read-model.md`
- `docs/architecture/exec-source-plane-role.md`
- `docs/program/22-adr/ADR-013-distributed-acquisition-centralized-control.md`
