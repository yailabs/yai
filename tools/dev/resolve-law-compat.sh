#!/usr/bin/env bash
set -euo pipefail

# Resolve compatibility material root consumed by tooling.
# Official runtime flow is embedded-first only.

yai_resolve_law_compat_root() {
  local root="${1:-}"

  if [[ -n "${YAI_LAW_COMPAT_ROOT:-}" && -d "${YAI_LAW_COMPAT_ROOT:-}" ]]; then
    echo "$YAI_LAW_COMPAT_ROOT"
    return 0
  fi

  if [[ -n "$root" && -d "$root/embedded/law" ]]; then
    echo "$root/embedded/law"
    return 0
  fi

  return 1
}
