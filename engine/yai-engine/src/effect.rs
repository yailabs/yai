//! Controlled filesystem effect contract and carrier mechanics.
//!
//! This module owns the single `filesystem.write` normalization and resource
//! boundary proved by SOURCE.REFOUNDATION.3. It does not own provider
//! execution, policy languages, carrier registries, or any C semantic mirror.

use crate::admission::{DecisionBasis, ExecutionEvidenceRequirement};
use crate::resource_control::{
    LocalProcessIdentity, ResourceFence, ResourceFenceAuthority, PROCESS_IDENTITY_SCHEMA,
};
use crate::transition::{
    CaseState, ResourceAttachmentState, ReviewAction, ReviewActionKind, ReviewRequirement,
    ReviewResolution, ReviewState, Transition, TransitionPayload, TransitionScope,
    REVIEW_REQUEST_SCHEMA_V1,
};
use serde::{Deserialize, Serialize};
#[cfg(target_os = "linux")]
use std::ffi::CString;
use std::fmt::Write as FmtWrite;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
#[cfg(target_os = "linux")]
use std::os::fd::{AsRawFd, FromRawFd};
#[cfg(unix)]
use std::os::raw::c_int;
#[cfg(target_os = "linux")]
use std::os::unix::ffi::OsStrExt;
#[cfg(target_os = "linux")]
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
unsafe extern "C" {
    fn kill(pid: c_int, sig: c_int) -> c_int;
}

pub const OPERATION_PROPOSAL_SCHEMA: &str = "yai.operation_proposal.filesystem_write.v1";
pub const PROCESS_SIGNAL_PROPOSAL_SCHEMA: &str = "yai.operation_proposal.process_signal.v1";
pub const OPERATION_SCHEMA_V1: &str = "yai.operation.v1";
pub const OPERATION_SCHEMA: &str = "yai.operation.v2";
pub const DECISION_SCHEMA: &str = "yai.decision.v3";
pub const DECISION_SCHEMA_V2: &str = "yai.decision.v2";
pub const DECISION_SCHEMA_V1: &str = "yai.decision.v1";
pub const EXECUTION_GRANT_SCHEMA: &str = "yai.execution_grant.v3";
pub const EXECUTION_GRANT_SCHEMA_V2: &str = "yai.execution_grant.v2";
pub const EXECUTION_GRANT_SCHEMA_V1: &str = "yai.execution_grant.v1";
pub const OBSERVATION_SCHEMA: &str = "yai.observation.filesystem.v1";
pub const PREPARED_EFFECT_SCHEMA_V1: &str = "yai.prepared_effect.v1";
pub const PREPARED_EFFECT_SCHEMA: &str = "yai.prepared_effect.v2";
pub const EFFECT_RECEIPT_SCHEMA: &str = "yai.effect_receipt.v1";
pub const LOCAL_FILESYSTEM_BINDING_SCHEMA_V1: &str = "yai.local_filesystem_binding.v1";
pub const LOCAL_FILESYSTEM_BINDING_SCHEMA: &str = "yai.local_filesystem_binding.v2";
pub const FILESYSTEM_CARRIER_BACKEND: &str = "rust.filesystem.openat2_atomic_replace.v2";
pub const PROCESS_OBSERVATION_SCHEMA: &str = "yai.observation.process.v1";
pub const PREPARED_PROCESS_EFFECT_SCHEMA: &str = "yai.prepared_process_effect.v1";
pub const PROCESS_EFFECT_RECEIPT_SCHEMA: &str = "yai.process_effect_receipt.v1";
pub const LOCAL_PROCESS_BINDING_SCHEMA: &str = "yai.local_process_binding.v1";
pub const PROCESS_SIGNAL_CARRIER_BACKEND: &str = "rust.process.signal.v1";
pub const DEFAULT_MAX_WRITE_BYTES: usize = 65_536;
pub const MAX_EXECUTION_GRANT_LIFETIME_MS: u64 = 30_000;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilesystemWriteProposal {
    pub schema: String,
    pub operation: String,
    pub resource: String,
    pub path: String,
    pub content: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessSignalProposal {
    pub schema: String,
    pub operation: String,
    pub resource: String,
    pub action: ProcessSignalAction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizationContext<'a> {
    pub case_id: &'a str,
    pub participant_id: &'a str,
    pub provider_result_id: &'a str,
    pub provider_invocation_id: &'a str,
    pub case_generation: u64,
    pub resource: &'a ResourceAttachmentState,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NormalizationFailure {
    pub code: NormalizationFailureCode,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormalizationFailureCode {
    MalformedJson,
    UnsupportedSchema,
    UnsupportedOperation,
    AttachmentMismatch,
    InvalidRelativePath,
    EmptyContent,
    PayloadTooLarge,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    #[default]
    FilesystemWrite,
    ProcessSignal,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OperationOrigin {
    ProviderResult {
        provider_result_id: String,
        provider_invocation_id: String,
    },
    CompatibilityReview {
        review_id: String,
        attempt_id: String,
    },
}

impl OperationOrigin {
    pub fn causal_refs(&self) -> Vec<String> {
        match self {
            Self::ProviderResult {
                provider_result_id,
                provider_invocation_id,
            } => vec![provider_result_id.clone(), provider_invocation_id.clone()],
            Self::CompatibilityReview {
                review_id,
                attempt_id,
            } => vec![review_id.clone(), attempt_id.clone()],
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct FilesystemWritePayload {
    pub relative_path: String,
    pub content: String,
    pub content_digest: String,
    pub content_bytes: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessSignalAction {
    Terminate,
    Suspend,
    Resume,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessEffectRetryPosture {
    SafeToRepeat,
    ObservationOnlyRecovery,
    UnsafeOrAmbiguousToRepeat,
}

pub fn process_signal_retry_posture(action: &ProcessSignalAction) -> ProcessEffectRetryPosture {
    match action {
        ProcessSignalAction::Terminate => ProcessEffectRetryPosture::UnsafeOrAmbiguousToRepeat,
        ProcessSignalAction::Suspend | ProcessSignalAction::Resume => {
            ProcessEffectRetryPosture::ObservationOnlyRecovery
        }
    }
}

impl ProcessSignalAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Terminate => "terminate",
            Self::Suspend => "suspend",
            Self::Resume => "resume",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProcessSignalPayload {
    pub action: ProcessSignalAction,
    pub target_identity_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Operation {
    pub schema: String,
    pub operation_id: String,
    pub operation_digest: String,
    pub case_id: String,
    pub participant_id: String,
    pub scope: TransitionScope,
    pub kind: OperationKind,
    pub resource_attachment_id: String,
    pub filesystem_write: FilesystemWritePayload,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_signal: Option<ProcessSignalPayload>,
    pub origin: OperationOrigin,
    pub expected_case_generation: u64,
}

#[derive(Serialize)]
struct OperationDigestMaterial<'a> {
    schema: &'a str,
    case_id: &'a str,
    participant_id: &'a str,
    scope: &'a TransitionScope,
    kind: &'a OperationKind,
    resource_attachment_id: &'a str,
    filesystem_write: &'a FilesystemWritePayload,
    origin: &'a OperationOrigin,
    expected_case_generation: u64,
}

#[derive(Serialize)]
struct OperationDigestMaterialV2<'a> {
    schema: &'a str,
    case_id: &'a str,
    participant_id: &'a str,
    scope: &'a TransitionScope,
    kind: &'a OperationKind,
    resource_attachment_id: &'a str,
    process_signal: &'a ProcessSignalPayload,
    origin: &'a OperationOrigin,
    expected_case_generation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionOutcome {
    Allow,
    Deny,
    RequireReview,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DecisionSource {
    pub policy_id: String,
    pub owner_participant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Decision {
    pub schema: String,
    pub decision_id: String,
    pub decision_digest: String,
    pub operation_id: String,
    pub operation_digest: String,
    pub outcome: DecisionOutcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<DecisionSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_basis: Option<DecisionBasis>,
    pub reason: String,
    pub basis_refs: Vec<String>,
    pub decided_at_case_generation: u64,
}

#[derive(Serialize)]
struct DecisionDigestMaterial<'a> {
    schema: &'a str,
    operation_id: &'a str,
    operation_digest: &'a str,
    outcome: &'a DecisionOutcome,
    source: &'a DecisionSource,
    reason: &'a str,
    basis_refs: &'a [String],
    decided_at_case_generation: u64,
}

#[derive(Serialize)]
struct DecisionDigestMaterialV2<'a> {
    schema: &'a str,
    operation_id: &'a str,
    operation_digest: &'a str,
    outcome: &'a DecisionOutcome,
    decision_basis_id: &'a str,
    decision_basis_digest: &'a str,
    reason: &'a str,
    decided_at_case_generation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantedEffect {
    FilesystemWrite,
    ProcessSignal,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExecutionGrant {
    pub schema: String,
    pub grant_id: String,
    pub integrity_digest: String,
    pub operation_id: String,
    pub operation_digest: String,
    pub decision_id: String,
    pub decision_digest: String,
    pub case_id: String,
    pub participant_id: String,
    pub resource_attachment_id: String,
    pub permitted_effect: GrantedEffect,
    pub normalized_target: String,
    pub intended_content_digest: String,
    pub expected_case_generation: u64,
    pub idempotency_key: String,
    pub require_pre_observation: bool,
    pub require_post_observation: bool,
    #[serde(default)]
    pub issued_at_unix_ms: u64,
    #[serde(default)]
    pub expires_at_unix_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_basis_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_basis_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_policy_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_policy_digest: Option<String>,
    #[serde(default)]
    pub policy_binding_refs: Vec<String>,
    #[serde(default)]
    pub policy_artifact_refs: Vec<String>,
    #[serde(default)]
    pub execution_evidence_requirements: Vec<ExecutionEvidenceRequirement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_action_ref: Option<String>,
}

#[derive(Serialize)]
struct GrantDigestMaterial<'a> {
    schema: &'a str,
    operation_id: &'a str,
    operation_digest: &'a str,
    decision_id: &'a str,
    decision_digest: &'a str,
    case_id: &'a str,
    participant_id: &'a str,
    resource_attachment_id: &'a str,
    permitted_effect: &'a GrantedEffect,
    normalized_target: &'a str,
    intended_content_digest: &'a str,
    expected_case_generation: u64,
    idempotency_key: &'a str,
    require_pre_observation: bool,
    require_post_observation: bool,
}

#[derive(Serialize)]
struct GrantDigestMaterialV2<'a> {
    schema: &'a str,
    operation_id: &'a str,
    operation_digest: &'a str,
    decision_id: &'a str,
    decision_digest: &'a str,
    decision_basis_id: &'a str,
    decision_basis_digest: &'a str,
    effective_policy_id: &'a str,
    effective_policy_digest: &'a str,
    policy_binding_refs: &'a [String],
    policy_artifact_refs: &'a [String],
    case_id: &'a str,
    participant_id: &'a str,
    resource_attachment_id: &'a str,
    permitted_effect: &'a GrantedEffect,
    normalized_target: &'a str,
    intended_content_digest: &'a str,
    expected_case_generation: u64,
    idempotency_key: &'a str,
    require_pre_observation: bool,
    require_post_observation: bool,
    execution_evidence_requirements: &'a [ExecutionEvidenceRequirement],
    review_action_ref: &'a Option<String>,
}

#[derive(Serialize)]
struct GrantDigestMaterialV3<'a> {
    schema: &'a str,
    operation_id: &'a str,
    operation_digest: &'a str,
    decision_id: &'a str,
    decision_digest: &'a str,
    decision_basis_id: &'a str,
    decision_basis_digest: &'a str,
    effective_policy_id: &'a str,
    effective_policy_digest: &'a str,
    policy_binding_refs: &'a [String],
    policy_artifact_refs: &'a [String],
    case_id: &'a str,
    participant_id: &'a str,
    resource_attachment_id: &'a str,
    permitted_effect: &'a GrantedEffect,
    normalized_target: &'a str,
    intended_content_digest: &'a str,
    expected_case_generation: u64,
    idempotency_key: &'a str,
    require_pre_observation: bool,
    require_post_observation: bool,
    issued_at_unix_ms: u64,
    expires_at_unix_ms: u64,
    execution_evidence_requirements: &'a [ExecutionEvidenceRequirement],
    review_action_ref: &'a Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceState {
    Absent,
    File,
    Directory,
    Symlink,
    Other,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FilesystemObservation {
    pub schema: String,
    pub observation_id: String,
    pub resource_attachment_id: String,
    pub relative_path: String,
    pub state: ResourceState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub observed_at_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PreparedEffect {
    pub schema: String,
    pub effect_id: String,
    pub operation_id: String,
    pub decision_id: String,
    pub grant_id: String,
    pub case_id: String,
    pub participant_id: String,
    pub resource_attachment_id: String,
    pub relative_path: String,
    pub expected_pre_observation: FilesystemObservation,
    pub intended_content_digest: String,
    pub idempotency_key: String,
    pub carrier_backend: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_fence: Option<ResourceFence>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectOutcome {
    Applied,
    AlreadyApplied,
    NoEffect,
    FailedNoEffect,
    Conflict,
    Indeterminate,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconciliationConclusion {
    EffectObserved,
    NoEffectObserved,
    Conflict,
    StillIndeterminate,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EffectReceipt {
    pub schema: String,
    pub receipt_id: String,
    pub effect_id: String,
    pub operation_id: String,
    pub decision_id: String,
    pub grant_id: String,
    pub resource_attachment_id: String,
    pub relative_path: String,
    pub pre_observation_id: String,
    pub post_observation_id: String,
    pub outcome: EffectOutcome,
    pub carrier_backend: String,
    pub carrier_attempted: bool,
    pub mutation_performed: bool,
    pub completed_at_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalFilesystemBinding {
    pub schema: String,
    pub case_id: String,
    pub attachment_id: String,
    pub canonical_root: String,
    #[serde(default)]
    pub root_device: u64,
    #[serde(default)]
    pub root_inode: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalProcessBinding {
    pub schema: String,
    pub case_id: String,
    pub attachment_id: String,
    pub process: LocalProcessIdentity,
}

impl LocalProcessBinding {
    pub fn capture(
        case_id: impl Into<String>,
        attachment_id: impl Into<String>,
        pid: u32,
    ) -> Result<Self, String> {
        let binding = Self {
            schema: LOCAL_PROCESS_BINDING_SCHEMA.to_string(),
            case_id: case_id.into(),
            attachment_id: attachment_id.into(),
            process: LocalProcessIdentity::capture(pid)?,
        };
        binding.validate()?;
        Ok(binding)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != LOCAL_PROCESS_BINDING_SCHEMA
            || self.case_id.is_empty()
            || self.attachment_id.is_empty()
            || self.process.schema != PROCESS_IDENTITY_SCHEMA
        {
            return Err("invalid_local_process_binding".to_string());
        }
        self.process.validate()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessObservedState {
    Running,
    Sleeping,
    Stopped,
    Zombie,
    Exited,
    Other,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProcessObservation {
    pub schema: String,
    pub observation_id: String,
    pub resource_attachment_id: String,
    pub process_identity: LocalProcessIdentity,
    pub state: ProcessObservedState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub observed_at_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PreparedProcessEffect {
    pub schema: String,
    pub effect_id: String,
    pub operation_id: String,
    pub decision_id: String,
    pub grant_id: String,
    pub case_id: String,
    pub participant_id: String,
    pub resource_attachment_id: String,
    pub action: ProcessSignalAction,
    pub expected_pre_observation: ProcessObservation,
    pub target_identity_digest: String,
    pub idempotency_key: String,
    pub carrier_backend: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_fence: Option<ResourceFence>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProcessEffectReceipt {
    pub schema: String,
    pub receipt_id: String,
    pub effect_id: String,
    pub operation_id: String,
    pub decision_id: String,
    pub grant_id: String,
    pub resource_attachment_id: String,
    pub action: ProcessSignalAction,
    pub pre_observation_id: String,
    pub post_observation_id: String,
    pub kernel_signal: i32,
    pub syscall_accepted: bool,
    pub observed_state: ProcessObservedState,
    pub outcome: EffectOutcome,
    pub carrier_backend: String,
    pub completed_at_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessCarrierResult {
    pub outcome: EffectOutcome,
    pub post_observation: ProcessObservation,
    pub kernel_signal: i32,
    pub signal_attempted: bool,
    pub syscall_accepted: bool,
    pub detail: String,
}

impl ProcessObservation {
    pub fn validate(&self) -> Result<(), String> {
        self.process_identity.validate()?;
        if self.schema != PROCESS_OBSERVATION_SCHEMA
            || self.observation_id.is_empty()
            || self.resource_attachment_id.is_empty()
            || self.observed_at_unix_ms == 0
            || (self.state == ProcessObservedState::Unavailable && self.error.is_none())
        {
            return Err("invalid_process_observation".to_string());
        }
        Ok(())
    }
}

impl PreparedProcessEffect {
    pub fn validate(&self) -> Result<(), String> {
        self.expected_pre_observation.validate()?;
        let fence = self
            .resource_fence
            .as_ref()
            .ok_or_else(|| "prepared_process_effect_resource_fence_required".to_string())?;
        fence.validate_integrity()?;
        if self.schema != PREPARED_PROCESS_EFFECT_SCHEMA
            || self.effect_id.is_empty()
            || self.operation_id.is_empty()
            || self.decision_id.is_empty()
            || self.grant_id.is_empty()
            || self.case_id.is_empty()
            || self.participant_id.is_empty()
            || self.resource_attachment_id.is_empty()
            || self.target_identity_digest.is_empty()
            || self.idempotency_key.is_empty()
            || self.carrier_backend != PROCESS_SIGNAL_CARRIER_BACKEND
            || self.expected_pre_observation.resource_attachment_id != self.resource_attachment_id
            || digest_bytes(
                self.expected_pre_observation
                    .process_identity
                    .canonical_identity()
                    .as_bytes(),
            ) != self.target_identity_digest
            || fence.case_id != self.case_id
            || fence.operation_id != self.operation_id
            || fence.grant_id != self.grant_id
            || fence.effect_id != self.effect_id
        {
            return Err("invalid_prepared_process_effect".to_string());
        }
        Ok(())
    }
}

impl ProcessEffectReceipt {
    pub fn validate(&self, observation: &ProcessObservation) -> Result<(), String> {
        observation.validate()?;
        if self.schema != PROCESS_EFFECT_RECEIPT_SCHEMA
            || self.receipt_id.is_empty()
            || self.effect_id.is_empty()
            || self.post_observation_id != observation.observation_id
            || self.resource_attachment_id != observation.resource_attachment_id
            || self.observed_state != observation.state
            || (self.outcome == EffectOutcome::Applied && !self.syscall_accepted)
        {
            return Err("invalid_process_effect_receipt".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CarrierFailpoint {
    None,
    FailBeforeMutation,
    CrashAfterVisibleEffect,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CarrierResult {
    pub outcome: EffectOutcome,
    pub post_observation: FilesystemObservation,
    pub carrier_attempted: bool,
    pub mutation_performed: bool,
    pub crash_injected_after_effect: bool,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectChainValidation {
    pub effect_id: String,
    pub operation_id: String,
    pub decision_id: String,
    pub grant_id: String,
    pub receipt_id: String,
    pub outcome: EffectOutcome,
}

pub fn normalize_filesystem_write_candidate(
    raw_provider_output: &str,
    context: &NormalizationContext<'_>,
) -> Result<Operation, NormalizationFailure> {
    let proposal: FilesystemWriteProposal =
        serde_json::from_str(raw_provider_output).map_err(|error| NormalizationFailure {
            code: NormalizationFailureCode::MalformedJson,
            detail: format!("provider output is not the exact proposal object: {error}"),
        })?;
    if proposal.schema != OPERATION_PROPOSAL_SCHEMA {
        return Err(NormalizationFailure {
            code: NormalizationFailureCode::UnsupportedSchema,
            detail: format!("unsupported proposal schema: {}", proposal.schema),
        });
    }
    if proposal.operation != "filesystem.write" {
        return Err(NormalizationFailure {
            code: NormalizationFailureCode::UnsupportedOperation,
            detail: format!("unsupported operation: {}", proposal.operation),
        });
    }
    if proposal.resource != context.resource.attachment_id {
        return Err(NormalizationFailure {
            code: NormalizationFailureCode::AttachmentMismatch,
            detail: format!(
                "proposal resource {} is not attached resource {}",
                proposal.resource, context.resource.attachment_id
            ),
        });
    }
    let relative_path =
        normalize_relative_path(&proposal.path).map_err(|detail| NormalizationFailure {
            code: NormalizationFailureCode::InvalidRelativePath,
            detail,
        })?;
    if proposal.content.is_empty() {
        return Err(NormalizationFailure {
            code: NormalizationFailureCode::EmptyContent,
            detail: "filesystem.write content must not be empty".to_string(),
        });
    }
    let content_bytes = proposal.content.len();
    if content_bytes > context.resource.max_write_bytes {
        return Err(NormalizationFailure {
            code: NormalizationFailureCode::PayloadTooLarge,
            detail: format!(
                "payload is {content_bytes} bytes; attachment limit is {}",
                context.resource.max_write_bytes
            ),
        });
    }
    let content_digest = digest_bytes(proposal.content.as_bytes());
    let filesystem_write = FilesystemWritePayload {
        relative_path: relative_path.clone(),
        content: proposal.content,
        content_digest,
        content_bytes,
    };
    let scope = TransitionScope {
        case_id: context.case_id.to_string(),
        participant_refs: vec![context.participant_id.to_string()],
        resource_refs: vec![context.resource.attachment_id.clone()],
        policy_refs: vec![context.resource.policy_id.clone()],
    };
    Ok(build_filesystem_write_operation(
        context.case_id,
        context.participant_id,
        context.case_generation,
        context.resource,
        scope,
        filesystem_write,
        OperationOrigin::ProviderResult {
            provider_result_id: context.provider_result_id.to_string(),
            provider_invocation_id: context.provider_invocation_id.to_string(),
        },
    ))
}

fn build_filesystem_write_operation(
    case_id: &str,
    participant_id: &str,
    case_generation: u64,
    resource: &ResourceAttachmentState,
    scope: TransitionScope,
    filesystem_write: FilesystemWritePayload,
    origin: OperationOrigin,
) -> Operation {
    let material = OperationDigestMaterial {
        schema: OPERATION_SCHEMA_V1,
        case_id,
        participant_id,
        scope: &scope,
        kind: &OperationKind::FilesystemWrite,
        resource_attachment_id: &resource.attachment_id,
        filesystem_write: &filesystem_write,
        origin: &origin,
        expected_case_generation: case_generation,
    };
    let operation_digest = digest_serialized(&material);
    Operation {
        schema: OPERATION_SCHEMA_V1.to_string(),
        operation_id: format!("operation:{}", digest_suffix(&operation_digest, 32)),
        operation_digest,
        case_id: case_id.to_string(),
        participant_id: participant_id.to_string(),
        scope,
        kind: OperationKind::FilesystemWrite,
        resource_attachment_id: resource.attachment_id.clone(),
        filesystem_write,
        process_signal: None,
        origin,
        expected_case_generation: case_generation,
    }
}

pub fn normalize_process_signal_candidate(
    raw_provider_output: &str,
    context: &NormalizationContext<'_>,
    process: &LocalProcessIdentity,
) -> Result<Operation, NormalizationFailure> {
    let proposal: ProcessSignalProposal =
        serde_json::from_str(raw_provider_output).map_err(|error| NormalizationFailure {
            code: NormalizationFailureCode::MalformedJson,
            detail: format!("provider output is not the exact process proposal object: {error}"),
        })?;
    if proposal.schema != PROCESS_SIGNAL_PROPOSAL_SCHEMA {
        return Err(NormalizationFailure {
            code: NormalizationFailureCode::UnsupportedSchema,
            detail: format!("unsupported proposal schema: {}", proposal.schema),
        });
    }
    if proposal.operation != "process.signal" {
        return Err(NormalizationFailure {
            code: NormalizationFailureCode::UnsupportedOperation,
            detail: format!("unsupported operation: {}", proposal.operation),
        });
    }
    if proposal.resource != context.resource.attachment_id {
        return Err(NormalizationFailure {
            code: NormalizationFailureCode::AttachmentMismatch,
            detail: "process proposal does not target the attached resource".to_string(),
        });
    }
    if context.resource.kind != crate::transition::ResourceKind::Process
        || !context
            .resource
            .process_signal_actions
            .contains(&proposal.action)
    {
        return Err(NormalizationFailure {
            code: NormalizationFailureCode::UnsupportedOperation,
            detail: "process signal action is not admitted by the attachment".to_string(),
        });
    }
    process.validate().map_err(|detail| NormalizationFailure {
        code: NormalizationFailureCode::AttachmentMismatch,
        detail,
    })?;
    let target_identity_digest = digest_bytes(process.canonical_identity().as_bytes());
    let payload = ProcessSignalPayload {
        action: proposal.action,
        target_identity_digest,
    };
    let scope = TransitionScope {
        case_id: context.case_id.to_string(),
        participant_refs: vec![context.participant_id.to_string()],
        resource_refs: vec![context.resource.attachment_id.clone()],
        policy_refs: vec![context.resource.policy_id.clone()],
    };
    let origin = OperationOrigin::ProviderResult {
        provider_result_id: context.provider_result_id.to_string(),
        provider_invocation_id: context.provider_invocation_id.to_string(),
    };
    let material = OperationDigestMaterialV2 {
        schema: OPERATION_SCHEMA,
        case_id: context.case_id,
        participant_id: context.participant_id,
        scope: &scope,
        kind: &OperationKind::ProcessSignal,
        resource_attachment_id: &context.resource.attachment_id,
        process_signal: &payload,
        origin: &origin,
        expected_case_generation: context.case_generation,
    };
    let operation_digest = digest_serialized(&material);
    Ok(Operation {
        schema: OPERATION_SCHEMA.to_string(),
        operation_id: format!("operation:{}", digest_suffix(&operation_digest, 32)),
        operation_digest,
        case_id: context.case_id.to_string(),
        participant_id: context.participant_id.to_string(),
        scope,
        kind: OperationKind::ProcessSignal,
        resource_attachment_id: context.resource.attachment_id.clone(),
        filesystem_write: FilesystemWritePayload::default(),
        process_signal: Some(payload),
        origin,
        expected_case_generation: context.case_generation,
    })
}

pub fn decide_filesystem_write(
    operation: &Operation,
    resource: &ResourceAttachmentState,
    current_case_generation: u64,
) -> Decision {
    let allowed = path_within_prefix(
        &resource.allowed_write_prefix,
        &operation.filesystem_write.relative_path,
    );
    let outcome = if !allowed {
        DecisionOutcome::Deny
    } else if resource.review_requirement == ReviewRequirement::RequireReview {
        DecisionOutcome::RequireReview
    } else {
        DecisionOutcome::Allow
    };
    let reason = if !allowed {
        "normalized target is outside the attachment write prefix"
    } else if outcome == DecisionOutcome::RequireReview {
        "attachment policy requires an eligible human participant review"
    } else {
        "normalized target is inside the attachment write prefix"
    };
    build_filesystem_decision(
        operation,
        resource,
        current_case_generation,
        outcome,
        reason,
    )
}

pub fn record_filesystem_decision(
    operation: &Operation,
    resource: &ResourceAttachmentState,
    current_case_generation: u64,
    outcome: DecisionOutcome,
    reason: &str,
) -> Result<Decision, String> {
    if reason.is_empty() {
        return Err("decision reason must not be empty".to_string());
    }
    if outcome != DecisionOutcome::Deny
        && !path_within_prefix(
            &resource.allowed_write_prefix,
            &operation.filesystem_write.relative_path,
        )
    {
        return Err("allow decision cannot exceed attachment write prefix".to_string());
    }
    Ok(build_filesystem_decision(
        operation,
        resource,
        current_case_generation,
        outcome,
        reason,
    ))
}

pub fn build_filesystem_review_request(
    operation: &Operation,
    initial_decision: &Decision,
    resource: &ResourceAttachmentState,
    current_case_generation: u64,
) -> Result<ReviewState, String> {
    validate_decision(
        operation,
        initial_decision,
        initial_decision.decided_at_case_generation,
    )?;
    if initial_decision.outcome != DecisionOutcome::RequireReview
        || resource.review_requirement != ReviewRequirement::RequireReview
        || initial_decision
            .source
            .as_ref()
            .is_none_or(|source| source.policy_id != resource.policy_id)
        || initial_decision.source.as_ref().is_none_or(|source| {
            source.owner_participant_id != resource.policy_owner_participant_id
        })
        || current_case_generation != initial_decision.decided_at_case_generation + 1
    {
        return Err("review_request_requires_committed_review_decision".to_string());
    }
    let material = serde_json::json!({
        "schema": REVIEW_REQUEST_SCHEMA_V1,
        "case_id": operation.case_id,
        "operation_id": operation.operation_id,
        "operation_digest": operation.operation_digest,
        "initial_decision_id": initial_decision.decision_id,
        "resource_attachment_id": resource.attachment_id,
        "normalized_target": operation.filesystem_write.relative_path,
        "requesting_participant": operation.participant_id,
        "eligible_reviewer": resource.policy_owner_participant_id,
        "created_at_generation": current_case_generation,
    });
    let digest = digest_bytes(material.to_string().as_bytes());
    Ok(ReviewState {
        review_id: format!("review:{}", &digest[..32]),
        schema: REVIEW_REQUEST_SCHEMA_V1.to_string(),
        integrity_digest: String::new(),
        case_id: operation.case_id.clone(),
        operation_id: operation.operation_id.clone(),
        operation_digest: operation.operation_digest.clone(),
        initial_decision_id: initial_decision.decision_id.clone(),
        decision_basis_id: String::new(),
        decision_basis_digest: String::new(),
        effective_policy_id: String::new(),
        effective_policy_digest: String::new(),
        policy_binding_refs: Vec::new(),
        policy_artifact_refs: Vec::new(),
        required_reviewer_roles: Vec::new(),
        resource_attachment_id: resource.attachment_id.clone(),
        normalized_target: operation.filesystem_write.relative_path.clone(),
        created_at_generation: current_case_generation,
        latest_action_id: None,
        effective_decision_id: None,
        invalidation_reason: None,
        invalidation_source_ref: None,
        invalidated_at_unix_ms: None,
        attempt_id: String::new(),
        requested_by_participant: operation.participant_id.clone(),
        target_participant: String::new(),
        reviewer_participant: resource.policy_owner_participant_id.clone(),
        operation_kind: String::new(),
        carrier_family: String::new(),
        target_display: String::new(),
        sandbox_path: String::new(),
        target_path: String::new(),
        policy_reason: initial_decision.reason.clone(),
        status: ReviewResolution::Pending,
        carrier_attempted: false,
        execution_performed: false,
        decision_ref: None,
        receipt_ref: None,
    })
}

pub fn resolve_filesystem_review_decision(
    operation: &Operation,
    resource: &ResourceAttachmentState,
    review: &ReviewState,
    action: &ReviewAction,
    current_case_generation: u64,
) -> Result<Decision, String> {
    action.validate_integrity()?;
    if review.schema != REVIEW_REQUEST_SCHEMA_V1
        || review.operation_id != operation.operation_id
        || review.operation_digest != operation.operation_digest
        || review.resource_attachment_id != resource.attachment_id
        || review.reviewer_participant != resource.policy_owner_participant_id
        || review.latest_action_id.as_deref() != Some(action.action_id.as_str())
        || action.review_id != review.review_id
        || action.operation_id != operation.operation_id
        || action.case_id != operation.case_id
        || action.reviewer_participant_id != review.reviewer_participant
        || resource.review_requirement != ReviewRequirement::RequireReview
        || !path_within_prefix(
            &resource.allowed_write_prefix,
            &operation.filesystem_write.relative_path,
        )
    {
        return Err("review_resolution_chain_or_policy_mismatch".to_string());
    }
    let outcome = match action.action {
        ReviewActionKind::Approve if review.status == ReviewResolution::Approved => {
            DecisionOutcome::Allow
        }
        ReviewActionKind::Deny if review.status == ReviewResolution::Denied => {
            DecisionOutcome::Deny
        }
        ReviewActionKind::Defer => {
            return Err("deferred_review_has_no_effective_decision".to_string())
        }
        _ => return Err("review_action_state_mismatch".to_string()),
    };
    let reason = format!("human review {}: {}", review.review_id, action.reason);
    let basis_refs = vec![
        resource.attachment_id.clone(),
        resource.policy_id.clone(),
        review.review_id.clone(),
        action.action_id.clone(),
    ];
    Ok(build_filesystem_decision_with_basis(
        operation,
        resource,
        current_case_generation,
        outcome,
        &reason,
        basis_refs,
    ))
}

fn build_filesystem_decision(
    operation: &Operation,
    resource: &ResourceAttachmentState,
    current_case_generation: u64,
    outcome: DecisionOutcome,
    reason: &str,
) -> Decision {
    build_filesystem_decision_with_basis(
        operation,
        resource,
        current_case_generation,
        outcome,
        reason,
        vec![resource.attachment_id.clone(), resource.policy_id.clone()],
    )
}

fn build_filesystem_decision_with_basis(
    operation: &Operation,
    resource: &ResourceAttachmentState,
    current_case_generation: u64,
    outcome: DecisionOutcome,
    reason: &str,
    basis_refs: Vec<String>,
) -> Decision {
    let source = DecisionSource {
        policy_id: resource.policy_id.clone(),
        owner_participant_id: resource.policy_owner_participant_id.clone(),
    };
    let material = DecisionDigestMaterial {
        schema: DECISION_SCHEMA_V1,
        operation_id: &operation.operation_id,
        operation_digest: &operation.operation_digest,
        outcome: &outcome,
        source: &source,
        reason,
        basis_refs: &basis_refs,
        decided_at_case_generation: current_case_generation,
    };
    let decision_digest = digest_serialized(&material);
    Decision {
        schema: DECISION_SCHEMA_V1.to_string(),
        decision_id: format!("decision:{}", digest_suffix(&decision_digest, 32)),
        decision_digest,
        operation_id: operation.operation_id.clone(),
        operation_digest: operation.operation_digest.clone(),
        outcome,
        source: Some(source),
        decision_basis: None,
        reason: reason.to_string(),
        basis_refs,
        decided_at_case_generation: current_case_generation,
    }
}

pub fn build_policy_decision(
    operation: &Operation,
    decision_basis: DecisionBasis,
    reason: &str,
) -> Result<Decision, String> {
    operation.validate()?;
    decision_basis.validate_integrity()?;
    if reason.trim().is_empty()
        || decision_basis.operation_id != operation.operation_id
        || decision_basis.operation_digest != operation.operation_digest
        || decision_basis.case_id != operation.case_id
    {
        return Err("policy_decision_input_mismatch".to_string());
    }
    let outcome = decision_basis.final_posture.clone();
    let decided_at_case_generation = decision_basis.evaluated_case_generation;
    let decision_digest = {
        let material = DecisionDigestMaterialV2 {
            schema: DECISION_SCHEMA,
            operation_id: &operation.operation_id,
            operation_digest: &operation.operation_digest,
            outcome: &outcome,
            decision_basis_id: &decision_basis.basis_id,
            decision_basis_digest: &decision_basis.integrity_digest,
            reason,
            decided_at_case_generation,
        };
        digest_serialized(&material)
    };
    let decision = Decision {
        schema: DECISION_SCHEMA.to_string(),
        decision_id: format!("decision:{}", digest_suffix(&decision_digest, 32)),
        decision_digest,
        operation_id: operation.operation_id.clone(),
        operation_digest: operation.operation_digest.clone(),
        outcome,
        source: None,
        decision_basis: Some(decision_basis),
        reason: reason.to_string(),
        basis_refs: Vec::new(),
        decided_at_case_generation,
    };
    decision.validate_integrity()?;
    Ok(decision)
}

pub fn issue_policy_execution_grant(
    operation: &Operation,
    decision: &Decision,
    current_case_generation: u64,
) -> Result<ExecutionGrant, String> {
    validate_decision(operation, decision, decision.decided_at_case_generation)?;
    if decision.schema != DECISION_SCHEMA || decision.outcome != DecisionOutcome::Allow {
        return Err("policy_execution_grant_requires_v2_allow_decision".to_string());
    }
    let basis = decision
        .decision_basis
        .as_ref()
        .ok_or_else(|| "policy_execution_grant_basis_missing".to_string())?;
    if current_case_generation != decision.decided_at_case_generation + 1
        || !basis.admission_obligations_satisfied()
    {
        return Err("policy_execution_grant_admission_incomplete".to_string());
    }
    let idempotency_key = format!(
        "effect-key:{}",
        digest_suffix(&operation.operation_digest, 32)
    );
    let execution_evidence_requirements = basis.execution_evidence_requirements();
    let review_action_ref = basis.review_action_ref.clone();
    let permitted_effect = operation.granted_effect();
    let normalized_target = operation.normalized_target()?;
    let intended_content_digest = operation.intended_effect_digest()?;
    let issued_at_unix_ms = basis.authority_evaluated_at_unix_ms;
    let platform_expiry = issued_at_unix_ms.saturating_add(MAX_EXECUTION_GRANT_LIFETIME_MS);
    let expires_at_unix_ms = basis
        .earliest_policy_expiry_unix_ms
        .map_or(platform_expiry, |policy_expiry| {
            policy_expiry.min(platform_expiry)
        });
    if issued_at_unix_ms == 0 || expires_at_unix_ms <= issued_at_unix_ms {
        return Err("policy_execution_grant_temporal_window_invalid".to_string());
    }
    let material = GrantDigestMaterialV3 {
        schema: EXECUTION_GRANT_SCHEMA,
        operation_id: &operation.operation_id,
        operation_digest: &operation.operation_digest,
        decision_id: &decision.decision_id,
        decision_digest: &decision.decision_digest,
        decision_basis_id: &basis.basis_id,
        decision_basis_digest: &basis.integrity_digest,
        effective_policy_id: &basis.effective_policy_id,
        effective_policy_digest: &basis.effective_policy_digest,
        policy_binding_refs: &basis.policy_binding_refs,
        policy_artifact_refs: &basis.policy_artifact_refs,
        case_id: &operation.case_id,
        participant_id: &operation.participant_id,
        resource_attachment_id: &operation.resource_attachment_id,
        permitted_effect: &permitted_effect,
        normalized_target: &normalized_target,
        intended_content_digest: &intended_content_digest,
        expected_case_generation: current_case_generation,
        idempotency_key: &idempotency_key,
        require_pre_observation: true,
        require_post_observation: true,
        issued_at_unix_ms,
        expires_at_unix_ms,
        execution_evidence_requirements: &execution_evidence_requirements,
        review_action_ref: &review_action_ref,
    };
    let integrity_digest = digest_serialized(&material);
    let grant = ExecutionGrant {
        schema: EXECUTION_GRANT_SCHEMA.to_string(),
        grant_id: format!("grant:{}", digest_suffix(&integrity_digest, 32)),
        integrity_digest,
        operation_id: operation.operation_id.clone(),
        operation_digest: operation.operation_digest.clone(),
        decision_id: decision.decision_id.clone(),
        decision_digest: decision.decision_digest.clone(),
        case_id: operation.case_id.clone(),
        participant_id: operation.participant_id.clone(),
        resource_attachment_id: operation.resource_attachment_id.clone(),
        permitted_effect,
        normalized_target,
        intended_content_digest,
        expected_case_generation: current_case_generation,
        idempotency_key,
        require_pre_observation: true,
        require_post_observation: true,
        issued_at_unix_ms,
        expires_at_unix_ms,
        decision_basis_id: Some(basis.basis_id.clone()),
        decision_basis_digest: Some(basis.integrity_digest.clone()),
        effective_policy_id: Some(basis.effective_policy_id.clone()),
        effective_policy_digest: Some(basis.effective_policy_digest.clone()),
        policy_binding_refs: basis.policy_binding_refs.clone(),
        policy_artifact_refs: basis.policy_artifact_refs.clone(),
        execution_evidence_requirements,
        review_action_ref,
    };
    grant.validate_integrity()?;
    Ok(grant)
}

#[cfg(test)]
pub(crate) fn reseal_policy_execution_grant_for_test(mut grant: ExecutionGrant) -> ExecutionGrant {
    let material = GrantDigestMaterialV3 {
        schema: &grant.schema,
        operation_id: &grant.operation_id,
        operation_digest: &grant.operation_digest,
        decision_id: &grant.decision_id,
        decision_digest: &grant.decision_digest,
        decision_basis_id: grant.decision_basis_id.as_deref().unwrap_or_default(),
        decision_basis_digest: grant.decision_basis_digest.as_deref().unwrap_or_default(),
        effective_policy_id: grant.effective_policy_id.as_deref().unwrap_or_default(),
        effective_policy_digest: grant.effective_policy_digest.as_deref().unwrap_or_default(),
        policy_binding_refs: &grant.policy_binding_refs,
        policy_artifact_refs: &grant.policy_artifact_refs,
        case_id: &grant.case_id,
        participant_id: &grant.participant_id,
        resource_attachment_id: &grant.resource_attachment_id,
        permitted_effect: &grant.permitted_effect,
        normalized_target: &grant.normalized_target,
        intended_content_digest: &grant.intended_content_digest,
        expected_case_generation: grant.expected_case_generation,
        idempotency_key: &grant.idempotency_key,
        require_pre_observation: grant.require_pre_observation,
        require_post_observation: grant.require_post_observation,
        issued_at_unix_ms: grant.issued_at_unix_ms,
        expires_at_unix_ms: grant.expires_at_unix_ms,
        execution_evidence_requirements: &grant.execution_evidence_requirements,
        review_action_ref: &grant.review_action_ref,
    };
    grant.integrity_digest = digest_serialized(&material);
    grant.grant_id = format!("grant:{}", digest_suffix(&grant.integrity_digest, 32));
    grant
}

pub fn issue_execution_grant(
    operation: &Operation,
    decision: &Decision,
    current_case_generation: u64,
) -> Result<ExecutionGrant, String> {
    validate_decision(operation, decision, decision.decided_at_case_generation)?;
    if current_case_generation < decision.decided_at_case_generation + 1 {
        return Err("execution_grant_requires_current_committed_decision".to_string());
    }
    if decision.outcome != DecisionOutcome::Allow {
        return Err("execution_grant_requires_allow_decision".to_string());
    }
    let idempotency_key = format!(
        "effect-key:{}",
        digest_suffix(&operation.operation_digest, 32)
    );
    let material = GrantDigestMaterial {
        schema: EXECUTION_GRANT_SCHEMA_V1,
        operation_id: &operation.operation_id,
        operation_digest: &operation.operation_digest,
        decision_id: &decision.decision_id,
        decision_digest: &decision.decision_digest,
        case_id: &operation.case_id,
        participant_id: &operation.participant_id,
        resource_attachment_id: &operation.resource_attachment_id,
        permitted_effect: &GrantedEffect::FilesystemWrite,
        normalized_target: &operation.filesystem_write.relative_path,
        intended_content_digest: &operation.filesystem_write.content_digest,
        expected_case_generation: current_case_generation,
        idempotency_key: &idempotency_key,
        require_pre_observation: true,
        require_post_observation: true,
    };
    let integrity_digest = digest_serialized(&material);
    Ok(ExecutionGrant {
        schema: EXECUTION_GRANT_SCHEMA_V1.to_string(),
        grant_id: format!("grant:{}", digest_suffix(&integrity_digest, 32)),
        integrity_digest,
        operation_id: operation.operation_id.clone(),
        operation_digest: operation.operation_digest.clone(),
        decision_id: decision.decision_id.clone(),
        decision_digest: decision.decision_digest.clone(),
        case_id: operation.case_id.clone(),
        participant_id: operation.participant_id.clone(),
        resource_attachment_id: operation.resource_attachment_id.clone(),
        permitted_effect: GrantedEffect::FilesystemWrite,
        normalized_target: operation.filesystem_write.relative_path.clone(),
        intended_content_digest: operation.filesystem_write.content_digest.clone(),
        expected_case_generation: current_case_generation,
        idempotency_key,
        require_pre_observation: true,
        require_post_observation: true,
        issued_at_unix_ms: 0,
        expires_at_unix_ms: 0,
        decision_basis_id: None,
        decision_basis_digest: None,
        effective_policy_id: None,
        effective_policy_digest: None,
        policy_binding_refs: Vec::new(),
        policy_artifact_refs: Vec::new(),
        execution_evidence_requirements: Vec::new(),
        review_action_ref: None,
    })
}

pub fn prepare_effect(
    operation: &Operation,
    decision: &Decision,
    grant: &ExecutionGrant,
    pre_observation: FilesystemObservation,
) -> Result<PreparedEffect, String> {
    validate_grant(operation, decision, grant, grant.expected_case_generation)?;
    if pre_observation.resource_attachment_id != operation.resource_attachment_id
        || pre_observation.relative_path != operation.filesystem_write.relative_path
    {
        return Err("prepare_observation_target_mismatch".to_string());
    }
    if pre_observation.state == ResourceState::Unavailable {
        return Err("prepare_requires_available_pre_observation".to_string());
    }
    let effect_id = format!("effect:{}", digest_suffix(&grant.integrity_digest, 32));
    Ok(PreparedEffect {
        schema: PREPARED_EFFECT_SCHEMA_V1.to_string(),
        effect_id,
        operation_id: operation.operation_id.clone(),
        decision_id: decision.decision_id.clone(),
        grant_id: grant.grant_id.clone(),
        case_id: operation.case_id.clone(),
        participant_id: operation.participant_id.clone(),
        resource_attachment_id: operation.resource_attachment_id.clone(),
        relative_path: operation.filesystem_write.relative_path.clone(),
        expected_pre_observation: pre_observation,
        intended_content_digest: operation.filesystem_write.content_digest.clone(),
        idempotency_key: grant.idempotency_key.clone(),
        carrier_backend: FILESYSTEM_CARRIER_BACKEND.to_string(),
        resource_fence: None,
    })
}

/// Builds the PREPARE intent for a Tenant-scoped effect.  It is deliberately
/// incomplete until `LmdbRecordStore::commit_fenced_effect_prepared` issues
/// the resource epoch and commits the fence plus Case PREPARE atomically.
pub fn prepare_fenced_effect(
    operation: &Operation,
    decision: &Decision,
    grant: &ExecutionGrant,
    pre_observation: FilesystemObservation,
) -> Result<PreparedEffect, String> {
    let mut prepared = prepare_effect(operation, decision, grant, pre_observation)?;
    prepared.schema = PREPARED_EFFECT_SCHEMA.to_string();
    Ok(prepared)
}

pub fn prepare_process_effect(
    operation: &Operation,
    decision: &Decision,
    grant: &ExecutionGrant,
    pre_observation: ProcessObservation,
) -> Result<PreparedProcessEffect, String> {
    validate_grant(operation, decision, grant, grant.expected_case_generation)?;
    let payload = operation
        .process_signal
        .as_ref()
        .ok_or_else(|| "prepare_process_signal_payload_missing".to_string())?;
    if operation.kind != OperationKind::ProcessSignal
        || grant.permitted_effect != GrantedEffect::ProcessSignal
        || pre_observation.resource_attachment_id != operation.resource_attachment_id
        || pre_observation.state == ProcessObservedState::Unavailable
    {
        return Err("prepare_process_observation_target_mismatch".to_string());
    }
    let observed_digest = digest_bytes(
        pre_observation
            .process_identity
            .canonical_identity()
            .as_bytes(),
    );
    if observed_digest != payload.target_identity_digest {
        return Err("prepare_process_birth_identity_mismatch".to_string());
    }
    Ok(PreparedProcessEffect {
        schema: PREPARED_PROCESS_EFFECT_SCHEMA.to_string(),
        effect_id: format!("effect:{}", digest_suffix(&grant.integrity_digest, 32)),
        operation_id: operation.operation_id.clone(),
        decision_id: decision.decision_id.clone(),
        grant_id: grant.grant_id.clone(),
        case_id: operation.case_id.clone(),
        participant_id: operation.participant_id.clone(),
        resource_attachment_id: operation.resource_attachment_id.clone(),
        action: payload.action.clone(),
        expected_pre_observation: pre_observation,
        target_identity_digest: payload.target_identity_digest.clone(),
        idempotency_key: grant.idempotency_key.clone(),
        carrier_backend: PROCESS_SIGNAL_CARRIER_BACKEND.to_string(),
        resource_fence: None,
    })
}

pub fn validate_decision(
    operation: &Operation,
    decision: &Decision,
    current_case_generation: u64,
) -> Result<(), String> {
    operation.validate()?;
    if (decision.schema != DECISION_SCHEMA && decision.schema != DECISION_SCHEMA_V1)
        || decision.operation_id != operation.operation_id
        || decision.operation_digest != operation.operation_digest
        || decision.decided_at_case_generation != current_case_generation
    {
        return Err("decision_operation_or_generation_mismatch".to_string());
    }
    if decision.validate_integrity().is_err() {
        return Err("decision_digest_mismatch".to_string());
    }
    Ok(())
}

pub fn validate_grant(
    operation: &Operation,
    decision: &Decision,
    grant: &ExecutionGrant,
    expected_generation: u64,
) -> Result<(), String> {
    validate_decision(operation, decision, decision.decided_at_case_generation)?;
    if decision.outcome != DecisionOutcome::Allow {
        return Err("grant_decision_is_not_allow".to_string());
    }
    let normalized_target = operation.normalized_target()?;
    let intended_content_digest = operation.intended_effect_digest()?;
    if (grant.schema != EXECUTION_GRANT_SCHEMA && grant.schema != EXECUTION_GRANT_SCHEMA_V1)
        || grant.operation_id != operation.operation_id
        || grant.operation_digest != operation.operation_digest
        || grant.decision_id != decision.decision_id
        || grant.decision_digest != decision.decision_digest
        || grant.case_id != operation.case_id
        || grant.participant_id != operation.participant_id
        || grant.resource_attachment_id != operation.resource_attachment_id
        || grant.permitted_effect != operation.granted_effect()
        || grant.normalized_target != normalized_target
        || grant.intended_content_digest != intended_content_digest
        || grant.expected_case_generation != expected_generation
        || grant.expected_case_generation <= decision.decided_at_case_generation
        || !grant.require_pre_observation
        || !grant.require_post_observation
    {
        return Err("execution_grant_contract_mismatch".to_string());
    }
    if grant.validate_integrity().is_err() {
        return Err("execution_grant_integrity_mismatch".to_string());
    }
    Ok(())
}

impl Operation {
    pub fn normalized_target(&self) -> Result<String, String> {
        match self.kind {
            OperationKind::FilesystemWrite => Ok(self.filesystem_write.relative_path.clone()),
            OperationKind::ProcessSignal => self
                .process_signal
                .as_ref()
                .map(|payload| payload.action.as_str().to_string())
                .ok_or_else(|| "process_signal_payload_missing".to_string()),
        }
    }

    pub fn intended_effect_digest(&self) -> Result<String, String> {
        match self.kind {
            OperationKind::FilesystemWrite => Ok(self.filesystem_write.content_digest.clone()),
            OperationKind::ProcessSignal => self
                .process_signal
                .as_ref()
                .map(|payload| {
                    digest_bytes(
                        format!(
                            "{}|{}",
                            payload.action.as_str(),
                            payload.target_identity_digest
                        )
                        .as_bytes(),
                    )
                })
                .ok_or_else(|| "process_signal_payload_missing".to_string()),
        }
    }

    pub fn granted_effect(&self) -> GrantedEffect {
        match self.kind {
            OperationKind::FilesystemWrite => GrantedEffect::FilesystemWrite,
            OperationKind::ProcessSignal => GrantedEffect::ProcessSignal,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if (self.schema != OPERATION_SCHEMA && self.schema != OPERATION_SCHEMA_V1)
            || self.case_id.is_empty()
            || self.participant_id.is_empty()
            || self.resource_attachment_id.is_empty()
            || self.scope.case_id != self.case_id
            || !self
                .scope
                .participant_refs
                .iter()
                .any(|value| value == &self.participant_id)
            || !self
                .scope
                .resource_refs
                .iter()
                .any(|value| value == &self.resource_attachment_id)
        {
            return Err("operation_contract_mismatch".to_string());
        }
        match &self.origin {
            OperationOrigin::ProviderResult {
                provider_result_id,
                provider_invocation_id,
            } if provider_result_id.is_empty() || provider_invocation_id.is_empty() => {
                return Err("operation_provider_origin_incomplete".to_string())
            }
            OperationOrigin::CompatibilityReview {
                review_id,
                attempt_id,
            } if review_id.is_empty() || attempt_id.is_empty() => {
                return Err("operation_review_origin_incomplete".to_string())
            }
            _ => {}
        }
        let digest = match (&*self.schema, &self.kind, &self.process_signal) {
            (OPERATION_SCHEMA_V1, OperationKind::FilesystemWrite, None) => {
                if normalize_relative_path(&self.filesystem_write.relative_path)?
                    != self.filesystem_write.relative_path
                    || digest_bytes(self.filesystem_write.content.as_bytes())
                        != self.filesystem_write.content_digest
                    || self.filesystem_write.content.len() != self.filesystem_write.content_bytes
                {
                    return Err("operation_payload_integrity_mismatch".to_string());
                }
                digest_serialized(&OperationDigestMaterial {
                    schema: &self.schema,
                    case_id: &self.case_id,
                    participant_id: &self.participant_id,
                    scope: &self.scope,
                    kind: &self.kind,
                    resource_attachment_id: &self.resource_attachment_id,
                    filesystem_write: &self.filesystem_write,
                    origin: &self.origin,
                    expected_case_generation: self.expected_case_generation,
                })
            }
            (OPERATION_SCHEMA, OperationKind::ProcessSignal, Some(process_signal))
                if !process_signal.target_identity_digest.is_empty() =>
            {
                if self.filesystem_write != FilesystemWritePayload::default() {
                    return Err("process_operation_contains_filesystem_payload".to_string());
                }
                digest_serialized(&OperationDigestMaterialV2 {
                    schema: &self.schema,
                    case_id: &self.case_id,
                    participant_id: &self.participant_id,
                    scope: &self.scope,
                    kind: &self.kind,
                    resource_attachment_id: &self.resource_attachment_id,
                    process_signal,
                    origin: &self.origin,
                    expected_case_generation: self.expected_case_generation,
                })
            }
            _ => return Err("operation_schema_kind_payload_mismatch".to_string()),
        };
        if digest != self.operation_digest {
            return Err("operation_digest_mismatch".to_string());
        }
        Ok(())
    }
}

impl Decision {
    pub fn validate_integrity(&self) -> Result<(), String> {
        if (self.schema != DECISION_SCHEMA
            && self.schema != DECISION_SCHEMA_V2
            && self.schema != DECISION_SCHEMA_V1)
            || self.decision_id.is_empty()
            || self.operation_id.is_empty()
            || self.operation_digest.is_empty()
            || self.reason.is_empty()
        {
            return Err("invalid_decision_contract".to_string());
        }
        let digest = if self.schema == DECISION_SCHEMA_V1 {
            let source = self
                .source
                .as_ref()
                .ok_or_else(|| "legacy_decision_source_missing".to_string())?;
            if source.policy_id.is_empty()
                || source.owner_participant_id.is_empty()
                || self.decision_basis.is_some()
            {
                return Err("invalid_legacy_decision_contract".to_string());
            }
            digest_serialized(&DecisionDigestMaterial {
                schema: &self.schema,
                operation_id: &self.operation_id,
                operation_digest: &self.operation_digest,
                outcome: &self.outcome,
                source,
                reason: &self.reason,
                basis_refs: &self.basis_refs,
                decided_at_case_generation: self.decided_at_case_generation,
            })
        } else {
            let basis = self
                .decision_basis
                .as_ref()
                .ok_or_else(|| "policy_decision_basis_missing".to_string())?;
            basis.validate_integrity()?;
            if (self.schema == DECISION_SCHEMA
                && basis.schema != crate::admission::DECISION_BASIS_SCHEMA
                && basis.schema != crate::admission::DECISION_BASIS_SCHEMA_V2)
                || (self.schema == DECISION_SCHEMA_V2
                    && basis.schema != crate::admission::DECISION_BASIS_SCHEMA_V1)
            {
                return Err("decision_basis_schema_generation_mismatch".to_string());
            }
            if self.source.is_some()
                || !self.basis_refs.is_empty()
                || basis.operation_id != self.operation_id
                || basis.operation_digest != self.operation_digest
                || basis.final_posture != self.outcome
                || basis.evaluated_case_generation != self.decided_at_case_generation
            {
                return Err("policy_decision_basis_mismatch".to_string());
            }
            digest_serialized(&DecisionDigestMaterialV2 {
                schema: &self.schema,
                operation_id: &self.operation_id,
                operation_digest: &self.operation_digest,
                outcome: &self.outcome,
                decision_basis_id: &basis.basis_id,
                decision_basis_digest: &basis.integrity_digest,
                reason: &self.reason,
                decided_at_case_generation: self.decided_at_case_generation,
            })
        };
        if digest != self.decision_digest
            || self.decision_id != format!("decision:{}", digest_suffix(&digest, 32))
        {
            return Err("decision_digest_mismatch".to_string());
        }
        Ok(())
    }
}

impl ExecutionGrant {
    pub fn validate_integrity(&self) -> Result<(), String> {
        if (self.schema != EXECUTION_GRANT_SCHEMA
            && self.schema != EXECUTION_GRANT_SCHEMA_V2
            && self.schema != EXECUTION_GRANT_SCHEMA_V1)
            || self.grant_id.is_empty()
            || self.operation_id.is_empty()
            || self.decision_id.is_empty()
            || self.case_id.is_empty()
            || self.participant_id.is_empty()
            || self.resource_attachment_id.is_empty()
            || self.normalized_target.is_empty()
            || self.idempotency_key.is_empty()
            || !self.require_pre_observation
            || !self.require_post_observation
        {
            return Err("invalid_execution_grant_contract".to_string());
        }
        let digest = if self.schema == EXECUTION_GRANT_SCHEMA_V1 {
            if self.decision_basis_id.is_some()
                || self.effective_policy_id.is_some()
                || !self.execution_evidence_requirements.is_empty()
            {
                return Err("legacy_execution_grant_claims_policy_basis".to_string());
            }
            digest_serialized(&GrantDigestMaterial {
                schema: &self.schema,
                operation_id: &self.operation_id,
                operation_digest: &self.operation_digest,
                decision_id: &self.decision_id,
                decision_digest: &self.decision_digest,
                case_id: &self.case_id,
                participant_id: &self.participant_id,
                resource_attachment_id: &self.resource_attachment_id,
                permitted_effect: &self.permitted_effect,
                normalized_target: &self.normalized_target,
                intended_content_digest: &self.intended_content_digest,
                expected_case_generation: self.expected_case_generation,
                idempotency_key: &self.idempotency_key,
                require_pre_observation: self.require_pre_observation,
                require_post_observation: self.require_post_observation,
            })
        } else if self.schema == EXECUTION_GRANT_SCHEMA_V2 {
            let basis_id = self.decision_basis_id.as_deref().unwrap_or_default();
            let basis_digest = self.decision_basis_digest.as_deref().unwrap_or_default();
            let effective_id = self.effective_policy_id.as_deref().unwrap_or_default();
            let effective_digest = self.effective_policy_digest.as_deref().unwrap_or_default();
            if basis_id.is_empty()
                || basis_digest.is_empty()
                || effective_id.is_empty()
                || effective_digest.is_empty()
                || self.policy_binding_refs.is_empty()
                || self.policy_artifact_refs.is_empty()
            {
                return Err("policy_execution_grant_basis_missing".to_string());
            }
            digest_serialized(&GrantDigestMaterialV2 {
                schema: &self.schema,
                operation_id: &self.operation_id,
                operation_digest: &self.operation_digest,
                decision_id: &self.decision_id,
                decision_digest: &self.decision_digest,
                decision_basis_id: basis_id,
                decision_basis_digest: basis_digest,
                effective_policy_id: effective_id,
                effective_policy_digest: effective_digest,
                policy_binding_refs: &self.policy_binding_refs,
                policy_artifact_refs: &self.policy_artifact_refs,
                case_id: &self.case_id,
                participant_id: &self.participant_id,
                resource_attachment_id: &self.resource_attachment_id,
                permitted_effect: &self.permitted_effect,
                normalized_target: &self.normalized_target,
                intended_content_digest: &self.intended_content_digest,
                expected_case_generation: self.expected_case_generation,
                idempotency_key: &self.idempotency_key,
                require_pre_observation: self.require_pre_observation,
                require_post_observation: self.require_post_observation,
                execution_evidence_requirements: &self.execution_evidence_requirements,
                review_action_ref: &self.review_action_ref,
            })
        } else {
            let basis_id = self.decision_basis_id.as_deref().unwrap_or_default();
            let basis_digest = self.decision_basis_digest.as_deref().unwrap_or_default();
            let effective_id = self.effective_policy_id.as_deref().unwrap_or_default();
            let effective_digest = self.effective_policy_digest.as_deref().unwrap_or_default();
            if basis_id.is_empty()
                || basis_digest.is_empty()
                || effective_id.is_empty()
                || effective_digest.is_empty()
                || self.policy_binding_refs.is_empty()
                || self.policy_artifact_refs.is_empty()
                || self.issued_at_unix_ms == 0
                || self.expires_at_unix_ms <= self.issued_at_unix_ms
                || self.expires_at_unix_ms - self.issued_at_unix_ms
                    > MAX_EXECUTION_GRANT_LIFETIME_MS
            {
                return Err("policy_execution_grant_temporal_or_basis_missing".to_string());
            }
            digest_serialized(&GrantDigestMaterialV3 {
                schema: &self.schema,
                operation_id: &self.operation_id,
                operation_digest: &self.operation_digest,
                decision_id: &self.decision_id,
                decision_digest: &self.decision_digest,
                decision_basis_id: basis_id,
                decision_basis_digest: basis_digest,
                effective_policy_id: effective_id,
                effective_policy_digest: effective_digest,
                policy_binding_refs: &self.policy_binding_refs,
                policy_artifact_refs: &self.policy_artifact_refs,
                case_id: &self.case_id,
                participant_id: &self.participant_id,
                resource_attachment_id: &self.resource_attachment_id,
                permitted_effect: &self.permitted_effect,
                normalized_target: &self.normalized_target,
                intended_content_digest: &self.intended_content_digest,
                expected_case_generation: self.expected_case_generation,
                idempotency_key: &self.idempotency_key,
                require_pre_observation: self.require_pre_observation,
                require_post_observation: self.require_post_observation,
                issued_at_unix_ms: self.issued_at_unix_ms,
                expires_at_unix_ms: self.expires_at_unix_ms,
                execution_evidence_requirements: &self.execution_evidence_requirements,
                review_action_ref: &self.review_action_ref,
            })
        };
        if digest != self.integrity_digest
            || self.grant_id != format!("grant:{}", digest_suffix(&digest, 32))
        {
            return Err("execution_grant_integrity_mismatch".to_string());
        }
        Ok(())
    }
}

impl LocalFilesystemBinding {
    pub fn new(
        case_id: impl Into<String>,
        attachment_id: impl Into<String>,
        root: &Path,
    ) -> Result<Self, String> {
        let canonical = fs::canonicalize(root)
            .map_err(|error| format!("failed to canonicalize binding root: {error}"))?;
        if !canonical.is_dir() {
            return Err("filesystem binding root must be a directory".to_string());
        }
        #[cfg(target_os = "linux")]
        let (root_device, root_inode) = {
            let metadata = fs::metadata(&canonical)
                .map_err(|error| format!("failed to identify binding root: {error}"))?;
            (metadata.dev(), metadata.ino())
        };
        #[cfg(not(target_os = "linux"))]
        let (root_device, root_inode) = (0, 0);
        Ok(Self {
            schema: LOCAL_FILESYSTEM_BINDING_SCHEMA.to_string(),
            case_id: case_id.into(),
            attachment_id: attachment_id.into(),
            canonical_root: canonical.to_string_lossy().into_owned(),
            root_device,
            root_inode,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if (self.schema != LOCAL_FILESYSTEM_BINDING_SCHEMA
            && self.schema != LOCAL_FILESYSTEM_BINDING_SCHEMA_V1)
            || self.case_id.is_empty()
            || self.attachment_id.is_empty()
            || !Path::new(&self.canonical_root).is_absolute()
            || (self.schema == LOCAL_FILESYSTEM_BINDING_SCHEMA
                && (self.root_device == 0 || self.root_inode == 0))
        {
            return Err("invalid_local_filesystem_binding".to_string());
        }
        Ok(())
    }

    fn validate_secure_carrier(&self) -> Result<(), String> {
        self.validate()?;
        if self.schema != LOCAL_FILESYSTEM_BINDING_SCHEMA {
            return Err("filesystem_binding_requires_inode_identity_v2".to_string());
        }
        #[cfg(target_os = "linux")]
        {
            Ok(())
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err("filesystem_secure_resolution_unsupported_platform".to_string())
        }
    }
}

pub fn observe_filesystem(
    binding: &LocalFilesystemBinding,
    resource: &ResourceAttachmentState,
    relative_path: &str,
    observation_id: impl Into<String>,
) -> FilesystemObservation {
    let observation_id = observation_id.into();
    if binding.schema == LOCAL_FILESYSTEM_BINDING_SCHEMA {
        return observe_filesystem_secure(binding, resource, relative_path, observation_id);
    }
    observe_filesystem_legacy(binding, resource, relative_path, observation_id)
}

fn observe_filesystem_legacy(
    binding: &LocalFilesystemBinding,
    resource: &ResourceAttachmentState,
    relative_path: &str,
    observation_id: String,
) -> FilesystemObservation {
    let normalized = match normalize_relative_path(relative_path) {
        Ok(value) => value,
        Err(error) => {
            return unavailable_observation(
                observation_id,
                &resource.attachment_id,
                relative_path,
                error,
            )
        }
    };
    let target = match resolve_target(binding, resource, &normalized) {
        Ok(target) => target,
        Err(error) => {
            return unavailable_observation(
                observation_id,
                &resource.attachment_id,
                &normalized,
                error,
            )
        }
    };
    let metadata = match fs::symlink_metadata(&target) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return FilesystemObservation {
                schema: OBSERVATION_SCHEMA.to_string(),
                observation_id,
                resource_attachment_id: resource.attachment_id.clone(),
                relative_path: normalized,
                state: ResourceState::Absent,
                content_digest: None,
                size_bytes: None,
                error: None,
                observed_at_unix_ms: unix_time_ms(),
            }
        }
        Err(error) => {
            return unavailable_observation(
                observation_id,
                &resource.attachment_id,
                &normalized,
                format!("metadata_failed: {error}"),
            )
        }
    };
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        return FilesystemObservation {
            schema: OBSERVATION_SCHEMA.to_string(),
            observation_id,
            resource_attachment_id: resource.attachment_id.clone(),
            relative_path: normalized,
            state: ResourceState::Symlink,
            content_digest: None,
            size_bytes: None,
            error: None,
            observed_at_unix_ms: unix_time_ms(),
        };
    }
    if metadata.is_file() {
        return match fs::read(&target) {
            Ok(bytes) => FilesystemObservation {
                schema: OBSERVATION_SCHEMA.to_string(),
                observation_id,
                resource_attachment_id: resource.attachment_id.clone(),
                relative_path: normalized,
                state: ResourceState::File,
                content_digest: Some(digest_bytes(&bytes)),
                size_bytes: Some(bytes.len() as u64),
                error: None,
                observed_at_unix_ms: unix_time_ms(),
            },
            Err(error) => unavailable_observation(
                observation_id,
                &resource.attachment_id,
                &normalized,
                format!("read_failed: {error}"),
            ),
        };
    }
    FilesystemObservation {
        schema: OBSERVATION_SCHEMA.to_string(),
        observation_id,
        resource_attachment_id: resource.attachment_id.clone(),
        relative_path: normalized,
        state: if metadata.is_dir() {
            ResourceState::Directory
        } else {
            ResourceState::Other
        },
        content_digest: None,
        size_bytes: Some(metadata.len()),
        error: None,
        observed_at_unix_ms: unix_time_ms(),
    }
}

#[cfg(target_os = "linux")]
#[repr(C)]
struct OpenHow {
    flags: u64,
    mode: u64,
    resolve: u64,
}

#[cfg(target_os = "linux")]
const RESOLVE_NO_MAGICLINKS: u64 = 0x02;
#[cfg(target_os = "linux")]
const RESOLVE_NO_SYMLINKS: u64 = 0x04;
#[cfg(target_os = "linux")]
const RESOLVE_BENEATH: u64 = 0x08;

#[cfg(target_os = "linux")]
fn open_verified_filesystem_root(binding: &LocalFilesystemBinding) -> Result<File, String> {
    binding.validate_secure_carrier()?;
    let root = CString::new(Path::new(&binding.canonical_root).as_os_str().as_bytes())
        .map_err(|_| "filesystem_binding_root_contains_nul".to_string())?;
    let fd = unsafe {
        libc::open(
            root.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if fd < 0 {
        return Err(format!(
            "filesystem_binding_root_open_failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    let file = unsafe { File::from_raw_fd(fd) };
    let metadata = file
        .metadata()
        .map_err(|error| format!("filesystem_binding_root_fstat_failed: {error}"))?;
    if !metadata.is_dir()
        || metadata.dev() != binding.root_device
        || metadata.ino() != binding.root_inode
    {
        return Err("filesystem_binding_root_birth_identity_mismatch".to_string());
    }
    Ok(file)
}

#[cfg(target_os = "linux")]
fn open_beneath(root: &File, path: &Path, flags: i32, mode: u32) -> std::io::Result<File> {
    let path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| std::io::Error::from_raw_os_error(libc::EINVAL))?;
    let how = OpenHow {
        flags: flags as u64,
        mode: u64::from(mode),
        resolve: RESOLVE_BENEATH | RESOLVE_NO_MAGICLINKS | RESOLVE_NO_SYMLINKS,
    };
    let fd = unsafe {
        libc::syscall(
            libc::SYS_openat2,
            root.as_raw_fd(),
            path.as_ptr(),
            &how as *const OpenHow,
            std::mem::size_of::<OpenHow>(),
        ) as i32
    };
    if fd < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(unsafe { File::from_raw_fd(fd) })
    }
}

#[cfg(target_os = "linux")]
fn observe_filesystem_secure(
    binding: &LocalFilesystemBinding,
    resource: &ResourceAttachmentState,
    relative_path: &str,
    observation_id: String,
) -> FilesystemObservation {
    let normalized = match normalize_relative_path(relative_path) {
        Ok(value) => value,
        Err(error) => {
            return unavailable_observation(
                observation_id,
                &resource.attachment_id,
                relative_path,
                error,
            )
        }
    };
    if binding.attachment_id != resource.attachment_id
        || !path_within_prefix(&resource.allowed_write_prefix, &normalized)
    {
        return unavailable_observation(
            observation_id,
            &resource.attachment_id,
            &normalized,
            "filesystem_secure_attachment_or_prefix_mismatch".to_string(),
        );
    }
    let root = match open_verified_filesystem_root(binding) {
        Ok(root) => root,
        Err(error) => {
            return unavailable_observation(
                observation_id,
                &resource.attachment_id,
                &normalized,
                error,
            )
        }
    };
    let mut components = normalized.split('/').collect::<Vec<_>>();
    let target_name = components.pop().expect("normalized path is non-empty");
    let parent = if components.is_empty() {
        match root.try_clone() {
            Ok(parent) => parent,
            Err(error) => {
                return unavailable_observation(
                    observation_id,
                    &resource.attachment_id,
                    &normalized,
                    format!("filesystem_root_clone_failed: {error}"),
                )
            }
        }
    } else {
        match open_beneath(
            &root,
            Path::new(&components.join("/")),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            0,
        ) {
            Ok(parent) => parent,
            Err(error) => {
                return unavailable_observation(
                    observation_id,
                    &resource.attachment_id,
                    &normalized,
                    format!("filesystem_secure_parent_resolution_rejected: {error}"),
                )
            }
        }
    };
    let mut target = match open_beneath(
        &parent,
        Path::new(target_name),
        libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        0,
    ) {
        Ok(target) => target,
        Err(error) if error.raw_os_error() == Some(libc::ENOENT) => {
            return FilesystemObservation {
                schema: OBSERVATION_SCHEMA.to_string(),
                observation_id,
                resource_attachment_id: resource.attachment_id.clone(),
                relative_path: normalized,
                state: ResourceState::Absent,
                content_digest: None,
                size_bytes: None,
                error: None,
                observed_at_unix_ms: unix_time_ms(),
            }
        }
        Err(error) => {
            return unavailable_observation(
                observation_id,
                &resource.attachment_id,
                &normalized,
                format!("filesystem_secure_resolution_rejected: {error}"),
            )
        }
    };
    let metadata = match target.metadata() {
        Ok(metadata) => metadata,
        Err(error) => {
            return unavailable_observation(
                observation_id,
                &resource.attachment_id,
                &normalized,
                format!("filesystem_secure_fstat_failed: {error}"),
            )
        }
    };
    if metadata.is_file() {
        let mut bytes = Vec::new();
        return match target.read_to_end(&mut bytes) {
            Ok(_) => FilesystemObservation {
                schema: OBSERVATION_SCHEMA.to_string(),
                observation_id,
                resource_attachment_id: resource.attachment_id.clone(),
                relative_path: normalized,
                state: ResourceState::File,
                content_digest: Some(digest_bytes(&bytes)),
                size_bytes: Some(bytes.len() as u64),
                error: None,
                observed_at_unix_ms: unix_time_ms(),
            },
            Err(error) => unavailable_observation(
                observation_id,
                &resource.attachment_id,
                &normalized,
                format!("filesystem_secure_read_failed: {error}"),
            ),
        };
    }
    FilesystemObservation {
        schema: OBSERVATION_SCHEMA.to_string(),
        observation_id,
        resource_attachment_id: resource.attachment_id.clone(),
        relative_path: normalized,
        state: if metadata.is_dir() {
            ResourceState::Directory
        } else {
            ResourceState::Other
        },
        content_digest: None,
        size_bytes: Some(metadata.len()),
        error: None,
        observed_at_unix_ms: unix_time_ms(),
    }
}

#[cfg(not(target_os = "linux"))]
fn observe_filesystem_secure(
    _binding: &LocalFilesystemBinding,
    resource: &ResourceAttachmentState,
    relative_path: &str,
    observation_id: String,
) -> FilesystemObservation {
    unavailable_observation(
        observation_id,
        &resource.attachment_id,
        relative_path,
        "filesystem_secure_resolution_unsupported_platform".to_string(),
    )
}

pub fn execute_filesystem_write(
    operation: &Operation,
    decision: &Decision,
    grant: &ExecutionGrant,
    prepared: &PreparedEffect,
    case_state: &CaseState,
    binding: &LocalFilesystemBinding,
    resource: &ResourceAttachmentState,
    failpoint: CarrierFailpoint,
) -> Result<CarrierResult, String> {
    if case_state.tenant_id.is_some() {
        return Err("tenant_effect_requires_carrier_resource_fence".to_string());
    }
    execute_filesystem_write_inner(
        operation,
        decision,
        grant,
        prepared,
        case_state,
        binding,
        resource,
        failpoint,
        || Ok(()),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn execute_fenced_filesystem_write<A: ResourceFenceAuthority>(
    authority: &A,
    fence: &ResourceFence,
    operation: &Operation,
    decision: &Decision,
    grant: &ExecutionGrant,
    prepared: &PreparedEffect,
    case_state: &CaseState,
    binding: &LocalFilesystemBinding,
    resource: &ResourceAttachmentState,
    failpoint: CarrierFailpoint,
) -> Result<CarrierResult, String> {
    binding.validate_secure_carrier()?;
    let prepared_fence = prepared
        .resource_fence
        .as_ref()
        .ok_or_else(|| "prepared_effect_resource_fence_missing".to_string())?;
    if prepared.schema != PREPARED_EFFECT_SCHEMA
        || case_state.tenant_id.as_deref() != Some(fence.tenant_id.as_str())
        || prepared_fence.resource_id != fence.resource_id
        || prepared_fence.tenant_id != fence.tenant_id
        || prepared_fence.case_id != fence.case_id
        || prepared_fence.operation_id != fence.operation_id
        || prepared_fence.grant_id != fence.grant_id
        || prepared_fence.effect_id != fence.effect_id
        || fence.resource_epoch < prepared_fence.resource_epoch
    {
        return Err("prepared_effect_resource_fence_mismatch".to_string());
    }
    // This call is intentionally inside the carrier boundary and immediately
    // precedes all physical inspection/mutation below.
    authority.validate_carrier_fence(fence)?;
    execute_filesystem_write_inner(
        operation,
        decision,
        grant,
        prepared,
        case_state,
        binding,
        resource,
        failpoint,
        || authority.validate_carrier_fence(fence),
    )
}

#[allow(clippy::too_many_arguments)]
fn execute_filesystem_write_inner<F: Fn() -> Result<(), String>>(
    operation: &Operation,
    decision: &Decision,
    grant: &ExecutionGrant,
    prepared: &PreparedEffect,
    case_state: &CaseState,
    binding: &LocalFilesystemBinding,
    resource: &ResourceAttachmentState,
    failpoint: CarrierFailpoint,
    validate_immediately_before_mutation: F,
) -> Result<CarrierResult, String> {
    validate_grant(operation, decision, grant, grant.expected_case_generation)?;
    binding.validate()?;
    if (prepared.schema != PREPARED_EFFECT_SCHEMA && prepared.schema != PREPARED_EFFECT_SCHEMA_V1)
        || prepared.operation_id != operation.operation_id
        || prepared.decision_id != decision.decision_id
        || prepared.grant_id != grant.grant_id
        || prepared.case_id != operation.case_id
        || prepared.participant_id != operation.participant_id
        || prepared.resource_attachment_id != operation.resource_attachment_id
        || prepared.relative_path != operation.filesystem_write.relative_path
        || prepared.intended_content_digest != operation.filesystem_write.content_digest
        || prepared.idempotency_key != grant.idempotency_key
        || binding.case_id != operation.case_id
        || binding.attachment_id != operation.resource_attachment_id
        || resource.attachment_id != operation.resource_attachment_id
    {
        return Err("prepared_effect_chain_mismatch".to_string());
    }
    let effect_state = case_state
        .effects
        .iter()
        .find(|effect| effect.effect_id == prepared.effect_id)
        .ok_or_else(|| "prepared_effect_not_materialized".to_string())?;
    if effect_state.status != crate::transition::EffectLifecycle::Prepared
        || effect_state.grant_id != grant.grant_id
    {
        return Err("grant_is_not_current_prepared_authority".to_string());
    }

    let current = observe_filesystem(
        binding,
        resource,
        &prepared.relative_path,
        format!("observation:{}:carrier-pre", prepared.effect_id),
    );
    if current.state == ResourceState::File
        && current.content_digest.as_deref() == Some(&prepared.intended_content_digest)
    {
        return Ok(CarrierResult {
            outcome: EffectOutcome::AlreadyApplied,
            post_observation: current,
            carrier_attempted: true,
            mutation_performed: false,
            crash_injected_after_effect: false,
            detail: "intended post-state already observed".to_string(),
        });
    }
    if !observations_equivalent(&current, &prepared.expected_pre_observation) {
        return Ok(CarrierResult {
            outcome: if current.state == ResourceState::Unavailable {
                EffectOutcome::Indeterminate
            } else {
                EffectOutcome::Conflict
            },
            post_observation: current,
            carrier_attempted: false,
            mutation_performed: false,
            crash_injected_after_effect: false,
            detail: "resource no longer matches prepared pre-state".to_string(),
        });
    }
    if matches!(
        current.state,
        ResourceState::Directory | ResourceState::Symlink | ResourceState::Other
    ) {
        return Ok(CarrierResult {
            outcome: EffectOutcome::Conflict,
            post_observation: current,
            carrier_attempted: false,
            mutation_performed: false,
            crash_injected_after_effect: false,
            detail: "target type cannot be replaced by this carrier".to_string(),
        });
    }
    if failpoint == CarrierFailpoint::FailBeforeMutation {
        return Ok(CarrierResult {
            outcome: EffectOutcome::FailedNoEffect,
            post_observation: current,
            carrier_attempted: true,
            mutation_performed: false,
            crash_injected_after_effect: false,
            detail: "injected carrier failure before mutation".to_string(),
        });
    }

    validate_immediately_before_mutation()?;
    if binding.schema == LOCAL_FILESYSTEM_BINDING_SCHEMA {
        secure_atomic_replace(
            binding,
            resource,
            &prepared.relative_path,
            operation.filesystem_write.content.as_bytes(),
            &prepared.effect_id,
        )?;
    } else {
        let target = resolve_target(binding, resource, &prepared.relative_path)?;
        atomic_replace_legacy(
            &target,
            operation.filesystem_write.content.as_bytes(),
            &prepared.effect_id,
        )?;
    }
    if failpoint == CarrierFailpoint::CrashAfterVisibleEffect {
        return Ok(CarrierResult {
            outcome: EffectOutcome::Indeterminate,
            post_observation: current,
            carrier_attempted: true,
            mutation_performed: true,
            crash_injected_after_effect: true,
            detail: "crash injected after durable rename and before post-observation".to_string(),
        });
    }
    let post = observe_filesystem(
        binding,
        resource,
        &prepared.relative_path,
        format!("observation:{}:post", prepared.effect_id),
    );
    let outcome = if post.state == ResourceState::File
        && post.content_digest.as_deref() == Some(&prepared.intended_content_digest)
    {
        EffectOutcome::Applied
    } else {
        EffectOutcome::Indeterminate
    };
    Ok(CarrierResult {
        outcome,
        post_observation: post,
        carrier_attempted: true,
        mutation_performed: true,
        crash_injected_after_effect: false,
        detail: "atomic replacement completed and resource re-observed".to_string(),
    })
}

pub fn observe_process(
    binding: &LocalProcessBinding,
    observation_id: impl Into<String>,
) -> ProcessObservation {
    let observation_id = observation_id.into();
    let pid = binding.process.pid;
    let stat_path = format!("/proc/{pid}/stat");
    let stat = match fs::read_to_string(&stat_path) {
        Ok(stat) => stat,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return ProcessObservation {
                schema: PROCESS_OBSERVATION_SCHEMA.to_string(),
                observation_id,
                resource_attachment_id: binding.attachment_id.clone(),
                process_identity: binding.process.clone(),
                state: ProcessObservedState::Exited,
                error: None,
                observed_at_unix_ms: unix_time_ms(),
            }
        }
        Err(error) => {
            return ProcessObservation {
                schema: PROCESS_OBSERVATION_SCHEMA.to_string(),
                observation_id,
                resource_attachment_id: binding.attachment_id.clone(),
                process_identity: binding.process.clone(),
                state: ProcessObservedState::Unavailable,
                error: Some(format!("process_stat_unavailable: {error}")),
                observed_at_unix_ms: unix_time_ms(),
            }
        }
    };
    let observed_identity = match LocalProcessIdentity::capture(pid) {
        Ok(identity) => identity,
        Err(error) => {
            return ProcessObservation {
                schema: PROCESS_OBSERVATION_SCHEMA.to_string(),
                observation_id,
                resource_attachment_id: binding.attachment_id.clone(),
                process_identity: binding.process.clone(),
                state: ProcessObservedState::Unavailable,
                error: Some(error),
                observed_at_unix_ms: unix_time_ms(),
            }
        }
    };
    if observed_identity != binding.process {
        return ProcessObservation {
            schema: PROCESS_OBSERVATION_SCHEMA.to_string(),
            observation_id,
            resource_attachment_id: binding.attachment_id.clone(),
            process_identity: observed_identity,
            state: ProcessObservedState::Unavailable,
            error: Some("process_birth_identity_mismatch".to_string()),
            observed_at_unix_ms: unix_time_ms(),
        };
    }
    let state = stat
        .rfind(')')
        .and_then(|end| stat[end + 1..].split_whitespace().next())
        .and_then(|value| value.chars().next())
        .map_or(ProcessObservedState::Other, |state| match state {
            'R' => ProcessObservedState::Running,
            'S' | 'D' => ProcessObservedState::Sleeping,
            'T' | 't' => ProcessObservedState::Stopped,
            'Z' => ProcessObservedState::Zombie,
            _ => ProcessObservedState::Other,
        });
    ProcessObservation {
        schema: PROCESS_OBSERVATION_SCHEMA.to_string(),
        observation_id,
        resource_attachment_id: binding.attachment_id.clone(),
        process_identity: observed_identity,
        state,
        error: None,
        observed_at_unix_ms: unix_time_ms(),
    }
}

fn process_signal_number(action: &ProcessSignalAction) -> Result<i32, String> {
    #[cfg(target_os = "linux")]
    {
        Ok(match action {
            ProcessSignalAction::Terminate => 15,
            ProcessSignalAction::Suspend => 19,
            ProcessSignalAction::Resume => 18,
        })
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = action;
        Err("process_signal_carrier_unsupported_platform".to_string())
    }
}

#[allow(clippy::too_many_arguments)]
pub fn execute_fenced_process_signal<A: ResourceFenceAuthority>(
    authority: &A,
    fence: &ResourceFence,
    operation: &Operation,
    decision: &Decision,
    grant: &ExecutionGrant,
    prepared: &PreparedProcessEffect,
    case_state: &CaseState,
    binding: &LocalProcessBinding,
) -> Result<ProcessCarrierResult, String> {
    validate_grant(operation, decision, grant, grant.expected_case_generation)?;
    binding.validate()?;
    prepared.validate()?;
    let prepared_fence = prepared
        .resource_fence
        .as_ref()
        .ok_or_else(|| "prepared_process_effect_resource_fence_missing".to_string())?;
    if case_state.tenant_id.as_deref() != Some(fence.tenant_id.as_str())
        || prepared_fence.resource_id != fence.resource_id
        || prepared_fence.case_id != fence.case_id
        || prepared_fence.operation_id != fence.operation_id
        || prepared_fence.grant_id != fence.grant_id
        || prepared_fence.effect_id != fence.effect_id
        || fence.resource_epoch < prepared_fence.resource_epoch
        || binding.case_id != operation.case_id
        || binding.attachment_id != operation.resource_attachment_id
        || binding.process.pid == std::process::id()
    {
        return Err("prepared_process_effect_chain_mismatch".to_string());
    }
    let effect = case_state
        .effects
        .iter()
        .find(|effect| effect.effect_id == prepared.effect_id)
        .ok_or_else(|| "prepared_process_effect_not_materialized".to_string())?;
    if effect.status != crate::transition::EffectLifecycle::Prepared
        || effect.grant_id != grant.grant_id
    {
        return Err("process_grant_is_not_current_prepared_authority".to_string());
    }
    let current = observe_process(
        binding,
        format!("observation:{}:carrier-pre", prepared.effect_id),
    );
    if current.state == ProcessObservedState::Exited {
        return Ok(ProcessCarrierResult {
            outcome: EffectOutcome::NoEffect,
            post_observation: current,
            kernel_signal: process_signal_number(&prepared.action)?,
            signal_attempted: false,
            syscall_accepted: false,
            detail: "exact attached process already exited; no signal sent".to_string(),
        });
    }
    if current.state == ProcessObservedState::Unavailable
        || current.process_identity != prepared.expected_pre_observation.process_identity
    {
        return Ok(ProcessCarrierResult {
            outcome: EffectOutcome::Conflict,
            post_observation: current,
            kernel_signal: process_signal_number(&prepared.action)?,
            signal_attempted: false,
            syscall_accepted: false,
            detail: "process birth identity no longer matches PREPARE".to_string(),
        });
    }
    // Carrier-side stale-writer boundary immediately before kill(2).
    authority.validate_carrier_fence(fence)?;
    let signal = process_signal_number(&prepared.action)?;
    #[cfg(unix)]
    let accepted = unsafe { kill(binding.process.pid as c_int, signal as c_int) == 0 };
    #[cfg(not(unix))]
    let accepted = false;
    let post = observe_process(binding, format!("observation:{}:post", prepared.effect_id));
    Ok(ProcessCarrierResult {
        outcome: if accepted {
            EffectOutcome::Applied
        } else {
            EffectOutcome::FailedNoEffect
        },
        post_observation: post,
        kernel_signal: signal,
        signal_attempted: true,
        syscall_accepted: accepted,
        detail: if accepted {
            "kernel accepted the exact semantic signal; observed process state is recorded separately"
                .to_string()
        } else {
            "kernel rejected the signal; no high-level process outcome is claimed".to_string()
        },
    })
}

pub fn build_process_effect_receipt(
    prepared: &PreparedProcessEffect,
    result: &ProcessCarrierResult,
) -> ProcessEffectReceipt {
    let material = format!(
        "{}|{}|{}|{}|{}",
        prepared.effect_id,
        prepared.grant_id,
        result.post_observation.observation_id,
        result.kernel_signal,
        result.syscall_accepted
    );
    ProcessEffectReceipt {
        schema: PROCESS_EFFECT_RECEIPT_SCHEMA.to_string(),
        receipt_id: format!(
            "effect-receipt:{}",
            digest_suffix(&digest_bytes(material.as_bytes()), 32)
        ),
        effect_id: prepared.effect_id.clone(),
        operation_id: prepared.operation_id.clone(),
        decision_id: prepared.decision_id.clone(),
        grant_id: prepared.grant_id.clone(),
        resource_attachment_id: prepared.resource_attachment_id.clone(),
        action: prepared.action.clone(),
        pre_observation_id: prepared.expected_pre_observation.observation_id.clone(),
        post_observation_id: result.post_observation.observation_id.clone(),
        kernel_signal: result.kernel_signal,
        syscall_accepted: result.syscall_accepted,
        observed_state: result.post_observation.state.clone(),
        outcome: result.outcome.clone(),
        carrier_backend: prepared.carrier_backend.clone(),
        completed_at_unix_ms: unix_time_ms(),
    }
}

pub fn build_effect_receipt(prepared: &PreparedEffect, result: &CarrierResult) -> EffectReceipt {
    let material = format!(
        "{}|{}|{}|{:?}|{}",
        prepared.effect_id,
        prepared.grant_id,
        result.post_observation.observation_id,
        result.outcome,
        result.mutation_performed
    );
    EffectReceipt {
        schema: EFFECT_RECEIPT_SCHEMA.to_string(),
        receipt_id: format!(
            "effect-receipt:{}",
            digest_suffix(&digest_bytes(material.as_bytes()), 32)
        ),
        effect_id: prepared.effect_id.clone(),
        operation_id: prepared.operation_id.clone(),
        decision_id: prepared.decision_id.clone(),
        grant_id: prepared.grant_id.clone(),
        resource_attachment_id: prepared.resource_attachment_id.clone(),
        relative_path: prepared.relative_path.clone(),
        pre_observation_id: prepared.expected_pre_observation.observation_id.clone(),
        post_observation_id: result.post_observation.observation_id.clone(),
        outcome: result.outcome.clone(),
        carrier_backend: prepared.carrier_backend.clone(),
        carrier_attempted: result.carrier_attempted,
        mutation_performed: result.mutation_performed,
        completed_at_unix_ms: unix_time_ms(),
    }
}

pub(crate) fn validate_execution_obligation_preparation(
    grant: &ExecutionGrant,
    prepared: &PreparedEffect,
) -> Result<(), String> {
    grant.validate_integrity()?;
    prepared.validate()?;
    if grant
        .execution_evidence_requirements
        .contains(&ExecutionEvidenceRequirement::PreObservation)
        && (!grant.require_pre_observation
            || prepared.expected_pre_observation.observation_id.is_empty()
            || prepared.expected_pre_observation.resource_attachment_id
                != grant.resource_attachment_id
            || prepared.expected_pre_observation.relative_path != grant.normalized_target)
    {
        return Err("required_pre_observation_evidence_missing".to_string());
    }
    Ok(())
}

pub(crate) fn validate_execution_obligation_closure(
    grant: &ExecutionGrant,
    prepared: &PreparedEffect,
    post_observation: &FilesystemObservation,
    receipt: &EffectReceipt,
) -> Result<(), String> {
    validate_execution_obligation_preparation(grant, prepared)?;
    post_observation.validate()?;
    if grant
        .execution_evidence_requirements
        .contains(&ExecutionEvidenceRequirement::PostObservation)
        && (!grant.require_post_observation
            || receipt.post_observation_id != post_observation.observation_id
            || post_observation.resource_attachment_id != grant.resource_attachment_id
            || post_observation.relative_path != grant.normalized_target)
    {
        return Err("required_post_observation_evidence_missing".to_string());
    }
    Ok(())
}

/// Mechanically validates closure of one finalized effect chain. Identity
/// links must exist as typed payload fields; transition order, timestamps and
/// presentation summaries are never used to infer missing links.
pub fn validate_finalized_effect_chain(
    transitions: &[Transition],
    effect_id: &str,
) -> Result<EffectChainValidation, String> {
    for transition in transitions {
        transition.validate()?;
    }
    let prepared = transitions
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::EffectPrepared { prepared } if prepared.effect_id == effect_id => {
                Some(prepared)
            }
            _ => None,
        })
        .ok_or_else(|| "effect_chain_missing_prepare".to_string())?;
    let operation = transitions
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::OperationRecorded { operation }
                if operation.operation_id == prepared.operation_id =>
            {
                Some(operation)
            }
            _ => None,
        })
        .ok_or_else(|| "effect_chain_missing_operation".to_string())?;
    let decision = transitions
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::DecisionRecorded { decision }
                if decision.decision_id == prepared.decision_id =>
            {
                Some(decision)
            }
            _ => None,
        })
        .ok_or_else(|| "effect_chain_missing_decision".to_string())?;
    let grant = transitions
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::ExecutionGrantIssued { grant }
                if grant.grant_id == prepared.grant_id =>
            {
                Some(grant)
            }
            _ => None,
        })
        .ok_or_else(|| "effect_chain_missing_grant".to_string())?;
    let (post_observation, receipt) = transitions
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::EffectFinalized {
                effect_id: finalized,
                post_observation,
                receipt,
            } if finalized == effect_id => Some((post_observation, receipt)),
            TransitionPayload::EffectReconciled {
                effect_id: reconciled,
                observation,
                receipt: Some(receipt),
                ..
            } if reconciled == effect_id => Some((observation, receipt)),
            _ => None,
        })
        .ok_or_else(|| "effect_chain_missing_finalization".to_string())?;

    validate_grant(operation, decision, grant, grant.expected_case_generation)?;
    validate_execution_obligation_closure(grant, prepared, post_observation, receipt)?;
    if prepared.operation_id != operation.operation_id
        || prepared.decision_id != decision.decision_id
        || prepared.grant_id != grant.grant_id
        || prepared.resource_attachment_id != operation.resource_attachment_id
        || prepared.relative_path != operation.filesystem_write.relative_path
        || prepared.intended_content_digest != operation.filesystem_write.content_digest
        || receipt.effect_id != prepared.effect_id
        || receipt.operation_id != operation.operation_id
        || receipt.decision_id != decision.decision_id
        || receipt.grant_id != grant.grant_id
        || receipt.resource_attachment_id != prepared.resource_attachment_id
        || receipt.pre_observation_id != prepared.expected_pre_observation.observation_id
        || receipt.post_observation_id != post_observation.observation_id
    {
        return Err("effect_chain_identity_mismatch".to_string());
    }
    Ok(EffectChainValidation {
        effect_id: effect_id.to_string(),
        operation_id: operation.operation_id.clone(),
        decision_id: decision.decision_id.clone(),
        grant_id: grant.grant_id.clone(),
        receipt_id: receipt.receipt_id.clone(),
        outcome: receipt.outcome.clone(),
    })
}

pub fn classify_reconciliation(
    prepared: &PreparedEffect,
    current: &FilesystemObservation,
) -> ReconciliationConclusion {
    if current.state == ResourceState::File
        && current.content_digest.as_deref() == Some(&prepared.intended_content_digest)
    {
        ReconciliationConclusion::EffectObserved
    } else if observations_equivalent(current, &prepared.expected_pre_observation) {
        ReconciliationConclusion::NoEffectObserved
    } else if current.state == ResourceState::Unavailable {
        ReconciliationConclusion::StillIndeterminate
    } else {
        ReconciliationConclusion::Conflict
    }
}

pub fn normalize_relative_path(value: &str) -> Result<String, String> {
    if value.is_empty() || value.as_bytes().contains(&0) {
        return Err("relative path is empty or contains NUL".to_string());
    }
    let path = Path::new(value);
    if path.is_absolute() {
        return Err("absolute paths are not allowed".to_string());
    }
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                let part = part
                    .to_str()
                    .ok_or_else(|| "path is not valid UTF-8".to_string())?;
                if part.is_empty() {
                    return Err("empty path component".to_string());
                }
                parts.push(part);
            }
            Component::CurDir => return Err("dot path components are not allowed".to_string()),
            Component::ParentDir => return Err("parent traversal is not allowed".to_string()),
            Component::RootDir | Component::Prefix(_) => {
                return Err("rooted paths are not allowed".to_string())
            }
        }
    }
    if parts.is_empty() {
        return Err("relative path has no normal components".to_string());
    }
    Ok(parts.join("/"))
}

pub fn normalize_write_prefix(value: &str) -> Result<String, String> {
    normalize_relative_path(value)
}

pub fn path_within_prefix(prefix: &str, relative_path: &str) -> bool {
    relative_path == prefix || relative_path.starts_with(&format!("{prefix}/"))
}

pub fn digest_bytes(bytes: &[u8]) -> String {
    let digest = sha256(bytes);
    let mut output = String::with_capacity(71);
    output.push_str("sha256:");
    for byte in digest {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn digest_serialized(value: &impl Serialize) -> String {
    let bytes = serde_json::to_vec(value).expect("canonical digest material serializes");
    digest_bytes(&bytes)
}

fn digest_suffix(value: &str, length: usize) -> &str {
    let suffix = value.strip_prefix("sha256:").unwrap_or(value);
    &suffix[..suffix.len().min(length)]
}

fn resolve_target(
    binding: &LocalFilesystemBinding,
    resource: &ResourceAttachmentState,
    relative_path: &str,
) -> Result<PathBuf, String> {
    binding.validate()?;
    if binding.attachment_id != resource.attachment_id {
        return Err("binding_attachment_mismatch".to_string());
    }
    let normalized = normalize_relative_path(relative_path)?;
    let root = fs::canonicalize(&binding.canonical_root)
        .map_err(|error| format!("binding_root_unavailable: {error}"))?;
    let target = root.join(&normalized);
    let parent = target
        .parent()
        .ok_or_else(|| "filesystem target has no parent".to_string())?;
    let canonical_parent =
        fs::canonicalize(parent).map_err(|error| format!("target_parent_unavailable: {error}"))?;
    if !canonical_parent.starts_with(&root) {
        return Err("filesystem binding symlink escape rejected".to_string());
    }
    Ok(target)
}

fn atomic_replace_legacy(target: &Path, content: &[u8], effect_id: &str) -> Result<(), String> {
    let parent = target
        .parent()
        .ok_or_else(|| "filesystem target has no parent".to_string())?;
    let safe_effect = effect_id
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    let temp = parent.join(format!(".yai-{safe_effect}.tmp"));
    match fs::remove_file(&temp) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("failed to remove stale effect temp file: {error}")),
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|error| format!("failed to create effect temp file: {error}"))?;
    file.write_all(content)
        .map_err(|error| format!("failed to write effect temp file: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("failed to sync effect temp file: {error}"))?;
    fs::rename(&temp, target)
        .map_err(|error| format!("failed to atomically replace target: {error}"))?;
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| format!("failed to sync target directory: {error}"))?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn secure_atomic_replace(
    binding: &LocalFilesystemBinding,
    resource: &ResourceAttachmentState,
    relative_path: &str,
    content: &[u8],
    effect_id: &str,
) -> Result<(), String> {
    secure_atomic_replace_with_hook(binding, resource, relative_path, content, effect_id, || {
        Ok(())
    })
}

#[cfg(target_os = "linux")]
fn secure_atomic_replace_with_hook<F: FnOnce() -> Result<(), String>>(
    binding: &LocalFilesystemBinding,
    resource: &ResourceAttachmentState,
    relative_path: &str,
    content: &[u8],
    effect_id: &str,
    after_parent_open: F,
) -> Result<(), String> {
    binding.validate_secure_carrier()?;
    if binding.attachment_id != resource.attachment_id {
        return Err("binding_attachment_mismatch".to_string());
    }
    let normalized = normalize_relative_path(relative_path)?;
    if !path_within_prefix(&resource.allowed_write_prefix, &normalized) {
        return Err("filesystem_target_outside_allowed_prefix".to_string());
    }
    let mut components = normalized.split('/').collect::<Vec<_>>();
    let target_name = components
        .pop()
        .ok_or_else(|| "filesystem_target_name_missing".to_string())?;
    let root = open_verified_filesystem_root(binding)?;
    let parent = if components.is_empty() {
        root.try_clone()
            .map_err(|error| format!("filesystem_root_clone_failed: {error}"))?
    } else {
        open_beneath(
            &root,
            Path::new(&components.join("/")),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            0,
        )
        .map_err(|error| format!("filesystem_secure_parent_resolution_rejected: {error}"))?
    };

    // Tests use this seam to replace the pathname after the trusted parent
    // directory descriptor has been opened. All subsequent operations remain
    // descriptor-relative and cannot follow the replacement outside the root.
    after_parent_open()?;

    let target_name = CString::new(target_name.as_bytes())
        .map_err(|_| "filesystem_target_name_contains_nul".to_string())?;
    let mut target_stat = std::mem::MaybeUninit::<libc::stat>::uninit();
    let stat_result = unsafe {
        libc::fstatat(
            parent.as_raw_fd(),
            target_name.as_ptr(),
            target_stat.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if stat_result == 0 {
        let target_stat = unsafe { target_stat.assume_init() };
        let file_type = target_stat.st_mode & libc::S_IFMT;
        if file_type == libc::S_IFLNK {
            return Err("filesystem_final_symlink_rejected".to_string());
        }
        if file_type != libc::S_IFREG {
            return Err("filesystem_target_type_not_replaceable".to_string());
        }
    } else {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::ENOENT) {
            return Err(format!("filesystem_target_fstatat_failed: {error}"));
        }
    }

    let safe_effect = effect_id
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    let temp_name = CString::new(format!(".yai-{safe_effect}.tmp"))
        .map_err(|_| "filesystem_temp_name_contains_nul".to_string())?;
    let unlink_result = unsafe { libc::unlinkat(parent.as_raw_fd(), temp_name.as_ptr(), 0) };
    if unlink_result != 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::ENOENT) {
            return Err(format!("failed to remove stale effect temp file: {error}"));
        }
    }
    let temp_fd = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            temp_name.as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            0o600,
        )
    };
    if temp_fd < 0 {
        return Err(format!(
            "failed to create descriptor-relative effect temp file: {}",
            std::io::Error::last_os_error()
        ));
    }
    let mut temp = unsafe { File::from_raw_fd(temp_fd) };
    let write_result = temp
        .write_all(content)
        .and_then(|()| temp.sync_all())
        .map_err(|error| format!("failed to write/sync effect temp file: {error}"));
    drop(temp);
    if let Err(error) = write_result {
        unsafe {
            libc::unlinkat(parent.as_raw_fd(), temp_name.as_ptr(), 0);
        }
        return Err(error);
    }
    let rename_result = unsafe {
        libc::renameat(
            parent.as_raw_fd(),
            temp_name.as_ptr(),
            parent.as_raw_fd(),
            target_name.as_ptr(),
        )
    };
    if rename_result != 0 {
        let error = std::io::Error::last_os_error();
        unsafe {
            libc::unlinkat(parent.as_raw_fd(), temp_name.as_ptr(), 0);
        }
        return Err(format!(
            "failed to atomically replace descriptor-relative target: {error}"
        ));
    }
    let sync_result = unsafe { libc::fsync(parent.as_raw_fd()) };
    if sync_result != 0 {
        return Err(format!(
            "failed to sync descriptor-relative target directory: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn secure_atomic_replace(
    _binding: &LocalFilesystemBinding,
    _resource: &ResourceAttachmentState,
    _relative_path: &str,
    _content: &[u8],
    _effect_id: &str,
) -> Result<(), String> {
    Err("filesystem_secure_resolution_unsupported_platform".to_string())
}

fn observations_equivalent(left: &FilesystemObservation, right: &FilesystemObservation) -> bool {
    left.resource_attachment_id == right.resource_attachment_id
        && left.relative_path == right.relative_path
        && left.state == right.state
        && left.content_digest == right.content_digest
        && left.size_bytes == right.size_bytes
}

fn unavailable_observation(
    observation_id: String,
    resource_attachment_id: &str,
    relative_path: &str,
    error: String,
) -> FilesystemObservation {
    FilesystemObservation {
        schema: OBSERVATION_SCHEMA.to_string(),
        observation_id,
        resource_attachment_id: resource_attachment_id.to_string(),
        relative_path: relative_path.to_string(),
        state: ResourceState::Unavailable,
        content_digest: None,
        size_bytes: None,
        error: Some(error),
        observed_at_unix_ms: unix_time_ms(),
    }
}

fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// Compact, dependency-free SHA-256. Test vectors below freeze the digest
// contract because these digests participate in operation/grant identity.
fn sha256(input: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut message = input.to_vec();
    let bit_len = (message.len() as u64).wrapping_mul(8);
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());
    let mut h = [
        0x6a09e667u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];
    for block in message.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (index, word) in block.chunks_exact(4).enumerate() {
            w[index] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for index in 16..64 {
            let s0 = w[index - 15].rotate_right(7)
                ^ w[index - 15].rotate_right(18)
                ^ (w[index - 15] >> 3);
            let s1 = w[index - 2].rotate_right(17)
                ^ w[index - 2].rotate_right(19)
                ^ (w[index - 2] >> 10);
            w[index] = w[index - 16]
                .wrapping_add(s0)
                .wrapping_add(w[index - 7])
                .wrapping_add(s1);
        }
        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];
        for index in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[index])
                .wrapping_add(w[index]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    let mut output = [0u8; 32];
    for (index, value) in h.iter().enumerate() {
        output[index * 4..index * 4 + 4].copy_from_slice(&value.to_be_bytes());
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resource(prefix: &str, limit: usize) -> ResourceAttachmentState {
        ResourceAttachmentState {
            attachment_id: "workspace".to_string(),
            kind: crate::transition::ResourceKind::Filesystem,
            allowed_write_prefix: prefix.to_string(),
            max_write_bytes: limit,
            policy_id: "policy:workspace".to_string(),
            policy_owner_participant_id: "participant:operator".to_string(),
            review_requirement: crate::transition::ReviewRequirement::Automatic,
            process_signal_actions: Vec::new(),
        }
    }

    fn normalize(
        raw: &str,
        resource: &ResourceAttachmentState,
    ) -> Result<Operation, NormalizationFailure> {
        normalize_filesystem_write_candidate(
            raw,
            &NormalizationContext {
                case_id: "case:test",
                participant_id: "participant:model",
                provider_result_id: "provider-result:1",
                provider_invocation_id: "invocation:1",
                case_generation: 7,
                resource,
            },
        )
    }

    #[test]
    fn sha256_contract_matches_standard_vectors() {
        assert_eq!(
            digest_bytes(b""),
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            digest_bytes(b"abc"),
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn path_normalization_rejects_escape_forms() {
        assert_eq!(
            normalize_relative_path("allowed/hello.txt").unwrap(),
            "allowed/hello.txt"
        );
        for invalid in [
            "",
            "../escape",
            "allowed/../escape",
            "/tmp/escape",
            "./hello",
        ] {
            assert!(
                normalize_relative_path(invalid).is_err(),
                "accepted {invalid}"
            );
        }
    }

    #[test]
    fn h14_process_signal_retry_matrix_defaults_to_observation_not_repetition() {
        assert_eq!(
            process_signal_retry_posture(&ProcessSignalAction::Terminate),
            ProcessEffectRetryPosture::UnsafeOrAmbiguousToRepeat
        );
        assert_eq!(
            process_signal_retry_posture(&ProcessSignalAction::Suspend),
            ProcessEffectRetryPosture::ObservationOnlyRecovery
        );
        assert_eq!(
            process_signal_retry_posture(&ProcessSignalAction::Resume),
            ProcessEffectRetryPosture::ObservationOnlyRecovery
        );
    }

    #[test]
    fn provider_candidate_is_strictly_normalized_before_operation_identity() {
        let resource = resource("allowed", 5);
        let operation = normalize(
            r#"{"schema":"yai.operation_proposal.filesystem_write.v1","operation":"filesystem.write","resource":"workspace","path":"allowed/hello.txt","content":"hello"}"#,
            &resource,
        )
        .expect("normalize exact proposal");
        assert_eq!(operation.kind, OperationKind::FilesystemWrite);
        assert_eq!(operation.expected_case_generation, 7);
        assert_eq!(operation.filesystem_write.content_bytes, 5);
        operation.validate().expect("operation integrity");

        let invalid = [
            ("not json", NormalizationFailureCode::MalformedJson),
            (
                r#"{"schema":"yai.operation_proposal.filesystem_write.v1","operation":"filesystem.write","resource":"workspace","path":"../escape","content":"x"}"#,
                NormalizationFailureCode::InvalidRelativePath,
            ),
            (
                r#"{"schema":"yai.operation_proposal.filesystem_write.v1","operation":"filesystem.write","resource":"other","path":"allowed/x","content":"x"}"#,
                NormalizationFailureCode::AttachmentMismatch,
            ),
            (
                r#"{"schema":"yai.operation_proposal.filesystem_write.v1","operation":"filesystem.write","resource":"workspace","path":"allowed/x","content":"123456"}"#,
                NormalizationFailureCode::PayloadTooLarge,
            ),
            (
                r#"{"schema":"yai.operation_proposal.filesystem_write.v1","operation":"shell","resource":"workspace","path":"allowed/x","content":"x"}"#,
                NormalizationFailureCode::UnsupportedOperation,
            ),
        ];
        for (raw, expected) in invalid {
            assert_eq!(normalize(raw, &resource).unwrap_err().code, expected);
        }
    }

    #[test]
    fn deny_cannot_issue_grant_and_grant_tampering_is_detected() {
        let resource = resource("allowed", 1024);
        let denied_operation = normalize(
            r#"{"schema":"yai.operation_proposal.filesystem_write.v1","operation":"filesystem.write","resource":"workspace","path":"other/hello.txt","content":"hello"}"#,
            &resource,
        )
        .expect("normalization does not decide policy");
        let denied = decide_filesystem_write(&denied_operation, &resource, 8);
        assert_eq!(denied.outcome, DecisionOutcome::Deny);
        assert!(issue_execution_grant(&denied_operation, &denied, 8).is_err());

        let allowed_operation = normalize(
            r#"{"schema":"yai.operation_proposal.filesystem_write.v1","operation":"filesystem.write","resource":"workspace","path":"allowed/hello.txt","content":"hello"}"#,
            &resource,
        )
        .expect("allowed operation");
        let allowed = decide_filesystem_write(&allowed_operation, &resource, 8);
        let mut grant =
            issue_execution_grant(&allowed_operation, &allowed, 9).expect("issue grant");
        assert!(validate_grant(&allowed_operation, &allowed, &grant, 10)
            .unwrap_err()
            .contains("contract_mismatch"));
        let mut wrong_case = grant.clone();
        wrong_case.case_id = "case:other".to_string();
        assert!(validate_grant(&allowed_operation, &allowed, &wrong_case, 9).is_err());
        let mut wrong_participant = grant.clone();
        wrong_participant.participant_id = "participant:other".to_string();
        assert!(validate_grant(&allowed_operation, &allowed, &wrong_participant, 9).is_err());
        grant.normalized_target = "allowed/tampered.txt".to_string();
        assert!(validate_grant(&allowed_operation, &allowed, &grant, 9)
            .unwrap_err()
            .contains("contract_mismatch"));
    }

    #[test]
    fn human_review_is_operation_bound_and_only_effective_allow_can_issue_grant() {
        let mut resource = resource("allowed", 1024);
        resource.review_requirement = ReviewRequirement::RequireReview;
        let operation = normalize(
            r#"{"schema":"yai.operation_proposal.filesystem_write.v1","operation":"filesystem.write","resource":"workspace","path":"allowed/reviewed.txt","content":"reviewed"}"#,
            &resource,
        )
        .expect("normalize review-bound operation");
        let initial = decide_filesystem_write(&operation, &resource, 8);
        assert_eq!(initial.outcome, DecisionOutcome::RequireReview);
        assert!(issue_execution_grant(&operation, &initial, 9).is_err());

        let mut review = build_filesystem_review_request(&operation, &initial, &resource, 9)
            .expect("build typed review request");
        let action = crate::transition::build_review_action(
            &review,
            &operation.case_id,
            &resource.policy_owner_participant_id,
            ReviewActionKind::Approve,
            "operator accepts this exact operation",
            10,
            "local_cli_claimed_participant",
        )
        .expect("build integrity-bound action");
        let mut tampered = action.clone();
        tampered.reason = "tampered".to_string();
        assert!(tampered.validate_integrity().is_err());

        review.status = ReviewResolution::Approved;
        review.latest_action_id = Some(action.action_id.clone());
        let effective =
            resolve_filesystem_review_decision(&operation, &resource, &review, &action, 11)
                .expect("derive effective allow from committed action posture");
        assert_eq!(effective.outcome, DecisionOutcome::Allow);
        assert!(effective.basis_refs.contains(&review.review_id));
        assert!(effective.basis_refs.contains(&action.action_id));
        issue_execution_grant(&operation, &effective, 12)
            .expect("only effective allow can issue grant");

        let wrong_reviewer = crate::transition::build_review_action(
            &review,
            &operation.case_id,
            "participant:intruder",
            ReviewActionKind::Approve,
            "not eligible",
            10,
            "local_cli_claimed_participant",
        )
        .expect("action construction is separate from eligibility");
        assert!(resolve_filesystem_review_decision(
            &operation,
            &resource,
            &review,
            &wrong_reviewer,
            11,
        )
        .is_err());
    }

    #[test]
    fn carrier_refuses_resource_pre_state_mismatch_without_mutation() {
        use crate::transition::{CaseLifecycle, EffectLifecycle, EffectState};

        let root = std::env::temp_dir().join(format!(
            "yai-effect-pre-state-{}-{}",
            std::process::id(),
            unix_time_ms()
        ));
        fs::create_dir_all(root.join("allowed")).expect("create resource root");
        let resource = resource("allowed", 1024);
        let binding =
            LocalFilesystemBinding::new("case:test", "workspace", &root).expect("binding");
        let operation = normalize(
            r#"{"schema":"yai.operation_proposal.filesystem_write.v1","operation":"filesystem.write","resource":"workspace","path":"allowed/hello.txt","content":"intended"}"#,
            &resource,
        )
        .expect("operation");
        let decision = decide_filesystem_write(&operation, &resource, 8);
        let grant = issue_execution_grant(&operation, &decision, 9).expect("grant");
        let pre = observe_filesystem(&binding, &resource, "allowed/hello.txt", "observation:pre");
        assert_eq!(pre.state, ResourceState::Absent);
        let prepared = prepare_effect(&operation, &decision, &grant, pre).expect("prepare");

        fs::write(root.join("allowed/hello.txt"), b"third-party-change")
            .expect("inject conflicting pre-state");
        let mut state = CaseState::new("case:test", CaseLifecycle::Open);
        state.generation = 11;
        state.effects.push(EffectState {
            effect_id: prepared.effect_id.clone(),
            operation_id: prepared.operation_id.clone(),
            decision_id: prepared.decision_id.clone(),
            grant_id: prepared.grant_id.clone(),
            resource_attachment_id: prepared.resource_attachment_id.clone(),
            relative_path: prepared.relative_path.clone(),
            intended_content_digest: prepared.intended_content_digest.clone(),
            pre_observation_id: prepared.expected_pre_observation.observation_id.clone(),
            post_observation_id: None,
            receipt_id: None,
            outcome: None,
            kind: OperationKind::FilesystemWrite,
            status: EffectLifecycle::Prepared,
            prepared_at_generation: 11,
            updated_at_generation: 11,
        });

        let result = execute_filesystem_write(
            &operation,
            &decision,
            &grant,
            &prepared,
            &state,
            &binding,
            &resource,
            CarrierFailpoint::None,
        )
        .expect("carrier classifies conflict");
        assert_eq!(result.outcome, EffectOutcome::Conflict);
        assert!(!result.carrier_attempted);
        assert!(!result.mutation_performed);
        assert_eq!(
            fs::read(root.join("allowed/hello.txt")).expect("conflicting file remains"),
            b"third-party-change"
        );
        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn observation_fails_closed_on_symlink_parent_escape() {
        use std::os::unix::fs::symlink;
        let unique = format!(
            "yai-effect-symlink-{}-{}",
            std::process::id(),
            unix_time_ms()
        );
        let root = std::env::temp_dir().join(unique);
        let outside = root.with_extension("outside");
        fs::create_dir_all(root.join("allowed")).expect("create root");
        fs::create_dir_all(&outside).expect("create outside");
        symlink(&outside, root.join("allowed/link")).expect("create symlink");
        let binding =
            LocalFilesystemBinding::new("case:test", "workspace", &root).expect("local binding");
        let observation = observe_filesystem(
            &binding,
            &resource("allowed", 1024),
            "allowed/link/escape.txt",
            "observation:escape",
        );
        assert_eq!(observation.state, ResourceState::Unavailable);
        assert!(observation
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("secure_parent_resolution_rejected"));
        fs::remove_dir_all(root).expect("remove root");
        fs::remove_dir_all(outside).expect("remove outside");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn h14_descriptor_relative_carrier_rejects_final_symlink_and_preserves_outside() {
        use std::os::unix::fs::symlink;
        let unique = format!(
            "yai-h14-final-symlink-{}-{}",
            std::process::id(),
            unix_time_ms()
        );
        let root = std::env::temp_dir().join(&unique);
        let outside = std::env::temp_dir().join(format!("{unique}-outside"));
        fs::create_dir_all(root.join("allowed")).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("victim.txt"), b"outside-before").unwrap();
        symlink(outside.join("victim.txt"), root.join("allowed/link.txt")).unwrap();
        let binding = LocalFilesystemBinding::new("case:h14", "workspace", &root).unwrap();
        let error = secure_atomic_replace(
            &binding,
            &resource("allowed", 1024),
            "allowed/link.txt",
            b"attacker-content",
            "effect:h14-final-symlink",
        )
        .expect_err("final symlink must be rejected");
        assert_eq!(error, "filesystem_final_symlink_rejected");
        assert_eq!(
            fs::read(outside.join("victim.txt")).unwrap(),
            b"outside-before"
        );
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn h14_post_prepare_parent_swap_cannot_escape_open_directory_descriptor() {
        use std::os::unix::fs::symlink;
        let unique = format!(
            "yai-h14-parent-swap-{}-{}",
            std::process::id(),
            unix_time_ms()
        );
        let root = std::env::temp_dir().join(&unique);
        let outside = std::env::temp_dir().join(format!("{unique}-outside"));
        fs::create_dir_all(root.join("allowed/safe")).unwrap();
        fs::create_dir_all(&outside).unwrap();
        let binding = LocalFilesystemBinding::new("case:h14", "workspace", &root).unwrap();
        secure_atomic_replace_with_hook(
            &binding,
            &resource("allowed", 1024),
            "allowed/safe/result.txt",
            b"contained",
            "effect:h14-parent-swap",
            || {
                fs::rename(
                    root.join("allowed/safe"),
                    root.join("allowed/safe-original"),
                )
                .map_err(|error| error.to_string())?;
                symlink(&outside, root.join("allowed/safe")).map_err(|error| error.to_string())?;
                Ok(())
            },
        )
        .expect("descriptor-relative write remains on the already-open safe directory");
        assert!(!outside.join("result.txt").exists());
        assert_eq!(
            fs::read(root.join("allowed/safe-original/result.txt")).unwrap(),
            b"contained"
        );
        let observation = observe_filesystem(
            &binding,
            &resource("allowed", 1024),
            "allowed/safe/result.txt",
            "observation:h14-parent-swap",
        );
        assert_eq!(observation.state, ResourceState::Unavailable);
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn h14_atomic_replace_does_not_mutate_an_outside_hard_link_inode() {
        let unique = format!("yai-h14-hardlink-{}-{}", std::process::id(), unix_time_ms());
        let root = std::env::temp_dir().join(&unique);
        let outside = std::env::temp_dir().join(format!("{unique}-outside.txt"));
        fs::create_dir_all(root.join("allowed")).unwrap();
        fs::write(&outside, b"shared-before").unwrap();
        fs::hard_link(&outside, root.join("allowed/linked.txt")).unwrap();
        let binding = LocalFilesystemBinding::new("case:h14", "workspace", &root).unwrap();
        secure_atomic_replace(
            &binding,
            &resource("allowed", 1024),
            "allowed/linked.txt",
            b"inside-after",
            "effect:h14-hardlink",
        )
        .unwrap();
        assert_eq!(fs::read(&outside).unwrap(), b"shared-before");
        assert_eq!(
            fs::read(root.join("allowed/linked.txt")).unwrap(),
            b"inside-after"
        );
        fs::remove_dir_all(root).unwrap();
        fs::remove_file(outside).unwrap();
    }
}
