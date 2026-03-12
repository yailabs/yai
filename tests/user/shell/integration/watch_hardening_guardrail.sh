#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CLI="$REPO/dist/bin/yai"
TMP="$(mktemp -d /tmp/yai-cli-watch-hardening-XXXXXX)"
trap 'rm -rf "$TMP"' EXIT

set +e
NO_COLOR=1 "$CLI" watch runtime --count 2 --interval 0.05 >"$TMP/watch_root.txt" 2>&1
RC_ROOT=$?
set -e
if [[ "$RC_ROOT" -ne 0 ]]; then
  echo "watch runtime should exit 0, got $RC_ROOT"
  cat "$TMP/watch_root.txt"
  exit 1
fi
rg -n "runtime ping" "$TMP/watch_root.txt" >/dev/null
if rg -n "ERR[[:space:]]+help|help\\)" "$TMP/watch_root.txt" >/dev/null; then
  echo "watch runtime fallback mismatch detected"
  cat "$TMP/watch_root.txt"
  exit 1
fi
if rg -n $'\x1b\\[' "$TMP/watch_root.txt" >/dev/null; then
  echo "NO_COLOR watch output contains ansi sequences"
  cat "$TMP/watch_root.txt"
  exit 1
fi

set +e
NO_COLOR=1 "$CLI" watch gov decision --count 1 --interval 0.05 >"$TMP/watch_gov.txt" 2>&1
RC_GOV=$?
set -e
if [[ "$RC_GOV" -ne 0 ]]; then
  echo "watch gov decision should exit 0, got $RC_GOV"
  cat "$TMP/watch_gov.txt"
  exit 1
fi
rg -n "gov decision status" "$TMP/watch_gov.txt" >/dev/null

set +e
NO_COLOR=1 "$CLI" watch source --count 1 --interval 0.05 >"$TMP/watch_source.txt" 2>&1
RC_SOURCE=$?
set -e
if [[ "$RC_SOURCE" -ne 0 ]]; then
  echo "watch source should exit 0, got $RC_SOURCE"
  cat "$TMP/watch_source.txt"
  exit 1
fi
rg -n "source status" "$TMP/watch_source.txt" >/dev/null

echo "watch_hardening_guardrail: ok"
