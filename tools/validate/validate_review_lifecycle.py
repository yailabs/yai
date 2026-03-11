#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CLI = ROOT / "tools" / "bin" / "yai-govern"
CID = "enterprise.ecohmedia.src-ecohmedia-digital-outbound.candidate.v1"
REVIEW_FILE = ROOT / "governance" / "ingestion" / "review" / f"{CID.replace('/', '_')}.review.v1.json"


def run(*args: str, check: bool = True) -> str:
    proc = subprocess.run(
        [sys.executable, str(CLI), *args],
        cwd=str(ROOT),
        check=check,
        capture_output=True,
        text=True,
    )
    if check and proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or proc.stdout.strip())
    return proc.stdout


def inspect() -> dict:
    out = run("review", "inspect", CID)
    return json.loads(out)


def main() -> int:
    # Always reset to withdrawn to keep deterministic baseline.
    if REVIEW_FILE.exists():
        REVIEW_FILE.unlink()
    run("review", "status", CID)
    run("review", "submit", CID, "--by", "validator", "--note", "lifecycle_check")
    run("review", "approve", CID, "--by", "validator")
    state = inspect()
    lifecycle_state = str(state.get("lifecycle_state", ""))
    approval_state = str(state.get("approval_state", ""))
    if lifecycle_state not in ("approved", "apply_eligible"):
        print(f"ERR: unexpected lifecycle_state after approve: {lifecycle_state}", file=sys.stderr)
        return 2
    if approval_state != "approved":
        print(f"ERR: unexpected approval_state after approve: {approval_state}", file=sys.stderr)
        return 3

    run("review", "withdraw", CID, "--by", "validator", "--reason", "post_check_cleanup")
    final_state = inspect()
    if str(final_state.get("lifecycle_state", "")) != "withdrawn":
        print("ERR: lifecycle cleanup failed (expected withdrawn)", file=sys.stderr)
        return 4

    print("[review-lifecycle] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
