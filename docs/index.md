# YAI documentation authority

This page is the canonical navigation and authority map. A document outside
this map may provide evidence, history, research, or local source navigation;
it cannot silently change YAI's constitution, current architecture, or roadmap.

## Reading order

1. [Constitution](constitution.md) — what must remain true independent of the
   current implementation.
2. [Architecture](architecture.md) — what the current executable repository
   actually implements.
3. [Roadmap](../ROADMAP.md) — the delta and ordered source-refoundation work.
   Its [post-I10 target](semantic-state-execution-target.md) separates adopted
   direction from provisional mechanisms; it is not current implementation.
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
| Architectural target | [semantic-state-execution-target.md](semantic-state-execution-target.md) | post-I10 direction and falsifiers, subordinate to Constitution and Roadmap | implemented State Fabric/compiler/paging, frozen schemas or new owners |
| Operations | [quickstart.md](quickstart.md), [test-cases.md](test-cases.md) | executable procedures and validation labels | semantic doctrine |
| Audit/evidence | `refoundation/foundation-recovery/`, `refoundation/integration/`, `refoundation/validation/`, labs and tests | frozen findings, experiments, qualification and bounded before/after records | current architectural authority or target completion without execution evidence |
| Research | [research index](research-index.md), [research bridge](research-lab-bridge.md), [operational-state mathematics](operational-state-mathematics.md) | non-canonical hypotheses, provenance, and promoted design input | runtime truth without implementation evidence |
| Development instructions | [CONTRIBUTING](../CONTRIBUTING.md), [AGENTS](../AGENTS.md) | contribution and automation rules | product/runtime semantics |

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

Reconciled publication anchor: `426b5086cc668a3f04b68e39b80a36ffa6faa3ca`
([R5](../refoundation/integration/replai-r5/REPORT.md)), whose parent is the
semantic baseline `fec2f17b0e71e6e12e8f40a37c5f44d874f4b7d5` (I06).
REPLAI is the sole native editor; R5 removed obsolete vendored linenoise without
changing cognitive semantics or the qualified pin. TEST.TOPOLOGY.0 remains live.
[Architecture](architecture.md) describes the implemented W20/I01–I06 foundation,
not the old pre-refoundation snapshot. [Roadmap](../ROADMAP.md) distinguishes
**I01–I06 COMPLETE**, **I07 UNSELECTED**, and the post-I10 Semantic State /
Execution Compilation program **RECORDED** as a future target. I07–I10 are not
specified to preserve numbering. A mechanism in the target document is not a
runtime component.
The [alignment evidence](../refoundation/validation/post-i10-semantic-state-alignment/REPORT.md)
records reconciliation against I06 and R5 without rewriting their published
source or evidence.

## De-authorized material

`work/spines/`, `work/waves/`, `work/archive/`, module-local historical
READMEs, old architecture mirrors, and lab narratives no longer own current
YAI meaning or status. They remain useful when a decision needs provenance or a
behavior needs characterization. Git history is the default owner of obsolete
chronology.

The historical `yai-dev` repository is a semantic mine, not a migration tree.
Valid historical properties are already represented in the canon or Roadmap;
reading `yai-dev` is not required to discover current YAI.

## Operations and research

- [Quickstart](quickstart.md)
- [Validation guide](test-cases.md)
- [Research Lab bridge](research-lab-bridge.md)
- [Research index](research-index.md)
- [Operational State Mathematics](operational-state-mathematics.md)

Research links are retained from the pre-existing worktree. They are design
inputs only unless an explicit constitutional/reference decision adopts them.
