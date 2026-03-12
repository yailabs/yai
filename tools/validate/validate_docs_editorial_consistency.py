#!/usr/bin/env python3
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

TARGETS = [
    "docs/architecture/README.md",
    "docs/architecture/overview/system-overview.md",
    "docs/architecture/runtime/architecture.md",
    "docs/architecture/workspace/architecture.md",
    "docs/architecture/foundation/architecture.md",
    "docs/architecture/mesh/architecture.md",
    "docs/architecture/protocol/transport.md",
    "docs/architecture/data/architecture.md",
    "docs/architecture/edge/daemon-local.md",
    "docs/architecture/control/assurance/architecture.md",
    "docs/guides/README.md",
    "docs/guides/developer/README.md",
    "docs/guides/developer/workflow/README.md",
    "docs/guides/developer/debugging/debugging.md",
    "docs/runbooks/README.md",
    "docs/runbooks/operations/operations.md",
    "docs/runbooks/qualification/qualification.md",
    "docs/runbooks/demos/demo.md",
    "docs/runbooks/remediation/remediation.md",
    "docs/reference/README.md",
    "docs/reference/protocol/README.md",
    "docs/reference/protocol/surface.md",
    "docs/reference/model/schema/README.md",
    "docs/reference/commands/README.md",
    "docs/reference/commands/behavior.md",
    "docs/program/README.md",
    "docs/program/rfc/README.md",
    "docs/program/adr/README.md",
    "docs/program/reports/README.md",
    "docs/program/rfc/rfc-001-runtime-topology-authority.md",
    "docs/program/rfc/rfc-002-unified-rpc-cli-contract.md",
    "docs/program/rfc/rfc-003-workspace-lifecycle-isolation.md",
    "docs/program/rfc/rfc-004-lock-pin-policy.md",
    "docs/program/rfc/rfc-005-formal-coverage-roadmap.md",
    "docs/program/adr/adr-001-single-runtime.md",
    "docs/program/adr/adr-006-unified-rpc.md",
    "docs/program/adr/adr-003-kernel-authority.md",
    "docs/program/adr/adr-012-audit-convergence-gates.md",
]

REQUIRED_FM = ["role:", "status:", "audience:", "owner_domain:"]
REQUIRED_SECTIONS = [
    "# Purpose",
    "# Scope",
    "# Relationships",
    "# Canonical Role",
    "# Main Body",
    "# Related Docs",
]

# Strict tone normalization is enforced on editorial framing blocks only.
FORBIDDEN_TONE = re.compile(
    r"\b(refactor|hardening|closeout|historical commentary|tranche)\b",
    flags=re.IGNORECASE,
)


def main() -> int:
    errors: list[str] = []
    for rel in TARGETS:
        p = ROOT / rel
        if not p.is_file():
            errors.append(f"missing editorial target: {rel}")
            continue

        text = p.read_text(encoding="utf-8", errors="ignore")
        if not text.startswith("---\n"):
            errors.append(f"missing front matter: {rel}")
            continue

        fm_end = text.find("\n---\n", 4)
        if fm_end < 0:
            errors.append(f"invalid front matter block: {rel}")
            continue

        fm = text[: fm_end + 5]
        for key in REQUIRED_FM:
            if key not in fm:
                errors.append(f"missing front matter key '{key[:-1]}' in {rel}")

        for sec in REQUIRED_SECTIONS:
            if sec not in text:
                errors.append(f"missing section '{sec}' in {rel}")

        # Tone normalization is evaluated on editorial body sections only,
        # excluding front matter metadata where legacy identifiers may remain.
        editorial_body = text[fm_end + 5 :]
        pre_main = editorial_body.split("# Main Body", 1)[0]
        if FORBIDDEN_TONE.search(pre_main):
            errors.append(f"forbidden editorial tone term in framing sections: {rel}")

    if errors:
        print("docs_editorial_consistency: FAIL")
        for e in errors:
            print(" -", e)
        return 1

    print("docs_editorial_consistency: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
