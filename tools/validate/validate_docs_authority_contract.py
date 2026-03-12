#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs"

REQUIRED_ARTIFACTS = [
    "docs/archive/migration/d18.2-canonical-authority-audit.md",
    "docs/archive/migration/d18.2-source-of-truth-contract-map.md",
    "docs/archive/migration/d18.2-cross-section-authority-contract.md",
]

CANONICAL_OWNERS = {
    "platform-overview": "docs/architecture/overview/system-overview.md",
    "runtime-architecture": "docs/architecture/runtime/architecture.md",
    "workspace-architecture": "docs/architecture/workspace/architecture.md",
    "mesh-architecture": "docs/architecture/mesh/architecture.md",
    "foundation-architecture": "docs/architecture/foundation/architecture.md",
    "protocol-architecture": "docs/architecture/protocol/transport.md",
    "data-architecture": "docs/architecture/data/architecture.md",
    "edge-architecture": "docs/architecture/edge/daemon-local.md",
    "graph-architecture": "docs/architecture/graph/architecture.md",
    "knowledge-architecture": "docs/architecture/knowledge/architecture.md",
    "orchestration-architecture": "docs/architecture/orchestration/architecture.md",
    "protocol-reference": "docs/reference/protocol/README.md",
    "schemas-reference": "docs/reference/schemas/README.md",
    "commands-reference": "docs/reference/commands/README.md",
    "operations-procedure": "docs/runbooks/operations/operations.md",
    "qualification-procedure": "docs/runbooks/qualification/qualification.md",
    "demos-procedure": "docs/runbooks/demos/demo.md",
    "remediation-procedure": "docs/runbooks/remediation/remediation.md",
    "runtime-convergence-evidence": "docs/program/reports/runtime-convergence-report.md",
    "audit-convergence-evidence": "docs/program/reports/audit-convergence-report.md",
}

HIGH_VISIBILITY = [
    "docs/README.md",
    "docs/architecture/README.md",
    "docs/guides/README.md",
    "docs/runbooks/README.md",
    "docs/reference/README.md",
    "docs/program/README.md",
] + list(CANONICAL_OWNERS.values())

VALID_ROLES = {"canonical", "support", "procedural", "reference", "historical"}


def parse_front_matter(text: str) -> dict[str, str]:
    if not text.startswith("---\n"):
        return {}
    end = text.find("\n---\n", 4)
    if end < 0:
        return {}
    block = text[4:end]
    out: dict[str, str] = {}
    for raw in block.splitlines():
        line = raw.strip()
        if not line or ":" not in line:
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
    fm_by_path: dict[str, dict[str, str]] = {}
    primary_index: dict[str, list[str]] = {}

    for rel in REQUIRED_ARTIFACTS:
        if not (ROOT / rel).is_file():
            errors.append(f"missing required D18.2 artifact: {rel}")

    for p in DOCS.rglob("*.md"):
        rel = p.relative_to(ROOT).as_posix()
        fm = parse_front_matter(p.read_text(encoding="utf-8", errors="ignore"))
        if fm:
            fm_by_path[rel] = fm
            topic = fm.get("primary_for", "").strip()
            if topic:
                primary_index.setdefault(topic, []).append(rel)

            role = fm.get("role", "").strip()
            if role and role not in VALID_ROLES:
                errors.append(f"invalid role '{role}' in {rel}")

            if role == "historical" and not rel.startswith("docs/archive/"):
                errors.append(f"historical doc must live under docs/archive: {rel}")

            if fm.get("status", "").strip() == "superseded" and not fm.get("superseded_by", "").strip():
                errors.append(f"superseded doc missing superseded_by: {rel}")

            if rel in HIGH_VISIBILITY and role in {"support", "procedural", "reference"}:
                deps = parse_list(fm.get("depends_on", ""))
                if not deps:
                    errors.append(f"{role} doc missing depends_on: {rel}")
                for dep in deps:
                    if dep.startswith("docs/archive/"):
                        errors.append(f"live doc depends on archive authority: {rel} -> {dep}")

    for topic, rel in CANONICAL_OWNERS.items():
        p = ROOT / rel
        if not p.is_file():
            errors.append(f"canonical owner missing for topic '{topic}': {rel}")
            continue
        fm = fm_by_path.get(rel)
        if not fm:
            errors.append(f"canonical owner missing front matter: {rel}")
            continue
        if fm.get("primary_for", "").strip() != topic:
            errors.append(f"canonical owner has wrong primary_for in {rel} (expected '{topic}')")
        if fm.get("status", "").strip() in {"", "historical", "superseded"}:
            errors.append(f"canonical owner has non-active status in {rel}")

    for topic, owners in primary_index.items():
        live = [p for p in owners if not p.startswith("docs/archive/")]
        if len(live) > 1:
            errors.append(
                f"multiple live docs claim same primary_for '{topic}': {', '.join(sorted(live))}"
            )

    for rel in HIGH_VISIBILITY:
        p = ROOT / rel
        if not p.is_file():
            errors.append(f"missing high-visibility doc: {rel}")
            continue
        fm = fm_by_path.get(rel)
        if not fm:
            errors.append(f"high-visibility doc missing front matter: {rel}")
            continue
        for key in ("role", "status", "owner_domain"):
            if not fm.get(key, "").strip():
                errors.append(f"high-visibility doc missing '{key}': {rel}")

    if errors:
        print("docs_authority_contract: FAIL")
        for err in errors:
            print(" -", err)
        return 1

    print("docs_authority_contract: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
