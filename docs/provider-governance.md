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
Transitions and replay through CaseState v12.

Capabilities are derived from synthetic probe evidence. The mechanical
vocabulary is `chat_text`, `structured_json_object`,
`model_exact_addressing`, `usage_accounting`, optional `health_probe`, and
optional `extension_compatible_telemetry`. It makes no statement about model
quality or provider authenticity. Every selected capability carries
capability-specific provenance; extension-observed telemetry cannot satisfy a
qualified chat, structured-output, exact-addressing or usage requirement.
Historical v1 `first_party_telemetry` remains readable as its original claim,
but new qualification records use the narrower compatibility wording.

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
