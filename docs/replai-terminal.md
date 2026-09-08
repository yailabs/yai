# Native REPLAI terminal consumer

`yai prompt` uses the native Rust `replai` library at exact revision
`df5538c718b8d068432032e7fb116fb8bfab158e` from `mothx9/replai`.
`cmd/yai/Cargo.toml` and its lockfile govern acquisition. There is no adjacent
checkout requirement, C binding, installed shared-library requirement or
fallback editor. The obsolete `vendor/linenoise` source has been removed. No vendor tree,
build script, FFI declaration or executable symbol supplies an alternative editor.

## Frontend and application ownership

`conversation_terminal.rs` owns prompt labels, command mapping, history admission
and the host loop. REPLAI owns the 65,536-byte editor, grapheme cursor, paste,
in-memory navigation, completion replacement, resize/redraw and termios/FD
lifecycle. The 200-entry history bound is frontend policy. It admits submitted
non-command text, preserves text verbatim, and is neither canonical history nor
a persisted draft store. Empty/whitespace-only input is not SEND.

`ConversationController` remains the application authority above the existing
canonical Transition/content owners. Opening, editing, history recall, paste,
completion, EOF and editing interrupt do not publish content or commit a Turn.
Enter submits one entire draft. The frontend calls `commit_parts`, reports the
committed Turn identity, then calls `execute_committed_turn`. Provider failure
cannot erase the committed user Turn. No edit-by-edit synchronization into a
Case draft, second conversation state machine or provider policy exists here.

I06 changes only the YAI application consumer: SEND atomically commits Turn and
execution intent, then the controller invokes the shared cognitive arbitration,
exact realization and finite composition boundary. Configure cognitive binding
and suitability as well as the provider envelope and mechanical qualification.
The terminal does not rank targets or infer media intent. `/retry TURN` preserves
that Turn/intent and reuses compatible recorded results. REPLAI revision and
terminal mechanics are unchanged. The PTY suite also proves arbitrated versus
pinned target choice and refusal of cross-target retry after uncertain delivery.

| Terminal or local action | Application decision |
| --- | --- |
| Submitted non-command text | Admit history; commit one ordered text part; execute the committed Turn |
| Ctrl-C while editing | Discard transient draft and reopen; never call controller Cancel |
| EOF / empty Ctrl-D | Leave without SEND |
| Ctrl-D on nonempty input | REPLAI forward grapheme deletion |
| Tab | Complete from the frontend command vocabulary; ambiguous choices are a coordinated notice |
| Input rejection | Coordinated warning; draft and canonical state preserved |
| Terminal failure | Return a CLI error; REPLAI attempts exact restoration |
| `/thread status`, `/thread list`, `/thread new [label]`, `/thread use ID` | Existing controller Inspect/ListThreads/NewThread/UseThread actions |
| `/retry TURN` | Existing controller Retry, referencing the same Turn |
| `/cancel` | Explicit controller Cancel; no canonical cancellation invented by the editor |
| `/refresh` | Reinspect canonical state; execution reads fresh state itself |
| `/transcript …`, `/memory propose`, `/thread archive ID` | Existing advanced compatibility handlers; archival metadata does not delete canonical Turns |
| `/exit`, `/quit` | Leave the frontend |

Thread inventory is now derived from committed Turns. A new empty thread is
controller-local until SEND, and optional legacy labels do not become a new
canonical field. Archiving an active thread retains the old audit metadata and
selects a new controller-local thread; canonical Turns remain inspectable.
Compatibility transcript/archive operations retain their former configured
journal/provider prerequisites. They are not another conversation owner.

The interactive path uses the Case's cognitive binding/arbitration within its
governed provider envelope. Explicit direct
provider/language/continuation flags are rejected there rather than silently
ignored or allowed to bypass selection. They remain available on the existing
`--once`/piped invocation path, and `--dry-run` retains its original invocation
preview through the same REPLAI editor. These modes do not acquire new SEND
semantics. `case enter` remains inspection/admission and optional shell setup;
it did not contain a separate interactive editor. No new `yai chat` command,
attachment/path inference or Markdown framework is added by the terminal.
Cognitive routing belongs to the I06 application host, not REPLAI.

The short Product entry `./yai open [CASE]` uses this same consumer. Guided
`init`/Case setup and in-Case `/connect`, `/review`, `/attach`, `/policy publish`,
`/admit` and `/workflow bind` use bounded REPLAI input with no navigation history
for setup answers. These are YAI application questions/confirmations, not a new
editor or library API. Ctrl-C/EOF cancels the pending question without approval.
Bare `/retry` resolves the current Participant/thread's latest canonical Turn;
the explicit-ID form remains. See the [cumulative runbook](zero-to-current.md).
The qualified pin, R5 removal and terminal mechanics above are unchanged.

## Terminal and execution lifecycle

The prompt label is `yai(CASE)`; REPLAI composes its accent, delimiter and spacing.
There is no private prompt palette or terminal canvas. NO_COLOR and TERM=dumb
follow the library rules, including no SGR residue. TERM=dumb disables color;
it does not establish a backend without cursor/erase support.

The interaction closes before application commands or execution write output.
Submitted text, interrupt and EOF restore captured termios and close only the
library's duplicate FDs. The next prompt reuses editor/history state. Ambiguous
completion and edit warnings use `external_output`, preserving draft and cursor.
There is no concurrently editable draft while a buffered provider request runs.
The released-editor execution phase prints bounded wait updates every five
seconds. These are application status, not token streaming or transport-abort
promises. Provider qualification prints its current probe and elapsed time.
`/connect` is text-conversation onboarding; `/connect workbench` explicitly tests
the additional native-function and JSON contracts before binding that profile.

A scoped `signal-hook` iterator observes SIGINT only around execution/retry,
forwarding it to the controller's existing cancellation object. Its worker is
closed and joined on scope exit/unwind. Signal-hook supplies safe signal delivery
absent from std; its MIT/Apache-2.0 package is locked independently of REPLAI.
It adds no cancellation behavior to the terminal library. The signal library
unregisters subscriptions on drop; this is not a promise to restore arbitrary
process-wide signal-handler installations byte for byte.

The controller's cancellation is conservative and checked before dispatch.
After dispatch, an interrupt may surface an interrupted transport read as
`delivery_indeterminate`; it does not justify automatic replay. A completed
buffered reply may instead be recorded. The frontend preserves whichever typed
outcome the existing provider/controller reports. SIGKILL and process abort
cannot promise Rust cleanup.

## Reproduction and evidence

Build uses ordinary Cargo Git acquisition, not a local path override:

```sh
cargo fetch --locked --manifest-path cmd/yai/Cargo.toml
make build
python3 -m venv build/terminal-tests
build/terminal-tests/bin/pip install -r tests/requirements-terminal.txt
PATH="$PWD/build/terminal-tests/bin:$PATH" make smoke-replai-terminal
PATH="$PWD/build/terminal-tests/bin:$PATH" make check characterization
```

By default the PTY test launches `./yai`, prepares an authorized bounded Case
through `./yai`, and reads canonical state/Turns through that same product CLI.
R5's explicit `YAI_REPLAI_TEST_ARTIFACT` override runs that same suite against an
independently built artifact; it does not change the operator launcher.
An independent pyte parser checks visible cells and cursor, while termios and
`/proc/PID/fd` are directly observed. The existing provider-governance HTTP
fixture has a bounded reply barrier so the test can inspect a committed Turn
before provider success/failure. It also checks exact inline UTF-8, digest,
participant, ordinal and provenance against canonical Turn inspection.

Tests retain ordered input/CLI output/state/PTY evidence in a unique printed
`/tmp/yai-r4-*` directory. `tests/classification.tsv` admits this product/loopback
qualification into the publication lane. Existing I01–I06, governance and
controller tests remain lower-level positive controls. Loopback HTTP is not
live external-provider interoperability or model quality evidence.
