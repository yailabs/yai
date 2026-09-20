#!/usr/bin/env python3
"""Bounded model-independent qualification for Cognitive Decision Frontier v1."""

from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[3]
TESTS = (
    (
        "frontier",
        "store::lmdb::tests::resource_access_tests::decision_plane_tests::"
        "cognitive_decision_frontier_is_typed_bounded_and_evolves_without_scoring",
        (
            "cognitive_decision_frontier schema=v1",
            "workflow_state_evolution=true",
            "multi_origin_closure=true",
            "optional_omitted=9",
            "mandatory_overflow=refused",
            "same_generation_revoke=stale",
            "scoring_calls=0",
            "model_calls=0",
            "transitions_from_frontier=0",
            "operations=0",
            "effects=0",
        ),
    ),
    (
        "w4",
        "store::lmdb::tests::resource_access_tests::historical_tests::recall_tests::"
        "scoped_paging_exact_policy_group_rehydration_eviction_restart_and_no_discovery",
        (
            "frontier_deferred_opportunity=true",
            "frontier_content_exposed=false",
            "frontier_implicit_page_in=false",
            "frontier_resident_removed=true",
            "frontier_current_revoke_refuses=true",
            "candidate_discovery_passes=0",
        ),
    ),
)


def command(test: str) -> list[str]:
    return [
        "cargo",
        "test",
        "--manifest-path",
        "engine/Cargo.toml",
        test,
        "--",
        "--exact",
        "--nocapture",
        "--test-threads=1",
    ]


def main() -> int:
    environment = os.environ.copy()
    environment.setdefault("CARGO_TARGET_DIR", str(ROOT / "target"))
    environment.setdefault("RUSTUP_TOOLCHAIN", "1.98.1")
    print("decision_frontier_run_id=cognitive-decision-frontier-v1")
    print(f"working_directory={ROOT}")
    print(
        "environment=provider:none model:none scorer:none "
        "persistent_store:temporary_lmdb toolchain:1.98.1"
    )
    print(
        "material_pre_state=fresh temporary LMDB per test; qualified Case/W, "
        "Workflow, Resource and W4 fixtures constructed by selected Rust tests"
    )
    for order, (name, test, required) in enumerate(TESTS, start=1):
        selected = command(test)
        print(f"execution_order={order} lane={name}")
        print("command=" + " ".join(selected))
        completed = subprocess.run(
            selected,
            cwd=ROOT,
            env=environment,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            check=False,
        )
        print(completed.stdout, end="")
        print(f"exit_status={completed.returncode} lane={name}")
        if completed.returncode:
            return completed.returncode
        missing = [item for item in required if item not in completed.stdout]
        if missing:
            print(
                f"missing {name} qualification evidence: " + ", ".join(missing),
                file=sys.stderr,
            )
            return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
