# Wave 10 — Policy-driven authority admission

Baseline: `c424718861925333f5881d5f7c5db42015558f94`

Intended commit: `feat: enforce case policy through typed authority admission`

Closure state: implementation, product evidence, full repository qualification,
format/docs/diff checks and pre-publication review are green. Repository-wide
`clippy -D warnings` still reports only warning sites already present at the
baseline; no reported site is introduced or modified by this Wave. Staged-diff
inspection and publication follow this report. The final commit SHA belongs in
the post-commit maintainer response because embedding it in its own commit is
self-referential.

## Direct recovery verdict

Fresh inspection of `8daca5604`, `a6b67ee39`, `681252f99`, adjacent
`1d912484d` history and the final legacy tree found one strongest useful path:
Case qualification fed mediation, mediation kept a nominal policy result,
authority result, review requirement, reason and evidence refs, and the runtime
could refuse unqualified context. It also found the reasons not to port that
architecture: default-ALLOW initialization, string profile/context matching,
ambient `operator_armed`/authority-lock state, mutable aliases, duplicated
outcome rewriting and a controlled-action envelope whose validation was ignored
by one real caller.

The refounded owner is one Rust `admission.rs`, not a plane. It consumes the
exact derived EffectivePolicy and emits immutable decision material. Case
continuity, policy authoring/materialization, review persistence and carrier
mechanics remain with their established owners.

## Implemented contract

- Governance input/artifact/IR evolved to v3/v3/v2 for one typed
  `authority_requirement` family. Proposer/reviewer required roles compose by
  deterministic set union/all-of.
- EffectivePolicy/materializer evolved to v2. Previous schemas remain readable
  with their original meaning.
- `yai.decision_basis.v1` binds one Operation to Case generation, resource
  envelope, exact EffectivePolicy/bindings/artifacts/rules, eligibility,
  obligations and final reason.
- `yai.decision.v2`, `yai.review_request.v2` and
  `yai.execution_grant.v2` integrity-bind that basis. Current durable Case
  contracts are `yai.transition.v6` and `yai.case_state.v6`; v1-v5 readers are
  retained.
- Live admission is closed-world: hard resource violation, ineligible proposer,
  applicable DENY, no explicit applicable ALLOW or impossible admission
  evidence yields DENY. There is no fallback to the resource-local v1 decider.
- Review exists only after the Operation is otherwise admissible. Eligibility
  comes from current Case roles and policy, not the legacy resource policy
  owner. Approval re-evaluates the same Operation and cannot survive a changed
  EffectivePolicy basis.
- `source_provenance` requires canonical ProviderInvocation/ProviderResult
  lineage. `audit_reason` requires a real ReviewAction reason. Policy pre/post
  observation requirements travel in the Grant; platform pre/post safety is
  unconditional.
- Grant commit transactionally re-materializes current policy and rejects a
  stale Decision basis. Binding replacement and Grant cannot race into stale
  authority.
- The runtime stops on unconfigured/blocked normative state before provider
  invocation. Historical Cases remain readable; live fixtures now bind
  explicit policy.

## Ownership

Canonical: immutable governance artifacts/events; Case Transitions including
Operation, Decision+DecisionBasis, ReviewRequest/Action, effective Decision,
ExecutionGrant and effect/receipt transitions.

Materialized current state: CaseState compact refs.

Derived: EffectivePolicy, NormativeReadiness, memory, graph, analytics,
context/residency, policy-current indexes and runtime metadata.

Ephemeral: EvidenceContext and evaluator temporary selector state.

Compatibility-only: Decision v1 source, Grant v1, ReviewRequest v1 and
`ResourceAttachmentState.policy_id`, `policy_owner_participant_id` and
`review_requirement`. New normal writers do not use those fields as authority.

Source footprint: `main.rs` 1,924 → 1,931 lines; tracked files 833 → 849;
C/H/Rust files 154 → 155; Rust files 30 → 31. Engine semantic modules
(excluding `lib.rs` and counting store as one owner) are 15 → 16. The only
new semantic owner is `admission.rs`; no LMDB database or index was added.

## Scope exclusions

No expiry, revoke, scheduled refresh, invalidation fanout, cancellation, Case
closure, Tenant/Principal authentication, scheduler, fencing, second carrier,
provider governance, daemon, C authority mirror, Workflow, Agent or Space was
added. Decision-level DEFER remains rejected: the distinct executable consumer
is durable `ReviewAction::Defer`.

## Evidence and remaining delta

Actual command transcripts are in [EXECUTION-EVIDENCE.md](EXECUTION-EVIDENCE.md).
The contract/differential/crash/negative matrices in this directory contain the
bounded re-diff. Wave 11 must freshly recover validity/expiry/refresh/revoke,
typed invalidation, historical explanation under retained old policy,
abandoned/expired Grant handling, cancellation and closure.
