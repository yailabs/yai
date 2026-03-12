#!/usr/bin/env python3
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OV = ROOT / "governance" / "overlays"
IDX = OV / "index"

idx = json.loads((IDX / "overlays.descriptors.index.json").read_text(encoding="utf-8"))
mtx = json.loads((IDX / "overlays.matrix.v1.json").read_text(encoding="utf-8"))

if idx.get("kind") != "overlays.descriptors.index":
    raise SystemExit("overlays.descriptors.index invalid kind")
if mtx.get("kind") != "overlays.matrix.v1":
    raise SystemExit("overlays.matrix invalid kind")

m = {e.get("overlay_id"): e for e in mtx.get("entries", [])}

for e in idx.get("entries", []):
    oid = e.get("overlay_id")
    dref = e.get("descriptor_ref")
    mref = e.get("materialized_manifest_ref")
    bref = e.get("materialized_bundle_ref")
    if not oid or not dref or not mref or not bref:
        raise SystemExit(f"overlay descriptor index missing fields: {e}")

    dp = ROOT / "governance" / dref
    if not dp.exists():
        raise SystemExit(f"missing overlay descriptor: {dref}")
    dobj = json.loads(dp.read_text(encoding="utf-8"))
    if dobj.get("kind") != "overlay_descriptor.v1":
        raise SystemExit(f"invalid overlay descriptor kind: {oid}")

    mp = ROOT / "governance" / mref
    if not mp.exists():
        raise SystemExit(f"missing overlay manifest: {mref}")
    bp = ROOT / "governance" / bref
    if not bp.exists() or not bp.is_dir():
        raise SystemExit(f"missing overlay bundle dir: {bref}")

    if oid not in m:
        raise SystemExit(f"overlays.matrix missing {oid}")

for rel in [
    "matrices/overlay-attachment.matrix.v1.json",
    "matrices/overlay-evidence.matrix.v1.json",
    "matrices/overlay-precedence.matrix.v1.json",
]:
    if not (OV / rel).exists():
        raise SystemExit(f"missing canonical overlay matrix: policy/overlays/{rel}")

print("overlay_descriptors: ok")
