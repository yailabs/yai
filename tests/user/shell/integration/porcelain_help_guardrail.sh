#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CLI="$REPO/dist/bin/yai"
TMP="$(mktemp -d /tmp/yai-cli-help-guard-XXXXXX)"
trap 'rm -rf "$TMP"' EXIT

"$CLI" help --groups >"$TMP/groups.txt" 2>&1
[[ -s "$TMP/groups.txt" ]]
rg -n "ws|source|run|gov|verify|inspect|bundle|doctor|watch" "$TMP/groups.txt" >/dev/null

"$CLI" help run >"$TMP/run.txt" 2>&1
rg -n "^run$" "$TMP/run.txt" >/dev/null
rg -n "^Topics:$" "$TMP/run.txt" >/dev/null
rg -n "runtime|lifecycle|control" "$TMP/run.txt" >/dev/null

"$CLI" up >/dev/null 2>&1 || true
"$CLI" runtime ping >"$TMP/root_ping.txt" 2>&1 || true
if rg -n "unknown command group|try: yai help --groups|invalid_arguments_or_contract_violation" "$TMP/root_ping.txt" >/dev/null; then
  echo "help garbage detected on normal command path"
  cat "$TMP/root_ping.txt"
  exit 1
fi

echo "porcelain_help_guardrail: ok"
