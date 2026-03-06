---
id: MP-COMMAND-COVERAGE-0.2.3
status: draft
runbook: N/A
phase: "0.2.3 — mind command wiring wave"
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
target_group: mind
target_command_count: 200
---

# MP-COMMAND-COVERAGE-0.2.3

## Rescope Notice
This MP was moved from `workspaces-lifecycle` to `command-coverage` because its command surface is not strictly workspace-lifecycle scoped.


## Objective
Plan and execute real runtime wiring for group `mind` without contract drift.

Group mission: Mind-facing decision-support surfaces (SC103 optional).

## Scope (Planned)
- Canonical target group: `mind`
- Canonical command count: `200`
- Family distribution (top): `agent_*` (20), `intent_*` (20), `memory_*` (20), `model_*` (20), `planner_*` (20), `prompt_*` (20), `proposal_*` (20), `session_*` (20), `state_*` (20), `context_*` (19)
- Delivery model: keep all registered commands invocable; implement selected handlers first; missing handlers remain deterministic (`nyi` equivalent).

## Representative command_id set
- `yai.mind.agent_audit`
- `yai.mind.agent_checkpoint`
- `yai.mind.agent_classify`
- `yai.mind.agent_embed`
- `yai.mind.agent_explain`
- `yai.mind.agent_flush`
- `yai.mind.agent_inspect`
- `yai.mind.agent_ping`
- `yai.mind.agent_plan`
- `yai.mind.agent_propose`
- `yai.mind.agent_queue`
- `yai.mind.agent_refresh`
- `yai.mind.agent_restore`
- `yai.mind.agent_retrieve`
- `yai.mind.agent_route`

## Definition of Done
- [ ] Group `mind` commands remain discoverable in CLI help.
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
