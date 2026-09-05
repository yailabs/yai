# Model/provider interlock I02 report

## State

I02 establishes YAI-owned cognitive capability bindings and deterministic
execution-lane planning without provider execution. The reconciled working
baseline was `5e75a803a610d6d6b1deb33693bc794f315294d8`; it preserves the requested
semantic anchor `89e44cd9a9c3aa3e33f4e3b3643e67b308addbc4` plus the authorized repository
launcher documentation follow-up.

Intended semantic commit: `feat: add cognitive capability bindings and execution lanes`.
The containing commit SHA and publication equality belong in the post-commit
handoff, not this versioned report.

## Architectural conclusion

The existing `ProviderTarget`, `ProviderQualification v3`, trust posture and
`CaseProviderBinding` remain the exact mechanical provider-governance layer.
I02 adds a separate closed v1 semantic vocabulary for
`primary_conversation`, `speech_to_text`, and `image_understanding`; exact
target-bound semantic evidence; and a canonical Case cognitive binding. An
authenticated operator can record an attestation with bounded provenance, but
the contract and CLI label it `operator_attested` and
`mechanically_qualified:false`.

The new pure planner consumes an explicit capability requirement. It never
infers speech-to-text from audio, model names, or input/output shapes. Exact
primary suitability yields a native primary plan; otherwise an exact suitable
auxiliary binding yields a derived auxiliary plan; missing or stale facts yield
a typed unresolved plan. Every result declares
`provider_execution:not_performed` and provider realization deferred.

Lane identity is derived from Case, Participant, role/capability and the exact
binding identity. Replacement changes it; replay without replacement preserves
it. Opaque continuations are accepted only for the exact lane and target, are
never rendered by default, and are disposable without invalidating the plan.

## Authority and version delta

- Transition: v13 -> v14 for cognitive bind/unbind facts; v1-v13 readers remain.
- CaseState: v12 -> v13 for bounded current cognitive bindings; v1-v12 readers remain.
- New logical schemas: `yai.semantic_suitability_evidence.v1`,
  `yai.case_cognitive_binding.v1`,
  `yai.cognitive_capability_requirement.v1`, and
  `yai.cognitive_execution_plan.v1`.
- Unchanged: ProviderQualification v3, ConversationTurn v1, Projection v7,
  ContextFrame v7, RetrievalSet v3, and ConversationContent v1.
- LMDB: 37/40. Semantic owner delta: +0. Operational owner delta: +0.

Suitability evidence uses the existing provider-governance database. Only the
Case binding is canonical. Plans and lanes are derived; continuations are
ephemeral. No provider/model/runtime/deployment owner was added.

## Compatibility and boundaries

Existing provider selection/failover and the provider envelope remain live.
A cognitive binding cannot admit or select a provider by itself. Current trust
and exact mechanical qualification are rechecked for planning; health does not
rewrite semantic meaning or lane identity. The I01 conversation host and
`yai prompt` remain unchanged. No `yai chat`, Replia, Studio, H20, W21, W22, or
I03 execution work was started.

YVEX `models1` was resolved and inspected read-only at wave start at
`3a6520945a5c103365178f48104f0ccdb5154624`. At closure the remote no longer
advertised `refs/heads/models1`; `main` and `models2` both advertised
`5b95ee82eee394581521d106c7b1ec479d472448`. A read-only comparison found no
change in `include/yvex/content.h`, `include/yvex/provider.h`, or
`docs/openai-compatibility.md` between the inspected start commit and current
`models2`. I02 introduced no YVEX ABI dependency and discovered no external
defect. Typed media realization and execution-provider adaptation remain
post-I02.

## Product surface

Six registry-backed Advanced commands record/show semantic evidence,
bind/unbind/show current Case cognitive relations, and derive a plan. All have
stable operation IDs, typed JSON, deterministic human output and registry-based
help/completion. No command dispatches a provider.

## Closure posture

YAI semantic planning is complete for the admitted I02 vocabulary. Provider
realization is explicitly deferred. The exact evidence, binding, role, route,
lane and continuation posture can be inspected while provider execution remains
absent.
