#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs"

REQUIRED_ARTIFACTS = [
    "docs/archive/migration/c17.8-live-set-compression-audit.md",
    "docs/archive/migration/c17.8-canonical-live-set-map.md",
    "docs/archive/migration/c17.8-final-archive-and-eviction-plan.md",
]

REPORT_WHITELIST = {
    "docs/program/reports/README.md",
    "docs/program/reports/audit-convergence-report.md",
    "docs/program/reports/runtime-convergence-report.md",
}

FORBIDDEN_TOKENS = ("closeout", "refoundation", "legacy-notes", "baseline")


def main() -> int:
    errors: list[str] = []

    for rel in REQUIRED_ARTIFACTS:
        if not (ROOT / rel).is_file():
            errors.append(f"missing required C17.8 artifact: {rel}")

    root_readme = ROOT / "docs/README.md"
    if root_readme.is_file():
        txt = root_readme.read_text(encoding="utf-8", errors="ignore")
        if "## Canonical Documentation Surface" not in txt:
            errors.append("missing canonical surface section in docs/README.md")
    else:
        errors.append("missing docs/README.md")

    # milestone-packs must not exist as primary live root in docs/program.
    mp_root = ROOT / "docs/program/milestone-packs"
    if mp_root.exists():
        errors.append("forbidden live program root family still present: docs/program/milestone-packs")

    # reports whitelist.
    rep_root = ROOT / "docs/program/reports"
    if rep_root.exists():
        for p in rep_root.rglob("*.md"):
            rel = p.relative_to(ROOT).as_posix()
            if rel not in REPORT_WHITELIST:
                errors.append(f"non-canonical live report present: {rel}")

    # naming guardrail on live canonical surfaces.
    for section in ("architecture", "guides", "reference", "runbooks", "program"):
        base = DOCS / section
        if not base.exists():
            continue
        for p in base.rglob("*.md"):
            rel = p.relative_to(ROOT).as_posix()
            if rel.startswith("docs/archive/") or rel.startswith("docs/program/archive/"):
                continue
            name = p.name.lower()
            for token in FORBIDDEN_TOKENS:
                if token in name:
                    errors.append(f"forbidden live naming token '{token}': {rel}")
                    break

    if errors:
        print("docs_live_set_compression: FAIL")
        for e in errors:
            print(" -", e)
        return 1

    print("docs_live_set_compression: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
