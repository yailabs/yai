#!/bin/sh
set -eu

# YAI - repository identity guard
#
# Purpose: prevent active source and canonical documentation from drifting back
# to retired repository identities. Historical evidence is excluded.

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
HITS=/tmp/yai-repository-identity-hit.$$
trap 'rm -f "$HITS"' EXIT

OLD_CORE='yai''-core'
OLD_ENV='ai''-environment'
OLD_YAI='old''-yai'
OLD_YAI_TEXT='old'' yai'
OLD_TITLE='YAI ''Core'
OLD_TITLE_HYPHEN='YAI''-core'
OLD_TMP='/tmp/yai''-core'
PATTERN="$OLD_CORE|$OLD_ENV|$OLD_YAI|$OLD_YAI_TEXT|$OLD_TITLE|$OLD_TITLE_HYPHEN|$OLD_TMP"

for path in README.md ROADMAP.md Makefile cmd tools docs include system proto engine net tests; do
  if [ -e "$ROOT/$path" ]; then
    grep -R -n -E --exclude-dir=.git --exclude-dir=build --exclude-dir=target \
      "$PATTERN" "$ROOT/$path" >>"$HITS" 2>/dev/null || true
  fi
done

if [ -s "$HITS" ]; then
  cat "$HITS" >&2
  printf 'active repository identity still references retired names\n' >&2
  exit 1
fi

grep -Fq 'yai-dev' "$ROOT/docs/index.md" || {
  printf 'historical yai-dev role missing from documentation authority map\n' >&2
  exit 1
}

printf 'check-repository-identity: ok\n'
