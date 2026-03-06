#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
YAI_BIN="${YAI_BIN:-$ROOT/../yai-cli/dist/bin/yai}"

if [[ ! -x "$YAI_BIN" ]]; then
  echo "missing cli binary: $YAI_BIN"
  exit 1
fi

WS1="ws_contract_01"
WS2="ws_contract_02"
WSR="ws_contract_root"
ROOT_PATH="/tmp/yai-ws-contract-root"

"$YAI_BIN" lifecycle down >/dev/null 2>&1 || true
"$YAI_BIN" lifecycle up >/dev/null 2>&1 || true

"$YAI_BIN" --json kernel ws create --ws-id "$WS1" >/tmp/yai_ws1_create.txt 2>&1 || true
"$YAI_BIN" --json kernel ws_status --ws-id "$WS1" >/tmp/yai_ws1_status.txt 2>&1 || true
"$YAI_BIN" --json kernel ws_list >/tmp/yai_ws_list_1.txt 2>&1 || true
if grep -q "SERVER_UNAVAILABLE" /tmp/yai_ws1_create.txt; then
  echo "workspace_runtime_contract_v1: skipped (runtime unavailable)"
  exit 0
fi
grep -q "$WS1" /tmp/yai_ws1_status.txt
grep -q "$WS1" /tmp/yai_ws_list_1.txt

"$YAI_BIN" --json kernel ws create --ws-id "$WS2" >/tmp/yai_ws2_create.txt 2>&1 || true
"$YAI_BIN" --json kernel ws_status --ws-id "$WS2" >/tmp/yai_ws2_status.txt 2>&1 || true
"$YAI_BIN" --json kernel ws_list >/tmp/yai_ws_list_2.txt 2>&1 || true
grep -q "$WS2" /tmp/yai_ws2_status.txt
grep -q "$WS2" /tmp/yai_ws_list_2.txt

"$YAI_BIN" --json kernel ws create --ws-id "$WSR" --root "$ROOT_PATH" >/tmp/yai_wsr_create.txt 2>&1 || true
"$YAI_BIN" --json kernel ws_status --ws-id "$WSR" >/tmp/yai_wsr_status.txt 2>&1 || true
grep -q "$ROOT_PATH" /tmp/yai_wsr_status.txt

"$YAI_BIN" --json kernel ws destroy --ws-id "$WS1" >/tmp/yai_ws1_destroy.txt 2>&1 || true
"$YAI_BIN" --json kernel ws_status --ws-id "$WS1" >/tmp/yai_ws1_status_after_destroy.txt 2>&1 || true
grep -q "missing" /tmp/yai_ws1_status_after_destroy.txt || grep -q "exists\":false" /tmp/yai_ws1_status_after_destroy.txt

"$YAI_BIN" --json kernel ws destroy --ws-id "$WS2" >/tmp/yai_ws2_destroy.txt 2>&1 || true
"$YAI_BIN" --json kernel ws destroy --ws-id "$WSR" >/tmp/yai_wsr_destroy.txt 2>&1 || true

echo "workspace_runtime_contract_v1: ok"
