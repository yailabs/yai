# Command architecture

YAI's CLI is a product projection over existing semantic owners. It simplifies
how a user reaches those owners; it does not replace their truth.

## Product grammar

Default help presents the memorable roots `init`, `open`, `doctor`, `case`, `workflow`,
`review`, `policy`, `provider`, `tenant`, `identity`, `runtime`, `help`, `version` and
`completion`. Case is the primary work object. Workflow is optional. Store,
journal, projection, context, graph, facts, process and carrier diagnostics
remain available through advanced help without being presented as ordinary
work.

The normal first-use journey is `./yai init`, then `./yai open NAME`. Missing
initialization values are asked through REPLAI only on a real terminal; explicit
flags still support deterministic automation and JSON. `open` resolves exact
`case:NAME` spelling among authorized state, or asks before creating it. Multiple
Tenants/Cases/executors require explicit selection. A ready Case reopens without
changing its generation; there is no mutable global current-Case file.

Participant bootstrap shows the exact roles, model disclosure and human
Principal link, then requires `admit`. The backend applies the bounded set of
existing participant transitions in one authorized transaction, refusing stale
generation or identity conflict without half a setup. This supplies no policy,
provider trust, resource permission or Grant. An existing partial setup can be
completed without recreating the Case. `/setup` is the explicit in-Case action.

## One compiled command authority

`cmd/yai/src/cli/registry.rs` owns the versioned `yai.cli.registry.v1`
descriptors compiled into the binary. Each descriptor binds a stable operation
ID to one canonical command path, syntax, visibility, execution lane, mutation
posture, output capability, handler adapter, aliases and removal metadata. A
deterministic SHA-256 digest identifies the interface projection.

The registry is not read from mutable runtime configuration. It contains syntax
metadata, not policy, Tenant roles, provider health or execution authority.
Registry validation rejects duplicate paths/IDs/aliases, invalid rules and
incomplete descriptors. The repository conformance audit additionally proves
that every descriptor handler ID is source-resolvable.

```text
argv
  -> longest exact registry path
  -> one descriptor
  -> centralized syntax validation
  -> typed invocation
  -> one execution lane / adapter
  -> existing domain owner
  -> typed CLI result
  -> human or machine renderer
```

There is no parser fallthrough between domain families. Compatibility aliases
resolve to the same operation ID and handler. Removed paths refuse with a
successor and perform no mutation.

## Lanes and visibility

Lanes describe process behavior: `LOCAL_DOMAIN`, `RUNTIME_HOST`,
`RUNTIME_CONTROL`, `INSPECTION`, `COMPATIBILITY` and `LOCAL_INTERACTIVE`.
Visibility independently classifies operations as `PRODUCT`, `ADVANCED`,
`PLUMBING`, `COMPATIBILITY` or `REMOVED`. Visibility never grants authority and
a lane is not inferred from the visual command group.

## Help and discovery

Human leaf/subtree help, `yai help --json`, aliases, removed successors and
shell completion derive from the compiled descriptors. `yai` and `yai --help`
render the product map without mutation. Direct help remains available for an
advanced command even though it is excluded from first-contact help.

The executable audit
`tests/characterization/cli-product-surface/audit_registry.py` obtains the
operation set from machine discovery and checks every leaf help path. It does
not maintain a second command list.

## Output boundary

Product handlers return typed result data once. The human renderer uses concise
messages, object sections and line-oriented tables; the machine renderer emits
`yai.cli.result.v1` or `yai.cli.error.v1`. JSON is deterministic, UTF-8,
untruncated and ANSI-free. Human success uses stdout; warnings, hints and errors
use stderr. JSON success is stdout-only and JSON error is stderr-only.

The Product entry `yai open [CASE]` (`yai case open` is its exact alias) has an
`interactive` output contract on `LOCAL_INTERACTIVE`. The explicit
`yai case workbench CASE --participant HUMAN --executor MODEL` remains available.
It uses native REPLAI and rejects `--json` before terminal acquisition; it does
not pretend its terminal event stream is a structured one-shot response.
All non-interactive Product operations still require structured output. Exact
inspection and automation remain available through those one-shot surfaces.
The workbench invokes typed application actions, not a child YAI executable.
`/work` commits an explicit finite execution budget with SEND; a resource
function request remains candidate material subject to current policy, review
and effect admission. Ordinary text SEND does not implicitly acquire tools.

Workbench vocabulary includes guided `/attach`, `/policy publish`, `/connect`,
`/admit`, `/review` and `/workflow bind`. Answers are application input handled
by the same typed controller actions as their retained complete forms:
`/attach FILE`, `/policy publish FILE REASON`, and explicit
`/connect ENDPOINT MODEL --trust approve --attest evidence:REF`.
`/connect` discovers the public endpoint catalog: one exact model is selected
automatically, multiple models require a name/number choice, an empty/invalid
catalog refuses. Catalog access is not inference or qualification. Literal
address locality is derived; ambiguous DNS scope is asked. An authentication
challenge asks only for an environment credential reference. One final explicit
confirmation covers trust and operator-attested suitability, with generated
provenance rather than a manually typed evidence ID. An existing primary uses
`replace` instead of `approve` in that same confirmation. The captured Case
generation and selected catalog identity are revalidated before qualification.
It qualifies text conversation only. `/connect workbench` explicitly
requires text, native function/result round trips and JSON output; the historical
complete `/connect ENDPOINT MODEL ...` form retains that full profile. There is
no silent fallback after failed workbench qualification. Neither profile infers
suitability or credentials. `/review` displays
the exact pending operation and uses the current human identity, with explicit
approve/deny/defer and reason. Bare `/retry` resolves the latest canonical Turn
of the current Participant/thread and calls the unchanged recovery boundary;
`/retry TURN_ID` remains exact. `/help` is compact; `/help all` and completion
share the retained full action vocabulary. Resource actions accept `workspace`
as exact spelling for `resource:workspace`, not a fuzzy match or persisted alias.
Other actions include
`/read`, `/discover`, `/admit`, `/material`, `/query`, `/fetch`, `/catalog`, `/test`,
`/operation ID` before human `/review`, bounded `/work`, and same-Turn `/retry`.
`/workflow bind|run|input|advance` and `/workflow patch validate|adopt` reuse the
existing Workflow owner. `/handoff offer|accept|result|reconcile` is explicit
cross-Case material exchange, never resource/authority transfer. `/rebuild`
repairs disposable views and verifies unchanged ledger plus replay; it does not
invent an embedding profile. The full [cumulative procedure](zero-to-current.md)
distinguishes shell bootstrap from REPLAI actions and external prerequisites.

Color is semantic and restrained, never the sole state indicator. Non-TTY and
JSON output are undecorated; `NO_COLOR` disables ANSI on TTYs. No command enters
an alternate screen or a hidden TUI.

## Compatibility boundary

`command_adapters.rs` maps operation identities to established command/domain
functions while those owners expose mixed return shapes. It is not a legacy
semantic owner, parser, command catalog or dispatch fallback. Compatibility
line output remains bounded for existing qualification consumers; product JSON
always passes through the typed envelope.

This registry is not a public SDK or network API authority. It does not add CLI
state, an ambient selected Case, a WorkflowRun owner, an Agent, a scheduler or
an LMDB database.

## Adaptive Workflow and Case handoff

Wave 17 extends the same registry; it does not add a parser or dispatch path.
`workflow patch propose` records a bounded candidate and `workflow patch
adopt` is a separate Tenant-Owner mutation. Model-originated candidates are
parsed from one exact ProviderResult with `workflow patch propose-model`; they
never self-adopt. `workflow status` remains a read-only projection of the base
Definition plus Case-local amendments and reports the effective revision and
topology digest.

Same-Tenant work transfer is Case-centric under `case handoff`. Offer,
accept/decline, target-local result and source reconciliation are separate
typed operations. The target receives only the bounded offer payload and keeps
its own Participants, policy, resources, provider, Decisions, Grants and
Effects. A waiting Workflow Handoff node occupies neither a runtime worker nor
a ResourceFence.

## Governed providers

Wave 18 adds the `provider` administration family and `case provider bind/show`
through the same compiled registry. Target registration, synthetic
qualification, probe and Tenant-Owner approval remain separate operations.
`provider show` deliberately renders configuration, qualification, governance
and health as four dimensions. It never collapses them into a single `ready`
label.

`case provider bind` stores exact immutable target references and explicit
preference/failover posture. Selection is an engine-owned deterministic
operation recorded in Case history; it exposes bounded exclusion codes rather
than a magic score. Existing `case provider attach` remains the legacy exact
pin, not an alias for a governed pool.

## Post-I01 conversation interaction boundary

The I01 draft commands remain executable, registry-backed Advanced plumbing
for automation and exact content/provenance qualification. They are no longer
presented as the ordinary human conversation journey. Canonical Turn
inspection remains available under `case conversation turn`.

The application-side conversation controller accepts typed ordered parts,
commits the Turn before model execution, and then reuses YAI's
Projection/Context/provider-governance invocation boundary. Ordinary
conversation does not inherit `case run` requirements for a Resource,
Workflow, Effect, policy, or operational budgets. Retry executes the same
committed Turn and cannot duplicate it. New thread identities are local until
their first SEND; thread lists are projections of committed Turn history.

The Advanced `yai prompt` now consumes [native REPLAI](replai-terminal.md).
The existing controller owns submission, canonical threads, retry and cancellation;
REPLAI owns only transient editing and generic terminal interaction. The
`case enter` inspection/shell setup surface, `prompt --once` and piped invocation
remain available. No separate `yai chat`, natural path parser or cognitive
execution command is introduced by the terminal cutover.

## Case memory representation and retrieval

Wave 19 keeps memory operations Case-first in the same registry:
`case memory show/search`, `case memory index status/build/rebuild`, and
`case memory retrieval show` are typed PRODUCT operations. The destructive,
derived-only `case memory index drop` operation is ADVANCED. Human search
prints rank, semantic kind, authority posture, source planes, description and
provenance generation; JSON exposes manifests, per-plane ranks and fusion
reasons without raw vectors.

`case participant view admit` records the existing bounded
`model/model_context` Case admission explicitly; role binding alone does not
grant a view. Index build requires an exact Tenant-approved loopback target
qualified with `provider qualify TARGET --embedding`, plus an
operator-declared encoder revision and dimension. Index paths are generated
from hashed identities under `$YAI_HOME/store/derived-memory/v1`, never from a
caller-supplied path.
