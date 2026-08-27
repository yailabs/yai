# Command source map

Authority: local navigation for current source only. Product architecture and
command status are owned by [`docs/architecture.md`](../docs/architecture.md).

- `cmd/yai/` is the Rust operator CLI and current operational center. Its
  `src/main.rs` contains command parsing, orchestration, provider HTTP,
  filesystem paths, facts, and compatibility behavior.
- `cmd/yaid/` is the C daemon entrypoint. The linked daemon exposes narrow Unix
  socket status/info/shutdown and fixture-loop behavior; it does not reach all
  components in the C archive.

This directory does not define canonical state, protocol, or future source
ownership.
