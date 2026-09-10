#!/usr/bin/env python3
"""Real product inspection and policy lifecycle, fresh LMDB, no provider."""
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]


def main():
    with tempfile.TemporaryDirectory(prefix="yai-experience-product-") as directory:
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

        def inspect(at, *args):
            return json.loads(cli("case", "experience", case, at, *args, "--json"))["data"]["value"]

        def generation():
            return json.loads(cli("case", "history", case, "--json"))["data"]["value"]["total_transitions"]

        def publish(path):
            intake = cli("policy", "ingest", path, "--tenant", tenant)
            artifact = re.search(r"^artifact_id: (.+)$", intake, re.M)[1]
            cli("policy", "validate", artifact, "--reason", "inspect exact test policy")
            cli("policy", "publish", artifact, "--reason", "explicit automated qualification")
            return artifact

        tenant, case, human = "tenant:experience", "case:experience", "participant:operator"
        cli("init", "--tenant", tenant, "--organization", "organization:engineering")
        cli("case", "create", case, "--tenant", tenant)
        cli("case", "participant", "role", "add", case, "--participant", human, "--role", "operation-proposer")
        cli("case", "participant", "link-principal", case, "--principal", "self", "--participant", human)
        first = publish(ROOT / "tests/cases/05-governance-cognitive/policy.json")
        cli("case", "policy", "bind", case, "--artifact", first, "--reason", "exact initial version")
        at = generation()
        one = inspect(at)
        p1 = one["events"][0]["object_refs"][0]
        source = json.loads((ROOT / "tests/cases/05-governance-cognitive/policy.json").read_text())
        source["source_version"] = "2"
        source["rules"][0]["effect"] = "deny"
        second_path = run / "policy2.json"
        second_path.write_text(json.dumps(source))
        second = publish(second_path)
        cli("case", "policy", "replace", case, "--binding", p1, "--artifact", second, "--reason", "explicit replacement not chronology")
        now = inspect("current")
        p2 = now["events"][-1]["object_refs"][0]
        q = ("--from", p1, "--to", p2)
        related = inspect("current", *q)
        assert [r["kind"] for r in related["relations"]] == ["policy_replacement"]
        assert inspect(at) == one
        assert inspect("current", "--from", p2, "--to", p1)["relations"] == []
        human_output = cli("case", "experience", case, "current", *q)
        assert "PolicyReplacement/Lifecycle" in human_output and "backing:" in human_output
        assert "Recording order is not causality" in human_output
        cli("policy", "revoke", second, "--reason", "current authority withdrawn")
        assert inspect("current", *q) == related  # history, not current authority
        cli("case", "policy", "unbind", case, "--binding", p2, "--reason", "retain history")
        final = inspect("current")
        assert any(r["kind"] == "policy_unbinding" for r in final["relations"])
        for flags in [("--limit", "1"), ("--from", p1), ("--hops", "0"), ("--participant", "participant:absent")]:
            cli("case", "experience", case, "current", *flags, reject=True)
        cli("case", "experience", case, generation() + 1, reject=True)
        history = cli("case", "history", case, "--json")
        cli("case", "policy", "rebuild", "--case", case)
        assert inspect("current") == final  # new process + rebuilt derived policy
        assert cli("case", "history", case, "--json") == history
        cli("case", "verify", case)
        print("experience_product=PASS provider=no_provider policy_replacement=qualified unbinding=qualified current_permission_not_restored=true human_readable=true restart_equal=true reads_append=0", flush=True)


if __name__ == "__main__":
    main()
