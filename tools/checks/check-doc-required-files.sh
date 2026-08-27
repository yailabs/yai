#!/bin/sh
set -eu

# YAI - required canonical documentation guard

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)

for file in \
  README.md \
  ROADMAP.md \
  docs/README.md \
  docs/index.md \
  docs/constitution.md \
  docs/architecture.md \
  docs/reference/semantics.md \
  docs/reference/state-transitions.md \
  docs/reference/context.md \
  docs/reference/boundaries.md \
  docs/quickstart.md \
  docs/test-cases.md \
  docs/legal.md \
  work/README.md \
  labs/README.md
do
  if [ ! -f "$ROOT/$file" ]; then
    printf 'required canonical documentation file missing: %s\n' "$file" >&2
    exit 1
  fi
done

printf 'check-doc-required-files: ok\n'
