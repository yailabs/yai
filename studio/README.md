# YAI Studio — local Case Workbench

Authority: local development, source placement and frontend verification.
[Studio architecture](../docs/studio.md) owns the product specification;
[Studio progression](ROADMAP.md) owns implementation ordering; the repository
[ROADMAP](../ROADMAP.md#product-interfaces) alone owns maturity and selection.

Normal Studio mode is a bounded live vertical over the resident local YAI Host. The desktop shell
authenticates the current operating-system principal through YAI, lists the real
Cases visible to it, resolves a Case Participant attachment and renders typed
application projections. Missing facts remain empty, unavailable, stale or
refused. Studio never reads LMDB, parses CLI output or substitutes fixtures after
a live failure.

The visible perspectives are Overview, Environment, Knowledge, Memory,
Authority, Work and Compute. The Context Panel separates Conversation,
Inspector and Activity. In the desktop build, Terminal is a real transient local
PTY. The browser surface explicitly reports that the desktop host is required
and never fakes a shell. Conversation is read-only because SEND is outside this
vertical.

Live and fixture data use the same `StudioApplication`, Workbench Kernel,
registered built-in contributions and Surface/Panel/navigation owners. Selecting
fixture data changes only the Case presentation source. Native desktop
capabilities are detected independently, so fixture mode inside Tauri still has
the real integrated PTY while fixture mode in a browser does not.

## Run the live desktop

From `studio/`, with Node 22.12+, npm, Rust and the
[Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/):

```sh
npm ci
YAI_HOME=/path/to/a/real/yai/home npm run desktop:dev
```

`YAI_HOME` must contain the YAI-owned `store/lmdb` environment. The normal web
surface cannot host YAI by itself: `npm run dev` deliberately shows
`transport_unavailable` and never loads sample data. The server binds
`127.0.0.1:1420` with strict port selection; stop an older Vite process if that
port is occupied.

Build the desktop without making it a core/CLI prerequisite:

```sh
npm run desktop:build -- -- --locked
```

The build produces an executable without installer/signing qualification.
Default window: 1600×960, minimum 1000×650. Studio owns its npm and Tauri Cargo
lockfiles; backend builds do not require Node or Tauri. The undecorated Tauri
window uses Studio-owned compact chrome so application menus, Case navigation
and native minimize/maximize/close controls occupy one desktop title row. Empty
title-bar space moves the window, double-click maximizes/restores it, and bounded
native operations resize it from every edge and corner. Interactive title-bar
controls remain controls and do not initiate dragging.

## Live behavior

Launching the Tauri desktop discovers or starts one resident Rust-owned YAI
Host for the selected `YAI_HOME`, waits for readiness and attaches through its
private versioned Unix socket. The Host owns the typed `yai-application`
instance and generation observation; Tauri is a client and does not poll YAI
persistence. Closing Studio ends its attachment and transient PTYs but leaves
the Host and Case continuity alive. Studio does not parse CLI output or start a
provider, YVEX or the separate `RuntimeInstance` scheduler.

The same lifecycle implementation is available headlessly:

```sh
YAI_HOME=/path/to/a/real/yai/home yai host status
YAI_HOME=/path/to/a/real/yai/home yai host start
YAI_HOME=/path/to/a/real/yai/home yai host restart
YAI_HOME=/path/to/a/real/yai/home yai host logs
YAI_HOME=/path/to/a/real/yai/home yai host stop
```

Repeated start discovers the same Host instead of creating a competitor.
Runtime supervision and platform login autostart remain open; see the
[product topology](../docs/studio.md#yai-product-topology) and [S1
program](ROADMAP.md#s1--product-host--lifecycle).

The Start Center lists authorized local Cases using `case.list`. Opening one
uses `case.open`, which requires a real principal-to-Participant link and returns
an ephemeral attachment. `case.summary` composes presentation views from current
authorized YAI owners. Source-grounded Knowledge resolves retained bytes through
YAI's content owner; Studio does not reopen repository paths. The resident Host
observes authorized Case-generation changes and fans out invalidation facts; a
typed Host heartbeat detects transport loss and event gaps. LiveClient refetches the typed summary and performs a
full resync on stale generation. Closing Studio does not close or mutate a Case.

The persistent product qualification Case is
`case:studio-live-qualification` in its operator-owned non-Golden `YAI_HOME`.
Normal UI presents **Studio Live Qualification** while Inspector technical
detail retains the canonical ref. Bounded reconciliation assets and the
CLI/application parity assertion live under
`tests/qualification/studio-product-vertical/`; the cumulative procedure is in
[`docs/zero-to-current.md`](../docs/zero-to-current.md#studio-resident-host-live-local-acceptance).
They inspect and advance ordinary YAI state and never recreate the Case or seed
frontend fixtures.

Once attached, browser or mouse Back does not return to the Start Center. Use
`File > Open Case…` or the Case selector in the unified titlebar to attach
another real Case. Back/Forward in that titlebar navigates frontend-local
perspectives and selections only.

Each primary Case perspective opens in its own persistent tab. Environment is a
typed explorer: Files use the hierarchy already projected by YAI, Sources open
a Source Surface and Resources open an operational Resource Surface. A Source
is never opened as if it were a file. Files use one reusable preview tab until
pinned; the center prioritizes their exact content while path, Source, revision,
digest, media type and provenance live in Inspector. Studio never scans the
filesystem and an unacquired Source shows explicit missingness.

Textual files default to a lazy-loaded CodeMirror 6 Surface with line numbers,
syntax-aware language modes, folding, bracket matching, selection, undo/redo,
find/replace, a Workbench-owned dirty marker and explicit Revert. JSON, TOML,
YAML, XML, Rust, TypeScript/JavaScript, Python, Shell, C/C++, CSS, HTML,
Markdown and plain text retain distinct source presentation. Markdown, SVG and CSV expose trusted alternatives through
`File > Open With…` without changing the underlying material identity. Exact
live text comes from authorized `material.read` resolution of immutable retained
Source backing. Studio rejects a response unless Case, object, Source, revision,
path, digest, media type, byte count and generation agree with the active
Surface; late responses cannot initialize a later preview file. Local edits are deliberately not saveable at current HEAD:
there is no qualified participant-origin filesystem-content mutation in YAI, so
Save is disabled and dirty close requires confirmation.

Markdown Preview renders real retained Markdown and marks previews of unsaved
local drafts. Relative links open only files already in qualified Case inventory;
raw HTML and remote image fetches are disabled. Image, PDF and native audio/video
renderers consume the same exact-byte `material.read` validation, including base64
responses. CSV/TSV Table supports full-set search/sort, 50-row pages, optional
column names from the first row, and keyboard/pointer column resizing. These paths
do not manufacture live material when YAI lacks a retained revision.

`npm run test:media` qualifies authored text/binary responses, digest refusal,
inert SVG and local preview behavior. `node ../tests/studio/preferences.mjs`
checks validated preferences, restart persistence, compact Settings and Edit-menu
focus. `YAI_STUDIO_TEST_HOME=/selected/home YAI_STUDIO_TEST_BINARY=/path/to/yai
node ../tests/studio/live-material-preview.mjs` is explicitly read-only against an
already-running Host and the persistent qualification Case; it does not start,
enrich or reset that Case. All use the fixture Vite server on port 1422 for the
browser renderer, and report their test-bridge/native boundaries explicitly.

Environment now offers Declare Source for existing compatible Resources, plus
Revoke Source on the typed Source Surface. Declaration supports discovery paths
and qualified named SQLite/HTTP requests; it does not fetch or acquire content.
`YAI_STUDIO_TEST_BINARY=/path/to/published/yai npm run test:environment` checks
these authored interactions against a real Host in a disposable profile, with
path/name refusals, duplicate reuse, identity collision and canonical replay.

Authority > Bind Policy accepts an exact published artifact reference. Selecting
a bound policy in Inspector offers Replace/Unbind; each action requires a reason
and the captured Case generation. `YAI_STUDIO_TEST_BINARY=/path/to/published/yai
npm run test:authority` qualifies all three positive/stale paths on a real Host.
A stale refusal offers Close and refresh Case, and never retries the mutation.

Window-local sessions retain each Case's tabs,
selection, navigation and dirty buffers across attachment switches and Host resync.
A dirty preview pins itself; reopening its file selects the existing tab. Undo and
cursor state survive tab/renderer changes. Incoming revisions preserve dirty
values until explicit reload/revert; byte length and SHA-256 are checked before
read content is admitted. Window-manager close and the titlebar use the same
unsaved-draft confirmation. These buffers are not crash-persistent or saved Case
state. Hiding the Panel, selecting Output or switching Case does not terminate a
PTY. Explicit terminal kill and desktop shutdown retain their existing cleanup.
Initial desktop dimensions are bounded by the monitor work area, retaining the
preferred tall size on larger monitors. CodeMirror uses the per-response style
nonce supplied by Tauri; desktop CSP does not allow arbitrary inline styles.

Studio never writes the repository through Tauri or React. Settings uses one singleton Surface with
internal sections. Drag the left, right and bottom splitters. Useful shortcuts
are:

| Shortcut | Local interaction |
|---|---|
| Ctrl/Command + 1…7 | Overview through Compute |
| Ctrl/Command + B | Case sidebar |
| Ctrl/Command + Shift + B | Context Panel |
| Ctrl/Command + J | Bottom tools |
| Ctrl/Command + ` | Focus the integrated terminal |
| Ctrl/Command + Shift + ` | Create a terminal |
| Ctrl/Command + Shift + P | Command Palette |
| Ctrl/Command + P | Quick Open exposed Case material and open Surfaces |
| Ctrl/Command + F | Search the active Surface through its renderer |
| Ctrl/Command + Shift + F | Search the Case when the data source exposes it |

Live Case search currently reports unavailable because the bounded application
facade exposes no qualified semantic/content search query. It does not scan the
filesystem or persistence. Command Palette, Quick Open, renderer search and
Settings search remain available. Settings is one singleton Surface; actual
editable entries are Studio-local preferences stored in the versioned
`yai.studio.preferences.v1` browser/WebView local-storage record. Case and Host
settings remain read-only or unavailable without typed owner operations.

## Integrated terminal

The Terminal bottom tool uses `@xterm/xterm` in React and `portable-pty` in the
Tauri host. It starts the user's configured shell where available, with the
user's home as a deliberate generic working directory. It supports multiple
terminal instances, selection, ANSI/full-screen programs, Unicode, scrollback,
clipboard, input, process exit/kill and PTY resize propagation. Panel resize,
window resize and adjacent panel changes refit xterm and send the resulting
rows/columns to the PTY. Terminal instances live in a compact vertical pane
inside the Terminal tool only when more than one shell exists. Drag its splitter
to resize the list down to an icon-only rail. While the list is hidden or
compact, the shared bottom-tool header keeps the active shell name, New Terminal
and Kill Terminal visible; expanding the list moves names and per-shell delete
actions into that pane instead of duplicating them. Killing the last shell closes
the Bottom Panel; reopening Terminal creates a new shell.

Terminal lifecycle is desktop-local. It is not a Case attachment, YAI execution,
Workflow or Computer Use surface. Studio never inserts Case text into the shell,
parses terminal output or treats PTY bytes as application facts. You may invoke
`yai` or other tools yourself; rendered Case truth continues to arrive only from
LiveClient. Closing the window kills all terminal children. Terminals do not
persist across reload or restart, and Case-aware cwd / Open in Terminal remain
unimplemented.

Linux is the currently built and interactively qualified platform. The selected
libraries expose macOS and Windows implementations, but those targets remain
unverified until native CI or operator acceptance runs there.

The reusable graph viewport supports pan, zoom, fit, drag, search/filter,
neighbor emphasis and selection. Relational layout is used for Memory;
directed layout is used for Authority and Workflow. Inputs remain derived YAI
relations or workflow/authority projections, never a frontend graph truth.

## Explicit fixture and gallery modes

Fixtures survive only as an opt-in development and visual-regression mode:

```sh
npm run dev:fixture
```

In that process, the deterministic routes remain available:
`?fixture=ordinary`, `?fixture=developer` and `?fixture=execution`.
They are visibly marked as fixture Case data; there is no automatic switch
from live to fixture mode.

Run fixture data in the native host, including the real PTY, with:

```sh
VITE_STUDIO_MODE=fixture npm run desktop:dev
```

The permanent UI primitives have a development-only gallery at `?gallery=1`.
It covers typography, surfaces, controls, rows, statuses, empty states and focus
states. Production builds do not expose the gallery route.

The `developer` fixture also qualifies heterogeneous Work Surface renderers.
Open Environment and select `runtime-boundary.svg`, `qualification-matrix`,
`runtime-contract.pdf`, `qualification-tone.wav`, `qualification-clip.webm` or
the unknown binary example. These exercise inert SVG/image presentation,
sortable/filterable table, lazy multi-page searchable PDF, native audio/video
playback and explicit unknown-media metadata fallback. Markdown/text/structured
text, Timeline, Experience Graph and Settings use the same Surface Group,
preview/pinning, navigation, search and Inspector seams. Fixture media is visibly
authored development input and never appears as a live fallback.

## Verification

```sh
npm ci
npm run typecheck
npm run test:kernel
npm run build
npm run desktop:build -- -- --locked
```

The Rust application boundary has independent tests from the repository root:

```sh
cargo test --manifest-path application/Cargo.toml --locked
```

For a read-only product probe against an already acquired Source revision:

```sh
YAI_HOME=/dedicated/home cargo run --manifest-path application/Cargo.toml \
  --example material_read -- \
  case:studio-live-qualification CASE_SOURCE_REF SOURCE_REVISION_REF studio/README.md
```

This command exercises the same application operation used by LiveClient. It
does not read a checkout path directly and does not mutate the Case.

The existing fixture browser harness remains under `tests/studio/` and runs only
against an explicit fixture-mode server. Real desktop acceptance requires a
dedicated non-Golden YAI_HOME and records Case ID, generation, viewport and YAI
SHA. See the cumulative [operator runbook](../docs/zero-to-current.md).

## Source responsibilities and current limits

- `application/yai-application/`: typed, authorized application projections;
  no persistence or Case semantic ownership.
- `application/yai-host/`: resident local application lifecycle, private IPC,
  discovery, attachments, update fanout and operational telemetry; no scheduler
  or Case semantic ownership.
- `src/platform/`: scoped commands, context, menus, keybindings, configuration,
  navigation, theme, lifecycle and host-capability services.
- `src/workbench/`: the single Kernel, region registries and Surface Group/input
  ownership.
- `src/contrib/`: statically authored YAI views, Surface renderers, auxiliary views and
  panel contributions.
- `src/clients/live.ts`: bounded LiveClient transport and application views;
  `src/clients/dataSource.ts` owns the common presentation seam and adapters.
- `src/live/`: reusable graph viewport and development gallery only.
- `src/terminal/`: xterm rendering and desktop-only terminal lifecycle UI.
- `src/clients/fixture.ts` and `src/start/`: explicit fixture data and bootstrap.
- `src/components/` and `src/styles/`: shared controls, icons and visual tokens.
- `src-tauri/`: narrow Host client, PTY host and desktop lifecycle.

This vertical does not implement governed participant-origin file Save,
stale-revision authoring resolution, conversation SEND, Case-attached Open in
Terminal, persistent PTY sessions, filesystem observation, remote service
transport, multi-client mutation correctness, Computer Use, YVEX management,
provider configuration, Mobile, packaging/signing or the complete public
application contract.

Root interaction regression suites run against an explicit fixture dev server:

```sh
# Separate terminal: npm run dev:fixture -- --port 1422
STUDIO_TEST_URL=http://127.0.0.1:1422 npm run test:reliability
npm run test:desktop:csp
```

This lane uses authored asynchronous application responses and an instrumented
Tauri bridge; it does not substitute for resident Host or native PTY acceptance.

Populated Knowledge/Inspector qualification (same fixture Vite server):

```sh
npm run test:knowledge
```

This checks full-set search, bounded lists/canvas, exact Inspector content,
qualified endpoint accounting, keyboard and pointer navigation at four sizes.
`STUDIO_PROJECTION=/path/to/captured-case-summary.json` optionally replays a
previous `case.summary` response. That mode is recorded projection evidence,
not a claim of live transport or mutation qualification.

Authored application actions use the actual connected Host catalog. File > New
Case includes explicit Participant setup and self-linking; the Case menu exposes
cancellation/closure, Inspector offers pending Review decisions, and Work accepts
declared HumanInput. These remain YAI-authorized mutations. Settings > Advanced
shows advertised operations separately from integrated UI. An older running Host
can truthfully offer fewer actions than the checkout; restart it explicitly when
qualifying a newer published build.

```sh
YAI_STUDIO_TEST_BINARY=/absolute/path/to/published/yai npm run test:application
```

This suite uses the same fixture Vite server for rendering, but sends live
application requests to a real Unix Host through a test-only browser bridge.
It creates and removes its own temporary YAI_HOME, uses normal product operations
for Case/Participant/Workflow/policy/Review setup, checks positive and refusal
paths and CLI canonical replay, and never mutates the operator qualification
Case. It does not claim native Tauri transport qualification by itself.
