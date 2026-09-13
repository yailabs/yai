#!/usr/bin/env python3
"""One fresh synthetic envelope control per invocation, not Golden qualification.

Pad a tiny valid JSON request with legal JSON whitespace to the exact failing
body length. This separates raw HTTP body acceptance from model/token density.
No Case content, tools, fallback or retry. Never infer a tokenizer from bytes.
"""
import argparse
import hashlib
import http.client
import json
import subprocess
import time
from pathlib import Path
from urllib.parse import urlsplit


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--endpoint", required=True)
    p.add_argument("--model", required=True)
    p.add_argument("--reference-body", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    p.add_argument("--density", choices=("padding", "text"), default="padding")
    a = p.parse_args()
    target = urlsplit(a.endpoint)
    assert target.scheme == "http" and target.hostname and not any((target.username, target.password, target.query, target.fragment, target.path.strip("/")))
    value = {"model": a.model, "stream": False, "max_tokens": 1,
             "messages": [{"role": "user", "content": "Synthetic capacity diagnostic. Reply OK."}]}
    content = json.dumps(value, separators=(",", ":")).encode()
    total = a.reference_body.stat().st_size
    assert len(content) < total <= 1024 * 1024
    if a.density == "text":
        extra = total - len(content)
        value["messages"][0]["content"] += " x" * (extra // 2) + " " * (extra % 2)
        content = json.dumps(value, separators=(",", ":")).encode()
    body = content + b" " * (total - len(content))
    a.output.mkdir(exist_ok=False)
    (a.output / "request-body.json").write_bytes(body)
    record = dict(run_id=a.output.name, order=1, proof_class="contract", diagnostic_only=True, provider_mode="external_yvex",
                  yai_sha=subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
                  prestate="fresh synthetic input, no Case work, no tools, no continuation",
                  endpoint=a.endpoint, model=a.model, request_body_bytes=len(body),
                  json_bytes_before_padding=len(content), json_whitespace_bytes=len(body)-len(content),
                  density=a.density,
                  body_sha256=hashlib.sha256(body).hexdigest(), max_tokens=1,
                  reference_body_sha256=hashlib.sha256(a.reference_body.read_bytes()).hexdigest(),
                  fallback_count=0, retry_count=0)
    connection = http.client.HTTPConnection(target.hostname, target.port or 80, timeout=30)
    started = time.time()
    try:
        connection.request("POST", "/v1/chat/completions", body,
                           {"Content-Type": "application/json", "Accept": "application/json", "Connection": "close"})
        response = connection.getresponse()
        result = response.read(1048577)
        assert len(result) <= 1048576
        record.update(status=response.status, headers=response.getheaders(), body=result.decode())
    except Exception as error:
        record["failure"] = str(error)
        record["delivery"] = "unknown; no retry"
    finally:
        connection.close()
    record["elapsed_seconds"] = time.time() - started
    (a.output / "result.json").write_text(json.dumps(record, indent=2) + "\n")
    print(json.dumps(record, indent=2))
    raise SystemExit(0 if record.get("status") == 200 else 1)


if __name__ == "__main__":
    main()
