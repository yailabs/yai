# Source Owner Ingest Model (v1)

Status: active
Owner: runtime
Effective date: 2026-03-10

## Purpose

Define the YD-4 baseline owner-side ingest flow for source plane operations.

This model turns source-plane contracts into executable owner runtime handling
while preserving centralized control and owner-side canonical persistence.

## Canonical flow

1. `yai-daemon` emits a control-call payload with source operation intent.
2. Owner runtime receives request through control ingress or peer ingress.
3. Session dispatch resolves command intent (`yai.source.enroll|attach|emit|status`).
4. Dispatch hands operation to `exec` source-ingest bridge.
5. `exec` validates shape and routes to operation-specific handler.
6. Owner runtime persists canonical source-plane records.
7. Owner runtime returns ack/reject reply.

## Exec mediation role

The ingest bridge is implemented in `exec` runtime slice and is not a direct
ad-hoc persistence shortcut.

Runtime anchors:

- `include/yai/exec/source_ingest.h`
- `lib/exec/runtime/source_ingest.c`
- `lib/core/session/session.c` (control-call routing into exec bridge)

This is the YD-4 verticalization baseline for transport mediation and owner
handoff.

## Supported operations (YD-4 baseline)

- `yai.source.enroll`
- `yai.source.attach`
- `yai.source.emit`
- `yai.source.status`

Each operation has request/reply contract intent anchors:

- `yai.source.enroll.call.v1` / `yai.source.enroll.reply.v1`
- `yai.source.attach.call.v1` / `yai.source.attach.reply.v1`
- `yai.source.emit.call.v1` / `yai.source.emit.reply.v1`
- `yai.source.status.call.v1` / `yai.source.status.reply.v1`

## Persistence behavior

The owner persists source-plane records through canonical data-plane append
flow; source daemons do not write canonical workspace truth directly.

Record classes written in YD-4 baseline:

- `source_node`
- `source_daemon_instance`
- `source_binding`
- `source_asset`
- `source_acquisition_event`
- `source_evidence_candidate`
- `source_owner_link`
- `source_enrollment_grant`

## Validation baseline

- workspace target must be valid and present on owner runtime
- operation intent must be known
- required source identifiers must be present per operation
- `attach`, `emit`, and `status` require owner-issued trust artifact token
- emit payload must include at least one supported source record item

## Reply baseline

- `ok/OK` with operation-specific ack reason and data payload
- `error/BAD_ARGS` for structural payload errors
- `error/INVALID_STATE` for unsupported operation state
- `error/INTERNAL_ERROR` for persistence/runtime failures

## Explicit non-goals in YD-4

- secure-native networking (mTLS/WireGuard)
- federation/multi-owner semantics
- edge authority/enforcement finality
- full replay/retry and transport reliability closure
- full source-plane graph materialization

## References

- `docs/architecture/source-plane-model.md`
- `docs/architecture/distributed-acquisition-plane-model.md`
- `docs/architecture/exec-source-plane-role.md`
