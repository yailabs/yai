# A19 Path Crosswalk (Old -> New)

## Test topology

- `tests/unit/brain` -> `tests/unit/knowledge`
- `tests/unit/exec` -> `tests/unit/orchestration`
- `tests/unit/law` -> `tests/unit/governance`
- `tests/integration/core_exec` -> `tests/integration/runtime`
- `tests/integration/core_brain` -> `tests/integration/orchestration`
- `tests/integration/law_resolution` -> `tests/integration/governance`
- `tests/integration/runtime_handshake` -> `tests/integration/runtime`
- `tests/integration/source_plane` -> `tests/integration/source-plane`
- `tests/integration/workspace_lifecycle` -> `tests/integration/workspace`
- `tests/domains/*` -> removed

## Protocol topology

- `lib/protocol/rpc_runtime.c` -> `lib/protocol/rpc/runtime.c`
- `lib/protocol/rpc_codec.c` -> `lib/protocol/rpc/codec.c`
- `lib/protocol/rpc_binary.c` -> `lib/protocol/binary/rpc_binary.c`
- `lib/protocol/message_types.c` -> `lib/protocol/contracts/message_types.c`
- `lib/protocol/source_plane_contract.c` -> `lib/protocol/contracts/source_plane_contract.c`

## Header topology

- `include/yai/protocol/rpc_runtime.h` -> `include/yai/protocol/rpc/runtime.h`
- `include/yai/protocol/rpc_codec.h` -> `include/yai/protocol/rpc/codec.h`
- `include/yai/protocol/rpc_binary.h` -> `include/yai/protocol/binary/rpc_binary.h`
- `include/yai/protocol/message_types.h` -> `include/yai/protocol/contracts/message_types.h`
- `include/yai/protocol/source_plane_contract.h` -> `include/yai/protocol/contracts/source_plane_contract.h`
- `include/yai/protocol/transport_contract.h` -> `include/yai/protocol/contracts/transport_contract.h`
