# Provider governance

Provider governance chooses an admitted cognitive substrate for a bounded YAI
invocation. It never authorizes an Operation. Policy, Decision, Review, Grant,
ResourceFence and Carrier remain the only effect-authority path.

## Ownership map

```text
Tenant
  -> immutable ProviderTarget
  -> evidence-bound ProviderQualification
  -> Tenant-Owner ProviderTrustEvent

ProviderTarget
  -> shared operational ProviderHealthState

Case
  -> CaseProviderBinding
  -> CaseCognitiveBinding (distinct semantic role/evidence relation)
  -> derived ProviderRequirement
  -> canonical ProviderSelection
  -> canonical ProviderInvocationStarted
  -> canonical ProviderAttemptOutcome / ProviderResult
```

`ProviderTarget`, qualification and trust form one Tenant-scoped governance
owner family. They share one LMDB database because their lifecycle and current
lookup are one administrative catalog. Health has a second small shared
operational database: it is fresh, multi-process routing input, not Case truth,
qualification or trust. Binding, selection and attempt outcomes are Case
Transitions and replay through current CaseState v15.

I02 adds a separate `yai.semantic_suitability_evidence.v1` record family to
the same Tenant provider-governance owner. It binds an exact target and target
digest to one closed YAI cognitive capability plus suite/run/provenance and an
explicit posture. An authenticated operator attestation is rendered as an
attestation, never mislabeled as mechanical qualification. It neither expands
mechanical ProviderQualification nor grants Case routing eligibility.

Current `yai.case_cognitive_binding.v2` is canonical Case history distinct from
`CaseProviderBinding`: the provider binding says which exact targets may be
routed to, while the cognitive binding says why one admitted target serves
`primary_conversation`, `speech_to_text`, or `image_understanding`. One current
primary slot exists per admitted participant and one auxiliary slot per
capability. Replacement and unbind are explicit historical transitions.

I05's v2 cognitive planner preserves a pinned exact target or deterministically
arbitrates an explicitly ordered candidate policy. Exact semantic suitability,
scope, integrity, current governance/health and known mechanical input shape
remain independently inspectable eligibility channels. Missing/currently
ineligible evidence yields exclusions or an unresolved plan, not inferred model
capability. Planning itself creates no ProviderSelection and dispatches nothing.
I03 revalidates the resulting exact plan before governed realization; transport
cannot silently replace its semantic target. I04 composes only explicit bounded
prerequisites; I06 routes the application conversation host through this stack.
Lane identity remains exact to Case, Participant and selected binding/target;
ephemeral continuation cannot cross lanes and is not Case continuity. Historical
v1 bindings/plans retain their original meaning; see current
[Architecture](architecture.md) and bounded I01–I06 evidence linked there.

Capabilities are derived from synthetic probe evidence. The mechanical
vocabulary is `chat_text`, `text_embedding`, `structured_json_object`,
`model_exact_addressing`, `usage_accounting`, optional `health_probe`, and
optional `extension_compatible_telemetry`. It makes no statement about model
quality or provider authenticity. Every selected capability carries
capability-specific provenance; extension-observed telemetry cannot satisfy a
qualified chat, structured-output, exact-addressing or usage requirement.
Historical v1 `first_party_telemetry` remains readable as its original claim,
but new qualification records use the narrower compatibility wording.
The v3 qualification contract introduced a parsed embedding envelope
and its exact finite dimension. Current v5 adds separately evidenced typed
realization shapes, native function calling and JSON-object output; old versions
remain readable without inventing absent mechanical evidence. An embedding
probe uses fixed synthetic non-Case text. W19 automatic memory encoding also
requires loopback locality, the exact qualified profile dimension and Tenant
approval, so generic provider approval alone cannot transmit Case memory.

Trust has three projections: `unreviewed`, `approved`, and `denied`. Only the
Tenant Owner can write approval or denial. Approval means the Tenant admits
the target for cognition routing; it is not an external security or compliance
claim. A target can independently be qualified, approved, healthy, or none of
those.

Health begins `unknown`, becomes fresh only through an authenticated synthetic
probe or a canonical provider-attempt outcome, and expires back to effective
`unknown`. Qualification, trust and health use YAI's persisted effective-time
floor, so wall-clock rollback cannot resurrect expired eligibility or stale
health. Three consecutive failures open the shared circuit for 30 seconds.
After cooldown, one exact boot-ID/PID/process-start identity owns the half-open
probe; live ownership excludes concurrent probes and dead ownership is
reclaimable. Provider/model text cannot write health.

Synthetic probes have an exact retained submission in this same Tenant owner.
`provider.probe` admits bounded synthetic work; `provider.probe.get` observes it
under current Owner authority. `provider.probe.list` / `yai provider probes TARGET`
recover all retained identities for one exact target (at most 64), including
interrupted runs when a client has lost its local reference. Listing checks the
current Tenant Owner and performs no network dispatch. The CLI qualification and onboarding adapters
reuse this carrier. Admission and the existing probe lease commit before HTTP;
an identical submission observes its existing run, while different input under
the same identity refuses. Reopening an interrupted submission never dispatches
it again. This operational record is not a Case Turn or Transition.

Measured evidence and optional qualification commit together. Credential rotation
during a run retains the measurement with `provider_probe_credentials_changed`
and does not qualify the new credential revision. Trust and Case binding remain
separate actions. Retention is bounded to 64 runs per target and 4096 per profile;
reaching either limit refuses a new submission without network activity. The
current contract provides no destructive cleanup of these retained identities.
Studio consumes this start/observe pair in the deployment Evidence section.
The local recovery reference is scoped to Host profile, Tenant and exact target;
results are re-observed through current authority after leaving the Surface.
Text, JSON/function roundtrip and embedding checks are explicit choices.
Failed local reference retention prevents dispatch; lost acknowledgement never
causes automatic redispatch. Manual evidence import remains an advanced path.

The shared probe reuses the current Rust CLI's exact response validation and
correlated function roundtrip. Catalog membership alone cannot establish exact
response identity: text and JSON probes reject missing or different response
model identities. Historical `yai-dev` commit `dda93ee3a` used a C
socket/curl probe and substring discovery in
`src/runtime/provider/runtime_provider_lifecycle.c`; those weaker mechanisms and
their lifecycle ownership are not recovered.

The target stores a credential reference, never credential material. Rotating
the secret behind the same reference records a non-secret monotonically
ordered credential revision in the existing governance owner. It preserves the
immutable target and trust history, invalidates credential-dependent current
qualification, resets operational health to unknown, and requires a new
qualification at that revision. Changing the credential reference itself
creates a new target identity.

## Selection and delivery

The Case binding holds exact immutable target refs in explicit preference
order. The selector first rejects wrong-Tenant or corrupt targets, missing
credentials, non-approved trust, missing/expired qualification, insufficient
qualified capabilities, open circuits, unavailable health, attempted targets,
and unsafe failover. Only then does it prefer healthy over unknown over
degraded, followed by binding order and target ID. The resulting selection and
bounded exclusion reasons are canonical Case facts.

An exact canonical selection admits the selected Participant's bounded model
projection for that invocation. This is derived invocation admission, not an
ambient interactive view: invocation start revalidates the binding, target,
qualification, trust and circuit in its Case transaction before any network
dispatch.

Ordinary governed Conversation and Workflow compile current Recall-aware W3
before that dispatch. The exact committed input supplies a deterministic Recall
query; task/output-contract and current control remain separate compilation
requirements. The common store compiler owns Recall and W, and the compatibility
lowerer only consumes W. Invocation admission requalifies the complete W under
current authority/backing in its transaction and refuses stale material. This
adds a full qualification pass at the final fence; it is not a freshness lease
or a provider-specific retrieval path. See the [typed contract](recall.md#ordinary-execution-consumer-convergence).
Old ProviderAttached pins and explicit W20 consolidation retain declared S-only
compatibility, never as a fallback from a failed governed W3 compilation.

Endpoint locality is revalidated against every resolved address before each
new governed connection. Mixed address classes fail closed. Loopback and
private targets cannot drift to public addresses, while remote targets cannot
resolve to loopback, private, link-local, multicast or unspecified addresses.
Remote targets use real TLS 1.2/1.3 through rustls with certificate-chain,
hostname and SNI validation. There is no product skip-verification switch and
redirects are refused rather than forwarding credentials to another authority.

One logical ModelWork turn may have at most three provider attempts. Failover
policy is only `none` or `safe_only`. A connect or write failure before any
application HTTP request byte is safe to route to the next eligible target.
DNS and TLS handshake bytes do not count as provider request delivery. Once
application bytes may have left YAI, missing or invalid response truth is
`delivery_indeterminate` or `response_invalid`; `safe_only` forbids automatic
alternate invocation. Generic 429/5xx responses, malformed 200 responses and
provider-supplied idempotency strings are not treated as proof that model work
did not execute. Exactly one canonical attempt outcome is admitted per
turn/attempt.

Provider continuation/KV references remain optional acceleration. A governed
selection clears them, so changing target or model rebuilds input from
canonical Case state. Semantic continuity never lives in provider KV.

## Compatibility and YVEX

External Golden is a separate characterization/qualification axis, not a
default gate on YAI implementation or publication. A live-provider property
gates only a wave that explicitly selects it. Historical failures below remain
external findings; they neither block unrelated semantic/security work nor
qualify a PASS. Exact-request preflight is bounded producer evidence, not general
target qualification or execution acceptance.

Mechanical shape qualification does **not** qualify a general executable
request envelope. An explicitly configured `yvex.http.v1` extension can consume
the public `yvex.openai.compat.v3` catalog and same-origin preflight contract.
After final wire lowering (including tools/feedback), YAI checks advertised body
bytes and submits exactly those bytes to preflight. Exact model/deployment and
capacity identities must agree; incompatible input refuses before inference.
Compatible input is sent unchanged. Generic providers and older public contracts
retain unknown token capacity; no token count is guessed from bytes.
`minimum_context_units` explicitly fails
closed when required; it must not be inferred from a model name. The capability
work-loop `max_input_units` check over the complete serialized body is an
application work bound (byte-derived units), not a model token count or an
attestation of the remote HTTP/token envelope. This bound now covers ordinary
text requests as well as capability work-loop requests.

`execution.get` accepts optional `include_context` for Conversation/composition
observations. It resolves canonical invocation lineage, validates archived W
against current disclosure, and returns exact retained W/Projection/ContextFrame
plus a derived serialized-request digest/byte count and capacity observation.
Missing retained backing stays unavailable. Reads do not dispatch, retry, or
claim that a preflight reserves resources. Studio exposes this explicit read
under Conversation → Execution details → Inspect model context; CLI can inspect
the corresponding retained artifact through `context inspect --id`.
Optional evidence refitting remains open: this boundary refuses rather than
truncating content or synthesizing a summary. Working State W is not YVEX E.

Refitting must preserve the existing disclosure/admission boundary, not merely
the eventual inference call. `execute_prepared` commits the exact invocation
lineage before sending its serialized content to preflight.
`commit_cognitive_invocation_authorized` requalifies W, Recall/source closure,
Participant and projection in the same write transaction. A policy revocation
can invalidate W even without a new Case generation; cached compilation is not
an authorization lease. Moving content-bearing preflight ahead of this check
would disclose Case material before that admission. Replacing W afterwards
would contradict the committed invocation lineage.

The remaining preparation boundary must therefore authorize each disclosed
candidate against current authority, retain its exact W/request identity and
capacity observation, and select one final admitted context without implicit
inference retry. Optional groups remain atomic, required references remain
required, and lack of room for mandatory state remains a refusal. Existing
W compilation budgets and exact-wire preflight are reusable parts of this
boundary, not proof that their adaptive composition already exists. The engine
`working_recall_policy_current_asof_freshness_tamper_and_atomic_budget` test
protects same-generation revocation; the Application exact-preflight test
protects unchanged wire bytes and refusal without inference.

The [external Golden forensic evidence](../labs/external-runtime/external-golden-closure/REPORT.md)
shows why these contracts cannot be conflated: YVEX accepts a 40,277-byte
whitespace-padded synthetic body but rejects the actual same-sized Golden body
with `token output capacity exceeded`. Small synthetic shape probes do not
authorize a complete Golden PASS. After deployment alignment, public compat.v3
catalog and preflight distinguish these quantities. Generation 1 offered 512
input/sequence tokens and refused the 12,146-token actual input. Generation 2
offers 16,384: the same body is compatible, with 4,238 output tokens available.
The fresh real product run nevertheless reached its 300-second terminal-test
deadline without a response; canonical Invocation has no ProviderResult.
Later operator-supplied logs locate this wait in prefill: 918/12,055 tokens
processed after 293.21 seconds, zero generated, then cancellation and cleanup.
The producer's internal status 499 was not an HTTP response received by YAI.
This operational evidence neither writes a canonical terminal result nor grants
general cancellation/retry authority; the internal performance cause remains
unidentified. No larger token limit or guessed timeout follows from these logs.
Its fresh body has 12,055 input tokens and 4,329 allowance in a separate
postmortem preflight. New identities change tokenization despite equal byte size.
Both bodies omit `max_tokens` (reported requested output 0); no zero-output
budget was inserted. Compatible preflight still says
`execution_or_resources_qualified=false`: it is neither execution nor a resource
reservation, and cannot turn that failed lifecycle into PASS. That historical
diagnostic resume did not implement automatic preflight; the current bounded
consumer above does not retroactively qualify that run. Body bytes, input tokens,
sequence capacity and output allowance remain distinct. Unknown remains unknown.

Historical `ProviderAttached` Cases retain their exact pinned path and do not
gain an approval/qualification requirement. New governed Cases use
`provider add`, `provider qualify`, `provider trust approve`, and `case provider
bind` through the compiled CLI registry.

The normal data plane is generic OpenAI-compatible HTTP with exact endpoint and
model identity. The optional `yvex.http.v1` extension may observe documented
`/health`, model visibility and bounded `yvex_completion_metrics`. Those facts
prove only extension-contract compatibility, not cryptographic YVEX identity.
They are operational telemetry only. YAI does not use or administer YVEX
Source, Artifact, Profile, Engine, Session, loaded-model state, or its private
local protocol.

Historical `yai.provider_selector.v1` records are validated by their known
historical selector contract. A future selector may choose differently for new
unresolved work without rewriting the target or exclusion reasoning recorded
for an earlier Case selection.
