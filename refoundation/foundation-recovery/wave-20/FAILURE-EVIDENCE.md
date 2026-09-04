# W20 failure evidence

## F-W20-01 — consolidation context over-admission found by source audit

The first W20 integration reused the general projection compiler with non-zero
provider-claim and interaction-turn budgets. Direct inspection showed that a
consolidation invocation could therefore receive unrelated current Effects,
Reviews, ProviderClaims, or derived retrieval in addition to its immutable
consolidation packet. This violated exact-input confinement even though the
existing fixture happened not to exploit the extra entries.

The fix makes `MemoryConsolidation` projection retain only Case/Tenant,
Participant, and selected Provider identity entries; optional history and
derived retrieval are suppressed. The focused test
`context::tests::consolidation_projection_contains_only_identity_envelope`
passes and also proves the strict no-tool/no-authority instructions are present.

## F-W20-02 — shared support ancestor initially treated as a cycle

The first support walk used global visitation semantics, causing a valid diamond
graph with one shared grounded ancestor to fail as cyclic. The final iterative
walk carries a path-local set per branch: shared ancestors pass, while
`A -> B -> A`, self-support, support without a canonical leaf, fan-in above 16,
and depth above 16 fail closed. The focused tests
`w20_s08_support_cycle_rejected_but_shared_ancestor_is_valid` and
`w20_support_depth_and_contradiction_storm_are_bounded` pass.

## F-W20-03 — validation harness compile defect

The first CLI focused test used `unwrap_err()` on a successful type lacking
`Debug`; Rust rejected the test build with `E0277`. `DecodedProviderResponse`
now derives `Debug`. The tool-call rejection test and the complete CLI suite
then pass. This was a test-build defect, not a product failure.

## F-W20-04 — multi-family projection erased W19 operational kind

The first complete characterization reached the provider semantic-continuity
fixture with a valid v6 frame, but the consequence turn returned HTTP 409. The
captured rejected frame proved that RetrievalSet v3 had selected the exact
observed OperationalMemory item and retained its observation/receipt
provenance, while the v3-to-Projection adapter had replaced
`resource_effect/finalized_observed_consequence` with the generic
`operational/mechanically_grounded` labels. This broke the established W19
typed-context contract.

The adapter now preserves the exact OperationalMemory semantic kind and
posture while episodic and semantic sources retain their explicit family and
epistemic labels. The focused semantic-continuity test subsequently passed all
four paths, and both `make characterization` and `make check` passed.

## F-W20-05 — active provider fixtures pinned to ContextFrame v5

The first characterization pass also found active loopback fixtures that still
required `yai.context_frame.v5` after W20 legitimately advanced serialized
Projection/Context semantics to v6. The affected current fixtures were updated
to v6; compatibility readers and historical fixtures were not rewritten. The
direct provider-model vertical passed before the complete suites were rerun.

## F-W20-06 — pre-existing H10 timing check was transient under parallel load

One characterization attempt reported
`h10_review_writes_rederive_roles_provenance_and_final_decision` with
`authority_decision_time_mismatch`. An immediate exact isolated rerun passed,
and the next unmodified complete `make characterization` and `make check` runs
both passed the same test. No H10 source was changed; this is retained as an
observed qualification-host transient rather than hidden or attributed to
W20.

No live YVEX or live encoder failure is claimed: the required endpoint/model
variables were not present.
