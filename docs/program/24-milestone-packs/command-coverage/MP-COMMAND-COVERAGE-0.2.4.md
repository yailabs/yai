---
id: MP-COMMAND-COVERAGE-0.2.4
status: draft
runbook: N/A
phase: "0.2.4 — orch command wiring wave"
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
target_group: orch
target_command_count: 200
---

# MP-COMMAND-COVERAGE-0.2.4

## Rescope Notice
This MP was moved from `workspaces-lifecycle` to `command-coverage` because its command surface is not strictly workspace-lifecycle scoped.


## Objective
Plan and execute real runtime wiring for group `orch` without contract drift.

Group mission: Orchestration plan/trial/report surfaces.

## Scope (Planned)
- Canonical target group: `orch`
- Canonical command count: `200`
- Family distribution (top): `job_*` (21), `pack_*` (21), `plan_*` (21), `report_*` (21), `runner_*` (21), `trial_*` (21), `dataset_*` (20), `queue_*` (20), `result_*` (20), `campaign_*` (13)
- Delivery model: keep all registered commands invocable; implement selected handlers first; missing handlers remain deterministic (`nyi` equivalent).

## Representative command_id set
- `yai.orch.campaign_audit`
- `yai.orch.campaign_cancel`
- `yai.orch.campaign_export`
- `yai.orch.campaign_import`
- `yai.orch.campaign_list`
- `yai.orch.campaign_plan`
- `yai.orch.campaign_queue`
- `yai.orch.campaign_report`
- `yai.orch.campaign_resume`
- `yai.orch.campaign_run`
- `yai.orch.campaign_status`
- `yai.orch.campaign_trace`
- `yai.orch.campaign_verify`
- `yai.orch.dataset_audit`
- `yai.orch.dataset_cancel`

## Definition of Done
- [ ] Group `orch` commands remain discoverable in CLI help.
- [ ] No `unknown command` for registered IDs in this group.
- [ ] Selected real handlers are wired end-to-end (CLI -> SDK -> Root -> Kernel/Engine).
- [ ] Non-implemented commands return deterministic error model (`ok/error/nyi` mapping).
- [ ] Evidence pointers/logs for executed commands are archived.

## Required evidence (to fill at execution time)
- [ ] Build/test logs for touched repos.
- [ ] Command execution matrix for this group.
- [ ] Runtime logs with deterministic outcomes.
- [ ] Audit mapping updates (claim-by-claim if Gate A scope is impacted).

## Risks and follow-ups
- Risk: wide family surface may hide semantic collisions.
- Mitigation: implement by family slices, keeping contract fixed.
- Follow-up: chain next MP after this group reaches stable deterministic behavior.

## Closure decision
Status: `PLANNED` (no runtime implementation claimed in this MP).
