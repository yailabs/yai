# Operational State Mathematics

## Purpose

This document is the canonical repository target for public header research
traceability links created by PROMOTION.1.

It promotes bounded mathematical design invariants from the external YAI
Research Lab into repo documentation. It does not claim that YAI is
mathematically proved, that the invariants are fully verified by runtime
behavior, or that blocked proof obligations have been satisfied.

## Status

```text
source wave: RESEARCH.MATH.1
promotion wave: PROMOTION.1
status: promoted design invariants, not runtime proof
evidence level: E1 design/documentation with E4 header trace references
```

## Provenance

Vault authority:

```text
obsidian://open?vault=YAI%20Research%20Lab&file=00_Index%2FMathematical%20Research%20Spine
```

Theorem package:

```text
obsidian://open?vault=YAI%20Research%20Lab&file=40_Proofs%2FRESEARCH.MATH.1%20Formal%20Problem%20Statement
```

Proof obligations:

```text
obsidian://open?vault=YAI%20Research%20Lab&file=40_Proofs%2FProof%20Obligations%20and%20Promotion%20Candidates
```

Machine-readable map:

```text
work/research/research-link-map.v1.json
```

## Shared assumptions

The RESEARCH.MATH.1 theorem package works on a common probability space:

$$
(\Omega,\mathcal F,\mathbb P).
$$

The raw history is ordered:

$$
H_t =
\left(
X_0,
(Q_k,C_k,M_k,Y_k,D_k,A_k,O_{k+1},R_{k+1})_{k=0}^{t-1}
\right).
$$

The history filtration is:

$$
\mathcal H_t=\sigma(H_t).
$$

The operational state is a measurable function of history:

$$
Z_t=\phi_t(H_t),
\qquad
\sigma(Z_t)\subseteq\mathcal H_t.
$$

These formulas define research notation. They do not assert a complete runtime
implementation of a single `phi_t`.

## Invariants

### YAI-RI-I1 Ordered history

Stable ID: `YAI-RI-I1`

Design invariant:

```text
Operational history must preserve order and multiplicity strongly enough for
audit, replay, state derivation, and future task-relative reasoning.
```

Formula:

$$
H_t =
\left(
X_0,
(Q_k,C_k,M_k,Y_k,D_k,A_k,O_{k+1},R_{k+1})_{k=0}^{t-1}
\right),
\qquad
\mathcal H_t=\sigma(H_t).
$$

Repo evidence:

```text
include/yai/store/record.h
```

Vault source:

```text
obsidian://open?vault=YAI%20Research%20Lab&file=15_Formal_Bindings%2FH_t%20Raw%20Case%20History
```

Non-goal: this invariant does not claim every runtime path already preserves a
complete mathematical filtration.

### YAI-RI-I2 State provenance

Stable ID: `YAI-RI-I2`

Design invariant:

```text
Authoritative operational state must be derived from observed history and keep
provenance to source evidence.
```

Formula:

$$
Z_t=\phi_t(H_t),
\qquad
\sigma(Z_t)\subseteq\mathcal H_t.
$$

Repo evidence:

```text
include/yai/store/record.h
```

Vault source:

```text
obsidian://open?vault=YAI%20Research%20Lab&file=15_Formal_Bindings%2FZ_t%20Operational%20State
```

Non-goal: this invariant does not claim a complete implemented `phi_t` or exact
state sufficiency.

### YAI-RI-I4 Gated side effects

Stable ID: `YAI-RI-I4`

Design invariant:

```text
State-changing side effects require an explicit decision or gate boundary before
execution is treated as authorized.
```

Research notation:

$$
d_t=G(Z_t,y_t,A_t,\pi).
$$

Repo evidence:

```text
include/yai/control/gate.h
include/yai/control/decision.h
```

Vault source:

```text
obsidian://open?vault=YAI%20Research%20Lab&file=15_Formal_Bindings%2FG%20Decision%20Gate%20Function
```

Non-goal: this invariant does not claim all future side-effect families or
provider execution boundaries are implemented.

### YAI-RI-I5 Receipt coupling

Stable ID: `YAI-RI-I5`

Design invariant:

```text
State-changing effects require receipt evidence coupled to the attempted action,
decision, observation, and state update path.
```

Receipt deficiency target:

$$
\eta_t^\rho =
I\left(
F_{t+1};
E_{t+1}
\mid
Z_t,A_t,O_{t+1},R_{t+1}
\right).
$$

Repo evidence:

```text
include/yai/effect/receipt.h
```

Vault sources:

```text
obsidian://open?vault=YAI%20Research%20Lab&file=15_Formal_Bindings%2FRho%20Receipt%20Evidence
obsidian://open?vault=YAI%20Research%20Lab&file=40_Proofs%2FReceipt%20Completeness%20Counterexample
```

Non-goal: this invariant does not claim general receipt completeness. Complete
receipt sufficiency remains blocked by proof obligations.

### YAI-RI-I7 Context omission disclosure

Stable ID: `YAI-RI-I7`

Design invariant:

```text
Context projections must disclose inclusion and omission boundaries so consumers
can distinguish selected state from unavailable or omitted state.
```

Context projection:

$$
C_t=\psi_t(Z_t,Q_t,M_t).
$$

Information decomposition:

$$
I(F_t;H_t\mid C_t,Q_t,M_t)
=
I(F_t;H_t\mid Z_t,Q_t,M_t)
+
I(F_t;Z_t\mid C_t,Q_t,M_t).
$$

Repo evidence:

```text
include/yai/projection/projection.h
```

Vault source:

```text
obsidian://open?vault=YAI%20Research%20Lab&file=15_Formal_Bindings%2FC_t%20Context%20Projection
```

Non-goal: this invariant does not claim Context Compiler implementation or
Context Compiler sufficiency.

## Research-only blocked results

These are not promoted by PROMOTION.1:

| Blocked result | Blocking proof obligation |
|---|---|
| Decoder existence | PO-4 |
| Contraction | PO-6 |
| Bounded local update error | PO-7 |
| General receipt completeness | PO-8 |
| Context Compiler sufficiency | PO-9 |
| Finite-case validation data | PO-10 |

## Header rule

Headers contain only stable invariant IDs and this document's anchors. They do
not carry formulas. C implementation files should receive invariant links only
when a concrete implementation realizes a specific invariant.
