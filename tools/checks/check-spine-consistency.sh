#!/usr/bin/env sh
set -eu

# YAI - spine consistency guard (CORE.SPINE.C1)
#
# Purpose:
#   Keep the consolidated spine surface honest so docs cannot drift back into
#   ambiguity between the core spine, module/repo spines and temporal SPINE.NN
#   waves, and cannot fabricate enforcement claims.
#
# Scope:
#   Checks the CORE.SPINE.C1 deliverables under work/spines and a few
#   repository invariants. Grep-based and intentionally simple.
#
# Non-goals:
#   Does not validate runtime behavior or carrier execution.

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"

TAXONOMY="work/spines/spine-taxonomy.md"
STATUS="work/spines/core-enforcement-status.md"
PROPERTIES="work/spines/core-properties.md"
HARDENING="work/spines/core-hardening-index.md"

fail() {
  printf 'check-spine-consistency: %s\n' "$1" >&2
  exit 1
}

# Deliverables must exist.
for file in "$TAXONOMY" "$STATUS" "$PROPERTIES" "$HARDENING"; do
  [ -f "$file" ] || fail "missing file: $file"
done

# (1) work/protocols must not exist; protocol material lives under proto/.
[ ! -e work/protocols ] || fail "work/protocols exists; protocol material belongs under proto/"

# (2) model_provider must not be claimed as executing while the carrier is still
#     a skeleton in system/effect/.
if grep -i 'model_provider' "$STATUS" | grep -Eq '(^|[^a-z_])(implemented|implemented_limited|interposed)([^a-z_]|$)'; then
  if grep -rq 'model_provider' system/effect/ 2>/dev/null; then
    fail "model_provider claimed implemented/interposed in $STATUS while system/effect/ still marks it skeleton"
  fi
fi

# (3) work/spines docs must not claim facts/projections are operational truth or
#     authority. These affirmative phrases cannot occur in the negated forms
#     ("facts are not truth", "projections are views, not authority").
for phrase in \
  "facts are truth" \
  "projections are truth" \
  "projection is truth" \
  "facts are authority" \
  "projections are authority" \
  "projection is authority" \
  "facts can authorize" \
  "projections can authorize"; do
  if grep -riF -- "$phrase" work/spines >/dev/null 2>&1; then
    fail "forbidden authority/truth claim in work/spines: \"$phrase\""
  fi
done

# (4) apps/ must not be claimed present when it is absent from this checkout.
if [ ! -d apps ]; then
  for phrase in \
    "apps/ is present" \
    "apps directory exists" \
    "apps/ is a yai root" \
    "apps/ is a canonical"; do
    if grep -riF -- "$phrase" work/spines >/dev/null 2>&1; then
      fail "apps/ claimed present in work/spines but apps/ is absent: \"$phrase\""
    fi
  done
fi

# (5) core-enforcement-status.md must carry the full status vocabulary.
for token in \
  implemented \
  implemented_limited \
  inspect_only \
  skeleton \
  planned \
  external_unknown \
  absent; do
  grep -qw "$token" "$STATUS" || fail "$STATUS missing status vocabulary token: $token"
done

# (6) core-properties.md must define CP1 through CP7.
for cp in CP1 CP2 CP3 CP4 CP5 CP6 CP7; do
  grep -qw "$cp" "$PROPERTIES" || fail "$PROPERTIES missing property: $cp"
done

# (7) spine-taxonomy.md must distinguish core spine, module/repo spines and
#     temporal SPINE.NN waves.
grep -qi 'Core Spine' "$TAXONOMY" || fail "$TAXONOMY missing Core Spine section"
grep -qi 'Module / Repo Spines' "$TAXONOMY" || fail "$TAXONOMY missing Module / Repo Spines section"
grep -qi 'Temporal Wave Markers' "$TAXONOMY" || fail "$TAXONOMY missing Temporal Wave Markers section"
grep -qF 'SPINE.NN' "$TAXONOMY" || fail "$TAXONOMY missing SPINE.NN temporal-wave distinction"

printf 'check-spine-consistency: ok\n'
