# =========================================
# YAI — Unified Root Build Spine
# =========================================

ROOT_DIR := $(abspath .)
PROTOCOL_CONTRACT_ROOT ?= $(ROOT_DIR)/include/yai/protocol/contracts
GOVERNANCE_COMPAT_ROOT ?= $(ROOT_DIR)/governance

BUILD_DIR ?= $(ROOT_DIR)/build
BIN_DIR ?= $(BUILD_DIR)/bin
OBJ_DIR ?= $(BUILD_DIR)/obj
LIB_DIR ?= $(BUILD_DIR)/lib
TEST_DIR ?= $(BUILD_DIR)/test

DIST_ROOT ?= $(ROOT_DIR)/dist
BIN_DIST ?= $(DIST_ROOT)/bin

CC ?= cc
PKG_CONFIG ?= pkg-config
CPPFLAGS ?= -I$(ROOT_DIR) -I$(ROOT_DIR)/include -I$(ROOT_DIR)/include/yai \
            -I$(ROOT_DIR)/lib/third_party/cjson \
            -I$(PROTOCOL_CONTRACT_ROOT)
CFLAGS ?= -Wall -Wextra -std=c11 -O2
LDFLAGS ?=
LDLIBS ?= -lm

LMDB_CFLAGS := $(shell $(PKG_CONFIG) --cflags liblmdb 2>/dev/null)
LMDB_LIBS := $(shell $(PKG_CONFIG) --libs liblmdb 2>/dev/null)
HIREDIS_CFLAGS := $(shell $(PKG_CONFIG) --cflags hiredis 2>/dev/null)
HIREDIS_LIBS := $(shell $(PKG_CONFIG) --libs hiredis 2>/dev/null)
DUCKDB_CFLAGS := $(if $(wildcard /opt/homebrew/opt/duckdb/include/duckdb.h),-I/opt/homebrew/opt/duckdb/include,)
DUCKDB_LIBS := $(if $(wildcard /opt/homebrew/opt/duckdb/lib/libduckdb.dylib),-L/opt/homebrew/opt/duckdb/lib -lduckdb,)

ifneq ($(strip $(LMDB_LIBS)),)
CPPFLAGS += $(LMDB_CFLAGS) -DYAI_HAVE_LMDB=1
LDLIBS += $(LMDB_LIBS)
endif
ifneq ($(strip $(HIREDIS_LIBS)),)
CPPFLAGS += $(HIREDIS_CFLAGS) -DYAI_HAVE_HIREDIS=1
LDLIBS += $(HIREDIS_LIBS)
endif
ifneq ($(strip $(DUCKDB_LIBS)),)
CPPFLAGS += $(DUCKDB_CFLAGS) -DYAI_HAVE_DUCKDB=1
LDLIBS += $(DUCKDB_LIBS)
endif

YAI_OBJ := $(OBJ_DIR)/cmd/yai/main.o
YAI_BIN := $(BIN_DIR)/yai
YAI_EDGE_OBJ := $(OBJ_DIR)/cmd/yai-edge/main.o
YAI_EDGE_BIN := $(BIN_DIR)/yai-edge
YAI_DAEMON_ALIAS_BIN := $(BIN_DIR)/yai-daemon

SUPPORT_SRCS := lib/support/ids.c lib/support/logger.c lib/support/errors.c lib/support/strings.c lib/support/paths.c
PLATFORM_SRCS := lib/platform/os.c lib/platform/fs.c lib/platform/clock.c lib/platform/uds.c
PROTOCOL_SRCS := \
	lib/protocol/rpc/runtime.c \
	lib/protocol/rpc/codec.c \
	lib/protocol/binary/rpc_binary.c \
	lib/protocol/contracts/message_types.c \
	lib/protocol/contracts/source_plane_contract.c
CORE_SRCS := \
	lib/runtime/lifecycle/bootstrap.c \
	lib/runtime/lifecycle/preboot.c \
	lib/runtime/lifecycle/startup_plan.c \
	lib/runtime/lifecycle/runtime_capabilities.c \
	lib/runtime/dispatch/control_transport.c \
	lib/runtime/dispatch/command_dispatch.c \
	lib/runtime/dispatch/attach_flow.c \
	lib/runtime/session/session.c \
	lib/runtime/session/session_reply.c \
	lib/runtime/session/session_utils.c \
	lib/runtime/workspace/project_tree.c \
	lib/runtime/workspace/workspace_registry.c \
	lib/runtime/workspace/workspace_runtime.c \
	lib/runtime/workspace/workspace_binding.c \
	lib/runtime/workspace/workspace_recovery.c \
	lib/runtime/policy/policy_state.c \
	lib/runtime/grants/grants_state.c \
	lib/runtime/containment/containment_state.c \
	lib/runtime/enforcement/enforcement.c \
	lib/runtime/authority/authority_registry.c \
	lib/runtime/authority/identity.c
GOVERNANCE_SRCS := \
	lib/governance/policy_effects.c \
	lib/governance/loader/governance_loader.c \
	lib/governance/loader/manifest_loader.c \
	lib/governance/loader/domain_model_matrix.c \
	lib/governance/loader/domain_loader.c \
	lib/governance/loader/compliance_loader.c \
	lib/governance/loader/overlay_loader.c \
	lib/governance/loader/compatibility_check.c \
	lib/governance/classification/event_classifier.c \
	lib/governance/classification/action_classifier.c \
	lib/governance/classification/provider_classifier.c \
	lib/governance/classification/resource_classifier.c \
	lib/governance/classification/protocol_classifier.c \
	lib/governance/classification/workspace_context.c \
	lib/governance/discovery/domain_discovery.c \
	lib/governance/discovery/signal_matcher.c \
	lib/governance/discovery/protocol_matcher.c \
	lib/governance/discovery/provider_matcher.c \
	lib/governance/discovery/resource_matcher.c \
	lib/governance/discovery/command_matcher.c \
	lib/governance/discovery/confidence_model.c \
	lib/governance/resolution/resolver.c \
	lib/governance/resolution/stack_builder.c \
	lib/governance/resolution/foundation_merge.c \
	lib/governance/resolution/domain_merge.c \
	lib/governance/resolution/compliance_merge.c \
	lib/governance/resolution/overlay_merge.c \
	lib/governance/resolution/precedence.c \
	lib/governance/resolution/fallback.c \
	lib/governance/resolution/conflict_resolution.c \
	lib/governance/resolution/effective_stack.c \
	lib/governance/mapping/event_to_domain.c \
	lib/governance/mapping/domain_to_policy.c \
	lib/governance/mapping/policy_to_effect.c \
	lib/governance/mapping/decision_to_evidence.c \
	lib/governance/mapping/decision_to_audit.c \
	lib/governance/debug/resolution_trace.c \
	lib/governance/debug/dump_effective_stack.c \
	lib/governance/debug/dump_discovery_result.c
ORCHESTRATION_RUNTIME_SRCS := \
	lib/orchestration/runtime/exec_runtime.c \
	lib/orchestration/runtime/config_loader.c \
	lib/orchestration/runtime/runtime_model.c \
	lib/orchestration/runtime/grounding_context.c \
	lib/orchestration/runtime/source_plane_contract.c \
	lib/orchestration/runtime/peer_registry.c \
	lib/orchestration/runtime/source_ingest.c \
	lib/orchestration/gates/provider_gate.c \
	lib/orchestration/gates/network_gate.c \
	lib/orchestration/gates/storage_gate.c \
	lib/orchestration/gates/resource_gate.c \
	lib/orchestration/bridge/engine_bridge.c \
	lib/orchestration/bridge/transport_client.c \
	lib/orchestration/bridge/rpc_router.c \
	lib/agents/safety/agent_enforcement.c \
	lib/agents/dispatch/agents_dispatch.c \
	lib/agents/roles/agent_code.c \
	lib/agents/roles/agent_historian.c \
	lib/agents/roles/agent_knowledge.c \
	lib/agents/roles/agent_system.c \
	lib/agents/roles/agent_validator.c \
	lib/orchestration/transport/brain_transport.c \
	lib/orchestration/transport/brain_protocol.c \
	lib/orchestration/transport/uds_server.c \
	lib/third_party/cjson/cJSON.c
ORCHESTRATION_SRCS := \
	lib/orchestration/planner/planner.c \
	lib/orchestration/workflow/rag_sessions.c \
	lib/orchestration/actions/rag_context_builder.c \
	lib/orchestration/actions/rag_prompts.c \
	lib/orchestration/execution/rag_pipeline.c
MESH_SRCS := \
	lib/mesh/identity/identity.c \
	lib/mesh/peer_registry/peer_registry.c \
	lib/mesh/membership/membership.c \
	lib/mesh/discovery/discovery.c \
	lib/mesh/awareness/awareness.c \
	lib/mesh/coordination/coordination.c \
	lib/mesh/transport/transport_state.c \
	lib/mesh/replay/replay_state.c \
	lib/mesh/conflict/conflict_state.c \
	lib/mesh/containment/containment_state.c \
	lib/mesh/enrollment/enrollment_state.c
PROVIDERS_SRCS := \
	lib/providers/registry/providers.c \
	lib/providers/registry/provider_registry.c \
	lib/providers/policy/provider_policy.c \
	lib/providers/selection/provider_selection.c \
	lib/providers/inference/client_inference.c \
	lib/providers/embedding/client_embedding.c \
	lib/providers/mocks/mock_provider.c \
	lib/providers/embedding/embedder_mock.c
KNOWLEDGE_SRCS := \
	lib/knowledge/runtime_compat.c \
	lib/knowledge/cognition/cognition.c \
	lib/knowledge/cognition/activation.c \
	lib/knowledge/cognition/reasoning/reasoning_roles.c \
	lib/knowledge/cognition/reasoning/scoring.c \
	lib/knowledge/memory/memory.c \
	lib/knowledge/memory/authority.c \
	lib/knowledge/memory/arena_store.c \
	lib/knowledge/memory/storage_bridge.c \
	lib/knowledge/episodic/episodic.c \
	lib/knowledge/semantic/semantic_db.c \
	lib/knowledge/vector/vector_index.c
DATA_SRCS := \
	lib/data/binding/store_binding.c \
	lib/data/binding/workspace_binding.c \
	lib/data/store/file_store.c \
	lib/data/store/duckdb_store.c \
	lib/data/records/event_records.c \
	lib/data/evidence/evidence_records.c \
	lib/data/query/inspect_query.c \
	lib/data/retention/retention.c \
	lib/data/lifecycle/archive.c
GRAPH_SRCS := \
	lib/graph/state/graph_backend.c \
	lib/graph/state/graph_backend_rpc.c \
	lib/graph/state/graph_facade.c \
	lib/graph/state/graph.c \
	lib/graph/state/ids.c \
	lib/graph/internal/counts.c \
	lib/graph/domains/activation.c \
	lib/graph/domains/authority.c \
	lib/graph/domains/episodic.c \
	lib/graph/domains/semantic.c \
	lib/graph/materialization/from_runtime_records.c \
	lib/graph/query/workspace_summary.c
EDGE_SRCS := \
	lib/edge/config.c \
	lib/edge/paths.c \
	lib/edge/runtime.c \
	lib/edge/edge_state.c \
	lib/edge/edge_services.c \
	lib/edge/edge_binding.c \
	lib/edge/action_point.c \
	lib/edge/local_runtime.c \
	lib/edge/lifecycle.c \
	lib/edge/internal.c \
	lib/edge/source_plane_model.c \
	lib/edge/source_ids.c \
	lib/third_party/cjson/cJSON.c

SUPPORT_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(SUPPORT_SRCS))
PLATFORM_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(PLATFORM_SRCS))
PROTOCOL_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(PROTOCOL_SRCS))
CORE_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(CORE_SRCS) $(GOVERNANCE_SRCS))
ORCHESTRATION_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(ORCHESTRATION_RUNTIME_SRCS) $(ORCHESTRATION_SRCS))
MESH_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(MESH_SRCS))
PROVIDERS_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(PROVIDERS_SRCS))
KNOWLEDGE_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(KNOWLEDGE_SRCS))
DATA_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(DATA_SRCS))
GRAPH_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(GRAPH_SRCS))
EDGE_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(EDGE_SRCS))

SUPPORT_LIB := $(LIB_DIR)/libyai_support.a
PLATFORM_LIB := $(LIB_DIR)/libyai_platform.a
PROTOCOL_LIB := $(LIB_DIR)/libyai_protocol.a
CORE_LIB := $(LIB_DIR)/libyai_core.a
ORCHESTRATION_LIB := $(LIB_DIR)/libyai_orchestration.a
MESH_LIB := $(LIB_DIR)/libyai_mesh.a
PROVIDERS_LIB := $(LIB_DIR)/libyai_providers.a
KNOWLEDGE_LIB := $(LIB_DIR)/libyai_knowledge.a
DATA_LIB := $(LIB_DIR)/libyai_data.a
GRAPH_LIB := $(LIB_DIR)/libyai_graph.a
EDGE_LIB := $(LIB_DIR)/libyai_edge.a

SPINE_DIRS := $(BIN_DIR) $(OBJ_DIR) $(LIB_DIR) $(TEST_DIR)

DOXYFILE := Doxyfile
DOXYGEN ?= doxygen
DOXY_OUT ?= $(DIST_ROOT)/docs/doxygen

.PHONY: all yai yai-edge yai-daemon foundations support platform protocol core orchestration exec mesh providers knowledge data graph edge daemon yd1-baseline \
        test test-unit test-integration test-e2e test-core test-runtime test-knowledge test-orchestration test-protocol test-governance test-providers test-edge test-mesh \
        test-demo-matrix verify-final-demo-matrix \
        clean clean-dist clean-all build build-all dist dist-all bundle verify \
        preflight-release docs docs-clean docs-verify proof-verify release-guards \
        release-guards-dev changelog-verify b13-convergence-check dirs help legacy-build \
        governance-sync governance-check

all: yai yai-edge foundations
	@echo "[YAI] unified binary spine ready: $(YAI_BIN) + $(YAI_EDGE_BIN)"

yai: $(YAI_BIN)
yai-edge: $(YAI_EDGE_BIN)
yai-daemon: yai-edge
	@cp "$(YAI_EDGE_BIN)" "$(YAI_DAEMON_ALIAS_BIN)"

foundations: support platform protocol mesh providers
core: $(CORE_LIB)
orchestration: $(ORCHESTRATION_LIB)
exec: orchestration
	@echo "[YAI] exec target is legacy alias; use 'make orchestration'"
providers: $(PROVIDERS_LIB)
mesh: $(MESH_LIB)
knowledge: $(KNOWLEDGE_LIB)
data: $(DATA_LIB)
graph: $(GRAPH_LIB)
edge: $(EDGE_LIB)
daemon: edge
	@echo "[YAI] daemon target is legacy alias; use 'make edge'"

support: $(SUPPORT_LIB)
platform: $(PLATFORM_LIB)
protocol: $(PROTOCOL_LIB)

test: test-unit test-integration test-e2e
	@echo "[YAI] unified test baseline complete"

test-unit: test-core test-runtime test-orchestration test-protocol test-knowledge test-governance test-providers test-edge test-mesh
	@echo "[YAI] unit suites complete"

test-integration:
	@tests/integration/runtime/run_runtime_exec_smoke.sh
	@tests/integration/orchestration/run_orchestration_smoke.sh
	@tests/integration/orchestration/run_orchestration_c_tests.sh
	@tests/integration/runtime/run_runtime_state_smoke.sh
	@tests/integration/edge/run_edge_smoke.sh
	@tests/integration/mesh/run_mesh_smoke.sh
	@tests/integration/workspace/workspace_runtime_contract_v1.sh
	@tests/integration/workspace/workspace_session_binding_contract_v1.sh
	@tests/integration/workspace/workspace_inspect_surfaces_v1.sh
	@tests/integration/workspace/workspace_real_flow_v1.sh
	@tests/integration/workspace/workspace_scientific_flow_v1.sh
	@tests/integration/workspace/workspace_digital_flow_v1.sh
	@tests/integration/workspace/workspace_event_surface_semantics_v1.sh
	@tests/integration/workspace/workspace_flow_state_readability_v1.sh
	@tests/integration/workspace/workspace_governed_vertical_slice_v1.sh
	@tests/integration/workspace/workspace_negative_paths_v1.sh
	@tests/integration/source-plane/source_owner_ingest_bridge_v1.sh
	@tests/integration/source-plane/edge_local_runtime_scan_spool_retry_v1.sh
	@tests/integration/source-plane/source_plane_read_model_v1.sh
	@tests/integration/runtime/run_runtime_handshake_smoke.sh
	@echo "[YAI] integration suites complete"

test-demo-matrix:
	@tests/integration/workspace/workspace_final_demo_matrix_v1.sh
	@echo "[YAI] final demo matrix suites complete"

verify-final-demo-matrix:
	@tools/dev/verify_final_demo_matrix.sh

test-e2e:
	@tests/e2e/run_entrypoint_e2e.sh
	@echo "[YAI] e2e suite complete"

test-core: yai
	@./build/bin/yai --help >/dev/null

test-runtime:
	@tests/unit/runtime/run_runtime_unit_tests.sh

test-knowledge:
	@tests/unit/knowledge/run_knowledge_unit_tests.sh

test-orchestration:
	@tests/unit/orchestration/run_orchestration_unit_tests.sh

test-protocol:
	@tests/unit/protocol/run_protocol_unit_tests.sh

test-governance:
	@tests/unit/governance/run_governance_unit_tests.sh
	@tests/integration/governance/run_governance_resolution_smoke.sh
	@echo "[YAI] governance-native resolution suites complete"

test-providers:
	@tests/unit/providers/run_providers_unit_tests.sh

test-edge:
	@tests/unit/edge/run_edge_unit_tests.sh

test-mesh:
	@tests/unit/mesh/run_mesh_unit_tests.sh

$(YAI_BIN): $(YAI_OBJ) $(CORE_LIB) $(ORCHESTRATION_LIB) $(KNOWLEDGE_LIB) $(PROVIDERS_LIB) $(DATA_LIB) $(GRAPH_LIB) $(EDGE_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(PROTOCOL_LIB) | dirs
	$(CC) $(LDFLAGS) $(YAI_OBJ) -o $@ $(CORE_LIB) $(ORCHESTRATION_LIB) $(KNOWLEDGE_LIB) $(PROVIDERS_LIB) $(DATA_LIB) $(GRAPH_LIB) $(EDGE_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(PROTOCOL_LIB) $(LDLIBS)

$(YAI_EDGE_BIN): $(YAI_EDGE_OBJ) $(EDGE_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) | dirs
	$(CC) $(LDFLAGS) $(YAI_EDGE_OBJ) -o $@ $(EDGE_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(LDLIBS)

$(SUPPORT_LIB): $(SUPPORT_OBJS) | dirs
	ar rcs $@ $^

$(PLATFORM_LIB): $(PLATFORM_OBJS) | dirs
	ar rcs $@ $^

$(PROTOCOL_LIB): $(PROTOCOL_OBJS) | dirs
	ar rcs $@ $^

$(CORE_LIB): $(CORE_OBJS) | dirs
	ar rcs $@ $^

$(ORCHESTRATION_LIB): $(ORCHESTRATION_OBJS) | dirs
	ar rcs $@ $^

$(MESH_LIB): $(MESH_OBJS) | dirs
	ar rcs $@ $^

$(PROVIDERS_LIB): $(PROVIDERS_OBJS) | dirs
	ar rcs $@ $^

$(KNOWLEDGE_LIB): $(KNOWLEDGE_OBJS) | dirs
	ar rcs $@ $^

$(DATA_LIB): $(DATA_OBJS) | dirs
	ar rcs $@ $^

$(GRAPH_LIB): $(GRAPH_OBJS) | dirs
	ar rcs $@ $^

$(EDGE_LIB): $(EDGE_OBJS) | dirs
	ar rcs $@ $^

$(OBJ_DIR)/%.o: %.c | dirs
	@mkdir -p $(dir $@)
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $< -o $@

dirs:
	@mkdir -p $(SPINE_DIRS)

build: yai yai-edge
	@echo "--- [YAI] primary entrypoints build complete (yai + yai-edge) ---"

yd1-baseline: yai yai-edge
	@echo "[YD-1] edge architecture refoundation baseline built"
	@echo "  owner runtime: build/bin/yai"
	@echo "  edge/source runtime: build/bin/yai-edge"
	@echo "  refs:"
	@echo "    docs/architecture/daemon-architecture-refoundation-model.md"
	@echo "    docs/program/22-adr/ADR-015-daemon-architecture-refoundation-slice.md"

legacy-build:
	@echo "--- [YAI] legacy-build removed: legacy top-level planes were decommissioned ---"

build-all: build
	@echo "--- [YAI] build-all complete (owner + edge baseline topology) ---"

dist: build
	@mkdir -p $(BIN_DIST)
	@cp "$(YAI_BIN)" "$(BIN_DIST)/yai"
	@if [ -f "$(YAI_EDGE_BIN)" ]; then cp "$(YAI_EDGE_BIN)" "$(BIN_DIST)/yai-edge"; cp "$(YAI_EDGE_BIN)" "$(BIN_DIST)/yai-daemon"; fi
	@echo "--- [YAI] dist staged in $(BIN_DIST) ---"

dist-all: dist
	@echo "--- [YAI] dist-all staged ---"

bundle: dist
	@tools/bin/yai-bundle

verify:
	@if [ -x ./tools/bin/yai-verify ]; then \
		./tools/bin/yai-verify; \
	else \
		echo "No verify script found at ./tools/bin/yai-verify"; \
	fi

governance-sync:
	@echo "governance sync: canonical governance/ tree is in-repo (no embedded export target)"

governance-check:
	@./tools/bin/yai-governance-compat-check

preflight-release:
	@tools/bin/yai-check-pins

docs:
	@mkdir -p $(DOXY_OUT)
	@$(DOXYGEN) $(DOXYFILE)
	@echo "✔ Doxygen: $(DOXY_OUT)/html/index.html"

docs-clean:
	@rm -rf $(DOXY_OUT)

docs-verify:
	@tools/bin/yai-docs-trace-check --all

proof-verify:
	@tools/bin/yai-proof-check

release-guards:
	@tools/bin/yai-check-pins
	@tools/bin/yai-proof-check

release-guards-dev:
	@STRICT_SPECS_HEAD=0 tools/bin/yai-check-pins
	@tools/bin/yai-proof-check

changelog-verify:
	@BASE_SHA="$$(git rev-parse origin/main)"; \
	HEAD_SHA="$$(git rev-parse HEAD)"; \
	tools/bin/yai-changelog-check --pr --base "$$BASE_SHA" --head "$$HEAD_SHA"

b13-convergence-check:
	@tools/release/unified_repo_convergence_smoke.sh

clean:
	rm -rf $(BUILD_DIR)

clean-dist:
	rm -rf $(DIST_ROOT)

clean-all: clean clean-dist

help:
	@echo "Primary build targets:"
	@echo "  all            (yai + yai-edge + foundation libs)"
	@echo "  yai            (build/bin/yai)"
	@echo "  yai-edge       (build/bin/yai-edge standalone edge/source runtime)"
	@echo "  yai-daemon     (legacy alias of build/bin/yai-edge)"
	@echo "  yd1-baseline   (build anchors + YD-1 architecture refs)"
	@echo "  daemon         (legacy alias; use edge)"
	@echo "  orchestration  (build orchestration control archive)"
	@echo "  exec           (legacy alias; use orchestration)"
	@echo "  foundations    (support/platform/protocol/providers archives)"
	@echo "  providers      (build provider infrastructure archive)"
	@echo "  test-unit      (core/orchestration/protocol/knowledge/governance unit suites)"
	@echo "  test-integration (runtime/orchestration/workspace/source-plane integration suites)"
	@echo "  test-knowledge (knowledge unit suite)"
	@echo "  test-orchestration (orchestration unit suite)"
	@echo "  test-protocol  (protocol unit suite)"
	@echo "  test-governance  (governance loader/discovery/resolution + smoke)"
	@echo "  test-e2e       (entrypoint e2e smoke)"
	@echo "  test           (full test baseline)"
	@echo "  b13-convergence-check (single-repo final convergence smoke)"
	@echo "  clean          (remove build artifacts)"
	@echo "  dist, dist-all, bundle"
	@echo "  verify, docs, docs-verify, proof-verify, release-guards, changelog-verify"


test-vertical-slice:
	@tests/integration/workspace/workspace_governed_vertical_slice_v1.sh
	@echo "[YAI] governed vertical slice complete"
