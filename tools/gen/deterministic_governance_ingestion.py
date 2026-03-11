#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import shlex
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, Iterable, List, Tuple

ROOT = Path(__file__).resolve().parents[2]
SCHEMA_DIR = ROOT / "governance" / "grammar" / "schema"
INGESTION_DIR = ROOT / "governance" / "ingestion"

SOURCE_SCHEMA = SCHEMA_DIR / "enterprise_governance_source.v1.schema.json"
PARSED_SCHEMA = SCHEMA_DIR / "governance_parsed_facts.v1.schema.json"
NORMALIZED_SCHEMA = SCHEMA_DIR / "enterprise_governance_normalized.v1.schema.json"
CANDIDATE_SCHEMA = SCHEMA_DIR / "enterprise_custom_governance.v1.schema.json"

KNOWN_FACT_TYPES = {
    "approval_required",
    "review_required",
    "sink_restricted",
    "evidence_required",
    "contract_required",
    "retention_required",
    "authority_escalation",
    "exception_request",
    "outbound_restriction",
    "publication_restriction",
    "distribution_restriction",
    "domain_targeted",
    "specialization_targeted",
}


@dataclass(frozen=True)
class RuleLine:
    statement: str
    statement_ref: str


def _read_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as f:
        return json.load(f)


def _write_json(path: Path, payload: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2, ensure_ascii=True)
        f.write("\n")


def _repo_rel(path: Path) -> str:
    p = path.resolve()
    try:
        return str(p.relative_to(ROOT))
    except Exception:
        return str(path)


def _now_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def _load_source(source_path: Path) -> Dict[str, Any]:
    source = _read_json(source_path)
    if source.get("kind") != "enterprise_governance_source":
        raise ValueError(f"{source_path}: kind must be enterprise_governance_source")
    if source.get("schema_version") != "v1":
        raise ValueError(f"{source_path}: schema_version must be v1")
    if not source.get("source_id"):
        raise ValueError(f"{source_path}: missing source_id")
    if not source.get("source_payload_ref"):
        raise ValueError(f"{source_path}: missing source_payload_ref")
    return source


def _source_id(source: Dict[str, Any]) -> str:
    raw = str(source.get("source_id", "")).strip()
    if not raw:
        raise ValueError("source_id is required")
    return raw.replace("/", "_")


def _payload_path(source: Dict[str, Any]) -> Path:
    payload_ref = str(source["source_payload_ref"])
    payload = (ROOT / payload_ref).resolve()
    if not payload.exists():
        raise FileNotFoundError(f"missing payload: {payload_ref}")
    return payload


def _rule_lines_from_markdown(path: Path) -> List[RuleLine]:
    lines: List[RuleLine] = []
    rel = _repo_rel(path)
    text = path.read_text(encoding="utf-8").splitlines()
    for i, raw in enumerate(text, start=1):
        line = raw.strip()
        if not line:
            continue
        if line.startswith("- "):
            line = line[2:].strip()
        if line.startswith("* "):
            line = line[2:].strip()
        if line.startswith("RULE "):
            lines.append(RuleLine(statement=line, statement_ref=f"{rel}:{i}"))
    return lines


def _rule_lines_from_yaml_rule_sheet(path: Path) -> List[RuleLine]:
    # Deterministic restricted mode: we only consume explicit RULE lines.
    lines: List[RuleLine] = []
    rel = _repo_rel(path)
    for i, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        line = raw.strip()
        if line.startswith("- "):
            line = line[2:].strip()
        if line.startswith("RULE "):
            lines.append(RuleLine(statement=line, statement_ref=f"{rel}:{i}"))
    return lines


def _rule_lines_from_json_rule_sheet(path: Path) -> List[RuleLine]:
    payload = _read_json(path)
    lines: List[RuleLine] = []
    rel = _repo_rel(path)
    rules = payload.get("rules", [])
    for idx, rule in enumerate(rules):
        ref = f"{rel}#rules[{idx}]"
        if isinstance(rule, str):
            statement = rule.strip()
            if statement.startswith("RULE "):
                lines.append(RuleLine(statement=statement, statement_ref=ref))
            continue
        if isinstance(rule, dict):
            fact_type = str(rule.get("fact_type", "")).strip()
            if not fact_type:
                continue
            tokens = [f"RULE {fact_type}"]
            for key in sorted(rule.keys()):
                if key == "fact_type":
                    continue
                val = rule[key]
                if isinstance(val, bool):
                    sval = "true" if val else "false"
                else:
                    sval = str(val)
                tokens.append(f"{key}={sval}")
            lines.append(RuleLine(statement=" ".join(tokens), statement_ref=ref))
    for idx, stmt in enumerate(payload.get("statements", [])):
        ref = f"{rel}#statements[{idx}]"
        if isinstance(stmt, str) and stmt.strip().startswith("RULE "):
            lines.append(RuleLine(statement=stmt.strip(), statement_ref=ref))
    return lines


def _parse_rule_statement(rule_line: RuleLine) -> Tuple[Dict[str, Any] | None, Dict[str, Any] | None]:
    try:
        tokens = shlex.split(rule_line.statement)
    except Exception as exc:
        return None, {"statement_ref": rule_line.statement_ref, "error": f"tokenization_error: {exc}"}
    if len(tokens) < 2 or tokens[0] != "RULE":
        return None, {"statement_ref": rule_line.statement_ref, "error": "invalid_rule_prefix"}

    fact_type = tokens[1].strip()
    attrs: Dict[str, str] = {}
    invalid_tokens: List[str] = []
    for t in tokens[2:]:
        if "=" not in t:
            invalid_tokens.append(t)
            continue
        k, v = t.split("=", 1)
        k = k.strip()
        v = v.strip()
        if not k:
            invalid_tokens.append(t)
            continue
        attrs[k] = v

    parsed: Dict[str, Any] = {
        "fact_type": fact_type,
        "attributes": attrs,
        "raw_statement": rule_line.statement,
        "raw_statement_ref": rule_line.statement_ref,
        "invalid_tokens": invalid_tokens,
    }
    return parsed, None


def _collect_rule_lines(source: Dict[str, Any], payload: Path) -> List[RuleLine]:
    source_format = source.get("source_format")
    if source_format == "markdown_rule_sheet":
        return _rule_lines_from_markdown(payload)
    if source_format == "yaml_rule_sheet":
        return _rule_lines_from_yaml_rule_sheet(payload)
    if source_format == "json_rule_sheet":
        return _rule_lines_from_json_rule_sheet(payload)
    raise ValueError(f"unsupported source_format: {source_format}")


def _parsed_path(source: Dict[str, Any], out: Path | None) -> Path:
    if out is not None:
        return out
    return INGESTION_DIR / "parsed" / f"{_source_id(source)}.parsed.v1.json"


def _normalized_path(source: Dict[str, Any], out: Path | None) -> Path:
    if out is not None:
        return out
    return INGESTION_DIR / "normalized" / f"{_source_id(source)}.normalized.v1.json"


def _candidate_path_from_source(source: Dict[str, Any], out: Path | None) -> Path:
    if out is not None:
        return out
    org = str(source.get("owner", {}).get("organization_id", "org")).strip() or "org"
    sid = _source_id(source).replace(".", "-")
    return INGESTION_DIR / "candidates" / f"enterprise.{org}.{sid}.candidate.v1.json"


def _parse(source_path: Path, out: Path | None) -> Path:
    source = _load_source(source_path)
    payload = _payload_path(source)
    rule_lines = _collect_rule_lines(source, payload)

    facts: List[Dict[str, Any]] = []
    unresolved: List[Dict[str, Any]] = []
    invalid: List[Dict[str, Any]] = []
    warnings: List[str] = []

    parsed_count = 0
    unknown_count = 0
    for idx, rule_line in enumerate(rule_lines, start=1):
        parsed_stmt, parse_error = _parse_rule_statement(rule_line)
        if parse_error is not None:
            invalid.append(parse_error)
            continue
        assert parsed_stmt is not None
        fact_type = str(parsed_stmt["fact_type"])
        attrs = dict(parsed_stmt["attributes"])
        invalid_tokens = list(parsed_stmt["invalid_tokens"])

        status = "parsed"
        if fact_type not in KNOWN_FACT_TYPES:
            status = "unresolved"
            unknown_count += 1
            unresolved.append(
                {
                    "statement_ref": parsed_stmt["raw_statement_ref"],
                    "reason": "unknown_fact_type",
                    "fact_type": fact_type,
                }
            )

        if invalid_tokens:
            warnings.append(
                f"{parsed_stmt['raw_statement_ref']}: ignored invalid tokens: {', '.join(invalid_tokens)}"
            )

        parsed_count += 1
        fact = {
            "fact_id": f"fact-{idx:04d}",
            "fact_type": fact_type,
            "source_ref": source["source_id"],
            "confidence": float(source.get("confidence_hint", 1.0)),
            "raw_statement_ref": parsed_stmt["raw_statement_ref"],
            "status": status,
            "attributes": attrs,
            "notes": attrs.get("rationale", ""),
        }
        scope_hint = attrs.get("scope") or (source.get("domain_targets") or [None])[0]
        target_hint = attrs.get("target") or attrs.get("sink") or attrs.get("workspace")
        if isinstance(scope_hint, str) and scope_hint:
            fact["scope_hint"] = scope_hint
        if isinstance(target_hint, str) and target_hint:
            fact["target_hint"] = target_hint
        facts.append(fact)

    coverage = {
        "source_format": source.get("source_format"),
        "total_statements": len(rule_lines),
        "parsed_statements": parsed_count,
        "recognized_fact_types": parsed_count - unknown_count,
        "unknown_fact_types": unknown_count,
        "invalid_statements": len(invalid),
    }

    parsed_payload = {
        "kind": "governance_parsed_facts",
        "schema_version": "v1",
        "source_ref": source["source_id"],
        "generated_at": _now_iso(),
        "facts": facts,
        "unresolved_items": unresolved,
        "invalid_items": invalid,
        "source_warnings": warnings,
        "coverage_summary": coverage,
    }

    dst = _parsed_path(source, out)
    _write_json(dst, parsed_payload)
    return dst


def _normalize(source_path: Path, parsed_path: Path | None, out: Path | None) -> Path:
    source = _load_source(source_path)
    parsed_doc_path = parsed_path or _parsed_path(source, None)
    parsed = _read_json(parsed_doc_path)

    facts = list(parsed.get("facts", []))
    rule_candidates: List[Dict[str, Any]] = []
    authority_candidates: List[Dict[str, Any]] = []
    evidence_candidates: List[Dict[str, Any]] = []
    precedence_candidates: List[Dict[str, Any]] = []
    exception_candidates: List[Dict[str, Any]] = []
    unresolved_ambiguities: List[Dict[str, Any]] = list(parsed.get("unresolved_items", []))
    conflict_hints: List[Dict[str, Any]] = []

    publication_modes: set[str] = set()
    distribution_modes: set[str] = set()

    for fact in facts:
        fact_id = str(fact.get("fact_id", ""))
        fact_type = str(fact.get("fact_type", ""))
        attrs = dict(fact.get("attributes", {}))
        if fact.get("status") == "unresolved":
            unresolved_ambiguities.append(
                {
                    "source_fact_ref": fact_id,
                    "reason": "fact_unresolved",
                    "fact_type": fact_type,
                }
            )
            continue

        rule_candidates.append(
            {
                "source_fact_ref": fact_id,
                "fact_type": fact_type,
                "action": attrs.get("action"),
                "target": attrs.get("target") or attrs.get("sink"),
                "mode": attrs.get("mode"),
                "severity": attrs.get("severity"),
            }
        )

        if fact_type == "authority_escalation":
            authority_candidates.append(
                {
                    "source_fact_ref": fact_id,
                    "role": attrs.get("role", "unknown"),
                    "reason": "escalated_authority",
                }
            )
        if fact_type in ("evidence_required", "approval_required"):
            evidence = attrs.get("evidence")
            if evidence:
                evidence_candidates.append(
                    {"source_fact_ref": fact_id, "evidence": evidence, "required": True}
                )
            if fact_type == "approval_required":
                precedence_candidates.append(
                    {
                        "source_fact_ref": fact_id,
                        "mode": "specialization+overlays+enterprise-object",
                        "reason": "approval_required",
                    }
                )
        if fact_type == "exception_request":
            exception_candidates.append(
                {
                    "source_fact_ref": fact_id,
                    "request_type": attrs.get("type", "unspecified"),
                    "scope": attrs.get("scope", "workspace"),
                }
            )
        if fact_type == "publication_restriction" and attrs.get("mode"):
            publication_modes.add(attrs["mode"])
        if fact_type == "distribution_restriction" and attrs.get("mode"):
            distribution_modes.add(attrs["mode"])

    if not precedence_candidates:
        unresolved_ambiguities.append(
            {
                "reason": "missing_precedence_hint",
                "suggested_mode": "specialization+overlays+enterprise-object",
            }
        )
    if len(publication_modes) > 1:
        conflict_hints.append(
            {
                "type": "publication_mode_conflict",
                "modes": sorted(publication_modes),
                "reason": "multiple publication restriction modes detected",
            }
        )
    if len(distribution_modes) > 1:
        conflict_hints.append(
            {
                "type": "distribution_mode_conflict",
                "modes": sorted(distribution_modes),
                "reason": "multiple distribution restriction modes detected",
            }
        )

    build_readiness = "ready_candidate"
    if unresolved_ambiguities or conflict_hints:
        build_readiness = "ready_with_review"

    normalized = {
        "kind": "enterprise_governance_normalized",
        "schema_version": "v1",
        "normalized_id": f"norm.{_source_id(source).replace('.', '-')}",
        "source_refs": [source["source_id"]],
        "parsed_fact_refs": [f.get("fact_id") for f in facts if f.get("fact_id")],
        "organization_scope": source.get("organization_scope", {}),
        "workspace_targets": source.get("workspace_targets", []),
        "domain_targets": source.get("domain_targets", []),
        "specialization_targets": source.get("specialization_targets", []),
        "rule_candidates": rule_candidates,
        "authority_candidates": authority_candidates,
        "evidence_candidates": evidence_candidates,
        "precedence_candidates": precedence_candidates,
        "exception_candidates": exception_candidates,
        "apply_mode_hints": ["workspace_attach"],
        "unresolved_ambiguities": unresolved_ambiguities,
        "conflict_hints": conflict_hints,
        "build_readiness": build_readiness,
        "notes": source.get("notes", ""),
        "generated_at": _now_iso(),
    }

    dst = _normalized_path(source, out)
    _write_json(dst, normalized)
    return dst


def _build_candidate(source_path: Path, normalized_path: Path | None, out: Path | None) -> Path:
    source_path_abs = source_path.resolve()
    source = _load_source(source_path_abs)
    normalized_doc_path = normalized_path or _normalized_path(source, None)
    normalized = _read_json(normalized_doc_path)

    org = str(source.get("owner", {}).get("organization_id", "org")) or "org"
    sid = _source_id(source).replace(".", "-")
    candidate_id = f"enterprise.{org}.{sid}.candidate.v1"
    unresolved = list(normalized.get("unresolved_ambiguities", []))
    conflicts = list(normalized.get("conflict_hints", []))

    precedence_mode = "specialization+overlays+enterprise-object"
    if normalized.get("precedence_candidates"):
        first = normalized["precedence_candidates"][0]
        precedence_mode = first.get("mode") or precedence_mode

    authority_roles = sorted(
        {
            str(a.get("role"))
            for a in normalized.get("authority_candidates", [])
            if str(a.get("role", "")).strip()
        }
    )
    evidence_required = sorted(
        {
            str(e.get("evidence"))
            for e in normalized.get("evidence_candidates", [])
            if str(e.get("evidence", "")).strip()
        }
    )

    runtime_consumable = len(unresolved) == 0 and len(conflicts) == 0
    review_state = "in_review"
    if runtime_consumable:
        review_state = "draft"

    candidate = {
        "kind": "enterprise_custom_governance",
        "schema_version": "v1",
        "id": candidate_id,
        "name": f"{source.get('title', 'Enterprise Governance')} Candidate",
        "version": datetime.now(timezone.utc).strftime("%Y.%m"),
        "owner": source.get("owner", {}),
        "organization_scope": normalized.get("organization_scope", {}),
        "workspace_targets": normalized.get("workspace_targets", []),
        "domain_targets": normalized.get("domain_targets", []),
        "specialization_targets": normalized.get("specialization_targets", []),
        "source_inputs": {
            "customer_policy_pack_refs": [],
            "compliance_refs": [],
            "overlay_refs": [],
            "specialization_policy_refs": [],
            "ingestion_source_refs": [_repo_rel(source_path_abs)],
            "parsed_fact_refs": normalized.get("parsed_fact_refs", []),
            "normalized_ref": _repo_rel(normalized_doc_path),
        },
        "policy_refs": sorted(
            {
                f"ingestion/fact/{rc.get('fact_type')}"
                for rc in normalized.get("rule_candidates", [])
                if rc.get("fact_type")
            }
        ),
        "overlay_refs": {
            "regulatory": [],
            "sector": [],
            "contextual": [],
        },
        "authority_profile": {
            "mode": "candidate",
            "required_roles": authority_roles,
        },
        "evidence_profile": {
            "mode": "candidate",
            "required": evidence_required,
        },
        "precedence_mode": precedence_mode,
        "status": "candidate",
        "review_state": review_state,
        "compatibility": {
            "law_range": ">=0.1.0",
            "runtime_targets": ["yai"],
            "export_profiles": ["runtime-consumer.v4"],
        },
        "apply_modes": normalized.get("apply_mode_hints", ["workspace_attach"]),
        "runtime_binding_model": "workspace-first",
        "runtime_family_targets": ["core", "exec", "data", "graph", "knowledge"],
        "topology_target": "unified-runtime-v1",
        "disallowed_subsystem_targets": ["brain", "mind"],
        "runtime_consumable": runtime_consumable,
        "runtime_entrypoint_refs": ["manifests/runtime.entrypoints.json#default-runtime"],
        "provenance": {
            "source_type": "deterministic_ingestion",
            "source_ref": _repo_rel(source_path_abs),
            "created_by": "tools/gen/deterministic_governance_ingestion.py",
            "updated_by": "tools/gen/deterministic_governance_ingestion.py",
            "normalized_ref": _repo_rel(normalized_doc_path),
            "generated_at": _now_iso(),
        },
        "notes": f"candidate built from deterministic ingestion; unresolved={len(unresolved)} conflicts={len(conflicts)}",
    }

    dst = _candidate_path_from_source(source, out)
    _write_json(dst, candidate)
    return dst


def _schema_validate(instance: Dict[str, Any], schema_path: Path) -> List[str]:
    try:
        from jsonschema import Draft202012Validator
    except Exception as exc:
        return [f"jsonschema unavailable: {exc}"]
    schema = _read_json(schema_path)
    validator = Draft202012Validator(schema)
    errors = []
    for err in sorted(validator.iter_errors(instance), key=lambda e: list(e.path)):
        where = "$." + ".".join(str(x) for x in err.path) if err.path else "$"
        errors.append(f"{where}: {err.message}")
    return errors


def _validate(source_path: Path, parsed_path: Path | None, normalized_path: Path | None, candidate_path: Path | None) -> int:
    source = _read_json(source_path)
    parsed_doc = _read_json(parsed_path or _parsed_path(source, None))
    normalized_doc = _read_json(normalized_path or _normalized_path(source, None))
    candidate_doc = _read_json(candidate_path or _candidate_path_from_source(source, None))

    checks: List[Tuple[str, Dict[str, Any], Path]] = [
        ("source", source, SOURCE_SCHEMA),
        ("parsed", parsed_doc, PARSED_SCHEMA),
        ("normalized", normalized_doc, NORMALIZED_SCHEMA),
        ("candidate", candidate_doc, CANDIDATE_SCHEMA),
    ]

    failed = False
    for name, doc, schema_path in checks:
        errors = _schema_validate(doc, schema_path)
        jsonschema_missing = any(e.startswith("jsonschema unavailable:") for e in errors)
        if jsonschema_missing:
            print(f"[ingest-validate] WARN {name}: {errors[0]}")
            continue
        if errors:
            failed = True
            print(f"[ingest-validate] FAIL {name}")
            for err in errors:
                print(f"- {err}")
        else:
            print(f"[ingest-validate] OK {name}")

    if candidate_doc.get("status") != "candidate":
        failed = True
        print("[ingest-validate] FAIL candidate status must be candidate")

    if failed:
        return 1
    print("[ingest-validate] OK pipeline artifacts")
    return 0


def _inspect_parsed(parsed_doc: Dict[str, Any]) -> None:
    facts = parsed_doc.get("facts", [])
    coverage = parsed_doc.get("coverage_summary", {})
    print("Parsed facts summary")
    print("-------------------")
    print(f"source_ref: {parsed_doc.get('source_ref')}")
    print(f"facts: {len(facts)}")
    print(f"unknown_fact_types: {coverage.get('unknown_fact_types', 0)}")
    print(f"invalid_statements: {coverage.get('invalid_statements', 0)}")
    print(f"warnings: {len(parsed_doc.get('source_warnings', []))}")
    print(f"unresolved_items: {len(parsed_doc.get('unresolved_items', []))}")


def _inspect_normalized(normalized_doc: Dict[str, Any]) -> None:
    print("Normalized IR summary")
    print("---------------------")
    print(f"normalized_id: {normalized_doc.get('normalized_id')}")
    print(f"rule_candidates: {len(normalized_doc.get('rule_candidates', []))}")
    print(f"authority_candidates: {len(normalized_doc.get('authority_candidates', []))}")
    print(f"evidence_candidates: {len(normalized_doc.get('evidence_candidates', []))}")
    print(f"precedence_candidates: {len(normalized_doc.get('precedence_candidates', []))}")
    print(f"unresolved_ambiguities: {len(normalized_doc.get('unresolved_ambiguities', []))}")
    print(f"conflict_hints: {len(normalized_doc.get('conflict_hints', []))}")
    print(f"build_readiness: {normalized_doc.get('build_readiness')}")


def _inspect(source_path: Path, stage: str) -> int:
    source = _load_source(source_path)
    if stage == "parsed":
        parsed_doc = _read_json(_parsed_path(source, None))
        _inspect_parsed(parsed_doc)
        return 0
    if stage == "normalized":
        normalized_doc = _read_json(_normalized_path(source, None))
        _inspect_normalized(normalized_doc)
        return 0
    print(f"unsupported stage: {stage}")
    return 1


def _parse_args(argv: Iterable[str] | None = None) -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Deterministic governance ingestion engine")
    sub = p.add_subparsers(dest="cmd", required=True)

    p_parse = sub.add_parser("parse", help="parse source -> parsed facts")
    p_parse.add_argument("--source", required=True, type=Path)
    p_parse.add_argument("--out", type=Path)

    p_norm = sub.add_parser("normalize", help="normalize parsed facts -> normalized IR")
    p_norm.add_argument("--source", required=True, type=Path)
    p_norm.add_argument("--parsed", type=Path)
    p_norm.add_argument("--out", type=Path)

    p_build = sub.add_parser("build-candidate", help="normalized IR -> candidate object")
    p_build.add_argument("--source", required=True, type=Path)
    p_build.add_argument("--normalized", type=Path)
    p_build.add_argument("--out", type=Path)

    p_val = sub.add_parser("validate", help="validate source/parsed/normalized/candidate artifacts")
    p_val.add_argument("--source", required=True, type=Path)
    p_val.add_argument("--parsed", type=Path)
    p_val.add_argument("--normalized", type=Path)
    p_val.add_argument("--candidate", type=Path)

    p_inspect = sub.add_parser("inspect", help="inspect parsed or normalized stage")
    p_inspect.add_argument("--source", required=True, type=Path)
    p_inspect.add_argument("--stage", required=True, choices=["parsed", "normalized"])

    return p.parse_args(argv)


def main(argv: Iterable[str] | None = None) -> int:
    args = _parse_args(argv)
    cmd = args.cmd
    if cmd == "parse":
        out = _parse(args.source, args.out)
        print(out.relative_to(ROOT))
        return 0
    if cmd == "normalize":
        out = _normalize(args.source, args.parsed, args.out)
        print(out.relative_to(ROOT))
        return 0
    if cmd == "build-candidate":
        out = _build_candidate(args.source, args.normalized, args.out)
        print(out.relative_to(ROOT))
        return 0
    if cmd == "validate":
        return _validate(args.source, args.parsed, args.normalized, args.candidate)
    if cmd == "inspect":
        return _inspect(args.source, args.stage)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
