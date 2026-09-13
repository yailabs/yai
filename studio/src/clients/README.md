# Application client placement

The future `StudioClient` boundary belongs here, with `live/` and `fixture/`
adapters feeding the same components. Neither adapter is implemented. No empty
TypeScript interface, copied Rust DTO, operation registry or transport is
declared before a qualified YAI contract exists.

See the canonical [Live vs Fixture Mode](../../../docs/studio.md#live-vs-fixture-mode)
and [application boundary](../../../docs/architecture.md#current-applicationclient-seams-and-limit).
