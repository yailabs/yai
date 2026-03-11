#!/usr/bin/env python3
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GOV = ROOT / "governance"
view_path = GOV / "overlays" / "index" / "overlay-compliance.runtime.v1.json"
view = json.loads(view_path.read_text())

if view.get("kind") != "overlay-compliance.runtime.v1":
    raise SystemExit("invalid kind")
if view.get("version") != "v1":
    raise SystemExit("invalid version")

for rel in view.get("matrices", {}).values():
    if not (GOV / rel).exists():
        raise SystemExit(f"missing matrix: {rel}")

for cls in ("regulatory", "sector", "contextual"):
    for ov in view.get("overlays", {}).get(cls, []):
        ref = ov.get("manifest_ref")
        if not ref:
            raise SystemExit(f"overlay missing manifest_ref in {cls}")
        if not (GOV / ref).exists():
            raise SystemExit(f"overlay manifest not found: {ref}")

for m in view.get("compliance_entries", []):
    ref = m.get("materialized_manifest_ref") or m.get("manifest_ref")
    if not ref:
        raise SystemExit("compliance entry missing manifest_ref")
    if not (GOV / ref).exists():
        raise SystemExit(f"compliance manifest not found: {ref}")

for d in view.get("compliance_descriptors", []):
    dref = d.get("descriptor_ref")
    if not dref:
        raise SystemExit("compliance descriptor entry missing descriptor_ref")
    if not (GOV / dref).exists():
        raise SystemExit(f"compliance descriptor not found: {dref}")

known_overlay_ids = set()
for cls in ("regulatory", "sector"):
    for ov in view.get("overlays", {}).get(cls, []):
        oid = ov.get("overlay_id")
        if oid:
            known_overlay_ids.add(oid)

if not known_overlay_ids:
    raise SystemExit("no known overlay ids")

for matrix_key in ("precedence", "evidence"):
    rel = view["matrices"][matrix_key]
    txt = (GOV / rel).read_text()
    if not any(oid in txt for oid in known_overlay_ids):
        raise SystemExit(f"matrix appears detached from overlay ids: {rel}")

print("overlay_compliance_runtime_view: ok")
