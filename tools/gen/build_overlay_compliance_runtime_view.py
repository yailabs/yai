#!/usr/bin/env python3
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GOV = ROOT / "governance"

compliance_idx = json.loads((GOV / "compliance" / "index" / "compliance.index.json").read_text())
compliance_desc_idx = json.loads((GOV / "compliance" / "index" / "compliance.descriptors.index.json").read_text())
overlay_desc_idx = json.loads((GOV / "overlays" / "index" / "overlays.descriptors.index.json").read_text())


def normalize_overlay_entry(entry: dict) -> dict:
    ov = dict(entry)
    oid = ov.get("overlay_id", "")
    klass = ov.get("overlay_class", "")
    if not ov.get("manifest_ref"):
        ov["manifest_ref"] = ov.get("materialized_manifest_ref", "")
    if not ov.get("materialized_bundle_ref") and oid and klass:
        if klass == "regulatory":
            ov["materialized_bundle_ref"] = f"overlays/materialized/regulatory/{oid}"
        elif klass == "sector":
            name = oid.split("sector.", 1)[1] if oid.startswith("sector.") else oid
            ov["materialized_bundle_ref"] = f"overlays/materialized/sector/{name}"
        elif klass == "contextual":
            name = oid.split("context.", 1)[1] if oid.startswith("context.") else oid
            ov["materialized_bundle_ref"] = f"overlays/materialized/contextual/{name}"
    return ov

reg_ovs = []
sec_ovs = []
ctx_ovs = []
for raw in overlay_desc_idx.get("entries", []):
    ov = normalize_overlay_entry(raw)
    klass = ov.get("overlay_class", "")
    if klass == "regulatory":
        reg_ovs.append(ov)
    elif klass == "sector":
        sec_ovs.append(ov)
    elif klass == "contextual":
        ctx_ovs.append(ov)

out = {
    "kind": "overlay-compliance.runtime.v1",
    "version": "v1",
    "sources": {
        "compliance_index": "compliance/index/compliance.index.json",
        "compliance_descriptors_index": "compliance/index/compliance.descriptors.index.json",
        "overlay_descriptors_index": "overlays/index/overlays.descriptors.index.json",
    },
    "overlays": {
        "regulatory": reg_ovs,
        "sector": sec_ovs,
        "contextual": ctx_ovs,
    },
    "compliance_entries": compliance_idx.get("entries", []),
    "compliance_descriptors": compliance_desc_idx.get("entries", []),
    "matrices": {
        "attachment": "overlays/matrices/overlay-attachment.matrix.v1.json",
        "precedence": "overlays/matrices/overlay-precedence.matrix.v1.json",
        "evidence": "overlays/matrices/overlay-evidence.matrix.v1.json",
    },
}

out_path = GOV / "overlays" / "index" / "overlay-compliance.runtime.v1.json"
out_path.write_text(json.dumps(out, indent=2) + "\n")
print(out_path)
