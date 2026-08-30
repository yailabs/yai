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
    issue_policy_execution_grant, validate_execution_obligation_closure,
    validate_execution_obligation_preparation, Decision, LocalFilesystemBinding, Operation,
    OperationOrigin, LOCAL_FILESYSTEM_BINDING_SCHEMA,
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
use crate::journal::Journal;
use crate::memory::{
    OperationalMemoryBuild, OperationalMemoryEntry, OperationalMemoryManifest,
    OPERATIONAL_MEMORY_DERIVATION, OPERATIONAL_MEMORY_MANIFEST_SCHEMA, OPERATIONAL_MEMORY_SCHEMA,
};
use crate::record::Record;
use crate::security::{
    AuthenticatedPrincipal, SecurityContext, SecurityEvent, SecurityEventAction, SecurityPrincipal,
    Tenant, TenantMembershipKind, SECURITY_EVENT_SCHEMA, SECURITY_PRINCIPAL_SCHEMA, TENANT_SCHEMA,
};
use crate::transition::{
    replay_case, AuthorityInvalidationReason, CaseCancellationState, CaseClosureState,
    CaseLifecycle, CaseState, ExecutionGrantInvalidation, GrantInvalidationDisposition,
    GrantLifecycle, PendingTransition, ReviewInvalidation, ReviewResolution, Transition,
    TransitionPayload, TransitionSource, CASE_STATE_SCHEMA, CASE_STATE_SCHEMA_V1,
    CASE_STATE_SCHEMA_V2, CASE_STATE_SCHEMA_V3, CASE_STATE_SCHEMA_V4, CASE_STATE_SCHEMA_V5,
    CASE_STATE_SCHEMA_V6, CASE_STATE_SCHEMA_V7, TRANSITION_SCHEMA, TRANSITION_SCHEMA_V1,
    TRANSITION_SCHEMA_V2, TRANSITION_SCHEMA_V3, TRANSITION_SCHEMA_V4, TRANSITION_SCHEMA_V5,
    TRANSITION_SCHEMA_V6, TRANSITION_SCHEMA_V7,
};
use lmdb::{
    Cursor, Database, DatabaseFlags, Environment, EnvironmentFlags, Error, RoTransaction,
    RwTransaction, Transaction, WriteFlags,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
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
    env: Environment,
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
        let env = Environment::new()
            .set_max_dbs(40)
            .set_map_size(map_size)
            .open(path)
            .map_err(|error| format!("failed to open LMDB env {}: {error}", path.display()))?;
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
        let blockers = closure_blockers(&state);
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
        let key = local_binding_key(case_id, attachment_id);
        let txn = self
            .env
            .begin_ro_txn()
            .map_err(|error| format!("failed to start local binding read: {error}"))?;
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
            "meta:local_filesystem_binding_schema",
            LOCAL_FILESYSTEM_BINDING_SCHEMA,
            &[],
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
        let mut builder = Environment::new();
        builder
            .set_max_dbs(32)
            .set_map_size(DEFAULT_LMDB_MAP_SIZE)
            .set_flags(EnvironmentFlags::READ_ONLY);
        let env = builder.open(path).map_err(|_| ())?;
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
    let end = rest
        .find(|ch: char| matches!(ch, ',' | '}'))
        .unwrap_or(rest.len());
    rest[..end].trim().parse::<usize>().unwrap_or(0)
}

fn json_u128_field(content: &str, key: &str) -> u128 {
    let marker = format!("\"{key}\":");
    let Some(start) = content.find(&marker).map(|index| index + marker.len()) else {
        return 0;
    };
    let rest = &content[start..];
    let end = rest
        .find(|ch: char| matches!(ch, ',' | '}'))
        .unwrap_or(rest.len());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admission::{
        evaluate_filesystem_admission, forged_evidence_resolution_for_test,
        CanonicalEvidenceResolution,
    };
    use crate::context::{RenderedInputMetadata, SemanticContextArtifact, RENDERED_INPUT_SCHEMA};
    use crate::effect::{
        build_effect_receipt, build_filesystem_review_request, classify_reconciliation,
        decide_filesystem_write, execute_filesystem_write, issue_execution_grant,
        issue_policy_execution_grant, normalize_filesystem_write_candidate, observe_filesystem,
        prepare_effect, reseal_policy_execution_grant_for_test, resolve_filesystem_review_decision,
        validate_finalized_effect_chain, CarrierFailpoint, CarrierResult, EffectOutcome,
        LocalFilesystemBinding, NormalizationContext, ReconciliationConclusion,
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
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Instant;
    use std::time::{SystemTime, UNIX_EPOCH};

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
}
