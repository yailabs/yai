#!/bin/sh
set -eu

# YAI - file header standard guard
#
# Purpose:
#   Ensure the repository has an active ownership/header standard and that the
#   principal ownership surfaces carry short agent-readable notes.
#
# Scope:
#   Checks doctrine docs, wave/extraction references, top-level build surface,
#   guard headers and principal directory READMEs.
#
# Non-goals:
#   This guard does not yet enforce headers across every .c, .h or .rs file.

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"

require_file() {
  if [ ! -f "$ROOT/$1" ]; then
    printf 'missing file header standard file: %s\n' "$1" >&2
    exit 1
  fi
}

require_phrase() {
  file="$1"
  phrase="$2"
  if ! grep -Fq "$phrase" "$ROOT/$file"; then
    printf 'missing file header standard phrase in %s: %s\n' "$file" "$phrase" >&2
    exit 1
  fi
}

require_file "work/archive/engineering-snapshots/file-header-standard.md"
require_file "work/agents/agent-operating-appendix.md"
require_file "work/waves/wave-template.md"
require_file "work/waves/operational-extraction-contract.md"
require_file "Makefile"
require_file "engine/README.md"
require_file "system/README.md"
require_file "tools/checks/README.md"
require_file "tests/smoke/README.md"

require_phrase "work/archive/engineering-snapshots/file-header-standard.md" "Purpose"
require_phrase "work/archive/engineering-snapshots/file-header-standard.md" "Ownership"
require_phrase "work/archive/engineering-snapshots/file-header-standard.md" "Boundary"
require_phrase "work/archive/engineering-snapshots/file-header-standard.md" "Transitional C shim"
require_phrase "work/agents/agent-operating-appendix.md" "Before Editing"
require_phrase "work/agents/agent-operating-appendix.md" "A file is not ready for agent modification"
require_phrase "work/waves/wave-template.md" "File/header impact"
require_phrase "work/waves/operational-extraction-contract.md" "agent-operating-appendix.md"
require_phrase "Makefile" "YAI - local build and validation surface"
require_phrase "engine/README.md" "Authority: local navigation"
require_phrase "system/README.md" "Authority: local navigation"
require_phrase "tools/checks/README.md" "Guard scripts should carry a short"
require_phrase "tests/smoke/README.md" "Smoke tests prove current behavior"

for script in "$ROOT"/tools/checks/*.sh; do
  if ! grep -Fq "YAI -" "$script"; then
    printf 'missing YAI purpose header in guard: %s\n' "${script#$ROOT/}" >&2
    exit 1
  fi
done

printf 'file-header-standard: ok\n'
