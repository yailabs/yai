# orch

`target_fs/sys/orch/` is the orchestration service surface.

## Scope

This domain keeps only the orchestration service entry surface.

## What stays here

- `cmd/orchestratord/`: canonical orchestration entrypoint
- this README as domain surface contract

## Service-facing concerns

The service surface may expose coordination, scheduling and supervision
capabilities, but their runtime implementation does not live here.

## What does not stay here

Execution pipelines, workflow state, agent runtime, cognition glue,
transport glue and orchestration internals belong to `target_fs/krt/orch/`.
