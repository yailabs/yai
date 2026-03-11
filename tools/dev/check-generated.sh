#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GOV_ROOT="$ROOT/governance"
FORMAL_ROOT="$ROOT/formal"

if [[ ! -d "$GOV_ROOT/contracts" ]]; then
  echo "no contract/schema source found (expected governance/)" >&2
  exit 2
fi

SPEC_CONTRACTS="$GOV_ROOT/contracts/vault/schema/vault_abi.json"
SPEC_LEGACY="$GOV_ROOT/specs/vault/schema/vault_abi.json"
if [[ -f "$SPEC_CONTRACTS" ]]; then
  SPEC="$SPEC_CONTRACTS"
elif [[ -f "$SPEC_LEGACY" ]]; then
  SPEC="$SPEC_LEGACY"
else
  echo "vault ABI spec not found under $GOV_ROOT" >&2
  exit 2
fi

ACTUAL_HEADER="$GOV_ROOT/contracts/vault/include/yai_vault_abi.h"
ACTUAL_TLA="$FORMAL_ROOT/tla/GOVERNANCE_IDS.tla"
if [[ ! -f "$ACTUAL_HEADER" || ! -f "$ACTUAL_TLA" ]]; then
  echo "generated targets missing under governance/contracts + formal/tla canonical roots" >&2
  exit 2
fi

GEN="$ROOT/tools/dev/gen-vault-abi"
TMP_DIR="$(mktemp -d)"
cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

if [[ "$GOV_ROOT" == "$ROOT/governance" ]]; then
  GEN_OUT="$TMP_DIR"
  mkdir -p "$GEN_OUT/governance/contracts" "$GEN_OUT/formal"
  "$GEN" --spec "$SPEC" --out-dir "$GEN_OUT"
  TMP_HEADER="$TMP_DIR/governance/contracts/vault/include/yai_vault_abi.h"
  TMP_TLA="$TMP_DIR/formal/tla/GOVERNANCE_IDS.tla"
else
  "$GEN" --spec "$SPEC" --out-dir "$TMP_DIR"
  TMP_HEADER="$TMP_DIR/governance/contracts/vault/include/yai_vault_abi.h"
  TMP_TLA="$TMP_DIR/formal/tla/GOVERNANCE_IDS.tla"
fi

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
compare_file "$ACTUAL_TLA" "$TMP_TLA" "GOVERNANCE_IDS.tla"

echo "ok: generated artifacts match (governance/contracts + formal/tla)"
