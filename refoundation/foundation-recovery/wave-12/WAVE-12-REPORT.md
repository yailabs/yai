# YAI.SOURCE.REFOUNDATION.12

State at pre-publication closure: implemented and qualified. Publication is
reported only after the commit in the final response, in accordance with the
non-self-referential evidence rule.

- Baseline: `0e9cccd6391ba624e7cbae9312cdfa74d74bb1df`
- Intended commit: `feat: add authenticated tenant security domains`
- Branch: existing `master`; no branch/worktree/stash/reset was used
- Historical tracked-diff checksum preserved:
  `3fdb219654405e6fd40b5c0d1b02b94c04fadef5aa57a139aa5fb8fd6db7777e`

## Direct legacy verdict

Fresh direct inspection covered `1238741f7493e24138b00951f0d3547ec165306b`,
`1efadcfe94bba0d8cedf436716d93248ee81596e` and
`c88e0193909f8bdad68b2841a81a9ba815e7a55c`, plus final-tree auth-context,
runtime-session, user-session and principal families. The strongest executable
post-auth path was the substrate session opened after canonical login: it
carried `authn_kind`, `authn_verified`, Principal and active Case identity. The
root/user epoch separately observed host UID/GID and prevented user/root Case
contamination. Those are recovered as explicit authentication, explicit
context propagation and Case-scoped visibility—not as the old session tree.

Rejected topology/mechanisms: password transport/account store, environment
identity (`USER`, `YAI_USER`, `HOME`), self-verified local-dev contexts,
caller-selected `OPERATOR`, arming booleans, ambient root authority, root/user
Case hierarchy as Tenant, canonical UserSession mega-state, and an auth/IAM
module forest. No executable independent Tenant or Organization lifecycle was
found. Tenant is earned by the current isolation owner test; Organization did
not earn an owner and remains projection-only metadata.

## Refounded security contracts

The one new semantic owner is `engine/yai-engine/src/security.rs`:

- `yai.security_principal.v1` is an immutable, integrity-bound enrollment of
  `local_posix_effective_credential` and `posix:euid:<uid>`. Kernel `geteuid`
  selects authority identity; real UID and real/effective GIDs are retained as
  provenance. Environment strings are never inputs. eUID 0 has no YAI Tenant
  authority without enrollment and membership.
- `AuthenticatedPrincipal` is invocation-scoped and has private fields. Its
  production constructor observes POSIX credentials; synthetic construction is
  test-only. The durable binding index is one-to-one and produces a stable
  Principal ID across reopen.
- `yai.tenant.v1` binds immutable `tenant_id`, `organization_ref`, owner
  Principal, creation time and digest. Owner/Member is the entire Wave12
  membership algebra. Membership grants scoped visibility; Owner additionally
  grants narrow administration. Neither grants a Case role.
- `yai.security_event.v1` provides append-only LocalPrincipalRegistered,
  TenantCreated and TenantMemberAdded evidence. No suspension/removal/account
  lifecycle was invented.
- `SecurityContext` is ephemeral: one authenticated Principal, one selected
  Tenant and one resolved membership for an invocation. It is not a bearer
  session, Participant, policy authority or ExecutionGrant.

Principal remains distinct from Participant. `ParticipantPrincipalLinked` is a
canonical `yai.transition.v8` payload, materialized compactly by
`yai.case_state.v8`. It integrity-binds Tenant, Principal, Participant and
creating Principal; it creates no role. Cardinality is one active human-linked
Participant per Principal per Case and one Principal per linked Participant.
Provider/model Participants remain unlinked.

New ReviewAction history uses `yai.review_action.v2`: each product command
authenticates again and resolves Principal → Tenant membership → canonical
Participant link → current Case roles → policy reviewer eligibility. Tenant
Owner is not automatically a reviewer. `--as` can only agree with the resolved
Principal where accepted; it cannot choose human/admin authority.

## Tenant ownership and isolation

Every new Case is created through `TenantCaseOpened`, has exactly one immutable
Tenant and cannot move. v1-v7 histories read/replay as legacy-unscoped; they may
be inspected and may settle already-started physical truth, but cannot begin a
new governed live authority chain.

New governance ownership is explicit:

- Policy source bytes remain global content (`policy_source_artifact.v4`).
- `yai.policy_artifact.v5` binds exact `tenant_id` and organization projection.
- v5 lineage is `tenant_id + policy_key`; `owner_ref` is provenance only.
- `yai.policy_lifecycle_event.v3` binds Tenant and authenticated Principal.
- `yai.case_policy_binding.v2` binds Tenant and administrative Principal.
- `yai.effective_policy.v3` is single-Tenant and rejects mixed domains.
- `yai.decision_basis.v3` binds the Case Tenant and is re-derived at the H10
  canonical write boundary.

Artifact v1-v4 and binding v1 remain readable historical contracts. Matching
`owner_ref` text never adopts them into a Tenant. Identical source bytes in two
Tenants retain a shared source digest but produce distinct artifacts, lineage
and lifecycle. Cross-Tenant bind/revoke is rejected.

Store-level checks, not CLI prechecks, enforce administrative and object-domain
authority. Case, policy and review reads are membership-gated before details
are rendered. Memory, graph, projection, residency and context remain Case
derived and inherit `Case → Tenant`; they do not become security owners. Exact
or overlapping canonical filesystem roots across different Tenants are
rejected. This is alias rejection for the current local filesystem carrier,
not Wave14 fencing or a VM/container boundary.

Wave10/11 invariants remain additive and dominant: authentication cannot bypass
semantic Decision re-derivation, policy validity/revoke, Grant expiry,
cancellation/closure or PREPARE settlement truth. A cancelled/closed Case is not
reactivated by its Tenant Owner.

## Ownership and footprint

Canonical security truth consists of immutable Principal/Tenant records,
append-only security events and Tenant/Principal refs in canonical governance
and Case records. CaseState is materialized current state. SecurityContext and
AuthenticatedPrincipal are ephemeral. EffectivePolicy, readiness, memory,
graph, context and indexes remain derived.

Measured footprint after the complete Wave12 evidence package is staged:

- tracked files: 880 → 898
- C/H/Rust source files: 156 → 158
- Rust files: 32 → 34
- engine semantic owners: 16 → 17 (`security.rs` only)
- `cmd/yai/src/main.rs`: 1,949 → 1,985 lines; added lines are command
  declaration/dispatch only, not domain semantics
- LMDB databases/indexes: +5 (`security_principals_by_id`,
  `security_principal_by_binding`, `tenants_by_id`, `tenant_memberships`,
  `security_events_by_id`)
- Transition/CaseState: v7 → v8

The second new Rust file is CLI-only `cmd/yai/src/security.rs`. No C owner,
daemon, Instance owner, Organization registry, account/session directory,
scheduler, fencing engine or provider-governance owner was introduced.

## Qualification and observed failures

The retained raw evidence is in `EXECUTION-EVIDENCE.md`. Final gates completed:
`make check`, `make characterization`, the full named Wave12 smoke set, 107 Rust
tests, focused Wave12 (3), H10 (2+1), Wave11 temporal (6), R1-R6/A1-A6 coverage,
the governed 26-turn runtime and bounded 128-iteration endurance. `cargo fmt
--check`, docs/layout and `git diff --check` are closure gates. Clippy exits 0;
the existing repository warning classes remain and no warning was added by
touched/new Wave12 code.

Implementation exposed and fixed four useful integration failures: a sandbox
AF_UNIX restriction (infrastructure only); non-atomic test derivation producing
`authority_decision_time_mismatch`; legacy live fixtures rejected by the new
Tenant barrier; and graph fixtures correctly receiving `case_not_visible` until
explicitly bootstrapped. The unchanged relevant reproductions are green. No
identity/isolation bypass was hidden.

## Foundation Recovery classification

- Principal/local POSIX authentication: `refounded_proven`, within the stated
  local OS/LMDB trust model.
- Tenant/security domain and Tenant-owned governance catalog:
  `refounded_proven` for current local product owners.
- Case/read/human-review identity isolation: `refounded_proven` and
  authenticated-Principal-qualified.
- Organization: `partially_refounded`, projection-only.
- canonical UserSession and persistent Instance identity: rejected/deferred by
  owner test.
- external SSO/account auth, credentials, membership removal and resource
  fencing: deferred/missing, not claimed.

## Exact Wave13 delta

Wave13 remains only: multi-Case runtime instance lifecycle if independently
earned, scheduling/fairness, worker pool, quotas, bounded backpressure and
restart recovery sweep, while preserving Tenant gates under concurrent runners.
No part of that family was implemented. Resource fencing, second carrier and
provider governance remain later boundaries.
