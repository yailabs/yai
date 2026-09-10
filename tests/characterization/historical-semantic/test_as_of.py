#!/usr/bin/env python3
"""Product commands against fresh LMDB; no model, synthetic provider or raw Transitions."""
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]


def main():
    with tempfile.TemporaryDirectory(prefix="yai-historical-product-") as directory:
        run = Path(directory)
        env = dict(os.environ, YAI_HOME=str(run / "home"))
        order = 0
        print(json.dumps(dict(run_id=run.name, cwd=str(ROOT), home=env["YAI_HOME"],
            pre_state="fresh disposable home", provider_mode="no_provider", baseline=subprocess.check_output(
                ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip())), flush=True)

        def cli(*args, reject=False):
            nonlocal order
            order += 1
            result = subprocess.run(["./yai", *map(str, args)], cwd=ROOT, env=env,
                capture_output=True, text=True, timeout=30)
            print(json.dumps(dict(order=order, command=["./yai", *map(str, args)], exit=result.returncode,
                stdout=result.stdout, stderr=result.stderr)), flush=True)
            assert (result.returncode != 0) == reject, result
            return result.stdout

        def inspect(at):
            return json.loads(cli("case", "as-of", case, at, "--json"))["data"]["value"]

        def generation():
            return json.loads(cli("case", "history", case, "--json"))["data"]["value"]["total_transitions"]

        tenant, case, human = "tenant:historical", "case:historical", "participant:operator"
        cli("init", "--tenant", tenant, "--organization", "organization:engineering")
        cli("case", "create", case, "--tenant", tenant)
        cli("case", "participant", "role", "add", case, "--participant", human, "--role", "operation-proposer")
        cli("case", "participant", "link-principal", case, "--principal", "self", "--participant", human)
        assert inspect(1)["state_then"]["participant"] is None
        before = generation()

        def publish(path):
            intake = cli("policy", "ingest", path, "--tenant", tenant)
            artifact = re.search(r"^artifact_id: (.+)$", intake, re.M)[1]
            cli("policy", "validate", artifact, "--reason", "exact historical source")
            cli("policy", "publish", artifact, "--reason", "explicit product qualification")
            return artifact

        first = publish(ROOT / "tests/cases/05-governance-cognitive/policy.json")
        cli("case", "policy", "bind", case, "--artifact", first, "--reason", "pin historical version")
        p1 = generation()
        old = inspect(p1)
        assert not inspect(before)["state_then"]["policy_bindings"]
        second_source = json.loads((ROOT / "tests/cases/05-governance-cognitive/policy.json").read_text())
        second_source["source_version"] = "2"
        second_source["rules"][0]["effect"] = "deny"
        second_path = run / "policy2.json"
        second_path.write_text(json.dumps(second_source))
        second = publish(second_path)
        cli("case", "policy", "replace", case, "--binding", old["state_then"]["policy_bindings"][0]["binding_id"],
            "--artifact", second, "--reason", "explicitly supersede policy")
        after = inspect(p1)
        assert old["normative_then"] == after["normative_then"]
        assert any(c["category"] == "superseded" for c in after["comparison_to_now"])
        cli("policy", "revoke", second, "--reason", "current authority withdrawn")
        revoked = inspect(p1)
        assert revoked["normative_now"]["validity"] == "revoked"
        assert revoked["normative_then"] == old["normative_then"]
        assert inspect(old["coordinate_transition_id"])["view_id"] == revoked["view_id"]
        current = generation()
        cli("case", "as-of", case, current + 1, reject=True)
        cli("case", "as-of", case, "2026-01-01", reject=True)
        cli("case", "as-of", case, p1, "--participant", "participant:absent", reject=True)
        cli("case", "as-of", case, p1, "--limit", "1", reject=True)
        history = cli("case", "history", case, "--json")
        cli("case", "policy", "rebuild", "--case", case)
        assert inspect(p1) == revoked
        assert cli("case", "history", case, "--json") == history
        cli("case", "verify", case)
        print("historical_product=PASS original_policy_preserved=true current_revocation=true no_inference=true queries_append=0 rebuild_equal=true", flush=True)


if __name__ == "__main__":
    main()
