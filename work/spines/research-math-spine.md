# Research Math Spine

## Purpose

This spine records mathematical research that has been promoted into bounded
YAI repo design invariants. It is an engineering spine, not a proof that YAI is
mathematically sufficient.

## Wave status

| Wave | Status | Result |
|---|---|---|
| RESEARCH.MATH.0 | completed | Formal variable binding from YAI evidence. |
| RESEARCH.MATH.1 | completed | Conditional theorem package for decision deficiency, information bounds, update stability, context loss and receipt deficiency. |
| PROMOTION.1 | completed | Bounded mathematical invariants and bidirectional source traceability. |
| PROGRAM.SPINE.0 | completed | Canonical research and implementation delivery spine. |
| ALPHA.CORE.0A | next | Operational trace algebra and contract mapping; deliveries AC0A-D1..D5. |
| ALPHA.CORE.0B | blocked | Invariant fixtures and conformance model; deliveries AC0B-D1..D5; blocked by AC0A-D5. |
| ALPHA.CORE.0C | blocked | Minimal contract and runtime enforcement; deliveries AC0C-D1..D5; blocked by AC0B-D5. |
| ALPHA.CORE.0D | blocked | Runtime conformance and evidence upgrade; deliveries AC0D-D1..D5; blocked by AC0C-D5. |

## Promoted design invariants

| ID | Invariant | Canonical doc | Header surfaces | Implementation claim |
|---|---|---|---|
| YAI-RI-I1 | Ordered history | `docs/operational-state-mathematics.md#yai-ri-i1-ordered-history` | `include/yai/store/record.h` | false |
| YAI-RI-I2 | State provenance | `docs/operational-state-mathematics.md#yai-ri-i2-state-provenance` | `include/yai/store/record.h` | false |
| YAI-RI-I4 | Gated side effects | `docs/operational-state-mathematics.md#yai-ri-i4-gated-side-effects` | `include/yai/control/gate.h`, `include/yai/control/decision.h` | false |
| YAI-RI-I5 | Receipt coupling | `docs/operational-state-mathematics.md#yai-ri-i5-receipt-coupling` | `include/yai/effect/receipt.h` | false |
| YAI-RI-I7 | Context omission disclosure | `docs/operational-state-mathematics.md#yai-ri-i7-context-omission-disclosure` | `include/yai/projection/projection.h` | false |

## Research-only boundaries

PROMOTION.1 does not promote these claims:

```text
decoder existence
contraction
bounded local update error
general receipt completeness
Context Compiler sufficiency
finite-case validation data
```

These remain blocked by the proof obligations recorded in the external
Research Lab.

## Traceability chain

The required chain is:

```text
public YAI header
-> docs/operational-state-mathematics.md#invariant-id
-> formula, assumptions, non-goals, repo evidence
-> Obsidian theorem/proof note
-> vault backlink to repo header path
```

The machine-readable map for agents is:

```text
work/research/research-link-map.v1.json
```

## Header rule

Header comments contain only:

```text
Research traceability
stable invariant ID
short invariant title
docs/operational-state-mathematics.md anchor
```

No long formulas belong in public headers.

## Source rule

C and Rust implementation files are not annotated in PROMOTION.1. Implementation
links should be added only when a concrete implementation realizes a specific
invariant, guard, fixture or test.

## Vault provenance

Vault spine:

```text
obsidian://open?vault=YAI%20Research%20Lab&file=00_Index%2FMathematical%20Research%20Spine
```

Delivery spine authority:

```text
obsidian://open?vault=YAI%20Research%20Lab&file=00_Index%2FYAI%20Research%20and%20Implementation%20Program
```

Theorem package:

```text
obsidian://open?vault=YAI%20Research%20Lab&file=40_Proofs%2FRESEARCH.MATH.1%20Formal%20Problem%20Statement
```

Proof obligations:

```text
obsidian://open?vault=YAI%20Research%20Lab&file=40_Proofs%2FProof%20Obligations%20and%20Promotion%20Candidates
```

## Frozen alpha sequence

The external YAI Research and Implementation Delivery Spine freezes the sequence:

```text
ALPHA.CORE.0A - Operational Trace Algebra and Contract Mapping - AC0A-D1..D5
ALPHA.CORE.0B - Invariant Fixtures and Conformance Model - AC0B-D1..D5
ALPHA.CORE.0C - Minimal Contract and Runtime Enforcement - AC0C-D1..D5
ALPHA.CORE.0D - Runtime Conformance and Evidence Upgrade - AC0D-D1..D5
```

This repo spine mirrors the promoted engineering sequence. Full wave contracts
remain in the external delivery spine authority.
