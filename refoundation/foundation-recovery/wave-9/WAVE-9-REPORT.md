# Wave 9 — Case policy binding and effective materialization

State: complete in this isolated Wave-9 commit. Baseline:
`a92673847808b46b6be82e8a96e90d45b1c12888`; final SHA is `this_commit` and
remote equality is recorded by the publication gate/final response.

## Direct archaeology and differential

Fresh inspection covered `8daca5604`, `681252f99`, `45c36bd0c`,
`575f76fcd`, `1d912484d` and `dda93ee3a`, across Case registry/runtime,
qualification, governance ingestion/loading/resolution, stated/legald views,
supervisor/control consumers and tests. The ledger was not used as authority.

The strongest executable legacy property was fail-closed runtime mediation when
normative refs/readiness were missing. The historical `case_policy_binding.c`
family was stubbed. Real runtime setup instead generated moving profile/path
aliases, persisted a free readiness boolean in latest registry JSON and later
used mutable generated DuckDB views. Later conflict/precedence code was a no-op
or injected defaults. Wave 9 recovers fail-closed inspectability and rejects
those owners and second truths. Exact evidence is indexed in
`direct-legacy-reinspection.tsv` and `legacy-case-policy-differential.tsv`.

## Implemented boundary

Canonical governance catalog content/history remains PolicySourceArtifact,
PolicyArtifact and lifecycle events. Canonical Case history now includes typed
`CasePolicyBound`, `CasePolicyReplaced` and `CasePolicyUnbound` payloads in
`yai.transition.v5`; `yai.case_state.v5` materializes compact active
`yai.case_policy_binding.v1` values. Readers retain v1-v4 compatibility.

Binding identity covers Case, owner-scoped lineage, exact artifact/version,
source/IR digests, bind-time Published event/sequence, resulting generation,
actor/reason and replacement ref. Bind and replace use one LMDB RW transaction
to validate current Case generation, exact immutable artifact integrity,
qualified/Published/runtime-consumable state and current lineage publication
before appending the Case Transition. One lineage has at most one active Case
binding. Replacement is one transition; no latest-version lookup mutates a
Case.

`yai.effective_policy.v1` is a derived/rebuildable value owned by the cohesive
`case_policy.rs` algorithm under `yai.policy_materializer.v1`. Sorted exact
inputs make binding/publication/cursor order irrelevant. For the current three
fact families: DENY dominates ALLOW, required review dominates false, evidence
obligations union, identical semantics merge provenance, and unsafe structural
collisions block. Every effective rule retains binding, artifact, IR rule,
fact, source and source-location refs.

Derived NormativeStatus is `unconfigured`, `ready` or `blocked`. Missing or
corrupt declared inputs block. Catalog drift is reported as current,
superseded, retired or no-current; it does not automatically upgrade, revoke or
invalidate a Case in Wave 9. The `effective_policy_by_case` LMDB DB is derived,
droppable and rebuildable. A cache failure after canonical bind leaves the
binding committed and later repair adds no Transition.

## Ownership verdict

Canonical:

- immutable PolicyArtifact content and governance lifecycle history;
- Case PolicyBinding Transition history.

Materialized canonical current state:

- CaseState active binding set.

Derived:

- EffectivePolicy and NormativeStatus;
- catalog drift/materialization diagnostics;
- `effective_policy_by_case` current cache and binding graph edges.

No Case policy action produces a Decision, ReviewRequest, ExecutionGrant,
PREPARE, carrier effect or provider/model invocation. Existing old Cases remain
unconfigured without changing their Wave 2-8 runtime/effect behavior.

## Proof results

- Product characterization: one real Case, two lineages, P@1 pinning after P@2
  publication, explicit replacement, pure status/rebuild, and Candidate
  rejection. Final scenario generation 9 with 9 canonical Transitions, two
  bindings, six input rules, three output rules and no authority objects.
- Additional product negatives: Validated, Superseded and Retired artifacts,
  stale generation and missing Case all exited 2.
- Replay/rebuild: exact binding set and EffectivePolicy semantic identity
  reproduced; no Transition or governance event appended.
- Derived failure: injected cache failure after canonical commit left one
  binding and absent cache; rebuild repaired it without duplicate Transition.
- Concurrency: two writers used the same expected Case generation; exactly one
  committed and the other received `stale_case_generation`.
- Multi-policy characterization: 24 independent artifacts, 72 input rules,
  three effective rules, 69 merged inputs, two conservative resolutions,
  zero blocking conflicts, 35,268 derived bytes; 4,242 ms in the retained
  qualification run. This is bounded characterization, not a benchmark claim.

The raw product and qualification evidence is in `EXECUTION-EVIDENCE.md`.

## Validation and discovered failures

`make check` initially exposed a real compatibility regression: a Wave-7 v4
LMDB schema metadata record was rejected by the new v5 store. The store now
accepts v4 metadata only as an explicit migration input and rewrites current
metadata to v5; the rerun passed. An early concurrent test opened the same LMDB
environment through overlapping process handles and hit `MDB_BAD_RSLOT`; the
fixture now closes the initial handle before thread-local opens, and the
product concurrency invariant passes.

`make characterization` first failed in the restricted sandbox with
`failed to start ipc server: invalid`. The same unchanged suite passed outside
the socket-restricting sandbox; this was test infrastructure, not a product
semantic failure. `make check` also caught removal of the characterized
`status: SPINE.51 Fact Plane Freeze` info line; that compatibility line was
restored before the green rerun.

Green final gates include `make check`, escalated `make characterization`, all
six required smoke families including the new Case-policy characterization,
88 Rust engine tests, the 26-turn agentless proof, 128-iteration endurance,
review/runtime/replay suites, format checks, docs/layout checks and
`git diff --check`. Clippy reports only the 14 engine and 17 CLI warnings that
predate/touch code outside the new Case-policy owner; Wave-9 `case_policy.rs`
and CLI module add none.

## Source footprint

- `main.rs`: 1,926 → 1,924 lines.
- tracked files after commit: 819 → 832.
- Rust files: 28 → 30.
- C/H files: 124 → 124.
- C/H/Rust source files: 152 → 154.
- new semantic owner: `engine/yai-engine/src/case_policy.rs`, justified by one
  coherent canonical binding integrity + derived materialization algorithm.
- new CLI family module: `cmd/yai/src/case_policy.rs`; parse/dispatch/render
  only.
- LMDB DB added: `effective_policy_by_case`, explicitly derived.

No C owner/header, daemon, registry, Agent, Workflow, governance plane or
provider-specific Case semantics was introduced.

## Recovery classification and remaining Wave-10 delta

Case PolicyBinding and EffectivePolicy materialization are promoted to
`refounded_proven`. Normative qualification is `partially_refounded` because
validity/stale/revoke semantics remain Wave 11. Current-family multi-artifact
conflict/missingness is refounded; broader scoped precedence remains partial.

The exact next delta is Wave 10: fresh archaeology followed by typed
Operation+EffectivePolicy applicability, DecisionBasis with causal
policy/rule/obligation refs, authority and review eligibility, obligation
satisfaction, and policy-bound operation-specific ExecutionGrant. Wave 10 must
not silently absorb Wave-11 expiry/revoke/cancellation work.
