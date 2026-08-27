# Engine source map

Authority: local navigation for current Rust source only. See
[`docs/architecture.md`](../docs/architecture.md) for reachability and state
authority.

- `yai-engine` implements JSONL record/journal handling, LMDB records and graph
  relations, and derived graph/projection/memory/query/reconcile helpers used
  by the Rust CLI.

The former marker FFI crate and smoke-only C→Rust bridge were removed during
source refoundation: neither had an installed ABI or product caller.

The directory name does not make every included object canonical. Current
journal and LMDB writes can diverge, and graph/index/memory/projection are not
target historical authority.
