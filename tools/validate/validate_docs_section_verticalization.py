#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

REQUIRED_DIRS = [
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
    "docs/guides/developer/operational-guides",
    "docs/reference/commands",
    "docs/reference/protocol",
    "docs/reference/schemas",
]

REQUIRED_READMES = [
    "docs/architecture/foundation/README.md",
    "docs/architecture/control/assurance/README.md",
    "docs/architecture/runtime/README.md",
    "docs/architecture/orchestration/README.md",
    "docs/architecture/edge/README.md",
    "docs/architecture/protocol/README.md",
    "docs/architecture/data/README.md",
    "docs/architecture/graph/README.md",
    "docs/architecture/knowledge/README.md",
    "docs/architecture/mesh/README.md",
    "docs/architecture/workspace/README.md",
    "docs/guides/developer/operational-guides/README.md",
    "docs/reference/commands/README.md",
    "docs/reference/protocol/README.md",
    "docs/reference/model/schema/README.md",
    "docs/reference/README.md",
]

FORBIDDEN_DIRS = [
    "docs/reference/cli",
    "docs/reference/protocol/contracts",
    "docs/reference/protocol/control",
    "docs/reference/protocol/protocol",
    "docs/reference/protocol/providers",
    "docs/reference/protocol/vault",
    "docs/reference/protocol/compliance",
    "docs/reference/protocol/cli",
    "docs/runbooks/operations/developer-runbooks",
]

ROOT_MD_ALLOWLIST = {
    "docs/architecture/runtime": {
        "README.md",
        "architecture.md",
        "enforcement.md",
        "data-sinks.md",
        "resolution.md",
    },
    "docs/architecture/protocol": {
        "README.md",
        "acquisition-plane.md",
        "source-plane.md",
        "transport.md",
    },
    "docs/architecture/mesh": {
        "README.md",
        "architecture.md",
        "topology.md",
    },
    "docs/architecture/edge": {
        "README.md",
        "daemon-local.md",
        "policy-enforcement.md",
    },
    "docs/architecture/data": {
        "README.md",
        "architecture.md",
    },
    "docs/architecture/graph": {
        "README.md",
        "architecture.md",
    },
    "docs/architecture/knowledge": {
        "README.md",
        "architecture.md",
    },
    "docs/architecture/foundation": {
        "README.md",
        "architecture.md",
    },
    "docs/architecture/formal": {
        "README.md",
        "architecture.md",
    },
    "docs/architecture/orchestration": {
        "README.md",
        "architecture.md",
    },
    "docs/architecture/workspace": {
        "README.md",
        "architecture.md",
        "boundaries.md",
        "lifecycle.md",
        "security.md",
        "distribution.md",
    },
    "docs/guides/developer": {"README.md"},
    "docs/reference/commands": {
        "README.md",
        "surface.md",
        "taxonomy.md",
        "behavior.md",
    },
    "docs/reference/protocol": {
        "README.md",
        "surface.md",
        "message-types.md",
        "transport.md",
        "rpc.md",
        "binary.md",
    },
    "docs/reference/schemas": {"README.md", "specs.md"},
}


def main() -> int:
    errors: list[str] = []

    for rel in REQUIRED_DIRS:
        if not (ROOT / rel).is_dir():
            errors.append(f"missing required verticalized dir: {rel}")

    for rel in REQUIRED_READMES:
        path = ROOT / rel
        if not path.is_file():
            errors.append(f"missing required section README: {rel}")
        else:
            text = path.read_text(encoding="utf-8").strip()
            if len(text.splitlines()) < 3:
                errors.append(f"section README too weak: {rel}")

    for rel in FORBIDDEN_DIRS:
        if (ROOT / rel).exists():
            errors.append(f"forbidden pre-verticalization dir still present: {rel}")

    for rel, allowed in ROOT_MD_ALLOWLIST.items():
        section = ROOT / rel
        if not section.is_dir():
            continue
        names = {p.name for p in section.glob("*.md")}
        extra = sorted(names - allowed)
        if extra:
            errors.append(
                f"section root has non-canonical files in {rel}: {', '.join(extra)}"
            )

    if errors:
        print("docs_section_verticalization: FAIL")
        for e in errors:
            print(" -", e)
        return 1

    print("docs_section_verticalization: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
