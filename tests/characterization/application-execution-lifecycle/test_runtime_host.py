#!/usr/bin/env python3
"""Real Host/RuntimeInstance lifecycle, disposable profile, no provider calls."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=Path("./yai"))
    binary = parser.parse_args().binary.resolve()
    root = Path(tempfile.mkdtemp(prefix="yai-host-runtime-"))
    env = dict(os.environ, YAI_HOME=str(root / "home"))

    def call(*args):
        result = subprocess.run([str(binary), *args], env=env, text=True,
                                capture_output=True, timeout=20)
        print(json.dumps({"argv": list(args), "exit": result.returncode,
                          "stdout": result.stdout, "stderr": result.stderr}), flush=True)
        assert result.returncode == 0, result.stderr
        return result.stdout

    def status():
        return json.loads(call("host", "status", "--json"))["data"]["value"]

    def await_posture(expected):
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            value = status()
            if value["runtime_supervision"] == expected:
                return value
            time.sleep(.1)
        raise AssertionError(f"runtime supervision did not become {expected}: {value}")

    success = False
    external = None
    try:
        call("host", "start", "--json")
        waiting = await_posture("waiting_for_identity")
        call("security", "bootstrap-local", "--tenant", "tenant:host-test",
             "--organization", "organization:host-test")
        running = await_posture("supervised_running")
        assert running["instance_id"] == waiting["instance_id"]
        # Each CLI connection has already disconnected. Host and scheduler live.
        runtime = call("runtime", "status")
        assert f"pid: {running['pid']}\n" in runtime
        assert "state: running" in runtime.lower()
        call("host", "start", "--json")
        assert status()["instance_id"] == running["instance_id"]
        call("host", "stop", "--json")
        assert status()["state"] != "running"
        assert "state: stopped" in call("runtime", "status").lower()
        call("host", "start", "--json")
        restarted = await_posture("supervised_running")
        assert restarted["instance_id"] != running["instance_id"]
        assert f"pid: {restarted['pid']}\n" in call("runtime", "status")
        call("host", "stop", "--json")
        assert "state: stopped" in call("runtime", "status").lower()
        # An independent existing scheduler retains its lease and lifecycle.
        with (root / "external-runtime.log").open("w") as log:
            external = subprocess.Popen([str(binary), "runtime", "serve"],
                                        env=env, stdout=log, stderr=log)
            deadline = time.monotonic() + 10
            while time.monotonic() < deadline:
                runtime = call("runtime", "status")
                if f"pid: {external.pid}\n" in runtime and "state: running" in runtime.lower():
                    break
                assert external.poll() is None
                time.sleep(.1)
            else:
                raise AssertionError("external scheduler did not start")
            call("host", "start", "--json")
            attached = await_posture("attached_existing_runtime")
            assert attached["pid"] != external.pid
            call("host", "stop", "--json")
            assert external.poll() is None
            assert f"pid: {external.pid}\n" in call("runtime", "status")
            call("runtime", "stop")
            assert external.wait(timeout=10) == 0
        success = True
        print("PASS: resident scheduler, disconnected clients, singleton, drain, restart, external-owner attachment")
    finally:
        if not success:
            subprocess.run([str(binary), "host", "stop", "--json"], env=env,
                           capture_output=True, timeout=20)
            if external is not None and external.poll() is None:
                subprocess.run([str(binary), "runtime", "stop"], env=env,
                               capture_output=True, timeout=20)
                external.wait(timeout=10)
            print(f"Failure profile retained: {root}")
        else:
            shutil.rmtree(root)


if __name__ == "__main__":
    main()
