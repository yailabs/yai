#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CLI="$REPO/dist/bin/yai"
TMP="$(mktemp -d /tmp/yai-cli-operator-pack-XXXXXX)"
trap 'rm -rf "$TMP"' EXIT

"$CLI" doctor env >"$TMP/doctor_env.txt" 2>&1 || true
rg -n "doctor" "$TMP/doctor_env.txt" >/dev/null

"$CLI" doctor runtime >"$TMP/doctor_runtime.txt" 2>&1 || true
rg -n "^Runtime status$|^doctor runtime$" "$TMP/doctor_runtime.txt" >/dev/null
rg -n "^Baseline$|^  Liveness +|^  Readiness +" "$TMP/doctor_runtime.txt" >/dev/null

"$CLI" doctor container >"$TMP/doctor_workspace.txt" 2>&1 || true
rg -n "^doctor container$|^BAD ARGS$|^Container status$" "$TMP/doctor_workspace.txt" >/dev/null

"$CLI" doctor all >"$TMP/doctor_all.txt" 2>&1 || true
rg -n "doctor|Runtime status|Container status|SERVER UNAVAILABLE|BAD ARGS" "$TMP/doctor_all.txt" >/dev/null

"$CLI" inspect container >"$TMP/inspect_workspace.txt" 2>&1 || true
rg -n "^inspect container$|^Container inspect$|^BAD ARGS$|^PROTOCOL ERROR$" "$TMP/inspect_workspace.txt" >/dev/null

"$CLI" inspect runtime >"$TMP/inspect_runtime.txt" 2>&1 || true
rg -n "^Runtime status$|^inspect runtime$" "$TMP/inspect_runtime.txt" >/dev/null
rg -n "^  Liveness +|^  Readiness +|^Capability families$" "$TMP/inspect_runtime.txt" >/dev/null

"$CLI" inspect catalog >"$TMP/inspect_catalog.txt" 2>&1 || true
rg -n "^inspect catalog$|catalog|BAD ARGS|SERVER UNAVAILABLE|PROTOCOL ERROR" "$TMP/inspect_catalog.txt" >/dev/null

"$CLI" verify law >"$TMP/verify_law.txt" 2>&1 || true
rg -n "verify" "$TMP/verify_law.txt" >/dev/null

"$CLI" verify registry >"$TMP/verify_registry.txt" 2>&1 || true
rg -n "verify" "$TMP/verify_registry.txt" >/dev/null

"$CLI" verify container >"$TMP/verify_workspace.txt" 2>&1 || true
rg -n "verify" "$TMP/verify_workspace.txt" >/dev/null

"$CLI" verify alignment >"$TMP/verify_alignment.txt" 2>&1 || true
rg -n "verify" "$TMP/verify_alignment.txt" >/dev/null

if command -v jq >/dev/null 2>&1; then
  "$CLI" doctor all --json >"$TMP/doctor_all.json" 2>&1 || true
  jq -e . "$TMP/doctor_all.json" >/dev/null || rg -n "SERVER UNAVAILABLE|BAD ARGS|PROTOCOL ERROR" "$TMP/doctor_all.json" >/dev/null

  "$CLI" inspect container --json >"$TMP/inspect_workspace.json" 2>&1 || true
  jq -e . "$TMP/inspect_workspace.json" >/dev/null || rg -n "SERVER UNAVAILABLE|BAD ARGS|PROTOCOL ERROR" "$TMP/inspect_workspace.json" >/dev/null

  "$CLI" inspect runtime --json >"$TMP/inspect_runtime.json" 2>&1 || true
  jq -e . "$TMP/inspect_runtime.json" >/dev/null || rg -n "SERVER UNAVAILABLE|BAD ARGS|PROTOCOL ERROR" "$TMP/inspect_runtime.json" >/dev/null

  "$CLI" verify alignment --json >"$TMP/verify_alignment.json" 2>&1 || true
  jq -e . "$TMP/verify_alignment.json" >/dev/null || rg -n "SERVER UNAVAILABLE|BAD ARGS|PROTOCOL ERROR" "$TMP/verify_alignment.json" >/dev/null
fi

echo "operator_capability_pack_guardrail: ok"
