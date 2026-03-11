# Law Runtime Convergence Audit

> Historical report (pre-cutover snapshot).  
> Current state: `deps/law` bridge retired; active runtime path is `embedded/law` only.

Date: 2026-03-08  
Scope: `law` + `yai` (cross-repo convergence, contract integrity, runtime path, tests, docs/tooling)

## Executive Verdict

- Structure: **Convergent with controlled bridge**
- Publish/export contract: **Coherent (validated)**
- Runtime canonical path: **Embedded-first and now stricter**
- Bridge contamination: **Reduced; still present as explicit transitional fallback**
- Test coverage: **Improved on contract risks; semantic genericity still limited**

Overall: convergence is real and operational, with remaining debt mostly in runtime semantic hardcoding and transitional payload presence.

## Active vs Bridge vs Residual Matrix

| Surface | Classification | Notes |
|---|---|---|
| `law/classification` | active canonical | Primary layer-1 taxonomy payload |
| `law/control-families` | active canonical | Primary family routing corpus |
| `law/specializations` | active canonical | Primary specialization corpus |
| `law/overlays/regulatory` | active canonical | Primary regulatory overlay corpus |
| `law/overlays/sector` | active canonical | Primary sector overlay corpus |
| `law/overlays/contextual` | active canonical | Primary contextual overlay corpus |
| `law/manifests/*` | active canonical | Publish/export/runtime contract source |
| `law/transitional/domain-family-seed` | bridge | Fallback payload only |
| `law/domains` | transitional tolerated | Legacy compatibility surface |ªª
| `law/compliance` | transitional tolerated | Legacy compatibility surface |
| `law/packs` | dead/reference-only | Legacy, non-primary |
| `law/runtime` | dead/reference-only | Legacy conceptual residue |
| `yai/embedded/law` | active runtime-facing | Primary runtime contract payload |
| `yai/lib/law` | active runtime-facing | Runtime loader/discovery/resolution path |
| `yai/include/yai/law` | active runtime-facing | Runtime API surface |
| `yai/tools/bin/yai-law-compat-check` | active runtime-facing | Contract verification gate |
| `yai/tools/bin/yai-law-embed-sync` | active runtime-facing | Embedded sync path |
| `yai/tools/dev/resolve-law-embed.sh` | active runtime-facing | Embedded root resolver |
| `yai/deps/law` | historical/removed | retired from active runtime and tooling paths |
| `yai/docs/architecture/runtime-law-integration-debug-report.md` | dead/reference-only | Historical report; intentionally legacy narrative |

## Canonical Runtime Path Verdict

Canonical path is:

1. `law` canonical manifests + six-layer corpus  
2. `law` publish/export tooling produces runtime contract  
3. `yai/embedded/law` consumed as primary runtime payload  
4. `yai` resolver executes classification -> family -> specialization -> overlays -> authority/evidence -> effect

Status: **clear and testable**, with one key caveat: discovery/resolution logic still uses internal IDs (`D1/D5/D8`) for decision branches, even if family/specialization are also present.

## Critical Findings

### High

1. Legacy bridge fallback was still implicitly reachable in loader/tooling in some paths.
2. Domain loader compatibility path still favored old layout semantics over six-layer-first lookup.

### Medium

1. Runtime semantic branching remains partially hardcoded around `D1/D5/D8`.
2. Transitional manifests are still exported as bridge payload in embedded contract.
3. Several historical docs still mention `deps/law` and legacy IDs (explicitly historical, but noisy).

### Low

1. Some top-level wording still references transitional language.
2. Historical report docs retain old risk statements by design.

## Test Coverage Gaps (Current)

- Strong coverage exists for:
  - manifest/contract loading
  - embedded compatibility
  - D1/D8/economic mixed integration paths
  - overlay-driven mixed resolution
- Remaining gaps:
  - broader specialization-generic discovery (beyond pilot trio)
  - deeper precedence/conflict matrix fuzzing
  - explicit bridge-enabled vs bridge-disabled behavioral matrix

## Remediation Applied In This Pass

### 1) Primary path hardening (done)

- `deps/law` fallback in runtime loader now requires explicit opt-in:
  - `YAI_LAW_ENABLE_LEGACY_BRIDGE=1`
- File: `lib/law/loader/law_loader.c`

### 2) Tooling bridge hardening (done)

- Compatibility resolver no longer auto-selects legacy path.
- Generated-check script now uses legacy path only with explicit bridge flag.
- Files:
  - `tools/dev/resolve-law-compat.sh`
  - `tools/dev/check-generated.sh`

### 3) Domain loader six-layer lookup improvement (done)

- Domain manifest loader now searches in this order:
  - `control-families/`
  - `specializations/`
  - `domains/` (legacy)
  - `transitional/domain-family-seed/` (bridge)
- File: `lib/law/loader/domain_loader.c`

### 4) Contract and anti-legacy tests added (done)

- Added `test_no_legacy_primary_path.c`
- Added `test_embedded_surface_matches_publish_index.py`
- Strengthened `test_contract_surface.c`
- Extended `test_domain_loader.c` for family/specialization + transitional compatibility
- Updated runner:
  - `tests/unit/law/run_law_unit_tests.sh`

## Verification Executed

- `tests/unit/law/run_law_unit_tests.sh` -> PASS
- `tests/integration/law_resolution/run_law_resolution_smoke.sh` -> PASS
- `./tools/bin/yai-law-compat-check` -> PASS
- `./tools/dev/check-generated.sh` -> PASS
- `(cd ../law && tools/bin/law-check-compat)` -> PASS
- `(cd ../law && python3 tools/validate/validate_publish_surface.py)` -> PASS

## Residual Debt After This Pass

1. Runtime discovery/resolution still branch-hardcoded around `D1/D5/D8` for pilot behavior.
2. Embedded export still includes small transitional seed bridge payload by design.
3. Historical docs contain intentional legacy references that should stay quarantined as historical.
4. Full bridge retirement (`deps/law` removal) still requires final cutover tranche.

## Conclusion

Convergence is now materially stronger: canonical/export/embedded/runtime boundaries are operationally consistent, legacy bridge usage is explicit and opt-in, and contract-level tests now cover key structural risks.
