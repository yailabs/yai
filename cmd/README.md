# Command source map

Authority: local navigation for current source only. Product architecture and
command status are owned by [`docs/architecture.md`](../docs/architecture.md).

- `cmd/yai/` is the Rust operator command and current operational center.
  `main.rs` owns parsing, dispatch, formatting, process initiation, and
  residual compatibility commands. Existing provider/case, review,
  filesystem, replay, graph, and analytics behavior is isolated in six
  boundary-based internal modules.
- `cmd/yaid/` is the C daemon entrypoint. The linked daemon exposes narrow Unix
  socket status/info/shutdown and fixture-loop behavior; it does not reach all
  components in the C archive.

These modules characterize current transitions and resource/derived
boundaries; they do not define canonical state or mandate future subsystems.
