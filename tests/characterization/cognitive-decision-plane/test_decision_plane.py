#!/usr/bin/env python3
"""Bounded model-independent qualification for Cognitive Decision Plane v1."""

from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[3]
TEST = (
    "store::lmdb::tests::resource_access_tests::decision_plane_tests::"
    "cognitive_decision_plane_is_w_bound_non_authoritative_and_current"
)
COMMAND = [
    "cargo",
    "test",
    "--manifest-path",
    "engine/Cargo.toml",
    TEST,
    "--",
    "--exact",
    "--nocapture",
    "--test-threads=1",
]


def main() -> int:
    environment = os.environ.copy()
    environment.setdefault("CARGO_TARGET_DIR", str(ROOT / "target"))
    print("decision_plane_run_id=cognitive-decision-plane-v1")
    print("execution_order=1")
    print(f"working_directory={ROOT}")
    print("command=" + " ".join(COMMAND))
    print("environment=provider:none model:none persistent_store:temporary_lmdb")
    print(
        "material_pre_state=fresh temporary LMDB; qualified Case/W fixture "
        "constructed by the selected Rust test"
    )
    completed = subprocess.run(
        COMMAND,
        cwd=ROOT,
        env=environment,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    print(completed.stdout, end="")
    print(f"exit_status={completed.returncode}")
    if completed.returncode:
        return completed.returncode
    required = (
        "cognitive_decision_plane schema=v1",
        "score_authority=false",
        "transitions=0",
        "effects=0",
        "provider_calls=0",
        "restart=equal",
        "same_generation_revoke=stale",
    )
    missing = [item for item in required if item not in completed.stdout]
    if missing:
        print("missing qualification evidence: " + ", ".join(missing), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
