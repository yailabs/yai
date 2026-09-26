#!/usr/bin/env python3
"""Qualify current CLI/Host discovery when a resident executable is replaced."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


def command(binary: Path, home: Path, *args: str) -> dict:
    result = subprocess.run(
        [str(binary), "host", *args, "--json"],
        env={**os.environ, "YAI_HOME": str(home)},
        capture_output=True,
        text=True,
        timeout=30,
        check=True,
    )
    value = json.loads(result.stdout)
    assert value["status"] == "ok", value
    return value["data"]["value"]


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: host-executable-posture.py /absolute/path/to/yai")
    binary = Path(sys.argv[1]).resolve(strict=True)
    with tempfile.TemporaryDirectory(prefix="yai-host-executable-") as root:
        root = Path(root)
        home = root / "home"
        home.mkdir()
        copied = root / "yai-host-copy"
        shutil.copy2(binary, copied)
        started = command(copied, home, "start")
        try:
            linked = command(binary, home, "status")
            assert linked["state"] == "running"
            assert linked["pid"] == started["pid"]
            assert linked["instance_id"] == started["instance_id"]
            assert linked["executable_posture"] == "linked", linked

            copied.unlink()
            replaced = command(binary, home, "status")
            assert replaced["pid"] == started["pid"]
            assert replaced["instance_id"] == started["instance_id"]
            assert replaced["executable_posture"] == "replaced_on_disk", replaced
            discovered = command(binary, home, "start")
            assert discovered["pid"] == started["pid"]
            assert discovered["executable_posture"] == "replaced_on_disk"
            print(json.dumps({
                "result": "PASS", "pid": started["pid"],
                "instance_id": started["instance_id"],
                "linked": linked["executable_posture"],
                "replaced": replaced["executable_posture"],
                "singleton_discovery_preserved": True,
            }, sort_keys=True))
        finally:
            stopped = command(binary, home, "stop")
            assert stopped["state"] == "stopped"


if __name__ == "__main__":
    main()
