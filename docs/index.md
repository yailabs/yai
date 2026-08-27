# YAI documentation authority

This page is the canonical navigation and authority map. A document outside
this map may provide evidence, history, research, or local source navigation;
it cannot silently change YAI's constitution, current architecture, or roadmap.

## Reading order

1. [Constitution](constitution.md) — what must remain true independent of the
   current implementation.
2. [Architecture](architecture.md) — what the frozen executable repository
   actually implements.
3. [Roadmap](../ROADMAP.md) — the delta and ordered source-refoundation work.
4. Reference contracts as needed:
   [semantics](reference/semantics.md),
   [state and transitions](reference/state-transitions.md),
   [context](reference/context.md), and
   [model/resource boundaries](reference/boundaries.md).

The root [README](../README.md) is the short public entry. It summarizes but
does not replace these owners.

## Authority classes

| Class | Current owner | Governs | Must not claim |
|---|---|---|---|
| Constitution | [constitution.md](constitution.md) | stable invariants and rejected owners | that a target invariant is implemented |
| Architecture | [architecture.md](architecture.md) | source-, test-, and reachability-backed current truth | target behavior as current capability |
| Reference | [reference/](reference/) | stable concepts and boundary contracts | project schedule or executable status |
| Roadmap | [ROADMAP.md](../ROADMAP.md) | gaps, sequencing, gates, and unresolved choices | completed behavior before evidence exists |
| Operations | [quickstart.md](quickstart.md), [test-cases.md](test-cases.md) | executable procedures and validation labels | semantic doctrine |
| Audit/evidence | `refoundation/audits/`, `labs/`, tests | frozen findings, experiments, and behavior proof | current architectural authority |
| Migration | `../refoundation/doc-refoundation/` from the YAI repository root, later migration records | a bounded before/after transition | permanent doctrine |
| Research | explicitly labeled research notes | non-canonical hypotheses, provenance, and promoted design input | runtime truth without implementation evidence |
| Development instructions | [CONTRIBUTING](../CONTRIBUTING.md), `.agents/AGENTS.md` | contribution and automation rules | product/runtime semantics |

The [legal posture](legal.md), license, security policy, and notices remain
authoritative for their own non-architectural subjects.

## Canonical semantic owners

- [Semantics](reference/semantics.md) owns names, definitions, aliases,
  dispositions, and rejected meanings.
- [State and transitions](reference/state-transitions.md) owns canonical state,
  transition phases, evidence roles, and external-effect recovery.
- [Context](reference/context.md) owns Projection, Residency, ContextFrame,
  ContextDelta posture, rendering, tokenization, and continuity distinctions.
- [Boundaries](reference/boundaries.md) owns the YAI↔provider/YVEX and
  YAI↔external-resource contracts.

No source directory is implied by this document split. A source owner still
requires an independent lifecycle, canonical resource or transition, execution
boundary, and stable multi-consumer contract.

## Current status

The Architecture baseline is YAI commit
`db183ae4c56bd16c7e6f31787ee4d90a51496d6d`. The worktree had pre-existing
research/header/spine/notebook changes when this documentation refoundation
began; those changes are not treated as frozen executable truth.

The architecture remains pre-refoundation. The Constitution is intentionally
stronger. The Roadmap is the only document permitted to bridge that gap.

## De-authorized material

`work/spines/`, `work/waves/`, `work/archive/`, module-local historical
READMEs, old architecture mirrors, and lab narratives no longer own current
YAI meaning or status. They remain useful when a decision needs provenance or a
behavior needs characterization. Git history is the default owner of obsolete
chronology.

The historical `yai-dev` repository is a semantic mine, not a migration tree.
Valid historical properties are already represented in the canon or Roadmap;
reading `yai-dev` is not required to discover current YAI.

## Operations

- [Quickstart](quickstart.md)
- [Validation guide](test-cases.md)
