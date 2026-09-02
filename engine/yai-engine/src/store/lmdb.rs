//! YAI - LMDB record store
//!
//! Purpose:
//!   Provide durable indexed record lookup for normalized record envelopes.
//!
//! Ownership:
//!   LMDB environment opening, record writes, summary counts and record index
//!   reads.
//!
//! Boundary:
//!   Does not own journal replay/audit truth, hot-state freshness, graph truth
//!   or analytical facts.
//!
//! Status:
//!   active

use crate::admission::{
    build_policy_review_request, evaluate_filesystem_admission, resolve_canonical_evidence,
    resolve_policy_review_decision, reviewer_is_eligible, AuthorityTemporalContext,
};
use crate::case_policy::{
    build_case_policy_binding, materialize_effective_policy, BindingValidity, EffectivePolicy,
    EffectivePolicyInput, NormativeReadiness, NormativeStatus, PolicyCatalogDrift,
    PolicyValidityPosture, CASE_POLICY_BINDING_SCHEMA, CASE_POLICY_BINDING_SCHEMA_V1,
    EFFECTIVE_POLICY_SCHEMA, EFFECTIVE_POLICY_SCHEMA_V1, EFFECTIVE_POLICY_SCHEMA_V2,
    POLICY_MATERIALIZER_VERSION, POLICY_MATERIALIZER_VERSION_V1, POLICY_MATERIALIZER_VERSION_V2,
};
use crate::compatibility::{
    decode_legacy_record, inspect_legacy_jsonl, LegacyDecodeOutcome, LegacyRecord,
};
use crate::context::SemanticContextArtifact;
use crate::effect::{
    build_workflow_deterministic_filesystem_operation,
    build_workflow_deterministic_process_operation, issue_policy_execution_grant,
    validate_execution_obligation_closure, validate_execution_obligation_preparation, Decision,
    LocalFilesystemBinding, LocalProcessBinding, Operation, OperationOrigin,
    LOCAL_FILESYSTEM_BINDING_SCHEMA, LOCAL_FILESYSTEM_BINDING_SCHEMA_V1,
};
use crate::governance::{
    build_lifecycle_event, compile_policy_source, lifecycle_from_events, scope_policy_compilation,
    PolicyArtifact, PolicyArtifactView, PolicyCompilation, PolicyIngestOutcome,
    PolicyLifecycleAction, PolicyLifecycleEvent, PolicyLifecycleEventInput, PolicyLifecycleOutcome,
    PolicyLifecycleState, PolicyLineage, PolicySourceArtifact, PolicyValidationStatus,
    PolicyValidityMode, POLICY_ARTIFACT_SCHEMA, POLICY_ARTIFACT_SCHEMA_V1,
    POLICY_ARTIFACT_SCHEMA_V2, POLICY_ARTIFACT_SCHEMA_V3, POLICY_ARTIFACT_SCHEMA_V4,
    POLICY_LIFECYCLE_EVENT_SCHEMA, POLICY_LIFECYCLE_EVENT_SCHEMA_V1,
    POLICY_LIFECYCLE_EVENT_SCHEMA_V2, POLICY_SOURCE_ARTIFACT_SCHEMA,
    POLICY_SOURCE_ARTIFACT_SCHEMA_V1, POLICY_SOURCE_ARTIFACT_SCHEMA_V2,
    POLICY_SOURCE_ARTIFACT_SCHEMA_V3,
};
use crate::handoff::{
    HandoffAcceptance, HandoffData, HandoffDecline, HandoffOffer, HandoffOutcome,
    HandoffReconciliation, HandoffResult,
};
use crate::journal::Journal;
use crate::memory::{
    OperationalMemoryBuild, OperationalMemoryEntry, OperationalMemoryManifest,
    OPERATIONAL_MEMORY_DERIVATION, OPERATIONAL_MEMORY_MANIFEST_SCHEMA, OPERATIONAL_MEMORY_SCHEMA,
};
use crate::record::Record;
use crate::resource_control::{
    filesystem_relation, rebuild_resource_control_state as replay_resource_control_state,
    ActiveResourceLease, FilesystemRelation, LocalProcessIdentity, ResourceControlAction,
    ResourceControlEvent, ResourceControlState, ResourceFence, ResourceFenceAuthority,
    ResourceIdentity, RESOURCE_CONTROL_EVENT_SCHEMA, RESOURCE_CONTROL_STATE_SCHEMA,
};
use crate::security::{
    AuthenticatedPrincipal, SecurityContext, SecurityEvent, SecurityEventAction, SecurityPrincipal,
    Tenant, TenantMembershipKind, SECURITY_EVENT_SCHEMA, SECURITY_PRINCIPAL_SCHEMA, TENANT_SCHEMA,
};
use crate::transition::{
    replay_case, AuthorityInvalidationReason, CaseCancellationState, CaseClosureState,
    CaseLifecycle, CaseState, ExecutionGrantInvalidation, GrantInvalidationDisposition,
    GrantLifecycle, PendingTransition, ReviewInvalidation, ReviewResolution, Transition,
    TransitionPayload, TransitionSource, CASE_STATE_SCHEMA, CASE_STATE_SCHEMA_V1,
    CASE_STATE_SCHEMA_V10, CASE_STATE_SCHEMA_V2, CASE_STATE_SCHEMA_V3, CASE_STATE_SCHEMA_V4,
    CASE_STATE_SCHEMA_V5, CASE_STATE_SCHEMA_V6, CASE_STATE_SCHEMA_V7, CASE_STATE_SCHEMA_V8,
    CASE_STATE_SCHEMA_V9, TRANSITION_SCHEMA, TRANSITION_SCHEMA_V1, TRANSITION_SCHEMA_V10,
    TRANSITION_SCHEMA_V2, TRANSITION_SCHEMA_V3, TRANSITION_SCHEMA_V4, TRANSITION_SCHEMA_V5,
    TRANSITION_SCHEMA_V6, TRANSITION_SCHEMA_V7, TRANSITION_SCHEMA_V8, TRANSITION_SCHEMA_V9,
};
use crate::workflow::{
    derive_effective_workflow_topology, evaluate_predicate, node_completion_predicate,
    preview_workflow_patch, resolve_workflow_with_definitions, CaseWorkflowBinding,
    DeterministicOperationTemplate, HumanInputKind, WorkflowAmendment, WorkflowCaseBinding,
    WorkflowConditionResolution, WorkflowDefinition, WorkflowDefinitionInput,
    WorkflowDeterministicProposalRecord, WorkflowExecutorBinding, WorkflowHumanInputRecord,
    WorkflowNodeExecution, WorkflowNodeKind, WorkflowNodePosture, WorkflowNodeSatisfaction,
    WorkflowPatchOperation, WorkflowPlanPatch, WorkflowPlanPatchInput, WorkflowPlanPatchOrigin,
    WorkflowPredicate, WorkflowResolution, WorkflowResourceBinding, MAX_WORKFLOW_AMENDMENTS,
    MAX_WORKFLOW_INPUT_BYTES, WORKFLOW_CONDITION_RESOLUTION_SCHEMA, WORKFLOW_DEFINITION_SCHEMA,
    WORKFLOW_DEFINITION_SCHEMA_V1, WORKFLOW_HUMAN_INPUT_SCHEMA, WORKFLOW_NODE_EXECUTION_SCHEMA,
    WORKFLOW_NODE_SATISFACTION_SCHEMA,
};
use lmdb::{
    Cursor, Database, DatabaseFlags, Environment, EnvironmentFlags, Error, RoTransaction,
    RwTransaction, Transaction, WriteFlags,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::{SystemTime, UNIX_EPOCH};

pub const DEFAULT_LMDB_MAP_SIZE: usize = 256 * 1024 * 1024;
pub const MINIMUM_LMDB_MAP_SIZE: usize = 16 * 1024 * 1024;
pub const SUPPORTED_POLICY_CATALOG_SOURCES: usize = 256;
pub const RECORD_SCHEMA: &str = "yai.record.v1";
pub const GRAPH_RELATION_SCHEMA: &str = "yai.graph_relation.v1";
pub const GRAPH_RELATION_STORE_NAME: &str = "lmdb_graph_relations_v0";
pub const CANONICAL_AUTHORITY_BACKEND: &str = "lmdb_transaction_authority_v1";
pub const LEGACY_COMPATIBILITY_SCHEMA: &str = "yai.legacy.compatibility.v1";
pub const SEMANTIC_CONTEXT_ARTIFACT_SCHEMA: &str = "yai.semantic_context_artifact.v1";
pub const CASE_RUNTIME_ADMISSION_SCHEMA: &str = "yai.case_runtime_admission.v1";
pub const RUNTIME_INSTANCE_SCHEMA: &str = "yai.runtime_instance.v2";
const RUNTIME_INSTANCE_SCHEMA_V1: &str = "yai.runtime_instance.v1";
pub const RUNTIME_WORK_ITEM_SCHEMA_V1: &str = "yai.runtime_work_item.v1";
pub const RUNTIME_WORK_ITEM_SCHEMA: &str = "yai.runtime_work_item.v2";
pub const RUNTIME_INSTANCE_ID: &str = "runtime-instance:local-default";
pub const MAX_RUNTIME_WORK_TASK_BYTES: usize = 64 * 1024;
pub const MAX_RUNTIME_WORK_REQUEST_ID_BYTES: usize = 256;
pub const MAX_RUNTIME_WORK_JOURNAL_PATH_BYTES: usize = 4096;
pub const AUTHORITY_TIME_FLOOR_KEY: &str = "meta:authority_time_floor_unix_ms";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordStoreStatusKind {
    Missing,
    NotInitialized,
    Ready,
    Unavailable,
}

impl RecordStoreStatusKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::NotInitialized => "not_initialized",
            Self::Ready => "ready",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordStoreStatus {
    pub path: PathBuf,
    pub backend: &'static str,
    pub status: RecordStoreStatusKind,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecordStoreSummary {
    pub records_total: usize,
    pub records_by_case: usize,
    pub records_by_kind: usize,
    pub records_by_subject: usize,
    pub records_by_receipt: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CanonicalAuthoritySummary {
    pub transitions_total: usize,
    pub cases_materialized: usize,
    pub legacy_compatibility_payloads: usize,
}

pub struct LmdbRecordStore {
    env: Arc<Environment>,
    records_by_id: Database,
    records_by_case: Database,
    records_by_kind: Database,
    records_by_subject: Database,
    records_by_receipt: Database,
    graph_relations_by_id: Database,
    graph_relations_by_case: Database,
    graph_relations_by_kind: Database,
    transitions_by_id: Database,
    case_transition_sequence: Database,
    case_state: Database,
    legacy_compatibility_payloads: Database,
    local_resource_bindings: Database,
    semantic_context_artifacts: Database,
    operational_memory_by_id: Database,
    operational_memory_case_index: Database,
    case_runtime_admission: Database,
    runtime_instances: Database,
    runtime_work_items: Database,
    runtime_work_idempotency: Database,
    policy_sources_by_id: Database,
    policy_artifacts_by_id: Database,
    policy_lifecycle_events_by_id: Database,
    policy_lifecycle_sequence: Database,
    policy_current_by_lineage: Database,
    effective_policy_by_case: Database,
    security_principals_by_id: Database,
    security_principal_by_binding: Database,
    tenants_by_id: Database,
    tenant_memberships: Database,
    security_events_by_id: Database,
    resource_control_states_by_id: Database,
    resource_control_events_by_id: Database,
    workflow_definitions: Database,
    schema_meta: Database,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityBootstrapOutcome {
    pub principal: SecurityPrincipal,
    pub tenant: Tenant,
    pub membership: TenantMembershipKind,
    pub events: Vec<SecurityEvent>,
    pub created: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrincipalTenantRelation {
    pub tenant: Tenant,
    pub membership: TenantMembershipKind,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CaseRuntimeAdmission {
    pub schema: String,
    pub case_id: String,
    pub run_id: String,
    pub owner_token: String,
    pub owner_pid: u32,
    pub acquired_at_unix_ms: u64,
    pub renewed_at_unix_ms: u64,
    pub expires_at_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseRuntimeAdmissionRequest {
    pub case_id: String,
    pub run_id: String,
    pub owner_token: String,
    pub owner_pid: u32,
    pub now_unix_ms: u64,
    pub lease_duration_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CaseRuntimeAdmissionOutcome {
    Acquired,
    Renewed,
    Reclaimed,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeInstanceLifecycle {
    Starting,
    Running,
    Draining,
    Stopped,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RuntimeInstanceConfig {
    pub workers: usize,
    pub max_active_per_tenant: usize,
    pub max_queued_per_tenant: usize,
    pub max_queued_total: usize,
}

impl RuntimeInstanceConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.workers == 0
            || self.workers > 64
            || self.max_active_per_tenant == 0
            || self.max_active_per_tenant > self.workers
            || self.max_queued_per_tenant == 0
            || self.max_queued_total == 0
            || self.max_queued_per_tenant > self.max_queued_total
        {
            return Err("invalid_runtime_instance_config".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RuntimeInstance {
    pub schema: String,
    pub instance_id: String,
    pub integrity_digest: String,
    pub principal_id: String,
    pub owner_pid: u32,
    #[serde(default)]
    pub owner_process_identity: String,
    pub owner_token: String,
    pub lifecycle: RuntimeInstanceLifecycle,
    pub config: RuntimeInstanceConfig,
    pub acquired_at_unix_ms: u64,
    pub heartbeat_at_unix_ms: u64,
    pub lease_expires_at_unix_ms: u64,
    pub recovered_items: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_dispatched_tenant: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drain_requested_at_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drain_requested_by_principal: Option<String>,
    pub last_detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeInstanceAcquireRequest {
    pub owner_pid: u32,
    pub owner_token: String,
    pub now_unix_ms: u64,
    pub lease_duration_ms: u64,
    pub config: RuntimeInstanceConfig,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeInstanceAcquireOutcome {
    Acquired,
    Renewed,
    Reclaimed,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RuntimeCaseBudgets {
    pub max_invocations: usize,
    pub max_operations: usize,
    pub max_semantic_units: usize,
    pub max_resident_items: usize,
    pub max_estimated_input_units: usize,
    pub max_provider_retries: usize,
    pub max_runtime_ms: Option<u64>,
    pub stop_on_deny: bool,
    pub continue_after_malformed: bool,
}

impl RuntimeCaseBudgets {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_invocations == 0
            || self.max_operations == 0
            || self.max_semantic_units == 0
            || self.max_resident_items == 0
            || self.max_estimated_input_units == 0
            || self.max_runtime_ms == Some(0)
        {
            return Err("invalid_runtime_case_budgets".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeWorkState {
    Queued,
    Running,
    WaitingReview,
    WaitingEffect,
    Blocked,
    Completed,
    Denied,
    Cancelled,
    Failed,
}

impl RuntimeWorkState {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Denied | Self::Cancelled | Self::Failed
        )
    }

    pub fn is_queued_capacity(&self) -> bool {
        matches!(
            self,
            Self::Queued | Self::WaitingReview | Self::WaitingEffect | Self::Blocked
        )
    }

    /// Operational WorkItem FSM. Requeue is explicit recovery/parking
    /// convergence; terminal states have no outbound edges.
    pub fn permits_transition_to(&self, next: &Self) -> bool {
        match self {
            Self::Queued => matches!(next, Self::Running | Self::Cancelled),
            Self::Running => matches!(
                next,
                Self::Queued
                    | Self::WaitingReview
                    | Self::WaitingEffect
                    | Self::Blocked
                    | Self::Completed
                    | Self::Denied
                    | Self::Cancelled
                    | Self::Failed
            ),
            Self::WaitingReview | Self::WaitingEffect | Self::Blocked => {
                matches!(next, Self::Queued | Self::Cancelled)
            }
            Self::Completed | Self::Denied | Self::Cancelled | Self::Failed => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RuntimeWorkflowContext {
    pub workflow_binding_id: String,
    pub workflow_definition_id: String,
    pub workflow_node_id: String,
    pub workflow_execution_id: String,
    pub workflow_node_kind: String,
}

impl RuntimeWorkflowContext {
    fn validate(&self) -> Result<(), String> {
        if self.workflow_binding_id.is_empty()
            || self.workflow_definition_id.is_empty()
            || self.workflow_node_id.is_empty()
            || self.workflow_execution_id.is_empty()
            || !matches!(
                self.workflow_node_kind.as_str(),
                "model_work" | "deterministic_work"
            )
        {
            return Err("runtime_workflow_context_invalid".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RuntimeWorkItem {
    pub schema: String,
    pub work_id: String,
    pub integrity_digest: String,
    pub request_id: String,
    pub request_digest: String,
    pub principal_id: String,
    pub tenant_id: String,
    pub case_id: String,
    pub participant_id: String,
    pub attachment_id: String,
    pub journal_path: String,
    pub task: String,
    pub budgets: RuntimeCaseBudgets,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failpoint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow: Option<RuntimeWorkflowContext>,
    pub enqueue_sequence: u64,
    pub state: RuntimeWorkState,
    pub attempt_count: u32,
    pub runtime_instance_id: Option<String>,
    pub runtime_owner_token: Option<String>,
    pub worker_id: Option<String>,
    pub last_stop_reason: String,
    pub enqueued_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl RuntimeWorkItem {
    pub fn validate_integrity(&self) -> Result<(), String> {
        if !matches!(
            self.schema.as_str(),
            RUNTIME_WORK_ITEM_SCHEMA | RUNTIME_WORK_ITEM_SCHEMA_V1
        ) || self.work_id.is_empty()
            || self.request_id.is_empty()
            || self.request_id.len() > MAX_RUNTIME_WORK_REQUEST_ID_BYTES
            || self.request_digest.is_empty()
            || self.principal_id.is_empty()
            || self.tenant_id.is_empty()
            || self.case_id.is_empty()
            || self.participant_id.is_empty()
            || self.attachment_id.is_empty()
            || self.journal_path.is_empty()
            || self.journal_path.len() > MAX_RUNTIME_WORK_JOURNAL_PATH_BYTES
            || self.task.is_empty()
            || self.task.len() > MAX_RUNTIME_WORK_TASK_BYTES
            || self
                .failpoint
                .as_ref()
                .is_some_and(|value| value.len() > 128)
            || self.enqueue_sequence == 0
        {
            return Err("invalid_runtime_work_item".to_string());
        }
        if self.schema == RUNTIME_WORK_ITEM_SCHEMA_V1 && self.workflow.is_some() {
            return Err("runtime_workflow_context_requires_work_item_v2".to_string());
        }
        if let Some(workflow) = &self.workflow {
            workflow.validate()?;
        }
        self.budgets.validate()?;
        if self.integrity_digest != runtime_work_integrity_digest(self)? {
            return Err("runtime_work_item_integrity_mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeWorkSubmission {
    pub request_id: String,
    pub tenant_id: String,
    pub case_id: String,
    pub participant_id: String,
    pub attachment_id: String,
    pub journal_path: String,
    pub task: String,
    pub budgets: RuntimeCaseBudgets,
    pub failpoint: Option<String>,
    pub now_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeWorkSubmissionOutcome {
    pub item: RuntimeWorkItem,
    pub created: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalCommit {
    pub transition: Transition,
    pub state: CaseState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseCancellationOutcome {
    pub changed: bool,
    pub commits: Vec<CanonicalCommit>,
    pub state: CaseState,
    pub invalidated_reviews: usize,
    pub abandoned_grants: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseClosureOutcome {
    pub changed: bool,
    pub commit: Option<CanonicalCommit>,
    pub state: CaseState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreparedCommitOutcome {
    Prepared(CanonicalCommit),
    GrantInvalidated(CanonicalCommit),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CasePolicyMutationOutcome {
    pub changed: bool,
    pub commit: Option<CanonicalCommit>,
    pub status: NormativeStatus,
    pub derived_cache_error: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LegacyCompatibilityImportReport {
    pub lines_total: usize,
    pub losslessly_promoted: usize,
    pub promoted_with_metadata: usize,
    pub preserved_opaque: usize,
    pub rejected_malformed: usize,
    pub repeated_record_ids: usize,
    pub payloads_written: usize,
    pub payloads_duplicate: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredRecordEnvelope {
    pub raw_json: String,
    pub schema: String,
    pub record_id: String,
    pub record_kind: String,
    pub case_ref: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecordListResult {
    pub records_total: usize,
    pub records: Vec<StoredRecordEnvelope>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphRelation {
    pub relation_id: String,
    pub case_ref: String,
    pub from_ref: String,
    pub to_ref: String,
    pub edge_kind: String,
    pub from_kind: String,
    pub to_kind: String,
    pub source_record_id: String,
    pub source_record_kind: String,
    pub confidence: String,
    pub created_at_unix_ms: u128,
    pub provenance: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GraphRelationListResult {
    pub relations_total: usize,
    pub relations: Vec<GraphRelation>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GraphMaterializeReport {
    pub relations_seen: usize,
    pub relations_written: usize,
    pub relations_duplicate: usize,
    pub relations_skipped: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeGraphNode {
    pub node_ref: String,
    pub node_kind: String,
    pub case_ref: String,
    pub source_record_ref: String,
    pub generation: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeGraphEdge {
    pub relation_id: String,
    pub edge_kind: String,
    pub from_ref: String,
    pub to_ref: String,
    pub case_ref: String,
    pub source_record_id: String,
    pub generation: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RuntimeGraphLoadResult {
    pub case_ref: String,
    pub nodes_total: usize,
    pub edges_total: usize,
    pub outgoing_index_entries: usize,
    pub incoming_index_entries: usize,
    pub generation: usize,
    pub dirty: bool,
    pub stale: bool,
    pub source: &'static str,
    pub durable_truth: &'static str,
    pub nodes: Vec<RuntimeGraphNode>,
    pub edges: Vec<RuntimeGraphEdge>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct JournalImportReport {
    pub records_seen: usize,
    pub records_written: usize,
    pub records_duplicate: usize,
    pub records_skipped: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReplayMetadata {
    pub replay_id: String,
    pub journal_identity: String,
    pub journal_path: String,
    pub record_schema: String,
    pub journal_schema: String,
    pub started_at: String,
    pub completed_at: String,
    pub lines_total: usize,
    pub lines_replayed: usize,
    pub records_written: usize,
    pub records_duplicate: usize,
    pub records_skipped: usize,
    pub invalid_entries: usize,
    pub unsupported_entries: usize,
    pub cursor_line: usize,
    pub status: String,
    pub compatibility: String,
}

struct SharedEnvironmentCacheEntry {
    environment: Weak<Environment>,
    map_size: usize,
}

fn lmdb_environment_cache() -> &'static Mutex<BTreeMap<PathBuf, SharedEnvironmentCacheEntry>> {
    static CACHE: OnceLock<Mutex<BTreeMap<PathBuf, SharedEnvironmentCacheEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn lmdb_store_open_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// LMDB requires a process to open a filesystem environment once and share
/// that handle across threads. The weak cache preserves normal test/store
/// lifetimes while preventing concurrent Runtime workers from opening the same
/// environment repeatedly.
fn shared_lmdb_environment(path: &Path, map_size: usize) -> Result<Arc<Environment>, String> {
    let key = fs::canonicalize(path)
        .map_err(|error| format!("failed to canonicalize {}: {error}", path.display()))?;
    let mut cache = lmdb_environment_cache()
        .lock()
        .map_err(|_| "LMDB environment cache lock poisoned".to_string())?;
    if let Some(entry) = cache.get(&key) {
        if let Some(environment) = entry.environment.upgrade() {
            if entry.map_size != map_size {
                return Err(format!(
                    "lmdb_environment_map_size_mismatch: existing={} requested={map_size}",
                    entry.map_size
                ));
            }
            return Ok(environment);
        }
    }
    let environment = Arc::new(
        Environment::new()
            .set_max_dbs(40)
            .set_map_size(map_size)
            .set_flags(EnvironmentFlags::NO_TLS)
            .open(&key)
            .map_err(|error| format!("failed to open LMDB env {}: {error}", key.display()))?,
    );
    cache.insert(
        key,
        SharedEnvironmentCacheEntry {
            environment: Arc::downgrade(&environment),
            map_size,
        },
    );
    Ok(environment)
}

impl LmdbRecordStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        Self::open_with_map_size(path, DEFAULT_LMDB_MAP_SIZE)
    }

    pub fn open_with_map_size(path: impl AsRef<Path>, map_size: usize) -> Result<Self, String> {
        if map_size < MINIMUM_LMDB_MAP_SIZE {
            return Err(format!(
                "lmdb_map_size_below_supported_minimum: minimum={MINIMUM_LMDB_MAP_SIZE} actual={map_size}"
            ));
        }
        let path = path.as_ref();
        fs::create_dir_all(path)
            .map_err(|error| format!("failed to create {}: {error}", path.display()))?;
        let _open_guard = lmdb_store_open_lock()
            .lock()
            .map_err(|_| "LMDB store open lock poisoned".to_string())?;
        let env = shared_lmdb_environment(path, map_size)?;
        let records_by_id = env
            .create_db(Some("records_by_id"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open records_by_id: {error}"))?;
        let records_by_case = env
            .create_db(Some("records_by_case"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open records_by_case: {error}"))?;
        let records_by_kind = env
            .create_db(Some("records_by_kind"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open records_by_kind: {error}"))?;
        let records_by_subject = env
            .create_db(Some("records_by_subject"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open records_by_subject: {error}"))?;
        let records_by_receipt = env
            .create_db(Some("records_by_receipt"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open records_by_receipt: {error}"))?;
        let graph_relations_by_id = env
            .create_db(Some("graph_relations_by_id"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open graph_relations_by_id: {error}"))?;
        let graph_relations_by_case = env
            .create_db(Some("graph_relations_by_case"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open graph_relations_by_case: {error}"))?;
        let graph_relations_by_kind = env
            .create_db(Some("graph_relations_by_kind"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open graph_relations_by_kind: {error}"))?;
        let transitions_by_id = env
            .create_db(Some("transitions_by_id"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open transitions_by_id: {error}"))?;
        let case_transition_sequence = env
            .create_db(Some("case_transition_sequence"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open case_transition_sequence: {error}"))?;
        let case_state = env
            .create_db(Some("case_state"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open case_state: {error}"))?;
        let legacy_compatibility_payloads = env
            .create_db(
                Some("legacy_compatibility_payloads"),
                DatabaseFlags::empty(),
            )
            .map_err(|error| format!("failed to open legacy_compatibility_payloads: {error}"))?;
        let local_resource_bindings = env
            .create_db(Some("local_resource_bindings"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open local_resource_bindings: {error}"))?;
        let semantic_context_artifacts = env
            .create_db(Some("semantic_context_artifacts"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open semantic_context_artifacts: {error}"))?;
        let operational_memory_by_id = env
            .create_db(Some("operational_memory_by_id"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open operational_memory_by_id: {error}"))?;
        let operational_memory_case_index = env
            .create_db(
                Some("operational_memory_case_index"),
                DatabaseFlags::empty(),
            )
            .map_err(|error| format!("failed to open operational_memory_case_index: {error}"))?;
        let case_runtime_admission = env
            .create_db(Some("case_runtime_admission"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open case_runtime_admission: {error}"))?;
        let runtime_instances = env
            .create_db(Some("runtime_instances"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open runtime_instances: {error}"))?;
        let runtime_work_items = env
            .create_db(Some("runtime_work_items"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open runtime_work_items: {error}"))?;
        let runtime_work_idempotency = env
            .create_db(Some("runtime_work_idempotency"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open runtime_work_idempotency: {error}"))?;
        let policy_sources_by_id = env
            .create_db(Some("policy_sources_by_id"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open policy_sources_by_id: {error}"))?;
        let policy_artifacts_by_id = env
            .create_db(Some("policy_artifacts_by_id"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open policy_artifacts_by_id: {error}"))?;
        let policy_lifecycle_events_by_id = env
            .create_db(
                Some("policy_lifecycle_events_by_id"),
                DatabaseFlags::empty(),
            )
            .map_err(|error| format!("failed to open policy_lifecycle_events_by_id: {error}"))?;
        let policy_lifecycle_sequence = env
            .create_db(Some("policy_lifecycle_sequence"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open policy_lifecycle_sequence: {error}"))?;
        let policy_current_by_lineage = env
            .create_db(Some("policy_current_by_lineage"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open policy_current_by_lineage: {error}"))?;
        let effective_policy_by_case = env
            .create_db(Some("effective_policy_by_case"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open effective_policy_by_case: {error}"))?;
        let security_principals_by_id = env
            .create_db(Some("security_principals_by_id"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open security_principals_by_id: {error}"))?;
        let security_principal_by_binding = env
            .create_db(
                Some("security_principal_by_binding"),
                DatabaseFlags::empty(),
            )
            .map_err(|error| format!("failed to open security_principal_by_binding: {error}"))?;
        let tenants_by_id = env
            .create_db(Some("tenants_by_id"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open tenants_by_id: {error}"))?;
        let tenant_memberships = env
            .create_db(Some("tenant_memberships"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open tenant_memberships: {error}"))?;
        let security_events_by_id = env
            .create_db(Some("security_events_by_id"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open security_events_by_id: {error}"))?;
        let resource_control_states_by_id = env
            .create_db(
                Some("resource_control_states_by_id"),
                DatabaseFlags::empty(),
            )
            .map_err(|error| format!("failed to open resource_control_states_by_id: {error}"))?;
        let resource_control_events_by_id = env
            .create_db(
                Some("resource_control_events_by_id"),
                DatabaseFlags::empty(),
            )
            .map_err(|error| format!("failed to open resource_control_events_by_id: {error}"))?;
        let workflow_definitions = env
            .create_db(Some("workflow_definitions"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open workflow_definitions: {error}"))?;
        let schema_meta = env
            .create_db(Some("schema_meta"), DatabaseFlags::empty())
            .map_err(|error| format!("failed to open schema_meta: {error}"))?;
        let store = Self {
            env,
            records_by_id,
            records_by_case,
            records_by_kind,
            records_by_subject,
            records_by_receipt,
            graph_relations_by_id,
            graph_relations_by_case,
            graph_relations_by_kind,
            transitions_by_id,
            case_transition_sequence,
            case_state,
            legacy_compatibility_payloads,
            local_resource_bindings,
            semantic_context_artifacts,
            operational_memory_by_id,
            operational_memory_case_index,
            case_runtime_admission,
            runtime_instances,
            runtime_work_items,
            runtime_work_idempotency,
            policy_sources_by_id,
            policy_artifacts_by_id,
            policy_lifecycle_events_by_id,
            policy_lifecycle_sequence,
            policy_current_by_lineage,
            effective_policy_by_case,
            security_principals_by_id,
            security_principal_by_binding,
            tenants_by_id,
            tenant_memberships,
            security_events_by_id,
            resource_control_states_by_id,
            resource_control_events_by_id,
            workflow_definitions,
            schema_meta,
        };
        store.ensure_schema()?;
        Ok(store)
    }

    pub fn status(path: impl AsRef<Path>) -> RecordStoreStatus {
        let path = path.as_ref().to_path_buf();
        let status = if !path.exists() {
            RecordStoreStatusKind::Missing
        } else if !path.is_dir() {
            RecordStoreStatusKind::Unavailable
        } else if !path.join("data.mdb").exists() {
            RecordStoreStatusKind::NotInitialized
        } else {
            match Self::schema_ready(&path) {
                Ok(true) => RecordStoreStatusKind::Ready,
                Ok(false) => RecordStoreStatusKind::NotInitialized,
                Err(()) => RecordStoreStatusKind::Unavailable,
            }
        };
        RecordStoreStatus {
            path,
            backend: "lmdb",
            status,
        }
    }

    /// Atomically enrolls the kernel-observed local Principal, creates one
    /// Tenant security domain, and binds the Principal as its owner. The
    /// operation is idempotent only for an exact repeat of the same bootstrap.
    pub fn bootstrap_local_security(
        &self,
        authenticated: &AuthenticatedPrincipal,
        tenant_id: &str,
        organization_ref: &str,
        now_unix_ms: u64,
    ) -> Result<SecurityBootstrapOutcome, String> {
        let proposed_principal = SecurityPrincipal::from_authenticated(authenticated, now_unix_ms)?;
        let proposed_tenant = Tenant::new(
            tenant_id,
            organization_ref,
            &proposed_principal.principal_id,
            now_unix_ms,
        )?;
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start security bootstrap: {error}"))?;

        let existing_principal = get_json_txn::<SecurityPrincipal, _>(
            &txn,
            self.security_principals_by_id,
            &proposed_principal.principal_id,
            "security_principal",
        )?;
        let principal_created = existing_principal.is_none();
        let principal = match existing_principal {
            Some(existing) => {
                existing.validate()?;
                if !existing.matches_authenticated(authenticated) {
                    return Err("security_principal_authentication_binding_mismatch".to_string());
                }
                existing
            }
            None => {
                if let Ok(existing_id) = txn.get(
                    self.security_principal_by_binding,
                    &authenticated.binding().binding_ref,
                ) {
                    let existing_id = std::str::from_utf8(existing_id)
                        .map_err(|error| format!("security_binding_index_not_utf8: {error}"))?;
                    if existing_id != proposed_principal.principal_id {
                        return Err("security_authentication_binding_ambiguous".to_string());
                    }
                }
                put_json_txn(
                    &mut txn,
                    self.security_principals_by_id,
                    &proposed_principal.principal_id,
                    &proposed_principal,
                    WriteFlags::NO_OVERWRITE,
                    "security principal",
                )?;
                txn.put(
                    self.security_principal_by_binding,
                    &authenticated.binding().binding_ref,
                    &proposed_principal.principal_id,
                    WriteFlags::NO_OVERWRITE,
                )
                .map_err(|error| format!("failed to index security principal binding: {error}"))?;
                proposed_principal.clone()
            }
        };

        let existing_tenant =
            get_json_txn::<Tenant, _>(&txn, self.tenants_by_id, tenant_id, "tenant")?;
        if let Some(existing) = &existing_tenant {
            existing.validate()?;
            if existing.organization_ref != organization_ref
                || existing.owner_principal_id != principal.principal_id
            {
                return Err("unsafe_duplicate_tenant_bootstrap".to_string());
            }
        }

        let membership_key = tenant_membership_key(tenant_id, &principal.principal_id);
        let existing_membership = get_json_txn::<TenantMembershipKind, _>(
            &txn,
            self.tenant_memberships,
            &membership_key,
            "tenant_membership",
        )?;
        if let Some(existing) = &existing_membership {
            if *existing != TenantMembershipKind::Owner {
                return Err("tenant_owner_membership_integrity_mismatch".to_string());
            }
        }

        let created = existing_tenant.is_none();
        let mut events = Vec::new();
        if principal_created {
            let principal_event = self.append_security_event_txn(
                &mut txn,
                SecurityEventAction::LocalPrincipalRegistered,
                &principal.principal_id,
                None,
                Some(&principal.principal_id),
                None,
                now_unix_ms,
                "kernel_authenticated_local_enrollment",
            )?;
            events.push(principal_event);
        }
        if created {
            put_json_txn(
                &mut txn,
                self.tenants_by_id,
                tenant_id,
                &proposed_tenant,
                WriteFlags::NO_OVERWRITE,
                "tenant",
            )?;
            put_json_txn(
                &mut txn,
                self.tenant_memberships,
                &membership_key,
                &TenantMembershipKind::Owner,
                WriteFlags::NO_OVERWRITE,
                "tenant membership",
            )?;
            let tenant_event = self.append_security_event_txn(
                &mut txn,
                SecurityEventAction::TenantCreated,
                &principal.principal_id,
                Some(tenant_id),
                Some(&principal.principal_id),
                Some(TenantMembershipKind::Owner),
                now_unix_ms,
                "local_security_bootstrap",
            )?;
            events.push(tenant_event);
        }
        txn.commit()
            .map_err(|error| format!("failed to commit security bootstrap: {error}"))?;
        Ok(SecurityBootstrapOutcome {
            principal,
            tenant: existing_tenant.unwrap_or(proposed_tenant),
            membership: TenantMembershipKind::Owner,
            events,
            created,
        })
    }

    pub fn resolve_security_context(
        &self,
        authenticated: &AuthenticatedPrincipal,
        tenant_id: &str,
    ) -> Result<SecurityContext, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start security context read: {error}"))?;
        self.resolve_security_context_txn(&txn, authenticated, tenant_id)
    }

    pub fn enrolled_principal(
        &self,
        authenticated: &AuthenticatedPrincipal,
    ) -> Result<SecurityPrincipal, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start Principal read: {error}"))?;
        self.authenticated_principal_txn(&txn, authenticated)
    }

    pub fn list_principal_tenants(
        &self,
        authenticated: &AuthenticatedPrincipal,
    ) -> Result<Vec<PrincipalTenantRelation>, String> {
        let principal = self.enrolled_principal(authenticated)?;
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start Tenant membership read: {error}"))?;
        let mut cursor = txn
            .open_ro_cursor(self.tenant_memberships)
            .map_err(|error| format!("failed to open Tenant membership cursor: {error}"))?;
        let suffix = format!("\0{}", principal.principal_id);
        let mut tenant_ids = Vec::new();
        for (key, value) in cursor.iter() {
            let key = std::str::from_utf8(key)
                .map_err(|error| format!("tenant_membership_key_not_utf8: {error}"))?;
            if let Some(tenant_id) = key.strip_suffix(&suffix) {
                let membership: TenantMembershipKind = serde_json::from_slice(value)
                    .map_err(|error| format!("tenant_membership_decode_failed: {error}"))?;
                tenant_ids.push((tenant_id.to_string(), membership));
            }
        }
        drop(cursor);
        tenant_ids.sort_by(|left, right| left.0.cmp(&right.0));
        tenant_ids
            .into_iter()
            .map(|(tenant_id, membership)| {
                let tenant =
                    get_json_txn::<Tenant, _>(&txn, self.tenants_by_id, &tenant_id, "tenant")?
                        .ok_or_else(|| "tenant_membership_dangling_tenant".to_string())?;
                tenant.validate()?;
                Ok(PrincipalTenantRelation { tenant, membership })
            })
            .collect()
    }

    pub fn get_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start Tenant read: {error}"))?;
        let tenant = get_json_txn::<Tenant, _>(&txn, self.tenants_by_id, tenant_id, "tenant")?;
        if let Some(value) = &tenant {
            value.validate()?;
        }
        Ok(tenant)
    }

    pub fn list_security_events(&self) -> Result<Vec<SecurityEvent>, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start security event read: {error}"))?;
        let mut cursor = txn
            .open_ro_cursor(self.security_events_by_id)
            .map_err(|error| format!("failed to open security event cursor: {error}"))?;
        let mut events = Vec::new();
        for (_, value) in cursor.iter() {
            let event: SecurityEvent = serde_json::from_slice(value)
                .map_err(|error| format!("security_event_decode_failed: {error}"))?;
            event.validate()?;
            events.push(event);
        }
        events.sort_by_key(|event| event.sequence);
        Ok(events)
    }

    pub fn add_tenant_member(
        &self,
        authenticated: &AuthenticatedPrincipal,
        tenant_id: &str,
        subject_principal_id: &str,
        now_unix_ms: u64,
    ) -> Result<SecurityEvent, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start Tenant membership write: {error}"))?;
        let context = self.resolve_security_context_txn(&txn, authenticated, tenant_id)?;
        context.require_owner()?;
        let subject = get_json_txn::<SecurityPrincipal, _>(
            &txn,
            self.security_principals_by_id,
            subject_principal_id,
            "security_principal",
        )?
        .ok_or_else(|| "subject_principal_not_enrolled".to_string())?;
        subject.validate()?;
        let key = tenant_membership_key(tenant_id, subject_principal_id);
        if get_json_txn::<TenantMembershipKind, _>(
            &txn,
            self.tenant_memberships,
            &key,
            "tenant_membership",
        )?
        .is_some()
        {
            return Err("tenant_membership_already_exists".to_string());
        }
        put_json_txn(
            &mut txn,
            self.tenant_memberships,
            &key,
            &TenantMembershipKind::Member,
            WriteFlags::NO_OVERWRITE,
            "tenant membership",
        )?;
        let event = self.append_security_event_txn(
            &mut txn,
            SecurityEventAction::TenantMemberAdded,
            context.principal_id(),
            Some(tenant_id),
            Some(subject_principal_id),
            Some(TenantMembershipKind::Member),
            now_unix_ms,
            "tenant_owner_membership_add",
        )?;
        txn.commit()
            .map_err(|error| format!("failed to commit Tenant membership write: {error}"))?;
        Ok(event)
    }

    fn authenticated_principal_txn<T: Transaction>(
        &self,
        txn: &T,
        authenticated: &AuthenticatedPrincipal,
    ) -> Result<SecurityPrincipal, String> {
        let principal_id = match txn.get(
            self.security_principal_by_binding,
            &authenticated.binding().binding_ref,
        ) {
            Ok(value) => std::str::from_utf8(value)
                .map_err(|error| format!("security_binding_index_not_utf8: {error}"))?
                .to_string(),
            Err(Error::NotFound) => return Err("local_principal_not_enrolled".to_string()),
            Err(error) => return Err(format!("failed to read security binding index: {error}")),
        };
        let principal = get_json_txn::<SecurityPrincipal, _>(
            txn,
            self.security_principals_by_id,
            &principal_id,
            "security_principal",
        )?
        .ok_or_else(|| "security_binding_index_dangling".to_string())?;
        principal.validate()?;
        if !principal.matches_authenticated(authenticated) {
            return Err("kernel_principal_binding_mismatch".to_string());
        }
        Ok(principal)
    }

    fn resolve_security_context_txn<T: Transaction>(
        &self,
        txn: &T,
        authenticated: &AuthenticatedPrincipal,
        tenant_id: &str,
    ) -> Result<SecurityContext, String> {
        let principal = self.authenticated_principal_txn(txn, authenticated)?;
        let tenant = get_json_txn::<Tenant, _>(txn, self.tenants_by_id, tenant_id, "tenant")?
            .ok_or_else(|| "tenant_not_visible".to_string())?;
        tenant.validate()?;
        let membership = get_json_txn::<TenantMembershipKind, _>(
            txn,
            self.tenant_memberships,
            &tenant_membership_key(tenant_id, &principal.principal_id),
            "tenant_membership",
        )?
        .ok_or_else(|| "tenant_not_visible".to_string())?;
        Ok(SecurityContext::resolved(
            principal.principal_id,
            tenant.tenant_id,
            membership,
            authenticated.binding().binding_ref.clone(),
        ))
    }

    fn resolve_case_owner_context_txn<T: Transaction>(
        &self,
        txn: &T,
        state: &CaseState,
        actor_ref: &str,
        authenticated: Option<&AuthenticatedPrincipal>,
    ) -> Result<Option<SecurityContext>, String> {
        match state.tenant_id.as_deref() {
            Some(tenant_id) => {
                let authenticated = authenticated
                    .ok_or_else(|| "authenticated_tenant_owner_required".to_string())?;
                let context = self.resolve_security_context_txn(txn, authenticated, tenant_id)?;
                context.require_owner()?;
                if actor_ref != context.principal_id() {
                    return Err("administrative_principal_provenance_mismatch".to_string());
                }
                Ok(Some(context))
            }
            None if authenticated.is_none() => Ok(None),
            None => Err("legacy_unscoped_case_cannot_accept_tenant_authority".to_string()),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn append_security_event_txn(
        &self,
        txn: &mut RwTransaction<'_>,
        action: SecurityEventAction,
        principal_id: &str,
        tenant_id: Option<&str>,
        subject_principal_id: Option<&str>,
        membership: Option<TenantMembershipKind>,
        committed_at_unix_ms: u64,
        reason: &str,
    ) -> Result<SecurityEvent, String> {
        let sequence = match txn.get(self.schema_meta, &"meta:security_event_sequence") {
            Ok(value) => std::str::from_utf8(value)
                .map_err(|error| format!("security_event_sequence_not_utf8: {error}"))?
                .parse::<u64>()
                .map_err(|error| format!("security_event_sequence_invalid: {error}"))?
                .checked_add(1)
                .ok_or_else(|| "security_event_sequence_overflow".to_string())?,
            Err(Error::NotFound) => 1,
            Err(error) => return Err(format!("failed to read security event sequence: {error}")),
        };
        let event = SecurityEvent {
            schema: SECURITY_EVENT_SCHEMA.to_string(),
            event_id: String::new(),
            sequence,
            action,
            principal_id: principal_id.to_string(),
            tenant_id: tenant_id.map(str::to_string),
            subject_principal_id: subject_principal_id.map(str::to_string),
            membership,
            committed_at_unix_ms,
            reason: reason.to_string(),
            integrity_digest: String::new(),
        }
        .seal()?;
        put_json_txn(
            txn,
            self.security_events_by_id,
            &event.event_id,
            &event,
            WriteFlags::NO_OVERWRITE,
            "security event",
        )?;
        txn.put(
            self.schema_meta,
            &"meta:security_event_sequence",
            &sequence.to_string(),
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to update security event sequence: {error}"))?;
        Ok(event)
    }

    pub fn append_record(&self, record: &Record, source_ref: &str) -> Result<(), String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start LMDB write transaction: {error}"))?;
        self.put_record(&mut txn, record, source_ref)?;
        txn.commit()
            .map_err(|error| format!("failed to commit LMDB record write: {error}"))
    }

    /// Stores a bounded, rebuildable semantic derivation for inspection and
    /// invocation lineage. This database is derived and is never consulted by
    /// canonical replay or CaseState reduction.
    pub fn put_semantic_context_artifact(
        &self,
        artifact: &SemanticContextArtifact,
    ) -> Result<(), String> {
        let key = semantic_context_artifact_key(artifact.id());
        let value = serde_json::to_string(artifact)
            .map_err(|error| format!("semantic_context_artifact_encode_failed: {error}"))?;
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start semantic context write: {error}"))?;
        txn.put(
            self.semantic_context_artifacts,
            &key,
            &value,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to store semantic context artifact: {error}"))?;
        txn.commit()
            .map_err(|error| format!("failed to commit semantic context artifact: {error}"))
    }

    pub fn get_semantic_context_artifact(
        &self,
        artifact_id: &str,
    ) -> Result<Option<SemanticContextArtifact>, String> {
        let key = semantic_context_artifact_key(artifact_id);
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start semantic context read: {error}"))?;
        match txn.get(self.semantic_context_artifacts, &key) {
            Ok(value) => serde_json::from_slice(value)
                .map(Some)
                .map_err(|error| format!("semantic_context_artifact_decode_failed: {error}")),
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!("failed to read semantic context artifact: {error}")),
        }
    }

    /// Removes only derived Projection/ContextFrame/render metadata. Canonical
    /// Transition history and CaseState are held in different databases.
    pub fn clear_semantic_context_artifacts(&self) -> Result<(), String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start semantic context clear: {error}"))?;
        txn.clear_db(self.semantic_context_artifacts)
            .map_err(|error| format!("failed to clear semantic context artifacts: {error}"))?;
        txn.commit()
            .map_err(|error| format!("failed to commit semantic context clear: {error}"))
    }

    /// Atomically replaces one Case's disposable operational-memory
    /// materialization. This transaction is intentionally separate from the
    /// canonical Transition + CaseState commit.
    pub fn replace_case_operational_memory(
        &self,
        build: &OperationalMemoryBuild,
    ) -> Result<(), String> {
        validate_operational_memory_build(build)?;
        let manifest_key = operational_memory_case_key(&build.manifest.case_id);
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start operational memory replace: {error}"))?;

        if let Ok(value) = txn.get(self.operational_memory_case_index, &manifest_key) {
            let previous: OperationalMemoryManifest = serde_json::from_slice(value)
                .map_err(|error| format!("operational_memory_manifest_decode_failed: {error}"))?;
            for memory_id in previous.memory_ids {
                let key = operational_memory_id_key(&memory_id);
                match txn.del(self.operational_memory_by_id, &key, None) {
                    Ok(()) | Err(Error::NotFound) => {}
                    Err(error) => {
                        return Err(format!(
                            "failed to remove obsolete memory {memory_id}: {error}"
                        ))
                    }
                }
            }
        }

        for entry in &build.entries {
            let key = operational_memory_id_key(&entry.memory_id);
            let value = serde_json::to_vec(entry)
                .map_err(|error| format!("operational_memory_encode_failed: {error}"))?;
            txn.put(
                self.operational_memory_by_id,
                &key,
                &value,
                WriteFlags::empty(),
            )
            .map_err(|error| format!("failed to store memory {}: {error}", entry.memory_id))?;
        }
        let manifest_value = serde_json::to_vec(&build.manifest)
            .map_err(|error| format!("operational_memory_manifest_encode_failed: {error}"))?;
        txn.put(
            self.operational_memory_case_index,
            &manifest_key,
            &manifest_value,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to store operational memory manifest: {error}"))?;
        txn.commit()
            .map_err(|error| format!("failed to commit operational memory replace: {error}"))
    }

    pub fn operational_memory_manifest(
        &self,
        case_id: &str,
    ) -> Result<Option<OperationalMemoryManifest>, String> {
        let key = operational_memory_case_key(case_id);
        let txn = self.env.begin_ro_txn().map_err(|error| {
            format!("failed to start operational memory manifest read: {error}")
        })?;
        match txn.get(self.operational_memory_case_index, &key) {
            Ok(value) => serde_json::from_slice(value)
                .map(Some)
                .map_err(|error| format!("operational_memory_manifest_decode_failed: {error}")),
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!(
                "failed to read operational memory manifest: {error}"
            )),
        }
    }

    pub fn get_operational_memory(
        &self,
        memory_id: &str,
    ) -> Result<Option<OperationalMemoryEntry>, String> {
        let key = operational_memory_id_key(memory_id);
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start operational memory read: {error}"))?;
        match txn.get(self.operational_memory_by_id, &key) {
            Ok(value) => serde_json::from_slice(value)
                .map(Some)
                .map_err(|error| format!("operational_memory_decode_failed: {error}")),
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!("failed to read operational memory: {error}")),
        }
    }

    pub fn list_operational_memory(
        &self,
        case_id: &str,
    ) -> Result<Vec<OperationalMemoryEntry>, String> {
        let Some(manifest) = self.operational_memory_manifest(case_id)? else {
            return Ok(Vec::new());
        };
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start operational memory list: {error}"))?;
        let mut entries = Vec::with_capacity(manifest.memory_ids.len());
        for memory_id in manifest.memory_ids {
            let key = operational_memory_id_key(&memory_id);
            let value = txn
                .get(self.operational_memory_by_id, &key)
                .map_err(|error| {
                    format!("operational_memory_manifest_dangling: {memory_id}: {error}")
                })?;
            let entry = serde_json::from_slice(value)
                .map_err(|error| format!("operational_memory_decode_failed: {error}"))?;
            entries.push(entry);
        }
        Ok(entries)
    }

    pub fn clear_case_operational_memory(&self, case_id: &str) -> Result<(), String> {
        let manifest_key = operational_memory_case_key(case_id);
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start operational memory clear: {error}"))?;
        if let Ok(value) = txn.get(self.operational_memory_case_index, &manifest_key) {
            let manifest: OperationalMemoryManifest = serde_json::from_slice(value)
                .map_err(|error| format!("operational_memory_manifest_decode_failed: {error}"))?;
            for memory_id in manifest.memory_ids {
                let key = operational_memory_id_key(&memory_id);
                match txn.del(self.operational_memory_by_id, &key, None) {
                    Ok(()) | Err(Error::NotFound) => {}
                    Err(error) => {
                        return Err(format!("failed to clear memory {memory_id}: {error}"))
                    }
                }
            }
        }
        match txn.del(self.operational_memory_case_index, &manifest_key, None) {
            Ok(()) | Err(Error::NotFound) => {}
            Err(error) => return Err(format!("failed to clear memory manifest: {error}")),
        }
        txn.commit()
            .map_err(|error| format!("failed to commit operational memory clear: {error}"))
    }

    pub fn clear_all_operational_memory(&self) -> Result<(), String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start operational memory drop: {error}"))?;
        txn.clear_db(self.operational_memory_by_id)
            .map_err(|error| format!("failed to clear operational memory entries: {error}"))?;
        txn.clear_db(self.operational_memory_case_index)
            .map_err(|error| format!("failed to clear operational memory manifests: {error}"))?;
        txn.commit()
            .map_err(|error| format!("failed to commit operational memory drop: {error}"))
    }

    /// Acquires the non-canonical, single-host admission for advancing one
    /// Case. The LMDB write transaction makes competing acquisition attempts
    /// mutually exclusive. `allow_reclaim` is supplied only after the runtime
    /// boundary has established that the recorded local process no longer
    /// exists; expiry remains the portable fallback.
    pub fn acquire_case_runtime_admission(
        &self,
        request: &CaseRuntimeAdmissionRequest,
        allow_reclaim: bool,
    ) -> Result<(CaseRuntimeAdmissionOutcome, CaseRuntimeAdmission), String> {
        validate_runtime_admission_request(request)?;
        let key = case_runtime_admission_key(&request.case_id);
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start Case runtime admission: {error}"))?;
        let existing = match txn.get(self.case_runtime_admission, &key) {
            Ok(value) => Some(decode_runtime_admission(value)?),
            Err(Error::NotFound) => None,
            Err(error) => return Err(format!("failed to read Case runtime admission: {error}")),
        };
        let outcome = match existing.as_ref() {
            Some(active)
                if active.run_id == request.run_id
                    && active.owner_token == request.owner_token
                    && active.owner_pid == request.owner_pid =>
            {
                CaseRuntimeAdmissionOutcome::Renewed
            }
            Some(active) if !allow_reclaim && active.expires_at_unix_ms > request.now_unix_ms => {
                return Err(format!(
                    "case_runtime_admission_active: run_id={} owner_pid={} expires_at_unix_ms={}",
                    active.run_id, active.owner_pid, active.expires_at_unix_ms
                ));
            }
            Some(_) => CaseRuntimeAdmissionOutcome::Reclaimed,
            None => CaseRuntimeAdmissionOutcome::Acquired,
        };
        let admission = CaseRuntimeAdmission {
            schema: CASE_RUNTIME_ADMISSION_SCHEMA.to_string(),
            case_id: request.case_id.clone(),
            run_id: request.run_id.clone(),
            owner_token: request.owner_token.clone(),
            owner_pid: request.owner_pid,
            acquired_at_unix_ms: existing
                .as_ref()
                .filter(|active| {
                    active.run_id == request.run_id
                        && active.owner_token == request.owner_token
                        && active.owner_pid == request.owner_pid
                })
                .map(|active| active.acquired_at_unix_ms)
                .unwrap_or(request.now_unix_ms),
            renewed_at_unix_ms: request.now_unix_ms,
            expires_at_unix_ms: request
                .now_unix_ms
                .saturating_add(request.lease_duration_ms),
        };
        let encoded = serde_json::to_vec(&admission)
            .map_err(|error| format!("case_runtime_admission_encode_failed: {error}"))?;
        txn.put(
            self.case_runtime_admission,
            &key,
            &encoded,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to write Case runtime admission: {error}"))?;
        txn.commit()
            .map_err(|error| format!("failed to commit Case runtime admission: {error}"))?;
        Ok((outcome, admission))
    }

    pub fn get_case_runtime_admission(
        &self,
        case_id: &str,
    ) -> Result<Option<CaseRuntimeAdmission>, String> {
        let key = case_runtime_admission_key(case_id);
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start Case runtime admission read: {error}"))?;
        match txn.get(self.case_runtime_admission, &key) {
            Ok(value) => decode_runtime_admission(value).map(Some),
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!("failed to read Case runtime admission: {error}")),
        }
    }

    pub fn release_case_runtime_admission(
        &self,
        case_id: &str,
        run_id: &str,
        owner_token: &str,
    ) -> Result<bool, String> {
        let key = case_runtime_admission_key(case_id);
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start Case runtime admission release: {error}"))?;
        let current = match txn.get(self.case_runtime_admission, &key) {
            Ok(value) => decode_runtime_admission(value)?,
            Err(Error::NotFound) => return Ok(false),
            Err(error) => return Err(format!("failed to read Case runtime admission: {error}")),
        };
        if current.run_id != run_id || current.owner_token != owner_token {
            return Err("case_runtime_admission_release_owner_mismatch".to_string());
        }
        txn.del(self.case_runtime_admission, &key, None)
            .map_err(|error| format!("failed to delete Case runtime admission: {error}"))?;
        txn.commit()
            .map_err(|error| format!("failed to commit Case runtime admission release: {error}"))?;
        Ok(true)
    }

    /// Acquires the single local operational scheduler lease. This durable
    /// record never enters Case Transition history.
    pub fn acquire_runtime_instance(
        &self,
        authenticated: &AuthenticatedPrincipal,
        request: &RuntimeInstanceAcquireRequest,
        allow_dead_owner_reclaim: bool,
    ) -> Result<(RuntimeInstanceAcquireOutcome, RuntimeInstance), String> {
        validate_runtime_instance_request(request)?;
        if request.owner_pid != std::process::id() {
            return Err("runtime_instance_owner_pid_not_current_process".to_string());
        }
        let owner_process_identity = runtime_process_identity(request.owner_pid)?;
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start RuntimeInstance acquisition: {error}"))?;
        let principal = self.authenticated_principal_txn(&txn, authenticated)?;
        let existing = get_json_txn::<RuntimeInstance, _>(
            &txn,
            self.runtime_instances,
            RUNTIME_INSTANCE_ID,
            "runtime_instance",
        )?;
        if let Some(current) = &existing {
            validate_runtime_instance(current)?;
        }
        let outcome = match &existing {
            Some(current)
                if current.owner_token == request.owner_token
                    && current.owner_pid == request.owner_pid
                    && current.owner_process_identity == owner_process_identity
                    && current.principal_id == principal.principal_id =>
            {
                RuntimeInstanceAcquireOutcome::Renewed
            }
            Some(current) if current.principal_id != principal.principal_id => {
                return Err("runtime_instance_principal_mismatch".to_string());
            }
            Some(current) if matches!(current.lifecycle, RuntimeInstanceLifecycle::Stopped) => {
                RuntimeInstanceAcquireOutcome::Reclaimed
            }
            Some(current)
                if allow_dead_owner_reclaim && !runtime_owner_process_is_live(current) =>
            {
                RuntimeInstanceAcquireOutcome::Reclaimed
            }
            Some(current) => {
                return Err(format!(
                    "runtime_instance_active: principal_id={} owner_pid={} owner_process_identity={} lease_expires_at_unix_ms={}",
                    current.principal_id,
                    current.owner_pid,
                    if current.owner_process_identity.is_empty() {
                        "legacy_unqualified"
                    } else {
                        &current.owner_process_identity
                    },
                    current.lease_expires_at_unix_ms
                ));
            }
            None => RuntimeInstanceAcquireOutcome::Acquired,
        };
        let acquired_at = existing
            .as_ref()
            .filter(|_| matches!(outcome, RuntimeInstanceAcquireOutcome::Renewed))
            .map(|current| current.acquired_at_unix_ms)
            .unwrap_or(request.now_unix_ms);
        let mut instance = RuntimeInstance {
            schema: RUNTIME_INSTANCE_SCHEMA.to_string(),
            instance_id: RUNTIME_INSTANCE_ID.to_string(),
            integrity_digest: String::new(),
            principal_id: principal.principal_id,
            owner_pid: request.owner_pid,
            owner_process_identity,
            owner_token: request.owner_token.clone(),
            lifecycle: RuntimeInstanceLifecycle::Starting,
            config: request.config.clone(),
            acquired_at_unix_ms: acquired_at,
            heartbeat_at_unix_ms: request.now_unix_ms,
            lease_expires_at_unix_ms: request
                .now_unix_ms
                .checked_add(request.lease_duration_ms)
                .ok_or_else(|| "runtime_instance_lease_overflow".to_string())?,
            recovered_items: 0,
            last_dispatched_tenant: existing
                .as_ref()
                .and_then(|current| current.last_dispatched_tenant.clone()),
            drain_requested_at_unix_ms: None,
            drain_requested_by_principal: None,
            last_detail: match outcome {
                RuntimeInstanceAcquireOutcome::Acquired => "instance_acquired",
                RuntimeInstanceAcquireOutcome::Renewed => "instance_renewed",
                RuntimeInstanceAcquireOutcome::Reclaimed => "stale_instance_reclaimed",
            }
            .to_string(),
        };
        instance.integrity_digest = runtime_instance_integrity_digest(&instance)?;
        put_json_txn(
            &mut txn,
            self.runtime_instances,
            RUNTIME_INSTANCE_ID,
            &instance,
            WriteFlags::empty(),
            "runtime instance",
        )?;
        txn.commit()
            .map_err(|error| format!("failed to commit RuntimeInstance acquisition: {error}"))?;
        Ok((outcome, instance))
    }

    pub fn activate_runtime_instance(
        &self,
        authenticated: &AuthenticatedPrincipal,
        owner_token: &str,
        now_unix_ms: u64,
        lease_duration_ms: u64,
        recovered_items: usize,
    ) -> Result<RuntimeInstance, String> {
        self.update_runtime_instance_owner(
            authenticated,
            owner_token,
            now_unix_ms,
            lease_duration_ms,
            Some(RuntimeInstanceLifecycle::Running),
            Some(recovered_items),
            "recovery_sweep_complete",
        )
    }

    pub fn heartbeat_runtime_instance(
        &self,
        authenticated: &AuthenticatedPrincipal,
        owner_token: &str,
        now_unix_ms: u64,
        lease_duration_ms: u64,
    ) -> Result<RuntimeInstance, String> {
        self.update_runtime_instance_owner(
            authenticated,
            owner_token,
            now_unix_ms,
            lease_duration_ms,
            None,
            None,
            "heartbeat",
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn update_runtime_instance_owner(
        &self,
        authenticated: &AuthenticatedPrincipal,
        owner_token: &str,
        now_unix_ms: u64,
        lease_duration_ms: u64,
        lifecycle: Option<RuntimeInstanceLifecycle>,
        recovered_items: Option<usize>,
        detail: &str,
    ) -> Result<RuntimeInstance, String> {
        if owner_token.is_empty() || lease_duration_ms == 0 {
            return Err("invalid_runtime_instance_owner_update".to_string());
        }
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start RuntimeInstance update: {error}"))?;
        let principal = self.authenticated_principal_txn(&txn, authenticated)?;
        let mut instance = get_json_txn::<RuntimeInstance, _>(
            &txn,
            self.runtime_instances,
            RUNTIME_INSTANCE_ID,
            "runtime_instance",
        )?
        .ok_or_else(|| "runtime_instance_missing".to_string())?;
        validate_runtime_instance(&instance)?;
        if instance.owner_token != owner_token || instance.principal_id != principal.principal_id {
            return Err("runtime_instance_owner_mismatch".to_string());
        }
        if !runtime_instance_owned_by_current_process(&instance)? {
            return Err("runtime_instance_owner_process_mismatch".to_string());
        }
        if let Some(lifecycle) = lifecycle {
            instance.lifecycle = lifecycle;
        }
        if let Some(recovered_items) = recovered_items {
            instance.recovered_items = recovered_items;
        }
        instance.heartbeat_at_unix_ms = now_unix_ms;
        instance.lease_expires_at_unix_ms = now_unix_ms
            .checked_add(lease_duration_ms)
            .ok_or_else(|| "runtime_instance_lease_overflow".to_string())?;
        instance.last_detail = detail.to_string();
        instance.integrity_digest = runtime_instance_integrity_digest(&instance)?;
        put_json_txn(
            &mut txn,
            self.runtime_instances,
            RUNTIME_INSTANCE_ID,
            &instance,
            WriteFlags::empty(),
            "runtime instance",
        )?;
        txn.commit()
            .map_err(|error| format!("failed to commit RuntimeInstance update: {error}"))?;
        Ok(instance)
    }

    pub fn request_runtime_instance_drain(
        &self,
        authenticated: &AuthenticatedPrincipal,
        now_unix_ms: u64,
    ) -> Result<RuntimeInstance, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start RuntimeInstance drain: {error}"))?;
        let principal = self.authenticated_principal_txn(&txn, authenticated)?;
        let mut instance = get_json_txn::<RuntimeInstance, _>(
            &txn,
            self.runtime_instances,
            RUNTIME_INSTANCE_ID,
            "runtime_instance",
        )?
        .ok_or_else(|| "runtime_instance_missing".to_string())?;
        validate_runtime_instance(&instance)?;
        if instance.principal_id != principal.principal_id {
            return Err("runtime_instance_not_visible".to_string());
        }
        if matches!(instance.lifecycle, RuntimeInstanceLifecycle::Stopped) {
            return Ok(instance);
        }
        instance.drain_requested_at_unix_ms = Some(now_unix_ms);
        instance.drain_requested_by_principal = Some(principal.principal_id);
        instance.last_detail = "operator_requested_drain".to_string();
        instance.integrity_digest = runtime_instance_integrity_digest(&instance)?;
        put_json_txn(
            &mut txn,
            self.runtime_instances,
            RUNTIME_INSTANCE_ID,
            &instance,
            WriteFlags::empty(),
            "runtime instance",
        )?;
        txn.commit()
            .map_err(|error| format!("failed to commit RuntimeInstance drain: {error}"))?;
        Ok(instance)
    }

    pub fn begin_runtime_instance_drain(
        &self,
        authenticated: &AuthenticatedPrincipal,
        owner_token: &str,
        now_unix_ms: u64,
        lease_duration_ms: u64,
    ) -> Result<RuntimeInstance, String> {
        self.update_runtime_instance_owner(
            authenticated,
            owner_token,
            now_unix_ms,
            lease_duration_ms,
            Some(RuntimeInstanceLifecycle::Draining),
            None,
            "owner_observed_drain_request",
        )
    }

    pub fn fail_runtime_instance_closed(
        &self,
        authenticated: &AuthenticatedPrincipal,
        owner_token: &str,
        now_unix_ms: u64,
        lease_duration_ms: u64,
        detail: &str,
    ) -> Result<RuntimeInstance, String> {
        self.update_runtime_instance_owner(
            authenticated,
            owner_token,
            now_unix_ms,
            lease_duration_ms,
            Some(RuntimeInstanceLifecycle::Draining),
            None,
            detail,
        )
    }

    pub fn stop_runtime_instance(
        &self,
        authenticated: &AuthenticatedPrincipal,
        owner_token: &str,
        now_unix_ms: u64,
    ) -> Result<RuntimeInstance, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start RuntimeInstance stop: {error}"))?;
        let principal = self.authenticated_principal_txn(&txn, authenticated)?;
        let mut instance = get_json_txn::<RuntimeInstance, _>(
            &txn,
            self.runtime_instances,
            RUNTIME_INSTANCE_ID,
            "runtime_instance",
        )?
        .ok_or_else(|| "runtime_instance_missing".to_string())?;
        if instance.owner_token != owner_token || instance.principal_id != principal.principal_id {
            return Err("runtime_instance_owner_mismatch".to_string());
        }
        if !runtime_instance_owned_by_current_process(&instance)? {
            return Err("runtime_instance_owner_process_mismatch".to_string());
        }
        instance.lifecycle = RuntimeInstanceLifecycle::Stopped;
        instance.heartbeat_at_unix_ms = now_unix_ms;
        instance.lease_expires_at_unix_ms = now_unix_ms;
        instance.last_detail = "workers_drained".to_string();
        instance.integrity_digest = runtime_instance_integrity_digest(&instance)?;
        put_json_txn(
            &mut txn,
            self.runtime_instances,
            RUNTIME_INSTANCE_ID,
            &instance,
            WriteFlags::empty(),
            "runtime instance",
        )?;
        txn.commit()
            .map_err(|error| format!("failed to commit RuntimeInstance stop: {error}"))?;
        Ok(instance)
    }

    pub fn get_runtime_instance_authorized(
        &self,
        authenticated: &AuthenticatedPrincipal,
    ) -> Result<Option<RuntimeInstance>, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start RuntimeInstance read: {error}"))?;
        let principal = self.authenticated_principal_txn(&txn, authenticated)?;
        let instance = get_json_txn::<RuntimeInstance, _>(
            &txn,
            self.runtime_instances,
            RUNTIME_INSTANCE_ID,
            "runtime_instance",
        )?;
        if let Some(value) = &instance {
            validate_runtime_instance(value)?;
            if value.principal_id != principal.principal_id {
                return Err("runtime_instance_not_visible".to_string());
            }
        }
        Ok(instance)
    }

    pub fn submit_runtime_work(
        &self,
        authenticated: &AuthenticatedPrincipal,
        submission: &RuntimeWorkSubmission,
    ) -> Result<RuntimeWorkSubmissionOutcome, String> {
        validate_runtime_work_submission(submission)?;
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start runtime work submission: {error}"))?;
        let principal = self.authenticated_principal_txn(&txn, authenticated)?;
        let instance = get_json_txn::<RuntimeInstance, _>(
            &txn,
            self.runtime_instances,
            RUNTIME_INSTANCE_ID,
            "runtime_instance",
        )?
        .ok_or_else(|| "runtime_instance_not_running".to_string())?;
        validate_runtime_instance(&instance)?;
        if !matches!(instance.lifecycle, RuntimeInstanceLifecycle::Running)
            || instance.lease_expires_at_unix_ms <= submission.now_unix_ms
        {
            return Err("runtime_instance_not_accepting_work".to_string());
        }
        if instance.principal_id != principal.principal_id {
            return Err("runtime_instance_principal_mismatch".to_string());
        }
        let context =
            self.resolve_security_context_txn(&txn, authenticated, &submission.tenant_id)?;
        context.require_owner()?;
        let state = self
            .get_case_state_txn(&txn, &submission.case_id)?
            .ok_or_else(|| "runtime_work_case_not_visible".to_string())?;
        if state.tenant_id.as_deref() != Some(submission.tenant_id.as_str()) {
            return Err("runtime_work_security_domain_mismatch".to_string());
        }
        if state.lifecycle != CaseLifecycle::Open || state.cancellation.is_some() {
            return Err("runtime_work_case_not_dispatchable".to_string());
        }
        if !state
            .participants
            .iter()
            .any(|participant| participant.participant_id == submission.participant_id)
            || !state
                .resources
                .iter()
                .any(|resource| resource.attachment_id == submission.attachment_id)
        {
            return Err("runtime_work_case_binding_mismatch".to_string());
        }
        let request_digest = runtime_submission_digest(&principal.principal_id, submission)?;
        let idempotency_key = runtime_work_idempotency_key(
            &principal.principal_id,
            &submission.tenant_id,
            &submission.request_id,
        );
        if let Ok(raw_work_id) = txn.get(self.runtime_work_idempotency, &idempotency_key) {
            let work_id = std::str::from_utf8(raw_work_id)
                .map_err(|error| format!("runtime_work_idempotency_not_utf8: {error}"))?;
            let existing = get_json_txn::<RuntimeWorkItem, _>(
                &txn,
                self.runtime_work_items,
                work_id,
                "runtime_work_item",
            )?
            .ok_or_else(|| "runtime_work_idempotency_dangling".to_string())?;
            existing.validate_integrity()?;
            if existing.request_digest != request_digest {
                return Err("runtime_work_idempotency_conflict".to_string());
            }
            return Ok(RuntimeWorkSubmissionOutcome {
                item: existing,
                created: false,
            });
        }
        let all = list_runtime_work_items_txn(&txn, self.runtime_work_items)?;
        let total_queued = all
            .iter()
            .filter(|item| item.state.is_queued_capacity())
            .count();
        if total_queued >= instance.config.max_queued_total {
            return Err("runtime_global_queue_capacity_exhausted".to_string());
        }
        let tenant_queued = all
            .iter()
            .filter(|item| {
                item.tenant_id == submission.tenant_id && item.state.is_queued_capacity()
            })
            .count();
        if tenant_queued >= instance.config.max_queued_per_tenant {
            return Err("runtime_tenant_queue_capacity_exhausted".to_string());
        }
        let sequence = next_runtime_work_sequence(&mut txn, self.schema_meta)?;
        let work_id = format!(
            "runtime-work:{}",
            crate::context::stable_digest(&format!(
                "{}\0{}\0{}",
                principal.principal_id, submission.tenant_id, submission.request_id
            ))
        );
        let mut item = RuntimeWorkItem {
            schema: RUNTIME_WORK_ITEM_SCHEMA.to_string(),
            work_id: work_id.clone(),
            integrity_digest: String::new(),
            request_id: submission.request_id.clone(),
            request_digest,
            principal_id: principal.principal_id,
            tenant_id: submission.tenant_id.clone(),
            case_id: submission.case_id.clone(),
            participant_id: submission.participant_id.clone(),
            attachment_id: submission.attachment_id.clone(),
            journal_path: submission.journal_path.clone(),
            task: submission.task.clone(),
            budgets: submission.budgets.clone(),
            failpoint: submission.failpoint.clone(),
            workflow: None,
            enqueue_sequence: sequence,
            state: RuntimeWorkState::Queued,
            attempt_count: 0,
            runtime_instance_id: None,
            runtime_owner_token: None,
            worker_id: None,
            last_stop_reason: "accepted".to_string(),
            enqueued_at_unix_ms: submission.now_unix_ms,
            updated_at_unix_ms: submission.now_unix_ms,
        };
        item.integrity_digest = runtime_work_integrity_digest(&item)?;
        item.validate_integrity()?;
        put_json_txn(
            &mut txn,
            self.runtime_work_items,
            &work_id,
            &item,
            WriteFlags::NO_OVERWRITE,
            "runtime work item",
        )?;
        txn.put(
            self.runtime_work_idempotency,
            &idempotency_key,
            &work_id,
            WriteFlags::NO_OVERWRITE,
        )
        .map_err(|error| format!("failed to index runtime work idempotency: {error}"))?;
        txn.commit()
            .map_err(|error| format!("failed to commit runtime work submission: {error}"))?;
        Ok(RuntimeWorkSubmissionOutcome {
            item,
            created: true,
        })
    }

    pub fn list_runtime_work_authorized(
        &self,
        authenticated: &AuthenticatedPrincipal,
    ) -> Result<Vec<RuntimeWorkItem>, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start runtime work read: {error}"))?;
        let principal = self.authenticated_principal_txn(&txn, authenticated)?;
        let mut visible = Vec::new();
        for item in list_runtime_work_items_txn(&txn, self.runtime_work_items)? {
            if item.principal_id != principal.principal_id {
                continue;
            }
            if self
                .resolve_security_context_txn(&txn, authenticated, &item.tenant_id)
                .is_ok()
            {
                visible.push(item);
            }
        }
        visible.sort_by_key(|item| item.enqueue_sequence);
        Ok(visible)
    }

    pub fn claim_runtime_work(
        &self,
        authenticated: &AuthenticatedPrincipal,
        owner_token: &str,
        work_id: &str,
        worker_id: &str,
        now_unix_ms: u64,
    ) -> Result<RuntimeWorkItem, String> {
        if worker_id.is_empty() {
            return Err("runtime_worker_id_missing".to_string());
        }
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start runtime work claim: {error}"))?;
        let principal = self.authenticated_principal_txn(&txn, authenticated)?;
        let mut instance = runtime_instance_owner_txn(
            &txn,
            self.runtime_instances,
            &principal.principal_id,
            owner_token,
            now_unix_ms,
        )?;
        if !matches!(instance.lifecycle, RuntimeInstanceLifecycle::Running) {
            return Err("runtime_instance_not_dispatching".to_string());
        }
        let mut item = get_json_txn::<RuntimeWorkItem, _>(
            &txn,
            self.runtime_work_items,
            work_id,
            "runtime_work_item",
        )?
        .ok_or_else(|| "runtime_work_item_missing".to_string())?;
        item.validate_integrity()?;
        if item.principal_id != principal.principal_id
            || !matches!(item.state, RuntimeWorkState::Queued)
        {
            return Err("runtime_work_item_not_claimable".to_string());
        }
        let context = self.resolve_security_context_txn(&txn, authenticated, &item.tenant_id)?;
        context.require_owner()?;
        let state = self
            .get_case_state_txn(&txn, &item.case_id)?
            .ok_or_else(|| "runtime_work_case_not_visible".to_string())?;
        if state.tenant_id.as_deref() != Some(item.tenant_id.as_str())
            || state.lifecycle != CaseLifecycle::Open
            || state.cancellation.is_some()
        {
            return Err("runtime_work_case_not_dispatchable".to_string());
        }
        let all = list_runtime_work_items_txn(&txn, self.runtime_work_items)?;
        if all.iter().any(|other| {
            other.work_id != item.work_id
                && other.case_id == item.case_id
                && (matches!(other.state, RuntimeWorkState::Running)
                    || (!other.state.is_terminal()
                        && other.enqueue_sequence < item.enqueue_sequence))
        }) {
            return Err("runtime_case_already_active".to_string());
        }
        let tenant_active = all
            .iter()
            .filter(|other| {
                matches!(other.state, RuntimeWorkState::Running)
                    && other.tenant_id == item.tenant_id
            })
            .count();
        if tenant_active >= instance.config.max_active_per_tenant {
            return Err("runtime_tenant_active_capacity_exhausted".to_string());
        }
        if !item.state.permits_transition_to(&RuntimeWorkState::Running) {
            return Err(
                "runtime_work_invalid_state_transition: queued_to_running_required".to_string(),
            );
        }
        item.state = RuntimeWorkState::Running;
        item.attempt_count = item
            .attempt_count
            .checked_add(1)
            .ok_or_else(|| "runtime_work_attempt_overflow".to_string())?;
        item.runtime_instance_id = Some(instance.instance_id.clone());
        item.runtime_owner_token = Some(owner_token.to_string());
        item.worker_id = Some(worker_id.to_string());
        item.last_stop_reason = "dispatched".to_string();
        item.updated_at_unix_ms = now_unix_ms;
        item.integrity_digest = runtime_work_integrity_digest(&item)?;
        put_json_txn(
            &mut txn,
            self.runtime_work_items,
            work_id,
            &item,
            WriteFlags::empty(),
            "runtime work item",
        )?;
        instance.last_dispatched_tenant = Some(item.tenant_id.clone());
        instance.last_detail = format!("dispatched_work:{}", item.work_id);
        instance.integrity_digest = runtime_instance_integrity_digest(&instance)?;
        put_json_txn(
            &mut txn,
            self.runtime_instances,
            RUNTIME_INSTANCE_ID,
            &instance,
            WriteFlags::empty(),
            "runtime instance",
        )?;
        txn.commit()
            .map_err(|error| format!("failed to commit runtime work claim: {error}"))?;
        Ok(item)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_runtime_work_state(
        &self,
        authenticated: &AuthenticatedPrincipal,
        owner_token: &str,
        work_id: &str,
        expected_worker_id: Option<&str>,
        state: RuntimeWorkState,
        reason: &str,
        now_unix_ms: u64,
    ) -> Result<RuntimeWorkItem, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start runtime work update: {error}"))?;
        let principal = self.authenticated_principal_txn(&txn, authenticated)?;
        let _instance = runtime_instance_owner_txn(
            &txn,
            self.runtime_instances,
            &principal.principal_id,
            owner_token,
            now_unix_ms,
        )?;
        let mut item = get_json_txn::<RuntimeWorkItem, _>(
            &txn,
            self.runtime_work_items,
            work_id,
            "runtime_work_item",
        )?
        .ok_or_else(|| "runtime_work_item_missing".to_string())?;
        item.validate_integrity()?;
        if item.principal_id != principal.principal_id || item.state.is_terminal() {
            return Err("runtime_work_item_terminal_or_not_visible".to_string());
        }
        if let Some(expected_worker_id) = expected_worker_id {
            if !matches!(item.state, RuntimeWorkState::Running)
                || item.worker_id.as_deref() != Some(expected_worker_id)
                || item.runtime_owner_token.as_deref() != Some(owner_token)
            {
                return Err("runtime_work_worker_lease_mismatch".to_string());
            }
        }
        if !item.state.permits_transition_to(&state) {
            return Err(format!(
                "runtime_work_invalid_state_transition: {:?}_to_{state:?}",
                item.state
            ));
        }
        item.state = state;
        item.last_stop_reason = reason.to_string();
        item.updated_at_unix_ms = now_unix_ms;
        item.runtime_instance_id = Some(RUNTIME_INSTANCE_ID.to_string());
        item.runtime_owner_token = Some(owner_token.to_string());
        item.worker_id = None;
        item.integrity_digest = runtime_work_integrity_digest(&item)?;
        put_json_txn(
            &mut txn,
            self.runtime_work_items,
            work_id,
            &item,
            WriteFlags::empty(),
            "runtime work item",
        )?;
        txn.commit()
            .map_err(|error| format!("failed to commit runtime work update: {error}"))?;
        Ok(item)
    }

    /// Repairs noncanonical workflow bookkeeping from an exact canonical
    /// NodeSatisfied fact. This is deliberately narrower than the ordinary
    /// WorkItem FSM: the caller cannot choose a terminal outcome, and no
    /// workflow/Case transition is created here.
    pub fn complete_runtime_work_from_workflow_satisfaction(
        &self,
        authenticated: &AuthenticatedPrincipal,
        owner_token: &str,
        work_id: &str,
        now_unix_ms: u64,
    ) -> Result<RuntimeWorkItem, String> {
        let mut txn = self.env.begin_rw_txn().map_err(|error| {
            format!("failed to start workflow WorkItem completion repair: {error}")
        })?;
        let principal = self.authenticated_principal_txn(&txn, authenticated)?;
        let _instance = runtime_instance_owner_txn(
            &txn,
            self.runtime_instances,
            &principal.principal_id,
            owner_token,
            now_unix_ms,
        )?;
        let mut item = get_json_txn::<RuntimeWorkItem, _>(
            &txn,
            self.runtime_work_items,
            work_id,
            "runtime_work_item",
        )?
        .ok_or_else(|| "runtime_work_item_missing".to_string())?;
        item.validate_integrity()?;
        if item.principal_id != principal.principal_id || item.state.is_terminal() {
            return Err("runtime_work_item_terminal_or_not_visible".to_string());
        }
        let workflow = item
            .workflow
            .as_ref()
            .ok_or_else(|| "runtime_work_is_not_workflow_attributed".to_string())?;
        let case_state = self
            .get_case_state_txn(&txn, &item.case_id)?
            .ok_or_else(|| "runtime_work_case_missing".to_string())?;
        let exact_satisfaction = case_state
            .workflow_satisfactions
            .iter()
            .any(|satisfaction| {
                satisfaction.binding_id == workflow.workflow_binding_id
                    && satisfaction.workflow_definition_id == workflow.workflow_definition_id
                    && satisfaction.node_id == workflow.workflow_node_id
                    && satisfaction.execution_id.as_deref()
                        == Some(workflow.workflow_execution_id.as_str())
            });
        if !exact_satisfaction {
            return Err("workflow_work_completion_not_canonically_proven".to_string());
        }
        item.state = RuntimeWorkState::Completed;
        item.last_stop_reason = "canonical_workflow_satisfaction_recovered".to_string();
        item.updated_at_unix_ms = now_unix_ms;
        item.runtime_instance_id = Some(RUNTIME_INSTANCE_ID.to_string());
        item.runtime_owner_token = Some(owner_token.to_string());
        item.worker_id = None;
        item.integrity_digest = runtime_work_integrity_digest(&item)?;
        put_json_txn(
            &mut txn,
            self.runtime_work_items,
            work_id,
            &item,
            WriteFlags::empty(),
            "runtime work item",
        )?;
        txn.commit().map_err(|error| {
            format!("failed to commit workflow WorkItem completion repair: {error}")
        })?;
        Ok(item)
    }

    /// Atomically persists immutable source/compiler artifacts and registers
    /// the candidate in the independent append-only governance history. The
    /// Case ledger is neither required nor mutated.
    pub fn ingest_policy_compilation(
        &self,
        compilation: &PolicyCompilation,
        actor_ref: &str,
    ) -> Result<PolicyIngestOutcome, String> {
        self.ingest_policy_compilation_inner(compilation, actor_ref, None)
    }

    pub fn ingest_tenant_policy_compilation(
        &self,
        authenticated: &AuthenticatedPrincipal,
        tenant_id: &str,
        compilation: &PolicyCompilation,
    ) -> Result<PolicyIngestOutcome, String> {
        let context = self.resolve_security_context(authenticated, tenant_id)?;
        context.require_owner()?;
        if compilation.artifact.tenant_id.as_deref() != Some(tenant_id) {
            return Err("policy_artifact_authenticated_tenant_mismatch".to_string());
        }
        let tenant = self
            .get_tenant(tenant_id)?
            .ok_or_else(|| "tenant_not_visible".to_string())?;
        if compilation.artifact.organization_ref.as_deref()
            != Some(tenant.organization_ref.as_str())
        {
            return Err("policy_artifact_organization_projection_mismatch".to_string());
        }
        self.ingest_policy_compilation_inner(compilation, context.principal_id(), Some(tenant_id))
    }

    fn ingest_policy_compilation_inner(
        &self,
        compilation: &PolicyCompilation,
        actor_ref: &str,
        authenticated_tenant: Option<&str>,
    ) -> Result<PolicyIngestOutcome, String> {
        compilation.validate()?;
        let rebuilt = compile_policy_source(compilation.source.content_utf8.as_bytes())?;
        let rebuilt = match compilation.artifact.tenant_id.as_deref() {
            Some(tenant_id) => {
                if authenticated_tenant != Some(tenant_id) {
                    return Err("authenticated_tenant_policy_intake_required".to_string());
                }
                scope_policy_compilation(
                    &rebuilt,
                    tenant_id,
                    compilation
                        .artifact
                        .organization_ref
                        .as_deref()
                        .ok_or_else(|| "tenant_policy_organization_ref_missing".to_string())?,
                )?
            }
            None => {
                if authenticated_tenant.is_some() {
                    return Err("legacy_policy_artifact_cannot_enter_tenant_authority".to_string());
                }
                rebuilt
            }
        };
        if rebuilt != *compilation {
            return Err("policy_compilation_not_reproducible_from_source".to_string());
        }
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start policy intake transaction: {error}"))?;
        if let Some(existing) = self.policy_artifact_for_declared_version_txn(
            &txn,
            &compilation.artifact.lineage(),
            &compilation.artifact.artifact_version,
        )? {
            if existing.artifact_id != compilation.artifact.artifact_id {
                return Err(format!(
                    "policy_version_identity_collision: lineage={} version={} existing={} candidate={}",
                    compilation.artifact.lineage().identity(),
                    compilation.artifact.artifact_version,
                    existing.artifact_id,
                    compilation.artifact.artifact_id
                ));
            }
        }
        let source_key = policy_source_key(&compilation.source.source_id);
        let source_created = match txn.get(self.policy_sources_by_id, &source_key) {
            Ok(value) => {
                let stored = decode_policy_source(value)?;
                if stored != compilation.source {
                    return Err("immutable_policy_source_identity_collision".to_string());
                }
                false
            }
            Err(Error::NotFound) => {
                let encoded = serde_json::to_vec(&compilation.source)
                    .map_err(|error| format!("policy_source_encode_failed: {error}"))?;
                txn.put(
                    self.policy_sources_by_id,
                    &source_key,
                    &encoded,
                    WriteFlags::NO_OVERWRITE,
                )
                .map_err(|error| policy_store_write_error("immutable policy source", error))?;
                true
            }
            Err(error) => return Err(format!("failed to inspect policy source: {error}")),
        };

        let artifact_key = policy_artifact_key(&compilation.artifact.artifact_id);
        let artifact_created = match txn.get(self.policy_artifacts_by_id, &artifact_key) {
            Ok(value) => {
                let stored = decode_policy_artifact(value)?;
                if stored != compilation.artifact {
                    return Err("immutable_policy_artifact_identity_collision".to_string());
                }
                false
            }
            Err(Error::NotFound) => {
                let encoded = serde_json::to_vec(&compilation.artifact)
                    .map_err(|error| format!("policy_artifact_encode_failed: {error}"))?;
                txn.put(
                    self.policy_artifacts_by_id,
                    &artifact_key,
                    &encoded,
                    WriteFlags::NO_OVERWRITE,
                )
                .map_err(|error| policy_store_write_error("immutable policy artifact", error))?;
                true
            }
            Err(error) => return Err(format!("failed to inspect policy artifact: {error}")),
        };

        if artifact_created {
            self.append_policy_event_txn(
                &mut txn,
                &compilation.artifact.artifact_id,
                PolicyLifecycleAction::CandidateRegistered,
                None,
                PolicyLifecycleState::Candidate,
                None,
                actor_ref,
                "immutable policy candidate registered",
            )?;
        }
        let view = self.policy_artifact_view_txn(&txn, &compilation.artifact.artifact_id)?;
        txn.commit()
            .map_err(|error| format!("failed to commit policy intake: {error}"))?;
        Ok(PolicyIngestOutcome {
            source_created,
            artifact_created,
            view,
        })
    }

    pub fn get_policy_source(
        &self,
        source_id: &str,
    ) -> Result<Option<PolicySourceArtifact>, String> {
        let key = policy_source_key(source_id);
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start policy source read: {error}"))?;
        match txn.get(self.policy_sources_by_id, &key) {
            Ok(value) => decode_policy_source(value).map(Some),
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!("failed to read policy source: {error}")),
        }
    }

    pub fn get_policy_artifact(&self, artifact_id: &str) -> Result<Option<PolicyArtifact>, String> {
        let key = policy_artifact_key(artifact_id);
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start policy artifact read: {error}"))?;
        match txn.get(self.policy_artifacts_by_id, &key) {
            Ok(value) => decode_policy_artifact(value).map(Some),
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!("failed to read policy artifact: {error}")),
        }
    }

    pub fn policy_artifact_view(
        &self,
        artifact_id: &str,
    ) -> Result<Option<PolicyArtifactView>, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start policy artifact view read: {error}"))?;
        match self.policy_artifact_view_txn(&txn, artifact_id) {
            Ok(view) => Ok(Some(view)),
            Err(error) if error == "policy_artifact_not_found" => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn policy_artifact_view_authorized(
        &self,
        authenticated: &AuthenticatedPrincipal,
        artifact_id: &str,
    ) -> Result<PolicyArtifactView, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start secured policy artifact read: {error}"))?;
        let artifact = self
            .policy_artifact_txn(&txn, artifact_id)
            .map_err(|_| "policy_artifact_not_visible".to_string())?;
        let tenant_id = artifact.tenant_id.as_deref().ok_or_else(|| {
            "legacy_policy_artifact_requires_compatibility_inspection".to_string()
        })?;
        self.resolve_security_context_txn(&txn, authenticated, tenant_id)
            .map_err(|_| "policy_artifact_not_visible".to_string())?;
        self.policy_artifact_view_txn(&txn, artifact_id)
    }

    pub fn list_policy_artifact_views_authorized(
        &self,
        authenticated: &AuthenticatedPrincipal,
        tenant_id: &str,
    ) -> Result<Vec<PolicyArtifactView>, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start secured policy list: {error}"))?;
        self.resolve_security_context_txn(&txn, authenticated, tenant_id)?;
        let mut cursor = txn
            .open_ro_cursor(self.policy_artifacts_by_id)
            .map_err(|error| format!("failed to open policy artifact cursor: {error}"))?;
        let mut artifact_ids = Vec::new();
        for (_, value) in cursor.iter() {
            let artifact = decode_policy_artifact(value)?;
            if artifact.tenant_id.as_deref() == Some(tenant_id) {
                artifact_ids.push(artifact.artifact_id);
            }
        }
        drop(cursor);
        artifact_ids.sort();
        artifact_ids
            .iter()
            .map(|artifact_id| self.policy_artifact_view_txn(&txn, artifact_id))
            .collect()
    }

    pub fn get_policy_source_authorized(
        &self,
        authenticated: &AuthenticatedPrincipal,
        tenant_id: &str,
        source_id: &str,
    ) -> Result<PolicySourceArtifact, String> {
        let views = self.list_policy_artifact_views_authorized(authenticated, tenant_id)?;
        if !views
            .iter()
            .any(|view| view.artifact.source_id == source_id)
        {
            return Err("policy_source_not_visible".to_string());
        }
        self.get_policy_source(source_id)?
            .ok_or_else(|| "policy_source_not_visible".to_string())
    }

    pub fn list_policy_artifact_views(&self) -> Result<Vec<PolicyArtifactView>, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start policy artifact list: {error}"))?;
        let mut cursor = txn
            .open_ro_cursor(self.policy_artifacts_by_id)
            .map_err(|error| format!("failed to open policy artifact cursor: {error}"))?;
        let mut artifact_ids = Vec::new();
        for (_, value) in cursor.iter() {
            artifact_ids.push(decode_policy_artifact(value)?.artifact_id);
        }
        drop(cursor);
        artifact_ids.sort();
        artifact_ids
            .iter()
            .map(|artifact_id| self.policy_artifact_view_txn(&txn, artifact_id))
            .collect()
    }

    pub fn list_policy_lifecycle_events(
        &self,
        artifact_id: Option<&str>,
    ) -> Result<Vec<PolicyLifecycleEvent>, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start policy lifecycle read: {error}"))?;
        self.policy_lifecycle_events_txn(&txn, artifact_id)
    }

    /// Returns the one published artifact for an owner-scoped lineage. The
    /// index is only an accelerator: missing index state falls back to the
    /// immutable artifacts and lifecycle history.
    pub fn current_published_policy(
        &self,
        owner_ref: &str,
        policy_key: &str,
    ) -> Result<Option<PolicyArtifactView>, String> {
        let lineage = PolicyLineage::new(owner_ref, policy_key)?;
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start current policy read: {error}"))?;
        let lineage_key = policy_lineage_key(&lineage);
        match txn.get(self.policy_current_by_lineage, &lineage_key) {
            Ok(value) => {
                let artifact_id = std::str::from_utf8(value)
                    .map_err(|error| format!("policy_current_index_not_utf8: {error}"))?;
                let view = self.policy_artifact_view_txn(&txn, artifact_id)?;
                if view.artifact.lineage() != lineage
                    || view.lifecycle != PolicyLifecycleState::Published
                {
                    return Err("policy_current_index_integrity_mismatch".to_string());
                }
                Ok(Some(view))
            }
            Err(Error::NotFound) => {
                let found =
                    self.published_policy_artifacts_for_lineage_txn(&txn, &lineage, None)?;
                match found.as_slice() {
                    [] => Ok(None),
                    [artifact_id] => self.policy_artifact_view_txn(&txn, artifact_id).map(Some),
                    _ => Err("policy_lineage_has_multiple_current_artifacts".to_string()),
                }
            }
            Err(error) => Err(format!("failed to read current policy lineage: {error}")),
        }
    }

    /// Rebuilds the non-authoritative current-lineage accelerator from the
    /// immutable artifacts and lifecycle events.
    pub fn rebuild_policy_current_index(&self) -> Result<usize, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start policy index rebuild: {error}"))?;
        txn.clear_db(self.policy_current_by_lineage)
            .map_err(|error| policy_store_write_error("current policy lineage index", error))?;
        let mut cursor = txn
            .open_ro_cursor(self.policy_artifacts_by_id)
            .map_err(|error| format!("failed to open policy artifact cursor: {error}"))?;
        let mut artifacts = Vec::new();
        for (_, value) in cursor.iter() {
            artifacts.push(decode_policy_artifact(value)?);
        }
        drop(cursor);
        let mut current = BTreeMap::<PolicyLineage, String>::new();
        for artifact in artifacts {
            if self.policy_lifecycle_state_txn(&txn, &artifact.artifact_id)?
                == PolicyLifecycleState::Published
                && current
                    .insert(artifact.lineage(), artifact.artifact_id.clone())
                    .is_some()
            {
                return Err("policy_lineage_has_multiple_current_artifacts".to_string());
            }
        }
        for (lineage, artifact_id) in &current {
            txn.put(
                self.policy_current_by_lineage,
                &policy_lineage_key(lineage),
                artifact_id,
                WriteFlags::NO_OVERWRITE,
            )
            .map_err(|error| policy_store_write_error("current policy lineage index", error))?;
        }
        txn.commit()
            .map_err(|error| policy_store_write_error("policy index rebuild", error))?;
        Ok(current.len())
    }

    pub fn validate_policy_artifact(
        &self,
        artifact_id: &str,
        actor_ref: &str,
        reason: &str,
    ) -> Result<PolicyLifecycleOutcome, String> {
        self.validate_policy_artifact_inner(artifact_id, actor_ref, reason, None)
    }

    pub fn validate_tenant_policy_artifact(
        &self,
        authenticated: &AuthenticatedPrincipal,
        artifact_id: &str,
        reason: &str,
    ) -> Result<PolicyLifecycleOutcome, String> {
        self.validate_policy_artifact_inner(
            artifact_id,
            &authenticated.projected_principal_id(),
            reason,
            Some(authenticated),
        )
    }

    fn validate_policy_artifact_inner(
        &self,
        artifact_id: &str,
        actor_ref: &str,
        reason: &str,
        authenticated: Option<&AuthenticatedPrincipal>,
    ) -> Result<PolicyLifecycleOutcome, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start policy validation transaction: {error}"))?;
        let artifact = self.policy_artifact_txn(&txn, artifact_id)?;
        self.authorize_policy_admin_txn(&txn, &artifact, actor_ref, authenticated)?;
        if artifact.validation.status != PolicyValidationStatus::Qualified {
            return Err(format!(
                "policy_artifact_qualification_blocked: {}",
                artifact.validation.blockers.join(",")
            ));
        }
        let current = self.policy_lifecycle_state_txn(&txn, artifact_id)?;
        let changed = match current {
            PolicyLifecycleState::Candidate => {
                self.append_policy_event_txn(
                    &mut txn,
                    artifact_id,
                    PolicyLifecycleAction::Validated,
                    Some(PolicyLifecycleState::Candidate),
                    PolicyLifecycleState::Validated,
                    None,
                    actor_ref,
                    reason,
                )?;
                true
            }
            PolicyLifecycleState::Validated | PolicyLifecycleState::Published => false,
            PolicyLifecycleState::Superseded
            | PolicyLifecycleState::Retired
            | PolicyLifecycleState::Revoked => {
                return Err(format!("policy_artifact_not_validatable_from: {current:?}"))
            }
        };
        let view = self.policy_artifact_view_txn(&txn, artifact_id)?;
        txn.commit()
            .map_err(|error| format!("failed to commit policy validation: {error}"))?;
        Ok(PolicyLifecycleOutcome { changed, view })
    }

    pub fn publish_policy_artifact(
        &self,
        artifact_id: &str,
        actor_ref: &str,
        reason: &str,
    ) -> Result<PolicyLifecycleOutcome, String> {
        self.publish_policy_artifact_inner(artifact_id, actor_ref, reason, None)
    }

    pub fn publish_tenant_policy_artifact(
        &self,
        authenticated: &AuthenticatedPrincipal,
        artifact_id: &str,
        reason: &str,
    ) -> Result<PolicyLifecycleOutcome, String> {
        self.publish_policy_artifact_inner(
            artifact_id,
            &authenticated.projected_principal_id(),
            reason,
            Some(authenticated),
        )
    }

    fn publish_policy_artifact_inner(
        &self,
        artifact_id: &str,
        actor_ref: &str,
        reason: &str,
        authenticated: Option<&AuthenticatedPrincipal>,
    ) -> Result<PolicyLifecycleOutcome, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start policy publication transaction: {error}"))?;
        let artifact = self.policy_artifact_txn(&txn, artifact_id)?;
        self.authorize_policy_admin_txn(&txn, &artifact, actor_ref, authenticated)?;
        if artifact.validation.status != PolicyValidationStatus::Qualified {
            return Err("blocked_policy_artifact_cannot_publish".to_string());
        }
        let current = self.policy_lifecycle_state_txn(&txn, artifact_id)?;
        if current == PolicyLifecycleState::Published {
            let view = self.policy_artifact_view_txn(&txn, artifact_id)?;
            return Ok(PolicyLifecycleOutcome {
                changed: false,
                view,
            });
        }
        if current != PolicyLifecycleState::Validated {
            return Err(format!(
                "policy_artifact_must_be_validated_before_publish: current={current:?}"
            ));
        }

        let lineage = artifact.lineage();
        let published =
            self.published_policy_artifacts_for_lineage_txn(&txn, &lineage, Some(artifact_id))?;
        if published.len() > 1 {
            return Err("policy_lineage_has_multiple_current_artifacts".to_string());
        }
        for previous_id in published {
            self.append_policy_event_txn(
                &mut txn,
                &previous_id,
                PolicyLifecycleAction::Superseded,
                Some(PolicyLifecycleState::Published),
                PolicyLifecycleState::Superseded,
                Some(artifact_id),
                actor_ref,
                "new immutable policy version published",
            )?;
        }
        self.append_policy_event_txn(
            &mut txn,
            artifact_id,
            PolicyLifecycleAction::Published,
            Some(PolicyLifecycleState::Validated),
            PolicyLifecycleState::Published,
            None,
            actor_ref,
            reason,
        )?;
        let lineage_key = policy_lineage_key(&lineage);
        txn.put(
            self.policy_current_by_lineage,
            &lineage_key,
            &artifact_id,
            WriteFlags::empty(),
        )
        .map_err(|error| policy_store_write_error("current policy lineage index", error))?;
        let view = self.policy_artifact_view_txn(&txn, artifact_id)?;
        txn.commit()
            .map_err(|error| format!("failed to commit policy publication: {error}"))?;
        Ok(PolicyLifecycleOutcome {
            changed: true,
            view,
        })
    }

    pub fn retire_policy_artifact(
        &self,
        artifact_id: &str,
        actor_ref: &str,
        reason: &str,
    ) -> Result<PolicyLifecycleOutcome, String> {
        self.retire_policy_artifact_inner(artifact_id, actor_ref, reason, None)
    }

    pub fn retire_tenant_policy_artifact(
        &self,
        authenticated: &AuthenticatedPrincipal,
        artifact_id: &str,
        reason: &str,
    ) -> Result<PolicyLifecycleOutcome, String> {
        self.retire_policy_artifact_inner(
            artifact_id,
            &authenticated.projected_principal_id(),
            reason,
            Some(authenticated),
        )
    }

    fn retire_policy_artifact_inner(
        &self,
        artifact_id: &str,
        actor_ref: &str,
        reason: &str,
        authenticated: Option<&AuthenticatedPrincipal>,
    ) -> Result<PolicyLifecycleOutcome, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start policy retirement transaction: {error}"))?;
        let artifact = self.policy_artifact_txn(&txn, artifact_id)?;
        self.authorize_policy_admin_txn(&txn, &artifact, actor_ref, authenticated)?;
        let current = self.policy_lifecycle_state_txn(&txn, artifact_id)?;
        if current == PolicyLifecycleState::Retired {
            let view = self.policy_artifact_view_txn(&txn, artifact_id)?;
            return Ok(PolicyLifecycleOutcome {
                changed: false,
                view,
            });
        }
        self.append_policy_event_txn(
            &mut txn,
            artifact_id,
            PolicyLifecycleAction::Retired,
            Some(current.clone()),
            PolicyLifecycleState::Retired,
            None,
            actor_ref,
            reason,
        )?;
        if current == PolicyLifecycleState::Published {
            let lineage_key = policy_lineage_key(&artifact.lineage());
            match txn.get(self.policy_current_by_lineage, &lineage_key) {
                Ok(value) if value == artifact_id.as_bytes() => {
                    txn.del(self.policy_current_by_lineage, &lineage_key, None)
                        .map_err(|error| {
                            policy_store_write_error("current policy lineage index", error)
                        })?;
                }
                Ok(_) | Err(Error::NotFound) => {}
                Err(error) => {
                    return Err(format!("failed to inspect current policy lineage: {error}"))
                }
            }
        }
        let view = self.policy_artifact_view_txn(&txn, artifact_id)?;
        txn.commit()
            .map_err(|error| format!("failed to commit policy retirement: {error}"))?;
        Ok(PolicyLifecycleOutcome {
            changed: true,
            view,
        })
    }

    /// Appends terminal authority withdrawal without mutating immutable policy
    /// content. Revocation is stronger than catalog retirement/supersession.
    pub fn revoke_policy_artifact(
        &self,
        artifact_id: &str,
        actor_ref: &str,
        reason: &str,
    ) -> Result<PolicyLifecycleOutcome, String> {
        self.revoke_policy_artifact_inner(artifact_id, actor_ref, reason, None)
    }

    pub fn revoke_tenant_policy_artifact(
        &self,
        authenticated: &AuthenticatedPrincipal,
        artifact_id: &str,
        reason: &str,
    ) -> Result<PolicyLifecycleOutcome, String> {
        self.revoke_policy_artifact_inner(
            artifact_id,
            &authenticated.projected_principal_id(),
            reason,
            Some(authenticated),
        )
    }

    fn revoke_policy_artifact_inner(
        &self,
        artifact_id: &str,
        actor_ref: &str,
        reason: &str,
        authenticated: Option<&AuthenticatedPrincipal>,
    ) -> Result<PolicyLifecycleOutcome, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start policy revocation transaction: {error}"))?;
        let artifact = self.policy_artifact_txn(&txn, artifact_id)?;
        self.authorize_policy_admin_txn(&txn, &artifact, actor_ref, authenticated)?;
        let current = self.policy_lifecycle_state_txn(&txn, artifact_id)?;
        if current == PolicyLifecycleState::Revoked {
            let view = self.policy_artifact_view_txn(&txn, artifact_id)?;
            return Ok(PolicyLifecycleOutcome {
                changed: false,
                view,
            });
        }
        if !matches!(
            current,
            PolicyLifecycleState::Published
                | PolicyLifecycleState::Superseded
                | PolicyLifecycleState::Retired
        ) {
            return Err(format!("policy_artifact_not_revocable_from: {current:?}"));
        }
        let authority_time =
            self.advance_authority_time_txn(&mut txn, authority_wall_time_unix_ms())?;
        self.append_policy_event_at_txn(
            &mut txn,
            artifact_id,
            PolicyLifecycleAction::Revoked,
            Some(current.clone()),
            PolicyLifecycleState::Revoked,
            None,
            actor_ref,
            reason,
            authority_time,
        )?;
        if current == PolicyLifecycleState::Published {
            let lineage_key = policy_lineage_key(&artifact.lineage());
            match txn.get(self.policy_current_by_lineage, &lineage_key) {
                Ok(value) if value == artifact_id.as_bytes() => {
                    txn.del(self.policy_current_by_lineage, &lineage_key, None)
                        .map_err(|error| {
                            policy_store_write_error("current policy lineage index", error)
                        })?;
                }
                Ok(_) | Err(Error::NotFound) => {}
                Err(error) => {
                    return Err(format!("failed to inspect current policy lineage: {error}"))
                }
            }
        }
        let view = self.policy_artifact_view_txn(&txn, artifact_id)?;
        txn.commit()
            .map_err(|error| format!("failed to commit policy revocation: {error}"))?;
        Ok(PolicyLifecycleOutcome {
            changed: true,
            view,
        })
    }

    fn policy_artifact_txn<T: Transaction>(
        &self,
        txn: &T,
        artifact_id: &str,
    ) -> Result<PolicyArtifact, String> {
        let key = policy_artifact_key(artifact_id);
        match txn.get(self.policy_artifacts_by_id, &key) {
            Ok(value) => decode_policy_artifact(value),
            Err(Error::NotFound) => Err("policy_artifact_not_found".to_string()),
            Err(error) => Err(format!("failed to read policy artifact: {error}")),
        }
    }

    fn authorize_policy_admin_txn<T: Transaction>(
        &self,
        txn: &T,
        artifact: &PolicyArtifact,
        actor_ref: &str,
        authenticated: Option<&AuthenticatedPrincipal>,
    ) -> Result<(), String> {
        match artifact.tenant_id.as_deref() {
            Some(tenant_id) => {
                let authenticated = authenticated
                    .ok_or_else(|| "authenticated_tenant_owner_required".to_string())?;
                let context = self.resolve_security_context_txn(txn, authenticated, tenant_id)?;
                context.require_owner()?;
                if actor_ref != context.principal_id() {
                    return Err("policy_admin_principal_provenance_mismatch".to_string());
                }
                Ok(())
            }
            None if authenticated.is_none() => Ok(()),
            None => Err("legacy_policy_artifact_not_adopted_by_tenant".to_string()),
        }
    }

    fn policy_lifecycle_events_txn<T: Transaction>(
        &self,
        txn: &T,
        artifact_id: Option<&str>,
    ) -> Result<Vec<PolicyLifecycleEvent>, String> {
        let mut cursor = txn
            .open_ro_cursor(self.policy_lifecycle_sequence)
            .map_err(|error| format!("failed to open policy lifecycle cursor: {error}"))?;
        let mut event_ids = Vec::new();
        for (key, value) in cursor.iter() {
            let sequence_key = std::str::from_utf8(key)
                .map_err(|error| format!("policy_lifecycle_sequence_key_not_utf8: {error}"))?;
            let sequence = sequence_key
                .strip_prefix("policy-lifecycle:sequence:")
                .ok_or_else(|| "policy_lifecycle_sequence_key_invalid".to_string())?
                .parse::<u64>()
                .map_err(|error| format!("policy_lifecycle_sequence_key_invalid: {error}"))?;
            let event_id = std::str::from_utf8(value)
                .map_err(|error| format!("policy_lifecycle_event_id_not_utf8: {error}"))?;
            event_ids.push((sequence, event_id.to_string()));
        }
        drop(cursor);
        let mut events = Vec::new();
        for (sequence, event_id) in event_ids {
            let key = policy_event_key(&event_id);
            let value = txn
                .get(self.policy_lifecycle_events_by_id, &key)
                .map_err(|error| format!("policy_lifecycle_sequence_dangling: {error}"))?;
            let event = decode_policy_lifecycle_event(value)?;
            if event.sequence != sequence || event.event_id != event_id {
                return Err("policy_lifecycle_sequence_integrity_mismatch".to_string());
            }
            if artifact_id
                .map(|expected| event.artifact_id == expected)
                .unwrap_or(true)
            {
                events.push(event);
            }
        }
        events.sort_by_key(|event| event.sequence);
        Ok(events)
    }

    fn policy_lifecycle_state_txn<T: Transaction>(
        &self,
        txn: &T,
        artifact_id: &str,
    ) -> Result<PolicyLifecycleState, String> {
        lifecycle_from_events(&self.policy_lifecycle_events_txn(txn, Some(artifact_id))?)
    }

    fn policy_artifact_view_txn<T: Transaction>(
        &self,
        txn: &T,
        artifact_id: &str,
    ) -> Result<PolicyArtifactView, String> {
        let artifact = self.policy_artifact_txn(txn, artifact_id)?;
        let events = self.policy_lifecycle_events_txn(txn, Some(artifact_id))?;
        let lifecycle = lifecycle_from_events(&events)?;
        let superseded_by = events
            .iter()
            .rev()
            .find(|event| event.action == PolicyLifecycleAction::Superseded)
            .and_then(|event| event.related_artifact_id.clone());
        let runtime_consumable = lifecycle == PolicyLifecycleState::Published
            && artifact.validation.status == PolicyValidationStatus::Qualified;
        let view = PolicyArtifactView {
            artifact,
            lifecycle,
            runtime_consumable,
            superseded_by,
            lifecycle_events: events,
        };
        view.validate()?;
        if let Some(replacement_id) = &view.superseded_by {
            let replacement = self.policy_artifact_txn(txn, replacement_id)?;
            if replacement.lineage() != view.artifact.lineage() {
                return Err("policy_supersession_crosses_lineage".to_string());
            }
        }
        Ok(view)
    }

    fn published_policy_artifacts_for_lineage_txn<T: Transaction>(
        &self,
        txn: &T,
        lineage: &PolicyLineage,
        exclude: Option<&str>,
    ) -> Result<Vec<String>, String> {
        let mut cursor = txn
            .open_ro_cursor(self.policy_artifacts_by_id)
            .map_err(|error| format!("failed to open policy artifact cursor: {error}"))?;
        let mut ids = Vec::new();
        for (_, value) in cursor.iter() {
            let artifact = decode_policy_artifact(value)?;
            if artifact.lineage() == *lineage
                && exclude.map(|id| id != artifact.artifact_id).unwrap_or(true)
            {
                ids.push(artifact.artifact_id);
            }
        }
        drop(cursor);
        let mut published = Vec::new();
        for artifact_id in ids {
            if self.policy_lifecycle_state_txn(txn, &artifact_id)?
                == PolicyLifecycleState::Published
            {
                published.push(artifact_id);
            }
        }
        published.sort();
        Ok(published)
    }

    fn policy_artifact_for_declared_version_txn<T: Transaction>(
        &self,
        txn: &T,
        lineage: &PolicyLineage,
        artifact_version: &str,
    ) -> Result<Option<PolicyArtifact>, String> {
        let mut cursor = txn
            .open_ro_cursor(self.policy_artifacts_by_id)
            .map_err(|error| format!("failed to open policy artifact cursor: {error}"))?;
        let mut found = None;
        for (_, value) in cursor.iter() {
            let artifact = decode_policy_artifact(value)?;
            if artifact.lineage() == *lineage && artifact.artifact_version == artifact_version {
                if found.is_some() {
                    return Err("policy_version_identity_not_unique_in_store".to_string());
                }
                found = Some(artifact);
            }
        }
        Ok(found)
    }

    #[allow(clippy::too_many_arguments)]
    fn append_policy_event_txn(
        &self,
        txn: &mut RwTransaction<'_>,
        artifact_id: &str,
        action: PolicyLifecycleAction,
        prior_state: Option<PolicyLifecycleState>,
        next_state: PolicyLifecycleState,
        related_artifact_id: Option<&str>,
        actor_ref: &str,
        reason: &str,
    ) -> Result<PolicyLifecycleEvent, String> {
        self.append_policy_event_at_txn(
            txn,
            artifact_id,
            action,
            prior_state,
            next_state,
            related_artifact_id,
            actor_ref,
            reason,
            unix_time_ms() as u64,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn append_policy_event_at_txn(
        &self,
        txn: &mut RwTransaction<'_>,
        artifact_id: &str,
        action: PolicyLifecycleAction,
        prior_state: Option<PolicyLifecycleState>,
        next_state: PolicyLifecycleState,
        related_artifact_id: Option<&str>,
        actor_ref: &str,
        reason: &str,
        committed_at_unix_ms: u64,
    ) -> Result<PolicyLifecycleEvent, String> {
        let sequence = next_policy_lifecycle_sequence(txn, self.schema_meta)?;
        let artifact = self.policy_artifact_txn(txn, artifact_id)?;
        let tenant_id = artifact.tenant_id.as_deref();
        let principal_id = tenant_id.map(|_| actor_ref);
        let event = build_lifecycle_event(
            sequence,
            PolicyLifecycleEventInput {
                artifact_id,
                action,
                prior_state,
                next_state,
                related_artifact_id,
                tenant_id,
                principal_id,
                actor_ref,
                reason,
                committed_at_unix_ms,
            },
        )?;
        let event_key = policy_event_key(&event.event_id);
        let sequence_key = policy_sequence_key(sequence);
        let encoded = serde_json::to_vec(&event)
            .map_err(|error| format!("policy_lifecycle_event_encode_failed: {error}"))?;
        txn.put(
            self.policy_lifecycle_events_by_id,
            &event_key,
            &encoded,
            WriteFlags::NO_OVERWRITE,
        )
        .map_err(|error| format!("failed to append policy lifecycle event: {error}"))?;
        txn.put(
            self.policy_lifecycle_sequence,
            &sequence_key,
            &event.event_id,
            WriteFlags::NO_OVERWRITE,
        )
        .map_err(|error| format!("failed to append policy lifecycle sequence: {error}"))?;
        Ok(event)
    }

    /// Atomically binds one exact currently-published artifact to an existing
    /// Case. Catalog eligibility and the canonical Case transition are checked
    /// and committed in the same LMDB write transaction.
    pub fn bind_case_policy(
        &self,
        case_id: &str,
        artifact_id: &str,
        expected_generation: u64,
        actor_ref: &str,
        reason: &str,
    ) -> Result<CasePolicyMutationOutcome, String> {
        self.bind_case_policy_inner(
            case_id,
            artifact_id,
            expected_generation,
            actor_ref,
            reason,
            false,
            None,
        )
    }

    pub fn bind_tenant_case_policy(
        &self,
        authenticated: &AuthenticatedPrincipal,
        case_id: &str,
        artifact_id: &str,
        expected_generation: u64,
        reason: &str,
    ) -> Result<CasePolicyMutationOutcome, String> {
        self.bind_case_policy_inner(
            case_id,
            artifact_id,
            expected_generation,
            &authenticated.projected_principal_id(),
            reason,
            false,
            Some(authenticated),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn bind_case_policy_inner(
        &self,
        case_id: &str,
        artifact_id: &str,
        expected_generation: u64,
        actor_ref: &str,
        reason: &str,
        inject_derived_cache_failure: bool,
        authenticated: Option<&AuthenticatedPrincipal>,
    ) -> Result<CasePolicyMutationOutcome, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start Case policy bind transaction: {error}"))?;
        let state = self
            .get_case_state_txn(&txn, case_id)?
            .ok_or_else(|| format!("case_state_not_found: {case_id}"))?;
        require_open_case_for_policy(&state)?;
        let security_context =
            self.resolve_case_owner_context_txn(&txn, &state, actor_ref, authenticated)?;
        if state.generation != expected_generation {
            return Err(format!(
                "stale_case_generation: expected={expected_generation} actual={}",
                state.generation
            ));
        }
        let artifact = self.policy_artifact_txn(&txn, artifact_id)?;
        if artifact.tenant_id != state.tenant_id {
            return Err("cross_tenant_case_policy_binding_rejected".to_string());
        }
        if let Some(current) = state
            .policy_bindings
            .iter()
            .find(|binding| binding.lineage_id == artifact.lineage().identity())
        {
            if current.artifact_id == artifact_id {
                drop(txn);
                return self.case_policy_no_change_outcome(case_id);
            }
            return Err(format!(
                "case_policy_lineage_already_bound: lineage={} binding={}",
                current.lineage_id, current.binding_id
            ));
        }
        let publication = self.binding_eligible_publication_txn(&txn, &artifact)?;
        let binding = build_case_policy_binding(
            case_id,
            &artifact,
            &publication.event_id,
            publication.sequence,
            state.generation + 1,
            actor_ref,
            reason,
            None,
        )?;
        let mut pending = PendingTransition::new(
            format!("transition:policy-bind:{}", binding.binding_id),
            case_id,
            state.generation,
            TransitionSource {
                component: "yai.case_policy".to_string(),
                participant_id: None,
                principal_id: security_context
                    .as_ref()
                    .map(|context| context.principal_id().to_string()),
                source_ref: Some(binding.artifact_id.clone()),
            },
            TransitionPayload::CasePolicyBound {
                binding: binding.clone(),
            },
        );
        pending.causal_refs = vec![
            binding.artifact_id.clone(),
            binding.publication_event_id.clone(),
        ];
        let commit = self.commit_transition_txn_at(
            &mut txn,
            pending,
            false,
            None,
            security_context.as_ref(),
        )?;
        txn.commit()
            .map_err(|error| format!("failed to commit Case policy binding: {error}"))?;
        self.case_policy_changed_outcome(case_id, commit, inject_derived_cache_failure)
    }

    pub fn replace_case_policy(
        &self,
        case_id: &str,
        prior_binding_id: &str,
        artifact_id: &str,
        expected_generation: u64,
        actor_ref: &str,
        reason: &str,
    ) -> Result<CasePolicyMutationOutcome, String> {
        self.replace_case_policy_inner(
            case_id,
            prior_binding_id,
            artifact_id,
            expected_generation,
            actor_ref,
            reason,
            None,
        )
    }

    pub fn replace_tenant_case_policy(
        &self,
        authenticated: &AuthenticatedPrincipal,
        case_id: &str,
        prior_binding_id: &str,
        artifact_id: &str,
        expected_generation: u64,
        reason: &str,
    ) -> Result<CasePolicyMutationOutcome, String> {
        self.replace_case_policy_inner(
            case_id,
            prior_binding_id,
            artifact_id,
            expected_generation,
            &authenticated.projected_principal_id(),
            reason,
            Some(authenticated),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn replace_case_policy_inner(
        &self,
        case_id: &str,
        prior_binding_id: &str,
        artifact_id: &str,
        expected_generation: u64,
        actor_ref: &str,
        reason: &str,
        authenticated: Option<&AuthenticatedPrincipal>,
    ) -> Result<CasePolicyMutationOutcome, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start Case policy replace transaction: {error}"))?;
        let state = self
            .get_case_state_txn(&txn, case_id)?
            .ok_or_else(|| format!("case_state_not_found: {case_id}"))?;
        require_open_case_for_policy(&state)?;
        let security_context =
            self.resolve_case_owner_context_txn(&txn, &state, actor_ref, authenticated)?;
        if state.generation != expected_generation {
            return Err(format!(
                "stale_case_generation: expected={expected_generation} actual={}",
                state.generation
            ));
        }
        let prior = state
            .policy_bindings
            .iter()
            .find(|binding| binding.binding_id == prior_binding_id)
            .ok_or_else(|| "case_policy_replace_prior_binding_not_found".to_string())?;
        if prior.artifact_id == artifact_id {
            drop(txn);
            return self.case_policy_no_change_outcome(case_id);
        }
        let artifact = self.policy_artifact_txn(&txn, artifact_id)?;
        if artifact.tenant_id != state.tenant_id {
            return Err("cross_tenant_case_policy_binding_rejected".to_string());
        }
        if artifact.lineage().identity() != prior.lineage_id {
            return Err("case_policy_replace_lineage_mismatch".to_string());
        }
        let publication = self.binding_eligible_publication_txn(&txn, &artifact)?;
        let binding = build_case_policy_binding(
            case_id,
            &artifact,
            &publication.event_id,
            publication.sequence,
            state.generation + 1,
            actor_ref,
            reason,
            Some(prior_binding_id.to_string()),
        )?;
        let mut pending = PendingTransition::new(
            format!("transition:policy-replace:{}", binding.binding_id),
            case_id,
            state.generation,
            TransitionSource {
                component: "yai.case_policy".to_string(),
                participant_id: None,
                principal_id: security_context
                    .as_ref()
                    .map(|context| context.principal_id().to_string()),
                source_ref: Some(binding.artifact_id.clone()),
            },
            TransitionPayload::CasePolicyReplaced {
                prior_binding_id: prior_binding_id.to_string(),
                binding: binding.clone(),
            },
        );
        pending.causal_refs = vec![
            prior_binding_id.to_string(),
            binding.artifact_id.clone(),
            binding.publication_event_id.clone(),
        ];
        let commit = self.commit_transition_txn_at(
            &mut txn,
            pending,
            false,
            None,
            security_context.as_ref(),
        )?;
        txn.commit()
            .map_err(|error| format!("failed to commit Case policy replacement: {error}"))?;
        self.case_policy_changed_outcome(case_id, commit, false)
    }

    pub fn unbind_case_policy(
        &self,
        case_id: &str,
        binding_id: &str,
        expected_generation: u64,
        actor_ref: &str,
        reason: &str,
    ) -> Result<CasePolicyMutationOutcome, String> {
        self.unbind_case_policy_inner(
            case_id,
            binding_id,
            expected_generation,
            actor_ref,
            reason,
            None,
        )
    }

    pub fn unbind_tenant_case_policy(
        &self,
        authenticated: &AuthenticatedPrincipal,
        case_id: &str,
        binding_id: &str,
        expected_generation: u64,
        reason: &str,
    ) -> Result<CasePolicyMutationOutcome, String> {
        self.unbind_case_policy_inner(
            case_id,
            binding_id,
            expected_generation,
            &authenticated.projected_principal_id(),
            reason,
            Some(authenticated),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn unbind_case_policy_inner(
        &self,
        case_id: &str,
        binding_id: &str,
        expected_generation: u64,
        actor_ref: &str,
        reason: &str,
        authenticated: Option<&AuthenticatedPrincipal>,
    ) -> Result<CasePolicyMutationOutcome, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start Case policy unbind transaction: {error}"))?;
        let state = self
            .get_case_state_txn(&txn, case_id)?
            .ok_or_else(|| format!("case_state_not_found: {case_id}"))?;
        require_open_case_for_policy(&state)?;
        let security_context =
            self.resolve_case_owner_context_txn(&txn, &state, actor_ref, authenticated)?;
        if state.generation != expected_generation {
            return Err(format!(
                "stale_case_generation: expected={expected_generation} actual={}",
                state.generation
            ));
        }
        let binding = state
            .policy_bindings
            .iter()
            .find(|binding| binding.binding_id == binding_id)
            .ok_or_else(|| "case_policy_unbind_current_binding_not_found".to_string())?;
        let mut pending = PendingTransition::new(
            format!(
                "transition:policy-unbind:{case_id}:{}",
                state.generation + 1
            ),
            case_id,
            state.generation,
            TransitionSource {
                component: "yai.case_policy".to_string(),
                participant_id: None,
                principal_id: security_context
                    .as_ref()
                    .map(|context| context.principal_id().to_string()),
                source_ref: Some(binding.binding_id.clone()),
            },
            TransitionPayload::CasePolicyUnbound {
                binding_id: binding.binding_id.clone(),
                lineage_id: binding.lineage_id.clone(),
                actor_ref: actor_ref.to_string(),
                reason: reason.to_string(),
            },
        );
        pending.causal_refs = vec![binding.binding_id.clone()];
        let commit = self.commit_transition_txn_at(
            &mut txn,
            pending,
            false,
            None,
            security_context.as_ref(),
        )?;
        txn.commit()
            .map_err(|error| format!("failed to commit Case policy unbind: {error}"))?;
        self.case_policy_changed_outcome(case_id, commit, false)
    }

    /// Pure derivation from CaseState plus exact immutable PolicyArtifacts.
    pub fn case_policy_status(&self, case_id: &str) -> Result<NormativeStatus, String> {
        self.case_policy_status_at(case_id, authority_wall_time_unix_ms())
    }

    /// Pure deterministic status derivation used by temporal qualification.
    /// The supplied clock cannot move authority below the committed floor.
    pub fn case_policy_status_at(
        &self,
        case_id: &str,
        observed_wall_time_unix_ms: u64,
    ) -> Result<NormativeStatus, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start Case policy status read: {error}"))?;
        let floor = self.authority_time_floor_txn(&txn)?;
        let mut status = self.materialize_case_policy_at_txn(
            &txn,
            case_id,
            observed_wall_time_unix_ms.max(floor),
            floor,
        )?;
        status.observed_wall_time_unix_ms = observed_wall_time_unix_ms;
        Ok(status)
    }

    /// Rebuilds the optional derived cache. Canonical Case/policy history is
    /// read-only input and is not modified.
    pub fn rebuild_effective_policy(&self, case_id: &str) -> Result<NormativeStatus, String> {
        let status = self.case_policy_status(case_id)?;
        self.put_effective_policy_status(case_id, &status)?;
        Ok(status)
    }

    pub fn drop_effective_policy(&self, case_id: &str) -> Result<bool, String> {
        let key = effective_policy_case_key(case_id);
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start EffectivePolicy cache deletion: {error}"))?;
        let removed = match txn.del(self.effective_policy_by_case, &key, None) {
            Ok(()) => true,
            Err(Error::NotFound) => false,
            Err(error) => return Err(format!("failed to delete EffectivePolicy cache: {error}")),
        };
        txn.commit()
            .map_err(|error| format!("failed to commit EffectivePolicy cache deletion: {error}"))?;
        Ok(removed)
    }

    pub fn cached_effective_policy(
        &self,
        case_id: &str,
    ) -> Result<Option<NormativeStatus>, String> {
        let key = effective_policy_case_key(case_id);
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start EffectivePolicy cache read: {error}"))?;
        match txn.get(self.effective_policy_by_case, &key) {
            Ok(value) => serde_json::from_slice(value)
                .map(Some)
                .map_err(|error| format!("effective_policy_cache_decode_failed: {error}")),
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!("failed to read EffectivePolicy cache: {error}")),
        }
    }

    fn case_policy_no_change_outcome(
        &self,
        case_id: &str,
    ) -> Result<CasePolicyMutationOutcome, String> {
        Ok(CasePolicyMutationOutcome {
            changed: false,
            commit: None,
            status: self.case_policy_status(case_id)?,
            derived_cache_error: None,
        })
    }

    fn case_policy_changed_outcome(
        &self,
        case_id: &str,
        commit: CanonicalCommit,
        inject_derived_cache_failure: bool,
    ) -> Result<CasePolicyMutationOutcome, String> {
        let status = self.case_policy_status(case_id)?;
        let derived_cache_error = if inject_derived_cache_failure {
            Some("injected_effective_policy_cache_failure".to_string())
        } else {
            self.put_effective_policy_status(case_id, &status).err()
        };
        Ok(CasePolicyMutationOutcome {
            changed: true,
            commit: Some(commit),
            status,
            derived_cache_error,
        })
    }

    fn put_effective_policy_status(
        &self,
        case_id: &str,
        status: &NormativeStatus,
    ) -> Result<(), String> {
        let key = effective_policy_case_key(case_id);
        let value = serde_json::to_vec(status)
            .map_err(|error| format!("effective_policy_cache_encode_failed: {error}"))?;
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start EffectivePolicy cache write: {error}"))?;
        txn.put(
            self.effective_policy_by_case,
            &key,
            &value,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to write EffectivePolicy cache: {error}"))?;
        txn.commit()
            .map_err(|error| format!("failed to commit EffectivePolicy cache: {error}"))
    }

    fn binding_eligible_publication_txn<T: Transaction>(
        &self,
        txn: &T,
        artifact: &PolicyArtifact,
    ) -> Result<PolicyLifecycleEvent, String> {
        artifact.validate()?;
        let view = self.policy_artifact_view_txn(txn, &artifact.artifact_id)?;
        if view.lifecycle != PolicyLifecycleState::Published || !view.runtime_consumable {
            return Err(format!(
                "policy_artifact_not_eligible_for_new_case_binding: lifecycle={:?} runtime_consumable={}",
                view.lifecycle, view.runtime_consumable
            ));
        }
        let current = self.current_published_policy_txn(txn, &artifact.lineage())?;
        if current.as_deref() != Some(artifact.artifact_id.as_str()) {
            return Err("policy_artifact_not_current_published_for_lineage".to_string());
        }
        view.lifecycle_events
            .iter()
            .rev()
            .find(|event| event.action == PolicyLifecycleAction::Published)
            .cloned()
            .ok_or_else(|| "policy_publication_evidence_missing".to_string())
    }

    fn current_published_policy_txn<T: Transaction>(
        &self,
        txn: &T,
        lineage: &PolicyLineage,
    ) -> Result<Option<String>, String> {
        let key = policy_lineage_key(lineage);
        match txn.get(self.policy_current_by_lineage, &key) {
            Ok(value) => {
                let artifact_id = std::str::from_utf8(value)
                    .map_err(|error| format!("policy_current_index_not_utf8: {error}"))?;
                let view = self.policy_artifact_view_txn(txn, artifact_id)?;
                if view.artifact.lineage() != *lineage
                    || view.lifecycle != PolicyLifecycleState::Published
                {
                    return Err("policy_current_index_integrity_mismatch".to_string());
                }
                Ok(Some(artifact_id.to_string()))
            }
            Err(Error::NotFound) => {
                let found = self.published_policy_artifacts_for_lineage_txn(txn, lineage, None)?;
                match found.as_slice() {
                    [] => Ok(None),
                    [artifact] => Ok(Some(artifact.clone())),
                    _ => Err("policy_lineage_has_multiple_current_artifacts".to_string()),
                }
            }
            Err(error) => Err(format!("failed to read current policy lineage: {error}")),
        }
    }

    fn materialize_case_policy_txn<T: Transaction>(
        &self,
        txn: &T,
        case_id: &str,
    ) -> Result<NormativeStatus, String> {
        let floor = self.authority_time_floor_txn(txn)?;
        let observed_wall_time = authority_wall_time_unix_ms();
        let mut status = self.materialize_case_policy_at_txn(
            txn,
            case_id,
            observed_wall_time.max(floor),
            floor,
        )?;
        status.observed_wall_time_unix_ms = observed_wall_time;
        Ok(status)
    }

    fn materialize_case_policy_at_txn<T: Transaction>(
        &self,
        txn: &T,
        case_id: &str,
        authority_time_unix_ms: u64,
        persisted_floor_unix_ms: u64,
    ) -> Result<NormativeStatus, String> {
        let state = self
            .get_case_state_txn(txn, case_id)?
            .ok_or_else(|| format!("case_state_not_found: {case_id}"))?;
        if state.policy_bindings.is_empty() {
            let mut status = materialize_effective_policy(case_id, Vec::new());
            status.authority_time_unix_ms = authority_time_unix_ms;
            status.observed_wall_time_unix_ms = authority_time_unix_ms;
            status.persisted_authority_floor_unix_ms = persisted_floor_unix_ms;
            return Ok(status);
        }
        let mut inputs = Vec::new();
        let mut missing = Vec::new();
        let mut drift = BTreeMap::new();
        let mut binding_validity = BTreeMap::new();
        for binding in &state.policy_bindings {
            if binding.tenant_id != state.tenant_id {
                missing.push(format!(
                    "{}:case_policy_binding_security_domain_mismatch",
                    binding.binding_id
                ));
                continue;
            }
            let artifact = match self.policy_artifact_txn(txn, &binding.artifact_id) {
                Ok(artifact) => artifact,
                Err(error) => {
                    missing.push(format!("{}:{error}", binding.binding_id));
                    continue;
                }
            };
            if let Err(error) = binding.matches_artifact(&artifact) {
                missing.push(format!("{}:{error}", binding.binding_id));
                continue;
            }
            let publication_ok = self
                .policy_lifecycle_events_txn(txn, Some(&artifact.artifact_id))?
                .iter()
                .any(|event| {
                    event.event_id == binding.publication_event_id
                        && event.sequence == binding.publication_event_sequence
                        && event.action == PolicyLifecycleAction::Published
                        && event.artifact_id == artifact.artifact_id
                        && event.tenant_id == binding.tenant_id
                });
            if !publication_ok {
                missing.push(format!(
                    "{}:binding_publication_evidence_invalid",
                    binding.binding_id
                ));
                continue;
            }
            let view = self.policy_artifact_view_txn(txn, &artifact.artifact_id)?;
            let current = self.current_published_policy_txn(txn, &artifact.lineage())?;
            let catalog_drift = match view.lifecycle {
                PolicyLifecycleState::Published
                    if current.as_deref() == Some(artifact.artifact_id.as_str()) =>
                {
                    PolicyCatalogDrift::Current
                }
                PolicyLifecycleState::Superseded => PolicyCatalogDrift::Superseded {
                    current_artifact_id: view
                        .superseded_by
                        .clone()
                        .or(current)
                        .unwrap_or_else(|| "unknown".to_string()),
                },
                PolicyLifecycleState::Retired => PolicyCatalogDrift::Retired,
                PolicyLifecycleState::Revoked => PolicyCatalogDrift::Revoked,
                _ => PolicyCatalogDrift::NoCurrentPublishedArtifact,
            };
            drift.insert(binding.lineage_id.clone(), catalog_drift.clone());
            let revoke_event_id = view
                .lifecycle_events
                .iter()
                .rev()
                .find(|event| event.action == PolicyLifecycleAction::Revoked)
                .map(|event| event.event_id.clone());
            let (posture, reason) = binding_validity_posture(
                &artifact,
                &catalog_drift,
                revoke_event_id.is_some(),
                authority_time_unix_ms,
            );
            binding_validity.insert(
                binding.lineage_id.clone(),
                BindingValidity {
                    binding_id: binding.binding_id.clone(),
                    lineage_id: binding.lineage_id.clone(),
                    artifact_id: artifact.artifact_id.clone(),
                    contract: artifact.validity.clone(),
                    posture,
                    reason,
                    revoke_event_id,
                },
            );
            inputs.push(EffectivePolicyInput {
                binding: binding.clone(),
                artifact,
                drift: catalog_drift,
            });
        }
        if !missing.is_empty() {
            missing.sort();
            return Ok(NormativeStatus {
                case_id: case_id.to_string(),
                readiness: NormativeReadiness::Blocked,
                validity: PolicyValidityPosture::Unavailable,
                authority_time_unix_ms,
                observed_wall_time_unix_ms: authority_time_unix_ms,
                persisted_authority_floor_unix_ms: persisted_floor_unix_ms,
                binding_validity,
                effective_policy: None,
                missing,
                blocking_conflicts: Vec::new(),
                catalog_drift: drift,
            });
        }
        let mut status = materialize_effective_policy(case_id, inputs);
        status.authority_time_unix_ms = authority_time_unix_ms;
        status.observed_wall_time_unix_ms = authority_time_unix_ms;
        status.persisted_authority_floor_unix_ms = persisted_floor_unix_ms;
        status.binding_validity = binding_validity;
        status.validity = status
            .binding_validity
            .values()
            .map(|entry| entry.posture.clone())
            .max_by_key(validity_severity)
            .unwrap_or(PolicyValidityPosture::Unavailable);
        Ok(status)
    }

    fn authority_time_floor_txn<T: Transaction>(&self, txn: &T) -> Result<u64, String> {
        match txn.get(self.schema_meta, &AUTHORITY_TIME_FLOOR_KEY) {
            Ok(value) => std::str::from_utf8(value)
                .map_err(|error| format!("authority_time_floor_not_utf8: {error}"))?
                .parse::<u64>()
                .map_err(|error| format!("authority_time_floor_invalid: {error}")),
            Err(Error::NotFound) => Ok(0),
            Err(error) => Err(format!("failed to read authority time floor: {error}")),
        }
    }

    fn advance_authority_time_txn(
        &self,
        txn: &mut RwTransaction<'_>,
        observed_wall_time_unix_ms: u64,
    ) -> Result<u64, String> {
        let effective = observed_wall_time_unix_ms.max(self.authority_time_floor_txn(txn)?);
        txn.put(
            self.schema_meta,
            &AUTHORITY_TIME_FLOOR_KEY,
            &effective.to_string(),
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to persist authority time floor: {error}"))?;
        Ok(effective)
    }

    /// A mutating review action uses this write boundary first. Read-only
    /// status never appends invalidation; an attempted authority use does.
    pub fn invalidate_review_if_policy_unusable(
        &self,
        case_id: &str,
        review_id: &str,
    ) -> Result<Option<CanonicalCommit>, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start review validity transaction: {error}"))?;
        let state = self
            .get_case_state_txn(&txn, case_id)?
            .ok_or_else(|| format!("case_state_not_found: {case_id}"))?;
        let review = state
            .reviews
            .iter()
            .find(|review| review.review_id == review_id)
            .ok_or_else(|| "review_not_found".to_string())?;
        if !matches!(
            review.status,
            ReviewResolution::Pending | ReviewResolution::Deferred
        ) {
            return Ok(None);
        }
        let authority_time =
            self.advance_authority_time_txn(&mut txn, authority_wall_time_unix_ms())?;
        let floor = self.authority_time_floor_txn(&txn)?;
        let status = self.materialize_case_policy_at_txn(&txn, case_id, authority_time, floor)?;
        let (reason, source_ref) = if let Some(cancellation) = &state.cancellation {
            (
                AuthorityInvalidationReason::CaseCancelled,
                cancellation.transition_id.clone(),
            )
        } else if state.lifecycle == CaseLifecycle::Closed {
            (
                AuthorityInvalidationReason::CaseClosed,
                state
                    .closure
                    .as_ref()
                    .map(|closure| closure.transition_id.clone())
                    .unwrap_or_else(|| case_id.to_string()),
            )
        } else if status.validity == PolicyValidityPosture::Valid
            && status.effective_policy.as_ref().is_some_and(|effective| {
                effective.effective_policy_id == review.effective_policy_id
                    && effective.semantic_digest == review.effective_policy_digest
            })
        {
            return Ok(None);
        } else {
            let reason = match status.validity {
                PolicyValidityPosture::RefreshRequired => {
                    AuthorityInvalidationReason::PolicyRefreshRequired
                }
                PolicyValidityPosture::Stale => AuthorityInvalidationReason::PolicyStale,
                PolicyValidityPosture::Expired => AuthorityInvalidationReason::PolicyExpired,
                PolicyValidityPosture::Revoked => AuthorityInvalidationReason::PolicyRevoked,
                _ => AuthorityInvalidationReason::PolicyBasisChanged,
            };
            let source = status
                .binding_validity
                .values()
                .find_map(|binding| binding.revoke_event_id.clone())
                .or_else(|| {
                    status
                        .effective_policy
                        .as_ref()
                        .map(|effective| effective.effective_policy_id.clone())
                })
                .unwrap_or_else(|| case_id.to_string());
            (reason, source)
        };
        let mut pending = PendingTransition::new(
            format!(
                "transition:review-invalidated:{review_id}:{}",
                state.generation + 1
            ),
            case_id,
            state.generation,
            TransitionSource::component("yai.temporal_governance"),
            TransitionPayload::ReviewInvalidated {
                invalidation: ReviewInvalidation {
                    review_id: review_id.to_string(),
                    reason,
                    source_ref: source_ref.clone(),
                    invalidated_at_unix_ms: authority_time,
                },
            },
        );
        pending.causal_refs = vec![review_id.to_string(), source_ref];
        let commit = self.commit_transition_txn(&mut txn, pending, false)?;
        txn.commit()
            .map_err(|error| format!("failed to commit review invalidation: {error}"))?;
        Ok(Some(commit))
    }

    /// Atomically installs the durable cancellation barrier and terminalizes
    /// every still-usable review and every issued, not-yet-prepared Grant.
    pub fn cancel_case(
        &self,
        case_id: &str,
        actor_ref: &str,
        reason: &str,
    ) -> Result<CaseCancellationOutcome, String> {
        self.cancel_case_inner(case_id, actor_ref, reason, None)
    }

    pub fn cancel_tenant_case(
        &self,
        authenticated: &AuthenticatedPrincipal,
        case_id: &str,
        reason: &str,
    ) -> Result<CaseCancellationOutcome, String> {
        self.cancel_case_inner(
            case_id,
            &authenticated.projected_principal_id(),
            reason,
            Some(authenticated),
        )
    }

    fn cancel_case_inner(
        &self,
        case_id: &str,
        actor_ref: &str,
        reason: &str,
        authenticated: Option<&AuthenticatedPrincipal>,
    ) -> Result<CaseCancellationOutcome, String> {
        if actor_ref.trim().is_empty() || reason.trim().is_empty() {
            return Err("case_cancellation_actor_and_reason_required".to_string());
        }
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start Case cancellation transaction: {error}"))?;
        let state = self
            .get_case_state_txn(&txn, case_id)?
            .ok_or_else(|| format!("case_state_not_found: {case_id}"))?;
        let security_context =
            self.resolve_case_owner_context_txn(&txn, &state, actor_ref, authenticated)?;
        if state.lifecycle == CaseLifecycle::Closed {
            return Err("case_already_closed".to_string());
        }
        if state.cancellation.is_some() {
            return Ok(CaseCancellationOutcome {
                changed: false,
                commits: Vec::new(),
                state,
                invalidated_reviews: 0,
                abandoned_grants: 0,
            });
        }
        let authority_time =
            self.advance_authority_time_txn(&mut txn, authority_wall_time_unix_ms())?;
        let cancellation_transition_id = format!("transition:case-cancel:{case_id}");
        let cancellation = CaseCancellationState {
            actor_ref: actor_ref.to_string(),
            reason: reason.split_whitespace().collect::<Vec<_>>().join(" "),
            requested_at_unix_ms: authority_time,
            transition_id: cancellation_transition_id.clone(),
        };
        let pending_reviews = state
            .reviews
            .iter()
            .filter(|review| {
                matches!(
                    review.status,
                    ReviewResolution::Pending | ReviewResolution::Deferred
                )
            })
            .map(|review| review.review_id.clone())
            .collect::<Vec<_>>();
        let issued_grants = state
            .grants
            .iter()
            .filter(|grant| grant.status == GrantLifecycle::Issued)
            .map(|grant| grant.grant_id.clone())
            .collect::<Vec<_>>();
        let mut commits = Vec::new();
        let cancel = PendingTransition::new(
            &cancellation_transition_id,
            case_id,
            state.generation,
            TransitionSource {
                component: "yai.case_lifecycle".to_string(),
                participant_id: None,
                principal_id: security_context
                    .as_ref()
                    .map(|context| context.principal_id().to_string()),
                source_ref: None,
            },
            TransitionPayload::CaseCancellationRequested { cancellation },
        );
        commits.push(self.commit_transition_txn_at(
            &mut txn,
            cancel,
            false,
            None,
            security_context.as_ref(),
        )?);
        for review_id in &pending_reviews {
            let current = commits.last().expect("cancel commit").state.generation;
            let mut pending = PendingTransition::new(
                format!("transition:review-invalidated:cancel:{review_id}"),
                case_id,
                current,
                TransitionSource::component("yai.case_lifecycle"),
                TransitionPayload::ReviewInvalidated {
                    invalidation: ReviewInvalidation {
                        review_id: review_id.clone(),
                        reason: AuthorityInvalidationReason::CaseCancelled,
                        source_ref: cancellation_transition_id.clone(),
                        invalidated_at_unix_ms: authority_time,
                    },
                },
            );
            pending.causal_refs = vec![review_id.clone(), cancellation_transition_id.clone()];
            commits.push(self.commit_transition_txn(&mut txn, pending, false)?);
        }
        for grant_id in &issued_grants {
            let current = commits.last().expect("cancel commit").state.generation;
            let mut pending = PendingTransition::new(
                format!("transition:grant-abandoned:cancel:{grant_id}"),
                case_id,
                current,
                TransitionSource::component("yai.case_lifecycle"),
                TransitionPayload::ExecutionGrantInvalidated {
                    invalidation: ExecutionGrantInvalidation {
                        grant_id: grant_id.clone(),
                        disposition: GrantInvalidationDisposition::Abandoned,
                        reason: "case_cancelled_before_prepare".to_string(),
                        source_ref: cancellation_transition_id.clone(),
                        invalidated_at_unix_ms: authority_time,
                    },
                },
            );
            pending.causal_refs = vec![grant_id.clone(), cancellation_transition_id.clone()];
            commits.push(self.commit_transition_txn(&mut txn, pending, false)?);
        }
        let final_state = commits.last().expect("cancel commit").state.clone();
        txn.commit()
            .map_err(|error| format!("failed to commit Case cancellation: {error}"))?;
        Ok(CaseCancellationOutcome {
            changed: true,
            commits,
            state: final_state,
            invalidated_reviews: pending_reviews.len(),
            abandoned_grants: issued_grants.len(),
        })
    }

    /// Atomically verifies the terminal boundary and appends the one CaseClosed
    /// transition. Physical resources and historical records are untouched.
    pub fn close_case(
        &self,
        case_id: &str,
        actor_ref: &str,
        reason: &str,
    ) -> Result<CaseClosureOutcome, String> {
        self.close_case_inner(case_id, actor_ref, reason, None)
    }

    pub fn close_tenant_case(
        &self,
        authenticated: &AuthenticatedPrincipal,
        case_id: &str,
        reason: &str,
    ) -> Result<CaseClosureOutcome, String> {
        self.close_case_inner(
            case_id,
            &authenticated.projected_principal_id(),
            reason,
            Some(authenticated),
        )
    }

    fn close_case_inner(
        &self,
        case_id: &str,
        actor_ref: &str,
        reason: &str,
        authenticated: Option<&AuthenticatedPrincipal>,
    ) -> Result<CaseClosureOutcome, String> {
        if actor_ref.trim().is_empty() || reason.trim().is_empty() {
            return Err("case_closure_actor_and_reason_required".to_string());
        }
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start Case closure transaction: {error}"))?;
        let state = self
            .get_case_state_txn(&txn, case_id)?
            .ok_or_else(|| format!("case_state_not_found: {case_id}"))?;
        let security_context =
            self.resolve_case_owner_context_txn(&txn, &state, actor_ref, authenticated)?;
        if state.lifecycle == CaseLifecycle::Closed {
            return Ok(CaseClosureOutcome {
                changed: false,
                commit: None,
                state,
            });
        }
        let cancellation = state
            .cancellation
            .as_ref()
            .ok_or_else(|| "case_close_requires_cancellation".to_string())?;
        let mut blockers = closure_blockers(&state);
        blockers.extend(self.handoff_close_blockers_txn(&txn, &state)?);
        blockers.sort();
        blockers.dedup();
        if !blockers.is_empty() {
            return Err(format!("case_close_blocked: {}", blockers.join(",")));
        }
        let now = authority_wall_time_unix_ms();
        let admission_key = case_runtime_admission_key(case_id);
        match txn.get(self.case_runtime_admission, &admission_key) {
            Ok(value) => {
                let claim: CaseRuntimeAdmission = serde_json::from_slice(value)
                    .map_err(|error| format!("case_runtime_admission_decode_failed: {error}"))?;
                if claim.expires_at_unix_ms > now {
                    return Err(format!(
                        "case_close_blocked: live_runtime_admission:{}",
                        claim.run_id
                    ));
                }
                txn.del(self.case_runtime_admission, &admission_key, None)
                    .map_err(|error| format!("failed to clear stale runtime admission: {error}"))?;
            }
            Err(Error::NotFound) => {}
            Err(error) => return Err(format!("failed to inspect runtime admission: {error}")),
        }
        let authority_time = self.advance_authority_time_txn(&mut txn, now)?;
        let transition_id = format!("transition:case-close:{case_id}");
        let mut pending = PendingTransition::new(
            &transition_id,
            case_id,
            state.generation,
            TransitionSource {
                component: "yai.case_lifecycle".to_string(),
                participant_id: None,
                principal_id: security_context
                    .as_ref()
                    .map(|context| context.principal_id().to_string()),
                source_ref: Some(cancellation.transition_id.clone()),
            },
            TransitionPayload::CaseClosed {
                closure: CaseClosureState {
                    actor_ref: actor_ref.to_string(),
                    reason: reason.split_whitespace().collect::<Vec<_>>().join(" "),
                    closed_at_unix_ms: authority_time,
                    cancellation_ref: cancellation.transition_id.clone(),
                    transition_id: transition_id.clone(),
                },
            },
        );
        pending.causal_refs = vec![cancellation.transition_id.clone()];
        let commit = self.commit_transition_txn_at(
            &mut txn,
            pending,
            false,
            None,
            security_context.as_ref(),
        )?;
        txn.commit()
            .map_err(|error| format!("failed to commit Case closure: {error}"))?;
        Ok(CaseClosureOutcome {
            changed: true,
            state: commit.state.clone(),
            commit: Some(commit),
        })
    }

    /// Atomically appends one immutable canonical Transition and replaces the
    /// corresponding rebuildable CaseState materialization.
    pub fn commit_effect_prepared(
        &self,
        pending: PendingTransition,
    ) -> Result<PreparedCommitOutcome, String> {
        let TransitionPayload::EffectPrepared { prepared } = &pending.payload else {
            return Err("commit_effect_prepared_requires_prepare_payload".to_string());
        };
        if prepared.schema == crate::effect::PREPARED_EFFECT_SCHEMA {
            return Err("tenant_prepare_requires_atomic_resource_control_boundary".to_string());
        }
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start prepare authority transaction: {error}"))?;
        let state = self
            .get_case_state_txn(&txn, &pending.case_id)?
            .ok_or_else(|| format!("case_state_not_found: {}", pending.case_id))?;
        if state.generation != pending.expected_generation {
            return Err(format!(
                "stale_case_generation: expected={} actual={}",
                pending.expected_generation, state.generation
            ));
        }
        let grant_state = state
            .grants
            .iter()
            .find(|grant| grant.grant_id == prepared.grant_id)
            .ok_or_else(|| "prepare_without_grant".to_string())?;
        if grant_state.status != GrantLifecycle::Issued {
            return Err("prepare_requires_issued_grant".to_string());
        }
        let authority_time =
            self.advance_authority_time_txn(&mut txn, authority_wall_time_unix_ms())?;
        let floor = self.authority_time_floor_txn(&txn)?;
        let status =
            self.materialize_case_policy_at_txn(&txn, &pending.case_id, authority_time, floor)?;
        let invalidation = if let Some(cancellation) = &state.cancellation {
            Some((
                GrantInvalidationDisposition::Abandoned,
                "case_cancelled_before_prepare".to_string(),
                cancellation.transition_id.clone(),
            ))
        } else if grant_state.expires_at_unix_ms != 0
            && authority_time >= grant_state.expires_at_unix_ms
        {
            Some((
                GrantInvalidationDisposition::Expired,
                "execution_grant_expired_before_prepare".to_string(),
                prepared.grant_id.clone(),
            ))
        } else if status.validity != PolicyValidityPosture::Valid {
            let disposition = if status.validity == PolicyValidityPosture::Revoked {
                GrantInvalidationDisposition::Revoked
            } else {
                GrantInvalidationDisposition::Expired
            };
            let source = status
                .binding_validity
                .values()
                .find_map(|binding| binding.revoke_event_id.clone())
                .unwrap_or_else(|| prepared.grant_id.clone());
            Some((
                disposition,
                format!("policy_invalid_before_prepare:{:?}", status.validity),
                source,
            ))
        } else {
            None
        };
        if let Some((disposition, reason, source_ref)) = invalidation {
            let mut invalidation_pending = PendingTransition::new(
                format!(
                    "transition:grant-invalidated:{}:{}",
                    prepared.grant_id,
                    state.generation + 1
                ),
                &pending.case_id,
                state.generation,
                TransitionSource::component("yai.temporal_governance"),
                TransitionPayload::ExecutionGrantInvalidated {
                    invalidation: ExecutionGrantInvalidation {
                        grant_id: prepared.grant_id.clone(),
                        disposition,
                        reason,
                        source_ref: source_ref.clone(),
                        invalidated_at_unix_ms: authority_time,
                    },
                },
            );
            invalidation_pending.causal_refs = vec![prepared.grant_id.clone(), source_ref];
            let commit = self.commit_transition_txn(&mut txn, invalidation_pending, false)?;
            txn.commit()
                .map_err(|error| format!("failed to commit Grant invalidation: {error}"))?;
            return Ok(PreparedCommitOutcome::GrantInvalidated(commit));
        }
        let commit = self.commit_transition_txn(&mut txn, pending, false)?;
        txn.commit()
            .map_err(|error| format!("failed to commit prepared effect: {error}"))?;
        Ok(PreparedCommitOutcome::Prepared(commit))
    }

    /// Atomically validates Grant/time/policy authority, acquires one shared
    /// filesystem resource generation, seals the fence into PREPARE, appends
    /// the Case transition, and publishes resource current/history state.
    /// No externally visible resource lease can exist without PREPARE and no
    /// PREPARE can exist without its resource lease.
    pub fn commit_fenced_effect_prepared(
        &self,
        mut pending: PendingTransition,
        owner_pid: u32,
    ) -> Result<PreparedCommitOutcome, String> {
        if owner_pid != std::process::id() {
            return Err("resource_fence_owner_must_be_current_process".to_string());
        }
        let prepared_snapshot = match &pending.payload {
            TransitionPayload::EffectPrepared { prepared } => prepared.clone(),
            _ => return Err("commit_fenced_effect_prepared_requires_prepare_payload".to_string()),
        };
        if prepared_snapshot.schema != crate::effect::PREPARED_EFFECT_SCHEMA
            || prepared_snapshot.resource_fence.is_some()
        {
            return Err("fenced_prepare_requires_unsealed_v2_intent".to_string());
        }
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start fenced prepare transaction: {error}"))?;
        let state = self
            .get_case_state_txn(&txn, &pending.case_id)?
            .ok_or_else(|| format!("case_state_not_found: {}", pending.case_id))?;
        let tenant_id = state
            .tenant_id
            .as_deref()
            .ok_or_else(|| "fenced_prepare_requires_tenant_case".to_string())?;
        if state.lifecycle == CaseLifecycle::Closed {
            return Err("closed_case_cannot_acquire_resource".to_string());
        }
        if state.generation != pending.expected_generation {
            return Err(format!(
                "stale_case_generation: expected={} actual={}",
                pending.expected_generation, state.generation
            ));
        }
        let grant_state = state
            .grants
            .iter()
            .find(|grant| grant.grant_id == prepared_snapshot.grant_id)
            .ok_or_else(|| "prepare_without_grant".to_string())?;
        if grant_state.status != GrantLifecycle::Issued {
            return Err("prepare_requires_issued_grant".to_string());
        }
        let authority_time =
            self.advance_authority_time_txn(&mut txn, authority_wall_time_unix_ms())?;
        let floor = self.authority_time_floor_txn(&txn)?;
        let status =
            self.materialize_case_policy_at_txn(&txn, &pending.case_id, authority_time, floor)?;
        let invalidation = if let Some(cancellation) = &state.cancellation {
            Some((
                GrantInvalidationDisposition::Abandoned,
                "case_cancelled_before_prepare".to_string(),
                cancellation.transition_id.clone(),
            ))
        } else if grant_state.expires_at_unix_ms != 0
            && authority_time >= grant_state.expires_at_unix_ms
        {
            Some((
                GrantInvalidationDisposition::Expired,
                "execution_grant_expired_before_prepare".to_string(),
                prepared_snapshot.grant_id.clone(),
            ))
        } else if status.validity != PolicyValidityPosture::Valid {
            let disposition = if status.validity == PolicyValidityPosture::Revoked {
                GrantInvalidationDisposition::Revoked
            } else {
                GrantInvalidationDisposition::Expired
            };
            let source = status
                .binding_validity
                .values()
                .find_map(|binding| binding.revoke_event_id.clone())
                .unwrap_or_else(|| prepared_snapshot.grant_id.clone());
            Some((
                disposition,
                format!("policy_invalid_before_prepare:{:?}", status.validity),
                source,
            ))
        } else {
            None
        };
        if let Some((disposition, reason, source_ref)) = invalidation {
            let mut invalidation_pending = PendingTransition::new(
                format!(
                    "transition:grant-invalidated:{}:{}",
                    prepared_snapshot.grant_id,
                    state.generation + 1
                ),
                &pending.case_id,
                state.generation,
                TransitionSource::component("yai.temporal_governance"),
                TransitionPayload::ExecutionGrantInvalidated {
                    invalidation: ExecutionGrantInvalidation {
                        grant_id: prepared_snapshot.grant_id.clone(),
                        disposition,
                        reason,
                        source_ref: source_ref.clone(),
                        invalidated_at_unix_ms: authority_time,
                    },
                },
            );
            invalidation_pending.causal_refs = vec![prepared_snapshot.grant_id.clone(), source_ref];
            let commit = self.commit_transition_txn(&mut txn, invalidation_pending, false)?;
            txn.commit()
                .map_err(|error| format!("failed to commit Grant invalidation: {error}"))?;
            return Ok(PreparedCommitOutcome::GrantInvalidated(commit));
        }

        let binding = self
            .local_filesystem_binding_txn(
                &txn,
                &pending.case_id,
                &prepared_snapshot.resource_attachment_id,
            )?
            .ok_or_else(|| "fenced_prepare_local_binding_missing".to_string())?;
        let identity = ResourceIdentity::filesystem(tenant_id, &binding.canonical_root)?;
        self.reject_active_resource_conflict_txn(&txn, &identity)?;
        let prior = self.resource_control_state_txn(&txn, &identity.resource_id)?;
        let next_epoch = match prior.as_ref() {
            Some(current) => current
                .resource_epoch
                .checked_add(1)
                .ok_or_else(|| "resource_epoch_exhausted".to_string())?,
            None => 1,
        };
        let next_sequence = match prior.as_ref() {
            Some(current) => current
                .event_sequence
                .checked_add(1)
                .ok_or_else(|| "resource_event_sequence_exhausted".to_string())?,
            None => 1,
        };
        let owner_process_identity = LocalProcessIdentity::capture(owner_pid)?.canonical_identity();
        let fence = ResourceFence::issue(
            &identity,
            next_epoch,
            &prepared_snapshot.case_id,
            &prepared_snapshot.operation_id,
            &prepared_snapshot.grant_id,
            &prepared_snapshot.effect_id,
            owner_pid,
            &owner_process_identity,
            authority_time,
        )?;
        let previous = self.resource_control_event_at_sequence_txn(
            &txn,
            &identity.resource_id,
            prior.as_ref().map(|state| state.event_sequence),
        )?;
        let event = ResourceControlEvent::build(
            ResourceControlAction::Acquired,
            &identity,
            &fence,
            next_sequence,
            authority_time,
            previous.as_ref(),
        )?;
        let control = ResourceControlState {
            schema: RESOURCE_CONTROL_STATE_SCHEMA.to_string(),
            identity,
            resource_epoch: next_epoch,
            event_sequence: next_sequence,
            last_event_id: Some(event.event_id.clone()),
            last_event_digest: Some(event.integrity_digest.clone()),
            active_lease: Some(ActiveResourceLease {
                fence: fence.clone(),
            }),
        };
        control.validate()?;
        let TransitionPayload::EffectPrepared { prepared } = &mut pending.payload else {
            unreachable!()
        };
        prepared.resource_fence = Some(fence.clone());
        pending.causal_refs.push(fence.fence_id.clone());
        let commit = self.commit_transition_txn_at_with_fence(
            &mut txn,
            pending,
            false,
            None,
            None,
            Some(&fence),
        )?;
        self.put_resource_control_event_txn(&mut txn, &event)?;
        self.put_resource_control_state_txn(&mut txn, &control)?;
        txn.commit()
            .map_err(|error| format!("failed to commit fenced prepared effect: {error}"))?;
        Ok(PreparedCommitOutcome::Prepared(commit))
    }

    /// Process-signal equivalent of `commit_fenced_effect_prepared`. It uses
    /// the same Grant/time/policy checks and the same atomic resource epoch +
    /// Case PREPARE transaction; only physical identity resolution differs.
    pub fn commit_fenced_process_effect_prepared(
        &self,
        mut pending: PendingTransition,
        owner_pid: u32,
    ) -> Result<PreparedCommitOutcome, String> {
        if owner_pid != std::process::id() {
            return Err("resource_fence_owner_must_be_current_process".to_string());
        }
        let prepared_snapshot = match &pending.payload {
            TransitionPayload::ProcessEffectPrepared { prepared } => prepared.clone(),
            _ => {
                return Err(
                    "commit_fenced_process_effect_prepared_requires_prepare_payload".to_string(),
                )
            }
        };
        if prepared_snapshot.schema != crate::effect::PREPARED_PROCESS_EFFECT_SCHEMA
            || prepared_snapshot.resource_fence.is_some()
        {
            return Err("fenced_process_prepare_requires_unsealed_intent".to_string());
        }
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start fenced process prepare: {error}"))?;
        let state = self
            .get_case_state_txn(&txn, &pending.case_id)?
            .ok_or_else(|| format!("case_state_not_found: {}", pending.case_id))?;
        let tenant_id = state
            .tenant_id
            .as_deref()
            .ok_or_else(|| "fenced_prepare_requires_tenant_case".to_string())?;
        if state.lifecycle == CaseLifecycle::Closed {
            return Err("closed_case_cannot_acquire_resource".to_string());
        }
        if state.generation != pending.expected_generation {
            return Err(format!(
                "stale_case_generation: expected={} actual={}",
                pending.expected_generation, state.generation
            ));
        }
        let grant_state = state
            .grants
            .iter()
            .find(|grant| grant.grant_id == prepared_snapshot.grant_id)
            .ok_or_else(|| "prepare_without_grant".to_string())?;
        if grant_state.status != GrantLifecycle::Issued {
            return Err("prepare_requires_issued_grant".to_string());
        }
        let authority_time =
            self.advance_authority_time_txn(&mut txn, authority_wall_time_unix_ms())?;
        let floor = self.authority_time_floor_txn(&txn)?;
        let status =
            self.materialize_case_policy_at_txn(&txn, &pending.case_id, authority_time, floor)?;
        let invalidation = if let Some(cancellation) = &state.cancellation {
            Some((
                GrantInvalidationDisposition::Abandoned,
                "case_cancelled_before_prepare".to_string(),
                cancellation.transition_id.clone(),
            ))
        } else if grant_state.expires_at_unix_ms != 0
            && authority_time >= grant_state.expires_at_unix_ms
        {
            Some((
                GrantInvalidationDisposition::Expired,
                "execution_grant_expired_before_prepare".to_string(),
                prepared_snapshot.grant_id.clone(),
            ))
        } else if status.validity != PolicyValidityPosture::Valid {
            let disposition = if status.validity == PolicyValidityPosture::Revoked {
                GrantInvalidationDisposition::Revoked
            } else {
                GrantInvalidationDisposition::Expired
            };
            let source = status
                .binding_validity
                .values()
                .find_map(|binding| binding.revoke_event_id.clone())
                .unwrap_or_else(|| prepared_snapshot.grant_id.clone());
            Some((
                disposition,
                format!("policy_invalid_before_prepare:{:?}", status.validity),
                source,
            ))
        } else {
            None
        };
        if let Some((disposition, reason, source_ref)) = invalidation {
            let mut invalidation_pending = PendingTransition::new(
                format!(
                    "transition:grant-invalidated:{}:{}",
                    prepared_snapshot.grant_id,
                    state.generation + 1
                ),
                &pending.case_id,
                state.generation,
                TransitionSource::component("yai.temporal_governance"),
                TransitionPayload::ExecutionGrantInvalidated {
                    invalidation: ExecutionGrantInvalidation {
                        grant_id: prepared_snapshot.grant_id.clone(),
                        disposition,
                        reason,
                        source_ref: source_ref.clone(),
                        invalidated_at_unix_ms: authority_time,
                    },
                },
            );
            invalidation_pending.causal_refs = vec![prepared_snapshot.grant_id.clone(), source_ref];
            let commit = self.commit_transition_txn(&mut txn, invalidation_pending, false)?;
            txn.commit()
                .map_err(|error| format!("failed to commit Grant invalidation: {error}"))?;
            return Ok(PreparedCommitOutcome::GrantInvalidated(commit));
        }

        let binding = self
            .local_process_binding_txn(
                &txn,
                &pending.case_id,
                &prepared_snapshot.resource_attachment_id,
            )?
            .ok_or_else(|| "fenced_prepare_local_process_binding_missing".to_string())?;
        if binding.process.canonical_identity()
            != prepared_snapshot
                .expected_pre_observation
                .process_identity
                .canonical_identity()
        {
            return Err("fenced_prepare_process_birth_identity_mismatch".to_string());
        }
        let identity = ResourceIdentity::process(tenant_id, &binding.process)?;
        self.reject_active_resource_conflict_txn(&txn, &identity)?;
        let prior = self.resource_control_state_txn(&txn, &identity.resource_id)?;
        let next_epoch = match prior.as_ref() {
            Some(current) => current
                .resource_epoch
                .checked_add(1)
                .ok_or_else(|| "resource_epoch_exhausted".to_string())?,
            None => 1,
        };
        let next_sequence = match prior.as_ref() {
            Some(current) => current
                .event_sequence
                .checked_add(1)
                .ok_or_else(|| "resource_event_sequence_exhausted".to_string())?,
            None => 1,
        };
        let owner_process_identity = LocalProcessIdentity::capture(owner_pid)?.canonical_identity();
        let fence = ResourceFence::issue(
            &identity,
            next_epoch,
            &prepared_snapshot.case_id,
            &prepared_snapshot.operation_id,
            &prepared_snapshot.grant_id,
            &prepared_snapshot.effect_id,
            owner_pid,
            &owner_process_identity,
            authority_time,
        )?;
        let previous = self.resource_control_event_at_sequence_txn(
            &txn,
            &identity.resource_id,
            prior.as_ref().map(|state| state.event_sequence),
        )?;
        let event = ResourceControlEvent::build(
            ResourceControlAction::Acquired,
            &identity,
            &fence,
            next_sequence,
            authority_time,
            previous.as_ref(),
        )?;
        let control = ResourceControlState {
            schema: RESOURCE_CONTROL_STATE_SCHEMA.to_string(),
            identity,
            resource_epoch: next_epoch,
            event_sequence: next_sequence,
            last_event_id: Some(event.event_id.clone()),
            last_event_digest: Some(event.integrity_digest.clone()),
            active_lease: Some(ActiveResourceLease {
                fence: fence.clone(),
            }),
        };
        control.validate()?;
        let TransitionPayload::ProcessEffectPrepared { prepared } = &mut pending.payload else {
            unreachable!()
        };
        prepared.resource_fence = Some(fence.clone());
        pending.causal_refs.push(fence.fence_id.clone());
        let commit = self.commit_transition_txn_at_with_fence(
            &mut txn,
            pending,
            false,
            None,
            None,
            Some(&fence),
        )?;
        self.put_resource_control_event_txn(&mut txn, &event)?;
        self.put_resource_control_state_txn(&mut txn, &control)?;
        txn.commit()
            .map_err(|error| format!("failed to commit fenced process PREPARE: {error}"))?;
        Ok(PreparedCommitOutcome::Prepared(commit))
    }

    /// Commits a terminal Case effect and releases the exact current resource
    /// fence in the same LMDB transaction. Indeterminate effects are not
    /// terminal and deliberately have no release path here.
    pub fn commit_fenced_effect_terminal(
        &self,
        pending: PendingTransition,
        fence: &ResourceFence,
    ) -> Result<CanonicalCommit, String> {
        fence.validate_integrity()?;
        let terminal_effect = match &pending.payload {
            TransitionPayload::EffectFinalized { effect_id, .. } => effect_id,
            TransitionPayload::ProcessEffectFinalized { effect_id, .. } => effect_id,
            TransitionPayload::EffectReconciled {
                effect_id,
                conclusion:
                    crate::effect::ReconciliationConclusion::EffectObserved
                    | crate::effect::ReconciliationConclusion::NoEffectObserved,
                ..
            } => effect_id,
            _ => return Err("resource_release_requires_terminal_effect_transition".to_string()),
        };
        if terminal_effect != &fence.effect_id || pending.case_id != fence.case_id {
            return Err("resource_release_effect_or_case_mismatch".to_string());
        }
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start fenced terminal transaction: {error}"))?;
        self.validate_carrier_fence_txn(&txn, fence, false)?;
        let commit = self.commit_transition_txn(&mut txn, pending, false)?;
        let mut state = self
            .resource_control_state_txn(&txn, &fence.resource_id)?
            .ok_or_else(|| "resource_control_state_missing_at_release".to_string())?;
        state.event_sequence = state
            .event_sequence
            .checked_add(1)
            .ok_or_else(|| "resource_event_sequence_exhausted".to_string())?;
        let released_at =
            self.advance_authority_time_txn(&mut txn, authority_wall_time_unix_ms())?;
        let previous = self.resource_control_event_at_sequence_txn(
            &txn,
            &fence.resource_id,
            Some(state.event_sequence - 1),
        )?;
        let event = ResourceControlEvent::build(
            ResourceControlAction::Released,
            &state.identity,
            fence,
            state.event_sequence,
            released_at,
            previous.as_ref(),
        )?;
        state.active_lease = None;
        state.schema = RESOURCE_CONTROL_STATE_SCHEMA.to_string();
        state.last_event_id = Some(event.event_id.clone());
        state.last_event_digest = Some(event.integrity_digest.clone());
        state.validate()?;
        self.put_resource_control_event_txn(&mut txn, &event)?;
        self.put_resource_control_state_txn(&mut txn, &state)?;
        txn.commit()
            .map_err(|error| format!("failed to commit terminal resource release: {error}"))?;
        Ok(commit)
    }

    /// Reclaims the same unresolved effect after the exact former owner
    /// process dies. The effect identity is unchanged; a new resource epoch
    /// makes every old carrier request stale.
    pub fn reclaim_resource_for_effect(
        &self,
        prior_fence: &ResourceFence,
        new_owner_pid: u32,
    ) -> Result<ResourceFence, String> {
        if new_owner_pid != std::process::id() {
            return Err("resource_reclaim_owner_must_be_current_process".to_string());
        }
        prior_fence.validate_integrity()?;
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start resource reclaim: {error}"))?;
        let mut state = self
            .resource_control_state_txn(&txn, &prior_fence.resource_id)?
            .ok_or_else(|| "resource_control_state_missing_at_reclaim".to_string())?;
        let active = state
            .active_lease
            .as_ref()
            .ok_or_else(|| "resource_reclaim_requires_active_effect".to_string())?;
        if active.fence != *prior_fence {
            return Err("resource_reclaim_prior_fence_not_current".to_string());
        }
        if active.fence.effect_id != prior_fence.effect_id
            || active.fence.case_id != prior_fence.case_id
            || active.fence.grant_id != prior_fence.grant_id
        {
            return Err("resource_reclaim_may_only_continue_same_effect".to_string());
        }
        if resource_owner_is_live(&active.fence) {
            return Err("live_resource_owner_cannot_be_reclaimed".to_string());
        }
        let owner_identity = LocalProcessIdentity::capture(new_owner_pid)?.canonical_identity();
        state.resource_epoch = state
            .resource_epoch
            .checked_add(1)
            .ok_or_else(|| "resource_epoch_exhausted".to_string())?;
        state.event_sequence = state
            .event_sequence
            .checked_add(1)
            .ok_or_else(|| "resource_event_sequence_exhausted".to_string())?;
        let now = self.advance_authority_time_txn(&mut txn, authority_wall_time_unix_ms())?;
        let fence = ResourceFence::issue(
            &state.identity,
            state.resource_epoch,
            &prior_fence.case_id,
            &prior_fence.operation_id,
            &prior_fence.grant_id,
            &prior_fence.effect_id,
            new_owner_pid,
            &owner_identity,
            now,
        )?;
        state.active_lease = Some(ActiveResourceLease {
            fence: fence.clone(),
        });
        let previous = self.resource_control_event_at_sequence_txn(
            &txn,
            &state.identity.resource_id,
            Some(state.event_sequence - 1),
        )?;
        let event = ResourceControlEvent::build(
            ResourceControlAction::Reclaimed,
            &state.identity,
            &fence,
            state.event_sequence,
            now,
            previous.as_ref(),
        )?;
        state.schema = RESOURCE_CONTROL_STATE_SCHEMA.to_string();
        state.last_event_id = Some(event.event_id.clone());
        state.last_event_digest = Some(event.integrity_digest.clone());
        state.validate()?;
        self.put_resource_control_event_txn(&mut txn, &event)?;
        self.put_resource_control_state_txn(&mut txn, &state)?;
        txn.commit()
            .map_err(|error| format!("failed to commit resource reclaim: {error}"))?;
        Ok(fence)
    }

    pub fn get_resource_control_state(
        &self,
        resource_id: &str,
    ) -> Result<Option<ResourceControlState>, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to read resource control state: {error}"))?;
        self.resource_control_state_txn(&txn, resource_id)
    }

    pub fn list_resource_control_events(
        &self,
        resource_id: &str,
    ) -> Result<Vec<ResourceControlEvent>, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to read resource control history: {error}"))?;
        self.resource_control_events_txn(&txn, resource_id)
    }

    /// Rebuilds only shared resource current authority from its append-only
    /// v2 event chain. No Case transition, Decision, Grant, Effect, or Case
    /// generation is read or changed by this recovery operation.
    pub fn rebuild_resource_control_state(
        &self,
        resource_id: &str,
    ) -> Result<ResourceControlState, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start resource history rebuild: {error}"))?;
        let events = self.resource_control_events_txn(&txn, resource_id)?;
        let rebuilt = replay_resource_control_state(&events)?;
        if rebuilt.identity.resource_id != resource_id {
            return Err("resource_history_rebuild_identity_mismatch".to_string());
        }
        self.put_resource_control_state_txn(&mut txn, &rebuilt)?;
        txn.commit()
            .map_err(|error| format!("failed to commit resource history rebuild: {error}"))?;
        Ok(rebuilt)
    }

    pub fn commit_transition(&self, pending: PendingTransition) -> Result<CanonicalCommit, String> {
        self.commit_transition_inner(pending, false)
    }

    /// Canonical write entry point for Tenant-scoped human/admin mutations.
    /// Authentication is re-resolved from the kernel binding and security
    /// catalog inside the same LMDB transaction that appends Case truth.
    pub fn commit_secured_transition(
        &self,
        authenticated: &AuthenticatedPrincipal,
        tenant_id: &str,
        pending: PendingTransition,
        owner_required: bool,
    ) -> Result<CanonicalCommit, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start secured canonical write: {error}"))?;
        let context = self.resolve_security_context_txn(&txn, authenticated, tenant_id)?;
        if owner_required {
            context.require_owner()?;
        }
        let commit =
            self.commit_transition_txn_at(&mut txn, pending, false, None, Some(&context))?;
        txn.commit()
            .map_err(|error| format!("failed to commit secured canonical write: {error}"))?;
        Ok(commit)
    }

    pub fn create_tenant_case(
        &self,
        authenticated: &AuthenticatedPrincipal,
        tenant_id: &str,
        case_id: &str,
    ) -> Result<CanonicalCommit, String> {
        let principal_id = authenticated.projected_principal_id();
        let pending = PendingTransition::new(
            format!("transition:tenant-case-open:{case_id}"),
            case_id,
            0,
            TransitionSource {
                component: "yai.case_security".to_string(),
                participant_id: None,
                principal_id: Some(principal_id.clone()),
                source_ref: Some(tenant_id.to_string()),
            },
            TransitionPayload::TenantCaseOpened {
                lifecycle: CaseLifecycle::Open,
                tenant_id: tenant_id.to_string(),
                principal_id,
            },
        );
        self.commit_secured_transition(authenticated, tenant_id, pending, true)
    }

    pub fn get_case_state_authorized(
        &self,
        authenticated: &AuthenticatedPrincipal,
        case_id: &str,
    ) -> Result<CaseState, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start secured Case read: {error}"))?;
        let state = self
            .get_case_state_txn(&txn, case_id)?
            .ok_or_else(|| "case_not_visible".to_string())?;
        let tenant_id = state
            .tenant_id
            .as_deref()
            .ok_or_else(|| "legacy_unscoped_case_read_requires_compatibility_path".to_string())?;
        self.resolve_security_context_txn(&txn, authenticated, tenant_id)
            .map_err(|_| "case_not_visible".to_string())?;
        Ok(state)
    }

    pub fn define_workflow(
        &self,
        authenticated: &AuthenticatedPrincipal,
        input: WorkflowDefinitionInput,
        now_unix_ms: u64,
    ) -> Result<WorkflowDefinition, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start WorkflowDefinition write: {error}"))?;
        let context = self.resolve_security_context_txn(&txn, authenticated, &input.tenant_id)?;
        context.require_owner()?;
        let definition = WorkflowDefinition::build(input, context.principal_id(), now_unix_ms)?;
        self.validate_definition_composition_txn(&txn, &definition)?;
        let version_key = workflow_definition_version_key(
            &definition.tenant_id,
            &definition.workflow_key,
            &definition.declared_version,
        );
        if let Ok(existing_id) = txn.get(self.workflow_definitions, &version_key) {
            let existing_id = std::str::from_utf8(existing_id)
                .map_err(|error| format!("workflow version index is not utf8: {error}"))?;
            let existing = self
                .workflow_definition_txn(&txn, existing_id)?
                .ok_or_else(|| "workflow_definition_version_index_dangling".to_string())?;
            if existing.content_digest != definition.content_digest {
                return Err("workflow_definition_version_collision".to_string());
            }
            return Ok(existing);
        }
        put_json_txn(
            &mut txn,
            self.workflow_definitions,
            &workflow_definition_key(&definition.workflow_definition_id),
            &definition,
            WriteFlags::NO_OVERWRITE,
            "WorkflowDefinition",
        )?;
        txn.put(
            self.workflow_definitions,
            &version_key,
            &definition.workflow_definition_id,
            WriteFlags::NO_OVERWRITE,
        )
        .map_err(|error| format!("failed to index WorkflowDefinition version: {error}"))?;
        txn.commit()
            .map_err(|error| format!("failed to commit WorkflowDefinition: {error}"))?;
        Ok(definition)
    }

    pub fn get_workflow_definition_authorized(
        &self,
        authenticated: &AuthenticatedPrincipal,
        workflow_definition_id: &str,
    ) -> Result<WorkflowDefinition, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start WorkflowDefinition read: {error}"))?;
        let definition = self
            .workflow_definition_txn(&txn, workflow_definition_id)?
            .ok_or_else(|| "workflow_definition_not_visible".to_string())?;
        self.resolve_security_context_txn(&txn, authenticated, &definition.tenant_id)
            .map_err(|_| "workflow_definition_not_visible".to_string())?;
        Ok(definition)
    }

    pub fn list_workflow_definitions_authorized(
        &self,
        authenticated: &AuthenticatedPrincipal,
        tenant_id: &str,
    ) -> Result<Vec<WorkflowDefinition>, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start WorkflowDefinition list: {error}"))?;
        self.resolve_security_context_txn(&txn, authenticated, tenant_id)?;
        let mut cursor = txn
            .open_ro_cursor(self.workflow_definitions)
            .map_err(|error| format!("failed to open WorkflowDefinition cursor: {error}"))?;
        let mut definitions = Vec::new();
        for (key, value) in cursor.iter() {
            if !key.starts_with(b"definition:") {
                continue;
            }
            let definition: WorkflowDefinition = serde_json::from_slice(value)
                .map_err(|error| format!("invalid WorkflowDefinition JSON: {error}"))?;
            definition.validate_integrity()?;
            if definition.tenant_id == tenant_id {
                definitions.push(definition);
            }
        }
        definitions.sort_by(|left, right| {
            left.workflow_key
                .cmp(&right.workflow_key)
                .then_with(|| left.declared_version.cmp(&right.declared_version))
                .then_with(|| {
                    left.workflow_definition_id
                        .cmp(&right.workflow_definition_id)
                })
        });
        Ok(definitions)
    }

    pub fn bind_case_workflow(
        &self,
        authenticated: &AuthenticatedPrincipal,
        case_id: &str,
        workflow_definition_id: &str,
        executor_bindings: Vec<WorkflowExecutorBinding>,
        resource_bindings: Vec<WorkflowResourceBinding>,
        now_unix_ms: u64,
    ) -> Result<CanonicalCommit, String> {
        self.bind_case_workflow_composed(
            authenticated,
            case_id,
            workflow_definition_id,
            executor_bindings,
            resource_bindings,
            Vec::new(),
            now_unix_ms,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn bind_case_workflow_composed(
        &self,
        authenticated: &AuthenticatedPrincipal,
        case_id: &str,
        workflow_definition_id: &str,
        executor_bindings: Vec<WorkflowExecutorBinding>,
        resource_bindings: Vec<WorkflowResourceBinding>,
        case_bindings: Vec<WorkflowCaseBinding>,
        now_unix_ms: u64,
    ) -> Result<CanonicalCommit, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start Case workflow bind: {error}"))?;
        let state = self
            .get_case_state_txn(&txn, case_id)?
            .ok_or_else(|| "case_not_visible".to_string())?;
        let tenant_id = state
            .tenant_id
            .as_deref()
            .ok_or_else(|| "workflow_binding_requires_tenant_case".to_string())?;
        let context = self.resolve_security_context_txn(&txn, authenticated, tenant_id)?;
        context.require_owner()?;
        if state.workflow_binding.is_some() {
            return Err("case_workflow_already_bound".to_string());
        }
        let definition = self
            .workflow_definition_txn(&txn, workflow_definition_id)?
            .ok_or_else(|| "workflow_definition_not_visible".to_string())?;
        if definition.tenant_id != tenant_id {
            return Err("cross_tenant_workflow_binding_rejected".to_string());
        }
        for binding in &executor_bindings {
            if !state
                .participants
                .iter()
                .any(|participant| participant.participant_id == binding.participant_id)
            {
                return Err("workflow_executor_participant_not_in_case".to_string());
            }
        }
        for binding in &resource_bindings {
            if !state
                .resources
                .iter()
                .any(|resource| resource.attachment_id == binding.attachment_id)
            {
                return Err("workflow_resource_not_in_case".to_string());
            }
        }
        for binding in &case_bindings {
            let target = self
                .get_case_state_txn(&txn, &binding.case_id)?
                .ok_or_else(|| "workflow_target_case_not_visible".to_string())?;
            if target.tenant_id.as_deref() != Some(tenant_id) || target.case_id == state.case_id {
                return Err("workflow_cross_tenant_or_self_case_binding_rejected".to_string());
            }
        }
        let binding = if case_bindings.is_empty() {
            CaseWorkflowBinding::build(
                tenant_id,
                case_id,
                &definition,
                executor_bindings,
                resource_bindings,
                state.generation + 1,
                context.principal_id(),
                now_unix_ms,
            )?
        } else {
            CaseWorkflowBinding::build_with_case_bindings(
                tenant_id,
                case_id,
                &definition,
                executor_bindings,
                resource_bindings,
                case_bindings,
                state.generation + 1,
                context.principal_id(),
                now_unix_ms,
            )?
        };
        let definitions = self.workflow_definition_graph_for_operations_txn(
            &txn,
            &definition,
            &state.workflow_amendments,
            &[],
        )?;
        derive_effective_workflow_topology(&definition, &binding, &[], &definitions)?;
        let mut pending = PendingTransition::new(
            format!("transition:workflow-bind:{}", binding.binding_id),
            case_id,
            state.generation,
            TransitionSource {
                component: "yai.workflow".to_string(),
                participant_id: None,
                principal_id: Some(context.principal_id().to_string()),
                source_ref: Some(definition.workflow_definition_id.clone()),
            },
            TransitionPayload::CaseWorkflowBound {
                binding: binding.clone(),
            },
        );
        pending.causal_refs = vec![definition.workflow_definition_id.clone()];
        let commit =
            self.commit_transition_txn_at(&mut txn, pending, false, None, Some(&context))?;
        txn.commit()
            .map_err(|error| format!("failed to commit Case workflow binding: {error}"))?;
        Ok(commit)
    }

    pub fn workflow_status_authorized(
        &self,
        authenticated: &AuthenticatedPrincipal,
        case_id: &str,
    ) -> Result<WorkflowResolution, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start workflow status read: {error}"))?;
        let state = self
            .get_case_state_txn(&txn, case_id)?
            .ok_or_else(|| "case_not_visible".to_string())?;
        let tenant_id = state
            .tenant_id
            .as_deref()
            .ok_or_else(|| "workflow_requires_tenant_case".to_string())?;
        self.resolve_security_context_txn(&txn, authenticated, tenant_id)
            .map_err(|_| "case_not_visible".to_string())?;
        let binding = state
            .workflow_binding
            .as_ref()
            .ok_or_else(|| "case_workflow_not_bound".to_string())?;
        let definition = self
            .workflow_definition_txn(&txn, &binding.workflow_definition_id)?
            .ok_or_else(|| "bound_workflow_definition_missing".to_string())?;
        if definition.integrity_digest != binding.workflow_definition_digest {
            return Err("bound_workflow_definition_digest_mismatch".to_string());
        }
        let definitions = self.workflow_definition_graph_for_operations_txn(
            &txn,
            &definition,
            &state.workflow_amendments,
            &[],
        )?;
        let history = self.list_case_transitions_txn(&txn, case_id)?;
        resolve_workflow_with_definitions(&definition, binding, &state, &history, &definitions)
    }

    pub fn workflow_model_output_contract_authorized(
        &self,
        authenticated: &AuthenticatedPrincipal,
        case_id: &str,
        workflow_execution_id: &str,
    ) -> Result<(crate::workflow::ModelWorkOutputContract, String), String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start workflow output-contract read: {error}"))?;
        let state = self
            .get_case_state_txn(&txn, case_id)?
            .ok_or_else(|| "case_not_visible".to_string())?;
        let tenant_id = state
            .tenant_id
            .as_deref()
            .ok_or_else(|| "workflow_requires_tenant_case".to_string())?;
        self.resolve_security_context_txn(&txn, authenticated, tenant_id)
            .map_err(|_| "case_not_visible".to_string())?;
        let binding = state
            .workflow_binding
            .as_ref()
            .ok_or_else(|| "case_workflow_not_bound".to_string())?;
        let definition = self
            .workflow_definition_txn(&txn, &binding.workflow_definition_id)?
            .ok_or_else(|| "bound_workflow_definition_missing".to_string())?;
        let definitions = self.workflow_definition_graph_for_operations_txn(
            &txn,
            &definition,
            &state.workflow_amendments,
            &[],
        )?;
        let topology = derive_effective_workflow_topology(
            &definition,
            binding,
            &state.workflow_amendments,
            &definitions,
        )?;
        let execution = state
            .workflow_executions
            .iter()
            .find(|execution| execution.execution_id == workflow_execution_id)
            .ok_or_else(|| "workflow_execution_not_found".to_string())?;
        let node = topology
            .node(&execution.node_id)
            .map(|value| &value.node)
            .ok_or_else(|| "workflow_node_not_found".to_string())?;
        match &node.kind {
            WorkflowNodeKind::ModelWork {
                output_contract, ..
            } => Ok((output_contract.clone(), topology.topology_digest)),
            _ => Err("workflow_execution_is_not_model_work".to_string()),
        }
    }

    pub fn propose_workflow_plan_patch_human(
        &self,
        authenticated: &AuthenticatedPrincipal,
        case_id: &str,
        input: WorkflowPlanPatchInput,
        now_unix_ms: u64,
    ) -> Result<CanonicalCommit, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start Workflow PlanPatch proposal: {error}"))?;
        let state = self
            .get_case_state_txn(&txn, case_id)?
            .ok_or_else(|| "case_not_visible".to_string())?;
        let tenant_id = state
            .tenant_id
            .as_deref()
            .ok_or_else(|| "workflow_patch_requires_tenant_case".to_string())?;
        let context = self.resolve_security_context_txn(&txn, authenticated, tenant_id)?;
        context.require_owner()?;
        let commit = self.propose_workflow_plan_patch_txn(
            &mut txn,
            &state,
            &context,
            input,
            WorkflowPlanPatchOrigin::AuthenticatedHuman {
                principal_id: context.principal_id().to_string(),
            },
            now_unix_ms,
        )?;
        txn.commit()
            .map_err(|error| format!("failed to commit Workflow PlanPatch proposal: {error}"))?;
        Ok(commit)
    }

    pub fn propose_workflow_plan_patch_from_provider_result(
        &self,
        authenticated: &AuthenticatedPrincipal,
        case_id: &str,
        provider_result_id: &str,
        now_unix_ms: u64,
    ) -> Result<CanonicalCommit, String> {
        let mut txn = self.env.begin_rw_txn().map_err(|error| {
            format!("failed to start model Workflow PlanPatch proposal: {error}")
        })?;
        let state = self
            .get_case_state_txn(&txn, case_id)?
            .ok_or_else(|| "case_not_visible".to_string())?;
        let tenant_id = state
            .tenant_id
            .as_deref()
            .ok_or_else(|| "workflow_patch_requires_tenant_case".to_string())?;
        let context = self.resolve_security_context_txn(&txn, authenticated, tenant_id)?;
        let binding = state
            .workflow_binding
            .as_ref()
            .ok_or_else(|| "case_workflow_not_bound".to_string())?;
        let definition = self
            .workflow_definition_txn(&txn, &binding.workflow_definition_id)?
            .ok_or_else(|| "bound_workflow_definition_missing".to_string())?;
        let definitions = self.workflow_definition_graph_for_operations_txn(
            &txn,
            &definition,
            &state.workflow_amendments,
            &[],
        )?;
        let topology = derive_effective_workflow_topology(
            &definition,
            binding,
            &state.workflow_amendments,
            &definitions,
        )?;
        let history = self.list_case_transitions_txn(&txn, case_id)?;
        let result_transition = history
            .iter()
            .find(|transition| {
                matches!(
                    &transition.payload,
                    TransitionPayload::ProviderResultRecorded { result_id, .. }
                        if result_id == provider_result_id
                )
            })
            .ok_or_else(|| "workflow_plan_patch_provider_result_missing".to_string())?;
        let (invocation_id, result_lineage) = match &result_transition.payload {
            TransitionPayload::ProviderResultRecorded {
                invocation_id,
                semantic_lineage,
                ..
            } => (invocation_id, semantic_lineage),
            _ => unreachable!(),
        };
        let invocation_transition = history
            .iter()
            .find(|transition| {
                matches!(
                    &transition.payload,
                    TransitionPayload::ProviderInvocationStarted {
                        invocation_id: candidate,
                        ..
                    } if candidate == invocation_id
                )
            })
            .ok_or_else(|| "workflow_plan_patch_provider_invocation_missing".to_string())?;
        let invocation_lineage = match &invocation_transition.payload {
            TransitionPayload::ProviderInvocationStarted {
                semantic_lineage, ..
            } => semantic_lineage,
            _ => unreachable!(),
        };
        if result_lineage != invocation_lineage {
            return Err("workflow_plan_patch_provider_lineage_mismatch".to_string());
        }
        let eligible_executions = state
            .workflow_executions
            .iter()
            .filter(|execution| {
                invocation_transition
                    .causal_refs
                    .iter()
                    .any(|value| value == &execution.execution_id)
                    && result_transition
                        .causal_refs
                        .iter()
                        .any(|value| value == &execution.execution_id)
            })
            .collect::<Vec<_>>();
        if eligible_executions.len() != 1 {
            return Err("workflow_plan_patch_provider_execution_ambiguous".to_string());
        }
        let execution = eligible_executions[0];
        if !result_transition
            .causal_refs
            .iter()
            .any(|value| value == invocation_id)
        {
            return Err("workflow_plan_patch_provider_invocation_mismatch".to_string());
        }
        if let Some(existing) = state.workflow_plan_patches.iter().find(|patch| {
            matches!(
                &patch.origin,
                WorkflowPlanPatchOrigin::ModelProviderResult {
                    provider_result_id: existing_result_id,
                    workflow_execution_id: existing_execution_id,
                } if existing_result_id == provider_result_id
                    && existing_execution_id == &execution.execution_id
            )
        }) {
            let transition = history
                .into_iter()
                .find(|transition| {
                    matches!(
                        &transition.payload,
                        TransitionPayload::WorkflowPlanPatchProposed { patch }
                            if patch.patch_id == existing.patch_id
                    )
                })
                .ok_or_else(|| "workflow_plan_patch_transition_missing".to_string())?;
            return Ok(CanonicalCommit { transition, state });
        }
        if state.workflow_satisfactions.iter().any(|satisfaction| {
            satisfaction.execution_id.as_deref() == Some(execution.execution_id.as_str())
        }) {
            return Err("workflow_plan_patch_provider_execution_completed".to_string());
        }
        let node = topology
            .node(&execution.node_id)
            .map(|value| &value.node)
            .ok_or_else(|| "workflow_node_not_found".to_string())?;
        if !matches!(
            node.kind,
            WorkflowNodeKind::ModelWork {
                output_contract: crate::workflow::ModelWorkOutputContract::PlanPatch,
                ..
            }
        ) {
            return Err("workflow_model_output_contract_is_not_plan_patch".to_string());
        }
        let output = match &result_transition.payload {
            TransitionPayload::ProviderResultRecorded { output, .. } => output,
            _ => unreachable!(),
        };
        let strict = crate::governance::parse_strict_json(output.as_bytes())
            .map_err(|error| format!("workflow_model_plan_patch_invalid: {error}"))?;
        let input: WorkflowPlanPatchInput = serde_json::from_value(strict)
            .map_err(|error| format!("workflow_model_plan_patch_invalid: {error}"))?;
        let commit = self.propose_workflow_plan_patch_txn(
            &mut txn,
            &state,
            &context,
            input,
            WorkflowPlanPatchOrigin::ModelProviderResult {
                provider_result_id: provider_result_id.to_string(),
                workflow_execution_id: execution.execution_id.clone(),
            },
            now_unix_ms,
        )?;
        txn.commit()
            .map_err(|error| format!("failed to commit model Workflow PlanPatch: {error}"))?;
        Ok(commit)
    }

    fn propose_workflow_plan_patch_txn(
        &self,
        txn: &mut RwTransaction<'_>,
        state: &CaseState,
        context: &SecurityContext,
        input: WorkflowPlanPatchInput,
        origin: WorkflowPlanPatchOrigin,
        now_unix_ms: u64,
    ) -> Result<CanonicalCommit, String> {
        input.validate()?;
        if let Some(existing) = state.workflow_plan_patches.iter().find(|patch| {
            patch.base_effective_topology_digest == input.base_effective_topology_digest
                && patch.operations == input.operations
                && patch.origin == origin
        }) {
            let transition = self
                .list_case_transitions_txn(txn, &state.case_id)?
                .into_iter()
                .find(|transition| {
                    matches!(
                        &transition.payload,
                        TransitionPayload::WorkflowPlanPatchProposed { patch }
                            if patch.patch_id == existing.patch_id
                    )
                })
                .ok_or_else(|| "workflow_plan_patch_transition_missing".to_string())?;
            return Ok(CanonicalCommit {
                transition,
                state: state.clone(),
            });
        }
        if state.lifecycle == CaseLifecycle::Closed || state.cancellation.is_some() {
            return Err("workflow_patch_case_terminal".to_string());
        }
        let binding = state
            .workflow_binding
            .as_ref()
            .ok_or_else(|| "case_workflow_not_bound".to_string())?;
        let definition = self
            .workflow_definition_txn(txn, &binding.workflow_definition_id)?
            .ok_or_else(|| "bound_workflow_definition_missing".to_string())?;
        let definitions = self.workflow_definition_graph_for_operations_txn(
            txn,
            &definition,
            &state.workflow_amendments,
            &[],
        )?;
        let topology = derive_effective_workflow_topology(
            &definition,
            binding,
            &state.workflow_amendments,
            &definitions,
        )?;
        if input.base_effective_topology_digest != topology.topology_digest {
            return Err("workflow_patch_stale".to_string());
        }
        let patch = WorkflowPlanPatch::build(
            input,
            context.tenant_id(),
            &state.case_id,
            binding,
            state
                .workflow_amendments
                .last()
                .map(|value| value.amendment_id.clone()),
            topology.revision,
            origin,
            state.generation + 1,
            now_unix_ms,
        )?;
        let mut causal_refs = vec![binding.binding_id.clone()];
        match &patch.origin {
            WorkflowPlanPatchOrigin::AuthenticatedHuman { principal_id } => {
                causal_refs.push(principal_id.clone())
            }
            WorkflowPlanPatchOrigin::ModelProviderResult {
                provider_result_id,
                workflow_execution_id,
            } => {
                causal_refs.extend([provider_result_id.clone(), workflow_execution_id.clone()]);
            }
        }
        let pending = PendingTransition {
            transition_id: format!("transition:workflow-patch:{}", patch.patch_id),
            case_id: state.case_id.clone(),
            expected_generation: state.generation,
            source: TransitionSource {
                component: "yai.workflow".to_string(),
                participant_id: None,
                principal_id: Some(context.principal_id().to_string()),
                source_ref: Some(patch.patch_id.clone()),
            },
            scope: None,
            causal_refs,
            payload: TransitionPayload::WorkflowPlanPatchProposed { patch },
            provenance: Vec::new(),
            summary: None,
        };
        self.commit_transition_txn_at(txn, pending, false, None, Some(context))
    }

    pub fn validate_workflow_plan_patch_authorized(
        &self,
        authenticated: &AuthenticatedPrincipal,
        case_id: &str,
        patch_id: &str,
    ) -> Result<crate::workflow::EffectiveWorkflowTopology, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start Workflow PlanPatch validation: {error}"))?;
        let state = self
            .get_case_state_txn(&txn, case_id)?
            .ok_or_else(|| "case_not_visible".to_string())?;
        let tenant_id = state
            .tenant_id
            .as_deref()
            .ok_or_else(|| "workflow_patch_requires_tenant_case".to_string())?;
        self.resolve_security_context_txn(&txn, authenticated, tenant_id)?;
        let binding = state
            .workflow_binding
            .as_ref()
            .ok_or_else(|| "case_workflow_not_bound".to_string())?;
        let definition = self
            .workflow_definition_txn(&txn, &binding.workflow_definition_id)?
            .ok_or_else(|| "bound_workflow_definition_missing".to_string())?;
        let patch = state
            .workflow_plan_patches
            .iter()
            .find(|patch| patch.patch_id == patch_id)
            .ok_or_else(|| "workflow_plan_patch_not_found".to_string())?;
        let definitions = self.workflow_definition_graph_for_operations_txn(
            &txn,
            &definition,
            &state.workflow_amendments,
            &patch.operations,
        )?;
        workflow_patch_frozen_history_barrier(&state, patch)?;
        preview_workflow_patch(
            &definition,
            binding,
            &state.workflow_amendments,
            patch,
            &definitions,
        )
    }

    pub fn adopt_workflow_plan_patch(
        &self,
        authenticated: &AuthenticatedPrincipal,
        case_id: &str,
        patch_id: &str,
        now_unix_ms: u64,
    ) -> Result<CanonicalCommit, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start Workflow amendment adoption: {error}"))?;
        let state = self
            .get_case_state_txn(&txn, case_id)?
            .ok_or_else(|| "case_not_visible".to_string())?;
        let tenant_id = state
            .tenant_id
            .as_deref()
            .ok_or_else(|| "workflow_patch_requires_tenant_case".to_string())?;
        let context = self.resolve_security_context_txn(&txn, authenticated, tenant_id)?;
        context.require_owner()?;
        let patch = state
            .workflow_plan_patches
            .iter()
            .find(|patch| patch.patch_id == patch_id)
            .ok_or_else(|| "workflow_plan_patch_not_found".to_string())?;
        if state
            .workflow_amendments
            .iter()
            .any(|amendment| amendment.patch_id == patch_id)
        {
            return Err("workflow_plan_patch_already_adopted".to_string());
        }
        if state.lifecycle == CaseLifecycle::Closed || state.cancellation.is_some() {
            return Err("workflow_patch_case_terminal".to_string());
        }
        if state.workflow_amendments.len() >= MAX_WORKFLOW_AMENDMENTS {
            return Err("workflow_amendment_count_bound_exceeded".to_string());
        }
        let binding = state
            .workflow_binding
            .as_ref()
            .ok_or_else(|| "case_workflow_not_bound".to_string())?;
        let definition = self
            .workflow_definition_txn(&txn, &binding.workflow_definition_id)?
            .ok_or_else(|| "bound_workflow_definition_missing".to_string())?;
        let mut definitions = self.workflow_definition_graph_for_operations_txn(
            &txn,
            &definition,
            &state.workflow_amendments,
            &[],
        )?;
        let history = self.list_case_transitions_txn(&txn, case_id)?;
        let resolution = resolve_workflow_with_definitions(
            &definition,
            binding,
            &state,
            &history,
            &definitions,
        )?;
        if resolution.completed {
            return Err("workflow_completed_cannot_be_amended".to_string());
        }
        if resolution.active_count != 0 {
            return Err("workflow_amendment_requires_quiescent_boundary".to_string());
        }
        definitions = self.workflow_definition_graph_for_operations_txn(
            &txn,
            &definition,
            &state.workflow_amendments,
            &patch.operations,
        )?;
        workflow_patch_frozen_history_barrier(&state, patch)?;
        let preview = preview_workflow_patch(
            &definition,
            binding,
            &state.workflow_amendments,
            patch,
            &definitions,
        )?;
        let amendment = WorkflowAmendment::build(
            patch,
            &preview.topology_digest,
            context.principal_id(),
            state.generation + 1,
            now_unix_ms,
        )?;
        let mut pending = PendingTransition::new(
            format!("transition:workflow-amendment:{}", amendment.amendment_id),
            case_id,
            state.generation,
            TransitionSource {
                component: "yai.workflow".to_string(),
                participant_id: None,
                principal_id: Some(context.principal_id().to_string()),
                source_ref: Some(amendment.amendment_id.clone()),
            },
            TransitionPayload::WorkflowAmendmentAdopted {
                amendment: amendment.clone(),
            },
        );
        pending.causal_refs = vec![patch.patch_id.clone()];
        let commit =
            self.commit_transition_txn_at(&mut txn, pending, false, None, Some(&context))?;
        txn.commit()
            .map_err(|error| format!("failed to commit Workflow amendment: {error}"))?;
        Ok(commit)
    }

    pub fn offer_case_handoff(
        &self,
        authenticated: &AuthenticatedPrincipal,
        source_case_id: &str,
        target_case_id: &str,
        request: HandoffData,
        required_target_roles: Vec<String>,
        now_unix_ms: u64,
    ) -> Result<CanonicalCommit, String> {
        self.offer_case_handoff_for_node(
            authenticated,
            source_case_id,
            target_case_id,
            "case-handoff:manual",
            "manual",
            request,
            required_target_roles,
            now_unix_ms,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn offer_case_handoff_for_node(
        &self,
        authenticated: &AuthenticatedPrincipal,
        source_case_id: &str,
        target_case_id: &str,
        source_binding_id: &str,
        source_node_id: &str,
        request: HandoffData,
        required_target_roles: Vec<String>,
        now_unix_ms: u64,
    ) -> Result<CanonicalCommit, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start HandoffOffer: {error}"))?;
        let source = self
            .get_case_state_txn(&txn, source_case_id)?
            .ok_or_else(|| "case_not_visible".to_string())?;
        let tenant_id = source
            .tenant_id
            .as_deref()
            .ok_or_else(|| "handoff_requires_tenant_case".to_string())?;
        let context = self.resolve_security_context_txn(&txn, authenticated, tenant_id)?;
        if source_binding_id == "case-handoff:manual" {
            context.require_owner()?;
        }
        let offer = HandoffOffer::build(
            tenant_id,
            source_case_id,
            target_case_id,
            source_binding_id,
            source_node_id,
            request,
            required_target_roles,
            context.principal_id(),
            source.generation + 1,
            now_unix_ms,
        )?;
        if let Some(existing) = source.handoff_offers.iter().find(|current| {
            current.source_binding_id == source_binding_id
                && current.source_node_id == source_node_id
        }) {
            if existing == &offer {
                let transition = self
                    .list_case_transitions_txn(&txn, source_case_id)?
                    .into_iter()
                    .find(|transition| {
                        matches!(
                            &transition.payload,
                            TransitionPayload::HandoffOffered { offer }
                                if offer.handoff_id == existing.handoff_id
                        )
                    })
                    .ok_or_else(|| "handoff_offer_transition_missing".to_string())?;
                return Ok(CanonicalCommit {
                    transition,
                    state: source,
                });
            }
            return Err("handoff_offer_already_exists".to_string());
        }
        let mut pending = PendingTransition::new(
            format!("transition:handoff-offer:{}", offer.handoff_id),
            source_case_id,
            source.generation,
            TransitionSource {
                component: "yai.handoff".to_string(),
                participant_id: None,
                principal_id: Some(context.principal_id().to_string()),
                source_ref: Some(offer.handoff_id.clone()),
            },
            TransitionPayload::HandoffOffered {
                offer: offer.clone(),
            },
        );
        pending.causal_refs = vec![source_binding_id.to_string(), source_node_id.to_string()];
        let commit =
            self.commit_transition_txn_at(&mut txn, pending, false, None, Some(&context))?;
        txn.commit()
            .map_err(|error| format!("failed to commit HandoffOffer: {error}"))?;
        Ok(commit)
    }

    pub fn accept_case_handoff(
        &self,
        authenticated: &AuthenticatedPrincipal,
        target_case_id: &str,
        source_case_id: &str,
        handoff_id: &str,
        participant_id: &str,
        now_unix_ms: u64,
    ) -> Result<CanonicalCommit, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start HandoffAcceptance: {error}"))?;
        let target = self
            .get_case_state_txn(&txn, target_case_id)?
            .ok_or_else(|| "case_not_visible".to_string())?;
        let tenant_id = target
            .tenant_id
            .as_deref()
            .ok_or_else(|| "handoff_requires_tenant_case".to_string())?;
        let context = self.resolve_security_context_txn(&txn, authenticated, tenant_id)?;
        let source = self
            .get_case_state_txn(&txn, source_case_id)?
            .ok_or_else(|| "handoff_source_case_missing".to_string())?;
        let offer = source
            .handoff_offers
            .iter()
            .find(|offer| offer.handoff_id == handoff_id)
            .ok_or_else(|| "handoff_offer_not_found".to_string())?;
        let acceptance = HandoffAcceptance::build(
            offer,
            context.principal_id(),
            participant_id,
            target.generation + 1,
            now_unix_ms,
        )?;
        let mut pending = PendingTransition::new(
            format!("transition:handoff-acceptance:{}", acceptance.acceptance_id),
            target_case_id,
            target.generation,
            TransitionSource {
                component: "yai.handoff".to_string(),
                participant_id: Some(participant_id.to_string()),
                principal_id: Some(context.principal_id().to_string()),
                source_ref: Some(acceptance.acceptance_id.clone()),
            },
            TransitionPayload::HandoffAccepted {
                acceptance: acceptance.clone(),
            },
        );
        pending.causal_refs = vec![handoff_id.to_string()];
        let commit =
            self.commit_transition_txn_at(&mut txn, pending, false, None, Some(&context))?;
        txn.commit()
            .map_err(|error| format!("failed to commit HandoffAcceptance: {error}"))?;
        Ok(commit)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn decline_case_handoff(
        &self,
        authenticated: &AuthenticatedPrincipal,
        target_case_id: &str,
        source_case_id: &str,
        handoff_id: &str,
        participant_id: &str,
        reason: &str,
        now_unix_ms: u64,
    ) -> Result<CanonicalCommit, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start HandoffDecline: {error}"))?;
        let target = self
            .get_case_state_txn(&txn, target_case_id)?
            .ok_or_else(|| "case_not_visible".to_string())?;
        let tenant_id = target
            .tenant_id
            .as_deref()
            .ok_or_else(|| "handoff_requires_tenant_case".to_string())?;
        let context = self.resolve_security_context_txn(&txn, authenticated, tenant_id)?;
        let source = self
            .get_case_state_txn(&txn, source_case_id)?
            .ok_or_else(|| "handoff_source_case_missing".to_string())?;
        let offer = source
            .handoff_offers
            .iter()
            .find(|offer| offer.handoff_id == handoff_id)
            .ok_or_else(|| "handoff_offer_not_found".to_string())?;
        let decline = HandoffDecline::build(
            offer,
            context.principal_id(),
            participant_id,
            reason,
            target.generation + 1,
            now_unix_ms,
        )?;
        let mut pending = PendingTransition::new(
            format!("transition:handoff-decline:{}", decline.decline_id),
            target_case_id,
            target.generation,
            TransitionSource {
                component: "yai.handoff".to_string(),
                participant_id: Some(participant_id.to_string()),
                principal_id: Some(context.principal_id().to_string()),
                source_ref: Some(decline.decline_id.clone()),
            },
            TransitionPayload::HandoffDeclined {
                decline: decline.clone(),
            },
        );
        pending.causal_refs = vec![handoff_id.to_string()];
        let commit =
            self.commit_transition_txn_at(&mut txn, pending, false, None, Some(&context))?;
        txn.commit()
            .map_err(|error| format!("failed to commit HandoffDecline: {error}"))?;
        Ok(commit)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_case_handoff_result(
        &self,
        authenticated: &AuthenticatedPrincipal,
        target_case_id: &str,
        handoff_id: &str,
        outcome: HandoffOutcome,
        result: HandoffData,
        evidence_refs: Vec<String>,
        participant_id: &str,
        now_unix_ms: u64,
    ) -> Result<CanonicalCommit, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start HandoffResult: {error}"))?;
        let target = self
            .get_case_state_txn(&txn, target_case_id)?
            .ok_or_else(|| "case_not_visible".to_string())?;
        let tenant_id = target
            .tenant_id
            .as_deref()
            .ok_or_else(|| "handoff_requires_tenant_case".to_string())?;
        let context = self.resolve_security_context_txn(&txn, authenticated, tenant_id)?;
        let acceptance = target
            .handoff_acceptances
            .iter()
            .find(|acceptance| acceptance.handoff_id == handoff_id)
            .ok_or_else(|| "handoff_acceptance_not_found".to_string())?;
        result.validate()?;
        let mut evidence_refs = evidence_refs;
        evidence_refs.sort();
        evidence_refs.dedup();
        if let Some(existing) = target
            .handoff_results
            .iter()
            .find(|candidate| candidate.handoff_id == handoff_id)
        {
            if existing.outcome == outcome
                && existing.result == result
                && existing.evidence_refs == evidence_refs
                && existing.recorded_by_principal_id == context.principal_id()
                && existing.recorded_by_participant_id == participant_id
            {
                let transition = self
                    .list_case_transitions_txn(&txn, target_case_id)?
                    .into_iter()
                    .find(|transition| {
                        matches!(
                            &transition.payload,
                            TransitionPayload::HandoffResultRecorded { result }
                                if result.result_id == existing.result_id
                        )
                    })
                    .ok_or_else(|| "handoff_result_transition_missing".to_string())?;
                return Ok(CanonicalCommit {
                    transition,
                    state: target,
                });
            }
            return Err("handoff_result_already_terminal".to_string());
        }
        if target.lifecycle == CaseLifecycle::Closed || target.cancellation.is_some() {
            return Err("handoff_result_target_case_terminal".to_string());
        }
        self.validate_handoff_evidence_refs_txn(&txn, target_case_id, &evidence_refs)?;
        let result = HandoffResult::build(
            acceptance,
            outcome,
            result,
            evidence_refs,
            context.principal_id(),
            participant_id,
            target.generation + 1,
            now_unix_ms,
        )?;
        let mut pending = PendingTransition::new(
            format!("transition:handoff-result:{}", result.result_id),
            target_case_id,
            target.generation,
            TransitionSource {
                component: "yai.handoff".to_string(),
                participant_id: Some(participant_id.to_string()),
                principal_id: Some(context.principal_id().to_string()),
                source_ref: Some(result.result_id.clone()),
            },
            TransitionPayload::HandoffResultRecorded {
                result: result.clone(),
            },
        );
        pending.causal_refs = std::iter::once(result.acceptance_id.clone())
            .chain(result.evidence_refs.iter().cloned())
            .collect();
        let commit =
            self.commit_transition_txn_at(&mut txn, pending, false, None, Some(&context))?;
        txn.commit()
            .map_err(|error| format!("failed to commit HandoffResult: {error}"))?;
        Ok(commit)
    }

    pub fn reconcile_case_handoff(
        &self,
        authenticated: &AuthenticatedPrincipal,
        source_case_id: &str,
        handoff_id: &str,
        now_unix_ms: u64,
    ) -> Result<CanonicalCommit, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start HandoffReconciliation: {error}"))?;
        let source = self
            .get_case_state_txn(&txn, source_case_id)?
            .ok_or_else(|| "case_not_visible".to_string())?;
        let tenant_id = source
            .tenant_id
            .as_deref()
            .ok_or_else(|| "handoff_requires_tenant_case".to_string())?;
        let context = self.resolve_security_context_txn(&txn, authenticated, tenant_id)?;
        context.require_owner()?;
        if let Some(existing) = source
            .handoff_reconciliations
            .iter()
            .find(|value| value.handoff_id == handoff_id)
        {
            let transition = self
                .list_case_transitions_txn(&txn, source_case_id)?
                .into_iter()
                .find(|transition| {
                    matches!(
                        &transition.payload,
                        TransitionPayload::HandoffReconciled { reconciliation }
                            if reconciliation.reconciliation_id == existing.reconciliation_id
                    )
                })
                .ok_or_else(|| "handoff_reconciliation_transition_missing".to_string())?;
            return Ok(CanonicalCommit {
                transition,
                state: source,
            });
        }
        let offer = source
            .handoff_offers
            .iter()
            .find(|offer| offer.handoff_id == handoff_id)
            .ok_or_else(|| "handoff_offer_not_found".to_string())?;
        let target = self
            .get_case_state_txn(&txn, &offer.target_case_id)?
            .ok_or_else(|| "handoff_target_case_missing".to_string())?;
        let reconciliation = if let Some(result) = target
            .handoff_results
            .iter()
            .find(|result| result.handoff_id == handoff_id)
        {
            HandoffReconciliation::build(
                result,
                context.principal_id(),
                source.generation + 1,
                now_unix_ms,
            )?
        } else if let Some(decline) = target
            .handoff_declines
            .iter()
            .find(|decline| decline.handoff_id == handoff_id)
        {
            HandoffReconciliation::build_declined(
                decline,
                context.principal_id(),
                source.generation + 1,
                now_unix_ms,
            )?
        } else if let Some(cancellation) = &target.cancellation {
            let acceptance_id = target
                .handoff_acceptances
                .iter()
                .find(|value| value.handoff_id == handoff_id)
                .map(|value| value.acceptance_id.clone());
            HandoffReconciliation::build_target_terminal(
                handoff_id,
                acceptance_id,
                &cancellation.transition_id,
                HandoffOutcome::Cancelled,
                context.principal_id(),
                source.generation + 1,
                now_unix_ms,
            )?
        } else if let Some(closure) = &target.closure {
            let acceptance_id = target
                .handoff_acceptances
                .iter()
                .find(|value| value.handoff_id == handoff_id)
                .map(|value| value.acceptance_id.clone());
            HandoffReconciliation::build_target_terminal(
                handoff_id,
                acceptance_id,
                &closure.transition_id,
                HandoffOutcome::Failed,
                context.principal_id(),
                source.generation + 1,
                now_unix_ms,
            )?
        } else {
            return Err("handoff_target_disposition_missing".to_string());
        };
        let mut pending = PendingTransition::new(
            format!(
                "transition:handoff-reconciliation:{}",
                reconciliation.reconciliation_id
            ),
            source_case_id,
            source.generation,
            TransitionSource {
                component: "yai.handoff".to_string(),
                participant_id: None,
                principal_id: Some(context.principal_id().to_string()),
                source_ref: Some(reconciliation.reconciliation_id.clone()),
            },
            TransitionPayload::HandoffReconciled {
                reconciliation: reconciliation.clone(),
            },
        );
        pending.causal_refs = vec![reconciliation.target_disposition_id().to_string()];
        let commit =
            self.commit_transition_txn_at(&mut txn, pending, false, None, Some(&context))?;
        txn.commit()
            .map_err(|error| format!("failed to commit HandoffReconciliation: {error}"))?;
        Ok(commit)
    }

    pub fn list_pending_case_handoffs_authorized(
        &self,
        authenticated: &AuthenticatedPrincipal,
        target_case_id: &str,
    ) -> Result<Vec<HandoffOffer>, String> {
        let target = self.get_case_state_authorized(authenticated, target_case_id)?;
        let tenant_id = target
            .tenant_id
            .as_deref()
            .ok_or_else(|| "handoff_requires_tenant_case".to_string())?;
        let cases = self.list_case_states_authorized(authenticated, Some(tenant_id), 1024)?;
        let terminal = target
            .handoff_acceptances
            .iter()
            .map(|value| value.handoff_id.clone())
            .chain(
                target
                    .handoff_declines
                    .iter()
                    .map(|value| value.handoff_id.clone()),
            )
            .collect::<BTreeSet<_>>();
        let mut offers = cases
            .into_iter()
            .flat_map(|state| state.handoff_offers)
            .filter(|offer| offer.target_case_id == target_case_id)
            .filter(|offer| !terminal.contains(&offer.handoff_id))
            .collect::<Vec<_>>();
        offers.sort_by(|left, right| left.handoff_id.cmp(&right.handoff_id));
        Ok(offers)
    }

    pub fn record_workflow_human_input(
        &self,
        authenticated: &AuthenticatedPrincipal,
        case_id: &str,
        node_id: &str,
        value: &str,
        now_unix_ms: u64,
    ) -> Result<CanonicalCommit, String> {
        if value.is_empty() || value.len() > MAX_WORKFLOW_INPUT_BYTES {
            return Err("workflow_human_input_bounds_invalid".to_string());
        }
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start workflow input write: {error}"))?;
        let state = self
            .get_case_state_txn(&txn, case_id)?
            .ok_or_else(|| "case_not_visible".to_string())?;
        let tenant_id = state
            .tenant_id
            .as_deref()
            .ok_or_else(|| "workflow_requires_tenant_case".to_string())?;
        let context = self.resolve_security_context_txn(&txn, authenticated, tenant_id)?;
        let binding = state
            .workflow_binding
            .as_ref()
            .ok_or_else(|| "case_workflow_not_bound".to_string())?;
        let definition = self
            .workflow_definition_txn(&txn, &binding.workflow_definition_id)?
            .ok_or_else(|| "bound_workflow_definition_missing".to_string())?;
        let definitions = self.workflow_definition_graph_for_operations_txn(
            &txn,
            &definition,
            &state.workflow_amendments,
            &[],
        )?;
        let topology = derive_effective_workflow_topology(
            &definition,
            binding,
            &state.workflow_amendments,
            &definitions,
        )?;
        let history = self.list_case_transitions_txn(&txn, case_id)?;
        let resolution = resolve_workflow_with_definitions(
            &definition,
            binding,
            &state,
            &history,
            &definitions,
        )?;
        let node_view = resolution
            .nodes
            .iter()
            .find(|node| node.node_id == node_id)
            .ok_or_else(|| "workflow_node_not_found".to_string())?;
        if node_view.posture != WorkflowNodePosture::WaitingHumanInput {
            return Err("workflow_human_input_node_not_ready".to_string());
        }
        let node = topology
            .node(node_id)
            .map(|value| &value.node)
            .ok_or_else(|| "workflow_node_not_found".to_string())?;
        let WorkflowNodeKind::HumanInput {
            actor_slot,
            required_roles,
            input_kind,
            max_bytes,
            ..
        } = &node.kind
        else {
            return Err("workflow_node_is_not_human_input".to_string());
        };
        if value.len() > *max_bytes {
            return Err("workflow_human_input_bounds_invalid".to_string());
        }
        if *input_kind == HumanInputKind::Json {
            let parsed: serde_json::Value = serde_json::from_str(value)
                .map_err(|_| "workflow_human_input_json_invalid".to_string())?;
            if parsed.is_array() || parsed.is_null() {
                return Err("workflow_human_input_json_kind_invalid".to_string());
            }
        }
        let participant_id = binding
            .participant_for_slot(actor_slot)
            .ok_or_else(|| "workflow_human_actor_slot_unbound".to_string())?;
        let linked = state.principal_participant_links.iter().any(|link| {
            link.principal_id == context.principal_id()
                && link.participant_id == participant_id
                && link.tenant_id == tenant_id
        });
        let participant = state
            .participants
            .iter()
            .find(|participant| participant.participant_id == participant_id)
            .ok_or_else(|| "workflow_human_participant_missing".to_string())?;
        if !linked
            || required_roles
                .iter()
                .any(|role| !participant.roles.contains(role))
        {
            return Err("workflow_human_input_principal_or_role_rejected".to_string());
        }
        let value_digest = crate::effect::digest_bytes(value.as_bytes());
        let identity = crate::context::stable_digest(&format!(
            "{}\0{}\0{}\0{}",
            binding.binding_id,
            node_id,
            context.principal_id(),
            value_digest
        ));
        let input = WorkflowHumanInputRecord {
            schema: WORKFLOW_HUMAN_INPUT_SCHEMA.to_string(),
            input_id: format!("workflow-input:{identity}"),
            binding_id: binding.binding_id.clone(),
            workflow_definition_id: definition.workflow_definition_id.clone(),
            node_id: node_id.to_string(),
            principal_id: context.principal_id().to_string(),
            participant_id: participant_id.to_string(),
            value: value.to_string(),
            value_digest,
            recorded_at_generation: state.generation + 1,
            recorded_at_unix_ms: now_unix_ms,
        };
        let mut pending = PendingTransition::new(
            format!("transition:{}", input.input_id),
            case_id,
            state.generation,
            TransitionSource {
                component: "yai.workflow".to_string(),
                participant_id: Some(participant_id.to_string()),
                principal_id: Some(context.principal_id().to_string()),
                source_ref: Some(input.input_id.clone()),
            },
            TransitionPayload::WorkflowHumanInputRecorded {
                input: input.clone(),
            },
        );
        pending.causal_refs = vec![binding.binding_id.clone(), node_id.to_string()];
        let commit =
            self.commit_transition_txn_at(&mut txn, pending, false, None, Some(&context))?;
        txn.commit()
            .map_err(|error| format!("failed to commit workflow human input: {error}"))?;
        Ok(commit)
    }

    pub fn advance_workflow_passive_progress(
        &self,
        authenticated: &AuthenticatedPrincipal,
        case_id: &str,
        max_steps: usize,
    ) -> Result<WorkflowResolution, String> {
        if max_steps == 0 || max_steps > crate::workflow::MAX_WORKFLOW_NODES {
            return Err("workflow_progress_bound_invalid".to_string());
        }
        for _ in 0..max_steps {
            let mut txn = self
                .env
                .begin_rw_txn()
                .map_err(|error| format!("failed to start workflow progression: {error}"))?;
            let state = self
                .get_case_state_txn(&txn, case_id)?
                .ok_or_else(|| "case_not_visible".to_string())?;
            let tenant_id = state
                .tenant_id
                .as_deref()
                .ok_or_else(|| "workflow_requires_tenant_case".to_string())?;
            let context = self.resolve_security_context_txn(&txn, authenticated, tenant_id)?;
            let binding = state
                .workflow_binding
                .as_ref()
                .ok_or_else(|| "case_workflow_not_bound".to_string())?;
            let definition = self
                .workflow_definition_txn(&txn, &binding.workflow_definition_id)?
                .ok_or_else(|| "bound_workflow_definition_missing".to_string())?;
            let definitions = self.workflow_definition_graph_for_operations_txn(
                &txn,
                &definition,
                &state.workflow_amendments,
                &[],
            )?;
            let topology = derive_effective_workflow_topology(
                &definition,
                binding,
                &state.workflow_amendments,
                &definitions,
            )?;
            let history = self.list_case_transitions_txn(&txn, case_id)?;
            let resolution = resolve_workflow_with_definitions(
                &definition,
                binding,
                &state,
                &history,
                &definitions,
            )?;
            let mut next: Option<PendingTransition> = None;
            for node_view in &resolution.nodes {
                let node = topology
                    .node(&node_view.node_id)
                    .map(|value| &value.node)
                    .ok_or_else(|| "workflow_node_not_found".to_string())?;
                match (&node.kind, &node_view.posture, node_view.reason.as_str()) {
                    (
                        WorkflowNodeKind::Condition { predicate },
                        WorkflowNodePosture::WaitingCondition,
                        _,
                    ) => {
                        let evaluation = evaluate_predicate(
                            &definition,
                            binding,
                            &state,
                            &history,
                            &node_view.node_id,
                            None,
                            predicate,
                        )?;
                        let predicate_digest = predicate.digest()?;
                        let fact = WorkflowConditionResolution {
                            schema: WORKFLOW_CONDITION_RESOLUTION_SCHEMA.to_string(),
                            resolution_id: workflow_fact_identity(
                                "condition",
                                &binding.binding_id,
                                &node_view.node_id,
                                &predicate_digest,
                                &evaluation.evidence_refs,
                            ),
                            binding_id: binding.binding_id.clone(),
                            workflow_definition_id: definition.workflow_definition_id.clone(),
                            node_id: node_view.node_id.clone(),
                            result: evaluation.value,
                            predicate_digest,
                            evaluated_at_generation: state.generation + 1,
                            evidence_refs: evaluation.evidence_refs,
                        };
                        let mut pending = PendingTransition::new(
                            format!("transition:{}", fact.resolution_id),
                            case_id,
                            state.generation,
                            workflow_transition_source(context.principal_id(), &fact.resolution_id),
                            TransitionPayload::WorkflowConditionResolved {
                                resolution: fact.clone(),
                            },
                        );
                        pending.causal_refs = vec![binding.binding_id.clone()];
                        pending.causal_refs.extend(fact.evidence_refs.clone());
                        next = Some(pending);
                    }
                    (
                        WorkflowNodeKind::Wait { predicate }
                        | WorkflowNodeKind::EffectGoal { predicate }
                        | WorkflowNodeKind::Handoff {
                            completion: predicate,
                            ..
                        },
                        WorkflowNodePosture::WaitingEffect,
                        "passive_predicate_satisfied_pending_commit",
                    ) => {
                        let evaluation = evaluate_predicate(
                            &definition,
                            binding,
                            &state,
                            &history,
                            &node_view.node_id,
                            None,
                            predicate,
                        )?;
                        next = Some(workflow_satisfaction_pending(
                            case_id,
                            &state,
                            binding,
                            &definition,
                            &node_view.node_id,
                            None,
                            predicate,
                            evaluation.evidence_refs,
                            context.principal_id(),
                        )?);
                    }
                    (
                        WorkflowNodeKind::Handoff {
                            target_case_slot,
                            request,
                            required_target_roles,
                            ..
                        },
                        WorkflowNodePosture::WaitingEffect,
                        "passive_predicate_not_satisfied",
                    ) if !state
                        .handoff_offers
                        .iter()
                        .any(|offer| offer.source_node_id == node_view.node_id) =>
                    {
                        let target_case_id = binding
                            .case_for_slot(target_case_slot)
                            .ok_or_else(|| "workflow_handoff_case_slot_unbound".to_string())?;
                        let offer = HandoffOffer::build(
                            tenant_id,
                            case_id,
                            target_case_id,
                            &binding.binding_id,
                            &node_view.node_id,
                            request.clone(),
                            required_target_roles.clone(),
                            context.principal_id(),
                            state.generation + 1,
                            u64::try_from(unix_time_ms()).unwrap_or(u64::MAX),
                        )?;
                        let mut pending = PendingTransition::new(
                            format!("transition:handoff-offer:{}", offer.handoff_id),
                            case_id,
                            state.generation,
                            workflow_transition_source(context.principal_id(), &offer.handoff_id),
                            TransitionPayload::HandoffOffered {
                                offer: offer.clone(),
                            },
                        );
                        pending.causal_refs =
                            vec![binding.binding_id.clone(), node_view.node_id.clone()];
                        next = Some(pending);
                    }
                    (
                        WorkflowNodeKind::Subflow { .. },
                        WorkflowNodePosture::WaitingEffect,
                        "subflow_children_complete_pending_satisfaction",
                    ) => {
                        let evidence_refs = workflow_subflow_completion_evidence(
                            &topology,
                            &state,
                            &node_view.node_id,
                        )?;
                        next = Some(workflow_subflow_satisfaction_pending(
                            case_id,
                            &state,
                            binding,
                            &definition,
                            &node_view.node_id,
                            evidence_refs,
                            context.principal_id(),
                        )?);
                    }
                    (
                        WorkflowNodeKind::ModelWork {
                            completion: predicate,
                            ..
                        }
                        | WorkflowNodeKind::DeterministicWork {
                            completion: predicate,
                            ..
                        },
                        WorkflowNodePosture::Active,
                        "completion_proven_pending_canonical_satisfaction",
                    ) => {
                        let execution_id = node_view
                            .execution_id
                            .as_deref()
                            .ok_or_else(|| "workflow_active_node_execution_missing".to_string())?;
                        let evaluation = evaluate_predicate(
                            &definition,
                            binding,
                            &state,
                            &history,
                            &node_view.node_id,
                            Some(execution_id),
                            predicate,
                        )?;
                        next = Some(workflow_satisfaction_pending(
                            case_id,
                            &state,
                            binding,
                            &definition,
                            &node_view.node_id,
                            Some(execution_id),
                            predicate,
                            evaluation.evidence_refs,
                            context.principal_id(),
                        )?);
                    }
                    _ => {}
                }
                if next.is_some() {
                    break;
                }
            }
            let Some(pending) = next else {
                drop(txn);
                return self.workflow_status_authorized(authenticated, case_id);
            };
            self.commit_transition_txn_at(&mut txn, pending, false, None, Some(&context))?;
            txn.commit()
                .map_err(|error| format!("failed to commit workflow progression: {error}"))?;
        }
        self.workflow_status_authorized(authenticated, case_id)
    }

    pub fn list_workflow_case_ids_authorized(
        &self,
        authenticated: &AuthenticatedPrincipal,
        limit: usize,
    ) -> Result<Vec<String>, String> {
        if limit == 0 || limit > 1024 {
            return Err("workflow_case_scan_bound_invalid".to_string());
        }
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start workflow Case scan: {error}"))?;
        self.authenticated_principal_txn(&txn, authenticated)?;
        let mut cursor = txn
            .open_ro_cursor(self.case_state)
            .map_err(|error| format!("failed to open CaseState cursor: {error}"))?;
        let mut case_ids = Vec::new();
        for (_, value) in cursor.iter() {
            let json = std::str::from_utf8(value)
                .map_err(|error| format!("invalid CaseState utf8: {error}"))?;
            let state = CaseState::from_json(json)?;
            let Some(tenant_id) = state.tenant_id.as_deref() else {
                continue;
            };
            if state.workflow_binding.is_some()
                && state.lifecycle == CaseLifecycle::Open
                && state.cancellation.is_none()
                && self
                    .resolve_security_context_txn(&txn, authenticated, tenant_id)
                    .is_ok()
            {
                case_ids.push(state.case_id);
                if case_ids.len() == limit {
                    break;
                }
            }
        }
        case_ids.sort();
        Ok(case_ids)
    }

    /// Lists canonical CaseState materializations visible to the authenticated
    /// Principal. This is a bounded read projection for product inspection; it
    /// does not create a second Case index or source of truth.
    pub fn list_case_states_authorized(
        &self,
        authenticated: &AuthenticatedPrincipal,
        tenant_filter: Option<&str>,
        limit: usize,
    ) -> Result<Vec<CaseState>, String> {
        if limit == 0 || limit > 1024 {
            return Err("case_scan_bound_invalid".to_string());
        }
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start Case scan: {error}"))?;
        self.authenticated_principal_txn(&txn, authenticated)?;
        let mut cursor = txn
            .open_ro_cursor(self.case_state)
            .map_err(|error| format!("failed to open CaseState cursor: {error}"))?;
        let mut states = Vec::new();
        for (_, value) in cursor.iter() {
            let json = std::str::from_utf8(value)
                .map_err(|error| format!("invalid CaseState utf8: {error}"))?;
            let state = CaseState::from_json(json)?;
            let Some(tenant_id) = state.tenant_id.as_deref() else {
                continue;
            };
            if tenant_filter.is_some_and(|filter| filter != tenant_id) {
                continue;
            }
            if self
                .resolve_security_context_txn(&txn, authenticated, tenant_id)
                .is_ok()
            {
                states.push(state);
            }
        }
        states.sort_by(|left, right| left.case_id.cmp(&right.case_id));
        states.truncate(limit);
        Ok(states)
    }

    pub fn materialize_workflow_ready_work(
        &self,
        authenticated: &AuthenticatedPrincipal,
        runtime_owner_token: &str,
        case_id: &str,
        journal_path: &str,
        work_failpoint: Option<&str>,
        now_unix_ms: u64,
    ) -> Result<Option<RuntimeWorkSubmissionOutcome>, String> {
        if journal_path.is_empty() || journal_path.len() > MAX_RUNTIME_WORK_JOURNAL_PATH_BYTES {
            return Err("workflow_runtime_journal_path_invalid".to_string());
        }
        if work_failpoint.is_some_and(|value| value.is_empty() || value.len() > 128) {
            return Err("workflow_runtime_failpoint_invalid".to_string());
        }
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start workflow ReadyWork write: {error}"))?;
        let principal = self.authenticated_principal_txn(&txn, authenticated)?;
        let instance = runtime_instance_owner_txn(
            &txn,
            self.runtime_instances,
            &principal.principal_id,
            runtime_owner_token,
            now_unix_ms,
        )?;
        if instance.lifecycle != RuntimeInstanceLifecycle::Running
            || instance.drain_requested_at_unix_ms.is_some()
        {
            return Ok(None);
        }
        let state = self
            .get_case_state_txn(&txn, case_id)?
            .ok_or_else(|| "case_not_visible".to_string())?;
        let tenant_id = state
            .tenant_id
            .as_deref()
            .ok_or_else(|| "workflow_requires_tenant_case".to_string())?;
        let context = self.resolve_security_context_txn(&txn, authenticated, tenant_id)?;
        let binding = state
            .workflow_binding
            .as_ref()
            .ok_or_else(|| "case_workflow_not_bound".to_string())?;
        let definition = self
            .workflow_definition_txn(&txn, &binding.workflow_definition_id)?
            .ok_or_else(|| "bound_workflow_definition_missing".to_string())?;
        let definitions = self.workflow_definition_graph_for_operations_txn(
            &txn,
            &definition,
            &state.workflow_amendments,
            &[],
        )?;
        let topology = derive_effective_workflow_topology(
            &definition,
            binding,
            &state.workflow_amendments,
            &definitions,
        )?;
        let history = self.list_case_transitions_txn(&txn, case_id)?;
        let resolution = resolve_workflow_with_definitions(
            &definition,
            binding,
            &state,
            &history,
            &definitions,
        )?;
        let Some(ready) = resolution.ready_work.first() else {
            return Ok(None);
        };
        if resolution.active_count != 0 {
            return Ok(None);
        }
        let effective = topology
            .node(&ready.node_id)
            .ok_or_else(|| "workflow_node_not_found".to_string())?;
        let node = &effective.node;
        let execution_id = workflow_execution_identity(binding, &ready.node_id);
        let all = list_runtime_work_items_txn(&txn, self.runtime_work_items)?;
        if let Some(existing) = all.iter().find(|item| {
            item.workflow
                .as_ref()
                .is_some_and(|workflow| workflow.workflow_execution_id == execution_id)
        }) {
            return Ok(Some(RuntimeWorkSubmissionOutcome {
                item: existing.clone(),
                created: false,
            }));
        }
        let total_queued = all
            .iter()
            .filter(|item| item.state.is_queued_capacity())
            .count();
        if total_queued >= instance.config.max_queued_total {
            return Ok(None);
        }
        let tenant_queued = all
            .iter()
            .filter(|item| item.tenant_id == tenant_id && item.state.is_queued_capacity())
            .count();
        if tenant_queued >= instance.config.max_queued_per_tenant {
            return Ok(None);
        }

        let (participant_id, attachment_id, task, budgets, node_kind) = match &node.kind {
            WorkflowNodeKind::ModelWork {
                executor_slot,
                task,
                budgets,
                resource_slot,
                ..
            } => {
                let participant = binding
                    .participant_for_slot(executor_slot)
                    .ok_or_else(|| "workflow_model_executor_slot_unbound".to_string())?;
                let attachment = resource_slot
                    .as_ref()
                    .and_then(|slot| binding.attachment_for_slot(slot))
                    .or_else(|| {
                        state
                            .resources
                            .first()
                            .map(|resource| resource.attachment_id.as_str())
                    })
                    .ok_or_else(|| "workflow_modelwork_requires_case_resource".to_string())?;
                (
                    participant.to_string(),
                    attachment.to_string(),
                    task.clone(),
                    RuntimeCaseBudgets {
                        max_invocations: budgets.max_turns,
                        max_operations: budgets.max_operations,
                        max_semantic_units: budgets.max_semantic_units,
                        max_resident_items: 64,
                        max_estimated_input_units: budgets.max_semantic_units.saturating_mul(16),
                        max_provider_retries: 1,
                        max_runtime_ms: None,
                        stop_on_deny: false,
                        continue_after_malformed: false,
                    },
                    "model_work".to_string(),
                )
            }
            WorkflowNodeKind::DeterministicWork {
                proposer_slot,
                operation,
                ..
            } => (
                binding
                    .participant_for_slot(proposer_slot)
                    .ok_or_else(|| "workflow_deterministic_proposer_slot_unbound".to_string())?
                    .to_string(),
                binding
                    .attachment_for_slot(operation.resource_slot())
                    .ok_or_else(|| "workflow_deterministic_resource_slot_unbound".to_string())?
                    .to_string(),
                serde_json::to_string(operation).map_err(|error| {
                    format!("workflow_deterministic_template_encode_failed: {error}")
                })?,
                RuntimeCaseBudgets {
                    max_invocations: 1,
                    max_operations: 1,
                    max_semantic_units: 1,
                    max_resident_items: 1,
                    max_estimated_input_units: 1,
                    max_provider_retries: 0,
                    max_runtime_ms: None,
                    stop_on_deny: true,
                    continue_after_malformed: false,
                },
                "deterministic_work".to_string(),
            ),
            _ => return Err("workflow_ready_work_node_not_executable".to_string()),
        };
        budgets.validate()?;
        if !state
            .participants
            .iter()
            .any(|participant| participant.participant_id == participant_id)
            || !state
                .resources
                .iter()
                .any(|resource| resource.attachment_id == attachment_id)
        {
            return Err("workflow_ready_work_case_binding_mismatch".to_string());
        }

        let execution = WorkflowNodeExecution {
            schema: WORKFLOW_NODE_EXECUTION_SCHEMA.to_string(),
            execution_id: execution_id.clone(),
            binding_id: binding.binding_id.clone(),
            workflow_definition_id: definition.workflow_definition_id.clone(),
            node_id: ready.node_id.clone(),
            case_id: case_id.to_string(),
            started_at_generation: state.generation + 1,
            started_at_unix_ms: now_unix_ms,
        };
        let mut pending = PendingTransition::new(
            format!("transition:{}", execution.execution_id),
            case_id,
            state.generation,
            TransitionSource {
                component: "yai.workflow".to_string(),
                participant_id: Some(participant_id.clone()),
                principal_id: Some(context.principal_id().to_string()),
                source_ref: Some(execution.execution_id.clone()),
            },
            TransitionPayload::WorkflowNodeExecutionStarted {
                execution: execution.clone(),
            },
        );
        pending.causal_refs = vec![binding.binding_id.clone(), ready.node_id.clone()];

        let request_id = format!("workflow-request:{}", execution.execution_id);
        let work_id = format!(
            "runtime-work:{}",
            crate::context::stable_digest(&format!(
                "{}\0{}\0{}",
                principal.principal_id, tenant_id, request_id
            ))
        );
        let workflow = RuntimeWorkflowContext {
            workflow_binding_id: binding.binding_id.clone(),
            workflow_definition_id: definition.workflow_definition_id.clone(),
            workflow_node_id: ready.node_id.clone(),
            workflow_execution_id: execution.execution_id.clone(),
            workflow_node_kind: node_kind,
        };
        let request_digest = crate::context::stable_digest(
            &serde_json::to_string(&serde_json::json!({
                "schema": RUNTIME_WORK_ITEM_SCHEMA,
                "request_id": request_id,
                "principal_id": principal.principal_id,
                "tenant_id": tenant_id,
                "case_id": case_id,
                "participant_id": participant_id,
                "attachment_id": attachment_id,
                "task": task,
                "budgets": budgets,
                "workflow": workflow,
            }))
            .map_err(|error| format!("workflow WorkItem request encode failed: {error}"))?,
        );
        let sequence = next_runtime_work_sequence(&mut txn, self.schema_meta)?;
        let mut item = RuntimeWorkItem {
            schema: RUNTIME_WORK_ITEM_SCHEMA.to_string(),
            work_id: work_id.clone(),
            integrity_digest: String::new(),
            request_id: request_id.clone(),
            request_digest,
            principal_id: principal.principal_id.clone(),
            tenant_id: tenant_id.to_string(),
            case_id: case_id.to_string(),
            participant_id,
            attachment_id,
            journal_path: journal_path.to_string(),
            task,
            budgets,
            failpoint: work_failpoint.map(str::to_string),
            workflow: Some(workflow),
            enqueue_sequence: sequence,
            state: RuntimeWorkState::Queued,
            attempt_count: 0,
            runtime_instance_id: None,
            runtime_owner_token: None,
            worker_id: None,
            last_stop_reason: "workflow_ready_work_accepted".to_string(),
            enqueued_at_unix_ms: now_unix_ms,
            updated_at_unix_ms: now_unix_ms,
        };
        item.integrity_digest = runtime_work_integrity_digest(&item)?;
        item.validate_integrity()?;

        if work_failpoint == Some("workflow_before_start_commit") {
            return Err("workflow_failpoint_before_start_commit".to_string());
        }

        self.commit_transition_txn_at(&mut txn, pending, false, None, Some(&context))?;
        put_json_txn(
            &mut txn,
            self.runtime_work_items,
            &work_id,
            &item,
            WriteFlags::NO_OVERWRITE,
            "workflow runtime work item",
        )?;
        txn.put(
            self.runtime_work_idempotency,
            &runtime_work_idempotency_key(&principal.principal_id, tenant_id, &request_id),
            &work_id,
            WriteFlags::NO_OVERWRITE,
        )
        .map_err(|error| format!("failed to index workflow WorkItem: {error}"))?;
        txn.commit()
            .map_err(|error| format!("failed to commit workflow ReadyWork: {error}"))?;
        Ok(Some(RuntimeWorkSubmissionOutcome {
            item,
            created: true,
        }))
    }

    pub fn record_workflow_deterministic_proposal(
        &self,
        authenticated: &AuthenticatedPrincipal,
        item: &RuntimeWorkItem,
    ) -> Result<WorkflowDeterministicProposalRecord, String> {
        item.validate_integrity()?;
        let workflow = item
            .workflow
            .as_ref()
            .filter(|workflow| workflow.workflow_node_kind == "deterministic_work")
            .ok_or_else(|| "runtime_work_is_not_deterministic_workflow".to_string())?;

        let proposal = {
            let mut txn = self.env.begin_rw_txn().map_err(|error| {
                format!("failed to start deterministic workflow proposal: {error}")
            })?;
            let state = self
                .get_case_state_txn(&txn, &item.case_id)?
                .ok_or_else(|| "case_not_visible".to_string())?;
            let tenant_id = state
                .tenant_id
                .as_deref()
                .ok_or_else(|| "workflow_requires_tenant_case".to_string())?;
            let context = self.resolve_security_context_txn(&txn, authenticated, tenant_id)?;
            if item.principal_id != context.principal_id()
                || item.tenant_id != tenant_id
                || state.workflow_binding.as_ref().is_none_or(|binding| {
                    binding.binding_id != workflow.workflow_binding_id
                        || binding.workflow_definition_id != workflow.workflow_definition_id
                })
            {
                return Err("workflow_runtime_security_or_binding_mismatch".to_string());
            }
            let binding = state.workflow_binding.as_ref().expect("checked");
            let definition = self
                .workflow_definition_txn(&txn, &binding.workflow_definition_id)?
                .ok_or_else(|| "bound_workflow_definition_missing".to_string())?;
            let definitions = self.workflow_definition_graph_for_operations_txn(
                &txn,
                &definition,
                &state.workflow_amendments,
                &[],
            )?;
            let topology = derive_effective_workflow_topology(
                &definition,
                binding,
                &state.workflow_amendments,
                &definitions,
            )?;
            let node = topology
                .node(&workflow.workflow_node_id)
                .map(|value| &value.node)
                .ok_or_else(|| "workflow_node_not_found".to_string())?;
            let WorkflowNodeKind::DeterministicWork {
                proposer_slot,
                operation,
                ..
            } = &node.kind
            else {
                return Err("workflow_deterministic_node_kind_mismatch".to_string());
            };
            let execution_exists = state.workflow_executions.iter().any(|execution| {
                execution.execution_id == workflow.workflow_execution_id
                    && execution.node_id == workflow.workflow_node_id
            });
            if !execution_exists
                || binding.participant_for_slot(proposer_slot) != Some(item.participant_id.as_str())
                || binding.attachment_for_slot(operation.resource_slot())
                    != Some(item.attachment_id.as_str())
            {
                return Err("workflow_deterministic_execution_binding_mismatch".to_string());
            }
            let template_digest =
                crate::context::stable_digest(&serde_json::to_string(operation).map_err(
                    |error| format!("workflow_deterministic_template_encode_failed: {error}"),
                )?);
            let proposal_digest = crate::context::stable_digest(&format!(
                "{}\0{}\0{}\0{}",
                binding.binding_id,
                workflow.workflow_execution_id,
                workflow.workflow_node_id,
                template_digest
            ));
            let expected = WorkflowDeterministicProposalRecord {
                schema: crate::workflow::WORKFLOW_DETERMINISTIC_PROPOSAL_SCHEMA.to_string(),
                proposal_id: format!("workflow-proposal:{proposal_digest}"),
                binding_id: binding.binding_id.clone(),
                workflow_definition_id: definition.workflow_definition_id.clone(),
                node_id: workflow.workflow_node_id.clone(),
                execution_id: workflow.workflow_execution_id.clone(),
                participant_id: item.participant_id.clone(),
                operation_kind: operation.operation_kind(),
                resource_attachment_id: item.attachment_id.clone(),
                template_digest,
                recorded_at_generation: state.generation + 1,
            };
            if let Some(existing) = state
                .workflow_deterministic_proposals
                .iter()
                .find(|proposal| proposal.execution_id == workflow.workflow_execution_id)
            {
                let mut expected_existing = expected.clone();
                expected_existing.recorded_at_generation = existing.recorded_at_generation;
                if existing != &expected_existing {
                    return Err("workflow_deterministic_proposal_existing_mismatch".to_string());
                }
                drop(txn);
                existing.clone()
            } else {
                let mut pending = PendingTransition::new(
                    format!("transition:{}", expected.proposal_id),
                    &item.case_id,
                    state.generation,
                    TransitionSource {
                        component: "yai.workflow".to_string(),
                        participant_id: Some(item.participant_id.clone()),
                        principal_id: Some(context.principal_id().to_string()),
                        source_ref: Some(expected.proposal_id.clone()),
                    },
                    TransitionPayload::WorkflowDeterministicProposalRecorded {
                        proposal: expected.clone(),
                    },
                );
                pending.causal_refs = vec![workflow.workflow_execution_id.clone()];
                self.commit_transition_txn_at(&mut txn, pending, false, None, Some(&context))?;
                txn.commit().map_err(|error| {
                    format!("failed to commit deterministic workflow proposal: {error}")
                })?;
                expected
            }
        };
        Ok(proposal)
    }

    pub fn record_workflow_deterministic_operation(
        &self,
        authenticated: &AuthenticatedPrincipal,
        item: &RuntimeWorkItem,
    ) -> Result<(WorkflowDeterministicProposalRecord, Operation), String> {
        let proposal = self.record_workflow_deterministic_proposal(authenticated, item)?;
        let operation = self.record_workflow_deterministic_operation_from_proposal(
            authenticated,
            item,
            &proposal,
        )?;
        Ok((proposal, operation))
    }

    pub fn record_workflow_deterministic_operation_from_proposal(
        &self,
        authenticated: &AuthenticatedPrincipal,
        item: &RuntimeWorkItem,
        proposal: &WorkflowDeterministicProposalRecord,
    ) -> Result<Operation, String> {
        item.validate_integrity()?;
        let workflow = item
            .workflow
            .as_ref()
            .filter(|workflow| workflow.workflow_node_kind == "deterministic_work")
            .ok_or_else(|| "runtime_work_is_not_deterministic_workflow".to_string())?;
        if proposal.execution_id != workflow.workflow_execution_id
            || proposal.binding_id != workflow.workflow_binding_id
            || proposal.workflow_definition_id != workflow.workflow_definition_id
            || proposal.node_id != workflow.workflow_node_id
            || proposal.participant_id != item.participant_id
            || proposal.resource_attachment_id != item.attachment_id
        {
            return Err("workflow_deterministic_proposal_work_mismatch".to_string());
        }

        let state = self.get_case_state_authorized(authenticated, &item.case_id)?;
        let canonical_proposal = state
            .workflow_deterministic_proposals
            .iter()
            .find(|candidate| candidate.execution_id == workflow.workflow_execution_id)
            .ok_or_else(|| "workflow_deterministic_proposal_not_canonical".to_string())?;
        if canonical_proposal != proposal {
            return Err("workflow_deterministic_proposal_canonical_mismatch".to_string());
        }
        let history = self.list_case_transitions(&item.case_id)?;
        if let Some(operation) = history
            .iter()
            .find_map(|transition| match &transition.payload {
                TransitionPayload::OperationRecorded { operation }
                    if matches!(
                        &operation.origin,
                        OperationOrigin::WorkflowDeterministicProposal { proposal_id, .. }
                            if proposal_id == &proposal.proposal_id
                    ) =>
                {
                    Some(operation.clone())
                }
                _ => None,
            })
        {
            return Ok(operation);
        }
        let resource = state
            .resources
            .iter()
            .find(|resource| resource.attachment_id == item.attachment_id)
            .ok_or_else(|| "workflow_deterministic_resource_missing".to_string())?;
        let binding = state
            .workflow_binding
            .as_ref()
            .ok_or_else(|| "case_workflow_not_bound".to_string())?;
        let definition = self
            .get_workflow_definition_authorized(authenticated, &binding.workflow_definition_id)?;
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start workflow topology read: {error}"))?;
        let definitions = self.workflow_definition_graph_for_operations_txn(
            &txn,
            &definition,
            &state.workflow_amendments,
            &[],
        )?;
        let topology = derive_effective_workflow_topology(
            &definition,
            binding,
            &state.workflow_amendments,
            &definitions,
        )?;
        let node = topology
            .node(&workflow.workflow_node_id)
            .map(|value| &value.node)
            .ok_or_else(|| "workflow_node_not_found".to_string())?;
        let WorkflowNodeKind::DeterministicWork {
            operation: template,
            ..
        } = &node.kind
        else {
            return Err("workflow_deterministic_node_kind_mismatch".to_string());
        };
        let operation = match template {
            DeterministicOperationTemplate::FilesystemWrite {
                relative_path,
                content,
                ..
            } => build_workflow_deterministic_filesystem_operation(
                &item.case_id,
                &item.participant_id,
                state.generation,
                resource,
                relative_path,
                content,
                &proposal.proposal_id,
                &workflow.workflow_execution_id,
            )?,
            DeterministicOperationTemplate::ProcessSignal { action, .. } => {
                let process = self
                    .get_local_process_binding(&item.case_id, &item.attachment_id)?
                    .ok_or_else(|| "local_process_binding_missing".to_string())?;
                build_workflow_deterministic_process_operation(
                    &item.case_id,
                    &item.participant_id,
                    state.generation,
                    resource,
                    &process.process,
                    action.clone(),
                    &proposal.proposal_id,
                    &workflow.workflow_execution_id,
                )?
            }
        };
        let mut pending = PendingTransition::new(
            format!("transition:{}", operation.operation_id),
            &item.case_id,
            state.generation,
            TransitionSource {
                component: "yai.workflow".to_string(),
                participant_id: Some(item.participant_id.clone()),
                principal_id: Some(authenticated.projected_principal_id()),
                source_ref: Some(proposal.proposal_id.clone()),
            },
            TransitionPayload::OperationRecorded {
                operation: operation.clone(),
            },
        );
        pending.scope = Some(operation.scope.clone());
        pending.causal_refs = operation.origin.causal_refs();
        self.commit_secured_transition(authenticated, &item.tenant_id, pending, false)?;
        Ok(operation)
    }

    fn commit_transition_inner(
        &self,
        pending: PendingTransition,
        inject_failure_before_commit: bool,
    ) -> Result<CanonicalCommit, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start canonical write transaction: {error}"))?;
        let commit = self.commit_transition_txn(&mut txn, pending, inject_failure_before_commit)?;
        txn.commit()
            .map_err(|error| format!("failed to commit canonical transaction: {error}"))?;
        Ok(commit)
    }

    fn commit_transition_txn(
        &self,
        txn: &mut RwTransaction<'_>,
        pending: PendingTransition,
        inject_failure_before_commit: bool,
    ) -> Result<CanonicalCommit, String> {
        self.commit_transition_txn_at(txn, pending, inject_failure_before_commit, None, None)
    }

    fn commit_transition_txn_at(
        &self,
        txn: &mut RwTransaction<'_>,
        pending: PendingTransition,
        inject_failure_before_commit: bool,
        authority_time_unix_ms: Option<u64>,
        security_context: Option<&SecurityContext>,
    ) -> Result<CanonicalCommit, String> {
        self.commit_transition_txn_at_with_fence(
            txn,
            pending,
            inject_failure_before_commit,
            authority_time_unix_ms,
            security_context,
            None,
        )
    }

    fn commit_transition_txn_at_with_fence(
        &self,
        txn: &mut RwTransaction<'_>,
        pending: PendingTransition,
        inject_failure_before_commit: bool,
        authority_time_unix_ms: Option<u64>,
        security_context: Option<&SecurityContext>,
        resource_fence: Option<&ResourceFence>,
    ) -> Result<CanonicalCommit, String> {
        match (&pending.payload, resource_fence) {
            (TransitionPayload::EffectPrepared { prepared }, Some(fence))
                if prepared.schema == crate::effect::PREPARED_EFFECT_SCHEMA
                    && prepared.resource_fence.as_ref() == Some(fence) => {}
            (TransitionPayload::EffectPrepared { prepared }, None)
                if prepared.schema != crate::effect::PREPARED_EFFECT_SCHEMA => {}
            (TransitionPayload::ProcessEffectPrepared { prepared }, Some(fence))
                if prepared.schema == crate::effect::PREPARED_PROCESS_EFFECT_SCHEMA
                    && prepared.resource_fence.as_ref() == Some(fence) => {}
            (TransitionPayload::EffectPrepared { .. }, _)
            | (TransitionPayload::ProcessEffectPrepared { .. }, _) => {
                return Err(
                    "fenced_prepare_rejected_outside_resource_control_transaction".to_string(),
                )
            }
            (_, Some(_)) => return Err("resource_fence_context_requires_prepare".to_string()),
            (_, None) => {}
        }
        let transition_key = transition_id_key(&pending.transition_id);
        match txn.get(self.transitions_by_id, &transition_key) {
            Ok(_) => {
                return Err(format!(
                    "duplicate_transition_id: {}",
                    pending.transition_id
                ))
            }
            Err(Error::NotFound) => {}
            Err(error) => {
                return Err(format!(
                    "failed to check transition identity {}: {error}",
                    pending.transition_id
                ));
            }
        }

        let current_state = self.get_case_state_txn(txn, &pending.case_id)?;
        let actual_generation = current_state
            .as_ref()
            .map(|state| state.generation)
            .unwrap_or(0);
        if pending.expected_generation != actual_generation {
            return Err(format!(
                "stale_case_generation: expected={} actual={actual_generation}",
                pending.expected_generation
            ));
        }
        self.validate_case_security_write_txn(
            txn,
            current_state.as_ref(),
            &pending,
            security_context,
        )?;
        self.validate_workflow_progression_txn(
            txn,
            current_state.as_ref(),
            &pending,
            security_context,
        )?;
        let authority_time = match authority_time_unix_ms {
            Some(authority_time) => authority_time,
            None => self.advance_authority_time_txn(txn, authority_wall_time_unix_ms())?,
        };
        self.validate_policy_authority_txn(
            txn,
            current_state.as_ref(),
            &pending.case_id,
            &pending.payload,
            authority_time,
        )?;
        let sequence = actual_generation + 1;
        let transition = Transition {
            schema: TRANSITION_SCHEMA.to_string(),
            transition_id: pending.transition_id,
            case_id: pending.case_id,
            sequence,
            committed_at_unix_ms: authority_time,
            source: pending.source,
            scope: pending.scope,
            causal_refs: pending.causal_refs,
            payload: pending.payload,
            provenance: pending.provenance,
            summary: pending.summary,
        };
        transition.validate()?;

        let next_state = if let Some(state) = current_state {
            state.reduce(&transition)?
        } else {
            let lifecycle = match &transition.payload {
                crate::transition::TransitionPayload::CaseOpened { lifecycle }
                | crate::transition::TransitionPayload::TenantCaseOpened { lifecycle, .. } => {
                    lifecycle
                }
                _ => return Err("case_history_must_start_with_case_opened".to_string()),
            };
            CaseState::new(&transition.case_id, lifecycle.clone()).reduce(&transition)?
        };
        let transition_json = transition.to_json()?;
        let state_json = next_state.to_json()?;
        let sequence_key = case_sequence_key(&transition.case_id, transition.sequence);
        let state_key = case_state_key(&transition.case_id);

        txn.put(
            self.transitions_by_id,
            &transition_key,
            &transition_json,
            WriteFlags::NO_OVERWRITE,
        )
        .map_err(|error| format!("failed to append canonical Transition: {error}"))?;
        txn.put(
            self.case_transition_sequence,
            &sequence_key,
            &transition.transition_id,
            WriteFlags::NO_OVERWRITE,
        )
        .map_err(|error| format!("failed to append Case transition sequence: {error}"))?;
        txn.put(
            self.case_state,
            &state_key,
            &state_json,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to materialize CaseState: {error}"))?;

        if inject_failure_before_commit {
            return Err("injected_failure_before_canonical_commit".to_string());
        }
        Ok(CanonicalCommit {
            transition,
            state: next_state,
        })
    }

    fn validate_case_security_write_txn<T: Transaction>(
        &self,
        txn: &T,
        current_state: Option<&CaseState>,
        pending: &PendingTransition,
        security_context: Option<&SecurityContext>,
    ) -> Result<(), String> {
        if let TransitionPayload::TenantCaseOpened {
            tenant_id,
            principal_id,
            ..
        } = &pending.payload
        {
            if current_state.is_some() {
                return Err("tenant_case_open_requires_new_case".to_string());
            }
            let context = security_context
                .ok_or_else(|| "authenticated_tenant_owner_required".to_string())?;
            context.require_owner()?;
            if context.tenant_id() != tenant_id
                || context.principal_id() != principal_id
                || pending.source.principal_id.as_deref() != Some(principal_id)
            {
                return Err("tenant_case_open_security_context_mismatch".to_string());
            }
            return Ok(());
        }

        let Some(state) = current_state else {
            return Ok(());
        };
        let Some(case_tenant_id) = state.tenant_id.as_deref() else {
            if security_context.is_some() {
                return Err("legacy_unscoped_case_cannot_accept_tenant_authority".to_string());
            }
            return Ok(());
        };
        if let Some(context) = security_context {
            if context.tenant_id() != case_tenant_id {
                return Err("cross_tenant_case_write_rejected".to_string());
            }
        }

        let owner_protected = matches!(
            pending.payload,
            TransitionPayload::ParticipantBound { .. }
                | TransitionPayload::ParticipantAdmitted { .. }
                | TransitionPayload::ParticipantPrincipalLinked { .. }
                | TransitionPayload::ProviderAttached { .. }
                | TransitionPayload::ResourceAttached { .. }
                | TransitionPayload::CasePolicyBound { .. }
                | TransitionPayload::CasePolicyReplaced { .. }
                | TransitionPayload::CasePolicyUnbound { .. }
                | TransitionPayload::CaseCancellationRequested { .. }
                | TransitionPayload::CaseClosed { .. }
                | TransitionPayload::CaseWorkflowBound { .. }
                | TransitionPayload::WorkflowAmendmentAdopted { .. }
        );
        if owner_protected {
            let context = security_context
                .ok_or_else(|| "authenticated_tenant_owner_required".to_string())?;
            context.require_owner()?;
            if pending.source.principal_id.as_deref() != Some(context.principal_id()) {
                return Err("administrative_principal_provenance_mismatch".to_string());
            }
        }

        if let TransitionPayload::ParticipantPrincipalLinked { link } = &pending.payload {
            let context = security_context
                .ok_or_else(|| "authenticated_tenant_owner_required".to_string())?;
            if link.case_id != pending.case_id
                || link.tenant_id != case_tenant_id
                || link.created_by_principal_id != context.principal_id()
            {
                return Err("principal_participant_link_security_domain_mismatch".to_string());
            }
            if get_json_txn::<TenantMembershipKind, _>(
                txn,
                self.tenant_memberships,
                &tenant_membership_key(case_tenant_id, &link.principal_id),
                "tenant_membership",
            )?
            .is_none()
            {
                return Err("linked_principal_not_tenant_member".to_string());
            }
        }

        if let TransitionPayload::ReviewActionRecorded { action } = &pending.payload {
            let context = security_context
                .ok_or_else(|| "authenticated_human_review_required".to_string())?;
            if action.schema != crate::transition::REVIEW_ACTION_SCHEMA
                || action.principal_id.as_deref() != Some(context.principal_id())
                || action.tenant_id.as_deref() != Some(case_tenant_id)
                || pending.source.principal_id.as_deref() != Some(context.principal_id())
            {
                return Err("authenticated_review_security_context_mismatch".to_string());
            }
            let linked = state.principal_participant_links.iter().any(|link| {
                link.principal_id == context.principal_id()
                    && link.participant_id == action.reviewer_participant_id
                    && link.tenant_id == case_tenant_id
            });
            if !linked {
                return Err("authenticated_principal_participant_link_required".to_string());
            }
        }
        Ok(())
    }

    fn validate_case_handoff_progression_txn(
        &self,
        txn: &RwTransaction<'_>,
        state: &CaseState,
        pending: &PendingTransition,
        context: &SecurityContext,
    ) -> Result<(), String> {
        let tenant_id = state
            .tenant_id
            .as_deref()
            .ok_or_else(|| "handoff_requires_tenant_case".to_string())?;
        match &pending.payload {
            TransitionPayload::HandoffOffered { offer } => {
                offer.validate()?;
                if offer.source_binding_id == "case-handoff:manual" {
                    context.require_owner()?;
                }
                let target = self
                    .get_case_state_txn(txn, &offer.target_case_id)?
                    .ok_or_else(|| "handoff_target_case_missing".to_string())?;
                if offer.source_case_id != state.case_id
                    || offer.tenant_id != tenant_id
                    || target.tenant_id.as_deref() != Some(tenant_id)
                    || target.lifecycle == CaseLifecycle::Closed
                    || target.cancellation.is_some()
                    || offer.offered_by_principal_id != context.principal_id()
                    || offer.offered_at_generation != state.generation + 1
                    || state
                        .handoff_offers
                        .iter()
                        .any(|value| value.handoff_id == offer.handoff_id)
                    || self.handoff_would_create_cycle_txn(
                        txn,
                        &offer.source_case_id,
                        &offer.target_case_id,
                    )?
                {
                    return Err("handoff_offer_rederivation_mismatch".to_string());
                }
            }
            TransitionPayload::HandoffAccepted { acceptance } => {
                let source = self
                    .get_case_state_txn(txn, &acceptance.source_case_id)?
                    .ok_or_else(|| "handoff_source_case_missing".to_string())?;
                let offer = source
                    .handoff_offers
                    .iter()
                    .find(|offer| offer.handoff_id == acceptance.handoff_id)
                    .ok_or_else(|| "handoff_offer_not_found".to_string())?;
                let participant = state
                    .participants
                    .iter()
                    .find(|value| value.participant_id == acceptance.accepted_by_participant_id)
                    .ok_or_else(|| "handoff_accepting_participant_missing".to_string())?;
                let linked = state.principal_participant_links.iter().any(|link| {
                    link.principal_id == context.principal_id()
                        && link.participant_id == acceptance.accepted_by_participant_id
                        && link.tenant_id == tenant_id
                });
                let expected = HandoffAcceptance::build(
                    offer,
                    context.principal_id(),
                    &acceptance.accepted_by_participant_id,
                    state.generation + 1,
                    acceptance.accepted_at_unix_ms,
                )?;
                if source.tenant_id.as_deref() != Some(tenant_id)
                    || source.cancellation.is_some()
                    || source.lifecycle == CaseLifecycle::Closed
                    || offer.target_case_id != state.case_id
                    || acceptance.accepted_by_principal_id != context.principal_id()
                    || acceptance.accepted_at_generation != state.generation + 1
                    || !linked
                    || offer
                        .required_target_roles
                        .iter()
                        .any(|role| !participant.roles.contains(role))
                    || state
                        .handoff_acceptances
                        .iter()
                        .any(|value| value.handoff_id == acceptance.handoff_id)
                    || state
                        .handoff_declines
                        .iter()
                        .any(|value| value.handoff_id == acceptance.handoff_id)
                    || acceptance != &expected
                {
                    return Err("handoff_acceptance_rederivation_mismatch".to_string());
                }
            }
            TransitionPayload::HandoffDeclined { decline } => {
                let source = self
                    .get_case_state_txn(txn, &decline.source_case_id)?
                    .ok_or_else(|| "handoff_source_case_missing".to_string())?;
                let offer = source
                    .handoff_offers
                    .iter()
                    .find(|offer| offer.handoff_id == decline.handoff_id)
                    .ok_or_else(|| "handoff_offer_not_found".to_string())?;
                let linked = state.principal_participant_links.iter().any(|link| {
                    link.principal_id == context.principal_id()
                        && link.participant_id == decline.declined_by_participant_id
                        && link.tenant_id == tenant_id
                });
                let expected = HandoffDecline::build(
                    offer,
                    context.principal_id(),
                    &decline.declined_by_participant_id,
                    &decline.reason,
                    state.generation + 1,
                    decline.declined_at_unix_ms,
                )?;
                if source.tenant_id.as_deref() != Some(tenant_id)
                    || source.cancellation.is_some()
                    || source.lifecycle == CaseLifecycle::Closed
                    || offer.target_case_id != state.case_id
                    || decline.declined_by_principal_id != context.principal_id()
                    || decline.declined_at_generation != state.generation + 1
                    || !linked
                    || state
                        .handoff_acceptances
                        .iter()
                        .any(|value| value.handoff_id == decline.handoff_id)
                    || state
                        .handoff_declines
                        .iter()
                        .any(|value| value.handoff_id == decline.handoff_id)
                    || decline != &expected
                {
                    return Err("handoff_decline_rederivation_mismatch".to_string());
                }
            }
            TransitionPayload::HandoffResultRecorded { result } => {
                let acceptance = state
                    .handoff_acceptances
                    .iter()
                    .find(|value| value.acceptance_id == result.acceptance_id)
                    .ok_or_else(|| "handoff_acceptance_not_found".to_string())?;
                let linked = state.principal_participant_links.iter().any(|link| {
                    link.principal_id == context.principal_id()
                        && link.participant_id == result.recorded_by_participant_id
                        && link.tenant_id == tenant_id
                });
                let expected = HandoffResult::build(
                    acceptance,
                    result.outcome.clone(),
                    result.result.clone(),
                    result.evidence_refs.clone(),
                    context.principal_id(),
                    &result.recorded_by_participant_id,
                    state.generation + 1,
                    result.recorded_at_unix_ms,
                )?;
                self.validate_handoff_evidence_refs_txn(
                    txn,
                    &state.case_id,
                    &result.evidence_refs,
                )?;
                if state.lifecycle == CaseLifecycle::Closed
                    || state.cancellation.is_some()
                    || acceptance.handoff_id != result.handoff_id
                    || result.target_case_id != state.case_id
                    || result.recorded_by_principal_id != context.principal_id()
                    || result.recorded_at_generation != state.generation + 1
                    || !linked
                    || state
                        .handoff_results
                        .iter()
                        .any(|value| value.handoff_id == result.handoff_id)
                    || result != &expected
                {
                    return Err("handoff_result_rederivation_mismatch".to_string());
                }
            }
            TransitionPayload::HandoffReconciled { reconciliation } => {
                let offer = state
                    .handoff_offers
                    .iter()
                    .find(|value| value.handoff_id == reconciliation.handoff_id)
                    .ok_or_else(|| "handoff_offer_not_found".to_string())?;
                let target = self
                    .get_case_state_txn(txn, &offer.target_case_id)?
                    .ok_or_else(|| "handoff_target_case_missing".to_string())?;
                let expected = self.expected_handoff_reconciliation(
                    offer,
                    &target,
                    context.principal_id(),
                    state.generation + 1,
                    reconciliation.reconciled_at_unix_ms,
                )?;
                if reconciliation != &expected {
                    return Err("handoff_reconciliation_rederivation_mismatch".to_string());
                }
            }
            _ => return Err("not_a_handoff_transition".to_string()),
        }
        Ok(())
    }

    fn expected_handoff_reconciliation(
        &self,
        offer: &HandoffOffer,
        target: &CaseState,
        principal_id: &str,
        generation: u64,
        now_unix_ms: u64,
    ) -> Result<HandoffReconciliation, String> {
        if let Some(result) = target
            .handoff_results
            .iter()
            .find(|value| value.handoff_id == offer.handoff_id)
        {
            return HandoffReconciliation::build(result, principal_id, generation, now_unix_ms);
        }
        if let Some(decline) = target
            .handoff_declines
            .iter()
            .find(|value| value.handoff_id == offer.handoff_id)
        {
            return HandoffReconciliation::build_declined(
                decline,
                principal_id,
                generation,
                now_unix_ms,
            );
        }
        let acceptance_id = target
            .handoff_acceptances
            .iter()
            .find(|value| value.handoff_id == offer.handoff_id)
            .map(|value| value.acceptance_id.clone());
        if let Some(cancellation) = &target.cancellation {
            return HandoffReconciliation::build_target_terminal(
                &offer.handoff_id,
                acceptance_id,
                &cancellation.transition_id,
                HandoffOutcome::Cancelled,
                principal_id,
                generation,
                now_unix_ms,
            );
        }
        if let Some(closure) = &target.closure {
            return HandoffReconciliation::build_target_terminal(
                &offer.handoff_id,
                acceptance_id,
                &closure.transition_id,
                HandoffOutcome::Failed,
                principal_id,
                generation,
                now_unix_ms,
            );
        }
        Err("handoff_target_disposition_missing".to_string())
    }

    fn validate_handoff_evidence_refs_txn<T: Transaction>(
        &self,
        txn: &T,
        target_case_id: &str,
        evidence_refs: &[String],
    ) -> Result<(), String> {
        if evidence_refs.is_empty() {
            return Ok(());
        }
        let history = self.list_case_transitions_txn(txn, target_case_id)?;
        for reference in evidence_refs {
            if !history
                .iter()
                .any(|transition| transition_contains_canonical_fact_ref(transition, reference))
            {
                return Err(format!(
                    "handoff_result_evidence_not_target_local: {reference}"
                ));
            }
        }
        Ok(())
    }

    fn handoff_close_blockers_txn<T: Transaction>(
        &self,
        txn: &T,
        source: &CaseState,
    ) -> Result<Vec<String>, String> {
        let reconciled = source
            .handoff_reconciliations
            .iter()
            .map(|value| value.handoff_id.as_str())
            .collect::<BTreeSet<_>>();
        let mut blockers = Vec::new();
        for offer in &source.handoff_offers {
            if reconciled.contains(offer.handoff_id.as_str()) {
                continue;
            }
            let target = self
                .get_case_state_txn(txn, &offer.target_case_id)?
                .ok_or_else(|| "handoff_target_case_missing".to_string())?;
            let accepted = target
                .handoff_acceptances
                .iter()
                .any(|value| value.handoff_id == offer.handoff_id);
            let disposition = target
                .handoff_results
                .iter()
                .any(|value| value.handoff_id == offer.handoff_id)
                || target
                    .handoff_declines
                    .iter()
                    .any(|value| value.handoff_id == offer.handoff_id)
                || (accepted && (target.cancellation.is_some() || target.closure.is_some()));
            if disposition {
                blockers.push(format!("handoff_settlement_required:{}", offer.handoff_id));
            } else if accepted {
                blockers.push(format!("accepted_handoff_unresolved:{}", offer.handoff_id));
            }
        }
        Ok(blockers)
    }

    fn handoff_would_create_cycle_txn<T: Transaction>(
        &self,
        txn: &T,
        source_case_id: &str,
        target_case_id: &str,
    ) -> Result<bool, String> {
        let mut cursor = txn
            .open_ro_cursor(self.case_state)
            .map_err(|error| format!("failed to inspect Handoff wait graph: {error}"))?;
        let mut cases = Vec::new();
        for (_, value) in cursor.iter() {
            let json = std::str::from_utf8(value)
                .map_err(|error| format!("invalid CaseState utf8: {error}"))?;
            cases.push(CaseState::from_json(json)?);
        }
        drop(cursor);
        let mut terminal = BTreeSet::new();
        for case in &cases {
            terminal.extend(
                case.handoff_reconciliations
                    .iter()
                    .map(|value| value.handoff_id.clone()),
            );
            terminal.extend(
                case.handoff_declines
                    .iter()
                    .map(|value| value.handoff_id.clone()),
            );
            terminal.extend(
                case.handoff_results
                    .iter()
                    .map(|value| value.handoff_id.clone()),
            );
            if case.cancellation.is_some() || case.closure.is_some() {
                terminal.extend(
                    case.handoff_acceptances
                        .iter()
                        .map(|value| value.handoff_id.clone()),
                );
            }
        }
        let mut adjacency = BTreeMap::<String, BTreeSet<String>>::new();
        for case in &cases {
            for offer in &case.handoff_offers {
                if !terminal.contains(&offer.handoff_id) {
                    adjacency
                        .entry(offer.source_case_id.clone())
                        .or_default()
                        .insert(offer.target_case_id.clone());
                }
            }
        }
        let mut pending = vec![target_case_id.to_string()];
        let mut seen = BTreeSet::new();
        while let Some(case_id) = pending.pop() {
            if case_id == source_case_id {
                return Ok(true);
            }
            if !seen.insert(case_id.clone()) {
                continue;
            }
            if let Some(targets) = adjacency.get(&case_id) {
                pending.extend(targets.iter().cloned());
            }
        }
        Ok(false)
    }

    fn validate_workflow_progression_txn(
        &self,
        txn: &RwTransaction<'_>,
        current_state: Option<&CaseState>,
        pending: &PendingTransition,
        security_context: Option<&SecurityContext>,
    ) -> Result<(), String> {
        let is_workflow = matches!(
            &pending.payload,
            TransitionPayload::CaseWorkflowBound { .. }
                | TransitionPayload::WorkflowNodeExecutionStarted { .. }
                | TransitionPayload::WorkflowNodeSatisfied { .. }
                | TransitionPayload::WorkflowConditionResolved { .. }
                | TransitionPayload::WorkflowHumanInputRecorded { .. }
                | TransitionPayload::WorkflowDeterministicProposalRecorded { .. }
                | TransitionPayload::WorkflowPlanPatchProposed { .. }
                | TransitionPayload::WorkflowAmendmentAdopted { .. }
                | TransitionPayload::HandoffOffered { .. }
                | TransitionPayload::HandoffAccepted { .. }
                | TransitionPayload::HandoffDeclined { .. }
                | TransitionPayload::HandoffResultRecorded { .. }
                | TransitionPayload::HandoffReconciled { .. }
        ) || matches!(
            &pending.payload,
            TransitionPayload::OperationRecorded { operation }
                if matches!(operation.origin, OperationOrigin::WorkflowDeterministicProposal { .. })
        );
        if !is_workflow {
            return Ok(());
        }
        let state = current_state
            .ok_or_else(|| "workflow_progression_requires_existing_case".to_string())?;
        let tenant_id = state
            .tenant_id
            .as_deref()
            .ok_or_else(|| "workflow_progression_requires_tenant_case".to_string())?;
        let context = security_context
            .ok_or_else(|| "authenticated_workflow_progression_required".to_string())?;
        if context.tenant_id() != tenant_id
            || pending.source.principal_id.as_deref() != Some(context.principal_id())
        {
            return Err("workflow_progression_security_context_mismatch".to_string());
        }

        let case_level_handoff = matches!(
            &pending.payload,
            TransitionPayload::HandoffAccepted { .. }
                | TransitionPayload::HandoffDeclined { .. }
                | TransitionPayload::HandoffResultRecorded { .. }
                | TransitionPayload::HandoffReconciled { .. }
        ) || matches!(
            &pending.payload,
            TransitionPayload::HandoffOffered { offer }
                if offer.source_binding_id == "case-handoff:manual"
        );
        if case_level_handoff {
            return self.validate_case_handoff_progression_txn(txn, state, pending, context);
        }

        if let TransitionPayload::CaseWorkflowBound { binding } = &pending.payload {
            let definition = self
                .workflow_definition_txn(txn, &binding.workflow_definition_id)?
                .ok_or_else(|| "bound_workflow_definition_missing".to_string())?;
            binding.validate(&definition)?;
            if binding.tenant_id != tenant_id
                || binding.case_id != state.case_id
                || binding.bound_at_generation != state.generation + 1
            {
                return Err("workflow_binding_rederivation_mismatch".to_string());
            }
            return Ok(());
        }

        let binding = state
            .workflow_binding
            .as_ref()
            .ok_or_else(|| "workflow_progression_without_binding".to_string())?;
        let definition = self
            .workflow_definition_txn(txn, &binding.workflow_definition_id)?
            .ok_or_else(|| "bound_workflow_definition_missing".to_string())?;
        if definition.integrity_digest != binding.workflow_definition_digest {
            return Err("bound_workflow_definition_digest_mismatch".to_string());
        }
        let extra_operations = match &pending.payload {
            TransitionPayload::WorkflowPlanPatchProposed { patch } => patch.operations.as_slice(),
            _ => &[],
        };
        let definitions = self.workflow_definition_graph_for_operations_txn(
            txn,
            &definition,
            &state.workflow_amendments,
            extra_operations,
        )?;
        let topology = derive_effective_workflow_topology(
            &definition,
            binding,
            &state.workflow_amendments,
            &definitions,
        )?;
        let history = self.list_case_transitions_txn(txn, &state.case_id)?;
        let resolution =
            resolve_workflow_with_definitions(&definition, binding, state, &history, &definitions)?;

        match &pending.payload {
            TransitionPayload::WorkflowNodeExecutionStarted { execution } => {
                let node = topology
                    .node(&execution.node_id)
                    .map(|value| &value.node)
                    .ok_or_else(|| "workflow_node_not_found".to_string())?;
                let ready = resolution
                    .ready_work
                    .iter()
                    .any(|work| work.node_id == execution.node_id && node.is_executable());
                let expected_id = workflow_execution_identity(binding, &execution.node_id);
                if !ready
                    || execution.execution_id != expected_id
                    || execution.binding_id != binding.binding_id
                    || execution.workflow_definition_id != definition.workflow_definition_id
                    || execution.case_id != state.case_id
                    || execution.started_at_generation != state.generation + 1
                {
                    return Err("workflow_execution_readiness_rederivation_mismatch".to_string());
                }
            }
            TransitionPayload::WorkflowConditionResolved { resolution: fact } => {
                let node = topology
                    .node(&fact.node_id)
                    .map(|value| &value.node)
                    .ok_or_else(|| "workflow_node_not_found".to_string())?;
                let WorkflowNodeKind::Condition { predicate } = &node.kind else {
                    return Err("workflow_condition_node_kind_mismatch".to_string());
                };
                let node_view = resolution
                    .nodes
                    .iter()
                    .find(|node| node.node_id == fact.node_id)
                    .ok_or_else(|| "workflow_node_not_found".to_string())?;
                let evaluation = evaluate_predicate(
                    &definition,
                    binding,
                    state,
                    &history,
                    &fact.node_id,
                    None,
                    predicate,
                )?;
                let expected_id = workflow_fact_identity(
                    "condition",
                    &binding.binding_id,
                    &fact.node_id,
                    &predicate.digest()?,
                    &evaluation.evidence_refs,
                );
                if node_view.posture != WorkflowNodePosture::WaitingCondition
                    || fact.resolution_id != expected_id
                    || fact.binding_id != binding.binding_id
                    || fact.workflow_definition_id != definition.workflow_definition_id
                    || fact.result != evaluation.value
                    || fact.predicate_digest != predicate.digest()?
                    || fact.evidence_refs != evaluation.evidence_refs
                    || fact.evaluated_at_generation != state.generation + 1
                {
                    return Err("workflow_condition_rederivation_mismatch".to_string());
                }
            }
            TransitionPayload::WorkflowNodeSatisfied { satisfaction } => {
                let node = topology
                    .node(&satisfaction.node_id)
                    .map(|value| &value.node)
                    .ok_or_else(|| "workflow_node_not_found".to_string())?;
                if matches!(
                    node.kind,
                    WorkflowNodeKind::Condition { .. } | WorkflowNodeKind::HumanInput { .. }
                ) {
                    return Err("workflow_node_kind_has_dedicated_satisfaction_fact".to_string());
                }
                if matches!(node.kind, WorkflowNodeKind::Subflow { .. }) {
                    let node_view = resolution
                        .nodes
                        .iter()
                        .find(|value| value.node_id == satisfaction.node_id)
                        .ok_or_else(|| "workflow_node_not_found".to_string())?;
                    let evidence_refs = workflow_subflow_completion_evidence(
                        &topology,
                        state,
                        &satisfaction.node_id,
                    )?;
                    let predicate_digest = workflow_subflow_predicate_digest(&satisfaction.node_id);
                    let expected_id = workflow_fact_identity(
                        "satisfaction",
                        &binding.binding_id,
                        &satisfaction.node_id,
                        &predicate_digest,
                        &evidence_refs,
                    );
                    if node_view.reason != "subflow_children_complete_pending_satisfaction"
                        || satisfaction.satisfaction_id != expected_id
                        || satisfaction.binding_id != binding.binding_id
                        || satisfaction.workflow_definition_id != definition.workflow_definition_id
                        || satisfaction.execution_id.is_some()
                        || satisfaction.predicate_digest != predicate_digest
                        || satisfaction.evidence_refs != evidence_refs
                        || satisfaction.evaluated_at_generation != state.generation + 1
                    {
                        return Err(
                            "workflow_subflow_satisfaction_rederivation_mismatch".to_string()
                        );
                    }
                    return Ok(());
                }
                let predicate = node_completion_predicate(node)
                    .ok_or_else(|| "workflow_node_has_no_completion_predicate".to_string())?;
                let execution_id = state
                    .workflow_executions
                    .iter()
                    .find(|execution| execution.node_id == satisfaction.node_id)
                    .map(|execution| execution.execution_id.as_str());
                let evaluation = evaluate_predicate(
                    &definition,
                    binding,
                    state,
                    &history,
                    &satisfaction.node_id,
                    execution_id,
                    predicate,
                )?;
                let expected_id = workflow_fact_identity(
                    "satisfaction",
                    &binding.binding_id,
                    &satisfaction.node_id,
                    &predicate.digest()?,
                    &evaluation.evidence_refs,
                );
                if !evaluation.value
                    || satisfaction.satisfaction_id != expected_id
                    || satisfaction.binding_id != binding.binding_id
                    || satisfaction.workflow_definition_id != definition.workflow_definition_id
                    || satisfaction.execution_id.as_deref() != execution_id
                    || satisfaction.predicate_digest != predicate.digest()?
                    || satisfaction.evidence_refs != evaluation.evidence_refs
                    || satisfaction.evaluated_at_generation != state.generation + 1
                {
                    return Err("workflow_satisfaction_rederivation_mismatch".to_string());
                }
            }
            TransitionPayload::WorkflowHumanInputRecorded { input } => {
                let node_view = resolution
                    .nodes
                    .iter()
                    .find(|node| node.node_id == input.node_id)
                    .ok_or_else(|| "workflow_node_not_found".to_string())?;
                let node = topology
                    .node(&input.node_id)
                    .map(|value| &value.node)
                    .ok_or_else(|| "workflow_node_not_found".to_string())?;
                let WorkflowNodeKind::HumanInput {
                    actor_slot,
                    required_roles,
                    input_kind,
                    max_bytes,
                    ..
                } = &node.kind
                else {
                    return Err("workflow_human_input_node_kind_mismatch".to_string());
                };
                let expected_participant = binding
                    .participant_for_slot(actor_slot)
                    .ok_or_else(|| "workflow_human_actor_slot_unbound".to_string())?;
                let participant = state
                    .participants
                    .iter()
                    .find(|participant| participant.participant_id == expected_participant)
                    .ok_or_else(|| "workflow_human_participant_missing".to_string())?;
                let linked = state.principal_participant_links.iter().any(|link| {
                    link.principal_id == context.principal_id()
                        && link.participant_id == expected_participant
                        && link.tenant_id == tenant_id
                });
                if node_view.posture != WorkflowNodePosture::WaitingHumanInput
                    || input.binding_id != binding.binding_id
                    || input.workflow_definition_id != definition.workflow_definition_id
                    || input.principal_id != context.principal_id()
                    || input.participant_id != expected_participant
                    || input.value.is_empty()
                    || input.value.len() > *max_bytes
                    || input.value_digest != crate::effect::digest_bytes(input.value.as_bytes())
                    || input.recorded_at_generation != state.generation + 1
                    || !linked
                    || required_roles
                        .iter()
                        .any(|role| !participant.roles.contains(role))
                {
                    return Err("workflow_human_input_rederivation_mismatch".to_string());
                }
                if *input_kind == HumanInputKind::Json {
                    let parsed = crate::governance::parse_strict_json(input.value.as_bytes())
                        .map_err(|error| format!("workflow_human_input_json_invalid: {error}"))?;
                    if parsed.is_array() || parsed.is_null() {
                        return Err("workflow_human_input_json_kind_invalid".to_string());
                    }
                }
            }
            TransitionPayload::WorkflowDeterministicProposalRecorded { proposal } => {
                let node = topology
                    .node(&proposal.node_id)
                    .map(|value| &value.node)
                    .ok_or_else(|| "workflow_node_not_found".to_string())?;
                let WorkflowNodeKind::DeterministicWork {
                    proposer_slot,
                    operation,
                    ..
                } = &node.kind
                else {
                    return Err("workflow_deterministic_node_kind_mismatch".to_string());
                };
                let expected_participant = binding
                    .participant_for_slot(proposer_slot)
                    .ok_or_else(|| "workflow_deterministic_proposer_slot_unbound".to_string())?;
                let expected_attachment = binding
                    .attachment_for_slot(operation.resource_slot())
                    .ok_or_else(|| "workflow_deterministic_resource_slot_unbound".to_string())?;
                let expected_template_digest =
                    crate::context::stable_digest(&serde_json::to_string(operation).map_err(
                        |error| format!("workflow_deterministic_template_encode_failed: {error}"),
                    )?);
                let execution = state.workflow_executions.iter().find(|execution| {
                    execution.node_id == proposal.node_id
                        && execution.execution_id == proposal.execution_id
                });
                if execution.is_none()
                    || proposal.binding_id != binding.binding_id
                    || proposal.workflow_definition_id != definition.workflow_definition_id
                    || proposal.participant_id != expected_participant
                    || proposal.resource_attachment_id != expected_attachment
                    || proposal.operation_kind != operation.operation_kind()
                    || proposal.template_digest != expected_template_digest
                    || proposal.recorded_at_generation != state.generation + 1
                {
                    return Err("workflow_deterministic_proposal_rederivation_mismatch".to_string());
                }
            }
            TransitionPayload::OperationRecorded { operation }
                if matches!(
                    operation.origin,
                    OperationOrigin::WorkflowDeterministicProposal { .. }
                ) =>
            {
                let OperationOrigin::WorkflowDeterministicProposal {
                    proposal_id,
                    workflow_execution_id,
                } = &operation.origin
                else {
                    unreachable!()
                };
                let proposal = state
                    .workflow_deterministic_proposals
                    .iter()
                    .find(|proposal| {
                        proposal.proposal_id == *proposal_id
                            && proposal.execution_id == *workflow_execution_id
                    })
                    .ok_or_else(|| {
                        "workflow_deterministic_operation_proposal_missing".to_string()
                    })?;
                let node = topology
                    .node(&proposal.node_id)
                    .map(|value| &value.node)
                    .ok_or_else(|| "workflow_node_not_found".to_string())?;
                let WorkflowNodeKind::DeterministicWork {
                    operation: template,
                    ..
                } = &node.kind
                else {
                    return Err("workflow_deterministic_node_kind_mismatch".to_string());
                };
                let resource = state
                    .resources
                    .iter()
                    .find(|resource| resource.attachment_id == proposal.resource_attachment_id)
                    .ok_or_else(|| "workflow_deterministic_resource_missing".to_string())?;
                let expected = match template {
                    DeterministicOperationTemplate::FilesystemWrite {
                        relative_path,
                        content,
                        ..
                    } => build_workflow_deterministic_filesystem_operation(
                        &state.case_id,
                        &proposal.participant_id,
                        state.generation,
                        resource,
                        relative_path,
                        content,
                        proposal_id,
                        workflow_execution_id,
                    )?,
                    DeterministicOperationTemplate::ProcessSignal { action, .. } => {
                        let binding = self
                            .local_process_binding_txn(
                                txn,
                                &state.case_id,
                                &proposal.resource_attachment_id,
                            )?
                            .ok_or_else(|| "local_process_binding_missing".to_string())?;
                        build_workflow_deterministic_process_operation(
                            &state.case_id,
                            &proposal.participant_id,
                            state.generation,
                            resource,
                            &binding.process,
                            action.clone(),
                            proposal_id,
                            workflow_execution_id,
                        )?
                    }
                };
                if operation != &expected {
                    return Err(
                        "workflow_deterministic_operation_rederivation_mismatch".to_string()
                    );
                }
            }
            TransitionPayload::WorkflowPlanPatchProposed { patch } => {
                patch.validate(binding)?;
                if patch.case_id != state.case_id
                    || patch.tenant_id != tenant_id
                    || patch.proposed_at_generation != state.generation + 1
                    || patch.base_effective_topology_digest != topology.topology_digest
                    || patch.base_revision != topology.revision
                    || patch.parent_amendment_id.as_deref()
                        != state
                            .workflow_amendments
                            .last()
                            .map(|value| value.amendment_id.as_str())
                {
                    return Err("workflow_plan_patch_rederivation_mismatch".to_string());
                }
                match &patch.origin {
                    WorkflowPlanPatchOrigin::AuthenticatedHuman { principal_id } => {
                        context.require_owner()?;
                        if principal_id != context.principal_id() {
                            return Err("workflow_plan_patch_human_origin_mismatch".to_string());
                        }
                    }
                    WorkflowPlanPatchOrigin::ModelProviderResult {
                        provider_result_id,
                        workflow_execution_id,
                    } => {
                        let result = history.iter().find(|transition| {
                            transition
                                .causal_refs
                                .iter()
                                .any(|value| value == workflow_execution_id)
                                && matches!(
                                    &transition.payload,
                                    TransitionPayload::ProviderResultRecorded { result_id, .. }
                                        if result_id == provider_result_id
                                )
                        });
                        let execution = state
                            .workflow_executions
                            .iter()
                            .find(|execution| execution.execution_id == *workflow_execution_id);
                        let node = execution
                            .and_then(|execution| topology.node(&execution.node_id))
                            .map(|effective| &effective.node);
                        if result.is_none()
                            || !matches!(
                                node.map(|value| &value.kind),
                                Some(WorkflowNodeKind::ModelWork {
                                    output_contract:
                                        crate::workflow::ModelWorkOutputContract::PlanPatch,
                                    ..
                                })
                            )
                        {
                            return Err("workflow_plan_patch_model_origin_mismatch".to_string());
                        }
                    }
                }
                workflow_patch_frozen_history_barrier(state, patch)?;
                preview_workflow_patch(
                    &definition,
                    binding,
                    &state.workflow_amendments,
                    patch,
                    &definitions,
                )?;
            }
            TransitionPayload::WorkflowAmendmentAdopted { amendment } => {
                context.require_owner()?;
                let patch = state
                    .workflow_plan_patches
                    .iter()
                    .find(|patch| patch.patch_id == amendment.patch_id)
                    .ok_or_else(|| "workflow_amendment_patch_missing".to_string())?;
                if resolution.completed
                    || resolution.active_count != 0
                    || amendment.adopted_at_generation != state.generation + 1
                    || state
                        .workflow_amendments
                        .iter()
                        .any(|value| value.patch_id == patch.patch_id)
                {
                    return Err("workflow_amendment_admission_rejected".to_string());
                }
                workflow_patch_frozen_history_barrier(state, patch)?;
                let preview = preview_workflow_patch(
                    &definition,
                    binding,
                    &state.workflow_amendments,
                    patch,
                    &definitions,
                )?;
                let expected = WorkflowAmendment::build(
                    patch,
                    &preview.topology_digest,
                    context.principal_id(),
                    state.generation + 1,
                    amendment.adopted_at_unix_ms,
                )?;
                if amendment != &expected {
                    return Err("workflow_amendment_rederivation_mismatch".to_string());
                }
            }
            TransitionPayload::HandoffOffered { offer } => {
                let effective = topology
                    .node(&offer.source_node_id)
                    .ok_or_else(|| "workflow_handoff_node_missing".to_string())?;
                let WorkflowNodeKind::Handoff {
                    target_case_slot,
                    request,
                    required_target_roles,
                    ..
                } = &effective.node.kind
                else {
                    return Err("workflow_handoff_node_kind_mismatch".to_string());
                };
                let target_case_id = binding
                    .case_for_slot(target_case_slot)
                    .ok_or_else(|| "workflow_handoff_case_slot_unbound".to_string())?;
                let target = self
                    .get_case_state_txn(txn, target_case_id)?
                    .ok_or_else(|| "handoff_target_case_missing".to_string())?;
                let node_view = resolution
                    .nodes
                    .iter()
                    .find(|node| node.node_id == offer.source_node_id)
                    .ok_or_else(|| "workflow_handoff_node_missing".to_string())?;
                if offer.source_case_id != state.case_id
                    || offer.target_case_id != target_case_id
                    || target.tenant_id.as_deref() != Some(tenant_id)
                    || target.lifecycle == CaseLifecycle::Closed
                    || target.cancellation.is_some()
                    || offer.source_binding_id != binding.binding_id
                    || offer.request != *request
                    || offer.required_target_roles != *required_target_roles
                    || offer.offered_by_principal_id != context.principal_id()
                    || offer.offered_at_generation != state.generation + 1
                    || node_view.posture != WorkflowNodePosture::WaitingEffect
                    || state
                        .handoff_offers
                        .iter()
                        .any(|current| current.source_node_id == offer.source_node_id)
                    || self.handoff_would_create_cycle_txn(
                        txn,
                        &offer.source_case_id,
                        &offer.target_case_id,
                    )?
                {
                    return Err("workflow_handoff_offer_rederivation_mismatch".to_string());
                }
            }
            TransitionPayload::HandoffAccepted { acceptance } => {
                let source = self
                    .get_case_state_txn(txn, &acceptance.source_case_id)?
                    .ok_or_else(|| "handoff_source_case_missing".to_string())?;
                let offer = source
                    .handoff_offers
                    .iter()
                    .find(|offer| offer.handoff_id == acceptance.handoff_id)
                    .ok_or_else(|| "handoff_offer_not_found".to_string())?;
                let participant = state
                    .participants
                    .iter()
                    .find(|value| value.participant_id == acceptance.accepted_by_participant_id)
                    .ok_or_else(|| "handoff_accepting_participant_missing".to_string())?;
                let linked = state.principal_participant_links.iter().any(|link| {
                    link.principal_id == context.principal_id()
                        && link.participant_id == acceptance.accepted_by_participant_id
                        && link.tenant_id == tenant_id
                });
                let expected = HandoffAcceptance::build(
                    offer,
                    context.principal_id(),
                    &acceptance.accepted_by_participant_id,
                    state.generation + 1,
                    acceptance.accepted_at_unix_ms,
                )?;
                if source.tenant_id.as_deref() != Some(tenant_id)
                    || source.cancellation.is_some()
                    || source.lifecycle == CaseLifecycle::Closed
                    || offer.target_case_id != state.case_id
                    || acceptance.accepted_by_principal_id != context.principal_id()
                    || acceptance.accepted_at_generation != state.generation + 1
                    || !linked
                    || offer
                        .required_target_roles
                        .iter()
                        .any(|role| !participant.roles.contains(role))
                    || acceptance != &expected
                {
                    return Err("handoff_acceptance_rederivation_mismatch".to_string());
                }
            }
            TransitionPayload::HandoffDeclined { decline } => {
                let source = self
                    .get_case_state_txn(txn, &decline.source_case_id)?
                    .ok_or_else(|| "handoff_source_case_missing".to_string())?;
                let offer = source
                    .handoff_offers
                    .iter()
                    .find(|offer| offer.handoff_id == decline.handoff_id)
                    .ok_or_else(|| "handoff_offer_not_found".to_string())?;
                let linked = state.principal_participant_links.iter().any(|link| {
                    link.principal_id == context.principal_id()
                        && link.participant_id == decline.declined_by_participant_id
                        && link.tenant_id == tenant_id
                });
                let expected = HandoffDecline::build(
                    offer,
                    context.principal_id(),
                    &decline.declined_by_participant_id,
                    &decline.reason,
                    state.generation + 1,
                    decline.declined_at_unix_ms,
                )?;
                if source.cancellation.is_some()
                    || source.lifecycle == CaseLifecycle::Closed
                    || offer.target_case_id != state.case_id
                    || decline.declined_by_principal_id != context.principal_id()
                    || decline.declined_at_generation != state.generation + 1
                    || !linked
                    || decline != &expected
                {
                    return Err("handoff_decline_rederivation_mismatch".to_string());
                }
            }
            TransitionPayload::HandoffResultRecorded { result } => {
                let acceptance = state
                    .handoff_acceptances
                    .iter()
                    .find(|value| value.acceptance_id == result.acceptance_id)
                    .ok_or_else(|| "handoff_acceptance_not_found".to_string())?;
                let linked = state.principal_participant_links.iter().any(|link| {
                    link.principal_id == context.principal_id()
                        && link.participant_id == result.recorded_by_participant_id
                        && link.tenant_id == tenant_id
                });
                let expected = HandoffResult::build(
                    acceptance,
                    result.outcome.clone(),
                    result.result.clone(),
                    result.evidence_refs.clone(),
                    context.principal_id(),
                    &result.recorded_by_participant_id,
                    state.generation + 1,
                    result.recorded_at_unix_ms,
                )?;
                self.validate_handoff_evidence_refs_txn(
                    txn,
                    &state.case_id,
                    &result.evidence_refs,
                )?;
                if state.lifecycle == CaseLifecycle::Closed
                    || state.cancellation.is_some()
                    || acceptance.handoff_id != result.handoff_id
                    || result.target_case_id != state.case_id
                    || result.recorded_by_principal_id != context.principal_id()
                    || result.recorded_at_generation != state.generation + 1
                    || !linked
                    || result != &expected
                {
                    return Err("handoff_result_rederivation_mismatch".to_string());
                }
            }
            TransitionPayload::HandoffReconciled { reconciliation } => {
                let offer = state
                    .handoff_offers
                    .iter()
                    .find(|value| value.handoff_id == reconciliation.handoff_id)
                    .ok_or_else(|| "handoff_offer_not_found".to_string())?;
                let target = self
                    .get_case_state_txn(txn, &offer.target_case_id)?
                    .ok_or_else(|| "handoff_target_case_missing".to_string())?;
                let expected = self.expected_handoff_reconciliation(
                    offer,
                    &target,
                    context.principal_id(),
                    state.generation + 1,
                    reconciliation.reconciled_at_unix_ms,
                )?;
                if reconciliation != &expected {
                    return Err("handoff_reconciliation_rederivation_mismatch".to_string());
                }
            }
            TransitionPayload::CaseWorkflowBound { .. } => unreachable!(),
            _ => {}
        }
        Ok(())
    }

    fn validate_policy_authority_txn(
        &self,
        txn: &RwTransaction<'_>,
        current_state: Option<&CaseState>,
        case_id: &str,
        payload: &TransitionPayload,
        authority_time_unix_ms: u64,
    ) -> Result<(), String> {
        let state = match current_state {
            Some(state) => state,
            None => return Ok(()),
        };
        match payload {
            TransitionPayload::DecisionRecorded { decision }
                if decision.schema == crate::effect::DECISION_SCHEMA =>
            {
                decision.validate_integrity()?;
                let history = self.list_case_transitions_txn(txn, case_id)?;
                let expected = self.derive_expected_policy_decision_at_time_txn(
                    txn,
                    state,
                    &history,
                    decision,
                    Some(authority_time_unix_ms),
                )?;
                if decision.decision_basis.as_ref().is_none_or(|basis| {
                    basis.authority_evaluated_at_unix_ms != authority_time_unix_ms
                }) {
                    return Err("authority_decision_time_mismatch".to_string());
                }
                if expected != *decision {
                    return Err("authority_decision_basis_mismatch".to_string());
                }
            }
            TransitionPayload::ReviewRequested { review }
                if review.schema == crate::transition::REVIEW_REQUEST_SCHEMA =>
            {
                review.validate_policy_integrity()?;
                let history = self.list_case_transitions_txn(txn, case_id)?;
                let decision_state = state
                    .last_decision
                    .as_ref()
                    .filter(|decision| {
                        decision.outcome == crate::effect::DecisionOutcome::RequireReview
                            && decision.operation_id == review.operation_id
                    })
                    .ok_or_else(|| "policy_review_initial_decision_not_current".to_string())?;
                let decision =
                    Self::canonical_decision(state, &history, &decision_state.decision_id)?;
                let operation = Self::canonical_operation(state, &history, &review.operation_id)?;
                let expected =
                    build_policy_review_request(&operation, &decision, state.generation)?;
                if expected != *review {
                    return Err("authority_review_request_mismatch".to_string());
                }
            }
            TransitionPayload::ReviewActionRecorded { action } => {
                action.validate_integrity()?;
                let review = state
                    .reviews
                    .iter()
                    .find(|review| review.review_id == action.review_id)
                    .ok_or_else(|| "policy_review_request_not_current".to_string())?;
                if review.schema != crate::transition::REVIEW_REQUEST_SCHEMA {
                    return Ok(());
                }
                if review.operation_id != action.operation_id
                    || action.case_id != case_id
                    || action.expected_case_generation != state.generation
                    || !matches!(
                        review.status,
                        crate::transition::ReviewResolution::Pending
                            | crate::transition::ReviewResolution::Deferred
                    )
                    || !reviewer_is_eligible(state, review, &action.reviewer_participant_id)
                {
                    return Err("review_action_binding_or_generation_mismatch".to_string());
                }
                let effective = self.current_ready_effective_policy_txn(txn, case_id)?;
                if review.effective_policy_id != effective.effective_policy_id
                    || review.effective_policy_digest != effective.semantic_digest
                    || review.policy_binding_refs != effective.binding_ids
                {
                    return Err("policy_authority_basis_stale".to_string());
                }
            }
            TransitionPayload::ExecutionGrantIssued { grant }
                if grant.schema == crate::effect::EXECUTION_GRANT_SCHEMA =>
            {
                grant.validate_integrity()?;
                if authority_time_unix_ms >= grant.expires_at_unix_ms {
                    return Err("policy_execution_grant_expired_before_issuance".to_string());
                }
                let history = self.list_case_transitions_txn(txn, case_id)?;
                let Some(last_transition) = history.last() else {
                    return Err("policy_grant_decision_not_adjacent".to_string());
                };
                let TransitionPayload::DecisionRecorded { decision } = &last_transition.payload
                else {
                    return Err("policy_grant_decision_not_adjacent".to_string());
                };
                if decision.schema != crate::effect::DECISION_SCHEMA
                    || decision.outcome != crate::effect::DecisionOutcome::Allow
                    || last_transition.sequence != state.generation
                    || state.last_decision.as_ref().is_none_or(|current| {
                        current.decision_id != decision.decision_id
                            || current.recorded_at_generation != state.generation
                    })
                    || state.generation != decision.decided_at_case_generation + 1
                {
                    return Err("policy_grant_decision_not_adjacent".to_string());
                }
                let prior_history = &history[..history.len() - 1];
                let prior_state = replay_case(case_id, prior_history)?;
                let floor = self.authority_time_floor_txn(txn)?;
                let current_status = self.materialize_case_policy_at_txn(
                    txn,
                    case_id,
                    authority_time_unix_ms,
                    floor,
                )?;
                if current_status.readiness != NormativeReadiness::Ready
                    || current_status.validity != PolicyValidityPosture::Valid
                {
                    return Err(format!(
                        "policy_grant_requires_ready_and_valid: readiness={:?} validity={:?}",
                        current_status.readiness, current_status.validity
                    ));
                }
                let decision_authority_time = decision
                    .decision_basis
                    .as_ref()
                    .ok_or_else(|| "policy_decision_basis_missing".to_string())?
                    .authority_evaluated_at_unix_ms;
                let expected_decision = self.derive_expected_policy_decision_at_time_txn(
                    txn,
                    &prior_state,
                    prior_history,
                    decision,
                    Some(decision_authority_time),
                )?;
                if expected_decision != *decision {
                    return Err("policy_grant_decision_semantics_stale".to_string());
                }
                let operation =
                    Self::canonical_operation(&prior_state, prior_history, &decision.operation_id)?;
                let expected_grant =
                    issue_policy_execution_grant(&operation, decision, state.generation)?;
                if expected_grant != *grant {
                    return Err("policy_execution_grant_semantic_mismatch".to_string());
                }
            }
            TransitionPayload::EffectPrepared { prepared } => {
                let history = self.list_case_transitions_txn(txn, case_id)?;
                let grants = history
                    .iter()
                    .filter_map(|transition| match &transition.payload {
                        TransitionPayload::ExecutionGrantIssued { grant }
                            if grant.grant_id == prepared.grant_id =>
                        {
                            Some(grant)
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                let [grant] = grants.as_slice() else {
                    return Err("execution_obligation_grant_ambiguous_or_missing".to_string());
                };
                if grant.schema == crate::effect::EXECUTION_GRANT_SCHEMA {
                    if authority_time_unix_ms >= grant.expires_at_unix_ms {
                        return Err("execution_grant_expired_before_prepare".to_string());
                    }
                    let floor = self.authority_time_floor_txn(txn)?;
                    let status = self.materialize_case_policy_at_txn(
                        txn,
                        case_id,
                        authority_time_unix_ms,
                        floor,
                    )?;
                    if status.readiness != NormativeReadiness::Ready
                        || status.validity != PolicyValidityPosture::Valid
                    {
                        return Err(format!(
                            "policy_invalid_before_prepare: readiness={:?} validity={:?}",
                            status.readiness, status.validity
                        ));
                    }
                }
                validate_execution_obligation_preparation(grant, prepared)?;
            }
            TransitionPayload::EffectFinalized {
                effect_id,
                post_observation,
                receipt,
            } => {
                self.validate_execution_obligation_closure_txn(
                    txn,
                    case_id,
                    effect_id,
                    post_observation,
                    receipt,
                )?;
            }
            TransitionPayload::EffectReconciled {
                effect_id,
                observation,
                receipt: Some(receipt),
                ..
            } => {
                self.validate_execution_obligation_closure_txn(
                    txn,
                    case_id,
                    effect_id,
                    observation,
                    receipt,
                )?;
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_execution_obligation_closure_txn<T: Transaction>(
        &self,
        txn: &T,
        case_id: &str,
        effect_id: &str,
        post_observation: &crate::effect::FilesystemObservation,
        receipt: &crate::effect::EffectReceipt,
    ) -> Result<(), String> {
        let history = self.list_case_transitions_txn(txn, case_id)?;
        let prepared = history
            .iter()
            .find_map(|transition| match &transition.payload {
                TransitionPayload::EffectPrepared { prepared }
                    if prepared.effect_id == effect_id =>
                {
                    Some(prepared)
                }
                _ => None,
            })
            .ok_or_else(|| "execution_obligation_prepare_missing".to_string())?;
        let grant = history
            .iter()
            .find_map(|transition| match &transition.payload {
                TransitionPayload::ExecutionGrantIssued { grant }
                    if grant.grant_id == prepared.grant_id =>
                {
                    Some(grant)
                }
                _ => None,
            })
            .ok_or_else(|| "execution_obligation_grant_missing".to_string())?;
        validate_execution_obligation_closure(grant, prepared, post_observation, receipt)
    }

    pub fn get_transition_by_id(&self, transition_id: &str) -> Result<Option<Transition>, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start canonical transition read: {error}"))?;
        self.get_transition_by_id_txn(&txn, transition_id)
    }

    pub fn list_case_transitions(&self, case_id: &str) -> Result<Vec<Transition>, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start Case transition read: {error}"))?;
        self.list_case_transitions_txn(&txn, case_id)
    }

    pub fn get_case_state(&self, case_id: &str) -> Result<Option<CaseState>, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start CaseState read: {error}"))?;
        self.get_case_state_txn(&txn, case_id)
    }

    pub fn derive_and_commit_policy_decision(
        &self,
        case_id: &str,
        operation_id: &str,
    ) -> Result<(Decision, CanonicalCommit), String> {
        self.derive_and_commit_policy_decision_inner(case_id, operation_id, None)
    }

    pub fn derive_and_commit_policy_review_decision(
        &self,
        case_id: &str,
        operation_id: &str,
        review_id: &str,
        action_id: &str,
    ) -> Result<(Decision, CanonicalCommit), String> {
        self.derive_and_commit_policy_decision_inner(
            case_id,
            operation_id,
            Some((review_id, action_id)),
        )
    }

    fn derive_and_commit_policy_decision_inner(
        &self,
        case_id: &str,
        operation_id: &str,
        review: Option<(&str, &str)>,
    ) -> Result<(Decision, CanonicalCommit), String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start policy Decision transaction: {error}"))?;
        let state = self
            .get_case_state_txn(&txn, case_id)?
            .ok_or_else(|| format!("case_state_not_found: {case_id}"))?;
        let history = self.list_case_transitions_txn(&txn, case_id)?;
        let authority_time =
            self.advance_authority_time_txn(&mut txn, authority_wall_time_unix_ms())?;
        let decision = if let Some((review_id, action_id)) = review {
            self.derive_review_policy_decision_txn(
                &txn,
                &state,
                &history,
                operation_id,
                (review_id, action_id),
                Some(authority_time),
            )?
        } else {
            self.derive_initial_policy_decision_txn(
                &txn,
                &state,
                &history,
                operation_id,
                Some(authority_time),
            )?
        };
        let basis = decision
            .decision_basis
            .as_ref()
            .ok_or_else(|| "policy_decision_basis_missing".to_string())?;
        let mut causal_refs = vec![decision.operation_id.clone(), basis.basis_id.clone()];
        causal_refs.push(basis.effective_policy_id.clone());
        causal_refs.extend(basis.policy_binding_refs.iter().cloned());
        causal_refs.extend(basis.policy_artifact_refs.iter().cloned());
        if let Some(action) = &basis.review_action_ref {
            causal_refs.push(action.clone());
        }
        let mut pending = PendingTransition::new(
            format!("transition:policy-decision:{}", decision.decision_id),
            case_id,
            state.generation,
            TransitionSource {
                component: "yai.policy_authority_admission".to_string(),
                participant_id: Some(basis.proposer_participant_id.clone()),
                principal_id: None,
                source_ref: Some(decision.decision_id.clone()),
            },
            TransitionPayload::DecisionRecorded {
                decision: decision.clone(),
            },
        );
        pending.causal_refs = causal_refs;
        let commit =
            self.commit_transition_txn_at(&mut txn, pending, false, Some(authority_time), None)?;
        txn.commit()
            .map_err(|error| format!("failed to commit policy Decision: {error}"))?;
        Ok((decision, commit))
    }

    pub fn derive_policy_decision(
        &self,
        case_id: &str,
        operation_id: &str,
    ) -> Result<Decision, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start policy decision read: {error}"))?;
        let state = self
            .get_case_state_txn(&txn, case_id)?
            .ok_or_else(|| format!("case_state_not_found: {case_id}"))?;
        let history = self.list_case_transitions_txn(&txn, case_id)?;
        self.derive_initial_policy_decision_txn(&txn, &state, &history, operation_id, None)
    }

    pub fn derive_policy_review_decision(
        &self,
        case_id: &str,
        operation_id: &str,
        review_id: &str,
        action_id: &str,
    ) -> Result<Decision, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start policy review decision read: {error}"))?;
        let state = self
            .get_case_state_txn(&txn, case_id)?
            .ok_or_else(|| format!("case_state_not_found: {case_id}"))?;
        let history = self.list_case_transitions_txn(&txn, case_id)?;
        self.derive_review_policy_decision_txn(
            &txn,
            &state,
            &history,
            operation_id,
            (review_id, action_id),
            None,
        )
    }

    fn current_ready_effective_policy_txn<T: Transaction>(
        &self,
        txn: &T,
        case_id: &str,
    ) -> Result<EffectivePolicy, String> {
        let status = self.materialize_case_policy_txn(txn, case_id)?;
        status
            .effective_policy
            .filter(|_| {
                status.readiness == NormativeReadiness::Ready
                    && status.validity == PolicyValidityPosture::Valid
            })
            .ok_or_else(|| {
                format!(
                    "policy_authority_requires_ready_and_valid: readiness={:?} validity={:?}",
                    status.readiness, status.validity
                )
            })
    }

    fn canonical_operation(
        state: &CaseState,
        history: &[Transition],
        operation_id: &str,
    ) -> Result<Operation, String> {
        let current = state
            .last_operation
            .as_ref()
            .filter(|operation| operation.operation_id == operation_id)
            .ok_or_else(|| "authority_operation_not_current".to_string())?;
        let matches = history
            .iter()
            .filter_map(|transition| match &transition.payload {
                TransitionPayload::OperationRecorded { operation }
                    if operation.operation_id == operation_id =>
                {
                    Some(operation)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let [operation] = matches.as_slice() else {
            return Err("authority_operation_history_ambiguous_or_missing".to_string());
        };
        if operation.operation_digest != current.operation_digest
            || operation.case_id != state.case_id
        {
            return Err("authority_operation_history_mismatch".to_string());
        }
        operation.validate()?;
        Ok((*operation).clone())
    }

    fn canonical_decision(
        state: &CaseState,
        history: &[Transition],
        decision_id: &str,
    ) -> Result<Decision, String> {
        let current = state
            .last_decision
            .as_ref()
            .filter(|decision| decision.decision_id == decision_id)
            .ok_or_else(|| "authority_decision_not_current".to_string())?;
        let matches = history
            .iter()
            .filter_map(|transition| match &transition.payload {
                TransitionPayload::DecisionRecorded { decision }
                    if decision.decision_id == decision_id =>
                {
                    Some(decision)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let [decision] = matches.as_slice() else {
            return Err("authority_decision_history_ambiguous_or_missing".to_string());
        };
        if decision.decision_digest != current.decision_digest {
            return Err("authority_decision_history_mismatch".to_string());
        }
        decision.validate_integrity()?;
        Ok((*decision).clone())
    }

    fn derive_initial_policy_decision_txn<T: Transaction>(
        &self,
        txn: &T,
        state: &CaseState,
        history: &[Transition],
        operation_id: &str,
        authority_time_unix_ms: Option<u64>,
    ) -> Result<Decision, String> {
        let operation = Self::canonical_operation(state, history, operation_id)?;
        let resource = state
            .resources
            .iter()
            .find(|resource| resource.attachment_id == operation.resource_attachment_id)
            .ok_or_else(|| "authority_resource_not_attached".to_string())?;
        let status = if let Some(authority_time) = authority_time_unix_ms {
            let floor = self.authority_time_floor_txn(txn)?;
            self.materialize_case_policy_at_txn(txn, &state.case_id, authority_time, floor)?
        } else {
            self.materialize_case_policy_txn(txn, &state.case_id)?
        };
        let effective = status
            .effective_policy
            .clone()
            .filter(|_| {
                status.readiness == NormativeReadiness::Ready
                    && status.validity == PolicyValidityPosture::Valid
            })
            .ok_or_else(|| {
                format!(
                    "policy_authority_requires_ready_and_valid: readiness={:?} validity={:?}",
                    status.readiness, status.validity
                )
            })?;
        let temporal = AuthorityTemporalContext {
            authority_time_unix_ms: status.authority_time_unix_ms,
            binding_validity: status.binding_validity.into_values().collect(),
        };
        let evidence = resolve_canonical_evidence(&operation, history, None)?;
        evaluate_filesystem_admission(
            &operation, state, resource, &effective, &evidence, &temporal,
        )
    }

    fn derive_review_policy_decision_txn<T: Transaction>(
        &self,
        txn: &T,
        state: &CaseState,
        history: &[Transition],
        operation_id: &str,
        review_and_action: (&str, &str),
        authority_time_unix_ms: Option<u64>,
    ) -> Result<Decision, String> {
        let (review_id, action_id) = review_and_action;
        let operation = Self::canonical_operation(state, history, operation_id)?;
        let resource = state
            .resources
            .iter()
            .find(|resource| resource.attachment_id == operation.resource_attachment_id)
            .ok_or_else(|| "authority_resource_not_attached".to_string())?;
        let review = state
            .reviews
            .iter()
            .find(|review| {
                review.review_id == review_id
                    && review.operation_id == operation_id
                    && review.latest_action_id.as_deref() == Some(action_id)
            })
            .ok_or_else(|| "canonical_review_resolution_not_current".to_string())?;
        let actions = history
            .iter()
            .filter_map(|transition| match &transition.payload {
                TransitionPayload::ReviewActionRecorded { action }
                    if action.action_id == action_id =>
                {
                    Some(action)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let [action] = actions.as_slice() else {
            return Err("canonical_review_action_ambiguous_or_missing".to_string());
        };
        let status = if let Some(authority_time) = authority_time_unix_ms {
            let floor = self.authority_time_floor_txn(txn)?;
            self.materialize_case_policy_at_txn(txn, &state.case_id, authority_time, floor)?
        } else {
            self.materialize_case_policy_txn(txn, &state.case_id)?
        };
        let effective = status
            .effective_policy
            .clone()
            .filter(|_| {
                status.readiness == NormativeReadiness::Ready
                    && status.validity == PolicyValidityPosture::Valid
            })
            .ok_or_else(|| {
                format!(
                    "policy_authority_requires_ready_and_valid: readiness={:?} validity={:?}",
                    status.readiness, status.validity
                )
            })?;
        let temporal = AuthorityTemporalContext {
            authority_time_unix_ms: status.authority_time_unix_ms,
            binding_validity: status.binding_validity.into_values().collect(),
        };
        let evidence = resolve_canonical_evidence(&operation, history, Some(action_id))?;
        resolve_policy_review_decision(
            &operation, state, resource, &effective, review, action, &evidence, &temporal,
        )
    }

    fn derive_expected_policy_decision_at_time_txn<T: Transaction>(
        &self,
        txn: &T,
        state: &CaseState,
        history: &[Transition],
        proposed: &Decision,
        authority_time_unix_ms: Option<u64>,
    ) -> Result<Decision, String> {
        let basis = proposed
            .decision_basis
            .as_ref()
            .ok_or_else(|| "policy_decision_basis_missing".to_string())?;
        if let Some(action_id) = basis.review_action_ref.as_deref() {
            let review = state
                .reviews
                .iter()
                .find(|review| {
                    review.operation_id == proposed.operation_id
                        && review.latest_action_id.as_deref() == Some(action_id)
                })
                .ok_or_else(|| "canonical_review_resolution_not_current".to_string())?;
            self.derive_review_policy_decision_txn(
                txn,
                state,
                history,
                &proposed.operation_id,
                (&review.review_id, action_id),
                authority_time_unix_ms,
            )
        } else {
            self.derive_initial_policy_decision_txn(
                txn,
                state,
                history,
                &proposed.operation_id,
                authority_time_unix_ms,
            )
        }
    }

    /// Persists a machine-local resource binding. This database is required
    /// for carrier resolution after restart but is not canonical Case history.
    pub fn put_local_filesystem_binding(
        &self,
        binding: &LocalFilesystemBinding,
    ) -> Result<(), String> {
        self.put_local_filesystem_binding_inner(binding, None)
    }

    pub fn put_tenant_local_filesystem_binding(
        &self,
        authenticated: &AuthenticatedPrincipal,
        binding: &LocalFilesystemBinding,
    ) -> Result<(), String> {
        self.put_local_filesystem_binding_inner(binding, Some(authenticated))
    }

    pub fn commit_tenant_resource_attachment(
        &self,
        authenticated: &AuthenticatedPrincipal,
        tenant_id: &str,
        pending: PendingTransition,
        binding: &LocalFilesystemBinding,
    ) -> Result<CanonicalCommit, String> {
        binding.validate()?;
        if binding.case_id != pending.case_id
            || !matches!(pending.payload, TransitionPayload::ResourceAttached { .. })
        {
            return Err("secured_resource_attachment_input_mismatch".to_string());
        }
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start secured resource attachment: {error}"))?;
        let context = self.resolve_security_context_txn(&txn, authenticated, tenant_id)?;
        context.require_owner()?;
        self.validate_cross_tenant_root_txn(&txn, &context, binding)?;
        let commit =
            self.commit_transition_txn_at(&mut txn, pending, false, None, Some(&context))?;
        let key = local_binding_key(&binding.case_id, &binding.attachment_id);
        let value = serde_json::to_string(binding)
            .map_err(|error| format!("local_binding_encode_failed: {error}"))?;
        txn.put(
            self.local_resource_bindings,
            &key,
            &value,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to persist local resource binding: {error}"))?;
        txn.commit()
            .map_err(|error| format!("failed to commit secured resource attachment: {error}"))?;
        Ok(commit)
    }

    /// Atomically appends a Tenant-scoped process attachment and persists the
    /// exact kernel process-birth binding used by the carrier. The binding is
    /// operational resolution material; the attachment remains Case truth.
    pub fn commit_tenant_process_attachment(
        &self,
        authenticated: &AuthenticatedPrincipal,
        tenant_id: &str,
        pending: PendingTransition,
        binding: &LocalProcessBinding,
    ) -> Result<CanonicalCommit, String> {
        binding.validate()?;
        let attachment_id = match &pending.payload {
            TransitionPayload::ResourceAttached { attachment }
                if attachment.kind == crate::transition::ResourceKind::Process =>
            {
                &attachment.attachment_id
            }
            _ => return Err("secured_process_attachment_input_mismatch".to_string()),
        };
        if binding.case_id != pending.case_id || binding.attachment_id != *attachment_id {
            return Err("secured_process_attachment_binding_mismatch".to_string());
        }
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start secured process attachment: {error}"))?;
        let context = self.resolve_security_context_txn(&txn, authenticated, tenant_id)?;
        context.require_owner()?;
        let commit =
            self.commit_transition_txn_at(&mut txn, pending, false, None, Some(&context))?;
        put_json_txn(
            &mut txn,
            self.local_resource_bindings,
            &local_binding_key(&binding.case_id, &binding.attachment_id),
            binding,
            WriteFlags::empty(),
            "local_process_binding",
        )?;
        txn.commit()
            .map_err(|error| format!("failed to commit secured process attachment: {error}"))?;
        Ok(commit)
    }

    fn put_local_filesystem_binding_inner(
        &self,
        binding: &LocalFilesystemBinding,
        authenticated: Option<&AuthenticatedPrincipal>,
    ) -> Result<(), String> {
        binding.validate()?;
        let key = local_binding_key(&binding.case_id, &binding.attachment_id);
        let value = serde_json::to_string(binding)
            .map_err(|error| format!("local_binding_encode_failed: {error}"))?;
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start local binding write: {error}"))?;
        let state = self
            .get_case_state_txn(&txn, &binding.case_id)?
            .ok_or_else(|| "local_binding_case_state_missing".to_string())?;
        let security_context = self.resolve_case_owner_context_txn(
            &txn,
            &state,
            &authenticated
                .map(AuthenticatedPrincipal::projected_principal_id)
                .unwrap_or_default(),
            authenticated,
        )?;
        if let Some(context) = &security_context {
            self.validate_cross_tenant_root_txn(&txn, context, binding)?;
        }
        txn.put(
            self.local_resource_bindings,
            &key,
            &value,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to persist local resource binding: {error}"))?;
        txn.commit()
            .map_err(|error| format!("failed to commit local resource binding: {error}"))
    }

    fn validate_cross_tenant_root_txn<T: Transaction>(
        &self,
        txn: &T,
        context: &SecurityContext,
        binding: &LocalFilesystemBinding,
    ) -> Result<(), String> {
        let mut cursor = txn
            .open_ro_cursor(self.local_resource_bindings)
            .map_err(|error| format!("failed to inspect resource isolation roots: {error}"))?;
        let mut existing = Vec::new();
        for (_, raw) in cursor.iter() {
            let envelope: serde_json::Value = serde_json::from_slice(raw)
                .map_err(|error| format!("local_binding_decode_failed: {error}"))?;
            if !matches!(
                envelope.get("schema").and_then(serde_json::Value::as_str),
                Some(LOCAL_FILESYSTEM_BINDING_SCHEMA | LOCAL_FILESYSTEM_BINDING_SCHEMA_V1)
            ) {
                continue;
            }
            let value: LocalFilesystemBinding = serde_json::from_slice(raw)
                .map_err(|error| format!("local_binding_decode_failed: {error}"))?;
            existing.push(value);
        }
        drop(cursor);
        let proposed_root = Path::new(&binding.canonical_root);
        for value in existing {
            if value.case_id == binding.case_id && value.attachment_id == binding.attachment_id {
                continue;
            }
            let Some(other_state) = self.get_case_state_txn(txn, &value.case_id)? else {
                return Err("local_binding_dangling_case_state".to_string());
            };
            if let Some(other_tenant) = other_state.tenant_id.as_deref() {
                let existing_root = Path::new(&value.canonical_root);
                if other_tenant != context.tenant_id()
                    && (proposed_root.starts_with(existing_root)
                        || existing_root.starts_with(proposed_root))
                {
                    return Err(format!(
                        "cross_tenant_filesystem_root_overlap: conflicting_case={}",
                        value.case_id
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn get_local_filesystem_binding(
        &self,
        case_id: &str,
        attachment_id: &str,
    ) -> Result<Option<LocalFilesystemBinding>, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start local binding read: {error}"))?;
        self.local_filesystem_binding_txn(&txn, case_id, attachment_id)
    }

    fn local_filesystem_binding_txn<T: Transaction>(
        &self,
        txn: &T,
        case_id: &str,
        attachment_id: &str,
    ) -> Result<Option<LocalFilesystemBinding>, String> {
        let key = local_binding_key(case_id, attachment_id);
        match txn.get(self.local_resource_bindings, &key) {
            Ok(value) => {
                let binding: LocalFilesystemBinding = serde_json::from_slice(value)
                    .map_err(|error| format!("local_binding_decode_failed: {error}"))?;
                binding.validate()?;
                Ok(Some(binding))
            }
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!("failed to read local resource binding: {error}")),
        }
    }

    pub fn get_local_process_binding(
        &self,
        case_id: &str,
        attachment_id: &str,
    ) -> Result<Option<LocalProcessBinding>, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start local process binding read: {error}"))?;
        self.local_process_binding_txn(&txn, case_id, attachment_id)
    }

    fn local_process_binding_txn<T: Transaction>(
        &self,
        txn: &T,
        case_id: &str,
        attachment_id: &str,
    ) -> Result<Option<LocalProcessBinding>, String> {
        let key = local_binding_key(case_id, attachment_id);
        match txn.get(self.local_resource_bindings, &key) {
            Ok(value) => {
                let binding: LocalProcessBinding = serde_json::from_slice(value)
                    .map_err(|error| format!("local_process_binding_decode_failed: {error}"))?;
                binding.validate()?;
                Ok(Some(binding))
            }
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!("failed to read local process binding: {error}")),
        }
    }

    fn resource_control_state_txn<T: Transaction>(
        &self,
        txn: &T,
        resource_id: &str,
    ) -> Result<Option<ResourceControlState>, String> {
        let key = resource_control_state_key(resource_id);
        match txn.get(self.resource_control_states_by_id, &key) {
            Ok(raw) => {
                let state: ResourceControlState = serde_json::from_slice(raw)
                    .map_err(|error| format!("resource_control_state_decode_failed: {error}"))?;
                state.validate()?;
                Ok(Some(state))
            }
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!("failed to read resource control state: {error}")),
        }
    }

    fn put_resource_control_state_txn(
        &self,
        txn: &mut RwTransaction<'_>,
        state: &ResourceControlState,
    ) -> Result<(), String> {
        state.validate()?;
        put_json_txn(
            txn,
            self.resource_control_states_by_id,
            &resource_control_state_key(&state.identity.resource_id),
            state,
            WriteFlags::empty(),
            "resource_control_state",
        )
    }

    fn resource_control_events_txn<T: Transaction>(
        &self,
        txn: &T,
        resource_id: &str,
    ) -> Result<Vec<ResourceControlEvent>, String> {
        let mut cursor = txn
            .open_ro_cursor(self.resource_control_events_by_id)
            .map_err(|error| format!("failed to open resource event cursor: {error}"))?;
        let mut events = Vec::new();
        for (_, raw) in cursor.iter() {
            let event: ResourceControlEvent = serde_json::from_slice(raw)
                .map_err(|error| format!("resource_control_event_decode_failed: {error}"))?;
            event.validate_integrity()?;
            if event.resource_id == resource_id {
                events.push(event);
            }
        }
        events.sort_by_key(|event| event.sequence);
        Ok(events)
    }

    fn resource_control_event_at_sequence_txn<T: Transaction>(
        &self,
        txn: &T,
        resource_id: &str,
        sequence: Option<u64>,
    ) -> Result<Option<ResourceControlEvent>, String> {
        let Some(sequence) = sequence else {
            return Ok(None);
        };
        let matches = self
            .resource_control_events_txn(txn, resource_id)?
            .into_iter()
            .filter(|event| event.sequence == sequence)
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [] => Err("resource_history_predecessor_missing".to_string()),
            [event] => Ok(Some(event.clone())),
            _ => Err("resource_history_duplicate_sequence".to_string()),
        }
    }

    fn validate_resource_event_append_txn<T: Transaction>(
        &self,
        txn: &T,
        event: &ResourceControlEvent,
    ) -> Result<(), String> {
        event.validate_integrity()?;
        let mut events = self.resource_control_events_txn(txn, &event.resource_id)?;
        if events
            .iter()
            .any(|current| current.sequence == event.sequence)
        {
            return Err("resource_history_duplicate_sequence".to_string());
        }
        if events
            .iter()
            .all(|current| current.schema == RESOURCE_CONTROL_EVENT_SCHEMA)
        {
            events.push(event.clone());
            replay_resource_control_state(&events)?;
            return Ok(());
        }

        // Compatibility posture for an already-persisted v1 prefix. The v1
        // event lacks a full identity/fence and cannot support from-zero
        // rebuild, but new appends still require its exact head and the
        // materialized state predecessor. Historical bytes are not rewritten.
        let prior = events
            .last()
            .ok_or_else(|| "resource_history_predecessor_missing".to_string())?;
        if event.sequence != prior.sequence.saturating_add(1)
            || event.previous_event_id.as_deref() != Some(prior.event_id.as_str())
            || event.previous_event_digest.as_deref() != Some(prior.integrity_digest.as_str())
        {
            return Err("resource_history_predecessor_mismatch".to_string());
        }
        let current = self
            .resource_control_state_txn(txn, &event.resource_id)?
            .ok_or_else(|| "resource_control_state_missing_for_event_append".to_string())?;
        let identity = event
            .resource_identity
            .as_ref()
            .ok_or_else(|| "resource_event_identity_missing".to_string())?;
        let fence = event
            .fence
            .as_ref()
            .ok_or_else(|| "resource_event_fence_missing".to_string())?;
        if current.identity != *identity {
            return Err("resource_history_identity_changed".to_string());
        }
        match event.action {
            ResourceControlAction::Acquired => {
                if current.active_lease.is_some()
                    || event.resource_epoch != current.resource_epoch.saturating_add(1)
                {
                    return Err("resource_history_invalid_acquisition".to_string());
                }
            }
            ResourceControlAction::Reclaimed => {
                let active = current
                    .active_lease
                    .as_ref()
                    .ok_or_else(|| "resource_history_reclaim_without_active_lease".to_string())?;
                if event.resource_epoch != current.resource_epoch.saturating_add(1)
                    || active.fence.effect_id != fence.effect_id
                    || active.fence.case_id != fence.case_id
                    || active.fence.grant_id != fence.grant_id
                {
                    return Err("resource_history_invalid_reclaim".to_string());
                }
            }
            ResourceControlAction::Released => {
                if event.resource_epoch != current.resource_epoch
                    || current.active_lease.as_ref().map(|lease| &lease.fence) != Some(fence)
                {
                    return Err("resource_history_invalid_release".to_string());
                }
            }
        }
        Ok(())
    }

    fn put_resource_control_event_txn(
        &self,
        txn: &mut RwTransaction<'_>,
        event: &ResourceControlEvent,
    ) -> Result<(), String> {
        self.validate_resource_event_append_txn(txn, event)?;
        put_json_txn(
            txn,
            self.resource_control_events_by_id,
            &resource_control_event_key(&event.event_id),
            event,
            WriteFlags::NO_OVERWRITE,
            "resource_control_event",
        )
    }

    fn reject_active_resource_conflict_txn<T: Transaction>(
        &self,
        txn: &T,
        proposed: &ResourceIdentity,
    ) -> Result<(), String> {
        proposed.validate()?;
        let mut cursor = txn
            .open_ro_cursor(self.resource_control_states_by_id)
            .map_err(|error| format!("failed to inspect resource controls: {error}"))?;
        for (_, raw) in cursor.iter() {
            let state: ResourceControlState = serde_json::from_slice(raw)
                .map_err(|error| format!("resource_control_state_decode_failed: {error}"))?;
            state.validate()?;
            if state.identity.tenant_id != proposed.tenant_id
                || state.active_lease.is_none()
                || state.identity.resource_kind != proposed.resource_kind
            {
                continue;
            }
            let conflict = match proposed.resource_kind {
                crate::resource_control::ControlledResourceKind::Filesystem => !matches!(
                    filesystem_relation(
                        &proposed.canonical_identity,
                        &state.identity.canonical_identity,
                    ),
                    FilesystemRelation::Disjoint
                ),
                crate::resource_control::ControlledResourceKind::Process => {
                    proposed.canonical_identity == state.identity.canonical_identity
                }
            };
            if conflict {
                let active = state.active_lease.expect("checked active lease");
                return Err(format!(
                    "resource_temporarily_owned: resource_id={} epoch={} case_id={} effect_id={}",
                    state.identity.resource_id,
                    state.resource_epoch,
                    active.fence.case_id,
                    active.fence.effect_id
                ));
            }
        }
        Ok(())
    }

    fn validate_carrier_fence_txn<T: Transaction>(
        &self,
        txn: &T,
        fence: &ResourceFence,
        require_live_owner: bool,
    ) -> Result<(), String> {
        fence.validate_integrity()?;
        let state = self
            .resource_control_state_txn(txn, &fence.resource_id)?
            .ok_or_else(|| "carrier_resource_control_state_missing".to_string())?;
        let current = state
            .active_lease
            .as_ref()
            .ok_or_else(|| "carrier_resource_not_owned".to_string())?;
        if current.fence != *fence
            || state.resource_epoch != fence.resource_epoch
            || state.identity.tenant_id != fence.tenant_id
            || state.identity.resource_kind != fence.resource_kind
        {
            return Err(format!(
                "stale_resource_fence: requested_epoch={} current_epoch={}",
                fence.resource_epoch, state.resource_epoch
            ));
        }
        if require_live_owner {
            let current = LocalProcessIdentity::capture(std::process::id())?;
            if fence.owner_pid != current.pid
                || fence.owner_process_identity != current.canonical_identity()
            {
                return Err("resource_fence_owner_process_mismatch".to_string());
            }
            if !resource_owner_is_live(fence) {
                return Err("resource_fence_owner_process_not_live".to_string());
            }
        }
        Ok(())
    }

    pub fn replay_case_state(&self, case_id: &str) -> Result<CaseState, String> {
        let transitions = self.list_case_transitions(case_id)?;
        replay_case(case_id, &transitions)
    }

    pub fn verify_case_state(&self, case_id: &str) -> Result<bool, String> {
        let materialized = self
            .get_case_state(case_id)?
            .ok_or_else(|| format!("case_state_not_found: {case_id}"))?;
        Ok(materialized == self.replay_case_state(case_id)?)
    }

    pub fn rebuild_case_state(&self, case_id: &str) -> Result<CaseState, String> {
        let transitions = self.list_case_transitions(case_id)?;
        let rebuilt = replay_case(case_id, &transitions)?;
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start CaseState rebuild: {error}"))?;
        let actual_sequence = self.last_case_sequence_txn(&txn, case_id)?;
        if actual_sequence != rebuilt.generation {
            return Err(format!(
                "case_history_changed_during_rebuild: expected={} actual={actual_sequence}",
                rebuilt.generation
            ));
        }
        let state_key = case_state_key(case_id);
        let state_json = rebuilt.to_json()?;
        txn.put(
            self.case_state,
            &state_key,
            &state_json,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to replace rebuilt CaseState: {error}"))?;
        txn.commit()
            .map_err(|error| format!("failed to commit rebuilt CaseState: {error}"))?;
        Ok(rebuilt)
    }

    /// Imports legacy bytes as compatibility evidence only. This never appends
    /// a canonical Transition and never materializes CaseState.
    pub fn import_legacy_compatibility(
        &self,
        contents: &str,
        source_ref: &str,
    ) -> Result<LegacyCompatibilityImportReport, String> {
        let corpus = inspect_legacy_jsonl(contents);
        let mut report = LegacyCompatibilityImportReport {
            lines_total: corpus.lines_total,
            losslessly_promoted: corpus.losslessly_promoted,
            promoted_with_metadata: corpus.promoted_with_metadata,
            preserved_opaque: corpus.preserved_opaque,
            rejected_malformed: corpus.rejected_malformed,
            repeated_record_ids: corpus.repeated_record_ids,
            ..Default::default()
        };
        let source_key = relation_id_component(source_ref);
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start legacy compatibility import: {error}"))?;
        for entry in corpus
            .entries
            .iter()
            .filter(|entry| entry.disposition != "rejected_malformed")
        {
            let key = format!("legacy:{source_key}:{:020}", entry.line_number);
            let value = serde_json::json!({
                "schema": LEGACY_COMPATIBILITY_SCHEMA,
                "source_ref": source_ref,
                "line_number": entry.line_number,
                "disposition": entry.disposition,
                "origin_schema": entry.schema,
                "record_id": entry.record_id,
                "record_kind": entry.record_kind,
                "reason": entry.reason,
                "raw_json": entry.raw_json,
            })
            .to_string();
            match txn.put(
                self.legacy_compatibility_payloads,
                &key,
                &value,
                WriteFlags::NO_OVERWRITE,
            ) {
                Ok(()) => report.payloads_written += 1,
                Err(Error::KeyExist) => report.payloads_duplicate += 1,
                Err(error) => {
                    return Err(format!(
                        "failed to preserve legacy compatibility line {}: {error}",
                        entry.line_number
                    ));
                }
            }
        }
        txn.commit()
            .map_err(|error| format!("failed to commit legacy compatibility import: {error}"))?;
        Ok(report)
    }

    pub fn legacy_compatibility_payload_count(&self) -> Result<usize, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to read legacy compatibility store: {error}"))?;
        count_entries(&txn, self.legacy_compatibility_payloads)
    }

    pub fn import_journal(&self, journal: &Journal, journal_ref: &str) -> Result<(), String> {
        self.import_journal_with_report(journal, journal_ref)
            .map(|_| ())
    }

    pub fn import_journal_with_report(
        &self,
        journal: &Journal,
        journal_ref: &str,
    ) -> Result<JournalImportReport, String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start LMDB journal import: {error}"))?;
        let mut report = JournalImportReport::default();
        for (index, record) in journal.records().iter().enumerate() {
            report.records_seen += 1;
            let source_ref = format!("{journal_ref}#{}", index + 1);
            if self.record_exists_txn(&txn, &record.id)? {
                report.records_duplicate += 1;
                continue;
            }
            self.put_record(&mut txn, record, &source_ref)?;
            report.records_written += 1;
        }
        txn.commit()
            .map_err(|error| format!("failed to commit LMDB journal import: {error}"))?;
        Ok(report)
    }

    pub fn put_replay_metadata(&self, metadata: &ReplayMetadata) -> Result<(), String> {
        let key = replay_metadata_key(&metadata.journal_identity);
        let value = metadata.to_json();
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start LMDB replay metadata write: {error}"))?;
        txn.put(self.schema_meta, &key, &value, WriteFlags::empty())
            .map_err(|error| {
                format!(
                    "failed to write replay metadata for {}: {error}",
                    metadata.journal_identity
                )
            })?;
        txn.commit()
            .map_err(|error| format!("failed to commit LMDB replay metadata: {error}"))
    }

    pub fn replay_metadata(
        &self,
        journal_identity: &str,
    ) -> Result<Option<ReplayMetadata>, String> {
        let key = replay_metadata_key(journal_identity);
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start LMDB replay metadata read: {error}"))?;
        match txn.get(self.schema_meta, &key) {
            Ok(value) => ReplayMetadata::from_bytes(value).map(Some),
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!(
                "failed to read replay metadata for {journal_identity}: {error}"
            )),
        }
    }

    pub fn summary(&self) -> Result<RecordStoreSummary, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start LMDB read transaction: {error}"))?;
        Ok(RecordStoreSummary {
            records_total: count_entries(&txn, self.records_by_id)?,
            records_by_case: count_entries(&txn, self.records_by_case)?,
            records_by_kind: count_entries(&txn, self.records_by_kind)?,
            records_by_subject: count_entries(&txn, self.records_by_subject)?,
            records_by_receipt: count_entries(&txn, self.records_by_receipt)?,
        })
    }

    pub fn canonical_summary(&self) -> Result<CanonicalAuthoritySummary, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start canonical summary read: {error}"))?;
        Ok(CanonicalAuthoritySummary {
            transitions_total: count_entries(&txn, self.transitions_by_id)?,
            cases_materialized: count_entries(&txn, self.case_state)?,
            legacy_compatibility_payloads: count_entries(&txn, self.legacy_compatibility_payloads)?,
        })
    }

    pub fn get_record_by_id(
        &self,
        record_id: &str,
    ) -> Result<Option<StoredRecordEnvelope>, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start LMDB record read: {error}"))?;
        self.get_record_by_id_txn(&txn, record_id)
    }

    pub fn list_records_by_case(
        &self,
        case_ref: &str,
        limit: usize,
    ) -> Result<RecordListResult, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start LMDB case index read: {error}"))?;
        let prefix = format!("record:case:{case_ref}:");
        self.list_records_by_index(&txn, self.records_by_case, &prefix, limit)
    }

    pub fn list_records_by_kind(
        &self,
        record_kind: &str,
        limit: usize,
    ) -> Result<RecordListResult, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start LMDB kind index read: {error}"))?;
        let prefix = format!("record:kind:{record_kind}:");
        self.list_records_by_index(&txn, self.records_by_kind, &prefix, limit)
    }

    pub fn list_records_by_subject(
        &self,
        subject_ref: &str,
        limit: usize,
    ) -> Result<RecordListResult, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start LMDB subject index read: {error}"))?;
        let prefix = format!("record:subject:{subject_ref}:");
        self.list_records_by_index(&txn, self.records_by_subject, &prefix, limit)
    }

    pub fn list_records_by_receipt(
        &self,
        receipt_ref: &str,
        limit: usize,
    ) -> Result<RecordListResult, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start LMDB receipt index read: {error}"))?;
        let prefix = format!("record:receipt:{receipt_ref}:");
        self.list_records_by_index(&txn, self.records_by_receipt, &prefix, limit)
    }

    pub fn materialize_graph_relations_for_case(
        &self,
        case_ref: &str,
    ) -> Result<GraphMaterializeReport, String> {
        self.materialize_graph_relations_for_case_inner(case_ref, false)
    }

    pub fn rebuild_graph_relations_for_case(
        &self,
        case_ref: &str,
    ) -> Result<GraphMaterializeReport, String> {
        self.clear_graph_relations_for_case(case_ref)?;
        self.materialize_graph_relations_for_case(case_ref)
    }

    fn clear_graph_relations_for_case(&self, case_ref: &str) -> Result<usize, String> {
        let prefix = format!("graph_relation:case:{case_ref}:");
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start graph relation clear: {error}"))?;
        let mut cursor = txn
            .open_ro_cursor(self.graph_relations_by_case)
            .map_err(|error| format!("failed to open graph relation clear cursor: {error}"))?;
        let mut entries = Vec::new();
        for (key, value) in cursor.iter() {
            if !key.starts_with(prefix.as_bytes()) {
                continue;
            }
            let relation_id = std::str::from_utf8(value)
                .map_err(|error| format!("invalid graph relation identity utf8: {error}"))?;
            entries.push((key.to_vec(), relation_id.to_string()));
        }
        drop(cursor);
        for (case_key, relation_id) in &entries {
            let id_key = format!("graph_relation:id:{relation_id}");
            let relation = match txn.get(self.graph_relations_by_id, &id_key) {
                Ok(value) => Some(GraphRelation::from_bytes(value)?),
                Err(Error::NotFound) => None,
                Err(error) => {
                    return Err(format!(
                        "failed to inspect graph relation {relation_id} for clear: {error}"
                    ));
                }
            };
            txn.del(self.graph_relations_by_case, case_key, None)
                .map_err(|error| format!("failed to clear graph case index: {error}"))?;
            match txn.del(self.graph_relations_by_id, &id_key, None) {
                Ok(()) | Err(Error::NotFound) => {}
                Err(error) => return Err(format!("failed to clear graph relation: {error}")),
            }
            if let Some(relation) = relation {
                let kind_key = format!(
                    "graph_relation:kind:{}:{}",
                    relation.edge_kind, relation.relation_id
                );
                match txn.del(self.graph_relations_by_kind, &kind_key, None) {
                    Ok(()) | Err(Error::NotFound) => {}
                    Err(error) => {
                        return Err(format!("failed to clear graph kind index: {error}"));
                    }
                }
            }
        }
        txn.commit()
            .map_err(|error| format!("failed to commit graph relation clear: {error}"))?;
        Ok(entries.len())
    }

    fn materialize_graph_relations_for_case_inner(
        &self,
        case_ref: &str,
        inject_failure_before_commit: bool,
    ) -> Result<GraphMaterializeReport, String> {
        let source_transitions = self.list_case_transitions(case_ref)?;
        let source_records = self.list_records_by_case(case_ref, usize::MAX)?;
        let created_at_unix_ms = unix_time_ms();
        let mut candidates = Vec::new();
        let mut skipped = 0usize;
        for transition in &source_transitions {
            let mut derived = derive_graph_relations_from_transition(transition, &mut skipped);
            candidates.append(&mut derived);
        }
        for record in &source_records.records {
            let mut derived = derive_graph_relations(record, created_at_unix_ms, &mut skipped);
            candidates.append(&mut derived);
        }

        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start LMDB graph relation write: {error}"))?;
        let mut report = GraphMaterializeReport {
            relations_seen: candidates.len(),
            relations_skipped: skipped,
            ..Default::default()
        };
        for relation in candidates {
            if self.graph_relation_exists_txn(&txn, &relation.relation_id)? {
                report.relations_duplicate += 1;
                continue;
            }
            self.put_graph_relation(&mut txn, &relation)?;
            report.relations_written += 1;
        }
        if inject_failure_before_commit {
            return Err("injected_graph_materialization_failure".to_string());
        }
        txn.commit()
            .map_err(|error| format!("failed to commit LMDB graph relations: {error}"))?;
        Ok(report)
    }

    pub fn list_graph_relations_by_case(
        &self,
        case_ref: &str,
        limit: usize,
    ) -> Result<GraphRelationListResult, String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start LMDB graph relation read: {error}"))?;
        let prefix = format!("graph_relation:case:{case_ref}:");
        self.list_graph_relations_by_index(&txn, self.graph_relations_by_case, &prefix, limit)
    }

    pub fn load_runtime_graph_for_case(
        &self,
        case_ref: &str,
    ) -> Result<RuntimeGraphLoadResult, String> {
        let relations = self.list_graph_relations_by_case(case_ref, usize::MAX)?;
        let generation = 1;
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut outgoing_refs: Vec<String> = Vec::new();
        let mut incoming_refs: Vec<String> = Vec::new();

        for relation in relations.relations {
            push_runtime_node(
                &mut nodes,
                RuntimeGraphNode {
                    node_ref: relation.from_ref.clone(),
                    node_kind: relation.from_kind.clone(),
                    case_ref: relation.case_ref.clone(),
                    source_record_ref: relation.source_record_id.clone(),
                    generation,
                },
            );
            push_runtime_node(
                &mut nodes,
                RuntimeGraphNode {
                    node_ref: relation.to_ref.clone(),
                    node_kind: relation.to_kind.clone(),
                    case_ref: relation.case_ref.clone(),
                    source_record_ref: relation.source_record_id.clone(),
                    generation,
                },
            );
            push_unique_string(&mut outgoing_refs, &relation.from_ref);
            push_unique_string(&mut incoming_refs, &relation.to_ref);
            edges.push(RuntimeGraphEdge {
                relation_id: relation.relation_id,
                edge_kind: relation.edge_kind,
                from_ref: relation.from_ref,
                to_ref: relation.to_ref,
                case_ref: relation.case_ref,
                source_record_id: relation.source_record_id,
                generation,
            });
        }

        Ok(RuntimeGraphLoadResult {
            case_ref: case_ref.to_string(),
            nodes_total: nodes.len(),
            edges_total: edges.len(),
            outgoing_index_entries: outgoing_refs.len(),
            incoming_index_entries: incoming_refs.len(),
            generation,
            dirty: false,
            stale: false,
            source: "graph_relations",
            durable_truth: "canonical_transition_or_legacy_compatibility_input",
            nodes,
            edges,
        })
    }

    fn ensure_schema(&self) -> Result<(), String> {
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start LMDB schema transaction: {error}"))?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:canonical_transition_schema",
            TRANSITION_SCHEMA,
            &[
                TRANSITION_SCHEMA_V10,
                TRANSITION_SCHEMA_V9,
                TRANSITION_SCHEMA_V8,
                TRANSITION_SCHEMA_V7,
                TRANSITION_SCHEMA_V6,
                TRANSITION_SCHEMA_V5,
                TRANSITION_SCHEMA_V4,
                TRANSITION_SCHEMA_V3,
                TRANSITION_SCHEMA_V2,
                TRANSITION_SCHEMA_V1,
            ],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:case_state_schema",
            CASE_STATE_SCHEMA,
            &[
                CASE_STATE_SCHEMA_V10,
                CASE_STATE_SCHEMA_V9,
                CASE_STATE_SCHEMA_V8,
                CASE_STATE_SCHEMA_V7,
                CASE_STATE_SCHEMA_V6,
                CASE_STATE_SCHEMA_V5,
                CASE_STATE_SCHEMA_V4,
                CASE_STATE_SCHEMA_V3,
                CASE_STATE_SCHEMA_V2,
                CASE_STATE_SCHEMA_V1,
            ],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:legacy_compatibility_schema",
            LEGACY_COMPATIBILITY_SCHEMA,
            &[],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:case_runtime_admission_schema",
            CASE_RUNTIME_ADMISSION_SCHEMA,
            &[],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:runtime_instance_schema",
            RUNTIME_INSTANCE_SCHEMA,
            &[RUNTIME_INSTANCE_SCHEMA_V1],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:runtime_work_item_schema",
            RUNTIME_WORK_ITEM_SCHEMA,
            &[RUNTIME_WORK_ITEM_SCHEMA_V1],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:local_filesystem_binding_schema",
            LOCAL_FILESYSTEM_BINDING_SCHEMA,
            &[LOCAL_FILESYSTEM_BINDING_SCHEMA_V1],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:semantic_context_artifact_schema",
            SEMANTIC_CONTEXT_ARTIFACT_SCHEMA,
            &[],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:operational_memory_schema",
            OPERATIONAL_MEMORY_SCHEMA,
            &[],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:operational_memory_manifest_schema",
            OPERATIONAL_MEMORY_MANIFEST_SCHEMA,
            &[],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:operational_memory_derivation",
            OPERATIONAL_MEMORY_DERIVATION,
            &[],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:policy_source_artifact_schema",
            POLICY_SOURCE_ARTIFACT_SCHEMA,
            &[
                POLICY_SOURCE_ARTIFACT_SCHEMA_V1,
                POLICY_SOURCE_ARTIFACT_SCHEMA_V2,
                POLICY_SOURCE_ARTIFACT_SCHEMA_V3,
            ],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:policy_artifact_schema",
            POLICY_ARTIFACT_SCHEMA,
            &[
                POLICY_ARTIFACT_SCHEMA_V4,
                POLICY_ARTIFACT_SCHEMA_V1,
                POLICY_ARTIFACT_SCHEMA_V2,
                POLICY_ARTIFACT_SCHEMA_V3,
            ],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:policy_lifecycle_event_schema",
            POLICY_LIFECYCLE_EVENT_SCHEMA,
            &[
                POLICY_LIFECYCLE_EVENT_SCHEMA_V2,
                POLICY_LIFECYCLE_EVENT_SCHEMA_V1,
            ],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:case_policy_binding_schema",
            CASE_POLICY_BINDING_SCHEMA,
            &[CASE_POLICY_BINDING_SCHEMA_V1],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:effective_policy_schema",
            EFFECTIVE_POLICY_SCHEMA,
            &[EFFECTIVE_POLICY_SCHEMA_V2, EFFECTIVE_POLICY_SCHEMA_V1],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:policy_materializer_version",
            POLICY_MATERIALIZER_VERSION,
            &[
                POLICY_MATERIALIZER_VERSION_V2,
                POLICY_MATERIALIZER_VERSION_V1,
            ],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:security_principal_schema",
            SECURITY_PRINCIPAL_SCHEMA,
            &[],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:tenant_schema",
            TENANT_SCHEMA,
            &[],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:security_event_schema",
            SECURITY_EVENT_SCHEMA,
            &[],
        )?;
        ensure_meta_upgradeable(
            &txn,
            self.schema_meta,
            "meta:workflow_definition_schema",
            WORKFLOW_DEFINITION_SCHEMA,
            &[WORKFLOW_DEFINITION_SCHEMA_V1],
        )?;
        for (key, value) in [
            ("meta:canonical_transition_schema", TRANSITION_SCHEMA),
            ("meta:case_state_schema", CASE_STATE_SCHEMA),
            (
                "meta:legacy_compatibility_schema",
                LEGACY_COMPATIBILITY_SCHEMA,
            ),
            (
                "meta:local_filesystem_binding_schema",
                LOCAL_FILESYSTEM_BINDING_SCHEMA,
            ),
            (
                "meta:semantic_context_artifact_schema",
                SEMANTIC_CONTEXT_ARTIFACT_SCHEMA,
            ),
            ("meta:operational_memory_schema", OPERATIONAL_MEMORY_SCHEMA),
            (
                "meta:operational_memory_manifest_schema",
                OPERATIONAL_MEMORY_MANIFEST_SCHEMA,
            ),
            (
                "meta:operational_memory_derivation",
                OPERATIONAL_MEMORY_DERIVATION,
            ),
            (
                "meta:case_runtime_admission_schema",
                CASE_RUNTIME_ADMISSION_SCHEMA,
            ),
            ("meta:runtime_instance_schema", RUNTIME_INSTANCE_SCHEMA),
            ("meta:runtime_work_item_schema", RUNTIME_WORK_ITEM_SCHEMA),
            (
                "meta:policy_source_artifact_schema",
                POLICY_SOURCE_ARTIFACT_SCHEMA,
            ),
            ("meta:policy_artifact_schema", POLICY_ARTIFACT_SCHEMA),
            (
                "meta:policy_lifecycle_event_schema",
                POLICY_LIFECYCLE_EVENT_SCHEMA,
            ),
            (
                "meta:case_policy_binding_schema",
                CASE_POLICY_BINDING_SCHEMA,
            ),
            ("meta:effective_policy_schema", EFFECTIVE_POLICY_SCHEMA),
            (
                "meta:policy_materializer_version",
                POLICY_MATERIALIZER_VERSION,
            ),
            ("meta:security_principal_schema", SECURITY_PRINCIPAL_SCHEMA),
            ("meta:tenant_schema", TENANT_SCHEMA),
            ("meta:security_event_schema", SECURITY_EVENT_SCHEMA),
            (
                "meta:workflow_definition_schema",
                WORKFLOW_DEFINITION_SCHEMA,
            ),
        ] {
            txn.put(self.schema_meta, &key, &value, WriteFlags::empty())
                .map_err(|error| format!("failed to write persisted schema {key}: {error}"))?;
        }
        txn.put(
            self.schema_meta,
            &"meta:schema",
            &RECORD_SCHEMA,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to write LMDB schema meta: {error}"))?;
        txn.put(
            self.schema_meta,
            &"meta:rebuild",
            &"not_started",
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to write LMDB rebuild meta: {error}"))?;
        txn.put(
            self.schema_meta,
            &"meta:graph_relation_schema",
            &GRAPH_RELATION_SCHEMA,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to write LMDB graph relation schema meta: {error}"))?;
        txn.commit()
            .map_err(|error| format!("failed to commit LMDB schema meta: {error}"))
    }

    fn get_transition_by_id_txn<T: Transaction>(
        &self,
        txn: &T,
        transition_id: &str,
    ) -> Result<Option<Transition>, String> {
        let key = transition_id_key(transition_id);
        match txn.get(self.transitions_by_id, &key) {
            Ok(value) => {
                let json = std::str::from_utf8(value)
                    .map_err(|error| format!("invalid canonical Transition utf8: {error}"))?;
                Transition::from_json(json).map(Some)
            }
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!(
                "failed to read canonical Transition {transition_id}: {error}"
            )),
        }
    }

    fn list_case_transitions_txn<T: Transaction>(
        &self,
        txn: &T,
        case_id: &str,
    ) -> Result<Vec<Transition>, String> {
        let prefix = case_sequence_prefix(case_id);
        let mut cursor = txn
            .open_ro_cursor(self.case_transition_sequence)
            .map_err(|error| format!("failed to open Case sequence cursor: {error}"))?;
        let mut identities = Vec::new();
        for (key, value) in cursor.iter() {
            if key.starts_with(prefix.as_bytes()) {
                let transition_id = std::str::from_utf8(value)
                    .map_err(|error| format!("invalid Case sequence identity utf8: {error}"))?;
                identities.push((key.to_vec(), transition_id.to_string()));
            }
        }
        drop(cursor);
        identities.sort_by(|left, right| left.0.cmp(&right.0));
        let mut transitions = Vec::with_capacity(identities.len());
        for (_, transition_id) in identities {
            let transition = self
                .get_transition_by_id_txn(txn, &transition_id)?
                .ok_or_else(|| format!("case_sequence_dangling_transition: {transition_id}"))?;
            transitions.push(transition);
        }
        Ok(transitions)
    }

    fn get_case_state_txn<T: Transaction>(
        &self,
        txn: &T,
        case_id: &str,
    ) -> Result<Option<CaseState>, String> {
        let key = case_state_key(case_id);
        match txn.get(self.case_state, &key) {
            Ok(value) => {
                let json = std::str::from_utf8(value)
                    .map_err(|error| format!("invalid CaseState utf8: {error}"))?;
                CaseState::from_json(json).map(Some)
            }
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!("failed to read CaseState {case_id}: {error}")),
        }
    }

    fn workflow_definition_txn<T: Transaction>(
        &self,
        txn: &T,
        workflow_definition_id: &str,
    ) -> Result<Option<WorkflowDefinition>, String> {
        let definition = get_json_txn::<WorkflowDefinition, _>(
            txn,
            self.workflow_definitions,
            &workflow_definition_key(workflow_definition_id),
            "WorkflowDefinition",
        )?;
        if let Some(definition) = &definition {
            definition.validate_integrity()?;
        }
        Ok(definition)
    }

    fn workflow_definition_graph_for_operations_txn<T: Transaction>(
        &self,
        txn: &T,
        root: &WorkflowDefinition,
        amendments: &[WorkflowAmendment],
        extra_operations: &[WorkflowPatchOperation],
    ) -> Result<BTreeMap<String, WorkflowDefinition>, String> {
        let mut definitions = BTreeMap::new();
        let mut pending = vec![root.clone()];
        let operation_refs = amendments
            .iter()
            .flat_map(|amendment| amendment.operations.iter())
            .chain(extra_operations.iter())
            .filter_map(|operation| match operation {
                WorkflowPatchOperation::AddNode {
                    node:
                        crate::workflow::WorkflowNode {
                            kind:
                                WorkflowNodeKind::Subflow {
                                    workflow_definition_id,
                                    workflow_definition_digest,
                                    ..
                                },
                            ..
                        },
                } => Some((
                    workflow_definition_id.clone(),
                    workflow_definition_digest.clone(),
                )),
                _ => None,
            })
            .collect::<Vec<_>>();
        for (definition_id, digest) in operation_refs {
            let child = self
                .workflow_definition_txn(txn, &definition_id)?
                .ok_or_else(|| "workflow_subflow_definition_missing".to_string())?;
            if child.tenant_id != root.tenant_id || child.integrity_digest != digest {
                return Err("workflow_subflow_definition_identity_mismatch".to_string());
            }
            pending.push(child);
        }
        while let Some(definition) = pending.pop() {
            if definitions.contains_key(&definition.workflow_definition_id) {
                continue;
            }
            if definitions.len() >= crate::workflow::MAX_REFERENCED_WORKFLOW_DEFINITIONS {
                return Err("workflow_definition_reference_bound_exceeded".to_string());
            }
            for node in &definition.nodes {
                if let WorkflowNodeKind::Subflow {
                    workflow_definition_id,
                    workflow_definition_digest,
                    ..
                } = &node.kind
                {
                    let child = self
                        .workflow_definition_txn(txn, workflow_definition_id)?
                        .ok_or_else(|| "workflow_subflow_definition_missing".to_string())?;
                    if child.tenant_id != root.tenant_id
                        || child.integrity_digest != *workflow_definition_digest
                    {
                        return Err("workflow_subflow_definition_identity_mismatch".to_string());
                    }
                    pending.push(child);
                }
            }
            definitions.insert(definition.workflow_definition_id.clone(), definition);
        }
        Ok(definitions)
    }

    fn validate_definition_composition_txn<T: Transaction>(
        &self,
        txn: &T,
        definition: &WorkflowDefinition,
    ) -> Result<(), String> {
        let mut definitions = BTreeMap::from([(
            definition.workflow_definition_id.clone(),
            definition.clone(),
        )]);
        let mut pending = definition
            .nodes
            .iter()
            .filter_map(|node| match &node.kind {
                WorkflowNodeKind::Subflow {
                    workflow_definition_id,
                    workflow_definition_digest,
                    ..
                } => Some((
                    workflow_definition_id.clone(),
                    workflow_definition_digest.clone(),
                )),
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut visiting = BTreeSet::from([definition.workflow_definition_id.clone()]);
        while let Some((definition_id, digest)) = pending.pop() {
            if definition_id == definition.workflow_definition_id {
                return Err("workflow_subflow_recursion_cycle".to_string());
            }
            let child = self
                .workflow_definition_txn(txn, &definition_id)?
                .ok_or_else(|| "workflow_subflow_definition_missing".to_string())?;
            if child.tenant_id != definition.tenant_id || child.integrity_digest != digest {
                return Err("workflow_subflow_definition_identity_mismatch".to_string());
            }
            if visiting.insert(definition_id.clone()) {
                for node in &child.nodes {
                    if let WorkflowNodeKind::Subflow {
                        workflow_definition_id,
                        workflow_definition_digest,
                        ..
                    } = &node.kind
                    {
                        pending.push((
                            workflow_definition_id.clone(),
                            workflow_definition_digest.clone(),
                        ));
                    }
                }
                definitions.insert(definition_id, child);
            }
        }
        Ok(())
    }

    fn last_case_sequence_txn<T: Transaction>(
        &self,
        txn: &T,
        case_id: &str,
    ) -> Result<u64, String> {
        let transitions = self.list_case_transitions_txn(txn, case_id)?;
        Ok(transitions
            .last()
            .map(|transition| transition.sequence)
            .unwrap_or(0))
    }

    #[cfg(test)]
    fn discard_case_state_for_test(&self, case_id: &str) -> Result<(), String> {
        let key = case_state_key(case_id);
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start test CaseState deletion: {error}"))?;
        match txn.del(self.case_state, &key, None) {
            Ok(()) | Err(Error::NotFound) => {}
            Err(error) => return Err(format!("failed to delete test CaseState: {error}")),
        }
        txn.commit()
            .map_err(|error| format!("failed to commit test CaseState deletion: {error}"))
    }

    #[cfg(test)]
    fn discard_policy_source_for_test(&self, source_id: &str) -> Result<(), String> {
        let key = policy_source_key(source_id);
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start test policy source deletion: {error}"))?;
        match txn.del(self.policy_sources_by_id, &key, None) {
            Ok(()) | Err(Error::NotFound) => {}
            Err(error) => return Err(format!("failed to delete test policy source: {error}")),
        }
        txn.commit()
            .map_err(|error| format!("failed to commit test policy source deletion: {error}"))
    }

    #[cfg(test)]
    fn discard_policy_artifact_for_test(&self, artifact_id: &str) -> Result<(), String> {
        let key = policy_artifact_key(artifact_id);
        let mut txn = self
            .env
            .begin_rw_txn()
            .map_err(|error| format!("failed to start test policy artifact deletion: {error}"))?;
        match txn.del(self.policy_artifacts_by_id, &key, None) {
            Ok(()) | Err(Error::NotFound) => {}
            Err(error) => return Err(format!("failed to delete test policy artifact: {error}")),
        }
        txn.commit()
            .map_err(|error| format!("failed to commit test policy artifact deletion: {error}"))
    }

    fn put_record(
        &self,
        txn: &mut RwTransaction<'_>,
        record: &Record,
        source_ref: &str,
    ) -> Result<(), String> {
        let id_key = format!("record:id:{}", record.id);
        let case_key = format!("record:case:{}:{}", record.case_ref, record.id);
        let kind_key = format!("record:kind:{}:{}", record.kind.as_str(), record.id);
        let value = record.to_record_plane_json(source_ref);
        txn.put(self.records_by_id, &id_key, &value, WriteFlags::empty())
            .map_err(|error| format!("failed to write records_by_id {}: {error}", record.id))?;
        txn.put(
            self.records_by_case,
            &case_key,
            &record.id,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to write records_by_case {}: {error}", record.id))?;
        txn.put(
            self.records_by_kind,
            &kind_key,
            &record.id,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to write records_by_kind {}: {error}", record.id))?;
        if !record.subject_ref.is_empty() && record.subject_ref != "subject:none" {
            let subject_key = format!("record:subject:{}:{}", record.subject_ref, record.id);
            txn.put(
                self.records_by_subject,
                &subject_key,
                &record.id,
                WriteFlags::empty(),
            )
            .map_err(|error| {
                format!("failed to write records_by_subject {}: {error}", record.id)
            })?;
        }
        if !record.receipt_id.is_empty() {
            let receipt_key = format!("record:receipt:{}:{}", record.receipt_id, record.id);
            txn.put(
                self.records_by_receipt,
                &receipt_key,
                &record.id,
                WriteFlags::empty(),
            )
            .map_err(|error| {
                format!("failed to write records_by_receipt {}: {error}", record.id)
            })?;
        }
        Ok(())
    }

    fn put_graph_relation(
        &self,
        txn: &mut RwTransaction<'_>,
        relation: &GraphRelation,
    ) -> Result<(), String> {
        let id_key = format!("graph_relation:id:{}", relation.relation_id);
        let case_key = format!(
            "graph_relation:case:{}:{}",
            relation.case_ref, relation.relation_id
        );
        let kind_key = format!(
            "graph_relation:kind:{}:{}",
            relation.edge_kind, relation.relation_id
        );
        let value = relation.to_json();
        txn.put(
            self.graph_relations_by_id,
            &id_key,
            &value,
            WriteFlags::empty(),
        )
        .map_err(|error| {
            format!(
                "failed to write graph_relations_by_id {}: {error}",
                relation.relation_id
            )
        })?;
        txn.put(
            self.schema_meta,
            &"meta:canonical_transition_schema",
            &TRANSITION_SCHEMA,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to write canonical transition schema meta: {error}"))?;
        txn.put(
            self.schema_meta,
            &"meta:case_state_schema",
            &CASE_STATE_SCHEMA,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to write CaseState schema meta: {error}"))?;
        txn.put(
            self.schema_meta,
            &"meta:canonical_authority_backend",
            &CANONICAL_AUTHORITY_BACKEND,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to write canonical authority backend meta: {error}"))?;
        txn.put(
            self.schema_meta,
            &"meta:legacy_compatibility_schema",
            &LEGACY_COMPATIBILITY_SCHEMA,
            WriteFlags::empty(),
        )
        .map_err(|error| format!("failed to write legacy compatibility schema meta: {error}"))?;
        txn.put(
            self.graph_relations_by_case,
            &case_key,
            &relation.relation_id,
            WriteFlags::empty(),
        )
        .map_err(|error| {
            format!(
                "failed to write graph_relations_by_case {}: {error}",
                relation.relation_id
            )
        })?;
        txn.put(
            self.graph_relations_by_kind,
            &kind_key,
            &relation.relation_id,
            WriteFlags::empty(),
        )
        .map_err(|error| {
            format!(
                "failed to write graph_relations_by_kind {}: {error}",
                relation.relation_id
            )
        })?;
        Ok(())
    }

    fn record_exists_txn(&self, txn: &RwTransaction<'_>, record_id: &str) -> Result<bool, String> {
        let id_key = format!("record:id:{record_id}");
        match txn.get(self.records_by_id, &id_key) {
            Ok(_) => Ok(true),
            Err(Error::NotFound) => Ok(false),
            Err(error) => Err(format!(
                "failed to check records_by_id {record_id}: {error}"
            )),
        }
    }

    fn graph_relation_exists_txn(
        &self,
        txn: &RwTransaction<'_>,
        relation_id: &str,
    ) -> Result<bool, String> {
        let id_key = format!("graph_relation:id:{relation_id}");
        match txn.get(self.graph_relations_by_id, &id_key) {
            Ok(_) => Ok(true),
            Err(Error::NotFound) => Ok(false),
            Err(error) => Err(format!(
                "failed to check graph_relations_by_id {relation_id}: {error}"
            )),
        }
    }

    fn get_record_by_id_txn(
        &self,
        txn: &RoTransaction<'_>,
        record_id: &str,
    ) -> Result<Option<StoredRecordEnvelope>, String> {
        let id_key = format!("record:id:{record_id}");
        match txn.get(self.records_by_id, &id_key) {
            Ok(value) => StoredRecordEnvelope::from_bytes(value).map(Some),
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!("failed to read records_by_id {record_id}: {error}")),
        }
    }

    fn get_graph_relation_by_id_txn(
        &self,
        txn: &RoTransaction<'_>,
        relation_id: &str,
    ) -> Result<Option<GraphRelation>, String> {
        let id_key = format!("graph_relation:id:{relation_id}");
        match txn.get(self.graph_relations_by_id, &id_key) {
            Ok(value) => GraphRelation::from_bytes(value).map(Some),
            Err(Error::NotFound) => Ok(None),
            Err(error) => Err(format!(
                "failed to read graph_relations_by_id {relation_id}: {error}"
            )),
        }
    }

    fn list_records_by_index(
        &self,
        txn: &RoTransaction<'_>,
        db: Database,
        prefix: &str,
        limit: usize,
    ) -> Result<RecordListResult, String> {
        let mut cursor = txn
            .open_ro_cursor(db)
            .map_err(|error| format!("failed to open LMDB index cursor: {error}"))?;
        let mut result = RecordListResult::default();
        for (key, value) in cursor.iter() {
            if !key.starts_with(prefix.as_bytes()) {
                continue;
            }
            result.records_total += 1;
            if result.records.len() >= limit {
                continue;
            }
            let record_id = std::str::from_utf8(value)
                .map_err(|error| format!("invalid LMDB index record id: {error}"))?;
            if let Some(record) = self.get_record_by_id_txn(txn, record_id)? {
                result.records.push(record);
            }
        }
        Ok(result)
    }

    fn list_graph_relations_by_index(
        &self,
        txn: &RoTransaction<'_>,
        db: Database,
        prefix: &str,
        limit: usize,
    ) -> Result<GraphRelationListResult, String> {
        let mut cursor = txn
            .open_ro_cursor(db)
            .map_err(|error| format!("failed to open LMDB graph relation cursor: {error}"))?;
        let mut result = GraphRelationListResult::default();
        for (key, value) in cursor.iter() {
            if !key.starts_with(prefix.as_bytes()) {
                continue;
            }
            result.relations_total += 1;
            if result.relations.len() >= limit {
                continue;
            }
            let relation_id = std::str::from_utf8(value)
                .map_err(|error| format!("invalid LMDB graph relation id: {error}"))?;
            if let Some(relation) = self.get_graph_relation_by_id_txn(txn, relation_id)? {
                result.relations.push(relation);
            }
        }
        Ok(result)
    }

    fn schema_ready(path: &Path) -> Result<bool, ()> {
        let env = shared_lmdb_environment(path, DEFAULT_LMDB_MAP_SIZE).map_err(|_| ())?;
        let Ok(schema_meta) = env.open_db(Some("schema_meta")) else {
            return Ok(false);
        };
        let Ok(txn) = env.begin_ro_txn() else {
            return Err(());
        };
        Ok(
            matches!(txn.get(schema_meta, &"meta:schema"), Ok(value) if value == RECORD_SCHEMA.as_bytes()),
        )
    }
}

impl ReplayMetadata {
    fn to_json(&self) -> String {
        format!(
            "{{\"replay_id\":\"{}\",\"journal_identity\":\"{}\",\"journal_path\":\"{}\",\"record_schema\":\"{}\",\"journal_schema\":\"{}\",\"started_at\":\"{}\",\"completed_at\":\"{}\",\"lines_total\":{},\"lines_replayed\":{},\"records_written\":{},\"records_duplicate\":{},\"records_skipped\":{},\"invalid_entries\":{},\"unsupported_entries\":{},\"cursor_line\":{},\"status\":\"{}\",\"compatibility\":\"{}\"}}",
            escape_json(&self.replay_id),
            escape_json(&self.journal_identity),
            escape_json(&self.journal_path),
            escape_json(&self.record_schema),
            escape_json(&self.journal_schema),
            escape_json(&self.started_at),
            escape_json(&self.completed_at),
            self.lines_total,
            self.lines_replayed,
            self.records_written,
            self.records_duplicate,
            self.records_skipped,
            self.invalid_entries,
            self.unsupported_entries,
            self.cursor_line,
            escape_json(&self.status),
            escape_json(&self.compatibility)
        )
    }

    fn from_bytes(value: &[u8]) -> Result<Self, String> {
        let raw_json = std::str::from_utf8(value)
            .map_err(|error| format!("invalid LMDB replay metadata utf8: {error}"))?;
        Ok(Self {
            replay_id: json_string_field(raw_json, "replay_id").unwrap_or_default(),
            journal_identity: json_string_field(raw_json, "journal_identity").unwrap_or_default(),
            journal_path: json_string_field(raw_json, "journal_path").unwrap_or_default(),
            record_schema: json_string_field(raw_json, "record_schema").unwrap_or_default(),
            journal_schema: json_string_field(raw_json, "journal_schema").unwrap_or_default(),
            started_at: json_string_field(raw_json, "started_at").unwrap_or_default(),
            completed_at: json_string_field(raw_json, "completed_at").unwrap_or_default(),
            lines_total: json_usize_field(raw_json, "lines_total"),
            lines_replayed: json_usize_field(raw_json, "lines_replayed"),
            records_written: json_usize_field(raw_json, "records_written"),
            records_duplicate: json_usize_field(raw_json, "records_duplicate"),
            records_skipped: json_usize_field(raw_json, "records_skipped"),
            invalid_entries: json_usize_field(raw_json, "invalid_entries"),
            unsupported_entries: json_usize_field(raw_json, "unsupported_entries"),
            cursor_line: json_usize_field(raw_json, "cursor_line"),
            status: json_string_field(raw_json, "status").unwrap_or_default(),
            compatibility: json_string_field(raw_json, "compatibility").unwrap_or_default(),
        })
    }
}

impl StoredRecordEnvelope {
    fn from_bytes(value: &[u8]) -> Result<Self, String> {
        let raw_json = std::str::from_utf8(value)
            .map_err(|error| format!("invalid LMDB record envelope utf8: {error}"))?
            .to_string();
        Ok(Self {
            schema: json_string_field(&raw_json, "schema").unwrap_or_default(),
            record_id: json_string_field(&raw_json, "record_id").unwrap_or_default(),
            record_kind: json_string_field(&raw_json, "record_kind").unwrap_or_default(),
            case_ref: json_string_field(&raw_json, "case_ref").unwrap_or_default(),
            raw_json,
        })
    }
}

impl GraphRelation {
    fn to_json(&self) -> String {
        format!(
            "{{\"schema\":\"{}\",\"relation_id\":\"{}\",\"case_ref\":\"{}\",\"from_ref\":\"{}\",\"to_ref\":\"{}\",\"edge_kind\":\"{}\",\"from_kind\":\"{}\",\"to_kind\":\"{}\",\"source_record_id\":\"{}\",\"source_record_kind\":\"{}\",\"confidence\":\"{}\",\"created_at_unix_ms\":{},\"provenance\":\"{}\"}}",
            GRAPH_RELATION_SCHEMA,
            escape_json(&self.relation_id),
            escape_json(&self.case_ref),
            escape_json(&self.from_ref),
            escape_json(&self.to_ref),
            escape_json(&self.edge_kind),
            escape_json(&self.from_kind),
            escape_json(&self.to_kind),
            escape_json(&self.source_record_id),
            escape_json(&self.source_record_kind),
            escape_json(&self.confidence),
            self.created_at_unix_ms,
            escape_json(&self.provenance)
        )
    }

    fn from_bytes(value: &[u8]) -> Result<Self, String> {
        let raw_json = std::str::from_utf8(value)
            .map_err(|error| format!("invalid LMDB graph relation utf8: {error}"))?;
        Ok(Self {
            relation_id: json_string_field(raw_json, "relation_id").unwrap_or_default(),
            case_ref: json_string_field(raw_json, "case_ref").unwrap_or_default(),
            from_ref: json_string_field(raw_json, "from_ref").unwrap_or_default(),
            to_ref: json_string_field(raw_json, "to_ref").unwrap_or_default(),
            edge_kind: json_string_field(raw_json, "edge_kind").unwrap_or_default(),
            from_kind: json_string_field(raw_json, "from_kind").unwrap_or_default(),
            to_kind: json_string_field(raw_json, "to_kind").unwrap_or_default(),
            source_record_id: json_string_field(raw_json, "source_record_id").unwrap_or_default(),
            source_record_kind: json_string_field(raw_json, "source_record_kind")
                .unwrap_or_default(),
            confidence: json_string_field(raw_json, "confidence").unwrap_or_default(),
            created_at_unix_ms: json_u128_field(raw_json, "created_at_unix_ms"),
            provenance: json_string_field(raw_json, "provenance").unwrap_or_default(),
        })
    }
}

fn derive_graph_relations_from_transition(
    transition: &Transition,
    skipped: &mut usize,
) -> Vec<GraphRelation> {
    let mut relations = Vec::new();
    add_transition_relation(
        &mut relations,
        skipped,
        transition,
        "transition_committed_in_case",
        "transition",
        &transition.transition_id,
        "case",
        &transition.case_id,
    );
    match &transition.payload {
        TransitionPayload::CaseOpened { .. } => {}
        TransitionPayload::TenantCaseOpened { tenant_id, .. } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "case_owned_by_tenant",
            "case",
            &transition.case_id,
            "tenant",
            tenant_id,
        ),
        TransitionPayload::ParticipantBound { participant_id, .. } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "participant_bound_to_case",
            "participant",
            participant_id,
            "case",
            &transition.case_id,
        ),
        TransitionPayload::ParticipantAdmitted { participant_id, .. } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "participant_admitted_to_case",
            "participant",
            participant_id,
            "case",
            &transition.case_id,
        ),
        TransitionPayload::ParticipantPrincipalLinked { link } => {
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "principal_linked_to_participant",
                "principal",
                &link.principal_id,
                "participant",
                &link.participant_id,
            );
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "principal_participant_link_scoped_to_tenant",
                "principal_participant_link",
                &link.link_id,
                "tenant",
                &link.tenant_id,
            );
        }
        TransitionPayload::ProviderAttached {
            participant_id,
            model_id,
            ..
        } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "participant_uses_model",
            "participant",
            participant_id,
            "model",
            model_id,
        ),
        TransitionPayload::ProviderInvocationStarted {
            invocation_id,
            model_id,
            ..
        } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "provider_invocation_uses_model",
            "provider_invocation",
            invocation_id,
            "model",
            model_id,
        ),
        TransitionPayload::ProviderResultRecorded {
            result_id,
            invocation_id,
            ..
        } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "provider_result_closes_invocation",
            "provider_result",
            result_id,
            "provider_invocation",
            invocation_id,
        ),
        TransitionPayload::InteractionTurnRecorded {
            turn_id,
            invocation_id,
            result_id,
            ..
        } => {
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "interaction_turn_uses_invocation",
                "interaction_turn",
                turn_id,
                "provider_invocation",
                invocation_id,
            );
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "interaction_turn_observes_result",
                "interaction_turn",
                turn_id,
                "provider_result",
                result_id,
            );
        }
        TransitionPayload::ModelInterpretationRecorded {
            interpretation_id,
            result_id,
            ..
        } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "model_interpretation_from_result",
            "model_interpretation",
            interpretation_id,
            "provider_result",
            result_id,
        ),
        TransitionPayload::ResourceAttached { attachment } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "resource_attached_to_case",
            "resource_attachment",
            &attachment.attachment_id,
            "case",
            &transition.case_id,
        ),
        TransitionPayload::OperationNormalizationFailed {
            provider_result_id, ..
        } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "normalization_failure_from_provider_result",
            "transition",
            &transition.transition_id,
            "provider_result",
            provider_result_id,
        ),
        TransitionPayload::OperationRecorded { operation } => {
            match &operation.origin {
                OperationOrigin::ProviderResult {
                    provider_result_id, ..
                } => add_transition_relation(
                    &mut relations,
                    skipped,
                    transition,
                    "operation_from_provider_result",
                    "operation",
                    &operation.operation_id,
                    "provider_result",
                    provider_result_id,
                ),
                OperationOrigin::CompatibilityReview { review_id, .. } => add_transition_relation(
                    &mut relations,
                    skipped,
                    transition,
                    "operation_from_review",
                    "operation",
                    &operation.operation_id,
                    "review_request",
                    review_id,
                ),
                OperationOrigin::WorkflowDeterministicProposal { proposal_id, .. } => {
                    add_transition_relation(
                        &mut relations,
                        skipped,
                        transition,
                        "operation_from_workflow_proposal",
                        "operation",
                        &operation.operation_id,
                        "workflow_deterministic_proposal",
                        proposal_id,
                    )
                }
            }
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "operation_targets_resource",
                "operation",
                &operation.operation_id,
                "resource_attachment",
                &operation.resource_attachment_id,
            );
        }
        TransitionPayload::DecisionRecorded { decision } => {
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "decision_controls_operation",
                "decision",
                &decision.decision_id,
                "operation",
                &decision.operation_id,
            );
            if let Some(basis) = &decision.decision_basis {
                add_transition_relation(
                    &mut relations,
                    skipped,
                    transition,
                    "decision_uses_basis",
                    "decision",
                    &decision.decision_id,
                    "decision_basis",
                    &basis.basis_id,
                );
                add_transition_relation(
                    &mut relations,
                    skipped,
                    transition,
                    "decision_uses_effective_policy",
                    "decision_basis",
                    &basis.basis_id,
                    "effective_policy",
                    &basis.effective_policy_id,
                );
                for artifact_id in &basis.policy_artifact_refs {
                    add_transition_relation(
                        &mut relations,
                        skipped,
                        transition,
                        "decision_basis_uses_policy_artifact",
                        "decision_basis",
                        &basis.basis_id,
                        "policy_artifact",
                        artifact_id,
                    );
                }
            }
        }
        TransitionPayload::ExecutionGrantIssued { grant } => {
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "execution_grant_from_decision",
                "execution_grant",
                &grant.grant_id,
                "decision",
                &grant.decision_id,
            );
            if let Some(basis_id) = &grant.decision_basis_id {
                add_transition_relation(
                    &mut relations,
                    skipped,
                    transition,
                    "execution_grant_uses_basis",
                    "execution_grant",
                    &grant.grant_id,
                    "decision_basis",
                    basis_id,
                );
            }
        }
        TransitionPayload::EffectPrepared { prepared } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "prepared_effect_consumes_grant",
            "prepared_effect",
            &prepared.effect_id,
            "execution_grant",
            &prepared.grant_id,
        ),
        TransitionPayload::EffectFinalized {
            effect_id, receipt, ..
        } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "effect_receipt_closes_prepared_effect",
            "effect_receipt",
            &receipt.receipt_id,
            "prepared_effect",
            effect_id,
        ),
        TransitionPayload::EffectIndeterminate { effect_id, .. } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "indeterminate_transition_tracks_effect",
            "transition",
            &transition.transition_id,
            "prepared_effect",
            effect_id,
        ),
        TransitionPayload::EffectReconciled {
            effect_id, receipt, ..
        } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "reconciliation_closes_or_tracks_effect",
            receipt
                .as_ref()
                .map(|_| "effect_receipt")
                .unwrap_or("transition"),
            receipt
                .as_ref()
                .map(|value| value.receipt_id.as_str())
                .unwrap_or(&transition.transition_id),
            "prepared_effect",
            effect_id,
        ),
        TransitionPayload::ProcessEffectPrepared { prepared } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "prepared_process_effect_consumes_grant",
            "prepared_process_effect",
            &prepared.effect_id,
            "execution_grant",
            &prepared.grant_id,
        ),
        TransitionPayload::ProcessEffectFinalized {
            effect_id, receipt, ..
        } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "process_effect_receipt_closes_prepared_effect",
            "process_effect_receipt",
            &receipt.receipt_id,
            "prepared_process_effect",
            effect_id,
        ),
        TransitionPayload::ProcessEffectIndeterminate { effect_id, .. } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "indeterminate_transition_tracks_process_effect",
            "transition",
            &transition.transition_id,
            "prepared_process_effect",
            effect_id,
        ),
        TransitionPayload::ReviewRequested { review } => {
            if review.operation_id.is_empty() {
                add_transition_relation(
                    &mut relations,
                    skipped,
                    transition,
                    "review_request_for_attempt",
                    "review_request",
                    &review.review_id,
                    "attempt",
                    &review.attempt_id,
                );
            } else {
                add_transition_relation(
                    &mut relations,
                    skipped,
                    transition,
                    "review_request_for_operation",
                    "review_request",
                    &review.review_id,
                    "operation",
                    &review.operation_id,
                );
                if !review.decision_basis_id.is_empty() {
                    add_transition_relation(
                        &mut relations,
                        skipped,
                        transition,
                        "review_request_uses_decision_basis",
                        "review_request",
                        &review.review_id,
                        "decision_basis",
                        &review.decision_basis_id,
                    );
                }
            }
        }
        TransitionPayload::ReviewActionRecorded { action } => {
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "review_action_resolves_request",
                "review_action",
                &action.action_id,
                "review_request",
                &action.review_id,
            );
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "review_action_by_participant",
                "review_action",
                &action.action_id,
                "participant",
                &action.reviewer_participant_id,
            );
        }
        TransitionPayload::ReviewInvalidated { invalidation } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "review_invalidated_by_authority_change",
            "transition",
            &transition.transition_id,
            "review_request",
            &invalidation.review_id,
        ),
        TransitionPayload::ExecutionGrantInvalidated { invalidation } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "execution_grant_invalidated",
            "transition",
            &transition.transition_id,
            "execution_grant",
            &invalidation.grant_id,
        ),
        TransitionPayload::CaseCancellationRequested { .. } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "case_cancellation_barrier",
            "transition",
            &transition.transition_id,
            "case",
            &transition.case_id,
        ),
        TransitionPayload::CaseClosed { .. } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "case_terminally_closed",
            "transition",
            &transition.transition_id,
            "case",
            &transition.case_id,
        ),
        TransitionPayload::CasePolicyBound { binding }
        | TransitionPayload::CasePolicyReplaced { binding, .. } => {
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "case_policy_binding_uses_artifact",
                "case_policy_binding",
                &binding.binding_id,
                "policy_artifact",
                &binding.artifact_id,
            );
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "case_has_policy_binding",
                "case",
                &transition.case_id,
                "case_policy_binding",
                &binding.binding_id,
            );
        }
        TransitionPayload::CasePolicyUnbound { binding_id, .. } => add_transition_relation(
            &mut relations,
            skipped,
            transition,
            "case_policy_binding_removed",
            "transition",
            &transition.transition_id,
            "case_policy_binding",
            binding_id,
        ),
        TransitionPayload::CaseWorkflowBound { binding } => {
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "case_bound_to_workflow_definition",
                "case",
                &transition.case_id,
                "workflow_definition",
                &binding.workflow_definition_id,
            );
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "case_has_workflow_binding",
                "case",
                &transition.case_id,
                "case_workflow_binding",
                &binding.binding_id,
            );
        }
        TransitionPayload::WorkflowNodeExecutionStarted { execution } => {
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "workflow_execution_runs_node",
                "workflow_execution",
                &execution.execution_id,
                "workflow_node",
                &execution.node_id,
            );
        }
        TransitionPayload::WorkflowNodeSatisfied { satisfaction } => {
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "workflow_satisfaction_proves_node",
                "workflow_satisfaction",
                &satisfaction.satisfaction_id,
                "workflow_node",
                &satisfaction.node_id,
            );
        }
        TransitionPayload::WorkflowConditionResolved { resolution } => {
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "workflow_condition_resolves_node",
                "workflow_condition_resolution",
                &resolution.resolution_id,
                "workflow_node",
                &resolution.node_id,
            );
        }
        TransitionPayload::WorkflowHumanInputRecorded { input } => {
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "workflow_human_input_satisfies_node",
                "workflow_human_input",
                &input.input_id,
                "workflow_node",
                &input.node_id,
            );
        }
        TransitionPayload::WorkflowDeterministicProposalRecorded { proposal } => {
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "workflow_proposal_from_execution",
                "workflow_deterministic_proposal",
                &proposal.proposal_id,
                "workflow_execution",
                &proposal.execution_id,
            );
        }
        TransitionPayload::WorkflowPlanPatchProposed { patch } => {
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "workflow_patch_targets_binding",
                "workflow_plan_patch",
                &patch.patch_id,
                "case_workflow_binding",
                &patch.binding_id,
            );
        }
        TransitionPayload::WorkflowAmendmentAdopted { amendment } => {
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "workflow_amendment_adopts_patch",
                "workflow_amendment",
                &amendment.amendment_id,
                "workflow_plan_patch",
                &amendment.patch_id,
            );
        }
        TransitionPayload::HandoffOffered { offer } => {
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "handoff_offer_targets_case",
                "handoff_offer",
                &offer.handoff_id,
                "case",
                &offer.target_case_id,
            );
        }
        TransitionPayload::HandoffAccepted { acceptance } => {
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "handoff_acceptance_accepts_offer",
                "handoff_acceptance",
                &acceptance.acceptance_id,
                "handoff_offer",
                &acceptance.handoff_id,
            );
        }
        TransitionPayload::HandoffDeclined { decline } => {
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "handoff_decline_declines_offer",
                "handoff_decline",
                &decline.decline_id,
                "handoff_offer",
                &decline.handoff_id,
            );
        }
        TransitionPayload::HandoffResultRecorded { result } => {
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "handoff_result_from_acceptance",
                "handoff_result",
                &result.result_id,
                "handoff_acceptance",
                &result.acceptance_id,
            );
        }
        TransitionPayload::HandoffReconciled { reconciliation } => {
            let disposition_kind = if reconciliation.target_result_id.is_some() {
                "handoff_result"
            } else if reconciliation.target_decline_id.is_some() {
                "handoff_decline"
            } else {
                "transition"
            };
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "handoff_reconciliation_uses_disposition",
                "handoff_reconciliation",
                &reconciliation.reconciliation_id,
                disposition_kind,
                reconciliation.target_disposition_id(),
            );
        }
        TransitionPayload::ReviewResolved {
            review_id,
            decision_ref,
            receipt_ref,
            ..
        } => {
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "review_decision_resolves_request",
                "decision",
                decision_ref,
                "review_request",
                review_id,
            );
            add_transition_relation(
                &mut relations,
                skipped,
                transition,
                "review_resolution_produces_receipt",
                "review_request",
                review_id,
                "receipt",
                receipt_ref,
            );
        }
    }
    relations
}

#[allow(clippy::too_many_arguments)]
fn add_transition_relation(
    relations: &mut Vec<GraphRelation>,
    skipped: &mut usize,
    transition: &Transition,
    edge_kind: &str,
    from_kind: &str,
    from_ref: &str,
    to_kind: &str,
    to_ref: &str,
) {
    if from_ref.is_empty() || to_ref.is_empty() {
        *skipped += 1;
        return;
    }
    relations.push(GraphRelation {
        relation_id: format!(
            "edge:{}:{}",
            edge_kind,
            relation_id_component(&transition.transition_id)
        ),
        case_ref: transition.case_id.clone(),
        from_ref: from_ref.to_string(),
        to_ref: to_ref.to_string(),
        edge_kind: edge_kind.to_string(),
        from_kind: from_kind.to_string(),
        to_kind: to_kind.to_string(),
        source_record_id: transition.transition_id.clone(),
        source_record_kind: transition.payload.kind().to_string(),
        confidence: "derived".to_string(),
        created_at_unix_ms: u128::from(transition.committed_at_unix_ms),
        provenance: "canonical_transition".to_string(),
    });
}

fn derive_graph_relations(
    record: &StoredRecordEnvelope,
    created_at_unix_ms: u128,
    skipped: &mut usize,
) -> Vec<GraphRelation> {
    let mut relations = Vec::new();
    let LegacyDecodeOutcome::Promoted(legacy) = decode_legacy_record(&record.raw_json) else {
        *skipped += 1;
        return relations;
    };
    let subject_ref = legacy.subject_ref.clone();
    let attempt_id = legacy.attempt_id.clone();
    let decision_id = legacy.decision_id.clone();
    let receipt_id = legacy.receipt_id.clone();

    add_relation(
        &mut relations,
        skipped,
        relation_from_record(
            record,
            "record_materializes_node",
            "record",
            &record.record_id,
            node_kind_for_record(&record.record_kind),
            &node_ref_for_record(
                record,
                &legacy,
                &subject_ref,
                &attempt_id,
                &decision_id,
                &receipt_id,
            ),
            created_at_unix_ms,
        ),
    );

    if has_subject_ref(&subject_ref) {
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "subject_participates_in_case",
                "subject",
                &subject_ref,
                "case",
                &record.case_ref,
                created_at_unix_ms,
            ),
        );
    }

    if matches!(record.record_kind.as_str(), "attempt" | "carrier_request") {
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "attempt_targets_subject",
                "attempt",
                &attempt_id,
                "subject",
                &subject_ref,
                created_at_unix_ms,
            ),
        );
    }

    if record.record_kind == "decision" {
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "decision_controls_attempt",
                "decision",
                &decision_id,
                "attempt",
                &attempt_id,
                created_at_unix_ms,
            ),
        );
    }

    if matches!(
        record.record_kind.as_str(),
        "receipt" | "effect_receipt" | "filesystem_receipt"
    ) {
        let explicitly_no_effect = legacy.compatibility_value("no_resource_effect") == Some("true")
            || matches!(
                legacy.compatibility_value("status"),
                Some("blocked" | "deferred" | "quarantined" | "not_executed")
            );
        let (target_kind, target_ref) = if explicitly_no_effect {
            (
                "attempt",
                if attempt_id.is_empty() {
                    record.record_id.clone()
                } else {
                    attempt_id.clone()
                },
            )
        } else {
            (
                "effect",
                if attempt_id.is_empty() {
                    format!("effect:{}", record.record_id)
                } else {
                    format!("effect:{attempt_id}")
                },
            )
        };
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                if explicitly_no_effect {
                    "receipt_records_no_effect"
                } else {
                    "receipt_records_effect"
                },
                "receipt",
                &receipt_id,
                target_kind,
                &target_ref,
                created_at_unix_ms,
            ),
        );
    }

    if matches!(
        record.record_kind.as_str(),
        "policy_rule" | "receipt_requirement" | "authority_scope"
    ) {
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "policy_constrains_subject",
                "policy",
                &record.record_id,
                "subject",
                &subject_ref,
                created_at_unix_ms,
            ),
        );
    }

    if matches!(
        record.record_kind.as_str(),
        "projection" | "projection_request" | "projection_result"
    ) {
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "projection_exposes_record",
                "projection",
                &record.record_id,
                "record",
                &record.record_id,
                created_at_unix_ms,
            ),
        );
    }

    if record.record_kind == "model_interpretation" {
        let model_output_ref = format!("model_output:{}", record.record_id);
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "model_output_produces_interpretation",
                "model_output",
                &model_output_ref,
                "model_interpretation",
                &record.record_id,
                created_at_unix_ms,
            ),
        );
    }

    if record.record_kind == "divergence" {
        let (target_kind, target_ref) = if !receipt_id.is_empty() {
            ("receipt", receipt_id.as_str())
        } else if !decision_id.is_empty() {
            ("decision", decision_id.as_str())
        } else {
            ("attempt", attempt_id.as_str())
        };
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "divergence_describes_mismatch",
                "divergence",
                &record.record_id,
                target_kind,
                target_ref,
                created_at_unix_ms,
            ),
        );
    }

    if record.record_kind == "review_request" {
        let review_ref = legacy
            .compatibility_value("review_id")
            .unwrap_or(&record.record_id);
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "review_request_for_attempt",
                "review_request",
                &review_ref,
                "attempt",
                &attempt_id,
                created_at_unix_ms,
            ),
        );
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "review_resolution_produces_receipt",
                "review_request",
                &review_ref,
                "receipt",
                &receipt_id,
                created_at_unix_ms,
            ),
        );
    }

    if record.record_kind == "review_decision" {
        let review_ref = legacy.compatibility_value("review_id").unwrap_or("");
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "review_decision_resolves_request",
                "review_decision",
                &record.record_id,
                "review_request",
                &review_ref,
                created_at_unix_ms,
            ),
        );
    }

    if record.record_kind == "control_pending" {
        let pending_ref = legacy
            .compatibility_value("pending_id")
            .unwrap_or(&record.record_id);
        add_relation(
            &mut relations,
            skipped,
            relation_from_record(
                record,
                "control_pending_blocks_attempt",
                "control_pending",
                &pending_ref,
                "attempt",
                &attempt_id,
                created_at_unix_ms,
            ),
        );
    }

    relations
}

fn relation_from_record(
    record: &StoredRecordEnvelope,
    edge_kind: &str,
    from_kind: &str,
    from_ref: &str,
    to_kind: &str,
    to_ref: &str,
    created_at_unix_ms: u128,
) -> Option<GraphRelation> {
    if record.case_ref.is_empty()
        || edge_kind.is_empty()
        || from_kind.is_empty()
        || from_ref.is_empty()
        || to_kind.is_empty()
        || to_ref.is_empty()
        || to_ref == "subject:none"
    {
        return None;
    }
    Some(GraphRelation {
        relation_id: format!(
            "edge:{}:{}",
            edge_kind,
            relation_id_component(&record.record_id)
        ),
        case_ref: record.case_ref.clone(),
        from_ref: from_ref.to_string(),
        to_ref: to_ref.to_string(),
        edge_kind: edge_kind.to_string(),
        from_kind: from_kind.to_string(),
        to_kind: to_kind.to_string(),
        source_record_id: record.record_id.clone(),
        source_record_kind: record.record_kind.clone(),
        confidence: "derived".to_string(),
        created_at_unix_ms,
        provenance: "record".to_string(),
    })
}

fn add_relation(
    relations: &mut Vec<GraphRelation>,
    skipped: &mut usize,
    relation: Option<GraphRelation>,
) {
    if let Some(relation) = relation {
        relations.push(relation);
    } else {
        *skipped += 1;
    }
}

fn push_runtime_node(nodes: &mut Vec<RuntimeGraphNode>, node: RuntimeGraphNode) {
    if nodes
        .iter()
        .any(|existing| existing.node_ref == node.node_ref)
    {
        return;
    }
    nodes.push(node);
}

fn push_unique_string(values: &mut Vec<String>, value: &str) {
    if values.iter().any(|existing| existing == value) {
        return;
    }
    values.push(value.to_string());
}

fn node_ref_for_record(
    record: &StoredRecordEnvelope,
    legacy: &LegacyRecord,
    subject_ref: &str,
    attempt_id: &str,
    decision_id: &str,
    receipt_id: &str,
) -> String {
    match record.record_kind.as_str() {
        "case" => record.case_ref.clone(),
        "subject_binding" | "subject_state" => subject_ref.to_string(),
        "attempt" | "carrier_request" => fallback_ref(attempt_id, &record.record_id),
        "decision" | "decision_basis" | "gate_result" => {
            fallback_ref(decision_id, &record.record_id)
        }
        "review_request" => legacy
            .compatibility_value("review_id")
            .unwrap_or(&record.record_id)
            .to_string(),
        "control_pending" => legacy
            .compatibility_value("pending_id")
            .unwrap_or(&record.record_id)
            .to_string(),
        "review_decision" => record.record_id.clone(),
        "receipt" | "effect_receipt" | "filesystem_receipt" => {
            fallback_ref(receipt_id, &record.record_id)
        }
        "policy_rule" | "receipt_requirement" | "authority_scope" => record.record_id.clone(),
        "projection" | "projection_request" | "projection_result" => record.record_id.clone(),
        "memory_candidate" => record.record_id.clone(),
        "model_interpretation" => record.record_id.clone(),
        "divergence" => record.record_id.clone(),
        _ => record.record_id.clone(),
    }
}

fn node_kind_for_record(record_kind: &str) -> &'static str {
    match record_kind {
        "case" => "case",
        "subject_binding" | "subject_state" => "subject",
        "attempt" => "attempt",
        "decision" | "decision_basis" | "gate_result" => "decision",
        "review_request" => "review_request",
        "review_decision" => "review_decision",
        "control_pending" => "control_pending",
        "carrier_request" => "dispatch",
        "receipt" | "effect_receipt" | "filesystem_receipt" => "receipt",
        "policy_rule" | "receipt_requirement" | "authority_scope" => "policy",
        "projection" | "projection_request" | "projection_result" => "projection",
        "memory_candidate" => "memory_candidate",
        "model_interpretation" => "model_interpretation",
        "divergence" => "divergence",
        _ => "record",
    }
}

fn fallback_ref(preferred: &str, fallback: &str) -> String {
    if preferred.is_empty() {
        fallback.to_string()
    } else {
        preferred.to_string()
    }
}

fn has_subject_ref(subject_ref: &str) -> bool {
    !subject_ref.is_empty() && subject_ref != "subject:none"
}

fn relation_id_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, ':' | '_' | '-' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn count_entries(txn: &RoTransaction<'_>, db: Database) -> Result<usize, String> {
    let mut cursor = txn
        .open_ro_cursor(db)
        .map_err(|error| format!("failed to open LMDB cursor: {error}"))?;
    Ok(cursor.iter().count())
}

fn ensure_meta_upgradeable<T: Transaction>(
    txn: &T,
    database: Database,
    key: &str,
    expected: &str,
    previous: &[&str],
) -> Result<(), String> {
    match txn.get(database, &key) {
        Ok(actual) if actual == expected.as_bytes() => Ok(()),
        Ok(actual) if previous.iter().any(|value| actual == value.as_bytes()) => Ok(()),
        Ok(actual) => Err(format!(
            "unsupported_persisted_schema: {key} expected={expected} actual={}",
            String::from_utf8_lossy(actual)
        )),
        Err(Error::NotFound) => Ok(()),
        Err(error) => Err(format!("failed to read persisted schema {key}: {error}")),
    }
}

fn transition_id_key(transition_id: &str) -> String {
    format!("transition:id:{transition_id}")
}

fn case_sequence_prefix(case_id: &str) -> String {
    format!("transition:case:{case_id}:")
}

fn case_sequence_key(case_id: &str, sequence: u64) -> String {
    format!("{}{:020}", case_sequence_prefix(case_id), sequence)
}

fn case_state_key(case_id: &str) -> String {
    format!("case_state:{case_id}")
}

fn workflow_definition_key(workflow_definition_id: &str) -> String {
    format!("definition:{workflow_definition_id}")
}

fn workflow_definition_version_key(
    tenant_id: &str,
    workflow_key: &str,
    declared_version: &str,
) -> String {
    format!("version:{tenant_id}\0{workflow_key}\0{declared_version}")
}

fn workflow_patch_frozen_history_barrier(
    state: &CaseState,
    patch: &WorkflowPlanPatch,
) -> Result<(), String> {
    let mut frozen = BTreeSet::new();
    frozen.extend(
        state
            .workflow_executions
            .iter()
            .map(|value| value.node_id.clone()),
    );
    frozen.extend(
        state
            .workflow_satisfactions
            .iter()
            .map(|value| value.node_id.clone()),
    );
    frozen.extend(
        state
            .workflow_conditions
            .iter()
            .map(|value| value.node_id.clone()),
    );
    frozen.extend(
        state
            .workflow_human_inputs
            .iter()
            .map(|value| value.node_id.clone()),
    );
    frozen.extend(
        state
            .workflow_deterministic_proposals
            .iter()
            .map(|value| value.node_id.clone()),
    );
    for operation in &patch.operations {
        let touches_frozen = match operation {
            WorkflowPatchOperation::AddNode { .. } => false,
            WorkflowPatchOperation::AddEdge { edge } => {
                frozen.contains(&edge.from) || frozen.contains(&edge.to)
            }
            WorkflowPatchOperation::DisableNode { node_id } => frozen.contains(node_id),
            WorkflowPatchOperation::DisableEdge { from, to, .. } => {
                frozen.contains(from) || frozen.contains(to)
            }
        };
        if touches_frozen {
            return Err("workflow_patch_frozen_history_barrier".to_string());
        }
    }
    Ok(())
}

fn workflow_execution_identity(binding: &CaseWorkflowBinding, node_id: &str) -> String {
    let digest = crate::context::stable_digest(&format!(
        "{}\0{}\0{}\0{}",
        binding.case_id, binding.binding_id, binding.workflow_definition_id, node_id
    ));
    format!("workflow-execution:{digest}")
}

fn workflow_fact_identity(
    kind: &str,
    binding_id: &str,
    node_id: &str,
    predicate_digest: &str,
    evidence_refs: &[String],
) -> String {
    let digest = crate::context::stable_digest(&format!(
        "{kind}\0{binding_id}\0{node_id}\0{predicate_digest}\0{}",
        evidence_refs.join("\0")
    ));
    format!("workflow-{kind}:{digest}")
}

fn workflow_transition_source(principal_id: &str, source_ref: &str) -> TransitionSource {
    TransitionSource {
        component: "yai.workflow".to_string(),
        participant_id: None,
        principal_id: Some(principal_id.to_string()),
        source_ref: Some(source_ref.to_string()),
    }
}

#[allow(clippy::too_many_arguments)]
fn workflow_satisfaction_pending(
    case_id: &str,
    state: &CaseState,
    binding: &CaseWorkflowBinding,
    definition: &WorkflowDefinition,
    node_id: &str,
    execution_id: Option<&str>,
    predicate: &WorkflowPredicate,
    evidence_refs: Vec<String>,
    principal_id: &str,
) -> Result<PendingTransition, String> {
    if evidence_refs.is_empty() {
        return Err("workflow_satisfaction_requires_canonical_evidence".to_string());
    }
    let predicate_digest = predicate.digest()?;
    let satisfaction = WorkflowNodeSatisfaction {
        schema: WORKFLOW_NODE_SATISFACTION_SCHEMA.to_string(),
        satisfaction_id: workflow_fact_identity(
            "satisfaction",
            &binding.binding_id,
            node_id,
            &predicate_digest,
            &evidence_refs,
        ),
        binding_id: binding.binding_id.clone(),
        workflow_definition_id: definition.workflow_definition_id.clone(),
        node_id: node_id.to_string(),
        execution_id: execution_id.map(str::to_string),
        predicate_digest,
        evaluated_at_generation: state.generation + 1,
        evidence_refs,
    };
    let mut pending = PendingTransition::new(
        format!("transition:{}", satisfaction.satisfaction_id),
        case_id,
        state.generation,
        workflow_transition_source(principal_id, &satisfaction.satisfaction_id),
        TransitionPayload::WorkflowNodeSatisfied {
            satisfaction: satisfaction.clone(),
        },
    );
    pending.causal_refs = vec![binding.binding_id.clone()];
    pending
        .causal_refs
        .extend(satisfaction.evidence_refs.clone());
    Ok(pending)
}

fn workflow_subflow_predicate_digest(node_id: &str) -> String {
    crate::effect::digest_bytes(format!("yai.workflow.subflow.complete.v1\0{node_id}").as_bytes())
}

fn workflow_subflow_completion_evidence(
    topology: &crate::workflow::EffectiveWorkflowTopology,
    state: &CaseState,
    node_id: &str,
) -> Result<Vec<String>, String> {
    let dependencies = topology
        .edges
        .iter()
        .filter(|edge| edge.to == node_id)
        .map(|edge| edge.from.as_str())
        .collect::<BTreeSet<_>>();
    let mut evidence = Vec::new();
    for dependency in dependencies {
        if let Some(value) = state
            .workflow_satisfactions
            .iter()
            .find(|value| value.node_id == dependency)
        {
            evidence.push(value.satisfaction_id.clone());
            continue;
        }
        if let Some(value) = state
            .workflow_conditions
            .iter()
            .find(|value| value.node_id == dependency)
        {
            evidence.push(value.resolution_id.clone());
            continue;
        }
        if let Some(value) = state
            .workflow_human_inputs
            .iter()
            .find(|value| value.node_id == dependency)
        {
            evidence.push(value.input_id.clone());
        }
    }
    evidence.sort();
    evidence.dedup();
    if evidence.is_empty() {
        return Err("workflow_subflow_completion_evidence_missing".to_string());
    }
    Ok(evidence)
}

#[allow(clippy::too_many_arguments)]
fn workflow_subflow_satisfaction_pending(
    case_id: &str,
    state: &CaseState,
    binding: &CaseWorkflowBinding,
    definition: &WorkflowDefinition,
    node_id: &str,
    evidence_refs: Vec<String>,
    principal_id: &str,
) -> Result<PendingTransition, String> {
    let predicate_digest = workflow_subflow_predicate_digest(node_id);
    let satisfaction = WorkflowNodeSatisfaction {
        schema: WORKFLOW_NODE_SATISFACTION_SCHEMA.to_string(),
        satisfaction_id: workflow_fact_identity(
            "satisfaction",
            &binding.binding_id,
            node_id,
            &predicate_digest,
            &evidence_refs,
        ),
        binding_id: binding.binding_id.clone(),
        workflow_definition_id: definition.workflow_definition_id.clone(),
        node_id: node_id.to_string(),
        execution_id: None,
        predicate_digest,
        evaluated_at_generation: state.generation + 1,
        evidence_refs,
    };
    let mut pending = PendingTransition::new(
        format!("transition:{}", satisfaction.satisfaction_id),
        case_id,
        state.generation,
        workflow_transition_source(principal_id, &satisfaction.satisfaction_id),
        TransitionPayload::WorkflowNodeSatisfied {
            satisfaction: satisfaction.clone(),
        },
    );
    pending.causal_refs = std::iter::once(binding.binding_id.clone())
        .chain(satisfaction.evidence_refs.iter().cloned())
        .collect();
    Ok(pending)
}

fn tenant_membership_key(tenant_id: &str, principal_id: &str) -> String {
    format!("{tenant_id}\0{principal_id}")
}

fn get_json_txn<V, T>(
    txn: &T,
    database: Database,
    key: &str,
    label: &str,
) -> Result<Option<V>, String>
where
    V: serde::de::DeserializeOwned,
    T: Transaction,
{
    match txn.get(database, &key) {
        Ok(value) => serde_json::from_slice(value)
            .map(Some)
            .map_err(|error| format!("{label}_decode_failed: {error}")),
        Err(Error::NotFound) => Ok(None),
        Err(error) => Err(format!("failed to read {label}: {error}")),
    }
}

fn put_json_txn<V: serde::Serialize>(
    txn: &mut RwTransaction<'_>,
    database: Database,
    key: &str,
    value: &V,
    flags: WriteFlags,
    label: &str,
) -> Result<(), String> {
    let encoded =
        serde_json::to_vec(value).map_err(|error| format!("{label}_encode_failed: {error}"))?;
    txn.put(database, &key, &encoded, flags)
        .map_err(|error| format!("failed to store {label}: {error}"))
}

fn effective_policy_case_key(case_id: &str) -> String {
    format!("effective_policy:case:{case_id}")
}

fn require_open_case_for_policy(state: &CaseState) -> Result<(), String> {
    if state.lifecycle != CaseLifecycle::Open {
        Err("case_policy_mutation_requires_open_case".to_string())
    } else if state.cancellation.is_some() {
        Err("case_policy_mutation_forbidden_after_cancellation".to_string())
    } else {
        Ok(())
    }
}

fn case_runtime_admission_key(case_id: &str) -> String {
    format!("case-runtime-admission:{case_id}")
}

fn policy_source_key(source_id: &str) -> String {
    format!("policy-source:id:{source_id}")
}

fn policy_artifact_key(artifact_id: &str) -> String {
    format!("policy-artifact:id:{artifact_id}")
}

fn policy_lineage_key(lineage: &PolicyLineage) -> String {
    format!("policy-current:lineage:{}", lineage.identity())
}

fn policy_store_write_error(operation: &str, error: Error) -> String {
    if error == Error::MapFull {
        format!("policy_catalog_capacity_exhausted: {operation}")
    } else {
        format!("failed to store {operation}: {error}")
    }
}

fn policy_event_key(event_id: &str) -> String {
    format!("policy-lifecycle:id:{event_id}")
}

fn policy_sequence_key(sequence: u64) -> String {
    format!("policy-lifecycle:sequence:{sequence:020}")
}

fn next_policy_lifecycle_sequence(
    txn: &mut RwTransaction<'_>,
    schema_meta: Database,
) -> Result<u64, String> {
    let key = "meta:policy_lifecycle_last_sequence";
    let current = match txn.get(schema_meta, &key) {
        Ok(value) => std::str::from_utf8(value)
            .map_err(|error| format!("policy_lifecycle_sequence_not_utf8: {error}"))?
            .parse::<u64>()
            .map_err(|error| format!("policy_lifecycle_sequence_invalid: {error}"))?,
        Err(Error::NotFound) => 0,
        Err(error) => return Err(format!("failed to read policy lifecycle sequence: {error}")),
    };
    let next = current
        .checked_add(1)
        .ok_or_else(|| "policy_lifecycle_sequence_exhausted".to_string())?;
    let encoded = next.to_string();
    txn.put(schema_meta, &key, &encoded, WriteFlags::empty())
        .map_err(|error| format!("failed to advance policy lifecycle sequence: {error}"))?;
    Ok(next)
}

fn decode_policy_source(value: &[u8]) -> Result<PolicySourceArtifact, String> {
    let source: PolicySourceArtifact = serde_json::from_slice(value)
        .map_err(|error| format!("policy_source_decode_failed: {error}"))?;
    source.validate()?;
    Ok(source)
}

fn decode_policy_artifact(value: &[u8]) -> Result<PolicyArtifact, String> {
    let artifact: PolicyArtifact = serde_json::from_slice(value)
        .map_err(|error| format!("policy_artifact_decode_failed: {error}"))?;
    artifact.validate()?;
    Ok(artifact)
}

fn decode_policy_lifecycle_event(value: &[u8]) -> Result<PolicyLifecycleEvent, String> {
    let event: PolicyLifecycleEvent = serde_json::from_slice(value)
        .map_err(|error| format!("policy_lifecycle_event_decode_failed: {error}"))?;
    event.validate()?;
    Ok(event)
}

fn validate_runtime_admission_request(request: &CaseRuntimeAdmissionRequest) -> Result<(), String> {
    if request.case_id.is_empty()
        || request.run_id.is_empty()
        || request.owner_token.is_empty()
        || request.owner_pid == 0
        || request.lease_duration_ms == 0
    {
        return Err("invalid_case_runtime_admission_request".to_string());
    }
    Ok(())
}

fn decode_runtime_admission(value: &[u8]) -> Result<CaseRuntimeAdmission, String> {
    let admission: CaseRuntimeAdmission = serde_json::from_slice(value)
        .map_err(|error| format!("case_runtime_admission_decode_failed: {error}"))?;
    if admission.schema != CASE_RUNTIME_ADMISSION_SCHEMA
        || admission.case_id.is_empty()
        || admission.run_id.is_empty()
        || admission.owner_token.is_empty()
        || admission.owner_pid == 0
        || admission.expires_at_unix_ms < admission.acquired_at_unix_ms
    {
        return Err("invalid_case_runtime_admission_record".to_string());
    }
    Ok(admission)
}

fn validate_runtime_instance_request(
    request: &RuntimeInstanceAcquireRequest,
) -> Result<(), String> {
    request.config.validate()?;
    if request.owner_pid == 0 || request.owner_token.is_empty() || request.lease_duration_ms == 0 {
        return Err("invalid_runtime_instance_request".to_string());
    }
    Ok(())
}

fn validate_runtime_instance(instance: &RuntimeInstance) -> Result<(), String> {
    instance.config.validate()?;
    if (instance.schema != RUNTIME_INSTANCE_SCHEMA && instance.schema != RUNTIME_INSTANCE_SCHEMA_V1)
        || instance.instance_id != RUNTIME_INSTANCE_ID
        || instance.principal_id.is_empty()
        || instance.owner_pid == 0
        || instance.owner_token.is_empty()
        || (instance.schema == RUNTIME_INSTANCE_SCHEMA
            && instance.owner_process_identity.is_empty())
        || instance.heartbeat_at_unix_ms < instance.acquired_at_unix_ms
        || instance.lease_expires_at_unix_ms < instance.heartbeat_at_unix_ms
    {
        return Err("invalid_runtime_instance_record".to_string());
    }
    if instance.integrity_digest != runtime_instance_integrity_digest(instance)? {
        return Err("runtime_instance_integrity_mismatch".to_string());
    }
    Ok(())
}

fn runtime_instance_integrity_digest(instance: &RuntimeInstance) -> Result<String, String> {
    let material_value = if instance.schema == RUNTIME_INSTANCE_SCHEMA_V1 {
        serde_json::json!({
            "schema": instance.schema,
            "instance_id": instance.instance_id,
            "principal_id": instance.principal_id,
            "owner_pid": instance.owner_pid,
            "owner_token": instance.owner_token,
            "lifecycle": instance.lifecycle,
            "config": instance.config,
            "acquired_at_unix_ms": instance.acquired_at_unix_ms,
            "heartbeat_at_unix_ms": instance.heartbeat_at_unix_ms,
            "lease_expires_at_unix_ms": instance.lease_expires_at_unix_ms,
            "recovered_items": instance.recovered_items,
            "last_detail": instance.last_detail,
        })
    } else {
        serde_json::json!({
            "schema": instance.schema,
            "instance_id": instance.instance_id,
            "principal_id": instance.principal_id,
            "owner_pid": instance.owner_pid,
            "owner_process_identity": instance.owner_process_identity,
            "owner_token": instance.owner_token,
            "lifecycle": instance.lifecycle,
            "config": instance.config,
            "acquired_at_unix_ms": instance.acquired_at_unix_ms,
            "heartbeat_at_unix_ms": instance.heartbeat_at_unix_ms,
            "lease_expires_at_unix_ms": instance.lease_expires_at_unix_ms,
            "recovered_items": instance.recovered_items,
            "last_dispatched_tenant": instance.last_dispatched_tenant,
            "drain_requested_at_unix_ms": instance.drain_requested_at_unix_ms,
            "drain_requested_by_principal": instance.drain_requested_by_principal,
            "last_detail": instance.last_detail,
        })
    };
    let material = serde_json::to_string(&material_value)
        .map_err(|error| format!("runtime_instance_encode_failed: {error}"))?;
    Ok(crate::context::stable_digest(&material))
}

#[cfg(target_os = "linux")]
fn runtime_process_identity(pid: u32) -> Result<String, String> {
    if pid <= 1 {
        return Err("runtime_process_identity_invalid_pid".to_string());
    }
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).map_err(|error| {
        format!("runtime_process_identity_unavailable: pid={pid} error={error}")
    })?;
    let command_end = stat
        .rfind(')')
        .ok_or_else(|| "runtime_process_identity_stat_malformed".to_string())?;
    let fields = stat[command_end + 1..]
        .split_whitespace()
        .collect::<Vec<_>>();
    let start_ticks = fields
        .get(19)
        .ok_or_else(|| "runtime_process_identity_stat_missing_starttime".to_string())?;
    start_ticks
        .parse::<u64>()
        .map_err(|error| format!("runtime_process_identity_starttime_invalid: {error}"))?;
    let boot_id = fs::read_to_string("/proc/sys/kernel/random/boot_id")
        .map_err(|error| format!("runtime_process_identity_boot_id_unavailable: {error}"))?;
    let boot_id = boot_id.trim();
    if boot_id.is_empty() {
        return Err("runtime_process_identity_boot_id_empty".to_string());
    }
    Ok(format!("linux-proc-v1:{boot_id}:{start_ticks}"))
}

#[cfg(not(target_os = "linux"))]
fn runtime_process_identity(_pid: u32) -> Result<String, String> {
    Err("runtime_process_identity_unsupported_platform".to_string())
}

fn process_identity_matches(
    stored_pid: u32,
    stored_identity: &str,
    observed_pid: u32,
    observed_identity: &str,
) -> bool {
    stored_pid == observed_pid
        && !stored_identity.is_empty()
        && stored_identity == observed_identity
}

fn runtime_owner_process_is_live(instance: &RuntimeInstance) -> bool {
    runtime_process_identity(instance.owner_pid).is_ok_and(|observed| {
        if instance.owner_process_identity.is_empty() {
            true
        } else {
            process_identity_matches(
                instance.owner_pid,
                &instance.owner_process_identity,
                instance.owner_pid,
                &observed,
            )
        }
    })
}

pub fn runtime_process_identity_is_live(pid: u32, expected_identity: &str) -> bool {
    runtime_process_identity(pid)
        .is_ok_and(|observed| process_identity_matches(pid, expected_identity, pid, &observed))
}

fn runtime_instance_owned_by_current_process(instance: &RuntimeInstance) -> Result<bool, String> {
    let current_pid = std::process::id();
    let current_identity = runtime_process_identity(current_pid)?;
    Ok(process_identity_matches(
        instance.owner_pid,
        &instance.owner_process_identity,
        current_pid,
        &current_identity,
    ))
}

fn runtime_instance_owner_txn<T: Transaction>(
    txn: &T,
    database: Database,
    principal_id: &str,
    owner_token: &str,
    now_unix_ms: u64,
) -> Result<RuntimeInstance, String> {
    let instance =
        get_json_txn::<RuntimeInstance, _>(txn, database, RUNTIME_INSTANCE_ID, "runtime_instance")?
            .ok_or_else(|| "runtime_instance_missing".to_string())?;
    validate_runtime_instance(&instance)?;
    if instance.principal_id != principal_id || instance.owner_token != owner_token {
        return Err("runtime_instance_owner_mismatch".to_string());
    }
    if !runtime_instance_owned_by_current_process(&instance)? {
        return Err("runtime_instance_owner_process_mismatch".to_string());
    }
    let _ = now_unix_ms;
    Ok(instance)
}

fn validate_runtime_work_submission(submission: &RuntimeWorkSubmission) -> Result<(), String> {
    submission.budgets.validate()?;
    if submission.request_id.is_empty()
        || submission.request_id.len() > MAX_RUNTIME_WORK_REQUEST_ID_BYTES
        || submission.request_id.len() > 256
        || submission.tenant_id.is_empty()
        || submission.case_id.is_empty()
        || submission.participant_id.is_empty()
        || submission.attachment_id.is_empty()
        || submission.journal_path.is_empty()
        || submission.journal_path.len() > MAX_RUNTIME_WORK_JOURNAL_PATH_BYTES
        || submission.task.is_empty()
        || submission.task.len() > MAX_RUNTIME_WORK_TASK_BYTES
        || submission
            .failpoint
            .as_ref()
            .is_some_and(|value| value.len() > 128)
    {
        return Err("invalid_runtime_work_submission".to_string());
    }
    Ok(())
}

fn runtime_submission_digest(
    principal_id: &str,
    submission: &RuntimeWorkSubmission,
) -> Result<String, String> {
    let material = serde_json::to_string(&serde_json::json!({
        "schema": RUNTIME_WORK_ITEM_SCHEMA,
        "request_id": submission.request_id,
        "principal_id": principal_id,
        "tenant_id": submission.tenant_id,
        "case_id": submission.case_id,
        "participant_id": submission.participant_id,
        "attachment_id": submission.attachment_id,
        "journal_path": submission.journal_path,
        "task": submission.task,
        "budgets": submission.budgets,
        "failpoint": submission.failpoint,
    }))
    .map_err(|error| format!("runtime_work_submission_encode_failed: {error}"))?;
    Ok(crate::context::stable_digest(&material))
}

fn runtime_work_integrity_digest(item: &RuntimeWorkItem) -> Result<String, String> {
    let mut material_value = serde_json::json!({
        "schema": item.schema,
        "work_id": item.work_id,
        "request_id": item.request_id,
        "request_digest": item.request_digest,
        "principal_id": item.principal_id,
        "tenant_id": item.tenant_id,
        "case_id": item.case_id,
        "participant_id": item.participant_id,
        "attachment_id": item.attachment_id,
        "journal_path": item.journal_path,
        "task": item.task,
        "budgets": item.budgets,
        "failpoint": item.failpoint,
        "enqueue_sequence": item.enqueue_sequence,
        "state": item.state,
        "attempt_count": item.attempt_count,
        "runtime_instance_id": item.runtime_instance_id,
        "runtime_owner_token": item.runtime_owner_token,
        "worker_id": item.worker_id,
        "last_stop_reason": item.last_stop_reason,
        "enqueued_at_unix_ms": item.enqueued_at_unix_ms,
        "updated_at_unix_ms": item.updated_at_unix_ms,
    });
    if item.schema != RUNTIME_WORK_ITEM_SCHEMA_V1 {
        material_value["workflow"] = serde_json::to_value(&item.workflow)
            .map_err(|error| format!("runtime_workflow_context_encode_failed: {error}"))?;
    }
    let material = serde_json::to_string(&material_value)
        .map_err(|error| format!("runtime_work_item_encode_failed: {error}"))?;
    Ok(crate::context::stable_digest(&material))
}

fn runtime_work_idempotency_key(principal_id: &str, tenant_id: &str, request_id: &str) -> String {
    format!("runtime-work-idempotency:{principal_id}\0{tenant_id}\0{request_id}")
}

fn next_runtime_work_sequence(
    txn: &mut RwTransaction<'_>,
    schema_meta: Database,
) -> Result<u64, String> {
    let key = "meta:runtime_work_last_sequence";
    let current = match txn.get(schema_meta, &key) {
        Ok(value) => std::str::from_utf8(value)
            .map_err(|error| format!("runtime_work_sequence_not_utf8: {error}"))?
            .parse::<u64>()
            .map_err(|error| format!("runtime_work_sequence_invalid: {error}"))?,
        Err(Error::NotFound) => 0,
        Err(error) => return Err(format!("failed to read runtime work sequence: {error}")),
    };
    let next = current
        .checked_add(1)
        .ok_or_else(|| "runtime_work_sequence_exhausted".to_string())?;
    let encoded = next.to_string();
    txn.put(schema_meta, &key, &encoded, WriteFlags::empty())
        .map_err(|error| format!("failed to advance runtime work sequence: {error}"))?;
    Ok(next)
}

fn list_runtime_work_items_txn<T: Transaction>(
    txn: &T,
    database: Database,
) -> Result<Vec<RuntimeWorkItem>, String> {
    let mut cursor = txn
        .open_ro_cursor(database)
        .map_err(|error| format!("failed to open runtime work cursor: {error}"))?;
    let mut items = Vec::new();
    for (_, raw) in cursor.iter() {
        let item: RuntimeWorkItem = serde_json::from_slice(raw)
            .map_err(|error| format!("runtime_work_item_decode_failed: {error}"))?;
        item.validate_integrity()?;
        items.push(item);
    }
    drop(cursor);
    Ok(items)
}

fn semantic_context_artifact_key(artifact_id: &str) -> String {
    format!("semantic-context:id:{artifact_id}")
}

fn operational_memory_id_key(memory_id: &str) -> String {
    format!("operational-memory:id:{memory_id}")
}

fn operational_memory_case_key(case_id: &str) -> String {
    format!("operational-memory:case:{case_id}")
}

fn validate_operational_memory_build(build: &OperationalMemoryBuild) -> Result<(), String> {
    if build.manifest.schema != OPERATIONAL_MEMORY_MANIFEST_SCHEMA {
        return Err(format!(
            "unsupported_operational_memory_manifest_schema: {}",
            build.manifest.schema
        ));
    }
    if build.manifest.derivation_version != OPERATIONAL_MEMORY_DERIVATION {
        return Err(format!(
            "unsupported_memory_derivation: {}",
            build.manifest.derivation_version
        ));
    }
    let manifest_ids = build
        .manifest
        .memory_ids
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let entry_ids = build
        .entries
        .iter()
        .map(|entry| entry.memory_id.clone())
        .collect::<std::collections::BTreeSet<_>>();
    if manifest_ids.len() != build.manifest.memory_ids.len()
        || entry_ids.len() != build.entries.len()
        || manifest_ids != entry_ids
    {
        return Err("operational_memory_manifest_entry_mismatch".to_string());
    }
    for entry in &build.entries {
        entry.validate()?;
        if entry.case_id != build.manifest.case_id
            || entry.derived_at_generation != build.manifest.source_generation
        {
            return Err("operational_memory_build_case_generation_mismatch".to_string());
        }
    }
    Ok(())
}

fn local_binding_key(case_id: &str, attachment_id: &str) -> String {
    format!(
        "local_binding:{}:{}:{}:{}",
        case_id.len(),
        case_id,
        attachment_id.len(),
        attachment_id
    )
}

fn resource_control_state_key(resource_id: &str) -> String {
    format!("resource-control:state:{resource_id}")
}

fn resource_control_event_key(event_id: &str) -> String {
    format!("resource-control:event:{event_id}")
}

fn resource_owner_is_live(fence: &ResourceFence) -> bool {
    LocalProcessIdentity::capture(fence.owner_pid)
        .is_ok_and(|identity| identity.canonical_identity() == fence.owner_process_identity)
}

fn json_string_field(content: &str, key: &str) -> Option<String> {
    let marker = format!("\"{key}\":\"");
    let start = content.find(&marker)? + marker.len();
    let rest = &content[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn json_usize_field(content: &str, key: &str) -> usize {
    let marker = format!("\"{key}\":");
    let Some(start) = content.find(&marker).map(|index| index + marker.len()) else {
        return 0;
    };
    let rest = &content[start..];
    let end = rest.find([',', '}']).unwrap_or(rest.len());
    rest[..end].trim().parse::<usize>().unwrap_or(0)
}

fn json_u128_field(content: &str, key: &str) -> u128 {
    let marker = format!("\"{key}\":");
    let Some(start) = content.find(&marker).map(|index| index + marker.len()) else {
        return 0;
    };
    let rest = &content[start..];
    let end = rest.find([',', '}']).unwrap_or(rest.len());
    rest[..end].trim().parse::<u128>().unwrap_or(0)
}

fn replay_metadata_key(journal_identity: &str) -> String {
    format!("meta:replay:{journal_identity}")
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn unix_time_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn authority_wall_time_unix_ms() -> u64 {
    // One authority write observes one whole-second value. This makes the
    // derive/commit comparison deterministic while the persisted floor still
    // prevents rollback from expanding authority.
    ((unix_time_ms() as u64) / 1_000) * 1_000
}

fn validity_severity(posture: &PolicyValidityPosture) -> u8 {
    match posture {
        PolicyValidityPosture::Valid => 0,
        PolicyValidityPosture::NotYetValid => 1,
        PolicyValidityPosture::RefreshRequired => 2,
        PolicyValidityPosture::Stale => 3,
        PolicyValidityPosture::Expired => 4,
        PolicyValidityPosture::Revoked => 5,
        PolicyValidityPosture::Unavailable => 6,
    }
}

fn binding_validity_posture(
    artifact: &PolicyArtifact,
    drift: &PolicyCatalogDrift,
    revoked: bool,
    authority_time_unix_ms: u64,
) -> (PolicyValidityPosture, String) {
    if revoked || matches!(drift, PolicyCatalogDrift::Revoked) {
        return (
            PolicyValidityPosture::Revoked,
            "explicit_policy_revocation".to_string(),
        );
    }
    let temporal = match artifact.validity.mode {
        PolicyValidityMode::Unbounded => (PolicyValidityPosture::Valid, "unbounded".to_string()),
        PolicyValidityMode::Bounded => {
            let valid_from = artifact.validity.valid_from_unix_ms.unwrap_or(u64::MAX);
            let refresh_after = artifact.validity.refresh_after_unix_ms.unwrap_or(u64::MAX);
            let expires_at = artifact.validity.expires_at_unix_ms.unwrap_or(0);
            if authority_time_unix_ms < valid_from {
                (
                    PolicyValidityPosture::NotYetValid,
                    "before_valid_from".to_string(),
                )
            } else if authority_time_unix_ms >= expires_at {
                (
                    PolicyValidityPosture::Expired,
                    "authority_time_reached_expires_at".to_string(),
                )
            } else if authority_time_unix_ms >= refresh_after {
                (
                    PolicyValidityPosture::RefreshRequired,
                    "authority_time_reached_refresh_after".to_string(),
                )
            } else {
                (
                    PolicyValidityPosture::Valid,
                    "inside_validity_window".to_string(),
                )
            }
        }
    };
    let catalog = match drift {
        PolicyCatalogDrift::Current => (PolicyValidityPosture::Valid, None),
        PolicyCatalogDrift::Superseded { .. } => (
            PolicyValidityPosture::Stale,
            Some("bound_artifact_superseded"),
        ),
        PolicyCatalogDrift::Retired => {
            (PolicyValidityPosture::Stale, Some("bound_artifact_retired"))
        }
        PolicyCatalogDrift::NoCurrentPublishedArtifact => (
            PolicyValidityPosture::Stale,
            Some("no_current_published_artifact"),
        ),
        PolicyCatalogDrift::Revoked => unreachable!("handled above"),
    };
    if validity_severity(&catalog.0) > validity_severity(&temporal.0) {
        (catalog.0, catalog.1.unwrap_or("catalog_stale").to_string())
    } else {
        temporal
    }
}

fn closure_blockers(state: &CaseState) -> Vec<String> {
    let mut blockers = Vec::new();
    for review in &state.reviews {
        if matches!(
            review.status,
            ReviewResolution::Pending | ReviewResolution::Deferred
        ) {
            blockers.push(format!("usable_review:{}", review.review_id));
        }
    }
    for grant in &state.grants {
        if grant.status == GrantLifecycle::Issued {
            blockers.push(format!("usable_grant:{}", grant.grant_id));
        }
    }
    for effect in &state.effects {
        if matches!(
            effect.status,
            crate::transition::EffectLifecycle::Prepared
                | crate::transition::EffectLifecycle::Indeterminate
        ) {
            blockers.push(format!("unresolved_effect:{}", effect.effect_id));
        }
    }
    blockers.sort();
    blockers
}

/// Returns whether a reference names canonical truth committed in this exact
/// Transition. Handoff evidence uses this closed projection so a syntactically
/// plausible identifier from another Case can never become target-local proof.
fn transition_contains_canonical_fact_ref(transition: &Transition, reference: &str) -> bool {
    if transition.transition_id == reference {
        return true;
    }
    match &transition.payload {
        TransitionPayload::ParticipantBound { participant_id, .. }
        | TransitionPayload::ParticipantAdmitted { participant_id, .. }
        | TransitionPayload::ProviderAttached { participant_id, .. } => participant_id == reference,
        TransitionPayload::ParticipantPrincipalLinked { link } => link.link_id == reference,
        TransitionPayload::ProviderInvocationStarted { invocation_id, .. } => {
            invocation_id == reference
        }
        TransitionPayload::ProviderResultRecorded {
            result_id,
            invocation_id,
            ..
        } => result_id == reference || invocation_id == reference,
        TransitionPayload::InteractionTurnRecorded {
            turn_id,
            invocation_id,
            result_id,
            ..
        } => turn_id == reference || invocation_id == reference || result_id == reference,
        TransitionPayload::ModelInterpretationRecorded {
            interpretation_id,
            result_id,
            ..
        } => interpretation_id == reference || result_id == reference,
        TransitionPayload::ResourceAttached { attachment } => attachment.attachment_id == reference,
        TransitionPayload::OperationNormalizationFailed {
            provider_result_id, ..
        } => provider_result_id == reference,
        TransitionPayload::OperationRecorded { operation } => operation.operation_id == reference,
        TransitionPayload::DecisionRecorded { decision } => {
            decision.decision_id == reference
                || decision
                    .decision_basis
                    .as_ref()
                    .is_some_and(|basis| basis.basis_id == reference)
        }
        TransitionPayload::ExecutionGrantIssued { grant } => grant.grant_id == reference,
        TransitionPayload::EffectPrepared { prepared } => prepared.effect_id == reference,
        TransitionPayload::ProcessEffectPrepared { prepared } => prepared.effect_id == reference,
        TransitionPayload::EffectFinalized {
            effect_id, receipt, ..
        } => effect_id == reference || receipt.receipt_id == reference,
        TransitionPayload::ProcessEffectFinalized {
            effect_id, receipt, ..
        } => effect_id == reference || receipt.receipt_id == reference,
        TransitionPayload::EffectIndeterminate { effect_id, .. }
        | TransitionPayload::ProcessEffectIndeterminate { effect_id, .. } => effect_id == reference,
        TransitionPayload::EffectReconciled {
            effect_id, receipt, ..
        } => {
            effect_id == reference
                || receipt
                    .as_ref()
                    .is_some_and(|value| value.receipt_id == reference)
        }
        TransitionPayload::ReviewRequested { review } => review.review_id == reference,
        TransitionPayload::ReviewActionRecorded { action } => action.action_id == reference,
        TransitionPayload::ReviewInvalidated { invalidation } => {
            invalidation.review_id == reference
        }
        TransitionPayload::ExecutionGrantInvalidated { invalidation } => {
            invalidation.grant_id == reference
        }
        TransitionPayload::CasePolicyBound { binding }
        | TransitionPayload::CasePolicyReplaced { binding, .. } => binding.binding_id == reference,
        TransitionPayload::CasePolicyUnbound { binding_id, .. } => binding_id == reference,
        TransitionPayload::CaseWorkflowBound { binding } => binding.binding_id == reference,
        TransitionPayload::WorkflowNodeExecutionStarted { execution } => {
            execution.execution_id == reference
        }
        TransitionPayload::WorkflowNodeSatisfied { satisfaction } => {
            satisfaction.satisfaction_id == reference
        }
        TransitionPayload::WorkflowConditionResolved { resolution } => {
            resolution.resolution_id == reference
        }
        TransitionPayload::WorkflowHumanInputRecorded { input } => input.input_id == reference,
        TransitionPayload::WorkflowDeterministicProposalRecorded { proposal } => {
            proposal.proposal_id == reference
        }
        TransitionPayload::WorkflowPlanPatchProposed { patch } => patch.patch_id == reference,
        TransitionPayload::WorkflowAmendmentAdopted { amendment } => {
            amendment.amendment_id == reference
        }
        TransitionPayload::HandoffOffered { offer } => offer.handoff_id == reference,
        TransitionPayload::HandoffAccepted { acceptance } => acceptance.acceptance_id == reference,
        TransitionPayload::HandoffDeclined { decline } => decline.decline_id == reference,
        TransitionPayload::HandoffResultRecorded { result } => result.result_id == reference,
        TransitionPayload::HandoffReconciled { reconciliation } => {
            reconciliation.reconciliation_id == reference
        }
        TransitionPayload::CaseOpened { .. }
        | TransitionPayload::TenantCaseOpened { .. }
        | TransitionPayload::CaseCancellationRequested { .. }
        | TransitionPayload::CaseClosed { .. }
        | TransitionPayload::ReviewResolved { .. } => false,
    }
}

impl ResourceFenceAuthority for LmdbRecordStore {
    fn validate_carrier_fence(&self, fence: &ResourceFence) -> Result<(), String> {
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to validate carrier fence: {error}"))?;
        self.validate_carrier_fence_txn(&txn, fence, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admission::{
        evaluate_filesystem_admission, forged_evidence_resolution_for_test,
        CanonicalEvidenceResolution,
    };
    use crate::context::{RenderedInputMetadata, SemanticContextArtifact, RENDERED_INPUT_SCHEMA};
    use crate::effect::{
        build_effect_receipt, build_filesystem_review_request, build_process_effect_receipt,
        classify_reconciliation, decide_filesystem_write, execute_fenced_filesystem_write,
        execute_fenced_process_signal, execute_filesystem_write, issue_execution_grant,
        issue_policy_execution_grant, normalize_filesystem_write_candidate,
        normalize_process_signal_candidate, observe_filesystem, observe_process, prepare_effect,
        prepare_fenced_effect, prepare_process_effect, reseal_policy_execution_grant_for_test,
        resolve_filesystem_review_decision, validate_finalized_effect_chain, CarrierFailpoint,
        CarrierResult, EffectOutcome, LocalFilesystemBinding, LocalProcessBinding,
        NormalizationContext, ProcessSignalAction, ReconciliationConclusion,
    };
    use crate::governance::{
        compile_policy_source, scope_policy_compilation, PolicyLifecycleState,
        PolicyValidationStatus, POLICY_SOURCE_INPUT_SCHEMA,
    };
    use crate::memory::{derive_operational_memory, OperationalMemoryKind};
    use crate::record::{Record, RecordKind};
    use crate::transition::{
        build_review_action, CaseLifecycle, InterpretationAuthority, PendingTransition,
        ProviderInvocationLineage, ResourceAttachmentState, ResourceKind, ReviewActionKind,
        ReviewRequirement, ReviewResolution, TransitionPayload, TransitionScope, TransitionSource,
    };
    use crate::workflow::{
        HumanInputKind, WorkflowBudgets, WorkflowDefinition, WorkflowDefinitionInput, WorkflowEdge,
        WorkflowEdgeKind, WorkflowExecutorBinding, WorkflowNode, WorkflowNodeKind,
        WorkflowPredicate, WorkflowResourceBinding, WORKFLOW_DEFINITION_SCHEMA,
    };
    use std::collections::BTreeSet;
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Instant;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn secured_pending(
        id: &str,
        case_id: &str,
        generation: u64,
        principal_id: &str,
        payload: TransitionPayload,
    ) -> PendingTransition {
        PendingTransition::new(
            id,
            case_id,
            generation,
            TransitionSource {
                component: "wave12-security-test".to_string(),
                participant_id: None,
                principal_id: Some(principal_id.to_string()),
                source_ref: None,
            },
            payload,
        )
    }

    fn temp_store_path(name: &str) -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("yai-{name}-{}-{now}", std::process::id()))
    }

    fn pending(
        id: &str,
        case_id: &str,
        generation: u64,
        payload: TransitionPayload,
    ) -> PendingTransition {
        PendingTransition::new(
            id,
            case_id,
            generation,
            TransitionSource::component("canonical-test"),
            payload,
        )
    }

    fn test_provider_lineage(generation: u64) -> ProviderInvocationLineage {
        ProviderInvocationLineage {
            projection_id: format!("projection:{generation}"),
            context_frame_id: format!("context-frame:{generation}"),
            case_generation: generation,
            rendered_input_id: format!("rendered-input:{generation}"),
            rendered_input_digest: format!("digest:{generation}"),
            output_contract_id: "output-contract:natural-language".to_string(),
            continuation_disposition: "not_provided".to_string(),
        }
    }

    fn test_temporal_context() -> AuthorityTemporalContext {
        AuthorityTemporalContext {
            authority_time_unix_ms: authority_wall_time_unix_ms(),
            binding_validity: vec![BindingValidity {
                binding_id: "binding:one".to_string(),
                lineage_id: "lineage:one".to_string(),
                artifact_id: "artifact:one".to_string(),
                contract: crate::governance::PolicyValidityContract::default(),
                posture: PolicyValidityPosture::Valid,
                reason: "unbounded".to_string(),
                revoke_event_id: None,
            }],
        }
    }

    fn policy_source(version: &str, required: bool) -> Vec<u8> {
        policy_source_for(
            "organization:example",
            "organization.example.filesystem",
            version,
            required,
        )
    }

    fn policy_source_for(
        owner_ref: &str,
        policy_key: &str,
        version: &str,
        required: bool,
    ) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "schema": POLICY_SOURCE_INPUT_SCHEMA,
            "policy_key": policy_key,
            "source_version": version,
            "owner_ref": owner_ref,
            "validity": {"mode":"unbounded"},
            "source_origin": {
                "source_system": "unit-test",
                "source_uri": format!("test://governance/{owner_ref}/{policy_key}/{version}")
            },
            "rules": [
                {
                    "kind": "review_requirement",
                    "rule_id": format!("review-v{version}"),
                    "operation_kind": "filesystem.write",
                    "resource_kind": "filesystem",
                    "required": required,
                    "reason": "filesystem writes use explicit governance"
                },
                {
                    "kind": "evidence_obligation",
                    "rule_id": format!("post-observation-v{version}"),
                    "operation_kind": "filesystem.write",
                    "resource_kind": "filesystem",
                    "obligation": "post_observation",
                    "reason": "observed consequence is required"
                }
            ]
        }))
        .expect("serialize policy source fixture")
    }

    fn large_policy_source(index: usize) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "schema": POLICY_SOURCE_INPUT_SCHEMA,
            "policy_key": format!("capacity.policy.{index}"),
            "source_version": "1",
            "owner_ref": "organization:capacity",
            "source_origin": {
                "source_system": "capacity-test",
                "source_uri": format!("test://capacity/{index}")
            },
            "validity": {"mode":"unbounded"},
            "rules": [{
                "kind": "future_bounded_rule",
                "opaque_payload": "x".repeat(240 * 1024)
            }]
        }))
        .expect("serialize large policy source")
    }

    fn commit_typed(
        store: &LmdbRecordStore,
        id: &str,
        case_id: &str,
        generation: u64,
        payload: TransitionPayload,
        scope: Option<TransitionScope>,
        causal_refs: Vec<String>,
    ) -> CanonicalCommit {
        let mut value = pending(id, case_id, generation, payload);
        value.scope = scope;
        value.causal_refs = causal_refs;
        store
            .commit_transition(value)
            .expect("commit typed transition")
    }

    #[test]
    fn canonical_transition_and_case_state_are_atomic_replayable_and_restart_safe() {
        let path = temp_store_path("canonical-authority");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        let opened = store
            .commit_transition(pending(
                "transition:open",
                "case:canonical",
                0,
                TransitionPayload::CaseOpened {
                    lifecycle: CaseLifecycle::Open,
                },
            ))
            .expect("commit Case open");
        assert_eq!(opened.transition.sequence, 1);
        assert_eq!(opened.state.generation, 1);

        let bound = store
            .commit_transition(pending(
                "transition:bound",
                "case:canonical",
                1,
                TransitionPayload::ParticipantBound {
                    participant_id: "participant:model".to_string(),
                    role: "model_participant".to_string(),
                },
            ))
            .expect("commit participant binding");
        assert_eq!(bound.transition.sequence, 2);
        assert_eq!(bound.state.generation, 2);
        assert!(store
            .verify_case_state("case:canonical")
            .expect("verify replay"));

        let transitions = store
            .list_case_transitions("case:canonical")
            .expect("list canonical transitions");
        assert_eq!(
            transitions
                .iter()
                .map(|transition| transition.sequence)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        let expected = bound.state;
        store
            .discard_case_state_for_test("case:canonical")
            .expect("discard materialization");
        assert_eq!(
            store.get_case_state("case:canonical").expect("state read"),
            None
        );
        assert_eq!(
            store
                .rebuild_case_state("case:canonical")
                .expect("rebuild CaseState"),
            expected
        );
        drop(store);

        let reopened = LmdbRecordStore::open(&path).expect("reopen after restart");
        assert_eq!(
            reopened
                .get_case_state("case:canonical")
                .expect("restart state"),
            Some(expected)
        );
        assert!(reopened
            .verify_case_state("case:canonical")
            .expect("restart replay"));
        drop(reopened);
        fs::remove_dir_all(path).expect("remove LMDB test store");
    }

    #[test]
    fn deleting_semantic_context_artifacts_cannot_delete_case_continuity() {
        let path = temp_store_path("context-artifact-derived");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        store
            .commit_transition(pending(
                "transition:open",
                "case:context-artifact",
                0,
                TransitionPayload::CaseOpened {
                    lifecycle: CaseLifecycle::Open,
                },
            ))
            .expect("commit canonical case");
        let before = store
            .get_case_state("case:context-artifact")
            .unwrap()
            .unwrap();
        let artifact = SemanticContextArtifact::RenderedInputMetadata(RenderedInputMetadata {
            schema: RENDERED_INPUT_SCHEMA.to_string(),
            rendered_input_id: "rendered-input:test".to_string(),
            context_frame_id: "context-frame:test".to_string(),
            provider_id: "provider:test".to_string(),
            model_id: "model:test".to_string(),
            content_digest: "digest:test".to_string(),
            content_chars: 42,
        });
        store
            .put_semantic_context_artifact(&artifact)
            .expect("store derived artifact");
        assert_eq!(
            store
                .get_semantic_context_artifact("rendered-input:test")
                .unwrap(),
            Some(artifact)
        );
        store
            .clear_semantic_context_artifacts()
            .expect("clear derived artifacts");
        assert_eq!(
            store
                .get_semantic_context_artifact("rendered-input:test")
                .unwrap(),
            None
        );
        assert_eq!(
            store
                .get_case_state("case:context-artifact")
                .unwrap()
                .unwrap(),
            before
        );
        fs::remove_dir_all(path).expect("remove temp store");
    }

    #[test]
    fn policy_intake_is_case_independent_idempotent_and_query_pure() {
        let path = temp_store_path("policy-intake");
        let store = LmdbRecordStore::open(&path).expect("open LMDB policy store");
        let compilation = compile_policy_source(&policy_source("1", true)).expect("compile policy");
        let canonical_before = store.canonical_summary().expect("canonical summary before");

        let first = store
            .ingest_policy_compilation(&compilation, "participant:policy-admin")
            .expect("ingest policy candidate");
        assert!(first.source_created);
        assert!(first.artifact_created);
        assert_eq!(first.view.lifecycle, PolicyLifecycleState::Candidate);
        assert!(!first.view.runtime_consumable);

        let duplicate = store
            .ingest_policy_compilation(&compilation, "participant:second-observer")
            .expect("duplicate intake is idempotent");
        assert!(!duplicate.source_created);
        assert!(!duplicate.artifact_created);
        assert_eq!(duplicate.view, first.view);
        assert_eq!(
            store
                .list_policy_lifecycle_events(None)
                .expect("events after duplicate")
                .len(),
            1
        );

        let event_count = store
            .list_policy_lifecycle_events(None)
            .expect("events before pure reads")
            .len();
        let listed = store
            .list_policy_artifact_views()
            .expect("list policy artifacts");
        assert_eq!(listed, vec![first.view]);
        assert_eq!(
            store
                .get_policy_source(&compilation.source.source_id)
                .expect("source read"),
            Some(compilation.source.clone())
        );
        assert_eq!(
            store
                .get_policy_artifact(&compilation.artifact.artifact_id)
                .expect("artifact read"),
            Some(compilation.artifact.clone())
        );
        assert_eq!(
            store
                .list_policy_lifecycle_events(None)
                .expect("events after pure reads")
                .len(),
            event_count
        );
        assert_eq!(
            store.canonical_summary().expect("canonical summary after"),
            canonical_before
        );
        assert!(store.get_case_state("case:__system__").unwrap().is_none());
        fs::remove_dir_all(path).expect("remove policy intake store");
    }

    #[test]
    fn policy_lifecycle_publishes_versions_without_mutating_history() {
        let path = temp_store_path("policy-lifecycle");
        let store = LmdbRecordStore::open(&path).expect("open policy lifecycle store");
        let v1 = compile_policy_source(&policy_source("1", true)).expect("compile v1");
        let v2 = compile_policy_source(&policy_source("2", false)).expect("compile v2");
        let v1_original = v1.artifact.clone();

        store
            .ingest_policy_compilation(&v1, "participant:policy-admin")
            .expect("ingest v1");
        assert!(store
            .publish_policy_artifact(
                &v1.artifact.artifact_id,
                "participant:policy-admin",
                "cannot publish before deterministic validation"
            )
            .unwrap_err()
            .contains("must_be_validated"));
        let validated = store
            .validate_policy_artifact(
                &v1.artifact.artifact_id,
                "participant:policy-admin",
                "deterministic qualification passed",
            )
            .expect("validate v1");
        assert_eq!(validated.view.lifecycle, PolicyLifecycleState::Validated);
        assert!(!validated.view.runtime_consumable);
        let published = store
            .publish_policy_artifact(
                &v1.artifact.artifact_id,
                "participant:policy-admin",
                "publish version one",
            )
            .expect("publish v1");
        assert_eq!(published.view.lifecycle, PolicyLifecycleState::Published);
        assert!(published.view.runtime_consumable);

        store
            .ingest_policy_compilation(&v2, "participant:policy-admin")
            .expect("ingest v2");
        store
            .validate_policy_artifact(
                &v2.artifact.artifact_id,
                "participant:policy-admin",
                "deterministic qualification passed",
            )
            .expect("validate v2");
        store
            .publish_policy_artifact(
                &v2.artifact.artifact_id,
                "participant:policy-admin",
                "publish version two",
            )
            .expect("publish v2");

        let old = store
            .policy_artifact_view(&v1.artifact.artifact_id)
            .expect("read v1 view")
            .expect("v1 exists");
        let current = store
            .policy_artifact_view(&v2.artifact.artifact_id)
            .expect("read v2 view")
            .expect("v2 exists");
        assert_eq!(old.artifact, v1_original);
        assert_eq!(old.lifecycle, PolicyLifecycleState::Superseded);
        assert_eq!(old.superseded_by, Some(v2.artifact.artifact_id.clone()));
        assert!(!old.runtime_consumable);
        assert_eq!(current.lifecycle, PolicyLifecycleState::Published);
        assert!(current.runtime_consumable);
        assert!(store
            .publish_policy_artifact(
                &v1.artifact.artifact_id,
                "participant:policy-admin",
                "attempt supersession cycle"
            )
            .unwrap_err()
            .contains("current=Superseded"));
        drop(store);

        let reopened = LmdbRecordStore::open(&path).expect("restart policy store");
        assert_eq!(
            reopened
                .get_policy_artifact(&v1.artifact.artifact_id)
                .unwrap(),
            Some(v1_original)
        );
        assert_eq!(
            reopened
                .policy_artifact_view(&v2.artifact.artifact_id)
                .unwrap()
                .unwrap()
                .lifecycle,
            PolicyLifecycleState::Published
        );
        let retired = reopened
            .retire_policy_artifact(
                &v2.artifact.artifact_id,
                "participant:policy-admin",
                "version withdrawn from future Case use",
            )
            .expect("retire v2");
        assert!(retired.changed);
        assert_eq!(retired.view.lifecycle, PolicyLifecycleState::Retired);
        assert!(!retired.view.runtime_consumable);
        assert!(
            !reopened
                .retire_policy_artifact(
                    &v2.artifact.artifact_id,
                    "participant:policy-admin",
                    "version withdrawn from future Case use",
                )
                .expect("repeat retirement")
                .changed
        );
        fs::remove_dir_all(path).expect("remove policy lifecycle store");
    }

    #[test]
    fn policy_lineage_is_owner_scoped_and_declared_versions_are_immutable() {
        let path = temp_store_path("policy-lineage");
        let store = LmdbRecordStore::open(&path).expect("open policy lineage store");
        let key = "production.filesystem";
        let owner_a = compile_policy_source(&policy_source_for("organization:a", key, "1", true))
            .expect("compile owner A");
        let owner_b = compile_policy_source(&policy_source_for("organization:b", key, "1", false))
            .expect("compile owner B");
        for artifact in [&owner_a, &owner_b] {
            store
                .ingest_policy_compilation(artifact, "participant:local-operator")
                .expect("ingest independent lineage");
            store
                .validate_policy_artifact(
                    &artifact.artifact.artifact_id,
                    "participant:local-operator",
                    "deterministic validation",
                )
                .expect("validate independent lineage");
            store
                .publish_policy_artifact(
                    &artifact.artifact.artifact_id,
                    "participant:local-operator",
                    "publish independent lineage",
                )
                .expect("publish independent lineage");
        }
        assert_eq!(
            store
                .current_published_policy("organization:a", key)
                .unwrap()
                .unwrap()
                .artifact
                .artifact_id,
            owner_a.artifact.artifact_id
        );
        assert_eq!(
            store
                .current_published_policy("organization:b", key)
                .unwrap()
                .unwrap()
                .artifact
                .artifact_id,
            owner_b.artifact.artifact_id
        );
        assert_eq!(
            store
                .policy_artifact_view(&owner_a.artifact.artifact_id)
                .unwrap()
                .unwrap()
                .lifecycle,
            PolicyLifecycleState::Published
        );

        let collision =
            compile_policy_source(&policy_source_for("organization:a", key, "1", false))
                .expect("compile changed bytes under declared version");
        let error = store
            .ingest_policy_compilation(&collision, "participant:not-the-owner")
            .unwrap_err();
        assert!(error.contains("policy_version_identity_collision"));
        assert!(store
            .get_policy_source(&collision.source.source_id)
            .unwrap()
            .is_none());

        let next = compile_policy_source(&policy_source_for("organization:a", key, "2", false))
            .expect("compile next immutable version");
        store
            .ingest_policy_compilation(&next, "participant:not-the-owner")
            .expect("actor provenance is not lineage ownership");
        assert_ne!(owner_a.artifact.artifact_id, next.artifact.artifact_id);
        assert_eq!(owner_a.artifact.artifact_version, "1");
        fs::remove_dir_all(path).expect("remove policy lineage store");
    }

    #[test]
    fn policy_publication_is_serialized_per_lineage_and_independent_across_lineages() {
        let path = temp_store_path("policy-publication-concurrency");
        let store = Arc::new(LmdbRecordStore::open(&path).expect("open concurrent policy store"));
        let versions = ["1", "2"]
            .into_iter()
            .map(|version| {
                compile_policy_source(&policy_source_for(
                    "organization:concurrent",
                    "concurrent.filesystem",
                    version,
                    version == "1",
                ))
                .expect("compile version")
            })
            .collect::<Vec<_>>();
        for version in &versions {
            store
                .ingest_policy_compilation(version, "participant:publisher")
                .unwrap();
            store
                .validate_policy_artifact(
                    &version.artifact.artifact_id,
                    "participant:publisher",
                    "validated",
                )
                .unwrap();
        }
        let barrier = Arc::new(Barrier::new(3));
        let mut handles = Vec::new();
        for version in &versions {
            let store = Arc::clone(&store);
            let barrier = Arc::clone(&barrier);
            let id = version.artifact.artifact_id.clone();
            handles.push(thread::spawn(move || {
                barrier.wait();
                store.publish_policy_artifact(
                    &id,
                    "participant:publisher",
                    "concurrent publication",
                )
            }));
        }
        barrier.wait();
        for handle in handles {
            handle.join().expect("publisher thread").expect("publish");
        }
        let views = versions
            .iter()
            .map(|version| {
                store
                    .policy_artifact_view(&version.artifact.artifact_id)
                    .unwrap()
                    .unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            views
                .iter()
                .filter(|view| view.lifecycle == PolicyLifecycleState::Published)
                .count(),
            1
        );
        assert_eq!(
            views
                .iter()
                .filter(|view| view.lifecycle == PolicyLifecycleState::Superseded)
                .count(),
            1
        );
        {
            let mut txn = store.env.begin_rw_txn().unwrap();
            txn.clear_db(store.policy_current_by_lineage).unwrap();
            txn.commit().unwrap();
        }
        assert!(store
            .current_published_policy("organization:concurrent", "concurrent.filesystem")
            .unwrap()
            .is_some());
        assert_eq!(store.rebuild_policy_current_index().unwrap(), 1);
        assert!(store
            .current_published_policy("organization:concurrent", "concurrent.filesystem")
            .unwrap()
            .is_some());

        let independent = ["a", "b"]
            .into_iter()
            .map(|owner| {
                compile_policy_source(&policy_source_for(
                    &format!("organization:{owner}"),
                    "independent.filesystem",
                    "1",
                    true,
                ))
                .unwrap()
            })
            .collect::<Vec<_>>();
        for artifact in &independent {
            store
                .ingest_policy_compilation(artifact, "participant:publisher")
                .unwrap();
            store
                .validate_policy_artifact(
                    &artifact.artifact.artifact_id,
                    "participant:publisher",
                    "validated",
                )
                .unwrap();
        }
        let barrier = Arc::new(Barrier::new(3));
        let mut handles = Vec::new();
        for artifact in &independent {
            let store = Arc::clone(&store);
            let barrier = Arc::clone(&barrier);
            let id = artifact.artifact.artifact_id.clone();
            handles.push(thread::spawn(move || {
                barrier.wait();
                store.publish_policy_artifact(
                    &id,
                    "participant:publisher",
                    "independent publication",
                )
            }));
        }
        barrier.wait();
        for handle in handles {
            handle.join().unwrap().unwrap();
        }
        for owner in ["organization:a", "organization:b"] {
            assert!(store
                .current_published_policy(owner, "independent.filesystem")
                .unwrap()
                .is_some());
        }
        drop(store);
        fs::remove_dir_all(path).expect("remove concurrent policy store");
    }

    #[test]
    fn aborted_policy_transactions_leave_no_partial_lifecycle() {
        let path = temp_store_path("policy-atomicity");
        let store = LmdbRecordStore::open(&path).expect("open atomicity store");
        let candidate = compile_policy_source(&policy_source("1", true)).unwrap();
        let source_key = policy_source_key(&candidate.source.source_id);
        let artifact_key = policy_artifact_key(&candidate.artifact.artifact_id);
        let source_bytes = serde_json::to_vec(&candidate.source).unwrap();
        let artifact_bytes = serde_json::to_vec(&candidate.artifact).unwrap();

        {
            let mut txn = store.env.begin_rw_txn().unwrap();
            txn.put(
                store.policy_sources_by_id,
                &source_key,
                &source_bytes,
                WriteFlags::NO_OVERWRITE,
            )
            .unwrap();
        }
        assert!(store
            .get_policy_source(&candidate.source.source_id)
            .unwrap()
            .is_none());
        {
            let mut txn = store.env.begin_rw_txn().unwrap();
            txn.put(
                store.policy_sources_by_id,
                &source_key,
                &source_bytes,
                WriteFlags::NO_OVERWRITE,
            )
            .unwrap();
            txn.put(
                store.policy_artifacts_by_id,
                &artifact_key,
                &artifact_bytes,
                WriteFlags::NO_OVERWRITE,
            )
            .unwrap();
        }
        assert!(store
            .get_policy_artifact(&candidate.artifact.artifact_id)
            .unwrap()
            .is_none());

        store
            .ingest_policy_compilation(&candidate, "participant:publisher")
            .unwrap();
        {
            let mut txn = store.env.begin_rw_txn().unwrap();
            store
                .append_policy_event_txn(
                    &mut txn,
                    &candidate.artifact.artifact_id,
                    PolicyLifecycleAction::Validated,
                    Some(PolicyLifecycleState::Candidate),
                    PolicyLifecycleState::Validated,
                    None,
                    "participant:publisher",
                    "abort validation",
                )
                .unwrap();
        }
        assert_eq!(
            store
                .policy_artifact_view(&candidate.artifact.artifact_id)
                .unwrap()
                .unwrap()
                .lifecycle,
            PolicyLifecycleState::Candidate
        );

        store
            .validate_policy_artifact(
                &candidate.artifact.artifact_id,
                "participant:publisher",
                "validated",
            )
            .unwrap();
        store
            .publish_policy_artifact(
                &candidate.artifact.artifact_id,
                "participant:publisher",
                "published",
            )
            .unwrap();
        let next = compile_policy_source(&policy_source("2", false)).unwrap();
        store
            .ingest_policy_compilation(&next, "participant:publisher")
            .unwrap();
        store
            .validate_policy_artifact(
                &next.artifact.artifact_id,
                "participant:publisher",
                "validated",
            )
            .unwrap();
        for include_new_publish in [false, true] {
            let mut txn = store.env.begin_rw_txn().unwrap();
            store
                .append_policy_event_txn(
                    &mut txn,
                    &candidate.artifact.artifact_id,
                    PolicyLifecycleAction::Superseded,
                    Some(PolicyLifecycleState::Published),
                    PolicyLifecycleState::Superseded,
                    Some(&next.artifact.artifact_id),
                    "participant:publisher",
                    "abort replacement",
                )
                .unwrap();
            if include_new_publish {
                store
                    .append_policy_event_txn(
                        &mut txn,
                        &next.artifact.artifact_id,
                        PolicyLifecycleAction::Published,
                        Some(PolicyLifecycleState::Validated),
                        PolicyLifecycleState::Published,
                        None,
                        "participant:publisher",
                        "abort new publication",
                    )
                    .unwrap();
            }
        }
        assert_eq!(
            store
                .policy_artifact_view(&candidate.artifact.artifact_id)
                .unwrap()
                .unwrap()
                .lifecycle,
            PolicyLifecycleState::Published
        );
        assert_eq!(
            store
                .policy_artifact_view(&next.artifact.artifact_id)
                .unwrap()
                .unwrap()
                .lifecycle,
            PolicyLifecycleState::Validated
        );
        {
            let mut txn = store.env.begin_rw_txn().unwrap();
            store
                .append_policy_event_txn(
                    &mut txn,
                    &candidate.artifact.artifact_id,
                    PolicyLifecycleAction::Retired,
                    Some(PolicyLifecycleState::Published),
                    PolicyLifecycleState::Retired,
                    None,
                    "participant:publisher",
                    "abort retirement",
                )
                .unwrap();
        }
        assert_eq!(
            store
                .policy_artifact_view(&candidate.artifact.artifact_id)
                .unwrap()
                .unwrap()
                .lifecycle,
            PolicyLifecycleState::Published
        );
        fs::remove_dir_all(path).expect("remove atomicity store");
    }

    #[test]
    fn persisted_policy_corruption_and_future_schemas_fail_closed() {
        let source_path = temp_store_path("policy-corrupt-source");
        let source_store = LmdbRecordStore::open(&source_path).unwrap();
        {
            let mut txn = source_store.env.begin_rw_txn().unwrap();
            txn.put(
                source_store.policy_sources_by_id,
                &policy_source_key("policy-source:broken"),
                &b"{".as_slice(),
                WriteFlags::empty(),
            )
            .unwrap();
            txn.commit().unwrap();
        }
        assert!(source_store
            .get_policy_source("policy-source:broken")
            .unwrap_err()
            .contains("decode_failed"));
        let valid = compile_policy_source(&policy_source("1", true)).unwrap();
        let mut future_source = valid.source.clone();
        future_source.schema = "yai.policy_source_artifact.v99".to_string();
        {
            let mut txn = source_store.env.begin_rw_txn().unwrap();
            txn.put(
                source_store.policy_sources_by_id,
                &policy_source_key(&future_source.source_id),
                &serde_json::to_vec(&future_source).unwrap(),
                WriteFlags::empty(),
            )
            .unwrap();
            txn.commit().unwrap();
        }
        assert!(source_store
            .get_policy_source(&future_source.source_id)
            .unwrap_err()
            .contains("unsupported_policy_source_artifact_schema"));
        drop(source_store);
        fs::remove_dir_all(source_path).unwrap();

        let artifact_path = temp_store_path("policy-corrupt-artifact");
        let artifact_store = LmdbRecordStore::open(&artifact_path).unwrap();
        let compilation = compile_policy_source(&policy_source("1", true)).unwrap();
        let mut future_artifact = compilation.artifact.clone();
        future_artifact.schema = "yai.policy_artifact.v99".to_string();
        {
            let mut txn = artifact_store.env.begin_rw_txn().unwrap();
            txn.put(
                artifact_store.policy_artifacts_by_id,
                &policy_artifact_key(&future_artifact.artifact_id),
                &serde_json::to_vec(&future_artifact).unwrap(),
                WriteFlags::empty(),
            )
            .unwrap();
            txn.commit().unwrap();
        }
        assert!(artifact_store
            .get_policy_artifact(&future_artifact.artifact_id)
            .unwrap_err()
            .contains("unsupported_policy_artifact_schema"));
        let mut corrupted = compilation.artifact.clone();
        corrupted.policy_ir.ir_digest = format!("sha256:{}", "0".repeat(64));
        let corrupted_bytes = serde_json::to_vec(&corrupted).unwrap();
        {
            let mut txn = artifact_store.env.begin_rw_txn().unwrap();
            txn.put(
                artifact_store.policy_artifacts_by_id,
                &policy_artifact_key(&corrupted.artifact_id),
                &corrupted_bytes,
                WriteFlags::empty(),
            )
            .unwrap();
            txn.commit().unwrap();
        }
        assert_eq!(
            artifact_store
                .get_policy_artifact(&corrupted.artifact_id)
                .unwrap_err(),
            "policy_ir_not_reproducible_from_parsed_facts"
        );
        drop(artifact_store);
        fs::remove_dir_all(artifact_path).unwrap();

        let event_path = temp_store_path("policy-corrupt-event");
        let event_store = LmdbRecordStore::open(&event_path).unwrap();
        event_store
            .ingest_policy_compilation(&compilation, "participant:publisher")
            .unwrap();
        let event = event_store
            .list_policy_lifecycle_events(None)
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        let mut future = event.clone();
        future.schema = "yai.policy_lifecycle_event.v99".to_string();
        {
            let mut txn = event_store.env.begin_rw_txn().unwrap();
            txn.put(
                event_store.policy_lifecycle_events_by_id,
                &policy_event_key(&event.event_id),
                &serde_json::to_vec(&future).unwrap(),
                WriteFlags::empty(),
            )
            .unwrap();
            txn.commit().unwrap();
        }
        assert!(event_store
            .list_policy_lifecycle_events(None)
            .unwrap_err()
            .contains("unsupported_policy_lifecycle_event_schema"));
        drop(event_store);
        fs::remove_dir_all(event_path).unwrap();

        let dangling_path = temp_store_path("policy-dangling-event");
        let dangling_store = LmdbRecordStore::open(&dangling_path).unwrap();
        {
            let mut txn = dangling_store.env.begin_rw_txn().unwrap();
            txn.put(
                dangling_store.policy_lifecycle_sequence,
                &policy_sequence_key(1),
                &"policy-event:missing",
                WriteFlags::empty(),
            )
            .unwrap();
            txn.commit().unwrap();
        }
        assert!(dangling_store
            .list_policy_lifecycle_events(None)
            .unwrap_err()
            .contains("sequence_dangling"));
        drop(dangling_store);
        fs::remove_dir_all(dangling_path).unwrap();
    }

    #[test]
    fn policy_catalog_capacity_contract_is_explicit_and_bounded() {
        let path = temp_store_path("policy-capacity-supported");
        let store = LmdbRecordStore::open(&path).expect("open default-size catalog");
        let mut retained_source_bytes = 0usize;
        for index in 0..SUPPORTED_POLICY_CATALOG_SOURCES {
            let source = large_policy_source(index);
            assert!(source.len() <= crate::governance::MAX_POLICY_SOURCE_BYTES);
            retained_source_bytes += source.len();
            let compilation = compile_policy_source(&source).expect("compile bounded source");
            store
                .ingest_policy_compilation(&compilation, "participant:capacity-test")
                .expect("supported catalog does not fill default map");
        }
        assert_eq!(
            store.list_policy_artifact_views().unwrap().len(),
            SUPPORTED_POLICY_CATALOG_SOURCES
        );
        let supported_db_bytes = fs::metadata(path.join("data.mdb")).unwrap().len();
        drop(store);
        fs::remove_dir_all(path).unwrap();

        let constrained_path = temp_store_path("policy-capacity-exhausted");
        let constrained =
            LmdbRecordStore::open_with_map_size(&constrained_path, MINIMUM_LMDB_MAP_SIZE)
                .expect("open minimum-size catalog");
        let mut capacity_error = None;
        let mut constrained_artifacts = 0usize;
        for index in 0..SUPPORTED_POLICY_CATALOG_SOURCES {
            let compilation = compile_policy_source(&large_policy_source(index)).unwrap();
            if let Err(error) =
                constrained.ingest_policy_compilation(&compilation, "participant:capacity-test")
            {
                capacity_error = Some(error);
                break;
            }
            constrained_artifacts += 1;
        }
        assert!(capacity_error
            .expect("minimum map eventually reaches explicit capacity")
            .contains("policy_catalog_capacity_exhausted"));
        let constrained_db_bytes = fs::metadata(constrained_path.join("data.mdb"))
            .unwrap()
            .len();
        eprintln!(
            "policy_catalog_capacity: default_map_bytes={DEFAULT_LMDB_MAP_SIZE} sources={} retained_source_bytes={retained_source_bytes} db_bytes={supported_db_bytes} minimum_map_bytes={MINIMUM_LMDB_MAP_SIZE} constrained_artifacts={constrained_artifacts} constrained_db_bytes={constrained_db_bytes}"
            , SUPPORTED_POLICY_CATALOG_SOURCES
        );
        drop(constrained);
        fs::remove_dir_all(constrained_path).unwrap();
    }

    #[test]
    fn policy_catalog_scale_characterization_is_bounded() {
        let path = temp_store_path("policy-catalog-scale");
        let store = LmdbRecordStore::open(&path).unwrap();
        let mut artifacts = Vec::new();
        let mut total_source_bytes = 0usize;
        for lineage in 0..32 {
            let source = policy_source_for(
                "organization:scale",
                &format!("scale.lineage.{lineage}"),
                "1",
                lineage % 2 == 0,
            );
            total_source_bytes += source.len();
            artifacts.push(compile_policy_source(&source).unwrap());
        }
        for version in 1..=8 {
            let source = policy_source_for(
                "organization:scale",
                "scale.versioned",
                &version.to_string(),
                version % 2 == 0,
            );
            total_source_bytes += source.len();
            artifacts.push(compile_policy_source(&source).unwrap());
        }

        let ingest_started = Instant::now();
        for artifact in &artifacts {
            store
                .ingest_policy_compilation(artifact, "participant:scale")
                .unwrap();
            store
                .validate_policy_artifact(
                    &artifact.artifact.artifact_id,
                    "participant:scale",
                    "scale validation",
                )
                .unwrap();
            store
                .publish_policy_artifact(
                    &artifact.artifact.artifact_id,
                    "participant:scale",
                    "scale publication",
                )
                .unwrap();
        }
        let ingest_ms = ingest_started.elapsed().as_millis();
        let list_started = Instant::now();
        let views = store.list_policy_artifact_views().unwrap();
        let list_ms = list_started.elapsed().as_millis();
        let inspect_started = Instant::now();
        let inspected = store
            .policy_artifact_view(&artifacts[20].artifact.artifact_id)
            .unwrap()
            .unwrap();
        let inspect_ms = inspect_started.elapsed().as_millis();
        assert_eq!(views.len(), 40);
        assert_eq!(store.list_policy_lifecycle_events(None).unwrap().len(), 127);
        assert_eq!(
            views
                .iter()
                .filter(|view| view.lifecycle == PolicyLifecycleState::Published)
                .count(),
            33
        );
        assert_eq!(inspected.artifact, artifacts[20].artifact);
        let db_bytes = fs::metadata(path.join("data.mdb")).unwrap().len();
        eprintln!(
            "policy_catalog_scale: lineages=33 artifacts=40 events=127 source_bytes={total_source_bytes} db_bytes={db_bytes} ingest_publish_ms={ingest_ms} list_ms={list_ms} inspect_ms={inspect_ms}"
        );
        drop(store);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn blocked_policy_and_source_payload_loss_fail_closed_without_erasing_provenance() {
        let path = temp_store_path("policy-blocked");
        let store = LmdbRecordStore::open(&path).expect("open blocked policy store");
        let blocked_bytes = serde_json::to_vec(&serde_json::json!({
            "schema": POLICY_SOURCE_INPUT_SCHEMA,
            "policy_key": "organization.example.unresolved",
            "source_version": "1",
            "owner_ref": "organization:example",
            "source_origin": {"source_system":"unit-test","source_uri":"test://governance/blocked"},
            "validity": {"mode":"unbounded"},
            "rules": [{"kind":"unsupported_future_rule","payload":"opaque"}]
        }))
        .unwrap();
        let blocked = compile_policy_source(&blocked_bytes).expect("compile unresolved source");
        assert_eq!(
            blocked.artifact.validation.status,
            PolicyValidationStatus::Blocked
        );
        store
            .ingest_policy_compilation(&blocked, "participant:policy-admin")
            .expect("retain blocked candidate");
        assert!(store
            .validate_policy_artifact(
                &blocked.artifact.artifact_id,
                "participant:policy-admin",
                "attempt validation"
            )
            .unwrap_err()
            .contains("qualification_blocked"));
        assert!(store
            .publish_policy_artifact(
                &blocked.artifact.artifact_id,
                "participant:policy-admin",
                "attempt publication"
            )
            .unwrap_err()
            .contains("cannot_publish"));
        let event_count = store
            .list_policy_lifecycle_events(Some(&blocked.artifact.artifact_id))
            .unwrap()
            .len();
        assert_eq!(event_count, 1);

        store
            .discard_policy_source_for_test(&blocked.source.source_id)
            .expect("simulate source payload loss");
        assert!(store
            .get_policy_source(&blocked.source.source_id)
            .unwrap()
            .is_none());
        let retained = store
            .get_policy_artifact(&blocked.artifact.artifact_id)
            .unwrap()
            .expect("artifact survives source payload loss");
        assert_eq!(retained.source_id, blocked.source.source_id);
        assert_eq!(retained.source_digest, blocked.source.content_digest);
        assert_eq!(retained.parsed, blocked.artifact.parsed);
        assert_eq!(retained.policy_ir, blocked.artifact.policy_ir);
        assert!(
            !store
                .policy_artifact_view(&blocked.artifact.artifact_id)
                .unwrap()
                .unwrap()
                .runtime_consumable
        );
        fs::remove_dir_all(path).expect("remove blocked policy store");
    }

    #[test]
    fn canonical_commit_rejects_duplicate_stale_and_partial_writes() {
        let path = temp_store_path("canonical-failures");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        store
            .commit_transition(pending(
                "transition:open",
                "case:failures",
                0,
                TransitionPayload::CaseOpened {
                    lifecycle: CaseLifecycle::Open,
                },
            ))
            .expect("open Case");

        assert!(store
            .commit_transition(pending(
                "transition:open",
                "case:failures",
                1,
                TransitionPayload::ParticipantBound {
                    participant_id: "participant:duplicate".to_string(),
                    role: "test".to_string(),
                },
            ))
            .unwrap_err()
            .contains("duplicate_transition_id"));
        assert!(store
            .commit_transition(pending(
                "transition:stale",
                "case:failures",
                0,
                TransitionPayload::ParticipantBound {
                    participant_id: "participant:stale".to_string(),
                    role: "test".to_string(),
                },
            ))
            .unwrap_err()
            .contains("stale_case_generation"));

        let injected = pending(
            "transition:rollback",
            "case:failures",
            1,
            TransitionPayload::ParticipantBound {
                participant_id: "participant:rollback".to_string(),
                role: "test".to_string(),
            },
        );
        assert!(store
            .commit_transition_inner(injected, true)
            .unwrap_err()
            .contains("injected_failure"));
        assert_eq!(
            store
                .get_transition_by_id("transition:rollback")
                .expect("transition lookup"),
            None
        );
        assert_eq!(
            store
                .get_case_state("case:failures")
                .expect("state lookup")
                .expect("state present")
                .generation,
            1
        );
        drop(store);
        fs::remove_dir_all(path).expect("remove LMDB test store");
    }

    #[test]
    fn controlled_effect_is_atomic_replayable_and_reconciles_visible_crash() {
        let path = temp_store_path("controlled-effect");
        let resource_root = temp_store_path("controlled-effect-resource");
        fs::create_dir_all(resource_root.join("allowed")).expect("create resource fixture");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        let case_id = "case:controlled";
        let model = "participant:model";
        let operator = "participant:operator";
        commit_typed(
            &store,
            "transition:open",
            case_id,
            0,
            TransitionPayload::CaseOpened {
                lifecycle: CaseLifecycle::Open,
            },
            None,
            vec![],
        );
        commit_typed(
            &store,
            "transition:model",
            case_id,
            1,
            TransitionPayload::ParticipantBound {
                participant_id: model.to_string(),
                role: "model_provider".to_string(),
            },
            None,
            vec![],
        );
        commit_typed(
            &store,
            "transition:operator",
            case_id,
            2,
            TransitionPayload::ParticipantBound {
                participant_id: operator.to_string(),
                role: "policy_owner".to_string(),
            },
            None,
            vec![],
        );
        commit_typed(
            &store,
            "transition:provider",
            case_id,
            3,
            TransitionPayload::ProviderAttached {
                participant_id: model.to_string(),
                provider_id: "provider:test".to_string(),
                provider_kind: "openai_compatible".to_string(),
                base_url: "http://127.0.0.1:1".to_string(),
                model_id: "model:test".to_string(),
                credential_ref: "env:TEST".to_string(),
            },
            None,
            vec![],
        );
        commit_typed(
            &store,
            "transition:invocation",
            case_id,
            4,
            TransitionPayload::ProviderInvocationStarted {
                invocation_id: "invocation:1".to_string(),
                participant_id: model.to_string(),
                provider_id: "provider:test".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:test".to_string(),
                semantic_lineage: Some(test_provider_lineage(4)),
            },
            None,
            vec![],
        );
        commit_typed(
            &store,
            "transition:result",
            case_id,
            5,
            TransitionPayload::ProviderResultRecorded {
                result_id: "provider-result:1".to_string(),
                invocation_id: "invocation:1".to_string(),
                provider_id: "provider:test".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:test".to_string(),
                semantic_lineage: Some(test_provider_lineage(4)),
                output: "candidate".to_string(),
            },
            None,
            vec!["invocation:1".to_string()],
        );
        let resource = ResourceAttachmentState {
            attachment_id: "workspace".to_string(),
            kind: ResourceKind::Filesystem,
            allowed_write_prefix: "allowed".to_string(),
            max_write_bytes: 1024,
            policy_id: "policy:workspace".to_string(),
            policy_owner_participant_id: operator.to_string(),
            review_requirement: crate::transition::ReviewRequirement::Automatic,
            process_signal_actions: Vec::new(),
        };
        commit_typed(
            &store,
            "transition:resource",
            case_id,
            6,
            TransitionPayload::ResourceAttached {
                attachment: resource.clone(),
            },
            None,
            vec![operator.to_string()],
        );
        let binding = LocalFilesystemBinding::new(case_id, "workspace", &resource_root)
            .expect("create local binding");
        store
            .put_local_filesystem_binding(&binding)
            .expect("persist local binding");

        let commit_effect =
            |store: &LmdbRecordStore,
             invocation: &str,
             result_id: &str,
             path: &str,
             content: &str,
             first_generation: u64,
             suffix: &str,
             crash_after_visible: bool|
             -> (crate::effect::PreparedEffect, crate::effect::CarrierResult) {
                let raw = format!(
                "{{\"schema\":\"yai.operation_proposal.filesystem_write.v1\",\"operation\":\"filesystem.write\",\"resource\":\"workspace\",\"path\":\"{path}\",\"content\":\"{content}\"}}"
            );
                let operation = normalize_filesystem_write_candidate(
                    &raw,
                    &NormalizationContext {
                        case_id,
                        participant_id: model,
                        provider_result_id: result_id,
                        provider_invocation_id: invocation,
                        case_generation: first_generation,
                        resource: &resource,
                    },
                )
                .expect("normalize operation");
                let operation_commit = commit_typed(
                    store,
                    &format!("transition:operation:{suffix}"),
                    case_id,
                    first_generation,
                    TransitionPayload::OperationRecorded {
                        operation: operation.clone(),
                    },
                    Some(operation.scope.clone()),
                    vec![result_id.to_string(), invocation.to_string()],
                );
                let decision = decide_filesystem_write(
                    &operation,
                    &resource,
                    operation_commit.state.generation,
                );
                assert_eq!(decision.outcome, crate::effect::DecisionOutcome::Allow);
                let decision_commit = commit_typed(
                    store,
                    &format!("transition:decision:{suffix}"),
                    case_id,
                    operation_commit.state.generation,
                    TransitionPayload::DecisionRecorded {
                        decision: decision.clone(),
                    },
                    None,
                    vec![operation.operation_id.clone()],
                );
                let grant =
                    issue_execution_grant(&operation, &decision, decision_commit.state.generation)
                        .expect("issue execution grant");
                let grant_commit = commit_typed(
                    store,
                    &format!("transition:grant:{suffix}"),
                    case_id,
                    decision_commit.state.generation,
                    TransitionPayload::ExecutionGrantIssued {
                        grant: grant.clone(),
                    },
                    None,
                    vec![operation.operation_id.clone(), decision.decision_id.clone()],
                );
                let pre = observe_filesystem(
                    &binding,
                    &resource,
                    path,
                    format!("observation:pre:{suffix}"),
                );
                let prepared = prepare_effect(&operation, &decision, &grant, pre)
                    .expect("prepare controlled effect");
                let prepared_commit = commit_typed(
                    store,
                    &format!("transition:prepare:{suffix}"),
                    case_id,
                    grant_commit.state.generation,
                    TransitionPayload::EffectPrepared {
                        prepared: prepared.clone(),
                    },
                    None,
                    vec![
                        operation.operation_id.clone(),
                        decision.decision_id.clone(),
                        grant.grant_id.clone(),
                        prepared.expected_pre_observation.observation_id.clone(),
                    ],
                );
                let result = execute_filesystem_write(
                    &operation,
                    &decision,
                    &grant,
                    &prepared,
                    &prepared_commit.state,
                    &binding,
                    &resource,
                    if crash_after_visible {
                        CarrierFailpoint::CrashAfterVisibleEffect
                    } else {
                        CarrierFailpoint::None
                    },
                )
                .expect("execute grant-validated carrier");
                (prepared, result)
            };

        let (prepared, result) = commit_effect(
            &store,
            "invocation:1",
            "provider-result:1",
            "allowed/one.txt",
            "one",
            7,
            "one",
            false,
        );
        assert_eq!(result.outcome, EffectOutcome::Applied);
        let receipt = build_effect_receipt(&prepared, &result);
        let finalized = commit_typed(
            &store,
            "transition:finalize:one",
            case_id,
            11,
            TransitionPayload::EffectFinalized {
                effect_id: prepared.effect_id.clone(),
                post_observation: result.post_observation.clone(),
                receipt: receipt.clone(),
            },
            None,
            vec![prepared.effect_id.clone(), receipt.receipt_id.clone()],
        );
        assert_eq!(
            finalized.state.effects[0].outcome,
            Some(EffectOutcome::Applied)
        );
        validate_finalized_effect_chain(
            &store.list_case_transitions(case_id).expect("list chain"),
            &prepared.effect_id,
        )
        .expect("closed effect chain");
        assert_eq!(
            fs::read_to_string(resource_root.join("allowed/one.txt")).unwrap(),
            "one"
        );

        commit_typed(
            &store,
            "transition:invocation:two",
            case_id,
            12,
            TransitionPayload::ProviderInvocationStarted {
                invocation_id: "invocation:2".to_string(),
                participant_id: model.to_string(),
                provider_id: "provider:test".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:test".to_string(),
                semantic_lineage: Some(test_provider_lineage(12)),
            },
            None,
            vec![],
        );
        commit_typed(
            &store,
            "transition:result:two",
            case_id,
            13,
            TransitionPayload::ProviderResultRecorded {
                result_id: "provider-result:2".to_string(),
                invocation_id: "invocation:2".to_string(),
                provider_id: "provider:test".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:test".to_string(),
                semantic_lineage: Some(test_provider_lineage(12)),
                output: "candidate two".to_string(),
            },
            None,
            vec!["invocation:2".to_string()],
        );
        let (crashed, crash_result) = commit_effect(
            &store,
            "invocation:2",
            "provider-result:2",
            "allowed/two.txt",
            "two",
            14,
            "two",
            true,
        );
        assert!(crash_result.crash_injected_after_effect);
        assert_eq!(
            store
                .get_case_state(case_id)
                .expect("state")
                .expect("present")
                .effects[1]
                .status,
            crate::transition::EffectLifecycle::Prepared
        );
        drop(store);

        let reopened = LmdbRecordStore::open(&path).expect("restart after visible effect");
        let state = reopened.get_case_state(case_id).unwrap().unwrap();
        let binding = reopened
            .get_local_filesystem_binding(case_id, "workspace")
            .unwrap()
            .expect("binding survives restart");
        let observed = observe_filesystem(
            &binding,
            &resource,
            "allowed/two.txt",
            "observation:reconcile:two",
        );
        assert_eq!(
            classify_reconciliation(&crashed, &observed),
            ReconciliationConclusion::EffectObserved
        );
        let reconciliation_result = CarrierResult {
            outcome: EffectOutcome::AlreadyApplied,
            post_observation: observed.clone(),
            carrier_attempted: false,
            mutation_performed: false,
            crash_injected_after_effect: false,
            detail: "restart observed intended post-state".to_string(),
        };
        let reconciled_receipt = build_effect_receipt(&crashed, &reconciliation_result);
        commit_typed(
            &reopened,
            "transition:reconcile:two",
            case_id,
            state.generation,
            TransitionPayload::EffectReconciled {
                effect_id: crashed.effect_id.clone(),
                conclusion: ReconciliationConclusion::EffectObserved,
                observation: observed,
                receipt: Some(reconciled_receipt),
            },
            None,
            vec![crashed.effect_id.clone()],
        );
        validate_finalized_effect_chain(
            &reopened.list_case_transitions(case_id).expect("list chain"),
            &crashed.effect_id,
        )
        .expect("reconciled chain closed");
        assert!(reopened
            .verify_case_state(case_id)
            .expect("replay equivalence"));
        let expected = reopened.get_case_state(case_id).unwrap().unwrap();
        reopened.discard_case_state_for_test(case_id).unwrap();
        assert_eq!(reopened.rebuild_case_state(case_id).unwrap(), expected);
        let graph = reopened
            .materialize_graph_relations_for_case(case_id)
            .expect("derived graph from typed transitions");
        assert!(graph.relations_written > 0);
        let history = reopened
            .list_case_transitions(case_id)
            .expect("canonical history for memory");
        let transition_count = history.len();
        let memory = derive_operational_memory(case_id, &history).expect("derive memory");
        assert!(memory.entries.iter().any(|entry| {
            entry.semantic_kind == OperationalMemoryKind::ResourceEffect
                && !entry.provenance.effect_receipt_ids.is_empty()
        }));
        assert!(memory.entries.iter().any(|entry| {
            matches!(
                &entry.value,
                crate::memory::OperationalMemoryValue::ResourceEffect { effect_id, .. }
                    if effect_id == &crashed.effect_id
            ) && entry.lifecycle == crate::memory::OperationalMemoryLifecycle::Active
        }));
        assert!(!memory.entries.iter().any(|entry| {
            matches!(
                &entry.value,
                crate::memory::OperationalMemoryValue::UnresolvedEffect { effect_id, .. }
                    if effect_id == &crashed.effect_id
            ) && entry.lifecycle == crate::memory::OperationalMemoryLifecycle::Active
        }));
        let mut invalid_memory = memory.clone();
        invalid_memory.manifest.schema = "yai.operational_memory_manifest.future".to_string();
        assert!(reopened
            .replace_case_operational_memory(&invalid_memory)
            .expect_err("invalid derived materialization must fail")
            .contains("unsupported_operational_memory_manifest_schema"));
        assert_eq!(
            reopened
                .list_case_transitions(case_id)
                .expect("canonical history survives derived failure")
                .len(),
            transition_count
        );
        reopened
            .replace_case_operational_memory(&memory)
            .expect("store derived memory");
        reopened
            .replace_case_operational_memory(&memory)
            .expect("idempotent replace");
        assert_eq!(
            reopened
                .list_operational_memory(case_id)
                .expect("list stored memory"),
            memory.entries
        );
        reopened
            .clear_all_operational_memory()
            .expect("drop derived store");
        assert!(reopened
            .list_operational_memory(case_id)
            .expect("memory absent")
            .is_empty());
        assert_eq!(
            reopened
                .list_case_transitions(case_id)
                .expect("canonical history survives")
                .len(),
            transition_count
        );
        assert_eq!(
            reopened
                .get_case_state(case_id)
                .expect("CaseState survives"),
            Some(expected.clone())
        );
        let rebuilt = derive_operational_memory(
            case_id,
            &reopened
                .list_case_transitions(case_id)
                .expect("history rebuild input"),
        )
        .expect("rebuild memory");
        assert_eq!(rebuilt, memory);
        reopened
            .replace_case_operational_memory(&rebuilt)
            .expect("persist rebuilt memory");
        drop(reopened);
        fs::remove_dir_all(path).expect("remove LMDB test store");
        fs::remove_dir_all(resource_root).expect("remove resource fixture");
    }

    #[test]
    fn legacy_compatibility_import_is_isolated_from_canonical_authority() {
        let path = temp_store_path("legacy-compatibility-import");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        let contents = concat!(
            "{\"schema\":\"yai.store.record.v0\",\"record_id\":\"record:one\",\"case_ref\":\"case:legacy\",\"record_kind\":\"case\",\"summary\":\"\"}\n",
            "{\"schema\":\"yai.store.record.v0\",\"record_id\":\"record:two\",\"case_ref\":\"case:legacy\",\"record_kind\":\"future_kind\"}\n",
            "{bad\n"
        );
        let first = store
            .import_legacy_compatibility(contents, "fixture:legacy")
            .expect("import compatibility corpus");
        assert_eq!(first.lines_total, 3);
        assert_eq!(first.losslessly_promoted, 1);
        assert_eq!(first.preserved_opaque, 1);
        assert_eq!(first.rejected_malformed, 1);
        assert_eq!(first.payloads_written, 2);
        assert_eq!(store.legacy_compatibility_payload_count().unwrap(), 2);
        assert!(store
            .list_case_transitions("case:legacy")
            .unwrap()
            .is_empty());
        assert_eq!(store.get_case_state("case:legacy").unwrap(), None);

        let second = store
            .import_legacy_compatibility(contents, "fixture:legacy")
            .expect("repeat compatibility import");
        assert_eq!(second.payloads_written, 0);
        assert_eq!(second.payloads_duplicate, 2);
        drop(store);
        fs::remove_dir_all(path).expect("remove LMDB test store");
    }

    #[test]
    fn persisted_future_canonical_schema_is_not_overwritten() {
        let path = temp_store_path("future-canonical-schema");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        let mut txn = store
            .env
            .begin_rw_txn()
            .expect("schema mutation transaction");
        txn.put(
            store.schema_meta,
            &"meta:canonical_transition_schema",
            &"yai.transition.v99",
            WriteFlags::empty(),
        )
        .expect("write future schema marker");
        txn.commit().expect("commit future schema marker");
        drop(store);

        let error = match LmdbRecordStore::open(&path) {
            Ok(_) => panic!("future persisted schema must not be overwritten"),
            Err(error) => error,
        };
        assert!(error.contains("unsupported_persisted_schema"));
        fs::remove_dir_all(path).expect("remove LMDB test store");
    }

    #[test]
    fn h13_wave13_runtime_instance_schema_marker_upgrades_without_rejecting_store() {
        let path = temp_store_path("runtime-instance-v1-schema-marker");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        let mut txn = store
            .env
            .begin_rw_txn()
            .expect("schema mutation transaction");
        txn.put(
            store.schema_meta,
            &"meta:runtime_instance_schema",
            &RUNTIME_INSTANCE_SCHEMA_V1,
            WriteFlags::empty(),
        )
        .expect("write Wave13 schema marker");
        txn.commit().expect("commit Wave13 schema marker");
        drop(store);

        let reopened =
            LmdbRecordStore::open(&path).expect("Wave13 runtime instance schema remains readable");
        let txn = reopened
            .env
            .begin_ro_txn()
            .expect("schema read transaction");
        assert_eq!(
            txn.get(reopened.schema_meta, &"meta:runtime_instance_schema")
                .expect("upgraded runtime instance schema marker"),
            RUNTIME_INSTANCE_SCHEMA.as_bytes()
        );
        drop(txn);
        drop(reopened);
        fs::remove_dir_all(path).expect("remove LMDB test store");
    }

    #[test]
    fn provider_payloads_reduce_without_summary_semantics() {
        let path = temp_store_path("typed-reducers");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        let case_id = "case:typed";
        let payloads = vec![
            TransitionPayload::CaseOpened {
                lifecycle: CaseLifecycle::Open,
            },
            TransitionPayload::ParticipantBound {
                participant_id: "participant:model".to_string(),
                role: "model_participant".to_string(),
            },
            TransitionPayload::ProviderAttached {
                participant_id: "participant:model".to_string(),
                provider_id: "provider:test".to_string(),
                provider_kind: "openai_compatible".to_string(),
                base_url: "http://127.0.0.1:1".to_string(),
                model_id: "model:test".to_string(),
                credential_ref: "env:TEST_KEY".to_string(),
            },
            TransitionPayload::ProviderInvocationStarted {
                invocation_id: "invocation:1".to_string(),
                participant_id: "participant:model".to_string(),
                provider_id: "provider:test".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:test".to_string(),
                semantic_lineage: Some(test_provider_lineage(3)),
            },
        ];
        for (index, payload) in payloads.into_iter().enumerate() {
            store
                .commit_transition(pending(
                    &format!("transition:{}", index + 1),
                    case_id,
                    index as u64,
                    payload,
                ))
                .expect("commit typed setup");
        }
        let mut result = pending(
            "transition:result",
            case_id,
            4,
            TransitionPayload::ProviderResultRecorded {
                result_id: "result:1".to_string(),
                invocation_id: "invocation:1".to_string(),
                provider_id: "provider:test".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:test".to_string(),
                semantic_lineage: Some(test_provider_lineage(3)),
                output: "typed output".to_string(),
            },
        );
        result.causal_refs = vec!["invocation:1".to_string()];
        result.summary = Some("invocation:wrong output_chars:999".to_string());
        store
            .commit_transition(result)
            .expect("commit typed result");

        let mut interpretation = pending(
            "transition:interpretation",
            case_id,
            5,
            TransitionPayload::ModelInterpretationRecorded {
                interpretation_id: "interpretation:1".to_string(),
                result_id: "result:1".to_string(),
                authority: InterpretationAuthority::NonAuthoritative,
            },
        );
        interpretation.causal_refs = vec!["result:1".to_string()];
        store
            .commit_transition(interpretation)
            .expect("commit interpretation");

        let committed = store
            .get_case_state(case_id)
            .expect("read typed state")
            .expect("typed state exists");
        assert_eq!(committed.generation, 6);
        assert_eq!(committed.last_provider_result.unwrap().output_chars, 12);
        assert!(store.verify_case_state(case_id).expect("verify replay"));
        assert!(store
            .materialize_graph_relations_for_case_inner(case_id, true)
            .unwrap_err()
            .contains("injected_graph_materialization_failure"));
        assert!(store
            .verify_case_state(case_id)
            .expect("derived failure cannot affect canonical state"));
        let graph_report = store
            .materialize_graph_relations_for_case(case_id)
            .expect("materialize typed graph");
        assert!(graph_report.relations_written > 0);
        let relations = store
            .list_graph_relations_by_case(case_id, usize::MAX)
            .expect("typed graph relations");
        assert!(relations.relations.iter().any(|relation| {
            relation.edge_kind == "provider_result_closes_invocation"
                && relation.provenance == "canonical_transition"
        }));
        let expected_relation_count = relations.relations_total;
        let rebuild = store
            .rebuild_graph_relations_for_case(case_id)
            .expect("rebuild typed graph from source authority");
        assert_eq!(rebuild.relations_written, expected_relation_count);
        assert_eq!(rebuild.relations_duplicate, 0);
        assert_eq!(
            store
                .list_graph_relations_by_case(case_id, usize::MAX)
                .expect("rebuilt relations")
                .relations_total,
            expected_relation_count
        );
        drop(store);
        fs::remove_dir_all(path).expect("remove LMDB test store");
    }

    #[test]
    fn freeze_supports_control_carrier_and_divergence_records() {
        let path = temp_store_path("record-freeze");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        let records = [
            Record::from_parts(
                "rec:freeze-attempt",
                "case:spine34-freeze",
                RecordKind::Attempt,
                "subject:filesystem-sandbox",
                "op:freeze-write",
                "",
                "",
                "attempt:file.write",
            ),
            Record::from_parts(
                "rec:freeze-decision",
                "case:spine34-freeze",
                RecordKind::Decision,
                "subject:filesystem-sandbox",
                "op:freeze-write",
                "decision:freeze-deny",
                "",
                "decision:deny",
            ),
            Record::from_parts(
                "rec:freeze-carrier-request",
                "case:spine34-freeze",
                RecordKind::CarrierRequest,
                "subject:filesystem-sandbox",
                "op:freeze-write",
                "decision:freeze-deny",
                "",
                "carrier:filesystem requested_outcome:blocked",
            ),
            Record::from_parts(
                "rec:freeze-effect-receipt",
                "case:spine34-freeze",
                RecordKind::EffectReceipt,
                "subject:filesystem-sandbox",
                "op:freeze-write",
                "decision:freeze-deny",
                "receipt:freeze-blocked",
                "receipt:blocked",
            ),
            Record::from_parts(
                "rec:freeze-divergence",
                "case:spine34-freeze",
                RecordKind::Divergence,
                "subject:filesystem-sandbox",
                "op:freeze-write",
                "decision:freeze-deny",
                "receipt:freeze-blocked",
                "divergence:none result:consistent",
            ),
        ];

        for record in &records {
            store
                .append_record(record, "spine34-freeze-test")
                .expect("append freeze record");
        }

        let summary = store.summary().expect("summary");
        assert_eq!(summary.records_total, 5);
        assert_eq!(summary.records_by_case, 5);
        assert_eq!(summary.records_by_kind, 5);
        assert_eq!(summary.records_by_subject, 5);
        assert_eq!(summary.records_by_receipt, 2);

        let divergence = store
            .list_records_by_kind("divergence", 10)
            .expect("list divergence records");
        assert_eq!(divergence.records_total, 1);
        assert_eq!(
            divergence
                .records
                .first()
                .map(|record| record.record_id.as_str()),
            Some("rec:freeze-divergence")
        );

        let receipt_records = store
            .list_records_by_receipt("receipt:freeze-blocked", 10)
            .expect("list receipt records");
        assert_eq!(receipt_records.records_total, 2);
        assert!(receipt_records
            .records
            .iter()
            .any(|record| record.record_kind == "effect_receipt"));
        assert!(receipt_records
            .records
            .iter()
            .any(|record| record.record_kind == "divergence"));

        let carrier_request = store
            .get_record_by_id("rec:freeze-carrier-request")
            .expect("get carrier request")
            .expect("carrier request present");
        assert_eq!(carrier_request.schema, RECORD_SCHEMA);
        assert_eq!(carrier_request.record_kind, "carrier_request");
        assert!(carrier_request
            .raw_json
            .contains("\"schema\":\"yai.record.v1\""));
        assert!(carrier_request
            .raw_json
            .contains("\"source\":{\"plane\":\"journal\""));

        drop(store);
        fs::remove_dir_all(path).expect("remove LMDB test store");
    }

    #[test]
    fn legacy_no_effect_receipt_does_not_derive_a_successful_effect_relation() {
        let path = temp_store_path("legacy-no-effect-graph");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        let receipt = Record::from_parts(
            "rec:fixture-no-effect",
            "case:fixture-no-effect",
            RecordKind::FilesystemReceipt,
            "subject:fixture",
            "attempt:fixture-write",
            "decision:fixture-descriptor",
            "receipt:fixture-no-effect",
            "fixture_receipt status:not_executed no_resource_effect:true carrier_attempted:false execution_performed:false",
        );
        store
            .append_record(&receipt, "legacy-fixture-test")
            .expect("append compatibility receipt");
        store
            .materialize_graph_relations_for_case("case:fixture-no-effect")
            .expect("materialize compatibility graph");
        let relations = store
            .list_graph_relations_by_case("case:fixture-no-effect", usize::MAX)
            .expect("list graph relations");
        assert!(relations.relations.iter().any(|relation| {
            relation.source_record_id == "rec:fixture-no-effect"
                && relation.edge_kind == "receipt_records_no_effect"
                && relation.to_kind == "attempt"
        }));
        assert!(!relations.relations.iter().any(|relation| {
            relation.source_record_id == "rec:fixture-no-effect"
                && relation.edge_kind == "receipt_records_effect"
        }));
        drop(store);
        fs::remove_dir_all(path).expect("remove LMDB test store");
    }

    #[test]
    fn journal_import_report_is_idempotent() {
        let path = temp_store_path("journal-import-idempotent");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        let mut journal = Journal::new();
        journal.append(Record::from_parts(
            "rec:journal-import-one",
            "case:journal-import",
            RecordKind::Receipt,
            "subject:journal-import",
            "op:journal-import",
            "decision:journal-import",
            "receipt:journal-import",
            "receipt:journal-import",
        ));
        journal.append(Record::from_parts(
            "rec:journal-import-two",
            "case:journal-import",
            RecordKind::Divergence,
            "subject:journal-import",
            "op:journal-import",
            "decision:journal-import",
            "receipt:journal-import",
            "divergence:none",
        ));

        let first = store
            .import_journal_with_report(&journal, "journal-import-test")
            .expect("first journal import");
        assert_eq!(first.records_seen, 2);
        assert_eq!(first.records_written, 2);
        assert_eq!(first.records_duplicate, 0);

        let second = store
            .import_journal_with_report(&journal, "journal-import-test")
            .expect("second journal import");
        assert_eq!(second.records_seen, 2);
        assert_eq!(second.records_written, 0);
        assert_eq!(second.records_duplicate, 2);

        let summary = store.summary().expect("summary");
        assert_eq!(summary.records_total, 2);
        assert_eq!(summary.records_by_case, 2);
        assert_eq!(summary.records_by_kind, 2);
        assert_eq!(summary.records_by_subject, 2);
        assert_eq!(summary.records_by_receipt, 2);

        drop(store);
        fs::remove_dir_all(path).expect("remove LMDB test store");
    }

    #[test]
    fn case_runtime_admission_is_exclusive_reclaimable_and_noncanonical() {
        let path = temp_store_path("case-runtime-admission");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        let case_id = "case:runtime-admission";
        store
            .commit_transition(pending(
                "transition:runtime-admission-open",
                case_id,
                0,
                TransitionPayload::CaseOpened {
                    lifecycle: CaseLifecycle::Open,
                },
            ))
            .expect("commit canonical case");
        let canonical_before = store
            .list_case_transitions(case_id)
            .expect("canonical history before claim");
        let owner_a = CaseRuntimeAdmissionRequest {
            case_id: case_id.to_string(),
            run_id: "run:a".to_string(),
            owner_token: "owner:a".to_string(),
            owner_pid: 1001,
            now_unix_ms: 1_000,
            lease_duration_ms: 100,
        };
        let (outcome, claim) = store
            .acquire_case_runtime_admission(&owner_a, false)
            .expect("first owner acquires");
        assert_eq!(outcome, CaseRuntimeAdmissionOutcome::Acquired);
        assert_eq!(claim.expires_at_unix_ms, 1_100);

        let owner_b_active = CaseRuntimeAdmissionRequest {
            case_id: case_id.to_string(),
            run_id: "run:b".to_string(),
            owner_token: "owner:b".to_string(),
            owner_pid: 1002,
            now_unix_ms: 1_050,
            lease_duration_ms: 100,
        };
        let error = store
            .acquire_case_runtime_admission(&owner_b_active, false)
            .expect_err("second active owner must fail closed");
        assert!(error.contains("case_runtime_admission_active"));

        let mut owner_a_renew = owner_a.clone();
        owner_a_renew.now_unix_ms = 1_060;
        let (outcome, renewed) = store
            .acquire_case_runtime_admission(&owner_a_renew, false)
            .expect("same owner renews");
        assert_eq!(outcome, CaseRuntimeAdmissionOutcome::Renewed);
        assert_eq!(renewed.acquired_at_unix_ms, 1_000);

        let mut owner_b_stale = owner_b_active;
        owner_b_stale.now_unix_ms = 1_200;
        let (outcome, reclaimed) = store
            .acquire_case_runtime_admission(&owner_b_stale, true)
            .expect("expired claim is reclaimed");
        assert_eq!(outcome, CaseRuntimeAdmissionOutcome::Reclaimed);
        assert_eq!(reclaimed.run_id, "run:b");
        assert!(store
            .release_case_runtime_admission(case_id, "run:a", "owner:a")
            .unwrap_err()
            .contains("release_owner_mismatch"));
        store
            .release_case_runtime_admission(case_id, "run:b", "owner:b")
            .expect("current owner releases");
        assert!(store
            .get_case_runtime_admission(case_id)
            .expect("read released claim")
            .is_none());
        assert_eq!(
            store
                .list_case_transitions(case_id)
                .expect("canonical history after claim lifecycle"),
            canonical_before,
            "runtime admission metadata must not mutate canonical history"
        );
        assert!(store
            .verify_case_state(case_id)
            .expect("replay after claims"));

        drop(store);
        fs::remove_dir_all(path).expect("remove LMDB test store");
    }

    #[test]
    fn typed_human_review_replays_without_promoting_approval_to_effect_truth() {
        let path = temp_store_path("typed-human-review-replay");
        let store = LmdbRecordStore::open(&path).expect("open LMDB test store");
        let case_id = "case:typed-human-review";
        let model = "participant:model";
        let reviewer = "participant:reviewer";
        commit_typed(
            &store,
            "transition:review-case-open",
            case_id,
            0,
            TransitionPayload::CaseOpened {
                lifecycle: CaseLifecycle::Open,
            },
            None,
            vec![],
        );
        commit_typed(
            &store,
            "transition:review-model-bound",
            case_id,
            1,
            TransitionPayload::ParticipantBound {
                participant_id: model.to_string(),
                role: "model_participant".to_string(),
            },
            None,
            vec![],
        );
        commit_typed(
            &store,
            "transition:review-human-bound",
            case_id,
            2,
            TransitionPayload::ParticipantBound {
                participant_id: reviewer.to_string(),
                role: "resource_policy_owner".to_string(),
            },
            None,
            vec![],
        );
        commit_typed(
            &store,
            "transition:review-provider",
            case_id,
            3,
            TransitionPayload::ProviderAttached {
                participant_id: model.to_string(),
                provider_id: "provider:review".to_string(),
                provider_kind: "openai_compatible".to_string(),
                base_url: "http://127.0.0.1:1".to_string(),
                model_id: "model:review".to_string(),
                credential_ref: "env:TEST".to_string(),
            },
            None,
            vec![],
        );
        let lineage = test_provider_lineage(4);
        commit_typed(
            &store,
            "transition:review-invocation",
            case_id,
            4,
            TransitionPayload::ProviderInvocationStarted {
                invocation_id: "invocation:review".to_string(),
                participant_id: model.to_string(),
                provider_id: "provider:review".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:review".to_string(),
                semantic_lineage: Some(lineage.clone()),
            },
            None,
            vec![],
        );
        commit_typed(
            &store,
            "transition:review-result",
            case_id,
            5,
            TransitionPayload::ProviderResultRecorded {
                result_id: "provider-result:review".to_string(),
                invocation_id: "invocation:review".to_string(),
                provider_id: "provider:review".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:review".to_string(),
                semantic_lineage: Some(lineage),
                output: "typed review proposal".to_string(),
            },
            None,
            vec!["invocation:review".to_string()],
        );
        let resource = ResourceAttachmentState {
            attachment_id: "workspace".to_string(),
            kind: ResourceKind::Filesystem,
            allowed_write_prefix: "allowed".to_string(),
            max_write_bytes: 128,
            policy_id: "policy:review".to_string(),
            policy_owner_participant_id: reviewer.to_string(),
            review_requirement: ReviewRequirement::RequireReview,
            process_signal_actions: Vec::new(),
        };
        commit_typed(
            &store,
            "transition:review-resource",
            case_id,
            6,
            TransitionPayload::ResourceAttached {
                attachment: resource.clone(),
            },
            None,
            vec![reviewer.to_string()],
        );
        let operation = normalize_filesystem_write_candidate(
            r#"{"schema":"yai.operation_proposal.filesystem_write.v1","operation":"filesystem.write","resource":"workspace","path":"allowed/reviewed.txt","content":"reviewed"}"#,
            &NormalizationContext {
                case_id,
                participant_id: model,
                provider_result_id: "provider-result:review",
                provider_invocation_id: "invocation:review",
                case_generation: 7,
                resource: &resource,
            },
        )
        .expect("normalize original operation");
        commit_typed(
            &store,
            "transition:review-operation",
            case_id,
            7,
            TransitionPayload::OperationRecorded {
                operation: operation.clone(),
            },
            Some(operation.scope.clone()),
            operation.origin.causal_refs(),
        );
        let initial = decide_filesystem_write(&operation, &resource, 8);
        assert_eq!(
            initial.outcome,
            crate::effect::DecisionOutcome::RequireReview
        );
        commit_typed(
            &store,
            "transition:review-required",
            case_id,
            8,
            TransitionPayload::DecisionRecorded {
                decision: initial.clone(),
            },
            Some(operation.scope.clone()),
            std::iter::once(operation.operation_id.clone())
                .chain(initial.basis_refs.iter().cloned())
                .collect(),
        );
        let review = build_filesystem_review_request(&operation, &initial, &resource, 9)
            .expect("build review request");
        commit_typed(
            &store,
            "transition:review-request",
            case_id,
            9,
            TransitionPayload::ReviewRequested {
                review: review.clone(),
            },
            Some(operation.scope.clone()),
            vec![operation.operation_id.clone(), initial.decision_id.clone()],
        );
        let action = build_review_action(
            &review,
            case_id,
            reviewer,
            ReviewActionKind::Approve,
            "approve exact operation",
            10,
            "local_cli_claimed_participant",
        )
        .expect("build review action");
        let action_commit = commit_typed(
            &store,
            "transition:review-action",
            case_id,
            10,
            TransitionPayload::ReviewActionRecorded {
                action: action.clone(),
            },
            Some(operation.scope.clone()),
            vec![review.review_id.clone(), operation.operation_id.clone()],
        );
        let resolved_review = action_commit
            .state
            .reviews
            .iter()
            .find(|item| item.review_id == review.review_id)
            .expect("materialized resolved review");
        assert_eq!(resolved_review.status, ReviewResolution::Approved);
        let effective =
            resolve_filesystem_review_decision(&operation, &resource, resolved_review, &action, 11)
                .expect("derive effective decision");
        let final_commit = commit_typed(
            &store,
            "transition:review-effective-decision",
            case_id,
            11,
            TransitionPayload::DecisionRecorded {
                decision: effective.clone(),
            },
            Some(operation.scope.clone()),
            std::iter::once(operation.operation_id.clone())
                .chain(effective.basis_refs.iter().cloned())
                .collect(),
        );
        assert_eq!(final_commit.state.generation, 12);
        assert_eq!(
            final_commit.state.reviews[0]
                .effective_decision_id
                .as_deref(),
            Some(effective.decision_id.as_str())
        );
        assert!(final_commit.state.effects.is_empty());
        assert!(store.verify_case_state(case_id).expect("review replay"));
        let history = store
            .list_case_transitions(case_id)
            .expect("review canonical history");
        let memory = derive_operational_memory(case_id, &history).expect("derive review memory");
        assert!(memory.entries.iter().any(|entry| {
            entry.semantic_kind == OperationalMemoryKind::Review
                && entry.description.contains("was approved")
        }));
        assert!(!memory
            .entries
            .iter()
            .any(|entry| entry.semantic_kind == OperationalMemoryKind::ResourceEffect));

        drop(store);
        fs::remove_dir_all(path).expect("remove LMDB test store");
    }

    fn open_policy_case(store: &LmdbRecordStore, case_id: &str) -> CaseState {
        store
            .commit_transition(pending(
                &format!("transition:open:{case_id}"),
                case_id,
                0,
                TransitionPayload::CaseOpened {
                    lifecycle: CaseLifecycle::Open,
                },
            ))
            .expect("open policy Case")
            .state
    }

    fn publish_compilation(store: &LmdbRecordStore, bytes: &[u8]) -> PolicyCompilation {
        let compilation = compile_policy_source(bytes).expect("compile policy source");
        store
            .ingest_policy_compilation(&compilation, "participant:policy-admin")
            .expect("ingest candidate");
        store
            .validate_policy_artifact(
                &compilation.artifact.artifact_id,
                "participant:policy-admin",
                "qualified for binding test",
            )
            .expect("validate candidate");
        store
            .publish_policy_artifact(
                &compilation.artifact.artifact_id,
                "participant:policy-admin",
                "publish for binding test",
            )
            .expect("publish policy");
        compilation
    }

    fn policy_with_rules(
        owner: &str,
        key: &str,
        version: &str,
        effect: &str,
        review: bool,
    ) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "schema": POLICY_SOURCE_INPUT_SCHEMA,
            "policy_key": key,
            "source_version": version,
            "owner_ref": owner,
            "source_origin": {
                "source_system": "wave9-test",
                "source_uri": format!("test://wave9/{owner}/{key}/{version}")
            },
            "validity": {"mode":"unbounded"},
            "rules": [
                {"kind":"operation_restriction","rule_id":format!("operation-{key}-{version}"),"operation_kind":"filesystem.write","resource_kind":"filesystem","effect":effect,"reason":"deterministic operation posture"},
                {"kind":"review_requirement","rule_id":format!("review-{key}-{version}"),"operation_kind":"filesystem.write","resource_kind":"filesystem","required":review,"reason":"deterministic review posture"},
                {"kind":"evidence_obligation","rule_id":format!("evidence-{key}-{version}"),"operation_kind":"filesystem.write","resource_kind":"filesystem","obligation":"post_observation","reason":"observed consequence required"}
            ]
        }))
        .expect("serialize Wave-9 policy")
    }

    fn h10_authority_policy(key: &str, version: &str, effect: &str, review: bool) -> Vec<u8> {
        let mut rules = vec![
            serde_json::json!({"kind":"operation_restriction","rule_id":format!("operation-{key}"),"operation_kind":"filesystem.write","resource_kind":"filesystem","effect":effect,"reason":"H10 operation posture"}),
            serde_json::json!({"kind":"review_requirement","rule_id":format!("review-{key}"),"operation_kind":"filesystem.write","resource_kind":"filesystem","required":review,"reason":"H10 review posture"}),
            serde_json::json!({"kind":"authority_requirement","rule_id":format!("proposer-{key}"),"operation_kind":"filesystem.write","resource_kind":"filesystem","subject":"proposer","required_role":"operation-proposer","reason":"H10 proposer eligibility"}),
            serde_json::json!({"kind":"evidence_obligation","rule_id":format!("source-{key}"),"operation_kind":"filesystem.write","resource_kind":"filesystem","obligation":"source_provenance","reason":"H10 canonical provider lineage"}),
            serde_json::json!({"kind":"evidence_obligation","rule_id":format!("pre-{key}"),"operation_kind":"filesystem.write","resource_kind":"filesystem","obligation":"pre_observation","reason":"H10 effect preparation evidence"}),
            serde_json::json!({"kind":"evidence_obligation","rule_id":format!("post-{key}"),"operation_kind":"filesystem.write","resource_kind":"filesystem","obligation":"post_observation","reason":"H10 effect closure"}),
        ];
        if review {
            rules.push(serde_json::json!({"kind":"authority_requirement","rule_id":format!("reviewer-{key}"),"operation_kind":"filesystem.write","resource_kind":"filesystem","subject":"reviewer","required_role":"operation-reviewer","reason":"H10 reviewer eligibility"}));
            rules.push(serde_json::json!({"kind":"evidence_obligation","rule_id":format!("audit-{key}"),"operation_kind":"filesystem.write","resource_kind":"filesystem","obligation":"audit_reason","reason":"H10 canonical review rationale"}));
        }
        serde_json::to_vec(&serde_json::json!({
            "schema": POLICY_SOURCE_INPUT_SCHEMA,
            "policy_key": key,
            "source_version": version,
            "owner_ref": "organization:h10",
            "source_origin": {
                "source_system": "h10-test",
                "source_uri": format!("test://h10/{key}/{version}")
            },
            "validity": {"mode":"unbounded"},
            "rules": rules,
        }))
        .expect("serialize H10 policy")
    }

    fn wave11_bounded_policy(
        key: &str,
        version: &str,
        effect: &str,
        valid_from: u64,
        refresh_after: u64,
        expires_at: u64,
    ) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "schema": POLICY_SOURCE_INPUT_SCHEMA,
            "policy_key": key,
            "source_version": version,
            "owner_ref": "organization:wave11",
            "source_origin": {
                "source_system": "wave11-test",
                "source_uri": format!("test://wave11/{key}/{version}")
            },
            "validity": {
                "mode": "bounded",
                "valid_from_unix_ms": valid_from,
                "refresh_after_unix_ms": refresh_after,
                "expires_at_unix_ms": expires_at
            },
            "rules": [
                {"kind":"operation_restriction","rule_id":format!("operation-{key}-{version}"),"operation_kind":"filesystem.write","resource_kind":"filesystem","effect":effect,"reason":"Wave11 bounded operation posture"},
                {"kind":"authority_requirement","rule_id":format!("proposer-{key}-{version}"),"operation_kind":"filesystem.write","resource_kind":"filesystem","subject":"proposer","required_role":"operation-proposer","reason":"Wave11 proposer authority"},
                {"kind":"evidence_obligation","rule_id":format!("source-{key}-{version}"),"operation_kind":"filesystem.write","resource_kind":"filesystem","obligation":"source_provenance","reason":"Wave11 source lineage"}
            ]
        }))
        .expect("serialize Wave11 bounded policy")
    }

    fn advance_authority_floor(store: &LmdbRecordStore, observed: u64) -> u64 {
        let mut txn = store.env.begin_rw_txn().expect("authority floor txn");
        let effective = store
            .advance_authority_time_txn(&mut txn, observed)
            .expect("advance authority floor");
        txn.commit().expect("commit authority floor");
        effective
    }

    fn setup_h10_authority_case(
        store: &LmdbRecordStore,
        case_id: &str,
        effect: &str,
        review: bool,
    ) -> (ResourceAttachmentState, Operation) {
        let proposer = "participant:model";
        let reviewer = "participant:reviewer";
        open_policy_case(store, case_id);
        commit_typed(
            store,
            &format!("transition:{case_id}:proposer"),
            case_id,
            1,
            TransitionPayload::ParticipantBound {
                participant_id: proposer.to_string(),
                role: "operation-proposer".to_string(),
            },
            None,
            vec![],
        );
        commit_typed(
            store,
            &format!("transition:{case_id}:reviewer"),
            case_id,
            2,
            TransitionPayload::ParticipantBound {
                participant_id: reviewer.to_string(),
                role: "operation-reviewer".to_string(),
            },
            None,
            vec![],
        );
        let resource = ResourceAttachmentState {
            attachment_id: "workspace".to_string(),
            kind: ResourceKind::Filesystem,
            allowed_write_prefix: "allowed".to_string(),
            max_write_bytes: 256,
            policy_id: "policy:legacy-inert".to_string(),
            policy_owner_participant_id: reviewer.to_string(),
            review_requirement: ReviewRequirement::Automatic,
            process_signal_actions: Vec::new(),
        };
        commit_typed(
            store,
            &format!("transition:{case_id}:resource"),
            case_id,
            3,
            TransitionPayload::ResourceAttached {
                attachment: resource.clone(),
            },
            None,
            vec![reviewer.to_string()],
        );
        let policy = publish_compilation(
            store,
            &h10_authority_policy("authority", "1", effect, review),
        );
        store
            .bind_case_policy(
                case_id,
                &policy.artifact.artifact_id,
                4,
                "participant:operator",
                "bind H10 authority policy",
            )
            .expect("bind H10 policy");
        commit_typed(
            store,
            &format!("transition:{case_id}:provider"),
            case_id,
            5,
            TransitionPayload::ProviderAttached {
                participant_id: proposer.to_string(),
                provider_id: "provider:h10".to_string(),
                provider_kind: "openai_compatible".to_string(),
                base_url: "http://127.0.0.1:1".to_string(),
                model_id: "model:h10".to_string(),
                credential_ref: "env:TEST".to_string(),
            },
            None,
            vec![],
        );
        let lineage = test_provider_lineage(6);
        commit_typed(
            store,
            &format!("transition:{case_id}:invocation"),
            case_id,
            6,
            TransitionPayload::ProviderInvocationStarted {
                invocation_id: format!("invocation:{case_id}"),
                participant_id: proposer.to_string(),
                provider_id: "provider:h10".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:h10".to_string(),
                semantic_lineage: Some(lineage.clone()),
            },
            None,
            vec![],
        );
        commit_typed(
            store,
            &format!("transition:{case_id}:result"),
            case_id,
            7,
            TransitionPayload::ProviderResultRecorded {
                result_id: format!("provider-result:{case_id}"),
                invocation_id: format!("invocation:{case_id}"),
                provider_id: "provider:h10".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:h10".to_string(),
                semantic_lineage: Some(lineage),
                output: "H10 typed operation proposal".to_string(),
            },
            None,
            vec![format!("invocation:{case_id}")],
        );
        let operation = normalize_filesystem_write_candidate(
            r#"{"schema":"yai.operation_proposal.filesystem_write.v1","operation":"filesystem.write","resource":"workspace","path":"allowed/h10.txt","content":"h10"}"#,
            &NormalizationContext {
                case_id,
                participant_id: proposer,
                provider_result_id: &format!("provider-result:{case_id}"),
                provider_invocation_id: &format!("invocation:{case_id}"),
                case_generation: 8,
                resource: &resource,
            },
        )
        .expect("normalize H10 operation");
        commit_typed(
            store,
            &format!("transition:{case_id}:operation"),
            case_id,
            8,
            TransitionPayload::OperationRecorded {
                operation: operation.clone(),
            },
            Some(operation.scope.clone()),
            operation.origin.causal_refs(),
        );
        (resource, operation)
    }

    #[test]
    fn wave9_exact_version_binding_replacement_replay_and_rebuild_are_deterministic() {
        let path = temp_store_path("wave9-version-pinning");
        let store = LmdbRecordStore::open(&path).expect("open store");
        let state = open_policy_case(&store, "case:wave9-version");
        let v1 = publish_compilation(
            &store,
            &policy_with_rules(
                "organization:acme",
                "filesystem-security",
                "1",
                "allow",
                false,
            ),
        );
        let bound = store
            .bind_case_policy(
                &state.case_id,
                &v1.artifact.artifact_id,
                state.generation,
                "participant:operator",
                "pin exact version one",
            )
            .expect("bind v1");
        assert!(bound.changed);
        assert_eq!(bound.status.readiness, NormativeReadiness::Ready);
        assert_eq!(
            bound.commit.as_ref().unwrap().state.policy_bindings.len(),
            1
        );
        assert_eq!(
            bound.commit.as_ref().unwrap().state.policy_bindings[0].artifact_version,
            "1"
        );
        let e1 = bound.status.effective_policy.clone().expect("effective v1");

        let v2 = publish_compilation(
            &store,
            &policy_with_rules(
                "organization:acme",
                "filesystem-security",
                "2",
                "deny",
                true,
            ),
        );
        let pinned = store
            .case_policy_status(&state.case_id)
            .expect("pinned status");
        assert_eq!(
            store
                .get_case_state(&state.case_id)
                .unwrap()
                .unwrap()
                .policy_bindings[0]
                .artifact_id,
            v1.artifact.artifact_id
        );
        assert_eq!(
            pinned
                .effective_policy
                .as_ref()
                .unwrap()
                .effective_policy_id,
            e1.effective_policy_id
        );
        assert!(matches!(
            pinned.catalog_drift.values().next(),
            Some(PolicyCatalogDrift::Superseded { .. })
        ));

        let before_replace = store.get_case_state(&state.case_id).unwrap().unwrap();
        let prior_binding = before_replace.policy_bindings[0].binding_id.clone();
        let replaced = store
            .replace_case_policy(
                &state.case_id,
                &prior_binding,
                &v2.artifact.artifact_id,
                before_replace.generation,
                "participant:operator",
                "explicitly replace with version two",
            )
            .expect("replace v1 with v2");
        let e2 = replaced
            .status
            .effective_policy
            .clone()
            .expect("effective v2");
        assert_ne!(e1.effective_policy_id, e2.effective_policy_id);
        assert_eq!(
            replaced.commit.as_ref().unwrap().state.policy_bindings[0].artifact_version,
            "2"
        );
        store
            .discard_policy_source_for_test(&v2.source.source_id)
            .expect("simulate future source-payload retention loss");
        assert_eq!(
            store.case_policy_status(&state.case_id).unwrap().readiness,
            NormativeReadiness::Ready
        );
        assert!(store
            .verify_case_state(&state.case_id)
            .expect("replay exact binding"));
        let transition_count = store.list_case_transitions(&state.case_id).unwrap().len();
        assert!(store
            .drop_effective_policy(&state.case_id)
            .expect("drop cache"));
        assert!(store
            .cached_effective_policy(&state.case_id)
            .unwrap()
            .is_none());
        let rebuilt = store
            .rebuild_effective_policy(&state.case_id)
            .expect("rebuild derived");
        assert_eq!(
            rebuilt.effective_policy.unwrap().effective_policy_id,
            e2.effective_policy_id
        );
        assert_eq!(
            store.list_case_transitions(&state.case_id).unwrap().len(),
            transition_count
        );
        assert!(store
            .list_case_transitions(&state.case_id)
            .unwrap()
            .iter()
            .all(|item| !matches!(
                item.payload,
                TransitionPayload::DecisionRecorded { .. }
                    | TransitionPayload::ExecutionGrantIssued { .. }
                    | TransitionPayload::EffectPrepared { .. }
            )));
        drop(store);
        fs::remove_dir_all(path).expect("remove store");
    }

    #[test]
    fn wave9_derived_cache_failure_preserves_canonical_binding_and_repairs_without_duplication() {
        let path = temp_store_path("wave9-derived-failure");
        let store = LmdbRecordStore::open(&path).expect("open store");
        let state = open_policy_case(&store, "case:wave9-derived-failure");
        let policy = publish_compilation(
            &store,
            &policy_with_rules("organization:acme", "derived-failure", "1", "deny", true),
        );
        let outcome = store
            .bind_case_policy_inner(
                &state.case_id,
                &policy.artifact.artifact_id,
                state.generation,
                "participant:operator",
                "inject derived cache failure after canonical commit",
                true,
                None,
            )
            .expect("canonical bind survives derived failure");
        assert!(outcome.changed);
        assert_eq!(
            outcome.derived_cache_error.as_deref(),
            Some("injected_effective_policy_cache_failure")
        );
        assert!(store
            .cached_effective_policy(&state.case_id)
            .expect("read absent cache")
            .is_none());
        let committed = store.get_case_state(&state.case_id).unwrap().unwrap();
        assert_eq!(committed.policy_bindings.len(), 1);
        let transition_count = store.list_case_transitions(&state.case_id).unwrap().len();
        let repaired = store
            .rebuild_effective_policy(&state.case_id)
            .expect("repair derived materialization");
        assert_eq!(repaired.readiness, NormativeReadiness::Ready);
        assert!(store
            .cached_effective_policy(&state.case_id)
            .unwrap()
            .is_some());
        assert_eq!(
            store.list_case_transitions(&state.case_id).unwrap().len(),
            transition_count
        );
        drop(store);
        fs::remove_dir_all(path).expect("remove store");
    }

    #[test]
    fn wave9_binding_admission_rejects_candidate_validated_and_missing_case() {
        let path = temp_store_path("wave9-admission");
        let store = LmdbRecordStore::open(&path).expect("open store");
        let state = open_policy_case(&store, "case:wave9-admission");
        let candidate = compile_policy_source(&policy_source("candidate", true)).unwrap();
        store
            .ingest_policy_compilation(&candidate, "participant:admin")
            .unwrap();
        assert!(store
            .bind_case_policy(
                &state.case_id,
                &candidate.artifact.artifact_id,
                state.generation,
                "participant:operator",
                "must fail candidate"
            )
            .unwrap_err()
            .contains("not_eligible"));
        store
            .validate_policy_artifact(
                &candidate.artifact.artifact_id,
                "participant:admin",
                "valid",
            )
            .unwrap();
        assert!(store
            .bind_case_policy(
                &state.case_id,
                &candidate.artifact.artifact_id,
                state.generation,
                "participant:operator",
                "must fail validated"
            )
            .unwrap_err()
            .contains("not_eligible"));
        store
            .publish_policy_artifact(
                &candidate.artifact.artifact_id,
                "participant:admin",
                "publish first version",
            )
            .unwrap();
        let replacement = publish_compilation(&store, &policy_source("replacement", false));
        assert!(store
            .bind_case_policy(
                &state.case_id,
                &candidate.artifact.artifact_id,
                state.generation,
                "participant:operator",
                "must fail superseded",
            )
            .unwrap_err()
            .contains("not_eligible"));
        store
            .retire_policy_artifact(
                &replacement.artifact.artifact_id,
                "participant:admin",
                "retire current version",
            )
            .unwrap();
        assert!(store
            .bind_case_policy(
                &state.case_id,
                &replacement.artifact.artifact_id,
                state.generation,
                "participant:operator",
                "must fail retired",
            )
            .unwrap_err()
            .contains("not_eligible"));
        assert_eq!(
            store
                .get_case_state(&state.case_id)
                .unwrap()
                .unwrap()
                .generation,
            1
        );
        assert!(store
            .bind_case_policy(
                "case:missing",
                &candidate.artifact.artifact_id,
                0,
                "participant:operator",
                "must fail missing Case"
            )
            .unwrap_err()
            .contains("case_state_not_found"));
        drop(store);
        fs::remove_dir_all(path).expect("remove store");
    }

    #[test]
    fn wave9_multi_artifact_composition_is_order_independent_conservative_and_provenanced() {
        let path = temp_store_path("wave9-composition");
        let store = LmdbRecordStore::open(&path).expect("open store");
        let state = open_policy_case(&store, "case:wave9-composition");
        let allow = publish_compilation(
            &store,
            &policy_with_rules("organization:acme", "baseline", "1", "allow", false),
        );
        let deny = publish_compilation(
            &store,
            &policy_with_rules("organization:acme", "hardening", "1", "deny", true),
        );
        store
            .bind_case_policy(
                &state.case_id,
                &deny.artifact.artifact_id,
                1,
                "participant:operator",
                "bind deny first",
            )
            .unwrap();
        let outcome = store
            .bind_case_policy(
                &state.case_id,
                &allow.artifact.artifact_id,
                2,
                "participant:operator",
                "bind allow second",
            )
            .unwrap();
        let effective = outcome
            .status
            .effective_policy
            .expect("effective multi policy");
        assert_eq!(effective.input_rule_count, 6);
        assert_eq!(effective.rules.len(), 3);
        assert_eq!(effective.resolved_conflict_count, 2);
        assert!(effective.rules.iter().any(|rule| matches!(rule, crate::case_policy::EffectivePolicyRule::OperationRestriction { effect: crate::governance::PolicyEffect::Deny, contributions, .. } if contributions.len() == 2)));
        assert!(effective.rules.iter().any(|rule| matches!(rule, crate::case_policy::EffectivePolicyRule::ReviewRequirement { required: true, contributions, .. } if contributions.len() == 2)));
        drop(store);
        fs::remove_dir_all(path).expect("remove store");
    }

    #[test]
    fn wave9_missing_artifact_blocks_readiness_without_erasing_binding_history() {
        let path = temp_store_path("wave9-missing");
        let store = LmdbRecordStore::open(&path).expect("open store");
        let state = open_policy_case(&store, "case:wave9-missing");
        let policy = publish_compilation(&store, &policy_source("1", true));
        store
            .bind_case_policy(
                &state.case_id,
                &policy.artifact.artifact_id,
                1,
                "participant:operator",
                "bind before simulated catalog loss",
            )
            .unwrap();
        let before = store.get_case_state(&state.case_id).unwrap().unwrap();
        let mut corrupted = policy.artifact.clone();
        corrupted.policy_ir.ir_digest = format!("sha256:{}", "0".repeat(64));
        {
            let mut txn = store.env.begin_rw_txn().unwrap();
            txn.put(
                store.policy_artifacts_by_id,
                &policy_artifact_key(&corrupted.artifact_id),
                &serde_json::to_vec(&corrupted).unwrap(),
                WriteFlags::empty(),
            )
            .unwrap();
            txn.commit().unwrap();
        }
        let corrupted_status = store
            .case_policy_status(&state.case_id)
            .expect("corrupt artifact becomes blocked diagnostics");
        assert_eq!(corrupted_status.readiness, NormativeReadiness::Blocked);
        assert!(corrupted_status.missing[0].contains("policy_ir_not_reproducible"));
        {
            let mut txn = store.env.begin_rw_txn().unwrap();
            txn.put(
                store.policy_artifacts_by_id,
                &policy_artifact_key(&policy.artifact.artifact_id),
                &serde_json::to_vec(&policy.artifact).unwrap(),
                WriteFlags::empty(),
            )
            .unwrap();
            txn.commit().unwrap();
        }
        store
            .discard_policy_artifact_for_test(&policy.artifact.artifact_id)
            .expect("simulate missing immutable artifact");
        let status = store
            .case_policy_status(&state.case_id)
            .expect("blocked status");
        assert_eq!(status.readiness, NormativeReadiness::Blocked);
        assert_eq!(status.missing.len(), 1);
        assert_eq!(
            store.get_case_state(&state.case_id).unwrap().unwrap(),
            before
        );
        assert!(store.verify_case_state(&state.case_id).unwrap());
        drop(store);
        fs::remove_dir_all(path).expect("remove store");
    }

    #[test]
    fn wave9_idempotence_unbind_multi_case_and_concurrent_mutation_are_safe() {
        let path = temp_store_path("wave9-concurrency");
        let store = LmdbRecordStore::open(&path).expect("open store");
        open_policy_case(&store, "case:wave9-a");
        open_policy_case(&store, "case:wave9-b");
        let first = publish_compilation(
            &store,
            &policy_with_rules("organization:acme", "shared", "1", "allow", false),
        );
        let second = publish_compilation(
            &store,
            &policy_with_rules("organization:acme", "independent", "1", "deny", true),
        );
        let bound_a = store
            .bind_case_policy(
                "case:wave9-a",
                &first.artifact.artifact_id,
                1,
                "participant:operator",
                "bind shared to A",
            )
            .unwrap();
        let duplicate = store
            .bind_case_policy(
                "case:wave9-a",
                &first.artifact.artifact_id,
                2,
                "participant:operator",
                "repeat exact binding",
            )
            .unwrap();
        assert!(!duplicate.changed);
        assert_eq!(
            store
                .get_case_state("case:wave9-a")
                .unwrap()
                .unwrap()
                .generation,
            2
        );
        store
            .bind_case_policy(
                "case:wave9-b",
                &first.artifact.artifact_id,
                1,
                "participant:operator",
                "bind shared to B",
            )
            .unwrap();
        assert_eq!(
            store
                .get_case_state("case:wave9-a")
                .unwrap()
                .unwrap()
                .policy_bindings
                .len(),
            1
        );
        assert_eq!(
            store
                .get_case_state("case:wave9-b")
                .unwrap()
                .unwrap()
                .policy_bindings
                .len(),
            1
        );
        let first_binding = bound_a.commit.unwrap().state.policy_bindings[0]
            .binding_id
            .clone();
        drop(store);

        let barrier = Arc::new(Barrier::new(3));
        let mut joins = Vec::new();
        for index in 0..2 {
            let thread_path = path.clone();
            let thread_barrier = Arc::clone(&barrier);
            let artifact = second.artifact.artifact_id.clone();
            joins.push(thread::spawn(move || {
                let thread_store =
                    LmdbRecordStore::open(&thread_path).expect("open concurrent store");
                thread_barrier.wait();
                thread_store.bind_case_policy(
                    "case:wave9-a",
                    &artifact,
                    2,
                    "participant:operator",
                    &format!("concurrent bind {index}"),
                )
            }));
        }
        barrier.wait();
        let results = joins
            .into_iter()
            .map(|join| join.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter(|result| result
                    .as_ref()
                    .err()
                    .is_some_and(|error| error.contains("stale_case_generation")))
                .count(),
            1
        );
        let store = LmdbRecordStore::open(&path).expect("reopen after concurrent writers");
        let current = store.get_case_state("case:wave9-a").unwrap().unwrap();
        assert_eq!(current.policy_bindings.len(), 2);
        let unbound = store
            .unbind_case_policy(
                "case:wave9-a",
                &first_binding,
                current.generation,
                "participant:operator",
                "remove shared policy",
            )
            .unwrap();
        assert_eq!(unbound.commit.unwrap().state.policy_bindings.len(), 1);
        assert!(store.verify_case_state("case:wave9-a").unwrap());
        drop(store);
        fs::remove_dir_all(path).expect("remove store");
    }

    #[test]
    fn wave9_many_policy_materialization_characterization_is_bounded() {
        let path = temp_store_path("wave9-many-policy");
        let store = LmdbRecordStore::open(&path).expect("open store");
        let case_id = "case:wave9-many-policy";
        open_policy_case(&store, case_id);
        let start = Instant::now();
        for index in 0..24_u64 {
            let policy = publish_compilation(
                &store,
                &policy_with_rules(
                    "organization:scale",
                    &format!("policy-{index:02}"),
                    "1",
                    if index % 2 == 0 { "allow" } else { "deny" },
                    index % 3 == 0,
                ),
            );
            store
                .bind_case_policy(
                    case_id,
                    &policy.artifact.artifact_id,
                    index + 1,
                    "participant:operator",
                    "bounded many-policy characterization",
                )
                .expect("bind independent lineage");
        }
        let status = store.case_policy_status(case_id).expect("materialize");
        let effective = status.effective_policy.expect("effective policy");
        let encoded_size = serde_json::to_vec(&effective).unwrap().len();
        assert_eq!(status.readiness, NormativeReadiness::Ready);
        assert_eq!(effective.binding_ids.len(), 24);
        assert_eq!(effective.input_rule_count, 72);
        assert_eq!(effective.rules.len(), 3);
        assert_eq!(effective.merged_rule_count, 69);
        assert_eq!(effective.resolved_conflict_count, 2);
        assert!(encoded_size < 100_000);
        println!(
            "wave9_multi_policy_characterization: artifacts=24 input_rules={} output_rules={} merged_rules={} resolved_conflicts={} blocking_conflicts={} derived_bytes={} elapsed_ms={}",
            effective.input_rule_count,
            effective.rules.len(),
            effective.merged_rule_count,
            effective.resolved_conflict_count,
            status.blocking_conflicts.len(),
            encoded_size,
            start.elapsed().as_millis()
        );
        drop(store);
        fs::remove_dir_all(path).expect("remove store");
    }

    #[test]
    fn h10_historical_p1_authority_chain_replays_after_p2_and_cache_rebuild() {
        let path = temp_store_path("h10-historical-replay");
        let root = temp_store_path("h10-historical-files");
        fs::create_dir_all(root.join("allowed")).expect("create carrier root");
        let store = LmdbRecordStore::open(&path).expect("open store");
        let case_id = "case:h10-historical-replay";
        let (resource, operation) = setup_h10_authority_case(&store, case_id, "allow", false);
        advance_authority_floor(
            &store,
            authority_wall_time_unix_ms().saturating_add(86_400_000),
        );
        let decision = store
            .derive_policy_decision(case_id, &operation.operation_id)
            .expect("derive P1 Decision");
        assert_eq!(decision.outcome, crate::effect::DecisionOutcome::Allow);
        let basis = decision.decision_basis.as_ref().unwrap();
        let mut decision_abort = PendingTransition::new(
            "transition:h10-c1-decision-abort",
            case_id,
            9,
            TransitionSource::component("h10-crash-test"),
            TransitionPayload::DecisionRecorded {
                decision: decision.clone(),
            },
        );
        decision_abort.scope = Some(operation.scope.clone());
        decision_abort.causal_refs = vec![
            operation.operation_id.clone(),
            basis.basis_id.clone(),
            basis.effective_policy_id.clone(),
        ];
        assert!(store
            .commit_transition_inner(decision_abort, true)
            .expect_err("C1 must abort after semantic verification")
            .contains("injected_failure_before_canonical_commit"));
        assert!(store
            .get_transition_by_id("transition:h10-c1-decision-abort")
            .unwrap()
            .is_none());
        assert_eq!(
            store.get_case_state(case_id).unwrap().unwrap().generation,
            9
        );
        let decision_commit = commit_typed(
            &store,
            "transition:h10-replay-decision",
            case_id,
            9,
            TransitionPayload::DecisionRecorded {
                decision: decision.clone(),
            },
            Some(operation.scope.clone()),
            vec![
                operation.operation_id.clone(),
                basis.basis_id.clone(),
                basis.effective_policy_id.clone(),
            ],
        );
        assert!(decision_commit.state.grants.is_empty());
        assert!(store.verify_case_state(case_id).expect("C2 replay"));
        let grant =
            issue_policy_execution_grant(&operation, &decision, decision_commit.state.generation)
                .expect("issue adjacent P1 Grant");
        assert!(grant
            .execution_evidence_requirements
            .contains(&crate::admission::ExecutionEvidenceRequirement::PreObservation));
        assert!(grant
            .execution_evidence_requirements
            .contains(&crate::admission::ExecutionEvidenceRequirement::PostObservation));
        let mut grant_abort = PendingTransition::new(
            "transition:h10-c5-grant-abort",
            case_id,
            decision_commit.state.generation,
            TransitionSource::component("h10-crash-test"),
            TransitionPayload::ExecutionGrantIssued {
                grant: grant.clone(),
            },
        );
        grant_abort.scope = Some(operation.scope.clone());
        grant_abort.causal_refs = vec![
            operation.operation_id.clone(),
            decision.decision_id.clone(),
            basis.basis_id.clone(),
            basis.effective_policy_id.clone(),
        ];
        assert!(store
            .commit_transition_inner(grant_abort, true)
            .expect_err("C5 must abort after Grant semantic verification")
            .contains("injected_failure_before_canonical_commit"));
        assert!(store
            .get_transition_by_id("transition:h10-c5-grant-abort")
            .unwrap()
            .is_none());
        assert!(store
            .get_case_state(case_id)
            .unwrap()
            .unwrap()
            .grants
            .is_empty());
        let grant_commit = commit_typed(
            &store,
            "transition:h10-replay-grant",
            case_id,
            decision_commit.state.generation,
            TransitionPayload::ExecutionGrantIssued {
                grant: grant.clone(),
            },
            Some(operation.scope.clone()),
            vec![
                operation.operation_id.clone(),
                decision.decision_id.clone(),
                basis.basis_id.clone(),
                basis.effective_policy_id.clone(),
            ],
        );
        let binding = LocalFilesystemBinding::new(case_id, "workspace", &root)
            .expect("local filesystem binding");
        let pre = observe_filesystem(
            &binding,
            &resource,
            &operation.filesystem_write.relative_path,
            "observation:h10-replay-pre",
        );
        let prepared =
            prepare_effect(&operation, &decision, &grant, pre).expect("prepare P1 effect");
        let prepared_commit = commit_typed(
            &store,
            "transition:h10-replay-prepare",
            case_id,
            grant_commit.state.generation,
            TransitionPayload::EffectPrepared {
                prepared: prepared.clone(),
            },
            Some(operation.scope.clone()),
            vec![
                operation.operation_id.clone(),
                decision.decision_id.clone(),
                grant.grant_id.clone(),
                prepared.expected_pre_observation.observation_id.clone(),
            ],
        );
        let result = execute_filesystem_write(
            &operation,
            &decision,
            &grant,
            &prepared,
            &prepared_commit.state,
            &binding,
            &resource,
            CarrierFailpoint::None,
        )
        .expect("execute P1 effect");
        let receipt = build_effect_receipt(&prepared, &result);
        let mut missing_post_receipt = receipt.clone();
        missing_post_receipt.post_observation_id = "observation:missing".to_string();
        assert_eq!(
            validate_execution_obligation_closure(
                &grant,
                &prepared,
                &result.post_observation,
                &missing_post_receipt,
            )
            .expect_err("required post-observation evidence must close exactly"),
            "required_post_observation_evidence_missing"
        );
        let finalized = commit_typed(
            &store,
            "transition:h10-replay-finalize",
            case_id,
            prepared_commit.state.generation,
            TransitionPayload::EffectFinalized {
                effect_id: prepared.effect_id.clone(),
                post_observation: result.post_observation.clone(),
                receipt: receipt.clone(),
            },
            Some(operation.scope.clone()),
            vec![prepared.effect_id.clone(), receipt.receipt_id.clone()],
        );
        let p2 = publish_compilation(
            &store,
            &h10_authority_policy("authority", "2", "deny", false),
        );
        let prior_binding = finalized.state.policy_bindings[0].binding_id.clone();
        let replaced = store
            .replace_case_policy(
                case_id,
                &prior_binding,
                &p2.artifact.artifact_id,
                finalized.state.generation,
                "participant:operator",
                "replace historical P1 with current P2",
            )
            .expect("replace P1 with P2");
        let current = store
            .derive_policy_decision(case_id, &operation.operation_id)
            .expect("derive current P2 Decision");
        assert_eq!(current.outcome, crate::effect::DecisionOutcome::Deny);
        assert!(store
            .drop_effective_policy(case_id)
            .expect("drop derived cache"));
        let rebuilt_policy = store
            .rebuild_effective_policy(case_id)
            .expect("rebuild P2 EffectivePolicy");
        assert_eq!(
            rebuilt_policy
                .effective_policy
                .as_ref()
                .unwrap()
                .effective_policy_id,
            replaced
                .status
                .effective_policy
                .as_ref()
                .unwrap()
                .effective_policy_id
        );
        let replay = store.replay_case_state(case_id).expect("replay after P2");
        assert_eq!(
            replay.last_decision.as_ref().unwrap().decision_id,
            decision.decision_id
        );
        assert!(replay
            .grants
            .iter()
            .any(|state| state.grant_id == grant.grant_id
                && state.status == crate::transition::GrantLifecycle::Finalized));
        let history = store
            .list_case_transitions(case_id)
            .expect("history after P2");
        let chain = validate_finalized_effect_chain(&history, &prepared.effect_id)
            .expect("historical P1 chain remains valid");
        assert_eq!(chain.receipt_id, receipt.receipt_id);
        assert!(store
            .verify_case_state(case_id)
            .expect("materialized replay"));
        println!(
            "h10_historical_replay: operation_generation=9 basis_generation={} decision_transition={} grant_expected_generation={} grant_transition={} prepare_transition={} finalize_transition={} replacement_transition={} p1_basis={} p1_decision={} p1_grant={} p1_receipt={} current_p2={} replay=true crash_c1_c2_c5=true",
            basis.evaluated_case_generation,
            decision_commit.state.generation,
            grant.expected_case_generation,
            grant_commit.state.generation,
            prepared_commit.state.generation,
            finalized.state.generation,
            replaced.commit.as_ref().unwrap().state.generation,
            basis.basis_id,
            decision.decision_id,
            grant.grant_id,
            receipt.receipt_id,
            rebuilt_policy.effective_policy.unwrap().effective_policy_id
        );
        drop(store);
        fs::remove_dir_all(path).expect("remove store");
        fs::remove_dir_all(root).expect("remove carrier root");
    }

    #[test]
    fn h10_review_writes_rederive_roles_provenance_and_final_decision() {
        let path = temp_store_path("h10-review-rederivation");
        let store = LmdbRecordStore::open(&path).expect("open store");
        let case_id = "case:h10-review-rederivation";
        let (resource, operation) = setup_h10_authority_case(&store, case_id, "allow", true);
        let state_before_decision = store.get_case_state(case_id).unwrap().unwrap();
        let effective = store
            .case_policy_status(case_id)
            .unwrap()
            .effective_policy
            .unwrap();
        let forged_evidence = forged_evidence_resolution_for_test(
            Some(vec![
                "provider-result:caller-claim".to_string(),
                "invocation:caller-claim".to_string(),
            ]),
            None,
            None,
        );
        let caller_claim_decision = evaluate_filesystem_admission(
            &operation,
            &state_before_decision,
            &resource,
            &effective,
            &forged_evidence,
            &test_temporal_context(),
        )
        .expect("construct content-valid Decision from caller evidence claim");
        let caller_claim_pending = PendingTransition::new(
            "transition:h10-caller-evidence-claim",
            case_id,
            state_before_decision.generation,
            TransitionSource::component("h10-adversarial-test"),
            TransitionPayload::DecisionRecorded {
                decision: caller_claim_decision,
            },
        );
        let caller_claim_error = store
            .commit_transition(caller_claim_pending)
            .expect_err("caller evidence claims must not cross canonical write boundary");
        assert!(
            caller_claim_error.contains("authority_decision_basis_mismatch"),
            "{caller_claim_error}"
        );
        let (initial, initial_commit) = store
            .derive_and_commit_policy_decision(case_id, &operation.operation_id)
            .expect("derive and atomically commit canonical initial review Decision");
        assert_eq!(
            initial.outcome,
            crate::effect::DecisionOutcome::RequireReview
        );
        let expected_review =
            build_policy_review_request(&operation, &initial, initial_commit.state.generation)
                .expect("build canonical ReviewRequest v2");
        let mut review_abort = PendingTransition::new(
            "transition:h10-c3-review-request-abort",
            case_id,
            initial_commit.state.generation,
            TransitionSource::component("h10-crash-test"),
            TransitionPayload::ReviewRequested {
                review: expected_review.clone(),
            },
        );
        review_abort.scope = Some(operation.scope.clone());
        review_abort.causal_refs = vec![
            operation.operation_id.clone(),
            initial.decision_id.clone(),
            expected_review.decision_basis_id.clone(),
            expected_review.effective_policy_id.clone(),
        ];
        assert!(store
            .commit_transition_inner(review_abort, true)
            .expect_err("C3 must abort after ReviewRequest semantic verification")
            .contains("injected_failure_before_canonical_commit"));
        assert!(store
            .get_transition_by_id("transition:h10-c3-review-request-abort")
            .unwrap()
            .is_none());
        assert!(store
            .get_case_state(case_id)
            .unwrap()
            .unwrap()
            .reviews
            .is_empty());
        let mut forged_review = expected_review.clone();
        forged_review.required_reviewer_roles = vec!["model".to_string()];
        let forged_review = forged_review
            .seal_policy_integrity()
            .expect("reseal content-valid forged ReviewRequest");
        let forged_review_pending = PendingTransition::new(
            "transition:h10-forged-review-request",
            case_id,
            initial_commit.state.generation,
            TransitionSource::component("h10-adversarial-test"),
            TransitionPayload::ReviewRequested {
                review: forged_review,
            },
        );
        let forged_review_error = store
            .commit_transition(forged_review_pending)
            .expect_err("forged reviewer roles must fail write-time re-derivation");
        assert!(
            forged_review_error.contains("authority_review_request_mismatch"),
            "{forged_review_error}"
        );
        let review_commit = commit_typed(
            &store,
            "transition:h10-review-request",
            case_id,
            initial_commit.state.generation,
            TransitionPayload::ReviewRequested {
                review: expected_review.clone(),
            },
            Some(operation.scope.clone()),
            vec![
                operation.operation_id.clone(),
                initial.decision_id.clone(),
                expected_review.decision_basis_id.clone(),
                expected_review.effective_policy_id.clone(),
            ],
        );

        let wrong_action = build_review_action(
            &expected_review,
            case_id,
            "participant:model",
            ReviewActionKind::Approve,
            "forged proposer approval",
            review_commit.state.generation,
            "h10-test",
        )
        .expect("build structurally valid ineligible action");
        let wrong_action_pending = PendingTransition::new(
            "transition:h10-ineligible-review-action",
            case_id,
            review_commit.state.generation,
            TransitionSource::component("h10-adversarial-test"),
            TransitionPayload::ReviewActionRecorded {
                action: wrong_action,
            },
        );
        let wrong_reviewer_error = store
            .commit_transition(wrong_action_pending)
            .expect_err("ineligible low-level ReviewAction must fail");
        assert!(
            wrong_reviewer_error.contains("review_action_binding_or_generation_mismatch"),
            "{wrong_reviewer_error}"
        );

        let action = build_review_action(
            &expected_review,
            case_id,
            "participant:reviewer",
            ReviewActionKind::Approve,
            "ticket H10-42 authorizes this exact operation",
            review_commit.state.generation,
            "h10-test",
        )
        .expect("build eligible action");
        let action_commit = commit_typed(
            &store,
            "transition:h10-eligible-review-action",
            case_id,
            review_commit.state.generation,
            TransitionPayload::ReviewActionRecorded {
                action: action.clone(),
            },
            Some(operation.scope.clone()),
            vec![
                expected_review.review_id.clone(),
                operation.operation_id.clone(),
            ],
        );
        assert_eq!(
            action_commit
                .state
                .last_decision
                .as_ref()
                .unwrap()
                .decision_id,
            initial.decision_id
        );
        assert!(store.verify_case_state(case_id).expect("C4 replay"));
        let current_review = action_commit
            .state
            .reviews
            .iter()
            .find(|review| review.review_id == expected_review.review_id)
            .expect("resolved current review");
        let history = store
            .list_case_transitions(case_id)
            .expect("canonical history");
        let evidence =
            resolve_canonical_evidence(&operation, &history, Some(action.action_id.as_str()))
                .expect("resolve canonical provider and review evidence");
        let mut forged_effective = store
            .case_policy_status(case_id)
            .expect("current policy status")
            .effective_policy
            .expect("current effective policy");
        for rule in &mut forged_effective.rules {
            if let crate::case_policy::EffectivePolicyRule::OperationRestriction {
                contributions,
                ..
            } = rule
            {
                contributions.clear();
            }
        }
        let forged_final = resolve_policy_review_decision(
            &operation,
            &action_commit.state,
            &action_commit.state.resources[0],
            &forged_effective,
            current_review,
            &action,
            &evidence,
            &test_temporal_context(),
        )
        .expect("build content-valid final Decision with forged provenance");
        let forged_final_pending = PendingTransition::new(
            "transition:h10-forged-final-decision",
            case_id,
            action_commit.state.generation,
            TransitionSource::component("h10-adversarial-test"),
            TransitionPayload::DecisionRecorded {
                decision: forged_final,
            },
        );
        let forged_final_error = store
            .commit_transition(forged_final_pending)
            .expect_err("forged final review Decision must fail semantic comparison");
        assert!(
            forged_final_error.contains("authority_decision_basis_mismatch"),
            "{forged_final_error}"
        );
        let (final_decision, final_commit) = store
            .derive_and_commit_policy_review_decision(
                case_id,
                &operation.operation_id,
                &expected_review.review_id,
                &action.action_id,
            )
            .expect("derive and atomically commit canonical final review Decision");
        assert_eq!(
            final_decision.outcome,
            crate::effect::DecisionOutcome::Allow
        );
        assert_eq!(final_commit.state.generation, 13);
        assert!(final_commit.state.grants.is_empty());
        assert!(store.verify_case_state(case_id).expect("historical replay"));
        println!(
            "h10_review_rederivation: caller_evidence={} forged_request={} wrong_reviewer={} forged_final={} canonical_final=true crash_c3_c4=true",
            caller_claim_error, forged_review_error, wrong_reviewer_error, forged_final_error
        );
        drop(store);
        fs::remove_dir_all(path).expect("remove store");
    }

    #[test]
    fn wave10_policy_mutation_between_decision_and_grant_fails_in_same_transaction() {
        let path = temp_store_path("wave10-stale-basis");
        let store = LmdbRecordStore::open(&path).expect("open store");
        let case_id = "case:wave10-stale-basis";
        let proposer = "participant:model";
        let reviewer = "participant:reviewer";
        open_policy_case(&store, case_id);
        commit_typed(
            &store,
            "transition:wave10-proposer",
            case_id,
            1,
            TransitionPayload::ParticipantBound {
                participant_id: proposer.to_string(),
                role: "operation-proposer".to_string(),
            },
            None,
            vec![],
        );
        commit_typed(
            &store,
            "transition:wave10-reviewer",
            case_id,
            2,
            TransitionPayload::ParticipantBound {
                participant_id: reviewer.to_string(),
                role: "operation-reviewer".to_string(),
            },
            None,
            vec![],
        );
        let resource = ResourceAttachmentState {
            attachment_id: "workspace".to_string(),
            kind: ResourceKind::Filesystem,
            allowed_write_prefix: "allowed".to_string(),
            max_write_bytes: 128,
            policy_id: "policy:legacy-inert".to_string(),
            policy_owner_participant_id: reviewer.to_string(),
            review_requirement: ReviewRequirement::Automatic,
            process_signal_actions: Vec::new(),
        };
        commit_typed(
            &store,
            "transition:wave10-resource",
            case_id,
            3,
            TransitionPayload::ResourceAttached {
                attachment: resource.clone(),
            },
            None,
            vec![reviewer.to_string()],
        );
        let v1 = publish_compilation(
            &store,
            &policy_with_rules("organization:acme", "authority", "1", "allow", false),
        );
        store
            .bind_case_policy(
                case_id,
                &v1.artifact.artifact_id,
                4,
                "participant:operator",
                "bind E1",
            )
            .expect("bind E1");
        commit_typed(
            &store,
            "transition:wave10-provider",
            case_id,
            5,
            TransitionPayload::ProviderAttached {
                participant_id: proposer.to_string(),
                provider_id: "provider:wave10".to_string(),
                provider_kind: "openai_compatible".to_string(),
                base_url: "http://127.0.0.1:1".to_string(),
                model_id: "model:wave10".to_string(),
                credential_ref: "env:TEST".to_string(),
            },
            None,
            vec![],
        );
        let lineage = test_provider_lineage(6);
        commit_typed(
            &store,
            "transition:wave10-invocation",
            case_id,
            6,
            TransitionPayload::ProviderInvocationStarted {
                invocation_id: "invocation:wave10".to_string(),
                participant_id: proposer.to_string(),
                provider_id: "provider:wave10".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:wave10".to_string(),
                semantic_lineage: Some(lineage.clone()),
            },
            None,
            vec![],
        );
        commit_typed(
            &store,
            "transition:wave10-result",
            case_id,
            7,
            TransitionPayload::ProviderResultRecorded {
                result_id: "provider-result:wave10".to_string(),
                invocation_id: "invocation:wave10".to_string(),
                provider_id: "provider:wave10".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:wave10".to_string(),
                semantic_lineage: Some(lineage),
                output: "policy-bound proposal".to_string(),
            },
            None,
            vec!["invocation:wave10".to_string()],
        );
        let operation = normalize_filesystem_write_candidate(
            r#"{"schema":"yai.operation_proposal.filesystem_write.v1","operation":"filesystem.write","resource":"workspace","path":"allowed/stale.txt","content":"stale"}"#,
            &NormalizationContext {
                case_id,
                participant_id: proposer,
                provider_result_id: "provider-result:wave10",
                provider_invocation_id: "invocation:wave10",
                case_generation: 8,
                resource: &resource,
            },
        )
        .expect("normalize operation");
        commit_typed(
            &store,
            "transition:wave10-operation",
            case_id,
            8,
            TransitionPayload::OperationRecorded {
                operation: operation.clone(),
            },
            Some(operation.scope.clone()),
            operation.origin.causal_refs(),
        );
        let (decision, decision_commit) = store
            .derive_and_commit_policy_decision(case_id, &operation.operation_id)
            .expect("derive and atomically commit E1 Decision");
        assert_eq!(decision.outcome, crate::effect::DecisionOutcome::Allow);
        let basis = decision.decision_basis.as_ref().expect("basis");
        let grant =
            issue_policy_execution_grant(&operation, &decision, decision_commit.state.generation)
                .expect("build E1 grant before intervening authority state");
        let mut forged_grant = grant.clone();
        forged_grant.policy_artifact_refs = vec!["policy-artifact:forged".to_string()];
        let forged_grant = reseal_policy_execution_grant_for_test(forged_grant);
        forged_grant
            .validate_integrity()
            .expect("forged Grant remains content-valid");
        let mut forged_grant_transition = PendingTransition::new(
            "transition:h10-forged-grant",
            case_id,
            decision_commit.state.generation,
            TransitionSource::component("h10-adversarial-test"),
            TransitionPayload::ExecutionGrantIssued {
                grant: forged_grant,
            },
        );
        forged_grant_transition.scope = Some(operation.scope.clone());
        let forged_grant_error = store
            .commit_transition(forged_grant_transition)
            .expect_err("content-valid forged Grant must fail semantic comparison");
        assert!(
            forged_grant_error.contains("policy_execution_grant_semantic_mismatch"),
            "{forged_grant_error}"
        );
        let role_change = commit_typed(
            &store,
            "transition:h10-intervening-role",
            case_id,
            decision_commit.state.generation,
            TransitionPayload::ParticipantBound {
                participant_id: reviewer.to_string(),
                role: "additional-authority-context".to_string(),
            },
            None,
            vec![reviewer.to_string()],
        );
        let mut role_stale_grant = PendingTransition::new(
            "transition:h10-role-stale-grant",
            case_id,
            role_change.state.generation,
            TransitionSource::component("h10-adversarial-test"),
            TransitionPayload::ExecutionGrantIssued {
                grant: grant.clone(),
            },
        );
        role_stale_grant.scope = Some(operation.scope.clone());
        role_stale_grant.causal_refs = vec![
            operation.operation_id.clone(),
            decision.decision_id.clone(),
            basis.basis_id.clone(),
        ];
        let role_stale_error = store
            .commit_transition(role_stale_grant)
            .expect_err("intervening role state must invalidate Decision adjacency");
        assert!(
            role_stale_error.contains("policy_grant_decision_not_adjacent"),
            "{role_stale_error}"
        );
        let v2 = publish_compilation(
            &store,
            &policy_with_rules("organization:acme", "authority", "2", "deny", false),
        );
        let prior_binding = decision_commit.state.policy_bindings[0].binding_id.clone();
        let replaced = store
            .replace_case_policy(
                case_id,
                &prior_binding,
                &v2.artifact.artifact_id,
                role_change.state.generation,
                "participant:operator",
                "replace E1 with E2 before Grant",
            )
            .expect("replace with E2");
        let state_after_replace = replaced.commit.as_ref().unwrap().state.clone();
        let mut forged_effective = replaced
            .status
            .effective_policy
            .clone()
            .expect("current E2");
        for rule in &mut forged_effective.rules {
            if let crate::case_policy::EffectivePolicyRule::OperationRestriction {
                effect, ..
            } = rule
            {
                *effect = crate::governance::PolicyEffect::Allow;
            }
        }
        let forged_allow = evaluate_filesystem_admission(
            &operation,
            &state_after_replace,
            &resource,
            &forged_effective,
            &CanonicalEvidenceResolution::default(),
            &test_temporal_context(),
        )
        .expect("forge self-consistent ALLOW basis over E2 identity");
        assert_eq!(forged_allow.outcome, crate::effect::DecisionOutcome::Allow);
        let forged_basis = forged_allow.decision_basis.as_ref().unwrap();
        let forged_basis_id = forged_basis.basis_id.clone();
        let forged_effective_policy_id = forged_basis.effective_policy_id.clone();
        let mut forged_transition = PendingTransition::new(
            "transition:h10-forged-allow",
            case_id,
            state_after_replace.generation,
            TransitionSource::component("h10-adversarial-test"),
            TransitionPayload::DecisionRecorded {
                decision: forged_allow,
            },
        );
        forged_transition.scope = Some(operation.scope.clone());
        forged_transition.causal_refs = vec![
            operation.operation_id.clone(),
            forged_basis_id,
            forged_effective_policy_id,
        ];
        let forged_error = store
            .commit_transition(forged_transition)
            .expect_err("content-valid forged ALLOW must fail semantic re-derivation");
        assert!(
            forged_error.contains("authority_decision_basis_mismatch"),
            "{forged_error}"
        );
        let mut stale_grant = PendingTransition::new(
            "transition:wave10-stale-grant",
            case_id,
            replaced.commit.as_ref().unwrap().state.generation,
            TransitionSource::component("wave10-test"),
            TransitionPayload::ExecutionGrantIssued {
                grant: grant.clone(),
            },
        );
        stale_grant.scope = Some(operation.scope.clone());
        stale_grant.causal_refs = vec![
            operation.operation_id.clone(),
            decision.decision_id.clone(),
            basis.basis_id.clone(),
            basis.effective_policy_id.clone(),
        ];
        let error = store
            .commit_transition(stale_grant)
            .expect_err("E1 Grant must not commit after E2 binding");
        assert!(
            error.contains("policy_grant_decision_not_adjacent"),
            "{error}"
        );
        assert!(store
            .get_case_state(case_id)
            .unwrap()
            .unwrap()
            .grants
            .is_empty());
        assert!(store.verify_case_state(case_id).expect("replay after race"));
        println!(
            "h10_authority_injection: forged_decision={} forged_grant={} role_stale_grant={} policy_stale_grant={} decision_basis={} evaluated_effective_policy={} current_effective_policy={} grant_committed=false",
            forged_error,
            forged_grant_error,
            role_stale_error,
            error,
            basis.basis_id,
            basis.effective_policy_id,
            replaced.status.effective_policy.unwrap().effective_policy_id,
        );
        drop(store);
        fs::remove_dir_all(path).expect("remove store");
    }

    #[test]
    fn wave11_policy_time_postures_revoke_stale_refresh_and_clock_floor_contract() {
        let path = temp_store_path("wave11-policy-time");
        let store = LmdbRecordStore::open(&path).expect("open store");
        let case_id = "case:wave11-policy-time";
        let now = authority_wall_time_unix_ms();
        open_policy_case(&store, case_id);
        let p1 = publish_compilation(
            &store,
            &wave11_bounded_policy(
                "temporal-authority",
                "1",
                "allow",
                now.saturating_sub(1_000),
                now + 10_000,
                now + 20_000,
            ),
        );
        let bound = store
            .bind_case_policy(
                case_id,
                &p1.artifact.artifact_id,
                1,
                "participant:operator",
                "bind bounded P1",
            )
            .expect("bind bounded P1");
        assert_eq!(bound.status.validity, PolicyValidityPosture::Valid);
        assert_eq!(
            store
                .case_policy_status_at(case_id, now + 10_000)
                .expect("refresh status")
                .validity,
            PolicyValidityPosture::RefreshRequired
        );
        assert_eq!(
            store
                .case_policy_status_at(case_id, now + 20_000)
                .expect("expiry status")
                .validity,
            PolicyValidityPosture::Expired
        );
        let p2 = publish_compilation(
            &store,
            &wave11_bounded_policy(
                "temporal-authority",
                "2",
                "deny",
                now.saturating_sub(1_000),
                now + 40_000,
                now + 50_000,
            ),
        );
        let stale = store
            .case_policy_status_at(case_id, now)
            .expect("P1 stale status");
        assert_eq!(stale.validity, PolicyValidityPosture::Stale);
        assert_eq!(
            stale
                .effective_policy
                .as_ref()
                .expect("P1 effective policy")
                .artifact_ids,
            vec![p1.artifact.artifact_id.clone()]
        );
        let prior = stale
            .effective_policy
            .as_ref()
            .expect("P1 effective policy")
            .binding_ids[0]
            .clone();
        let refreshed = store
            .replace_case_policy(
                case_id,
                &prior,
                &p2.artifact.artifact_id,
                2,
                "participant:operator",
                "explicitly replace P1 with P2",
            )
            .expect("replace P1 with P2");
        assert_eq!(refreshed.status.validity, PolicyValidityPosture::Valid);
        let future = publish_compilation(
            &store,
            &wave11_bounded_policy(
                "future-independent-lineage",
                "1",
                "allow",
                now + 100_000,
                now + 110_000,
                now + 120_000,
            ),
        );
        let with_future = store
            .bind_case_policy(
                case_id,
                &future.artifact.artifact_id,
                refreshed.commit.as_ref().unwrap().state.generation,
                "participant:operator",
                "bind future independent lineage",
            )
            .expect("bind not-yet-valid policy");
        assert_eq!(
            with_future.status.validity,
            PolicyValidityPosture::NotYetValid
        );
        store
            .retire_policy_artifact(
                &future.artifact.artifact_id,
                "participant:policy-admin",
                "withdraw future catalog entry",
            )
            .expect("retire future policy");
        assert_eq!(
            store.case_policy_status(case_id).unwrap().validity,
            PolicyValidityPosture::Stale
        );
        let floor = advance_authority_floor(&store, now + 50_000);
        let rollback = store
            .case_policy_status_at(case_id, now)
            .expect("rollback status");
        assert_eq!(rollback.persisted_authority_floor_unix_ms, floor);
        assert_eq!(rollback.authority_time_unix_ms, floor);
        assert_eq!(rollback.validity, PolicyValidityPosture::Expired);
        let revoked = store
            .revoke_policy_artifact(
                &p2.artifact.artifact_id,
                "participant:policy-admin",
                "withdraw Wave11 test authority",
            )
            .expect("revoke P2");
        assert_eq!(revoked.view.lifecycle, PolicyLifecycleState::Revoked);
        let revoked_status = store.case_policy_status(case_id).expect("revoked status");
        assert_eq!(revoked_status.validity, PolicyValidityPosture::Revoked);
        let event_count = revoked.view.lifecycle_events.len();
        let repeated = store
            .revoke_policy_artifact(
                &p2.artifact.artifact_id,
                "participant:policy-admin",
                "same terminal revoke",
            )
            .expect("idempotent revoke");
        assert!(!repeated.changed);
        assert_eq!(repeated.view.lifecycle_events.len(), event_count);
        println!(
            "wave11_temporal_governance: p1={} p2={} future={} valid=true not_yet_valid=true refresh_required=true expired=true rollback_floor={} stale_pinned_to_p1=true retired_stale=true weakest_composition=true explicit_refresh=true revoke_event={} revoked=true",
            p1.artifact.artifact_id,
            p2.artifact.artifact_id,
            future.artifact.artifact_id,
            floor,
            revoked
                .view
                .lifecycle_events
                .last()
                .expect("revoke event")
                .event_id
        );
        drop(store);
        fs::remove_dir_all(path).expect("remove store");
    }

    #[test]
    fn wave11_revoked_review_is_durably_invalidated_and_cannot_approve() {
        let path = temp_store_path("wave11-review-revoke");
        let store = LmdbRecordStore::open(&path).expect("open store");
        let case_id = "case:wave11-review-revoke";
        let (_resource, operation) = setup_h10_authority_case(&store, case_id, "allow", true);
        let (decision, decision_commit) = store
            .derive_and_commit_policy_decision(case_id, &operation.operation_id)
            .expect("derive and atomically commit review Decision");
        let review =
            build_policy_review_request(&operation, &decision, decision_commit.state.generation)
                .expect("build review");
        let review_commit = commit_typed(
            &store,
            "transition:wave11-review-request",
            case_id,
            decision_commit.state.generation,
            TransitionPayload::ReviewRequested {
                review: review.clone(),
            },
            Some(operation.scope.clone()),
            vec![
                operation.operation_id.clone(),
                decision.decision_id.clone(),
                review.decision_basis_id.clone(),
                review.effective_policy_id.clone(),
            ],
        );
        let artifact_id = review_commit.state.policy_bindings[0].artifact_id.clone();
        store
            .revoke_policy_artifact(
                &artifact_id,
                "participant:policy-admin",
                "invalidate pending review",
            )
            .expect("revoke review policy");
        let invalidated = store
            .invalidate_review_if_policy_unusable(case_id, &review.review_id)
            .expect("derive review invalidation")
            .expect("invalidation committed");
        assert!(matches!(
            invalidated.transition.payload,
            TransitionPayload::ReviewInvalidated { .. }
        ));
        let action = build_review_action(
            &review,
            case_id,
            "participant:reviewer",
            ReviewActionKind::Approve,
            "must not revive revoked policy",
            invalidated.state.generation,
            "wave11-test",
        )
        .expect("build structurally valid action");
        let approval_error = store
            .commit_transition(PendingTransition::new(
                "transition:wave11-invalid-review-approval",
                case_id,
                invalidated.state.generation,
                TransitionSource::component("wave11-test"),
                TransitionPayload::ReviewActionRecorded { action },
            ))
            .expect_err("invalidated review cannot approve");
        assert!(approval_error.contains("review_action_binding_or_generation_mismatch"));
        let state = store.get_case_state(case_id).unwrap().unwrap();
        assert_eq!(state.reviews[0].status, ReviewResolution::Invalidated);
        assert!(state.grants.is_empty());
        assert!(store.verify_case_state(case_id).expect("review replay"));
        println!(
            "wave11_review_invalidation: review={} artifact={} invalidation_transition={} approval_error={} grants=0 replay=true",
            review.review_id,
            artifact_id,
            invalidated.transition.transition_id,
            approval_error
        );
        drop(store);
        fs::remove_dir_all(path).expect("remove store");
    }

    #[test]
    fn wave11_grant_expiry_before_prepare_is_terminal_and_effect_free() {
        let path = temp_store_path("wave11-grant-expiry");
        let root = temp_store_path("wave11-grant-expiry-files");
        fs::create_dir_all(root.join("allowed")).expect("create root");
        let store = LmdbRecordStore::open(&path).expect("open store");
        let case_id = "case:wave11-grant-expiry";
        let (resource, operation) = setup_h10_authority_case(&store, case_id, "allow", false);
        let (decision, decision_commit) = store
            .derive_and_commit_policy_decision(case_id, &operation.operation_id)
            .expect("derive and atomically commit Decision");
        let basis = decision.decision_basis.as_ref().expect("basis");
        let grant =
            issue_policy_execution_grant(&operation, &decision, decision_commit.state.generation)
                .expect("issue finite Grant");
        assert!(grant.expires_at_unix_ms > grant.issued_at_unix_ms);
        let grant_commit = commit_typed(
            &store,
            "transition:wave11-expiry-grant",
            case_id,
            decision_commit.state.generation,
            TransitionPayload::ExecutionGrantIssued {
                grant: grant.clone(),
            },
            Some(operation.scope.clone()),
            vec![
                operation.operation_id.clone(),
                decision.decision_id.clone(),
                basis.basis_id.clone(),
                basis.effective_policy_id.clone(),
            ],
        );
        advance_authority_floor(&store, grant.expires_at_unix_ms);
        let binding = LocalFilesystemBinding::new(case_id, "workspace", &root).unwrap();
        let pre = observe_filesystem(
            &binding,
            &resource,
            &operation.filesystem_write.relative_path,
            "observation:wave11-expiry-pre",
        );
        let prepared = prepare_effect(&operation, &decision, &grant, pre).expect("build PREPARE");
        let mut pending = PendingTransition::new(
            "transition:wave11-expired-prepare",
            case_id,
            grant_commit.state.generation,
            TransitionSource::component("wave11-test"),
            TransitionPayload::EffectPrepared {
                prepared: prepared.clone(),
            },
        );
        pending.scope = Some(operation.scope.clone());
        pending.causal_refs = vec![grant.grant_id.clone(), prepared.effect_id.clone()];
        let invalidation = match store
            .commit_effect_prepared(pending)
            .expect("temporal PREPARE admission")
        {
            PreparedCommitOutcome::GrantInvalidated(commit) => commit,
            PreparedCommitOutcome::Prepared(_) => panic!("expired Grant reached PREPARE"),
        };
        let state = invalidation.state;
        assert_eq!(state.grants[0].status, GrantLifecycle::Expired);
        assert!(state.effects.is_empty());
        assert!(store
            .verify_case_state(case_id)
            .expect("expired Grant replay"));
        println!(
            "wave11_grant_expiry: grant={} issued_at={} expires_at={} authority_floor={} invalidation_transition={} prepare=false effects=0",
            grant.grant_id,
            grant.issued_at_unix_ms,
            grant.expires_at_unix_ms,
            store.case_policy_status(case_id).unwrap().persisted_authority_floor_unix_ms,
            invalidation.transition.transition_id
        );
        drop(store);
        fs::remove_dir_all(path).expect("remove store");
        fs::remove_dir_all(root).expect("remove root");
    }

    #[test]
    fn wave11_prepare_is_nonretroactive_cut_across_revoke_and_cancel() {
        let path = temp_store_path("wave11-prepare-cut");
        let root = temp_store_path("wave11-prepare-cut-files");
        fs::create_dir_all(root.join("allowed")).expect("create root");
        let store = LmdbRecordStore::open(&path).expect("open store");
        let case_id = "case:wave11-prepare-cut";
        let (resource, operation) = setup_h10_authority_case(&store, case_id, "allow", false);
        let (decision, decision_commit) = store
            .derive_and_commit_policy_decision(case_id, &operation.operation_id)
            .expect("derive and atomically commit Decision");
        let basis = decision.decision_basis.as_ref().expect("basis");
        let grant =
            issue_policy_execution_grant(&operation, &decision, decision_commit.state.generation)
                .expect("issue Grant");
        let grant_commit = commit_typed(
            &store,
            "transition:wave11-cut-grant",
            case_id,
            decision_commit.state.generation,
            TransitionPayload::ExecutionGrantIssued {
                grant: grant.clone(),
            },
            Some(operation.scope.clone()),
            vec![
                operation.operation_id.clone(),
                decision.decision_id.clone(),
                basis.basis_id.clone(),
                basis.effective_policy_id.clone(),
            ],
        );
        let binding = LocalFilesystemBinding::new(case_id, "workspace", &root).unwrap();
        let pre = observe_filesystem(
            &binding,
            &resource,
            &operation.filesystem_write.relative_path,
            "observation:wave11-cut-pre",
        );
        let prepared = prepare_effect(&operation, &decision, &grant, pre).expect("prepare effect");
        let mut prepare_pending = PendingTransition::new(
            "transition:wave11-cut-prepare",
            case_id,
            grant_commit.state.generation,
            TransitionSource::component("wave11-test"),
            TransitionPayload::EffectPrepared {
                prepared: prepared.clone(),
            },
        );
        prepare_pending.scope = Some(operation.scope.clone());
        prepare_pending.causal_refs = vec![
            operation.operation_id.clone(),
            decision.decision_id.clone(),
            grant.grant_id.clone(),
            prepared.expected_pre_observation.observation_id.clone(),
        ];
        let prepared_commit = match store
            .commit_effect_prepared(prepare_pending)
            .expect("commit PREPARE before contraction")
        {
            PreparedCommitOutcome::Prepared(commit) => commit,
            PreparedCommitOutcome::GrantInvalidated(_) => panic!("valid Grant invalidated"),
        };
        let artifact_id = prepared_commit.state.policy_bindings[0].artifact_id.clone();
        store
            .revoke_policy_artifact(
                &artifact_id,
                "participant:policy-admin",
                "revoke after PREPARE",
            )
            .expect("revoke after PREPARE");
        let cancelled = store
            .cancel_case(case_id, "participant:operator", "cancel after PREPARE")
            .expect("cancel after PREPARE");
        assert_eq!(cancelled.abandoned_grants, 0);
        assert_eq!(cancelled.state.grants[0].status, GrantLifecycle::Prepared);
        let prepared_close_error = store
            .close_case(case_id, "participant:operator", "unsafe prepared close")
            .expect_err("PREPARE must block closure");
        assert!(prepared_close_error.contains("unresolved_effect"));
        let crashed = execute_filesystem_write(
            &operation,
            &decision,
            &grant,
            &prepared,
            &cancelled.state,
            &binding,
            &resource,
            CarrierFailpoint::CrashAfterVisibleEffect,
        )
        .expect("visible carrier effect after PREPARE cut");
        assert_eq!(crashed.outcome, EffectOutcome::Indeterminate);
        let indeterminate = commit_typed(
            &store,
            "transition:wave11-cut-indeterminate",
            case_id,
            cancelled.state.generation,
            TransitionPayload::EffectIndeterminate {
                effect_id: prepared.effect_id.clone(),
                reason: "carrier returned after visible mutation before observation".to_string(),
                observation: None,
            },
            None,
            vec![prepared.effect_id.clone()],
        );
        let indeterminate_close_error = store
            .close_case(
                case_id,
                "participant:operator",
                "unsafe indeterminate close",
            )
            .expect_err("indeterminate effect must block closure");
        assert!(indeterminate_close_error.contains("unresolved_effect"));
        let observed = observe_filesystem(
            &binding,
            &resource,
            &prepared.relative_path,
            "observation:wave11-cut-reconcile",
        );
        assert_eq!(
            classify_reconciliation(&prepared, &observed),
            ReconciliationConclusion::EffectObserved
        );
        let reconciled_result = CarrierResult {
            outcome: EffectOutcome::Applied,
            post_observation: observed.clone(),
            carrier_attempted: true,
            mutation_performed: true,
            crash_injected_after_effect: false,
            detail: "reconciled intended post-state".to_string(),
        };
        let receipt = build_effect_receipt(&prepared, &reconciled_result);
        let reconciled = commit_typed(
            &store,
            "transition:wave11-cut-reconcile",
            case_id,
            indeterminate.state.generation,
            TransitionPayload::EffectReconciled {
                effect_id: prepared.effect_id.clone(),
                conclusion: ReconciliationConclusion::EffectObserved,
                observation: observed,
                receipt: Some(receipt.clone()),
            },
            Some(operation.scope.clone()),
            vec![prepared.effect_id.clone(), receipt.receipt_id.clone()],
        );
        assert_eq!(reconciled.state.grants[0].status, GrantLifecycle::Finalized);
        assert_eq!(
            reconciled.state.effects[0].status,
            crate::transition::EffectLifecycle::Finalized
        );
        let closed = store
            .close_case(
                case_id,
                "participant:operator",
                "close after reconciliation",
            )
            .expect("close after reconciliation");
        assert_eq!(closed.state.lifecycle, CaseLifecycle::Closed);
        assert!(root.join("allowed/h10.txt").exists());
        assert!(store
            .verify_case_state(case_id)
            .expect("PREPARE cut replay"));
        println!(
            "wave11_prepare_cut: grant={} prepare_generation={} revoke_after_prepare=true cancellation_generation={} prepared_close={} indeterminate_generation={} indeterminate_close={} reconcile_generation={} close_generation={} receipt={} effect_truth_preserved=true",
            grant.grant_id,
            prepared_commit.state.generation,
            cancelled.state.generation,
            prepared_close_error,
            indeterminate.state.generation,
            indeterminate_close_error,
            reconciled.state.generation,
            closed.state.generation,
            receipt.receipt_id
        );
        drop(store);
        fs::remove_dir_all(path).expect("remove store");
        fs::remove_dir_all(root).expect("remove root");
    }

    #[test]
    fn wave11_cancellation_and_closure_are_atomic_terminal_and_replayable() {
        let path = temp_store_path("wave11-case-terminal");
        let store = LmdbRecordStore::open(&path).expect("open store");
        let case_id = "case:wave11-terminal";
        open_policy_case(&store, case_id);
        let cancelled = store
            .cancel_case(case_id, "participant:operator", "operator requested stop")
            .expect("cancel Case");
        assert!(cancelled.changed);
        assert!(cancelled.state.cancellation.is_some());
        let repeated = store
            .cancel_case(case_id, "participant:operator", "different later reason")
            .expect("idempotent cancel");
        assert!(!repeated.changed);
        assert_eq!(repeated.state.generation, cancelled.state.generation);
        let blocked = store
            .commit_transition(pending(
                "transition:wave11-post-cancel-operation",
                case_id,
                cancelled.state.generation,
                TransitionPayload::ParticipantBound {
                    participant_id: "participant:late".to_string(),
                    role: "operation-proposer".to_string(),
                },
            ))
            .expect_err("cancelled Case must reject new canonical work");
        assert!(blocked.contains("case_cancelled_write_barrier"));
        let closed = store
            .close_case(case_id, "participant:operator", "terminal safe closure")
            .expect("close Case");
        assert!(closed.changed);
        assert_eq!(closed.state.lifecycle, CaseLifecycle::Closed);
        let repeated_close = store
            .close_case(case_id, "participant:operator", "repeat close")
            .expect("idempotent close");
        assert!(!repeated_close.changed);
        assert_eq!(repeated_close.state.generation, closed.state.generation);
        let closed_error = store
            .commit_transition(pending(
                "transition:wave11-post-close",
                case_id,
                closed.state.generation,
                TransitionPayload::ParticipantBound {
                    participant_id: "participant:never".to_string(),
                    role: "operation-proposer".to_string(),
                },
            ))
            .expect_err("closed Case write barrier");
        assert!(closed_error.contains("case_closed_write_barrier"));
        let replay = store.replay_case_state(case_id).expect("terminal replay");
        assert_eq!(replay, closed.state);
        assert!(store.verify_case_state(case_id).expect("terminal verify"));
        println!(
            "wave11_case_terminal: cancellation_transition={} cancellation_generation={} close_transition={} close_generation={} cancel_idempotent=true close_idempotent=true post_cancel={} post_close={} replay_closed=true",
            cancelled.commits[0].transition.transition_id,
            cancelled.commits[0].transition.sequence,
            closed.commit.as_ref().unwrap().transition.transition_id,
            closed.commit.as_ref().unwrap().transition.sequence,
            blocked,
            closed_error
        );
        drop(store);
        let reopened = LmdbRecordStore::open(&path).expect("reopen terminal store");
        assert_eq!(
            reopened.get_case_state(case_id).unwrap().unwrap().lifecycle,
            CaseLifecycle::Closed
        );
        drop(reopened);
        fs::remove_dir_all(path).expect("remove store");
    }

    #[test]
    fn wave12_kernel_principals_tenants_and_case_links_are_isolated_and_restart_safe() {
        let path = temp_store_path("wave12-security-domains");
        let store = LmdbRecordStore::open(&path).expect("open security store");
        let owner_a = AuthenticatedPrincipal::for_test(12001);
        let owner_b = AuthenticatedPrincipal::for_test(12002);
        let bootstrap_a = store
            .bootstrap_local_security(
                &owner_a,
                "tenant:wave12-a",
                "organization:shared",
                1_200_001,
            )
            .expect("bootstrap Tenant A");
        let bootstrap_b = store
            .bootstrap_local_security(
                &owner_b,
                "tenant:wave12-b",
                "organization:shared",
                1_200_002,
            )
            .expect("bootstrap Tenant B");
        assert_ne!(
            bootstrap_a.principal.principal_id,
            bootstrap_b.principal.principal_id
        );
        assert!(
            !store
                .bootstrap_local_security(
                    &owner_a,
                    "tenant:wave12-a",
                    "organization:shared",
                    1_200_001,
                )
                .expect("exact bootstrap is idempotent")
                .created
        );
        assert!(store
            .bootstrap_local_security(
                &owner_b,
                "tenant:wave12-a",
                "organization:shared",
                1_200_003,
            )
            .unwrap_err()
            .contains("unsafe_duplicate_tenant_bootstrap"));

        let case_a = store
            .create_tenant_case(&owner_a, "tenant:wave12-a", "case:wave12-a")
            .expect("create Tenant A Case");
        let case_b = store
            .create_tenant_case(&owner_b, "tenant:wave12-b", "case:wave12-b")
            .expect("create Tenant B Case");
        assert_eq!(case_a.state.tenant_id.as_deref(), Some("tenant:wave12-a"));
        assert_eq!(case_b.state.tenant_id.as_deref(), Some("tenant:wave12-b"));
        assert_eq!(
            store
                .get_case_state_authorized(&owner_a, "case:wave12-b")
                .unwrap_err(),
            "case_not_visible"
        );
        assert_eq!(
            store
                .get_case_state_authorized(&owner_b, "case:wave12-a")
                .unwrap_err(),
            "case_not_visible"
        );

        store
            .add_tenant_member(
                &owner_a,
                "tenant:wave12-a",
                &bootstrap_b.principal.principal_id,
                1_200_004,
            )
            .expect("owner adds enrolled member");
        assert_eq!(
            store
                .resolve_security_context(&owner_b, "tenant:wave12-a")
                .expect("member context")
                .membership(),
            &TenantMembershipKind::Member
        );
        let principal_a = bootstrap_a.principal.principal_id.clone();
        let participant = store
            .commit_secured_transition(
                &owner_a,
                "tenant:wave12-a",
                secured_pending(
                    "transition:wave12-human",
                    "case:wave12-a",
                    case_a.state.generation,
                    &principal_a,
                    TransitionPayload::ParticipantBound {
                        participant_id: "participant:human".to_string(),
                        role: "operation-reviewer".to_string(),
                    },
                ),
                true,
            )
            .expect("owner binds Case Participant");
        let link = crate::transition::PrincipalParticipantLink::new(
            "case:wave12-a",
            "tenant:wave12-a",
            &bootstrap_b.principal.principal_id,
            "participant:human",
            &principal_a,
            1_200_005,
        )
        .expect("build Principal Participant link");
        let mut link_pending = secured_pending(
            "transition:wave12-human-link",
            "case:wave12-a",
            participant.state.generation,
            &principal_a,
            TransitionPayload::ParticipantPrincipalLinked { link: link.clone() },
        );
        link_pending.causal_refs = vec![
            bootstrap_b.principal.principal_id.clone(),
            "participant:human".to_string(),
        ];
        let linked = store
            .commit_secured_transition(&owner_a, "tenant:wave12-a", link_pending, true)
            .expect("link member Principal to Participant");
        assert_eq!(linked.state.principal_participant_links, vec![link]);
        assert_eq!(
            linked.state.participants[0].roles,
            vec!["operation-reviewer".to_string()]
        );

        let member_admin = store
            .commit_secured_transition(
                &owner_b,
                "tenant:wave12-a",
                secured_pending(
                    "transition:wave12-member-admin",
                    "case:wave12-a",
                    linked.state.generation,
                    &bootstrap_b.principal.principal_id,
                    TransitionPayload::ParticipantBound {
                        participant_id: "participant:forbidden".to_string(),
                        role: "admin".to_string(),
                    },
                ),
                true,
            )
            .expect_err("Tenant member is not an administrator");
        assert_eq!(member_admin, "tenant_owner_required");
        let injected = store
            .commit_transition(secured_pending(
                "transition:wave12-low-level-injection",
                "case:wave12-a",
                linked.state.generation,
                &principal_a,
                TransitionPayload::ParticipantBound {
                    participant_id: "participant:forged".to_string(),
                    role: "admin".to_string(),
                },
            ))
            .expect_err("Principal strings cannot bypass canonical security");
        assert_eq!(injected, "authenticated_tenant_owner_required");

        assert_eq!(
            store
                .cancel_tenant_case(&owner_b, "case:wave12-a", "member cannot cancel")
                .expect_err("Tenant member cannot cancel"),
            "tenant_owner_required"
        );
        let cancellation = store
            .cancel_tenant_case(&owner_a, "case:wave12-a", "owner cancellation")
            .expect("Tenant owner cancels Case");
        assert_eq!(
            cancellation.commits[0]
                .transition
                .source
                .principal_id
                .as_deref(),
            Some(principal_a.as_str())
        );
        let closure = store
            .close_tenant_case(&owner_a, "case:wave12-a", "owner closure")
            .expect("Tenant owner closes Case");
        assert_eq!(
            closure
                .commit
                .as_ref()
                .unwrap()
                .transition
                .source
                .principal_id
                .as_deref(),
            Some(principal_a.as_str())
        );
        assert_eq!(closure.state.lifecycle, CaseLifecycle::Closed);
        assert_eq!(
            store
                .commit_secured_transition(
                    &owner_a,
                    "tenant:wave12-a",
                    secured_pending(
                        "transition:wave12-post-close",
                        "case:wave12-a",
                        closure.state.generation,
                        &principal_a,
                        TransitionPayload::ParticipantBound {
                            participant_id: "participant:post-close".to_string(),
                            role: "forbidden".to_string(),
                        },
                    ),
                    true,
                )
                .expect_err("Tenant owner cannot mutate a closed Case"),
            "case_closed_write_barrier"
        );

        drop(store);
        let reopened = LmdbRecordStore::open(&path).expect("reopen security store");
        let replay = reopened
            .replay_case_state("case:wave12-a")
            .expect("replay tenant Case");
        assert_eq!(replay.tenant_id.as_deref(), Some("tenant:wave12-a"));
        assert_eq!(replay.principal_participant_links.len(), 1);
        println!(
            "wave12_security_domains: principal_a={} principal_b={} tenant_a={} tenant_b={} case_a={} case_b={} link={} cross_tenant_read=denied member_admin={} low_level_injection={} cancellation_actor={} closure_actor={} owner_cannot_reopen_closed=true restart_tenant={} organization_projection_shared_without_cross_access=true",
            bootstrap_a.principal.principal_id,
            bootstrap_b.principal.principal_id,
            bootstrap_a.tenant.tenant_id,
            bootstrap_b.tenant.tenant_id,
            case_a.state.case_id,
            case_b.state.case_id,
            replay.principal_participant_links[0].link_id,
            member_admin,
            injected,
            cancellation.commits[0]
                .transition
                .source
                .principal_id
                .as_deref()
                .unwrap(),
            closure
                .commit
                .as_ref()
                .unwrap()
                .transition
                .source
                .principal_id
                .as_deref()
                .unwrap(),
            replay.tenant_id.as_deref().unwrap()
        );
        drop(reopened);
        fs::remove_dir_all(path).expect("remove security store");
    }

    #[test]
    fn wave12_policy_namespace_is_tenant_bound_and_cross_tenant_binding_fails_closed() {
        let path = temp_store_path("wave12-policy-isolation");
        let store = LmdbRecordStore::open(&path).expect("open policy isolation store");
        let owner_a = AuthenticatedPrincipal::for_test(12101);
        let owner_b = AuthenticatedPrincipal::for_test(12102);
        store
            .bootstrap_local_security(
                &owner_a,
                "tenant:policy-a",
                "organization:shared",
                1_210_001,
            )
            .unwrap();
        store
            .bootstrap_local_security(
                &owner_b,
                "tenant:policy-b",
                "organization:shared",
                1_210_002,
            )
            .unwrap();
        let case_a = store
            .create_tenant_case(&owner_a, "tenant:policy-a", "case:policy-a")
            .unwrap();
        let case_b = store
            .create_tenant_case(&owner_b, "tenant:policy-b", "case:policy-b")
            .unwrap();
        let source_bytes =
            policy_source_for("organization:shared", "production.security", "1", false);
        let global = compile_policy_source(&source_bytes).expect("compile global source content");
        let scoped_a =
            scope_policy_compilation(&global, "tenant:policy-a", "organization:shared").unwrap();
        let scoped_b =
            scope_policy_compilation(&global, "tenant:policy-b", "organization:shared").unwrap();
        assert_eq!(
            scoped_a.source.content_digest,
            scoped_b.source.content_digest
        );
        assert_ne!(scoped_a.artifact.artifact_id, scoped_b.artifact.artifact_id);
        assert_ne!(
            scoped_a.artifact.lineage().identity(),
            scoped_b.artifact.lineage().identity()
        );
        for (auth, tenant, compilation) in [
            (&owner_a, "tenant:policy-a", &scoped_a),
            (&owner_b, "tenant:policy-b", &scoped_b),
        ] {
            store
                .ingest_tenant_policy_compilation(auth, tenant, compilation)
                .unwrap();
            store
                .validate_tenant_policy_artifact(
                    auth,
                    &compilation.artifact.artifact_id,
                    "tenant validation",
                )
                .unwrap();
            store
                .publish_tenant_policy_artifact(
                    auth,
                    &compilation.artifact.artifact_id,
                    "tenant publication",
                )
                .unwrap();
        }
        let cross = store
            .bind_tenant_case_policy(
                &owner_a,
                "case:policy-a",
                &scoped_b.artifact.artifact_id,
                case_a.state.generation,
                "cross tenant must fail",
            )
            .expect_err("cross-Tenant binding");
        assert_eq!(cross, "cross_tenant_case_policy_binding_rejected");
        assert!(store
            .get_case_state("case:policy-a")
            .unwrap()
            .unwrap()
            .policy_bindings
            .is_empty());
        let bound_a = store
            .bind_tenant_case_policy(
                &owner_a,
                "case:policy-a",
                &scoped_a.artifact.artifact_id,
                case_a.state.generation,
                "bind Tenant A policy",
            )
            .unwrap();
        let bound_b = store
            .bind_tenant_case_policy(
                &owner_b,
                "case:policy-b",
                &scoped_b.artifact.artifact_id,
                case_b.state.generation,
                "bind Tenant B policy",
            )
            .unwrap();
        let wrong_revoke = store
            .revoke_tenant_policy_artifact(
                &owner_a,
                &scoped_b.artifact.artifact_id,
                "must not cross Tenant",
            )
            .expect_err("Tenant A cannot revoke Tenant B policy");
        assert_eq!(wrong_revoke, "tenant_not_visible");
        store
            .revoke_tenant_policy_artifact(
                &owner_a,
                &scoped_a.artifact.artifact_id,
                "Tenant A revoke",
            )
            .unwrap();
        assert_eq!(
            store
                .policy_artifact_view_authorized(&owner_b, &scoped_b.artifact.artifact_id)
                .unwrap()
                .lifecycle,
            PolicyLifecycleState::Published
        );
        println!(
            "wave12_policy_isolation: shared_source_digest={} artifact_a={} artifact_b={} lineage_a={} lineage_b={} binding_a={} binding_b={} cross_bind={} cross_revoke={} tenant_b_lifecycle=published",
            scoped_a.source.content_digest,
            scoped_a.artifact.artifact_id,
            scoped_b.artifact.artifact_id,
            scoped_a.artifact.lineage().identity(),
            scoped_b.artifact.lineage().identity(),
            bound_a
                .commit
                .as_ref()
                .unwrap()
                .state
                .policy_bindings
                .last()
                .unwrap()
                .binding_id,
            bound_b
                .commit
                .as_ref()
                .unwrap()
                .state
                .policy_bindings
                .last()
                .unwrap()
                .binding_id,
            cross,
            wrong_revoke
        );
        drop(store);
        fs::remove_dir_all(path).expect("remove policy isolation store");
    }

    #[test]
    fn wave12_cross_tenant_filesystem_roots_reject_exact_and_overlapping_aliases() {
        let path = temp_store_path("wave12-root-isolation");
        let root = temp_store_path("wave12-shared-root");
        fs::create_dir_all(root.join("nested")).expect("create root hierarchy");
        let store = LmdbRecordStore::open(&path).expect("open root isolation store");
        let owner_a = AuthenticatedPrincipal::for_test(12201);
        let owner_b = AuthenticatedPrincipal::for_test(12202);
        for (auth, tenant, organization) in [
            (&owner_a, "tenant:root-a", "organization:root-a"),
            (&owner_b, "tenant:root-b", "organization:root-b"),
        ] {
            store
                .bootstrap_local_security(auth, tenant, organization, 1_220_000)
                .unwrap();
        }
        let case_a = store
            .create_tenant_case(&owner_a, "tenant:root-a", "case:root-a")
            .unwrap();
        let case_b = store
            .create_tenant_case(&owner_b, "tenant:root-b", "case:root-b")
            .unwrap();
        let resource = |id: &str| ResourceAttachmentState {
            attachment_id: id.to_string(),
            kind: ResourceKind::Filesystem,
            allowed_write_prefix: "allowed".to_string(),
            max_write_bytes: 1024,
            policy_id: "compatibility:inert".to_string(),
            policy_owner_participant_id: "compatibility:inert".to_string(),
            review_requirement: ReviewRequirement::Automatic,
            process_signal_actions: Vec::new(),
        };
        let principal_a = owner_a.projected_principal_id();
        let principal_b = owner_b.projected_principal_id();
        let participant_a = store
            .commit_secured_transition(
                &owner_a,
                "tenant:root-a",
                secured_pending(
                    "transition:root-a-resource-participant",
                    "case:root-a",
                    case_a.state.generation,
                    &principal_a,
                    TransitionPayload::ParticipantBound {
                        participant_id: "participant:resource-a".to_string(),
                        role: "resource-compatibility-owner".to_string(),
                    },
                ),
                true,
            )
            .unwrap();
        let participant_b = store
            .commit_secured_transition(
                &owner_b,
                "tenant:root-b",
                secured_pending(
                    "transition:root-b-resource-participant",
                    "case:root-b",
                    case_b.state.generation,
                    &principal_b,
                    TransitionPayload::ParticipantBound {
                        participant_id: "participant:resource-b".to_string(),
                        role: "resource-compatibility-owner".to_string(),
                    },
                ),
                true,
            )
            .unwrap();
        let mut attach_a = secured_pending(
            "transition:root-a",
            "case:root-a",
            participant_a.state.generation,
            &principal_a,
            TransitionPayload::ResourceAttached {
                attachment: ResourceAttachmentState {
                    policy_owner_participant_id: "participant:resource-a".to_string(),
                    ..resource("workspace-a")
                },
            },
        );
        attach_a.causal_refs = vec!["participant:resource-a".to_string()];
        store
            .commit_tenant_resource_attachment(
                &owner_a,
                "tenant:root-a",
                attach_a,
                &LocalFilesystemBinding::new("case:root-a", "workspace-a", &root).unwrap(),
            )
            .expect("Tenant A attaches root");
        let mut attach_b_exact = secured_pending(
            "transition:root-b-exact",
            "case:root-b",
            participant_b.state.generation,
            &principal_b,
            TransitionPayload::ResourceAttached {
                attachment: ResourceAttachmentState {
                    policy_owner_participant_id: "participant:resource-b".to_string(),
                    ..resource("workspace-b-exact")
                },
            },
        );
        attach_b_exact.causal_refs = vec!["participant:resource-b".to_string()];
        let exact = store
            .commit_tenant_resource_attachment(
                &owner_b,
                "tenant:root-b",
                attach_b_exact,
                &LocalFilesystemBinding::new("case:root-b", "workspace-b-exact", &root).unwrap(),
            )
            .expect_err("exact root reuse across Tenants");
        assert!(exact.contains("cross_tenant_filesystem_root_overlap"));
        let mut attach_b_overlap = secured_pending(
            "transition:root-b-overlap",
            "case:root-b",
            participant_b.state.generation,
            &principal_b,
            TransitionPayload::ResourceAttached {
                attachment: ResourceAttachmentState {
                    policy_owner_participant_id: "participant:resource-b".to_string(),
                    ..resource("workspace-b-overlap")
                },
            },
        );
        attach_b_overlap.causal_refs = vec!["participant:resource-b".to_string()];
        let overlap = store
            .commit_tenant_resource_attachment(
                &owner_b,
                "tenant:root-b",
                attach_b_overlap,
                &LocalFilesystemBinding::new(
                    "case:root-b",
                    "workspace-b-overlap",
                    &root.join("nested"),
                )
                .unwrap(),
            )
            .expect_err("overlapping root reuse across Tenants");
        assert!(overlap.contains("cross_tenant_filesystem_root_overlap"));
        assert!(store
            .get_case_state("case:root-b")
            .unwrap()
            .unwrap()
            .resources
            .is_empty());
        println!(
            "wave12_root_isolation: tenant_a=tenant:root-a tenant_b=tenant:root-b root={} exact={} overlap={} tenant_b_resource_count=0",
            root.display(),
            exact,
            overlap
        );
        drop(store);
        fs::remove_dir_all(path).expect("remove root isolation store");
        fs::remove_dir_all(root).expect("remove root hierarchy");
    }

    fn setup_runtime_case(
        store: &LmdbRecordStore,
        owner: &AuthenticatedPrincipal,
        tenant_id: &str,
        case_id: &str,
        suffix: &str,
    ) -> PathBuf {
        let principal_id = owner.projected_principal_id();
        store
            .create_tenant_case(owner, tenant_id, case_id)
            .expect("create runtime Case");
        let participant = store
            .commit_secured_transition(
                owner,
                tenant_id,
                secured_pending(
                    &format!("transition:runtime-participant:{suffix}"),
                    case_id,
                    1,
                    &principal_id,
                    TransitionPayload::ParticipantBound {
                        participant_id: "participant:model".to_string(),
                        role: "operation-proposer".to_string(),
                    },
                ),
                true,
            )
            .expect("bind runtime participant");
        let root = temp_store_path(&format!("runtime-root-{suffix}"));
        fs::create_dir_all(&root).expect("create runtime resource root");
        let attachment = ResourceAttachmentState {
            attachment_id: "resource:workspace".to_string(),
            kind: ResourceKind::Filesystem,
            allowed_write_prefix: "allowed".to_string(),
            max_write_bytes: 4096,
            policy_id: "compatibility-only".to_string(),
            policy_owner_participant_id: "participant:model".to_string(),
            review_requirement: ReviewRequirement::Automatic,
            process_signal_actions: Vec::new(),
        };
        let mut pending = secured_pending(
            &format!("transition:runtime-resource:{suffix}"),
            case_id,
            participant.state.generation,
            &principal_id,
            TransitionPayload::ResourceAttached { attachment },
        );
        pending.causal_refs = vec!["participant:model".to_string()];
        store
            .commit_tenant_resource_attachment(
                owner,
                tenant_id,
                pending,
                &LocalFilesystemBinding::new(case_id, "resource:workspace", &root)
                    .expect("runtime local binding"),
            )
            .expect("attach runtime resource");
        root
    }

    fn runtime_budgets() -> RuntimeCaseBudgets {
        RuntimeCaseBudgets {
            max_invocations: 2,
            max_operations: 2,
            max_semantic_units: 128,
            max_resident_items: 16,
            max_estimated_input_units: 1024,
            max_provider_retries: 0,
            max_runtime_ms: Some(10_000),
            stop_on_deny: true,
            continue_after_malformed: false,
        }
    }

    #[test]
    fn wave13_runtime_instance_is_exclusive_reclaimable_and_noncanonical() {
        let path = temp_store_path("wave13-runtime-instance");
        let store = LmdbRecordStore::open(&path).expect("open runtime store");
        let owner = AuthenticatedPrincipal::for_test(13001);
        store
            .bootstrap_local_security(&owner, "tenant:runtime", "organization:test", 1)
            .expect("bootstrap runtime owner");
        let config = RuntimeInstanceConfig {
            workers: 2,
            max_active_per_tenant: 1,
            max_queued_per_tenant: 4,
            max_queued_total: 8,
        };
        let first = RuntimeInstanceAcquireRequest {
            owner_pid: std::process::id(),
            owner_token: "runtime-owner:first".to_string(),
            now_unix_ms: 100,
            lease_duration_ms: 10,
            config: config.clone(),
        };
        let (outcome, _) = store
            .acquire_runtime_instance(&owner, &first, false)
            .expect("acquire instance");
        assert_eq!(outcome, RuntimeInstanceAcquireOutcome::Acquired);
        let split = store
            .acquire_runtime_instance(
                &owner,
                &RuntimeInstanceAcquireRequest {
                    owner_token: "runtime-owner:split".to_string(),
                    ..first.clone()
                },
                false,
            )
            .expect_err("second live instance rejected");
        assert!(split.contains("runtime_instance_active"));
        let live_after_lease = store
            .acquire_runtime_instance(
                &owner,
                &RuntimeInstanceAcquireRequest {
                    owner_token: "runtime-owner:reclaimed".to_string(),
                    now_unix_ms: 111,
                    ..first.clone()
                },
                true,
            )
            .expect_err("live process identity prevents lease-only reclaim");
        assert!(live_after_lease.contains("runtime_instance_active"));
        store
            .stop_runtime_instance(&owner, &first.owner_token, 111)
            .expect("live owner may stop after a delayed heartbeat");
        let (reclaimed, state) = store
            .acquire_runtime_instance(
                &owner,
                &RuntimeInstanceAcquireRequest {
                    owner_token: "runtime-owner:reclaimed".to_string(),
                    now_unix_ms: 112,
                    ..first
                },
                true,
            )
            .expect("stopped instance reclaimed");
        assert_eq!(reclaimed, RuntimeInstanceAcquireOutcome::Reclaimed);
        assert_eq!(state.lifecycle, RuntimeInstanceLifecycle::Starting);
        assert!(store.list_security_events().unwrap().len() >= 2);
        assert!(store.list_case_transitions("case:none").unwrap().is_empty());
        drop(store);
        fs::remove_dir_all(path).expect("remove runtime store");
    }

    #[test]
    fn wave13_one_process_shares_one_lmdb_environment_across_workers() {
        let path = temp_store_path("wave13-shared-environment");
        let store = LmdbRecordStore::open(&path).expect("open shared runtime store");
        let mut workers = Vec::new();
        for _ in 0..4 {
            let worker_path = path.clone();
            workers.push(std::thread::spawn(move || {
                for _ in 0..25 {
                    let worker_store =
                        LmdbRecordStore::open(&worker_path).expect("reuse shared LMDB environment");
                    worker_store
                        .summary()
                        .expect("read through shared environment");
                }
            }));
        }
        for worker in workers {
            worker.join().expect("LMDB worker remains live");
        }
        drop(store);
        fs::remove_dir_all(path).expect("remove shared environment store");
    }

    #[test]
    fn wave13_work_items_are_idempotent_bounded_isolated_and_case_serialized() {
        let path = temp_store_path("wave13-runtime-work");
        let store = LmdbRecordStore::open(&path).expect("open runtime work store");
        let owner = AuthenticatedPrincipal::for_test(13011);
        let other = AuthenticatedPrincipal::for_test(13012);
        store
            .bootstrap_local_security(&owner, "tenant:runtime-a", "organization:test", 10)
            .expect("bootstrap owner");
        store
            .bootstrap_local_security(&other, "tenant:runtime-b", "organization:test", 11)
            .expect("bootstrap other owner");
        let root = setup_runtime_case(&store, &owner, "tenant:runtime-a", "case:runtime-a", "a");
        let config = RuntimeInstanceConfig {
            workers: 2,
            max_active_per_tenant: 1,
            max_queued_per_tenant: 2,
            max_queued_total: 2,
        };
        let token = "runtime-owner:work";
        store
            .acquire_runtime_instance(
                &owner,
                &RuntimeInstanceAcquireRequest {
                    owner_pid: std::process::id(),
                    owner_token: token.to_string(),
                    now_unix_ms: 20,
                    lease_duration_ms: 1_000,
                    config,
                },
                false,
            )
            .expect("acquire instance");
        store
            .activate_runtime_instance(&owner, token, 20, 1_000, 0)
            .expect("activate instance");
        let submission = RuntimeWorkSubmission {
            request_id: "request:one".to_string(),
            tenant_id: "tenant:runtime-a".to_string(),
            case_id: "case:runtime-a".to_string(),
            participant_id: "participant:model".to_string(),
            attachment_id: "resource:workspace".to_string(),
            journal_path: "/tmp/runtime-journal.jsonl".to_string(),
            task: "write first".to_string(),
            budgets: runtime_budgets(),
            failpoint: None,
            now_unix_ms: 21,
        };
        let first = store
            .submit_runtime_work(&owner, &submission)
            .expect("submit first");
        assert!(first.created);
        let repeated = store
            .submit_runtime_work(&owner, &submission)
            .expect("repeat idempotently");
        assert!(!repeated.created);
        assert_eq!(first.item.work_id, repeated.item.work_id);
        let second = store
            .submit_runtime_work(
                &owner,
                &RuntimeWorkSubmission {
                    request_id: "request:two".to_string(),
                    task: "write first".to_string(),
                    now_unix_ms: 22,
                    ..submission.clone()
                },
            )
            .expect("same prompt with different request stays distinct");
        assert_ne!(first.item.work_id, second.item.work_id);
        let capacity = store
            .submit_runtime_work(
                &owner,
                &RuntimeWorkSubmission {
                    request_id: "request:three".to_string(),
                    now_unix_ms: 23,
                    ..submission.clone()
                },
            )
            .expect_err("Tenant queue capacity applies");
        assert_eq!(capacity, "runtime_global_queue_capacity_exhausted");
        let early_second = store
            .claim_runtime_work(&owner, token, &second.item.work_id, "worker:1", 24)
            .expect_err("same Case preserves FIFO and one active item");
        assert_eq!(early_second, "runtime_case_already_active");
        let running = store
            .claim_runtime_work(&owner, token, &first.item.work_id, "worker:0", 24)
            .expect("claim first");
        assert_eq!(running.state, RuntimeWorkState::Running);
        assert_eq!(
            store
                .get_runtime_instance_authorized(&owner)
                .unwrap()
                .unwrap()
                .last_dispatched_tenant
                .as_deref(),
            Some("tenant:runtime-a")
        );
        let mut forged = running.clone();
        forged.task = "forged".to_string();
        assert_eq!(
            forged.validate_integrity().unwrap_err(),
            "runtime_work_item_integrity_mismatch"
        );
        let invalid_queued_terminal = store
            .update_runtime_work_state(
                &owner,
                token,
                &second.item.work_id,
                None,
                RuntimeWorkState::Completed,
                "forged direct completion",
                25,
            )
            .expect_err("Queued cannot jump directly to Completed");
        assert!(invalid_queued_terminal.contains("runtime_work_invalid_state_transition"));
        let cross_tenant = store
            .submit_runtime_work(
                &other,
                &RuntimeWorkSubmission {
                    request_id: "request:cross".to_string(),
                    tenant_id: "tenant:runtime-b".to_string(),
                    case_id: "case:runtime-a".to_string(),
                    now_unix_ms: 25,
                    ..submission
                },
            )
            .expect_err("other Principal cannot inject work");
        assert_eq!(cross_tenant, "runtime_instance_principal_mismatch");
        let before = store
            .get_case_state("case:runtime-a")
            .unwrap()
            .unwrap()
            .generation;
        assert_eq!(store.list_runtime_work_authorized(&owner).unwrap().len(), 2);
        let after = store
            .get_case_state("case:runtime-a")
            .unwrap()
            .unwrap()
            .generation;
        assert_eq!(before, after, "queue status is pure Case observation");
        store
            .update_runtime_work_state(
                &owner,
                token,
                &first.item.work_id,
                Some("worker:0"),
                RuntimeWorkState::Completed,
                "first work completed",
                26,
            )
            .expect("Running may complete");
        store
            .claim_runtime_work(&owner, token, &second.item.work_id, "worker:1", 27)
            .expect("second work becomes Running after first terminalizes");
        store
            .stop_runtime_instance(&owner, token, 28)
            .expect("old process owner stops operational instance");
        let new_token = "runtime-owner:new-epoch";
        store
            .acquire_runtime_instance(
                &owner,
                &RuntimeInstanceAcquireRequest {
                    owner_pid: std::process::id(),
                    owner_token: new_token.to_string(),
                    now_unix_ms: 29,
                    lease_duration_ms: 1_000,
                    config: RuntimeInstanceConfig {
                        workers: 2,
                        max_active_per_tenant: 1,
                        max_queued_per_tenant: 2,
                        max_queued_total: 2,
                    },
                },
                true,
            )
            .expect("new owner epoch reclaims stopped instance");
        store
            .activate_runtime_instance(&owner, new_token, 29, 1_000, 0)
            .expect("activate new owner epoch");
        let stale_owner = store
            .update_runtime_work_state(
                &owner,
                token,
                &second.item.work_id,
                Some("worker:1"),
                RuntimeWorkState::Completed,
                "stale old-worker result",
                30,
            )
            .expect_err("old owner epoch cannot report completion");
        assert!(stale_owner.contains("runtime_instance_owner_mismatch"));
        let stale_worker = store
            .update_runtime_work_state(
                &owner,
                new_token,
                &second.item.work_id,
                Some("worker:1"),
                RuntimeWorkState::Completed,
                "new owner cannot adopt old worker result",
                30,
            )
            .expect_err("new owner epoch cannot adopt stale worker result");
        assert_eq!(stale_worker, "runtime_work_worker_lease_mismatch");
        drop(store);
        fs::remove_dir_all(path).expect("remove runtime work store");
        fs::remove_dir_all(root).expect("remove runtime resource root");
    }

    #[test]
    fn h13_runtime_work_fsm_is_explicit_and_terminal_states_are_closed() {
        use RuntimeWorkState::*;
        let valid = [
            (Queued, Running),
            (Queued, Cancelled),
            (Running, Queued),
            (Running, WaitingReview),
            (Running, WaitingEffect),
            (Running, Blocked),
            (Running, Completed),
            (Running, Denied),
            (Running, Cancelled),
            (Running, Failed),
            (WaitingReview, Queued),
            (WaitingReview, Cancelled),
            (WaitingEffect, Queued),
            (WaitingEffect, Cancelled),
            (Blocked, Queued),
            (Blocked, Cancelled),
        ];
        for (from, to) in valid {
            assert!(from.permits_transition_to(&to), "{from:?} -> {to:?}");
        }
        let invalid = [
            (Queued, Completed),
            (WaitingReview, Completed),
            (Blocked, Running),
            (Completed, Queued),
            (Denied, Running),
            (Cancelled, Running),
            (Failed, Running),
        ];
        for (from, to) in invalid {
            assert!(!from.permits_transition_to(&to), "{from:?} -> {to:?}");
        }
    }

    #[test]
    fn h13_pid_reuse_discriminator_and_current_process_identity_are_explicit() {
        let current = runtime_process_identity(std::process::id())
            .expect("Linux process identity is observable");
        assert!(current.starts_with("linux-proc-v1:"));
        assert!(process_identity_matches(123, "birth:A", 123, "birth:A"));
        assert!(!process_identity_matches(123, "birth:A", 123, "birth:B"));
        assert!(!process_identity_matches(123, "birth:A", 124, "birth:A"));
    }

    #[test]
    fn h13_owner_token_cannot_impersonate_another_process_identity() {
        let path = temp_store_path("h13-process-owner");
        let store = LmdbRecordStore::open(&path).expect("open owner store");
        let owner = AuthenticatedPrincipal::for_test(13101);
        store
            .bootstrap_local_security(&owner, "tenant:h13-owner", "organization:test", 1)
            .expect("bootstrap owner");
        let token = "runtime-owner:known-token";
        store
            .acquire_runtime_instance(
                &owner,
                &RuntimeInstanceAcquireRequest {
                    owner_pid: std::process::id(),
                    owner_token: token.to_string(),
                    now_unix_ms: 10,
                    lease_duration_ms: 1_000,
                    config: RuntimeInstanceConfig {
                        workers: 1,
                        max_active_per_tenant: 1,
                        max_queued_per_tenant: 1,
                        max_queued_total: 1,
                    },
                },
                false,
            )
            .expect("acquire owner");
        let mut txn = store.env.begin_rw_txn().expect("owner mutation txn");
        let mut instance = get_json_txn::<RuntimeInstance, _>(
            &txn,
            store.runtime_instances,
            RUNTIME_INSTANCE_ID,
            "runtime_instance",
        )
        .unwrap()
        .unwrap();
        instance.owner_process_identity = "linux-proc-v1:forged:999".to_string();
        instance.integrity_digest = runtime_instance_integrity_digest(&instance).unwrap();
        put_json_txn(
            &mut txn,
            store.runtime_instances,
            RUNTIME_INSTANCE_ID,
            &instance,
            WriteFlags::empty(),
            "runtime instance",
        )
        .unwrap();
        txn.commit().unwrap();
        let error = store
            .heartbeat_runtime_instance(&owner, token, 20, 1_000)
            .expect_err("token plus Principal cannot replace process identity");
        assert_eq!(error, "runtime_instance_owner_process_mismatch");
        drop(store);
        fs::remove_dir_all(path).expect("remove owner store");
    }

    #[test]
    fn h13_shared_environment_map_size_contract_is_explicit() {
        let path = temp_store_path("h13-map-size");
        let store = LmdbRecordStore::open_with_map_size(&path, MINIMUM_LMDB_MAP_SIZE)
            .expect("open constrained shared environment");
        LmdbRecordStore::open_with_map_size(&path, MINIMUM_LMDB_MAP_SIZE)
            .expect("same requested map size reuses environment");
        let error = match LmdbRecordStore::open_with_map_size(&path, DEFAULT_LMDB_MAP_SIZE) {
            Ok(_) => panic!("different map size silently reused environment"),
            Err(error) => error,
        };
        assert!(error.contains("lmdb_environment_map_size_mismatch"));
        drop(store);
        fs::remove_dir_all(path).expect("remove map-size store");
    }

    #[test]
    fn h13_shared_lmdb_environment_survives_read_write_thread_stress() {
        let path = temp_store_path("h13-shared-environment-stress");
        let store = LmdbRecordStore::open(&path).expect("open stress store");
        for worker_count in [1usize, 2, 4, 8] {
            let mut workers = Vec::new();
            for worker in 0..worker_count {
                let worker_path = path.clone();
                workers.push(std::thread::spawn(move || {
                    for iteration in 0..32 {
                        let worker_store = LmdbRecordStore::open(&worker_path)
                            .expect("reuse shared LMDB environment");
                        let record = Record::from_parts(
                            format!("record:h13:{worker_count}:{worker}:{iteration}"),
                            "case:h13-lmdb-stress",
                            RecordKind::InteractionTurn,
                            format!("subject:worker-{worker}"),
                            "",
                            "",
                            "",
                            "bounded concurrent LMDB write",
                        );
                        worker_store
                            .append_record(&record, "h13-lmdb-stress")
                            .expect("short concurrent write");
                        worker_store.summary().expect("concurrent read");
                    }
                }));
            }
            for worker in workers {
                worker.join().expect("LMDB worker does not panic");
            }
        }
        assert_eq!(store.summary().unwrap().records_total, (1 + 2 + 4 + 8) * 32);
        drop(store);
        let reopened = LmdbRecordStore::open(&path).expect("reopen after all worker drops");
        assert_eq!(
            reopened.summary().unwrap().records_total,
            (1 + 2 + 4 + 8) * 32
        );
        drop(reopened);
        fs::remove_dir_all(path).expect("remove stress store");
    }

    #[test]
    fn h13_runtime_work_historical_scan_scale_is_characterized() {
        let path = temp_store_path("h13-runtime-work-scale");
        let store = LmdbRecordStore::open(&path).expect("open scale store");
        let mut inserted = 0usize;
        for target in [100usize, 1_000, 5_000] {
            let mut txn = store.env.begin_rw_txn().expect("scale write txn");
            for sequence in inserted + 1..=target {
                let mut item = RuntimeWorkItem {
                    schema: RUNTIME_WORK_ITEM_SCHEMA.to_string(),
                    work_id: format!("runtime-work:scale-{sequence:05}"),
                    integrity_digest: String::new(),
                    request_id: format!("request:scale-{sequence:05}"),
                    request_digest: format!("digest:scale-{sequence:05}"),
                    principal_id: "principal:scale".to_string(),
                    tenant_id: "tenant:scale".to_string(),
                    case_id: format!("case:scale-{sequence:05}"),
                    participant_id: "participant:model".to_string(),
                    attachment_id: "resource:workspace".to_string(),
                    journal_path: "/tmp/scale-journal.jsonl".to_string(),
                    task: "terminal historical work".to_string(),
                    budgets: runtime_budgets(),
                    failpoint: None,
                    workflow: None,
                    enqueue_sequence: sequence as u64,
                    state: RuntimeWorkState::Completed,
                    attempt_count: 1,
                    runtime_instance_id: Some(RUNTIME_INSTANCE_ID.to_string()),
                    runtime_owner_token: None,
                    worker_id: None,
                    last_stop_reason: "completed".to_string(),
                    enqueued_at_unix_ms: sequence as u64,
                    updated_at_unix_ms: sequence as u64,
                };
                item.integrity_digest = runtime_work_integrity_digest(&item).unwrap();
                put_json_txn(
                    &mut txn,
                    store.runtime_work_items,
                    &item.work_id,
                    &item,
                    WriteFlags::empty(),
                    "runtime work scale item",
                )
                .unwrap();
            }
            txn.commit().expect("commit scale records");
            inserted = target;
            let started = std::time::Instant::now();
            let txn = store.env.begin_ro_txn().expect("scale read txn");
            let items = list_runtime_work_items_txn(&txn, store.runtime_work_items)
                .expect("list terminal history");
            let elapsed = started.elapsed();
            assert_eq!(items.len(), target);
            println!(
                "h13_runtime_work_list_scale: terminal_items={target} elapsed_us={}",
                elapsed.as_micros()
            );
        }
        drop(store);
        fs::remove_dir_all(path).expect("remove scale store");
    }

    #[test]
    fn h13_runtime_heartbeat_stays_inside_lease_margin_under_eight_writer_contention() {
        let path = temp_store_path("h13-heartbeat-stress");
        let store = LmdbRecordStore::open(&path).expect("open heartbeat store");
        let owner = AuthenticatedPrincipal::for_test(13131);
        store
            .bootstrap_local_security(&owner, "tenant:h13-heartbeat", "organization:test", 1)
            .expect("bootstrap heartbeat owner");
        let token = "runtime-owner:heartbeat";
        store
            .acquire_runtime_instance(
                &owner,
                &RuntimeInstanceAcquireRequest {
                    owner_pid: std::process::id(),
                    owner_token: token.to_string(),
                    now_unix_ms: 1,
                    lease_duration_ms: 5_000,
                    config: RuntimeInstanceConfig {
                        workers: 8,
                        max_active_per_tenant: 8,
                        max_queued_per_tenant: 64,
                        max_queued_total: 128,
                    },
                },
                false,
            )
            .expect("acquire heartbeat instance");
        store
            .activate_runtime_instance(&owner, token, 1, 5_000, 0)
            .expect("activate heartbeat instance");
        let completed = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut workers = Vec::new();
        for worker in 0..8usize {
            let worker_path = path.clone();
            let completed = std::sync::Arc::clone(&completed);
            workers.push(std::thread::spawn(move || {
                let worker_store = LmdbRecordStore::open(&worker_path).unwrap();
                for iteration in 0..64usize {
                    let record = Record::from_parts(
                        format!("record:h13-heartbeat:{worker}:{iteration}"),
                        "case:h13-heartbeat",
                        RecordKind::InteractionTurn,
                        format!("subject:worker-{worker}"),
                        "",
                        "",
                        "",
                        "heartbeat contention write",
                    );
                    worker_store
                        .append_record(&record, "h13-heartbeat-stress")
                        .unwrap();
                }
                completed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }));
        }
        let mut heartbeat_sequence = 2u64;
        let mut max_heartbeat_ms = 0u128;
        while completed.load(std::sync::atomic::Ordering::SeqCst) < 8 {
            let started = std::time::Instant::now();
            store
                .heartbeat_runtime_instance(&owner, token, heartbeat_sequence, 5_000)
                .expect("heartbeat under write contention");
            max_heartbeat_ms = max_heartbeat_ms.max(started.elapsed().as_millis());
            heartbeat_sequence += 1;
            std::thread::yield_now();
        }
        for worker in workers {
            worker.join().expect("contention worker remains live");
        }
        println!(
            "h13_heartbeat_stress: workers=8 writes={} max_heartbeat_ms={max_heartbeat_ms} lease_margin_ms=5000",
            8 * 64
        );
        assert!(max_heartbeat_ms < 5_000);
        drop(store);
        fs::remove_dir_all(path).expect("remove heartbeat store");
    }

    #[derive(Clone)]
    struct Wave14FilesystemAuthority {
        resource: ResourceAttachmentState,
        operation: Operation,
        decision: Decision,
        grant: crate::effect::ExecutionGrant,
        prepared_intent: crate::effect::PreparedEffect,
    }

    fn setup_wave14_filesystem_authority(
        store: &LmdbRecordStore,
        owner: &AuthenticatedPrincipal,
        tenant_id: &str,
        case_id: &str,
        suffix: &str,
        root: &Path,
        content: &str,
    ) -> Wave14FilesystemAuthority {
        let principal_id = owner.projected_principal_id();
        store
            .create_tenant_case(owner, tenant_id, case_id)
            .expect("create Wave14 Case");
        let participant = store
            .commit_secured_transition(
                owner,
                tenant_id,
                secured_pending(
                    &format!("transition:w14:{suffix}:participant"),
                    case_id,
                    1,
                    &principal_id,
                    TransitionPayload::ParticipantBound {
                        participant_id: "participant:model".to_string(),
                        role: "operation-proposer".to_string(),
                    },
                ),
                true,
            )
            .expect("bind Wave14 proposer");
        let resource = ResourceAttachmentState {
            attachment_id: "resource:shared".to_string(),
            kind: ResourceKind::Filesystem,
            allowed_write_prefix: "allowed".to_string(),
            max_write_bytes: 4096,
            policy_id: format!("policy:w14:{suffix}"),
            policy_owner_participant_id: "participant:model".to_string(),
            review_requirement: ReviewRequirement::Automatic,
            process_signal_actions: Vec::new(),
        };
        let mut attachment = secured_pending(
            &format!("transition:w14:{suffix}:resource"),
            case_id,
            participant.state.generation,
            &principal_id,
            TransitionPayload::ResourceAttached {
                attachment: resource.clone(),
            },
        );
        attachment.causal_refs = vec!["participant:model".to_string()];
        store
            .commit_tenant_resource_attachment(
                owner,
                tenant_id,
                attachment,
                &LocalFilesystemBinding::new(case_id, "resource:shared", root).unwrap(),
            )
            .expect("attach shared filesystem root");
        let source = serde_json::to_vec(&serde_json::json!({
            "schema": POLICY_SOURCE_INPUT_SCHEMA,
            "policy_key": format!("wave14.shared.{suffix}"),
            "source_version": "1",
            "owner_ref": "organization:wave14",
            "source_origin": {
                "source_system": "wave14-test",
                "source_uri": format!("test://wave14/{suffix}")
            },
            "validity": {"mode":"unbounded"},
            "rules": [
                {"kind":"operation_restriction","rule_id":"allow","operation_kind":"filesystem.write","resource_kind":"filesystem","effect":"allow","reason":"Wave14 test allow"},
                {"kind":"authority_requirement","rule_id":"proposer","operation_kind":"filesystem.write","resource_kind":"filesystem","subject":"proposer","required_role":"operation-proposer","reason":"Wave14 proposer"},
                {"kind":"evidence_obligation","rule_id":"source","operation_kind":"filesystem.write","resource_kind":"filesystem","obligation":"source_provenance","reason":"Wave14 canonical source"},
                {"kind":"evidence_obligation","rule_id":"pre","operation_kind":"filesystem.write","resource_kind":"filesystem","obligation":"pre_observation","reason":"Wave14 pre observation"},
                {"kind":"evidence_obligation","rule_id":"post","operation_kind":"filesystem.write","resource_kind":"filesystem","obligation":"post_observation","reason":"Wave14 post observation"}
            ]
        }))
        .unwrap();
        let global = compile_policy_source(&source).expect("compile Wave14 policy");
        let scoped = scope_policy_compilation(&global, tenant_id, "organization:wave14")
            .expect("scope Wave14 policy");
        store
            .ingest_tenant_policy_compilation(owner, tenant_id, &scoped)
            .unwrap();
        store
            .validate_tenant_policy_artifact(owner, &scoped.artifact.artifact_id, "Wave14")
            .unwrap();
        store
            .publish_tenant_policy_artifact(owner, &scoped.artifact.artifact_id, "Wave14")
            .unwrap();
        let state = store.get_case_state(case_id).unwrap().unwrap();
        store
            .bind_tenant_case_policy(
                owner,
                case_id,
                &scoped.artifact.artifact_id,
                state.generation,
                "Wave14 exact binding",
            )
            .unwrap();
        let state = store.get_case_state(case_id).unwrap().unwrap();
        store
            .commit_secured_transition(
                owner,
                tenant_id,
                secured_pending(
                    &format!("transition:w14:{suffix}:provider"),
                    case_id,
                    state.generation,
                    &principal_id,
                    TransitionPayload::ProviderAttached {
                        participant_id: "participant:model".to_string(),
                        provider_id: "provider:test".to_string(),
                        provider_kind: "openai_compatible".to_string(),
                        base_url: "http://127.0.0.1:1".to_string(),
                        model_id: "model:test".to_string(),
                        credential_ref: "env:TEST".to_string(),
                    },
                ),
                true,
            )
            .unwrap();
        let state = store.get_case_state(case_id).unwrap().unwrap();
        let lineage = test_provider_lineage(state.generation);
        let invocation_id = format!("invocation:w14:{suffix}");
        let result_id = format!("provider-result:w14:{suffix}");
        commit_typed(
            store,
            &format!("transition:w14:{suffix}:invocation"),
            case_id,
            state.generation,
            TransitionPayload::ProviderInvocationStarted {
                invocation_id: invocation_id.clone(),
                participant_id: "participant:model".to_string(),
                provider_id: "provider:test".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:test".to_string(),
                semantic_lineage: Some(lineage.clone()),
            },
            None,
            vec![],
        );
        let state = store.get_case_state(case_id).unwrap().unwrap();
        commit_typed(
            store,
            &format!("transition:w14:{suffix}:result"),
            case_id,
            state.generation,
            TransitionPayload::ProviderResultRecorded {
                result_id: result_id.clone(),
                invocation_id: invocation_id.clone(),
                provider_id: "provider:test".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:test".to_string(),
                semantic_lineage: Some(lineage),
                output: "Wave14 candidate".to_string(),
            },
            None,
            vec![invocation_id.clone()],
        );
        let state = store.get_case_state(case_id).unwrap().unwrap();
        let raw = format!(
            "{{\"schema\":\"yai.operation_proposal.filesystem_write.v1\",\"operation\":\"filesystem.write\",\"resource\":\"resource:shared\",\"path\":\"allowed/shared.txt\",\"content\":{}}}",
            serde_json::to_string(content).unwrap()
        );
        let operation = normalize_filesystem_write_candidate(
            &raw,
            &NormalizationContext {
                case_id,
                participant_id: "participant:model",
                provider_result_id: &result_id,
                provider_invocation_id: &invocation_id,
                case_generation: state.generation,
                resource: &resource,
            },
        )
        .unwrap();
        let operation_commit = commit_typed(
            store,
            &format!("transition:w14:{suffix}:operation"),
            case_id,
            state.generation,
            TransitionPayload::OperationRecorded {
                operation: operation.clone(),
            },
            Some(operation.scope.clone()),
            operation.origin.causal_refs(),
        );
        let (decision, decision_commit) = store
            .derive_and_commit_policy_decision(case_id, &operation.operation_id)
            .expect("derive Wave14 Decision");
        assert_eq!(decision.outcome, crate::effect::DecisionOutcome::Allow);
        assert_eq!(
            decision.decided_at_case_generation,
            operation_commit.state.generation
        );
        let grant =
            issue_policy_execution_grant(&operation, &decision, decision_commit.state.generation)
                .unwrap();
        let basis = decision.decision_basis.as_ref().unwrap();
        let grant_commit = commit_typed(
            store,
            &format!("transition:w14:{suffix}:grant"),
            case_id,
            decision_commit.state.generation,
            TransitionPayload::ExecutionGrantIssued {
                grant: grant.clone(),
            },
            Some(operation.scope.clone()),
            vec![
                operation.operation_id.clone(),
                decision.decision_id.clone(),
                basis.basis_id.clone(),
                basis.effective_policy_id.clone(),
            ],
        );
        let binding = store
            .get_local_filesystem_binding(case_id, "resource:shared")
            .unwrap()
            .unwrap();
        let pre = observe_filesystem(
            &binding,
            &resource,
            "allowed/shared.txt",
            format!("observation:w14:{suffix}:pre"),
        );
        let prepared_intent = prepare_fenced_effect(&operation, &decision, &grant, pre).unwrap();
        assert_eq!(
            grant_commit.state.generation,
            grant.expected_case_generation + 1
        );
        Wave14FilesystemAuthority {
            resource,
            operation,
            decision,
            grant,
            prepared_intent,
        }
    }

    fn wave14_prepare_pending(
        store: &LmdbRecordStore,
        suffix: &str,
        chain: &Wave14FilesystemAuthority,
    ) -> PendingTransition {
        let generation = store
            .get_case_state(&chain.operation.case_id)
            .unwrap()
            .unwrap()
            .generation;
        let mut pending = PendingTransition::new(
            format!("transition:w14:{suffix}:prepare"),
            &chain.operation.case_id,
            generation,
            TransitionSource::component("wave14-test"),
            TransitionPayload::EffectPrepared {
                prepared: chain.prepared_intent.clone(),
            },
        );
        pending.causal_refs = vec![
            chain.operation.operation_id.clone(),
            chain.decision.decision_id.clone(),
            chain.grant.grant_id.clone(),
            chain
                .prepared_intent
                .expected_pre_observation
                .observation_id
                .clone(),
        ];
        pending
    }

    #[test]
    fn wave14_shared_resource_epoch_blocks_competitor_and_stale_carrier() {
        let path = temp_store_path("wave14-shared-resource");
        let root = temp_store_path("wave14-shared-root");
        fs::create_dir_all(root.join("allowed")).unwrap();
        let store = LmdbRecordStore::open(&path).unwrap();
        let owner = AuthenticatedPrincipal::for_test(14001);
        store
            .bootstrap_local_security(&owner, "tenant:wave14", "organization:wave14", 1_400_001)
            .unwrap();
        let first = setup_wave14_filesystem_authority(
            &store,
            &owner,
            "tenant:wave14",
            "case:wave14-a",
            "a",
            &root,
            "epoch-one",
        );
        let mut second = setup_wave14_filesystem_authority(
            &store,
            &owner,
            "tenant:wave14",
            "case:wave14-b",
            "b",
            &root,
            "epoch-two",
        );
        let first_commit = match store
            .commit_fenced_effect_prepared(
                wave14_prepare_pending(&store, "a", &first),
                std::process::id(),
            )
            .unwrap()
        {
            PreparedCommitOutcome::Prepared(commit) => commit,
            PreparedCommitOutcome::GrantInvalidated(_) => panic!("first Grant invalidated"),
        };
        let first_prepared = match first_commit.transition.payload {
            TransitionPayload::EffectPrepared { prepared } => prepared,
            _ => unreachable!(),
        };
        let fence_one = first_prepared.resource_fence.clone().unwrap();
        assert_eq!(fence_one.resource_epoch, 1);
        assert_eq!(
            store
                .reclaim_resource_for_effect(&fence_one, std::process::id())
                .expect_err("live exact owner cannot be reclaimed"),
            "live_resource_owner_cannot_be_reclaimed"
        );
        let blocked = store
            .commit_fenced_effect_prepared(
                wave14_prepare_pending(&store, "b", &second),
                std::process::id(),
            )
            .expect_err("second Case cannot PREPARE active shared resource");
        assert!(blocked.contains("resource_temporarily_owned"));
        assert!(store
            .get_case_state("case:wave14-b")
            .unwrap()
            .unwrap()
            .effects
            .is_empty());
        let first_binding = store
            .get_local_filesystem_binding("case:wave14-a", "resource:shared")
            .unwrap()
            .unwrap();
        let first_result = execute_fenced_filesystem_write(
            &store,
            &fence_one,
            &first.operation,
            &first.decision,
            &first.grant,
            &first_prepared,
            &first_commit.state,
            &first_binding,
            &first.resource,
            CarrierFailpoint::None,
        )
        .unwrap();
        let first_receipt = build_effect_receipt(&first_prepared, &first_result);
        let mut first_final = PendingTransition::new(
            "transition:w14:a:final",
            "case:wave14-a",
            first_commit.state.generation,
            TransitionSource::component("wave14-test"),
            TransitionPayload::EffectFinalized {
                effect_id: first_prepared.effect_id.clone(),
                post_observation: first_result.post_observation.clone(),
                receipt: first_receipt.clone(),
            },
        );
        first_final.causal_refs = vec![first_prepared.effect_id.clone(), first_receipt.receipt_id];
        store
            .commit_fenced_effect_terminal(first_final, &fence_one)
            .unwrap();
        let second_binding = store
            .get_local_filesystem_binding("case:wave14-b", "resource:shared")
            .unwrap()
            .unwrap();
        let second_pre = observe_filesystem(
            &second_binding,
            &second.resource,
            "allowed/shared.txt",
            "observation:w14:b:pre-after-release",
        );
        second.prepared_intent = prepare_fenced_effect(
            &second.operation,
            &second.decision,
            &second.grant,
            second_pre,
        )
        .unwrap();
        let second_commit = match store
            .commit_fenced_effect_prepared(
                wave14_prepare_pending(&store, "b", &second),
                std::process::id(),
            )
            .unwrap()
        {
            PreparedCommitOutcome::Prepared(commit) => commit,
            PreparedCommitOutcome::GrantInvalidated(_) => panic!("second Grant invalidated"),
        };
        let second_prepared = match second_commit.transition.payload {
            TransitionPayload::EffectPrepared { prepared } => prepared,
            _ => unreachable!(),
        };
        let fence_two = second_prepared.resource_fence.clone().unwrap();
        assert_eq!(fence_two.resource_epoch, 2);
        let stale = execute_fenced_filesystem_write(
            &store,
            &fence_one,
            &first.operation,
            &first.decision,
            &first.grant,
            &first_prepared,
            &first_commit.state,
            &first_binding,
            &first.resource,
            CarrierFailpoint::None,
        )
        .expect_err("epoch one carrier must be stale after epoch two acquisition");
        assert!(stale.contains("stale_resource_fence"));
        assert_eq!(
            fs::read_to_string(root.join("allowed/shared.txt")).unwrap(),
            "epoch-one"
        );
        let second_result = execute_fenced_filesystem_write(
            &store,
            &fence_two,
            &second.operation,
            &second.decision,
            &second.grant,
            &second_prepared,
            &second_commit.state,
            &second_binding,
            &second.resource,
            CarrierFailpoint::None,
        )
        .unwrap();
        let second_receipt = build_effect_receipt(&second_prepared, &second_result);
        let mut second_final = PendingTransition::new(
            "transition:w14:b:final",
            "case:wave14-b",
            second_commit.state.generation,
            TransitionSource::component("wave14-test"),
            TransitionPayload::EffectFinalized {
                effect_id: second_prepared.effect_id.clone(),
                post_observation: second_result.post_observation.clone(),
                receipt: second_receipt.clone(),
            },
        );
        second_final.causal_refs =
            vec![second_prepared.effect_id.clone(), second_receipt.receipt_id];
        store
            .commit_fenced_effect_terminal(second_final, &fence_two)
            .unwrap();
        assert_eq!(
            fs::read_to_string(root.join("allowed/shared.txt")).unwrap(),
            "epoch-two"
        );
        assert!(store
            .get_resource_control_state(&fence_two.resource_id)
            .unwrap()
            .unwrap()
            .active_lease
            .is_none());
        assert_eq!(
            store
                .list_resource_control_events(&fence_two.resource_id)
                .unwrap()
                .len(),
            4
        );
        assert!(store.verify_case_state("case:wave14-a").unwrap());
        assert!(store.verify_case_state("case:wave14-b").unwrap());
        println!(
            "wave14_fencing: resource={} epoch1={} blocked={} epoch2={} stale={} final=epoch-two history_events=4",
            fence_two.resource_id,
            fence_one.resource_epoch,
            blocked,
            fence_two.resource_epoch,
            stale
        );
        drop(store);
        fs::remove_dir_all(path).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn h14_content_valid_fence_requires_exact_canonical_presence() {
        let path = temp_store_path("h14-forged-fence");
        let root = temp_store_path("h14-forged-fence-root");
        fs::create_dir_all(root.join("allowed")).unwrap();
        let store = LmdbRecordStore::open(&path).unwrap();
        let owner = AuthenticatedPrincipal::for_test(14101);
        store
            .bootstrap_local_security(&owner, "tenant:h14", "organization:wave14", 1_410_001)
            .unwrap();
        let chain = setup_wave14_filesystem_authority(
            &store,
            &owner,
            "tenant:h14",
            "case:h14-forgery",
            "forgery",
            &root,
            "must-not-be-written",
        );
        let commit = match store
            .commit_fenced_effect_prepared(
                wave14_prepare_pending(&store, "forgery", &chain),
                std::process::id(),
            )
            .unwrap()
        {
            PreparedCommitOutcome::Prepared(commit) => commit,
            PreparedCommitOutcome::GrantInvalidated(_) => panic!("Grant invalidated"),
        };
        let prepared = match &commit.transition.payload {
            TransitionPayload::EffectPrepared { prepared } => prepared.clone(),
            _ => unreachable!(),
        };
        let current = prepared.resource_fence.clone().unwrap();
        let control = store
            .get_resource_control_state(&current.resource_id)
            .unwrap()
            .unwrap();
        let identity = control.identity.clone();
        let process = LocalProcessIdentity::capture(std::process::id()).unwrap();
        let forged = ResourceFence::issue(
            &identity,
            current.resource_epoch,
            &current.case_id,
            &current.operation_id,
            &current.grant_id,
            &current.effect_id,
            std::process::id(),
            &process.canonical_identity(),
            current.issued_at_unix_ms + 1,
        )
        .unwrap();
        forged.validate_integrity().unwrap();
        assert_ne!(forged.fence_id, current.fence_id);
        let binding = store
            .get_local_filesystem_binding("case:h14-forgery", "resource:shared")
            .unwrap()
            .unwrap();
        let error = execute_fenced_filesystem_write(
            &store,
            &forged,
            &chain.operation,
            &chain.decision,
            &chain.grant,
            &prepared,
            &commit.state,
            &binding,
            &chain.resource,
            CarrierFailpoint::None,
        )
        .expect_err("content-valid absent fence cannot become carrier authority");
        assert!(error.contains("stale_resource_fence"));
        assert!(!root.join("allowed/shared.txt").exists());

        for (case_id, operation_id, grant_id, effect_id, owner_identity) in [
            (
                "case:wrong",
                current.operation_id.as_str(),
                current.grant_id.as_str(),
                current.effect_id.as_str(),
                process.canonical_identity(),
            ),
            (
                current.case_id.as_str(),
                "operation:wrong",
                current.grant_id.as_str(),
                current.effect_id.as_str(),
                process.canonical_identity(),
            ),
            (
                current.case_id.as_str(),
                current.operation_id.as_str(),
                "grant:wrong",
                current.effect_id.as_str(),
                process.canonical_identity(),
            ),
            (
                current.case_id.as_str(),
                current.operation_id.as_str(),
                current.grant_id.as_str(),
                "effect:wrong",
                process.canonical_identity(),
            ),
            (
                current.case_id.as_str(),
                current.operation_id.as_str(),
                current.grant_id.as_str(),
                current.effect_id.as_str(),
                "linux-process-v1:wrong:999:1".to_string(),
            ),
        ] {
            let candidate = ResourceFence::issue(
                &identity,
                current.resource_epoch,
                case_id,
                operation_id,
                grant_id,
                effect_id,
                std::process::id(),
                &owner_identity,
                current.issued_at_unix_ms + 2,
            )
            .unwrap();
            candidate.validate_integrity().unwrap();
            assert!(store.validate_carrier_fence(&candidate).is_err());
        }
        println!(
            "h14_fence_forgery: canonical_fence={} forged_fence={} result={} physical_mutations=0",
            current.fence_id, forged.fence_id, error
        );
        drop(store);
        fs::remove_dir_all(path).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn h14_resource_history_rebuild_is_exact_and_rejects_invalid_append() {
        let path = temp_store_path("h14-resource-rebuild");
        let root = temp_store_path("h14-resource-rebuild-root");
        fs::create_dir_all(root.join("allowed")).unwrap();
        let store = LmdbRecordStore::open(&path).unwrap();
        let owner = AuthenticatedPrincipal::for_test(14102);
        store
            .bootstrap_local_security(
                &owner,
                "tenant:h14-rebuild",
                "organization:wave14",
                1_410_002,
            )
            .unwrap();
        let chain = setup_wave14_filesystem_authority(
            &store,
            &owner,
            "tenant:h14-rebuild",
            "case:h14-rebuild",
            "rebuild",
            &root,
            "rebuild-content",
        );
        let commit = match store
            .commit_fenced_effect_prepared(
                wave14_prepare_pending(&store, "rebuild", &chain),
                std::process::id(),
            )
            .unwrap()
        {
            PreparedCommitOutcome::Prepared(commit) => commit,
            PreparedCommitOutcome::GrantInvalidated(_) => panic!("Grant invalidated"),
        };
        let prepared = match &commit.transition.payload {
            TransitionPayload::EffectPrepared { prepared } => prepared.clone(),
            _ => unreachable!(),
        };
        let fence = prepared.resource_fence.clone().unwrap();
        let expected = store
            .get_resource_control_state(&fence.resource_id)
            .unwrap()
            .unwrap();
        let case_generation = commit.state.generation;

        let first_event = store
            .list_resource_control_events(&fence.resource_id)
            .unwrap()
            .pop()
            .unwrap();
        let forged_fence = ResourceFence::issue(
            &expected.identity,
            expected.resource_epoch + 1,
            "case:unrelated",
            "operation:unrelated",
            "grant:unrelated",
            "effect:unrelated",
            std::process::id(),
            &LocalProcessIdentity::capture(std::process::id())
                .unwrap()
                .canonical_identity(),
            first_event.committed_at_unix_ms + 1,
        )
        .unwrap();
        let impossible = ResourceControlEvent::build(
            ResourceControlAction::Acquired,
            &expected.identity,
            &forged_fence,
            first_event.sequence + 1,
            first_event.committed_at_unix_ms + 1,
            Some(&first_event),
        )
        .unwrap();
        let mut invalid_txn = store.env.begin_rw_txn().unwrap();
        let invalid = store
            .put_resource_control_event_txn(&mut invalid_txn, &impossible)
            .expect_err("second acquisition while active is impossible");
        assert_eq!(invalid, "resource_history_invalid_acquisition");
        invalid_txn.abort();

        let mut delete_txn = store.env.begin_rw_txn().unwrap();
        delete_txn
            .del(
                store.resource_control_states_by_id,
                &resource_control_state_key(&fence.resource_id),
                None,
            )
            .unwrap();
        delete_txn.commit().unwrap();
        assert!(store
            .get_resource_control_state(&fence.resource_id)
            .unwrap()
            .is_none());
        let rebuilt = store
            .rebuild_resource_control_state(&fence.resource_id)
            .unwrap();
        assert_eq!(rebuilt, expected);
        assert_eq!(
            store
                .get_case_state("case:h14-rebuild")
                .unwrap()
                .unwrap()
                .generation,
            case_generation
        );
        println!(
            "h14_resource_rebuild: resource={} events=1 epoch={} active=true case_generation_unchanged={} invalid_append={}",
            fence.resource_id, rebuilt.resource_epoch, case_generation, invalid
        );
        drop(store);
        fs::remove_dir_all(path).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn wave14_process_signal_uses_same_authority_spine_and_exact_birth_fence() {
        let path = temp_store_path("wave14-process-carrier");
        let store = LmdbRecordStore::open(&path).unwrap();
        let owner = AuthenticatedPrincipal::for_test(14002);
        let tenant_id = "tenant:wave14-process";
        let case_id = "case:wave14-process";
        store
            .bootstrap_local_security(&owner, tenant_id, "organization:wave14", 1_400_002)
            .unwrap();
        let mut child = std::process::Command::new("sh")
            .arg("-c")
            .arg("while :; do sleep 1; done")
            .spawn()
            .expect("spawn test-owned process fixture");
        let pid = child.id();
        let principal_id = owner.projected_principal_id();
        store
            .create_tenant_case(&owner, tenant_id, case_id)
            .unwrap();
        let participant = store
            .commit_secured_transition(
                &owner,
                tenant_id,
                secured_pending(
                    "transition:w14:process:participant",
                    case_id,
                    1,
                    &principal_id,
                    TransitionPayload::ParticipantBound {
                        participant_id: "participant:model".to_string(),
                        role: "operation-proposer".to_string(),
                    },
                ),
                true,
            )
            .unwrap();
        let attachment = ResourceAttachmentState {
            attachment_id: "resource:process-fixture".to_string(),
            kind: ResourceKind::Process,
            allowed_write_prefix: String::new(),
            max_write_bytes: 0,
            policy_id: "policy:process-signal".to_string(),
            policy_owner_participant_id: "participant:model".to_string(),
            review_requirement: ReviewRequirement::Automatic,
            process_signal_actions: vec![ProcessSignalAction::Suspend],
        };
        let binding = LocalProcessBinding::capture(case_id, &attachment.attachment_id, pid)
            .expect("capture exact process birth");
        let mut attach = secured_pending(
            "transition:w14:process:resource",
            case_id,
            participant.state.generation,
            &principal_id,
            TransitionPayload::ResourceAttached {
                attachment: attachment.clone(),
            },
        );
        attach.causal_refs = vec!["participant:model".to_string()];
        store
            .commit_tenant_process_attachment(&owner, tenant_id, attach, &binding)
            .unwrap();
        let source = serde_json::to_vec(&serde_json::json!({
            "schema": POLICY_SOURCE_INPUT_SCHEMA,
            "policy_key": "wave14.process.signal",
            "source_version": "1",
            "owner_ref": "organization:wave14",
            "source_origin": {"source_system":"wave14-test","source_uri":"test://wave14/process"},
            "validity": {"mode":"unbounded"},
            "rules": [
                {"kind":"operation_restriction","rule_id":"allow","operation_kind":"process.signal","resource_kind":"process","effect":"allow","reason":"test-owned process action"},
                {"kind":"authority_requirement","rule_id":"proposer","operation_kind":"process.signal","resource_kind":"process","subject":"proposer","required_role":"operation-proposer","reason":"explicit Case role"},
                {"kind":"evidence_obligation","rule_id":"source","operation_kind":"process.signal","resource_kind":"process","obligation":"source_provenance","reason":"canonical model source"},
                {"kind":"evidence_obligation","rule_id":"pre","operation_kind":"process.signal","resource_kind":"process","obligation":"pre_observation","reason":"exact birth pre observation"},
                {"kind":"evidence_obligation","rule_id":"post","operation_kind":"process.signal","resource_kind":"process","obligation":"post_observation","reason":"truthful kernel result"}
            ]
        }))
        .unwrap();
        let global = compile_policy_source(&source).unwrap();
        let scoped = scope_policy_compilation(&global, tenant_id, "organization:wave14").unwrap();
        store
            .ingest_tenant_policy_compilation(&owner, tenant_id, &scoped)
            .unwrap();
        store
            .validate_tenant_policy_artifact(&owner, &scoped.artifact.artifact_id, "Wave14")
            .unwrap();
        store
            .publish_tenant_policy_artifact(&owner, &scoped.artifact.artifact_id, "Wave14")
            .unwrap();
        let state = store.get_case_state(case_id).unwrap().unwrap();
        store
            .bind_tenant_case_policy(
                &owner,
                case_id,
                &scoped.artifact.artifact_id,
                state.generation,
                "bind process authority",
            )
            .unwrap();
        let state = store.get_case_state(case_id).unwrap().unwrap();
        store
            .commit_secured_transition(
                &owner,
                tenant_id,
                secured_pending(
                    "transition:w14:process:provider",
                    case_id,
                    state.generation,
                    &principal_id,
                    TransitionPayload::ProviderAttached {
                        participant_id: "participant:model".to_string(),
                        provider_id: "provider:test".to_string(),
                        provider_kind: "openai_compatible".to_string(),
                        base_url: "http://127.0.0.1:1".to_string(),
                        model_id: "model:test".to_string(),
                        credential_ref: "env:TEST".to_string(),
                    },
                ),
                true,
            )
            .unwrap();
        let state = store.get_case_state(case_id).unwrap().unwrap();
        let lineage = test_provider_lineage(state.generation);
        commit_typed(
            &store,
            "transition:w14:process:invocation",
            case_id,
            state.generation,
            TransitionPayload::ProviderInvocationStarted {
                invocation_id: "invocation:w14:process".to_string(),
                participant_id: "participant:model".to_string(),
                provider_id: "provider:test".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:test".to_string(),
                semantic_lineage: Some(lineage.clone()),
            },
            None,
            vec![],
        );
        let state = store.get_case_state(case_id).unwrap().unwrap();
        commit_typed(
            &store,
            "transition:w14:process:result",
            case_id,
            state.generation,
            TransitionPayload::ProviderResultRecorded {
                result_id: "provider-result:w14:process".to_string(),
                invocation_id: "invocation:w14:process".to_string(),
                provider_id: "provider:test".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:test".to_string(),
                semantic_lineage: Some(lineage),
                output: "process proposal".to_string(),
            },
            None,
            vec!["invocation:w14:process".to_string()],
        );
        let state = store.get_case_state(case_id).unwrap().unwrap();
        let operation = normalize_process_signal_candidate(
            r#"{"schema":"yai.operation_proposal.process_signal.v1","operation":"process.signal","resource":"resource:process-fixture","action":"suspend"}"#,
            &NormalizationContext {
                case_id,
                participant_id: "participant:model",
                provider_result_id: "provider-result:w14:process",
                provider_invocation_id: "invocation:w14:process",
                case_generation: state.generation,
                resource: &attachment,
            },
            &binding.process,
        )
        .unwrap();
        commit_typed(
            &store,
            "transition:w14:process:operation",
            case_id,
            state.generation,
            TransitionPayload::OperationRecorded {
                operation: operation.clone(),
            },
            Some(operation.scope.clone()),
            operation.origin.causal_refs(),
        );
        let (decision, decision_commit) = store
            .derive_and_commit_policy_decision(case_id, &operation.operation_id)
            .unwrap();
        assert_eq!(decision.outcome, crate::effect::DecisionOutcome::Allow);
        let grant =
            issue_policy_execution_grant(&operation, &decision, decision_commit.state.generation)
                .unwrap();
        let basis = decision.decision_basis.as_ref().unwrap();
        let grant_commit = commit_typed(
            &store,
            "transition:w14:process:grant",
            case_id,
            decision_commit.state.generation,
            TransitionPayload::ExecutionGrantIssued {
                grant: grant.clone(),
            },
            Some(operation.scope.clone()),
            vec![
                operation.operation_id.clone(),
                decision.decision_id.clone(),
                basis.basis_id.clone(),
                basis.effective_policy_id.clone(),
            ],
        );
        let pre = observe_process(&binding, "observation:w14:process:pre");
        let prepared_intent = prepare_process_effect(&operation, &decision, &grant, pre).unwrap();
        let mut prepare = PendingTransition::new(
            "transition:w14:process:prepare",
            case_id,
            grant_commit.state.generation,
            TransitionSource::component("wave14-test"),
            TransitionPayload::ProcessEffectPrepared {
                prepared: prepared_intent.clone(),
            },
        );
        prepare.causal_refs = vec![
            operation.operation_id.clone(),
            decision.decision_id.clone(),
            grant.grant_id.clone(),
            prepared_intent
                .expected_pre_observation
                .observation_id
                .clone(),
        ];
        let prepare_commit = match store
            .commit_fenced_process_effect_prepared(prepare, std::process::id())
            .unwrap()
        {
            PreparedCommitOutcome::Prepared(commit) => commit,
            PreparedCommitOutcome::GrantInvalidated(_) => panic!("process Grant invalidated"),
        };
        let prepared = match prepare_commit.transition.payload {
            TransitionPayload::ProcessEffectPrepared { prepared } => prepared,
            _ => unreachable!(),
        };
        let fence = prepared.resource_fence.clone().unwrap();
        let mut reused_pid_binding = binding.clone();
        reused_pid_binding.process.start_ticks += 1;
        let reused = execute_fenced_process_signal(
            &store,
            &fence,
            &operation,
            &decision,
            &grant,
            &prepared,
            &prepare_commit.state,
            &reused_pid_binding,
        )
        .unwrap();
        assert_eq!(reused.outcome, EffectOutcome::Conflict);
        assert!(!reused.signal_attempted);
        assert!(!reused.syscall_accepted);
        let result = execute_fenced_process_signal(
            &store,
            &fence,
            &operation,
            &decision,
            &grant,
            &prepared,
            &prepare_commit.state,
            &binding,
        )
        .unwrap();
        assert!(result.signal_attempted);
        assert!(result.syscall_accepted);
        let indeterminate = commit_typed(
            &store,
            "transition:w14:process:indeterminate",
            case_id,
            prepare_commit.state.generation,
            TransitionPayload::ProcessEffectIndeterminate {
                effect_id: prepared.effect_id.clone(),
                reason: "crash acknowledgement window under qualification".to_string(),
                observation: Some(result.post_observation.clone()),
            },
            Some(operation.scope.clone()),
            vec![prepared.effect_id.clone()],
        );
        assert!(store
            .get_resource_control_state(&fence.resource_id)
            .unwrap()
            .unwrap()
            .active_lease
            .is_some());
        let receipt = build_process_effect_receipt(&prepared, &result);
        let mut final_transition = PendingTransition::new(
            "transition:w14:process:final",
            case_id,
            indeterminate.state.generation,
            TransitionSource::component("wave14-test"),
            TransitionPayload::ProcessEffectFinalized {
                effect_id: prepared.effect_id.clone(),
                observation: result.post_observation.clone(),
                receipt: receipt.clone(),
            },
        );
        final_transition.causal_refs = vec![prepared.effect_id.clone(), receipt.receipt_id.clone()];
        store
            .commit_fenced_effect_terminal(final_transition, &fence)
            .unwrap();
        assert!(store.verify_case_state(case_id).unwrap());
        println!(
            "wave14_process_carrier: fixture_pid={} boot_id={} start_ticks={} operation={} decision={} grant={} resource={} epoch={} fence={} signal={} syscall_accepted={} pid_reuse_signal_attempted=false indeterminate_retained_lease=true observed_state={:?} receipt={} finalized=true",
            binding.process.pid,
            binding.process.boot_id,
            binding.process.start_ticks,
            operation.operation_id,
            decision.decision_id,
            grant.grant_id,
            fence.resource_id,
            fence.resource_epoch,
            fence.fence_id,
            result.kernel_signal,
            result.syscall_accepted,
            result.post_observation.state,
            receipt.receipt_id
        );
        child.kill().ok();
        child.wait().ok();
        drop(store);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn wave15_definition_binding_human_input_condition_and_replay_are_deterministic() {
        let path = temp_store_path("wave15-workflow-kernel");
        let store = LmdbRecordStore::open(&path).expect("open workflow store");
        let owner = AuthenticatedPrincipal::for_test(15001);
        let bootstrap = store
            .bootstrap_local_security(&owner, "tenant:wave15", "organization:wave15", 1_500_001)
            .expect("bootstrap workflow Tenant");
        let opened = store
            .create_tenant_case(&owner, "tenant:wave15", "case:wave15-workflow")
            .expect("create workflow Case");
        let actor = store
            .commit_secured_transition(
                &owner,
                "tenant:wave15",
                secured_pending(
                    "transition:wave15-actor",
                    "case:wave15-workflow",
                    opened.state.generation,
                    &bootstrap.principal.principal_id,
                    TransitionPayload::ParticipantBound {
                        participant_id: "participant:operator".to_string(),
                        role: "workflow-input".to_string(),
                    },
                ),
                true,
            )
            .expect("bind workflow input participant");
        let link = crate::transition::PrincipalParticipantLink::new(
            "case:wave15-workflow",
            "tenant:wave15",
            &bootstrap.principal.principal_id,
            "participant:operator",
            &bootstrap.principal.principal_id,
            1_500_002,
        )
        .expect("build actor link");
        let mut link_pending = secured_pending(
            "transition:wave15-actor-link",
            "case:wave15-workflow",
            actor.state.generation,
            &bootstrap.principal.principal_id,
            TransitionPayload::ParticipantPrincipalLinked { link },
        );
        link_pending.causal_refs = vec![
            bootstrap.principal.principal_id.clone(),
            "participant:operator".to_string(),
        ];
        store
            .commit_secured_transition(&owner, "tenant:wave15", link_pending, true)
            .expect("link authenticated actor");

        let input = WorkflowDefinitionInput {
            schema: WORKFLOW_DEFINITION_SCHEMA.to_string(),
            tenant_id: "tenant:wave15".to_string(),
            workflow_key: "controlled-remediation".to_string(),
            declared_version: "1".to_string(),
            name: "Controlled remediation".to_string(),
            description: "Human data followed by a frozen branch".to_string(),
            nodes: vec![
                WorkflowNode {
                    node_id: "supplier-code".to_string(),
                    kind: WorkflowNodeKind::HumanInput {
                        actor_slot: "operator".to_string(),
                        prompt: "Enter supplier code".to_string(),
                        required_roles: vec!["workflow-input".to_string()],
                        input_kind: HumanInputKind::Text,
                        max_bytes: 64,
                    },
                },
                WorkflowNode {
                    node_id: "has-input".to_string(),
                    kind: WorkflowNodeKind::Condition {
                        predicate: WorkflowPredicate::NodeSatisfied {
                            node_id: "supplier-code".to_string(),
                        },
                    },
                },
            ],
            edges: vec![WorkflowEdge {
                from: "supplier-code".to_string(),
                to: "has-input".to_string(),
                kind: WorkflowEdgeKind::Always,
            }],
        };
        let definition = store
            .define_workflow(&owner, input.clone(), 1_500_003)
            .expect("define immutable workflow");
        let exact_retry = store
            .define_workflow(&owner, input.clone(), 1_500_004)
            .expect("exact semantic definition retry");
        assert_eq!(
            definition.workflow_definition_id,
            exact_retry.workflow_definition_id
        );
        let mut collision = input;
        collision.description = "changed bytes under same version".to_string();
        assert_eq!(
            store
                .define_workflow(&owner, collision, 1_500_005)
                .expect_err("changed same version must fail"),
            "workflow_definition_version_collision"
        );
        let bound = store
            .bind_case_workflow(
                &owner,
                "case:wave15-workflow",
                &definition.workflow_definition_id,
                vec![WorkflowExecutorBinding {
                    slot: "operator".to_string(),
                    participant_id: "participant:operator".to_string(),
                }],
                Vec::new(),
                1_500_006,
            )
            .expect("bind exact workflow");
        assert!(bound.state.workflow_binding.is_some());
        assert_eq!(
            store
                .workflow_status_authorized(&owner, "case:wave15-workflow")
                .unwrap()
                .nodes[0]
                .posture,
            WorkflowNodePosture::WaitingHumanInput
        );
        let input_commit = store
            .record_workflow_human_input(
                &owner,
                "case:wave15-workflow",
                "supplier-code",
                "SUP-42",
                1_500_007,
            )
            .expect("record authenticated human input");
        assert!(input_commit.state.reviews.is_empty());
        let complete = store
            .advance_workflow_passive_progress(&owner, "case:wave15-workflow", 8)
            .expect("resolve frozen Condition");
        assert!(complete.completed);
        assert_eq!(complete.satisfied_count, 2);
        let rebuilt_state = store
            .rebuild_case_state("case:wave15-workflow")
            .expect("rebuild Case workflow state");
        let history = store
            .list_case_transitions("case:wave15-workflow")
            .expect("read workflow history");
        let rebuilt = crate::workflow::resolve_workflow(
            &definition,
            rebuilt_state.workflow_binding.as_ref().unwrap(),
            &rebuilt_state,
            &history,
        )
        .expect("re-resolve from definition, binding, and Case history");
        assert_eq!(complete, rebuilt);
        assert!(store.verify_case_state("case:wave15-workflow").unwrap());
        drop(store);
        fs::remove_dir_all(path).unwrap();
    }

    fn h15_setup_case(
        store: &LmdbRecordStore,
        owner: &AuthenticatedPrincipal,
        tenant_id: &str,
        case_id: &str,
        suffix: &str,
    ) -> PathBuf {
        let root = setup_runtime_case(store, owner, tenant_id, case_id, suffix);
        let principal_id = owner.projected_principal_id();
        let state = store.get_case_state(case_id).unwrap().unwrap();
        let link = crate::transition::PrincipalParticipantLink::new(
            case_id,
            tenant_id,
            &principal_id,
            "participant:model",
            &principal_id,
            1_515_000,
        )
        .unwrap();
        let mut pending = secured_pending(
            &format!("transition:h15-link:{suffix}"),
            case_id,
            state.generation,
            &principal_id,
            TransitionPayload::ParticipantPrincipalLinked { link },
        );
        pending.causal_refs = vec![principal_id, "participant:model".to_string()];
        store
            .commit_secured_transition(owner, tenant_id, pending, true)
            .unwrap();
        root
    }

    fn h15_model_definition(tenant_id: &str, key: &str) -> WorkflowDefinitionInput {
        WorkflowDefinitionInput {
            schema: WORKFLOW_DEFINITION_SCHEMA.to_string(),
            tenant_id: tenant_id.to_string(),
            workflow_key: key.to_string(),
            declared_version: "1".to_string(),
            name: "H15 concurrent model work".to_string(),
            description: String::new(),
            nodes: vec![WorkflowNode {
                node_id: "analyze".to_string(),
                kind: WorkflowNodeKind::ModelWork {
                    executor_slot: "model".to_string(),
                    task: "analyze canonical Case state".to_string(),
                    output_contract: Default::default(),
                    completion: WorkflowPredicate::ExecutionProviderResult,
                    budgets: WorkflowBudgets::default(),
                    resource_slot: Some("workspace".to_string()),
                },
            }],
            edges: Vec::new(),
        }
    }

    fn h15_bind_model_workflow(
        store: &LmdbRecordStore,
        owner: &AuthenticatedPrincipal,
        case_id: &str,
        definition: &WorkflowDefinition,
    ) {
        store
            .bind_case_workflow(
                owner,
                case_id,
                &definition.workflow_definition_id,
                vec![WorkflowExecutorBinding {
                    slot: "model".to_string(),
                    participant_id: "participant:model".to_string(),
                }],
                vec![WorkflowResourceBinding {
                    slot: "workspace".to_string(),
                    attachment_id: "resource:workspace".to_string(),
                }],
                1_515_100,
            )
            .unwrap();
    }

    fn h15_start_runtime(
        store: &LmdbRecordStore,
        owner: &AuthenticatedPrincipal,
        token: &str,
        max_queued_total: usize,
    ) {
        store
            .acquire_runtime_instance(
                owner,
                &RuntimeInstanceAcquireRequest {
                    owner_pid: std::process::id(),
                    owner_token: token.to_string(),
                    now_unix_ms: 1_515_200,
                    lease_duration_ms: 60_000,
                    config: RuntimeInstanceConfig {
                        workers: 8,
                        max_active_per_tenant: 8,
                        max_queued_per_tenant: max_queued_total,
                        max_queued_total,
                    },
                },
                false,
            )
            .unwrap();
        store
            .activate_runtime_instance(owner, token, 1_515_201, 60_000, 0)
            .unwrap();
    }

    #[test]
    fn h15_eight_way_same_node_start_is_atomic_and_exactly_once() {
        let path = temp_store_path("h15-same-node-start");
        let store = LmdbRecordStore::open(&path).unwrap();
        let owner = AuthenticatedPrincipal::for_test(15101);
        store
            .bootstrap_local_security(&owner, "tenant:h15-start", "organization:h15", 1)
            .unwrap();
        let root = h15_setup_case(
            &store,
            &owner,
            "tenant:h15-start",
            "case:h15-start",
            "same-node",
        );
        let definition = store
            .define_workflow(
                &owner,
                h15_model_definition("tenant:h15-start", "same-node"),
                1_515_010,
            )
            .unwrap();
        h15_bind_model_workflow(&store, &owner, "case:h15-start", &definition);
        let token = "runtime-owner:h15-start";
        h15_start_runtime(&store, &owner, token, 16);

        let barrier = Arc::new(Barrier::new(8));
        let mut contenders = Vec::new();
        for index in 0..8 {
            let contender_path = path.clone();
            let contender_owner = owner.clone();
            let contender_barrier = Arc::clone(&barrier);
            contenders.push(thread::spawn(move || {
                let contender_store = LmdbRecordStore::open(&contender_path).unwrap();
                contender_barrier.wait();
                contender_store
                    .materialize_workflow_ready_work(
                        &contender_owner,
                        "runtime-owner:h15-start",
                        "case:h15-start",
                        "/tmp/h15-same-node.jsonl",
                        None,
                        1_515_300 + index,
                    )
                    .unwrap()
            }));
        }
        let outcomes = contenders
            .into_iter()
            .map(|contender| contender.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            outcomes
                .iter()
                .filter_map(Option::as_ref)
                .filter(|outcome| outcome.created)
                .count(),
            1
        );
        assert_eq!(
            outcomes
                .iter()
                .filter_map(Option::as_ref)
                .map(|outcome| outcome.item.work_id.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            1
        );
        let state = store.get_case_state("case:h15-start").unwrap().unwrap();
        assert_eq!(state.workflow_executions.len(), 1);
        let work = store.list_runtime_work_authorized(&owner).unwrap();
        assert_eq!(work.len(), 1);
        let transitions_before = store.list_case_transitions("case:h15-start").unwrap();
        for _ in 0..8 {
            store
                .workflow_status_authorized(&owner, "case:h15-start")
                .unwrap();
        }
        assert_eq!(
            transitions_before,
            store.list_case_transitions("case:h15-start").unwrap()
        );
        println!(
            "h15_same_node_start: contenders=8 canonical_starts={} work_items={} unique_work_ids=1 status_writes=0",
            state.workflow_executions.len(),
            work.len()
        );
        drop(store);
        fs::remove_dir_all(path).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn h15_queue_backpressure_prevents_orphan_workflow_start() {
        let path = temp_store_path("h15-workflow-backpressure");
        let store = LmdbRecordStore::open(&path).unwrap();
        let owner = AuthenticatedPrincipal::for_test(15102);
        store
            .bootstrap_local_security(&owner, "tenant:h15-backpressure", "organization:h15", 1)
            .unwrap();
        let root = h15_setup_case(
            &store,
            &owner,
            "tenant:h15-backpressure",
            "case:h15-backpressure",
            "backpressure",
        );
        let definition = store
            .define_workflow(
                &owner,
                h15_model_definition("tenant:h15-backpressure", "backpressure"),
                1_515_020,
            )
            .unwrap();
        h15_bind_model_workflow(&store, &owner, "case:h15-backpressure", &definition);
        h15_start_runtime(&store, &owner, "runtime-owner:h15-backpressure", 1);
        assert_eq!(
            store
                .materialize_workflow_ready_work(
                    &owner,
                    "runtime-owner:h15-backpressure",
                    "case:h15-backpressure",
                    "/tmp/h15-before-start.jsonl",
                    Some("workflow_before_start_commit"),
                    1_515_209,
                )
                .unwrap_err(),
            "workflow_failpoint_before_start_commit"
        );
        assert!(store
            .get_case_state("case:h15-backpressure")
            .unwrap()
            .unwrap()
            .workflow_executions
            .is_empty());
        assert!(store
            .list_runtime_work_authorized(&owner)
            .unwrap()
            .is_empty());
        store
            .submit_runtime_work(
                &owner,
                &RuntimeWorkSubmission {
                    request_id: "request:h15-capacity-holder".to_string(),
                    tenant_id: "tenant:h15-backpressure".to_string(),
                    case_id: "case:h15-backpressure".to_string(),
                    participant_id: "participant:model".to_string(),
                    attachment_id: "resource:workspace".to_string(),
                    journal_path: "/tmp/h15-capacity.jsonl".to_string(),
                    task: "capacity holder".to_string(),
                    budgets: runtime_budgets(),
                    failpoint: None,
                    now_unix_ms: 1_515_210,
                },
            )
            .unwrap();
        let result = store
            .materialize_workflow_ready_work(
                &owner,
                "runtime-owner:h15-backpressure",
                "case:h15-backpressure",
                "/tmp/h15-backpressure.jsonl",
                None,
                1_515_211,
            )
            .unwrap();
        assert!(result.is_none());
        let state = store
            .get_case_state("case:h15-backpressure")
            .unwrap()
            .unwrap();
        assert!(state.workflow_executions.is_empty());
        assert_eq!(
            store
                .workflow_status_authorized(&owner, "case:h15-backpressure")
                .unwrap()
                .ready_work
                .len(),
            1
        );
        drop(store);
        fs::remove_dir_all(path).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn h15_conflicting_human_input_and_passive_resolution_are_first_writer_wins() {
        let path = temp_store_path("h15-human-condition-race");
        let store = LmdbRecordStore::open(&path).unwrap();
        let owner = AuthenticatedPrincipal::for_test(15103);
        store
            .bootstrap_local_security(&owner, "tenant:h15-human", "organization:h15", 1)
            .unwrap();
        let root = h15_setup_case(
            &store,
            &owner,
            "tenant:h15-human",
            "case:h15-human",
            "human",
        );
        let definition = store
            .define_workflow(
                &owner,
                WorkflowDefinitionInput {
                    schema: WORKFLOW_DEFINITION_SCHEMA.to_string(),
                    tenant_id: "tenant:h15-human".to_string(),
                    workflow_key: "human-race".to_string(),
                    declared_version: "1".to_string(),
                    name: "Human race".to_string(),
                    description: String::new(),
                    nodes: vec![
                        WorkflowNode {
                            node_id: "input".to_string(),
                            kind: WorkflowNodeKind::HumanInput {
                                actor_slot: "model".to_string(),
                                prompt: "Supply bounded JSON".to_string(),
                                required_roles: vec!["operation-proposer".to_string()],
                                input_kind: HumanInputKind::Json,
                                max_bytes: 256,
                            },
                        },
                        WorkflowNode {
                            node_id: "condition".to_string(),
                            kind: WorkflowNodeKind::Condition {
                                predicate: WorkflowPredicate::NodeSatisfied {
                                    node_id: "input".to_string(),
                                },
                            },
                        },
                    ],
                    edges: vec![WorkflowEdge {
                        from: "input".to_string(),
                        to: "condition".to_string(),
                        kind: WorkflowEdgeKind::Always,
                    }],
                },
                1_515_030,
            )
            .unwrap();
        store
            .bind_case_workflow(
                &owner,
                "case:h15-human",
                &definition.workflow_definition_id,
                vec![WorkflowExecutorBinding {
                    slot: "model".to_string(),
                    participant_id: "participant:model".to_string(),
                }],
                Vec::new(),
                1_515_031,
            )
            .unwrap();
        let malformed_generation = store
            .get_case_state("case:h15-human")
            .unwrap()
            .unwrap()
            .generation;
        assert!(
            store
                .record_workflow_human_input(
                    &owner,
                    "case:h15-human",
                    "input",
                    "{malformed",
                    1_515_032,
                )
                .unwrap_err()
                .contains("workflow_human_input_json_invalid")
        );
        assert!(store
            .record_workflow_human_input(
                &owner,
                "case:h15-human",
                "input",
                r#"{"key":1,"key":2}"#,
                1_515_033,
            )
            .unwrap_err()
            .contains("duplicate_json_key"));
        assert_eq!(
            store
                .get_case_state("case:h15-human")
                .unwrap()
                .unwrap()
                .generation,
            malformed_generation
        );

        let barrier = Arc::new(Barrier::new(8));
        let mut contenders = Vec::new();
        for index in 0..8 {
            let contender_path = path.clone();
            let contender_owner = owner.clone();
            let contender_barrier = Arc::clone(&barrier);
            contenders.push(thread::spawn(move || {
                let contender_store = LmdbRecordStore::open(&contender_path).unwrap();
                contender_barrier.wait();
                contender_store.record_workflow_human_input(
                    &contender_owner,
                    "case:h15-human",
                    "input",
                    if index % 2 == 0 {
                        r#"{"value":"A"}"#
                    } else {
                        r#"{"value":"B"}"#
                    },
                    1_515_100 + index,
                )
            }));
        }
        let results = contenders
            .into_iter()
            .map(|contender| contender.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        let state_after_input = store.get_case_state("case:h15-human").unwrap().unwrap();
        assert_eq!(state_after_input.workflow_human_inputs.len(), 1);
        assert!(state_after_input.reviews.is_empty());

        let passive_barrier = Arc::new(Barrier::new(8));
        let mut passive = Vec::new();
        for _ in 0..8 {
            let contender_path = path.clone();
            let contender_owner = owner.clone();
            let contender_barrier = Arc::clone(&passive_barrier);
            passive.push(thread::spawn(move || {
                let contender_store = LmdbRecordStore::open(&contender_path).unwrap();
                contender_barrier.wait();
                contender_store.advance_workflow_passive_progress(
                    &contender_owner,
                    "case:h15-human",
                    8,
                )
            }));
        }
        for contender in passive {
            assert!(contender.join().unwrap().unwrap().completed);
        }
        let state = store.get_case_state("case:h15-human").unwrap().unwrap();
        assert_eq!(state.workflow_human_inputs.len(), 1);
        assert_eq!(state.workflow_conditions.len(), 1);
        assert!(state.workflow_conditions[0].result);
        println!(
            "h15_human_condition_race: contenders=8 accepted_inputs=1 conflicting_inputs_rejected=7 condition_results=1 review_actions=0"
        );
        drop(store);
        fs::remove_dir_all(path).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn h15_definition_publication_integrity_retention_and_restoration_are_exact() {
        let path = temp_store_path("h15-definition-integrity");
        let store = LmdbRecordStore::open(&path).unwrap();
        let owner = AuthenticatedPrincipal::for_test(15104);
        store
            .bootstrap_local_security(&owner, "tenant:h15-definition", "organization:h15", 1)
            .unwrap();
        let input = h15_model_definition("tenant:h15-definition", "concurrent-definition");
        let barrier = Arc::new(Barrier::new(8));
        let mut publishers = Vec::new();
        for index in 0..8 {
            let publisher_path = path.clone();
            let publisher_owner = owner.clone();
            let publisher_input = input.clone();
            let publisher_barrier = Arc::clone(&barrier);
            publishers.push(thread::spawn(move || {
                let publisher_store = LmdbRecordStore::open(&publisher_path).unwrap();
                publisher_barrier.wait();
                publisher_store.define_workflow(
                    &publisher_owner,
                    publisher_input,
                    1_515_400 + index,
                )
            }));
        }
        let definitions = publishers
            .into_iter()
            .map(|publisher| publisher.join().unwrap().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            definitions
                .iter()
                .map(|definition| definition.workflow_definition_id.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            1
        );
        assert_eq!(
            store
                .list_workflow_definitions_authorized(&owner, "tenant:h15-definition")
                .unwrap()
                .len(),
            1
        );
        let definition = definitions[0].clone();

        let collision_barrier = Arc::new(Barrier::new(2));
        let mut collisions = Vec::new();
        for index in 0..2 {
            let collision_path = path.clone();
            let collision_owner = owner.clone();
            let collision_barrier = Arc::clone(&collision_barrier);
            let mut collision_input = input.clone();
            collision_input.declared_version = "2".to_string();
            collision_input.description = format!("competing-content-{index}");
            collisions.push(thread::spawn(move || {
                let collision_store = LmdbRecordStore::open(&collision_path).unwrap();
                collision_barrier.wait();
                collision_store.define_workflow(
                    &collision_owner,
                    collision_input,
                    1_515_500 + index,
                )
            }));
        }
        let collision_results = collisions
            .into_iter()
            .map(|publisher| publisher.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            collision_results
                .iter()
                .filter(|result| result.is_ok())
                .count(),
            1
        );
        assert_eq!(
            collision_results
                .iter()
                .filter(|result| {
                    result
                        .as_ref()
                        .err()
                        .is_some_and(|error| error == "workflow_definition_version_collision")
                })
                .count(),
            1
        );

        let root = h15_setup_case(
            &store,
            &owner,
            "tenant:h15-definition",
            "case:h15-definition",
            "definition",
        );
        h15_bind_model_workflow(&store, &owner, "case:h15-definition", &definition);
        let expected = store
            .workflow_status_authorized(&owner, "case:h15-definition")
            .unwrap();
        let key = workflow_definition_key(&definition.workflow_definition_id);
        let original = {
            let txn = store.env.begin_ro_txn().unwrap();
            txn.get(store.workflow_definitions, &key).unwrap().to_vec()
        };
        {
            let mut txn = store.env.begin_rw_txn().unwrap();
            txn.del(store.workflow_definitions, &key, None).unwrap();
            txn.commit().unwrap();
        }
        assert_eq!(
            store
                .workflow_status_authorized(&owner, "case:h15-definition")
                .unwrap_err(),
            "bound_workflow_definition_missing"
        );
        {
            let mut txn = store.env.begin_rw_txn().unwrap();
            txn.put(
                store.workflow_definitions,
                &key,
                &original,
                WriteFlags::NO_OVERWRITE,
            )
            .unwrap();
            txn.commit().unwrap();
        }
        assert_eq!(
            expected,
            store
                .workflow_status_authorized(&owner, "case:h15-definition")
                .unwrap()
        );

        let mut corrupt = definition.clone();
        corrupt.description = "tampered without resealing".to_string();
        {
            let mut txn = store.env.begin_rw_txn().unwrap();
            let bytes = serde_json::to_vec(&corrupt).unwrap();
            txn.put(
                store.workflow_definitions,
                &key,
                &bytes,
                WriteFlags::empty(),
            )
            .unwrap();
            txn.commit().unwrap();
        }
        assert_eq!(
            store
                .workflow_status_authorized(&owner, "case:h15-definition")
                .unwrap_err(),
            "workflow_definition_content_identity_mismatch"
        );
        {
            let mut txn = store.env.begin_rw_txn().unwrap();
            txn.put(
                store.workflow_definitions,
                &key,
                &original,
                WriteFlags::empty(),
            )
            .unwrap();
            txn.commit().unwrap();
        }
        assert_eq!(
            expected,
            store
                .workflow_status_authorized(&owner, "case:h15-definition")
                .unwrap()
        );
        println!(
            "h15_definition_integrity: concurrent_exact_publishers=8 stored=1 version_collision_winners=1 missing=fail_closed corrupt=fail_closed exact_restore=equal"
        );
        drop(store);
        fs::remove_dir_all(path).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn h15_deterministic_proposal_and_operation_recovery_are_exactly_once() {
        let path = temp_store_path("h15-deterministic-recovery");
        let store = LmdbRecordStore::open(&path).unwrap();
        let owner = AuthenticatedPrincipal::for_test(15105);
        store
            .bootstrap_local_security(&owner, "tenant:h15-deterministic", "organization:h15", 1)
            .unwrap();
        let root = h15_setup_case(
            &store,
            &owner,
            "tenant:h15-deterministic",
            "case:h15-deterministic",
            "deterministic",
        );
        let definition = store
            .define_workflow(
                &owner,
                WorkflowDefinitionInput {
                    schema: WORKFLOW_DEFINITION_SCHEMA.to_string(),
                    tenant_id: "tenant:h15-deterministic".to_string(),
                    workflow_key: "deterministic-recovery".to_string(),
                    declared_version: "1".to_string(),
                    name: "Deterministic recovery".to_string(),
                    description: String::new(),
                    nodes: vec![WorkflowNode {
                        node_id: "apply".to_string(),
                        kind: WorkflowNodeKind::DeterministicWork {
                            proposer_slot: "model".to_string(),
                            operation: DeterministicOperationTemplate::FilesystemWrite {
                                resource_slot: "workspace".to_string(),
                                relative_path: "allowed/h15-recovery.txt".to_string(),
                                content: "one canonical deterministic proposal\n".to_string(),
                            },
                            completion: WorkflowPredicate::ExecutionEffectFinalized,
                        },
                    }],
                    edges: Vec::new(),
                },
                1_515_600,
            )
            .unwrap();
        h15_bind_model_workflow(&store, &owner, "case:h15-deterministic", &definition);
        h15_start_runtime(&store, &owner, "runtime-owner:h15-deterministic", 4);
        let work = store
            .materialize_workflow_ready_work(
                &owner,
                "runtime-owner:h15-deterministic",
                "case:h15-deterministic",
                "/tmp/h15-deterministic.jsonl",
                None,
                1_515_601,
            )
            .unwrap()
            .unwrap()
            .item;

        let first_proposal = store
            .record_workflow_deterministic_proposal(&owner, &work)
            .unwrap();
        let recovered_proposal = store
            .record_workflow_deterministic_proposal(&owner, &work)
            .unwrap();
        assert_eq!(first_proposal, recovered_proposal);
        let first_operation = store
            .record_workflow_deterministic_operation_from_proposal(&owner, &work, &first_proposal)
            .unwrap();
        let recovered_operation = store
            .record_workflow_deterministic_operation_from_proposal(
                &owner,
                &work,
                &recovered_proposal,
            )
            .unwrap();
        assert_eq!(first_operation, recovered_operation);
        let history = store
            .list_case_transitions("case:h15-deterministic")
            .unwrap();
        assert_eq!(
            history
                .iter()
                .filter(|transition| matches!(
                    &transition.payload,
                    TransitionPayload::WorkflowDeterministicProposalRecorded { .. }
                ))
                .count(),
            1
        );
        assert_eq!(
            history
                .iter()
                .filter(|transition| matches!(
                    &transition.payload,
                    TransitionPayload::OperationRecorded { .. }
                ))
                .count(),
            1
        );
        println!(
            "h15_deterministic_recovery: proposal_id={} operation_id={} proposal_count=1 operation_count=1 provider_invocations=0",
            first_proposal.proposal_id, first_operation.operation_id
        );
        drop(store);
        fs::remove_dir_all(path).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn h15_process_workflow_start_contender() {
        let Ok(store_path) = std::env::var("H15_WORKFLOW_CONTENDER_STORE") else {
            return;
        };
        let index = std::env::var("H15_WORKFLOW_CONTENDER_INDEX").unwrap();
        let control = PathBuf::from(std::env::var("H15_WORKFLOW_CONTENDER_CONTROL").unwrap());
        fs::write(control.join(format!("ready-{index}")), b"ready").unwrap();
        for _ in 0..2_000 {
            if control.join("go").exists() {
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }
        assert!(control.join("go").exists());
        let store = LmdbRecordStore::open(store_path).unwrap();
        let owner = AuthenticatedPrincipal::for_test(15106);
        let contender_number = index.parse::<u64>().unwrap();
        let owner_token = format!("runtime-owner:h15-process-start:{index}");
        let acquired = store.acquire_runtime_instance(
            &owner,
            &RuntimeInstanceAcquireRequest {
                owner_pid: std::process::id(),
                owner_token: owner_token.clone(),
                now_unix_ms: 1_515_710 + contender_number,
                lease_duration_ms: 60_000,
                config: RuntimeInstanceConfig {
                    workers: 1,
                    max_active_per_tenant: 1,
                    max_queued_per_tenant: 16,
                    max_queued_total: 16,
                },
            },
            false,
        );
        let (result, winner) = match acquired {
            Ok(_) => {
                store
                    .activate_runtime_instance(
                        &owner,
                        &owner_token,
                        1_515_810 + contender_number,
                        60_000,
                        0,
                    )
                    .unwrap();
                let outcome = store
                    .materialize_workflow_ready_work(
                        &owner,
                        &owner_token,
                        "case:h15-process-start",
                        "/tmp/h15-process-start.jsonl",
                        None,
                        1_515_910 + contender_number,
                    )
                    .unwrap();
                (
                    outcome.as_ref().map_or_else(
                        || "created=false\nwork_id=none\n".to_string(),
                        |submission| {
                            format!(
                                "created={}\nwork_id={}\n",
                                submission.created, submission.item.work_id
                            )
                        },
                    ),
                    true,
                )
            }
            Err(error) if error.contains("runtime_instance_active") => (
                format!("created=false\nwork_id=none\nadmission={error}\n"),
                false,
            ),
            Err(error) => panic!("unexpected contender admission error: {error}"),
        };
        fs::write(control.join(format!("result-{index}")), result).unwrap();
        if winner {
            for _ in 0..2_000 {
                if control.join("finish").exists() {
                    break;
                }
                thread::sleep(Duration::from_millis(2));
            }
            assert!(control.join("finish").exists());
        }
    }

    #[test]
    fn h15_eight_process_same_node_start_is_atomic_and_exactly_once() {
        let path = temp_store_path("h15-process-same-node-start");
        let control = temp_store_path("h15-process-same-node-control");
        fs::create_dir_all(&control).unwrap();
        let store = LmdbRecordStore::open(&path).unwrap();
        let owner = AuthenticatedPrincipal::for_test(15106);
        store
            .bootstrap_local_security(&owner, "tenant:h15-process-start", "organization:h15", 1)
            .unwrap();
        let root = h15_setup_case(
            &store,
            &owner,
            "tenant:h15-process-start",
            "case:h15-process-start",
            "process-start",
        );
        let definition = store
            .define_workflow(
                &owner,
                h15_model_definition("tenant:h15-process-start", "process-start"),
                1_515_700,
            )
            .unwrap();
        h15_bind_model_workflow(&store, &owner, "case:h15-process-start", &definition);

        let executable = std::env::current_exe().unwrap();
        let mut contenders = Vec::new();
        for index in 0..8 {
            contenders.push(
                std::process::Command::new(&executable)
                    .arg("--exact")
                    .arg("store::lmdb::tests::h15_process_workflow_start_contender")
                    .arg("--nocapture")
                    .env("H15_WORKFLOW_CONTENDER_STORE", &path)
                    .env("H15_WORKFLOW_CONTENDER_CONTROL", &control)
                    .env("H15_WORKFLOW_CONTENDER_INDEX", index.to_string())
                    .spawn()
                    .unwrap(),
            );
        }
        for _ in 0..2_000 {
            let ready = fs::read_dir(&control)
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name().to_string_lossy().starts_with("ready-"))
                .count();
            if ready == 8 {
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(
            fs::read_dir(&control)
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name().to_string_lossy().starts_with("ready-"))
                .count(),
            8
        );
        fs::write(control.join("go"), b"go").unwrap();
        for _ in 0..2_000 {
            let results = fs::read_dir(&control)
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name().to_string_lossy().starts_with("result-"))
                .count();
            if results == 8 {
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(
            fs::read_dir(&control)
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name().to_string_lossy().starts_with("result-"))
                .count(),
            8
        );
        fs::write(control.join("finish"), b"finish").unwrap();
        for contender in contenders {
            assert!(contender.wait_with_output().unwrap().status.success());
        }
        let results = (0..8)
            .map(|index| fs::read_to_string(control.join(format!("result-{index}"))).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            results
                .iter()
                .filter(|result| result.contains("created=true"))
                .count(),
            1
        );
        let state = store
            .get_case_state("case:h15-process-start")
            .unwrap()
            .unwrap();
        assert_eq!(state.workflow_executions.len(), 1);
        assert_eq!(store.list_runtime_work_authorized(&owner).unwrap().len(), 1);
        println!(
            "h15_process_same_node_start: processes=8 canonical_starts=1 work_items=1 provider_invocations=0"
        );
        drop(store);
        fs::remove_dir_all(path).unwrap();
        fs::remove_dir_all(control).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn h15_content_valid_progression_objects_cannot_bypass_rederivation() {
        let path = temp_store_path("h15-progression-forgery");
        let store = LmdbRecordStore::open(&path).unwrap();
        let owner = AuthenticatedPrincipal::for_test(15107);
        let tenant_id = "tenant:h15-forgery";
        let case_id = "case:h15-forgery";
        store
            .bootstrap_local_security(&owner, tenant_id, "organization:h15", 1)
            .unwrap();
        let root = h15_setup_case(&store, &owner, tenant_id, case_id, "forgery");
        let definition = store
            .define_workflow(
                &owner,
                h15_model_definition(tenant_id, "forgery"),
                1_515_800,
            )
            .unwrap();
        h15_bind_model_workflow(&store, &owner, case_id, &definition);
        h15_start_runtime(&store, &owner, "runtime-owner:h15-forgery", 4);
        let work = store
            .materialize_workflow_ready_work(
                &owner,
                "runtime-owner:h15-forgery",
                case_id,
                "/tmp/h15-forgery.jsonl",
                None,
                1_515_801,
            )
            .unwrap()
            .unwrap()
            .item;
        let workflow = work.workflow.as_ref().unwrap();
        let state = store.get_case_state(case_id).unwrap().unwrap();
        let principal_id = owner.projected_principal_id();
        let duplicate_execution = WorkflowNodeExecution {
            schema: WORKFLOW_NODE_EXECUTION_SCHEMA.to_string(),
            execution_id: workflow.workflow_execution_id.clone(),
            binding_id: workflow.workflow_binding_id.clone(),
            workflow_definition_id: workflow.workflow_definition_id.clone(),
            node_id: workflow.workflow_node_id.clone(),
            case_id: case_id.to_string(),
            started_at_generation: state.generation + 1,
            started_at_unix_ms: 1_515_802,
        };
        let duplicate_error = store
            .commit_secured_transition(
                &owner,
                tenant_id,
                secured_pending(
                    "transition:h15:forged-duplicate-start",
                    case_id,
                    state.generation,
                    &principal_id,
                    TransitionPayload::WorkflowNodeExecutionStarted {
                        execution: duplicate_execution,
                    },
                ),
                true,
            )
            .unwrap_err();
        assert_eq!(
            duplicate_error,
            "workflow_execution_readiness_rederivation_mismatch"
        );

        let state = store.get_case_state(case_id).unwrap().unwrap();
        let forged_satisfaction = WorkflowNodeSatisfaction {
            schema: WORKFLOW_NODE_SATISFACTION_SCHEMA.to_string(),
            satisfaction_id: "workflow-satisfaction:content-valid-forgery".to_string(),
            binding_id: workflow.workflow_binding_id.clone(),
            workflow_definition_id: workflow.workflow_definition_id.clone(),
            node_id: workflow.workflow_node_id.clone(),
            execution_id: Some(workflow.workflow_execution_id.clone()),
            predicate_digest: WorkflowPredicate::ExecutionProviderResult.digest().unwrap(),
            evaluated_at_generation: state.generation + 1,
            evidence_refs: vec!["provider-result:unrelated".to_string()],
        };
        let satisfaction_error = store
            .commit_secured_transition(
                &owner,
                tenant_id,
                secured_pending(
                    "transition:h15:forged-satisfaction",
                    case_id,
                    state.generation,
                    &principal_id,
                    TransitionPayload::WorkflowNodeSatisfied {
                        satisfaction: forged_satisfaction,
                    },
                ),
                true,
            )
            .unwrap_err();
        assert_eq!(
            satisfaction_error,
            "workflow_satisfaction_rederivation_mismatch"
        );
        let final_state = store.get_case_state(case_id).unwrap().unwrap();
        assert_eq!(final_state.generation, state.generation);
        assert!(final_state.workflow_satisfactions.is_empty());
        println!(
            "h15_progression_forgery: duplicate_start=rejected false_satisfaction=rejected fake_evidence=rejected generation_unchanged=true"
        );
        drop(store);
        fs::remove_dir_all(path).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn wave17_planpatch_adoption_is_case_local_stale_safe_and_replayable() {
        let path = temp_store_path("wave17-planpatch");
        let store = LmdbRecordStore::open(&path).unwrap();
        let owner = AuthenticatedPrincipal::for_test(17001);
        let tenant_id = "tenant:wave17-patch";
        store
            .bootstrap_local_security(&owner, tenant_id, "organization:wave17", 1)
            .unwrap();
        let root = h15_setup_case(&store, &owner, tenant_id, "case:wave17-patch", "w17-patch");
        let definition = store
            .define_workflow(
                &owner,
                WorkflowDefinitionInput {
                    schema: WORKFLOW_DEFINITION_SCHEMA.to_string(),
                    tenant_id: tenant_id.to_string(),
                    workflow_key: "adaptive".to_string(),
                    declared_version: "2".to_string(),
                    name: "Adaptive fixture".to_string(),
                    description: String::new(),
                    nodes: vec![WorkflowNode {
                        node_id: "human-boundary".to_string(),
                        kind: WorkflowNodeKind::HumanInput {
                            actor_slot: "model".to_string(),
                            prompt: "keep the workflow quiescent".to_string(),
                            required_roles: vec![],
                            input_kind: HumanInputKind::Text,
                            max_bytes: 64,
                        },
                    }],
                    edges: vec![],
                },
                1_700_010,
            )
            .unwrap();
        store
            .bind_case_workflow(
                &owner,
                "case:wave17-patch",
                &definition.workflow_definition_id,
                vec![WorkflowExecutorBinding {
                    slot: "model".to_string(),
                    participant_id: "participant:model".to_string(),
                }],
                vec![WorkflowResourceBinding {
                    slot: "workspace".to_string(),
                    attachment_id: "resource:workspace".to_string(),
                }],
                1_700_011,
            )
            .unwrap();
        let initial = store
            .workflow_status_authorized(&owner, "case:wave17-patch")
            .unwrap();
        let patch_a = store
            .propose_workflow_plan_patch_human(
                &owner,
                "case:wave17-patch",
                WorkflowPlanPatchInput {
                    schema: crate::workflow::WORKFLOW_PLAN_PATCH_SCHEMA.to_string(),
                    base_effective_topology_digest: initial.effective_topology_digest.clone(),
                    operations: vec![WorkflowPatchOperation::AddNode {
                        node: WorkflowNode {
                            node_id: "future-wait-a".to_string(),
                            kind: WorkflowNodeKind::Wait {
                                predicate: WorkflowPredicate::CaseLifecycle {
                                    lifecycle: CaseLifecycle::Open,
                                },
                            },
                        },
                    }],
                },
                1_700_012,
            )
            .unwrap()
            .state
            .workflow_plan_patches
            .last()
            .unwrap()
            .clone();
        let patch_b = store
            .propose_workflow_plan_patch_human(
                &owner,
                "case:wave17-patch",
                WorkflowPlanPatchInput {
                    schema: crate::workflow::WORKFLOW_PLAN_PATCH_SCHEMA.to_string(),
                    base_effective_topology_digest: initial.effective_topology_digest,
                    operations: vec![WorkflowPatchOperation::AddNode {
                        node: WorkflowNode {
                            node_id: "future-wait-b".to_string(),
                            kind: WorkflowNodeKind::Wait {
                                predicate: WorkflowPredicate::CaseLifecycle {
                                    lifecycle: CaseLifecycle::Open,
                                },
                            },
                        },
                    }],
                },
                1_700_013,
            )
            .unwrap()
            .state
            .workflow_plan_patches
            .last()
            .unwrap()
            .clone();
        store
            .validate_workflow_plan_patch_authorized(&owner, "case:wave17-patch", &patch_a.patch_id)
            .unwrap();
        store
            .adopt_workflow_plan_patch(&owner, "case:wave17-patch", &patch_a.patch_id, 1_700_014)
            .unwrap();
        assert_eq!(
            store
                .adopt_workflow_plan_patch(
                    &owner,
                    "case:wave17-patch",
                    &patch_b.patch_id,
                    1_700_015,
                )
                .unwrap_err(),
            "workflow_patch_stale"
        );
        let final_resolution = store
            .workflow_status_authorized(&owner, "case:wave17-patch")
            .unwrap();
        assert_eq!(final_resolution.effective_revision, 1);
        assert!(final_resolution
            .nodes
            .iter()
            .any(|node| node.node_id == "future-wait-a"));
        assert!(!final_resolution
            .nodes
            .iter()
            .any(|node| node.node_id == "future-wait-b"));
        let rebuilt = store.rebuild_case_state("case:wave17-patch").unwrap();
        assert_eq!(rebuilt.workflow_amendments.len(), 1);
        assert_eq!(
            final_resolution,
            store
                .workflow_status_authorized(&owner, "case:wave17-patch")
                .unwrap()
        );
        println!(
            "w17_planpatch: proposals=2 adopted=1 stale=1 revision={} topology_digest={}",
            final_resolution.effective_revision, final_resolution.effective_topology_digest
        );
        drop(store);
        fs::remove_dir_all(path).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn wave17_same_tenant_handoff_moves_data_without_authority() {
        let path = temp_store_path("wave17-handoff");
        let store = LmdbRecordStore::open(&path).unwrap();
        let owner = AuthenticatedPrincipal::for_test(17002);
        let tenant_id = "tenant:wave17-handoff";
        store
            .bootstrap_local_security(&owner, tenant_id, "organization:wave17", 1)
            .unwrap();
        let root_a = h15_setup_case(&store, &owner, tenant_id, "case:handoff-a", "w17-ha");
        let root_b = h15_setup_case(&store, &owner, tenant_id, "case:handoff-b", "w17-hb");
        let offer_commit = store
            .offer_case_handoff(
                &owner,
                "case:handoff-a",
                "case:handoff-b",
                HandoffData {
                    kind: crate::handoff::HandoffDataKind::Json,
                    value: "{\"task\":\"inspect\"}".to_string(),
                },
                vec!["operation-proposer".to_string()],
                1_700_100,
            )
            .unwrap();
        let offer = offer_commit.state.handoff_offers.last().unwrap().clone();
        assert_eq!(
            store
                .list_pending_case_handoffs_authorized(&owner, "case:handoff-b")
                .unwrap()
                .len(),
            1
        );
        let barrier = Arc::new(Barrier::new(8));
        let mut contenders = Vec::new();
        for index in 0..8 {
            let path = path.clone();
            let owner = owner.clone();
            let barrier = barrier.clone();
            let handoff_id = offer.handoff_id.clone();
            contenders.push(thread::spawn(move || {
                let store = LmdbRecordStore::open(&path).unwrap();
                barrier.wait();
                store.accept_case_handoff(
                    &owner,
                    "case:handoff-b",
                    "case:handoff-a",
                    &handoff_id,
                    "participant:model",
                    1_700_101 + index,
                )
            }));
        }
        let acceptances = contenders
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            acceptances.iter().filter(|result| result.is_ok()).count(),
            1
        );
        assert_eq!(
            store
                .record_case_handoff_result(
                    &owner,
                    "case:handoff-b",
                    &offer.handoff_id,
                    HandoffOutcome::Succeeded,
                    HandoffData {
                        kind: crate::handoff::HandoffDataKind::Json,
                        value: "{\"finding\":\"bounded\"}".to_string(),
                    },
                    vec!["transition:nonexistent-target-evidence".to_string()],
                    "participant:model",
                    1_700_102,
                )
                .unwrap_err(),
            "handoff_result_evidence_not_target_local: transition:nonexistent-target-evidence"
        );
        let acceptance_id = store
            .get_case_state("case:handoff-b")
            .unwrap()
            .unwrap()
            .handoff_acceptances[0]
            .acceptance_id
            .clone();
        store
            .record_case_handoff_result(
                &owner,
                "case:handoff-b",
                &offer.handoff_id,
                HandoffOutcome::Succeeded,
                HandoffData {
                    kind: crate::handoff::HandoffDataKind::Json,
                    value: "{\"finding\":\"bounded\"}".to_string(),
                },
                vec![acceptance_id],
                "participant:model",
                1_700_102,
            )
            .unwrap();
        let reconciled = store
            .reconcile_case_handoff(&owner, "case:handoff-a", &offer.handoff_id, 1_700_103)
            .unwrap();
        assert_eq!(reconciled.state.handoff_reconciliations.len(), 1);
        assert!(reconciled.state.grants.is_empty());
        assert!(reconciled.state.effects.is_empty());
        let target = store.get_case_state("case:handoff-b").unwrap().unwrap();
        assert_eq!(target.handoff_acceptances.len(), 1);
        assert_eq!(target.handoff_results.len(), 1);
        assert!(target.handoff_reconciliations.is_empty());
        assert!(store.verify_case_state("case:handoff-a").unwrap());
        assert!(store.verify_case_state("case:handoff-b").unwrap());
        println!(
            "w17_handoff: handoff_id={} acceptance_contenders=8 acceptance_winners=1 source_facts=offer,reconciliation target_facts=acceptance,result source_grants=0 source_effects=0",
            offer.handoff_id
        );
        drop(store);
        fs::remove_dir_all(path).unwrap();
        fs::remove_dir_all(root_a).unwrap();
        fs::remove_dir_all(root_b).unwrap();
    }

    #[test]
    fn wave17_model_planpatch_is_strict_candidate_and_cannot_self_adopt() {
        let path = temp_store_path("wave17-model-planpatch");
        let store = LmdbRecordStore::open(&path).unwrap();
        let owner = AuthenticatedPrincipal::for_test(17003);
        let tenant_id = "tenant:wave17-model";
        let case_id = "case:wave17-model";
        store
            .bootstrap_local_security(&owner, tenant_id, "organization:wave17", 1)
            .unwrap();
        let root = h15_setup_case(&store, &owner, tenant_id, case_id, "w17-model");
        let definition = store
            .define_workflow(
                &owner,
                WorkflowDefinitionInput {
                    schema: WORKFLOW_DEFINITION_SCHEMA.to_string(),
                    tenant_id: tenant_id.to_string(),
                    workflow_key: "model-adaptation".to_string(),
                    declared_version: "2".to_string(),
                    name: "Model adaptation".to_string(),
                    description: String::new(),
                    nodes: vec![
                        WorkflowNode {
                            node_id: "propose-plan".to_string(),
                            kind: WorkflowNodeKind::ModelWork {
                                executor_slot: "model".to_string(),
                                task: "return one strict PlanPatch JSON value".to_string(),
                                completion: WorkflowPredicate::ExecutionProviderResult,
                                budgets: WorkflowBudgets::default(),
                                resource_slot: Some("workspace".to_string()),
                                output_contract:
                                    crate::workflow::ModelWorkOutputContract::PlanPatch,
                            },
                        },
                        WorkflowNode {
                            node_id: "owner-boundary".to_string(),
                            kind: WorkflowNodeKind::HumanInput {
                                actor_slot: "model".to_string(),
                                prompt: "owner-controlled continuation".to_string(),
                                required_roles: vec![],
                                input_kind: HumanInputKind::Text,
                                max_bytes: 64,
                            },
                        },
                    ],
                    edges: vec![WorkflowEdge {
                        from: "propose-plan".to_string(),
                        to: "owner-boundary".to_string(),
                        kind: WorkflowEdgeKind::Always,
                    }],
                },
                1_700_200,
            )
            .unwrap();
        h15_bind_model_workflow(&store, &owner, case_id, &definition);
        let state = store.get_case_state(case_id).unwrap().unwrap();
        store
            .commit_secured_transition(
                &owner,
                tenant_id,
                secured_pending(
                    "transition:w17:model-provider",
                    case_id,
                    state.generation,
                    &owner.projected_principal_id(),
                    TransitionPayload::ProviderAttached {
                        participant_id: "participant:model".to_string(),
                        provider_id: "provider:fixture".to_string(),
                        provider_kind: "openai_compatible".to_string(),
                        base_url: "http://127.0.0.1:1".to_string(),
                        model_id: "model:fixture".to_string(),
                        credential_ref: "env:W17_TEST".to_string(),
                    },
                ),
                true,
            )
            .unwrap();
        let base = store.workflow_status_authorized(&owner, case_id).unwrap();
        h15_start_runtime(&store, &owner, "runtime-owner:w17-model", 4);
        let work = store
            .materialize_workflow_ready_work(
                &owner,
                "runtime-owner:w17-model",
                case_id,
                "/tmp/w17-model-planpatch.jsonl",
                None,
                1_700_201,
            )
            .unwrap()
            .unwrap()
            .item;
        let execution_id = work
            .workflow
            .as_ref()
            .unwrap()
            .workflow_execution_id
            .clone();
        let state = store.get_case_state(case_id).unwrap().unwrap();
        let malformed_invocation_id = "provider-invocation:w17-model-malformed";
        let malformed_lineage = test_provider_lineage(state.generation);
        commit_typed(
            &store,
            "transition:w17:model-malformed-invocation",
            case_id,
            state.generation,
            TransitionPayload::ProviderInvocationStarted {
                invocation_id: malformed_invocation_id.to_string(),
                participant_id: "participant:model".to_string(),
                provider_id: "provider:fixture".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:fixture".to_string(),
                semantic_lineage: Some(malformed_lineage.clone()),
            },
            None,
            vec![execution_id.clone()],
        );
        let state = store.get_case_state(case_id).unwrap().unwrap();
        let malformed_result_id = "provider-result:w17-model-malformed";
        commit_typed(
            &store,
            "transition:w17:model-malformed-result",
            case_id,
            state.generation,
            TransitionPayload::ProviderResultRecorded {
                result_id: malformed_result_id.to_string(),
                invocation_id: malformed_invocation_id.to_string(),
                provider_id: "provider:fixture".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:fixture".to_string(),
                semantic_lineage: Some(malformed_lineage),
                output: "I think we should add a node".to_string(),
            },
            None,
            vec![malformed_invocation_id.to_string(), execution_id.clone()],
        );
        assert!(store
            .propose_workflow_plan_patch_from_provider_result(
                &owner,
                case_id,
                malformed_result_id,
                1_700_201,
            )
            .unwrap_err()
            .starts_with("workflow_model_plan_patch_invalid"));
        assert!(store
            .get_case_state(case_id)
            .unwrap()
            .unwrap()
            .workflow_plan_patches
            .is_empty());
        let state = store.get_case_state(case_id).unwrap().unwrap();
        let forged_invocation_id = "provider-invocation:w17-model-forged-origin";
        let forged_lineage = test_provider_lineage(state.generation);
        commit_typed(
            &store,
            "transition:w17:model-forged-origin-invocation",
            case_id,
            state.generation,
            TransitionPayload::ProviderInvocationStarted {
                invocation_id: forged_invocation_id.to_string(),
                participant_id: "participant:model".to_string(),
                provider_id: "provider:fixture".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:fixture".to_string(),
                semantic_lineage: Some(forged_lineage.clone()),
            },
            None,
            vec![],
        );
        let state = store.get_case_state(case_id).unwrap().unwrap();
        let forged_result_id = "provider-result:w17-model-forged-origin";
        commit_typed(
            &store,
            "transition:w17:model-forged-origin-result",
            case_id,
            state.generation,
            TransitionPayload::ProviderResultRecorded {
                result_id: forged_result_id.to_string(),
                invocation_id: forged_invocation_id.to_string(),
                provider_id: "provider:fixture".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:fixture".to_string(),
                semantic_lineage: Some(forged_lineage),
                output: "{}".to_string(),
            },
            None,
            vec![forged_invocation_id.to_string(), execution_id.clone()],
        );
        assert_eq!(
            store
                .propose_workflow_plan_patch_from_provider_result(
                    &owner,
                    case_id,
                    forged_result_id,
                    1_700_201,
                )
                .unwrap_err(),
            "workflow_plan_patch_provider_execution_ambiguous"
        );
        let state = store.get_case_state(case_id).unwrap().unwrap();
        let invocation_id = "provider-invocation:w17-model";
        let lineage = test_provider_lineage(state.generation);
        commit_typed(
            &store,
            "transition:w17:model-invocation",
            case_id,
            state.generation,
            TransitionPayload::ProviderInvocationStarted {
                invocation_id: invocation_id.to_string(),
                participant_id: "participant:model".to_string(),
                provider_id: "provider:fixture".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:fixture".to_string(),
                semantic_lineage: Some(lineage.clone()),
            },
            None,
            vec![execution_id.clone()],
        );
        let patch_output = serde_json::to_string(&WorkflowPlanPatchInput {
            schema: crate::workflow::WORKFLOW_PLAN_PATCH_SCHEMA.to_string(),
            base_effective_topology_digest: base.effective_topology_digest,
            operations: vec![WorkflowPatchOperation::AddNode {
                node: WorkflowNode {
                    node_id: "model-added-wait".to_string(),
                    kind: WorkflowNodeKind::Wait {
                        predicate: WorkflowPredicate::CaseLifecycle {
                            lifecycle: CaseLifecycle::Open,
                        },
                    },
                },
            }],
        })
        .unwrap();
        let state = store.get_case_state(case_id).unwrap().unwrap();
        let result_id = "provider-result:w17-model";
        commit_typed(
            &store,
            "transition:w17:model-result",
            case_id,
            state.generation,
            TransitionPayload::ProviderResultRecorded {
                result_id: result_id.to_string(),
                invocation_id: invocation_id.to_string(),
                provider_id: "provider:fixture".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:fixture".to_string(),
                semantic_lineage: Some(lineage),
                output: patch_output,
            },
            None,
            vec![invocation_id.to_string(), execution_id.clone()],
        );
        let proposed = store
            .propose_workflow_plan_patch_from_provider_result(&owner, case_id, result_id, 1_700_202)
            .unwrap();
        let patch = proposed.state.workflow_plan_patches.last().unwrap().clone();
        let (output_contract, output_topology_digest) = store
            .workflow_model_output_contract_authorized(&owner, case_id, &execution_id)
            .unwrap();
        assert_eq!(
            output_contract,
            crate::workflow::ModelWorkOutputContract::PlanPatch
        );
        assert_eq!(output_topology_digest, patch.base_effective_topology_digest);
        let repeated = store
            .propose_workflow_plan_patch_from_provider_result(&owner, case_id, result_id, 1_700_299)
            .unwrap();
        assert_eq!(
            repeated.transition.transition_id,
            proposed.transition.transition_id
        );
        assert_eq!(repeated.state.workflow_plan_patches.len(), 1);
        assert!(matches!(
            patch.origin,
            WorkflowPlanPatchOrigin::ModelProviderResult { .. }
        ));
        assert!(proposed.state.workflow_amendments.is_empty());
        assert_eq!(
            store
                .adopt_workflow_plan_patch(&owner, case_id, &patch.patch_id, 1_700_203)
                .unwrap_err(),
            "workflow_amendment_requires_quiescent_boundary"
        );
        store
            .advance_workflow_passive_progress(&owner, case_id, 4)
            .unwrap();
        store
            .adopt_workflow_plan_patch(&owner, case_id, &patch.patch_id, 1_700_204)
            .unwrap();
        let final_state = store.get_case_state(case_id).unwrap().unwrap();
        assert_eq!(final_state.workflow_plan_patches.len(), 1);
        assert_eq!(final_state.workflow_amendments.len(), 1);
        println!(
            "w17_model_patch: provider_results=3 malformed_candidates=0 forged_origin_candidates=0 valid_candidates=1 duplicate_candidates=0 auto_adoptions=0 owner_adoptions=1 patch_id={}",
            patch.patch_id
        );
        drop(store);
        fs::remove_dir_all(path).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn wave17_eight_way_amendment_adoption_has_one_winner() {
        let path = temp_store_path("wave17-patch-race");
        let store = LmdbRecordStore::open(&path).unwrap();
        let owner = AuthenticatedPrincipal::for_test(17004);
        let tenant_id = "tenant:wave17-race";
        let case_id = "case:wave17-race";
        store
            .bootstrap_local_security(&owner, tenant_id, "organization:wave17", 1)
            .unwrap();
        store
            .create_tenant_case(&owner, tenant_id, case_id)
            .unwrap();
        let definition = store
            .define_workflow(
                &owner,
                WorkflowDefinitionInput {
                    schema: WORKFLOW_DEFINITION_SCHEMA.to_string(),
                    tenant_id: tenant_id.to_string(),
                    workflow_key: "race".to_string(),
                    declared_version: "2".to_string(),
                    name: "Patch race".to_string(),
                    description: String::new(),
                    nodes: vec![WorkflowNode {
                        node_id: "wait-close".to_string(),
                        kind: WorkflowNodeKind::Wait {
                            predicate: WorkflowPredicate::CaseLifecycle {
                                lifecycle: CaseLifecycle::Closed,
                            },
                        },
                    }],
                    edges: vec![],
                },
                1_700_300,
            )
            .unwrap();
        store
            .bind_case_workflow(
                &owner,
                case_id,
                &definition.workflow_definition_id,
                vec![],
                vec![],
                1_700_301,
            )
            .unwrap();
        let base = store
            .workflow_status_authorized(&owner, case_id)
            .unwrap()
            .effective_topology_digest;
        let mut patch_ids = Vec::new();
        for index in 0..8 {
            let commit = store
                .propose_workflow_plan_patch_human(
                    &owner,
                    case_id,
                    WorkflowPlanPatchInput {
                        schema: crate::workflow::WORKFLOW_PLAN_PATCH_SCHEMA.to_string(),
                        base_effective_topology_digest: base.clone(),
                        operations: vec![WorkflowPatchOperation::AddNode {
                            node: WorkflowNode {
                                node_id: format!("candidate-{index}"),
                                kind: WorkflowNodeKind::Wait {
                                    predicate: WorkflowPredicate::CaseLifecycle {
                                        lifecycle: CaseLifecycle::Open,
                                    },
                                },
                            },
                        }],
                    },
                    1_700_310 + index,
                )
                .unwrap();
            patch_ids.push(
                commit
                    .state
                    .workflow_plan_patches
                    .last()
                    .unwrap()
                    .patch_id
                    .clone(),
            );
        }
        let barrier = Arc::new(Barrier::new(8));
        let mut contenders = Vec::new();
        for (index, patch_id) in patch_ids.into_iter().enumerate() {
            let path = path.clone();
            let owner = owner.clone();
            let barrier = barrier.clone();
            contenders.push(thread::spawn(move || {
                let store = LmdbRecordStore::open(&path).unwrap();
                barrier.wait();
                store.adopt_workflow_plan_patch(
                    &owner,
                    case_id,
                    &patch_id,
                    1_700_400 + index as u64,
                )
            }));
        }
        let outcomes = contenders
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
        assert!(outcomes
            .iter()
            .filter_map(|result| result.as_ref().err())
            .all(|error| error == "workflow_patch_stale"));
        let state = store.get_case_state(case_id).unwrap().unwrap();
        assert_eq!(state.workflow_amendments.len(), 1);
        assert_eq!(
            store
                .workflow_status_authorized(&owner, case_id)
                .unwrap()
                .effective_revision,
            1
        );
        println!(
            "w17_patch_race: contenders=8 winners=1 stale=7 amendments={} revision=1",
            state.workflow_amendments.len()
        );
        drop(store);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn wave17_handoff_cycle_and_cross_tenant_edges_fail_closed() {
        let path = temp_store_path("wave17-handoff-negative");
        let store = LmdbRecordStore::open(&path).unwrap();
        let owner = AuthenticatedPrincipal::for_test(17005);
        store
            .bootstrap_local_security(&owner, "tenant:w17-a", "organization:a", 1)
            .unwrap();
        store
            .create_tenant_case(&owner, "tenant:w17-a", "case:cycle-a")
            .unwrap();
        store
            .create_tenant_case(&owner, "tenant:w17-a", "case:cycle-b")
            .unwrap();
        store
            .offer_case_handoff(
                &owner,
                "case:cycle-a",
                "case:cycle-b",
                HandoffData {
                    kind: crate::handoff::HandoffDataKind::Text,
                    value: "A waits for B".to_string(),
                },
                vec![],
                1_700_500,
            )
            .unwrap();
        assert_eq!(
            store
                .offer_case_handoff(
                    &owner,
                    "case:cycle-b",
                    "case:cycle-a",
                    HandoffData {
                        kind: crate::handoff::HandoffDataKind::Text,
                        value: "B waits for A".to_string(),
                    },
                    vec![],
                    1_700_501,
                )
                .unwrap_err(),
            "handoff_offer_rederivation_mismatch"
        );
        store
            .bootstrap_local_security(&owner, "tenant:w17-b", "organization:b", 2)
            .unwrap();
        store
            .create_tenant_case(&owner, "tenant:w17-a", "case:cross-source")
            .unwrap();
        store
            .create_tenant_case(&owner, "tenant:w17-b", "case:other-tenant")
            .unwrap();
        assert_eq!(
            store
                .offer_case_handoff(
                    &owner,
                    "case:cross-source",
                    "case:other-tenant",
                    HandoffData {
                        kind: crate::handoff::HandoffDataKind::Text,
                        value: "must not cross Tenant".to_string(),
                    },
                    vec![],
                    1_700_502,
                )
                .unwrap_err(),
            "handoff_offer_rederivation_mismatch"
        );
        assert!(store
            .get_case_state("case:other-tenant")
            .unwrap()
            .unwrap()
            .handoff_acceptances
            .is_empty());
        println!(
            "w17_handoff_negative: active_cycle=rejected cross_tenant=rejected target_payloads=0"
        );
        drop(store);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn wave17_subflow_progresses_inside_one_case_and_replays_without_run_owner() {
        let path = temp_store_path("wave17-subflow-progress");
        let store = LmdbRecordStore::open(&path).unwrap();
        let owner = AuthenticatedPrincipal::for_test(17006);
        let tenant_id = "tenant:wave17-subflow";
        let case_id = "case:wave17-subflow";
        store
            .bootstrap_local_security(&owner, tenant_id, "organization:wave17", 1)
            .unwrap();
        store
            .create_tenant_case(&owner, tenant_id, case_id)
            .unwrap();
        let child = store
            .define_workflow(
                &owner,
                WorkflowDefinitionInput {
                    schema: WORKFLOW_DEFINITION_SCHEMA.to_string(),
                    tenant_id: tenant_id.to_string(),
                    workflow_key: "child-passive".to_string(),
                    declared_version: "2".to_string(),
                    name: "Child passive".to_string(),
                    description: String::new(),
                    nodes: vec![WorkflowNode {
                        node_id: "child-ready".to_string(),
                        kind: WorkflowNodeKind::Wait {
                            predicate: WorkflowPredicate::CaseLifecycle {
                                lifecycle: CaseLifecycle::Open,
                            },
                        },
                    }],
                    edges: vec![],
                },
                1_700_600,
            )
            .unwrap();
        let parent = store
            .define_workflow(
                &owner,
                WorkflowDefinitionInput {
                    schema: WORKFLOW_DEFINITION_SCHEMA.to_string(),
                    tenant_id: tenant_id.to_string(),
                    workflow_key: "parent-passive".to_string(),
                    declared_version: "2".to_string(),
                    name: "Parent passive".to_string(),
                    description: String::new(),
                    nodes: vec![WorkflowNode {
                        node_id: "child-instance".to_string(),
                        kind: WorkflowNodeKind::Subflow {
                            workflow_definition_id: child.workflow_definition_id.clone(),
                            workflow_definition_digest: child.integrity_digest.clone(),
                            executor_slot_mapping: vec![],
                            resource_slot_mapping: vec![],
                            case_slot_mapping: vec![],
                        },
                    }],
                    edges: vec![],
                },
                1_700_601,
            )
            .unwrap();
        store
            .bind_case_workflow(
                &owner,
                case_id,
                &parent.workflow_definition_id,
                vec![],
                vec![],
                1_700_602,
            )
            .unwrap();
        let before = store.workflow_status_authorized(&owner, case_id).unwrap();
        assert_eq!(before.nodes.len(), 2);
        assert_eq!(before.ready_work.len(), 0);
        let complete = store
            .advance_workflow_passive_progress(&owner, case_id, 4)
            .unwrap();
        assert!(complete.completed);
        assert_eq!(complete.satisfied_count, 2);
        assert!(complete
            .nodes
            .iter()
            .any(|node| node.node_id == "root/child-instance/child-ready"));
        let before_rebuild_digest = complete.effective_topology_digest.clone();
        let rebuilt = store.rebuild_case_state(case_id).unwrap();
        assert_eq!(rebuilt.workflow_satisfactions.len(), 2);
        let replayed = store.workflow_status_authorized(&owner, case_id).unwrap();
        assert_eq!(complete, replayed);
        assert_eq!(replayed.effective_topology_digest, before_rebuild_digest);
        assert!(store
            .list_runtime_work_authorized(&owner)
            .unwrap()
            .is_empty());
        println!(
            "w17_subflow_progress: cases=1 definitions=2 qualified_nodes=2 work_items=0 completed=true digest={}",
            replayed.effective_topology_digest
        );
        drop(store);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn wave17_workflow_handoff_waits_worker_free_until_source_reconciliation() {
        let path = temp_store_path("wave17-workflow-handoff");
        let store = LmdbRecordStore::open(&path).unwrap();
        let owner = AuthenticatedPrincipal::for_test(17007);
        let tenant_id = "tenant:wave17-workflow-handoff";
        let source_case = "case:workflow-handoff-source";
        let target_case = "case:workflow-handoff-target";
        store
            .bootstrap_local_security(&owner, tenant_id, "organization:wave17", 1)
            .unwrap();
        let root_source = h15_setup_case(&store, &owner, tenant_id, source_case, "w17-whs");
        let root_target = h15_setup_case(&store, &owner, tenant_id, target_case, "w17-wht");
        let definition = store
            .define_workflow(
                &owner,
                WorkflowDefinitionInput {
                    schema: WORKFLOW_DEFINITION_SCHEMA.to_string(),
                    tenant_id: tenant_id.to_string(),
                    workflow_key: "handoff-node".to_string(),
                    declared_version: "2".to_string(),
                    name: "Handoff node".to_string(),
                    description: String::new(),
                    nodes: vec![WorkflowNode {
                        node_id: "delegate".to_string(),
                        kind: WorkflowNodeKind::Handoff {
                            target_case_slot: "target".to_string(),
                            request: HandoffData {
                                kind: crate::handoff::HandoffDataKind::Json,
                                value: "{\"task\":\"target-local-work\"}".to_string(),
                            },
                            required_target_roles: vec!["operation-proposer".to_string()],
                            completion: WorkflowPredicate::HandoffReconciled { outcome: None },
                        },
                    }],
                    edges: vec![],
                },
                1_700_700,
            )
            .unwrap();
        store
            .bind_case_workflow_composed(
                &owner,
                source_case,
                &definition.workflow_definition_id,
                vec![],
                vec![],
                vec![WorkflowCaseBinding {
                    slot: "target".to_string(),
                    case_id: target_case.to_string(),
                }],
                1_700_701,
            )
            .unwrap();
        let waiting = store
            .advance_workflow_passive_progress(&owner, source_case, 4)
            .unwrap();
        assert!(!waiting.completed);
        let source = store.get_case_state(source_case).unwrap().unwrap();
        assert_eq!(source.handoff_offers.len(), 1);
        assert!(store
            .list_runtime_work_authorized(&owner)
            .unwrap()
            .is_empty());
        let handoff_id = source.handoff_offers[0].handoff_id.clone();
        store
            .accept_case_handoff(
                &owner,
                target_case,
                source_case,
                &handoff_id,
                "participant:model",
                1_700_702,
            )
            .unwrap();
        store
            .record_case_handoff_result(
                &owner,
                target_case,
                &handoff_id,
                HandoffOutcome::Succeeded,
                HandoffData {
                    kind: crate::handoff::HandoffDataKind::Text,
                    value: "target-local result".to_string(),
                },
                vec![],
                "participant:model",
                1_700_703,
            )
            .unwrap();
        assert!(
            !store
                .workflow_status_authorized(&owner, source_case)
                .unwrap()
                .completed
        );
        store
            .reconcile_case_handoff(&owner, source_case, &handoff_id, 1_700_704)
            .unwrap();
        let completed = store
            .advance_workflow_passive_progress(&owner, source_case, 4)
            .unwrap();
        assert!(completed.completed);
        assert_eq!(completed.satisfied_count, 1);
        assert!(store
            .list_runtime_work_authorized(&owner)
            .unwrap()
            .is_empty());
        assert!(store.verify_case_state(source_case).unwrap());
        assert!(store.verify_case_state(target_case).unwrap());
        println!(
            "w17_workflow_handoff: offer=1 accept=1 result=1 reconcile=1 source_satisfaction=1 workers_held=0 handoff_id={handoff_id}"
        );
        drop(store);
        fs::remove_dir_all(path).unwrap();
        fs::remove_dir_all(root_source).unwrap();
        fs::remove_dir_all(root_target).unwrap();
    }

    #[test]
    fn wave17_handoff_terminal_case_matrix_preserves_both_histories() {
        let path = temp_store_path("wave17-handoff-terminal");
        let store = LmdbRecordStore::open(&path).unwrap();
        let owner = AuthenticatedPrincipal::for_test(17008);
        let tenant_id = "tenant:wave17-handoff-terminal";
        store
            .bootstrap_local_security(&owner, tenant_id, "organization:wave17", 1)
            .unwrap();

        let before_source = h15_setup_case(
            &store,
            &owner,
            tenant_id,
            "case:handoff-before-source",
            "w17-hbs",
        );
        let before_target = h15_setup_case(
            &store,
            &owner,
            tenant_id,
            "case:handoff-before-target",
            "w17-hbt",
        );
        let offer = store
            .offer_case_handoff(
                &owner,
                "case:handoff-before-source",
                "case:handoff-before-target",
                HandoffData {
                    kind: crate::handoff::HandoffDataKind::Text,
                    value: "not accepted before source cancellation".to_string(),
                },
                vec!["operation-proposer".to_string()],
                1_700_800,
            )
            .unwrap()
            .state
            .handoff_offers[0]
            .clone();
        store
            .cancel_tenant_case(
                &owner,
                "case:handoff-before-source",
                "source cancelled before acceptance",
            )
            .unwrap();
        assert!(store
            .accept_case_handoff(
                &owner,
                "case:handoff-before-target",
                "case:handoff-before-source",
                &offer.handoff_id,
                "participant:model",
                1_700_801,
            )
            .unwrap_err()
            .contains("handoff_acceptance_rederivation_mismatch"));

        let after_source = h15_setup_case(
            &store,
            &owner,
            tenant_id,
            "case:handoff-after-source",
            "w17-has",
        );
        let after_target = h15_setup_case(
            &store,
            &owner,
            tenant_id,
            "case:handoff-after-target",
            "w17-hat",
        );
        let accepted_offer = store
            .offer_case_handoff(
                &owner,
                "case:handoff-after-source",
                "case:handoff-after-target",
                HandoffData {
                    kind: crate::handoff::HandoffDataKind::Text,
                    value: "accepted target later cancelled".to_string(),
                },
                vec!["operation-proposer".to_string()],
                1_700_802,
            )
            .unwrap()
            .state
            .handoff_offers[0]
            .clone();
        store
            .accept_case_handoff(
                &owner,
                "case:handoff-after-target",
                "case:handoff-after-source",
                &accepted_offer.handoff_id,
                "participant:model",
                1_700_803,
            )
            .unwrap();
        store
            .cancel_tenant_case(
                &owner,
                "case:handoff-after-target",
                "target cancelled after acceptance",
            )
            .unwrap();
        let post_cancel_result = store.record_case_handoff_result(
            &owner,
            "case:handoff-after-target",
            &accepted_offer.handoff_id,
            HandoffOutcome::Succeeded,
            HandoffData {
                kind: crate::handoff::HandoffDataKind::Text,
                value: "forged success after cancellation".to_string(),
            },
            vec![],
            "participant:model",
            1_700_803,
        );
        assert_eq!(
            post_cancel_result.unwrap_err(),
            "handoff_result_target_case_terminal"
        );
        let reconciliation = store
            .reconcile_case_handoff(
                &owner,
                "case:handoff-after-source",
                &accepted_offer.handoff_id,
                1_700_804,
            )
            .unwrap();
        assert_eq!(
            reconciliation.state.handoff_reconciliations[0].outcome,
            HandoffOutcome::Cancelled
        );
        assert!(store
            .get_case_state("case:handoff-after-target")
            .unwrap()
            .unwrap()
            .handoff_results
            .is_empty());
        assert!(store
            .verify_case_state("case:handoff-before-source")
            .unwrap());
        assert!(store
            .verify_case_state("case:handoff-before-target")
            .unwrap());
        assert!(store
            .verify_case_state("case:handoff-after-source")
            .unwrap());
        assert!(store
            .verify_case_state("case:handoff-after-target")
            .unwrap());
        println!(
            "w17_handoff_terminal: source_cancel_before_accept=rejected target_cancel_after_accept=reconciled_cancelled target_results=0 histories_verified=4"
        );
        drop(store);
        fs::remove_dir_all(path).unwrap();
        for root in [before_source, before_target, after_source, after_target] {
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn wave17_four_case_handoff_chain_replays_without_process_owner() {
        let path = temp_store_path("wave17-handoff-chain");
        let store = LmdbRecordStore::open(&path).unwrap();
        let owner = AuthenticatedPrincipal::for_test(17009);
        let tenant_id = "tenant:wave17-chain";
        store
            .bootstrap_local_security(&owner, tenant_id, "organization:wave17", 1)
            .unwrap();
        let cases = [
            "case:chain-a",
            "case:chain-b",
            "case:chain-c",
            "case:chain-d",
        ];
        let roots = cases
            .iter()
            .enumerate()
            .map(|(index, case_id)| {
                h15_setup_case(
                    &store,
                    &owner,
                    tenant_id,
                    case_id,
                    &format!("w17-chain-{index}"),
                )
            })
            .collect::<Vec<_>>();
        let mut offers = Vec::new();
        for index in 0..3 {
            let offer = store
                .offer_case_handoff(
                    &owner,
                    cases[index],
                    cases[index + 1],
                    HandoffData {
                        kind: crate::handoff::HandoffDataKind::Json,
                        value: format!("{{\"step\":{index}}}"),
                    },
                    vec![],
                    1_701_000 + index as u64,
                )
                .unwrap()
                .state
                .handoff_offers
                .last()
                .unwrap()
                .clone();
            store
                .accept_case_handoff(
                    &owner,
                    cases[index + 1],
                    cases[index],
                    &offer.handoff_id,
                    "participant:model",
                    1_701_010 + index as u64,
                )
                .unwrap();
            offers.push(offer);
        }
        for index in (0..3).rev() {
            store
                .record_case_handoff_result(
                    &owner,
                    cases[index + 1],
                    &offers[index].handoff_id,
                    HandoffOutcome::Succeeded,
                    HandoffData {
                        kind: crate::handoff::HandoffDataKind::Json,
                        value: format!("{{\"completed_step\":{index}}}"),
                    },
                    vec![],
                    "participant:model",
                    1_701_020 + (2 - index) as u64,
                )
                .unwrap();
            store
                .reconcile_case_handoff(
                    &owner,
                    cases[index],
                    &offers[index].handoff_id,
                    1_701_030 + (2 - index) as u64,
                )
                .unwrap();
        }
        for case_id in cases {
            assert!(store.verify_case_state(case_id).unwrap());
            let rebuilt = store.rebuild_case_state(case_id).unwrap();
            assert_eq!(
                rebuilt,
                store.get_case_state(case_id).unwrap().unwrap(),
                "each Case must replay independently"
            );
        }
        let source = store.get_case_state(cases[0]).unwrap().unwrap();
        let middle_b = store.get_case_state(cases[1]).unwrap().unwrap();
        let middle_c = store.get_case_state(cases[2]).unwrap().unwrap();
        let target = store.get_case_state(cases[3]).unwrap().unwrap();
        assert_eq!(source.handoff_reconciliations.len(), 1);
        assert_eq!(middle_b.handoff_reconciliations.len(), 1);
        assert_eq!(middle_c.handoff_reconciliations.len(), 1);
        assert!(target.handoff_reconciliations.is_empty());
        assert!(cases.iter().all(|case_id| store
            .get_case_state(case_id)
            .unwrap()
            .unwrap()
            .grants
            .is_empty()));
        println!(
            "w17_handoff_chain: cases=4 edges=3 accepts=3 results=3 reconciliations=3 histories_replayed=4 process_owners=0 imported_grants=0"
        );
        drop(store);
        fs::remove_dir_all(path).unwrap();
        for root in roots {
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[path = "hardening17_tests.rs"]
    mod hardening17_tests;
}
