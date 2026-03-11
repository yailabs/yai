#!/usr/bin/env python3
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
COMP = ROOT / "governance" / "compliance"
IDX = COMP / "index"

idx = json.loads((IDX / "compliance.index.json").read_text(encoding="utf-8"))
didx = json.loads((IDX / "compliance.descriptors.index.json").read_text(encoding="utf-8"))
mtx = json.loads((IDX / "compliance.matrix.v1.json").read_text(encoding="utf-8"))

if didx.get("kind") != "compliance.descriptors.index":
    raise SystemExit("compliance.descriptors.index invalid kind")
if mtx.get("kind") != "compliance.matrix.v1":
    raise SystemExit("compliance.matrix invalid kind")

m = {e.get("compliance_id"): e for e in mtx.get("entries", [])}
for e in didx.get("entries", []):
    cid = e.get("compliance_id")
    if not cid:
        raise SystemExit("compliance descriptor index missing compliance_id")
    dref = e.get("descriptor_ref")
    mref = e.get("materialized_manifest_ref")
    bref = e.get("materialized_bundle_ref")
    if not dref or not mref or not bref:
        raise SystemExit(f"compliance descriptor index missing refs for {cid}")

    dp = ROOT / "governance" / dref
    if not dp.exists():
        raise SystemExit(f"missing compliance descriptor: {dref}")
    dobj = json.loads(dp.read_text(encoding="utf-8"))
    if dobj.get("kind") != "compliance_descriptor.v1":
        raise SystemExit(f"invalid compliance descriptor kind: {cid}")

    mp = ROOT / "governance" / mref
    if not mp.exists():
        raise SystemExit(f"missing compliance materialized manifest: {mref}")
    bp = ROOT / "governance" / bref
    if not bp.exists() or not bp.is_dir():
        raise SystemExit(f"missing compliance materialized bundle: {bref}")

    if cid not in m:
        raise SystemExit(f"compliance matrix missing {cid}")

for e in idx.get("entries", []):
    cid = e.get("compliance_id")
    if not cid:
        raise SystemExit("compliance.index missing compliance_id")
    if not e.get("descriptor_ref"):
        raise SystemExit(f"compliance.index missing descriptor_ref for {cid}")
    if not e.get("materialized_manifest_ref"):
        raise SystemExit(f"compliance.index missing materialized_manifest_ref for {cid}")

print("compliance_descriptors: ok")
