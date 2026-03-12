#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CLI="$REPO/dist/bin/yai"
TMP="$(mktemp -d /tmp/yai-cli-truth-surface-wsv83-XXXXXX)"
WS_ID="wsv83_diag_${RANDOM}"
trap 'rm -rf "$TMP"' EXIT

extract_kv() {
  local file="$1"
  local key="$2"
  awk -v k="$key" '
    $1==k {
      $1="";
      gsub(/^ +/, "", $0);
      print $0;
      exit 0;
    }' "$file"
}

"$CLI" down >/dev/null 2>&1 || true
set +e
NO_COLOR=1 "$CLI" up >"$TMP/lifecycle_up.txt" 2>&1
RC_UP=$?
set -e
if [[ "$RC_UP" -ne 0 && "$RC_UP" -ne 40 ]]; then
  echo "expected up rc 0 or 40, got $RC_UP"
  cat "$TMP/lifecycle_up.txt"
  exit 1
fi
if [[ "$RC_UP" -eq 40 ]]; then
  echo "truth_surface_coherence_wsv83_guardrail: skipped (runtime unavailable)"
  exit 0
fi

NO_COLOR=1 "$CLI" ws unset >/dev/null 2>&1 || true

NO_COLOR=1 "$CLI" doctor runtime >"$TMP/doctor_runtime_unbound.txt" 2>&1
NO_COLOR=1 "$CLI" status >"$TMP/status_runtime_unbound.txt" 2>&1
NO_COLOR=1 "$CLI" inspect runtime >"$TMP/inspect_runtime_unbound.txt" 2>&1

for f in doctor_runtime_unbound status_runtime_unbound inspect_runtime_unbound; do
  rg -n "^Runtime status$" "$TMP/$f.txt" >/dev/null
  rg -n "^Baseline$|^Container binding$" "$TMP/$f.txt" >/dev/null
  rg -n "^  Liveness +|^  Readiness +" "$TMP/$f.txt" >/dev/null
done

L1="$(extract_kv "$TMP/doctor_runtime_unbound.txt" "Liveness")"
R1="$(extract_kv "$TMP/doctor_runtime_unbound.txt" "Readiness")"
L2="$(extract_kv "$TMP/status_runtime_unbound.txt" "Liveness")"
R2="$(extract_kv "$TMP/status_runtime_unbound.txt" "Readiness")"
L3="$(extract_kv "$TMP/inspect_runtime_unbound.txt" "Liveness")"
R3="$(extract_kv "$TMP/inspect_runtime_unbound.txt" "Readiness")"
if [[ "$L1" != "$L2" || "$L1" != "$L3" || "$R1" != "$R2" || "$R1" != "$R3" ]]; then
  echo "runtime surface contradiction detected"
  echo "doctor:  L=$L1 R=$R1"
  echo "status:  L=$L2 R=$R2"
  echo "inspect: L=$L3 R=$R3"
  exit 1
fi

rg -n "^  Container selected +no$" "$TMP/doctor_runtime_unbound.txt" >/dev/null
rg -n "^  Container bound +no$" "$TMP/doctor_runtime_unbound.txt" >/dev/null

NO_COLOR=1 "$CLI" ws create "$WS_ID" >/dev/null
NO_COLOR=1 "$CLI" ws set "$WS_ID" >/dev/null
NO_COLOR=1 "$CLI" doctor runtime >"$TMP/doctor_runtime_bound.txt" 2>&1
rg -n "^  Container selected +yes$" "$TMP/doctor_runtime_bound.txt" >/dev/null
rg -n "^  Container bound +yes$|^  Container bound +unknown$" "$TMP/doctor_runtime_bound.txt" >/dev/null

NO_COLOR=1 "$CLI" ws unset >/dev/null 2>&1 || true
set +e
NO_COLOR=1 "$CLI" ws graph summary >"$TMP/ws_graph_no_ws.txt" 2>&1
RC_GRAPH=$?
set -e
if [[ "$RC_GRAPH" -ne 20 ]]; then
  echo "expected ws graph summary rc 20 with no container, got $RC_GRAPH"
  cat "$TMP/ws_graph_no_ws.txt"
  exit 1
fi
rg -n "^BAD ARGS$" "$TMP/ws_graph_no_ws.txt" >/dev/null
rg -n "^Hint: Run: yai ws set <ws-id>$" "$TMP/ws_graph_no_ws.txt" >/dev/null

NO_COLOR=1 "$CLI" down >/dev/null 2>&1 || true
set +e
NO_COLOR=1 "$CLI" ws status >"$TMP/ws_status_down.txt" 2>&1
RC_WS_DOWN=$?
set -e
if [[ "$RC_WS_DOWN" -ne 40 ]]; then
  echo "expected ws status rc 40 when runtime is down, got $RC_WS_DOWN"
  cat "$TMP/ws_status_down.txt"
  exit 1
fi
rg -n "^SERVER UNAVAILABLE$" "$TMP/ws_status_down.txt" >/dev/null
rg -n "^Hint: Run: yai up$|^Hint: Start the service with: yai up$" "$TMP/ws_status_down.txt" >/dev/null

echo "truth_surface_coherence_wsv83_guardrail: ok"
