#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${YAI_QUAL_ROOT:-}" ]]; then
  YAI_QUAL_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
fi

YAI_BIN="${YAI_BIN:-$YAI_QUAL_ROOT/build/bin/yai}"
YAI_DAEMON_BIN="${YAI_DAEMON_BIN:-$YAI_QUAL_ROOT/build/bin/yai-daemon}"
YAI_QUAL_TMP_ROOT="${YAI_QUAL_TMP_ROOT:-${TMPDIR:-/tmp}/yai-qualification}"
mkdir -p "$YAI_QUAL_TMP_ROOT"

yai_qual_log() {
  printf '[qualification] %s\n' "$*"
}

yai_qual_require_bins() {
  local need=()
  [[ -x "$YAI_BIN" ]] || need+=(yai)
  [[ -x "$YAI_DAEMON_BIN" ]] || need+=(yai-daemon)
  if (( ${#need[@]} > 0 )); then
    yai_qual_log "building missing binaries: ${need[*]}"
    make -C "$YAI_QUAL_ROOT" "${need[@]}" >/dev/null
  fi
}

yai_qual_new_home() {
  mktemp -d "$YAI_QUAL_TMP_ROOT/home.XXXXXX"
}

yai_qual_runtime_start() {
  local home="$1"
  local sock="$2"
  local log_file="$3"
  HOME="$home" YAI_RUNTIME_INGRESS="$sock" "$YAI_BIN" down >/dev/null 2>&1 || true
  rm -f "$sock" >/dev/null 2>&1 || true
  (cd "$YAI_QUAL_ROOT" && HOME="$home" YAI_RUNTIME_INGRESS="$sock" "$YAI_BIN" >"$log_file" 2>&1) &
  echo $!
}

yai_qual_wait_socket() {
  local sock="$1"
  local tries="${2:-80}"
  for _ in $(seq 1 "$tries"); do
    [[ -S "$sock" ]] && return 0
    sleep 0.1
  done
  return 1
}

yai_qual_runtime_stop() {
  local home="$1"
  local sock="$2"
  local pid="${3:-}"
  HOME="$home" YAI_RUNTIME_INGRESS="$sock" "$YAI_BIN" down >/dev/null 2>&1 || true
  if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
    kill "$pid" >/dev/null 2>&1 || true
    wait "$pid" >/dev/null 2>&1 || true
  fi
}
