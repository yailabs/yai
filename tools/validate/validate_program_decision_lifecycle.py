#!/usr/bin/env python3
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

LEDGER = ROOT / "docs/program/archive/legacy/decision-ledger.md"

CORE_SETS = {
    "adr": sorted((ROOT / "docs/program/adr").glob("adr-*.md")),
    "rfc": sorted((ROOT / "docs/program/rfc").glob("rfc-*.md")),
    "report": sorted((ROOT / "docs/program/reports").glob("*-report.md")),
}

README_SET = [
    ROOT / "docs/program/README.md",
    ROOT / "docs/program/adr/README.md",
    ROOT / "docs/program/rfc/README.md",
    ROOT / "docs/program/reports/README.md",
    ROOT / "docs/program/archive/README.md",
]

ALLOWED = {
    "adr": {"draft", "accepted", "active", "superseded", "historical"},
    "rfc": {"draft", "accepted", "active", "superseded", "historical"},
    "report": {"active", "superseded", "historical"},
}

REQUIRED_KEYS = {
    "role",
    "status",
    "decision_id",
    "owner_domain",
    "supersedes",
    "superseded_by",
    "implements",
    "evidenced_by",
    "related",
}

ID_RE = re.compile(r"^(ADR|RFC|RPT)-\d{3}$")
TABLE_ID_RE = re.compile(r"^(ADR|RFC|RPT)-\d{3}$")


def parse_front_matter(path: Path) -> dict[str, str]:
    text = path.read_text(encoding="utf-8", errors="ignore")
    if not text.startswith("---\n"):
        return {}
    end = text.find("\n---\n", 4)
    if end < 0:
        return {}
    block = text[4:end]
    data: dict[str, str] = {}
    for line in block.splitlines():
        if ":" not in line:
            continue
        key, value = line.split(":", 1)
        data[key.strip()] = value.strip()
    return data


def parse_list(value: str) -> list[str]:
    value = value.strip()
    if not value:
        return []
    if value.startswith("[") and value.endswith("]"):
        inner = value[1:-1].strip()
        if not inner:
            return []
        return [v.strip().strip("'\"") for v in inner.split(",") if v.strip()]
    return [value.strip().strip("'\"")]


def ledger_rows(text: str) -> dict[str, str]:
    rows: dict[str, str] = {}
    for line in text.splitlines():
        line = line.strip()
        if not line.startswith("|"):
            continue
        cols = [c.strip() for c in line.split("|")[1:-1]]
        if len(cols) < 2:
            continue
        did = cols[0]
        owner = cols[1].strip("`")
        if TABLE_ID_RE.match(did) and owner.startswith("docs/"):
            rows[did] = owner
    return rows


def current_active_ids(text: str) -> set[str]:
    out: set[str] = set()
    section = None
    for line in text.splitlines():
        if line.startswith("## "):
            section = line.strip()
        if section != "## Current Active Decisions":
            continue
        s = line.strip()
        if not s.startswith("|"):
            continue
        cols = [c.strip() for c in s.split("|")[1:-1]]
        if cols and TABLE_ID_RE.match(cols[0]):
            out.add(cols[0])
    return out


def main() -> int:
    errors: list[str] = []

    if not LEDGER.is_file():
        errors.append("missing decision ledger: docs/program/archive/legacy/decision-ledger.md")
        print("program_decision_lifecycle: FAIL")
        for e in errors:
            print(" -", e)
        return 1

    fm_by_path: dict[str, dict[str, str]] = {}
    active_owner_by_id: dict[str, list[str]] = {}
    all_core_paths: list[Path] = []

    for family, paths in CORE_SETS.items():
        for p in paths:
            rel = p.relative_to(ROOT).as_posix()
            all_core_paths.append(p)
            if not p.is_file():
                errors.append(f"missing program core doc: {rel}")
                continue
            fm = parse_front_matter(p)
            if not fm:
                errors.append(f"missing front matter: {rel}")
                continue
            fm_by_path[rel] = fm
            missing = [k for k in REQUIRED_KEYS if not fm.get(k, "").strip()]
            for m in missing:
                errors.append(f"missing key '{m}' in {rel}")

            status = fm.get("status", "")
            if status and status not in ALLOWED[family]:
                errors.append(f"invalid status '{status}' for {family} doc: {rel}")

            did = fm.get("decision_id", "")
            if did and not ID_RE.match(did) and did not in {"MP-INDEX"}:
                errors.append(f"invalid decision_id format in {rel}: {did}")

            if status in {"active", "accepted"} and did:
                active_owner_by_id.setdefault(did, []).append(rel)

            if status in {"superseded", "historical"} and not parse_list(fm.get("superseded_by", "")):
                errors.append(f"{status} doc missing superseded_by: {rel}")

    # Consistency for supersedes/superseded_by links.
    for rel, fm in fm_by_path.items():
        src = rel
        for target in parse_list(fm.get("supersedes", "")):
            if not target:
                continue
            tp = ROOT / target
            if not tp.is_file():
                errors.append(f"supersedes target missing: {src} -> {target}")
                continue
            tfm = parse_front_matter(tp)
            back = parse_list(tfm.get("superseded_by", ""))
            if src not in back:
                errors.append(f"missing reciprocal superseded_by: {target} should reference {src}")

    for did, owners in active_owner_by_id.items():
        if len(owners) > 1:
            errors.append(
                f"multiple active/accepted owners for decision_id {did}: {', '.join(sorted(owners))}"
            )

    # README guards.
    for p in README_SET:
        rel = p.relative_to(ROOT).as_posix()
        if not p.is_file():
            errors.append(f"missing program README: {rel}")
            continue
        fm = parse_front_matter(p)
        if not fm:
            errors.append(f"missing front matter in README: {rel}")
            continue
        for key in ("decision_id", "status"):
            if not fm.get(key, "").strip():
                errors.append(f"README missing '{key}': {rel}")

    # Ledger alignment.
    ledger_text = LEDGER.read_text(encoding="utf-8", errors="ignore")
    rows = ledger_rows(ledger_text)
    active_ids = current_active_ids(ledger_text)

    if not rows:
        errors.append("decision ledger has no parseable decision rows")

    # Every ADR/RFC/report should appear in the ledger by decision_id.
    expected_ids: dict[str, str] = {}
    for p in all_core_paths:
        rel = p.relative_to(ROOT).as_posix()
        fm = fm_by_path.get(rel, {})
        did = fm.get("decision_id", "")
        if did and did != "MP-INDEX":
            expected_ids[did] = rel

    for did, rel in expected_ids.items():
        if did not in rows:
            errors.append(f"decision_id missing from ledger: {did} ({rel})")

    for did, owner in rows.items():
        p = ROOT / owner
        if not p.is_file():
            errors.append(f"ledger owner_doc missing: {did} -> {owner}")
            continue
        fm = parse_front_matter(p)
        if fm.get("decision_id", "") != did:
            errors.append(f"ledger mismatch decision_id for {owner}: expected {did}")

    for did in active_ids:
        owner = rows.get(did)
        if not owner:
            continue
        fm = parse_front_matter(ROOT / owner)
        if fm.get("status", "") in {"historical", "superseded"}:
            errors.append(f"historical/superseded decision listed in Current Active Decisions: {did}")

    if errors:
        print("program_decision_lifecycle: FAIL")
        for e in errors:
            print(" -", e)
        return 1

    print("program_decision_lifecycle: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
