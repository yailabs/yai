#!/usr/bin/env python3
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SPEC = ROOT / "governance" / "specializations"
IDX = SPEC / "index"
DESCRIPTORS = SPEC / "descriptors"
MATERIALIZED = SPEC / "materialized"

SPECIALIZATIONS_INDEX = IDX / "specializations.index.json"
TAXONOMY = IDX / "specialization-taxonomy.json"
SCENARIOS = IDX / "scenario-classes.json"

DESCRIPTORS.mkdir(parents=True, exist_ok=True)

spec_index = json.loads(SPECIALIZATIONS_INDEX.read_text(encoding="utf-8"))
taxonomy = json.loads(TAXONOMY.read_text(encoding="utf-8"))
scenarios = json.loads(SCENARIOS.read_text(encoding="utf-8"))

tax_by_id = {e.get("specialization_id"): e for e in taxonomy.get("entries", [])}
scenario_map = {}
for e in scenarios.get("classes", []):
    sp = e.get("specialization", "")
    sid = sp.split(".", 1)[1] if "." in sp else sp
    if sid:
        scenario_map.setdefault(sid, []).append(e.get("scenario_class", ""))

index_entries = []
matrix_entries = []
updated_specs = []

for e in spec_index.get("specializations", []):
    sid = e.get("specialization_id", "")
    if not sid:
        continue
    family = e.get("family", "")
    canonical = e.get("canonical_name") or (f"{family}.{sid}" if family else sid)
    status = e.get("status", "active")
    wave = e.get("verticalization_wave", "")

    bundle_ref = f"specializations/materialized/{sid}"
    manifest_ref = f"{bundle_ref}/manifest.json"
    bundle_dir = ROOT / "governance" / "specializations" / "materialized" / sid
    manifest = {}
    mp = bundle_dir / "manifest.json"
    if mp.exists():
        manifest = json.loads(mp.read_text(encoding="utf-8"))

    def refs(subdir: str):
        d = bundle_dir / subdir
        if not d.exists() or not d.is_dir():
            return []
        out = []
        for p in sorted(d.glob("*.json")):
            out.append(f"{bundle_ref}/{subdir}/{p.name}")
        return out

    descriptor_ref = f"specializations/descriptors/{sid}.descriptor.v1.json"
    descriptor = {
        "kind": "specialization_descriptor.v1",
        "version": "v1",
        "specialization_id": sid,
        "canonical_name": canonical,
        "family": family,
        "specialization_class": tax_by_id.get(sid, {}).get("class", "general"),
        "status": status,
        "review_state": manifest.get("review_state", "managed"),
        "verticalization_wave": wave,
        "runtime": {
            "manifest_ref": manifest_ref,
            "bundle_ref": bundle_ref,
            "scenario_classes": [c for c in scenario_map.get(sid, []) if c],
        },
        "composition": {
            "model_refs": refs("model"),
            "policy_refs": refs("policy"),
            "evidence_refs": refs("evidence"),
            "authority_refs": refs("authority"),
            "discovery_refs": refs("discovery"),
            "scenario_refs": refs("scenarios"),
        },
        "taxonomy": {
            "risk_ref": manifest.get("risk_ref", "classification/risk.taxonomy.json"),
            "action_ref": manifest.get("action_ref", "classification/action.taxonomy.json"),
            "asset_ref": manifest.get("asset_ref", "classification/asset.taxonomy.json"),
            "signal_ref": manifest.get("signal_ref", "classification/signal.taxonomy.json"),
        },
        "source": {
            "generator": "tools/gen/build_specialization_descriptors.py",
            "materialized_bundle_ref": bundle_ref,
        },
    }

    (DESCRIPTORS / f"{sid}.descriptor.v1.json").write_text(
        json.dumps(descriptor, indent=2) + "\n", encoding="utf-8"
    )

    index_entries.append(
        {
            "specialization_id": sid,
            "canonical_name": canonical,
            "family": family,
            "descriptor_ref": descriptor_ref,
            "materialized_manifest_ref": manifest_ref,
            "materialized_bundle_ref": bundle_ref,
            "status": status,
            "verticalization_wave": wave,
        }
    )
    matrix_entries.append(
        {
            "specialization_id": sid,
            "family": family,
            "descriptor_ref": descriptor_ref,
            "policy_count": len(descriptor["composition"]["policy_refs"]),
            "evidence_count": len(descriptor["composition"]["evidence_refs"]),
            "authority_count": len(descriptor["composition"]["authority_refs"]),
            "discovery_count": len(descriptor["composition"]["discovery_refs"]),
            "scenario_class_count": len(descriptor["runtime"]["scenario_classes"]),
        }
    )

    up = dict(e)
    up["canonical_name"] = canonical
    up["manifest_ref"] = manifest_ref
    up["descriptor_ref"] = descriptor_ref
    up["materialized_manifest_ref"] = manifest_ref
    up["materialized_bundle_ref"] = bundle_ref
    updated_specs.append(up)

spec_index["specializations"] = updated_specs
SPECIALIZATIONS_INDEX.write_text(json.dumps(spec_index, indent=2) + "\n", encoding="utf-8")

for e in taxonomy.get("entries", []):
    sid = e.get("specialization_id")
    if sid:
        e["manifest_ref"] = f"specializations/materialized/{sid}/manifest.json"
        e["descriptor_ref"] = f"specializations/descriptors/{sid}.descriptor.v1.json"
TAXONOMY.write_text(json.dumps(taxonomy, indent=2) + "\n", encoding="utf-8")

spec_desc_index = {
    "kind": "specializations.descriptors.index",
    "version": "v1",
    "entries": index_entries,
}
(IDX / "specializations.descriptors.index.json").write_text(
    json.dumps(spec_desc_index, indent=2) + "\n", encoding="utf-8"
)

spec_matrix = {
    "kind": "specialization.matrix.v1",
    "version": "v1",
    "entries": matrix_entries,
}
(IDX / "specialization.matrix.v1.json").write_text(
    json.dumps(spec_matrix, indent=2) + "\n", encoding="utf-8"
)

print(IDX / "specializations.descriptors.index.json")
print(IDX / "specialization.matrix.v1.json")
print(DESCRIPTORS)
