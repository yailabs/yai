#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs"

PROGRAM_REQUIRED_DIRS = [
    "docs/program/adr",
    "docs/program/rfc",
    "docs/program/policies",
    "docs/program/templates",
    "docs/program/reports",
    "docs/program/archive",
]

PROGRAM_REQUIRED_FILES = [
    "docs/program/README.md",
    "docs/program/archive/legacy/decision-ledger.md",
    "docs/program/archive/README.md",
]

REQUIRED_FREEZE_FILES = [
    "docs/archive/migration/d18.6-docs-freeze-readiness-audit.md",
    "docs/policies/live-docs-admission-policy.md",
    "docs/policies/live-docs-naming-policy.md",
    "docs/policies/docs-structure-policy.md",
    "docs/policies/archive-transfer-policy.md",
]

ENTRYPOINTS = [
    "docs/README.md",
    "docs/architecture/README.md",
    "docs/architecture/overview/system-overview.md",
    "docs/architecture/overview/subsystem-map.md",
    "docs/architecture/overview/repository-map.md",
]

FORBIDDEN_TOKENS = ("closeout", "refoundation", "legacy-notes", "baseline", "wave", "tranche")


def parse_front_matter(path: Path) -> dict[str, str]:
    text = path.read_text(encoding="utf-8", errors="ignore")
    if not text.startswith("---\n"):
        return {}
    end = text.find("\n---\n", 4)
    if end < 0:
        return {}
    block = text[4:end]
    out: dict[str, str] = {}
    for line in block.splitlines():
        if ":" not in line:
            continue
        key, value = line.split(":", 1)
        out[key.strip()] = value.strip()
    return out


def parse_list(value: str) -> list[str]:
    value = value.strip()
    if not value:
        return []
    if value.startswith("[") and value.endswith("]"):
        inner = value[1:-1].strip()
        if not inner:
            return []
        return [x.strip().strip("'\"") for x in inner.split(",") if x.strip()]
    return [value.strip().strip("'\"")]


def main() -> int:
    errors: list[str] = []

    for rel in REQUIRED_FREEZE_FILES:
        if not (ROOT / rel).is_file():
            errors.append(f"missing freeze artifact: {rel}")

    for rel in PROGRAM_REQUIRED_DIRS:
        if not (ROOT / rel).is_dir():
            errors.append(f"missing required program dir: {rel}")

    for rel in PROGRAM_REQUIRED_FILES:
        if not (ROOT / rel).is_file():
            errors.append(f"missing required program file: {rel}")

    if (ROOT / "docs/program/milestone-packs").exists():
        errors.append("forbidden live program root still present: docs/program/milestone-packs")

    # Report live whitelist contract: small authoritative set only.
    report_root = ROOT / "docs/program/reports"
    live_reports = sorted(
        p.relative_to(ROOT).as_posix() for p in report_root.glob("*.md") if p.name != "README.md"
    )
    if not (1 <= len(live_reports) <= 7):
        errors.append(
            f"live report count outside freeze bounds (1..7): {len(live_reports)}"
        )

    # Naming freeze: block historical tokenized filenames in live surfaces.
    for section in ("architecture", "guides", "reference", "runbooks", "program", "generated"):
        base = DOCS / section
        if not base.is_dir():
            continue
        for p in base.rglob("*.md"):
            rel = p.relative_to(ROOT).as_posix()
            if rel.startswith("docs/archive/") or rel.startswith("docs/program/archive/"):
                continue
            lower = p.name.lower()
            for token in FORBIDDEN_TOKENS:
                if token in lower:
                    errors.append(f"forbidden live naming token '{token}': {rel}")
                    break

    # Ensure entry surface remains canonical and present.
    for rel in ENTRYPOINTS:
        if not (ROOT / rel).is_file():
            errors.append(f"missing canonical entrypoint: {rel}")

    root_readme = ROOT / "docs/README.md"
    if root_readme.is_file():
        txt = root_readme.read_text(encoding="utf-8", errors="ignore")
        if "## Canonical Documentation Surface" not in txt:
            errors.append("missing canonical surface section in docs/README.md")
    else:
        errors.append("missing docs/README.md")

    # No authoritative dependencies from live docs to archive lanes.
    for p in DOCS.rglob("*.md"):
        rel = p.relative_to(ROOT).as_posix()
        if rel.startswith("docs/archive/") or rel.startswith("docs/program/archive/"):
            continue
        fm = parse_front_matter(p)
        if not fm:
            continue
        role = fm.get("role", "").strip()
        depends = parse_list(fm.get("depends_on", ""))
        if role in {"canonical", "support", "reference", "procedural"}:
            for dep in depends:
                if dep.startswith("docs/archive/") or dep.startswith("docs/program/archive/"):
                    errors.append(f"authoritative dependency points to archive: {rel} -> {dep}")

    if errors:
        print("docs_freeze_contract: FAIL")
        for e in errors:
            print(" -", e)
        return 1

    print("docs_freeze_contract: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
