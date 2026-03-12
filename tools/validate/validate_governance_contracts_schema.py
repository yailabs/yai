#!/usr/bin/env python3
import json
from pathlib import Path


def _load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def _require(path: Path, root: Path):
    if not path.exists():
        raise SystemExit(f"governance_contracts_schema: missing {path.relative_to(root)}")


def main() -> int:
    root = Path(__file__).resolve().parents[2]
    gov = root / "governance"
    protocol_contracts = root / "lib" / "protocol" / "contracts" / "schema"
    protocol_headers = root / "include" / "yai" / "protocol" / "contracts"
    schema = gov / "schema"
    manifests = gov / "manifests"
    grammar_schema = gov / "grammar" / "schema"
    registry_schema = gov / "registry" / "schema"

    required_contracts = [
        protocol_contracts / "control" / "control_plane.v1.json",
        protocol_contracts / "control" / "control_call.v1.json",
        protocol_contracts / "control" / "exec_reply.v1.json",
        protocol_contracts / "providers" / "providers.v1.json",
        protocol_contracts / "vault" / "vault_abi.json",
        protocol_headers / "protocol.h",
        protocol_headers / "rpc_runtime.h",
        protocol_headers / "yai_vault_abi.h",
    ]
    required_schema = [
        schema / "policy.v1.schema.json",
        schema / "decision_record.v1.schema.json",
        schema / "governance_review_state.v1.schema.json",
        schema / "evidence_index.v1.schema.json",
        schema / "workspace_governance_attachment.v1.schema.json",
        schema / "retention_policy.v1.schema.json",
        schema / "containment_metrics.v1.schema.json",
        schema / "verification_report.v1.schema.json",
    ]
    required_cross = [
        manifests / "governance.manifest.json",
        manifests / "runtime.entrypoints.json",
        manifests / "governance-attachability.constraints.v1.json",
        grammar_schema / "governance_manifest.v1.schema.json",
        grammar_schema / "runtime_entrypoint.v1.schema.json",
        registry_schema / "commands.v1.schema.json",
        registry_schema / "primitives.v1.schema.json",
    ]

    for p in required_contracts + required_schema + required_cross:
        _require(p, root)

    control_plane = _load_json(protocol_contracts / "control" / "control_plane.v1.json")
    providers = _load_json(protocol_contracts / "providers" / "providers.v1.json")
    runtime_entrypoints = _load_json(manifests / "runtime.entrypoints.json")
    attachability = _load_json(manifests / "governance-attachability.constraints.v1.json")
    review_state = _load_json(schema / "governance_review_state.v1.schema.json")
    attachment = _load_json(schema / "workspace_governance_attachment.v1.schema.json")

    planes = control_plane.get("rpc", {}).get("ontology", {}).get("primary_planes", [])
    if "knowledge" not in planes:
        raise SystemExit("governance_contracts_schema: control plane ontology must include knowledge")

    provider_plane = providers.get("ontology", {}).get("primary_plane")
    if provider_plane != "knowledge":
        raise SystemExit("governance_contracts_schema: providers primary_plane must be knowledge")

    entries = runtime_entrypoints.get("entrypoints", [])
    if not entries:
        raise SystemExit("governance_contracts_schema: runtime.entrypoints has no entries")
    first = entries[0]
    for key in [
        "governance_attachability_constraints_ref",
        "resolution_order_ref",
        "compliance_resolution_order_ref",
        "compatibility_ref",
    ]:
        if not str(first.get(key, "")).startswith("model/manifests/"):
            raise SystemExit(f"governance_contracts_schema: runtime entrypoint invalid {key}")

    if "constraints" not in attachability or "allowed_by_kind" not in attachability["constraints"]:
        raise SystemExit("governance_contracts_schema: attachability manifest missing constraints.allowed_by_kind")

    disallowed_review = (
        review_state.get("properties", {})
        .get("disallowed_subsystem_targets", {})
        .get("items", {})
        .get("enum", [])
    )
    disallowed_attachment = (
        attachment.get("properties", {})
        .get("disallowed_subsystem_targets", {})
        .get("items", {})
        .get("enum", [])
    )
    for expected in ("brain", "mind"):
        if expected not in disallowed_review or expected not in disallowed_attachment:
            raise SystemExit("governance_contracts_schema: disallowed subsystem target set incomplete")

    print("governance_contracts_schema: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
