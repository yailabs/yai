# I06 — conversation execution intent and cognitive host routing

Baseline: `e9002b7a452856a7b67eb602d66360f83330ec2d`.
Initial master/HEAD/origin/master/remote equality and clean worktree were reconciled.
Intended commit: `feat: route conversation execution through cognitive intent`.
Pre-publication state: implementation and local publication qualification complete;
isolated commit, push and equality verification remain the publication step.
The containing commit SHA and push equality belong in the final handoff.

## Architectural conclusion

The old controller committed an I01 Turn, flattened text, and independently
iterated the Case provider envelope. That execution loop is removed. Both the
Advanced cognitive client and ConversationController now call the same
frontend-independent `cognitive_execution` functions extracted from I03/I04.
These functions still use the I05 snapshot/arbitration planner and I03 exact
selection/invocation/result owners. This is factoring a demonstrated two-consumer
boundary, not a new router, orchestration service or owner.

The application default is PrimaryConversation because SEND is conversational.
An optional explicit prerequisite addresses ordered input ordinals before SEND;
after immutable content publication it binds exact Turn part IDs. Audio, image,
MIME and model/provider names never establish this choice. At most the existing
I04 prerequisite is admitted. A native-capable primary bypasses it.

## Canonical intent, versions and ownership

`ConversationExecutionIntentRecorded { request }` adopts the existing
`yai.cognitive_composition_request.v1` body. It binds Case/Tenant/Participant,
source Turn/digest, ordered parts, PrimaryConversation and the optional explicit
capability/source subset. No provider decision or computational state is included.
One immutable intent per Turn; repeated identical adoption is idempotent.
Changing intent requires a new submission, not mutation of a previous SEND.

Host SEND appends Turn and intent atomically in one LMDB write transaction,
after complete object-first publication. Failure of the second append rolls
back the first; complete unreferenced content objects remain permitted under I01.
A Turn without intent (historical or engineering plumbing) adopts the default
only when explicitly executed by the conversation host. Raw draft SEND remains
usable independently, without requiring cognition.

Transition v16 → v17 is the canonical delta. Readers and the physical schema
marker accept v1–v16; migration does not rewrite history. CaseState remains v14,
without a new intent field. Intent is resolved from canonical history and
validated against the preceding exact Turn during admission and replay.
CLI TurnView v1 → v2 exposes its optional execution intent.
ConversationTurn v1, content store, derived content, I04 request/closure, plan v2,
binding v1/v2, ProviderQualification v4, Projection/ContextFrame v7 and
RetrievalSet v3 are unchanged. Semantic owner delta 0; operational owner delta 0;
LMDB delta 0, still 37/40.

## Direct/composed execution and recovery

The application returns intent plus typed composition/source closure, current
validation plan, original executed plan, lane/target, optional prerequisite
derivation, exact final selection/invocation/result and Projection/Context IDs.
The terminal presents compact lineage; it does not implement cognitive policy.

Retry retains Turn and intent, and lets existing cognitive algorithms reconstruct
fresh plans under current evidence. Compatible canonical auxiliary derivations
and recorded provider results are reused. A completed result is not another
remote call merely because /retry was requested. Any unresolved potentially
delivered invocation on the same Turn blocks redispatch even if a new target
or source closure would otherwise change the requirement identity. This is an
application safety gate, not a new delivery taxonomy. Exact I03 admission guards
remain active below it. Missing/invalid prerequisites block dependent primary
execution. Provider output and derived content remain candidate application
material, never operational authority.

Cancellation is observed before future dispatch/stages. A buffered request
already dispatched is not claimed aborted. Lost continuations reconstruct
semantically; no cross-lane state is passed. Explicit resolution of uncertain
delivery is future work.

## Retained versus retired surfaces

Retired: controller text flattening, independent provider-order loop and its
automatic transport-target fallback. Retained: `./yai prompt` native REPLAI,
thread projection, typed host input, Advanced cognitive inspection/realization,
I01 deterministic content plumbing, noninteractive legacy `prompt --once` /
piped diagnostics and the separate bounded operational Case runtime.
No new registry operation or Product `yai chat`. No REPLAI revision change,
R5 work, terminal path/media UI, Studio, H20, W21 or W22.

## Direct archaeology and recovery verdict

Current controller, terminal, cognitive planning/governance, I03 realization,
I04 composition, Transition/LMDB, context/provider transport and I01–I05/R4
contracts were directly inspected. Source determined that the I04 request body
already expresses the needed semantic intent; only canonical adoption was missing.

yai-dev HEAD `5c1c7b9d099eea9f2947146cd821d6501c4a6ddf` was reinspected with history:
selection_engine.c, frame_build.c, conversation_runtime.c/.h and
conversation_composer.c; relevant epochs include `2a4018147`,
`dda93ee3a`, `6ce6a3bde`, `cffb318b9`, `e9ad7f498`.
Legacy explicit requested modes and input→frame→selection references preserve
useful intent/provenance distinctions. Prompt-substring classification,
model/provider-name inference, mutable workset and Agent/orchestrator ownership
remain rejected. The runtime conversation umbrella is compatibility scaffolding,
not an execution owner to restore. Current typed I04 contracts are stronger.

## Qualification and external boundary

See [EXECUTION-EVIDENCE.md](EXECUTION-EVIDENCE.md) for actual runs, failure
evidence and mode-specific conclusions, and [MANUAL-ACCEPTANCE.md](MANUAL-ACCEPTANCE.md)
for ordinary operator commands. Loopback is real adapter/HTTP proof, not YVEX
interoperability or model quality.

`make test-fast` and the final offline `make check characterization` publication
union passed. Canonical no-provider recovery, real loopback contract/product
flows, real REPLAI PTYs, I01–I05, H19/W20 and retained bounded endurance remain
covered. Catalog audit: 424 entries, 336 compiled Rust identities, 83 assertion
files. These are reachability counts, not an aggregate external proof.
The independent operator replay also completed and cleaned its disposable Home.
Clippy retains the baseline 12 engine / 13 CLI warnings outside changed logic.
Fmt, registry, docs/layout, manual Bash syntax, TSV and diff checks passed.

YVEX EXTERNAL FINDINGS: endpoint/model configuration is absent. The explicit
external lane reports `blocked_external_dependency` / `DEPLOYMENT_LIMITATION`,
no external request and no invocation. No YVEX source, CLI or deployment was
inspected/administered. The prompt's public ref is not claimed independently
observed. Native public typed-media YVEX interoperability remains unqualified.

## Remaining post-I06 pressure

Production semantic evidence beyond fixture/operator attestation; public
external typed-media qualification; explicit resolution of uncertain delivery;
future streaming and attachment/capture frontend actions. Noninteractive legacy
diagnostic migration can be evaluated separately. No next wave is started.
