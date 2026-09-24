#!/usr/bin/env python3
"""Opt-in, non-production Laya characterization over qualified YAI capture files.

Generate captures with YAI_FAST_SEARCH_CAPTURE_DIR=/tmp/... and the independent
semantic-working-state sufficiency suite. Install Laya outside Git and pass an
exact local checkpoint directory. This script never changes a Case or scores
hidden/unqualified candidates.
"""

import argparse
import hashlib
import json
from pathlib import Path
import resource
import statistics
import time


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def question(capture):
    return {"navigation": {
        "type": "choice",
        "instructions": (
            "For the current memory question, choose the next exact evidence group "
            "to inspect. Choose the deterministic path if no single group is "
            "sufficient. This choice is not a fact or authorization."
        ),
        "criteria": {
            item["id"]: item["description"]
            for item in capture["candidates"]
        },
    }}


def answer(model, capture):
    started = time.perf_counter()
    result = model.system_one(capture["query"], question(capture))
    elapsed_ms = (time.perf_counter() - started) * 1000
    return result["answers"]["navigation"], elapsed_ms


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--captures", type=Path, required=True)
    parser.add_argument("--model", type=Path, required=True)
    parser.add_argument("--expected-sha256", required=True)
    parser.add_argument("--model-revision", required=True)
    args = parser.parse_args()
    captures = [json.loads(path.read_text()) for path in sorted(args.captures.glob("*.json"))]
    if not captures:
        raise SystemExit("No YAI Fast Search captures found")
    weights = args.model / "model.safetensors"
    observed_hash = sha256(weights)
    if observed_hash != args.expected_sha256:
        raise SystemExit(f"Checkpoint mismatch: {observed_hash}")

    # Importing torch/Laya here keeps the repository's production path clean.
    from laya import Agent, __version__ as laya_version
    import torch

    started = time.perf_counter()
    model = Agent(str(args.model), device="cpu")
    load_ms = (time.perf_counter() - started) * 1000
    results = []
    for capture in captures:
        try:
            prediction, decision_ms = answer(model, capture)
            selected = next(item for item in capture["candidates"]
                            if item["id"] == prediction["choice"])
            results.append({
                "task": capture["task"],
                "working_state_id": capture["working_state_id"],
                "navigation_id": capture["navigation_id"],
                "candidate_count": capture["candidate_count"],
                "omitted_optional": capture["omitted_optional"],
                "preparation_ms": capture["preparation_ms"],
                "model_decision_ms": round(decision_ms, 3),
                "selected_kind": selected["kind"],
                "selected_required_hits": selected["required_hits"],
                "effective_required_hits": (capture["standard_required_hits"]
                                            if selected["kind"] == "deterministic_path"
                                            else selected["required_hits"]),
                "selected_forbidden_hits": selected["forbidden_hits"],
                "selected_distractor_hits": selected["distractor_hits"],
                "standard_required_hits": capture["standard_required_hits"],
                "standard_forbidden_hits": capture["standard_forbidden_hits"],
                "relative_scores": prediction["probabilities"],
                "uncalibrated_reported_confidence": prediction["confidence"],
            })
        except Exception as error:
            results.append({"task": capture["task"], "candidate_count": capture["candidate_count"],
                            "error": type(error).__name__ + ": " + str(error)})

    repeat_capture = next(capture for capture in captures if capture["candidate_count"] >= 2)
    warm_ms = []
    for _ in range(5):
        _, elapsed_ms = answer(model, repeat_capture)
        warm_ms.append(elapsed_ms)
    batch_ms = None
    try:
        started = time.perf_counter()
        model.predict_batch([repeat_capture["query"]] * 8, question(repeat_capture))
        batch_ms = round((time.perf_counter() - started) * 1000, 3)
    except Exception as error:
        batch_ms = type(error).__name__ + ": " + str(error)
    candidate_pressure = []
    descriptions = [item["description"] for item in repeat_capture["candidates"]]
    for count in (2, 8, 16, 32):
        criteria = {f"synthetic:{index}": descriptions[index % len(descriptions)]
                    for index in range(count)}
        try:
            started = time.perf_counter()
            model.system_one(repeat_capture["query"], {"navigation": {
                "type": "choice", "instructions": "Select an exact evidence group or fallback.",
                "criteria": criteria,
            }})
            candidate_pressure.append({"count": count, "ms": round((time.perf_counter() - started) * 1000, 3)})
        except Exception as error:
            candidate_pressure.append({"count": count, "error": type(error).__name__ + ": " + str(error)})
    print(json.dumps({
        "schema": "yai.fast_search.laya_reference.v1",
        "model_revision": args.model_revision,
        "checkpoint_sha256": observed_hash,
        "checkpoint_bytes": weights.stat().st_size,
        "laya_package_version": laya_version,
        "device": "cpu",
        "gpu_available": torch.cuda.is_available(),
        "load_ms": round(load_ms, 3),
        "peak_rss_kib": resource.getrusage(resource.RUSAGE_SELF).ru_maxrss,
        "warm_median_ms": round(statistics.median(warm_ms), 3),
        "batch_eight_ms": batch_ms,
        "synthetic_candidate_pressure": candidate_pressure,
        "expensive_primary_model_calls_avoided": None,
        "navigation_steps": None,
        "results": results,
        "interpretation": (
            "Selection is optional and non-authoritative; scores are relative, not "
            "calibrated correctness probabilities. Standard W remains available."
        ),
    }, indent=2))


if __name__ == "__main__":
    main()
