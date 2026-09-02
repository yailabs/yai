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
.PHONY: info check-layout check-docs check-doc-links check-doc-root-canon
.PHONY: check-labs check-lab-runs check-lab-notebooks check-context-residency-lab
.PHONY: lab-context-residency smoke-lab-context-residency smoke-lab-context-residency-case-native smoke-lab-context-residency-matrix
.PHONY: check-repository-identity check-archive-historical-records check-source-surface-clean check-foundation-freeze
.PHONY: build-c build-rust build install-local uninstall-local doctor-local print-install-paths
.PHONY: smoke-new1 smoke-new2 smoke-new3 smoke-new4 smoke-new8 smoke-new11 smoke-new12 smoke-new18b
.PHONY: smoke-spine23 smoke-spine24 smoke-spine24a smoke-spine25 smoke-spine26 smoke-spine27 smoke-spine29 smoke-spine30 smoke-spine31 smoke-spine32 smoke-spine33
.PHONY: smoke-spine33c smoke-spine33d smoke-spine33e smoke-spine34 smoke-spine35 smoke-spine36 smoke-spine37 smoke-spine38 smoke-spine39 smoke-spine40
.PHONY: smoke-spine41 smoke-spine42 smoke-spine43 smoke-spine44 smoke-spine45 smoke-spine46 smoke-spine47
.PHONY: smoke-spine48 smoke-spine49 smoke-spine50 smoke-spine51 smoke-controlled-effect smoke-semantic-continuity smoke-agentless-case-runtime smoke-human-review-runtime smoke-governance-intake smoke-governance-hardening smoke-case-policy-materialization smoke-policy-authority-admission smoke-policy-authority-hardening smoke-temporal-governance smoke-tenant-security smoke-multi-case-runtime smoke-multi-case-runtime-hardening smoke-shared-resource-fencing smoke-shared-resource-fencing-hardening smoke-second-carrier smoke-workflow-kernel smoke-workflow-kernel-hardening smoke-cli-product-surface smoke-adaptive-workflow smoke-adaptive-workflow-hardening smoke-provider-governance qualification-yvex-provider endurance-agentless-case-runtime characterization smoke check clean

CC ?= cc
AR ?= ar
CFLAGS ?= -std=c11 -Wall -Wextra -Werror -Iinclude
PREFIX ?= $(HOME)/.local
YAI_HOME ?= $(HOME)/.yai
BUILD_DIR := build
RUST_TARGET_DIR := target
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

C_PRODUCT_SOURCES := \
	system/internal/string_util.c \
	system/base/id.c \
	system/base/error.c \
	system/hot/hot_snapshot.c \
	system/hot/hot_state.c \
	system/case/case_context.c \
	system/case/case_ref.c \
	system/case/case_session.c \
	system/case/case_world.c \
	system/subject/subject_ref.c \
	system/daemon/daemon_status.c \
	system/store/record.c \
	system/store/journal.c \
	system/store/record_codec.c \
	system/store/journal_file.c \
	system/projection/projection.c

C_COMPONENT_SOURCES := \
	system/internal/string_util.c \
	system/base/id.c \
	system/base/error.c \
	system/hot/hot_flags.c \
	system/hot/hot_snapshot.c \
	system/hot/hot_state.c \
	system/case/case_context.c \
	system/case/case_ref.c \
	system/case/case_session.c \
	system/case/case_world.c \
	system/subject/subject_ref.c \
	system/subject/subject_binding.c \
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
	system/daemon/daemon_status.c \
	system/effect/carrier.c \
	system/effect/carrier_contract.c \
	system/effect/carrier_family.c \
	system/effect/carrier_outcome.c \
	system/effect/carrier_receipt.c \
	system/effect/effect_hash.c \
	system/effect/receipt.c \
	system/effect/receipt_guarantee.c \
	system/effect/carriers/filesystem_carrier.c \
	system/effect/carriers/process_carrier.c \
	system/effect/process_signal.c \
	system/effect/process_state.c \
	system/observation/host_observation.c \
	system/observation/host_probe.c \
	system/observation/observation_result.c \
	system/observation/observation_target.c \
	system/reconcile/divergence_candidate.c \
	system/store/record.c \
	system/store/journal.c \
	system/store/record_codec.c \
	system/store/journal_file.c \
	system/projection/projection.c \
	system/projection/projection_kind.c \
	system/projection/redaction.c \
	system/projection/freshness.c \
	system/projection/projection_request.c \
	system/projection/projection_result.c \
	system/projection/visibility_scope.c

C_PRODUCT_OBJECTS := $(patsubst %.c,$(BUILD_DIR)/product/%.o,$(C_PRODUCT_SOURCES))
C_COMPONENT_OBJECTS := $(patsubst %.c,$(BUILD_DIR)/component/%.o,$(C_COMPONENT_SOURCES))
C_LIBRARY := $(BUILD_DIR)/libyai_core_new13.a
C_COMPONENT_LIBRARY := $(BUILD_DIR)/libyai_component_characterization.a
YAID := $(BUILD_DIR)/yaid
SMOKE_MINIMUM_LOOP := $(BUILD_DIR)/test_minimum_loop
SMOKE_PERSISTENT_JOURNAL := $(BUILD_DIR)/test_persistent_journal
SMOKE_CONTROL_GATE := $(BUILD_DIR)/test_control_gate
SMOKE_FILESYSTEM_CARRIER := $(BUILD_DIR)/test_filesystem_carrier
SMOKE_PROJECTION_HARDENING := $(BUILD_DIR)/test_projection_hardening
SMOKE_CASE_CONTEXT := $(BUILD_DIR)/test_case_context
SMOKE_HOT_STATE := $(BUILD_DIR)/test_hot_state
SMOKE_CARRIER_CONTRACT_FILESYSTEM := $(BUILD_DIR)/test_carrier_contract_filesystem
SMOKE_PROCESS_CARRIER := $(BUILD_DIR)/test_process_carrier
SMOKE_HOST_OBSERVATION_PROBE := $(BUILD_DIR)/test_host_observation_probe
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
SMOKE_GRAPH_RELATION_WRITE_PATH := tests/smoke/graph-relation-write-path/test_graph_relation_write_path.sh
SMOKE_RUNTIMEGRAPH_WORKING_SET := tests/smoke/runtimegraph-working-set/test_runtimegraph_working_set.sh
SMOKE_RUNTIMEGRAPH_REBUILD := tests/smoke/runtimegraph-rebuild/test_runtimegraph_rebuild.sh
SMOKE_RUNTIMEGRAPH_QUERY := tests/smoke/runtimegraph-query/test_runtimegraph_query.sh
SMOKE_GRAPH_RUNTIMEGRAPH_FREEZE := tests/smoke/graph-runtimegraph-freeze/test_graph_runtimegraph_freeze.sh
SMOKE_DUCKDB_FACT_PLANE := tests/smoke/duckdb-fact-plane/test_duckdb_fact_plane.sh
SMOKE_RECEIPT_DECISION_PROJECTION_FACTS := tests/smoke/receipt-decision-projection-facts/test_receipt_decision_projection_facts.sh
SMOKE_MODEL_BEHAVIOR_POLICY_FACTS := tests/smoke/model-behavior-policy-facts/test_model_behavior_policy_facts.sh
SMOKE_MEMORY_DIVERGENCE_CARRIER_FACTS := tests/smoke/memory-divergence-carrier-facts/test_memory_divergence_carrier_facts.sh
SMOKE_FACT_REPORTS_CLI := tests/smoke/fact-reports-cli/test_fact_reports_cli.sh
SMOKE_FACT_PLANE_FREEZE := tests/smoke/fact-plane-freeze/test_fact_plane_freeze.sh
SMOKE_DAEMON_IPC := tests/smoke/daemon-ipc/test_daemon_ipc.sh
SMOKE_DAEMON_CORE_LOOP := tests/smoke/daemon-core-loop/test_daemon_core_loop.sh
CHARACTERIZATION_PROVIDER_MODEL := tests/characterization/provider-model-vertical/test_provider_model_vertical.sh
CHARACTERIZATION_SEMANTIC_CONTINUITY := tests/characterization/provider-semantic-continuity/test_provider_semantic_continuity.sh
CHARACTERIZATION_DIRECT_FILESYSTEM := tests/characterization/direct-filesystem-bypass/test_direct_filesystem_bypass.sh
CHARACTERIZATION_CONTROLLED_EFFECT := tests/characterization/controlled-effect-vertical/test_controlled_effect_vertical.sh
CHARACTERIZATION_AGENTLESS_CASE_RUNTIME := tests/characterization/agentless-case-runtime/test_agentless_case_runtime.sh
CHARACTERIZATION_HUMAN_REVIEW_RUNTIME := tests/characterization/human-review-runtime/test_human_review_runtime.sh
CHARACTERIZATION_GOVERNANCE_INTAKE := tests/characterization/governance-intake/test_governance_intake.sh
CHARACTERIZATION_GOVERNANCE_HARDENING := tests/characterization/governance-hardening/test_governance_hardening.sh
CHARACTERIZATION_CASE_POLICY := tests/characterization/case-policy-materialization/test_case_policy_materialization.sh
CHARACTERIZATION_POLICY_AUTHORITY := tests/characterization/policy-authority-admission/test_policy_authority_admission.sh
CHARACTERIZATION_POLICY_AUTHORITY_HARDENING := tests/characterization/policy-authority-hardening/test_policy_authority_hardening.sh
CHARACTERIZATION_TEMPORAL_GOVERNANCE := tests/characterization/temporal-governance/test_temporal_governance.sh
CHARACTERIZATION_TENANT_SECURITY := tests/characterization/tenant-security/test_tenant_security.sh
CHARACTERIZATION_MULTI_CASE_RUNTIME := tests/characterization/multi-case-runtime/test_multi_case_runtime.sh
CHARACTERIZATION_MULTI_CASE_RUNTIME_HARDENING := tests/characterization/multi-case-runtime-hardening/test_terminal_ack_reproduction.sh
CHARACTERIZATION_SHARED_RESOURCE_FENCING := tests/characterization/shared-resource-fencing/test_shared_resource_fencing.sh
CHARACTERIZATION_SHARED_RESOURCE_FENCING_HARDENING := tests/characterization/shared-resource-fencing-hardening/test_shared_resource_fencing_hardening.sh
CHARACTERIZATION_SECOND_CARRIER := tests/characterization/second-carrier/test_second_carrier.sh
CHARACTERIZATION_WORKFLOW_KERNEL := tests/characterization/workflow-kernel/test_workflow_kernel.sh
CHARACTERIZATION_WORKFLOW_MODELWORK := tests/characterization/workflow-kernel/test_workflow_modelwork.sh
CHARACTERIZATION_WORKFLOW_RESOURCE_BUSY := tests/characterization/workflow-kernel/test_workflow_resource_busy.sh
CHARACTERIZATION_WORKFLOW_REVIEW := tests/characterization/workflow-kernel/test_workflow_review.sh
CHARACTERIZATION_WORKFLOW_KERNEL_HARDENING := tests/characterization/workflow-kernel-hardening/test_workflow_kernel_hardening.sh
CHARACTERIZATION_CLI_PRODUCT_SURFACE := tests/characterization/cli-product-surface/test_cli_product_surface.sh
CHARACTERIZATION_ADAPTIVE_WORKFLOW := tests/characterization/adaptive-workflow/test_adaptive_workflow.sh
CHARACTERIZATION_ADAPTIVE_WORKFLOW_HARDENING := tests/characterization/adaptive-workflow-hardening/test_adaptive_workflow_hardening.sh
CHARACTERIZATION_PROVIDER_GOVERNANCE := tests/characterization/provider-governance/test_provider_governance.sh
QUALIFICATION_YVEX_PROVIDER := tests/integration/yvex/qualification_yvex_provider.sh

info:
	@printf "yai: admitted operational-state transition system with one controlled filesystem vertical\n"
	@printf "foundation-recovery-baseline: 3403ecdd2a321b689e41d747cbeb9d9e7c58e5e1\n"
	@printf "documentation: docs/index.md\n"
	@printf "roadmap: ROADMAP.md\n"
	@printf "source-layout: include/ system/ engine/ cmd/\n"
	@printf "runtime-home: YAI_HOME=%s socket=%s\n" "$(YAI_HOME)" "$(YAI_DAEMON_SOCKET)"
	@printf "hot-state: %s/hot-state.json\n" "$(YAI_RUN_DIR)"
	@printf "record-store: %s\n" "$(YAI_RECORD_STORE_DIR)"
	@printf "fact-plane: DuckDB yai.fact.v1 extraction plus compact CLI reports\n"
	@printf "state-authority: LMDB yai.transition.v7 plus atomic yai.case_state.v7; v1-v6 readable\n"
	@printf "legacy-journal: yai.store.record.v0 compatibility input/export only\n"
	@printf "effect-boundary: controlled filesystem.write Grant/PREPARE/FINALIZE plus reconciliation\n"
	@printf "c-product: narrow yaid/store/hot/projection dependency set\n"
	@printf "c-components: separately linked characterization library\n"
	@printf "engine-bridge: removed; no product C-to-Rust call edge\n"
	@printf "lib: removed\n"
	@printf "daemon: moved to cmd/yaid + system/daemon\n"
	@printf "provider-runtime: agentless bounded Case loop with typed Projection/ContextFrame lineage\n"
	@printf "semantic-residency: derived yai.residency_plan.v1; explicit item/context budgets\n"
	@printf "operational-memory: derived yai.operational_memory.v1 with qualified retrieval and canonical fallback\n"
	@printf "governance-intake: owner-scoped immutable yai.policy_artifact.v4 + typed yai.policy_ir.v2 temporal lifecycle\n"
	@printf "provider-registry: removed; case-bound invocation remains\n"
	@printf "crates: removed\n"
	@printf "ctl: removed\n"
	@printf "install-local: active PREFIX=%s YAI_HOME=%s\n" "$(PREFIX)" "$(YAI_HOME)"

check-layout:
	@./tools/checks/check-no-old-roots.sh
	@./tools/checks/check-required-layout.sh
	@./tools/checks/check-source-placement.sh
	@./tools/checks/check-source-surface-clean.sh

check-docs:
	@./tools/checks/check-doc-root-canon.sh
	@./tools/checks/check-doc-canonical-location.sh
	@./tools/checks/check-doc-required-files.sh
	@python3 tools/checks/check-doc-links.py
	@./tools/checks/check-repository-identity.sh

check-doc-links:
	@python3 tools/checks/check-doc-links.py

check-doc-root-canon:
	@./tools/checks/check-doc-root-canon.sh

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

check-foundation-freeze:
	@./tools/checks/check-foundation-freeze.sh

$(BUILD_DIR)/product/%.o: %.c
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) -c "$<" -o "$@"

$(BUILD_DIR)/component/%.o: %.c
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) -c "$<" -o "$@"

$(C_LIBRARY): $(C_PRODUCT_OBJECTS)
	@mkdir -p "$(dir $@)"
	$(RM) "$@"
	$(AR) rcs "$@" $(C_PRODUCT_OBJECTS)

$(C_COMPONENT_LIBRARY): $(C_COMPONENT_OBJECTS)
	@mkdir -p "$(dir $@)"
	$(RM) "$@"
	$(AR) rcs "$@" $(C_COMPONENT_OBJECTS)

$(YAID): cmd/yaid/main.c system/daemon/ipc.c system/daemon/core_loop.c $(C_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) cmd/yaid/main.c system/daemon/ipc.c system/daemon/core_loop.c $(C_LIBRARY) -o "$@"

$(SMOKE_MINIMUM_LOOP): tests/smoke/minimum-loop/test_minimum_loop.c $(C_COMPONENT_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/minimum-loop/test_minimum_loop.c $(C_COMPONENT_LIBRARY) -o "$@"

$(SMOKE_PERSISTENT_JOURNAL): tests/smoke/persistent-journal/test_persistent_journal.c $(C_COMPONENT_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/persistent-journal/test_persistent_journal.c $(C_COMPONENT_LIBRARY) -o "$@"

$(SMOKE_CONTROL_GATE): tests/smoke/control-gate/test_control_gate.c $(C_COMPONENT_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/control-gate/test_control_gate.c $(C_COMPONENT_LIBRARY) -o "$@"

$(SMOKE_FILESYSTEM_CARRIER): tests/smoke/filesystem-carrier/test_filesystem_carrier.c $(C_COMPONENT_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/filesystem-carrier/test_filesystem_carrier.c $(C_COMPONENT_LIBRARY) -o "$@"


	@mkdir -p "$(dir $@)"

	@mkdir -p "$(dir $@)"

$(SMOKE_PROJECTION_HARDENING): tests/smoke/projection-hardening/test_projection_hardening.c $(C_COMPONENT_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/projection-hardening/test_projection_hardening.c $(C_COMPONENT_LIBRARY) -o "$@"

	@mkdir -p "$(dir $@)"

$(SMOKE_CASE_CONTEXT): tests/smoke/case-context/test_case_context.c $(C_COMPONENT_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/case-context/test_case_context.c $(C_COMPONENT_LIBRARY) -o "$@"


$(SMOKE_HOT_STATE): tests/smoke/hot-state/test_hot_state.c $(C_COMPONENT_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/hot-state/test_hot_state.c $(C_COMPONENT_LIBRARY) -o "$@"

	@mkdir -p "$(dir $@)"

	@mkdir -p "$(dir $@)"

$(SMOKE_CARRIER_CONTRACT_FILESYSTEM): tests/smoke/carrier-contract-filesystem/test_carrier_contract_filesystem.c $(C_COMPONENT_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/carrier-contract-filesystem/test_carrier_contract_filesystem.c $(C_COMPONENT_LIBRARY) -o "$@"

$(SMOKE_PROCESS_CARRIER): tests/smoke/process-carrier/test_process_carrier.c $(C_COMPONENT_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/process-carrier/test_process_carrier.c $(C_COMPONENT_LIBRARY) -o "$@"

$(SMOKE_HOST_OBSERVATION_PROBE): tests/smoke/host-observation-probe/test_host_observation_probe.c $(C_COMPONENT_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/host-observation-probe/test_host_observation_probe.c $(C_COMPONENT_LIBRARY) -o "$@"

	@mkdir -p "$(dir $@)"

	@mkdir -p "$(dir $@)"

	@mkdir -p "$(dir $@)"

	@mkdir -p "$(dir $@)"

	@mkdir -p "$(dir $@)"

$(SMOKE_HOT_STATE_SESSION): tests/smoke/hot-state-session/test_hot_state_session.c $(C_COMPONENT_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/hot-state-session/test_hot_state_session.c $(C_COMPONENT_LIBRARY) -o "$@"

$(SMOKE_PROJECTION_FRESHNESS): tests/smoke/projection-freshness/test_projection_freshness.c $(C_COMPONENT_LIBRARY)
	@mkdir -p "$(dir $@)"
	$(CC) $(CFLAGS) tests/smoke/projection-freshness/test_projection_freshness.c $(C_COMPONENT_LIBRARY) -o "$@"

build-c: $(C_LIBRARY) $(YAID)

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




smoke-new8: $(SMOKE_PROJECTION_HARDENING)
	@$(SMOKE_PROJECTION_HARDENING)


smoke-new11: $(YAID) build-rust
	@$(SMOKE_DAEMON_IPC)

smoke-new12: $(YAID) build-rust
	@$(SMOKE_DAEMON_CORE_LOOP)

smoke-new18b: $(SMOKE_CASE_CONTEXT)
	@$(SMOKE_CASE_CONTEXT)


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

smoke-spine40: build-rust
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

smoke-controlled-effect: $(YAID) build-rust
	@$(CHARACTERIZATION_CONTROLLED_EFFECT)




smoke-spine33c: $(SMOKE_CARRIER_CONTRACT_FILESYSTEM)
	@$(SMOKE_CARRIER_CONTRACT_FILESYSTEM)

smoke-spine33d: $(SMOKE_PROCESS_CARRIER) build-rust
	@$(SMOKE_PROCESS_CARRIER)
	@$(YAI_BIN) process signal --pid 999999 --signal TERM --dry-run | grep -F -- "carrier_attempted: false" >/dev/null
	@$(YAI_BIN) process signal --pid 999999 --signal KILL | grep -F -- "reason: unsafe_process_target" >/dev/null

smoke-spine33e: $(SMOKE_HOST_OBSERVATION_PROBE) build-rust
	@$(SMOKE_HOST_OBSERVATION_PROBE)
	@$(YAI_BIN) process observe --pid $$$$ | grep -F -- "observation_is_enforcement: false" >/dev/null
	@$(YAI_BIN) observe compare-process --pid $$$$ --expected running | grep -F -- "result: matched" >/dev/null
	@$(YAI_BIN) observe compare-process --pid $$$$ --expected stopped | grep -F -- "divergence_candidate: expected_stopped_but_running" >/dev/null






smoke: smoke-new1 smoke-new2 smoke-new3 smoke-new4 smoke-new8 smoke-new11 smoke-new12 smoke-new18b smoke-spine23 smoke-spine24 smoke-spine24a smoke-spine25 smoke-spine26 smoke-spine27 smoke-spine29 smoke-spine30 smoke-spine31 smoke-spine32 smoke-spine33 smoke-spine33c smoke-spine33d smoke-spine33e smoke-spine34 smoke-spine35 smoke-spine36 smoke-spine37 smoke-spine38 smoke-spine39 smoke-spine40 smoke-spine41 smoke-spine42 smoke-spine43 smoke-spine44 smoke-spine45 smoke-spine46 smoke-spine47 smoke-spine48 smoke-spine49 smoke-spine50 smoke-spine51 smoke-controlled-effect smoke-semantic-continuity smoke-agentless-case-runtime smoke-human-review-runtime smoke-governance-intake smoke-governance-hardening smoke-case-policy-materialization smoke-policy-authority-admission

smoke-semantic-continuity: $(YAID) build-rust
	@$(CHARACTERIZATION_SEMANTIC_CONTINUITY)

smoke-agentless-case-runtime: $(YAID) build-rust
	@$(CHARACTERIZATION_AGENTLESS_CASE_RUNTIME)

smoke-human-review-runtime: $(YAID) build-rust
	@$(CHARACTERIZATION_HUMAN_REVIEW_RUNTIME)

smoke-governance-intake: build-rust
	@$(CHARACTERIZATION_GOVERNANCE_INTAKE)

smoke-governance-hardening: build-rust
	@$(CHARACTERIZATION_GOVERNANCE_HARDENING)

smoke-case-policy-materialization: $(YAID) build-rust
	@$(CHARACTERIZATION_CASE_POLICY)

smoke-policy-authority-admission: $(YAID) build-rust
	@$(CHARACTERIZATION_POLICY_AUTHORITY)

smoke-policy-authority-hardening:
	@$(CHARACTERIZATION_POLICY_AUTHORITY_HARDENING)

smoke-temporal-governance: $(YAID) build-rust
	@$(CHARACTERIZATION_TEMPORAL_GOVERNANCE)

smoke-tenant-security: build-rust
	@$(CHARACTERIZATION_TENANT_SECURITY)

smoke-multi-case-runtime: $(YAID) build-rust
	@$(CHARACTERIZATION_MULTI_CASE_RUNTIME)

smoke-multi-case-runtime-hardening: $(YAID) build-rust
	@$(CHARACTERIZATION_MULTI_CASE_RUNTIME_HARDENING)

smoke-shared-resource-fencing: build-rust
	@$(CHARACTERIZATION_SHARED_RESOURCE_FENCING)

smoke-shared-resource-fencing-hardening: build-rust
	@$(CHARACTERIZATION_SHARED_RESOURCE_FENCING_HARDENING)
	@tests/characterization/shared-resource-fencing-hardening/test_process_uncertainty.sh

smoke-second-carrier: build-rust
	@$(CHARACTERIZATION_SECOND_CARRIER)

smoke-workflow-kernel: $(YAID) build-rust
	@$(CHARACTERIZATION_WORKFLOW_KERNEL)
	@$(CHARACTERIZATION_WORKFLOW_MODELWORK)
	@$(CHARACTERIZATION_WORKFLOW_RESOURCE_BUSY)
	@$(CHARACTERIZATION_WORKFLOW_REVIEW)

smoke-workflow-kernel-hardening: build-rust
	@$(CHARACTERIZATION_WORKFLOW_KERNEL_HARDENING)

smoke-cli-product-surface: build-rust
	@$(CHARACTERIZATION_CLI_PRODUCT_SURFACE)
	@python3 tests/characterization/cli-product-surface/audit_registry.py --binary ./yai

smoke-adaptive-workflow: build-rust
	@$(CHARACTERIZATION_ADAPTIVE_WORKFLOW)

smoke-adaptive-workflow-hardening: build-rust
	@$(CHARACTERIZATION_ADAPTIVE_WORKFLOW_HARDENING)

smoke-provider-governance: build-rust
	@$(CHARACTERIZATION_PROVIDER_GOVERNANCE)

qualification-yvex-provider: build-rust
	@$(QUALIFICATION_YVEX_PROVIDER)

endurance-agentless-case-runtime: smoke-agentless-case-runtime

characterization: smoke-new4 smoke-new11 smoke-new12 smoke-spine39 smoke-spine45 smoke-spine51
	@$(CHARACTERIZATION_PROVIDER_MODEL)
	@$(CHARACTERIZATION_SEMANTIC_CONTINUITY)
	@$(CHARACTERIZATION_DIRECT_FILESYSTEM)
	@$(CHARACTERIZATION_CONTROLLED_EFFECT)
	@$(CHARACTERIZATION_AGENTLESS_CASE_RUNTIME)
	@$(CHARACTERIZATION_HUMAN_REVIEW_RUNTIME)
	@$(CHARACTERIZATION_POLICY_AUTHORITY)
	@$(CHARACTERIZATION_GOVERNANCE_INTAKE)
	@$(CHARACTERIZATION_GOVERNANCE_HARDENING)
	@$(CHARACTERIZATION_CLI_PRODUCT_SURFACE)
	@$(CHARACTERIZATION_ADAPTIVE_WORKFLOW)
	@$(CHARACTERIZATION_ADAPTIVE_WORKFLOW_HARDENING)
	@$(CHARACTERIZATION_PROVIDER_GOVERNANCE)

check: check-layout check-docs build smoke

clean:
	rm -rf "$(BUILD_DIR)" "$(RUST_TARGET_DIR)" engine/target cmd/yai/target
