#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
LAW_ROOT="${LAW_COMPAT_ROOT:-}"
if [[ -z "$LAW_ROOT" ]]; then
  CANDIDATE="$(cd "$ROOT/.." && pwd)/law"
  [[ -d "$CANDIDATE" ]] && LAW_ROOT="$CANDIDATE"
fi
if [[ -z "$LAW_ROOT" ]]; then
  echo "LAW root not found (expected ../law)" >&2
  exit 2
fi

OUT_ROOT="$ROOT/build/test/brain"
OBJ_DIR="$OUT_ROOT/obj"
BIN_DIR="$OUT_ROOT/bin"
mkdir -p "$OBJ_DIR" "$BIN_DIR"

BRAIN_SRCS=(
  lib/core/lifecycle/runtime_capabilities.c
  lib/core/workspace/workspace_binding.c
  lib/core/workspace/workspace_recovery.c
  lib/data/binding/store_binding.c
  lib/data/binding/workspace_binding.c
  lib/knowledge/runtime_compat.c
  lib/knowledge/cognition/cognition.c
  lib/exec/agents/agents_dispatch.c
  lib/exec/agents/agent_code.c
  lib/exec/agents/agent_historian.c
  lib/exec/agents/agent_knowledge.c
  lib/exec/agents/agent_system.c
  lib/exec/agents/agent_validator.c
  lib/exec/orchestration/planner.c
  lib/exec/orchestration/rag_sessions.c
  lib/exec/orchestration/rag_context_builder.c
  lib/exec/orchestration/rag_prompts.c
  lib/exec/orchestration/rag_pipeline.c
  lib/knowledge/cognition/reasoning/reasoning_roles.c
  lib/knowledge/cognition/reasoning/scoring.c
  lib/knowledge/memory/memory.c
  lib/knowledge/memory/arena_store.c
  lib/knowledge/memory/storage_bridge.c
  lib/graph/state/graph_backend.c
  lib/graph/state/graph_backend_rpc.c
  lib/graph/state/graph_facade.c
  lib/graph/state/graph.c
  lib/graph/state/ids.c
  lib/graph/domains/activation.c
  lib/graph/domains/authority.c
  lib/graph/domains/episodic.c
  lib/graph/domains/semantic.c
  lib/knowledge/memory/semantic_db.c
  lib/knowledge/memory/vector_index.c
  lib/knowledge/providers/providers.c
  lib/knowledge/providers/provider_registry.c
  lib/exec/bridge/client_bridge.c
  lib/knowledge/providers/mock_provider.c
  lib/knowledge/providers/embedder_mock.c
  lib/exec/transport/brain_transport.c
  lib/exec/transport/brain_protocol.c
  lib/exec/transport/uds_server.c
)

OBJS=()
for src in "${BRAIN_SRCS[@]}"; do
  obj="$OBJ_DIR/${src%.c}.o"
  mkdir -p "$(dirname "$obj")"
  cc -Wall -Wextra -std=c11 -O2 -I"$ROOT/include" -I"$ROOT/include/yai" -I"$LAW_ROOT/contracts/protocol/include" -c "$ROOT/$src" -o "$obj"
  OBJS+=("$obj")
done

UNIT_TESTS=(
  tests/unit/brain/mind_legacy_tests/test_memory_graph.c
  tests/unit/brain/mind_legacy_tests/test_memory_domains.c
  tests/unit/brain/mind_legacy_tests/test_providers.c
  tests/unit/brain/mind_legacy_tests/test_transport.c
  tests/unit/brain/mind_legacy_tests/test_cognition_agents.c
  tests/unit/brain/mind_legacy_tests/test_reasoning_scoring.c
  tests/unit/brain/mind_legacy_tests/test_rag_pipeline.c
  tests/unit/brain/mind_legacy_tests/test_mind_flow.c
  tests/unit/brain/mind_legacy_tests/test_lifecycle_reinit.c
)

for t in "${UNIT_TESTS[@]}"; do
  name="$(basename "${t%.c}")"
  cc -Wall -Wextra -std=c11 -O2 -I"$ROOT/include" -I"$ROOT/include/yai" "$ROOT/$t" "${OBJS[@]}" -o "$BIN_DIR/$name" -lm
  "$BIN_DIR/$name"
done
