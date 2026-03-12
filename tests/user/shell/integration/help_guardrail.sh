#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CLI="$REPO/dist/bin/yai"
TMP="$(mktemp -d /tmp/yai-cli-help-v1-XXXXXX)"
trap 'rm -rf "$TMP"' EXIT

"$CLI" help --groups >"$TMP/groups.txt" 2>&1
rg -n "^YAI Command Surface$" "$TMP/groups.txt" >/dev/null
rg -n "^Main Surface Commands:$" "$TMP/groups.txt" >/dev/null
rg -n "ws|source|run|gov|verify|inspect|bundle|config|doctor|watch|help|version" "$TMP/groups.txt" >/dev/null

"$CLI" help run >"$TMP/help_run.txt" 2>&1
rg -n "^run$" "$TMP/help_run.txt" >/dev/null
rg -n "^Topics:$" "$TMP/help_run.txt" >/dev/null
rg -n "runtime|lifecycle|control" "$TMP/help_run.txt" >/dev/null

"$CLI" help ws >"$TMP/help_ws.txt" 2>&1
rg -n "^ws$" "$TMP/help_ws.txt" >/dev/null
rg -n "graph|db|data|cognition|policy|domain|recovery|debug|query" "$TMP/help_ws.txt" >/dev/null
rg -n "set|switch|unset|clear|current|status|inspect|domain get|domain set|policy dry-run|policy attach|policy activate|policy detach|policy effective|debug resolution|run <action>" "$TMP/help_ws.txt" >/dev/null

"$CLI" help ws graph >"$TMP/help_ws_graph.txt" 2>&1
rg -n "^ws graph$" "$TMP/help_ws_graph.txt" >/dev/null
rg -n "summary|container|governance|decision|evidence|authority|artifact|lineage|recent" "$TMP/help_ws_graph.txt" >/dev/null

"$CLI" help ws db >"$TMP/help_ws_db.txt" 2>&1
rg -n "^ws db$" "$TMP/help_ws_db.txt" >/dev/null
rg -n "status|bindings|stores|classes|count|tail" "$TMP/help_ws_db.txt" >/dev/null

"$CLI" help ws data >"$TMP/help_ws_data.txt" 2>&1
rg -n "^ws data$" "$TMP/help_ws_data.txt" >/dev/null
rg -n "events|evidence|governance|authority|artifacts|enforcement" "$TMP/help_ws_data.txt" >/dev/null

"$CLI" help ws cognition >"$TMP/help_ws_cognition.txt" 2>&1
rg -n "^ws cognition$" "$TMP/help_ws_cognition.txt" >/dev/null
rg -n "status|transient|memory|providers|context" "$TMP/help_ws_cognition.txt" >/dev/null

"$CLI" help source >"$TMP/help_source.txt" 2>&1
rg -n "^source$" "$TMP/help_source.txt" >/dev/null
rg -n "enroll|attach|list|status|inspect|retry-drain" "$TMP/help_source.txt" >/dev/null

"$CLI" help source enroll >"$TMP/help_source_enroll.txt" 2>&1
rg -n "^source enroll$" "$TMP/help_source_enroll.txt" >/dev/null
rg -n "yai source enroll <source-label>" "$TMP/help_source_enroll.txt" >/dev/null

set +e
"$CLI" run >"$TMP/run_missing_subcommand.txt" 2>&1
RC_RUN=$?
set -e
if [[ "$RC_RUN" -ne 20 ]]; then
  echo "expected rc=20 for missing run subcommand, got $RC_RUN"
  cat "$TMP/run_missing_subcommand.txt"
  exit 1
fi
rg -n "^BAD ARGS$" "$TMP/run_missing_subcommand.txt" >/dev/null

set +e
"$CLI" foo ping >"$TMP/unknown_group.txt" 2>&1
RC_GROUP=$?
set -e
if [[ "$RC_GROUP" -ne 20 ]]; then
  echo "expected rc=20 for unknown group, got $RC_GROUP"
  cat "$TMP/unknown_group.txt"
  exit 1
fi
rg -n "^BAD ARGS$" "$TMP/unknown_group.txt" >/dev/null
rg -n "^Unknown command group: foo$" "$TMP/unknown_group.txt" >/dev/null
rg -n "^Hint: Run: yai help --groups$" "$TMP/unknown_group.txt" >/dev/null

"$CLI" help gov decision >"$TMP/help_gov_decision.txt" 2>&1
rg -n "^gov decision$" "$TMP/help_gov_decision.txt" >/dev/null
rg -n "^Operations:$" "$TMP/help_gov_decision.txt" >/dev/null
rg -n "make|status|trace" "$TMP/help_gov_decision.txt" >/dev/null

echo "help_guardrail: ok"
