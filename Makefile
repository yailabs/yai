# =========================================
# YAI — Unified Root Build Spine
# =========================================

ROOT_DIR := $(abspath .)
LAW_COMPAT_ROOT ?= $(if $(wildcard $(ROOT_DIR)/../law/contracts),$(ROOT_DIR)/../law,$(ROOT_DIR)/embedded/law)

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
            -I$(LAW_COMPAT_ROOT)/contracts/protocol/include \
            -I$(LAW_COMPAT_ROOT)/contracts/protocol/runtime/include \
            -I$(LAW_COMPAT_ROOT)/contracts/vault/include
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
YAI_DAEMON_OBJ := $(OBJ_DIR)/cmd/yai-daemon/main.o
YAI_DAEMON_BIN := $(BIN_DIR)/yai-daemon

SUPPORT_SRCS := lib/support/ids.c lib/support/logger.c lib/support/errors.c lib/support/strings.c lib/support/paths.c
PLATFORM_SRCS := lib/platform/os.c lib/platform/fs.c lib/platform/clock.c lib/platform/uds.c
PROTOCOL_SRCS := lib/protocol/rpc_runtime.c lib/protocol/rpc_codec.c lib/protocol/rpc_binary.c lib/protocol/message_types.c lib/protocol/source_plane_contract.c
CORE_SRCS := \
	lib/core/lifecycle/bootstrap.c \
	lib/core/lifecycle/preboot.c \
	lib/core/lifecycle/startup_plan.c \
	lib/core/lifecycle/runtime_capabilities.c \
	lib/core/dispatch/control_transport.c \
	lib/core/dispatch/command_dispatch.c \
	lib/core/dispatch/attach_flow.c \
	lib/core/session/session.c \
	lib/core/session/session_reply.c \
	lib/core/session/session_utils.c \
	lib/core/workspace/project_tree.c \
	lib/core/workspace/workspace_registry.c \
	lib/core/workspace/workspace_runtime.c \
	lib/core/workspace/workspace_binding.c \
	lib/core/workspace/workspace_recovery.c \
	lib/core/enforcement/enforcement.c \
	lib/core/authority/authority_registry.c \
	lib/core/authority/identity.c
LAW_SRCS := \
	lib/law/policy_effects.c \
	lib/law/loader/law_loader.c \
	lib/law/loader/manifest_loader.c \
	lib/law/loader/domain_loader.c \
	lib/law/loader/compliance_loader.c \
	lib/law/loader/overlay_loader.c \
	lib/law/loader/compatibility_check.c \
	lib/law/classification/event_classifier.c \
	lib/law/classification/action_classifier.c \
	lib/law/classification/provider_classifier.c \
	lib/law/classification/resource_classifier.c \
	lib/law/classification/protocol_classifier.c \
	lib/law/classification/workspace_context.c \
	lib/law/discovery/domain_discovery.c \
	lib/law/discovery/signal_matcher.c \
	lib/law/discovery/protocol_matcher.c \
	lib/law/discovery/provider_matcher.c \
	lib/law/discovery/resource_matcher.c \
	lib/law/discovery/command_matcher.c \
	lib/law/discovery/confidence_model.c \
	lib/law/resolution/resolver.c \
	lib/law/resolution/stack_builder.c \
	lib/law/resolution/foundation_merge.c \
	lib/law/resolution/domain_merge.c \
	lib/law/resolution/compliance_merge.c \
	lib/law/resolution/overlay_merge.c \
	lib/law/resolution/precedence.c \
	lib/law/resolution/fallback.c \
	lib/law/resolution/conflict_resolution.c \
	lib/law/resolution/effective_stack.c \
	lib/law/mapping/event_to_domain.c \
	lib/law/mapping/domain_to_policy.c \
	lib/law/mapping/policy_to_effect.c \
	lib/law/mapping/decision_to_evidence.c \
	lib/law/mapping/decision_to_audit.c \
	lib/law/debug/resolution_trace.c \
	lib/law/debug/dump_effective_stack.c \
	lib/law/debug/dump_discovery_result.c
EXEC_SRCS := \
	lib/exec/runtime/exec_runtime.c \
	lib/exec/runtime/config_loader.c \
	lib/exec/runtime/runtime_model.c \
	lib/exec/runtime/source_plane_contract.c \
	lib/exec/runtime/peer_registry.c \
	lib/exec/runtime/source_ingest.c \
	lib/exec/gates/provider_gate.c \
	lib/exec/gates/network_gate.c \
	lib/exec/gates/storage_gate.c \
	lib/exec/gates/resource_gate.c \
	lib/exec/bridge/engine_bridge.c \
	lib/exec/bridge/client_bridge.c \
	lib/exec/bridge/transport_client.c \
	lib/exec/bridge/rpc_router.c \
	lib/exec/agents/agent_enforcement.c \
	lib/exec/agents/agents_dispatch.c \
	lib/exec/agents/agent_code.c \
	lib/exec/agents/agent_historian.c \
	lib/exec/agents/agent_knowledge.c \
	lib/exec/agents/agent_system.c \
	lib/exec/agents/agent_validator.c \
	lib/exec/orchestration/planner.c \
	lib/exec/orchestration/rag_sessions.c \
	lib/exec/orchestration/rag_context_builder.c \
	lib/exec/orchestration/rag_prompts.c \
	lib/exec/orchestration/rag_pipeline.c \
	lib/exec/transport/brain_transport.c \
	lib/exec/transport/brain_protocol.c \
	lib/exec/transport/uds_server.c \
	lib/third_party/cjson/cJSON.c
KNOWLEDGE_SRCS := \
	lib/knowledge/runtime_compat.c \
	lib/knowledge/cognition/cognition.c \
	lib/knowledge/cognition/reasoning/reasoning_roles.c \
	lib/knowledge/cognition/reasoning/scoring.c \
	lib/knowledge/memory/memory.c \
	lib/knowledge/memory/arena_store.c \
	lib/knowledge/memory/storage_bridge.c \
	lib/knowledge/memory/semantic_db.c \
	lib/knowledge/memory/vector_index.c \
	lib/knowledge/providers/providers.c \
	lib/knowledge/providers/provider_registry.c \
	lib/knowledge/providers/mock_provider.c \
	lib/knowledge/providers/embedder_mock.c
DATA_SRCS := \
	lib/data/binding/store_binding.c \
	lib/data/binding/workspace_binding.c \
	lib/data/store/file_store.c \
	lib/data/store/duckdb_store.c \
	lib/data/records/event_records.c \
	lib/data/query/inspect_query.c \
	lib/data/lifecycle/retention.c \
	lib/data/lifecycle/archive.c
GRAPH_SRCS := \
	lib/graph/state/graph_backend.c \
	lib/graph/state/graph_backend_rpc.c \
	lib/graph/state/graph_facade.c \
	lib/graph/state/graph.c \
	lib/graph/state/ids.c \
	lib/graph/domains/activation.c \
	lib/graph/domains/authority.c \
	lib/graph/domains/episodic.c \
	lib/graph/domains/semantic.c \
	lib/graph/materialization/from_runtime_records.c \
	lib/graph/query/workspace_summary.c
DAEMON_SRCS := \
	lib/daemon/config.c \
	lib/daemon/paths.c \
	lib/daemon/runtime.c \
	lib/daemon/edge_state.c \
	lib/daemon/edge_services.c \
	lib/daemon/edge_binding.c \
	lib/daemon/action_point.c \
	lib/daemon/local_runtime.c \
	lib/daemon/lifecycle.c \
	lib/daemon/internal.c \
	lib/daemon/source_plane_model.c \
	lib/daemon/source_ids.c \
	lib/third_party/cjson/cJSON.c

SUPPORT_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(SUPPORT_SRCS))
PLATFORM_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(PLATFORM_SRCS))
PROTOCOL_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(PROTOCOL_SRCS))
CORE_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(CORE_SRCS) $(LAW_SRCS))
EXEC_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(EXEC_SRCS))
KNOWLEDGE_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(KNOWLEDGE_SRCS))
DATA_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(DATA_SRCS))
GRAPH_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(GRAPH_SRCS))
DAEMON_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(DAEMON_SRCS))

SUPPORT_LIB := $(LIB_DIR)/libyai_support.a
PLATFORM_LIB := $(LIB_DIR)/libyai_platform.a
PROTOCOL_LIB := $(LIB_DIR)/libyai_protocol.a
CORE_LIB := $(LIB_DIR)/libyai_core.a
EXEC_LIB := $(LIB_DIR)/libyai_exec.a
KNOWLEDGE_LIB := $(LIB_DIR)/libyai_knowledge.a
DATA_LIB := $(LIB_DIR)/libyai_data.a
GRAPH_LIB := $(LIB_DIR)/libyai_graph.a
DAEMON_LIB := $(LIB_DIR)/libyai_daemon.a

SPINE_DIRS := $(BIN_DIR) $(OBJ_DIR) $(LIB_DIR) $(TEST_DIR)

DOXYFILE := Doxyfile
DOXYGEN ?= doxygen
DOXY_OUT ?= $(DIST_ROOT)/docs/doxygen

.PHONY: all yai yai-daemon foundations support platform protocol core exec knowledge data graph daemon yd1-baseline \
        test test-unit test-integration test-e2e test-core test-brain test-protocol test-law \
        test-demo-matrix verify-final-demo-matrix \
        clean clean-dist clean-all build build-all dist dist-all bundle verify \
        preflight-release docs docs-clean docs-verify proof-verify release-guards \
        release-guards-dev changelog-verify dirs help legacy-build \
        law-embed-sync law-embed-check

all: yai yai-daemon foundations
	@echo "[YAI] unified binary spine ready: $(YAI_BIN) + $(YAI_DAEMON_BIN)"

yai: $(YAI_BIN)
yai-daemon: $(YAI_DAEMON_BIN)

foundations: support platform protocol
core: $(CORE_LIB)
exec: $(EXEC_LIB)
knowledge: $(KNOWLEDGE_LIB)
data: $(DATA_LIB)
graph: $(GRAPH_LIB)
daemon: $(DAEMON_LIB)

support: $(SUPPORT_LIB)
platform: $(PLATFORM_LIB)
protocol: $(PROTOCOL_LIB)

test: test-unit test-integration test-e2e
	@echo "[YAI] unified test baseline complete"

test-unit: test-core test-protocol test-brain test-law
	@echo "[YAI] unit suites complete"

test-integration:
	@tests/integration/core_exec/run_core_exec_smoke.sh
	@tests/integration/core_brain/run_core_brain_smoke.sh
	@tests/integration/core_brain/run_core_brain_c_tests.sh
	@tests/integration/workspace_lifecycle/workspace_runtime_contract_v1.sh
	@tests/integration/workspace_lifecycle/workspace_session_binding_contract_v1.sh
	@tests/integration/workspace_lifecycle/workspace_inspect_surfaces_v1.sh
	@tests/integration/workspace_lifecycle/workspace_real_flow_v1.sh
	@tests/integration/workspace_lifecycle/workspace_scientific_flow_v1.sh
	@tests/integration/workspace_lifecycle/workspace_digital_flow_v1.sh
	@tests/integration/workspace_lifecycle/workspace_event_surface_semantics_v1.sh
	@tests/integration/workspace_lifecycle/workspace_flow_state_readability_v1.sh
	@tests/integration/workspace_lifecycle/workspace_governed_vertical_slice_v1.sh
	@tests/integration/workspace_lifecycle/workspace_negative_paths_v1.sh
	@tests/integration/source_plane/source_owner_ingest_bridge_v1.sh
	@tests/integration/source_plane/daemon_local_runtime_scan_spool_retry_v1.sh
	@tests/integration/source_plane/source_plane_read_model_v1.sh
	@tests/integration/runtime_handshake/run_runtime_handshake_smoke.sh
	@echo "[YAI] integration suites complete"

test-demo-matrix:
	@tests/integration/workspace_lifecycle/workspace_final_demo_matrix_v1.sh
	@echo "[YAI] final demo matrix suites complete"

verify-final-demo-matrix:
	@tools/dev/verify_final_demo_matrix.sh

test-e2e:
	@tests/e2e/run_entrypoint_e2e.sh
	@echo "[YAI] e2e suite complete"

test-core: yai
	@./build/bin/yai --help >/dev/null

test-brain:
	@tests/unit/brain/run_brain_unit_tests.sh

test-protocol:
	@tests/unit/exec/run_exec_unit_tests.sh
	@tests/unit/protocol/run_protocol_unit_tests.sh

test-law:
	@tests/unit/law/run_law_unit_tests.sh
	@tests/integration/law_resolution/run_law_resolution_smoke.sh
	@echo "[YAI] law-native resolution suites complete"

$(YAI_BIN): $(YAI_OBJ) $(CORE_LIB) $(EXEC_LIB) $(KNOWLEDGE_LIB) $(DATA_LIB) $(GRAPH_LIB) $(DAEMON_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(PROTOCOL_LIB) | dirs
	$(CC) $(LDFLAGS) $(YAI_OBJ) -o $@ $(CORE_LIB) $(EXEC_LIB) $(KNOWLEDGE_LIB) $(DATA_LIB) $(GRAPH_LIB) $(DAEMON_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(PROTOCOL_LIB) $(LDLIBS)

$(YAI_DAEMON_BIN): $(YAI_DAEMON_OBJ) $(DAEMON_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) | dirs
	$(CC) $(LDFLAGS) $(YAI_DAEMON_OBJ) -o $@ $(DAEMON_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(LDLIBS)

$(SUPPORT_LIB): $(SUPPORT_OBJS) | dirs
	ar rcs $@ $^

$(PLATFORM_LIB): $(PLATFORM_OBJS) | dirs
	ar rcs $@ $^

$(PROTOCOL_LIB): $(PROTOCOL_OBJS) | dirs
	ar rcs $@ $^

$(CORE_LIB): $(CORE_OBJS) | dirs
	ar rcs $@ $^

$(EXEC_LIB): $(EXEC_OBJS) | dirs
	ar rcs $@ $^

$(KNOWLEDGE_LIB): $(KNOWLEDGE_OBJS) | dirs
	ar rcs $@ $^

$(DATA_LIB): $(DATA_OBJS) | dirs
	ar rcs $@ $^

$(GRAPH_LIB): $(GRAPH_OBJS) | dirs
	ar rcs $@ $^

$(DAEMON_LIB): $(DAEMON_OBJS) | dirs
	ar rcs $@ $^

$(OBJ_DIR)/%.o: %.c | dirs
	@mkdir -p $(dir $@)
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $< -o $@

dirs:
	@mkdir -p $(SPINE_DIRS)

build: yai yai-daemon
	@echo "--- [YAI] primary entrypoints build complete (yai + yai-daemon) ---"

yd1-baseline: yai yai-daemon
	@echo "[YD-1] daemon architecture refoundation baseline built"
	@echo "  owner runtime: build/bin/yai"
	@echo "  edge/source runtime: build/bin/yai-daemon"
	@echo "  refs:"
	@echo "    docs/architecture/daemon-architecture-refoundation-model.md"
	@echo "    docs/program/22-adr/ADR-015-daemon-architecture-refoundation-slice.md"

legacy-build:
	@echo "--- [YAI] legacy-build removed: legacy top-level planes were decommissioned ---"

build-all: build
	@echo "--- [YAI] build-all complete (owner + daemon baseline topology) ---"

dist: build
	@mkdir -p $(BIN_DIST)
	@cp "$(YAI_BIN)" "$(BIN_DIST)/yai"
	@if [ -f "$(YAI_DAEMON_BIN)" ]; then cp "$(YAI_DAEMON_BIN)" "$(BIN_DIST)/yai-daemon"; fi
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

law-embed-sync:
	@./tools/bin/yai-law-embed-sync

law-embed-check:
	@./tools/bin/yai-law-compat-check

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

clean:
	rm -rf $(BUILD_DIR)

clean-dist:
	rm -rf $(DIST_ROOT)

clean-all: clean clean-dist

help:
	@echo "Primary build targets:"
	@echo "  all            (yai + yai-daemon + foundation libs)"
	@echo "  yai            (build/bin/yai)"
	@echo "  yai-daemon     (build/bin/yai-daemon standalone edge/source runtime)"
	@echo "  yd1-baseline   (build anchors + YD-1 architecture refs)"
	@echo "  daemon         (build daemon static library)"
	@echo "  foundations    (support/platform/protocol archives)"
	@echo "  test-unit      (core/protocol/knowledge+graph unit suites)"
	@echo "  test-integration (runtime/core-exec/workspace + legacy core_brain scripts)"
	@echo "  test-law         (law loader/discovery/resolution + smoke)"
	@echo "  test-e2e       (entrypoint e2e smoke)"
	@echo "  test           (full test baseline)"
	@echo "  clean          (remove build artifacts)"
	@echo "  dist, dist-all, bundle"
	@echo "  verify, docs, docs-verify, proof-verify, release-guards, changelog-verify"


test-vertical-slice:
	@tests/integration/workspace_lifecycle/workspace_governed_vertical_slice_v1.sh
	@echo "[YAI] governed vertical slice complete"
