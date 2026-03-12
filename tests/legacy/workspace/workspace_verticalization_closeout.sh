#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_ROOT="$ROOT"
GOVERNANCE_ROOT="$ROOT/governance"
CLI_ROOT="$ROOT"
SDK_ROOT="$ROOT"
TMP="$(mktemp -d /tmp/yai-wsv6-closeout-XXXXXX)"
trap 'rm -rf "$TMP"' EXIT

need_file() {
  local p="$1"
  [[ -f "$p" ]] || { echo "wsv6_closeout: missing file: $p"; exit 2; }
}

need_dir() {
  local p="$1"
  [[ -d "$p" ]] || { echo "wsv6_closeout: missing dir: $p"; exit 2; }
}

need_dir "$GOVERNANCE_ROOT"
need_dir "$CLI_ROOT"
need_dir "$SDK_ROOT"
need_file "$YAI_ROOT/user/shell/parse/parse.c"
need_file "$GOVERNANCE_ROOT/model/registry/commands.v1.json"
need_file "$CLI_ROOT/user/shell/help/help.c"
need_file "$SDK_ROOT/sdk/c/libyai/include/yai/sdk/sdk.h"

# 1) Runtime command-id substrate exists for canonical families.
rg -n "yai\\.workspace\\.graph\\.(summary|workspace|governance|decision|evidence|authority|artifact|lineage|recent)" \
  "$YAI_ROOT/user/shell/parse/parse.c" >/dev/null
rg -n "yai\\.workspace\\.(query|events\\.tail|status|inspect|domain_get|domain_set|policy_effective|policy_attach|policy_activate|policy_detach|policy_dry_run|debug_resolution|open|create|set|switch|unset|clear|reset|destroy)" \
  "$YAI_ROOT/user/shell/parse/parse.c" >/dev/null

# 2) Governance registry has canonical ws topics/families.
python3 - "$GOVERNANCE_ROOT/model/registry/commands.v1.json" <<'PY'
import json, sys
from collections import defaultdict
p = sys.argv[1]
obj = json.loads(open(p, "r", encoding="utf-8").read())
cmds = obj.get("commands", [])
topics = defaultdict(set)
for c in cmds:
    if c.get("entrypoint") != "ws":
        continue
    t = c.get("topic", "")
    op = c.get("op", "")
    if t and op:
        topics[t].add(op)
required = {
    "general": {"create","open","set","switch","current","status","inspect","unset","clear","reset","destroy"},
    "graph": {"summary","workspace","governance","decision","evidence","authority","artifact","lineage","recent"},
    "db": {"status","bindings","stores","classes","count","tail"},
    "data": {"events","evidence","governance","authority","artifacts","enforcement"},
    "knowledge": {"status","transient","memory","providers","context"},
    "policy": {"attach","detach","activate","dry-run","effective"},
    "domain": {"get","set"},
    "recovery": {"status","load","reopen"},
    "debug": {"resolution"},
    "query": {"family"},
}
missing = []
for topic, ops in required.items():
    got = topics.get(topic, set())
    if not ops.issubset(got):
        missing.append((topic, sorted(ops - got)))
if missing:
    for t, m in missing:
        print(f"missing topic/op: {t} -> {','.join(m)}", file=sys.stderr)
    raise SystemExit(1)
print("governance-registry-ws-topics: ok")
PY

# 3) CLI help exposes canonical ws families.
YAI_SDK_COMPAT_REGISTRY_DIR="$GOVERNANCE_ROOT" "$CLI_ROOT/build/bin/yai" help ws >"$TMP/help_ws.txt" 2>&1
rg -n "graph|db|data|knowledge|policy|domain|recovery|debug|query" "$TMP/help_ws.txt" >/dev/null

YAI_SDK_COMPAT_REGISTRY_DIR="$GOVERNANCE_ROOT" "$CLI_ROOT/build/bin/yai" help ws graph >"$TMP/help_ws_graph.txt" 2>&1
rg -n "summary|workspace|governance|decision|evidence|authority|artifact|lineage|recent" "$TMP/help_ws_graph.txt" >/dev/null

# 4) SDK public surface exports typed family headers/helpers.
rg -n "yai/sdk/(db|policy|recovery|debug)\\.h" "$SDK_ROOT/sdk/c/libyai/include/yai/sdk/sdk.h" >/dev/null
rg -n "yai_sdk_ws_(graph_|db_|data_|knowledge_|policy_|domain_|recovery_|debug_resolution)" \
  "$SDK_ROOT/sdk/c/libyai/include/yai/sdk" >/dev/null

echo "workspace_verticalization_closeout: ok"
