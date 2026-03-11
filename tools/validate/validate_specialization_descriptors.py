#!/usr/bin/env python3
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SPEC = ROOT / "governance" / "specializations"
IDX = SPEC / "index"

spec_idx = json.loads((IDX / "specializations.index.json").read_text(encoding="utf-8"))
desc_idx = json.loads((IDX / "specializations.descriptors.index.json").read_text(encoding="utf-8"))
matrix = json.loads((IDX / "specialization.matrix.v1.json").read_text(encoding="utf-8"))
taxonomy = json.loads((IDX / "specialization-taxonomy.json").read_text(encoding="utf-8"))

if desc_idx.get("kind") != "specializations.descriptors.index":
    raise SystemExit("specializations.descriptors.index invalid kind")
if matrix.get("kind") != "specialization.matrix.v1":
    raise SystemExit("specialization.matrix invalid kind")

matrix_map = {e.get("specialization_id"): e for e in matrix.get("entries", [])}
tax_map = {e.get("specialization_id"): e for e in taxonomy.get("entries", [])}

for e in desc_idx.get("entries", []):
    sid = e.get("specialization_id")
    dref = e.get("descriptor_ref")
    mref = e.get("materialized_manifest_ref")
    bref = e.get("materialized_bundle_ref")
    if not sid or not dref or not mref or not bref:
        raise SystemExit(f"descriptor index entry missing fields: {e}")

    dp = ROOT / "governance" / dref
    if not dp.exists():
        raise SystemExit(f"missing descriptor file: {dref}")
    dobj = json.loads(dp.read_text(encoding="utf-8"))
    if dobj.get("kind") != "specialization_descriptor.v1":
        raise SystemExit(f"invalid descriptor kind: {sid}")
    if dobj.get("specialization_id") != sid:
        raise SystemExit(f"descriptor specialization mismatch: {sid}")
    if dobj.get("runtime", {}).get("manifest_ref") != mref:
        raise SystemExit(f"descriptor manifest mismatch: {sid}")

    mp = ROOT / "governance" / mref
    if not mp.exists():
        raise SystemExit(f"missing materialized manifest: {mref}")

    bp = ROOT / "governance" / bref
    if not bp.exists() or not bp.is_dir():
        raise SystemExit(f"missing materialized bundle dir: {bref}")

    if sid not in matrix_map:
        raise SystemExit(f"specialization.matrix missing: {sid}")

    te = tax_map.get(sid, {})
    if te and te.get("descriptor_ref") != dref:
        raise SystemExit(f"taxonomy descriptor_ref mismatch: {sid}")

for e in spec_idx.get("specializations", []):
    sid = e.get("specialization_id")
    if not sid:
        raise SystemExit("specializations.index missing specialization_id")
    if not e.get("descriptor_ref"):
        raise SystemExit(f"specializations.index missing descriptor_ref: {sid}")
    if not e.get("materialized_manifest_ref"):
        raise SystemExit(f"specializations.index missing materialized_manifest_ref: {sid}")

print("specialization_descriptors: ok")
