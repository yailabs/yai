#!/usr/bin/env python3
"""Compare one durable Case across CLI owners and yai-application.

This assertion layer compares semantic identities and bounded facts rather than
human CLI formatting. It never reads persistence and never mutates the Case.
"""

import argparse
import json
import subprocess
from pathlib import Path
from typing import Any


def run_json(command: list[str]) -> dict[str, Any]:
    completed = subprocess.run(command, check=True, text=True, capture_output=True)
    return json.loads(completed.stdout)


def cli_value(yai: str, *arguments: str) -> Any:
    result = run_json([yai, *arguments, "--json"])
    data = result["data"]
    return data.get("value", data.get("case", data))


def require_equal(label: str, left: Any, right: Any) -> None:
    if left != right:
        raise AssertionError(f"{label}: CLI={left!r} application={right!r}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--yai", default="./target/debug/yai")
    parser.add_argument(
        "--application-probe",
        default="application/target/debug/examples/case_summary",
    )
    parser.add_argument("--case", default="case:studio-live-qualification")
    parser.add_argument("--output")
    args = parser.parse_args()

    case = cli_value(args.yai, "case", "show", args.case)
    inventory = cli_value(args.yai, "case", "sources", "inventory", args.case)
    knowledge = cli_value(args.yai, "case", "knowledge", "inspect", args.case)
    workflow = cli_value(args.yai, "workflow", "status", args.case)
    resources = cli_value(args.yai, "case", "resource", "list", args.case)
    provider = cli_value(args.yai, "case", "provider", "show", args.case)
    application_result = run_json([args.application_probe, args.case])
    if application_result["result_state"] != "success":
        raise AssertionError(f"case.summary failed: {application_result}")
    application = application_result["data"]

    cli_source_revisions = {
        source["source_id"]: source.get("progress", {})
        .get("revision", {})
        .get("revision_id")
        for source in inventory["sources"]
    }
    app_source_revisions = {
        source["id"]: source.get("revision_ref")
        for source in application["environment"]["sources"]
    }
    cli_files = {
        (source["source_id"], item["path"], item["digest"])
        for source in inventory["sources"]
        for item in source.get("progress", {}).get("revision", {}).get("items", [])
    }
    app_files = {
        (item["source_ref"], item["path"], item["digest"])
        for item in application["environment"]["files"]
    }
    resource_rows = resources["rows"]
    cli_resource_ids = {row[0] for row in resource_rows}
    app_resource_ids = {item["id"] for item in application["environment"]["resources"]}
    cli_knowledge_source_ids = {item["source_id"] for item in knowledge["view"]["sources"]}
    app_knowledge_source_ids = {item["source_ref"] for item in application["knowledge"]["sources"]}

    require_equal("Case ref", case["case_id"], application["case"]["case_ref"])
    require_equal("generation", case["generation"], application["case"]["generation"])
    require_equal("freshness generation", case["generation"], application["freshness"]["generation"])
    require_equal("Memory generation", case["generation"], application["memory"]["generation"])
    require_equal(
        "Participants",
        {item["participant_id"] for item in case["participants"]},
        {item["id"] for item in application["overview"]["participants"]},
    )
    require_equal("Resources", cli_resource_ids, app_resource_ids)
    require_equal("Source revisions", cli_source_revisions, app_source_revisions)
    require_equal("Files", cli_files, app_files)
    require_equal("Knowledge source refs", cli_knowledge_source_ids, app_knowledge_source_ids)
    measurements = knowledge["measurements"]
    require_equal("Knowledge counts", {
        "sources": measurements["source_documents"],
        "units": measurements["units"],
        "entities": measurements["entities"],
        "claims": measurements["claims"],
        "relations": measurements["relations"],
        "contradictions": measurements["contradictions"],
    }, application["knowledge"]["counts"])
    require_equal(
        "Policy binding count",
        case["policy_bindings"],
        len(application["authority"]["policies"]),
    )
    require_equal(
        "Workflow definition",
        workflow["workflow_definition_id"],
        application["work"]["resolution"]["workflow_definition_id"],
    )
    require_equal(
        "Workflow binding",
        workflow["workflow_binding_id"],
        application["work"]["resolution"]["workflow_binding_id"],
    )
    require_equal(
        "Workflow nodes",
        [(node["node_id"], node["posture"]) for node in workflow["nodes"]],
        [(node["node_id"], node["posture"]) for node in application["work"]["nodes"]],
    )
    provider_fields = {item["name"]: item["value"] for item in provider["fields"]}
    require_equal("Provider posture", provider_fields["provider mode"], "unconfigured")
    require_equal("Application provider targets", application["compute"]["targets"], [])
    require_equal("Human display label", application["case"]["display_name"], "Studio Live Qualification")

    summary = {
        "case_ref": args.case,
        "display_name": application["case"]["display_name"],
        "generation": case["generation"],
        "participants": len(case["participants"]),
        "resources": len(cli_resource_ids),
        "sources": len(cli_source_revisions),
        "files": len(cli_files),
        "knowledge": application["knowledge"]["counts"],
        "timeline": len(application["memory"]["timeline"]),
        "graph_relations": len(application["memory"]["relations"]),
        "policies": len(application["authority"]["policies"]),
        "workflow": workflow["workflow_definition_id"],
        "workflow_completed": workflow["completed"],
        "provider": "unconfigured",
        "conversation_turns": len(application["conversation"]["turns"]),
        "result": "coherent",
    }
    rendered = json.dumps(summary, indent=2, sort_keys=True) + "\n"
    if args.output:
        Path(args.output).write_text(rendered)
    print(rendered, end="")


if __name__ == "__main__":
    main()
