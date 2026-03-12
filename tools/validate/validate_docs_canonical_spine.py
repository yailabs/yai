#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

REQUIRED = [
    "docs/architecture/workspace/architecture.md",
    "docs/architecture/workspace/boundaries.md",
    "docs/architecture/workspace/lifecycle.md",
    "docs/architecture/workspace/security.md",
    "docs/architecture/workspace/distribution.md",
    "docs/architecture/foundation/architecture.md",
    "docs/architecture/mesh/architecture.md",
    "docs/architecture/edge/daemon-local.md",
    "docs/architecture/data/architecture.md",
    "docs/architecture/graph/architecture.md",
    "docs/architecture/protocol/transport.md",
    "docs/program/reports/runtime-convergence-report.md",
    "docs/program/reports/audit-convergence-report.md",
    "docs/guides/developer/operational-guides/operations.md",
    "docs/guides/developer/operational-guides/workspace-operations.md",
    "docs/runbooks/operations/operations.md",
    "docs/runbooks/qualification/qualification.md",
    "docs/runbooks/demos/demo.md",
    "docs/runbooks/remediation/remediation.md",
]

FORBIDDEN = [
    "docs/architecture/workspace/workspace-validation-matrix.md",
    "docs/architecture/workspace/workspace-closeout-ws6.md",
    "docs/architecture/foundation/source-plane-model-refoundation-rf01.md",
    "docs/guides/developer/workspace-governance-apply-walkthrough.md",
    "docs/runbooks/demos/demo-workspace-runbook.md",
    "docs/runbooks/qualification/qualification-lan-qualification.md",
    "docs/program/reports/workspace-verticalization-gap-classification-v0.1.0.md",
    "docs/program/reports/audit-convergence/UNIFIED-RUNTIME-MANUAL-TEST-COMMAND-PACK-v0.1.0.md",
]


def main() -> int:
    errs: list[str] = []
    for rel in REQUIRED:
        if not (ROOT / rel).is_file():
            errs.append(f"missing required canonical spine doc: {rel}")

    for rel in FORBIDDEN:
        if (ROOT / rel).exists():
            errs.append(f"forbidden overlap/satellite doc still present: {rel}")

    if errs:
        print("docs_canonical_spine: FAIL")
        for e in errs:
            print(" -", e)
        return 1

    print("docs_canonical_spine: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
