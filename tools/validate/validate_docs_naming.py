#!/usr/bin/env python3
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs"

DISALLOWED_SUFFIXES = (
    "-model.md",
    "-baseline.md",
    "-closeout.md",
    "-refoundation.md",
    "-legacy-notes.md",
    "-residue.md",
)

# Live canonical architecture/guides/runbooks/reference docs must not drift
# toward report/migration artifacts. Program reports are the canonical home.
DISALLOWED_LIVE_CLASSES = (
    "-report.md",
    "-migration.md",
)

FORBIDDEN_ARCH_DIRS = [
    DOCS / "architecture" / "edge-mesh",
    DOCS / "architecture" / "data-graph",
    DOCS / "architecture" / "providers-agents",
    DOCS / "architecture" / "formal-foundation",
]

ADR_RE = re.compile(r"adr-\d{3}-[a-z0-9-]+\.md$")
RFC_RE = re.compile(r"rfc-\d{3}-[a-z0-9-]+\.md$")
MP_RE = re.compile(r"mp-[a-z0-9]+-\d{3}-[a-z0-9-]+(?:-v\d+-\d+-\d+)?\.md$")


def is_live_doc(path: Path) -> bool:
    rel = path.relative_to(DOCS)
    if rel.parts[0] == "archive":
        return False
    if rel.parts[0] == "program" and len(rel.parts) > 1 and rel.parts[1] == "archive":
        return False
    return True


def main() -> int:
    errors: list[str] = []

    for p in FORBIDDEN_ARCH_DIRS:
        if p.exists():
            errors.append(f"forbidden old architecture folder exists: {p.relative_to(ROOT)}")

    for md in DOCS.rglob("*.md"):
        if not is_live_doc(md):
            continue

        rel = str(md.relative_to(ROOT))
        name = md.name

        if name in {"README.md"}:
            continue

        for suffix in DISALLOWED_SUFFIXES:
            if name.endswith(suffix):
                errors.append(f"disallowed live-doc suffix '{suffix}': {rel}")
                break

        rel_parts = md.relative_to(DOCS).parts
        if rel_parts and rel_parts[0] in {"architecture", "guides", "runbooks", "reference"}:
            for suffix in DISALLOWED_LIVE_CLASSES:
                if name.endswith(suffix):
                    errors.append(
                        f"disallowed live-doc class '{suffix}' under {rel_parts[0]}: {rel}"
                    )
                    break

    adr_dir = DOCS / "program" / "adr"
    if adr_dir.exists():
        for p in adr_dir.glob("*.md"):
            if p.name == "README.md":
                continue
            if not ADR_RE.match(p.name):
                errors.append(f"ADR naming mismatch: {p.relative_to(ROOT)}")

    rfc_dir = DOCS / "program" / "rfc"
    if rfc_dir.exists():
        for p in rfc_dir.glob("*.md"):
            if p.name == "README.md":
                continue
            if not RFC_RE.match(p.name):
                errors.append(f"RFC naming mismatch: {p.relative_to(ROOT)}")

    mp_dir = DOCS / "program" / "milestone-packs"
    if mp_dir.exists():
        for p in mp_dir.rglob("*.md"):
            if p.name == "README.md":
                continue
            if not MP_RE.match(p.name):
                errors.append(f"MP naming mismatch: {p.relative_to(ROOT)}")

    if errors:
        print("docs_naming: FAIL")
        for e in errors:
            print(f" - {e}")
        return 1

    print("docs_naming: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
