# Spine Taxonomy

CORE.SPINE.C1 introduces this document as the canonical map of what a "spine"
means in the `yai` repository. It exists because three different things were
being called "spine": the core runtime spine, per-module/per-repo spines, and
the temporal `SPINE.NN` wave markers. They are not the same kind of object and
must not be merged.

This document is a taxonomy, not a roadmap. The active roadmap stays in
`work/spines/yai-spine.md`.

## Core Spine

The canonical YAI core spine is the runtime / control-plane spine. Its active
roadmap lives in:

```text
work/spines/yai-spine.md
```

The core spine owns:

```text
case-bound control
authority / proposal separation
gates and gate outcomes
decisions and decision basis
capability leases
effects and effect hashing
receipts and receipt guarantees
record truth (journal + LMDB record plane)
projections
graph / fact derivation
replay
review / operator authority
carrier mediation
```

There is exactly one core spine. `SPINE.NN` waves implement it incrementally;
they do not fork it.

## Module / Repo Spines

Module and repository spines are scoped to one subsystem or one repository. They
describe how that module participates, not how the core runtime decides. They
are not replacements for, nor alternatives to, the core spine.

| Spine | Surface | Authority relation to core |
|---|---|---|
| NET spine | `work/spines/net-spine.md`, root `net/` module | Moves streams. NET transports receipts; YAI decides authority. |
| CLORI spine | external `clori` repository | Executes neural computation as an external capability node. Not vendored into `yai`. |
| interfaces spine | `interfaces/docs/intf-studio-spine.md` | Owns `INTF.SPINE.*` and downstream `STUDIO.SPINE.*`. Removed from the `yai` roadmap. |
| console / client spine | `console` repository | Downstream consumer of projections and interfaces. |
| studio / app spine | governed by the interfaces repo `STUDIO.SPINE.*` | Downstream client surface. |

Canonical rule:

```text
A module/repo spine can participate in the core runtime.
It cannot become the core runtime.
```

## Temporal Wave Markers

`SPINE.NN` (and `NET.SPINE.NN`, `CORE.*`, `REPO.HYGIENE.N`) are temporal
implementation waves / checkpoints. They are not separate product spines and
not parallel calendars. A macro label such as `WORLD`, `CONTROL`, `MODEL` or
`DATA` describes the impact of a wave; it is not a numbered workstream.

```text
SPINE.NN = an implementable, manually verifiable delivery against the core spine
Macro label = affected system area inside one wave
```

Historical `SPINE.NN` waves are never renamed. CORE.SPINE.C1 does not rewrite
them; it only clarifies how to read them.

## Source-of-Truth Rule

```text
Public README files are orientation surfaces.
work/spines/source-surface.md and work/spines/command-surface.md are the
higher-trust implementation maps for the active yai checkout.
Code and tests beat docs when they conflict.
```

When this taxonomy and the code disagree, the code wins and this document is the
bug. The honest enforcement state of the core spine is recorded in
`work/spines/core-enforcement-status.md`; the durable properties it must hold are
recorded in `work/spines/core-properties.md`; the next executable hardening
waves are listed in `work/spines/core-hardening-index.md`.

## Unknown / Private Repo Rule

```text
yai-dev, studio, private apps and any repository not present in this checkout
must not be treated as verified by the yai repo. They are external/unknown
unless physically present here. A claim about them is a claim about an external
surface, not about yai.
```

The canonical `yai` checkout has no `apps/` directory. No document may describe
`apps/` as a present, canonical `yai` root unless it actually exists here.
