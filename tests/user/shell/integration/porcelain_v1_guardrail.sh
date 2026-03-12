#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CLI="$REPO/dist/bin/yai"

TMP="$(mktemp -d /tmp/yai-cli-porcelain-XXXXXX)"
trap 'rm -rf "$TMP"' EXIT

"$CLI" down >/dev/null 2>&1 || true
set +e
"$CLI" up >"$TMP/up.txt" 2>&1
RC_UP=$?
set -e
if [[ "$RC_UP" -ne 0 && "$RC_UP" -ne 40 ]]; then
  echo "expected up rc 0 or 40, got $RC_UP"
  cat "$TMP/up.txt"
  exit 1
fi
rg -n "^runtime up$" "$TMP/up.txt" >/dev/null
if [[ "$RC_UP" -eq 0 ]]; then
  rg -n "^  Operation +up$|^  Runtime liveness +reachable$" "$TMP/up.txt" >/dev/null
else
  rg -n "^(RUNTIME NOT READY|SERVER UNAVAILABLE)$" "$TMP/up.txt" >/dev/null
fi

set +e
"$CLI" runtime ping >"$TMP/root_ping.txt" 2>&1
RC_PING=$?
set -e
if [[ "$RC_PING" -ne 0 && "$RC_PING" -ne 40 ]]; then
  echo "expected runtime ping rc 0 or 40, got $RC_PING"
  cat "$TMP/root_ping.txt"
  exit 1
fi
rg -n "^runtime ping$" "$TMP/root_ping.txt" >/dev/null
rg -n "^(OK|RUNTIME NOT READY|SERVER UNAVAILABLE)$" "$TMP/root_ping.txt" >/dev/null

set +e
"$CLI" ws status >"$TMP/ws_status.txt" 2>&1
RC_WS_STATUS=$?
set -e
if [[ "$RC_WS_STATUS" -ne 0 && "$RC_WS_STATUS" -ne 40 ]]; then
  echo "expected ws status rc 0 or 40, got $RC_WS_STATUS"
  cat "$TMP/ws_status.txt"
  exit 1
fi
rg -n "^container status$|^container\\.status$|^container status$" "$TMP/ws_status.txt" >/dev/null || true
if [[ "$RC_WS_STATUS" -eq 0 ]]; then
  rg -n "^Container status$" "$TMP/ws_status.txt" >/dev/null
  rg -n "^Binding$|^Runtime$|^Context$" "$TMP/ws_status.txt" >/dev/null
  rg -n "^  Active +yes$|^  Active +no$" "$TMP/ws_status.txt" >/dev/null
else
  rg -n "^SERVER UNAVAILABLE$" "$TMP/ws_status.txt" >/dev/null
fi

set +e
"$CLI" ws current >"$TMP/ws_current.txt" 2>&1
RC_WS_CURRENT=$?
set -e
if [[ "$RC_WS_CURRENT" -ne 0 && "$RC_WS_CURRENT" -ne 40 ]]; then
  echo "expected ws current rc 0 or 40, got $RC_WS_CURRENT"
  cat "$TMP/ws_current.txt"
  exit 1
fi
if [[ "$RC_WS_CURRENT" -eq 0 ]]; then
  rg -n "^Container current$" "$TMP/ws_current.txt" >/dev/null
  rg -n "^Identity$|^Session$" "$TMP/ws_current.txt" >/dev/null
  rg -n "^  Id +|^  Binding +" "$TMP/ws_current.txt" >/dev/null
else
  rg -n "^SERVER UNAVAILABLE$" "$TMP/ws_current.txt" >/dev/null
fi

set +e
"$CLI" ws clear >/dev/null 2>&1 || true
"$CLI" ws run payment.authorize provider=bank >"$TMP/ws_run_no_ws.txt" 2>&1
RC_WS_RUN_NO_WS=$?
set -e
if [[ "$RC_WS_RUN_NO_WS" -ne 20 && "$RC_WS_RUN_NO_WS" -ne 30 && "$RC_WS_RUN_NO_WS" -ne 40 ]]; then
  echo "expected ws run without active container to fail with rc=20/30/40, got $RC_WS_RUN_NO_WS"
  cat "$TMP/ws_run_no_ws.txt"
  exit 1
fi
if [[ "$RC_WS_RUN_NO_WS" -eq 20 ]]; then
  rg -n "^BAD ARGS$" "$TMP/ws_run_no_ws.txt" >/dev/null
  rg -n "No container selected|No active container binding for runtime execution|No active container selected for runtime execution" "$TMP/ws_run_no_ws.txt" >/dev/null
elif [[ "$RC_WS_RUN_NO_WS" -eq 30 ]]; then
  rg -n "^DENIED$" "$TMP/ws_run_no_ws.txt" >/dev/null
else
  rg -n "^SERVER UNAVAILABLE$" "$TMP/ws_run_no_ws.txt" >/dev/null
fi

set +e
"$CLI" --verbose runtime ping >"$TMP/verbose.txt" 2>&1
RC_VERBOSE=$?
set -e
if [[ "$RC_VERBOSE" -ne 0 && "$RC_VERBOSE" -ne 40 ]]; then
  echo "expected verbose runtime ping rc 0 or 40, got $RC_VERBOSE"
  cat "$TMP/verbose.txt"
  exit 1
fi
rg -n "^runtime ping$" "$TMP/verbose.txt" >/dev/null
rg -n "^Details:$" "$TMP/verbose.txt" >/dev/null
rg -n "^  status=" "$TMP/verbose.txt" >/dev/null
rg -n "^  code=" "$TMP/verbose.txt" >/dev/null
rg -n "^  reason=" "$TMP/verbose.txt" >/dev/null
rg -n "^  command_id=" "$TMP/verbose.txt" >/dev/null
rg -n "^  target_plane=" "$TMP/verbose.txt" >/dev/null

set +e
"$CLI" --json runtime ping >"$TMP/json.txt" 2>&1
RC_JSON=$?
set -e
if [[ "$RC_JSON" -ne 0 && "$RC_JSON" -ne 40 ]]; then
  echo "expected json runtime ping rc 0 or 40, got $RC_JSON"
  cat "$TMP/json.txt"
  exit 1
fi
jq -e . "$TMP/json.txt" >/dev/null

set +e
"$CLI" --verbose-contract runtime ping >"$TMP/contract.txt" 2>&1
RC_CONTRACT=$?
set -e
if [[ "$RC_CONTRACT" -ne 0 && "$RC_CONTRACT" -ne 40 ]]; then
  echo "expected verbose-contract runtime ping rc 0 or 40, got $RC_CONTRACT"
  cat "$TMP/contract.txt"
  exit 1
fi
rg -n "^control_call:$" "$TMP/contract.txt" >/dev/null
rg -n "^exec_reply:$" "$TMP/contract.txt" >/dev/null
awk '
  /^control_call:$/ {mode=1; next}
  /^exec_reply:$/   {mode=2; next}
  mode==1 {print > "'"$TMP"'/control_call.json"}
  mode==2 {print > "'"$TMP"'/exec_reply.json"}
' "$TMP/contract.txt"
if command -v jq >/dev/null 2>&1; then
  jq -e . "$TMP/control_call.json" >/dev/null
  jq -e . "$TMP/exec_reply.json" >/dev/null
fi

set +e
"$CLI" boot artifact_audit >"$TMP/nyi.txt" 2>&1
RC_NYI=$?
set -e
if [[ "$RC_NYI" -eq 10 ]]; then
  rg -n "^boot artifact audit$" "$TMP/nyi.txt" >/dev/null
  rg -n "^NOT IMPLEMENTED$" "$TMP/nyi.txt" >/dev/null
elif [[ "$RC_NYI" -eq 40 ]]; then
  rg -n "^boot artifact audit$" "$TMP/nyi.txt" >/dev/null
  rg -n "^SERVER UNAVAILABLE$" "$TMP/nyi.txt" >/dev/null
elif [[ "$RC_NYI" -eq 50 ]]; then
  rg -n "^boot artifact audit$" "$TMP/nyi.txt" >/dev/null
  rg -n "^INTERNAL ERROR$" "$TMP/nyi.txt" >/dev/null
else
  echo "expected rc=10 (nyi) or rc=40 (runtime unavailable) or rc=50 (internal), got $RC_NYI"
  cat "$TMP/nyi.txt"
  exit 1
fi

"$CLI" down >/dev/null 2>&1 || true
set +e
"$CLI" runtime ping >"$TMP/offline.txt" 2>&1
RC_OFF=$?
set -e
if [[ "$RC_OFF" -ne 40 ]]; then
  echo "expected rc=40 for server unavailable, got $RC_OFF"
  cat "$TMP/offline.txt"
  exit 1
fi
rg -n "^runtime ping$" "$TMP/offline.txt" >/dev/null
rg -n "^SERVER UNAVAILABLE$" "$TMP/offline.txt" >/dev/null

echo "porcelain_v1_guardrail: ok"
