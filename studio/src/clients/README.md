# Frontend presentation seam

`StudioClient` exposes an authored Start Center catalog, Case-composition sections,
scenario choices and a workspace presentation to the offline shell.
`FixtureClient` supplies synthetic values from `tests/fixtures/studio/`. Small
types describe what the implemented views render, including grouped explorer
items, temporal events, graph relations and inspector content. They do not copy
Rust CaseState or declare a YAI API/protocol.

No LiveClient, I/O, execution simulation or silent fallback exists. A future
live adapter must consume a qualified YAI application contract and may require
these presentation types to evolve. UI demand does not create backend authority.

See [Live vs Fixture Mode](../../../docs/studio.md#live-vs-fixture-mode) and
the [application boundary](../../../docs/architecture.md#current-applicationclient-seams-and-limit).
