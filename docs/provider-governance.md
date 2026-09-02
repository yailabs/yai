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

Capabilities are derived from synthetic probe evidence. The v1 mechanical
vocabulary is `chat_text`, `structured_json_object`,
`model_exact_addressing`, `usage_accounting`, optional `health_probe`, and
optional first-party telemetry. It makes no statement about model quality.
Every selected capability carries provenance; normal text and PlanPatch
requirements demand qualified evidence.

Trust has three projections: `unreviewed`, `approved`, and `denied`. Only the
Tenant Owner can write approval or denial. Approval means the Tenant admits
the target for cognition routing; it is not an external security or compliance
claim. A target can independently be qualified, approved, healthy, or none of
those.

Health begins `unknown`, becomes fresh only through an authenticated synthetic
probe or a canonical provider-attempt outcome, and expires back to effective
`unknown`. Three consecutive failures open the shared circuit for 30 seconds;
the next posture is half-open until a real observation closes or reopens it.
Provider/model text cannot write health.

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

One logical ModelWork turn may have at most three provider attempts. Failover
policy is only `none` or `safe_only`. A connect or write failure before any
request byte is safe to route to the next eligible target. Once bytes may have
left YAI, missing or invalid response truth is `delivery_indeterminate` or
`response_invalid`; `safe_only` forbids automatic alternate invocation. A
generic HTTP status is not treated as proof that model work did not execute.
Exactly one canonical attempt outcome is admitted per turn/attempt.

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
are operational telemetry only. YAI does not use or administer YVEX Source,
Artifact, Profile, Engine, Session, loaded-model state, or its private local
protocol.

W18's repository transport qualifies loopback/private plain HTTP fixtures. A
remote target must be declared HTTPS, but this dependency-minimal binary does
not yet implement TLS transport; it remains configured but cannot qualify and
therefore cannot be selected. This is an explicit deployment limitation, not
a silent downgrade to remote HTTP.
