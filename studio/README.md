# YAI Studio — fixture workbench

Authority: local development, source placement and frontend verification.
[Studio architecture](../docs/studio.md) owns the product specification;
[ROADMAP](../ROADMAP.md#product-interfaces) alone owns maturity and selection.

This is an offline Case Workbench shell. Navigation, material tabs, panel resize,
collapse, focus and local drafts are real frontend interactions. Every Case,
Participant, document, code change, execution, provider and evidence row is
**authored synthetic fixture content**, not a live observation. No backend,
provider, credentials or existing Case is required.

## Run independently

From `studio/`, with Node 22.12+ and npm:

```sh
npm ci
npm run dev
```

The server binds `127.0.0.1:1420` with strict port selection. Use these queries,
or the visible FIXTURE selector in either web or desktop:

| Query | Authored scenario |
|---|---|
| `?fixture=ordinary` | Contract discussion, source excerpts and brief |
| `?fixture=developer` | Illustrative code diff, tests and output |
| `?fixture=execution` | Frozen running/review/failure states |

Missing query selects ordinary; unknown values show an error. There is no live
attempt or fallback. Switching scenarios resets material tabs and local draft
but retains layout. Reloading restores defaults. Nothing uses browser storage.

For desktop, install Rust and the platform's
[Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/), then:

```sh
npm run desktop:dev
npm run desktop:build -- -- --locked
```

Tauri embeds assets and fixtures. Default window: 1440×900, minimum 1100×680.
The build produces an executable without installer/signing qualification. There
are no native commands, plugins, sidecars, YAI engine dependency or management
APIs. The transparent compile-time PNG remains technical input, not artwork.
Studio owns its npm/Cargo lockfiles. Core/CLI Make/Cargo entrypoints require no
Node/Tauri toolchain or Studio test dependencies.

## Interaction

Choose Case, Sources, Files, Work or Providers in the activity rail. Explorer
items and source/evidence links open central materials. Tabs are local navigation;
closing a changed fixture tab does not discard a real edit. Diff/documents are
read-only. Review links inspect only. Terminal has no host or command input.
Execution states never advance with time.

Drag separators, or focus them and use arrow keys, Home/End. Tab lists also
support arrows/Home/End. Top-bar controls and these shortcuts toggle panels:

| Shortcut | Local interaction |
|---|---|
| Ctrl/Command + B | Case Explorer |
| Ctrl/Command + Shift + B | Conversation |
| Ctrl/Command + J | Bottom tools |

Panel sizes, bottom tab and draft survive collapse/reopen. Drafts are never
saved or sent; scenario changes/reload reset them.

## Verification and screenshots

```sh
npm run typecheck
npm run build
```

With the dev server running and host Chromium installed (default
`/usr/bin/chromium`, override with `STUDIO_CHROMIUM`):

```sh
npm run test:browser -- --matrix --run studio-shell-local
```

The small Playwright-library harness checks interaction, focus, resize bounds,
scroll, offline requests and repeated pixel-identical screenshots at 1280×800,
1440×900, 1728×1117 and 1920×1080. It writes twelve scenario PNGs, interaction
captures and a manifest under ignored `build/studio-shell/`. Keep the same
browser/fonts for pixel comparisons. `--screenshots-only` skips interactions;
omitting `--matrix` captures 1440×900. `--output` selects a separate directory,
relative to the working directory.

To verify production assets, start `npm run preview`, then:

```sh
npm run test:browser -- --url http://127.0.0.1:4173 --matrix --output ../build/studio-shell-production
```

The executable harness is under `tests/studio/`, fixture data under
`tests/fixtures/studio/`. `playwright-core` is development-only: no test runner
or browser download is added to npm install. It is outside the backend Make
and classification union. Retain actual commands with the existing
`tools/validation/capture_evidence.py`; see [validation](../docs/test-cases.md#studio-bootstrap-isolation)
and the [cumulative operator procedure](../docs/zero-to-current.md#studio-offline-visual-acceptance).

For isolated Xvfb rendering without DRI3, use `GDK_BACKEND=x11` with
`WEBKIT_DISABLE_DMABUF_RENDERER=1 LIBGL_ALWAYS_SOFTWARE=1`. These are test settings,
not product defaults or GPU/platform qualification.

## Source responsibilities and limits

- `src/app/`: fixture selection and desktop chrome.
- `src/clients/`: small rendering types and FixtureClient; no YAI wire contract.
- `src/workbench/`: Explorer, material host, conversation, tools and local layout.
- `src/components/`: shared tabs and inline icons.
- `src/styles/`: CSS tokens and workbench presentation.
- `src-tauri/`: native window/assets only.

LiveClient, application transport, dynamic events, real source provenance,
PTY, Open in Terminal, filesystem observation, Computer Use, YVEX management,
provider configuration, remote/mobile and the final editor/design system remain
unimplemented. Fixture interaction cannot qualify Case semantics or human live
acceptance.
