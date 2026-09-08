#!/usr/bin/env python3
"""Read-only roadmap structure/count audit, not a maturity verdict producer.

ROADMAP.md is the only maturity input. --summary prints a derived replacement
line; it never writes files or promotes a property. Evidence scope needs review.
"""
import argparse
from collections import Counter
from pathlib import Path
import re
import sys
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parents[2]
STATES = {"🟢 ESTABLISHED": "ESTABLISHED", "🟡 PARTIAL": "PARTIAL",
          "🔴 OPEN": "OPEN", "⚪ LATER": "LATER"}
TEMPORAL = {"SELECTED_NOT_STARTED", "IN_PROGRESS", "BLOCKED"}
AUTHORITY = "Authority: living public project control."
SECTIONS = (
    "At a Glance / Current Snapshot", "System Maturity", "Strategic Programs",
    "Cognitive State Spectrum", "Current Execution Sequence",
    "General Substrate Progression", "Product / Research Qualification Path",
    "Cross-axis Traceability", "Explicit Nonclaims and Deferred Scope",
    "Progression and Promotion Discipline", "Living-update Rules",
)
REFERENCE = re.compile(r"\[[^\]\n]+\]\[([^\]\n]+)\]")
INLINE = re.compile(r"\[[^\]\n]+\]\(([^)\n]+)\)")


def section(text, name):
    start, end = f"<!-- {name}:start -->", f"<!-- {name}:end -->"
    if text.count(start) != 1 or text.count(end) != 1:
        raise ValueError(f"expected one {name} marker pair")
    before, body = text.split(start)
    if end in before:
        raise ValueError(f"reversed {name} markers")
    return body.split(end)[0].strip()


def table_rows(body, header, allow_headings=False):
    rows = []
    for line in body.splitlines():
        if not line.strip() or (allow_headings and line.startswith("### ")):
            continue
        if not line.startswith("|") or not line.endswith("|"):
            raise ValueError(f"unexpected table content: {line}")
        cells = [cell.strip() for cell in line[1:-1].split("|")]
        if len(cells) != len(header) or not all(cells):
            raise ValueError(f"malformed table row: {line}")
        if cells == header or all(re.fullmatch(r":?-{3,}:?", c) for c in cells):
            continue
        rows.append(cells)
    if not rows:
        raise ValueError("empty control table")
    return rows


def summary(rows):
    counts = Counter(STATES[row[2]] for row in rows)
    return " ".join(f"{state}={counts[state]}" for state in STATES.values()) + f" TOTAL={len(rows)}"


def validate(text, root=ROOT, check_summary=True):
    root = root.resolve()
    if text.count(AUTHORITY) != 1:
        raise ValueError("expected one living public project control declaration")
    for heading in SECTIONS:
        if text.splitlines().count(f"## {heading}") != 1:
            raise ValueError(f"missing/duplicate control section: {heading}")
    # Reject competing live control surfaces, not historical evidence archives.
    documents = list(root.glob("*.md")) + list((root / "docs").rglob("*.md"))
    for path in documents:
        if path == root / "ROADMAP.md":
            continue
        if (path.stem.upper() in {"ROADMAP", "STATUS", "FEATURES", "PLAN"}
                or re.search(r"^Authority: living public project control\.",
                             path.read_text(encoding="utf-8"), re.M)):
            raise ValueError(f"competing live roadmap authority: {path.relative_to(root)}")

    definitions = {}
    for name, target in re.findall(r"^\[([^\]\n]+)\]:\s+(\S+)\s*$", text, re.M):
        name = name.casefold()
        if name in definitions:
            raise ValueError(f"duplicate reference: {name}")
        definitions[name] = target
    used = {name.casefold() for name in REFERENCE.findall(text)}
    if used - definitions.keys():
        raise ValueError(f"undefined references: {sorted(used-definitions.keys())}")
    for target in [*definitions.values(), *INLINE.findall(text)]:
        if target.startswith(("https://", "http://", "mailto:")):
            continue  # No network on the documentation lane; not live verification.
        path_text, _, fragment = target.partition("#")
        path = (root / unquote(path_text)).resolve() if path_text else root / "ROADMAP.md"
        if not path.is_relative_to(root) or not path.exists():
            raise ValueError(f"missing or escaping reference target: {target}")
        if fragment:
            content = text if path == root / "ROADMAP.md" else path.read_text(encoding="utf-8")
            anchors = set()
            for heading in re.findall(r"^#{1,6} (.+)$", content, re.M):
                slug = re.sub(r"[^\w\- ]", "", heading.lower()).replace(" ", "-")
                unique, suffix = slug, 0
                while unique in anchors:
                    suffix += 1
                    unique = f"{slug}-{suffix}"
                anchors.add(unique)
            if unquote(fragment) not in anchors:
                raise ValueError(f"missing heading reference: {target}")

    rows = table_rows(section(text, "maturity"),
                      ["ID", "Property", "Maturity", "Evidence / precise boundary"], True)
    ids = set()
    for identity, _, state, evidence in rows:
        if not re.fullmatch(r"[A-Z][0-9]{2}", identity) or identity in ids:
            raise ValueError(f"invalid/duplicate maturity identity: {identity}")
        ids.add(identity)
        if state not in STATES:
            raise ValueError(f"invalid maturity state: {identity}: {state}")
        if STATES[state] == "ESTABLISHED" and not (REFERENCE.search(evidence) or INLINE.search(evidence)):
            raise ValueError(f"established row lacks evidence link: {identity}")
    expected = summary(rows)
    actual = section(text, "maturity-summary")
    if check_summary and actual != expected:
        raise ValueError(f"stale maturity summary; expected: {expected}")

    execution = table_rows(section(text, "execution"),
                           ["Boundary", "Temporal state", "Programs", "Required after-state"])
    if len(execution) != 1 or execution[0][1] not in TEMPORAL:
        raise ValueError("expected exactly one selected execution boundary with a valid temporal state")
    boundary, state = execution[0][:2]
    snapshot = re.findall(r"^\| Selected engineering boundary \| (.+) \|$", text, re.M)
    if len(snapshot) != 1 or f"{boundary} — {state}" not in snapshot[0]:
        raise ValueError("snapshot and selected execution boundary disagree")
    return expected


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--summary", action="store_true", help="print counts from rows; do not write")
    args = parser.parse_args()
    try:
        result = validate((ROOT / "ROADMAP.md").read_text(encoding="utf-8"),
                          check_summary=not args.summary)
    except (ValueError, OSError) as error:
        print(f"check-roadmap: FAIL: {error}", file=sys.stderr)
        return 1
    print(result if args.summary else f"check-roadmap: PASS {result}; selected_boundaries=1")
    return 0


if __name__ == "__main__":
    sys.exit(main())
