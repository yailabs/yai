---
id: MP-WORKSPACES-LIFECYCLE-0.2.1
status: draft
runbook: docs/program/23-runbooks/workspaces-lifecycle.md
phase: "0.2.1 — boot command wiring wave"
owners:
  - runtime
adrs:
  - docs/program/22-adr/ADR-006-unified-rpc.md
  - docs/program/22-adr/ADR-007-workspace-isolation.md
  - docs/program/22-adr/ADR-011-contract-baseline-lock.md
  - docs/program/22-adr/ADR-012-audit-convergence-gates.md
spec_anchors:
  - yai-law/registry/commands.v1.json
  - docs/program/23-runbooks/workspaces-lifecycle-command-map.v2.md
target_group: boot
target_command_count: 200
---

# MP-WORKSPACES-LIFECYCLE-0.2.1

## Objective
Plan and execute real runtime wiring for group `boot` without contract drift.

Group mission: Bootstrap and bring-up surfaces.

## Scope (Planned)
- Canonical target group: `boot`
- Canonical command count: `200`
- Family distribution (top): `artifact_*` (20), `cache_*` (20), `deps_*` (20), `health_*` (20), `node_*` (20), `probe_*` (20), `profile_*` (20), `runtime_*` (20), `service_*` (20), `overlay_*` (19)
- Delivery model: keep all registered commands invocable; implement selected handlers first; missing handlers remain deterministic (`nyi` equivalent).

## Representative command_id set
- `yai.boot.artifact_audit`
- `yai.boot.artifact_bootstrap`
- `yai.boot.artifact_check`
- `yai.boot.artifact_guard`
- `yai.boot.artifact_index`
- `yai.boot.artifact_inspect`
- `yai.boot.artifact_prepare`
- `yai.boot.artifact_reconcile`
- `yai.boot.artifact_refresh`
- `yai.boot.artifact_reload`
- `yai.boot.artifact_report`
- `yai.boot.artifact_resume`
- `yai.boot.artifact_seal`
- `yai.boot.artifact_snapshot`
- `yai.boot.artifact_status`

## Definition of Done
- [ ] Group `boot` commands remain discoverable in CLI help.
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
