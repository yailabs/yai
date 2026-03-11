#!/usr/bin/env python3
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
path = ROOT / "governance" / "domains" / "index" / "domain-model.matrix.v1.json"

obj = json.loads(path.read_text())

if obj.get("kind") != "domain-model.matrix.v1":
    raise SystemExit("invalid kind")
if obj.get("version") != "v1":
    raise SystemExit("invalid version")

runtime_families = obj.get("runtime_families", [])
if not runtime_families:
    raise SystemExit("runtime_families missing")

entries = obj.get("entries", [])
if not entries:
    raise SystemExit("entries missing")

ids = set()
for e in entries:
    lid = e.get("lookup_id")
    if not lid:
        raise SystemExit("entry lookup_id missing")
    if lid in ids:
        raise SystemExit(f"duplicate lookup_id: {lid}")
    ids.add(lid)
    if not e.get("manifest_ref"):
        raise SystemExit(f"entry manifest_ref missing: {lid}")

fam_resolution = {fr.get("family"): fr for fr in obj.get("family_resolution", [])}
for fam in runtime_families:
    fr = fam_resolution.get(fam)
    if not fr:
        raise SystemExit(f"missing family_resolution for {fam}")
    default_spec = fr.get("default_specialization")
    cands = set(fr.get("specialization_candidates", []))
    if default_spec and default_spec != "general" and default_spec not in cands:
        raise SystemExit(f"default specialization not in candidates for {fam}")

print("domain_model_matrix: ok")
