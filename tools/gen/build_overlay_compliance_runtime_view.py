#!/usr/bin/env python3
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GOV = ROOT / "governance"

reg_idx = json.loads((GOV / "overlays" / "regulatory" / "index" / "regulatory.index.json").read_text())
sec_idx = json.loads((GOV / "overlays" / "sector" / "index" / "sector.index.json").read_text())
ctx_idx = json.loads((GOV / "overlays" / "contextual" / "index" / "contextual.index.json").read_text())
compliance_idx = json.loads((GOV / "compliance" / "index" / "compliance.index.json").read_text())


def normalize_overlay_entry(kind: str, entry: dict) -> dict:
    ov = dict(entry)
    oid = ov.get("overlay_id", "")
    if not ov.get("manifest_ref") and oid:
        if kind == "regulatory":
            ov["manifest_ref"] = f"overlays/regulatory/{oid}/manifest.json"
        elif kind == "sector":
            name = oid.split("sector.", 1)[1] if oid.startswith("sector.") else oid
            ov["manifest_ref"] = f"overlays/sector/{name}/manifest.json"
    return ov

reg_ovs = [normalize_overlay_entry("regulatory", x) for x in reg_idx.get("overlays", [])]
sec_ovs = [normalize_overlay_entry("sector", x) for x in sec_idx.get("overlays", [])]
ctx_ovs = [normalize_overlay_entry("contextual", x) for x in ctx_idx.get("overlays", [])]

out = {
    "kind": "overlay-compliance.runtime.v1",
    "version": "v1",
    "sources": {
        "compliance_index": "compliance/index/compliance.index.json",
        "regulatory_index": "overlays/regulatory/index/regulatory.index.json",
        "sector_index": "overlays/sector/index/sector.index.json",
        "contextual_index": "overlays/contextual/index/contextual.index.json",
    },
    "overlays": {
        "regulatory": reg_ovs,
        "sector": sec_ovs,
        "contextual": ctx_ovs,
    },
    "compliance_entries": compliance_idx.get("entries", []),
    "matrices": {
        "regulatory_attachment": "overlays/regulatory/index/overlay-attachment-matrix.json",
        "sector_attachment": "overlays/sector/index/overlay-attachment-matrix.json",
        "regulatory_precedence": "overlays/regulatory/index/overlay-precedence-matrix.json",
        "sector_precedence": "overlays/sector/index/overlay-precedence-matrix.json",
        "regulatory_evidence": "overlays/regulatory/index/overlay-evidence-matrix.json",
        "sector_evidence": "overlays/sector/index/overlay-evidence-matrix.json",
    },
}

out_path = GOV / "overlays" / "index" / "overlay-compliance.runtime.v1.json"
out_path.write_text(json.dumps(out, indent=2) + "\n")
print(out_path)
