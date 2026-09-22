#!/usr/bin/env python3
"""Bounded real-provider transport characterization, not YAI semantic proof.

Records raw responses and provider-reported token usage; never estimates tokens
from text length or calls end-to-end throughput decode throughput.
"""
import argparse
import json
from pathlib import Path
import statistics
import time
import urllib.error
import urllib.parse
import urllib.request


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--endpoint", required=True, help="loopback OpenAI-compatible /v1")
    parser.add_argument("--model", required=True)
    parser.add_argument("--runtime", required=True, help="exact runtime/version")
    parser.add_argument("--weights", required=True, help="weight revision and precision")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--samples", type=int, default=5)
    args = parser.parse_args()
    endpoint = urllib.parse.urlsplit(args.endpoint)
    if endpoint.scheme != "http" or endpoint.hostname not in ("127.0.0.1", "::1") or endpoint.username:
        parser.error("this local benchmark accepts only credential-free loopback HTTP")
    if not 1 <= args.samples <= 100:
        parser.error("samples must be 1..100")
    args.output.mkdir(parents=True, exist_ok=False)
    manifest = dict(runtime=args.runtime, weights=args.weights, model=args.model,
                    endpoint=args.endpoint, samples=args.samples, started_unix=time.time(),
                    purpose="provider transport only; no YAI authority or model-quality claim")
    (args.output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    records = []
    for index in range(args.samples + 1):
        request = dict(model=args.model, stream=False, temperature=0, max_tokens=64,
                       messages=[dict(role="user", content="Briefly explain why a database transaction should be atomic.")])
        started = time.perf_counter()
        code, raw, error = None, "", None
        try:
            wire = urllib.request.Request(args.endpoint.rstrip("/") + "/chat/completions",
                data=json.dumps(request).encode(), headers={"Content-Type": "application/json"})
            with urllib.request.urlopen(wire, timeout=120) as response:
                code, raw = response.status, response.read().decode()
        except urllib.error.HTTPError as exc:
            code, raw, error = exc.code, exc.read().decode(errors="replace"), str(exc)
        except (OSError, ValueError) as exc:
            error = str(exc)
        elapsed = time.perf_counter() - started
        try:
            body = json.loads(raw)
            choices = body.get("choices", [])
            valid = (code == 200 and bool(choices)
                     and isinstance(choices[0].get("message", {}).get("content"), str)
                     and bool(choices[0]["message"]["content"].strip()))
            usage = body.get("usage")
        except (ValueError, AttributeError, TypeError, IndexError):
            valid, usage = False, None
        record = dict(index=index, warmup=index == 0, request=request, http_status=code,
                      response_raw=raw, error=error, valid_chat_response=valid,
                      elapsed_seconds=elapsed, provider_reported_usage=usage)
        records.append(record)
        with (args.output / "samples.jsonl").open("a") as output:
            output.write(json.dumps(record) + "\n")
        print(json.dumps({key: value for key, value in record.items() if key not in ("response_raw", "request")}), flush=True)
        if not valid:
            break
    measured = records[1:]
    durations = [r["elapsed_seconds"] for r in measured if r["valid_chat_response"]]
    passed = len(measured) == args.samples and all(r["valid_chat_response"] for r in records)
    summary = dict(passed=passed, completed_samples=len(durations),
                   median_request_seconds=statistics.median(durations) if durations else None,
                   min_request_seconds=min(durations) if durations else None,
                   max_request_seconds=max(durations) if durations else None,
                   ttft_seconds=None, decode_tokens_per_second=None,
                   notes="Nonstreaming request latency includes prefill/decode/transport; warmup excluded. No decode speed or correctness score inferred.")
    (args.output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary), flush=True)
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
