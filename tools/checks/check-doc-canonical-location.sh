#!/bin/sh
# YAI - canonical docs location guard
#
# Purpose:
#   Prevent active documentation from drifting back into legacy roots.
#
# Scope:
#   Checks canonical docs directory presence and known obsolete docs copies.
#
# Non-goals:
#   Does not classify archive contents.
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)

if [ ! -d "$ROOT/docs" ]; then
  printf 'canonical docs directory missing: %s/docs\n' "$ROOT" >&2
  exit 1
fi

if [ -e "$ROOT/Documentation" ]; then
  printf 'legacy Documentation tree found inside yai: %s/Documentation\n' "$ROOT" >&2
  exit 1
fi

if [ -e "$ROOT/docs/internal/architecture/core-spine/new-core/docs" ]; then
  printf 'old active docs copy still exists inside yai docs/internal\n' >&2
  exit 1
fi

if [ -d "$ROOT/docs/architecture" ]; then
  printf 'obsolete multi-owner docs/architecture/ tree still exists\n' >&2
  exit 1
fi

if [ ! -d "$ROOT/docs/reference" ]; then
  printf 'canonical reference directory missing: %s/docs/reference\n' "$ROOT" >&2
  exit 1
fi

printf 'check-doc-canonical-location: ok\n'
