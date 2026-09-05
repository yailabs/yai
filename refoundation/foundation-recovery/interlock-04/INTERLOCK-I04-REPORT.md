# Model/provider interlock I04 report

## State

I04 closes bounded cognitive execution composition over published I03 baseline
`e1787b7950ec3ac8cbdff97d04224023483120ee`. YAI can now execute either a
proven direct primary realization or one explicit auxiliary prerequisite,
publish/reuse its exact derived content, reconstruct an ordered source closure,
and perform a fresh primary realization through the real I02/I03 boundaries.

Intended semantic commit: `feat: add cognitive execution composition`. The
containing SHA and remote publication equality belong in the post-commit
handoff.

## Architectural conclusion

Composition is process-local execution control, not a new owner. The explicit
`yai.cognitive_composition_request.v1` binds one primary-conversation goal,
canonical Turn and ordered source selection, plus at most one operator-stated
`speech_to_text` or `image_understanding` prerequisite. MIME, provider names
and model names never create semantic intent.

`yai.cognitive_source_closure.v1` is a deterministic content-addressed view. It
records every selected original part in Turn order as retained, replaced by
one exact derived object, or consumed by that derivation. Delivery order and
object identity are explicit. The closure never copies bytes, flattens
provenance, mutates the Turn, or becomes CaseState.

The direct path first proves the primary I02 semantic binding and current I03
mechanical shape. If both are present, the primary target consumes the original
closure and the auxiliary is bypassed. Otherwise an explicit prerequisite is
required. Its fresh I02 auxiliary plan is realized through I03, then the exact
canonical derived relation replaces only its named source parts in the
downstream closure. A fresh primary I02 plan consumes that closure through the
same exact-target I03 realization seam.

ProviderSelection causal refs retain the composition request, source closure,
source Turn and, when applicable, derived-content/ProviderResult identities.
Provider-specific serialization remains noncanonical adapter work.

## Restart and failure semantics

No composition run record is needed. After a prerequisite is canonically
published, a new process reconstructs the same request, validates the exact
source/binding/evidence/target/qualification/normalizer provenance, verifies
the content bytes, and reuses it without remote redispatch. It then rebuilds
the same source closure and can continue the primary stage. A repeated complete
composition also recovers the recorded primary result rather than duplicating
work.

A changed binding, target, evidence or current ProviderQualification cannot
reuse stale execution assumptions. Existing delivery taxonomy remains
authoritative. An unresolved, failed, malformed, non-normalizable or
delivery-indeterminate prerequisite publishes no usable derivation and blocks
the primary. Each internal I03 realization is constrained to the route the
composition just established; a concurrent native/derived route change fails
before dispatch and requires replanning. No different cognitive target is
substituted.

Auxiliary and primary stages retain separate deterministic I02 lanes. No
continuation migrates between them; composition remains correct with no
continuation at all. No provider session, engine, lease, KV/recurrent state,
residency or transport connection enters semantic identity.

## Version and ownership delta

- New derived schemas: `yai.cognitive_composition_request.v1` and
  `yai.cognitive_source_closure.v1`.
- Unchanged canonical schemas: Transition v15, CaseState v13,
  ConversationTurn v1 and ConversationDerivedContent v1.
- Unchanged provider/context contracts: ProviderQualification v4,
  CognitiveExecutionPlan v1, Projection v7, ContextFrame v7 and RetrievalSet
  v3.
- LMDB: 37/40.
- Semantic owner delta: +0. Operational owner delta: +0.

Case still owns canonical continuity; ConversationContentStore still owns
immutable bytes; I02 still owns pure planning; I03 still owns exact one-plan
realization; provider governance still owns target evidence and selection.
There is no Orchestrator, Agent, WorkflowRun, pipeline store, lane manager or
new database.

## Product and compatibility

One Advanced registry-backed `yai case cognitive compose` operation exposes
the bounded controller with typed JSON and deterministic human output. Existing
I01 draft/Turn plumbing, I02 planning, I03 single-plan realization,
conversation host, `yai prompt`, provider governance, memory and Case runtime
remain intact. I04 adds no Product `yai chat`, Replia, Studio, natural path
parsing, streaming, Workflow appropriation, H20, W21 or W22.

## External boundary

Public YVEX `main` resolved to
`3f4a1c182d35e5a0e163adb81008ae7a366efcc6`; `models1` was not advertised.
No operator endpoint/model variables were available, so no live YVEX
multipart composition is claimed. Both executing stages were qualified through
real deterministic loopback OpenAI-compatible provider dispatch. Public YVEX
typed-media realization remains an external adapter pressure, not a YAI defect
or permission to consume a private protocol.

## Closure posture

I04 demonstrates both required paths: direct primary realization with proven
native mechanical shape, and explicit prerequisite → auxiliary I02/I03
realization → canonical derived content → deterministic source closure → fresh
primary I02/I03 realization. Composition remains finite, exact-target,
restart-safe, provenance-preserving and ownerless. Dynamic arbitration,
recursive graphs, automatic host routing and production multimodal provider
onboarding remain explicit post-I04 work.
