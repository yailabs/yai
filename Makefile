# =========================================
# YAI — Unified Root Build Spine
# =========================================

ROOT_DIR := $(abspath .)
PROTOCOL_CONTRACT_ROOT ?= $(ROOT_DIR)/include/yai/protocol
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
CPPFLAGS ?= -I$(ROOT_DIR) -I$(ROOT_DIR)/include -I$(ROOT_DIR)/kernel/include \
            -I$(ROOT_DIR)/third_party/cjson \
            -I$(PROTOCOL_CONTRACT_ROOT)
CPPFLAGS += -I$(ROOT_DIR)/user/include \
            -I$(ROOT_DIR)/sdk/c/libyai/include \
            -I$(ROOT_DIR)/sdk/c/libyai/third_party/cjson
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

YAI_CLI_MAIN_OBJ := $(OBJ_DIR)/user/bin/yai/main.o
YAI_CTL_MAIN_OBJ := $(OBJ_DIR)/user/bin/yai-ctl/main.o
YAI_SH_MAIN_OBJ := $(OBJ_DIR)/user/bin/yai-sh/main.o
YAI_BIN := $(BIN_DIR)/yai
YAI_CTL_BIN := $(BIN_DIR)/yai-ctl
YAI_SH_BIN := $(BIN_DIR)/yai-sh
YAI_DAEMON_OBJ := $(OBJ_DIR)/sys/daemon/yai-daemond/main.o
YAI_DAEMOND_BIN := $(BIN_DIR)/yai-daemond
YAI_DAEMON_BIN := $(BIN_DIR)/yai-daemon
YAI_CONTAINERD_OBJ := $(OBJ_DIR)/sys/container/yai-containerd/main.o
YAI_CONTAINERD_BIN := $(BIN_DIR)/yai-containerd
YAI_GRAPHD_OBJ := $(OBJ_DIR)/sys/graph/yai-graphd/main.o
YAI_GRAPHD_BIN := $(BIN_DIR)/yai-graphd
YAI_DATAD_OBJ := $(OBJ_DIR)/sys/data/yai-datad/main.o
YAI_DATAD_BIN := $(BIN_DIR)/yai-datad
YAI_NETD_OBJ := $(OBJ_DIR)/sys/network/yai-netd/main.o
YAI_NETD_BIN := $(BIN_DIR)/yai-netd
YAI_ORCHESTRATORD_OBJ := $(OBJ_DIR)/sys/orchestration/yai-orchestratord/main.o
YAI_ORCHESTRATORD_BIN := $(BIN_DIR)/yai-orchestratord
YAI_POLICYD_OBJ := $(OBJ_DIR)/sys/policy/yai-policyd/main.o
YAI_POLICYD_BIN := $(BIN_DIR)/yai-policyd
YAI_GOVERNANCED_OBJ := $(OBJ_DIR)/sys/governance/yai-governanced/main.o
YAI_GOVERNANCED_BIN := $(BIN_DIR)/yai-governanced
YAI_METRICSD_OBJ := $(OBJ_DIR)/sys/observability/yai-metricsd/main.o
YAI_METRICSD_BIN := $(BIN_DIR)/yai-metricsd
YAI_AUDITD_OBJ := $(OBJ_DIR)/sys/observability/yai-auditd/main.o
YAI_AUDITD_BIN := $(BIN_DIR)/yai-auditd
YAI_SUPERVISORD_OBJ := $(OBJ_DIR)/sys/supervisor/yai-supervisord/main.o
YAI_SUPERVISORD_BIN := $(BIN_DIR)/yai-supervisord
YAI_EDGE_ALIAS_BIN := $(BIN_DIR)/yai-edge

SUPPORT_SRCS := kernel/support/core/ids.c kernel/support/core/logger.c kernel/support/core/errors.c kernel/support/core/strings.c kernel/support/core/paths.c
PLATFORM_SRCS := kernel/platform/core/os.c kernel/platform/core/fs.c kernel/platform/core/clock.c kernel/platform/core/uds.c
PROTOCOL_SRCS := \
	protocol/lib/core/rpc/runtime.c \
	protocol/lib/core/rpc/codec.c \
	protocol/lib/core/binary/rpc_binary.c \
	protocol/lib/core/control/source_plane.c \
	protocol/lib/core/control/message_types.c
CORE_SRCS := \
	kernel/lifecycle/system_shm_bootstrap.c \
	sys/supervisor/lifecycle/runtime_preboot.c \
	sys/supervisor/lifecycle/startup_plan.c \
	sys/supervisor/lifecycle/runtime_capabilities.c \
	sys/services/dispatch/control_transport.c \
	sys/services/dispatch/command_dispatch.c \
	sys/services/dispatch/attach_flow.c \
	sys/services/registry/registry.c \
	sys/services/sockets/sockets.c \
	sys/services/manifests/manifests.c \
	sys/container/session/scope/project_tree.c \
	sys/container/session/scope/scope_registry.c \
	sys/container/session/scope/scope_runtime.c \
	sys/container/session/scope/scope_binding.c \
	sys/container/session/scope/scope_recovery.c \
	sys/policy/state/policy_state.c \
	sys/policy/state/grants_state.c \
	sys/policy/state/containment_state.c \
	sys/policy/engine/engine.c \
	sys/policy/grants/grants.c \
	sys/policy/overlays/overlays.c \
	sys/policy/review/review.c \
	sys/policy/enforcement/enforcement.c \
	sys/supervisor/admission/admission.c \
	sys/supervisor/recovery/recovery.c \
	sys/supervisor/registry/registry.c \
	sys/observability/metrics/metrics.c \
	sys/observability/traces/traces.c \
	sys/observability/reporting/reporting.c \
	kernel/policy/authority_registry.c \
	sys/policy/enforcement/authority_policy_gate.c
GOVERNANCE_SRCS := \
	sys/governance/authority/authority.c \
	sys/governance/escalation/escalation.c \
	sys/governance/decisions/decisions.c \
	sys/governance/publication/publication.c \
	sys/governance/core/policy_effects.c \
	sys/governance/core/loader/governance_loader.c \
	sys/governance/core/loader/manifest_loader.c \
	sys/governance/core/loader/domain_model_matrix.c \
	sys/governance/core/loader/domain_loader.c \
	sys/governance/core/loader/compliance_loader.c \
	sys/governance/core/loader/overlay_loader.c \
	sys/governance/core/loader/compatibility_check.c \
	sys/governance/core/classification/event_classifier.c \
	sys/governance/core/classification/action_classifier.c \
	sys/governance/core/classification/provider_classifier.c \
	sys/governance/core/classification/resource_classifier.c \
	sys/governance/core/classification/protocol_classifier.c \
	sys/governance/core/classification/workspace_context.c \
	sys/governance/core/discovery/domain_discovery.c \
	sys/governance/core/discovery/signal_matcher.c \
	sys/governance/core/discovery/protocol_matcher.c \
	sys/governance/core/discovery/provider_matcher.c \
	sys/governance/core/discovery/resource_matcher.c \
	sys/governance/core/discovery/command_matcher.c \
	sys/governance/core/discovery/confidence_model.c \
	sys/governance/core/resolution/resolver.c \
	sys/governance/core/resolution/stack_builder.c \
	sys/governance/core/resolution/foundation_merge.c \
	sys/governance/core/resolution/domain_merge.c \
	sys/governance/core/resolution/compliance_merge.c \
	sys/governance/core/resolution/overlay_merge.c \
	sys/governance/core/resolution/precedence.c \
	sys/governance/core/resolution/fallback.c \
	sys/governance/core/resolution/conflict_resolution.c \
	sys/governance/core/resolution/effective_stack.c \
	sys/governance/core/mapping/event_to_domain.c \
	sys/governance/core/mapping/domain_to_policy.c \
	sys/governance/core/mapping/policy_to_effect.c \
	sys/governance/core/mapping/decision_to_evidence.c \
	sys/governance/core/mapping/decision_to_audit.c \
	sys/governance/core/debug/resolution_trace.c \
	sys/governance/core/debug/dump_effective_stack.c \
	sys/governance/core/debug/dump_discovery_result.c
ORCHESTRATION_RUNTIME_SRCS := \
	sys/orchestration/internal/runtime/runtime_control.c \
	sys/orchestration/internal/runtime/config_loader.c \
	sys/orchestration/internal/orchestration_model.c \
	sys/orchestration/internal/runtime/grounding_context.c \
	sys/orchestration/internal/bridge/network_bridge.c \
	sys/orchestration/internal/runtime/peer_registry_bridge.c \
	sys/orchestration/internal/runtime/ingestion.c \
	sys/orchestration/internal/bridge/storage_bridge.c \
	sys/orchestration/internal/bridge/runtime_bridge.c \
	sys/orchestration/internal/bridge/transport_client.c \
	sys/orchestration/internal/bridge/rpc_router.c \
	sys/orchestration/internal/agents/core/safety/agent_enforcement.c \
	sys/orchestration/internal/agents/core/dispatch/agents_dispatch.c \
	sys/orchestration/internal/agents/core/roles/agent_code.c \
	sys/orchestration/internal/agents/core/roles/agent_historian.c \
	sys/orchestration/internal/agents/core/roles/agent_knowledge.c \
	sys/orchestration/internal/agents/core/roles/agent_system.c \
	sys/orchestration/internal/agents/core/roles/agent_validator.c \
	sys/orchestration/internal/transport/transport_runtime.c \
	sys/orchestration/internal/transport/transport_protocol.c \
	sys/orchestration/internal/transport/uds_server.c \
	third_party/cjson/cJSON.c
ORCHESTRATION_SRCS := \
	sys/orchestration/planner/planner.c \
	sys/orchestration/workflow/rag_sessions.c \
	sys/orchestration/execution/rag_context_builder.c \
	sys/orchestration/execution/rag_prompts.c \
	sys/orchestration/execution/rag_pipeline.c
NETWORK_SRCS := \
	sys/network/topology/identity.c \
	sys/network/topology/topology.c \
	sys/network/topology/membership.c \
	sys/network/topology/peer_registry.c \
	sys/network/topology/trust.c \
	sys/network/topology/binding.c \
	sys/network/discovery/discovery.c \
	sys/network/discovery/enrollment.c \
	sys/network/routing/coordination.c \
	sys/network/mesh/mesh_topology.c \
	sys/network/mesh/mesh_peering.c \
	sys/network/mesh/containment.c \
	sys/network/transport/transport_runtime.c \
	sys/network/transport/transport_client.c \
	sys/network/transport/overlay_transport.c \
	sys/network/routing/replay_state.c \
	sys/network/routing/conflict_state.c \
	sys/network/mesh/mesh_policy.c
PROVIDERS_SRCS := \
	sys/network/providers/catalog.c \
	sys/network/providers/provider_registry.c \
	sys/network/providers/provider_selection.c \
	sys/network/providers/provider_policy.c \
	sys/network/providers/inference/client_inference.c \
	sys/network/providers/embedding/client_embedding.c \
	sys/network/providers/mocks/mock_provider.c \
	sys/network/providers/embedding/embedder_mock.c
KNOWLEDGE_SRCS := \
	sys/orchestration/internal/cognition/core/runtime_compat.c \
	sys/orchestration/internal/cognition/core/cognition/cognition.c \
	sys/orchestration/internal/cognition/core/cognition/activation.c \
	sys/orchestration/internal/cognition/core/cognition/reasoning/reasoning_roles.c \
	sys/orchestration/internal/cognition/core/cognition/reasoning/scoring.c \
	sys/orchestration/internal/cognition/core/memory/memory.c \
	sys/orchestration/internal/cognition/core/memory/authority.c \
	sys/orchestration/internal/cognition/core/memory/arena_store.c \
	sys/orchestration/internal/cognition/core/memory/storage_bridge.c \
	sys/orchestration/internal/cognition/core/episodic/episodic.c \
	sys/orchestration/internal/cognition/core/semantic/semantic_db.c \
	sys/orchestration/internal/cognition/core/vector/vector_index.c
DATA_SRCS := \
	sys/data/internal/store_binding.c \
	sys/data/internal/scope_binding.c \
	sys/data/store/file_store.c \
	sys/data/store/duckdb_store.c \
	sys/data/records/event_records.c \
	sys/data/evidence/evidence_records.c \
	sys/data/internal/query_surfaces.c \
	sys/data/retention/retention.c \
	sys/data/archive/archive.c
GRAPH_SRCS := \
	sys/graph/internal/backend_inmemory.c \
	sys/graph/internal/backend_rpc.c \
	sys/graph/internal/graph_facade.c \
	sys/graph/internal/state_reset.c \
	sys/graph/internal/ids.c \
	sys/graph/internal/counts.c \
	sys/graph/summary/activation_summary.c \
	sys/graph/summary/authority_summary.c \
	sys/graph/summary/episodic_summary.c \
	sys/graph/summary/semantic_summary.c \
	sys/graph/materialization/from_runtime_records.c \
	sys/graph/query/graph_summary_query.c \
	sys/graph/lineage/lineage_summary.c \
	third_party/cjson/cJSON.c
DAEMON_SRCS := \
	sys/daemon/yai-daemond/runtime/runtime_config.c \
	sys/daemon/yai-daemond/runtime/runtime_paths.c \
	sys/daemon/yai-daemond/runtime/runtime_services.c \
	sys/daemon/yai-daemond/runtime/runtime_state.c \
	sys/daemon/yai-daemond/runtime/runtime_source_ids.c \
	sys/daemon/yai-daemond/config.c \
	sys/daemon/yai-daemond/daemon_runtime.c \
	sys/daemon/replay/process.c \
	sys/daemon/yai-daemond/bootstrap.c \
	sys/daemon/bindings/runtime_binding.c \
	sys/daemon/bindings/network_binding.c \
	sys/daemon/bindings/actions.c \
	sys/daemon/yai-daemond/lifecycle.c \
	sys/daemon/mediation/mediation.c \
	sys/daemon/health/observation.c \
	sys/daemon/internal/internal.c \
	third_party/cjson/cJSON.c
CONTAINER_SRCS := \
	sys/container/session/bindings.c \
	sys/container/session/control_session.c \
	sys/container/session/control_reply.c \
	sys/container/session/control_surface.c \
	sys/container/runtime/config.c \
	sys/container/runtime/container.c \
	sys/container/runtime/grants_view.c \
	sys/container/runtime/identity.c \
	sys/container/runtime/internal/container_model.c \
	sys/container/runtime/lifecycle.c \
	sys/container/mounts/mounts.c \
	sys/container/rootfs/paths.c \
	sys/container/runtime/policy_view.c \
	sys/container/recovery/recovery.c \
	sys/container/runtime/registry/registry.c \
	sys/container/rootfs/root_projection.c \
	sys/container/runtime/runtime_surface.c \
	sys/container/runtime/runtime_view.c \
	sys/container/runtime/services.c \
	sys/container/session/session_binding.c \
	sys/container/runtime/state.c \
	sys/container/rootfs/tree.c
USER_CLI_SRCS := \
	user/shell/core/app.c \
	user/shell/core/control_call.c \
	user/shell/core/errors.c \
	user/shell/core/fmt.c \
	user/shell/core/lifecycle.c \
	user/shell/help/help.c \
	user/shell/session/session.c \
	user/shell/builtins/builtins.c \
	user/shell/prompt/prompt.c \
	user/shell/parse/parse.c \
	user/shell/render/display_map.c \
	user/shell/render/render.c \
	user/shell/render/render_reply.c \
	user/shell/render/style_map.c \
	user/shell/term/color.c \
	user/shell/term/keys.c \
	user/shell/term/pager.c \
	user/shell/term/screen.c \
	user/shell/term/term.c \
	user/shell/watch/watch.c \
	user/shell/watch/watch_model.c \
	user/shell/watch/watch_target.c \
	user/shell/watch/watch_ui.c
USER_LIBYAI_SRCS := \
	sdk/c/libyai/catalog/catalog.c \
	sdk/c/libyai/client/client.c \
	sdk/c/libyai/models/runtime_models.c \
	sdk/c/libyai/platform/context.c \
	sdk/c/libyai/platform/log.c \
	sdk/c/libyai/platform/paths.c \
	sdk/c/libyai/platform/transport.c \
	sdk/c/libyai/protocol/reply_map.c \
	sdk/c/libyai/registry/registry.c \
	sdk/c/libyai/registry/registry_cache.c \
	sdk/c/libyai/registry/registry_help.c \
	sdk/c/libyai/registry/registry_load.c \
	sdk/c/libyai/registry/registry_paths.c \
	sdk/c/libyai/registry/registry_query.c \
	sdk/c/libyai/registry/registry_validate.c \
	sdk/c/libyai/reply/reply_builder.c \
	sdk/c/libyai/reply/reply_json.c \
	sdk/c/libyai/rpc/rpc_client.c \
	sdk/c/libyai/source/source.c \
	sdk/c/libyai/sdk_public.c \
	sdk/c/libyai/third_party/cjson/cJSON.c
SUPPORT_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(SUPPORT_SRCS))
PLATFORM_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(PLATFORM_SRCS))
PROTOCOL_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(PROTOCOL_SRCS))
CORE_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(CORE_SRCS) $(GOVERNANCE_SRCS))
ORCHESTRATION_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(ORCHESTRATION_RUNTIME_SRCS) $(ORCHESTRATION_SRCS))
NETWORK_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(NETWORK_SRCS))
PROVIDERS_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(PROVIDERS_SRCS))
KNOWLEDGE_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(KNOWLEDGE_SRCS))
DATA_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(DATA_SRCS))
GRAPH_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(GRAPH_SRCS))
DAEMON_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(DAEMON_SRCS))
CONTAINER_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(CONTAINER_SRCS))
USER_CLI_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(USER_CLI_SRCS))
USER_LIBYAI_OBJS := $(patsubst %.c,$(OBJ_DIR)/%.o,$(USER_LIBYAI_SRCS))

SUPPORT_LIB := $(LIB_DIR)/libyai_support.a
PLATFORM_LIB := $(LIB_DIR)/libyai_platform.a
PROTOCOL_LIB := $(LIB_DIR)/libyai_protocol.a
CORE_LIB := $(LIB_DIR)/libyai_core.a
ORCHESTRATION_LIB := $(LIB_DIR)/libyai_orchestration.a
NETWORK_LIB := $(LIB_DIR)/libyai_network.a
PROVIDERS_LIB := $(LIB_DIR)/libyai_providers.a
KNOWLEDGE_LIB := $(LIB_DIR)/libyai_knowledge.a
DATA_LIB := $(LIB_DIR)/libyai_data.a
GRAPH_LIB := $(LIB_DIR)/libyai_graph.a
DAEMON_LIB := $(LIB_DIR)/libyai_daemon.a
CONTAINER_LIB := $(LIB_DIR)/libyai_container.a
USER_LIBYAI := $(LIB_DIR)/libyai_user_sdk.a

SPINE_DIRS := $(BIN_DIR) $(OBJ_DIR) $(LIB_DIR) $(TEST_DIR)

DOXYFILE := docs/transitional/root-meta/Doxyfile
DOXYGEN ?= doxygen
DOXY_OUT ?= $(DIST_ROOT)/docs/doxygen

.PHONY: all yai yai-ctl yai-sh yai-daemond yai-daemon yai-containerd yai-graphd yai-datad yai-netd yai-orchestratord yai-policyd yai-governanced yai-metricsd yai-auditd yai-supervisord yai-edge foundations support platform protocol core orchestration exec network mesh providers knowledge data graph edge daemon container yd1-baseline kernel-check kernel-smoke \
        test test-unit test-integration test-e2e test-core test-runtime test-knowledge test-orchestration test-protocol test-governance test-providers test-daemon test-edge test-mesh test-sys-container \
        test-demo-matrix verify-final-demo-matrix \
        clean clean-dist clean-all build build-all dist dist-all bundle verify \
        preflight-release docs docs-clean docs-verify proof-verify release-guards \
        release-guards-dev changelog-verify b13-convergence-check dirs help \
        governance-sync governance-check

all: yai yai-ctl yai-sh yai-daemond yai-daemon yai-containerd yai-graphd yai-datad yai-netd yai-orchestratord yai-policyd yai-governanced yai-metricsd yai-auditd yai-supervisord foundations
	@echo "[YAI] unified binary spine ready: $(YAI_BIN) + $(YAI_CTL_BIN) + $(YAI_SH_BIN) + $(YAI_DAEMOND_BIN) + $(YAI_DAEMON_BIN) + $(YAI_CONTAINERD_BIN) + $(YAI_GRAPHD_BIN) + $(YAI_DATAD_BIN) + $(YAI_NETD_BIN) + $(YAI_ORCHESTRATORD_BIN) + $(YAI_POLICYD_BIN) + $(YAI_GOVERNANCED_BIN) + $(YAI_METRICSD_BIN) + $(YAI_AUDITD_BIN) + $(YAI_SUPERVISORD_BIN)"

yai: $(YAI_BIN)
yai-ctl: $(YAI_CTL_BIN)
yai-sh: $(YAI_SH_BIN)
yai-daemond: $(YAI_DAEMOND_BIN)
yai-daemon: yai-daemond
	@cp "$(YAI_DAEMOND_BIN)" "$(YAI_DAEMON_BIN)"
yai-containerd: $(YAI_CONTAINERD_BIN)
yai-graphd: $(YAI_GRAPHD_BIN)
yai-datad: $(YAI_DATAD_BIN)
yai-netd: $(YAI_NETD_BIN)
yai-orchestratord: $(YAI_ORCHESTRATORD_BIN)
yai-policyd: $(YAI_POLICYD_BIN)
yai-governanced: $(YAI_GOVERNANCED_BIN)
yai-metricsd: $(YAI_METRICSD_BIN)
yai-auditd: $(YAI_AUDITD_BIN)
yai-supervisord: $(YAI_SUPERVISORD_BIN)
yai-edge: yai-daemon
	@cp "$(YAI_DAEMON_BIN)" "$(YAI_EDGE_ALIAS_BIN)"

foundations: support platform protocol network providers
core: $(CORE_LIB)
orchestration: $(ORCHESTRATION_LIB)
exec: orchestration
	@echo "[YAI] exec target is legacy alias; use 'make orchestration'"
providers: $(PROVIDERS_LIB)
network: $(NETWORK_LIB)
mesh: network
	@echo "[YAI] mesh target is legacy alias; use 'make network'"
knowledge: $(KNOWLEDGE_LIB)
data: $(DATA_LIB)
graph: $(GRAPH_LIB)
daemon: $(DAEMON_LIB)
container: $(CONTAINER_LIB)
edge: daemon
	@echo "[YAI] edge target is legacy alias; use 'make daemon'"

support: $(SUPPORT_LIB)
platform: $(PLATFORM_LIB)
protocol: $(PROTOCOL_LIB)
kernel-check:
	@$(MAKE) -C kernel check

kernel-smoke:
	@$(MAKE) -C kernel smoke

test: test-unit test-integration test-e2e
	@echo "[YAI] unified test baseline complete"

test-unit: test-core test-runtime test-orchestration test-protocol test-knowledge test-governance test-providers test-daemon test-mesh
	@echo "[YAI] unit suites complete"

test-integration:
	@tests/sys/orchestration/run_orchestration_smoke.sh
	@tests/sys/orchestration/run_orchestration_c_tests.sh
	@tests/sys/daemon/run_daemon_smoke.sh
	@tests/sys/network/mesh/run_mesh_smoke.sh
	@tests/legacy/source-plane/source_owner_ingest_bridge.sh
	@tests/legacy/source-plane/daemon_local_runtime_scan_spool_retry.sh
	@tests/legacy/source-plane/source_plane_read_model.sh
	@tests/integration/container/run_container_domain_smoke.sh
	@echo "[YAI] integration suites complete"

test-demo-matrix:
	@tests/legacy/workspace/workspace_demo_matrix.sh
	@echo "[YAI] final demo matrix suites complete"

verify-final-demo-matrix:
	@tools/dev/verify_final_demo_matrix.sh

test-e2e:
	@tests/legacy/e2e/run_entrypoint_e2e.sh
	@echo "[YAI] e2e suite complete"

test-core: yai
	@./build/bin/yai --help >/dev/null

test-runtime:
	@tests/legacy/runtime/unit/run_runtime_unit_tests.sh

test-knowledge:
	@tests/legacy/unit/knowledge/run_knowledge_unit_tests.sh

test-orchestration:
	@tests/legacy/unit/orchestration/run_orchestration_unit_tests.sh

test-protocol:
	@tests/legacy/unit/protocol/run_protocol_unit_tests.sh

test-governance:
	@tests/legacy/unit/governance/run_governance_unit_tests.sh
	@tests/sys/governance/run_governance_resolution_smoke.sh
	@echo "[YAI] governance-native resolution suites complete"

test-providers:
	@tests/legacy/unit/network/providers/run_providers_unit_tests.sh

test-daemon:
	@tests/legacy/unit/daemon/run_daemon_unit_tests.sh

test-edge: test-daemon
	@echo "[YAI] edge test target is legacy alias; use test-daemon"

test-mesh:
	@tests/legacy/unit/network/mesh/run_mesh_unit_tests.sh

test-sys-container: yai-containerd
	@tests/sys/container/containerd_smoke.sh

$(YAI_BIN): $(YAI_CLI_MAIN_OBJ) $(USER_CLI_OBJS) $(USER_LIBYAI) $(CORE_LIB) $(ORCHESTRATION_LIB) $(KNOWLEDGE_LIB) $(PROVIDERS_LIB) $(DATA_LIB) $(GRAPH_LIB) $(NETWORK_LIB) $(DAEMON_LIB) $(CONTAINER_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(PROTOCOL_LIB) | dirs
	$(CC) $(LDFLAGS) $(YAI_CLI_MAIN_OBJ) $(USER_CLI_OBJS) -o $@ $(USER_LIBYAI) $(CORE_LIB) $(ORCHESTRATION_LIB) $(KNOWLEDGE_LIB) $(PROVIDERS_LIB) $(DATA_LIB) $(GRAPH_LIB) $(NETWORK_LIB) $(DAEMON_LIB) $(CONTAINER_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(PROTOCOL_LIB) $(LDLIBS)

$(YAI_CTL_BIN): $(YAI_CTL_MAIN_OBJ) | dirs
	$(CC) $(LDFLAGS) $(YAI_CTL_MAIN_OBJ) -o $@ $(LDLIBS)

$(YAI_SH_BIN): $(YAI_SH_MAIN_OBJ) $(USER_CLI_OBJS) $(USER_LIBYAI) $(CORE_LIB) $(ORCHESTRATION_LIB) $(KNOWLEDGE_LIB) $(PROVIDERS_LIB) $(DATA_LIB) $(GRAPH_LIB) $(NETWORK_LIB) $(DAEMON_LIB) $(CONTAINER_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(PROTOCOL_LIB) | dirs
	$(CC) $(LDFLAGS) $(YAI_SH_MAIN_OBJ) $(USER_CLI_OBJS) -o $@ $(USER_LIBYAI) $(CORE_LIB) $(ORCHESTRATION_LIB) $(KNOWLEDGE_LIB) $(PROVIDERS_LIB) $(DATA_LIB) $(GRAPH_LIB) $(NETWORK_LIB) $(DAEMON_LIB) $(CONTAINER_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(PROTOCOL_LIB) $(LDLIBS)

$(YAI_DAEMOND_BIN): $(YAI_DAEMON_OBJ) $(DAEMON_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) | dirs
	$(CC) $(LDFLAGS) $(YAI_DAEMON_OBJ) -o $@ $(DAEMON_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(LDLIBS)

$(YAI_CONTAINERD_BIN): $(YAI_CONTAINERD_OBJ) $(CONTAINER_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) | dirs
	$(CC) $(LDFLAGS) $(YAI_CONTAINERD_OBJ) -o $@ $(CONTAINER_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(LDLIBS)

$(YAI_GRAPHD_BIN): $(YAI_GRAPHD_OBJ) $(GRAPH_LIB) $(KNOWLEDGE_LIB) $(DATA_LIB) $(PROTOCOL_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) | dirs
	$(CC) $(LDFLAGS) $(YAI_GRAPHD_OBJ) -o $@ $(GRAPH_LIB) $(KNOWLEDGE_LIB) $(DATA_LIB) $(PROTOCOL_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(LDLIBS)

$(YAI_DATAD_BIN): $(YAI_DATAD_OBJ) $(DATA_LIB) $(PROTOCOL_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) | dirs
	$(CC) $(LDFLAGS) $(YAI_DATAD_OBJ) -o $@ $(DATA_LIB) $(PROTOCOL_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(LDLIBS)

$(YAI_NETD_BIN): $(YAI_NETD_OBJ) $(NETWORK_LIB) $(PROVIDERS_LIB) $(PROTOCOL_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) | dirs
	$(CC) $(LDFLAGS) $(YAI_NETD_OBJ) -o $@ $(NETWORK_LIB) $(PROVIDERS_LIB) $(PROTOCOL_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(LDLIBS)

$(YAI_ORCHESTRATORD_BIN): $(YAI_ORCHESTRATORD_OBJ) $(ORCHESTRATION_LIB) $(KNOWLEDGE_LIB) $(PROTOCOL_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) | dirs
	$(CC) $(LDFLAGS) $(YAI_ORCHESTRATORD_OBJ) -o $@ $(ORCHESTRATION_LIB) $(KNOWLEDGE_LIB) $(PROTOCOL_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(LDLIBS)

$(YAI_POLICYD_BIN): $(YAI_POLICYD_OBJ) $(CORE_LIB) $(PROTOCOL_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) | dirs
	$(CC) $(LDFLAGS) $(YAI_POLICYD_OBJ) -o $@ $(CORE_LIB) $(PROTOCOL_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(LDLIBS)

$(YAI_GOVERNANCED_BIN): $(YAI_GOVERNANCED_OBJ) $(CORE_LIB) $(PROTOCOL_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) | dirs
	$(CC) $(LDFLAGS) $(YAI_GOVERNANCED_OBJ) -o $@ $(CORE_LIB) $(PROTOCOL_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(LDLIBS)

$(YAI_METRICSD_BIN): $(YAI_METRICSD_OBJ) $(CORE_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) | dirs
	$(CC) $(LDFLAGS) $(YAI_METRICSD_OBJ) -o $@ $(CORE_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(LDLIBS)

$(YAI_AUDITD_BIN): $(YAI_AUDITD_OBJ) $(CORE_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) | dirs
	$(CC) $(LDFLAGS) $(YAI_AUDITD_OBJ) -o $@ $(CORE_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(LDLIBS)

$(YAI_SUPERVISORD_BIN): $(YAI_SUPERVISORD_OBJ) $(CORE_LIB) $(PROTOCOL_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) | dirs
	$(CC) $(LDFLAGS) $(YAI_SUPERVISORD_OBJ) -o $@ $(CORE_LIB) $(PROTOCOL_LIB) $(SUPPORT_LIB) $(PLATFORM_LIB) $(LDLIBS)

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

$(NETWORK_LIB): $(NETWORK_OBJS) | dirs
	ar rcs $@ $^

$(PROVIDERS_LIB): $(PROVIDERS_OBJS) | dirs
	ar rcs $@ $^

$(KNOWLEDGE_LIB): $(KNOWLEDGE_OBJS) | dirs
	ar rcs $@ $^

$(DATA_LIB): $(DATA_OBJS) | dirs
	ar rcs $@ $^

$(GRAPH_LIB): $(GRAPH_OBJS) | dirs
	ar rcs $@ $^

$(DAEMON_LIB): $(DAEMON_OBJS) | dirs
	ar rcs $@ $^

$(CONTAINER_LIB): $(CONTAINER_OBJS) | dirs
	ar rcs $@ $^

$(USER_LIBYAI): $(USER_LIBYAI_OBJS) | dirs
	ar rcs $@ $^

$(OBJ_DIR)/%.o: %.c | dirs
	@mkdir -p $(dir $@)
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $< -o $@

dirs:
	@mkdir -p $(SPINE_DIRS)

build: yai yai-ctl yai-sh yai-daemon yai-containerd yai-graphd yai-datad yai-netd yai-orchestratord yai-policyd yai-governanced yai-metricsd yai-auditd yai-supervisord
	@echo "--- [YAI] primary service entrypoints build complete ---"

yd1-baseline: yai yai-daemon yai-containerd
	@echo "[YD-1] edge architecture refoundation baseline built"
	@echo "  owner runtime: build/bin/yai"
	@echo "  daemon runtime: build/bin/yai-daemon"
	@echo "  container manager: build/bin/yai-containerd"
	@echo "  refs:"
	@echo "    docs/architecture/daemon-architecture-refoundation-model.md"
	@echo "    docs/program/adr/ADR-015-daemon-architecture-refoundation-slice.md"

build-all: build
	@echo "--- [YAI] build-all complete (owner + daemon baseline topology) ---"

dist: build
	@mkdir -p $(BIN_DIST)
	@cp "$(YAI_BIN)" "$(BIN_DIST)/yai"
	@if [ -f "$(YAI_CTL_BIN)" ]; then cp "$(YAI_CTL_BIN)" "$(BIN_DIST)/yai-ctl"; fi
	@if [ -f "$(YAI_DAEMOND_BIN)" ]; then cp "$(YAI_DAEMOND_BIN)" "$(BIN_DIST)/yai-daemond"; fi
	@if [ -f "$(YAI_DAEMON_BIN)" ]; then cp "$(YAI_DAEMON_BIN)" "$(BIN_DIST)/yai-daemon"; cp "$(YAI_DAEMON_BIN)" "$(BIN_DIST)/yai-edge"; fi
	@if [ -f "$(YAI_CONTAINERD_BIN)" ]; then cp "$(YAI_CONTAINERD_BIN)" "$(BIN_DIST)/yai-containerd"; fi
	@if [ -f "$(YAI_GRAPHD_BIN)" ]; then cp "$(YAI_GRAPHD_BIN)" "$(BIN_DIST)/yai-graphd"; fi
	@if [ -f "$(YAI_DATAD_BIN)" ]; then cp "$(YAI_DATAD_BIN)" "$(BIN_DIST)/yai-datad"; fi
	@if [ -f "$(YAI_NETD_BIN)" ]; then cp "$(YAI_NETD_BIN)" "$(BIN_DIST)/yai-netd"; fi
	@if [ -f "$(YAI_ORCHESTRATORD_BIN)" ]; then cp "$(YAI_ORCHESTRATORD_BIN)" "$(BIN_DIST)/yai-orchestratord"; fi
	@if [ -f "$(YAI_POLICYD_BIN)" ]; then cp "$(YAI_POLICYD_BIN)" "$(BIN_DIST)/yai-policyd"; fi
	@if [ -f "$(YAI_GOVERNANCED_BIN)" ]; then cp "$(YAI_GOVERNANCED_BIN)" "$(BIN_DIST)/yai-governanced"; fi
	@if [ -f "$(YAI_METRICSD_BIN)" ]; then cp "$(YAI_METRICSD_BIN)" "$(BIN_DIST)/yai-metricsd"; fi
	@if [ -f "$(YAI_AUDITD_BIN)" ]; then cp "$(YAI_AUDITD_BIN)" "$(BIN_DIST)/yai-auditd"; fi
	@if [ -f "$(YAI_SUPERVISORD_BIN)" ]; then cp "$(YAI_SUPERVISORD_BIN)" "$(BIN_DIST)/yai-supervisord"; fi
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
	@echo "  all            (yai + yai-daemond + yai-daemon alias + foundation libs)"
	@echo "  yai            (build/bin/yai)"
	@echo "  yai-ctl        (build/bin/yai-ctl userland shell control alias)"
	@echo "  yai-sh         (build/bin/yai-sh canonical interactive shell entrypoint)"
	@echo "  yai-daemond (build/bin/yai-daemond canonical daemon manager)"
	@echo "  yai-daemon     (compat alias of build/bin/yai-daemond)"
	@echo "  yai-containerd (build/bin/yai-containerd container manager runtime)"
	@echo "  yai-edge       (legacy alias of build/bin/yai-daemon)"
	@echo "  yd1-baseline   (build anchors + YD-1 architecture refs)"
	@echo "  daemon         (build daemon runtime archive)"
	@echo "  container      (build container runtime archive)"
	@echo "  legacy         (build legacy runtime compatibility archive)"
	@echo "  edge           (legacy alias; use daemon)"
	@echo "  orchestration  (build orchestration control archive)"
	@echo "  exec           (legacy alias; use orchestration)"
	@echo "  foundations    (support/platform/protocol/providers archives)"
	@echo "  kernel-check   (syntax check of kernel privileged core sources)"
	@echo "  kernel-smoke   (run kernel vertical smoke: lifecycle/admission/containment/grants)"
	@echo "  providers      (build provider infrastructure archive)"
	@echo "  test-unit      (core/orchestration/protocol/knowledge/governance unit suites)"
	@echo "  test-integration (orchestration/daemon/network/source-plane + sys/container integration suites)"
	@echo "  test-knowledge (knowledge unit suite)"
	@echo "  test-orchestration (orchestration unit suite)"
	@echo "  test-protocol  (protocol unit suite)"
	@echo "  test-governance  (governance loader/discovery/resolution + smoke)"
	@echo "  test-sys-container (container manager system smoke)"
	@echo "  test-e2e       (entrypoint e2e smoke)"
	@echo "  test           (full test baseline)"
	@echo "  b13-convergence-check (single-repo final convergence smoke)"
	@echo "  clean          (remove build artifacts)"
	@echo "  dist, dist-all, bundle"
	@echo "  verify, docs, docs-verify, proof-verify, release-guards, changelog-verify"


test-vertical-slice:
	@tests/legacy/workspace/workspace_governed_vertical_slice.sh
	@echo "[YAI] governed vertical slice complete"
