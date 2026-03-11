#!/usr/bin/env python3
import json
from pathlib import Path


def _must_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    root = Path(__file__).resolve().parents[2]
    mroot = root / "governance" / "manifests"
    required = [
        "law.manifest.json",
        "runtime.entrypoints.json",
        "publish.index.json",
        "publish.layers.json",
        "compatibility.matrix.json",
        "domain-resolution-order.json",
        "compliance-resolution-order.json",
        "governance-attachability.constraints.v1.json",
        "customer-policy-packs.index.json",
        "enterprise-custom-governance.index.json",
    ]
    for rel in required:
        p = mroot / rel
        if not p.exists():
            raise SystemExit(f"governance_manifest_spine: missing {p.relative_to(root)}")

    law_manifest = _must_json(mroot / "law.manifest.json")
    runtime_entrypoints = _must_json(mroot / "runtime.entrypoints.json")
    publish_index = _must_json(mroot / "publish.index.json")
    publish_layers = _must_json(mroot / "publish.layers.json")

    if law_manifest.get("resolution_entrypoints_ref") != "manifests/runtime.entrypoints.json":
        raise SystemExit("governance_manifest_spine: law.manifest.json invalid resolution_entrypoints_ref")
    if law_manifest.get("compatibility") != "manifests/compatibility.matrix.json":
        raise SystemExit("governance_manifest_spine: law.manifest.json invalid compatibility ref")

    entries = runtime_entrypoints.get("entrypoints", [])
    if not entries:
        raise SystemExit("governance_manifest_spine: runtime.entrypoints.json has no entrypoints")
    first = entries[0]
    for key in [
        "law_manifest_ref",
        "customer_policy_pack_index_ref",
        "governance_attachability_constraints_ref",
        "resolution_order_ref",
        "compliance_resolution_order_ref",
        "compatibility_ref",
    ]:
        if not str(first.get(key, "")).startswith("manifests/"):
            raise SystemExit(f"governance_manifest_spine: entrypoint missing manifests ref for {key}")

    runtime_target = None
    for target in publish_index.get("targets", []):
        if target.get("target") == "runtime-embedded":
            runtime_target = target
            break
    if runtime_target is None:
        raise SystemExit("governance_manifest_spine: publish.index.json missing runtime-embedded target")
    if "manifests" not in runtime_target.get("includes", []):
        raise SystemExit("governance_manifest_spine: runtime-embedded target must include manifests")

    runtime_surface = publish_layers.get("runtime_surface", {})
    if runtime_surface.get("manifests") != "manifests/":
        raise SystemExit("governance_manifest_spine: publish.layers.json runtime_surface.manifests mismatch")

    print("governance_manifest_spine: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
