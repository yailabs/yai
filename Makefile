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
CPPFLAGS ?= -I$(ROOT_DIR) -I$(ROOT_DIR)/include -I$(ROOT_DIR)/include/yai \
            -I$(ROOT_DIR)/lib/third_party/cjson \
            -I$(LAW_COMPAT_ROOT)/contracts/protocol/include \
            -I$(LAW_COMPAT_ROOT)/contracts/protocol/runtime/include \
            -I$(LAW_COMPAT_ROOT)/contracts/vault/include
CFLAGS ?= -Wall -Wextra -std=c11 -O2
LDFLAGS ?=
LDLIBS ?= -lm

YAI_OBJ := $(OBJ_DIR)/cmd/yai/main.o
YAI_BIN := $(BIN_DIR)/yai

SUPPORT_SRCS := lib/support/ids.c lib/support/logger.c lib/support/errors.c lib/support/strings.c lib/support/paths.c
PLATFORM_SRCS := lib/platform/os.c lib/platform/fs.c lib/platform/clock.c lib/platform/uds.c
PROTOCOL_SRCS := lib/protocol/rpc_runtime.c lib/protocol/rpc_codec.c lib/protocol/rpc_binary.c lib/protocol/message_types.c
CORE_SRCS := \
	lib/core/lifecycle/bootstrap.c \
	lib/core/lifecycle/preboot.c \
	lib/core/lifecycle/startup_plan.c \
	lib/core/dispatch/control_transport.c \
	lib/core/dispatch/command_dispatch.c \
	lib/core/dispatch/attach_flow.c \
	lib/core/session/session.c \
	lib/core/session/session_reply.c \
	lib/core/session/session_utils.c \
	lib/core/workspace/project_tree.c \
	lib/core/workspace/workspace_registry.c \
	lib/core/workspace/workspace_runtime.c \
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
	lib/exec/gates/provider_gate.c \
	lib/exec/gates/network_gate.c \
	lib/exec/gates/storage_gate.c \
	lib/exec/gates/resource_gate.c \
	lib/exec/bridge/engine_bridge.c \
	lib/exec/bridge/transport_client.c \
	lib/exec/bridge/rpc_router.c \
	lib/exec/agents/agent_enforcement.c \
	lib/third_party/cjson/cJSON.c
BRAIN_SRCS := \
	lib/brain/lifecycle/brain_lifecycle.c \
	lib/brain/cognition/cognition.c \
	lib/brain/cognition/agents/agents_dispatch.c \
	lib/brain/cognition/agents/agent_code.c \
	lib/brain/cognition/agents/agent_historian.c \
	lib/brain/cognition/agents/agent_knowledge.c \
	lib/brain/cognition/agents/agent_system.c \
	lib/brain/cognition/agents/agent_validator.c \
	lib/brain/cognition/orchestration/planner.c \
	lib/brain/cognition/orchestration/rag_sessions.c \
	lib/brain/cognition/orchestration/rag_context_builder.c \
	lib/brain/cognition/orchestration/rag_prompts.c \
	lib/brain/cognition/orchestration/rag_pipeline.c \
	lib/brain/cognition/reasoning/reasoning_roles.c \
	lib/brain/cognition/reasoning/scoring.c \
	lib/brain/memory/memory.c \
	lib/brain/memory/arena_store.c \
	lib/brain/memory/storage_bridge.c \
	lib/brain/memory/graph/graph_backend.c \
	lib/brain/memory/graph/graph_backend_rpc.c \
	lib/brain/memory/graph/graph_facade.c \
	lib/brain/memory/graph/graph.c \
	lib/brain/memory/graph/ids.c \
	lib/brain/memory/graph/domain_activation.c \
	lib/brain/memory/graph/domain_authority.c \
	lib/brain/memory/graph/domain_episodic.c \
	lib/brain/memory/graph/domain_semantic.c \
	lib/brain/memory/graph/semantic_db.c \
	lib/brain/memory/graph/vector_index.c \
	lib/brain/bridge/providers.c \
	lib/brain/bridge/provider_registry.c \
	lib/brain/bridge/client_bridge.c \
	lib/brain/bridge/mock_provider.c \
	lib/brain/bridge/embedder_mock.c \
	lib/brain/transport/brain_transport.c \
	lib/brain/transport/brain_protocol.c \
	lib/brain/transport/uds_server.c

SUPPORT_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(SUPPORT_SRCS))
PLATFORM_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(PLATFORM_SRCS))
PROTOCOL_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(PROTOCOL_SRCS))
CORE_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(CORE_SRCS) $(LAW_SRCS))
EXEC_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(EXEC_SRCS))
BRAIN_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(BRAIN_SRCS))

SUPPORT_LIB := $(LIB_DIR)/libyai_support.a
PLATFORM_LIB := $(LIB_DIR)/libyai_platform.a
PROTOCOL_LIB := $(LIB_DIR)/libyai_protocol.a
CORE_LIB := $(LIB_DIR)/libyai_core.a
EXEC_LIB := $(LIB_DIR)/libyai_exec.a
BRAIN_LIB := $(LIB_DIR)/libyai_brain.a

SPINE_DIRS := $(BIN_DIR) $(OBJ_DIR) $(LIB_DIR) $(TEST_DIR)

DOXYFILE := Doxyfile
DOXYGEN ?= doxygen
DOXY_OUT ?= $(DIST_ROOT)/docs/doxygen

.PHONY: all yai foundations support platform protocol core exec brain \
        test test-unit test-integration test-e2e test-core test-brain test-protocol test-law \
        clean clean-dist clean-all build build-all dist dist-all bundle verify \
        preflight-release docs docs-clean docs-verify proof-verify release-guards \
        release-guards-dev changelog-verify dirs help legacy-build \
        law-embed-sync law-embed-check

all: yai foundations
	@echo "[YAI] unified binary spine ready: $(YAI_BIN)"

yai: $(YAI_BIN)

foundations: support platform protocol
core: $(CORE_LIB)
exec: $(EXEC_LIB)
brain: $(BRAIN_LIB)

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
	@tests/integration/runtime_handshake/run_runtime_handshake_smoke.sh
	@echo "[YAI] integration suites complete"

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

$(YAI_BIN): $(YAI_OBJ) $(CORE_LIB) $(EXEC_LIB) $(BRAIN_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(PROTOCOL_LIB) | dirs
	$(CC) $(LDFLAGS) $(YAI_OBJ) -o $@ $(CORE_LIB) $(EXEC_LIB) $(BRAIN_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(PROTOCOL_LIB) $(LDLIBS)

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

$(BRAIN_LIB): $(BRAIN_OBJS) | dirs
	ar rcs $@ $^

$(OBJ_DIR)/%.o: %.c | dirs
	@mkdir -p $(dir $@)
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $< -o $@

dirs:
	@mkdir -p $(SPINE_DIRS)

build: yai
	@echo "--- [YAI] primary entrypoint build complete (yai) ---"

legacy-build:
	@echo "--- [YAI] legacy-build removed: legacy top-level planes were decommissioned ---"

build-all: build
	@echo "--- [YAI] build-all complete (primary topology only) ---"

dist: build
	@mkdir -p $(BIN_DIST)
	@cp "$(YAI_BIN)" "$(BIN_DIST)/yai"
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
	@echo "  all            (yai + foundation libs)"
	@echo "  yai            (build/bin/yai)"
	@echo "  foundations    (support/platform/protocol archives)"
	@echo "  test-unit      (core/protocol/brain unit suites)"
	@echo "  test-integration (runtime/core-exec/core-brain/workspace)"
	@echo "  test-law         (law loader/discovery/resolution + smoke)"
	@echo "  test-e2e       (entrypoint e2e smoke)"
	@echo "  test           (full test baseline)"
	@echo "  clean          (remove build artifacts)"
	@echo "  dist, dist-all, bundle"
	@echo "  verify, docs, docs-verify, proof-verify, release-guards, changelog-verify"
