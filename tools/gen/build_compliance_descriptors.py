#!/usr/bin/env python3
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
COMP = ROOT / "governance" / "compliance"
IDX = COMP / "index"
DESCRIPTORS = COMP / "descriptors"
MATERIALIZED = COMP / "materialized"

DESCRIPTORS.mkdir(parents=True, exist_ok=True)

comp_index = json.loads((IDX / "compliance.index.json").read_text(encoding="utf-8"))
app = json.loads((IDX / "applicability.matrix.json").read_text(encoding="utf-8"))
dcx = json.loads((IDX / "domain-crosswalk.json").read_text(encoding="utf-8"))
acx = json.loads((IDX / "authority-crosswalk.json").read_text(encoding="utf-8"))

app_map = {e.get("compliance_id"): e for e in app.get("rules", [])}
dcx_map = {e.get("compliance_id"): e for e in dcx.get("entries", [])}
acx_map = {e.get("compliance_id"): e for e in acx.get("entries", [])}

entries = []
matrix_entries = []
updated = []

for e in comp_index.get("entries", []):
    cid = e.get("compliance_id", "")
    if not cid:
        continue

    mat_dir = MATERIALIZED / cid
    if not mat_dir.exists():
        # Legacy sector IDs: sector.finance -> materialized/sector.finance
        mat_dir = MATERIALIZED / cid.replace("/", ".")
    manifest_ref = f"policy/compliance/materialized/{mat_dir.name}/manifest.json"
    descriptor_ref = f"policy/compliance/descriptors/{cid}.descriptor.v1.json"

    def refs(subdir: str):
        d = mat_dir / subdir
        if not d.exists() or not d.is_dir():
            return []
        return [f"policy/compliance/materialized/{mat_dir.name}/{subdir}/{p.name}" for p in sorted(d.glob("*.json"))]

    descriptor = {
        "kind": "compliance_descriptor.v1",
        "version": "v1",
        "compliance_id": cid,
        "canonical_name": e.get("canonical_name", cid),
        "display_name": e.get("display_name", cid),
        "regime_class": e.get("layer_type", "compliance"),
        "status": e.get("status", "active"),
        "runtime": {
            "manifest_ref": manifest_ref,
            "materialized_bundle_ref": f"policy/compliance/materialized/{mat_dir.name}",
            "attachment_levels": e.get("attachment_levels", []),
        },
        "applicability": app_map.get(cid, {}),
        "authority": acx_map.get(cid, {}),
        "domain_crosswalk": dcx_map.get(cid, {}),
        "composition": {
            "policy_refs": refs("policy"),
            "mapping_refs": refs("mappings"),
            "taxonomy_refs": refs("taxonomy"),
            "retention_refs": refs("retention"),
            "overlay_refs": refs("overlays"),
        },
        "profiles": {
            "effect_profile": e.get("effect_profile", []),
            "evidence_profile": e.get("evidence_profile", ""),
            "authority_interaction": e.get("authority_interaction", ""),
            "sector_scope": e.get("sector_scope", []),
            "primary_targets": e.get("primary_targets", []),
        },
        "source": {
            "generator": "tools/gen/build_compliance_descriptors.py",
            "legacy_manifest_ref": e.get("manifest_ref", ""),
        },
    }

    (DESCRIPTORS / f"{cid}.descriptor.v1.json").write_text(json.dumps(descriptor, indent=2) + "\n", encoding="utf-8")

    entries.append(
        {
            "compliance_id": cid,
            "canonical_name": descriptor["canonical_name"],
            "descriptor_ref": descriptor_ref,
            "materialized_manifest_ref": manifest_ref,
            "materialized_bundle_ref": f"policy/compliance/materialized/{mat_dir.name}",
            "status": descriptor["status"],
            "regime_class": descriptor["regime_class"],
        }
    )
    matrix_entries.append(
        {
            "compliance_id": cid,
            "descriptor_ref": descriptor_ref,
            "attachment_level_count": len(descriptor["runtime"].get("attachment_levels", [])),
            "policy_ref_count": len(descriptor["composition"].get("policy_refs", [])),
            "mapping_ref_count": len(descriptor["composition"].get("mapping_refs", [])),
            "taxonomy_ref_count": len(descriptor["composition"].get("taxonomy_refs", [])),
        }
    )

    ue = dict(e)
    ue["manifest_ref"] = manifest_ref
    ue["descriptor_ref"] = descriptor_ref
    ue["materialized_manifest_ref"] = manifest_ref
    ue["materialized_bundle_ref"] = f"policy/compliance/materialized/{mat_dir.name}"
    updated.append(ue)

comp_index["entries"] = updated
(IDX / "compliance.index.json").write_text(json.dumps(comp_index, indent=2) + "\n", encoding="utf-8")

comp_desc_idx = {"kind": "compliance.descriptors.index", "version": "v1", "entries": entries}
(IDX / "compliance.descriptors.index.json").write_text(json.dumps(comp_desc_idx, indent=2) + "\n", encoding="utf-8")

comp_matrix = {"kind": "compliance.matrix.v1", "version": "v1", "entries": matrix_entries}
(IDX / "compliance.matrix.v1.json").write_text(json.dumps(comp_matrix, indent=2) + "\n", encoding="utf-8")

print(IDX / "compliance.descriptors.index.json")
print(IDX / "compliance.matrix.v1.json")
print(DESCRIPTORS)
