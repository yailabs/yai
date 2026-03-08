#!/usr/bin/env bash
set -euo pipefail

# Resolve canonical YAI binary for the single-entrypoint runtime topology.
# Usage:
#   yai_resolve_bin "<repo_root>" -> yai path

yai_resolve_bin() {
  local root="${1:-}"

  if [[ -n "${BIN:-}" && -x "${BIN:-}" ]]; then
    echo "$BIN"
    return 0
  fi

  if [[ -n "${YAI_BIN:-}" && -x "${YAI_BIN:-}" ]]; then
    echo "$YAI_BIN"
    return 0
  fi

  if [[ -n "$root" && -x "$root/build/bin/yai" ]]; then
    echo "$root/build/bin/yai"
    return 0
  fi

  if command -v yai >/dev/null 2>&1; then
    command -v yai
    return 0
  fi

  if [[ -x "$HOME/.yai/artifacts/yai/bin/yai" ]]; then
    echo "$HOME/.yai/artifacts/yai/bin/yai"
    return 0
  fi

  return 1
}