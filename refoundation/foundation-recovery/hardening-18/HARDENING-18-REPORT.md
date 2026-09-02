# Foundation Hardening 18 report

State: H18 was published in semantic commit `731a229c` and evidence commit
`af98a2f0`. A post-publication legacy-health correction and its full repository
qualification are complete; publication of that isolated follow-up remains
pending at the time this report revision is committed.

Authored W18 final baseline: `4b41b81440becace100748b229a0e84bc63c2c7c`.
Actual clean published H18 baseline:
`47986c56c6cd71b91a97efe29b31905ebf93ad41`. The one-commit delta is
`docs: consolidate agent guidance and research traceability`; direct diff
inspection found documentation, research traces, header comments, notebook
metadata, root `AGENTS.md`, and deletion of `.agents/AGENTS.md`, with no W18
runtime semantic change.

Intended commit: `harden: close provider trust health and failover boundaries`.
This report does not predict the SHA of the commit containing itself.

Post-publication follow-up baseline:
`af98a2f0d2451ec928da75fab6a07a69cf59ebb9`. Direct review found that the v1
compatibility reader validated the historical unsealed shape but then returned
its `Healthy` posture and circuit timestamp as current operational input. The
follow-up degrades v1 posture to sealed v2 `Unknown`; it retains only the
conservative fact that a circuit was `Open`, with a new cooldown anchored at
the store-owned time floor. Intended follow-up commit:
`fix: fail closed on legacy provider health`.

## Archaeology

Fresh inspection covered the provider registry, profiles, routing,
admissibility, capability projection, selection and LAN transport at commits
`f8bad7fe`, `51a40e11`, `46794683`, `e89ee42`, `94ba627`, `cffb318`,
`2a401814`, and the adjacent LAN commit `95bffa0`. Useful properties were the
provider/model distinction, explicit candidate/exclusion intent, bounded
framing and timeouts, and explicit fallback events. Rejected mechanisms were
name/prompt heuristics, environment defaults, subjective capability flags,
fake readiness, persisted bearer material, discovery-created trust, and the
legacy LAN TLS path, which verified a CA but omitted hostname verification and
SNI.

## Owner recheck

- ProviderTarget, Qualification, Trust and non-secret credential revisions
  remain one Tenant-scoped provider-governance owner family.
- Capabilities remain derived from evidence; there is no capability owner.
- Health remains shared operational, Tenant/target-exact and non-authoritative.
- Case binding, selection and attempts remain Case-canonical Transition facts.
- No ProviderManager, RouterManager, CircuitManager, CredentialManager or
  SelectionLedger was introduced.

## Qualification and capability provenance

Immutable qualification history is scanned and validated to derive current
posture. `qualification-current` is a discardable cache and is never read as
authority. Ordering is deterministic by qualification time then exact identity;
credential revision is an additional exact eligibility boundary. Expiry is
exclusive at `now == valid_until`, and the persisted authority-time floor
prevents rollback resurrection. Future caller time is bounded and historical
corruption fails closed.

New qualification bytes use `yai.provider_qualification.v2`. Historical v1 is
validated with its original digest and revision zero. The v2 capability is
`ExtensionCompatibleTelemetry` with `ExtensionObserved` provenance. Extension
evidence may satisfy only its exact telemetry family; it cannot outrank or
cross-promote into qualified chat, JSON, exact addressing, usage or health.

## Trust and credentials

Trust current posture is rebuilt from a contiguous immutable sequence.
Missing, duplicate, corrupt or cross-target sequence material fails closed.
The 64-contender test produced one sequence per committed event with no fork.

`yai.provider_credential_revision.v1` records an owner-authorized monotonic
revision and bounded operator label, never the secret or a secret hash.
Rotation preserves immutable target and trust, invalidates credential-dependent
current qualification, and resets health to Unknown. Requalification binds the
new revision. Changing the reference itself still creates a new ProviderTarget.

## Health and half-open admission

`yai.provider_health.v2` binds an integrity digest, effective-time floor and
exact probe owner. A direct forged current posture therefore fails on read.
Historical unsealed v1 health remains readable but cannot promote a target to
Healthy: it projects to sealed v2 Unknown, while an existing Open circuit is
retained with a fresh cooldown anchored at the store-owned authority-time floor;
its unsealed timestamp can neither shorten nor indefinitely extend cooldown.
Healthy freshness and circuit cooldown use the monotonic persisted floor, so
wall-clock rollback cannot resurrect stale health or move a circuit backward.
Observation time is store-derived. Half-open admission is one cross-process
owner identified by boot ID, PID, `/proc` start time and token. A live exact
owner excludes contenders; a dead owner is reclaimable. Success closes and
resets; failure reopens. Twenty outage/recovery cycles retained no owner or
worker leak.

## Endpoint, DNS and TLS

Every governed connection resolves again and rejects the complete answer set
if any address violates declared locality. IPv4, IPv6, IPv4-mapped IPv6,
RFC1918, ULA, link-local, unspecified and multicast classes are explicit.

Remote HTTPS is implemented with rustls and WebPKI roots. Certificate chain,
hostname and SNI are validated; TLS 1.2/1.3 are the library defaults. There is
no product skip-verification switch or plaintext downgrade. DNS/TLS failures
before application HTTP bytes are `not_dispatched`. Once plaintext may have
entered TLS record production, failure is conservatively
`delivery_indeterminate`. Redirects are refused, so credentials are never
forwarded to another authority.

## Selector, delivery and failover

Known historical selector v1 has its own validator. Unknown future selector
versions fail closed; a future selector may affect only new unresolved turns.
Sixty-four contenders for one logical attempt produced one Case selection.

Application HTTP bytes, not DNS/TCP/TLS bytes, define delivery. Strict bounded
HTTP rejects duplicate Content-Length, unsupported transfer framing, partial
headers/body, oversized data, invalid UTF-8 and trailing bytes. Generic 429,
5xx, malformed success and response-schema failure never prove non-execution.
Provider-supplied request IDs or idempotency strings never create same- or
cross-target exactly-once authority. `safe_only` retries only mechanically
`not_dispatched` attempts.

## Extension and YVEX

The YVEX `models1` branch resolved to
`1f7ff1cd11ab8aec0976a9c8b0ee88ac5c73f010`. Its OpenAI profile v2 documents
loopback `/health`, `/v1/models`, `/v1/chat/completions`, JSON-object validation
and `yvex_completion_metrics`; local protocol v18 remains private, UID-scoped
and explicitly not a public remote or stable SDK contract. No HTTP authenticity
mechanism was found. Accordingly `yvex.http.v1` proves extension-contract
compatibility only, never YVEX binary identity, Trust, model quality, Decision
or Effect truth. The YVEX repository was not modified.

Live qualification state is `blocked_external_dependency`: both
`YAI_EXTERNAL_PROVIDER_BASE_URL` and `YAI_EXTERNAL_PROVIDER_MODEL` were absent.

## Compatibility, footprint and scale

Transition/CaseState remain v12. ProviderTarget, Trust, Binding, Requirement,
Selection and Attempt remain v1. Qualification and Health advance to v2;
CredentialRevision is v1. Legacy ProviderAttached exact-pin behavior is
unchanged and gains neither governance nor failover.

LMDB remains 37/40: one provider-governance DB, one provider-runtime-health
DB, no new DB. Semantic owners are unchanged; operational owners are
unchanged. `cmd/yai/src/main.rs` remains 12 lines.

Observed full-bound characterization: 128 Tenant targets; target 129 refused;
32 Case candidates; candidate 33 refused; 1,000 deterministic canonical
selections; 256 immutable qualification runs retained; 20 outage/recovery
cycles. The final focused 1,000-selection run reported 32,929 us minimum,
134,253 us maximum, 83,285 us mean and 15,310,848 LMDB bytes on this
development host.
These numbers are characterization, not a performance claim.

## Foundation Recovery classification

- ProviderTarget: `refounded_proven + integrity/network-boundary-qualified`.
- Qualification: `evidence-bound + replay/time-qualified`.
- Capability: `derived + provenance-qualified`.
- Trust: `Tenant-governed + concurrency-qualified`.
- Health: `operational_non_authority + rollback/circuit-qualified`.
- Selection: `Case-canonical + concurrency/upgrade-qualified`.
- Failover: `Case-canonical + delivery-boundary-qualified`.
- Provider extension: `optional/non-authority + spoofing-qualified`.

## Post-H18 boundary

No new numbered Foundation Wave is declared. The evidence-based reassessment is
in `post-h18-roadmap-reassessment.md`.
