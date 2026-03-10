---
id: ARCH-COMP-MIND
status: historical
effective_date: 2026-02-23
revision: 1
owner: mind
law_refs:
  - ../law/foundation/boundaries/L3-mind.md
  - ../law/foundation/invariants/I-004-cognitive-reconfiguration.md
---

# Mind Component

> Historical record: this page documents pre-unified-runtime topology and is
> not an active architecture source for current runtime ownership.

## Role

Cognitive proposer plane (L3) that can suggest plans/actions but cannot directly enforce external effects.

## Current Implementation Status

partial

## Interfaces and Entry Points

- `mind/src/main.c`
- `mind/include/mind_cognition.h`
- `mind/src/cognition/orchestration/planner.c`
- `mind/src/transport/protocol.c`

## Authority and Boundary Rules

- Mind is proposer-only and must not become an authority or enforcement surface.
- All effectful decisions remain governed by Root/Kernel/Engine contracts.
- Proposal context must preserve workspace and trace identifiers end-to-end.

## Companion Docs

- Overview: `docs/10-platform/architecture/components/mind-overview.md`
- Boundaries: `docs/10-platform/architecture/components/mind-boundaries.md`

## Traceability

- ADR refs: `docs/program/22-adr/ADR-005-mind-proposer.md`, `docs/program/22-adr/ADR-003-kernel-authority.md`
- Runbook refs: `docs/program/23-runbooks/mind-redis-stm.md`, `docs/program/23-runbooks/root-hardening.md`
- MP refs: `docs/program/24-milestone-packs/root-hardening/MP-ROOT-HARDENING-0.1.5.md`
- L0 anchors: `../law/foundation/boundaries/L3-mind.md`, `../law/foundation/invariants/I-004-cognitive-reconfiguration.md`

## Known Drift / Gaps

- End-to-end evidence for proposer path through Kernel enforcement is still partial.
- Redis STM integration and governance checks are not fully closed at milestone-pack level.

## Next Alignment Steps

- Close `RB-MIND-REDIS-STM` phase evidence with mandatory verify/suite/proof outputs.
- Keep architecture alignment and claims registry synchronized as mind milestones close.
