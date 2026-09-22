#!/usr/bin/env python3
"""Retain bounded, unedited command evidence; no provider or test implementation."""
import argparse
import hashlib
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

def source_state():
    """Fingerprint dirty sources without retaining their contents or secrets."""
    root = Path(subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True).strip())
    digest = hashlib.sha256()
    for args in (["git", "diff", "--binary", "HEAD"],
                 ["git", "diff", "--cached", "--binary"]):
        data = subprocess.check_output(args, cwd=root)
        digest.update(len(data).to_bytes(8, "big"))
        digest.update(data)
    untracked = subprocess.check_output(["git", "ls-files", "--others", "--exclude-standard", "-z"], cwd=root)
    for name in sorted(untracked.split(b"\0")):
        if not name:
            continue
        path = root / os.fsdecode(name)
        digest.update(name + b"\0")
        if path.is_symlink():
            digest.update(os.fsencode(os.readlink(path)))
        elif path.is_file():
            with path.open("rb") as stream:
                for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                    digest.update(chunk)
        else:
            digest.update(b"unavailable-at-capture")
    return {"head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip(),
            "dirty_source_sha256": digest.hexdigest()}

before = source_state()
started = time.time()
proc = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
after = source_state()
def excerpt(data):
    text = data.decode("utf-8", errors="replace")
    return {"bytes": len(data), "truncated": len(text) > 24000,
            "head": text[:12000], "tail": text[-12000:] if len(text) > 12000 else ""}
record = {"run_id": a.run, "order": a.order, "prestate": a.prestate,
          "provider_mode": a.mode, "cwd": os.getcwd(), "command": command,
          "yai_sha": before["head"], "source_before": before, "source_after": after,
          "source_changed_during_run": before != after,
          "environment": {k: os.environ[k] for k in ("YAI_HOME", "YAI_PROVIDER_RESPONSE_TIMEOUT_SECS", "CARGO_NET_OFFLINE", "CARGO_TARGET_DIR", "RUSTUP_TOOLCHAIN", "TMPDIR", "PATH", "YAI_EXTERNAL_PROVIDER_BASE_URL", "YAI_EXTERNAL_PROVIDER_MODEL", "YAI_CONNECTION_EVIDENCE") if k in os.environ},
          "started_unix": started, "elapsed_seconds": time.time() - started,
          "exit": proc.returncode, "stdout": excerpt(proc.stdout), "stderr": excerpt(proc.stderr)}
with Path(a.output).open("a") as stream:
    stream.write(json.dumps(record, sort_keys=True) + "\n")
print(json.dumps(record, indent=2))
raise SystemExit(proc.returncode)
