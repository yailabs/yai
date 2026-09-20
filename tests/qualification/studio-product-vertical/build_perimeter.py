#!/usr/bin/env python3
"""Build the bounded repository perimeter without committing a local path."""

import argparse
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
POLICY_PATH = "tests/qualification/studio-product-vertical/policy.json"


def source(name: str, path: str, media_type: str, *, roles=("knowledge",), bootstrap=False):
    return {
        "name": name,
        "resource": "resource:yai-repository",
        "roles": list(roles),
        "action": {"action": "discover", "path": path},
        "media_type": media_type,
        "bootstrap_policy": bootstrap,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    perimeter = {
        "schema": "yai.source_perimeter.v1",
        "name": "studio-product-vertical",
        "participant": "participant:operator",
        "resources": [{
            "schema": "yai.resource_definition.v1",
            "attachment_id": "resource:yai-repository",
            "policy_owner": "participant:operator",
            "participant_ids": ["participant:operator"],
            "operations": ["discover", "admit_content", "content_read"],
            "read_prefixes": [
                POLICY_PATH,
                "studio/README.md",
                "studio/ROADMAP.md",
                "cmd/README.md",
                "studio/package.json",
                "application/yai-application/Cargo.toml",
            ],
            "names": [],
            "max_output_bytes": 65536,
            "max_items": 16,
            "address": {"kind": "discovery", "root": str(ROOT)},
        }],
        "sources": [
            source("qualification-policy", POLICY_PATH, "application/json", roles=("policy",), bootstrap=True),
            source("studio-readme", "studio/README.md", "text/markdown"),
            source("studio-roadmap", "studio/ROADMAP.md", "text/markdown"),
            source("cli-readme", "cmd/README.md", "text/markdown"),
            source("studio-package", "studio/package.json", "application/json"),
            source("application-manifest", "application/yai-application/Cargo.toml", "text/plain"),
        ],
    }
    Path(args.output).write_text(json.dumps(perimeter, indent=2) + "\n")


if __name__ == "__main__":
    main()
