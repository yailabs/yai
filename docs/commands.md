# Command architecture

YAI's CLI is a product projection over existing semantic owners. It simplifies
how a user reaches those owners; it does not replace their truth.

## Product grammar

Default help presents the memorable roots `init`, `doctor`, `case`, `workflow`,
`review`, `policy`, `tenant`, `identity`, `runtime`, `help`, `version` and
`completion`. Case is the primary work object. Workflow is optional. Store,
journal, projection, context, graph, facts, process and carrier diagnostics
remain available through advanced help without being presented as ordinary
work.

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
