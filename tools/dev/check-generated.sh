#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GOV_ROOT="$ROOT/governance"
PROTOCOL_LIB_ROOT="$ROOT/lib/control/contracts/schema"
PROTOCOL_INCLUDE_ROOT="$ROOT/include/yai/protocol/contracts"
FORMAL_ROOT="$ROOT/formal"

if [[ ! -d "$PROTOCOL_LIB_ROOT" || ! -d "$PROTOCOL_INCLUDE_ROOT" ]]; then
  echo "no contract/schema source found (expected lib/protocol + include/yai/protocol/contracts)" >&2
  exit 2
fi

SPEC_CONTRACTS="$PROTOCOL_LIB_ROOT/vault/vault_abi.json"
SPEC_LEGACY="$GOV_ROOT/vault/model/schema/vault_abi.json"
if [[ -f "$SPEC_CONTRACTS" ]]; then
  SPEC="$SPEC_CONTRACTS"
elif [[ -f "$SPEC_LEGACY" ]]; then
  SPEC="$SPEC_LEGACY"
else
  echo "vault ABI spec not found under $GOV_ROOT" >&2
  exit 2
fi

ACTUAL_HEADER="$PROTOCOL_INCLUDE_ROOT/yai_vault_abi.h"
ACTUAL_TLA="$FORMAL_ROOT/modules/yai_ids.tla"
if [[ ! -f "$ACTUAL_HEADER" || ! -f "$ACTUAL_TLA" ]]; then
  echo "generated targets missing under include/lib protocol contracts + control/assurance/modules canonical roots" >&2
  exit 2
fi

GEN="$ROOT/tools/dev/gen-vault-abi"
TMP_DIR="$(mktemp -d)"
cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

GEN_OUT="$TMP_DIR"
mkdir -p "$GEN_OUT/lib/control/contracts/schema" "$GEN_OUT/include/yai/protocol/contracts" "$GEN_OUT/control/assurance/modules"
"$GEN" --spec "$SPEC" --out-dir "$GEN_OUT"
TMP_HEADER="$TMP_DIR/include/yai/control/contracts/yai_vault_abi.h"
TMP_TLA="$TMP_DIR/control/assurance/modules/yai_ids.tla"

strip_generated() {
  sed -E '/^\/\* Generated: /d; /^\\\* Generated: /d' "$1"
}

compare_file() {
  local expected="$1"
  local generated="$2"
  local label="$3"
  if ! diff -u <(strip_generated "$expected") <(strip_generated "$generated") >/dev/null; then
    echo "drift detected in $label" >&2
    diff -u <(strip_generated "$expected") <(strip_generated "$generated") || true
    return 1
  fi
}

compare_file "$ACTUAL_HEADER" "$TMP_HEADER" "yai_vault_abi.h"
compare_file "$ACTUAL_TLA" "$TMP_TLA" "yai_ids.tla"

echo "ok: generated artifacts match (protocol contracts + control/assurance/modules)"
