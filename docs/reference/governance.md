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

## Case-bound model security

MODEL COMPLIANCE IS NEVER A SECURITY PREREQUISITE. Model/provider computation
is untrusted with respect to Case authority. A request is not authorization;
output is not Case truth; a proposal is not an admitted mutation; model state
is not authority. Policy projected into context informs computation but does
not enforce it. Enforcement remains outside the model even when the model
deliberately follows injected instructions.

The control plane is the authenticated Principal/Participant, Case, current
EffectivePolicy and admission, review/Grant, Workflow and Resource contracts.
The cognitive/data plane includes documents, D, Recall, tool output, model
plans/output and future computational E. Untrusted semantic content may
influence cognition, but cannot directly influence authorization. There is no
prompt filter, model-trust boolean or second policy engine in this boundary.

### Current mediated paths

| YAI-mediated path | Current enforcement owner / bounded responsibility |
|---|---|
| Native model tool proposal | Provider function-contract validation and canonical ProviderResult/Invocation lineage; `record_provider_capability_request` normalizes against the current CaseCapabilityView. Case/Participant/Resource identity comes from qualified owners, not model-added fields. |
| Filesystem read and discovery, immutable content admission/read | Current typed resource admission, exact Case-local binding and access contract, confined paths and bounded output; publication revalidates authority. Source acquisition composes these owners. |
| SQLite | Exact bound database and named read-only query/profile, not arbitrary SQL or a model-selected database. |
| HTTP | Exact admitted endpoint and named bounded GET; no arbitrary recipient, redirect or generic send privilege. |
| MCP | Admitted endpoint/catalog/tool and validated arguments; effect tools use current Decision/review/Grant and PREPARE fence. Arbitrary effects inside a remote server are not a YAI sandbox guarantee. |
| Process and filesystem effects | Existing named/confined runner or exact filesystem/signal carrier, current ALLOW, finite Grant, canonical PREPARE and dispatch fence. No arbitrary shell authority follows from a tool name. |
| Workflow/review/Grant and semantic admission | Existing authenticated transition/admission owners; model candidates cannot self-approve, issue Grants or publish policy. Valid admitted proposals remain possible; arbitrary model prose is not a Transition. |
| Reuse of completed Resource outcomes | `validate_resource_result_reuse_authorized` rechecks authenticated subject, current policy/roles/review and original Decision basis before the application returns cached observation/content/effect outcomes. Historical ALLOW is not present disclosure permission. |

The shared `controlled_effect::access` application path serves native product
consumers; it is not CLI formatting or a second retriever. Existing typed
CaseCapabilityView, Operation, DecisionBasis and ResourceActionOutcome provide
requestability/decision evidence. No new security-envelope schema, canonical
owner, database or long-lived capability lease is introduced.

Reusing a result does not repeat an external effect. A changed/revoked policy
basis refuses reuse, even at the same Case generation; re-evaluating current
roles/review prevents an unchanged policy ID from standing in for authority.
The check appends no Transition or Grant. Fresh reads use their pre-dispatch
admission and publication checks; effects retain the established PREPARE and
dispatch authority cuts. This is not continuous locking of arbitrary remote
systems through a potentially long external call. Prepared uncertainty retains
its existing reconciliation semantics.

The original model-security boundary excluded legacy carrier-internal
pre-PREPARE observations. The current Case-bound Resource observation contract
below closes that exclusion for the native filesystem-write and process-signal
paths, including explicit recovery reads. It does not turn every host I/O or
operator compatibility diagnostic into a governed Case operation.

### Case-bound Resource observation

No protected native Case carrier observation precedes its current authority
cut. `LmdbRecordStore::observe_filesystem_authorized` and
`observe_process_authorized` are typed application operations, not CLI rendering.
They authenticate the current caller, resolve exact canonical Operation,
Decision and issued/prepared Grant, and reuse existing policy/review admission.
Current roles, current Ready/Valid EffectivePolicy identity/digest, Grant expiry,
Case cancellation and Resource scope are requalified. Knowing an old Grant or
an attachment ID is not authorization. The caller supplies no substitute path,
PID, Participant, Resource object or local binding.
For deterministic Workflow operations the existing canonical proposal/assignment
and the Principal on the committed Operation identify the executor; this is not
provider impersonation or a blanket Tenant-owner exception. A different Tenant
member cannot reuse that assignment, and hidden/absent operation references have
the same refusal before host observation.

`qualify_carrier_observation_txn` composes these existing owners in one bounded
authorization transaction. Only after it succeeds is the persisted exact local
binding passed to the host observation primitive. No Transition, new Grant or
security database is produced by this read; the existing authority-time floor
may advance. No Recall, W or retrieval is involved.

| Read site / purpose | Qualification and physical boundary |
|---|---|
| Filesystem pre-PREPARE digest/size/type/existence | Authenticated application operation; canonical relative target and admitted root; descriptor-relative confinement. Planning/precondition evidence is still a protected read. |
| Process pre-PREPARE `/proc` state and birth identity | Same current-authority operation, exact persisted process binding; no caller PID substitution. |
| Filesystem carrier re-observation before replacement | Existing `ResourceFenceAuthority` implementation now checks current policy/Grant/admission as well as live fence ownership before inspection, and again at the final mutation fence. Root identity must match the fence before host I/O. |
| Process carrier re-observation before signal | Fence/current authority now precede the `/proc` read, not only `kill`; exact process-birth target must match the fence. The final signal fence remains separate. |
| Explicit reconciliation and uncertain process recovery | Native consumers use the authorized observation operations. Denial leaves PREPARE/indeterminate uncertainty intact; it cannot manufacture a terminal result or justify redispatch. |
| Post-dispatch observations/receipts | Bounded completion of the just-authorized carrier attempt. Existing receipt, terminal publication and lease settlement retain their separate owners; a recorded external effect is not erased by later policy contraction. |
| Setup and infrastructure metadata | Explicit attachment setup already authenticates the Tenant owner before root/PID capture. Reading YAI's own process birth identity to validate a fence owner is infrastructure bookkeeping, not observation of a requested protected Resource. Resource identity hashing itself performs no host read. |

The named confined `ProcessRun` carrier already qualifies current Resource
dispatch before opening/hashing its executable in `BoundedProcess::prepare`,
then revalidates before spawn. It is not the legacy process-signal gap.
Low-level Rust observation helpers remain host primitives for carriers and
component fixtures; the native Case application no longer calls them directly.
C component carriers and the operator-only `carrier fs-read` compatibility
diagnostic are not model-offered Case Resource operations and do not acquire a
new Case-security guarantee from this change.

This is a temporal cut, not a continuous authority/host lock. Authority may
change after qualification; the later PREPARE and final dispatch cuts remain
independent. A failed pre-observation read appends no canonical mutation.
PREPARE may separately record the existing legitimate Grant invalidation.
A denied fresh inspection after PREPARE does not erase external uncertainty:
existing receipts can still settle already-observed effects, while a new
recovery read needs current authorization. Expired/revoked authority receives
no permanent recovery lease. Host contents can still change between checks;
descriptor confinement, exact process birth identity and precondition/fence
checks retain their bounded TOCTOU responsibilities, not a global atomicity claim.

Executable evidence extends the existing tests in `store/lmdb.rs`:
`carrier_observation_current_authority_precedes_host_io`,
`wave14_process_signal_uses_same_authority_spine_and_exact_birth_fence` and
`h10_review_writes_rederive_roles_provenance_and_final_decision`.
Thread-local test counters at both host observation entrypoints prove zero
protected observations for refused calls. Positive reads, approved review,
policy scope contraction, same-generation revoke, cross-Case/absent identity,
binding substitution, restart and refusal before PREPARE/dispatch are covered.
Participant role re-evaluation is additionally tested against a contracted
current snapshot; no durable role-removal API is claimed from the additive
ParticipantBound lifecycle. The durable scope-loss oracle uses real policy
publication/replacement. Timings separate current-authority qualification from
host I/O and make no constant-history-cost claim.

Observation archaeology rechecked `yai-dev` at `5c1c7b9d0`, its
`dda93ee3a` runtime admission/dispatch relocation and adjacent carrier/control
guards (`check-runtime-control-admission-hook.py`,
`check-ipc-dispatch-control-spine.py`). Those prove bounded call-context
admission/deferred mutation, not an exact policy-current host-read mechanism.
Current Rust carrier history (`0b48ede`, `f6c7c8b`) supplies the stronger
Grant/PREPARE/shared-resource fence algorithms preserved here. Recover the
fail-closed-before-dispatch property in these owners, not old planes, registries
or runtime loops. No historical tree or separate recovery ledger is restored.

Authorized input access cannot authorize an unrelated sink. External publication,
send/write/network effects require their own admitted target and operation.
Returning output to its authorized caller is not automatic declassification for
broader recipients. Credentials/privileged handles remain brokered outside
model-visible tool arguments where the existing adapters own them.

### Evidence and limits

`make smoke-case-capability-realization` drives the actual ConversationController
against a deterministic noncompliant loopback peer. An admitted Markdown source
derives as `source_stated`, then reaches the peer through a governed read. The
peer follows its instructions and attempts protected reads, target/path escape,
identity substitution, unoffered writes/process/network calls, self-grant and
review bypass. Normalization/admission must refuse while the legitimate confined
read still succeeds. Policy/grants/effects are not widened, protected bytes are
not observed, and same-generation policy revoke prevents cached read reuse.
The resource-access engine tests additionally exercise authenticated reuse,
restart, foreign identities and canonical-history invariance. Existing admission,
temporal authority, resource-adapter and Golden local suites remain the evidence
for review/Grant/fence and Workflow behavior; the local fixture is not live-model
qualification or universal prompt-injection detection.

Qualification is reproducible through the native command above and
`CARGO_TARGET_DIR=target cargo test --manifest-path engine/Cargo.toml -p yai-engine resource_access_tests:: -- --nocapture --test-threads=1`.
The latter includes source-revoke, hidden/unknown equality, exact cross-Case
candidate refusal and restart tests. `audit_reason_requires_real_review_action_evidence`
also proves historical reviewed disclosure can remain allowed while a new effect
under a non-current operation is refused; losing reviewer eligibility refuses both.
`wave11_revoked_review_is_durably_invalidated_and_cannot_approve` and
`wave11_grant_expiry_before_prepare_is_terminal_and_effect_free` retain the
existing review/Grant temporal oracles.

The tests print separate reuse-gate timing and full native attack timings. Cost
includes current policy qualification and canonical history/evidence resolution;
there is no Recall/W compilation or provider request inside the reuse gate, and
no claim of constant Case-age cost. The native adversarial corpus is bounded to
nine distinct attempts; a larger cumulative fixture encountered a transport
response limitation before its final attack and is not claimed as a security
PASS. That limitation was not repaired by changing transport or relaxing security.

Legacy archaeology: `yai-dev` at `5c1c7b9d0` retains the C runtime-control
admission hook and its guard under `src/runtime/decision/` and `tools/checks/runtime/`;
the adjacent decision mediation/capability runtime context did not provide a
stronger qualified effect monitor. Recover the fail-closed control-versus-data
property in current Rust admission, not the old planes/registries or stub context.
Current executable Rust owners and tests, not the historical hook's existence,
establish this boundary.

Four rings remain distinct:

| Ring | Owner and claim |
|---|---|
| Semantic/authority security | YAI: current truth, disclosure and admission. |
| Case reference-monitor/capability security | YAI: bounded mediated Resource reads/effects and result reuse. |
| Computational isolation | YVEX/provider/process/container/runtime: ambient capabilities of model computation; not established by this YAI wave. |
| Infrastructure isolation | Host/kernel/filesystem/network/secrets/GPU/external systems; not established by this YAI wave. |

A model process with direct host filesystem/network/credential access outside
YAI mediation is **not contained by this guarantee**. Future provider/runtime
capability negotiation must describe actual isolation posture and exact
model/artifact/composition/deployment/runtime/configuration identity; an alias
is not evidence of trust. No such new capability protocol is implemented here.

### Future computational state and mixed sources

Future E is untrusted computation: it may be stale, poisoned, wrong or cross-Case.
State Update cannot mutate D/H/S/Policy/authority. Revoked W material requires
qualified removal/Reconcile, or E invalidation and rebuild when selective removal
is unqualified; never assume latent forgetting. Cross-Case E sharing is forbidden
by default absent an explicitly qualified sharing contract. E/B1 remains YVEX
research, not a YAI format or implementation.

Physical source identity/role is not the semantic authority of every content
unit. Mixed normative and documentary content remains future finer-grained
routing pressure inside the unified frontier. Policy-like units require candidate
validation/publication/binding; other units remain knowledge/evidence. Neither a
whole-document governance role nor a model classification elevates every unit.
The present strict policy grammar continues to refuse unresolved mixed prose;
this paragraph does not claim that generic mixed-source routing is implemented.

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

### Document representations and interpretation boundary

`./yai policy extract FILE` is bounded read-only inspection, not publication.
The REPLAI workbench exposes the same seam as `/policy extract FILE`. Guided
`/policy publish` shows the extraction and validation before explicit consent;
publication rechecks the inspected original digest so file drift cannot silently
substitute another object. No raw Transition construction is required.

All supported forms contain the same strict v4 JSON language above:

| Representation | Supported profile | Exact source location |
|---|---|---|
| JSON | Existing bounded strict JSON | Existing `$.rules[n]` field location |
| Markdown | Exactly one fenced `yai-policy-json` block | JSON location plus block line range; other nonblank lines remain unresolved |
| PDF | Text-only policy sheet with `YAI-POLICY-JSON-BEGIN` / `YAI-POLICY-JSON-END` markers | JSON location plus page/extracted-line block range, not invented glyph coordinates |

Original bytes are limited to 256 KiB. The PDF adapter uses pinned lopdf 0.44.0,
strict parsing, bounded stream/text extraction, at most 32 pages/2,048 objects
and a conservative lexical nesting bound. Encrypted, annotation-bearing,
non-text drawing/form/image profiles, blank extracted pages and malformed input
refuse; there is no OCR. This is not general enterprise PDF interpretation.
Extraction order is the parser's text order, not a claim of visual layout fidelity.

Document sources use `yai.policy_source_artifact.v5` with original bytes, media
type, extractor identity and block location. JSON sources remain v4. Revalidation
re-extracts the original document and reconstructs parsed facts and unresolved
items; the catalog must not recompile only the extracted JSON and lose identity.
Changing bytes at the same policy lineage/version is a conflict, not an update.

Prose without a block is inspectable but cannot be ingested as authoritative
policy. A block plus other prose retains unresolved items and cannot qualify.
Ambiguity must be resolved by producing an explicit reviewed structured source
and going through normal validation/publication/binding. No heuristic, confidence
score, model interpretation or instruction embedded in a document creates rules.
The original source remains data; cognition receives normalized EffectivePolicy
semantics, not an elevated copy of arbitrary source instructions.

### Two consumers of the same normative source

EffectivePolicy feeds both deterministic admission and the mandatory scoped
working-state authority view. The latter reports readiness/validity, exact policy
identity and normalized restrictions/review/role/evidence rules for visible
resource kinds. It grants nothing. Each Operation still receives a current
DecisionBasis and the existing ALLOW / DENY / REQUIRE_REVIEW lifecycle.
Catalog revocation invalidates old W even without a Case-generation increment;
mandatory authority material cannot be silently removed to satisfy a budget.
Recent Decision evidence remains history under the policy that applied then.

### Original structured source contract

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
