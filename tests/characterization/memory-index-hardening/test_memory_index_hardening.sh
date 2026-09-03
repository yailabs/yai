#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"

CARGO_TARGET_DIR="$ROOT/target" cargo test --manifest-path "$ROOT/engine/yai-engine/Cargo.toml" \
  memory_index::tests::h19_ -- --test-threads=1
CARGO_TARGET_DIR="$ROOT/target" cargo test --manifest-path "$ROOT/cmd/yai/Cargo.toml" \
  memory_cli::tests::h19_ -- --test-threads=1

printf 'memory_index_hardening: pass\n'
printf 'adversarial_matrix: H19-S01..H19-S24\n'
printf 'authority_delta: zero\n'
printf 'lmdb_delta: zero\n'
