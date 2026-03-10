---
id: ARCH-COMP-MIND-OVERVIEW
status: historical
effective_date: 2026-02-23
revision: 1
owner: mind
law_refs:
  - ../law/foundation/boundaries/L3-mind.md
  - ../law/foundation/invariants/I-004-cognitive-reconfiguration.md
---

# Architecture Overview — Mind (L3)

> Historical record: this page documents pre-unified-runtime topology and is
> not an active architecture source for current runtime ownership.

## Role

Mind is the cognitive proposer plane (L3). It prepares context and proposals but does not execute irreversible external effects.

## Current Implementation Status

partial

## Interfaces and Entry Points

- `mind/src/main.c`
- `mind/include/mind_cognition.h`
- `mind/src/cognition/orchestration/planner.c`
- `mind/src/transport/protocol.c`

## Authority and Boundary Rules

- Mind must remain proposer-only; authority and enforcement remain in lower planes.
- Effectful decisions require governed handoff to Root/Kernel/Engine.
- Proposal context must preserve workspace and trace identifiers.

## Traceability

- ADR refs: `docs/program/22-adr/ADR-005-mind-proposer.md`, `docs/program/22-adr/ADR-003-kernel-authority.md`
- Runbook refs: `docs/program/23-runbooks/mind-redis-stm.md`, `docs/program/23-runbooks/root-hardening.md`
- MP refs: `docs/program/24-milestone-packs/root-hardening/MP-ROOT-HARDENING-0.1.5.md`
- L0 anchors: `../law/foundation/boundaries/L3-mind.md`, `../law/foundation/invariants/I-004-cognitive-reconfiguration.md`

## Known Drift / Gaps

- End-to-end evidence for proposer flow through Kernel enforcement is partial.
- Some governance closure items are still pending at milestone-pack level.

## Next Alignment Steps

- Close remaining proposer-path evidence in runbook/milestone-pack artifacts.
- Keep alignment outputs synchronized after each mind-plane change.
