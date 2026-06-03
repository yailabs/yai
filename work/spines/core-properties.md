# Core Properties

CORE.SPINE.C1 records the durable properties the YAI core runtime must hold.
This is an engineering property registry, not a thesis. Each property has a
statement, the current enforcement status, evidence paths, a falsifier test idea,
and the next hardening wave that strengthens it.

Status values reuse the vocabulary in `work/spines/core-enforcement-status.md`.

## CP1 — Proposal is not authority

Statement:

```text
A model, provider or operator-visible text channel may propose, but may not
authorize, approve, decide, execute or mutate operational truth.
```

- Current enforcement status: implemented_limited
- Evidence paths: system/control/decision.c, system/control/gate.c, cmd/yai review surface (SPINE.44A-C), current-status.md ("model cannot approve", "model proposal is not execution")
- Falsifier test idea: feed a model-authored "approve" / "execute" string and assert it never reaches carrier dispatch; the only approve authority is the operator reviewer.
- Next hardening wave: CORE.NORMALIZE.1 (typed proposed-operation admission)

## CP2 — Complete mediated effect

Statement:

```text
Any executed effect must be bound to a case, a decision, a carrier outcome and
a receipt.
```

- Current enforcement status: implemented_limited
- Evidence paths: system/effect/carriers/filesystem_carrier.c, system/effect/dispatch.c, system/effect/receipt.c, system/effect/carrier_outcome.c
- Falsifier test idea: drive a filesystem write to execution and assert the residue contains all four of {case_ref, decision, carrier_outcome, receipt}; missing any → fail.
- Next hardening wave: CORE.CARRIER.1 (process), CORE.CARRIER.2 (network/http)

## CP3 — Capability soundness

Statement:

```text
Effective permission must not exceed the intersection of subject role, authority
scope, resource scope, visibility scope, policy basis and generation/snapshot
state.
```

- Current enforcement status: implemented_limited
- Evidence paths: system/control/capability_lease.c (permits_execution), system/effect/dispatch_admission.c (yai_dispatch_admit), tests/smoke/control-lease-dispatch/, system/control/authority_scope.c, system/effect/resource_scope.c, system/projection/visibility_scope.c (SPINE.51B: subject_lacks_execute_authority, resource_outside_scope)
- Falsifier test idea: request execution with a non-minted lease and an allow decision and assert admission denies (`lease_does_not_permit_execution`); covered by tests/smoke/control-lease-dispatch/.
- Next hardening wave: CORE.ENFORCE.1 delivered the fail-closed admission gate (lease+decision); rewiring carrier signatures to require the admission token is the next increment.

## CP4 — No fake execution

Statement:

```text
Non-execution paths must record non-execution and must not fabricate receipts,
effects or host observations.
```

- Current enforcement status: implemented
- Evidence paths: system/effect/carrier_skeleton.c, system/effect/carrier_outcome_harness.c, review loop (carrier_attempted: false, execution_performed: false)
- Falsifier test idea: exercise every skeleton carrier and a denied/deferred/quarantined review path; assert none emit a receipt or host observation and all report carrier_attempted: false.
- Next hardening wave: CORE.CARRIER.2 (preserve no-fake-execution as first non-fs carrier lands)

## CP5 — Derived planes are non-authoritative

Statement:

```text
Facts, projections, RuntimeGraph, memory/context frames and analytics surfaces
must not authorize, approve, deny, execute or mutate durable operational truth.
```

- Current enforcement status: implemented
- Evidence paths: SPINE.46-51 fact plane (facts_are_truth: false), system/projection/, system/graph/runtime_graph.c, system/memory/memory_candidate.c, yai-spine.md case runtime invariants
- Falsifier test idea: attempt to drive a decision or carrier dispatch from a fact row / projection / RuntimeGraph node alone and assert it is rejected; assert "facts are not truth" posture in extraction output.
- Next hardening wave: CORE.POLICY.1 (keep policy basis separate from derived planes)

## CP6 — Replay / idempotence

Statement:

```text
Replay must be deterministic and idempotent for the supported
journal/record/fact paths.
```

- Current enforcement status: implemented
- Evidence paths: engine/yai-engine/src/reconcile.rs, journal replay (SPINE.36-39), fact extraction idempotency (SPINE.47-49)
- Falsifier test idea: replay the same journal twice and assert records_written on the second run is 0 / all duplicate; assert deterministic fact IDs `fact:<kind>:<source_record_id>`.
- Next hardening wave: CORE.DATA.1 (C/Rust parity preserves idempotence)

## CP7 — Divergence visibility

Statement:

```text
Expected and observed state must remain comparable where a carrier supports host
observation; divergence must be representable and never hidden.
```

- Current enforcement status: implemented_limited
- Evidence paths: system/reconcile/divergence.c, system/reconcile/carrier_consistency.c, host observation probe (SPINE.33E), carrier receipt/divergence hardening (SPINE.33I)
- Falsifier test idea: force expected != observed for the process/filesystem carrier and assert a divergence candidate is produced rather than silently reconciled.
- Next hardening wave: CORE.CARRIER.1 (extend observation to hardened process carrier)
