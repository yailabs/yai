# Governance source and PolicyArtifact reference

Authority: current Wave-8/H8 governance authoring contracts. This reference does
not define Case policy evaluation or operational authority.

## Boundary

Wave 8 implements this supply chain:

```text
constrained source bytes
  → PolicySourceArtifact
  → ParsedPolicy facts
  → normalized PolicyIr
  → immutable PolicyArtifact candidate
  → validation
  → explicit publication
```

It does not implement:

```text
PolicyArtifact → Case PolicyBinding → EffectivePolicy → DecisionBasis
```

A published artifact is eligible to become a future materialization input. It
does not authorize an Operation, qualify a Case, produce a Decision, establish
review eligibility or issue an ExecutionGrant.

## Input grammar

`yai.policy_source_input.v2` is bounded UTF-8 JSON with no unknown top-level
fields:

```json
{
  "schema": "yai.policy_source_input.v2",
  "policy_key": "organization.example.filesystem",
  "source_version": "1",
  "owner_ref": "organization:example",
  "source_origin": {
    "source_system": "policy-intake",
    "source_uri": "internal://governance/filesystem"
  },
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

Wave 8 implements these parsed fact kinds:

| Kind | Typed semantic payload | Current consumer |
|---|---|---|
| `operation_restriction` | operation/resource selector, ALLOW or DENY posture, reason | Policy IR construction and conflict detection only |
| `review_requirement` | operation/resource selector, required boolean, reason | Policy IR construction and conflict detection only |
| `evidence_obligation` | operation/resource selector, pre/post observation, audit reason or source-provenance obligation | Policy IR construction only |

These types anticipate real current review/effect consumers without attaching
the artifact to them in Wave 8.

## Source and provenance

`yai.policy_source_artifact.v2` stores:

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

`yai.parsed_policy.v1` gives every typed fact a deterministic identity, source
artifact ref and JSON location such as `$.rules[0]`. `yai.policy_ir.v1` carries
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
blocked candidate remains inspectable. Validation never guesses precedence;
precedence and multi-source materialization belong to Wave 9.

## Immutable PolicyArtifact and lifecycle

`yai.policy_artifact.v2` binds:

```text
artifact_id
owner-scoped lineage = owner_ref + policy_key
artifact_version unique inside the lineage
declared source origin
source_id + source_digest
ParsedPolicy + parsed_digest
PolicyIr + IR digest
deterministic validation disposition
```

Artifact identity is content/provenance-derived. Identical bytes at the same
lineage/version are idempotent; changed bytes collide and fail before any
write. Different owners cannot supersede each other, and declared versions are
not interpreted as SemVer or sorted authority. Stored artifact bytes are
immutable. Lifecycle is a separate append-only
`yai.policy_lifecycle_event.v1` history with global LMDB order, prior/next
state, actor ref, reason, optional related artifact, time and integrity digest.

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

The shared LMDB map defaults to 256 MiB (formerly 16 MiB) and embedding callers
may configure it down to the documented 16 MiB minimum. The H8 supported
catalog contract covers 256 retained sources of up to 256 KiB under the
default. Capacity exhaustion is explicit and transactionally harmless.

## Operator trust and commands

The local CLI supports:

```text
yai policy ingest <source.json> --as <operator-ref>
yai policy inspect <source-id|artifact-id>
yai policy validate <artifact-id> --as <operator-ref>
yai policy publish <artifact-id> --as <operator-ref>
yai policy retire <artifact-id> --as <operator-ref> --reason <reason>
yai policy list
```

`--as` is recorded actor provenance from a local command and is distinct from
the artifact's declared `owner_ref`; it cannot alter lineage. H8 does not prove OS
identity, enterprise authentication, remote signature or publish authority.
Those trust and authority gaps remain explicit.

## Compatibility and non-claims

Historical `yai-dev` JSON candidates/manifests are archaeology, not accepted
Wave-8 input. They lacked immutable content identity and were mutated in place
during lifecycle operations. No compatibility reader silently promotes them.

Wave 8/H8 does not claim free-form policy interpretation, YAML/Markdown support,
Case binding, EffectivePolicy, precedence, authority resolution, review-policy
integration, Grant binding, expiry/revocation, tenant ownership or general
retention/privacy policy.
