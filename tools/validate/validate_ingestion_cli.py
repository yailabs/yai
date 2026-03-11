#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CLI = ROOT / "tools" / "bin" / "yai-govern"


def run(*args: str) -> int:
    cmd = [str(CLI), *args]
    return subprocess.call(cmd, cwd=str(ROOT))


def main() -> int:
    if not CLI.exists():
        print(f"[ingestion-cli] FAIL: missing {CLI.relative_to(ROOT)}")
        return 1

    checks = [
        ("source", "list"),
        ("source", "inspect", "src.ecohmedia.digital-outbound"),
        ("parse", "src.ecohmedia.digital-outbound"),
        ("parsed", "inspect", "src.ecohmedia.digital-outbound"),
        ("normalize", "src.ecohmedia.digital-outbound"),
        ("normalized", "inspect", "norm.src-ecohmedia-digital-outbound"),
        ("build", "norm.src-ecohmedia-digital-outbound"),
        ("candidate", "inspect", "enterprise.ecohmedia.src-ecohmedia-digital-outbound.candidate.v1"),
        ("validate", "enterprise.ecohmedia.src-ecohmedia-digital-outbound.candidate.v1"),
        ("status", "enterprise.ecohmedia.src-ecohmedia-digital-outbound.candidate.v1"),
    ]

    for c in checks:
        rc = run(*c)
        if rc != 0:
            print(f"[ingestion-cli] FAIL: {' '.join(c)} rc={rc}")
            return 1

    print("[ingestion-cli] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
