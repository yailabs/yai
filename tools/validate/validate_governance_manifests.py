#!/usr/bin/env python3
import json
from pathlib import Path


def _must_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    root = Path(__file__).resolve().parents[2]
    mroot = root / "governance" / "manifests"
    required = [
        "governance.manifest.json",
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
            raise SystemExit(f"governance_manifests: missing {p.relative_to(root)}")

    governance_manifest = _must_json(mroot / "governance.manifest.json")
    runtime_entrypoints = _must_json(mroot / "runtime.entrypoints.json")
    publish_index = _must_json(mroot / "publish.index.json")
    publish_layers = _must_json(mroot / "publish.layers.json")

    if governance_manifest.get("resolution_entrypoints_ref") != "model/manifests/runtime.entrypoints.json":
        raise SystemExit("governance_manifests: governance.manifest.json invalid resolution_entrypoints_ref")
    if governance_manifest.get("compatibility") != "model/manifests/compatibility.matrix.json":
        raise SystemExit("governance_manifests: governance.manifest.json invalid compatibility ref")

    entries = runtime_entrypoints.get("entrypoints", [])
    if not entries:
        raise SystemExit("governance_manifests: runtime.entrypoints.json has no entrypoints")
    first = entries[0]
    for key in [
        "governance_manifest_ref",
        "customer_policy_pack_index_ref",
        "governance_attachability_constraints_ref",
        "resolution_order_ref",
        "compliance_resolution_order_ref",
        "compatibility_ref",
    ]:
        if not str(first.get(key, "")).startswith("model/manifests/"):
            raise SystemExit(f"governance_manifests: entrypoint missing manifests ref for {key}")

    runtime_target = None
    for target in publish_index.get("targets", []):
        if target.get("target") == "runtime-governance":
            runtime_target = target
            break
    if runtime_target is None:
        raise SystemExit("governance_manifests: publish.index.json missing runtime-governance target")
    if "manifests" not in runtime_target.get("includes", []):
        raise SystemExit("governance_manifests: runtime-governance target must include manifests")

    runtime_surface = publish_layers.get("runtime_surface", {})
    if runtime_surface.get("manifests") != "model/manifests/":
        raise SystemExit("governance_manifests: publish.layers.json runtime_surface.manifests mismatch")

    print("governance_manifests: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
