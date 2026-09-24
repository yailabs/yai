#!/usr/bin/env python3
"""Bounded behavioral evaluations over the real typed local Host protocol.

This is a qualification client, never a Case owner or command interpreter.
Profiles supply exact product inputs; results retain semantic assertions separately
from human/model assessment. No shell, CLI parsing or private store access.
"""
import argparse
import hashlib
import itertools
import json
import os
from pathlib import Path
import socket
import time

SCHEMA = "yai.behavioral_corpus.v1"
DIMENSIONS = {"KNOWS", "SEES", "RECALLS", "REMEMBERS", "REASONS", "CAN_DO",
              "REFUSES", "RECOVERS", "ISOLATES", "PERFORMS"}
MAX_FRAME = 16 * 1024 * 1024


class PendingObservation(Exception):
    """A bounded observation ended without proving completion or failure."""


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def fingerprint(value):
    return hashlib.sha256(canonical(value).encode()).hexdigest()


def pointer(value, path):
    if path == "":
        return value
    if not path.startswith("/"):
        raise ValueError("Expected JSON pointer")
    for key in path[1:].split("/"):
        key = key.replace("~1", "/").replace("~0", "~")
        value = value[int(key)] if isinstance(value, list) else value[key]
    return value


def resolve(value, bindings):
    if isinstance(value, dict):
        if set(value) == {"$ref"}:
            return pointer(bindings, value["$ref"])
        return {key: resolve(item, bindings) for key, item in value.items()}
    if isinstance(value, list):
        return [resolve(item, bindings) for item in value]
    return value


def variants(spec, limit):
    if spec.get("schema") != SCHEMA:
        raise ValueError("Unsupported behavioral corpus schema")
    seen = set()
    expanded = []
    for test in spec["evaluations"]:
        identity = test["id"]
        if identity in seen or not identity:
            raise ValueError("Duplicate/empty evaluation identity")
        seen.add(identity)
        if not set(test["dimensions"]) <= DIMENSIONS or not test["dimensions"]:
            raise ValueError("Unknown/empty evaluation dimensions")
        for field in ("meaning", "preconditions", "required_refs", "allowed_actions",
                      "forbidden_actions", "expected_consequences", "repeatability", "mode"):
            if field not in test:
                raise ValueError(f"Missing {identity}.{field}")
        axes = test.get("variants", {})
        keys = sorted(axes)
        if any(not isinstance(axes[key], list) or not axes[key] for key in keys):
            raise ValueError("Variant axes must be nonempty arrays")
        if any(len({canonical(value) for value in axes[key]}) != len(axes[key]) for key in keys):
            raise ValueError("Duplicate variant value would repeat a logical evaluation")
        count = 1
        for key in keys:
            count *= len(axes[key])
        if len(expanded) + count > limit:
            raise ValueError("Variant bound exceeded; select a bounded suite explicitly")
        for values in itertools.product(*(axes[key] for key in keys)):
            variant = dict(zip(keys, values))
            expanded.append((test, variant, f"{identity}:{fingerprint(variant)[:16]}"))
    return expanded


def overlay(base, additions):
    """Explicit identity-keyed Case specialization; preserve the generic seed."""
    if additions.get("schema") != SCHEMA:
        raise ValueError("Unsupported overlay schema")
    updates = additions["evaluations"]
    identities = [item["id"] for item in updates]
    if len(set(identities)) != len(identities):
        raise ValueError("Duplicate overlay identity")
    merged = {item["id"]: item for item in base["evaluations"]}
    if len(merged) != len(base["evaluations"]):
        raise ValueError("Duplicate base identity")
    for item in updates:
        merged[item["id"]] = {**merged.get(item["id"], {}), **item}
    return {"schema": SCHEMA, "evaluations": list(merged.values())}


class Host:
    def __init__(self, home, timeout=30):
        self.home = Path(home).resolve()
        self.discovery = json.loads((self.home / "run/host/discovery.json").read_text())
        if Path(self.discovery["yai_home"]).resolve() != self.home:
            raise ValueError("Host profile mismatch")
        self.timeout = timeout

    def call(self, operation, inputs, correlation):
        discovery = self.discovery
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as connection:
            connection.settimeout(self.timeout)
            connection.connect(discovery["endpoint"])
            with connection.makefile("rwb") as stream:
                def send(value):
                    body = (canonical(value) + "\n").encode()
                    if len(body) > MAX_FRAME:
                        raise ValueError("Request frame bound exceeded")
                    stream.write(body)
                    stream.flush()

                def receive():
                    body = stream.readline(MAX_FRAME + 1)
                    if not body.endswith(b"\n") or len(body) > MAX_FRAME:
                        raise ValueError("Truncated/oversized Host frame")
                    return json.loads(body)

                send(dict(kind="handshake", protocol=discovery["protocol"],
                          client_id=f"behavioral:{os.getpid()}:{correlation}",
                          client_kind="qualification", pid=os.getpid(),
                          yai_home_identity=discovery["yai_home_identity"]))
                hello = receive()
                if (hello.get("kind") != "handshake"
                        or hello.get("protocol") != discovery["protocol"]
                        or hello.get("yai_home_identity") != discovery["yai_home_identity"]
                        or hello.get("host_instance_id") != discovery["instance_id"]):
                    raise ValueError("Host handshake/profile/instance mismatch; rediscover explicitly")
                send(dict(kind="application_request", request=dict(
                    protocol="yai.studio.application.v1", operation_ref=operation,
                    correlation_ref=correlation, input=inputs)))
                result = receive()
                if result.get("kind") != "application_response":
                    raise ValueError(f"Unexpected Host frame: {result.get('kind')}")
                return result["result"]


def exact(actual, expected):
    """JSON structural equality without Python's bool/integer equivalence."""
    if type(actual) is not type(expected):
        return False
    if isinstance(actual, dict):
        return actual.keys() == expected.keys() and all(exact(actual[k], v) for k, v in expected.items())
    if isinstance(actual, list):
        return len(actual) == len(expected) and all(exact(a, b) for a, b in zip(actual, expected))
    return actual == expected


def assert_result(assertion, bindings):
    actual = pointer(bindings, assertion["path"])
    expected = resolve(assertion["value"], bindings)
    op = assertion["op"]
    if op in {"equal", "not_equal"}:
        passed = exact(actual, expected) if op == "equal" else not exact(actual, expected)
    elif op == "contains":
        passed = any(exact(item, expected) for item in actual) if isinstance(actual, list) else expected in actual
    elif op == "excludes":
        passed = not any(exact(item, expected) for item in actual) if isinstance(actual, list) else expected not in actual
    elif op == "some":
        if not isinstance(actual, list) or not isinstance(expected, dict) or not expected:
            raise ValueError("some requires array and nonempty exact field assertions")
        def matches(item):
            if not isinstance(item, dict):
                return False
            try:
                for key, value in expected.items():
                    observed = pointer(item, key) if key.startswith("/") else item[key]
                    if not exact(observed, value):
                        return False
                return True
            except (KeyError, IndexError, TypeError, ValueError):
                return False
        passed = any(matches(item) for item in actual)
    else:
        raise ValueError(f"Unsupported assertion: {op}")
    if not passed:
        raise AssertionError(f"{assertion['path']} {op} failed")


def evaluate(test, variant, profile, host, impacts, allow_mutations, emit):
    bindings = dict(profile=profile, variant=variant, results={})
    steps = test.get("steps", [])
    if not steps:
        return "NOT_RUN"
    if not test.get("assertions"):
        raise ValueError("Executable evaluation needs semantic assertions")
    # Validate the complete authored sequence before the first possible mutation.
    # A malformed later step must not leave an otherwise avoidable partial run.
    step_ids = [step["id"] for step in steps]
    if any(not isinstance(identity, str) or not identity for identity in step_ids) or len(set(step_ids)) != len(step_ids):
        raise ValueError("Duplicate/empty step identity")
    for step in steps:
        operation = step["operation"]
        if operation not in impacts or operation not in test["allowed_actions"]:
            raise ValueError("Operation absent from current catalog/evaluation allowlist")
        if operation in test["forbidden_actions"]:
            raise ValueError("Evaluation attempts a forbidden action")
        if impacts[operation] not in {"read", "derived_computation"} and not allow_mutations:
            raise ValueError("Mutation requires explicit --allow-mutations and selected profile")
        observe = step.get("observe")
        if observe is not None:
            if impacts[operation] != "read":
                raise ValueError("Repeated observation requires an actual read operation")
            if not isinstance(observe, dict) or set(observe) != {"path", "while", "max_observations", "interval_ms"}:
                raise ValueError("Observation needs explicit bounds and pending postures")
            if not isinstance(observe["path"], str) or not observe["path"].startswith("/"):
                raise ValueError("Observation requires an exact response pointer")
            if not isinstance(observe["while"], list) or not observe["while"]:
                raise ValueError("Observation requires explicit pending postures")
            if type(observe["max_observations"]) is not int or not 1 <= observe["max_observations"] <= 600:
                raise ValueError("Observation count must be bounded to 1..600")
            if type(observe["interval_ms"]) is not int or not 0 <= observe["interval_ms"] <= 1000:
                raise ValueError("Observation interval must be bounded to 0..1000 ms")
    for assertion in [*test["assertions"], *(a for step in steps for a in step.get("assertions", []))]:
        if assertion.get("op") not in {"equal", "not_equal", "contains", "excludes", "some"} or "value" not in assertion or not isinstance(assertion.get("path"), str):
            raise ValueError("Malformed semantic assertion")
    for index, step in enumerate(steps):
        operation = step["operation"]
        inputs = resolve(step["input"], bindings)
        observe = step.get("observe")
        for observation in range(observe["max_observations"] if observe else 1):
            started = time.monotonic()
            # Never retry on transport loss, including during read observation.
            # An effectful exact retry must be a separate authored step.
            result = host.call(operation, inputs, f"behavioral:{time.time_ns()}:{index}:{observation}")
            emit(dict(step=index, observation=observation, operation=operation, input=inputs, result=result,
                      elapsed_ms=(time.monotonic() - started) * 1000))
            if not observe or result.get("result_state") != "success" or not any(
                exact(pointer(result, observe["path"]), pending) for pending in observe["while"]):
                break
            if observation + 1 == observe["max_observations"]:
                raise PendingObservation(f"{step['id']}: observation bound exhausted; no redispatch and no execution verdict")
            time.sleep(observe["interval_ms"] / 1000)
        bindings["results"][step["id"]] = result
        # Preconditions can depend on a prior authoritative read. Refuse before
        # subsequent steps rather than discovering a stale pre-state after effects.
        for assertion in step.get("assertions", []):
            assert_result(assertion, bindings)
    for assertion in test["assertions"]:
        assert_result(assertion, bindings)
    return "PASS"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("corpus", type=Path)
    parser.add_argument("--profile", type=Path)
    parser.add_argument("--overlay", type=Path, action="append", default=[])
    parser.add_argument("--home", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--list", action="store_true")
    parser.add_argument("--limit", type=int, default=64)
    parser.add_argument("--allow-mutations", action="store_true")
    args = parser.parse_args()
    spec = json.loads(args.corpus.read_text())
    for path in args.overlay:
        spec = overlay(spec, json.loads(path.read_text()))
    selected = variants(spec, args.limit)
    if args.list:
        for test, variant, identity in selected:
            print(canonical(dict(id=identity, dimensions=test["dimensions"], variant=variant,
                                 mode=test["mode"], executable=bool(test.get("steps")))))
        return
    if not (args.profile and args.home and args.output):
        parser.error("Execution needs explicit --profile, --home and --output")
    profile = json.loads(args.profile.read_text())
    host = Host(args.home)
    catalog = host.call("application.capabilities", {}, "behavioral:catalog")
    if catalog["result_state"] != "success":
        raise ValueError("Capability discovery refused")
    impacts = {op["operation_id"]: op["impact"] for op in catalog["data"]["operations"]}
    run = f"behavioral-{time.time_ns()}"
    failed = False
    pending = False
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("x") as evidence:
        order = 0
        def emit(record):
            nonlocal order
            order += 1
            evidence.write(canonical(dict(run_id=run, order=order, **record)) + "\n")
            evidence.flush()
        emit(dict(corpus_sha256=fingerprint(spec), profile_sha256=fingerprint(profile),
                  host_instance=host.discovery["instance_id"], catalog=catalog,
                  evidence_class="behavioral", cwd=os.getcwd()))
        for test, variant, identity in selected:
            try:
                posture = evaluate(test, variant, profile, host, impacts,
                                   args.allow_mutations, lambda r: emit(dict(evaluation=identity, **r)))
                emit(dict(evaluation=identity, dimensions=test["dimensions"], result=posture,
                          language_quality="NOT_ASSESSED"))
            except PendingObservation as error:
                pending = True
                emit(dict(evaluation=identity, result="INCOMPLETE", error=str(error), language_quality="NOT_ASSESSED"))
            except (ValueError, KeyError, AssertionError, OSError) as error:
                failed = True
                emit(dict(evaluation=identity, result="FAIL", error=str(error)))
        print(canonical(dict(run_id=run, result="FAIL" if failed else "INCOMPLETE" if pending else "EXECUTED",
                             evaluations=len(selected), evidence=str(args.output))))
    raise SystemExit(1 if failed else 2 if pending else 0)


if __name__ == "__main__":
    main()
