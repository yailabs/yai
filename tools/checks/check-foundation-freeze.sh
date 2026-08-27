#!/bin/sh
# YAI - foundation and documentation authority guard
#
# Purpose: keep required source roots stable while ensuring historical project
# control material cannot regain current documentation authority.
#
# Scope: repository roots and authority entrypoints only.
# Non-goals: runtime behavior, target source ownership, or historical phrases.
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)

require_dir() {
  if [ ! -d "$ROOT/$1" ]; then
    printf 'foundation required directory missing: %s\n' "$1" >&2
    exit 1
  fi
}

require_file() {
  if [ ! -f "$ROOT/$1" ]; then
    printf 'foundation required file missing: %s\n' "$1" >&2
    exit 1
  fi
}

for dir in include system engine cmd/yai cmd/yaid proto tests docs docs/reference tools vendor work labs; do
  require_dir "$dir"
done

for file in README.md ROADMAP.md docs/index.md docs/constitution.md docs/architecture.md work/README.md labs/README.md; do
  require_file "$file"
done

for forbidden in src lib crates ctl daemon agents runtime substrate orchestrator models capabilities lineage analytics governance knowledge state workflow; do
  if [ -e "$ROOT/$forbidden" ]; then
    printf 'foundation forbidden root found: %s\n' "$forbidden" >&2
    exit 1
  fi
done

if ! grep -Fq 'historical evidence and work records only' "$ROOT/work/README.md"; then
  printf 'work project-control history is not de-authorized\n' >&2
  exit 1
fi

if ! grep -Fq 'lab-local procedures, inputs, and captured results only' "$ROOT/labs/README.md"; then
  printf 'labs are not scoped to experimental evidence\n' >&2
  exit 1
fi

printf 'check-foundation-freeze: ok\n'
