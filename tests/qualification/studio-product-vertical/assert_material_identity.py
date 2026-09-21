#!/usr/bin/env python3
"""Read-only exact-material oracle for the persistent Studio qualification Case."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
from pathlib import Path


PATHS = (
    "application/yai-application/Cargo.toml",
    "cmd/README.md",
    "studio/package.json",
    "studio/README.md",
    "studio/ROADMAP.md",
    "tests/qualification/studio-product-vertical/policy.json",
)


def operation(command: list[str], yai_home: Path) -> dict:
    environment = dict(os.environ)
    environment["YAI_HOME"] = str(yai_home)
    completed = subprocess.run(command, env=environment, check=True, text=True, capture_output=True)
    result = json.loads(completed.stdout)
    if result.get("result_state") != "success" or not result.get("data"):
        raise AssertionError(f"operation failed: {command}: {result}")
    return result["data"]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--yai-home", type=Path, required=True)
    parser.add_argument("--case", default="case:studio-live-qualification")
    parser.add_argument("--summary", default="target/debug/examples/case_summary")
    parser.add_argument("--read", default="target/debug/examples/material_read")
    arguments = parser.parse_args()

    summary = operation([arguments.summary, arguments.case], arguments.yai_home)
    files = {item["path"]: item for item in summary["environment"]["files"]}
    records = []
    for path in PATHS:
        expected = files[path]
        actual = operation(
            [arguments.read, arguments.case, expected["source_ref"], expected["revision_ref"], path],
            arguments.yai_home,
        )
        for field in ("case_ref", "source_ref", "revision_ref", "path", "digest", "bytes", "media_type"):
            wanted = arguments.case if field == "case_ref" else expected[field]
            if actual[field] != wanted:
                raise AssertionError(f"{path}: {field}: expected {wanted!r}, got {actual[field]!r}")
        raw = actual["content"].encode("utf-8")
        if len(raw) != actual["bytes"]:
            raise AssertionError(f"{path}: byte count mismatch")
        records.append(
            {
                "path": path,
                "source_ref": actual["source_ref"],
                "revision_ref": actual["revision_ref"],
                "digest": actual["digest"],
                "media_type": actual["media_type"],
                "bytes": actual["bytes"],
                "generation": actual["generation"],
                "content_sha256": hashlib.sha256(raw).hexdigest(),
                "fingerprint": actual["content"][:96].replace("\n", "\\n"),
            }
        )
    print(json.dumps({"case_ref": arguments.case, "generation": summary["case"]["generation"], "materials": records}, indent=2))


if __name__ == "__main__":
    main()
