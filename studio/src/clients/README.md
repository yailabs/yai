# Frontend Case-data seam

`CaseDataSource` is the bounded input to the single Studio Workbench.
`LiveDataSource` maps the qualified local `LiveClient` application projections;
`FixtureDataSource` maps deterministic authored values supplied by
`FixtureClient`. Both produce the same small `CasePresentation` types consumed
by built-in contributions. These types describe only implemented presentation
needs and neither copy Rust `CaseState` nor declare a YAI wire protocol.

`FixtureClient` retains its authored Start Center catalog, composition sections
and scenarios under `tests/fixtures/studio/`. It performs no I/O or execution
simulation and is never a live-failure fallback. Host capabilities are resolved
separately, so choosing fixture data does not disable native desktop facilities.
UI demand does not create backend authority.

See [Live vs Fixture Mode](../../../docs/studio.md#live-vs-fixture-mode) and
the [application boundary](../../../docs/architecture.md#current-applicationclient-seams-and-limit).
