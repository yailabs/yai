#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

FORBIDDEN = [
    "runtime-embedded",
    "foundation/contracts",
]

# README may mention removed wrappers in historical notes.
ALLOW_SUBSTR = {
    "tools/bin/README.md": ["removed; hard-fail"],
}

SCAN_ROOTS = [
    "Makefile",
    "tools/bin",
    "tools/dev",
    "tools/gen",
    "tools/validate",
    "tests/unit",
    "tests/integration",
]

EXT_OK = {".sh", ".py", ".mk", ".c", ".h", ""}


def should_scan(path: Path) -> bool:
    if path.is_dir():
        return False
    if path.name in {"Makefile"}:
        return True
    if path.suffix in EXT_OK:
        return True
    if path.name == "README.md" and str(path).replace("\\", "/") in ALLOW_SUBSTR:
        return True
    return False


def main() -> int:
    repo = Path(__file__).resolve().parents[2]
    errors: list[str] = []

    for root_rel in SCAN_ROOTS:
        root = repo / root_rel
        if not root.exists():
            continue
        files = [root] if root.is_file() else [p for p in root.rglob("*") if should_scan(p)]
        for f in files:
            rel = f.relative_to(repo).as_posix()
            if rel in {
                "tools/validate/validate_tooling_legacy_refs.py",
                "tools/validate/validate_root_framing.py",
                "tools/validate/validate_root_topology.py",
            }:
                continue
            text = f.read_text(encoding="utf-8", errors="ignore")
            for token in FORBIDDEN:
                if token not in text:
                    continue
                allow = ALLOW_SUBSTR.get(rel)
                if allow and all(a in text for a in allow):
                    continue
                errors.append(f"{rel}: forbidden legacy token '{token}'")

    if errors:
        print("tooling_legacy_refs: FAIL")
        for e in errors:
            print(" -", e)
        return 1

    print("tooling_legacy_refs: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
