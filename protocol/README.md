# Protocol

Canonical protocol stack for YAI.

## Scope

`protocol/` defines the canonical communication surfaces and wire contracts used by the system.

## Layout

- `control/` control-plane messages and command/reply contracts
- `rpc/` RPC envelope, codec, and protocol runtime support
- `source/` source-plane protocol surfaces
- `transport/` transport-level protocol surfaces
- `wire/` low-level wire and ABI-facing protocol artifacts
- `binary/` binary framing and serialization helpers

## Rules

- `protocol/` contains protocol surfaces, message definitions, codecs, and wire-level support.
- System-service implementations do not belong here unless they are strictly protocol-runtime support.
- Test vectors belong under `tests/protocol/`, not under `protocol/`.