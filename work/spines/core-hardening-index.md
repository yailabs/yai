# Core Hardening Index

CORE.SPINE.C1 converts the engineering audit into executable future waves. These
are not implemented by CORE.SPINE.C1; they are the next implementation waves
that strengthen the properties in `work/spines/core-properties.md` against the
gaps in `work/spines/core-enforcement-status.md`.

Each wave below is sized to the SPINE implementation rule: small enough to
implement, validate and manually inspect in one delivery. None of these waves is
scheduled to run inside CORE.SPINE.C1.

## CORE.ENFORCE.1 — Control Lease Dispatch Verification (delivered, partial)

- Status: delivered (partial). Audit finding: the C carrier path was gated by the
  control decision only; the CapabilityLease was derived (SPINE.51B) but not
  consumed before execution, and carriers were reachable only from tests (no
  runtime dispatch path yet).
- Delivered: `yai_capability_lease_permits_execution` (fail-closed lease
  predicate) and `yai_dispatch_admit` (deterministic admission gate requiring a
  permitting lease AND an allow decision), proven by
  tests/smoke/control-lease-dispatch/ and guarded by
  tools/checks/check-control-lease-dispatch.sh.
- Remaining increment: rewire executable carrier signatures to require the
  admission token (today they re-check the decision as defense in depth, but do
  not yet receive the lease/admission directly).
- Dependency/order notes: admission gate is in place; the carrier-signature
  rewiring should ride with CORE.CARRIER.1.

## CORE.CARRIER.1 — Process Carrier Hardening

- Rationale: the process carrier is implemented_limited (test-owned signal
  control, arbitrary PID blocked). Move it toward a clearer
  interposed/controlled execution contract (CP2, CP7).
- Scope: tighten the process carrier execution contract and host observation;
  represent divergence explicitly.
- Non-goals: no network/database/git carriers; no daemon op execution over IPC.
- Files likely touched: system/effect/carriers/process_carrier.c, system/effect/process_state.c, system/effect/process_signal.c, system/reconcile/.
- Validation targets: controlled execution + receipt + divergence candidate on expected!=observed.
- Dependency/order notes: after CORE.ENFORCE.1.

## CORE.CARRIER.2 — Network HTTP Carrier Skeleton-to-Receipt

- Rationale: introduce the first non-filesystem/non-process interposed carrier
  path with deterministic decision/receipt and no fake execution (CP2, CP4).
- Scope: promote the network_http skeleton into a minimal interposed carrier
  with a real receipt; preserve carrier_attempted/execution_performed honesty.
- Non-goals: no database/git carriers; no NET transport runtime; no provider
  execution.
- Files likely touched: system/effect/carrier_skeleton.c → new network carrier, system/effect/receipt.c, proto/ fixtures.
- Validation targets: a controlled HTTP effect emits decision + carrier_outcome + receipt; blocked paths emit no fake receipt.
- Dependency/order notes: after CORE.CARRIER.1.

## CORE.POLICY.1 — Minimal Deterministic Policy Engine

- Rationale: policy_rule and obligation exist as fixture-like primitives with no
  evaluator. Promote them to a deterministic, inspectable evaluator (CP3, CP5).
- Scope: a deterministic policy evaluator over existing primitives; no LLM
  classification.
- Non-goals: no policy authoring UI; no remote policy; no model-driven policy.
- Files likely touched: system/control/policy_rule.c, system/control/obligation.c, new evaluator, tests/smoke/.
- Validation targets: same inputs → same decision basis; policy stays separate from derived planes.
- Dependency/order notes: independent of carrier waves; pairs with CORE.ENFORCE.1.

## CORE.NORMALIZE.1 — Proposed Operation Admission Contract

- Rationale: define and test the boundary from model/provider output or observed
  content to a typed proposed operation (CP1). Natural language is not an
  operation; tool output is not authority; ambiguity fails closed.
- Scope: typed proposed-operation admission with a fail-closed default.
- Non-goals: no model runner; no provider attachment; no automatic gate import.
- Files likely touched: system/op/, system/control/decision.c, tests/smoke/.
- Validation targets: free text and ambiguous tool output never admit as an operation; only typed, unambiguous proposals admit (still as proposals).
- Dependency/order notes: strengthens CP1 ahead of any model carrier work.

## CORE.JOURNAL.1 — Tamper-Evident Journal Chain

- Rationale: the journal is append-only but not tamper-evident. Add
  hash-chain/checkpoint posture so append-only becomes verifiable (CP6).
- Scope: hash-chain or checkpoint over journal entries/segments.
- Non-goals: no encryption; no remote attestation; no record-plane redesign.
- Files likely touched: system/store/journal.c, system/store/journal_file.c, engine/yai-engine/src/.
- Validation targets: a mutated journal segment is detectable; replay still idempotent.
- Dependency/order notes: independent; complements CORE.DATA.1.

## CORE.DATA.1 — C Shim / Rust Data-Plane Parity

- Rationale: data planes live partly in transitional C shims and partly in Rust
  yai-engine. Define parity tests and migration order (CP6).
- Scope: parity tests for store/graph/index/memory/projection/reconcile shims vs
  engine; document migration order.
- Non-goals: no full rewrite; no behavior change in this wave.
- Files likely touched: system/{store,graph,index,memory,projection,reconcile}, engine/yai-engine/src/, tests/.
- Validation targets: C and Rust paths produce identical records/facts for the same input.
- Dependency/order notes: after CORE.JOURNAL.1 for journal parity.

## CORE.LAB.1 — Adversarial Lab Automation

- Rationale: the filesystem-loop lab encodes adversarial scenarios (Lab A-K) as
  a runbook. Convert them into automated or semi-automated CI evidence.
- Scope: turn lab scenarios into repeatable checks/smoke wired into the guard
  suite.
- Non-goals: no new runtime behavior; no new carriers.
- Files likely touched: labs/filesystem-loop/, tools/checks/, tests/smoke/, Makefile.
- Validation targets: lab adversarial cases run in CI and fail loudly on regression.
- Dependency/order notes: after CORE.CARRIER.1/2 so hardened carriers are covered.

## CORE.MODEL.1 — model_provider Carrier Replan

- Rationale: model_provider is skeleton and scheduled late (SPINE.93+). Decide
  whether it must be pulled forward because it closes the agentic loop that
  buyers/users understand — without violating CP1/CP4.
- Scope: a replan decision and, if pulled forward, a skeleton-to-controlled plan
  that keeps model output as proposal and forbids fake execution.
- Non-goals: no model training; no runner implementation; no claim of execution
  until a carrier executes.
- Files likely touched: work/spines/yai-spine.md (roadmap), system/effect/carrier_skeleton.c, work/waves/.
- Validation targets: model_provider status stays skeleton/planned in
  core-enforcement-status.md until code executes; replan is recorded as a wave.
- Dependency/order notes: depends on CORE.NORMALIZE.1 (typed proposal admission)
  and CORE.ENFORCE.1 (lease before dispatch).
