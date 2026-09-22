#!/usr/bin/env python3
"""Consume one real CLI-admitted Operation through Host Application transport.

The caller supplies a disposable qualification home with an admitted operation
and no PREPARE. This is test orchestration, never an Application CLI adapter.
"""
import argparse
import json
import os
from pathlib import Path
import socket
import subprocess
import time


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--home", required=True)
    parser.add_argument("--case", required=True)
    parser.add_argument("--participant", required=True)
    parser.add_argument("--operation", required=True)
    parser.add_argument("--file", required=True)
    options = parser.parse_args()
    home = Path(options.home)
    target = Path(options.file)
    assert not target.exists(), "fixture must stop before the effect"
    binary = Path("./yai").resolve()
    env = dict(os.environ, YAI_HOME=str(home))
    order = 0

    def lifecycle(action):
        result = subprocess.run([str(binary), "host", action, "--json"],
            env=env, capture_output=True, text=True, timeout=30)
        print(json.dumps({"run_id":home.parent.name,"host":action,"exit":result.returncode,
            "stdout":result.stdout,"stderr":result.stderr}), flush=True)
        assert result.returncode == 0, result.stderr

    def call(operation, data, lose=False, expected="success"):
        nonlocal order
        order += 1
        discovery = json.loads((home / "run/host/discovery.json").read_text())
        request = {"protocol":"yai.studio.application.v1", "operation_ref":operation,
            "correlation_ref":f"effect-host:{order}", "input":data}
        with socket.socket(socket.AF_UNIX) as connection:
            connection.settimeout(20)
            connection.connect(discovery["endpoint"])
            with connection.makefile("rb") as reader:
                connection.sendall((json.dumps({"kind":"handshake","protocol":discovery["protocol"],
                    "client_id":f"qualification:effect:{os.getpid()}:{order}","client_kind":"qualification",
                    "pid":os.getpid(),"yai_home_identity":discovery["yai_home_identity"]})+"\n").encode())
                assert json.loads(reader.readline())["kind"] == "handshake"
                connection.sendall((json.dumps({"kind":"application_request","request":request})+"\n").encode())
                if lose:
                    print(json.dumps({"run_id":home.parent.name,"order":order,
                        "request":request,"action":"disconnect_without_acknowledgement"}), flush=True)
                    return None
                response = json.loads(reader.readline())
                print(json.dumps({"run_id":home.parent.name,"order":order,
                    "request":request,"response":response}), flush=True)
                assert response["kind"] == "application_response", response
                result = response["result"]
                if expected is not None:
                    assert result["result_state"] == expected, result
                    return result.get("data")
                return result

    started = False
    try:
        lifecycle("start")
        started = True
        base = {"case_ref":options.case,"participant_ref":options.participant}
        observe = dict(base, execution={"domain":"controlled_effect","operation_ref":options.operation})
        generation = call("execution.get", observe)["observed_generation"]
        submit = dict(base, operation_ref=options.operation, expected_generation=generation)
        call("effect.submit", dict(submit, participant_ref="participant:hidden"), expected="unauthorized")
        assert not target.exists()
        call("effect.submit", submit, lose=True)
        deadline = time.monotonic() + 15
        final = None
        while time.monotonic() < deadline:
            result = call("execution.get", observe, expected=None)
            if result["result_state"] == "success":
                value = result["data"]
                if (value.get("progress") or {}).get("status") == "finalized":
                    final = value
                    break
            else:
                assert result["result_state"] == "stale", result
            time.sleep(0.05)
        assert final is not None, "Host did not complete the disconnected client's admitted effect"
        assert final["progress"]["outcome"] == "applied", final
        assert target.read_bytes() == b"hello from controlled YAI\n"
        inode = target.stat().st_ino
        assert call("effect.submit", submit) == final
        old_instance = json.loads((home / "run/host/discovery.json").read_text())
        lifecycle("restart")
        new_instance = json.loads((home / "run/host/discovery.json").read_text())
        assert old_instance != new_instance
        assert call("execution.get", observe) == final
        assert call("effect.submit", submit) == final
        assert target.stat().st_ino == inode, "retry replaced the file again"
        assert call("execution.get", observe)["observed_generation"] == final["observed_generation"]
        print("PASS: Host effect survives lost acknowledgement and restart; same receipt, generation and file inode", flush=True)
    finally:
        if started:
            lifecycle("stop")


if __name__ == "__main__":
    main()
