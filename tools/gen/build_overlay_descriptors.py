#!/usr/bin/env python3
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OV = ROOT / "governance" / "overlays"
IDX = OV / "index"
DESCRIPTORS = OV / "descriptors"
MATERIALIZED = OV / "materialized"
MATRICES = OV / "matrices"

DESCRIPTORS.mkdir(parents=True, exist_ok=True)
MATRICES.mkdir(parents=True, exist_ok=True)

reg_idx = json.loads((OV / "regulatory" / "index" / "regulatory.index.json").read_text(encoding="utf-8"))
sec_idx = json.loads((OV / "sector" / "index" / "sector.index.json").read_text(encoding="utf-8"))
ctx_idx = json.loads((OV / "contextual" / "index" / "contextual.index.json").read_text(encoding="utf-8"))

reg_attach = json.loads((OV / "regulatory" / "index" / "overlay-attachment-matrix.json").read_text(encoding="utf-8"))
reg_prec = json.loads((OV / "regulatory" / "index" / "overlay-precedence-matrix.json").read_text(encoding="utf-8"))
reg_ev = json.loads((OV / "regulatory" / "index" / "overlay-evidence-matrix.json").read_text(encoding="utf-8"))
sec_attach = json.loads((OV / "sector" / "index" / "overlay-attachment-matrix.json").read_text(encoding="utf-8"))
sec_prec = json.loads((OV / "sector" / "index" / "overlay-precedence-matrix.json").read_text(encoding="utf-8"))
sec_ev = json.loads((OV / "sector" / "index" / "overlay-evidence-matrix.json").read_text(encoding="utf-8"))

all_attach = []
all_attach.extend(reg_attach.get("entries", []))
all_attach.extend(sec_attach.get("entries", []))

all_prec = []
all_prec.extend(reg_prec.get("precedence", []))
all_prec.extend(sec_prec.get("precedence", []))

all_ev = []
all_ev.extend(reg_ev.get("entries", []))
all_ev.extend(sec_ev.get("entries", []))

# contextual derived matrices
ctx_attach = []
ctx_ev = []
ctx_prec = []
for o in ctx_idx.get("overlays", []):
    oid = o.get("overlay_id", "")
    if not oid:
        continue
    short = oid.split("context.", 1)[1] if oid.startswith("context.") else oid
    ctx_attach.append({
        "overlay": oid,
        "family": "*",
        "specialization": "*",
        "action": "*",
        "asset": "*",
        "risk": "*",
        "scope": short,
    })
    ctx_ev.append({"overlay": oid, "required": ["context_scope", "review_trace"]})
    ctx_prec.append({
        "stack": f"contextual.{short}",
        "order": ["deny", "review_required", "allow"],
        "authority_raise": short in {"organization", "workspace"},
    })

all_attach.extend(ctx_attach)
all_ev.extend(ctx_ev)
all_prec.extend(ctx_prec)

(MATRICES / "overlay-attachment.matrix.v1.json").write_text(
    json.dumps({"kind": "overlay_attachment_matrix.v1", "version": "v1", "entries": all_attach}, indent=2) + "\n",
    encoding="utf-8",
)
(MATRICES / "overlay-evidence.matrix.v1.json").write_text(
    json.dumps({"kind": "overlay_evidence_matrix.v1", "version": "v1", "entries": all_ev}, indent=2) + "\n",
    encoding="utf-8",
)
(MATRICES / "overlay-precedence.matrix.v1.json").write_text(
    json.dumps({"kind": "overlay_precedence_matrix.v1", "version": "v1", "precedence": all_prec}, indent=2) + "\n",
    encoding="utf-8",
)


def list_json_refs(base: Path, rel_prefix: str):
    out = []
    if not base.exists() or not base.is_dir():
        return out
    for sub in ("policy", "mappings", "retention", "taxonomy", "docs"):
        d = base / sub
        if d.exists() and d.is_dir():
            for p in sorted(d.glob("*.json")):
                out.append(f"{rel_prefix}/{sub}/{p.name}")
    return out


def build_entry(overlay_class: str, overlay_id: str, status: str, manifest_ref: str):
    if overlay_class == "regulatory":
        name = overlay_id
        b = MATERIALIZED / "regulatory" / name
        bundle_ref = f"overlays/materialized/regulatory/{name}"
    elif overlay_class == "sector":
        name = overlay_id.split("sector.", 1)[1] if overlay_id.startswith("sector.") else overlay_id
        b = MATERIALIZED / "sector" / name
        bundle_ref = f"overlays/materialized/sector/{name}"
    else:
        name = overlay_id.split("context.", 1)[1] if overlay_id.startswith("context.") else overlay_id
        b = MATERIALIZED / "contextual" / name
        bundle_ref = f"overlays/materialized/contextual/{name}"

    mref = f"{bundle_ref}/manifest.json"
    dref = f"overlays/descriptors/{overlay_id}.descriptor.v1.json"

    att = [x for x in all_attach if x.get("overlay") == overlay_id]
    ev = [x for x in all_ev if x.get("overlay") == overlay_id]
    pr = [x for x in all_prec if overlay_id in x.get("stack", "")]

    descriptor = {
        "kind": "overlay_descriptor.v1",
        "version": "v1",
        "overlay_id": overlay_id,
        "overlay_class": overlay_class,
        "status": status or "active",
        "runtime": {
            "manifest_ref": mref,
            "materialized_bundle_ref": bundle_ref,
            "attachment_count": len(att),
            "precedence_count": len(pr),
            "evidence_count": len(ev),
        },
        "attachment_rules": att,
        "precedence_rules": pr,
        "evidence_rules": ev,
        "composition": {
            "block_refs": list_json_refs(b, bundle_ref),
        },
        "source": {
            "generator": "tools/gen/build_overlay_descriptors.py",
            "legacy_manifest_ref": manifest_ref,
        },
    }

    (DESCRIPTORS / f"{overlay_id}.descriptor.v1.json").write_text(json.dumps(descriptor, indent=2) + "\n", encoding="utf-8")

    return {
        "overlay_id": overlay_id,
        "overlay_class": overlay_class,
        "status": descriptor["status"],
        "descriptor_ref": dref,
        "materialized_manifest_ref": mref,
        "materialized_bundle_ref": bundle_ref,
    }


desc_entries = []

updated_reg = []
for e in reg_idx.get("overlays", []):
    oid = e.get("overlay_id", "")
    if not oid:
        continue
    mr = e.get("manifest_ref") or f"overlays/materialized/regulatory/{oid}/manifest.json"
    de = build_entry("regulatory", oid, e.get("status", "active"), mr)
    desc_entries.append(de)
    ne = dict(e)
    ne["manifest_ref"] = de["materialized_manifest_ref"]
    ne["descriptor_ref"] = de["descriptor_ref"]
    ne["materialized_bundle_ref"] = de["materialized_bundle_ref"]
    updated_reg.append(ne)
reg_idx["overlays"] = updated_reg

updated_sec = []
for e in sec_idx.get("overlays", []):
    oid = e.get("overlay_id", "")
    if not oid:
        continue
    name = oid.split("sector.", 1)[1] if oid.startswith("sector.") else oid
    mr = e.get("manifest_ref") or f"overlays/materialized/sector/{name}/manifest.json"
    de = build_entry("sector", oid, e.get("status", "active"), mr)
    desc_entries.append(de)
    ne = dict(e)
    ne["manifest_ref"] = de["materialized_manifest_ref"]
    ne["descriptor_ref"] = de["descriptor_ref"]
    ne["materialized_bundle_ref"] = de["materialized_bundle_ref"]
    updated_sec.append(ne)
sec_idx["overlays"] = updated_sec

updated_ctx = []
for e in ctx_idx.get("overlays", []):
    oid = e.get("overlay_id", "")
    if not oid:
        continue
    name = oid.split("context.", 1)[1] if oid.startswith("context.") else oid
    mr = e.get("manifest_ref") or f"overlays/materialized/contextual/{name}/manifest.json"
    de = build_entry("contextual", oid, e.get("status", "active"), mr)
    desc_entries.append(de)
    ne = dict(e)
    ne["manifest_ref"] = de["materialized_manifest_ref"]
    ne["descriptor_ref"] = de["descriptor_ref"]
    ne["materialized_bundle_ref"] = de["materialized_bundle_ref"]
    updated_ctx.append(ne)
ctx_idx["overlays"] = updated_ctx

(OV / "regulatory" / "index" / "regulatory.index.json").write_text(json.dumps(reg_idx, indent=2) + "\n", encoding="utf-8")
(OV / "sector" / "index" / "sector.index.json").write_text(json.dumps(sec_idx, indent=2) + "\n", encoding="utf-8")
(OV / "contextual" / "index" / "contextual.index.json").write_text(json.dumps(ctx_idx, indent=2) + "\n", encoding="utf-8")

(IDX / "overlays.descriptors.index.json").write_text(
    json.dumps({"kind": "overlays.descriptors.index", "version": "v1", "entries": desc_entries}, indent=2) + "\n",
    encoding="utf-8",
)
(IDX / "overlays.matrix.v1.json").write_text(
    json.dumps(
        {
            "kind": "overlays.matrix.v1",
            "version": "v1",
            "entries": [
                {
                    "overlay_id": e.get("overlay_id"),
                    "overlay_class": e.get("overlay_class"),
                    "descriptor_ref": e.get("descriptor_ref"),
                }
                for e in desc_entries
            ],
        },
        indent=2,
    )
    + "\n",
    encoding="utf-8",
)

print(IDX / "overlays.descriptors.index.json")
print(IDX / "overlays.matrix.v1.json")
print(DESCRIPTORS)
