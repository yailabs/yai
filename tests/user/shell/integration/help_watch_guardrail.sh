#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CLI="$REPO/dist/bin/yai"
TMP="$(mktemp -d /tmp/yai-cli-help-watch-XXXXXX)"
trap 'rm -rf "$TMP"' EXIT

"$CLI" help >"$TMP/help.txt" 2>&1
rg -n "^YAI Command Surface$" "$TMP/help.txt" >/dev/null
rg -n "^Usage:$" "$TMP/help.txt" >/dev/null
rg -n "^Main Surface Commands:$" "$TMP/help.txt" >/dev/null
rg -n "^  watch[[:space:]]+watch mode$" "$TMP/help.txt" >/dev/null

"$CLI" help watch >"$TMP/help_watch.txt" 2>&1
rg -n "^watch$" "$TMP/help_watch.txt" >/dev/null
rg -n "^Topics:$" "$TMP/help_watch.txt" >/dev/null
rg -n "^  runtime[[:space:]]+watch runtime reachability loops$" "$TMP/help_watch.txt" >/dev/null
rg -n "^  source[[:space:]]+watch source-plane status loops$" "$TMP/help_watch.txt" >/dev/null

set +e
"$CLI" help watch nonsense >"$TMP/watch_topic_bad.txt" 2>&1
RC_WATCH_BAD=$?
set -e
if [[ "$RC_WATCH_BAD" -ne 20 ]]; then
  echo "expected rc=20 for unknown watch topic, got $RC_WATCH_BAD"
  cat "$TMP/watch_topic_bad.txt"
  exit 1
fi
rg -n "^help$" "$TMP/watch_topic_bad.txt" >/dev/null
rg -n "^BAD ARGS$" "$TMP/watch_topic_bad.txt" >/dev/null

"$CLI" help watch runtime >"$TMP/help_watch_root.txt" 2>&1
rg -n "^watch runtime$" "$TMP/help_watch_root.txt" >/dev/null
rg -n "^Usage:$" "$TMP/help_watch_root.txt" >/dev/null

"$CLI" help watch source >"$TMP/help_watch_source.txt" 2>&1
rg -n "^watch source$" "$TMP/help_watch_source.txt" >/dev/null
rg -n "yai watch source status" "$TMP/help_watch_source.txt" >/dev/null

YAI_PAGER=1 "$CLI" --no-pager help watch >"$TMP/help_no_pager.txt" 2>&1
rg -n "^watch$" "$TMP/help_no_pager.txt" >/dev/null

set +e
NO_COLOR=1 "$CLI" watch runtime ping --count 2 --interval 0.05 --no-clear >"$TMP/watch.txt" 2>&1
RC_WATCH=$?
set -e
if [[ "$RC_WATCH" -ne 0 ]]; then
  echo "unexpected watch rc=$RC_WATCH"
  cat "$TMP/watch.txt"
  exit 1
fi
rg -n "runtime ping" "$TMP/watch.txt" >/dev/null

set +e
NO_COLOR=1 "$CLI" watch source --count 1 --interval 0.05 --no-clear >"$TMP/watch_source.txt" 2>&1
RC_WATCH_SOURCE=$?
set -e
if [[ "$RC_WATCH_SOURCE" -ne 0 ]]; then
  echo "unexpected watch source rc=$RC_WATCH_SOURCE"
  cat "$TMP/watch_source.txt"
  exit 1
fi
rg -n "source status" "$TMP/watch_source.txt" >/dev/null

echo "help_watch_guardrail: ok"
