//! Controlled filesystem effect contract and carrier mechanics.
//!
//! This module owns the single `filesystem.write` normalization and resource
//! boundary proved by SOURCE.REFOUNDATION.3. It does not own provider
//! execution, policy languages, carrier registries, or any C semantic mirror.

use crate::admission::{DecisionBasis, ExecutionEvidenceRequirement};
use crate::transition::{
    CaseState, ResourceAttachmentState, ReviewAction, ReviewActionKind, ReviewRequirement,
    ReviewResolution, ReviewState, Transition, TransitionPayload, TransitionScope,
    REVIEW_REQUEST_SCHEMA_V1,
};
use serde::{Deserialize, Serialize};
use std::fmt::Write as FmtWrite;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const OPERATION_PROPOSAL_SCHEMA: &str = "yai.operation_proposal.filesystem_write.v1";
pub const OPERATION_SCHEMA: &str = "yai.operation.v1";
pub const DECISION_SCHEMA: &str = "yai.decision.v2";
pub const DECISION_SCHEMA_V1: &str = "yai.decision.v1";
pub const EXECUTION_GRANT_SCHEMA: &str = "yai.execution_grant.v2";
pub const EXECUTION_GRANT_SCHEMA_V1: &str = "yai.execution_grant.v1";
pub const OBSERVATION_SCHEMA: &str = "yai.observation.filesystem.v1";
pub const PREPARED_EFFECT_SCHEMA: &str = "yai.prepared_effect.v1";
pub const EFFECT_RECEIPT_SCHEMA: &str = "yai.effect_receipt.v1";
pub const LOCAL_FILESYSTEM_BINDING_SCHEMA: &str = "yai.local_filesystem_binding.v1";
pub const FILESYSTEM_CARRIER_BACKEND: &str = "rust.filesystem.atomic_replace.v1";
pub const DEFAULT_MAX_WRITE_BYTES: usize = 65_536;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilesystemWriteProposal {
    pub schema: String,
    pub operation: String,
    pub resource: String,
    pub path: String,
    pub content: String,
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    FilesystemWrite,
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FilesystemWritePayload {
    pub relative_path: String,
    pub content: String,
    pub content_digest: String,
    pub content_bytes: usize,
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
        schema: OPERATION_SCHEMA,
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
        schema: OPERATION_SCHEMA.to_string(),
        operation_id: format!("operation:{}", digest_suffix(&operation_digest, 32)),
        operation_digest,
        case_id: case_id.to_string(),
        participant_id: participant_id.to_string(),
        scope,
        kind: OperationKind::FilesystemWrite,
        resource_attachment_id: resource.attachment_id.clone(),
        filesystem_write,
        origin,
        expected_case_generation: case_generation,
    }
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
    if current_case_generation < decision.decided_at_case_generation + 1
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
    let material = GrantDigestMaterialV2 {
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
        permitted_effect: &GrantedEffect::FilesystemWrite,
        normalized_target: &operation.filesystem_write.relative_path,
        intended_content_digest: &operation.filesystem_write.content_digest,
        expected_case_generation: current_case_generation,
        idempotency_key: &idempotency_key,
        require_pre_observation: true,
        require_post_observation: true,
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
        permitted_effect: GrantedEffect::FilesystemWrite,
        normalized_target: operation.filesystem_write.relative_path.clone(),
        intended_content_digest: operation.filesystem_write.content_digest.clone(),
        expected_case_generation: current_case_generation,
        idempotency_key,
        require_pre_observation: true,
        require_post_observation: true,
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
        schema: PREPARED_EFFECT_SCHEMA.to_string(),
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
    if (grant.schema != EXECUTION_GRANT_SCHEMA && grant.schema != EXECUTION_GRANT_SCHEMA_V1)
        || grant.operation_id != operation.operation_id
        || grant.operation_digest != operation.operation_digest
        || grant.decision_id != decision.decision_id
        || grant.decision_digest != decision.decision_digest
        || grant.case_id != operation.case_id
        || grant.participant_id != operation.participant_id
        || grant.resource_attachment_id != operation.resource_attachment_id
        || grant.normalized_target != operation.filesystem_write.relative_path
        || grant.intended_content_digest != operation.filesystem_write.content_digest
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
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != OPERATION_SCHEMA
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
        if normalize_relative_path(&self.filesystem_write.relative_path)?
            != self.filesystem_write.relative_path
            || digest_bytes(self.filesystem_write.content.as_bytes())
                != self.filesystem_write.content_digest
            || self.filesystem_write.content.len() != self.filesystem_write.content_bytes
        {
            return Err("operation_payload_integrity_mismatch".to_string());
        }
        let material = OperationDigestMaterial {
            schema: &self.schema,
            case_id: &self.case_id,
            participant_id: &self.participant_id,
            scope: &self.scope,
            kind: &self.kind,
            resource_attachment_id: &self.resource_attachment_id,
            filesystem_write: &self.filesystem_write,
            origin: &self.origin,
            expected_case_generation: self.expected_case_generation,
        };
        if digest_serialized(&material) != self.operation_digest {
            return Err("operation_digest_mismatch".to_string());
        }
        Ok(())
    }
}

impl Decision {
    pub fn validate_integrity(&self) -> Result<(), String> {
        if (self.schema != DECISION_SCHEMA && self.schema != DECISION_SCHEMA_V1)
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
        if (self.schema != EXECUTION_GRANT_SCHEMA && self.schema != EXECUTION_GRANT_SCHEMA_V1)
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
        Ok(Self {
            schema: LOCAL_FILESYSTEM_BINDING_SCHEMA.to_string(),
            case_id: case_id.into(),
            attachment_id: attachment_id.into(),
            canonical_root: canonical.to_string_lossy().into_owned(),
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != LOCAL_FILESYSTEM_BINDING_SCHEMA
            || self.case_id.is_empty()
            || self.attachment_id.is_empty()
            || !Path::new(&self.canonical_root).is_absolute()
        {
            return Err("invalid_local_filesystem_binding".to_string());
        }
        Ok(())
    }
}

pub fn observe_filesystem(
    binding: &LocalFilesystemBinding,
    resource: &ResourceAttachmentState,
    relative_path: &str,
    observation_id: impl Into<String>,
) -> FilesystemObservation {
    let observation_id = observation_id.into();
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
    validate_grant(operation, decision, grant, grant.expected_case_generation)?;
    binding.validate()?;
    if prepared.schema != PREPARED_EFFECT_SCHEMA
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
        || effect_state.prepared_at_generation != case_state.generation
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

    let target = resolve_target(binding, resource, &prepared.relative_path)?;
    atomic_replace(
        &target,
        operation.filesystem_write.content.as_bytes(),
        &prepared.effect_id,
    )?;
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

fn atomic_replace(target: &Path, content: &[u8], effect_id: &str) -> Result<(), String> {
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

    #[cfg(unix)]
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
            .contains("symlink escape"));
        fs::remove_dir_all(root).expect("remove root");
        fs::remove_dir_all(outside).expect("remove outside");
    }
}
