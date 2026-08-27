#!/usr/bin/env python3
"""Validate local links in canonical and authority-boundary Markdown files."""

from __future__ import annotations

import re
import sys
from pathlib import Path
from urllib.parse import unquote


ROOT = Path(__file__).resolve().parents[2]
LINK = re.compile(r"(?<!!)\[[^\]]*\]\(([^)]+)\)")


def markdown_files() -> list[Path]:
    files = [ROOT / "README.md", ROOT / "ROADMAP.md"]
    files.extend((ROOT / "docs").rglob("*.md"))
    files.extend(
        ROOT / path
        for path in (
            "cmd/README.md",
            "engine/README.md",
            "include/yai/README.md",
            "system/README.md",
            "net/README.md",
            "proto/README.md",
            "proto/net.md",
            "proto/schemas/README.md",
            "proto/fixtures/README.md",
            "work/README.md",
            "work/spines/README.md",
            "work/waves/README.md",
            "work/discovery/README.md",
            "work/archive/engineering-snapshots/README.md",
            "labs/README.md",
            "labs/registry.md",
        )
    )
    return sorted({path for path in files if path.is_file()})


def target_from(raw: str) -> str:
    target = raw.strip()
    if target.startswith("<") and ">" in target:
        return target[1 : target.index(">")]
    # Markdown permits an optional title after a whitespace-separated target.
    return target.split(maxsplit=1)[0]


def main() -> int:
    errors: list[str] = []
    for source in markdown_files():
        text = source.read_text(encoding="utf-8")
        for match in LINK.finditer(text):
            raw_target = target_from(match.group(1))
            if not raw_target or raw_target.startswith(("#", "http://", "https://", "mailto:")):
                continue
            path_part = unquote(raw_target.split("#", 1)[0])
            if not path_part:
                continue
            target = (source.parent / path_part).resolve()
            try:
                target.relative_to(ROOT)
            except ValueError:
                errors.append(f"{source.relative_to(ROOT)}: link escapes repository: {raw_target}")
                continue
            if not target.exists():
                errors.append(f"{source.relative_to(ROOT)}: missing link target: {raw_target}")

    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print(f"check-doc-links: ok ({len(markdown_files())} files)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
