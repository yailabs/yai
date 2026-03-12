#!/usr/bin/env python3
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs"

ALLOWED_L1 = {
    "README.md",
    "architecture",
    "guides",
    "reference",
    "runbooks",
    "program",
    "generated",
    "archive",
    "policies",
}

REQUIRED_POLICY_FILES = [
    "docs/policies/live-docs-admission-policy.md",
    "docs/policies/live-docs-naming-policy.md",
    "docs/policies/docs-structure-policy.md",
    "docs/policies/archive-transfer-policy.md",
]

FORBIDDEN_TOKENS = ("closeout", "refoundation", "legacy-notes", "baseline", "wave", "tranche")
ENTRYPOINT_REQUIRED = [
    "docs/README.md",
    "docs/architecture/README.md",
    "docs/guides/README.md",
    "docs/reference/README.md",
    "docs/runbooks/README.md",
    "docs/program/README.md",
]


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

    for rel in REQUIRED_POLICY_FILES:
        if not (ROOT / rel).is_file():
            errors.append(f"missing required surface policy file: {rel}")

    for rel in ENTRYPOINT_REQUIRED:
        if not (ROOT / rel).is_file():
            errors.append(f"missing required entrypoint README: {rel}")

    root_readme = ROOT / "docs/README.md"
    if root_readme.is_file():
        txt = root_readme.read_text(encoding="utf-8", errors="ignore")
        if "## Canonical Documentation Surface" not in txt:
            errors.append("missing canonical surface section in docs/README.md")
    else:
        errors.append("missing docs/README.md")

    # L1 surface lock.
    if DOCS.is_dir():
        for p in DOCS.iterdir():
            name = p.name
            if name.startswith("."):
                continue
            if name not in ALLOWED_L1:
                errors.append(f"non-canonical docs L1 entry present: docs/{name}")

    # Program root lock.
    if (ROOT / "docs/program/milestone-packs").exists():
        errors.append("forbidden docs/program/milestone-packs present in live root")

    # Live naming guardrail.
    for p in DOCS.rglob("*.md"):
        rel = p.relative_to(ROOT).as_posix()
        if rel.startswith("docs/archive/") or rel.startswith("docs/program/archive/"):
            continue
        lower = p.name.lower()
        for token in FORBIDDEN_TOKENS:
            if token in lower:
                errors.append(f"forbidden live naming token '{token}': {rel}")
                break

    # Admission metadata guard on high-visibility docs.
    for rel in ENTRYPOINT_REQUIRED:
        fm = parse_front_matter(ROOT / rel)
        if not fm:
            errors.append(f"missing front matter in high-visibility doc: {rel}")
            continue
        for key in ("role", "status", "owner_domain"):
            if not fm.get(key, "").strip():
                errors.append(f"missing '{key}' in high-visibility doc: {rel}")

    # Archive boundary: non-canonical docs must not depend on archive for authority.
    for p in DOCS.rglob("*.md"):
        rel = p.relative_to(ROOT).as_posix()
        if rel.startswith("docs/archive/") or rel.startswith("docs/program/archive/"):
            continue
        fm = parse_front_matter(p)
        if not fm:
            continue
        depends = parse_list(fm.get("depends_on", ""))
        for dep in depends:
            if dep.startswith("docs/archive/") or dep.startswith("docs/program/archive/"):
                errors.append(f"live docs dependency points to archive: {rel} -> {dep}")

    if errors:
        print("docs_surface_policy: FAIL")
        for e in errors:
            print(" -", e)
        return 1

    print("docs_surface_policy: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
