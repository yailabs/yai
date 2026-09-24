#!/usr/bin/env python3
"""Reconstruct and qualify a bounded portfolio through normal CLI/Host operations."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import threading
import time

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools/validation"))
from behavioral_corpus import Host, fingerprint
from profile_storage import measure_profile_storage


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--yai", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, default=Path(__file__).with_name("portfolio.json"))
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--retain", action="store_true", help="Retain this newly allocated profile for explicit Studio inspection")
    args = parser.parse_args()
    manifest = json.loads(args.manifest.read_text())
    if manifest.get("schema") != "yai.case_portfolio.v1" or not 2 <= len(manifest["cases"]) <= 4:
        raise ValueError("Select a bounded v1 parallel portfolio")
    cases = manifest["cases"]
    if len({c["case_ref"] for c in cases}) != len(cases):
        raise ValueError("Independent Cases require distinct identities")
    binary = str(args.yai.resolve())
    run = f"portfolio-{time.time_ns()}"
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("x") as evidence:
        print(json.dumps(dict(run_id=run, evidence=str(args.output))), flush=True)
        # Never accept an existing profile: destructive cleanup is confined to
        # the fresh directory allocated by this run, never YAI_HOME from the shell.
        root = Path(tempfile.mkdtemp(prefix="yai-behavioral-portfolio-"))
        home = root / "home"
        env = {**os.environ, "YAI_HOME": str(home)}
        lock = threading.Lock()
        order = 0
        def emit(record):
            nonlocal order
            with lock:
                order += 1
                evidence.write(json.dumps(dict(run_id=run, order=order, **record)) + "\n")
                evidence.flush()
        def cli(*parts):
            command = [binary, *parts, "--json"]
            result = subprocess.run(command, env=env, cwd=ROOT, capture_output=True, text=True, timeout=45)
            emit(dict(command=command, cwd=str(ROOT), yai_home=str(home), exit=result.returncode,
                      stdout=result.stdout, stderr=result.stderr))
            if result.returncode:
                raise RuntimeError("CLI qualification refused; inspect raw evidence")
            return json.loads(result.stdout)
        def call(host, operation, inputs, expected="success"):
            response = host.call(operation, inputs, f"{run}:{time.time_ns()}")
            emit(dict(operation=operation, input=inputs, result=response))
            if response["result_state"] != expected:
                raise AssertionError(f"{operation}: expected {expected}, observed {response['result_state']}")
            return response.get("data")
        started = False
        try:
            emit(dict(manifest_sha256=fingerprint(manifest), material_pre_state="fresh isolated profile",
                      profile=str(home), evidence_class="deterministic_local_product", provider="none"))
            # A failed readiness acknowledgement can still leave a live Host.
            # Stop our fresh profile before deleting it even if start failed.
            started = True
            cli("host", "start")
            host = Host(home)
            call(host, "identity.bootstrap", dict(tenant_id=manifest["tenant"], organization_ref=manifest["organization"]))
            for case in cases:
                call(host, "case.create", dict(case_ref=case["case_ref"], tenant_id=manifest["tenant"]))
                call(host, "participant.role.add", dict(case_ref=case["case_ref"], participant_ref=manifest["participant"], role="operator"))
                call(host, "participant.principal.link", dict(case_ref=case["case_ref"], participant_ref=manifest["participant"], principal_ref="self"))
            before = {c["case_ref"]:call(host, "case.summary", dict(case_ref=c["case_ref"])) for c in cases}
            def advance(case):
                client = Host(home)
                inputs = dict(case_ref=case["case_ref"], participant_ref=manifest["participant"], role=case["role"])
                call(client, "participant.role.add", inputs)
                first = call(client, "case.summary", dict(case_ref=case["case_ref"]))
                call(client, "participant.role.add", inputs)  # exact semantic replay, not a new role
                repeated = call(client, "case.summary", dict(case_ref=case["case_ref"]))
                assert repeated == first, "Repeated role mutation changed canonical state"
                assert first["case"]["generation"] == before[case["case_ref"]]["case"]["generation"] + 1
                call(client, "case.summary", dict(case_ref=case["case_ref"], expected_generation=before[case["case_ref"]]["case"]["generation"]), "stale")
                return first
            with ThreadPoolExecutor(max_workers=len(cases)) as pool:
                after = list(pool.map(advance, cases))
            for case, state in zip(cases, after):
                assert state["case"]["case_ref"] == case["case_ref"]
                roles = {role for p in state["overview"]["participants"] for role in p["roles"]}
                assert case["role"] in roles
                assert not roles.intersection(c["role"] for c in cases if c != case), "Cross-Case role leakage"
                cli("case", "verify", case["case_ref"])
            instance = host.discovery["instance_id"]
            cli("host", "restart")
            reopened = Host(home)
            assert reopened.discovery["instance_id"] != instance
            for case, state in zip(cases, after):
                assert call(reopened, "case.summary", dict(case_ref=case["case_ref"])) == state
                cli("case", "verify", case["case_ref"])
        finally:
            if started:
                cli("host", "stop")
            storage = measure_profile_storage(home) if home.exists() else None
            emit(dict(observation="stopped_profile_storage", storage=storage))
            if not args.retain:
                shutil.rmtree(root)
        emit(dict(result="PASS", dimensions=["CAN_DO", "REFUSES", "RECOVERS", "REMEMBERS", "ISOLATES"],
                  model_behavior="NOT_ASSESSED", cases=len(cases), independent_clients=len(cases),
                  allocated_profile_bytes=storage["allocated_bytes"] if storage else None,
                  storage_posture=storage["posture"] if storage else "not_observed", nested_case=manifest["nested_case"],
                  cleanup="retained_stopped_profile" if args.retain else "removed_after_stop"))
        print(json.dumps(dict(result="PASS", run_id=run, cases=len(cases), profile=str(home) if args.retain else "removed_after_stop", evidence=str(args.output))))


if __name__ == "__main__":
    main()
