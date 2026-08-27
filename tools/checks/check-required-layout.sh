#!/bin/sh
# YAI - required layout guard
#
# Purpose:
#   Ensure the canonical source and documentation roots exist.
#
# Scope:
#   Checks required directories and anchor files.
#
# Non-goals:
#   Does not verify file ownership headers.
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)

require_dir() {
  if [ ! -d "$ROOT/$1" ]; then
    printf 'required directory missing: %s\n' "$1" >&2
    exit 1
  fi
}

require_file() {
  if [ ! -f "$ROOT/$1" ]; then
    printf 'required file missing: %s\n' "$1" >&2
    exit 1
  fi
}

for dir in docs docs/reference include/yai system system/daemon system/engine_bridge cmd/yai cmd/yaid proto/schemas proto/fixtures tests tools/checks vendor; do
  require_dir "$dir"
done

for dir in tests/unit tests/integration tests/adversarial tests/smoke; do
  require_dir "$dir"
done

for root in base subject case op control effect observation store index graph memory projection reconcile; do
  require_dir "include/yai/$root"
  require_dir "system/$root"
done

require_dir "include/yai/daemon"
require_dir "system/internal"
require_file "include/yai/yai.h"
require_file "README.md"
require_file "VERSION"
require_file "Makefile"
require_file "cmd/yai/Cargo.toml"
require_file "cmd/yai/src/main.rs"
require_file "cmd/yaid/main.c"
require_file "engine/Cargo.toml"
require_file "engine/README.md"
require_file "engine/yai-engine/Cargo.toml"
require_file "engine/yai-engine-ffi/Cargo.toml"
require_file "system/daemon/ipc.c"
require_file "system/daemon/core_loop.c"
require_file "system/daemon/daemon_status.c"
require_file "system/engine_bridge/rust_engine_backend.c"
require_file "tests/smoke/minimum-loop/test_minimum_loop.c"
require_file "tests/smoke/persistent-journal/test_persistent_journal.c"
require_file "tests/smoke/control-gate/test_control_gate.c"
require_file "tests/smoke/filesystem-carrier/test_filesystem_carrier.c"
require_file "tests/smoke/graph-reconstruction/test_graph_reconstruction.c"

printf 'check-required-layout: ok\n'
