# Governance source and PolicyArtifact reference

Authority: current Wave-8/H8 governance authoring, Wave-9 materialization,
Wave-10/H10 admission, Wave-11 temporal governance and Wave-12 Tenant security
contracts.

## Boundary

Wave 8/H8 implements this Case-independent supply chain:

```text
constrained source bytes
  → PolicySourceArtifact
  → ParsedPolicy facts
  → normalized PolicyIr
  → immutable PolicyArtifact candidate
  → validation
  → explicit publication
```

Wave 9 implements the next distinct boundary:

```text
published PolicyArtifact
  → exact canonical CasePolicyBinding
  → derived EffectivePolicy
  → derived NormativeReadiness
```

A published artifact is merely eligible for binding. A binding is durable Case
configuration, not a policy result. EffectivePolicy is deterministic normative
input, not authority. Only Wave-10 evaluation of a normalized Operation under a
Ready EffectivePolicy can produce DecisionBasis and a Decision; only final
ALLOW under the same current basis can issue an ExecutionGrant.

## Input grammar

`yai.policy_source_input.v4` is bounded UTF-8 JSON with no unknown top-level
fields:

```json
{
  "schema": "yai.policy_source_input.v4",
  "policy_key": "organization.example.filesystem",
  "source_version": "1",
  "owner_ref": "organization:example",
  "source_origin": {
    "source_system": "policy-intake",
    "source_uri": "internal://governance/filesystem"
  },
  "validity": { "mode": "unbounded" },
  "rules": [
    {
      "kind": "review_requirement",
      "rule_id": "review-workspace-write",
      "operation_kind": "filesystem.write",
      "resource_kind": "filesystem",
      "required": true,
      "reason": "workspace writes require review"
    }
  ]
}
```

Limits are 256 KiB, 128 rules and 32 JSON levels. UTF-8 BOM, invalid UTF-8,
duplicate keys at every object level, non-ASCII/confusable identifiers and
local absolute/file URIs fail closed. Known rules are strict objects. Malformed
JSON, missing fields, unknown schema versions and unknown fields in a known
rule fail parsing. An unknown rule `kind` is retained as an unresolved item so
an operator can inspect the candidate, but blocks qualification and
publication. No LLM interprets source material.

The current compiler implements these parsed fact kinds:

| Kind | Typed semantic payload | Current consumer |
|---|---|---|
| `operation_restriction` | operation/resource selector, ALLOW or DENY posture, reason | EffectivePolicy and closed-world operation admission |
| `review_requirement` | operation/resource selector, required boolean, reason | EffectivePolicy and ReviewRequest posture |
| `evidence_obligation` | operation/resource selector, pre/post observation, audit reason or source-provenance obligation | admission/effect evidence requirements |
| `authority_requirement` | proposer/reviewer subject plus required Case role for an operation/resource selector | proposer and reviewer eligibility |

Authority requirements are additive/all-of. They are scoped to one evaluation
and never create ambient Participant permission.

## Source and provenance

`yai.policy_source_artifact.v4` stores:

```text
source_id
content_digest
source_format = constrained_json
policy_key
source_version
owner_ref
declared source_system + source_uri
bounded exact UTF-8 content
```

`source_id` is the SHA-256 digest of the exact source bytes. Exact duplicate
ingest is idempotent. Any byte edit produces a different source identity. Full
content is currently retained so the compiler input can be reconstructed;
operator inspection withholds it by default. Origin is bounded, digest-covered
declared provenance; it is not authenticated ownership and never records the
local import path. The v1 contracts remain readable but explicitly report
origin unavailable. The origin is part of the exact source document: changing
it changes source identity, while repeated intake of the same bytes coalesces
idempotently and does not create a separate observation history. A future
consumer needing repeated intake observations must earn a separate event
contract. A future privacy/retention policy
may separate payload retention, but cannot erase the content digest or the
provenance carried by retained artifacts.

There is currently no product source-deletion lifecycle. If a source artifact
is unavailable or corrupt, its PolicyArtifact still contains the source digest,
declared origin, typed facts and IR, but byte-level recompilation is no longer
claimable. That failure posture is not advertised as a retention feature.

`yai.parsed_policy.v2` gives every typed fact a deterministic identity, source
artifact ref and JSON location such as `$.rules[0]`. `yai.policy_ir.v2` carries
those fact refs and source locations on every normalized rule. Combined rules
retain all contributing refs; citations are never invented.

## Normalization and qualification

Normalization is deterministic for the same bytes/compiler version. It:

- removes source formatting from typed semantics;
- orders output deterministically;
- merges semantically identical rules while retaining provenance;
- preserves unknown kinds as unresolved;
- records contradictory outcomes for the same typed selector as conflicts;
- emits a SHA-256 IR digest.

`yai.policy_validator.v1` re-derives the complete validation disposition from
stored Policy IR, so altered status/blockers/IR cannot qualify. It returns
`qualified` only when at least one supported
normalized rule exists and there are no unresolved items or conflicts. A
blocked candidate remains inspectable. Validation never guesses cross-artifact
precedence; Wave 9 materialization applies the explicit conservative algebra
documented below.

## Immutable PolicyArtifact and lifecycle

`yai.policy_artifact.v5` binds:

```text
artifact_id
Tenant-scoped lineage = tenant_id + policy_key
tenant_id + organization_ref projection
artifact_version unique inside the lineage
declared source origin
source_id + source_digest
ParsedPolicy + parsed_digest
PolicyIr + IR digest
deterministic validation disposition
```

Artifact identity is content/provenance/security-domain-derived. Identical
source bytes may have one source digest across Tenants, but authority artifacts
and lineages remain Tenant-distinct. Changed bytes at the same lineage/version
collide and fail before any write. Different Tenants cannot supersede each
other, and declared versions are
not interpreted as SemVer or sorted authority. Stored artifact bytes are
immutable. Lifecycle is a separate append-only
`yai.policy_lifecycle_event.v3` history with global LMDB order, Tenant and
authenticated Principal, prior/next state, reason, optional related artifact,
time and integrity digest.

```text
candidate
  → validated
  → published
       → superseded
       → retired

candidate | validated | superseded
  → retired
```

Publication requires deterministic validation. Publishing a new validated
artifact in the same owner-scoped lineage supersedes the prior published artifact
atomically; both immutable versions remain inspectable. Retired and superseded
artifacts cannot be republished.

`runtime_consumable` is a derived view:

```text
validation == qualified AND lifecycle == published
```

It means only “eligible for later Case PolicyBinding/materialization.”

## Exact Case PolicyBinding

`yai.case_policy_binding.v2` binds one Tenant-scoped Case to one exact immutable
Tenant-owned artifact.
The binding contains the Case and binding identities, owner-scoped lineage,
artifact ID and declared version, source and IR digests, bind-time publication
event ID/sequence, resulting Case generation, claimed local actor/reason and an
optional replaced-binding ref. Its identity and integrity digest cover those
fields.

Binding was introduced as a canonical v5 payload; current `yai.transition.v8`
records it and `yai.case_state.v8` materializes only active compact binding
records, with one binding per lineage.
Bind and replace validate catalog eligibility and append the Case Transition in
one LMDB write transaction. New binding requires the exact artifact to be
integrity-valid, qualified, currently published and `runtime_consumable`.
Replacement is one atomic transition; unbind is another. A repeated identical
bind is an idempotent no-op.

Publication of `P@2` never changes a Case pinned to `P@1`. The Case reports
catalog drift (`current`, `superseded`, `retired`, or no current publication)
until an operator explicitly replaces the binding. Drift remains separate from
Wave-11 validity/revoke posture. Binding v2 records the authenticated Principal;
source `owner_ref` cannot substitute for Tenant equality.

## EffectivePolicy and normative readiness

`yai.effective_policy.v3` is derived and rebuildable from current CaseState
bindings plus their exact retained PolicyArtifacts. The materializer contract
is `yai.policy_materializer.v3`; both retain the exact immutable Case Tenant.
It sorts inputs independent of ingest,
publication, binding and LMDB cursor order. Its semantic identity covers the
Case, sorted exact binding/artifact inputs, materializer version and normalized
effective rules, but not wall clock, process or unrelated Case generations.

The v2 composition algebra is intentionally small:

- DENY dominates ALLOW for the same operation/resource selector;
- `required=true` dominates `false` review posture;
- evidence obligations form a deterministic set union;
- proposer/reviewer role requirements form deterministic all-of sets;
- identical effective semantics merge all contributing fact/rule/artifact
  provenance;
- duplicate active lineage, missing/corrupt artifact, binding mismatch or an
  unrepresentable collision blocks materialization.

Every effective rule retains all contributing artifact, Policy IR rule, fact,
source-location and source-artifact refs plus an explicit resolution reason.
No source bytes are reparsed during materialization.

Normative readiness is a derived view:

```text
unconfigured  no active bindings
ready         bindings and exact artifacts validate; materialization succeeds
blocked       declared inputs are missing, corrupt or cannot compose safely
```

Readiness is not Case lifecycle and is never a stored free authority boolean.
The optional `effective_policy_by_case` LMDB cache is derived and droppable;
status/rebuild reads do not append Case or governance history. Cache failure
after a canonical bind leaves the binding committed and repairable.

## Policy-driven operation admission

Wave 10 adds one operational consumer without moving materialization authority.
Current `yai.decision_basis.v3` binds the Case Tenant and generation,
Operation/resource, EffectivePolicy v3 identity/digest/materializer, exact Case
bindings and artifacts,
matched rules/provenance, resource-envelope result, proposer/reviewer role
eligibility, temporal posture, evidence obligations and final posture.
`yai.decision.v3` embeds and integrity-binds that basis; old schemas remain
compatibility-readable.

The current closed-world algebra is deterministic:

1. invalid Case/Operation or non-Ready normative state fails;
2. hard resource-envelope violation denies;
3. missing Case-bound proposer role denies;
4. applicable DENY denies;
5. absence of explicit applicable ALLOW denies;
6. impossible admission evidence denies;
7. audit rationale or policy review creates REQUIRE_REVIEW only when an
   eligible Case participant exists;
8. otherwise admission is ALLOW.

`source_provenance` requires the canonical ProviderInvocation/ProviderResult
lineage. An evaluator-generated reason cannot satisfy `audit_reason`; an
eligible approved ReviewAction reason can. Policy pre/post observation
obligations travel in finite `yai.execution_grant.v3`, while the carrier's mandatory
pre/post safety remains unconditional.

`yai.review_request.v2` binds the original Operation, DecisionBasis,
EffectivePolicy and required reviewer roles. ReviewAction v2 resolves and
records the per-invocation authenticated Principal through Tenant membership
and an explicit Principal/Participant link; the link adds no Case role. Policy
change before review resolution is stale and cannot yield ALLOW. Grant issuance transactionally
re-materializes current readiness and requires the exact same EffectivePolicy
and binding set, preventing a Decision under E1 from issuing a Grant after E2
becomes current. These checks are immediate basis consistency, not Wave-11
expiry/revoke semantics.

## Persistence authority

The existing LMDB environment hosts four canonical governance databases:

- immutable policy sources by ID;
- immutable policy artifacts by ID;
- lifecycle events by ID;
- append order for lifecycle events.

A fifth, non-authoritative index accelerates current publication lookup by
lineage and is rebuildable from artifacts/events. Superseding the old artifact,
publishing the new artifact and updating this index share one LMDB transaction;
abort/restart cannot expose half publication.

This is a canonical governance history with an independent multi-Case future
lifecycle. It is not CaseState, a Case Transition stream, graph/memory derived
state, or `case:__system__`. A PolicyArtifact may be published with zero Cases.
Inspection/listing does not mutate either governance history or Case history.

The same environment now hosts one separate security catalog owner:
`yai.security_principal.v1`, `yai.tenant.v1`, and append-only
`yai.security_event.v1` records with indexes by Principal binding, Tenant and
membership. `AuthenticatedPrincipal` is a sealed invocation value derived from
kernel real/effective POSIX credentials; the effective UID binding selects the
Principal. Organization is immutable Tenant metadata, not a second isolation
domain. Security catalog reads are pure.

The shared LMDB map defaults to 256 MiB (formerly 16 MiB) and embedding callers
may configure it down to the documented 16 MiB minimum. The H8 supported
catalog contract covers 256 retained sources of up to 256 KiB under the
default. Capacity exhaustion is explicit and transactionally harmless.

## Operator trust and commands

The local authenticated CLI supports:

```text
yai security bootstrap-local --tenant <tenant:id> --organization <organization:id>
yai identity whoami
yai tenant list
yai tenant status --tenant <tenant:id>
yai policy ingest <source.json> --tenant <tenant:id>
yai policy inspect <source-id|artifact-id>
yai policy validate <artifact-id> --reason <reason>
yai policy publish <artifact-id> --reason <reason>
yai policy retire <artifact-id> --reason <reason>
yai policy revoke <artifact-id> --reason <reason>
yai policy list
yai case policy bind --case <case:id> --artifact <artifact:id> --expected-generation <n> --reason <reason>
yai case policy replace --case <case:id> --binding <binding:id> --artifact <artifact:id> --expected-generation <n> --reason <reason>
yai case policy unbind --case <case:id> --binding <binding:id> --expected-generation <n> --reason <reason>
yai case policy status <case-id>
yai case policy rebuild <case-id>
```

New scoped mutations authenticate on every invocation and re-check Principal,
Tenant and Owner membership at the store boundary. A retained `--as` value is
compatibility-only and must equal the resolved Principal; it cannot authenticate
or select authority. This local POSIX trust model does not prove enterprise
authentication, external Organization identity, remote signature or security
against a process with unrestricted out-of-band LMDB file access.

## Compatibility and non-claims

Historical `yai-dev` JSON candidates/manifests are archaeology, not accepted
Wave-8 input. They lacked immutable content identity and were mutated in place
during lifecycle operations. No compatibility reader silently promotes them.

The current Waves 8–12 implementation does not claim free-form policy
interpretation, YAML/Markdown support, generic RBAC/ABAC, SSO/account identity,
automatic refresh, cross-host Tenant isolation, distributed revoke or general
retention/privacy policy.
