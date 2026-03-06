---
id: MP-COMMAND-COVERAGE-0.2.5
status: draft
runbook: N/A
phase: "0.2.5 — substrate command wiring wave"
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
target_group: substrate
target_command_count: 200
---

# MP-COMMAND-COVERAGE-0.2.5

## Rescope Notice
This MP was moved from `workspaces-lifecycle` to `command-coverage` because its command surface is not strictly workspace-lifecycle scoped.


## Objective
Plan and execute real runtime wiring for group `substrate` without contract drift.

Group mission: Substrate storage/runtime primitives surfaces.

## Scope (Planned)
- Canonical target group: `substrate`
- Canonical command count: `200`
- Family distribution (top): `artifact_*` (21), `hash_*` (21), `key_*` (21), `ns_*` (21), `record_*` (21), `ref_*` (21), `schema_*` (21), `store_*` (21), `timeline_*` (6), `cap_*` (1)
- Delivery model: keep all registered commands invocable; implement selected handlers first; missing handlers remain deterministic (`nyi` equivalent).

## Representative command_id set
- `yai.substrate.artifact`
- `yai.substrate.artifact_attach`
- `yai.substrate.artifact_audit`
- `yai.substrate.artifact_compact`
- `yai.substrate.artifact_compose`
- `yai.substrate.artifact_declare`
- `yai.substrate.artifact_detach`
- `yai.substrate.artifact_export`
- `yai.substrate.artifact_gc`
- `yai.substrate.artifact_import`
- `yai.substrate.artifact_index`
- `yai.substrate.artifact_inspect`
- `yai.substrate.artifact_list`
- `yai.substrate.artifact_resolve`
- `yai.substrate.artifact_restore`

## Definition of Done
- [ ] Group `substrate` commands remain discoverable in CLI help.
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
