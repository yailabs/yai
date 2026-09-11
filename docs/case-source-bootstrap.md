# Case source bootstrap

The source frontier acquires exact material, not source-grounded domain knowledge.
A source can declare `policy`, `knowledge` and `operational` roles together. Roles
grant no authority: policy publication and Case binding remain explicit.

## Product lifecycle

```text
declare perimeter → inventory (no source payload read)
    → acquire exact bootstrap policy files → inspect policy candidates
    → explicit publish/bind → EffectivePolicy READY
    → acquire permitted ordinary sources → inspect coverage / resume
```

`case sources declare CASE --file PERIMETER.json` accepts
`yai.source_perimeter.v1`: a name, a linked `participant`, optional existing
resource definitions and explicit `sources`. Each source declares `name`,
`resource`, `roles`, `action`, `media_type` and `bootstrap_policy`. Resource
definitions use the same contract as `case resource import`; there is no second
connector registry. The authenticated Tenant Owner must be linked to that exact
Participant. This bounded release supports at most 128 source declarations per
Case and a 64-KiB perimeter input, not an environment-wide crawler.

Supported source requests are `discover` over a confined local file/directory,
`database_query` over an existing bounded SQLite resource, and `http_fetch` over
an existing exact HTTP resource. These use existing resource envelopes. SQLite
and HTTP acquisition retains the exact bounded observation representation, not
an entire database or service. Other request families are rejected; this is not
generic repository/wiki/cloud/MCP crawling. A repository may be an explicitly
bounded filesystem subtree. Media type is a declaration, not inferred truth.

Before any Case policy has ever been bound, a declared `bootstrap_policy` source
may acquire **only its exact regular policy file** through the confined reader,
within the resource's byte envelope. No sibling scan, directory policy intake,
symlink traversal or ambient setup read is authorized. JSON, the existing
structured Markdown block and qualified text-PDF profile reuse the existing
policy parser. Unsupported or ambiguous material remains refused/needs
processing; no model interpretation or OCR runs. Revoking/unbinding policy does
not restore this initial setup authority.

`case sources acquire CASE` captures policy candidates first and leaves ordinary
sources pending until governance is READY. Inspect the returned artifact with
`policy show ARTIFACT` before explicitly calling:

```sh
./yai case sources publish CASE --source NAME --reason 'Reviewed exact policy candidate'
```

All declared bootstrap policy sources must be acquired before this publication
step. Existing catalog validation/publication and exact Case binding enforce
readiness and conflicts; a knowledge-only source cannot enter this route.
Non-bootstrap policy-role material is acquired as ordinary exact material; this
release does not automatically interpret or publish it. The standalone policy
authoring lifecycle remains available under its existing authority.

Then `case sources acquire CASE` or `case sources resume CASE` executes ordinary
Resource requests, Decisions and content admission/observations. DENY precedes
payload acquisition; REQUIRE_REVIEW pauses through the existing review owner.
General completion of reviewed acquisition through this source porcelain is not
qualified: review cannot be treated as a standing permission, and a request that
still requires review on current re-evaluation remains refused/paused. Existing
resource review execution is unchanged.
No model, hidden retry toward another target or alternative provider is used.
`--limit COUNT` bounds sources advanced per call. Completed sources are skipped;
unfinished work survives restart in canonical Case history. A retry of a failed
read must obtain current admission; it cannot resurrect a stale Operation.

Three modes share these commands: policy-only setup; policy plus an ordinary
perimeter; and incremental declarations in an already governed Case. The last
mode does not regain bootstrap authority.

## Inventory, revisions and recovery

`case sources inventory CASE` is a readable inventory; add `--json` for exact
source/revision/backing and Decision references. Counts describe **explicitly
declared sources only**, not a percentage of an unknown source environment.
`discovered` means declared for consideration, not payload acquisition. Acquiring,
acquired, denied, awaiting-review, inaccessible, needs-processing and revoked
remain distinct. Policy readiness is reported independently of partial ordinary
coverage. Acquired material has no derived knowledge status.

`case sources acquire CASE --source NAME --refresh` explicitly samples a new
revision through current authority. An unchanged revision reuses immutable
backing; changed bytes get a different revision identity. There is no continuous
watcher. Directory revisions are bounded sets of individually observed exact
files, **not an atomic filesystem snapshot**. Interruption can leave individual
admissions complete while the source revision remains incomplete; resume resolves
that canonical work rather than treating partial processing as policy authority.
The existing filesystem bounds remain in force (128 entries, 64-KiB aggregate
payload, depth 8); ordinary immutable content follows the existing non-empty
content profile. These are supported request bounds, not universal
source-environment sizes.

`case sources read CASE --source NAME [--revision REVISION]` resolves the exact
retained backing, with current policy and scope checks before returning data.
Missing/corrupt backing refuses; current live files never substitute for an old
revision. Binary originals retain their exact backing but are not printed as text.
Current policy may refuse a read even though acquisition succeeded historically.

`case sources revoke CASE --source NAME --reason REASON` revokes that source
frontier relationship permanently without deleting original bytes or rewriting
history. It is **not** PolicyArtifact revocation, a general content-erasure API,
or a claim that every future graph/Recall consumer has been integrated. Policy
revocation independently makes affected current source reads/acquisition refuse.
Metadata is restricted to the declaring linked Owner; it is not a cross-Participant
audit authority. Known historical source existence never grants current access.

## Ownership and identity

Transition v19 adds `CaseSourceDeclared` and `CaseSourceProgressed`; CaseState v16
replays their compact relations. There is no SourceStore or BootstrapJobStore.
The resource-access owner composes these relations with existing policy catalog,
content admission and observation owners. LMDB remains **37 databases of 40**.
Previous v18/v15 histories remain readable; older schemas cannot encode the new
source fields. Read-only inventory does not append Transitions.

A declaration binds Case, logical name/perimeter, Principal/Participant, exact
resource configuration, requested action, roles, media declaration and setup
posture. A revision hashes that source identity plus sorted path/digest/byte-count
material. Retried backing references do not invent a different content revision.
Progress binds its exact predecessor and attempt; stale progress refuses.
Coverage must close against canonical discovery/observation and exact admitted
backing, not a caller's assertion that acquisition completed.

Dual policy+knowledge setup uses the **same existing PolicySourceArtifact
original**, with no second byte copy for its knowledge role. Ordinary file bytes
use the existing immutable ConversationContentStore plus CaseContentAdmission;
operational metadata uses ResourceObservation. Identical bytes can share the
existing physical/content identity without collapsing logical provenance.
Policy originals may be reused across independently authorized Cases. Ordinary
content objects remain Case-bound; generalized cross-Case content reuse remains
an unimplemented pressure, not shared memory or shared permission.

This boundary does not implement M07 documentary claims, entities, wiki pages,
knowledge-aware Recall, Recall-aware W, model classification or YVEX state.
The [Roadmap](../ROADMAP.md) owns maturity; the cumulative
[ZERO-TO-CURRENT](zero-to-current.md) owns operator acceptance.
The [bounded qualification report](https://github.com/yailabs/yai/blob/c187648e9d9909d4d9b6711f131fa726d48cd585/refoundation/validation/case-source-bootstrap/REPORT.md)
retains product transcripts, adversarial contracts, cost and remaining gaps.
