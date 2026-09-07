#!/usr/bin/env python3
"""Repository validation metadata, Make graph and exact Rust-test partitioning.

tests/classification.tsv is the sole classification authority. This tool does
not implement tests, providers, or a persistent test-result cache.
"""
import argparse
import csv
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
CATALOG = ROOT / "tests/classification.tsv"
CLASSES = {"unit", "component", "contract", "product", "recovery", "endurance",
           "external", "manual", "historical"}
FIELDS = "id kind path selector proof_class evidence_posture provider_mode network mutation cadence entrypoint reachability property".split()
MANIFESTS = {"engine": "engine/Cargo.toml", "cli": "cmd/yai/Cargo.toml"}
BASELINE = "6e332851b5066cbb1da25f816b8db31b74580acb"


def read_catalog(path=CATALOG):
    with path.open(newline="") as stream:
        reader = csv.DictReader(stream, delimiter="\t")
        if reader.fieldnames != FIELDS:
            raise ValueError("classification columns differ from the executable contract")
        rows = list(reader)
    ids = set()
    for row in rows:
        if any(not row.get(field) for field in FIELDS) or None in row:
            raise ValueError(f"incomplete classification: {row}")
        if row["id"] in ids:
            raise ValueError(f"duplicate validation identity: {row['id']}")
        ids.add(row["id"])
        if row["proof_class"] not in CLASSES:
            raise ValueError(f"unknown proof class: {row['id']}")
        if row["kind"] not in {"make", "rust", "manual", "support", "historical"}:
            raise ValueError(f"unknown executor: {row['id']}")
        if row["provider_mode"] not in {"no_provider", "loopback_fixture", "external_yvex", "external_other"}:
            raise ValueError(f"unknown provider mode: {row['id']}")
        if row["cadence"] not in {"fast", "publication", "golden", "endurance", "external", "manual", "support", "historical"}:
            raise ValueError(f"unknown cadence: {row['id']}")
        if row["evidence_posture"] not in {"regression", "characterization", "qualification", "historical", "support"}:
            raise ValueError(f"unknown evidence posture: {row['id']}")
        if row["network"] not in {"none", "local_ipc", "loopback", "external"}:
            raise ValueError(f"unknown network posture: {row['id']}")
        external = row["provider_mode"].startswith("external_")
        if external != (row["network"] == "external"):
            raise ValueError(f"contradictory external dependency: {row['id']}")
        if external and row["cadence"] not in {"external", "manual"}:
            raise ValueError(f"external provider in local gate: {row['id']}")
        if row["proof_class"] == "external" and not external:
            raise ValueError(f"fixture cannot qualify external interoperability: {row['id']}")
        if row["provider_mode"] == "loopback_fixture" and row["network"] != "loopback":
            raise ValueError(f"fixture transport must be explicit: {row['id']}")
        for source in row["path"].split(";"):
            if not (ROOT / source).is_file():
                raise ValueError(f"missing classified source: {source}")
    return rows


def lane_rows(rows, lane):
    if lane == "golden-local":
        return [r for r in rows if r["cadence"] == "golden"]
    if lane == "characterization":
        return [r for r in rows if r["evidence_posture"] == "characterization"
                and r["cadence"] in {"fast", "publication"}]
    if lane == "fast":
        return [r for r in rows if r["cadence"] == "fast"]
    if lane in {"local", "release"}:
        return [r for r in rows if r["cadence"] in {"fast", "publication"}
                and (lane == "release" or r["proof_class"] != "endurance")]
    return [r for r in rows if r["proof_class"] == lane
            and r["kind"] not in {"support", "historical"}]


def make_graph(rows):
    for lane in sorted(CLASSES - {"manual", "historical"} | {"fast", "local", "release", "characterization", "golden-local"}):
        # Order is stable, shared leaves execute once within a Make invocation.
        deps = sorted({r["entrypoint"] for r in lane_rows(rows, lane)
                       if r["kind"] in {"make", "rust"}})
        print(f"VALIDATION_{lane.upper().replace('-', '_')} := {' '.join(deps)}")
    for entry in sorted({r["entrypoint"] for r in rows if r['kind']=='rust' and r["entrypoint"].startswith("test-rust-")}):
        print(f".PHONY: {entry}\n{entry}: validation-rust-build")
        print(f"\t@python3 tools/validation/topology.py rust --entry {entry}")
    # Validation builds use installed dependencies; never fetch crates on a
    # supposedly local lane. Ordinary `make build` can still acquire dependencies.
    local = sorted({r['entrypoint'] for r in lane_rows(rows, 'release') + lane_rows(rows, 'golden-local')})
    print(f"{' '.join(local)}: export CARGO_NET_OFFLINE = true")


def cargo_env():
    return dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "target"), CARGO_NET_OFFLINE="true")


def binaries():
    result = {}
    for suite, manifest in MANIFESTS.items():
        command = ["cargo", "test", "--manifest-path", manifest, "--no-run", "--message-format=json"]
        proc = subprocess.run(command, cwd=ROOT, env=cargo_env(), text=True, stdout=subprocess.PIPE)
        if proc.returncode:
            raise ValueError(f"Rust test build failed: {manifest}, exit={proc.returncode}")
        found = []
        for line in proc.stdout.splitlines():
            item = json.loads(line)
            if item.get("reason") == "compiler-artifact" and item.get("profile", {}).get("test") and item.get("executable"):
                found.append(item["executable"])
        if len(found) != 1:
            raise ValueError(f"unclassified Rust test binary topology: {suite}: {found}")
        result[suite] = found[0]
    return result


def test_names(binary, ignored=False):
    command = [binary, "--list", "--format=terse"] + (["--ignored"] if ignored else [])
    output = subprocess.check_output(command, text=True, cwd=ROOT)
    return {line.removesuffix(": test") for line in output.splitlines() if line.endswith(": test")}


def audit(rows, with_rust=True):
    make = (ROOT / "Makefile").read_text()
    if '\t@$(YAI_BIN)' in make:
        raise ValueError('Make product assertions must invoke ./yai, not the build artifact')
    targets = set(re.findall(r"^([a-zA-Z][\w-]*):", make, re.M))
    for row in rows:
        if row["kind"] == "make" and row["selector"] not in targets:
            raise ValueError(f"unreachable Make entry: {row['selector']}")
        if row["kind"] == "rust" and not row["entrypoint"].startswith("test-rust-") and row["entrypoint"] not in targets:
            raise ValueError(f"unreachable delegated Rust entry: {row['id']}")
    # Every current leaf smoke and every assertion script/C file has an explicit
    # gate or a documented manual/support posture. Fixture helpers are not tests.
    classified_targets = {r["selector"] for r in rows if r["kind"] == "make"}
    leaves = {t for t in targets if t.startswith("smoke-") and not t.startswith("smoke-lab-")}
    if leaves - classified_targets:
        raise ValueError(f"unclassified smoke leaves: {sorted(leaves - classified_targets)}")
    covered = {p for r in rows for p in r["path"].split(";")}
    sources = {str(p.relative_to(ROOT)) for base in ("tests/smoke", "tests/characterization", "tests/integration")
               for p in (ROOT / base).rglob("*") if p.suffix in {".sh", ".c", ".py"}}
    if sources - covered:
        raise ValueError(f"unclassified assertion/support files: {sorted(sources - covered)}")
    for source in sources:
        if source.endswith('.sh') and 'target/debug/yai' in (ROOT/source).read_text():
            raise ValueError(f"product procedure bypasses ./yai launcher: {source}")
    # These are actual dry-run recipe selections, not a hand-maintained second
    # registry. The original commit is retained as a coverage positive control.
    old = subprocess.check_output(['git', 'show', f'{BASELINE}:Makefile'], cwd=ROOT, text=True)
    old_smoke = set(re.search(r'^smoke: (.+)$', old, re.M)[1].split())
    release = {r['entrypoint'] for r in lane_rows(rows, 'release')}
    if old_smoke - release:
        raise ValueError(f"publication coverage lost: {sorted(old_smoke-release)}")
    plan = subprocess.check_output(['make', '--no-print-directory', '-n', 'check', 'characterization'],
                                   cwd=ROOT, text=True)
    actual = re.findall(r'topology.py label --entry ([\w-]+)', plan)
    expected = {r['entrypoint'] for r in lane_rows(rows, 'release') if r['kind']=='make'}
    if set(actual) != expected or len(actual) != len(set(actual)):
        raise ValueError(f"Make recipe reachability/duplication differs: missing={sorted(expected-set(actual))}, extra={sorted(set(actual)-expected)}")
    golden = {r['entrypoint'] for r in lane_rows(rows, 'golden-local') if r['kind']=='make'}
    if not golden or golden & expected:
        raise ValueError('Golden lifecycle must be explicit and separate from the ordinary release gate')
    golden_plan = subprocess.check_output(['make', '--no-print-directory', '-n', 'test-golden-local'], cwd=ROOT, text=True)
    actual_golden = re.findall(r'topology.py label --entry ([\w-]+)', golden_plan)
    if set(actual_golden) != golden or len(actual_golden) != len(golden):
        raise ValueError('Golden Make reachability differs from classification')
    rust_entries = set(re.findall(r'topology.py rust --entry ([\w-]+)', plan))
    expected_rust = {r['entrypoint'] for r in lane_rows(rows, 'release')
                     if r['entrypoint'].startswith('test-rust-') and r['kind']=='rust'}
    if rust_entries != expected_rust:
        raise ValueError('Make Rust partition reachability differs from classification')
    print(f"coverage_parity: PASS old_smoke_leaves={len(old_smoke)} current_make_leaves={len(expected)} combined_duplicate_make_leaves=0")
    bins = binaries() if with_rust else {}
    for suite, binary in bins.items():
        expected = {r["selector"] for r in rows if r["kind"] == "rust" and r["id"].startswith(suite + ":")}
        actual = test_names(binary)
        if actual != expected:
            raise ValueError(f"Rust catalog drift {suite}: unclassified={sorted(actual-expected)}, stale={sorted(expected-actual)}")
        ignored = test_names(binary, ignored=True)
        accidental = [r['id'] for r in rows if r['kind']=='rust' and r['id'].startswith(suite+':')
                      and r['selector'] in ignored and r['cadence'] in {'fast','publication'}
                      and r['entrypoint'].startswith('test-rust-')]
        if accidental:
            raise ValueError(f"ignored tests silently promoted into publication: {accidental}")
        unreachable = {r['selector'] for r in rows if r['kind']=='rust' and r['id'].startswith(suite+':')
                       and r['selector'] not in ignored and r['cadence'] not in {'fast','publication'}}
        if unreachable:
            raise ValueError(f"formerly default Rust tests lost from publication: {sorted(unreachable)}")
    print(f"validation_catalog: PASS entries={len(rows)} assertion_files={len(sources)} rust_tests={sum(r['kind']=='rust' for r in rows)}")
    return bins


def run_rust(rows, entry):
    selected = [r for r in rows if r["kind"] == "rust" and r["entrypoint"] == entry]
    if not selected:
        raise ValueError(f"empty Rust execution group: {entry}")
    bins = binaries()
    for suite, binary in bins.items():
        group = [r for r in selected if r["id"].startswith(suite + ":")]
        if not group:
            continue
        names = sorted(r["selector"] for r in group)
        if not set(names) <= test_names(binary):
            raise ValueError(f"stale Rust selectors in {entry}")
        command = [binary, "--exact", "--include-ignored", "--color=never", *names]
        # Retain original default-suite libtest concurrency. Only the opt-in
        # large scale fixtures serialize their aggregate memory pressure.
        # Ignored environment-mutating host tests keep their existing wrappers.
        if any(r['cadence']=='endurance' for r in group):
            command.append("--test-threads=1")
        print(json.dumps({"entry": entry, "suite": suite, "proof_class": group[0]["proof_class"],
                          "provider_modes": sorted({r['provider_mode'] for r in group}),
                          "cwd": str(ROOT), "command": command, "selected_tests": len(names)}), flush=True)
        started = time.monotonic()
        result = subprocess.run(command, cwd=ROOT, env=cargo_env())
        print(json.dumps({"entry": entry, "suite": suite, "exit": result.returncode,
                          "elapsed_seconds": round(time.monotonic()-started, 3)}), flush=True)
        if result.returncode:
            return result.returncode
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["audit", "make", "list", "rust", "label"])
    parser.add_argument("--lane", default="release")
    parser.add_argument("--entry")
    parser.add_argument("--static", action="store_true")
    args = parser.parse_args()
    try:
        rows = read_catalog()
        if args.action == "make":
            make_graph(rows)
        elif args.action == "audit":
            audit(rows, not args.static)
        elif args.action == "list":
            print(json.dumps(lane_rows(rows, args.lane), indent=2))
        elif args.action == "label":
            selected = [r for r in rows if r['kind']=='make' and r['entrypoint']==args.entry]
            if len(selected) != 1:
                raise ValueError(f"unclassified or ambiguous execution entry: {args.entry}")
            print(json.dumps({"validation_entry": selected[0]}), flush=True)
        elif args.action == "rust":
            return run_rust(rows, args.entry)
    except (ValueError, OSError) as error:
        print(f"validation_topology: FAIL: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
