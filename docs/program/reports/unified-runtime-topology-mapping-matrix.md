# Unified Runtime Topology Mapping Matrix (RF-1 Annex)

Status: normative annex for RF-1.

Columns:
- old path
- target path
- target family
- action
- rationale
- downstream owner

| old path | target path | target family | action | rationale | downstream owner |
|---|---|---|---|---|---|
| include/yai/brain/brain.h | include/yai/knowledge/cognition.h | knowledge | rename | remove brain public namespace | RF-3 |
| include/yai/brain/cognition.h | include/yai/knowledge/cognition.h | knowledge | move | cognition is knowledge substrate | RF-3 |
| include/yai/brain/errors.h | include/yai/knowledge/memory.h | knowledge | absorb | errors/types fold into knowledge API | RF-3 |
| include/yai/brain/memory.h | include/yai/knowledge/memory.h | knowledge | move | memory belongs to knowledge family | RF-3 |
| include/yai/brain/providers.h | include/yai/knowledge/providers.h | knowledge | move | provider API belongs to knowledge providers | RF-3 |
| include/yai/brain/transport.h | include/yai/exec/transport_client.h | exec | move | runtime transport is execution capability | RF-3 |
| include/yai/brain/types.h | include/yai/knowledge/memory.h | knowledge | absorb | brain type surface removed | RF-3 |
| lib/brain/bridge/client_bridge.c | lib/exec/bridge/client_bridge.c | exec | move | runtime bridge belongs to execution routing | RF-2 |
| lib/brain/bridge/embedder_mock.c | lib/knowledge/providers/embedder_mock.c | knowledge | move | embedder mock is provider concern | RF-2 |
| lib/brain/bridge/mock_provider.c | lib/knowledge/providers/mock_provider.c | knowledge | move | provider mock belongs to providers | RF-2 |
| lib/brain/bridge/provider_registry.c | lib/knowledge/providers/provider_registry.c | knowledge | move | registry of providers is knowledge provider layer | RF-2 |
| lib/brain/bridge/providers.c | lib/knowledge/providers/providers.c | knowledge | move | provider facade belongs to knowledge | RF-2 |
| lib/brain/cognition/cognition.c | lib/knowledge/cognition/cognition.c | knowledge | move | cognition core in knowledge | RF-2 |
| lib/brain/cognition/cognition_internal.h | lib/knowledge/cognition/internal.h | knowledge | rename | internal header normalized to family conventions | RF-2 |
| lib/brain/cognition/agents/agent_code.c | lib/exec/agents/agent_code.c | exec | move | agents consolidate in exec | RF-2 |
| lib/brain/cognition/agents/agent_historian.c | lib/exec/agents/agent_historian.c | exec | move | agents consolidate in exec | RF-2 |
| lib/brain/cognition/agents/agent_knowledge.c | lib/exec/agents/agent_knowledge.c | exec | move | agents consolidate in exec | RF-2 |
| lib/brain/cognition/agents/agent_system.c | lib/exec/agents/agent_system.c | exec | move | agents consolidate in exec | RF-2 |
| lib/brain/cognition/agents/agent_validator.c | lib/exec/agents/agent_validator.c | exec | move | agents consolidate in exec | RF-2 |
| lib/brain/cognition/agents/agents_dispatch.c | lib/exec/agents/agents_dispatch.c | exec | move | dispatch is execution coordination | RF-2 |
| lib/brain/cognition/orchestration/planner.c | lib/exec/orchestration/planner.c | exec | move | orchestration belongs to exec | RF-2 |
| lib/brain/cognition/orchestration/rag_context_builder.c | lib/exec/orchestration/rag_context_builder.c | exec | move | orchestration belongs to exec | RF-2 |
| lib/brain/cognition/orchestration/rag_pipeline.c | lib/exec/orchestration/rag_pipeline.c | exec | move | orchestration belongs to exec | RF-2 |
| lib/brain/cognition/orchestration/rag_prompts.c | lib/exec/orchestration/rag_prompts.c | exec | move | orchestration belongs to exec | RF-2 |
| lib/brain/cognition/orchestration/rag_sessions.c | lib/exec/orchestration/rag_sessions.c | exec | move | orchestration belongs to exec | RF-2 |
| lib/brain/cognition/reasoning/reasoning_roles.c | lib/knowledge/cognition/reasoning/reasoning_roles.c | knowledge | move | reasoning substrate remains knowledge | RF-2 |
| lib/brain/cognition/reasoning/scoring.c | lib/knowledge/cognition/reasoning/scoring.c | knowledge | move | scoring is knowledge reasoning substrate | RF-2 |
| lib/brain/lifecycle/brain_lifecycle.c | lib/core/lifecycle/startup_plan.c + lib/core/workspace/workspace_binding.c | core | delete | brain lifecycle collapses into runtime/core lifecycle | RF-4 |
| lib/brain/memory/arena_store.c | lib/knowledge/memory/arena_store.c | knowledge | move | memory infra belongs to knowledge | RF-2 |
| lib/brain/memory/memory.c | lib/knowledge/memory/memory.c | knowledge | move | memory core belongs to knowledge | RF-2 |
| lib/brain/memory/storage_bridge.c | lib/graph/materialization/from_decisions.c + lib/data/records/decision_records.c | graph + data | split | bridge split between graph materialization and record references | RF-2 |
| lib/brain/memory/graph/domain_activation.c | lib/graph/domains/activation.c | graph | move | graph domain module | RF-2 |
| lib/brain/memory/graph/domain_authority.c | lib/graph/domains/authority.c | graph | move | graph domain module | RF-2 |
| lib/brain/memory/graph/domain_episodic.c | lib/graph/domains/episodic.c | graph | move | graph domain module | RF-2 |
| lib/brain/memory/graph/domain_semantic.c | lib/graph/domains/semantic.c | graph | move | graph domain module | RF-2 |
| lib/brain/memory/graph/graph.c | lib/graph/state/graph.c | graph | move | graph state module | RF-2 |
| lib/brain/memory/graph/graph_backend.c | lib/graph/state/graph_backend.c | graph | move | graph state backend | RF-2 |
| lib/brain/memory/graph/graph_backend.h | lib/graph/state/internal.h | graph | rename | backend header normalized to internal API | RF-2 |
| lib/brain/memory/graph/graph_backend_rpc.c | lib/graph/state/graph_backend_rpc.c | graph | move | graph state backend rpc | RF-2 |
| lib/brain/memory/graph/graph_facade.c | lib/graph/state/graph_facade.c | graph | move | graph state facade | RF-2 |
| lib/brain/memory/graph/graph_state_internal.h | lib/graph/state/graph_state_internal.h | graph | move | graph internal state contract | RF-2 |
| lib/brain/memory/graph/ids.c | lib/graph/state/ids.c | graph | move | graph ids belong to graph state | RF-2 |
| lib/brain/memory/graph/semantic_db.c | lib/knowledge/memory/semantic_db.c | knowledge | move | semantic db serves knowledge memory | RF-2 |
| lib/brain/memory/graph/vector_index.c | lib/knowledge/memory/vector_index.c | knowledge | move | vector index serves knowledge memory | RF-2 |
| lib/brain/transport/brain_protocol.c | lib/exec/transport/brain_protocol.c | exec | move | runtime transport belongs to exec | RF-2 |
| lib/brain/transport/brain_transport.c | lib/exec/transport/brain_transport.c | exec | move | runtime transport belongs to exec | RF-2 |
| lib/brain/transport/uds_server.c | lib/exec/transport/uds_server.c | exec | move | runtime transport belongs to exec | RF-2 |
| lib/exec/agents/agent_enforcement.c | lib/exec/agents/agent_enforcement.c | exec | absorb | remains canonical exec agent; absorb overlaps | RF-6 |
| lib/exec/runtime/config_loader.c | lib/exec/runtime/config_loader.c | exec | keep | already canonical runtime exec | RF-6 |
| lib/exec/runtime/exec_runtime.c | lib/exec/runtime/exec_runtime.c | exec | keep | already canonical runtime exec | RF-6 |
| lib/exec/runtime/runtime_model.c | lib/exec/runtime/runtime_model.c | exec | keep | already canonical runtime exec | RF-6 |
| lib/exec/bridge/engine_bridge.c | lib/exec/bridge/engine_bridge.c | exec | keep | already canonical bridge | RF-6 |
| lib/exec/bridge/rpc_router.c | lib/exec/bridge/rpc_router.c | exec | keep | already canonical bridge | RF-6 |
| lib/exec/bridge/transport_client.c | lib/exec/bridge/transport_client.c | exec | keep | already canonical bridge | RF-6 |
| include/yai/brain (directory) | none | n/a | delete | brain namespace removal | RF-8 |
| lib/brain (directory) | none | n/a | delete | brain container removal | RF-8 |

## Consolidation rules required by RF-1

- Agents canonical home: `lib/exec/agents/`.
- Orchestration canonical home: `lib/exec/orchestration/`.
- Graph runtime truth canonical home: `lib/graph/`.
- Knowledge substrate canonical home: `lib/knowledge/`.
- Runtime lifecycle authority canonical home: `lib/core/lifecycle/` + `lib/core/workspace/`.
