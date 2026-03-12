#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CLI="$REPO/dist/bin/yai"
TMP="$(mktemp -d /tmp/yai-cli-output-v1-XXXXXX)"
trap 'rm -rf "$TMP"' EXIT

"$CLI" down >/dev/null 2>&1 || true
set +e
"$CLI" up >"$TMP/lifecycle_up.txt" 2>&1
RC_UP=$?
set -e
if [[ "$RC_UP" -ne 0 && "$RC_UP" -ne 40 ]]; then
  echo "expected up rc 0 or 40, got $RC_UP"
  cat "$TMP/lifecycle_up.txt"
  exit 1
fi

set +e
"$CLI" runtime ping >"$TMP/root_ping_tty_maybe.txt" 2>&1
RC_PING1=$?
set -e
if [[ "$RC_PING1" -ne 0 && "$RC_PING1" -ne 40 ]]; then
  echo "expected runtime ping rc 0 or 40, got $RC_PING1"
  cat "$TMP/root_ping_tty_maybe.txt"
  exit 1
fi
set +e
NO_COLOR=1 "$CLI" runtime ping >"$TMP/root_ping_nocolor.txt" 2>&1
RC_PING2=$?
set -e
if [[ "$RC_PING2" -ne 0 && "$RC_PING2" -ne 40 ]]; then
  echo "expected NO_COLOR runtime ping rc 0 or 40, got $RC_PING2"
  cat "$TMP/root_ping_nocolor.txt"
  exit 1
fi
if rg -n $'\x1b\\[' "$TMP/root_ping_nocolor.txt" >/dev/null; then
  echo "NO_COLOR output contains ansi sequences"
  cat "$TMP/root_ping_nocolor.txt"
  exit 1
fi

set +e
"$CLI" runtime ping >"$TMP/root_ping.txt" 2>&1
RC_PING3=$?
set -e
if [[ "$RC_PING3" -ne 0 && "$RC_PING3" -ne 40 ]]; then
  echo "expected runtime ping rc 0 or 40, got $RC_PING3"
  cat "$TMP/root_ping.txt"
  exit 1
fi
if rg -n "command_id=|exec_reply:|control_call:|ok: OK:" "$TMP/root_ping.txt" >/dev/null; then
  echo "default output contains forbidden tokens"
  cat "$TMP/root_ping.txt"
  exit 1
fi
LINES="$(wc -l < "$TMP/root_ping.txt")"
if [[ "$LINES" -lt 2 || "$LINES" -gt 4 ]]; then
  echo "default output should be 2-4 lines, got $LINES"
  cat "$TMP/root_ping.txt"
  exit 1
fi

if command -v jq >/dev/null 2>&1; then
  set +e
  "$CLI" --json runtime ping >"$TMP/root_ping.json" 2>&1
  RC_JSON=0
  set -e
  if [[ "$RC_JSON" -ne 0 && "$RC_JSON" -ne 40 ]]; then
    echo "expected --json runtime ping rc 0 or 40, got $RC_JSON"
    cat "$TMP/root_ping.json"
    exit 1
  fi
  jq -e . "$TMP/root_ping.json" >/dev/null
  if rg -n $'\x1b\\[' "$TMP/root_ping.json" >/dev/null; then
    echo "--json output contains ansi sequences"
    cat "$TMP/root_ping.json"
    exit 1
  fi
fi

set +e
"$CLI" --verbose-contract runtime ping >"$TMP/root_ping_contract.txt" 2>&1
RC_CONTRACT=$?
set -e
if [[ "$RC_CONTRACT" -ne 0 && "$RC_CONTRACT" -ne 40 ]]; then
  echo "expected --verbose-contract runtime ping rc 0 or 40, got $RC_CONTRACT"
  cat "$TMP/root_ping_contract.txt"
  exit 1
fi
rg -n "^Contract verbose$" "$TMP/root_ping_contract.txt" >/dev/null
rg -n "^Control call$" "$TMP/root_ping_contract.txt" >/dev/null
rg -n "^Exec reply$" "$TMP/root_ping_contract.txt" >/dev/null
rg -n "^Raw payloads$" "$TMP/root_ping_contract.txt" >/dev/null
rg -n "^control_call_json:$" "$TMP/root_ping_contract.txt" >/dev/null
rg -n "^exec_reply_json:$" "$TMP/root_ping_contract.txt" >/dev/null
awk '
  /^control_call_json:$/ {mode=1; next}
  /^exec_reply_json:$/   {mode=2; next}
  mode==1 {print > "'"$TMP"'/control_call.json"}
  mode==2 {print > "'"$TMP"'/exec_reply.json"}
' "$TMP/root_ping_contract.txt"
if command -v jq >/dev/null 2>&1; then
  jq -e . "$TMP/control_call.json" >/dev/null
  jq -e . "$TMP/exec_reply.json" >/dev/null
fi

set +e
"$CLI" boot artifact_audit >"$TMP/nyi.txt" 2>&1
RC_NYI=$?
set -e
if [[ "$RC_NYI" -eq 0 ]]; then
  rg -n "^OK$" "$TMP/nyi.txt" >/dev/null
elif [[ "$RC_NYI" -eq 10 ]]; then
  rg -n "^NOT IMPLEMENTED$" "$TMP/nyi.txt" >/dev/null
elif [[ "$RC_NYI" -eq 20 ]]; then
  rg -n "^BAD ARGS$" "$TMP/nyi.txt" >/dev/null
elif [[ "$RC_NYI" -eq 40 ]]; then
  rg -n "^SERVER UNAVAILABLE$" "$TMP/nyi.txt" >/dev/null
else
  echo "expected rc=0/10/20/40 for compatibility surface, got $RC_NYI"
  cat "$TMP/nyi.txt"
  exit 1
fi

"$CLI" down >/dev/null 2>&1 || true
set +e
"$CLI" runtime ping >"$TMP/root_ping_down.txt" 2>&1
RC_DOWN=$?
set -e
if [[ "$RC_DOWN" -ne 40 ]]; then
  echo "expected rc=40 for runtime down, got $RC_DOWN"
  cat "$TMP/root_ping_down.txt"
  exit 1
fi
rg -n "^SERVER UNAVAILABLE$" "$TMP/root_ping_down.txt" >/dev/null
rg -n "^Hint: Start the service with: yai up$" "$TMP/root_ping_down.txt" >/dev/null

set +e
"$CLI" nonsense >"$TMP/bad_args.txt" 2>&1
RC_BAD=$?
set -e
if [[ "$RC_BAD" -ne 20 ]]; then
  echo "expected rc=20 for bad args, got $RC_BAD"
  cat "$TMP/bad_args.txt"
  exit 1
fi
rg -n "^BAD ARGS$" "$TMP/bad_args.txt" >/dev/null

YAI_PAGER=off "$CLI" help --groups >"$TMP/groups_no_pager.txt" 2>&1
[[ -s "$TMP/groups_no_pager.txt" ]]

NO_COLOR=1 "$CLI" watch runtime ping --interval 0.1 --count 3 >"$TMP/watch_non_tty.txt" 2>&1
LINES_WATCH="$(wc -l < "$TMP/watch_non_tty.txt")"
if [[ "$LINES_WATCH" -lt 3 ]]; then
  echo "watch non-tty fallback should print at least 3 lines"
  cat "$TMP/watch_non_tty.txt"
  exit 1
fi
if rg -n $'\x1b\\[' "$TMP/watch_non_tty.txt" >/dev/null; then
  echo "watch non-tty output contains ansi sequences"
  cat "$TMP/watch_non_tty.txt"
  exit 1
fi

echo "output_contract_v1_guardrail: ok"
