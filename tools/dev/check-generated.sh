#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SPEC_CONTRACTS="$ROOT/deps/yai-law/contracts/vault/schema/vault_abi.json"
SPEC_LEGACY="$ROOT/deps/yai-law/specs/vault/schema/vault_abi.json"
if [[ -f "$SPEC_CONTRACTS" ]]; then
  SPEC="$SPEC_CONTRACTS"
else
  SPEC="$SPEC_LEGACY"
fi
GEN="$ROOT/tools/dev/gen-vault-abi"

TMP_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

"$GEN" --spec "$SPEC" --out-dir "$TMP_DIR"

strip_generated() {
  sed -e '/^\/\* Generated:/d' -e '/^\\\* Generated:/d'
}

DIFF_A=$(diff -u <(strip_generated < "$ROOT/deps/yai-law/contracts/vault/include/yai_vault_abi.h") \
                 <(strip_generated < "$TMP_DIR/deps/yai-law/contracts/vault/include/yai_vault_abi.h") || true)
if [[ -n "$DIFF_A" ]]; then
  echo "ERROR: yai_vault_abi.h drift"
  echo "$DIFF_A"
  exit 1
fi

DIFF_B=$(diff -u <(strip_generated < "$ROOT/deps/yai-law/formal/tla/LAW_IDS.tla") \
                 <(strip_generated < "$TMP_DIR/deps/yai-law/formal/tla/LAW_IDS.tla") || true)
if [[ -n "$DIFF_B" ]]; then
  echo "ERROR: LAW_IDS.tla drift"
  echo "$DIFF_B"
  exit 1
fi

echo "OK: generated files are in sync"

true
