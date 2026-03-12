#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

REQUIRED = [
    "control/assurance/models/yai_system.tla",
    "control/assurance/traceability/model.map.json",
    "control/assurance/traceability/invariant-linkage.json",
    "control/assurance/traceability/enforcement-linkage.json",
    "control/assurance/model/schema/formal_traceability_map.v1.schema.json",
]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", required=True)
    args = parser.parse_args()

    root = Path(args.root).resolve()
    errors: list[str] = []

    for rel in REQUIRED:
        p = root / rel
        if not p.exists():
            errors.append(f"missing required formal file: {rel}")

    for rel in [
        "control/assurance/traceability/model.map.json",
        "control/assurance/traceability/invariant-linkage.json",
        "control/assurance/traceability/enforcement-linkage.json",
    ]:
        p = root / rel
        if p.exists():
            try:
                json.loads(p.read_text(encoding="utf-8"))
            except Exception as exc:
                errors.append(f"invalid json: {rel} ({exc})")

    if errors:
        print("formal_traceability_check: FAIL")
        for e in errors:
            print(" -", e)
        return 1

    print("formal_traceability_check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
