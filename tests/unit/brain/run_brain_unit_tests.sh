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
  lib/brain/lifecycle/brain_lifecycle.c
  lib/brain/cognition/cognition.c
  lib/brain/cognition/agents/agents_dispatch.c
  lib/brain/cognition/agents/agent_code.c
  lib/brain/cognition/agents/agent_historian.c
  lib/brain/cognition/agents/agent_knowledge.c
  lib/brain/cognition/agents/agent_system.c
  lib/brain/cognition/agents/agent_validator.c
  lib/brain/cognition/orchestration/planner.c
  lib/brain/cognition/orchestration/rag_sessions.c
  lib/brain/cognition/orchestration/rag_context_builder.c
  lib/brain/cognition/orchestration/rag_prompts.c
  lib/brain/cognition/orchestration/rag_pipeline.c
  lib/brain/cognition/reasoning/reasoning_roles.c
  lib/brain/cognition/reasoning/scoring.c
  lib/brain/memory/memory.c
  lib/brain/memory/arena_store.c
  lib/brain/memory/storage_bridge.c
  lib/brain/memory/graph/graph_backend.c
  lib/brain/memory/graph/graph_backend_rpc.c
  lib/brain/memory/graph/graph_facade.c
  lib/brain/memory/graph/graph.c
  lib/brain/memory/graph/ids.c
  lib/brain/memory/graph/domain_activation.c
  lib/brain/memory/graph/domain_authority.c
  lib/brain/memory/graph/domain_episodic.c
  lib/brain/memory/graph/domain_semantic.c
  lib/brain/memory/graph/semantic_db.c
  lib/brain/memory/graph/vector_index.c
  lib/brain/bridge/providers.c
  lib/brain/bridge/provider_registry.c
  lib/brain/bridge/client_bridge.c
  lib/brain/bridge/mock_provider.c
  lib/brain/bridge/embedder_mock.c
  lib/brain/transport/brain_transport.c
  lib/brain/transport/brain_protocol.c
  lib/brain/transport/uds_server.c
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
