#!/usr/bin/env python3
"""Exercise every compiled YAI command descriptor through its help path.

The command list is deliberately obtained from `yai help --advanced --json`;
this audit has no independently maintained command catalog.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path


def invoke(binary: Path, words: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(binary), *words],
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=Path("./yai"))
    parser.add_argument("--inventory-tsv", action="store_true")
    parser.add_argument(
        "--source-root", type=Path, default=Path(__file__).resolve().parents[3]
    )
    args = parser.parse_args()
    args.binary = args.binary.resolve()

    discovery_run = invoke(args.binary, ["help", "--advanced", "--json"])
    if discovery_run.returncode != 0:
        sys.stderr.write(discovery_run.stderr)
        return discovery_run.returncode
    discovery = json.loads(discovery_run.stdout)
    if discovery.get("schema") != "yai.cli.command_discovery.v1":
        raise SystemExit("unexpected discovery schema")

    operations = discovery["operations"]
    failures: list[str] = []
    for operation in operations:
        result = invoke(args.binary, [*operation["path"], "--help"])
        if result.returncode != 0:
            failures.append(
                f'{operation["operation_id"]}: help exit={result.returncode}: '
                f"{result.stderr.strip()}"
            )
        elif (
            operation["operation_id"] != "yai.meta.help"
            and operation["operation_id"] not in result.stdout
        ):
            failures.append(f'{operation["operation_id"]}: operation ID absent from help')
    help_failure_count = len(failures)

    handler_sources = [
        args.source_root / "cmd/yai/src/cli/product.rs",
        args.source_root / "cmd/yai/src/command_adapters.rs",
    ]
    handler_text = "\n".join(path.read_text() for path in handler_sources)
    exact_handlers = set(re.findall(r'"(yai\.[a-z0-9_.]+)"', handler_text))
    exact_handlers.add("yai.meta.help")  # admitted and completed by parser/help
    prefix_handlers = set(re.findall(r'starts_with\("(yai\.[a-z0-9_.]+)"\)', handler_text))
    handler_failures = []
    for operation in operations:
        operation_id = operation["operation_id"]
        if operation["visibility"] == "removed":
            continue
        if operation_id in exact_handlers or any(
            operation_id.startswith(prefix) for prefix in prefix_handlers
        ):
            continue
        handler_failures.append(operation_id)
        failures.append(f"{operation_id}: handler ID is not source-resolvable")

    if args.inventory_tsv:
        print(
            "operation_id\tpath\tvisibility\tlane\tmutation\toutput\t"
            "positionals\tflags\taliases\tlegacy_path\tremoved_successor"
        )
        for operation in operations:
            print(
                "\t".join(
                    (
                        operation["operation_id"],
                        " ".join(operation["path"]),
                        operation["visibility"],
                        operation["lane"],
                        operation["mutation"],
                        operation["output"],
                        json.dumps(operation["positionals"], sort_keys=True),
                        json.dumps(operation["flags"], sort_keys=True),
                        json.dumps(operation["aliases"], sort_keys=True),
                        "-"
                        if not operation["legacy_path"]
                        else " ".join(operation["legacy_path"]),
                        "-"
                        if operation["removed_successor"] is None
                        else " ".join(operation["removed_successor"]),
                    )
                )
            )

    counts: dict[str, int] = {}
    for operation in operations:
        counts[operation["visibility"]] = counts.get(operation["visibility"], 0) + 1
    summary = {
        "schema": discovery["schema"],
        "registry_digest": discovery["cli_registry_digest"],
        "operation_count": len(operations),
        "visibility_counts": dict(sorted(counts.items())),
        "help_failures": help_failure_count,
        "handler_failures": len(handler_failures),
    }
    stream = sys.stderr if args.inventory_tsv else sys.stdout
    print(json.dumps(summary, sort_keys=True), file=stream)
    for failure in failures:
        print(failure, file=sys.stderr)
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
