#!/usr/bin/env python3
"""Retain bounded, unedited command evidence; no provider or test implementation."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time

p = argparse.ArgumentParser()
p.add_argument("--output", required=True)
p.add_argument("--run", required=True)
p.add_argument("--order", required=True, type=int)
p.add_argument("--prestate", required=True)
p.add_argument("--mode", required=True)
p.add_argument("command", nargs=argparse.REMAINDER)
a = p.parse_args()
command = a.command[1:] if a.command[:1] == ["--"] else a.command
started = time.time()
proc = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
def excerpt(data):
    text = data.decode("utf-8", errors="replace")
    return {"bytes": len(data), "truncated": len(text) > 24000,
            "head": text[:12000], "tail": text[-12000:] if len(text) > 12000 else ""}
record = {"run_id": a.run, "order": a.order, "prestate": a.prestate,
          "provider_mode": a.mode, "cwd": os.getcwd(), "command": command,
          "yai_sha": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
          "environment": {k: os.environ[k] for k in ("YAI_HOME", "YAI_PROVIDER_RESPONSE_TIMEOUT_SECS", "CARGO_NET_OFFLINE", "TMPDIR", "PATH", "YAI_EXTERNAL_PROVIDER_BASE_URL", "YAI_EXTERNAL_PROVIDER_MODEL", "YAI_CONNECTION_EVIDENCE") if k in os.environ},
          "started_unix": started, "elapsed_seconds": time.time() - started,
          "exit": proc.returncode, "stdout": excerpt(proc.stdout), "stderr": excerpt(proc.stderr)}
with Path(a.output).open("a") as stream:
    stream.write(json.dumps(record, sort_keys=True) + "\n")
print(json.dumps(record, indent=2))
raise SystemExit(proc.returncode)
