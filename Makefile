# YAI - local build and validation surface
#
# Purpose:
#   Build the C/Rust control core, run doctrine guards and smoke validation.
#
# Ownership:
#   Repository-local build graph, install-local layout and canonical check/smoke
#   entrypoints.
#
# Boundary:
#   This file does not own runtime semantics, legal policy or data-plane truth.
#
.PHONY: info check-layout check-docs check-doc-root-canon check-case-runtime-semantics-roadmap check-duckdb-fact-plane check-receipt-decision-projection-facts check-model-behavior-policy-facts check-memory-divergence-carrier-facts check-fact-reports-cli check-fact-plane-freeze check-case-runtime-discovery check-casehandle-capability-boundary check-model-native-actor-roadmap check-spine-consistency check-control-lease-dispatch check-labs check-lab-runs check-lab-notebooks check-repository-identity check-archive-historical-records check-source-surface-clean check-file-header-standard check-pack-doctrine check-foundation-freeze check-hot-state-doctrine check-hot-state-freeze check-lmdb-record-plane-doctrine check-lmdb-record-plane-freeze check-journal-replay-boundary check-journal-replay-to-lmdb check-replay-idempotency-schema-version check-replay-diagnostics-report check-journal-replay-freeze check-control-carrier-substrate check-operation-dispatch-multiplex check-carrier-contract-v1 check-process-carrier-signal-control check-host-observation-probe check-carrier-coverage-matrix check-non-process-carrier-skeletons check-carrier-outcome-harness check-carrier-receipt-divergence check-retrieval-runner-roadmap check-context-compiler-retrieval-mtp-roadmap check-provider-runtime-lan-target-surface check-data-context-runtime-roadmap check-graph-runtimegraph-doctrine check-graph-relation-write-path check-runtimegraph-working-set check-runtimegraph-rebuild check-runtimegraph-query-causal-path check-operator-review-loop check-cli-review-interaction-surface check-review-loop-test-matrix check-graph-runtimegraph-freeze build-c build-rust build-rust-ffi build install-local uninstall-local doctor-local print-install-paths smoke-new1 smoke-new2 smoke-new3 smoke-new4 smoke-new5 smoke-new6 smoke-new7 smoke-new8 smoke-new9 smoke-new10 smoke-new11 smoke-new12 smoke-new18b smoke-new18c smoke-spine23 smoke-spine24 smoke-spine24a smoke-spine25 smoke-spine26 smoke-spine27 smoke-spine29 smoke-spine30 smoke-spine31 smoke-spine32 smoke-spine33 smoke-spine33a smoke-spine33b smoke-spine33c smoke-spine33d smoke-spine33e smoke-spine33f smoke-spine33g smoke-spine33h smoke-spine33i smoke-spine33l smoke-spine34 smoke-spine35 smoke-spine36 smoke-spine37 smoke-spine38 smoke-spine39 smoke-spine40 smoke-spine41 smoke-spine42 smoke-spine43 smoke-spine44 smoke-spine44a smoke-spine44b smoke-spine44c smoke-spine45 smoke-spine46 smoke-spine47 smoke-spine48 smoke-spine49 smoke-spine50 smoke-spine51 smoke-spine51b smoke-core-enforce1 smoke check clean
.PHONY: net-info build-net-c check-net-c-sources check-net-headers check-net-stream-fixtures check-net-node-fixtures check-net-capability-fixtures check-net-endpoint-fixtures check-net-health-fixtures check-net-lifecycle-fixtures check-net-transport-fixtures check-net-ipc-fixtures check-net-boundary
.PHONY: check-context-residency-lab lab-context-residency smoke-lab-context-residency smoke-lab-context-residency-case-native smoke-lab-context-residency-matrix

CC ?= cc
AR ?= ar
CFLAGS ?= -std=c11 -Wall -Wextra -Werror -Iinclude
RUST_FFI_LIBS ?= $(shell pkg-config --libs lmdb 2>/dev/null) -lpthread -ldl -lm
PREFIX ?= $(HOME)/.local
YAI_HOME ?= $(HOME)/.yai
BUILD_DIR := build
RUST_TARGET_DIR := target
RUST_ENGINE_STATIC := $(RUST_TARGET_DIR)/debug/libyai_core_engine.a
INSTALL_BINDIR := $(PREFIX)/bin
YAI_BIN := $(RUST_TARGET_DIR)/debug/yai
YAI_RUN_DIR := $(YAI_HOME)/run
YAI_STORE_DIR := $(YAI_HOME)/store
YAI_RECORD_STORE_DIR := $(YAI_STORE_DIR)/lmdb
YAI_LOG_DIR := $(YAI_HOME)/log
YAI_TMP_DIR := $(YAI_HOME)/tmp
YAI_CASES_DIR := $(YAI_HOME)/cases
YAI_SOCKETS_DIR := $(YAI_HOME)/sockets
YAI_CONFIG_DIR := $(YAI_HOME)/config
YAI_DAEMON_SOCKET := $(YAI_RUN_DIR)/yaid.sock

C_SOURCES := \
	system/internal/string_util.c \
	system/base/id.c \
	system/base/error.c \
	system/hot/hot_flags.c \
	system/hot/hot_snapshot.c \
	system/hot/hot_state.c \
	system/case/case_context.c \
	system/case/case_handle.c \
	system/case/interaction_thread.c \
	system/case/participant_view.c \
	system/case/case_ref.c \
	system/case/case_session.c \
	system/case/case_world.c \
	system/subject/subject_ref.c \
	system/subject/subject_binding.c \
	system/subject/subject_handle.c \
	system/subject/subject_state.c \
	system/op/attempt.c \
	system/control/failure_mode.c \
	system/control/authority_scope.c \
	system/control/policy_rule.c \
	system/control/gate.c \
	system/control/gate_outcome.c \
	system/control/decision_basis.c \
	system/control/decision.c \
	system/control/obligation.c \
	system/control/receipt_requirement.c \
	system/control/capability_lease.c \
	system/daemon/daemon_status.c \
	system/effect/carrier.c \
	system/effect/carrier_contract.c \
	system/effect/carrier_coverage.c \
	system/effect/carrier_family.c \
	system/effect/carrier_mode.c \
	system/effect/carrier_outcome.c \
	system/effect/carrier_outcome_harness.c \
	system/effect/carrier_receipt.c \
	system/effect/carrier_skeleton.c \
	system/effect/dispatch.c \
	system/effect/dispatch_admission.c \
	system/effect/dispatch_lane.c \
	system/effect/dispatch_multiplex.c \
	system/effect/dispatch_plan.c \
	system/effect/effect_hash.c \
	system/effect/receipt.c \
	system/effect/receipt_guarantee.c \
	system/effect/resource_scope.c \
	system/effect/carriers/filesystem_carrier.c \
	system/effect/carriers/process_carrier.c \
	system/effect/process_signal.c \
	system/effect/process_state.c \
	system/observation/host_observation.c \
	system/observation/host_probe.c \
	system/observation/observation_result.c \
	system/observation/observation_target.c \
	system/memory/memory_kind.c \
	system/memory/memory_scope.c \
	system/memory/memory_candidate.c \
	system/reconcile/divergence.c \
	system/reconcile/divergence_candidate.c \
	system/reconcile/carrier_consistency.c \
	system/reconcile/reconcile.c \
	system/reconcile/recovery.c \
	system/index/query.c \
	system/index/query_filter.c \
	system/index/query_result.c \
	system/graph/node_kind.c \
	system/graph/edge_kind.c \
	system/graph/edge.c \
	system/graph/graph.c \
	system/graph/reconstruct.c \
	system/graph/runtime_graph.c \
	system/store/record.c \
	system/store/journal.c \
	system/store/record_codec.c \
	system/store/journal_file.c \
	system/engine_bridge/rust_engine_backend.c \
	net/core/net.c \
	net/stream/stream.c \
	net/node/node.c \
	net/capability/capability.c \
	net/endpoint/endpoint.c \
	net/health/health.c \
	net/lifecycle/lifecycle.c \
	net/transport/transport.c \
	system/projection/projection.c \
	system/projection/projection_kind.c \
	system/projection/redaction.c \
	system/projection/freshness.c \
	system/projection/projection_request.c \
	system/projection/projection_result.c \
	system/projection/visibility_scope.c

C_OBJECTS := $(patsubst %.c,$(BUILD_DIR)/%.o,$(C_SOURCES))
C_LIBRARY := $(BUILD_DIR)/libyai_core_new13.a
YAID := $(BUILD_DIR)/yaid
SMOKE_MINIMUM_LOOP := $(BUILD_DIR)/test_minimum_loop
SMOKE_PERSISTENT_JOURNAL := $(BUILD_DIR)/test_persistent_journal
SMOKE_CONTROL_GATE := $(BUILD_DIR)/test_control_gate
SMOKE_FILESYSTEM_CARRIER := $(BUILD_DIR)/test_filesystem_carrier
SMOKE_GRAPH_RECONSTRUCTION := $(BUILD_DIR)/test_graph_reconstruction
SMOKE_OPERATIONAL_MEMORY := $(BUILD_DIR)/test_operational_memory
SMOKE_RECONCILE_DIVERGENCE := $(BUILD_DIR)/test_reconcile_divergence
SMOKE_PROJECTION_HARDENING := $(BUILD_DIR)/test_projection_hardening
SMOKE_QUERY_BOUNDARY := $(BUILD_DIR)/test_query_boundary
SMOKE_RUST_ENGINE_R1 := $(BUILD_DIR)/test_rust_engine_r1
SMOKE_CASE_CONTEXT := $(BUILD_DIR)/test_case_context
SMOKE_INTERACTION_THREAD := $(BUILD_DIR)/test_interaction_thread
SMOKE_HOT_STATE := $(BUILD_DIR)/test_hot_state
SMOKE_CONTROL_CARRIER_SUBSTRATE := $(BUILD_DIR)/test_control_carrier_substrate
SMOKE_OPERATION_DISPATCH_MULTIPLEX := $(BUILD_DIR)/test_operation_dispatch_multiplex
SMOKE_CARRIER_CONTRACT_FILESYSTEM := $(BUILD_DIR)/test_carrier_contract_filesystem
SMOKE_PROCESS_CARRIER := $(BUILD_DIR)/test_process_carrier
SMOKE_CONTROL_LEASE_DISPATCH := $(BUILD_DIR)/test_control_lease_dispatch
SMOKE_HOST_OBSERVATION_PROBE := $(BUILD_DIR)/test_host_observation_probe
SMOKE_CARRIER_COVERAGE_MATRIX := $(BUILD_DIR)/test_carrier_coverage_matrix
SMOKE_NON_PROCESS_CARRIER_SKELETONS := $(BUILD_DIR)/test_non_process_carrier_skeletons
SMOKE_CARRIER_OUTCOME_HARNESS := $(BUILD_DIR)/test_carrier_outcome_harness
SMOKE_CARRIER_RECEIPT_DIVERGENCE := $(BUILD_DIR)/test_carrier_receipt_divergence
SMOKE_HOT_STATE_SNAPSHOT := tests/smoke/hot-state-snapshot/test_hot_state_snapshot.sh
SMOKE_COMMAND_SURFACE := tests/smoke/command-surface/test_command_surface.sh
SMOKE_HOT_STATE_SESSION := $(BUILD_DIR)/test_hot_state_session
SMOKE_PROJECTION_FRESHNESS := $(BUILD_DIR)/test_projection_freshness
SMOKE_HOT_STATE_CLI := tests/smoke/hot-state-cli/test_hot_state_cli.sh
SMOKE_RECORD_STORE_CLI := tests/smoke/record-store-cli/test_record_store_cli.sh
SMOKE_RECORD_STORE_WRITE := tests/smoke/record-store-write/test_record_store_write.sh
SMOKE_RECORD_STORE_READ_QUERY := tests/smoke/record-store-read-query/test_record_store_read_query.sh
SMOKE_RECORD_STORE_SUBJECT_RECEIPT := tests/smoke/record-store-subject-receipt-indexes/test_record_store_subject_receipt_indexes.sh
SMOKE_RECORD_STORE_CLI_MANUAL := tests/smoke/record-store-cli-manual-validation/test_record_store_cli_manual_validation.sh
SMOKE_RECORD_STORE_FREEZE := tests/smoke/record-store-freeze/test_record_store_freeze.sh
SMOKE_JOURNAL_REPLAY_BOUNDARY := tests/smoke/journal-replay-boundary/test_journal_replay_boundary.sh
SMOKE_JOURNAL_REPLAY_TO_LMDB := tests/smoke/journal-replay-to-lmdb/test_journal_replay_to_lmdb.sh
SMOKE_REPLAY_IDEMPOTENCY_SCHEMA := tests/smoke/replay-idempotency-schema-version/test_replay_idempotency_schema_version.sh
SMOKE_REPLAY_DIAGNOSTICS_REPORT := tests/smoke/replay-diagnostics-report/test_replay_diagnostics_report.sh
SMOKE_JOURNAL_REPLAY_FREEZE := tests/smoke/journal-replay-freeze/test_journal_replay_freeze.sh
SMOKE_GRAPH_RUNTIMEGRAPH_SCHEMA := $(BUILD_DIR)/test_graph_runtimegraph_schema
SMOKE_GRAPH_RELATION_WRITE_PATH := tests/smoke/graph-relation-write-path/test_graph_relation_write_path.sh
SMOKE_RUNTIMEGRAPH_WORKING_SET := tests/smoke/runtimegraph-working-set/test_runtimegraph_working_set.sh
SMOKE_RUNTIMEGRAPH_REBUILD := tests/smoke/runtimegraph-rebuild/test_runtimegraph_rebuild.sh
SMOKE_RUNTIMEGRAPH_QUERY := tests/smoke/runtimegraph-query/test_runtimegraph_query.sh
SMOKE_OPERATOR_REVIEW_LOOP := tests/smoke/operator-review-loop/test_operator_review_loop.sh
SMOKE_OPERATOR_REVIEW_CLI := tests/smoke/operator-review-cli/test_operator_review_cli.sh
SMOKE_REVIEW_LOOP_TEST_MATRIX := tests/smoke/review-loop-test-matrix/test_review_loop_test_matrix.sh
SMOKE_GRAPH_RUNTIMEGRAPH_FREEZE := tests/smoke/graph-runtimegraph-freeze/test_graph_runtimegraph_freeze.sh
SMOKE_DUCKDB_FACT_PLANE := tests/smoke/duckdb-fact-plane/test_duckdb_fact_plane.sh
SMOKE_RECEIPT_DECISION_PROJECTION_FACTS := tests/smoke/receipt-decision-projection-facts/test_receipt_decision_projection_facts.sh
SMOKE_MODEL_BEHAVIOR_POLICY_FACTS := tests/smoke/model-behavior-policy-facts/test_model_behavior_policy_facts.sh
SMOKE_MEMORY_DIVERGENCE_CARRIER_FACTS := tests/smoke/memory-divergence-carrier-facts/test_memory_divergence_carrier_facts.sh
SMOKE_FACT_REPORTS_CLI := tests/smoke/fact-reports-cli/test_fact_reports_cli.sh
SMOKE_FACT_PLANE_FREEZE := tests/smoke/fact-plane-freeze/test_fact_plane_freeze.sh
SMOKE_CASEHANDLE_CAPABILITY_BOUNDARY := tests/smoke/casehandle-capability-boundary/test_casehandle_capability_boundary.sh
SMOKE_DAEMON_IPC := tests/smoke/daemon-ipc/test_daemon_ipc.sh
SMOKE_DAEMON_CORE_LOOP := tests/smoke/daemon-core-loop/test_daemon_core_loop.sh
SMOKE_PROVIDER_RUNTIME_SURFACE := tests/smoke/provider-runtime-surface/test_provider_runtime_surface.sh
NET_ENUM_NAMES_TEST := $(BUILD_DIR)/test_net_enum_names

info:
	@printf "yai: local AI operational control core\n"
	@printf "status: SPINE.51 Fact Plane Freeze\n"
	@printf "completed: SPINE.20 Local Runtime Layout through SPINE.51 Fact Plane Freeze\n"
	@printf "next: SPINE.52 Live Projection Frame Schema\n"
	@printf "target-layout: include/ system/ engine/ cmd/\n"
	@printf "runtime-home: YAI_HOME=%s socket=%s\n" "$(YAI_HOME)" "$(YAI_DAEMON_SOCKET)"
	@printf "hot-state: %s/hot-state.json\n" "$(YAI_RUN_DIR)"
	@printf "record-store: %s\n" "$(YAI_RECORD_STORE_DIR)"
	@printf "fact-plane: DuckDB yai.fact.v1 extraction plus compact CLI reports\n"
	@printf "pack-doctrine: active work/archive/engineering-snapshots/pack-format.md\n"
	@printf "foundation-freeze: filesystem closed; runtime layout exists; active docs compact; extraction contract active\n"
	@printf "data-spine-c: transitional keep_temporarily\n"
	@printf "engine-bridge: active\n"
	@printf "lib: removed\n"
	@printf "daemon: moved to cmd/yaid + system/daemon\n"
	@printf "provider-runtime: planned surface active\n"
	@printf "device-registry: active\n"
	@printf "crates: removed\n"
	@printf "ctl: removed\n"
	@printf "install-local: active PREFIX=%s YAI_HOME=%s\n" "$(PREFIX)" "$(YAI_HOME)"

net-info:
	@printf "net: yai runtime communication module\n"
	@printf "status: C translation unit scaffold started\n"
	@printf "reference: NET.SPINE.9C.0\n"
	@printf "discovery: not implemented\n"
	@printf "transport: not implemented\n"
	@printf "routing: not implemented\n"
	@printf "external-nodes: planned\n"

build-net-c: $(NET_ENUM_NAMES_TEST)
	@$(NET_ENUM_NAMES_TEST)

check-net-boundary:
	@./tools/checks/check-net-boundary.sh

check-net-c-sources:
	@./tools/checks/check-net-c-sources.sh

check-net-headers:
	@./tools/checks/check-net-headers.sh

check-net-stream-fixtures:
	@python3 tools/checks/check-net-stream-fixtures.py

check-net-node-fixtures:
	@python3 tools/checks/check-net-node-fixtures.py

check-net-capability-fixtures:
	@python3 tools/checks/check-net-capability-fixtures.py

check-net-endpoint-fixtures:
	@python3 tools/checks/check-net-endpoint-fixtures.py

check-net-health-fixtures:
	@python3 tools/checks/check-net-health-fixtures.py

check-net-lifecycle-fixtures:
	@python3 tools/checks/check-net-lifecycle-fixtures.py

check-net-transport-fixtures:
	@python3 tools/checks/check-net-transport-fixtures.py

check-net-ipc-fixtures:
	@python3 tools/checks/check-net-ipc-fixtures.py

check-layout:
	@./tools/checks/check-no-old-roots.sh
	@./tools/checks/check-required-layout.sh
	@./tools/checks/check-source-placement.sh
	@./tools/checks/check-source-surface-clean.sh

check-docs:
	@./tools/checks/check-doc-root-canon.sh
	@./tools/checks/check-doc-canonical-location.sh
	@./tools/checks/check-doc-required-files.sh
	@./tools/checks/check-doc-no-old-root-language.sh
	@./tools/checks/check-lab-notebooks.sh
	@./tools/checks/check-repository-identity.sh
	@./tools/checks/check-archive-historical-records.sh
	@./tools/checks/check-pack-doctrine.sh
	@./tools/checks/check-foundation-freeze.sh
	@./tools/checks/check-hot-state-doctrine.sh
	@./tools/checks/check-hot-state-freeze.sh
	@./tools/checks/check-lmdb-record-plane-doctrine.sh
	@./tools/checks/check-lmdb-record-plane-freeze.sh
	@./tools/checks/check-journal-replay-boundary.sh
	@./tools/checks/check-journal-replay-to-lmdb.sh
	@./tools/checks/check-replay-idempotency-schema-version.sh
	@./tools/checks/check-replay-diagnostics-report.sh
	@./tools/checks/check-journal-replay-freeze.sh
	@./tools/checks/check-file-header-standard.sh
	@./tools/checks/check-control-carrier-substrate.sh
	@./tools/checks/check-operation-dispatch-multiplex.sh
	@./tools/checks/check-carrier-contract-v1.sh
	@./tools/checks/check-process-carrier-signal-control.sh
	@./tools/checks/check-host-observation-probe.sh
	@./tools/checks/check-carrier-coverage-matrix.sh
	@./tools/checks/check-non-process-carrier-skeletons.sh
	@./tools/checks/check-carrier-outcome-harness.sh
	@./tools/checks/check-carrier-receipt-divergence.sh
	@./tools/checks/check-retrieval-runner-roadmap.sh
	@./tools/checks/check-context-compiler-retrieval-mtp-roadmap.sh
	@./tools/checks/check-provider-runtime-lan-target-surface.sh
	@./tools/checks/check-data-context-runtime-roadmap.sh
	@./tools/checks/check-graph-runtimegraph-doctrine.sh
	@./tools/checks/check-graph-relation-write-path.sh
	@./tools/checks/check-runtimegraph-working-set.sh
	@./tools/checks/check-runtimegraph-rebuild.sh
	@./tools/checks/check-runtimegraph-query-causal-path.sh
	@./tools/checks/check-operator-review-loop.sh
	@./tools/checks/check-cli-review-interaction-surface.sh
	@./tools/checks/check-review-loop-test-matrix.sh
	@./tools/checks/check-graph-runtimegraph-freeze.sh
	@./tools/checks/check-case-runtime-semantics-roadmap.sh
	@./tools/checks/check-duckdb-fact-plane.sh
	@./tools/checks/check-receipt-decision-projection-facts.sh
	@./tools/checks/check-model-behavior-policy-facts.sh
	@./tools/checks/check-memory-divergence-carrier-facts.sh
	@./tools/checks/check-fact-reports-cli.sh
	@./tools/checks/check-fact-plane-freeze.sh
	@./tools/checks/check-case-runtime-discovery.sh
	@./tools/checks/check-casehandle-capability-boundary.sh
	@./tools/checks/check-model-native-actor-roadmap.sh
	@./tools/checks/check-net-c-sources.sh
	@./tools/checks/check-net-boundary.sh
	@./tools/checks/check-context-residency-lab.sh
	@./tools/checks/check-spine-consistency.sh
	@./tools/checks/check-control-lease-dispatch.sh

check-doc-root-canon:
	@./tools/checks/check-doc-root-canon.sh

check-case-runtime-semantics-roadmap:
	@./tools/checks/check-case-runtime-semantics-roadmap.sh

check-duckdb-fact-plane:
	@./tools/checks/check-duckdb-fact-plane.sh

check-receipt-decision-projection-facts:
	@./tools/checks/check-receipt-decision-projection-facts.sh

check-model-behavior-policy-facts:
	@./tools/checks/check-model-behavior-policy-facts.sh

check-memory-divergence-carrier-facts:
	@./tools/checks/check-memory-divergence-carrier-facts.sh

check-fact-reports-cli:
	@./tools/checks/check-fact-reports-cli.sh

check-fact-plane-freeze:
	@./tools/checks/check-fact-plane-freeze.sh

check-case-runtime-discovery:
	@./tools/checks/check-case-runtime-discovery.sh

check-casehandle-capability-boundary:
	@./tools/checks/check-casehandle-capability-boundary.sh

check-model-native-actor-roadmap:
	@./tools/checks/check-model-native-actor-roadmap.sh

check-spine-consistency:
	@./tools/checks/check-spine-consistency.sh

check-control-lease-dispatch:
	@./tools/checks/check-control-lease-dispatch.sh

check-labs:
	@./tools/checks/check-labs-layout.sh
	@./tools/checks/check-lab-notebooks.sh

check-lab-runs:
	@./tools/checks/check-lab-run-contract.sh

check-lab-notebooks:
	@./tools/checks/check-lab-notebooks.sh

check-context-residency-lab:
	@./tools/checks/check-context-residency-lab.sh

lab-context-residency:
	@labs/context-residency/run.sh --mode no-ai

smoke-lab-context-residency: smoke-lab-context-residency-matrix

smoke-lab-context-residency-case-native:
	@labs/context-residency/run.sh --mode case-native

smoke-lab-context-residency-matrix:
	@labs/context-residency/run.sh --mode no-ai

check-repository-identity:
	@./tools/checks/check-repository-identity.sh

check-archive-historical-records:
	@./tools/checks/check-archive-historical-records.sh

check-source-surface-clean:
	@./tools/checks/check-source-surface-clean.sh

check-file-header-standard:
	@./tools/checks/check-file-header-standard.sh

check-pack-doctrine:
	@./tools/checks/check-pack-doctrine.sh

check-foundation-freeze:
	@./tools/checks/check-foundation-freeze.sh

check-hot-state-doctrine:
	@./tools/checks/check-hot-state-doctrine.sh

check-hot-state-freeze:
	@./tools/checks/check-hot-state-freeze.sh

check-lmdb-record-plane-doctrine:
	@./tools/checks/check-lmdb-record-plane-doctrine.sh

check-lmdb-record-plane-freeze:
	@./tools/checks/check-lmdb-record-plane-freeze.sh

check-journal-replay-boundary:
	@./tools/checks/check-journal-replay-boundary.sh

check-journal-replay-to-lmdb:
	@./tools/checks/check-journal-replay-to-lmdb.sh

check-replay-idempotency-schema-version:
	@./tools/checks/check-replay-idempotency-schema-version.sh

check-replay-diagnostics-report:
	@./tools/checks/check-replay-diagnostics-report.sh

check-journal-replay-freeze:
	@./tools/checks/check-journal-replay-freeze.sh

check-control-carrier-substrate:
	@./tools/checks/check-control-carrier-substrate.sh

check-operation-dispatch-multiplex:
	@./tools/checks/check-operation-dispatch-multiplex.sh

check-carrier-contract-v1:
	@./tools/checks/check-carrier-contract-v1.sh

check-process-carrier-signal-control:
	@./tools/checks/check-process-carrier-signal-control.sh

check-host-observation-probe:
	@./tools/checks/check-host-observation-probe.sh

check-carrier-coverage-matrix:
	@./tools/checks/check-carrier-coverage-matrix.sh

check-retrieval-runner-roadmap:
	@./tools/checks/check-retrieval-runner-roadmap.sh

check-context-compiler-retrieval-mtp-roadmap:
	@./tools/checks/check-context-compiler-retrieval-mtp-roadmap.sh

check-provider-runtime-lan-target-surface:
	@./tools/checks/check-provider-runtime-lan-target-surface.sh

check-data-context-runtime-roadmap:
	@./tools/checks/check-data-context-runtime-roadmap.sh

check-graph-runtimegraph-doctrine:
	@./tools/checks/check-graph-runtimegraph-doctrine.sh

check-graph-relation-write-path:
	@./tools/checks/check-graph-relation-write-path.sh

check-runtimegraph-working-set:
	@./tools/checks/check-runtimegraph-working-set.sh

check-runtimegraph-rebuild:
	@./tools/checks/check-runtimegraph-rebuild.sh

check-runtimegraph-query-causal-path:
	@./tools/checks/check-runtimegraph-query-causal-path.sh

check-operator-review-loop:
	@./tools/checks/check-operator-review-loop.sh

check-cli-review-interaction-surface:
	@./tools/checks/check-cli-review-interaction-surface.sh

check-review-loop-test-matrix:
	@./tools/checks/check-review-loop-test-matrix.sh

check-graph-runtimegraph-freeze:
	@./tools/checks/check-graph-runtimegraph-freeze.sh

check-non-process-carrier-skeletons:
	@./tools/checks/check-non-process-carrier-skeletons.sh

check-carrier-outcome-harness:
	@./tools/checks/check-carrier-outcome-harness.sh

check-carrier-receipt-divergence:
	@./tools/checks/check-carrier-receipt-divergence.sh

$(BUILD_DIR)/%.o: %.c
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) -c "$<" -o "$@"

$(C_LIBRARY): $(C_OBJECTS)
	@mkdir -p "$(dir $@)"
	$(AR) rcs "$@" $(C_OBJECTS)

$(YAID): cmd/yaid/main.c system/daemon/ipc.c system/daemon/core_loop.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) cmd/yaid/main.c system/daemon/ipc.c system/daemon/core_loop.c $(C_LIBRARY) -o "$@"

$(SMOKE_MINIMUM_LOOP): tests/smoke/minimum-loop/test_minimum_loop.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/minimum-loop/test_minimum_loop.c $(C_LIBRARY) -o "$@"

$(SMOKE_PERSISTENT_JOURNAL): tests/smoke/persistent-journal/test_persistent_journal.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/persistent-journal/test_persistent_journal.c $(C_LIBRARY) -o "$@"

$(SMOKE_CONTROL_GATE): tests/smoke/control-gate/test_control_gate.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/control-gate/test_control_gate.c $(C_LIBRARY) -o "$@"

$(SMOKE_FILESYSTEM_CARRIER): tests/smoke/filesystem-carrier/test_filesystem_carrier.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/filesystem-carrier/test_filesystem_carrier.c $(C_LIBRARY) -o "$@"

$(SMOKE_GRAPH_RECONSTRUCTION): tests/smoke/graph-reconstruction/test_graph_reconstruction.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/graph-reconstruction/test_graph_reconstruction.c $(C_LIBRARY) -o "$@"

$(SMOKE_OPERATIONAL_MEMORY): tests/smoke/operational-memory/test_operational_memory.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/operational-memory/test_operational_memory.c $(C_LIBRARY) -o "$@"

$(SMOKE_RECONCILE_DIVERGENCE): tests/smoke/reconcile-divergence/test_reconcile_divergence.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/reconcile-divergence/test_reconcile_divergence.c $(C_LIBRARY) -o "$@"

$(SMOKE_PROJECTION_HARDENING): tests/smoke/projection-hardening/test_projection_hardening.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/projection-hardening/test_projection_hardening.c $(C_LIBRARY) -o "$@"

$(SMOKE_QUERY_BOUNDARY): tests/smoke/query-boundary/test_query_boundary.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/query-boundary/test_query_boundary.c $(C_LIBRARY) -o "$@"

$(SMOKE_RUST_ENGINE_R1): tests/smoke/rust-engine-r1/test_rust_engine_r1.c $(C_LIBRARY) $(RUST_ENGINE_STATIC)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/rust-engine-r1/test_rust_engine_r1.c $(C_LIBRARY) $(RUST_ENGINE_STATIC) $(RUST_FFI_LIBS) -o "$@"

$(SMOKE_CASE_CONTEXT): tests/smoke/case-context/test_case_context.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/case-context/test_case_context.c $(C_LIBRARY) -o "$@"

$(SMOKE_INTERACTION_THREAD): tests/smoke/interaction-thread/test_interaction_thread.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/interaction-thread/test_interaction_thread.c $(C_LIBRARY) -o "$@"

$(SMOKE_HOT_STATE): tests/smoke/hot-state/test_hot_state.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/hot-state/test_hot_state.c $(C_LIBRARY) -o "$@"

$(SMOKE_CONTROL_CARRIER_SUBSTRATE): tests/smoke/control-carrier-substrate/test_control_carrier_substrate.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/control-carrier-substrate/test_control_carrier_substrate.c $(C_LIBRARY) -o "$@"

$(SMOKE_OPERATION_DISPATCH_MULTIPLEX): tests/smoke/operation-dispatch-multiplex/test_operation_dispatch_multiplex.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/operation-dispatch-multiplex/test_operation_dispatch_multiplex.c $(C_LIBRARY) -o "$@"

$(SMOKE_CARRIER_CONTRACT_FILESYSTEM): tests/smoke/carrier-contract-filesystem/test_carrier_contract_filesystem.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/carrier-contract-filesystem/test_carrier_contract_filesystem.c $(C_LIBRARY) -o "$@"

$(SMOKE_PROCESS_CARRIER): tests/smoke/process-carrier/test_process_carrier.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/process-carrier/test_process_carrier.c $(C_LIBRARY) -o "$@"

$(SMOKE_CONTROL_LEASE_DISPATCH): tests/smoke/control-lease-dispatch/test_control_lease_dispatch.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/control-lease-dispatch/test_control_lease_dispatch.c $(C_LIBRARY) -o "$@"

$(SMOKE_HOST_OBSERVATION_PROBE): tests/smoke/host-observation-probe/test_host_observation_probe.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/host-observation-probe/test_host_observation_probe.c $(C_LIBRARY) -o "$@"

$(SMOKE_CARRIER_COVERAGE_MATRIX): tests/smoke/carrier-coverage-matrix/test_carrier_coverage_matrix.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/carrier-coverage-matrix/test_carrier_coverage_matrix.c $(C_LIBRARY) -o "$@"

$(SMOKE_NON_PROCESS_CARRIER_SKELETONS): tests/smoke/non-process-carrier-skeletons/test_non_process_carrier_skeletons.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/non-process-carrier-skeletons/test_non_process_carrier_skeletons.c $(C_LIBRARY) -o "$@"

$(SMOKE_CARRIER_OUTCOME_HARNESS): tests/smoke/carrier-outcome-harness/test_carrier_outcome_harness.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/carrier-outcome-harness/test_carrier_outcome_harness.c $(C_LIBRARY) -o "$@"

$(SMOKE_CARRIER_RECEIPT_DIVERGENCE): tests/smoke/carrier-receipt-divergence/test_carrier_receipt_divergence.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/carrier-receipt-divergence/test_carrier_receipt_divergence.c $(C_LIBRARY) -o "$@"

$(SMOKE_GRAPH_RUNTIMEGRAPH_SCHEMA): tests/smoke/graph-runtimegraph-schema/test_graph_runtimegraph_schema.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/graph-runtimegraph-schema/test_graph_runtimegraph_schema.c $(C_LIBRARY) -o "$@"

$(SMOKE_HOT_STATE_SESSION): tests/smoke/hot-state-session/test_hot_state_session.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/hot-state-session/test_hot_state_session.c $(C_LIBRARY) -o "$@"

$(SMOKE_PROJECTION_FRESHNESS): tests/smoke/projection-freshness/test_projection_freshness.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/projection-freshness/test_projection_freshness.c $(C_LIBRARY) -o "$@"

$(NET_ENUM_NAMES_TEST): tests/net/test_net_enum_names.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/net/test_net_enum_names.c $(C_LIBRARY) -o "$@"

build-c: build-rust-ffi $(C_LIBRARY) $(YAID) $(SMOKE_MINIMUM_LOOP) $(SMOKE_PERSISTENT_JOURNAL) $(SMOKE_CONTROL_GATE) $(SMOKE_FILESYSTEM_CARRIER) $(SMOKE_GRAPH_RECONSTRUCTION) $(SMOKE_OPERATIONAL_MEMORY) $(SMOKE_RECONCILE_DIVERGENCE) $(SMOKE_PROJECTION_HARDENING) $(SMOKE_QUERY_BOUNDARY) $(SMOKE_RUST_ENGINE_R1) $(SMOKE_CASE_CONTEXT) $(SMOKE_INTERACTION_THREAD) $(SMOKE_HOT_STATE) $(SMOKE_CONTROL_CARRIER_SUBSTRATE) $(SMOKE_OPERATION_DISPATCH_MULTIPLEX) $(SMOKE_CARRIER_CONTRACT_FILESYSTEM) $(SMOKE_PROCESS_CARRIER) $(SMOKE_CONTROL_LEASE_DISPATCH) $(SMOKE_HOST_OBSERVATION_PROBE) $(SMOKE_CARRIER_COVERAGE_MATRIX) $(SMOKE_NON_PROCESS_CARRIER_SKELETONS) $(SMOKE_CARRIER_OUTCOME_HARNESS) $(SMOKE_CARRIER_RECEIPT_DIVERGENCE) $(SMOKE_GRAPH_RUNTIMEGRAPH_SCHEMA) $(SMOKE_HOT_STATE_SESSION) $(SMOKE_PROJECTION_FRESHNESS)

build-rust-ffi:
	CARGO_TARGET_DIR=$(RUST_TARGET_DIR) cargo build --manifest-path engine/Cargo.toml -p yai-engine

build-rust:
	CARGO_TARGET_DIR=$(RUST_TARGET_DIR) cargo build --manifest-path engine/Cargo.toml --workspace
	CARGO_TARGET_DIR=$(RUST_TARGET_DIR) cargo test --manifest-path engine/Cargo.toml --workspace
	CARGO_TARGET_DIR=$(RUST_TARGET_DIR) cargo build --manifest-path cmd/yai/Cargo.toml
	CARGO_TARGET_DIR=$(RUST_TARGET_DIR) cargo test --manifest-path cmd/yai/Cargo.toml

build: build-c build-rust

print-install-paths:
	@printf "PREFIX=%s\n" "$(PREFIX)"
	@printf "YAI_HOME=%s\n" "$(YAI_HOME)"
	@printf "yai=%s/yai\n" "$(INSTALL_BINDIR)"
	@printf "yaid=%s/yaid\n" "$(INSTALL_BINDIR)"
	@printf "run=%s\n" "$(YAI_RUN_DIR)"
	@printf "store=%s\n" "$(YAI_STORE_DIR)"
	@printf "record_store=%s\n" "$(YAI_RECORD_STORE_DIR)"
	@printf "log=%s\n" "$(YAI_LOG_DIR)"
	@printf "tmp=%s\n" "$(YAI_TMP_DIR)"
	@printf "cases=%s\n" "$(YAI_CASES_DIR)"
	@printf "sockets=%s\n" "$(YAI_SOCKETS_DIR)"
	@printf "config=%s\n" "$(YAI_CONFIG_DIR)"
	@printf "socket=%s\n" "$(YAI_DAEMON_SOCKET)"

install-local: build
	@mkdir -p "$(INSTALL_BINDIR)"
	@mkdir -p "$(YAI_RUN_DIR)" "$(YAI_RECORD_STORE_DIR)" "$(YAI_LOG_DIR)" "$(YAI_TMP_DIR)" "$(YAI_CASES_DIR)" "$(YAI_SOCKETS_DIR)" "$(YAI_CONFIG_DIR)"
	@cp "$(YAI_BIN)" "$(INSTALL_BINDIR)/yai"
	@cp "$(YAID)" "$(INSTALL_BINDIR)/yaid"
	@chmod +x "$(INSTALL_BINDIR)/yai" "$(INSTALL_BINDIR)/yaid"
	@printf "installed: %s/yai\n" "$(INSTALL_BINDIR)"
	@printf "installed: %s/yaid\n" "$(INSTALL_BINDIR)"
	@$(MAKE) --no-print-directory doctor-local PREFIX="$(PREFIX)" YAI_HOME="$(YAI_HOME)"

uninstall-local:
	@rm -f "$(INSTALL_BINDIR)/yai" "$(INSTALL_BINDIR)/yaid"
	@printf "uninstalled local yai binaries from %s\n" "$(INSTALL_BINDIR)"

doctor-local:
	@printf "yai local doctor\n"
	@printf "PREFIX: %s\n" "$(PREFIX)"
	@printf "YAI_HOME: %s\n" "$(YAI_HOME)"
	@printf "YAI_HOME_status: %s\n" "$$(if [ -d "$(YAI_HOME)" ]; then printf "ok"; else printf "missing"; fi)"
	@printf "binary_path: %s\n" "$$(if [ -x "$(INSTALL_BINDIR)/yai" ]; then printf "%s/yai" "$(INSTALL_BINDIR)"; else printf "missing"; fi)"
	@printf "yaid_path: %s\n" "$$(if [ -x "$(INSTALL_BINDIR)/yaid" ]; then printf "%s/yaid" "$(INSTALL_BINDIR)"; else printf "missing"; fi)"
	@printf "yai_version: %s\n" "$$(if [ -x "$(INSTALL_BINDIR)/yai" ]; then "$(INSTALL_BINDIR)/yai" --version; else printf "missing"; fi)"
	@printf "run_dir: %s\n" "$$(if [ -d "$(YAI_RUN_DIR)" ]; then printf "%s" "$(YAI_RUN_DIR)"; else printf "missing"; fi)"
	@printf "store_dir: %s\n" "$$(if [ -d "$(YAI_STORE_DIR)" ]; then printf "%s" "$(YAI_STORE_DIR)"; else printf "missing"; fi)"
	@printf "record_store_path: %s\n" "$(YAI_RECORD_STORE_DIR)"
	@printf "record_store_status: %s\n" "$$(if [ -d "$(YAI_RECORD_STORE_DIR)" ]; then printf "not_initialized"; else printf "missing"; fi)"
	@printf "record_store_backend: lmdb\n"
	@printf "log_dir: %s\n" "$$(if [ -d "$(YAI_LOG_DIR)" ]; then printf "%s" "$(YAI_LOG_DIR)"; else printf "missing"; fi)"
	@printf "tmp_dir: %s\n" "$$(if [ -d "$(YAI_TMP_DIR)" ]; then printf "%s" "$(YAI_TMP_DIR)"; else printf "missing"; fi)"
	@printf "cases_dir: %s\n" "$$(if [ -d "$(YAI_CASES_DIR)" ]; then printf "%s" "$(YAI_CASES_DIR)"; else printf "missing"; fi)"
	@printf "sockets_dir: %s\n" "$$(if [ -d "$(YAI_SOCKETS_DIR)" ]; then printf "%s" "$(YAI_SOCKETS_DIR)"; else printf "missing"; fi)"
	@printf "config_dir: %s\n" "$$(if [ -d "$(YAI_CONFIG_DIR)" ]; then printf "%s" "$(YAI_CONFIG_DIR)"; else printf "missing"; fi)"
	@printf "daemon_socket_default: %s\n" "$(YAI_DAEMON_SOCKET)"
	@printf "hot_state_path: %s/hot-state.json\n" "$(YAI_RUN_DIR)"
	@printf "hot_state_status: %s\n" "$$(if [ -f "$(YAI_RUN_DIR)/hot-state.json" ]; then printf "present"; else printf "missing"; fi)"
	@printf "runtime_layout_status: %s\n" "$$(if [ -d "$(YAI_RUN_DIR)" ] && [ -d "$(YAI_STORE_DIR)" ] && [ -d "$(YAI_LOG_DIR)" ] && [ -d "$(YAI_TMP_DIR)" ] && [ -d "$(YAI_CASES_DIR)" ] && [ -d "$(YAI_SOCKETS_DIR)" ] && [ -d "$(YAI_CONFIG_DIR)" ]; then printf "ok"; else printf "incomplete"; fi)"
	@case ":$$PATH:" in *:"$(INSTALL_BINDIR)":*) printf "PATH_status: ok\n" ;; *) printf "PATH_status: warning add %s to PATH\n" "$(INSTALL_BINDIR)" ;; esac

smoke-new1: $(SMOKE_MINIMUM_LOOP)
	@$(SMOKE_MINIMUM_LOOP)

smoke-new2: $(SMOKE_PERSISTENT_JOURNAL)
	@$(SMOKE_PERSISTENT_JOURNAL)

smoke-new3: $(SMOKE_CONTROL_GATE)
	@$(SMOKE_CONTROL_GATE)

smoke-new4: $(SMOKE_FILESYSTEM_CARRIER)
	@$(SMOKE_FILESYSTEM_CARRIER)

smoke-new5: $(SMOKE_GRAPH_RECONSTRUCTION)
	@$(SMOKE_GRAPH_RECONSTRUCTION)

smoke-new6: $(SMOKE_OPERATIONAL_MEMORY)
	@$(SMOKE_OPERATIONAL_MEMORY)

smoke-new7: $(SMOKE_RECONCILE_DIVERGENCE)
	@$(SMOKE_RECONCILE_DIVERGENCE)

smoke-new8: $(SMOKE_PROJECTION_HARDENING)
	@$(SMOKE_PROJECTION_HARDENING)

smoke-new9: $(SMOKE_QUERY_BOUNDARY)
	@$(SMOKE_QUERY_BOUNDARY)

smoke-new10: $(SMOKE_RUST_ENGINE_R1)
	@$(SMOKE_RUST_ENGINE_R1)

smoke-new11: $(YAID) build-rust
	@$(SMOKE_DAEMON_IPC)

smoke-new12: $(YAID) build-rust
	@$(SMOKE_DAEMON_CORE_LOOP)

smoke-new18b: $(SMOKE_CASE_CONTEXT)
	@$(SMOKE_CASE_CONTEXT)

smoke-new18c: $(SMOKE_INTERACTION_THREAD)
	@$(SMOKE_INTERACTION_THREAD)

smoke-spine23: $(SMOKE_HOT_STATE)
	@$(SMOKE_HOT_STATE)

smoke-spine24: $(YAID) build-rust
	@$(SMOKE_HOT_STATE_SNAPSHOT)

smoke-spine24a: $(YAID) build-rust
	@$(SMOKE_COMMAND_SURFACE)

smoke-spine25: $(SMOKE_HOT_STATE_SESSION)
	@$(SMOKE_HOT_STATE_SESSION)

smoke-spine26: $(SMOKE_PROJECTION_FRESHNESS)
	@$(SMOKE_PROJECTION_FRESHNESS)

smoke-spine27: $(YAID) build-rust
	@$(SMOKE_HOT_STATE_CLI)

smoke-spine29: build-rust
	@$(SMOKE_RECORD_STORE_CLI)

smoke-spine30: $(YAID) build-rust
	@$(SMOKE_RECORD_STORE_WRITE)

smoke-spine31: $(YAID) build-rust
	@$(SMOKE_RECORD_STORE_READ_QUERY)

smoke-spine32: $(YAID) build-rust
	@$(SMOKE_RECORD_STORE_SUBJECT_RECEIPT)

smoke-spine33: $(YAID) build-rust
	@$(SMOKE_RECORD_STORE_CLI_MANUAL)

smoke-spine34: $(YAID) build-rust
	@$(SMOKE_RECORD_STORE_FREEZE)

smoke-spine35: build-rust
	@$(SMOKE_JOURNAL_REPLAY_BOUNDARY)

smoke-spine36: build-rust
	@$(SMOKE_JOURNAL_REPLAY_TO_LMDB)

smoke-spine37: build-rust
	@$(SMOKE_REPLAY_IDEMPOTENCY_SCHEMA)

smoke-spine38: build-rust
	@$(SMOKE_REPLAY_DIAGNOSTICS_REPORT)

smoke-spine39: build-rust
	@$(SMOKE_JOURNAL_REPLAY_FREEZE)

smoke-spine40: $(SMOKE_GRAPH_RUNTIMEGRAPH_SCHEMA) build-rust
	@$(SMOKE_GRAPH_RUNTIMEGRAPH_SCHEMA)
	@$(YAI_BIN) graph schema | grep -F -- "graph_schema:" >/dev/null
	@$(YAI_BIN) graph schema | grep -F -- "- case" >/dev/null
	@$(YAI_BIN) graph schema | grep -F -- "- decision_controls_attempt" >/dev/null
	@$(YAI_BIN) graph runtime-status | grep -F -- "status: active_minimal" >/dev/null
	@$(YAI_BIN) graph runtime-status | grep -F -- "role: in_memory_active_case_working_set" >/dev/null
	@$(YAI_BIN) graph runtime-status | grep -F -- "working_set: per_command_ephemeral" >/dev/null
	@$(YAI_BIN) graph runtime-status | grep -F -- "relation_write_path: active_minimal" >/dev/null
	@printf "graph_schema:node_kinds ok\n"
	@printf "graph_schema:edge_kinds ok\n"
	@printf "runtime_graph:boundary ok\n"
	@printf "runtime_graph:active_minimal ok\n"

smoke-spine41: build-rust
	@$(SMOKE_GRAPH_RELATION_WRITE_PATH)

smoke-spine42: build-rust
	@$(SMOKE_RUNTIMEGRAPH_WORKING_SET)

smoke-spine43: build-rust
	@$(SMOKE_RUNTIMEGRAPH_REBUILD)

smoke-spine44: build-rust
	@$(SMOKE_RUNTIMEGRAPH_QUERY)

smoke-spine44a: $(YAID) build-rust
	@$(SMOKE_OPERATOR_REVIEW_LOOP)

smoke-spine44b: $(YAID) build-rust
	@$(SMOKE_OPERATOR_REVIEW_CLI)

smoke-spine44c: $(YAID) build-rust
	@$(SMOKE_REVIEW_LOOP_TEST_MATRIX)

smoke-spine45: $(YAID) build-rust
	@$(SMOKE_GRAPH_RUNTIMEGRAPH_FREEZE)

smoke-spine46: build-rust
	@$(SMOKE_DUCKDB_FACT_PLANE)

smoke-spine47: $(YAID) build-rust
	@$(SMOKE_RECEIPT_DECISION_PROJECTION_FACTS)

smoke-spine48: $(YAID) build-rust
	@$(SMOKE_MODEL_BEHAVIOR_POLICY_FACTS)

smoke-spine49: $(YAID) build-rust
	@$(SMOKE_MEMORY_DIVERGENCE_CARRIER_FACTS)

smoke-spine50: $(YAID) build-rust
	@$(SMOKE_FACT_REPORTS_CLI)

smoke-spine51: $(YAID) build-rust
	@$(SMOKE_FACT_PLANE_FREEZE)

smoke-spine51b: $(YAID) build-rust
	@$(SMOKE_CASEHANDLE_CAPABILITY_BOUNDARY)

smoke-spine33a: $(SMOKE_CONTROL_CARRIER_SUBSTRATE) build-rust
	@$(SMOKE_CONTROL_CARRIER_SUBSTRATE)
	@$(YAI_BIN) carrier families | grep -F -- "carrier_families:" >/dev/null
	@$(YAI_BIN) carrier families | grep -F -- "- filesystem" >/dev/null
	@$(YAI_BIN) carrier families | grep -F -- "- process" >/dev/null
	@$(YAI_BIN) carrier families | grep -F -- "- network_http" >/dev/null
	@$(YAI_BIN) carrier families | grep -F -- "- database" >/dev/null
	@$(YAI_BIN) carrier families | grep -F -- "- repository_git" >/dev/null
	@$(YAI_BIN) carrier families | grep -F -- "- service" >/dev/null
	@$(YAI_BIN) carrier families | grep -F -- "- endpoint" >/dev/null
	@$(YAI_BIN) carrier families | grep -F -- "- socket" >/dev/null
	@$(YAI_BIN) carrier families | grep -F -- "- listener" >/dev/null
	@$(YAI_BIN) carrier families | grep -F -- "- model_provider" >/dev/null
	@$(YAI_BIN) carrier families | grep -F -- "- observation" >/dev/null
	@$(YAI_BIN) carrier families | grep -F -- "- review" >/dev/null

smoke-spine33b: $(SMOKE_OPERATION_DISPATCH_MULTIPLEX) build-rust
	@$(SMOKE_OPERATION_DISPATCH_MULTIPLEX)
	@$(YAI_BIN) carrier lanes | grep -F -- "filesystem_lane" >/dev/null
	@$(YAI_BIN) carrier lanes | grep -F -- "process_lane" >/dev/null
	@$(YAI_BIN) carrier lanes | grep -F -- "network_http_lane" >/dev/null
	@$(YAI_BIN) carrier route --family filesystem | grep -F -- "dispatch_status: routable" >/dev/null
	@$(YAI_BIN) carrier route --family filesystem | grep -F -- "execution_performed: false" >/dev/null
	@$(YAI_BIN) carrier route --family process | grep -F -- "lane: process_lane" >/dev/null
	@$(YAI_BIN) carrier route --family unknown | grep -F -- "dispatch_status: not_supported" >/dev/null

smoke-spine33c: $(SMOKE_CARRIER_CONTRACT_FILESYSTEM) $(YAID) build-rust
	@$(SMOKE_CARRIER_CONTRACT_FILESYSTEM)
	@$(YAI_BIN) carrier inspect filesystem | grep -F -- "carrier: filesystem" >/dev/null
	@$(YAI_BIN) carrier inspect filesystem | grep -F -- "contract: carrier.v1" >/dev/null
	@$(YAI_BIN) carrier inspect filesystem | grep -F -- "receipt_required: yes" >/dev/null
	@$(YAI_BIN) carrier inspect filesystem | grep -F -- "read: observed" >/dev/null
	@$(YAI_BIN) carrier inspect filesystem | grep -F -- "write: decision_required" >/dev/null

smoke-spine33d: $(SMOKE_PROCESS_CARRIER) build-rust
	@$(SMOKE_PROCESS_CARRIER)
	@$(YAI_BIN) carrier inspect process | grep -F -- "carrier: process" >/dev/null
	@$(YAI_BIN) carrier inspect process | grep -F -- "contract: carrier.v1" >/dev/null
	@$(YAI_BIN) process signal --pid 999999 --signal TERM --dry-run | grep -F -- "carrier_attempted: false" >/dev/null
	@$(YAI_BIN) process signal --pid 999999 --signal KILL | grep -F -- "reason: unsafe_process_target" >/dev/null

smoke-core-enforce1: $(SMOKE_CONTROL_LEASE_DISPATCH)
	@$(SMOKE_CONTROL_LEASE_DISPATCH)

smoke-spine33e: $(SMOKE_HOST_OBSERVATION_PROBE) build-rust
	@$(SMOKE_HOST_OBSERVATION_PROBE)
	@$(YAI_BIN) observe process --pid $$$$ | grep -F -- "observation_is_enforcement: false" >/dev/null
	@$(YAI_BIN) observe compare-process --pid $$$$ --expected running | grep -F -- "result: matched" >/dev/null
	@$(YAI_BIN) observe compare-process --pid $$$$ --expected stopped | grep -F -- "divergence_candidate: expected_stopped_but_running" >/dev/null

smoke-spine33f: $(SMOKE_CARRIER_COVERAGE_MATRIX) build-rust
	@$(SMOKE_CARRIER_COVERAGE_MATRIX)
	@$(YAI_BIN) carrier coverage | grep -F -- "carrier_coverage:" >/dev/null
	@$(YAI_BIN) carrier coverage --family filesystem | grep -F -- "controlled: active_minimal" >/dev/null
	@$(YAI_BIN) carrier coverage --family process | grep -F -- "observed: active_minimal" >/dev/null
	@$(YAI_BIN) carrier coverage --family database | grep -F -- "execution_available: false" >/dev/null
	@$(YAI_BIN) carrier coverage --family unknown | grep -F -- "controlled: unsupported" >/dev/null

smoke-spine33g: $(SMOKE_NON_PROCESS_CARRIER_SKELETONS) build-rust
	@$(SMOKE_NON_PROCESS_CARRIER_SKELETONS)
	@for family in network_http database repository_git service endpoint socket listener model_provider review; do \
		$(YAI_BIN) carrier inspect $$family | grep -F -- "carrier: $$family" >/dev/null; \
		$(YAI_BIN) carrier inspect $$family | grep -F -- "contract: carrier.v1" >/dev/null; \
		$(YAI_BIN) carrier inspect $$family | grep -F -- "status: skeleton" >/dev/null; \
		$(YAI_BIN) carrier inspect $$family | grep -F -- "execution_available: false" >/dev/null; \
		$(YAI_BIN) carrier inspect $$family | grep -F -- "receipt_required: yes" >/dev/null; \
	done
	@$(YAI_BIN) carrier coverage --family database | grep -F -- "carrier_contract: carrier.v1" >/dev/null
	@$(YAI_BIN) carrier coverage --family model_provider | grep -F -- "imported: skeleton" >/dev/null

smoke-spine33h: $(SMOKE_CARRIER_OUTCOME_HARNESS) build-rust
	@$(SMOKE_CARRIER_OUTCOME_HARNESS)
	@$(YAI_BIN) carrier outcome-test --family filesystem --outcome blocked | grep -F -- "reason: policy_blocked_harness" >/dev/null
	@$(YAI_BIN) carrier outcome-test --family process --outcome blocked | grep -F -- "reason: unsafe_or_policy_blocked" >/dev/null
	@$(YAI_BIN) carrier outcome-test --family database --outcome blocked | grep -F -- "execution_performed: false" >/dev/null
	@$(YAI_BIN) carrier outcome-test --family database --outcome blocked | grep -F -- "carrier_attempted: false" >/dev/null
	@$(YAI_BIN) carrier outcome-test --family network_http --outcome failed | grep -F -- "reason: simulated_failure_posture" >/dev/null
	@$(YAI_BIN) carrier outcome-test --family repository_git --mode observed --outcome mismatch | grep -F -- "divergence_candidate: generated" >/dev/null
	@$(YAI_BIN) carrier outcome-test --family service --outcome quarantined | grep -F -- "carrier_attempted: false" >/dev/null
	@$(YAI_BIN) carrier outcome-test --family endpoint --mode observed --outcome observed | grep -F -- "effective_outcome: observed" >/dev/null
	@$(YAI_BIN) carrier outcome-test --family socket --outcome deferred | grep -F -- "effective_outcome: deferred" >/dev/null
	@$(YAI_BIN) carrier outcome-test --family listener --outcome not_attempted | grep -F -- "effective_outcome: not_attempted" >/dev/null
	@$(YAI_BIN) carrier outcome-test --family model_provider --outcome waiting_operator | grep -F -- "effective_outcome: waiting_operator" >/dev/null
	@$(YAI_BIN) carrier outcome-test --family review --outcome waiting_agent | grep -F -- "effective_outcome: waiting_agent" >/dev/null
	@$(YAI_BIN) carrier outcome-test --family unknown --outcome blocked | grep -F -- "carrier_status: unsupported" >/dev/null
	@$(YAI_BIN) carrier outcome-test --family unknown --outcome blocked | grep -F -- "receipt_required: no" >/dev/null
	@$(YAI_BIN) carrier outcome-test --family unknown --outcome blocked | grep -F -- "receipt_posture: none" >/dev/null
	@$(YAI_BIN) carrier outcome-test --family unknown --outcome blocked | grep -F -- "reason: unsupported_carrier_family" >/dev/null
	@if $(YAI_BIN) carrier outcome-test --family database --outcome impossible >/tmp/yai-invalid-outcome.out 2>&1; then \
		cat /tmp/yai-invalid-outcome.out; \
		exit 1; \
	fi
	@grep -F -- "error: unsupported_outcome" /tmp/yai-invalid-outcome.out >/dev/null
	@rm -f /tmp/yai-invalid-outcome.out
	@printf "outcome:filesystem blocked ok\n"
	@printf "outcome:process blocked ok\n"
	@printf "outcome:database blocked simulated\n"
	@printf "outcome:network_http failed simulated\n"
	@printf "outcome:repository_git mismatch simulated divergence_candidate\n"
	@printf "outcome:service quarantined simulated\n"
	@printf "outcome:endpoint observed simulated\n"
	@printf "outcome:socket deferred simulated\n"
	@printf "outcome:listener not_attempted simulated\n"
	@printf "outcome:model_provider waiting_operator simulated\n"
	@printf "outcome:review waiting_agent simulated\n"
	@printf "outcome:unknown not_attempted ok\n"
	@printf "no_execution:skeletons confirmed\n"

smoke-spine33i: $(SMOKE_CARRIER_RECEIPT_DIVERGENCE) build-rust
	@$(SMOKE_CARRIER_RECEIPT_DIVERGENCE)
	@$(YAI_BIN) carrier reconcile-outcome --scenario clean_executed | grep -F -- "result: consistent" >/dev/null
	@$(YAI_BIN) carrier reconcile-outcome --scenario clean_blocked | grep -F -- "divergence_candidate: none" >/dev/null
	@$(YAI_BIN) carrier reconcile-outcome --scenario denied_but_attempted | grep -F -- "divergence_candidate: denied_but_attempted" >/dev/null
	@$(YAI_BIN) carrier reconcile-outcome --scenario denied_but_attempted | grep -F -- "severity: critical" >/dev/null
	@$(YAI_BIN) carrier reconcile-outcome --scenario executed_without_receipt | grep -F -- "divergence_candidate: executed_without_receipt" >/dev/null
	@$(YAI_BIN) carrier reconcile-outcome --scenario executed_without_receipt | grep -F -- "severity: error" >/dev/null
	@$(YAI_BIN) carrier reconcile-outcome --scenario blocked_but_effect_observed | grep -F -- "observed_effect: yes" >/dev/null
	@$(YAI_BIN) carrier reconcile-outcome --scenario blocked_but_effect_observed | grep -F -- "severity: critical" >/dev/null
	@$(YAI_BIN) carrier reconcile-outcome --scenario receipt_claimed_executed_but_not_observed | grep -F -- "divergence_candidate: receipt_claimed_executed_but_not_observed" >/dev/null
	@$(YAI_BIN) carrier reconcile-outcome --scenario failed_with_partial_effect | grep -F -- "divergence_candidate: failed_with_partial_effect" >/dev/null
	@$(YAI_BIN) carrier reconcile-outcome --scenario skeleton_executed_unexpectedly | grep -F -- "divergence_candidate: skeleton_executed_unexpectedly" >/dev/null
	@$(YAI_BIN) carrier reconcile-outcome --scenario skeleton_executed_unexpectedly | grep -F -- "execution_side_effect: none" >/dev/null
	@printf "reconcile:clean_executed consistent\n"
	@printf "reconcile:clean_blocked consistent\n"
	@printf "reconcile:denied_but_attempted critical\n"
	@printf "reconcile:executed_without_receipt error\n"
	@printf "reconcile:blocked_but_effect_observed critical\n"
	@printf "reconcile:failed_with_partial_effect error\n"
	@printf "reconcile:skeleton_executed_unexpectedly critical\n"

smoke-spine33l: build-rust
	@$(SMOKE_PROVIDER_RUNTIME_SURFACE)

smoke: smoke-new1 smoke-new2 smoke-new3 smoke-new4 smoke-new5 smoke-new6 smoke-new7 smoke-new8 smoke-new9 smoke-new10 smoke-new11 smoke-new12 smoke-new18b smoke-new18c smoke-spine23 smoke-spine24 smoke-spine24a smoke-spine25 smoke-spine26 smoke-spine27 smoke-spine29 smoke-spine30 smoke-spine31 smoke-spine32 smoke-spine33 smoke-spine33a smoke-spine33b smoke-spine33c smoke-spine33d smoke-spine33e smoke-spine33f smoke-spine33g smoke-spine33h smoke-spine33i smoke-spine33l smoke-spine34 smoke-spine35 smoke-spine36 smoke-spine37 smoke-spine38 smoke-spine39 smoke-spine40 smoke-spine41 smoke-spine42 smoke-spine43 smoke-spine44 smoke-spine44a smoke-spine44b smoke-spine44c smoke-spine45 smoke-spine46 smoke-spine47 smoke-spine48 smoke-spine49 smoke-spine50 smoke-spine51 smoke-spine51b smoke-core-enforce1

check: check-layout check-docs build smoke

clean:
	rm -rf "$(BUILD_DIR)" "$(RUST_TARGET_DIR)" engine/target cmd/yai/target
