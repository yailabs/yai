#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

CANONICAL_REQUIRED = {
    "cmd",
    "include",
    "lib",
    "governance",
    "foundation",
    "formal",
    "docs",
    "tests",
    "tools",
    "data",
}

FORBIDDEN_ROOT_NAMES = {
    "transitional",
    "archive_tmp",
    "embedded",
    "grammar",
    "schema",
    "registry",
    "manifests",
    "overlays",
    "domains",
    "packs",
    "ingestion",
    "vectors",
    "families",
    "specializations",
    "classification",
}

FORBIDDEN_ROOT_FILES: set[str] = set()

REQUIRED_FOUNDATION_SUBDIRS = {"axioms", "invariants", "boundaries", "extensions", "terminology"}
REQUIRED_FORMAL_SUBDIRS = {
    "audit",
    "artifacts",
    "configs",
    "models",
    "modules",
    "schema",
    "traceability",
}
REQUIRED_MESH_IMPL_FILES = {
    "lib/mesh/identity/identity.c",
    "lib/mesh/peer_model/registry/peer_registry.c",
    "lib/mesh/membership/membership.c",
    "lib/mesh/discovery/discovery.c",
    "lib/mesh/awareness/awareness.c",
    "lib/mesh/coordination/coordination.c",
}
REQUIRED_EDGE_IMPL_FILES = {
    "lib/edge/lifecycle.c",
    "lib/edge/binding.c",
    "lib/edge/actions.c",
    "lib/edge/state.c",
    "lib/edge/runtime.c",
}
REQUIRED_RUNTIME_IMPL_FILES = {
    "lib/runtime/policy/policy_state.c",
    "lib/runtime/grants/grants_state.c",
    "lib/runtime/containment/containment_state.c",
}


IGNORED_ROOT = {
    ".git",
    ".github",
    ".vscode",
    ".pr",
    "__pycache__",
    "build",
    "dist",
}


def present_root_entries(repo: Path) -> set[str]:
    return {p.name for p in repo.iterdir() if p.name not in IGNORED_ROOT}


def main() -> int:
    repo = Path(__file__).resolve().parents[2]
    root_entries = present_root_entries(repo)
    errors: list[str] = []

    missing = sorted(CANONICAL_REQUIRED - root_entries)
    for entry in missing:
        errors.append(f"missing canonical root entry: {entry}")

    forbidden_dirs = sorted(FORBIDDEN_ROOT_NAMES & root_entries)
    for entry in forbidden_dirs:
        errors.append(f"forbidden root directory present: {entry}")

    forbidden_files = sorted(FORBIDDEN_ROOT_FILES & root_entries)
    for entry in forbidden_files:
        errors.append(f"forbidden root file present: {entry}")

    # C2 semantic boundary guardrails.
    if (repo / "governance" / "foundation").exists():
        errors.append("forbidden governance duplicate present: foundation/foundation")
    if (repo / "governance" / "formal").exists():
        errors.append("forbidden governance duplicate present: foundation/formal")
    if (repo / "governance" / "contracts").exists():
        errors.append("forbidden secondary protocol root present: foundation/contracts")

    for name in sorted(REQUIRED_FOUNDATION_SUBDIRS):
        if not (repo / "foundation" / name).exists():
            errors.append(f"foundation/ missing required subdir: {name}")
    for name in sorted(REQUIRED_FORMAL_SUBDIRS):
        if not (repo / "formal" / name).exists():
            errors.append(f"control/assurance/ missing required subdir: {name}")
    for rel in sorted(REQUIRED_MESH_IMPL_FILES):
        if not (repo / rel).exists():
            errors.append(f"mesh realization missing critical file: {rel}")
    for rel in sorted(REQUIRED_EDGE_IMPL_FILES):
        if not (repo / rel).exists():
            errors.append(f"edge realization missing critical file: {rel}")
    for rel in sorted(REQUIRED_RUNTIME_IMPL_FILES):
        if not (repo / rel).exists():
            errors.append(f"runtime realization missing critical file: {rel}")

    if errors:
        print("root_topology: FAIL")
        for e in errors:
            print(" -", e)
        return 1

    print("root_topology: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
