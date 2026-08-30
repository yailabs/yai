# Hardening 10 — Authority admission semantic closure

Baseline: `55db3a7ce06c101df36db1d6b78a6e2b44d63b4f`

Intended commit: `harden: close policy authority admission semantics`

Pre-publication state: implementation and focused/product/repository qualification
are green. Publication is intentionally recorded only in the post-commit response;
placing the resulting SHA in this commit would be self-referential.

## Direct archaeology verdict

Fresh inspection of `8daca560431912441beadc15f0c57ad17b094073`,
`a6b67ee398c75767f3869f2f18de996748137b6f` and
`681252f99ba651350f1be9081a78afb838cd0229` recovered one property that
Wave 10 had not yet closed mechanically: an evidence reference or content-valid
authority record is not verified canonical evidence. The legacy evidence model
named `unknown`, `unverified`, `verified` and `tampered`, but its executable
process producer emitted `unverified` and no promotion consumer was found.
Legacy mediation also separated policy, authority, review and projection in
shape, while defaulting ALLOW and passing authority through strings.

H10 recovers the distinction, not those owners. `admission.rs` remains the one
authority algorithm; LMDB supplies one transaction snapshot and rejects
semantically false writes. Supervisor, Governance/Authority planes,
`authority_context`, `operator_armed`, default ALLOW and the controlled-action
mega-envelope remain rejected.

## Closed write boundary

`validate_integrity()` continues to mean schema/content/identity integrity.
Canonical write admission now independently proves semantic validity:

- Decision v2 is re-derived from the canonical current Operation, Ready
  EffectivePolicy, resource envelope, Case participant roles and evidence
  resolved from Transition history. Exact semantic equality is required.
- ReviewRequest v2 is rebuilt from the committed REQUIRE_REVIEW Decision and
  exact basis. ReviewAction eligibility is checked at the low-level writer,
  against the open current request, exact generation and Case-bound roles.
- The final review Decision is re-derived from the canonical request/action and
  the same current policy basis; caller-provided action/reason refs cannot
  satisfy `audit_reason`.
- Grant v2 requires an immediately adjacent final ALLOW Decision. Its basis is
  re-derived against the pre-Decision canonical snapshot, and the expected
  Grant is reconstructed and compared before append.
- Any intervening Transition forces a new Decision. The runtime does that for
  the same Operation and canonical ReviewAction; it never asks the provider to
  propose again merely to refresh authority.
- Policy `pre_observation` and `post_observation` requirements are closed by the
  real PreparedEffect/Observation/Receipt chain. Platform observation safety
  remains unconditional.

Canonical source provenance requires exactly ordered and internally consistent
`ProviderInvocationStarted → ProviderResultRecorded → OperationRecorded`
Transitions in the same Case. Hash/ID knowledge, caller strings, memory, graph
or runtime checkpoint claims are not evidence.

## Generation invariant

For a Decision whose `decided_at_case_generation = G`, the Decision transition
commits at Case generation `G + 1`. Grant derivation requires current Case
generation exactly `G + 1`; the Grant transition then commits at `G + 2`, and
PREPARE at `G + 3`. No `>=` freshness is accepted.

The historical proof emitted: Operation generation 9, basis generation 9,
Decision transition 10, Grant expected generation 10, Grant transition 11,
PREPARE 12 and FINALIZE 13. Replacing P1 with P2 at 14 did not invalidate replay
of the historical P1 chain; future evaluation used P2.

## Ownership and schemas

No serialized schema changed: DecisionBasis v1, Decision v2, ReviewRequest v2,
ExecutionGrant v2 and Transition/CaseState v6 remain honest. No Rust semantic
owner, LMDB database/index, C owner, daemon or policy family was added.

Source footprint: `main.rs` 1,931 → 1,931 lines; tracked files 849 → 863;
C/H/Rust files 155 → 155; Rust files 31 → 31; semantic engine owners 16 → 16;
LMDB databases/indexes added: zero. The 14 new tracked files are the compact H10
evidence package plus its one executable characterization script.

Canonical: immutable PolicyArtifacts/lifecycle plus Operation, Decision and
DecisionBasis, ReviewRequest/Action, Grant, observations/receipt and effect
Transitions. Materialized current state: CaseState. Derived: EffectivePolicy,
NormativeReadiness, memory, graph, analytics, runtime metadata and current
indexes. Ephemeral: evaluator state produced only from canonical resolution.

`ResourceAttachmentState.policy_id`, `policy_owner_participant_id` and
`review_requirement` remain serialized/written by the v1 attachment command and
needed for v1 compatibility. They are inert for Decision v2 and reviewer
eligibility. Removal requires a later resource schema/compatibility migration,
not H10.

## Failures found and fixed

The first adversarial test proved a real bypass: a content-valid forged ALLOW
with the correct EffectivePolicy ID/digest could be committed after changing
the actual effective rule to DENY. The unchanged reproduction is now rejected
with `authority_decision_basis_mismatch`.

Exact Grant adjacency then exposed a product regression: provider attachment
during a human pause inserted a canonical Transition between final Decision and
Grant. H10 retained the strong invariant and changed the runtime to re-derive a
new Decision on the same Operation. The unchanged human-review smoke is green.
A v1 review compatibility regression in the new low-level action gate was also
fixed by applying v2 semantic admission only to v2 requests; v1 remains
readable/replayable and no normal governed writer emits it.

## Qualification and scope

Actual commands, exits and bounded raw output are in
[EXECUTION-EVIDENCE.md](EXECUTION-EVIDENCE.md). `make check`, escalated
`make characterization`, the H10 smoke, all 97 Rust tests, 26-turn agentless
proof, 128-iteration bounded endurance families, R1–R6, historical replay, formatting,
layout and diff checks are green. The first sandboxed characterization failure
(`failed to start ipc server: invalid`) is retained and classified as an IPC
sandbox restriction; the identical permitted rerun passed.

H10 adds no Wave-11 expiry, revoke, refresh, invalidation, cancellation,
closure, Grant TTL/revocation, Tenant/Principal or scheduler semantics.
