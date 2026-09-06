# REPLAI / YAI.CONSUMER.CUTOVER.0

Baseline: YAI master `d3ca2cac5f6575241839c65083d14336e5c1fedc`, tree
`319d7dd16c1de69a3b1fd25e12cfdc16f2fbf05f`. The expected `6e332851…` advanced
only through the published test-topology closure, which changes qualification
selection rather than the controller/product implementation. The canonical
worktree was initially clean. Implementation used that same master over LAN SSH
on Exon; no alternate branch or Git worktree was created.

Intended commit: `feat: migrate interactive prompt to native REPLAI`.
Pre-publication state: implemented and locally qualified; awaiting isolated
commit, ordinary push and remote equality verification. Actual final SHA and remote equality belong in the
post-publication handoff, not this self-containing report.

## Boundary and dependency

REPLAI is pinned to `df5538c718b8d068432032e7fb116fb8bfab158e`, tree
`df79f4e025d212012edd9cdb4d0170a3ed5d7ebe`, through Cargo Git revision and lockfile.
Later library documentation commits do not justify advancing the qualified pin.
The Rust application consumes the native public API, with no C adapter, copied
header, shared-library installation or adjacent-checkout requirement.

`yai prompt` was the only live line editor. Its new thin frontend maps events
into the existing `ConversationController`; REPLAI owns transient editor state,
paste, history navigation, completion replacement and terminal resources. YAI
owns labels, command candidates, history admission and application actions.
`case enter` was inspection/admission/shell setup, and remains so. One-shot and
piped provider plumbing retain their existing paths. No new chat command or
cognitive orchestration is introduced.

The linenoise build script and FFI/live loop are removed. Its vendored source is
physically retained, unreachable, for R5. `nm` of the actual YAI binary observes
REPLAI Interaction and no linenoise symbol; the isolated producer build also
succeeds with the entire vendor directory absent.

`commit_parts` precedes `execute_committed_turn`. Editing, recall, paste, Tab,
editing interrupt and EOF never synchronize a draft into canonical state.
SEND admits one text part to the existing authority, independently of provider
success. Controller source changes are limited to its implemented-frontend
status string, matching expectation and explanatory comments. Case, content,
provider governance and cognitive algorithms are unchanged.

[The implemented contract](../../../docs/replai-terminal.md) defines commands,
transcript/archive compatibility, direct-provider flag admission, history,
completion, coordinated output and conservative execution cancellation.

## Historical property recovered

Inspected `yai-dev` at `5c1c7b9d099eea9f2947146cd821d6501c4a6ddf`, its core-spine
40 prompting/projection document and `prompting_handler.c` history at
`cffb318b980456f2671a297e14a6b05f5ac68320`. Recovered invariant: prompt/projection
is non-authoritative material; application admission owns durable effects.
No old planes, Agent owner, C parsing implementation or terminal code is copied.
The current YAI linenoise build/FFI was introduced at
`f86c8a3` (`Add in-case prompt surface retention`). The later I01 controller,
not that historical call shape, now provides the independently tested canonical
commit/execution seam. Reuse preserves its commit-before-provider property.

## Qualification and limits

See [executable evidence](EXECUTION-EVIDENCE.md) for actual run identities,
commands, state observations and retained raw output. Terminal qualification is
Linux x86_64, Rust/Cargo 1.93.1. The real YAI executable is driven through PTYs;
canonical reads use the product CLI against isolated LMDB state. pyte is an
independent terminal-cell oracle. The existing HTTP fixture delays a real
non-synthetic response until the test independently reads the canonical Turn.

The publication union retains I01 multipart, controller, I02 planning, I03 typed
realization, I04 composition, governance, recovery and bounded endurance controls.
The CLI Clippy diagnostics match the baseline's 13 warnings exactly; this wave
does not claim warnings-as-errors cleanliness. No engine lint cleanup is folded
into the terminal migration.

No GitHub Actions workflows exist in this repository (API query returned 0).
The authoritative local publication union is therefore the qualification gate;
no remote CI pass is invented. Valgrind is not installed on this host; observed
termios and repeated process FD/resource checks establish the required R4
minimum, not allocator-wide memory-checker qualification.

## YVEX EXTERNAL FINDINGS

`DEPLOYMENT_LIMITATION`: no operator-supplied live endpoint and exposed model
were available. No live YVEX request was attempted and no YVEX source, runtime
or model was administered. Successful bounded loopback HTTP proves the generic
provider/controller path; it is not live model/interoperability evidence.

REPLAI, YVEX and YAI's cognitive/domain owners were not modified. R5 is deferred.
