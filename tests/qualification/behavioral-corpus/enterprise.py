#!/usr/bin/env python3
"""Inspect or advance a selected operational Case using ordinary YAI CLI.

No reset, generated business evidence, fixture injection or implicit provider
dispatch. The repository documents are real release criteria/contracts; their
acquisition is not proof those criteria pass.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools/validation"))
from behavioral_corpus import Host
CASE = "case:yai-enterprise-launch"
PARTICIPANT = "participant:operator"
POLICY = "tests/qualification/studio-product-vertical/policy.json"
MATERIAL = [("release-validation", "docs/test-cases.md", "text/markdown"),
            ("provider-contract", "docs/provider-governance.md", "text/markdown"),
            ("application-capabilities", "docs/reference/application-capabilities.md", "text/markdown"),
            ("application-package", "application/yai-application/Cargo.toml", "text/plain"),
            ("studio-package", "studio/package.json", "application/json")]


def perimeter(root):
    paths = [POLICY, *(path for _, path, _ in MATERIAL)]
    for path in paths:
        if not (root / path).is_file() or (root / path).stat().st_size > 65536:
            raise ValueError(f"Material absent or outside acquisition bound: {path}")
    resource = dict(schema="yai.resource_definition.v1", attachment_id="resource:infra-operations" if CASE == "case:tech-infra-inference-service" else "resource:enterprise-repository",
                    policy_owner=PARTICIPANT, participant_ids=[PARTICIPANT],
                    operations=["discover", "admit_content", "content_read"], read_prefixes=paths,
                    names=[], max_output_bytes=65536, max_items=16,
                    address=dict(kind="discovery", root=str(root)))
    sources = [dict(name="release-policy", resource=resource["attachment_id"], roles=["policy"],
                    action=dict(action="discover", path=POLICY), media_type="application/json", bootstrap_policy=True)]
    sources.extend(dict(name=name, resource=resource["attachment_id"], roles=["knowledge"],
                        action=dict(action="discover", path=path), media_type=media, bootstrap_policy=False)
                   for name, path, media in MATERIAL)
    return dict(schema="yai.source_perimeter.v1", name="infra-operations" if CASE == "case:tech-infra-inference-service" else "enterprise-release", participant=PARTICIPANT,
                resources=[resource], sources=sources)


def main():
    global CASE, MATERIAL
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["inspect", "advance"])
    parser.add_argument("--scenario", choices=["release", "infrastructure"], default="release")
    parser.add_argument("--yai", type=Path, required=True)
    parser.add_argument("--tenant", required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--perimeter", type=Path, required=True, help="Generated path-qualified manifest location")
    parser.add_argument("--profile", type=Path, help="Write a new corpus profile from acquired Source identities, not Recall results")
    args = parser.parse_args()
    if args.scenario == "infrastructure":
        CASE = "case:tech-infra-inference-service"
        MATERIAL = [("operations-brief", "tests/qualification/behavioral-corpus/infra-operations.md", "text/markdown"),
                    ("operations-workflow", "tests/qualification/behavioral-corpus/infra-workflow.json", "application/json"),
                    ("provider-contract", "docs/provider-governance.md", "text/markdown")]
    if not os.environ.get("YAI_HOME"):
        parser.error("Explicit YAI_HOME required")
    run = f"{args.scenario}-{time.time_ns()}"
    order = 0
    args.evidence.parent.mkdir(parents=True, exist_ok=True)
    with args.evidence.open("x") as evidence:
        def emit(record):
            nonlocal order
            order += 1
            evidence.write(json.dumps(dict(run_id=run, order=order, **record)) + "\n")
            evidence.flush()

        def cli(*parts):
            argv = [str(args.yai.resolve()), *map(str, parts), "--json"]
            proc = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True, timeout=120)
            emit(dict(command=argv, cwd=str(ROOT), yai_home=os.environ["YAI_HOME"],
                      exit=proc.returncode, stdout=proc.stdout, stderr=proc.stderr))
            if proc.returncode:
                raise RuntimeError(f"Product command refused; inspect evidence: {parts}")
            data = json.loads(proc.stdout)["data"]
            return data.get("value", data.get("case", data))

        catalog = cli("case", "list")
        exists = any(row[0] == CASE for row in catalog["rows"])
        emit(dict(material_pre_state="existing" if exists else "absent", case_ref=CASE))
        if not exists:
            if args.mode == "inspect":
                print(json.dumps(dict(case_ref=CASE, posture="absent", run_id=run)))
                return
            cli("case", "create", CASE, "--tenant", args.tenant)
        before = cli("case", "show", CASE)
        if before["tenant_id"] != args.tenant:
            raise ValueError("Existing Case belongs to another Tenant")
        cli("case", "verify", CASE)
        inventory = cli("case", "sources", "inventory", CASE)
        def write_profile():
            if not args.profile:
                return
            source_name, material_path, _ = MATERIAL[0]
            source = next(s for s in inventory["sources"] if s["name"] == source_name)
            if source["phase"] != "acquired":
                raise ValueError(f"Corpus profile requires acquired {source_name}")
            revision = source["progress"]["revision"]
            material = next(item for item in revision["items"] if item["path"] == material_path)
            profile = dict(case_ref=CASE, tenant_ref=args.tenant, participant_ref=PARTICIPANT,
                           release_source_ref=source["source_id"], release_revision_ref=revision["revision_id"],
                           release_digest=material["digest"])
            if args.scenario == "infrastructure":
                for suffix in ("source_ref", "revision_ref", "digest"):
                    profile["operations_" + suffix] = profile.pop("release_" + suffix)
            with args.profile.open("x") as stream:
                json.dump(profile, stream, indent=2)
                stream.write("\n")
            emit(dict(profile=str(args.profile), profile_source="acquisition_inventory", exact_refs=profile))
        if args.mode == "inspect":
            write_profile()
            print(json.dumps(dict(case_ref=CASE, generation=before["generation"], run_id=run)))
            return
        roles = next((p["roles"] for p in before["participants"] if p["participant_id"] == PARTICIPANT), [])
        for role in ("operation-proposer", "workflow-input"):
            if role not in roles:
                cli("case", "participant", "role", "add", CASE, "--participant", PARTICIPANT, "--role", role)
        # Linking intentionally refuses duplicates. Inspect the current authorized
        # attachment rather than treating that refusal as a successful mutation.
        attachment = Host(os.environ["YAI_HOME"]).call("case.summary", {"case_ref": CASE}, run)
        emit(dict(operation="case.summary", input={"case_ref": CASE}, result=attachment))
        linked = attachment.get("data", {}).get("case", {}).get("participant_ref")
        if linked and linked != PARTICIPANT:
            raise ValueError("Reconcile the existing Principal attachment explicitly")
        if linked != PARTICIPANT:
            cli("case", "participant", "link-principal", CASE, "--participant", PARTICIPANT, "--principal", "self")
        args.perimeter.parent.mkdir(parents=True, exist_ok=True)
        definition = perimeter(ROOT)
        args.perimeter.write_text(json.dumps(definition, indent=2) + "\n")
        cli("case", "sources", "declare", CASE, "--file", args.perimeter)
        inventory = cli("case", "sources", "inventory", CASE)
        policy = next(s for s in inventory["sources"] if s["name"] == "release-policy")
        if policy["phase"] != "acquired":
            inventory = cli("case", "sources", "acquire", CASE, "--source", "release-policy")
        if inventory["effective_policy"]["readiness"] != "ready":
            cli("case", "sources", "publish", CASE, "--source", "release-policy", "--reason",
                "Permit declared documentary acquisition and reads only; no infrastructure effect or operational approval")
        for name, _, _ in MATERIAL:
            inventory = cli("case", "sources", "inventory", CASE)
            source = next(s for s in inventory["sources"] if s["name"] == name)
            if source["phase"] != "acquired":
                cli("case", "sources", "resume", CASE, "--source", name)
        cli("case", "knowledge", "build", CASE)
        cli("case", "verify", CASE)
        after = cli("case", "show", CASE)
        inventory = cli("case", "sources", "inventory", CASE)
        emit(dict(result="advanced", before_generation=before["generation"], after_generation=after["generation"],
                  source_refs=[s["name"] for s in inventory["sources"]], assessment="not_assessed", scenario=args.scenario))
        write_profile()
        print(json.dumps(dict(run_id=run, case_ref=CASE, generation=after["generation"], assessment="not_assessed", scenario=args.scenario)))


if __name__ == "__main__":
    main()
