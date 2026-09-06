# I05 — governed cognitive target arbitration

Requested anchor: `d3ca2cac5f6575241839c65083d14336e5c1fedc`.
Reconciled implementation baseline: `80d9077f24990711f9879b71c317fc2711d2d7fa`.
HEAD, origin/master and remote master agreed; initial worktree was clean.
The intervening native REPLAI consumer commit was inspected and preserved.
Intended commit: `feat: add governed cognitive target arbitration`.
Pre-publication posture: implementation and local publication qualification
complete; isolated commit/push/equality remain the publication step. The containing
commit's SHA and publication equality belong in the final handoff.

## Ownership and compatibility decision

Case already owns cognitive configuration. No parallel policy store, selector
registry, runtime, Agent or Orchestrator is justified. The existing binding
contract gains v2 ordered preference; v1 remains a single pinned target. Its
first `target_id`/digest/evidence is the first preference, followed by explicit
`target_policy.kind=ordered_eligible` alternatives. A policy contains 2–8
distinct exact targets; a single target remains pinned. Bind/replacement/unbind
reuse the same authenticated, atomic canonical Transition path. Replacement
remains explicit, idempotent identical configuration does not append history,
and old bindings remain historical. No independent policy lifecycle exists.

This representation deliberately retains the v1 first-target fields for
compatibility, but does not silently broaden v1 meaning: ordered policies carry
`yai.case_cognitive_binding.v2`. Human inspection labels their policy and
candidate order. Only the plan's `selected_target_id` means execution choice.

Transition v15 → v16 admits the versioned canonical policy; CaseState v13 → v14
materializes it in existing bounded slots. CognitiveExecutionPlan v1 → v2 binds
the arbitration input, candidate assessments and evidence snapshot. Old
Transition v1–v15 and CaseState v1–v13 readers remain; historical plan v1 JSON
remains readable, but execution requires a fresh current plan. No
ProviderQualification v4, Projection v7, ContextFrame v7, RetrievalSet v3,
ConversationTurn v1 or ConversationDerivedContent v1 semantic change.
Semantic owner delta 0; operational owner delta 0; LMDB delta 0 (37/40).

## Arbitration and exact execution

Preference order is explicit configuration, not ProviderSelection's health
ranking. The pure cognitive planner evaluates bounded candidates against one
LMDB snapshot. It explains missing/changed target, envelope mismatch, stale
binding evidence, absent capability suitability, missing/stale qualification,
unsupported wire shape, trust denial, unavailable credential, open circuit and
unavailable provider independently. No names, MIME-only semantic inference,
prices, model quality or YVEX physical facts participate.

Unknown/degraded health does not invent failure or alter preference; current
Unavailable/non-Closed circuit excludes. Effective operational facts are sealed
without hashing the advancing authority clock floor itself. Qualification and
trust event identities remain exact. Suitability posture remains explicit:
operator attestations are not an automatic semantic evaluator. The loopback
suite qualifies adapter shapes, not model intelligence or transcription quality.

Known source shape is input to planning. An explicit `--shape` on inspection
does not validate file contents; I03 still resolves canonical content, validates
the actual wire shape, and checks qualification. Native suitability and
mechanical shapes remain independent. No auxiliary stage is manufactured when
a primary candidate can consume the original closure.

New exact plans retain one target and lane. Their derived arbitration digest
does not create canonical authority. I03 recomputes the snapshot before
selection and at invocation admission; all candidate posture changes are
conservatively relevant, even if the selected target itself remains valid.
A new target changes the lane; qualification/trust changes require a new plan
but do not by themselves rewrite binding/lane identity. Pinned lanes retain
their historical identity algorithm. Computational continuations remain
optional and cannot cross lanes or targets.

The single-plan realization path and finite I04 composition both consume the
same shape-aware planner. Source closures, immutable Turns, derived content
and exact ProviderResult lineage remain unchanged. Completed compatible
auxiliary work remains reusable after restart. Selection admission also checks
prior uncertain delivery for the same semantic requirement across target and
binding changes. It never permits implicit cross-target retry. The exact
provider requirement is built once by a shared semantic helper, preventing a
caller from relabeling purpose to evade that check.

Historical I03 negative scenarios were reconciled, not weakened: a malformed
response is not retry-safe under the existing delivery taxonomy. Switching the
binding no longer permits replaying that same uncertain work elsewhere. The
I03 suite now explicitly asserts this refusal, then uses independent newly
submitted content for the subsequent normalization test. Every previous
normalization/isolation/recovery assertion remains. The earlier mechanical
shape-refusal diagnostic is preserved, with exact candidate exclusions added;
missing wire qualification is not relabeled as missing semantic suitability.

## Qualification closure

`make test-fast` passed without a provider. The final
`PATH="$PWD/build/r4-venv/bin:$PATH" CARGO_NET_OFFLINE=true make check characterization`
passed the complete publication union: no-provider unit/component/recovery,
real-loopback contract/product flows, I01–I04, H19/W20, bounded endurance and the
intervening REPLAI terminal regression. The existing pinned terminal-test venv
was reused; no dependency installation or provider deployment was performed.
Standalone I05 proves native and composed dispatch, restart reuse and prevention
of cross-target retry. Manual-command replay independently proves ordered
selection, trust exclusion, pinned refusal and immutable SEND through `./yai`.

The catalog audit reports 420 entries, 334 compiled Rust tests, 83 assertion
files and 77 Make leaves; zero duplicate leaves in the combined gate. Those
counts describe reachability, not external-provider qualification. Four new
Rust proofs and one loopback product/recovery entry are classified. Registry
audit remains 180 operations, zero handler/help failures. Fmt, Bash manual
syntax, diff checks and Clippy complete; Clippy retains existing warnings outside
the changed logic (12 engine, 13 CLI), not a warning-free claim.

## Direct archaeology

Inspected cognitive.rs, provider_governance.rs, cognitive_cli.rs, provider.rs,
Transition/CaseState reducers/version guards, LMDB bind/selection/invocation and
derived-content admission, registry and current I01–I04/validation dossiers.
Current source, rather than old future-work text, dictated the implementation.

Reinspected yai-dev runtime_provider_selection.c, provider routing headers,
model_provider_compatibility.c and history at `dda93ee3a`; also the pre-drain
`2a4018147^` models/registry/selection_engine.c and models/routing/model_fit.c.
The thin runtime selector ignored task_class and returned transport names.
The older selection engine had explicit candidate/exclusion machinery but also
environment-forced defaults, prompt-substring classification and model/provider
heuristics. Recover the separation of admission and explainable choice, not
heuristic semantic inference, old registries, orchestrator or provider owners.
Existing Rust governance/evidence producers are stronger and remain authoritative.

## External and remaining boundary

No operator endpoint/model configuration was present. YVEX live qualification
is not claimed. Per the current repository black-box contract, no YVEX source,
repository, model, engine or session was inspected or administered. The SHA in
the request remains an operator-supplied reference, not an observed live target.
Classification: DEPLOYMENT_LIMITATION. Generic loopback evidence is separate.

Remaining post-I05: production semantic qualification evidence, public typed
media interoperability, deliberate product-host integration, and any later
evidence-backed preference dimensions. No automatic target retry controller,
learned arbitration, hardware placement, recursive composition, streaming,
non-text output, REPLAI change, Studio, H20, W21 or W22 begins here.

See EXECUTION-EVIDENCE.md for actual qualified claims and failures, and
MANUAL-ACCEPTANCE.md for the isolated operator procedure.
