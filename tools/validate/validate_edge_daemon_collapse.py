#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

SCAN_ROOTS = [
    "cmd",
    "include",
    "lib",
    "tests",
    "tools",
    "Makefile",
    "README.md",
    "FOUNDATION.md",
    "GOVERNANCE.md",
    "COMPATIBILITY.md",
    "VERSIONING.md",
]

FORBIDDEN_TOKENS = [
    "<yai/daemon/",
    "include/yai/daemon",
    "lib/daemon/",
    "cmd/yai-daemon/",
    "daemon_local_runtime_scan_spool_retry_v1.sh",
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

    if (repo / "include" / "yai" / "daemon").exists():
        errors.append("forbidden canonical namespace present: include/yai/daemon")
    if (repo / "lib" / "daemon").exists():
        errors.append("forbidden canonical implementation domain present: lib/daemon")
    if (repo / "cmd" / "yai-daemon").exists():
        errors.append("forbidden canonical entrypoint domain present: cmd/yai-daemon")

    # C13/C16 guardrail: canonical edge subdomains must not be placeholder-empty.
    edge_root = repo / "lib" / "edge"
    if edge_root.exists():
        for d in edge_root.iterdir():
            if d.is_dir() and not any(d.iterdir()):
                errors.append(f"empty edge placeholder directory not allowed: {d.relative_to(repo).as_posix()}")

    for rel in SCAN_ROOTS:
        root = repo / rel
        if not root.exists():
            continue
        for f in iter_files(root):
            fr = f.relative_to(repo).as_posix()
            if fr.startswith("docs/") or fr.startswith("transitional/"):
                continue
            if fr == "tools/validate/validate_edge_daemon_collapse.py":
                continue
            text = f.read_text(encoding="utf-8", errors="ignore")
            for token in FORBIDDEN_TOKENS:
                if token in text:
                    errors.append(f"{fr}: forbidden daemon-domain token '{token}'")

    mk = (repo / "Makefile").read_text(encoding="utf-8", errors="ignore")
    if "\nyai-edge:" not in mk:
        errors.append("Makefile: missing canonical yai-edge target")
    if "\ndaemon:" in mk and "legacy alias" not in mk:
        errors.append("Makefile: daemon target exists without explicit legacy-alias marker")
    if "\nyai-daemon:" in mk and "legacy alias" not in mk:
        errors.append("Makefile: yai-daemon target exists without explicit legacy-alias marker")

    if errors:
        print("edge_daemon_collapse: FAIL")
        for e in errors:
            print(" -", e)
        return 1

    print("edge_daemon_collapse: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
