# supervisor

`sys/supervisor/` is the L2 control root for system admission, lifecycle and recovery.

## Canonical service

- `yai-supervisord/`

## Responsibilities

- system admission mediation
- system registry coordination
- lifecycle supervision across L2 services
- recovery coordination and escalation handoff

## Boundaries

- consumes kernel admission and registry primitives, does not re-implement them
- does not own domain engines (graph, data, policy, governance)
- does not replace `containerd` or `orchestratord`
