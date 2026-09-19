# YAI Studio — local Case Workbench

Authority: local development, source placement and frontend verification.
[Studio architecture](../docs/studio.md) owns the product specification;
[ROADMAP](../ROADMAP.md#product-interfaces) alone owns maturity and selection.

Normal Studio mode is a bounded, single-host live vertical. The desktop shell
authenticates the current operating-system principal through YAI, lists the real
Cases visible to it, resolves a Case Participant attachment and renders typed
application projections. Missing facts remain empty, unavailable, stale or
refused. Studio never reads LMDB, parses CLI output or substitutes fixtures after
a live failure.

The visible perspectives are Overview, Environment, Knowledge, Memory,
Authority, Work and Compute. The Context Panel separates Conversation,
Inspector and Activity. The bottom tool surface is real layout, while Terminal
explicitly reports that no PTY is attached. Conversation is read-only because
SEND is outside this vertical.

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
Default window: 1440×900, minimum 1100×680. Studio owns its npm and Tauri Cargo
lockfiles; backend builds do not require Node or Tauri.

## Live behavior

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

The reusable graph viewport supports pan, zoom, fit, drag, search/filter,
neighbor emphasis and selection. Relational layout is used for Memory;
directed layout is used for Authority and Workflow. Inputs remain derived YAI
relations or workflow/authority projections, never a frontend graph truth.

## Explicit fixture and gallery modes

Fixtures survive only as an opt-in development and visual-regression mode:

```sh
npm run dev:fixture
```

In that process, the existing deterministic routes remain available:
`?fixture=ordinary`, `?fixture=developer`, `?fixture=execution&snapshot=1..3`
and `?view=new`. They are visibly marked `FIXTURE`; there is no automatic switch
from live to fixture mode.

The permanent UI primitives have a development-only gallery at `?gallery=1`.
It covers typography, surfaces, controls, rows, statuses, empty states and focus
states. Production builds do not expose the gallery route.

## Verification

```sh
npm ci
npm run typecheck
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
- `src/clients/live.ts`: LiveClient and small presentation types.
- `src/live/`: live workbench, reusable graphs and development gallery.
- `src/clients/fixture.ts`, `src/start/`, `src/workbench/`: explicit fixture mode.
- `src/components/` and `src/styles/`: shared controls, icons and visual tokens.
- `src-tauri/`: local invocation/event adapter and desktop lifecycle.

This vertical does not implement conversation SEND, PTY, Open in Terminal,
filesystem observation, remote service transport, multi-client mutation
correctness, Computer Use, YVEX management, provider configuration, Mobile,
packaging/signing or the complete public application contract.
