#!/usr/bin/env sh
set -eu

# YAI - canonical documentation authority guard
#
# Purpose: keep constitution, current architecture, reference, roadmap,
# operations, evidence, research, and development instructions distinct.

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"

fail() {
  printf 'check-doc-root-canon: %s\n' "$1" >&2
  exit 1
}

for file in \
  README.md \
  ROADMAP.md \
  docs/index.md \
  docs/constitution.md \
  docs/architecture.md \
  docs/reference/semantics.md \
  docs/reference/state-transitions.md \
  docs/reference/context.md \
  docs/reference/boundaries.md \
  docs/quickstart.md \
  docs/test-cases.md \
  docs/legal.md
do
  [ -f "$file" ] || fail "missing canonical file: $file"
done

for forbidden in \
  docs/architecture \
  docs/internal \
  docs/engineering \
  docs/spines \
  docs/product \
  docs/status \
  docs/protocols \
  docs/adr \
  docs/labs \
  docs/lab-standards
do
  [ ! -e "$forbidden" ] || fail "obsolete authority path exists: $forbidden"
done

for removed in docs/glossary.md docs/providers.md docs/technical-brief.md; do
  [ ! -e "$removed" ] || fail "absorbed competing document exists: $removed"
done

grep -Fq 'Authority: constitutional target.' docs/constitution.md ||
  fail 'constitution authority declaration missing'
grep -Fq 'Authority: implementation truth.' docs/architecture.md ||
  fail 'architecture authority declaration missing'
grep -Fq 'Authority: implementation delta' ROADMAP.md ||
  fail 'roadmap authority declaration missing'
grep -Fq 'YAI governs the admitted transformation of canonical operational state.' docs/constitution.md ||
  fail 'constitutional primitive missing'
grep -Fq '`Space` is rejected as a canonical owner.' docs/constitution.md ||
  fail 'Space rejection missing'
grep -Fq '`Agent` is rejected as a canonical runtime owner.' docs/constitution.md ||
  fail 'Agent rejection missing'
grep -Fq 'committed Transition Ledger' docs/constitution.md ||
  fail 'single historical authority missing'
grep -Fq 'INDETERMINATE' docs/reference/state-transitions.md ||
  fail 'external-effect uncertainty missing'
grep -Fq 'Evidence is not a universal stored object.' docs/reference/state-transitions.md ||
  fail 'Evidence role distinction missing'
grep -Fq 'ContextDelta — DEFER' docs/reference/context.md ||
  fail 'ContextDelta decision missing'
grep -Fq '!= provider continuation / KV identity' docs/reference/context.md ||
  fail 'semantic/token/KV identity distinction missing'
grep -Fq 'YAI semantic continuity' docs/reference/boundaries.md ||
  fail 'semantic/computational continuity boundary missing'
grep -Fq 'commits ledger and materialization together.' docs/architecture.md ||
  fail 'current canonical state-authority implementation missing'
grep -Fq 'Old `journal replay`' docs/architecture.md ||
  fail 'legacy journal compatibility boundary missing'
grep -Fq 'The direct `carrier fs-write` command and Rust primitive were removed' docs/architecture.md ||
  fail 'controlled filesystem boundary implementation missing'
grep -Fq 'only `filesystem.write`' docs/architecture.md ||
  fail 'first carrier scope limitation missing'
grep -Fq 'Authority: historical evidence and work records only.' work/README.md ||
  fail 'work/ is not explicitly de-authorized'
grep -Fq 'Authority: lab-local procedures, inputs, and captured results only.' labs/README.md ||
  fail 'labs/ is not scoped to evidence'

if find include/yai system -mindepth 2 -name README.md -print -quit | grep -q .; then
  find include/yai system -mindepth 2 -name README.md -print >&2
  fail 'module-per-noun README authority remains'
fi

printf 'doc_root_canon: ok\n'
