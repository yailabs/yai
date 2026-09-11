#!/usr/bin/env python3
"""Normal product Recall, fresh LMDB, exact policy sources; no model fixture."""
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]


def main():
    with tempfile.TemporaryDirectory(prefix="yai-recall-product-") as directory:
        run = Path(directory)
        env = dict(os.environ, YAI_HOME=str(run / "home"))
        order = 0
        print(json.dumps(dict(run_id=run.name, cwd=str(ROOT), home=env["YAI_HOME"],
            pre_state="fresh disposable home", provider_mode="no_provider",
            baseline=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip())), flush=True)

        def cli(*args, reject=False):
            nonlocal order
            order += 1
            result = subprocess.run(["./yai", *map(str, args)], cwd=ROOT, env=env,
                capture_output=True, text=True, timeout=30)
            print(json.dumps(dict(order=order, command=["./yai", *map(str, args)], exit=result.returncode,
                stdout=result.stdout, stderr=result.stderr)), flush=True)
            assert (result.returncode != 0) == reject, result
            return result.stdout

        def data(*args):
            return json.loads(cli(*args, "--json"))["data"]["value"]

        def recalled(query="policy", *flags):
            return data("case", "recall", case, query, *flags)["trace"]

        def publish(path):
            intake = cli("policy", "ingest", path, "--tenant", tenant)
            artifact = re.search(r"^artifact_id: (.+)$", intake, re.M)[1]
            cli("policy", "validate", artifact, "--reason", "inspect exact Recall oracle source")
            cli("policy", "publish", artifact, "--reason", "explicit automated qualification")
            return artifact

        tenant, case, participant = "tenant:recall", "case:recall", "participant:operator"
        cli("init", "--tenant", tenant, "--organization", "organization:engineering")
        cli("case", "create", case, "--tenant", tenant)
        cli("case", "participant", "role", "add", case, "--participant", participant, "--role", "operation-proposer")
        cli("case", "participant", "link-principal", case, "--principal", "self", "--participant", participant)
        first = publish(ROOT / "tests/cases/05-governance-cognitive/policy.json")
        cli("case", "policy", "bind", case, "--artifact", first, "--reason", "initial qualified version")
        original = recalled()
        at = original["generation"]
        p1 = original["events"][0]["event"]["object_refs"][0]
        source = json.loads((ROOT / "tests/cases/05-governance-cognitive/policy.json").read_text())
        source["source_version"] = "2"
        source["rules"][0]["effect"] = "deny"
        path = run / "policy2.json"
        path.write_text(json.dumps(source))
        second = publish(path)
        cli("case", "policy", "replace", case, "--binding", p1, "--artifact", second, "--reason", "explicit policy supersession")
        current = recalled("policy", "--ref", p1)
        assert current["closure_complete"]
        assert len(current["events"]) == 2
        assert any(r["kind"] == "policy_replacement" for r in current["relations"])
        assert current["events"][0]["validity_at_cut"] == "historical_binding_not_current_at_cut"
        old = recalled("policy", "--at", at)
        assert len(old["events"]) == 1
        assert old["events"][0]["validity_at_cut"] == "bound_at_cut_not_execution_permission"
        assert old["trace_id"] != current["trace_id"]
        by_transition = recalled("policy", "--at", old["events"][0]["event"]["transition_id"])
        assert by_transition == old  # normalized exact coordinate identity
        human = cli("case", "recall", case, "policy", "--ref", p1)
        for text in ["CASE RECALL", "SEGMENT", "SOURCE", "PolicyReplacement/Lifecycle", "Recording order is not causality"]:
            assert text in human
        assert recalled("nomatchingterm")["events"] == []
        for flags in [("--at", "9999999"), ("--ref", "binding:absent"), ("--participant", "participant:hidden"),
                      ("--limit", "0"), ("--limit", "1", "--ref", p1), ("--candidates", "0"), ("--hops", "0")]:
            cli("case", "recall", case, "policy", *flags, reject=True)
        history = data("case", "history", case)
        cli("case", "policy", "rebuild", "--case", case)
        assert recalled("policy", "--ref", p1) == current  # separate executable/process
        assert data("case", "history", case) == history
        cli("case", "verify", case)
        print("recall_product=PASS source_closed=true current_asof=distinct exact_transition=equivalent policy_supersession=qualified human_readable=true restart_rebuild=equal reads_append=0 provider=no_provider W_injection=none", flush=True)


if __name__ == "__main__":
    main()
