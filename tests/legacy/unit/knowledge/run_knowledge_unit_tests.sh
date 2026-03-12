#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
CONTRACT_ROOT="${YAI_PROTOCOL_CONTRACT_ROOT:-$ROOT/include/yai/protocol}"
if [[ ! -d "$CONTRACT_ROOT" ]]; then
  echo "contract root not found (expected include/yai/protocol)" >&2
  exit 2
fi

OUT_ROOT="$ROOT/build/test/knowledge"
OBJ_DIR="$OUT_ROOT/obj"
BIN_DIR="$OUT_ROOT/bin"
rm -rf "$OBJ_DIR" "$BIN_DIR"
mkdir -p "$OBJ_DIR" "$BIN_DIR"

BRAIN_SRCS=(
  lib/runtime/lifecycle/runtime_capabilities.c
  runtime/compatibility/legacy/lib/runtime/workspace/workspace_binding.c
  runtime/compatibility/legacy/lib/runtime/workspace/workspace_recovery.c
  lib/runtime/policy/policy_state.c
  lib/runtime/grants/grants_state.c
  lib/runtime/containment/containment_state.c
  lib/data/binding/store_binding.c
  runtime/compatibility/legacy/lib/data/binding/workspace_binding.c
  lib/data/query/inspect_query.c
  lib/knowledge/runtime_compat.c
  lib/knowledge/cognition/cognition.c
  lib/agents/dispatch/agents_dispatch.c
  lib/agents/roles/agent_code.c
  lib/agents/roles/agent_historian.c
  lib/agents/roles/agent_knowledge.c
  lib/agents/roles/agent_system.c
  lib/agents/roles/agent_validator.c
  lib/orchestration/planner/planner.c
  lib/orchestration/workflow/rag_sessions.c
  lib/orchestration/actions/rag_context_builder.c
  lib/orchestration/actions/rag_prompts.c
  lib/orchestration/execution/rag_pipeline.c
  lib/orchestration/runtime/grounding_context.c
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
  lib/graph/materialization/from_runtime_records.c
  lib/knowledge/semantic/semantic_db.c
  lib/knowledge/vector/vector_index.c
  lib/knowledge/cognition/activation.c
  lib/knowledge/memory/authority.c
  lib/knowledge/episodic/episodic.c
  lib/network/providers/catalog.c
  lib/network/providers/provider_registry.c
  lib/network/providers/provider_policy.c
  lib/network/providers/provider_selection.c
  lib/network/providers/inference/client_inference.c
  lib/network/providers/embedding/client_embedding.c
  lib/network/providers/mocks/mock_provider.c
  lib/network/providers/embedding/embedder_mock.c
  lib/orchestration/transport/transport_runtime.c
  lib/orchestration/transport/transport_protocol.c
  lib/orchestration/transport/uds_server.c
  lib/control/protocol/control/source_plane.c
  lib/third_party/cjson/cJSON.c
)

OBJS=()
for src in "${BRAIN_SRCS[@]}"; do
  obj="$OBJ_DIR/${src%.c}.o"
  mkdir -p "$(dirname "$obj")"
  cc -Wall -Wextra -std=c11 -O2 -I"$ROOT/include" -I"$ROOT/include/yai" -I"$ROOT/lib/third_party/cjson" -I"$CONTRACT_ROOT" -c "$ROOT/$src" -o "$obj"
  OBJS+=("$obj")
done

UNIT_TESTS=(
  tests/legacy/unit/knowledge/mind_legacy_tests/test_memory_graph.c
  tests/legacy/unit/knowledge/mind_legacy_tests/test_memory_domains.c
  tests/legacy/unit/knowledge/mind_legacy_tests/test_providers.c
  tests/legacy/unit/knowledge/mind_legacy_tests/test_transport.c
  tests/legacy/unit/knowledge/mind_legacy_tests/test_cognition_agents.c
  tests/legacy/unit/knowledge/mind_legacy_tests/test_reasoning_scoring.c
  tests/legacy/unit/knowledge/mind_legacy_tests/test_rag_pipeline.c
  tests/legacy/unit/knowledge/mind_legacy_tests/test_mind_flow.c
  tests/legacy/unit/knowledge/mind_legacy_tests/test_lifecycle_reinit.c
)

RAN=0
for t in "${UNIT_TESTS[@]}"; do
  if [[ ! -f "$ROOT/$t" ]]; then
    echo "knowledge_unit_tests: skip missing $t" >&2
    continue
  fi
  name="$(basename "${t%.c}")"
  cc -Wall -Wextra -std=c11 -O2 -I"$ROOT/include" -I"$ROOT/include/yai" -I"$ROOT/lib/third_party/cjson" "$ROOT/$t" "${OBJS[@]}" -o "$BIN_DIR/$name" -lm
  "$BIN_DIR/$name"
  RAN=1
done

if [[ "$RAN" -eq 0 ]]; then
  echo "knowledge_unit_tests: no C unit cases found, build-only check passed"
fi
