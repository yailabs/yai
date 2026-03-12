#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

REQUIRED_DIRS = [
    "docs/architecture",
    "docs/guides",
    "docs/runbooks",
    "docs/reference",
    "docs/program",
    "docs/generated",
    "docs/archive",
    "docs/archive/migration",
    "docs/architecture/foundation",
    "docs/architecture/formal",
    "docs/architecture/runtime",
    "docs/architecture/orchestration",
    "docs/architecture/edge",
    "docs/architecture/protocol",
    "docs/architecture/data",
    "docs/architecture/graph",
    "docs/architecture/knowledge",
    "docs/architecture/mesh",
    "docs/architecture/workspace",
    "docs/guides/user/guide",
    "docs/program/policies/style",
]

REQUIRED_READMES = [
    "docs/README.md",
    "docs/architecture/README.md",
    "docs/guides/README.md",
    "docs/runbooks/README.md",
    "docs/reference/README.md",
    "docs/program/README.md",
    "docs/generated/README.md",
    "docs/runbooks/operations/README.md",
    "docs/guides/developer/README.md",
    "docs/architecture/foundation/README.md",
    "docs/architecture/formal/README.md",
    "docs/architecture/runtime/README.md",
    "docs/architecture/orchestration/README.md",
    "docs/architecture/edge/README.md",
    "docs/architecture/protocol/README.md",
    "docs/architecture/data/README.md",
    "docs/architecture/graph/README.md",
    "docs/architecture/knowledge/README.md",
    "docs/architecture/mesh/README.md",
    "docs/architecture/workspace/README.md",
]

FORBIDDEN_DOC_DIRS = [
    "docs/developer",
    "docs/developers",
    "docs/interfaces",
    "docs/platform",
    "docs/pointers",
    "docs/operators",
    "docs/program/21-rfc",
    "docs/program/22-adr",
    "docs/program/23-runbooks",
    "docs/program/24-milestone-packs",
    "docs/program/25-templates",
    "docs/program/26-policies",
    "docs/program/27-security",
    "docs/_generated",
    "docs/runbooks/operations/program",
    "docs/guides/developer/dev-guide",
    "docs/architecture/edge-mesh",
    "docs/architecture/data-graph",
    "docs/architecture/providers-agents",
    "docs/architecture/formal-foundation",
    "docs/guides/user/user-guide",
    "docs/program/policies/_policy",
]

FORBIDDEN_DOC_FILES = [
    "docs/reference/interfaces-legacy-notes.md",
]


def main() -> int:
    errors: list[str] = []

    for rel in REQUIRED_DIRS:
        if not (ROOT / rel).is_dir():
            errors.append(f"missing required docs dir: {rel}")

    for rel in REQUIRED_READMES:
        if not (ROOT / rel).is_file():
            errors.append(f"missing required docs README: {rel}")

    for rel in FORBIDDEN_DOC_DIRS:
        if (ROOT / rel).exists():
            errors.append(f"forbidden legacy docs area still present: {rel}")

    for rel in FORBIDDEN_DOC_FILES:
        if (ROOT / rel).exists():
            errors.append(f"forbidden legacy docs file still present: {rel}")

    if errors:
        print("docs_hierarchy: FAIL")
        for e in errors:
            print(f" - {e}")
        return 1

    print("docs_hierarchy: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
