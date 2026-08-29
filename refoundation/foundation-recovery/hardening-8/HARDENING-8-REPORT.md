# Foundation Hardening 8

State: complete in the containing published H8 commit. The exact publication
SHA is reported with that commit because a commit cannot embed its own hash.
Baseline: `7ce67afee34a3dbe879c2e5bee945602492be70c`.

This package is navigation/evidence, never authority. Source and executable
tests outrank it.

## Direct legacy reinspection

H8 returned directly to `yai-dev` and inspected the named March–May epochs,
not the Wave-8 ledger. The strongest executable intake remained the
`45c36bd0c` authoring pipeline plus the `575f76fcd` canonical-loader cutover.
It preserved source → parsed → normalized → candidate stages, explicit
`runtime_consumable`/apply eligibility, conflicts, provenance and review
posture. It also used mutable files, path refs, broad registries and weak
loader parsing.

Newly explicit in H8: legacy source manifests separated `source_system` and
`source_uri` from organization ownership; candidate provenance referred to a
source path; supersession refs lived in mutable review metadata without a
lineage integrity mechanism. The later `885ddac44`/`c3f599890` forms connected
`runtime_consumable` to attachability/runtime metadata but expanded the
registry/manifest forest. `1ebfb8d84` neutralized examples;
`c5501101f`/`2eeb8ea22`/`3cb2e94a4` consolidated and removed duplicate law and
protocol roots; `a22476726`, `526c824bd`, `7b3c1b7bc`, `2a4018147`, and
`5c1c7b9d` moved/drained the topology rather than producing a stronger
immutable intake catalog.

## Refounded hardening

- `PolicyLineage` is exactly `(owner_ref, policy_key)`. Its deterministic ID
  is an index key, not a Tenant or authority object.
- A lineage/declared-version tuple admits one immutable artifact content.
  Exact repeat is idempotent; changed bytes return
  `policy_version_identity_collision` before any write.
- `yai.policy_source_input.v2`, `yai.policy_source_artifact.v2`, and
  `yai.policy_artifact.v2` add bounded, digest-covered declared
  `source_system`/`source_uri`; v1 remains read-compatible with explicit origin
  absence. Local import paths are not persisted.
- Strict JSON rejects duplicate keys at every object depth, UTF-8 BOM, invalid
  UTF-8, more than 32 JSON levels, unknown fields on known contracts and
  non-ASCII authoritative identifiers.
- Artifact validation rebuilds Policy IR from parsed facts and re-derives the
  complete validator disposition. Stored booleans/blockers are not authority.
- Supersession is same-lineage only and the complete old-supersede/new-publish
  sequence is one LMDB transaction. One LMDB writer serializes concurrent
  same-lineage publication; tests prove exactly one current artifact.
- `policy_current_by_lineage` is a transactionally maintained, rebuildable
  accelerator. Immutable artifacts plus lifecycle events remain canonical.
- LMDB default map size is 256 MiB, configurable by the embedding caller with
  a 16 MiB supported minimum. Capacity failure is explicit.

## Source origin verdict

Origin is recovered now because future audit and Case binding need to
distinguish bytes from claimed source. It is declared provenance, not verified
owner or actor identity. Origin is part of the exact source document; changing
it changes content identity. Repeated intake of identical bytes is idempotent
and does not create a new intake-observation event. No current consumer earns
such an event. Sensitive values are bounded to 120/512 bytes and local/file
paths are rejected.

## Atomicity, concurrency and integrity

Tests abort after source put, after artifact put/before candidate event, after
validation event, after old supersession, after new publication and after
retirement. Reopen/view observes only the pre-transaction state. Corrupt
source/artifact/event JSON, digests, sequence refs, future schemas and dangling
events fail closed. Repeated reads do not append events or Case Transitions.

## Capacity and scale

The max-source contract test stored 256 sources containing 62,985,252 bytes;
`data.mdb` reached 65,175,552 bytes under the 268,435,456-byte default. A
16,777,216-byte map accepted 65 such artifacts and then returned explicit
catalog-capacity exhaustion with no partial commit.

A separate debug characterization used 33 lineages, 40 artifacts and 127
lifecycle events: 25,664 source bytes, 536,576-byte LMDB file, 1,192 ms total
ingest/validate/publish, 251 ms full list, and 5 ms one-artifact inspect. These
are non-release, single-run order-of-growth observations, not performance
claims. The exact Wave-9 lookup is indexed; a list index is deferred until a
real catalog workload demonstrates need.

## Authority boundary

H8 creates no Case PolicyBinding, EffectivePolicy, Case Transition, Decision,
ReviewRequest, Grant, PREPARE, carrier effect, provider or model invocation.
`actor_ref` is claimed local lifecycle provenance and is never substituted for
artifact `owner_ref`.

## Source ownership and validation

No new semantic source owner was added. `governance.rs` remains the compiler
contract, `store/lmdb.rs` the shared persistence/transaction boundary, and
`policy.rs` the CLI renderer/dispatcher. Product C/Rust source-file count stays
138 → 138 and `main.rs` stays 1,926 → 1,926 lines. The new shell
characterization is test ownership, not product semantics.

Validation evidence: `make check` and `make characterization` green; 79 Rust
engine tests green; governance intake and H8 characterization green;
exact-byte/query/authority boundaries green; 256-source capacity and
40-artifact scale characterizations green; both Cargo format checks and
`git diff --check` green. Repository publication verification is reported with
the final pushed SHA.

## Recovery classification

During hardening, governance intake and PolicyArtifact lifecycle were
`partially_refounded`. After the direct reinspection and executable H8 proofs,
both are promoted to `refounded_proven`, reopenable when policy binding,
authority, tenant identity or retention adds adjacent evidence. Shared catalog
ownership remains `partially_refounded` because authenticated organization/
tenant isolation is Wave 12.

## Exact Wave-9 prerequisites

Wave 9 must freshly reinspect `yai-dev` and add only: immutable Case
PolicyBinding refs, deterministic multi-artifact EffectivePolicy
materialization, normative readiness, applicability/precedence/conflict/
missingness, and invalidation inputs required by those consumers. It must use
owner-scoped published artifact lookup, preserve historical versions, and must
not turn `runtime_consumable` into authority. H8 does not begin that work.
