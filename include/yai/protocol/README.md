# Protocol Public Headers

Canonical protocol code surface:

- `control/` for protocol control-plane contracts and ids
- `rpc/` for RPC contract/runtime/codec headers
- `transport/` for transport envelope headers
- `binary/` for binary framing headers

Protocol contracts are canonicalized under `control/contracts/`; implementation headers stay under `include/yai/protocol/`.
