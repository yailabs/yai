# orchestration

`sys/orchestration/` is the canonical L2 orchestration plane.

## Canonical service

- `yai-orchestratord/`

## Active domains

- `planner/` plan decomposition and ordering
- `execution/` execution pipeline and action assembly
- `workflow/` workflow session state
- `coordination/` orchestration-level coordination surface
- `scheduling/` scheduling surface
- `supervision/` supervision surface
- `internal/` service glue (runtime/bridge/transport/model)
- `include/yai/orchestration/` canonical orchestration headers

## Boundaries

- orchestration consumes kernel and container primitives, it does not own them
- orchestration is not a container manager
- orchestration does not own kernel admission/containment/grants roots
- orchestration is not a user-facing SDK/CLI plane

## Cutover status

DR-1 hard cut applied: active orchestration code moved out of
`runtime/compatibility/lib/orchestration/` into this root.
