# Container Domain Recovery Surface

`sys/container/recovery/` owns container-domain recovery paths at L2.

## Responsibilities

- detect recoverable vs unrecoverable container states
- prepare and execute domain recovery sequences
- support reopen/reconcile flows for degraded domains
- support sealing/inspection handoff when recovery is not safe

## Implemented surfaces

- `recovery.c`: recovery checks, preparation and execution coordination

## Boundary

- kernel keeps privileged lifecycle/admission/grants validity roots
- `sys/container/recovery` manages container-domain recovery behavior above those roots
