#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ING = ROOT / "governance" / "ingestion"
GRAMMAR_SCHEMA = ROOT / "governance" / "grammar" / "schema"
MANIFESTS = ROOT / "governance" / "manifests"
REGISTRY = ROOT / "governance" / "registry"


def _require(path: Path) -> None:
    if not path.exists():
        raise SystemExit(f"governance_ingestion_pipeline: missing {path.relative_to(ROOT)}")


def _read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def _run_py(script: Path) -> None:
    proc = subprocess.run([sys.executable, str(script)], cwd=str(ROOT), capture_output=True, text=True)
    if proc.returncode != 0:
        out = (proc.stdout + "\n" + proc.stderr).strip()
        raise SystemExit(f"governance_ingestion_pipeline: {script.name} failed\n{out}")


def main() -> int:
    for d in ["sources", "candidates", "parsed", "normalized", "review", "templates", "examples"]:
        _require(ING / d)

    for p in [
        ING / "templates" / "enterprise_governance_source.template.v1.json",
        ING / "sources" / "src.ecohmedia.digital-outbound.source.v1.json",
        ING / "parsed" / "src.ecohmedia.digital-outbound.parsed.v1.json",
        ING / "normalized" / "src.ecohmedia.digital-outbound.normalized.v1.json",
        ING / "candidates" / "enterprise.ecohmedia.src-ecohmedia-digital-outbound.candidate.v1.json",
        GRAMMAR_SCHEMA / "enterprise_governance_source.v1.schema.json",
        GRAMMAR_SCHEMA / "governance_parsed_facts.v1.schema.json",
        GRAMMAR_SCHEMA / "enterprise_governance_normalized.v1.schema.json",
        GRAMMAR_SCHEMA / "enterprise_custom_governance.v1.schema.json",
        MANIFESTS / "runtime.entrypoints.json",
        REGISTRY / "governable-objects.v1.json",
    ]:
        _require(p)

    source = _read_json(ING / "sources" / "src.ecohmedia.digital-outbound.source.v1.json")
    parsed = _read_json(ING / "parsed" / "src.ecohmedia.digital-outbound.parsed.v1.json")
    normalized = _read_json(ING / "normalized" / "src.ecohmedia.digital-outbound.normalized.v1.json")
    candidate = _read_json(ING / "candidates" / "enterprise.ecohmedia.src-ecohmedia-digital-outbound.candidate.v1.json")
    review = _read_json(ING / "review" / "enterprise.ecohmedia.src-ecohmedia-digital-outbound.candidate.v1.review.v1.json")

    if source.get("kind") != "enterprise_governance_source":
        raise SystemExit("governance_ingestion_pipeline: invalid source kind")
    if parsed.get("kind") != "governance_parsed_facts":
        raise SystemExit("governance_ingestion_pipeline: invalid parsed kind")
    if normalized.get("kind") != "enterprise_governance_normalized":
        raise SystemExit("governance_ingestion_pipeline: invalid normalized kind")
    if candidate.get("kind") != "enterprise_custom_governance":
        raise SystemExit("governance_ingestion_pipeline: invalid candidate kind")
    if review.get("kind") != "governance_review_state":
        raise SystemExit("governance_ingestion_pipeline: invalid review kind")

    source_id = str(source.get("source_id", ""))
    if parsed.get("source_ref") != source_id:
        raise SystemExit("governance_ingestion_pipeline: parsed source_ref mismatch")
    if source_id not in normalized.get("source_refs", []):
        raise SystemExit("governance_ingestion_pipeline: normalized source_refs mismatch")

    if candidate.get("runtime_binding_model") != "workspace-first":
        raise SystemExit("governance_ingestion_pipeline: candidate runtime_binding_model must be workspace-first")
    for fam in candidate.get("runtime_family_targets", []):
        if fam not in {"core", "exec", "data", "graph", "knowledge"}:
            raise SystemExit(f"governance_ingestion_pipeline: invalid runtime_family_target {fam}")

    if review.get("object_id") != candidate.get("id"):
        raise SystemExit("governance_ingestion_pipeline: review object_id mismatch")

    _run_py(ROOT / "tools" / "validate" / "validate_ingestion_cli.py")
    _run_py(ROOT / "tools" / "validate" / "validate_review_lifecycle.py")

    print("governance_ingestion_pipeline: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
