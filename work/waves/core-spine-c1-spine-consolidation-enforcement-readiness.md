# CORE.SPINE.C1 - Spine Consolidation / Enforcement Readiness

Status: implemented

Macro:

```text
CONTROL
OBSERVABILITY
```

Type: consolidation/readiness wave. Not a runtime implementation wave.

## Purpose

Convert the engineering audit into concrete repository structure without
rewriting history or turning the repo into a whitepaper. Make the spine system
unambiguous: which spine is the canonical core spine, what NET/CLORI/interfaces/
console/studio are relative to it, whether `SPINE.NN` markers are spines or
temporal waves, what is enforced today versus skeleton/planned, which gaps block
stronger consequence enforcement, and which claims are forbidden until code
proves them. Add a lightweight guard so the docs cannot drift back into
ambiguity.

## Scope

- Add a canonical spine taxonomy document.
- Add an honest core enforcement status document.
- Add a core property registry (CP1-CP7).
- Add a next-wave hardening index.
- Add a grep-based consistency guard and wire it into the Makefile.
- Update spine README pointers.

## Command Surface

No command surface changes.

## Files Added / Changed

Added:

```text
work/spines/spine-taxonomy.md
work/spines/core-enforcement-status.md
work/spines/core-properties.md
work/spines/core-hardening-index.md
tools/checks/check-spine-consistency.sh
work/waves/core-spine-c1-spine-consolidation-enforcement-readiness.md
```

Changed:

```text
Makefile                    .PHONY, check-docs aggregation, standalone target
work/spines/README.md       pointers to the new consolidation docs
```

## What Was Consolidated

- The word "spine" now has one taxonomy: `yai-spine.md` is the canonical core
  spine; NET/CLORI/interfaces/console/studio are module/repo spines; `SPINE.NN`
  are temporal implementation waves, not separate product spines.
- The current enforcement state is recorded honestly with a restricted status
  vocabulary (implemented / implemented_limited / inspect_only / skeleton /
  planned / external_unknown / absent) and a restricted enforcement-level
  vocabulary. Evidence paths point at real code (system/control, system/effect,
  engine/yai-engine, system/projection, system/graph, etc.).
- The carrier reality is stated plainly: only filesystem (interposed) and
  process (implemented_limited) carriers execute; network_http, database, git,
  service/endpoint/socket/listener and model_provider are skeleton surfaces in
  system/effect/carrier_skeleton.c.
- Core properties CP1-CP7 are registered with falsifier test ideas and the next
  hardening wave that strengthens each.
- The audit's gaps are mapped to executable waves (CORE.ENFORCE.1,
  CORE.CARRIER.1/2, CORE.POLICY.1, CORE.NORMALIZE.1, CORE.JOURNAL.1,
  CORE.DATA.1, CORE.LAB.1, CORE.MODEL.1).

## What Was Not Changed

- No new carriers, no model_provider, no policy engine implemented.
- No historical `SPINE.NN` wave renamed.
- No protocol material moved back under `work/protocols` (it does not exist).
- No public README positioning rewritten beyond the spine README pointers.
- No private/unknown repos (yai-dev, studio, apps) treated as verified.
- No `apps/` claimed present (it is absent from this checkout).

## Forbidden Claims (enforced by the guard)

```text
work/protocols must not exist
model_provider must not be claimed implemented/interposed while skeleton
facts/projections must not be described as operational truth or authority
apps/ must not be claimed present while absent
status vocabulary, CP1-CP7 and the spine taxonomy distinctions must be present
```

## Validation

Run:

```text
git diff --check
make info
make check-layout
make check-docs
make check-spine-consistency
```

Targeted checks:

```text
test ! -e work/protocols
grep -nE 'CP[1-7]' work/spines/core-properties.md
model_provider stays skeleton in work/spines/core-enforcement-status.md
no affirmative "facts are truth" / "projections are authority" in work/spines
apps/ absent and not claimed present
```

Note: there is no `make yai` target in this checkout; the original prompt's
`make yai` validation line does not apply. `make check` runs check-layout,
check-docs (which now includes check-spine-consistency), build and smoke.

## yai-dev Residue

CORE.SPINE.C1 is internal `yai` documentation and guard hygiene. No `yai-dev`
source file was inspected or modified; this wave introduces no concept mined
from `yai-dev`.

## Next Recommended Wave

CORE.ENFORCE.1 - Control Lease Dispatch Verification. It closes the highest-
leverage gap (whether CapabilityLease is consumed before carrier dispatch or
only inspected) and unblocks CORE.CARRIER.1/2.
