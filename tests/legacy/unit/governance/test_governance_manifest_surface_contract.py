#!/usr/bin/env python3
import json
from pathlib import Path


def main() -> int:
    root = Path(__file__).resolve().parents[3]
    manifests = root / "governance" / "manifests"

    publish_idx = json.loads((manifests / "publish.index.json").read_text(encoding="utf-8"))
    layers = json.loads((manifests / "publish.layers.json").read_text(encoding="utf-8"))
    governance_manifest = json.loads((manifests / "governance.manifest.json").read_text(encoding="utf-8"))
    runtime_entrypoints = json.loads((manifests / "runtime.entrypoints.json").read_text(encoding="utf-8"))

    runtime = None
    for t in publish_idx.get("targets", []):
        if t.get("target") == "runtime-governance":
            runtime = t
            break
    if runtime is None:
        raise SystemExit("governance_manifest_surface_contract: missing runtime-governance target")

    for rel in runtime.get("includes", []):
        if not (root / "governance" / rel).exists():
            raise SystemExit(f"governance_manifest_surface_contract: missing runtime include: governance/{rel}")

    runtime_surface = layers.get("runtime_surface", {})
    required = [
        runtime_surface.get("classification"),
        runtime_surface.get("families"),
        runtime_surface.get("specializations"),
        runtime_surface.get("manifests"),
        runtime_surface.get("overlays", {}).get("regulatory"),
        runtime_surface.get("overlays", {}).get("sector"),
        runtime_surface.get("overlays", {}).get("contextual"),
    ]
    for rel in required:
        if not isinstance(rel, str) or not rel:
            raise SystemExit("governance_manifest_surface_contract: invalid runtime_surface map")
        if not (root / "governance" / rel).exists():
            raise SystemExit(f"governance_manifest_surface_contract: missing runtime surface path: governance/{rel}")

    if governance_manifest.get("resolution_entrypoints_ref") != "model/manifests/runtime.entrypoints.json":
        raise SystemExit("governance_manifest_surface_contract: invalid governance manifest resolution_entrypoints_ref")

    entries = runtime_entrypoints.get("entrypoints", [])
    if not entries:
        raise SystemExit("governance_manifest_surface_contract: no runtime entrypoints")
    first = entries[0]
    if first.get("governance_manifest_ref") != "model/manifests/governance.manifest.json":
        raise SystemExit("governance_manifest_surface_contract: invalid entrypoint governance_manifest_ref")
    if first.get("resolution_order_ref") != "model/manifests/domain-resolution-order.json":
        raise SystemExit("governance_manifest_surface_contract: invalid entrypoint resolution_order_ref")

    print("governance_manifest_surface_contract: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
