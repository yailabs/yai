#!/usr/bin/env python3
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CF = ROOT / "governance" / "families"
IDX = CF / "index"
DESCRIPTORS = CF / "descriptors"

families_index_path = IDX / "families.index.json"
hierarchy_path = IDX / "family-hierarchy.json"
naming_path = IDX / "family-naming-registry.json"

families_index = json.loads(families_index_path.read_text(encoding="utf-8"))
hierarchy = json.loads(hierarchy_path.read_text(encoding="utf-8"))
naming = json.loads(naming_path.read_text(encoding="utf-8"))

hier_by_name = {e.get("canonical_name"): e for e in hierarchy.get("entries", [])}
naming_by_name = {e.get("canonical_name"): e for e in naming.get("entries", [])}

DESCRIPTORS.mkdir(parents=True, exist_ok=True)

index_entries = []
matrix_entries = []

for f in families_index.get("families", []):
    canonical = f.get("canonical_name")
    if not canonical:
        continue
    manifest_path = CF / "materialized" / f"{canonical}.manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8")) if manifest_path.exists() else {}

    hierarchy_entry = hier_by_name.get(canonical, {})
    naming_entry = naming_by_name.get(canonical, {})

    descriptor_rel = f"families/descriptors/{canonical}.descriptor.v1.json"
    descriptor_path = ROOT / "governance" / "families" / "descriptors" / f"{canonical}.descriptor.v1.json"

    specializations = manifest.get("specializations") or hierarchy_entry.get("specializations") or []

    descriptor = {
        "kind": "control_family_descriptor.v1",
        "version": "v1",
        "family_id": f.get("family_id", manifest.get("family_id", "")),
        "canonical_name": canonical,
        "display_name": manifest.get("display_name") or naming_entry.get("display_name", canonical),
        "slug": manifest.get("slug") or naming_entry.get("slug", canonical),
        "internal_id": manifest.get("internal_id") or naming_entry.get("internal_id", f.get("family_id", "")),
        "legacy_label": naming_entry.get("legacy_label", ""),
        "status": f.get("status", manifest.get("status", "active")),
        "maturity": f.get("maturity", "seed"),
        "classification_ref": manifest.get("classification_ref", "classification/classification-map.json"),
        "specializations_ref": manifest.get("specializations_ref", "specializations/index/specializations.index.json"),
        "materialized_manifest_ref": f.get("materialized_manifest_ref") or f.get("manifest_ref") or f"families/materialized/{canonical}.manifest.json",
        "hierarchy": {
            "parent": hierarchy_entry.get("parent", ""),
            "specializations": specializations,
        },
        "resolution": {
            "default_specialization": specializations[0] if specializations else "general",
            "overlay_refs": manifest.get("overlay_refs", {}),
            "authority_profile": manifest.get("authority_profile", {}),
            "evidence_profile": manifest.get("evidence_profile", {}),
        },
        "applicability": {
            "domains": [canonical],
            "risk_focus": manifest.get("risk_focus", []),
            "lead_vertical": str(f.get("lead_vertical", "false")).lower() == "true",
        },
        "hooks": {
            "policy_composition": {
                "specializations": specializations,
                "legacy_seed_ref": manifest.get("legacy_seed_ref", ""),
            },
            "evidence": {
                "requires_policy_match_trace": bool(manifest.get("evidence_profile", {}).get("requires_policy_match_trace", False)),
                "retention_hint": manifest.get("evidence_profile", {}).get("retention_hint", "standard"),
            },
        },
        "source": {
            "materialized_manifest_ref": f"families/materialized/{canonical}.manifest.json",
            "generator": "tools/gen/build_control_family_descriptors.py",
        },
    }

    descriptor_path.write_text(json.dumps(descriptor, indent=2) + "\n", encoding="utf-8")

    index_entries.append(
        {
            "family_id": descriptor["family_id"],
            "canonical_name": canonical,
            "descriptor_ref": descriptor_rel,
            "materialized_manifest_ref": descriptor["materialized_manifest_ref"],
            "status": descriptor["status"],
            "maturity": descriptor["maturity"],
        }
    )

    matrix_entries.append(
        {
            "family_id": descriptor["family_id"],
            "canonical_name": canonical,
            "descriptor_ref": descriptor_rel,
            "default_specialization": descriptor["resolution"]["default_specialization"],
            "specialization_count": len(specializations),
            "lead_vertical": descriptor["applicability"]["lead_vertical"],
        }
    )

families_descriptors_index = {
    "kind": "families.descriptors.index",
    "version": "v1",
    "entries": index_entries,
}
(IDX / "families.descriptors.index.json").write_text(
    json.dumps(families_descriptors_index, indent=2) + "\n", encoding="utf-8"
)

family_matrix = {
    "kind": "family.matrix.v1",
    "version": "v1",
    "entries": matrix_entries,
}
(IDX / "family.matrix.v1.json").write_text(json.dumps(family_matrix, indent=2) + "\n", encoding="utf-8")

updated_families = []
for f in families_index.get("families", []):
    canonical = f.get("canonical_name")
    if not canonical:
        continue
    enriched = dict(f)
    enriched.setdefault("descriptor_ref", f"families/descriptors/{canonical}.descriptor.v1.json")
    enriched.setdefault("materialized_manifest_ref", enriched.get("manifest_ref", f"families/materialized/{canonical}.manifest.json"))
    enriched["manifest_ref"] = enriched["materialized_manifest_ref"]
    updated_families.append(enriched)

families_index["families"] = updated_families
families_index_path.write_text(json.dumps(families_index, indent=2) + "\n", encoding="utf-8")

print(IDX / "families.descriptors.index.json")
print(IDX / "family.matrix.v1.json")
print(DESCRIPTORS)
