---
id: MP-COMMAND-COVERAGE-0.2.1
status: in_progress
runbook: N/A
phase: "0.2.1 — root+kernel+boot status wave"
owners:
  - runtime
adrs:
  - docs/program/22-adr/ADR-006-unified-rpc.md
  - docs/program/22-adr/ADR-007-workspace-isolation.md
  - docs/program/22-adr/ADR-011-contract-baseline-lock.md
  - docs/program/22-adr/ADR-012-audit-convergence-gates.md
spec_anchors:
  - deps/yai-law/registry/commands.v1.json
  - docs/program/23-runbooks/workspaces-lifecycle-command-map.v2.md
target_group: root+kernel+boot
target_command_count: 15
---

# MP-COMMAND-COVERAGE-0.2.1

## Rescope Notice
This MP was moved from `workspaces-lifecycle` to `command-coverage` because its command surface is not strictly workspace-lifecycle scoped.


## Objective
Implement real runtime handlers for first-priority status surfaces (`root`, `kernel`, `boot`) with deterministic `yai.exec.reply.v1`.

Group mission: control-plane observability/readiness before wider command expansion.

Execution output contract for this MP:
- Runtime reply envelope: `yai.exec.reply.v1` (strict).
- CLI canonical short line (all outcomes, including `rc=0`):
  - `yai: <status>: <code>: <reason>`
- CLI verbose contract line (`--verbose-contract`):
  - `yai: <status>: <code>: <reason> (command_id=<id>, trace_id=<tid>, target_plane=<plane>)`
- SDK rc mapping remains deterministic from `status/code`.

## Scope (Wave target)
- Canonical target groups: `root`, `kernel`, `boot`
- Canonical command count: `15`
- Delivery model: implement real handlers for target IDs; keep non-target IDs deterministic (`nyi`).

## Implemented-real target command_id
- `yai.root.handshake_status`
- `yai.root.router_status`
- `yai.root.session_status`
- `yai.root.authority_status`
- `yai.root.envelope_validate`
- `yai.kernel.ws_status`
- `yai.kernel.ws_list`
- `yai.kernel.session_status`
- `yai.kernel.boundary_status`
- `yai.kernel.policy_status`
- `yai.boot.status`
- `yai.boot.runtime_status`
- `yai.boot.health_status`
- `yai.boot.service_status`
- `yai.boot.node_status`

## Definition of Done
- [x] Target IDs remain discoverable in CLI help.
- [ ] No `unknown command` for registered IDs in this group.
- [x] Selected real handlers are wired end-to-end (CLI -> SDK -> Root -> Kernel).
- [x] Non-target commands remain deterministic (`NOT_IMPLEMENTED` with exec.reply.v1).
- [ ] No empty output for successful target command execution (`rc=0` with canonical line).
- [ ] Evidence pointers/logs for executed commands are archived from operator run.

## Required evidence (to fill at execution time)
- [x] Build log for touched repo (`yai`, kernel compile green).
- [ ] Command execution matrix for these 15 IDs.
- [ ] Runtime logs with deterministic outcomes on operator environment.
- [ ] Audit mapping updates (claim-by-claim if Gate A scope is impacted).

## Risks and follow-ups
- Risk: local sandbox runtime bootstrap may fail (shared-memory environment), blocking in-sandbox runtime proof.
- Mitigation: run operator verification on host terminal and archive logs.
- Follow-up: continue with MP 0.2.2 for non-status operational verbs in same groups.

## Closure decision
Status: `IN_PROGRESS` (runtime handlers landed; operator evidence collection pending).
