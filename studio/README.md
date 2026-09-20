# YAI Studio — local Case Workbench

Authority: local development, source placement and frontend verification.
[Studio architecture](../docs/studio.md) owns the product specification;
[Studio progression](ROADMAP.md) owns implementation ordering; the repository
[ROADMAP](../ROADMAP.md#product-interfaces) alone owns maturity and selection.

Normal Studio mode is a bounded, single-host live vertical. The desktop shell
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

Launching the Tauri desktop loads the typed `yai-application` boundary in the
Studio process. It does not run the CLI, parse terminal output, or automatically
start `yai start`, a daemon, a provider or YVEX. Those runtime services retain
their own configured lifecycle; the integrated PTY is likewise an independent
user shell.

This is the **current** executable topology. The selected product target is a
resident YAI Local Host, normally one per `YAI_HOME`, shared by Studio, CLI and
future structured clients. Host discovery/startup, lifecycle commands,
autostart, shared attachments and host telemetry remain unimplemented; see the
[product topology](../docs/studio.md#yai-product-topology) and
[S1 program](ROADMAP.md#s1--product-host--lifecycle).

The Start Center lists authorized local Cases using `case.list`. Opening one
uses `case.open`, which requires a real principal-to-Participant link and returns
an ephemeral attachment. `case.summary` composes presentation views from current
authorized YAI owners. A Tauri-local update bridge observes authorized Case
generation changes and emits invalidation facts; an authorized typed heartbeat
detects delivery gaps. LiveClient refetches the typed summary and performs a
full resync on stale generation. Closing Studio does not close or mutate a Case.

Once attached, browser or mouse Back does not return to the Start Center. Use
`File > Open Case…` or the Case selector in the unified titlebar to attach
another real Case. Back/Forward in that titlebar navigates frontend-local
perspectives and selections only.

Each primary Case perspective opens in its own persistent tab. Files, documents
and sources open in one reusable preview tab; double click or `Open in work
surface` pins the material. Environment contains the source-qualified Files,
Sources and Resources inventory. Studio does not scan the filesystem: an
unacquired source shows explicit missingness. Other secondary facts reveal in
the contextual Inspector. Settings uses one singleton surface with internal
sections. Drag the left, right and bottom splitters. Useful shortcuts are:

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
inside the Terminal tool, separate from the shared bottom-tool header; drag its
splitter to resize the list.

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
cargo test --manifest-path application/Cargo.toml
```

The existing fixture browser harness remains under `tests/studio/` and runs only
against an explicit fixture-mode server. Real desktop acceptance requires a
dedicated non-Golden YAI_HOME and records Case ID, generation, viewport and YAI
SHA. See the cumulative [operator runbook](../docs/zero-to-current.md).

## Source responsibilities and current limits

- `application/yai-application/`: typed, authorized application projections;
  no persistence or Case semantic ownership.
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
- `src-tauri/`: local invocation/event adapter, narrow PTY host and desktop lifecycle.

This vertical does not implement conversation SEND, Case-attached Open in
Terminal, persistent PTY sessions, filesystem observation, remote service
transport, multi-client mutation correctness, Computer Use, YVEX management,
provider configuration, Mobile, packaging/signing or the complete public
application contract.
