# Frontend presentation seam

`StudioClient` exposes scenario choices and a workspace presentation to the
offline shell. `FixtureClient` supplies authored synthetic values from
`tests/fixtures/studio/`. The small types describe what the implemented views
render; they do not copy Rust CaseState or declare a YAI API/protocol.

No LiveClient, I/O, execution simulation or silent fallback exists. A future
live adapter must consume a qualified YAI application contract and may require
these presentation types to evolve. UI demand does not create backend authority.

See [Live vs Fixture Mode](../../../docs/studio.md#live-vs-fixture-mode) and
the [application boundary](../../../docs/architecture.md#current-applicationclient-seams-and-limit).
