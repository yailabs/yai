# YAI Research Index

## Purpose

This document is the repo-side research navigation hub. It helps YAI repo
readers trace promoted research context without making external Research Lab
material canonical.

PROMOTION.0 created the initial bidirectional link spine. PROMOTION.1 adds
bounded mathematical design invariants, a canonical repo math document, a repo
math spine, a machine-readable link map, and minimal public header traceability.

## Authority boundary

The YAI Research Lab is an external, non-canonical research surface. YAI repo
canon remains this repository's promoted docs, contracts, schemas, fixtures,
guards, tests, source code, and wave/spine records.

Research Lab links in this file are provenance and operator-local navigation
aids. They do not make vault notes repo canon and do not create implementation
claims.

## Current research spine

```text
OBSIDIAN.GOV.0 - Vault Research Program Spine
YAI.AUDIT.1 - Canonical Architecture Intake from YAI Repo
RESEARCH.MATH.0 - Formal Variable Binding from YAI Evidence
RESEARCH.BRIDGE.0 - Research Lab <-> YAI Repo Promotion Method
PROMOTION.0 - Research Report and Bidirectional Link Spine
RESEARCH.MATH.1 - Decision-Theoretic Approximate Operational Sufficiency and Stability Bounds
PROMOTION.1 - Bounded Mathematical Invariants and Bidirectional Source Traceability
```

Next expected wave:

```text
ALPHA.CORE.0 - Operational State Kernel Contract Design
```

## Promoted research links

Repo-side links:

- [Research Lab bridge](research-lab-bridge.md)
- [Operational State Mathematics](operational-state-mathematics.md)
- [Research math spine](../work/spines/research-math-spine.md)
- [Research link map](../work/research/research-link-map.v1.json)
- [YAI spine](../work/spines/yai-spine.md)

Operator-local Obsidian convenience links, not repo canon:

- Mathematical Research Spine: `obsidian://open?vault=YAI%20Research%20Lab&file=00_Index%2FMathematical%20Research%20Spine`
- Research Lab Roadmap: `obsidian://open?vault=YAI%20Research%20Lab&file=00_Index%2FResearch%20Program%20Roadmap`
- Formal Binding Index: `obsidian://open?vault=YAI%20Research%20Lab&file=15_Formal_Bindings%2FFormal%20Binding%20Index`
- Approximate Sufficiency Condition: `obsidian://open?vault=YAI%20Research%20Lab&file=15_Formal_Bindings%2FApproximate%20Sufficiency%20Condition`
- Binding Gap Matrix: `obsidian://open?vault=YAI%20Research%20Lab&file=15_Formal_Bindings%2FBinding%20Gap%20Matrix`

## Formal binding layer

The Research Lab currently contains formal bindings for:

```text
H_t Raw Case History
Z_t Operational State
U State Update Function
Rho Receipt Evidence
G Decision Gate Function
C_t Context Projection
Approximate Sufficiency Condition
Binding Gap Matrix
```

These bindings are research navigation and provenance for future promotion.
They are not implementation claims.

## Mathematical work status

RESEARCH.MATH.1 produced a conditional theorem package. PROMOTION.1 promotes
only bounded design invariants, not a proof that YAI is mathematically
sufficient and not a runtime verification claim.

Promoted design invariant IDs:

```text
YAI-RI-I1 - Ordered history
YAI-RI-I2 - State provenance
YAI-RI-I4 - Gated side effects
YAI-RI-I5 - Receipt coupling
YAI-RI-I7 - Context omission disclosure
```

Decoder existence, contraction, bounded local update error, general receipt
completeness and Context Compiler sufficiency remain research-only.

## Source/header linking policy

Source and header files should not link directly to speculative research notes.
Future source/header annotations should point first to promoted repo docs, which
may then point to Research Lab provenance.

PROMOTION.1 header comments point to:

```text
docs/operational-state-mathematics.md
```

## What is not claimed

This document does not claim that the Context Compiler is implemented.
This document does not claim that a Pack runtime is implemented.
This document does not claim that provider execution is implemented.
This document does not claim that approximate operational sufficiency is mathematically proven.
This document does not claim that promoted design invariants are fully verified by runtime behavior.
This document does not change runtime behavior.

## Next promotion target

ALPHA.CORE.0 should translate promoted invariants into contracts, fixtures,
guards, tests and invalidation criteria.
