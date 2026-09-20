#!/usr/bin/env python3
"""Bounded model-free qualification for the Cognitive Decision hot path."""

from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[3]
TESTS = (
    (
        "single-step",
        "store::lmdb::tests::resource_access_tests::decision_plane_tests::"
        "cognitive_decision_hot_path_composes_one_snapshot_and_retains_final_fence",
        (
            "cognitive_decision_hot_path schema=v1",
            "baseline_read_transactions=12",
            "optimized_read_transactions=2",
            "optimized_w_recompilations=2",
            "optimized_candidate_discovery_passes=1",
            "same_generation_revoke=stale",
            "tampered_frontier=refused",
            "restart_stale=refused",
            "transitions=0",
            "effects=0",
            "model_calls=0",
            "provider_calls=0",
        ),
    ),
    (
        "repeated-pressure",
        "store::lmdb::tests::resource_access_tests::decision_plane_tests::"
        "cognitive_decision_hot_path_repeated_and_pressure_characterization",
        (
            "cognitive_decision_hot_path_repeated",
            "steps_1_reads=2",
            "steps_10_reads=20",
            "steps_100_reads=200",
            "candidate_passes_100=100",
            "irrelevant_change=conservative_stale",
            "pressure_selected=32",
            "pressure_omitted=9",
            "transitions_from_decisions=0",
        ),
    ),
    (
        "w4-security",
        "store::lmdb::tests::resource_access_tests::historical_tests::recall_tests::"
        "scoped_paging_exact_policy_group_rehydration_eviction_restart_and_no_discovery",
        (
            "frontier_deferred_opportunity=true",
            "frontier_content_exposed=false",
            "frontier_implicit_page_in=false",
            "frontier_current_revoke_refuses=true",
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
    print("decision_hot_path_run_id=cognitive-decision-hot-path-v1")
    print(f"working_directory={ROOT}")
    print(
        "environment=provider:none model:none scorer:deterministic_fixture "
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
