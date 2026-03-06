---
id: MP-COMMAND-COVERAGE-0.2.2
status: draft
runbook: N/A
phase: "0.2.2 — root+kernel+boot operational wave"
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
target_command_count: 45
---

# MP-COMMAND-COVERAGE-0.2.2

## Rescope Notice
This MP was moved from `workspaces-lifecycle` to `command-coverage` because its command surface is not strictly workspace-lifecycle scoped.


## Objective
Expand real coverage in `root+kernel+boot` beyond status surfaces, preserving deterministic contract behavior.

Group mission: operational control verbs needed for bootstrap, router governance, and workspace observability.

Contract baseline inherited from MP 0.2.1:
- Runtime responses must be `yai.exec.reply.v1`.
- CLI default must emit canonical short line for success/nyi/error:
  - `yai: <status>: <code>: <reason>`
- `--verbose-contract` must emit:
  - `yai: <status>: <code>: <reason> (command_id=<id>, trace_id=<tid>, target_plane=<plane>)`

## Scope (Planned)
- Canonical target groups: `root`, `kernel`, `boot`
- Canonical command count (initial slice): `45` (15 per group)
- Delivery model: implement selected operational handlers first; keep non-target commands deterministic (`nyi`).

## Representative command_id set
- `yai.root.handshake_ping`
- `yai.root.router_trace`
- `yai.root.authority_validate`
- `yai.kernel.ws_trace`
- `yai.kernel.boundary_validate`
- `yai.kernel.policy_validate`
- `yai.boot.runtime_check`
- `yai.boot.health_check`
- `yai.boot.service_inspect`
- `yai.boot.node_report`

## Definition of Done
- [ ] Target IDs remain discoverable in CLI help.
- [ ] No `unknown command` for registered IDs in this group.
- [ ] Selected real handlers are wired end-to-end (CLI -> SDK -> Root -> Kernel).
- [ ] Non-target commands return deterministic error model (`ok/error/nyi` mapping).
- [ ] Evidence pointers/logs for executed commands are archived.

## Required evidence (to fill at execution time)
- [ ] Build/test logs for touched repos.
- [ ] Command execution matrix for this group.
- [ ] Runtime logs with deterministic outcomes.
- [ ] Audit mapping updates (claim-by-claim if Gate A scope is impacted).

## Risks and follow-ups
- Risk: operational verbs may require additional state stores not yet formalized.
- Mitigation: start with read-mostly verbs and deterministic synthetic metadata where needed.
- Follow-up: MP 0.2.3+ expands to governance and policy-heavy groups.

## Closure decision
Status: `PLANNED` (no runtime implementation claimed in this MP).
