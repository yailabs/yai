#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

SCAN_ROOTS = ["lib", "tests", "tools", "cmd", "Makefile"]

LEGACY_IMPL_PATHS = [
    "lib/edge/action_point.c",
    "lib/edge/edge_binding.c",
    "lib/edge/edge_services.c",
    "lib/edge/edge_state.c",
    "lib/edge/local_runtime.c",
    "lib/edge/source_ids.c",
    "lib/edge/source_plane_model.c",
    "lib/orchestration/bridge/engine_bridge.c",
    "lib/orchestration/runtime/exec_runtime.c",
    "lib/orchestration/runtime/source_ingest.c",
    "lib/orchestration/gates/network_gate.c",
    "lib/orchestration/gates/provider_gate.c",
    "lib/orchestration/gates/resource_gate.c",
    "lib/orchestration/gates/storage_gate.c",
    "lib/orchestration/transport/brain_protocol.c",
    "lib/orchestration/transport/brain_transport.c",
    "lib/providers/model/registry/providers.c",
]

LEGACY_PATH_TOKENS = [
    "lib/edge/action_point.c",
    "lib/edge/edge_binding.c",
    "lib/edge/edge_services.c",
    "lib/edge/edge_state.c",
    "lib/edge/local_runtime.c",
    "lib/edge/source_ids.c",
    "lib/edge/source_plane_model.c",
    "lib/orchestration/bridge/engine_bridge.c",
    "lib/orchestration/runtime/exec_runtime.c",
    "lib/orchestration/runtime/source_ingest.c",
    "lib/orchestration/gates/network_gate.c",
    "lib/orchestration/gates/provider_gate.c",
    "lib/orchestration/gates/resource_gate.c",
    "lib/orchestration/gates/storage_gate.c",
    "lib/orchestration/transport/brain_protocol.c",
    "lib/orchestration/transport/brain_transport.c",
    "lib/providers/model/registry/providers.c",
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

    for rel in LEGACY_IMPL_PATHS:
        if (repo / rel).exists():
            errors.append(f"legacy implementation file still present: {rel}")

    for rel in SCAN_ROOTS:
        root = repo / rel
        if not root.exists():
            continue
        for f in iter_files(root):
            fr = f.relative_to(repo).as_posix()
            if fr == "tools/validate/validate_implementation_naming.py":
                continue
            text = f.read_text(encoding="utf-8", errors="ignore")
            for token in LEGACY_PATH_TOKENS:
                if token in text:
                    errors.append(f"{fr}: legacy implementation token '{token}'")

    if errors:
        print("implementation_naming: FAIL")
        for err in errors:
            print(" -", err)
        return 1

    print("implementation_naming: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
