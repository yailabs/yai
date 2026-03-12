#!/usr/bin/env python3
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

MP_FAMILIES = [
    "contract-baseline-lock",
    "command-coverage",
    "engine-attach",
    "root-hardening",
    "specs-refactor-foundation",
    "workspaces-lifecycle",
    "data-plane",
]

FORBIDDEN_REPORT_SHARDS = [
    "docs/program/reports/audit-convergence/AUDIT-CONVERGENCE-MATRIX-v0.1.0.md",
    "docs/program/reports/audit-convergence/EXECUTION-PLAN-v0.1.0.md",
    "docs/program/reports/audit-convergence/SC102-GATEA-WORKPLAN-v0.1.0.md",
]

REQUIRED_REPORT = "docs/program/reports/audit-convergence-report.md"

FORBIDDEN_MIGRATION = [
    "docs/archive/migration/c17-docs-audit.md",
    "docs/archive/migration/c17-docs-duplication-map.md",
    "docs/archive/migration/c17-docs-relocation-plan.md",
    "docs/archive/migration/c17.1-docs-topology-audit.md",
    "docs/archive/migration/c17.2-docs-naming-audit.md",
    "docs/archive/migration/c17.3-docs-overlap-audit.md",
    "docs/archive/migration/c17.4-docs-section-verticalization-audit.md",
]

REQUIRED_C175 = [
    "docs/archive/migration/c17.5-program-archive-weight-audit.md",
    "docs/archive/migration/c17.5-milestone-pack-retention-plan.md",
    "docs/archive/migration/c17.5-report-retention-and-eviction-plan.md",
]

VERSION_RE = re.compile(r"^(.*)-v(\d+)-(\d+)-(\d+)\.md$")


def main() -> int:
    errors: list[str] = []

    for fam in MP_FAMILIES:
        d = ROOT / "docs/program/archive/milestone-packs" / fam
        if not d.is_dir():
            errors.append(f"missing milestone-pack family dir: {d.relative_to(ROOT)}")
            continue

        groups: dict[str, int] = {}
        for p in d.glob("*.md"):
            if p.name == "README.md":
                continue
            m = VERSION_RE.match(p.name)
            if not m:
                continue
            base = m.group(1)
            groups[base] = groups.get(base, 0) + 1

        multi = {k: v for k, v in groups.items() if v > 1}
        if multi:
            details = ", ".join([f"{k}={v}" for k, v in sorted(multi.items())])
            errors.append(f"milestone latest-only violated in {fam}: {details}")

    if not (ROOT / REQUIRED_REPORT).is_file():
        errors.append(f"missing canonical consolidated report: {REQUIRED_REPORT}")

    for rel in FORBIDDEN_REPORT_SHARDS:
        if (ROOT / rel).exists():
            errors.append(f"forbidden report shard still present: {rel}")

    for rel in FORBIDDEN_MIGRATION:
        if (ROOT / rel).exists():
            errors.append(f"forbidden granular migration file still present: {rel}")

    for rel in REQUIRED_C175:
        if not (ROOT / rel).is_file():
            errors.append(f"missing required C17.5 artifact: {rel}")

    if errors:
        print("docs_archive_reduction: FAIL")
        for e in errors:
            print(" -", e)
        return 1

    print("docs_archive_reduction: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
