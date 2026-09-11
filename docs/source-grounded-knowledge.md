# Source-grounded domain knowledge

`D` answers **what an admitted source states**, not what is verified, observed
operationally or currently authoritative. The bounded deterministic implementation
consumes [Case source bootstrap](case-source-bootstrap.md) revisions. It does not
acquire sources, publish policy, append knowledge facts to history, call a model,
change Recall or inject material into Working State.

## Product and typed boundary

```sh
./yai case knowledge build CASE
./yai case knowledge inspect CASE --json
./yai case knowledge search CASE 'billing retention'
./yai case knowledge graph CASE
./yai case knowledge wiki CASE
./yai case knowledge resolve CASE KNOWLEDGE_UNIT_ID --json
./yai case knowledge inspect CASE --source NAME --revision SOURCE_REVISION_ID --json
```

`build` and `inspect` both derive a fresh read-only view; build does not create a
canonical object or persistent cache. `--source` restricts an exact logical name
or source ID; `--revision` requires that selector. `--limit` bounds semantic units,
not source permissions, and refuses overflow rather than silently truncating
claims. Search returns at most 32 candidate hits with BM25 scores and matched
terms; structured output includes their qualified source-addressed view.

`LmdbRecordStore::case_knowledge_authorized` supplies one authorized read snapshot
to `memory_hierarchy::knowledge`. `KnowledgeRequest`, `KnowledgeResult`,
`KnowledgeView`, units and locations are typed Rust results below CLI formatting.
The CLI only parses arguments, calls these contracts and renders them. The
application boundary remains PARTIAL overall; no complete public SDK/API, Studio
or interfaces integration is implied.

The derived result is `yai.source_knowledge.v1`, profile
`yai.source_knowledge.deterministic.v1`. Its identity includes Case/requester,
visible source revisions and applicability, units, graph/conflict meaning,
profile and unit budget. Timings are separate from identity. Hidden/unacquired
sources and private whole-history hashes do not enter the view identity or
corpus statistics. Equivalent rebuilds reproduce the view; changing exact source
material, current visible applicability or backing availability changes it.

## Qualified inputs and coordinates

| Source family | Bounded extraction and coordinates | Not claimed |
|---|---|---|
| UTF-8 text / Markdown | Source document, line-addressed blocks; Markdown ATX heading hierarchy/topics and explicit structured blocks below | Arbitrary prose interpretation, general NER, complete CommonMark AST/table semantics; code comments are not headings |
| JSON | Duplicate-key rejecting parser; object/array structure and exact JSON Pointers to scalar documentary values | Permissive repair, inferred schema meaning or truth |
| Text-bearing PDF | Existing bounded text-only `lopdf 0.44.0` profile; page plus extracted-line ranges and structured-block pointers | Original glyph/byte coordinates, OCR, image understanding, annotations or arbitrary PDF content |
| Filesystem / repository-like tree | Previously acquired bounded files with original paths; per-file document structure and text/code lines | Git connectors, code AST/module/symbol resolution, automatic full-tree reacquisition |
| SQLite result/metadata | Exact retained ResourceObservation JSON; structured columns/rows. The admitted query `SELECT name, sql FROM sqlite_master WHERE type='table' ORDER BY name` additionally qualifies table/schema-description units | Live DB introspection during derivation, SQL DDL parsing, all database drivers or arbitrary query aliases interpreted as schema truth |

SQLite table identities use `urn:yai:sqlite:RESOURCE_ID:PERCENT_ENCODED_TABLE_NAME`.
Other query results remain captured observations, not invented table definitions.
Their recorded posture and exact Observation reference remain visible.

Media declaration and supported path suffix choose a deterministic profile.
Malformed/duplicate JSON, unclosed structured blocks, invalid/empty/image-bearing
PDF and exceeded parser bounds produce `needs_processing`; unsupported formats
such as Office documents produce `unsupported`. Exact original backing remains
identified even when no units can be derived. No OCR/model fallback runs.

PDF extraction is shared with the existing governance document adapter, but
policy block interpretation/compilation remains in governance. Knowledge parsing
never calls the Policy IR compiler or publication/binding operations.

## Explicit documentary semantics

Plain text remains source-stated text. A transparent structured baseline can
declare exact entities, topics, claims and references in JSON:

```json
{
  "id": "urn:domain:billing",
  "topics": ["billing"],
  "claims": {"retention_days": 90},
  "references": ["urn:domain:auth", "resource:database"]
}
```

Objects can be nested in an `entities` array. Markdown uses an explicit
`yai-knowledge-json` fenced block; qualified PDF text uses
`YAI-KNOWLEDGE-JSON-BEGIN` / `YAI-KNOWLEDGE-JSON-END` on one page. Existing policy
JSON block markers are also readable as documentary data, without policy
interpretation. JSON Pointer locations inside blocks retain their containing
line/page span. Unrecognized prose is preserved, not silently interpreted.

Entity association requires an explicit `urn:` or `entity:` identifier, not a
similar name, heading or rank. Topic labels/headings aid navigation only.
Documentary scalar values preserve their exact JSON value and pointer; direct
`claims` properties under an explicit entity permit cross-source comparison.
Same explicit entity/property with different values yields an **unresolved
source-value disagreement**, not a winner by recency. This is a bounded structural
comparison, not general logical contradiction detection or claim verification.

The epistemic classes are `source_stated`, `deterministic_structure` and
`recorded_observation`. A documentary SQL/schema claim may disagree with the
retained SQLite observation; neither is relabelled as the other or promoted to
current `S`. The layer never changes existing W20 assertions or their mechanical
supersession rules.

The graph owner derives typed containment, explicit entity definition/reference,
source support, exact admitted Resource linkage and unresolved disagreement edges.
Every edge closes over selected units/backing. Similarity and chronological
proximity produce no factual or causal edge. A reference to an unqualified target
remains source text without an edge revealing whether the target is hidden or
absent. Graph access remains disposable; no parallel canonical graph is created.

One dual-role policy+knowledge revision reuses its exact PolicySourceArtifact
original. Its documentary values say what that source contains; EffectivePolicy
independently governs current operations. Building/searching/wiki rendering
neither republishes nor rebinds it. Imported instruction-like text remains data.

## Source closure, freshness and reuse

Current source access follows the bounded bootstrap contract: authenticated
Tenant Owner, exact declaring Principal linked to the source Participant,
knowledge role, acquired/non-revoked source, current READY EffectivePolicy and
per-content current read admission. Qualification happens before payload reads,
graph construction, counts and ranking. Hidden and unavailable exact selectors
share a refusal. Current catalog policy revocation is checked even without a Case
generation change. Historical source permission never authorizes current access.

Original files may remain at their ordinary paths; derivation never rereads them
independently. This release resolves the bootstrap's **already retained bounded
snapshots/observations**, not a new snapshotless in-place lifecycle. It does not
copy an entire repository or require a user upload. An old revision uses its old
exact backing; changing a live path without explicit acquisition changes nothing
in this view. Missing/corrupt retained payload produces `backing_unavailable`,
suppresses its units/edges and marks closure incomplete. A still-existing live
file is never substituted. Source-in-place-only historical retention remains a
separate unqualified source-lifecycle pressure.

After explicit source refresh, the new derivation gets a different identity and
the old revision remains inspectable with `--source` / `--revision` if currently
permitted. Source applicability is attached to each document, not inferred from
the age of its claims. Existing returned values are snapshots, not fresh access
tokens; each product query resolves current authorization/backing again.

Exact policy originals and their content/profile extraction identity can be
reused across independently governed Cases. Case-bound source/unit identities,
Resource links and visibility remain independent. No persistent shared extraction
cache is introduced; ordinary content's cross-Case lifecycle remains unqualified.
Matching digest, extraction identity or entity spelling never grants permission.

## Bounds, rebuild and qualification

The inherited frontier allows 128 declarations, bounded file trees and existing
resource envelopes. Knowledge additionally admits at most 1,024 visible source
documents, 4 MiB of original material, 8,192 units (default 4,096), 4,096 bytes per
unit, 65,536 relations and 16 MiB serialized view. JSON structure depth is 32;
inherited PDF bounds remain 32 pages, 2,048 objects and bounded decompression.
Bounds refuse or explicitly mark processing unavailable; they do not weaken
authority or silently replace mandatory exact source material.

The existing W19/H19 `MemoryLexicalIndex` accepts already-qualified text for BM25.
No embeddings or encoder calls are needed. Source qualification, document
derivation, graph organization and index construction are measured separately;
host cost still grows with source/history size. Repeated policy/history resolution
is measured pressure, not solved by introducing a new store or index.

The generated Markdown/terminal wiki is read-only navigation: sources, entities,
topics, claims, contradictions and exact closure. It is not a prose summary or a
second memory authority. No editable wiki lifecycle is implemented.

Run `make smoke-source-grounded-knowledge` for the actual CLI/LMDB oracle and
the normal publication union for regressions (use the documented
[terminal-test environment](replai-terminal.md#reproduction-and-evidence) for PTY
tests):

```sh
PATH="$PWD/build/terminal-tests/bin:$PATH" make check characterization test-golden-local
```

The product oracle includes
Markdown/JSON/PDF/tree/SQLite, dual roles, unsupported/empty inputs, current versus
historical revision, source and policy revoke, cross-Case policy backing reuse,
exact payload loss/restoration, existing graph/hierarchy rebuild and zero appended
Transitions. Every invocation starts a fresh process. Knowledge graph/BM25 are
rebuilt in memory on every call; no persistent M07 cache exists to drop. The
existing hierarchy is likewise absent-by-design; an unused vector namespace is
dropped without pretending it was a populated M07 index.

The engine tests independently exercise malformed extraction, source-scope
refusal, explicit IDs versus similar text, relation counterfactuals, disagreement,
backing integrity, bounds and deterministic identity. No live-provider or Human
Golden verdict is supplied by these deterministic tests.

Archaeology recovered the provenance/non-authority boundary, not the historical
`yai-dev` Knowledge plane. At `5c1c7b9d0`, drained recall ranking still used
activation/pin/recency bonuses; `2a4018147` removed the old `src/knowledge` tree
into narrower owners. Its ownership guards and analytics consumption contract
required provenance, uncertainty and control before action. Current W19/H19
qualification, W20 epistemic separation, graph experience and source-bootstrap
backing are stronger owners; old hotness scores cannot establish documentary
truth. No historical directory tree, registry or source code was copied.
