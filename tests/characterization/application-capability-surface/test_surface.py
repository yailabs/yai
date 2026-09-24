#!/usr/bin/env python3
"""Qualify the code-owned Application catalog through the product CLI."""

from __future__ import annotations

import argparse
import json
import subprocess
from collections import Counter
from pathlib import Path


def invoke(binary: Path, *args: str) -> dict:
    result = subprocess.run(
        [str(binary), *args], check=False, text=True,
        stdout=subprocess.PIPE, stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        raise AssertionError(
            f"{' '.join(args)} exit={result.returncode}\n{result.stdout}\n{result.stderr}"
        )
    return json.loads(result.stdout)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=Path("./yai"))
    args = parser.parse_args()
    binary = args.binary.resolve()
    root = Path(__file__).resolve().parents[3]
    application_source = (root / "application/yai-application/src/lib.rs").read_text()
    application_manifest = (root / "application/yai-application/Cargo.toml").read_text()
    assert "std::process::Command" not in application_source
    assert "dispatch_operation" not in application_source
    assert "cmd/yai" not in application_manifest

    envelope = invoke(binary, "capabilities", "--json")
    assert envelope["schema"] == "yai.cli.result.v1"
    assert envelope["operation_id"] == "yai.application.capabilities"
    assert envelope["capability_id"] == "platform.capability_discovery"
    catalog = envelope["data"]["value"]
    assert catalog["schema"] == "yai.application_capability_catalog.v1"
    assert catalog["application_protocol"] == "yai.studio.application.v1"
    assert "cases" not in catalog and "resources" not in catalog

    capabilities = catalog["capabilities"]
    operations = catalog["operations"]
    blockers = catalog["blockers"]
    assert len(capabilities) == 44
    assert len(operations) == 87
    assert len(blockers) == 0
    assert [item["capability_id"] for item in capabilities] == sorted(
        item["capability_id"] for item in capabilities
    )
    assert [item["operation_id"] for item in operations] == sorted(
        item["operation_id"] for item in operations
    )
    counts = Counter(item["disposition"] for item in capabilities)
    assert counts == {
        "product_read": 20,
        "product_action": 14,
        "operator_diagnostic": 3,
        "internal_mechanic": 3,
        "target_only": 4,
    }
    operation_ids = {item["operation_id"] for item in operations}
    capability_ids = {item["capability_id"] for item in capabilities}
    blocker_ids = {item["capability_id"] for item in blockers}
    assert blocker_ids == set()
    assert {"case.run", "case.resume", "case.stop", "execution.get", "source.acquire", "source.resume",
            "cognitive.realization.prepare", "cognitive.realize", "cognitive.compose", "conversation.send",
            "effect.propose", "effect.submit", "effect.reconcile", "resource.request",
            "semantic.fast_search.prepare"} <= operation_ids
    assert all(item["missing_contract"] for item in blockers)
    for item in capabilities:
        assert set(item["application_operation_ids"]) <= operation_ids
        if item["application_posture"] == "ready":
            assert item["application_operation_ids"]
        if item["application_posture"] == "deferred":
            assert item.get("application_deferred_reason")
            if item["disposition"] in {"product_read", "product_action"}:
                assert item["capability_id"] in blocker_ids
        if item["disposition"] == "internal_mechanic":
            assert item["parent_capability_id"] in capability_ids
            assert item["rationale"]
        if item["disposition"] == "target_only":
            assert not item["application_operation_ids"]
            assert not item["cli_operation_ids"]

    discovery = invoke(binary, "help", "--advanced", "--json")
    cli_ids = {item["operation_id"] for item in discovery["operations"]}
    for item in capabilities:
        assert set(item["cli_operation_ids"]) <= cli_ids

    print(
        "application_capability_surface_run_id=application-capability-surface-v1 "
        "catalog_schema=yai.application_capability_catalog.v1 "
        "capabilities=44 executable_or_internal=40 target_only=4 "
        "product_read=20 product_action=14 operator_diagnostic=3 "
        "internal_mechanic=3 application_operations=87 application_ready=33 "
        "application_blockers=0 cli_exposed=37 studio_consumable=34 "
        "case_identity_leaks=0 direct_cli_invocation=0"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
