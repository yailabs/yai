# YAI documentation authority

This page is the canonical navigation and authority map. A document outside
this map may provide evidence, history, research, or local source navigation;
it cannot silently change YAI's constitution, current architecture, or roadmap.

## Reading order

1. [Constitution](constitution.md) — what must remain true independent of the
   current implementation.
2. [Architecture](architecture.md) — what the current executable repository
   actually implements.
3. [Roadmap](../ROADMAP.md) — sole living public project control: snapshot,
   maturity, programs, selected boundary, dependencies and promotion.
   Its [semantic-state target](semantic-state-execution-target.md) separates adopted
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
| Roadmap | [ROADMAP.md](../ROADMAP.md) | live macro state, maturity, programs, dependencies, selected execution boundary and promotion | completed behavior before evidence exists; competing status/maturity registries |
| Architectural target | [semantic-state-execution-target.md](semantic-state-execution-target.md) | semantic/computational-state doctrine and falsifiers, subordinate to Constitution and Roadmap | implemented State Fabric/compiler/paging, frozen schemas or new owners; a second execution queue |
| Operations | [quickstart.md](quickstart.md), [test-cases.md](test-cases.md) | executable procedures and validation labels | semantic doctrine |
| Audit/evidence | [tests](../tests/README.md), [labs](../labs/README.md), immutable Git references for historical reports | executable proof, retained observations and historical qualification | current architectural authority, a report per wave, or target completion without execution evidence |
| Research | [research index](research-index.md), [research bridge](research-lab-bridge.md), [operational-state mathematics](operational-state-mathematics.md) | non-canonical hypotheses, provenance, and promoted design input | runtime truth without implementation evidence |
| Development instructions | [CONTRIBUTING](../CONTRIBUTING.md), [AGENTS](../AGENTS.md) | contribution and automation rules | product/runtime semantics |

The [legal posture](legal.md), license, security policy, and notices remain
authoritative for their own non-architectural subjects.

## Canonical semantic owners

- [Source-grounded domain knowledge](source-grounded-knowledge.md) defines the
  bounded derived documentary contract over admitted source revisions; it owns
  no canonical truth or second source registry.

- [Semantics](reference/semantics.md) owns names, definitions, aliases,
  dispositions, and rejected meanings.
- [State and transitions](reference/state-transitions.md) owns canonical state,
  transition phases, evidence roles, and external-effect recovery.
- [Context](reference/context.md) describes qualified S/W compilation and derived
  semantic delta, context-compatible Projection, Residency, ContextFrame,
  ContextDelta posture, rendering, tokenization, and continuity distinctions.
- [Boundaries](reference/boundaries.md) owns the YAI↔provider/YVEX and
  YAI↔external-resource contracts.

No source directory is implied by this document split. A source owner still
requires an independent lifecycle, canonical resource or transition, execution
boundary, and stable multi-consumer contract.

## Current status

[Architecture](architecture.md) records current executable contracts including
Golden and guided Case interaction. [Roadmap](../ROADMAP.md) owns the replace-in-place
snapshot and selected future boundary; this navigation page does not duplicate
its maturity or qualification verdicts. I01–I06 remain completed evidence, not
an obligation to invent I07–I10. Native REPLAI, R5 removal, later qualified repin
and TEST.TOPOLOGY.0 are preserved. A concept in the
[target doctrine](semantic-state-execution-target.md) is not a runtime component.
The [earlier alignment evidence](https://github.com/yailabs/yai/blob/c187648e9d9909d4d9b6711f131fa726d48cd585/refoundation/validation/post-i10-semantic-state-alignment/REPORT.md)
remains an exact historical record, not a current baseline instruction.

## De-authorized material

`work/spines/`, `work/waves/`, `work/archive/`, module-local historical
READMEs, old architecture mirrors, and lab narratives no longer own current
YAI meaning or status. They remain useful when a decision needs provenance or a
behavior needs characterization. Git history is the default owner of obsolete
chronology.

The former root `refoundation/` dossier tree is retained in Git history, not in
the current layout. Evidence links pin the commit containing the original
reports and captures; removal of duplicate chronology does not change their
qualification verdicts. New work updates the existing contracts, tests, Roadmap
and cumulative operator procedure. It does not create a parallel dossier for
each milestone. Independently needed observations, including unpublished
external qualification work, remain scoped lab evidence rather than project
control.

The historical `yai-dev` repository is a semantic mine, not a migration tree.
Valid historical properties are already represented in the canon or Roadmap;
reading `yai-dev` is not required to discover current YAI.

## Operations and research

- [Quickstart](quickstart.md)
- [Validation guide](test-cases.md)
- [ZERO-TO-CURRENT Golden operator acceptance](zero-to-current.md)
- [Research Lab bridge](research-lab-bridge.md)
- [Research index](research-index.md)
- [Operational State Mathematics](operational-state-mathematics.md)

Research links are retained from the pre-existing worktree. They are design
inputs only unless an explicit constitutional/reference decision adopts them.
