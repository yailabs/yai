# Studio bootstrap

Authority: local build instructions and source placement only.
[Product/frontend architecture](../docs/studio.md) owns the Studio specification;
[ROADMAP](../ROADMAP.md#product-interfaces) alone owns maturity and selection.

This package builds a deliberately empty Case Workbench scaffold. It has no
application client, fixtures, terminal, provider connection or Case state.

## Build independently

From `studio/`, using Node 22.12+ (Node 24 LTS recommended) and npm:

```sh
npm ci
npm run build
npm run dev
```

The browser development server binds only `127.0.0.1:1420` and fails if occupied.
For the desktop shell, install Rust and the platform's
[Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/), then:

```sh
npm run desktop:dev
npm run desktop:build -- -- --locked
```

The desktop build embeds the frontend and produces an executable without
installers, signing or distribution qualification. `src-tauri/Cargo.lock` and
`package-lock.json` belong to this package. Core/CLI builds and backend tests
use their existing Make/Cargo entrypoints and require none of these frontend
dependencies. There is no root npm workspace or Interfaces runtime dependency.

## Source placement

- `src/app/`: bootstrap composition; React renders and handles interaction.
- `src/clients/`: documented placement for future LiveClient/FixtureClient;
  no API is declared yet.
- `src-tauri/`: window/assets bootstrap only, no commands, plugins, sidecars,
  capabilities or YAI engine dependency.

Add Case presentation, components, surfaces, local state, terminal and YVEX
modules when they acquire executable responsibilities. Empty noun directories
are not owners. The skeleton has no CSS or fake dashboard.
`src-tauri/icons/icon.png` is a transparent 1×1 RGBA compile-time input required by Tauri, not product artwork.

Separate verification commands and claim limits live in
[test cases](../docs/test-cases.md#studio-bootstrap-isolation).

For an isolated Xvfb capture on a host without DRI3, force `GDK_BACKEND=x11`
and use `WEBKIT_DISABLE_DMABUF_RENDERER=1 LIBGL_ALWAYS_SOFTWARE=1` in the test
environment. This is a headless rendering procedure, not a product runtime
default or GPU/platform qualification.
