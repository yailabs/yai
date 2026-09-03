# Foundation Hardening 19 report

## State

H19 implementation and deterministic qualification are complete against
baseline `95730f1a11025491a3f0f7ccec0aba8b1d3f036b`. Publication state at report
authoring is `complete_qualified_prepublication`; because all six required live
provider/encoder variables were absent, the intended published state is
`complete_published_external_acceptance_pending`. The intended semantic commit
message is `harden: close derived memory index boundaries`. This report does not
claim the SHA of the commit that contains it.

W19 history is unchanged. `H19-SCOPE-CORRECTION.tsv` corrects the old forward
delta: episodic/semantic memory, consolidation, contradiction resolution,
semantic retention, promotion, and learned reranking are not H19.

## Constitutional result

Transition history remains canonical. OperationalMemory, representation
documents, embeddings, corpus/index manifests, BM25, exact-cosine indexes, and
RetrievalSet v2 remain derived and rebuildable. H19 adds zero semantic owners,
zero operational owners, and zero LMDB databases; the repository remains
37/40. Representation profiles remain immutable runtime/admin configuration.

The last semantic barrier now re-resolves each indexed memory ID against exact
current OperationalMemory and verifies Case, generation, lifecycle, visibility,
authority posture, and source digest. The payload passed to Projection is the
current OperationalMemory entry, never derived representation text. A
self-consistent forged representation is rejected; a self-consistent vector can
affect retrieval quality but cannot inject or promote content.

## Derived store

Physical format is `yai.derived_memory_store.v2`. It is segmented into
`seal.json`, bounded `metadata.json`, fixed-width `vectors.f32le`, and an atomic
`current.json`. Float rows are little-endian float32 in exact embedding metadata
order. Maximums are 50,000 documents, 4,096 dimensions independently,
25,000,000 vector elements, 100,000,000 raw vector bytes, 256 MiB metadata, and
384 MiB total. Cross-products exceeding the element/envelope budget fail before
encoder work or allocation.

On Linux, all mutable and read paths are anchored to retained directory
descriptors with `openat2`, `RESOLVE_BENEATH`, `RESOLVE_NO_SYMLINKS`,
`RESOLVE_NO_MAGICLINKS`, `O_NOFOLLOW`, and descriptor-relative
`mkdirat`/`renameat`/`unlinkat`. Directories must be owned by the effective UID
and not group/other writable; files must additionally be regular, private, and
single-link. Non-Linux derived-store operations fail closed rather than claim
Linux guarantees.

Publication performs component write/fsync, temp-directory fsync, build rename,
builds-parent fsync, pointer temp write/fsync, pointer rename, and profile fsync
in that order. Readers hold a shared profile lock across pointer and all
components; builders/drop/GC hold the exclusive lock. At most two complete
builds are retained per Case/profile, without semantic retention semantics.

## Retrieval and provider governance

BM25 corpus statistics and vector top-k operate only on the already-qualified
document set. Hidden, superseded, wrong-Resource, and wrong-causal items cannot
alter visible IDF, top-k, RRF ranks, candidate counts, or exact-anchor privilege.
RRF uses checked integer arithmetic, refuses duplicate plane IDs/invalid ranks,
and ties on exact IDs. Floating-point determinism is claimed only for the
admitted Rust implementation/build; retrieval IDs are not canonical authority.

One builder acquires the profile lock and rechecks an equivalent sealed build
before external encoding. A 32-process product run issued zero duplicate
embedding requests after the first build. Every batch revalidates the exact
target/model/revision/dimension, Tenant, `TextEmbedding` qualification, trust,
health/circuit, loopback locality, and credential snapshot before and after
dispatch. No alternate encoder is used. Indexed batch results bind by explicit
response index and reject missing, extra, duplicate, malformed, wrong-model,
wrong-dimension, empty, zero, and non-finite vectors.

Corrupt, missing, stale, or source-divergent derived state disables fuzzy planes
and leaves qualified exact/operational retrieval available. `yai case memory
index verify CASE --profile PROFILE` performs explicit deep plus current-source
verification and distinguishes `current`, `stale`, `corrupt`, `missing`, and
`source_divergent`. Status exposes physical format, bytes, generation, profile,
manifest, dimension, integrity, and ANN posture.

## Performance and ANN

The W19 50k fixture hybrid time was 39,411,645 microseconds. The defect was
query-time deep validation: rebuilding BM25, recomputing/serializing derived
content, and monolithic JSON work on every query. H19 separates publication-time
deep validation, bounded sealed load validation, and selected-source
revalidation. Current release measurements at 50k x 8 are 46.553 ms load
validation, 486.823 ms source revalidation, 88.464 ms BM25, 12.342 ms exact
cosine, and 1.182 s total cold hybrid. This is roughly a 33x material reduction;
no SLA is claimed.

At 50k x 384, the largest admitted realistic cross-product characterized,
exact scan was 36.881 ms and raw vectors were 76.8 MB. 50k x 768 is refused
before allocation because it exceeds 25 million elements. ANN remains deferred:
exact vector scan is not the measured dominant cost, remains the correctness
oracle, and no HNSW dependency was justified.

## Compatibility and external boundary

Logical W19 v1 representation/profile/embedding/corpus/index/query schemas and
RetrievalSet v2 are unchanged. RetrievalSet v1 readers remain. Provider
Qualification v3, Projection v5, ContextFrame v5, and rendered input v5 remain.
Only disposable physical storage advanced; old physical v1 is rebuilt from
canonical memory.

The start-time remote YVEX `models1` SHA is
`e5c77060daae458f4b22664b910ed3582ec8a2a0`, resolved with `git ls-remote` and
inspected read-only. It differs from local HEAD `5b3aa34b...` and local
`origin/models1` `1f7ff1cd...`. Current `yvex.openai.compat.v2` supports Chat
Completions, Responses, structured JSON, and typed function/tool calls, and
explicitly refuses embeddings. H19 keeps a separate loopback encoder. Provider
tool output remains candidate material for future W22 normalization and never
executes directly.

Live YVEX+DeepSeek and real local-encoder acceptance was not run because the
operator variables were absent. This is `DEPLOYMENT_LIMITATION`, not a YVEX
defect. `MANUAL-ACCEPTANCE.md` contains the exact registry-backed zero-to-use
case and has been Bash syntax checked.

## Deliberate exclusions

No W20 episodic/semantic/consolidation/contradiction/retention work was started.
No W22 filesystem read/list/stat/search, process observation, generic shell, or
provider-tool execution capability was added. The root README was not modified.
