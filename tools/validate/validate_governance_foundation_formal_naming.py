#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

SCAN_ROOTS = [
    "governance",
    "foundation",
    "formal",
    "tools/bin",
    "tools/validate",
    "lib/governance",
]

REQUIRED_CANONICAL = [
    "foundation/boundaries/L1-runtime-core.md",
    "foundation/boundaries/L2-orchestration.md",
    "foundation/boundaries/L3-agents.md",
    "formal/legacy/tla/yai_runtime_legacy.tla",
    "formal/legacy/tla/yai_ids_legacy.tla",
    "formal/legacy/tla/yai_precedence_legacy.tla",
    "formal/legacy/tla/yai_resolution_legacy.tla",
    "formal/legacy/configs/yai_runtime_legacy.cfg",
    "formal/legacy/configs/yai_runtime_legacy.quick.cfg",
    "formal/legacy/configs/yai_runtime_legacy.deep.cfg",
    "formal/legacy/formal_legacy_traceability.v1.json",
    "foundation/schema/compliance_context.v1.schema.json",
    "foundation/schema/retention_policy.v1.schema.json",
    "formal/schema/formal_traceability.v1.schema.json",
    "formal/schema/formal_resolution_trace.v1.schema.json",
]

FORBIDDEN_PATHS = [
    "foundation/boundaries/L1-core.md",
    "foundation/boundaries/L2-exec.md",
    "foundation/boundaries/L3-brain.md",
    "formal/legacy/tla/YAI_KERNEL.tla",
    "formal/legacy/tla/GOVERNANCE_IDS.tla",
    "formal/legacy/tla/GOVERNANCE_PRECEDENCE.tla",
    "formal/legacy/tla/GOVERNANCE_RESOLUTION.tla",
    "formal/legacy/configs/YAI_KERNEL.cfg",
    "formal/legacy/configs/YAI_KERNEL.quick.cfg",
    "formal/legacy/configs/YAI_KERNEL.deep.cfg",
    "formal/legacy/traceability.v1.json",
    "foundation/schema/compliance.context.v1.json",
    "foundation/schema/retention.policy.v1.json",
    "formal/schema/traceability.v1.schema.json",
    "formal/schema/resolution_trace.v1.schema.json",
]

FORBIDDEN_TOKENS = [
    "foundation/boundaries/L1-core.md",
    "foundation/boundaries/L2-exec.md",
    "foundation/boundaries/L3-brain.md",
    "formal/legacy/tla/YAI_KERNEL.tla",
    "formal/legacy/tla/GOVERNANCE_IDS.tla",
    "formal/legacy/tla/GOVERNANCE_PRECEDENCE.tla",
    "formal/legacy/tla/GOVERNANCE_RESOLUTION.tla",
    "formal/legacy/configs/YAI_KERNEL",
    "formal/legacy/traceability.v1.json",
    "schema/compliance.context.v1.json",
    "schema/retention.policy.v1.json",
    "formal/schema/traceability.v1.schema.json",
    "formal/schema/resolution_trace.v1.schema.json",
]


def iter_files(root: Path):
    if root.is_file():
        yield root
        return
    for p in root.rglob("*"):
        if p.is_file():
            yield p


def main() -> int:
    repo = Path(__file__).resolve().parents[2]
    errors: list[str] = []

    for rel in REQUIRED_CANONICAL:
        if not (repo / rel).exists():
            errors.append(f"missing canonical file: {rel}")

    for rel in FORBIDDEN_PATHS:
        if (repo / rel).exists():
            errors.append(f"forbidden legacy file present: {rel}")

    for rel in SCAN_ROOTS:
        root = repo / rel
        if not root.exists():
            continue
        for f in iter_files(root):
            fr = f.relative_to(repo).as_posix()
            if fr == "tools/validate/validate_governance_foundation_formal_naming.py":
                continue
            text = f.read_text(encoding="utf-8", errors="ignore")
            for token in FORBIDDEN_TOKENS:
                if token in text:
                    errors.append(f"{fr}: forbidden naming token '{token}'")

    if errors:
        print("governance_foundation_formal_naming: FAIL")
        for err in errors:
            print(" -", err)
        return 1

    print("governance_foundation_formal_naming: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
