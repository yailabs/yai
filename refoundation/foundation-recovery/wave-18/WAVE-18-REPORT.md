# Wave 18 report

State: semantic implementation and qualification published; evidence binding
complete in the follow-up publication commit.

Baseline: `8d6d9fe79f450e42f25324d4e80987e2873a1ae2` on `master`, equal to
`origin/master` and `refs/heads/master` before editing. H17 semantic baseline:
`2df9bb5c2bbd7efc53ed54527111522af329cf93`. Intended semantic commit:
`feat: add provider governance and safe failover`. Published semantic SHA:
`406a6b52c44e66c3506f63de5c7d10bb01a20c62`.

All 13 historical dirty entries remain preserved and excluded from the W18
whitelist. Their eight tracked paths retain checksum
`3fdb219654405e6fd40b5c0d1b02b94c04fadef5aa57a139aa5fb8fd6db7777e`.

## Archaeology and reference

Fresh `yai-dev` inspection covered provider registry/profile/routing,
knowledge capability selection, model selection records and their consumers at
`f8bad7fe3c443f056029780ac6a27a1979092715`,
`51a40e11f703cc7fe522df38a225a86c51ccc8c9`,
`46794683e9761041da7901f224c229c2de0e48a8`,
`e89ee42a6522f52caee1a52503aa67bbd515eff5`,
`94ba627091afbf1100ab386ac1de3d4fb1d2502c`,
`cffb318b980456f2671a297e14a6b05f5ac68320`, and
`2a4018147219044dfe1fad2268759b1f2a585945`. The useful properties were
Provider/Model separation, explicit requirements, candidate/exclusion sets,
selection lineage, locality, and visible fallback. Name-substring routing,
hard-coded catalogs, fake health, subjective capability flags, environmental
default authority, prompt heuristics, magic scores, and Agent routing remain
rejected. Exact file findings are in
`direct-yai-dev-provider-reinspection.tsv`.

The read-only YVEX `models1` study resolved
`5b3aa34be8999ad8240403e884074833d80c301d`. Its documented generic HTTP
surface supports `/health`, `/v1/models`, Chat Completions and JSON-object
probing; `yvex_completion_metrics` is operational telemetry. The UID-scoped
private Unix local protocol remains outside the core integration. No YVEX
source or runtime was modified or administered.

## Owner verdicts

- ProviderTarget, ProviderQualification and Trust share one Tenant-scoped
  provider-governance owner and named DB.
- Capability is evidence-derived and has no independent owner.
- ProviderHealth is a shared, fresh operational non-authority owner and named
  DB.
- CaseProviderBinding, ProviderSelection and ProviderAttemptOutcome are Case
  Transition facts; no global selection ledger exists.
- ProviderRequirement is pure derived input to selection.

This is the minimum lifecycle decomposition found by the owner tests. LMDB
grows from 35 to 37 named DBs, within the 40-DB environment contract.

## Implemented contract

`yai.provider_target.v1` is immutable and binds Tenant, adapter, normalized
endpoint, exact model and credential-reference identity. Endpoint validation
permits loopback HTTP, requires external HTTPS, and refuses credentials,
queries, fragments and ambiguous paths. Secret values are never persisted.

`yai.provider_qualification.v1` derives mechanical capabilities from a fixed
synthetic probe. The v1 vocabulary is chat text, structured JSON object, exact
model addressing, usage accounting, optional health probe and optional
first-party telemetry. It does not evaluate model quality. Requalification is
immutable; the current projection is monotonic by qualification time and ID.

Trust is an ordered Tenant-Owner event with unreviewed, approved and denied
postures. Health is orthogonal: unknown, healthy, degraded or unavailable plus
closed/open/half-open circuit. Observations expire after 60 seconds; three
failures open the circuit for 30 seconds; half-open recovery requires an
explicit probe.

`yai.case_provider_binding.v1` records one to 32 exact targets in explicit
order, failover `none` or `safe_only`, and a maximum of three attempts. Legacy
`ProviderAttached` history remains an exact pinned compatibility mode without
retroactive governance requirements.

Provider requirements come only from the invocation contract: normal
ModelWork requires qualified chat text; PlanPatch additionally requires
qualified structured JSON. Selection hard-filters Tenant/integrity,
credentials, trust, qualification, capabilities, health/circuit and failover
admissibility, then orders by health posture, binding order and target ID.
Stable exclusion codes explain every rejected candidate.

Selection, invocation and attempt outcome are linked to one logical semantic
turn and attempt number. Connect/zero-byte failures are `not_dispatched` and
may fail over under `safe_only`. Once request bytes may have left YAI, absent
truth is `delivery_indeterminate`; malformed successful response is
`response_invalid`; neither may automatically invoke another provider.
Provider/model changes clear continuation/KV optimization and rebuild the
ContextFrame from canonical Case state.

## Product and compatibility surface

Registry-backed PRODUCT operations are `provider add|list|show|probe|qualify`,
`provider trust approve|deny`, and `case provider bind|show`. Human and JSON
rendering use W16 typed output. `case show` exposes binding, candidate count,
last selection and last attempt posture without qualification internals.
`main.rs` remains 12 lines.

The compiled registry has 143 canonical operations: 72 PRODUCT, 9 ADVANCED,
45 PLUMBING, 16 COMPATIBILITY and 1 REMOVED; 9 aliases. Registry digest:
`sha256:d2b6c5d2987fb18eb283033fb624e98b588afa970e0d4dc9b3994acb7d4f7da2`.

Schemas advance to `yai.transition.v12` and `yai.case_state.v12`; readers for
v1-v11 remain admitted. ProviderTarget, Qualification, Trust, Health, Binding,
Requirement, Selection and Attempt schemas are v1.

## Non-claims and H18 delta

This wave does not provide remote TLS transport in the dependency-minimal
binary, generic cross-provider idempotency, quality routing, cost routing,
provider administration, YVEX model loading, YVEX private-protocol stability,
or provider-owned authority. H18 retains qualification corruption/replay,
credential rotation, endpoint/DNS drift, extreme trust/health/selection races,
unusual HTTP/TLS boundaries, idempotency attacks, extension/telemetry forgery,
long outage recovery and admitted-bound endurance. See
`remaining-h18-delta.tsv`.

## External qualification

`YAI_EXTERNAL_PROVIDER_BASE_URL` and `YAI_EXTERNAL_PROVIDER_MODEL` were both
absent. Therefore
`yvex_external_qualification_state=blocked_external_dependency`. This is a
deployment limitation, not a YVEX defect and not a fabricated pass. Local
generic fixtures remain the executable completion authority.
