# Governance source and PolicyArtifact reference

Authority: current Wave-8 governance authoring contracts. This reference does
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

`yai.policy_source_input.v1` is bounded UTF-8 JSON with no unknown top-level
fields:

```json
{
  "schema": "yai.policy_source_input.v1",
  "policy_key": "organization.example.filesystem",
  "source_version": "1",
  "owner_ref": "organization:example",
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

Limits are 256 KiB and 128 rules. Known rules are strict objects. Malformed
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

`yai.policy_source_artifact.v1` stores:

```text
source_id
content_digest
source_format = constrained_json
policy_key
source_version
owner_ref
bounded exact UTF-8 content
```

`source_id` is the SHA-256 digest of the exact source bytes. Exact duplicate
ingest is idempotent. Any byte edit produces a different source identity. Full
content is currently retained so the compiler input can be reconstructed;
operator inspection withholds it by default. A future privacy/retention policy
may separate payload retention, but cannot erase the content digest or the
provenance carried by retained artifacts.

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

`yai.policy_validator.v1` returns `qualified` only when at least one supported
normalized rule exists and there are no unresolved items or conflicts. A
blocked candidate remains inspectable. Validation never guesses precedence;
precedence and multi-source materialization belong to Wave 9.

## Immutable PolicyArtifact and lifecycle

`yai.policy_artifact.v1` binds:

```text
artifact_id
policy_key + artifact_version + owner_ref
source_id + source_digest
ParsedPolicy + parsed_digest
PolicyIr + IR digest
deterministic validation disposition
```

Artifact identity is content/provenance-derived. Stored artifact bytes are
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
artifact under the same `policy_key` supersedes the prior published artifact
atomically; both immutable versions remain inspectable. Retired and superseded
artifacts cannot be republished.

`runtime_consumable` is a derived view:

```text
validation == qualified AND lifecycle == published
```

It means only “eligible for later Case PolicyBinding/materialization.”

## Persistence authority

The existing LMDB environment hosts four governance databases:

- immutable policy sources by ID;
- immutable policy artifacts by ID;
- lifecycle events by ID;
- append order for lifecycle events.

This is a canonical governance history with an independent multi-Case future
lifecycle. It is not CaseState, a Case Transition stream, graph/memory derived
state, or `case:__system__`. A PolicyArtifact may be published with zero Cases.
Inspection/listing does not mutate either governance history or Case history.

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

`--as` is recorded provenance from a local command. Wave 8 does not prove OS
identity, enterprise authentication, remote signature or publish authority.
Those trust and authority gaps remain explicit.

## Compatibility and non-claims

Historical `yai-dev` JSON candidates/manifests are archaeology, not accepted
Wave-8 input. They lacked immutable content identity and were mutated in place
during lifecycle operations. No compatibility reader silently promotes them.

Wave 8 does not claim free-form policy interpretation, YAML/Markdown support,
Case binding, EffectivePolicy, precedence, authority resolution, review-policy
integration, Grant binding, expiry/revocation, tenant ownership or general
retention/privacy policy.
