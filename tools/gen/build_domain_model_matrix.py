#!/usr/bin/env python3
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GOV = ROOT / "governance"

classification_map = json.loads((GOV / "classification" / "classification-map.json").read_text())
domains_index = json.loads((GOV / "domains" / "index" / "domains.index.json").read_text())
families_index = json.loads((GOV / "families" / "index" / "families.descriptors.index.json").read_text())
spec_index = json.loads((GOV / "specializations" / "index" / "specializations.index.json").read_text())

family_to_domain = {}
for d in domains_index.get("domains", []):
    fam = d.get("family_canonical_name")
    did = d.get("domain_id")
    if fam and did:
        family_to_domain[fam] = did

family_manifest = {}
family_descriptor = {}
for f in families_index.get("entries", []):
    fam = f.get("canonical_name")
    if not fam:
        continue
    dref = f.get("descriptor_ref")
    mref = f.get("materialized_manifest_ref") or f.get("manifest_ref")
    if dref:
        family_descriptor[fam] = dref
        try:
            dobj = json.loads((ROOT / "governance" / dref).read_text())
            mref = dobj.get("materialized_manifest_ref", mref)
        except Exception:
            pass
    if mref:
        family_manifest[fam] = mref

specs_by_family = {}
for s in spec_index.get("specializations", []):
    fam = s.get("family")
    sid = s.get("specialization_id")
    ref = s.get("manifest_ref")
    if not fam or not sid or not ref:
        continue
    specs_by_family.setdefault(fam, []).append({"id": sid, "manifest_ref": ref})

runtime_families = [m.get("family") for m in classification_map.get("maps", []) if m.get("family")]
runtime_families = [f for f in runtime_families if f in family_to_domain]

default_pref = {
    "digital": "network-egress",
    "economic": "payments",
    "scientific": "parameter-governance",
}

entries = []
family_resolution = []

for fam in runtime_families:
    did = family_to_domain[fam]
    fmanifest = family_manifest.get(fam, f"families/materialized/{fam}.manifest.json")
    specs = specs_by_family.get(fam, [])
    default_spec = default_pref.get(fam)
    if default_spec not in {s["id"] for s in specs}:
        default_spec = specs[0]["id"] if specs else "general"

    family_resolution.append(
        {
            "family": fam,
            "domain_id": did,
            "default_specialization": default_spec,
            "family_descriptor_ref": family_descriptor.get(fam, ""),
            "manifest_ref": fmanifest,
            "specialization_candidates": [s["id"] for s in specs],
        }
    )

    entries.append(
        {
            "lookup_id": fam,
            "kind": "family",
            "family": fam,
            "domain_id": did,
            "family_descriptor_ref": family_descriptor.get(fam, ""),
            "manifest_ref": fmanifest,
            "default_specialization": default_spec,
        }
    )
    entries.append(
        {
            "lookup_id": did,
            "kind": "domain",
            "family": fam,
            "domain_id": did,
            "family_descriptor_ref": family_descriptor.get(fam, ""),
            "manifest_ref": fmanifest,
            "default_specialization": default_spec,
        }
    )

for fam in runtime_families:
    did = family_to_domain[fam]
    for s in specs_by_family.get(fam, []):
        entries.append(
            {
                "lookup_id": s["id"],
                "kind": "specialization",
                "family": fam,
                "domain_id": did,
                "manifest_ref": s["manifest_ref"],
            }
        )

out = {
    "kind": "domain-model.matrix.v1",
    "version": "v1",
    "model": "schema-first,index-driven",
    "runtime_families": runtime_families,
    "family_resolution": family_resolution,
    "entries": entries,
}

out_path = GOV / "domains" / "index" / "domain-model.matrix.v1.json"
out_path.write_text(json.dumps(out, indent=2) + "\n")
print(out_path)
