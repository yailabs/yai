#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CATALOG = ROOT / "governance" / "registry" / "agent-safe-primitives.v1.json"
ALLOWED_OWNERS = {"core", "exec", "data", "graph", "knowledge", "law", "support"}
ALLOWED_BINDING_MODELS = {"workspace-first", "global-baseline", "authoring-only"}


def main() -> int:
    if not CATALOG.exists():
        print(f"[agent-safe-primitives] missing catalog: {CATALOG}", file=sys.stderr)
        return 2
    doc = json.loads(CATALOG.read_text(encoding="utf-8"))
    if doc.get("kind") != "agent_safe_primitives_catalog":
        print("[agent-safe-primitives] invalid kind", file=sys.stderr)
        return 3
    primitives = doc.get("primitives", [])
    if not isinstance(primitives, list) or not primitives:
        print("[agent-safe-primitives] primitives empty", file=sys.stderr)
        return 4

    seen = set()
    forbidden_count = 0
    human_confirmed = 0
    for p in primitives:
        pid = str(p.get("id", ""))
        if not pid:
            print("[agent-safe-primitives] primitive missing id", file=sys.stderr)
            return 5
        if pid in seen:
            print(f"[agent-safe-primitives] duplicate id: {pid}", file=sys.stderr)
            return 6
        seen.add(pid)
        if p.get("kind") == "forbidden_direct":
            forbidden_count += 1
            if p.get("allowed_for_agent") is not False:
                print(f"[agent-safe-primitives] forbidden primitive cannot be agent-allowed: {pid}", file=sys.stderr)
                return 7
        if p.get("requires_human_confirmation") is True:
            human_confirmed += 1
        owner = str(p.get("runtime_family_owner", ""))
        if owner and owner not in ALLOWED_OWNERS:
            print(f"[agent-safe-primitives] invalid runtime_family_owner for {pid}: {owner}", file=sys.stderr)
            return 10
        if not owner:
            print(f"[agent-safe-primitives] missing runtime_family_owner: {pid}", file=sys.stderr)
            return 11

        binding_model = str(p.get("runtime_binding_model", ""))
        if binding_model and binding_model not in ALLOWED_BINDING_MODELS:
            print(f"[agent-safe-primitives] invalid runtime_binding_model for {pid}: {binding_model}", file=sys.stderr)
            return 12
        if not binding_model:
            print(f"[agent-safe-primitives] missing runtime_binding_model: {pid}", file=sys.stderr)
            return 13

        if p.get("workspace_binding_required") is not True and binding_model == "workspace-first":
            print(f"[agent-safe-primitives] workspace-first primitive must require workspace binding: {pid}", file=sys.stderr)
            return 14

    if forbidden_count == 0:
        print("[agent-safe-primitives] missing forbidden_direct class", file=sys.stderr)
        return 8
    if human_confirmed == 0:
        print("[agent-safe-primitives] missing human_confirmation_required primitives", file=sys.stderr)
        return 9

    print(f"[agent-safe-primitives] OK: {len(primitives)} primitives validated")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
