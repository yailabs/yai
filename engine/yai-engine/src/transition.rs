//! Canonical committed-transition and materialized CaseState semantics.
//!
//! `Transition` is immutable historical authority. `CaseState` is a
//! deterministic, rebuildable reduction of a Case's ordered transitions.
//! Provider output and review outcomes are represented as typed payloads; the
//! optional summary is presentation material and is never read by the reducer.

use crate::case_policy::CasePolicyBinding;
use crate::effect::{
    digest_bytes, Decision, DecisionOutcome, EffectOutcome, EffectReceipt, ExecutionGrant,
    FilesystemObservation, NormalizationFailure, Operation, OperationOrigin, PreparedEffect,
    ReconciliationConclusion, EFFECT_RECEIPT_SCHEMA, OBSERVATION_SCHEMA, PREPARED_EFFECT_SCHEMA,
};
use serde::{Deserialize, Serialize};

pub const TRANSITION_SCHEMA_V1: &str = "yai.transition.v1";
pub const TRANSITION_SCHEMA_V2: &str = "yai.transition.v2";
pub const TRANSITION_SCHEMA_V3: &str = "yai.transition.v3";
pub const TRANSITION_SCHEMA_V4: &str = "yai.transition.v4";
pub const TRANSITION_SCHEMA_V5: &str = "yai.transition.v5";
pub const TRANSITION_SCHEMA_V6: &str = "yai.transition.v6";
pub const TRANSITION_SCHEMA: &str = "yai.transition.v7";
pub const CASE_STATE_SCHEMA_V1: &str = "yai.case_state.v1";
pub const CASE_STATE_SCHEMA_V2: &str = "yai.case_state.v2";
pub const CASE_STATE_SCHEMA_V3: &str = "yai.case_state.v3";
pub const CASE_STATE_SCHEMA_V4: &str = "yai.case_state.v4";
pub const CASE_STATE_SCHEMA_V5: &str = "yai.case_state.v5";
pub const CASE_STATE_SCHEMA_V6: &str = "yai.case_state.v6";
pub const CASE_STATE_SCHEMA: &str = "yai.case_state.v7";
pub const REVIEW_REQUEST_SCHEMA: &str = "yai.review_request.v2";
pub const REVIEW_REQUEST_SCHEMA_V1: &str = "yai.review_request.v1";
pub const REVIEW_ACTION_SCHEMA: &str = "yai.review_action.v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityInvalidationReason {
    PolicyRefreshRequired,
    PolicyStale,
    PolicyExpired,
    PolicyRevoked,
    PolicyBasisChanged,
    CaseCancelled,
    CaseClosed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReviewInvalidation {
    pub review_id: String,
    pub reason: AuthorityInvalidationReason,
    pub source_ref: String,
    pub invalidated_at_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantInvalidationDisposition {
    Expired,
    Revoked,
    Abandoned,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExecutionGrantInvalidation {
    pub grant_id: String,
    pub disposition: GrantInvalidationDisposition,
    pub reason: String,
    pub source_ref: String,
    pub invalidated_at_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CaseCancellationState {
    pub actor_ref: String,
    pub reason: String,
    pub requested_at_unix_ms: u64,
    pub transition_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CaseClosureState {
    pub actor_ref: String,
    pub reason: String,
    pub closed_at_unix_ms: u64,
    pub cancellation_ref: String,
    pub transition_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    pub schema: String,
    pub transition_id: String,
    pub case_id: String,
    pub sequence: u64,
    pub committed_at_unix_ms: u64,
    pub source: TransitionSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<TransitionScope>,
    #[serde(default)]
    pub causal_refs: Vec<String>,
    pub payload: TransitionPayload,
    #[serde(default)]
    pub provenance: Vec<TransitionProvenance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingTransition {
    pub transition_id: String,
    pub case_id: String,
    pub expected_generation: u64,
    pub source: TransitionSource,
    pub scope: Option<TransitionScope>,
    pub causal_refs: Vec<String>,
    pub payload: TransitionPayload,
    pub provenance: Vec<TransitionProvenance>,
    pub summary: Option<String>,
}

impl PendingTransition {
    pub fn new(
        transition_id: impl Into<String>,
        case_id: impl Into<String>,
        expected_generation: u64,
        source: TransitionSource,
        payload: TransitionPayload,
    ) -> Self {
        Self {
            transition_id: transition_id.into(),
            case_id: case_id.into(),
            expected_generation,
            source,
            scope: None,
            causal_refs: Vec::new(),
            payload,
            provenance: Vec::new(),
            summary: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransitionSource {
    pub component: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub participant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
}

impl TransitionSource {
    pub fn component(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            participant_id: None,
            source_ref: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransitionScope {
    pub case_id: String,
    #[serde(default)]
    pub participant_refs: Vec<String>,
    #[serde(default)]
    pub resource_refs: Vec<String>,
    #[serde(default)]
    pub policy_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransitionProvenance {
    pub origin_schema: String,
    pub origin_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_record_id: Option<String>,
    pub promotion: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum TransitionPayload {
    CaseOpened {
        lifecycle: CaseLifecycle,
    },
    ParticipantBound {
        participant_id: String,
        role: String,
    },
    ParticipantAdmitted {
        participant_id: String,
        consumer: String,
        view_kind: String,
    },
    ProviderAttached {
        participant_id: String,
        #[serde(default)]
        provider_id: String,
        provider_kind: String,
        base_url: String,
        model_id: String,
        credential_ref: String,
    },
    ProviderInvocationStarted {
        invocation_id: String,
        participant_id: String,
        #[serde(default)]
        provider_id: String,
        provider_kind: String,
        model_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        semantic_lineage: Option<ProviderInvocationLineage>,
    },
    ProviderResultRecorded {
        result_id: String,
        invocation_id: String,
        #[serde(default)]
        provider_id: String,
        provider_kind: String,
        model_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        semantic_lineage: Option<ProviderInvocationLineage>,
        output: String,
    },
    InteractionTurnRecorded {
        turn_id: String,
        thread_id: String,
        participant_id: String,
        invocation_id: String,
        result_id: String,
        operator_input: String,
    },
    ModelInterpretationRecorded {
        interpretation_id: String,
        result_id: String,
        authority: InterpretationAuthority,
    },
    ResourceAttached {
        attachment: ResourceAttachmentState,
    },
    OperationNormalizationFailed {
        provider_result_id: String,
        failure: NormalizationFailure,
    },
    OperationRecorded {
        operation: Operation,
    },
    DecisionRecorded {
        decision: Decision,
    },
    ExecutionGrantIssued {
        grant: ExecutionGrant,
    },
    EffectPrepared {
        prepared: PreparedEffect,
    },
    EffectFinalized {
        effect_id: String,
        post_observation: FilesystemObservation,
        receipt: EffectReceipt,
    },
    EffectIndeterminate {
        effect_id: String,
        reason: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        observation: Option<FilesystemObservation>,
    },
    EffectReconciled {
        effect_id: String,
        conclusion: ReconciliationConclusion,
        observation: FilesystemObservation,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        receipt: Option<EffectReceipt>,
    },
    ReviewRequested {
        review: ReviewState,
    },
    ReviewActionRecorded {
        action: ReviewAction,
    },
    ReviewInvalidated {
        invalidation: ReviewInvalidation,
    },
    ExecutionGrantInvalidated {
        invalidation: ExecutionGrantInvalidation,
    },
    CaseCancellationRequested {
        cancellation: CaseCancellationState,
    },
    CaseClosed {
        closure: CaseClosureState,
    },
    CasePolicyBound {
        binding: CasePolicyBinding,
    },
    CasePolicyReplaced {
        prior_binding_id: String,
        binding: CasePolicyBinding,
    },
    CasePolicyUnbound {
        binding_id: String,
        lineage_id: String,
        actor_ref: String,
        reason: String,
    },
    /// Input compatibility for pre-Wave-7 fixture review history. New writers
    /// use `ReviewActionRecorded` followed by an effective `DecisionRecorded`.
    ReviewResolved {
        review_id: String,
        attempt_id: String,
        resolution: ReviewResolution,
        reason: String,
        decision_ref: String,
        receipt_ref: String,
        carrier_attempted: bool,
        execution_performed: bool,
    },
}

impl TransitionPayload {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::CaseOpened { .. } => "case_opened",
            Self::ParticipantBound { .. } => "participant_bound",
            Self::ParticipantAdmitted { .. } => "participant_admitted",
            Self::ProviderAttached { .. } => "provider_attached",
            Self::ProviderInvocationStarted { .. } => "provider_invocation_started",
            Self::ProviderResultRecorded { .. } => "provider_result_recorded",
            Self::InteractionTurnRecorded { .. } => "interaction_turn_recorded",
            Self::ModelInterpretationRecorded { .. } => "model_interpretation_recorded",
            Self::ResourceAttached { .. } => "resource_attached",
            Self::OperationNormalizationFailed { .. } => "operation_normalization_failed",
            Self::OperationRecorded { .. } => "operation_recorded",
            Self::DecisionRecorded { .. } => "decision_recorded",
            Self::ExecutionGrantIssued { .. } => "execution_grant_issued",
            Self::EffectPrepared { .. } => "effect_prepared",
            Self::EffectFinalized { .. } => "effect_finalized",
            Self::EffectIndeterminate { .. } => "effect_indeterminate",
            Self::EffectReconciled { .. } => "effect_reconciled",
            Self::ReviewRequested { .. } => "review_requested",
            Self::ReviewActionRecorded { .. } => "review_action_recorded",
            Self::ReviewInvalidated { .. } => "review_invalidated",
            Self::ExecutionGrantInvalidated { .. } => "execution_grant_invalidated",
            Self::CaseCancellationRequested { .. } => "case_cancellation_requested",
            Self::CaseClosed { .. } => "case_closed",
            Self::CasePolicyBound { .. } => "case_policy_bound",
            Self::CasePolicyReplaced { .. } => "case_policy_replaced",
            Self::CasePolicyUnbound { .. } => "case_policy_unbound",
            Self::ReviewResolved { .. } => "review_resolved",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseLifecycle {
    Open,
    Closed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterpretationAuthority {
    NonAuthoritative,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewResolution {
    Pending,
    PendingOperator,
    Approved,
    Denied,
    Deferred,
    Quarantined,
    Invalidated,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewActionKind {
    Approve,
    Deny,
    Defer,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReviewAction {
    pub schema: String,
    pub action_id: String,
    pub integrity_digest: String,
    pub review_id: String,
    pub operation_id: String,
    pub case_id: String,
    pub reviewer_participant_id: String,
    pub action: ReviewActionKind,
    pub reason: String,
    pub expected_case_generation: u64,
    pub source: String,
}

pub fn build_review_action(
    review: &ReviewState,
    case_id: &str,
    reviewer_participant_id: &str,
    action: ReviewActionKind,
    reason: &str,
    expected_case_generation: u64,
    source: &str,
) -> Result<ReviewAction, String> {
    if review.schema != REVIEW_REQUEST_SCHEMA && review.schema != REVIEW_REQUEST_SCHEMA_V1
        || case_id.is_empty()
        || reviewer_participant_id.is_empty()
        || reason.trim().is_empty()
        || source.is_empty()
    {
        return Err("invalid_review_action_input".to_string());
    }
    let reason = reason.split_whitespace().collect::<Vec<_>>().join(" ");
    let material = review_action_digest_material(
        REVIEW_ACTION_SCHEMA,
        &review.review_id,
        &review.operation_id,
        case_id,
        reviewer_participant_id,
        &action,
        &reason,
        expected_case_generation,
        source,
    );
    let integrity_digest = crate::effect::digest_bytes(material.to_string().as_bytes());
    let result = ReviewAction {
        schema: REVIEW_ACTION_SCHEMA.to_string(),
        action_id: format!("review-action:{}", &integrity_digest[..32]),
        integrity_digest,
        review_id: review.review_id.clone(),
        operation_id: review.operation_id.clone(),
        case_id: case_id.to_string(),
        reviewer_participant_id: reviewer_participant_id.to_string(),
        action,
        reason,
        expected_case_generation,
        source: source.to_string(),
    };
    result.validate_integrity()?;
    Ok(result)
}

fn review_action_digest_material(
    schema: &str,
    review_id: &str,
    operation_id: &str,
    case_id: &str,
    reviewer_participant_id: &str,
    action: &ReviewActionKind,
    reason: &str,
    expected_case_generation: u64,
    source: &str,
) -> serde_json::Value {
    serde_json::json!({
        "schema": schema,
        "review_id": review_id,
        "operation_id": operation_id,
        "case_id": case_id,
        "reviewer_participant_id": reviewer_participant_id,
        "action": action,
        "reason": reason,
        "expected_case_generation": expected_case_generation,
        "source": source,
    })
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CaseState {
    pub schema: String,
    pub case_id: String,
    pub generation: u64,
    pub lifecycle: CaseLifecycle,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancellation: Option<CaseCancellationState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closure: Option<CaseClosureState>,
    #[serde(default)]
    pub participants: Vec<ParticipantState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderAttachmentState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_provider_invocation: Option<ProviderInvocationState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_provider_result: Option<ProviderResultState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_model_interpretation: Option<ModelInterpretationState>,
    #[serde(default)]
    pub reviews: Vec<ReviewState>,
    #[serde(default)]
    pub policy_bindings: Vec<CasePolicyBinding>,
    #[serde(default)]
    pub resources: Vec<ResourceAttachmentState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_normalization_failure: Option<NormalizationFailureState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_operation: Option<OperationState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_decision: Option<DecisionState>,
    #[serde(default)]
    pub grants: Vec<GrantState>,
    #[serde(default)]
    pub effects: Vec<EffectState>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ParticipantState {
    pub participant_id: String,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub admitted_views: Vec<AdmittedView>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdmittedView {
    pub consumer: String,
    pub view_kind: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderInvocationLineage {
    pub projection_id: String,
    pub context_frame_id: String,
    pub case_generation: u64,
    pub rendered_input_id: String,
    pub rendered_input_digest: String,
    pub output_contract_id: String,
    pub continuation_disposition: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderAttachmentState {
    pub participant_id: String,
    #[serde(default)]
    pub provider_id: String,
    pub provider_kind: String,
    pub base_url: String,
    pub model_id: String,
    pub credential_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderInvocationState {
    pub invocation_id: String,
    pub participant_id: String,
    #[serde(default)]
    pub provider_id: String,
    pub provider_kind: String,
    pub model_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_lineage: Option<ProviderInvocationLineage>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderResultState {
    pub result_id: String,
    pub invocation_id: String,
    #[serde(default)]
    pub provider_id: String,
    pub provider_kind: String,
    pub model_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_lineage: Option<ProviderInvocationLineage>,
    pub output_chars: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelInterpretationState {
    pub interpretation_id: String,
    pub result_id: String,
    pub authority: InterpretationAuthority,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReviewState {
    pub review_id: String,
    #[serde(default)]
    pub schema: String,
    #[serde(default)]
    pub integrity_digest: String,
    #[serde(default)]
    pub case_id: String,
    #[serde(default)]
    pub operation_id: String,
    #[serde(default)]
    pub operation_digest: String,
    #[serde(default)]
    pub initial_decision_id: String,
    #[serde(default)]
    pub decision_basis_id: String,
    #[serde(default)]
    pub decision_basis_digest: String,
    #[serde(default)]
    pub effective_policy_id: String,
    #[serde(default)]
    pub effective_policy_digest: String,
    #[serde(default)]
    pub policy_binding_refs: Vec<String>,
    #[serde(default)]
    pub policy_artifact_refs: Vec<String>,
    #[serde(default)]
    pub required_reviewer_roles: Vec<String>,
    #[serde(default)]
    pub resource_attachment_id: String,
    #[serde(default)]
    pub normalized_target: String,
    #[serde(default)]
    pub created_at_generation: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_action_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_decision_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invalidation_reason: Option<AuthorityInvalidationReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invalidation_source_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invalidated_at_unix_ms: Option<u64>,
    /* Compatibility-only fields for yai.transition.v1-v3 and legacy records. */
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub attempt_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub requested_by_participant: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub target_participant: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub reviewer_participant: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub operation_kind: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub carrier_family: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub target_display: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub sandbox_path: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub target_path: String,
    pub policy_reason: String,
    pub status: ReviewResolution,
    #[serde(default)]
    pub carrier_attempted: bool,
    #[serde(default)]
    pub execution_performed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt_ref: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    Filesystem,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewRequirement {
    #[default]
    Automatic,
    RequireReview,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResourceAttachmentState {
    pub attachment_id: String,
    pub kind: ResourceKind,
    pub allowed_write_prefix: String,
    pub max_write_bytes: usize,
    pub policy_id: String,
    pub policy_owner_participant_id: String,
    #[serde(default)]
    pub review_requirement: ReviewRequirement,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NormalizationFailureState {
    pub provider_result_id: String,
    pub failure: NormalizationFailure,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OperationState {
    pub operation_id: String,
    pub operation_digest: String,
    pub participant_id: String,
    pub resource_attachment_id: String,
    pub relative_path: String,
    pub intended_content_digest: String,
    pub origin: OperationOrigin,
    pub recorded_at_generation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DecisionState {
    pub decision_id: String,
    pub decision_digest: String,
    pub operation_id: String,
    pub operation_digest: String,
    pub outcome: DecisionOutcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_basis_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_policy_id: Option<String>,
    pub recorded_at_generation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantLifecycle {
    Issued,
    Prepared,
    Finalized,
    Rejected,
    Expired,
    Revoked,
    Abandoned,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GrantState {
    pub grant_id: String,
    pub integrity_digest: String,
    pub operation_id: String,
    pub decision_id: String,
    pub resource_attachment_id: String,
    pub idempotency_key: String,
    pub status: GrantLifecycle,
    pub issued_at_generation: u64,
    #[serde(default)]
    pub issued_at_unix_ms: u64,
    #[serde(default)]
    pub expires_at_unix_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invalidation_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invalidation_source_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invalidated_at_unix_ms: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectLifecycle {
    Prepared,
    Finalized,
    Indeterminate,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EffectState {
    pub effect_id: String,
    pub operation_id: String,
    pub decision_id: String,
    pub grant_id: String,
    pub resource_attachment_id: String,
    pub relative_path: String,
    pub intended_content_digest: String,
    pub pre_observation_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_observation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<EffectOutcome>,
    pub status: EffectLifecycle,
    pub prepared_at_generation: u64,
    pub updated_at_generation: u64,
}

impl CaseState {
    pub fn new(case_id: impl Into<String>, lifecycle: CaseLifecycle) -> Self {
        Self {
            schema: CASE_STATE_SCHEMA.to_string(),
            case_id: case_id.into(),
            generation: 0,
            lifecycle,
            cancellation: None,
            closure: None,
            participants: Vec::new(),
            provider: None,
            last_provider_invocation: None,
            last_provider_result: None,
            last_model_interpretation: None,
            reviews: Vec::new(),
            policy_bindings: Vec::new(),
            resources: Vec::new(),
            last_normalization_failure: None,
            last_operation: None,
            last_decision: None,
            grants: Vec::new(),
            effects: Vec::new(),
        }
    }

    pub fn reduce(&self, transition: &Transition) -> Result<Self, String> {
        transition.validate()?;
        if self.case_id != transition.case_id {
            return Err("case_state_case_mismatch".to_string());
        }
        if transition.sequence != self.generation + 1 {
            return Err(format!(
                "case_sequence_mismatch: expected={} actual={}",
                self.generation + 1,
                transition.sequence
            ));
        }

        let mut next = self.clone();
        if self.generation > 0 && self.lifecycle == CaseLifecycle::Closed {
            return Err("case_closed_write_barrier".to_string());
        }
        if self.cancellation.is_some()
            && !matches!(
                transition.payload,
                TransitionPayload::ProviderResultRecorded { .. }
                    | TransitionPayload::ReviewInvalidated { .. }
                    | TransitionPayload::ExecutionGrantInvalidated { .. }
                    | TransitionPayload::EffectFinalized { .. }
                    | TransitionPayload::EffectIndeterminate { .. }
                    | TransitionPayload::EffectReconciled { .. }
                    | TransitionPayload::CaseClosed { .. }
            )
        {
            return Err("case_cancelled_write_barrier".to_string());
        }
        match &transition.payload {
            TransitionPayload::CaseOpened { lifecycle } => {
                if transition.sequence != 1 {
                    return Err("case_opened_must_be_first_transition".to_string());
                }
                next.lifecycle = lifecycle.clone();
            }
            TransitionPayload::ParticipantBound {
                participant_id,
                role,
            } => {
                let participant = upsert_participant(&mut next.participants, participant_id);
                push_unique(&mut participant.roles, role);
            }
            TransitionPayload::ParticipantAdmitted {
                participant_id,
                consumer,
                view_kind,
            } => {
                let participant = upsert_participant(&mut next.participants, participant_id);
                let admitted = AdmittedView {
                    consumer: consumer.clone(),
                    view_kind: view_kind.clone(),
                };
                if !participant.admitted_views.contains(&admitted) {
                    participant.admitted_views.push(admitted);
                }
            }
            TransitionPayload::ProviderAttached {
                participant_id,
                provider_id,
                provider_kind,
                base_url,
                model_id,
                credential_ref,
            } => {
                if !next
                    .participants
                    .iter()
                    .any(|participant| participant.participant_id == *participant_id)
                {
                    return Err("provider_participant_not_bound".to_string());
                }
                next.provider = Some(ProviderAttachmentState {
                    participant_id: participant_id.clone(),
                    provider_id: provider_id.clone(),
                    provider_kind: provider_kind.clone(),
                    base_url: base_url.clone(),
                    model_id: model_id.clone(),
                    credential_ref: credential_ref.clone(),
                });
            }
            TransitionPayload::ProviderInvocationStarted {
                invocation_id,
                participant_id,
                provider_id,
                provider_kind,
                model_id,
                semantic_lineage,
            } => {
                let Some(provider) = next.provider.as_ref() else {
                    return Err("provider_not_attached".to_string());
                };
                if provider.participant_id != *participant_id
                    || (!provider.provider_id.is_empty()
                        && !provider_id.is_empty()
                        && provider.provider_id != *provider_id)
                    || provider.provider_kind != *provider_kind
                    || provider.model_id != *model_id
                {
                    return Err("provider_invocation_attachment_mismatch".to_string());
                }
                next.last_provider_invocation = Some(ProviderInvocationState {
                    invocation_id: invocation_id.clone(),
                    participant_id: participant_id.clone(),
                    provider_id: provider_id.clone(),
                    provider_kind: provider_kind.clone(),
                    model_id: model_id.clone(),
                    semantic_lineage: semantic_lineage.clone(),
                });
            }
            TransitionPayload::ProviderResultRecorded {
                result_id,
                invocation_id,
                provider_id,
                provider_kind,
                model_id,
                semantic_lineage,
                output,
            } => {
                let Some(invocation) = next.last_provider_invocation.as_ref() else {
                    return Err("provider_result_without_invocation".to_string());
                };
                if invocation.invocation_id != *invocation_id
                    || (!invocation.provider_id.is_empty()
                        && !provider_id.is_empty()
                        && invocation.provider_id != *provider_id)
                    || invocation.provider_kind != *provider_kind
                    || invocation.model_id != *model_id
                    || !provider_lineage_matches(
                        invocation.semantic_lineage.as_ref(),
                        semantic_lineage.as_ref(),
                    )
                {
                    return Err("provider_result_invocation_mismatch".to_string());
                }
                next.last_provider_result = Some(ProviderResultState {
                    result_id: result_id.clone(),
                    invocation_id: invocation_id.clone(),
                    provider_id: provider_id.clone(),
                    provider_kind: provider_kind.clone(),
                    model_id: model_id.clone(),
                    semantic_lineage: semantic_lineage.clone(),
                    output_chars: output.chars().count(),
                });
            }
            TransitionPayload::InteractionTurnRecorded { .. } => {}
            TransitionPayload::ModelInterpretationRecorded {
                interpretation_id,
                result_id,
                authority,
            } => {
                let Some(result) = next.last_provider_result.as_ref() else {
                    return Err("interpretation_without_provider_result".to_string());
                };
                if result.result_id != *result_id {
                    return Err("interpretation_result_mismatch".to_string());
                }
                next.last_model_interpretation = Some(ModelInterpretationState {
                    interpretation_id: interpretation_id.clone(),
                    result_id: result_id.clone(),
                    authority: authority.clone(),
                });
            }
            TransitionPayload::ResourceAttached { attachment } => {
                attachment.validate()?;
                if !next.participants.iter().any(|participant| {
                    participant.participant_id == attachment.policy_owner_participant_id
                }) {
                    return Err("resource_policy_owner_not_bound".to_string());
                }
                if next
                    .resources
                    .iter()
                    .any(|resource| resource.attachment_id == attachment.attachment_id)
                {
                    return Err("resource_attachment_already_exists".to_string());
                }
                next.resources.push(attachment.clone());
            }
            TransitionPayload::OperationNormalizationFailed {
                provider_result_id,
                failure,
            } => {
                let Some(result) = next.last_provider_result.as_ref() else {
                    return Err("normalization_failure_without_provider_result".to_string());
                };
                if result.result_id != *provider_result_id {
                    return Err("normalization_failure_result_mismatch".to_string());
                }
                next.last_normalization_failure = Some(NormalizationFailureState {
                    provider_result_id: provider_result_id.clone(),
                    failure: failure.clone(),
                });
            }
            TransitionPayload::OperationRecorded { operation } => {
                operation.validate()?;
                if operation.case_id != next.case_id
                    || operation.expected_case_generation != next.generation
                {
                    return Err("operation_case_or_generation_mismatch".to_string());
                }
                if !next
                    .participants
                    .iter()
                    .any(|participant| participant.participant_id == operation.participant_id)
                {
                    return Err("operation_participant_not_bound".to_string());
                }
                if !next
                    .resources
                    .iter()
                    .any(|resource| resource.attachment_id == operation.resource_attachment_id)
                {
                    return Err("operation_resource_not_attached".to_string());
                }
                match &operation.origin {
                    OperationOrigin::ProviderResult {
                        provider_result_id,
                        provider_invocation_id,
                    } => {
                        let Some(result) = next.last_provider_result.as_ref() else {
                            return Err("operation_without_provider_result".to_string());
                        };
                        if result.result_id != *provider_result_id
                            || result.invocation_id != *provider_invocation_id
                        {
                            return Err("operation_provider_lineage_mismatch".to_string());
                        }
                    }
                    OperationOrigin::CompatibilityReview {
                        review_id,
                        attempt_id,
                    } => {
                        if !next.reviews.iter().any(|review| {
                            review.review_id == *review_id
                                && review.attempt_id == *attempt_id
                                && review.status == ReviewResolution::PendingOperator
                        }) {
                            return Err("operation_review_lineage_mismatch".to_string());
                        }
                    }
                }
                next.last_operation = Some(OperationState {
                    operation_id: operation.operation_id.clone(),
                    operation_digest: operation.operation_digest.clone(),
                    participant_id: operation.participant_id.clone(),
                    resource_attachment_id: operation.resource_attachment_id.clone(),
                    relative_path: operation.filesystem_write.relative_path.clone(),
                    intended_content_digest: operation.filesystem_write.content_digest.clone(),
                    origin: operation.origin.clone(),
                    recorded_at_generation: transition.sequence,
                });
                next.last_normalization_failure = None;
            }
            TransitionPayload::DecisionRecorded { decision } => {
                decision.validate_integrity()?;
                let Some(operation) = next.last_operation.as_ref() else {
                    return Err("decision_without_operation".to_string());
                };
                if decision.operation_id != operation.operation_id
                    || decision.operation_digest != operation.operation_digest
                    || decision.decided_at_case_generation != next.generation
                {
                    return Err("decision_operation_or_generation_mismatch".to_string());
                }
                let resource = next
                    .resources
                    .iter()
                    .find(|resource| resource.attachment_id == operation.resource_attachment_id)
                    .ok_or_else(|| "decision_resource_not_attached".to_string())?;
                let (policy_id, decision_basis_id, effective_policy_id) =
                    if decision.schema == crate::effect::DECISION_SCHEMA_V1 {
                        let source = decision
                            .source
                            .as_ref()
                            .ok_or_else(|| "legacy_decision_source_missing".to_string())?;
                        if source.policy_id != resource.policy_id
                            || source.owner_participant_id != resource.policy_owner_participant_id
                            || source.owner_participant_id == operation.participant_id
                        {
                            return Err("decision_source_not_attachment_policy".to_string());
                        }
                        (Some(source.policy_id.clone()), None, None)
                    } else {
                        let basis = decision
                            .decision_basis
                            .as_ref()
                            .ok_or_else(|| "policy_decision_basis_missing".to_string())?;
                        if basis.case_id != next.case_id
                            || basis.operation_id != operation.operation_id
                            || basis.operation_digest != operation.operation_digest
                            || basis.resource_attachment_id != resource.attachment_id
                            || basis.proposer_participant_id != operation.participant_id
                            || basis.policy_binding_refs
                                != next
                                    .policy_bindings
                                    .iter()
                                    .map(|binding| binding.binding_id.clone())
                                    .collect::<Vec<_>>()
                        {
                            return Err("policy_decision_case_basis_mismatch".to_string());
                        }
                        (
                            None,
                            Some(basis.basis_id.clone()),
                            Some(basis.effective_policy_id.clone()),
                        )
                    };
                next.last_decision = Some(DecisionState {
                    decision_id: decision.decision_id.clone(),
                    decision_digest: decision.decision_digest.clone(),
                    operation_id: decision.operation_id.clone(),
                    operation_digest: decision.operation_digest.clone(),
                    outcome: decision.outcome.clone(),
                    policy_id,
                    decision_basis_id,
                    effective_policy_id,
                    recorded_at_generation: transition.sequence,
                });
                if decision.outcome != DecisionOutcome::RequireReview {
                    if let Some(review) = next.reviews.iter_mut().find(|review| {
                        review.operation_id == decision.operation_id
                            && (decision
                                .basis_refs
                                .iter()
                                .any(|basis| basis == &review.review_id)
                                || decision
                                    .decision_basis
                                    .as_ref()
                                    .and_then(|basis| basis.review_action_ref.as_ref())
                                    == review.latest_action_id.as_ref())
                    }) {
                        let Some(action_id) = review.latest_action_id.as_ref() else {
                            return Err("effective_review_decision_without_action".to_string());
                        };
                        if !decision.basis_refs.iter().any(|basis| basis == action_id)
                            && decision
                                .decision_basis
                                .as_ref()
                                .and_then(|basis| basis.review_action_ref.as_ref())
                                != Some(action_id)
                        {
                            return Err("effective_review_decision_action_mismatch".to_string());
                        }
                        review.effective_decision_id = Some(decision.decision_id.clone());
                    }
                }
            }
            TransitionPayload::ExecutionGrantIssued { grant } => {
                grant.validate_integrity()?;
                let Some(operation) = next.last_operation.as_ref() else {
                    return Err("grant_without_operation".to_string());
                };
                let Some(decision) = next.last_decision.as_ref() else {
                    return Err("grant_without_decision".to_string());
                };
                if decision.outcome != DecisionOutcome::Allow
                    || grant.operation_id != operation.operation_id
                    || grant.operation_digest != operation.operation_digest
                    || grant.decision_id != decision.decision_id
                    || grant.decision_digest != decision.decision_digest
                    || grant.case_id != next.case_id
                    || grant.participant_id != operation.participant_id
                    || grant.resource_attachment_id != operation.resource_attachment_id
                    || grant.expected_case_generation != next.generation
                    || next
                        .grants
                        .iter()
                        .any(|existing| existing.grant_id == grant.grant_id)
                {
                    return Err("grant_chain_or_generation_mismatch".to_string());
                }
                if grant.schema == crate::effect::EXECUTION_GRANT_SCHEMA
                    && (grant.decision_basis_id.as_deref() != decision.decision_basis_id.as_deref()
                        || grant.effective_policy_id.as_deref()
                            != decision.effective_policy_id.as_deref()
                        || grant.policy_binding_refs
                            != next
                                .policy_bindings
                                .iter()
                                .map(|binding| binding.binding_id.clone())
                                .collect::<Vec<_>>())
                {
                    return Err("policy_grant_case_basis_mismatch".to_string());
                }
                next.grants.push(GrantState {
                    grant_id: grant.grant_id.clone(),
                    integrity_digest: grant.integrity_digest.clone(),
                    operation_id: grant.operation_id.clone(),
                    decision_id: grant.decision_id.clone(),
                    resource_attachment_id: grant.resource_attachment_id.clone(),
                    idempotency_key: grant.idempotency_key.clone(),
                    status: GrantLifecycle::Issued,
                    issued_at_generation: transition.sequence,
                    issued_at_unix_ms: grant.issued_at_unix_ms,
                    expires_at_unix_ms: grant.expires_at_unix_ms,
                    invalidation_reason: None,
                    invalidation_source_ref: None,
                    invalidated_at_unix_ms: None,
                });
            }
            TransitionPayload::EffectPrepared { prepared } => {
                prepared.validate()?;
                let Some(operation) = next.last_operation.as_ref() else {
                    return Err("prepare_without_operation".to_string());
                };
                let Some(decision) = next.last_decision.as_ref() else {
                    return Err("prepare_without_decision".to_string());
                };
                let grant_index = next
                    .grants
                    .iter()
                    .position(|grant| grant.grant_id == prepared.grant_id)
                    .ok_or_else(|| "prepare_without_grant".to_string())?;
                let grant = &next.grants[grant_index];
                if grant.status != GrantLifecycle::Issued
                    || grant.issued_at_generation != next.generation
                    || prepared.operation_id != operation.operation_id
                    || prepared.decision_id != decision.decision_id
                    || prepared.resource_attachment_id != operation.resource_attachment_id
                    || prepared.relative_path != operation.relative_path
                    || prepared.intended_content_digest != operation.intended_content_digest
                    || prepared.idempotency_key != grant.idempotency_key
                    || next.effects.iter().any(|effect| {
                        effect.effect_id == prepared.effect_id
                            || effect.grant_id == prepared.grant_id
                            || effect.operation_id == prepared.operation_id
                            || effect.pre_observation_id
                                == prepared.expected_pre_observation.observation_id
                    })
                {
                    return Err("prepare_chain_or_one_time_grant_mismatch".to_string());
                }
                next.grants[grant_index].status = GrantLifecycle::Prepared;
                next.effects.push(EffectState {
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
                    prepared_at_generation: transition.sequence,
                    updated_at_generation: transition.sequence,
                });
            }
            TransitionPayload::EffectFinalized {
                effect_id,
                post_observation,
                receipt,
            } => {
                validate_finalization(effect_id, post_observation, receipt)?;
                let effect_index = next
                    .effects
                    .iter()
                    .position(|effect| effect.effect_id == *effect_id)
                    .ok_or_else(|| "finalize_without_prepared_effect".to_string())?;
                let effect = &next.effects[effect_index];
                if !matches!(
                    effect.status,
                    EffectLifecycle::Prepared | EffectLifecycle::Indeterminate
                ) || receipt.operation_id != effect.operation_id
                    || receipt.decision_id != effect.decision_id
                    || receipt.grant_id != effect.grant_id
                    || receipt.resource_attachment_id != effect.resource_attachment_id
                    || receipt.relative_path != effect.relative_path
                    || receipt.pre_observation_id != effect.pre_observation_id
                    || receipt.post_observation_id != post_observation.observation_id
                {
                    return Err("finalize_chain_mismatch".to_string());
                }
                apply_finalized_effect(
                    &mut next,
                    effect_index,
                    post_observation,
                    receipt,
                    transition.sequence,
                )?;
            }
            TransitionPayload::EffectIndeterminate {
                effect_id,
                reason,
                observation,
            } => {
                require_value("effect_indeterminate.reason", reason)?;
                let effect = next
                    .effects
                    .iter_mut()
                    .find(|effect| effect.effect_id == *effect_id)
                    .ok_or_else(|| "indeterminate_without_prepared_effect".to_string())?;
                if effect.status == EffectLifecycle::Finalized {
                    return Err("finalized_effect_cannot_become_indeterminate".to_string());
                }
                if let Some(observation) = observation {
                    observation.validate()?;
                    if observation.resource_attachment_id != effect.resource_attachment_id
                        || observation.relative_path != effect.relative_path
                    {
                        return Err("indeterminate_observation_target_mismatch".to_string());
                    }
                    effect.post_observation_id = Some(observation.observation_id.clone());
                }
                effect.status = EffectLifecycle::Indeterminate;
                effect.outcome = Some(EffectOutcome::Indeterminate);
                effect.updated_at_generation = transition.sequence;
            }
            TransitionPayload::EffectReconciled {
                effect_id,
                conclusion,
                observation,
                receipt,
            } => {
                observation.validate()?;
                let effect_index = next
                    .effects
                    .iter()
                    .position(|effect| effect.effect_id == *effect_id)
                    .ok_or_else(|| "reconcile_without_prepared_effect".to_string())?;
                let effect = &next.effects[effect_index];
                if effect.status == EffectLifecycle::Finalized
                    || observation.resource_attachment_id != effect.resource_attachment_id
                    || observation.relative_path != effect.relative_path
                {
                    return Err("reconciliation_target_or_state_mismatch".to_string());
                }
                match conclusion {
                    ReconciliationConclusion::EffectObserved
                    | ReconciliationConclusion::NoEffectObserved => {
                        let receipt = receipt.as_ref().ok_or_else(|| {
                            "conclusive_reconciliation_requires_receipt".to_string()
                        })?;
                        validate_finalization(effect_id, observation, receipt)?;
                        apply_finalized_effect(
                            &mut next,
                            effect_index,
                            observation,
                            receipt,
                            transition.sequence,
                        )?;
                    }
                    ReconciliationConclusion::Conflict
                    | ReconciliationConclusion::StillIndeterminate => {
                        if receipt.is_some() {
                            return Err(
                                "indeterminate_reconciliation_cannot_have_receipt".to_string()
                            );
                        }
                        let effect = &mut next.effects[effect_index];
                        effect.status = EffectLifecycle::Indeterminate;
                        effect.outcome =
                            Some(if *conclusion == ReconciliationConclusion::Conflict {
                                EffectOutcome::Conflict
                            } else {
                                EffectOutcome::Indeterminate
                            });
                        effect.post_observation_id = Some(observation.observation_id.clone());
                        effect.updated_at_generation = transition.sequence;
                    }
                }
            }
            TransitionPayload::ReviewRequested { review } => {
                if next
                    .reviews
                    .iter()
                    .any(|existing| existing.review_id == review.review_id)
                {
                    return Err("review_already_exists".to_string());
                }
                if supports_wave7_contract(&transition.schema) {
                    let operation = next
                        .last_operation
                        .as_ref()
                        .ok_or_else(|| "review_without_operation".to_string())?;
                    let decision = next
                        .last_decision
                        .as_ref()
                        .ok_or_else(|| "review_without_initial_decision".to_string())?;
                    let resource = next
                        .resources
                        .iter()
                        .find(|resource| resource.attachment_id == review.resource_attachment_id)
                        .ok_or_else(|| "review_resource_not_attached".to_string())?;
                    let policy_v2 = review.schema == REVIEW_REQUEST_SCHEMA;
                    if policy_v2 && review.case_id != next.case_id
                        || review.operation_id != operation.operation_id
                        || review.operation_digest != operation.operation_digest
                        || review.initial_decision_id != decision.decision_id
                        || decision.operation_id != operation.operation_id
                        || decision.outcome != DecisionOutcome::RequireReview
                        || review.requested_by_participant != operation.participant_id
                        || review.normalized_target != operation.relative_path
                        || review.created_at_generation != next.generation
                    {
                        return Err("review_request_chain_mismatch".to_string());
                    }
                    if policy_v2 {
                        if review.decision_basis_id.is_empty()
                            || review.decision_basis_digest.is_empty()
                            || review.effective_policy_id.is_empty()
                            || review.effective_policy_digest.is_empty()
                            || review.required_reviewer_roles.is_empty()
                            || decision.decision_basis_id.as_deref()
                                != Some(review.decision_basis_id.as_str())
                            || decision.effective_policy_id.as_deref()
                                != Some(review.effective_policy_id.as_str())
                            || !next.participants.iter().any(|participant| {
                                review
                                    .required_reviewer_roles
                                    .iter()
                                    .all(|role| participant.roles.contains(role))
                            })
                        {
                            return Err("policy_review_request_basis_mismatch".to_string());
                        }
                    } else if review.reviewer_participant != resource.policy_owner_participant_id
                        || !next.participants.iter().any(|participant| {
                            participant.participant_id == review.reviewer_participant
                        })
                    {
                        return Err("review_request_chain_mismatch".to_string());
                    }
                }
                next.reviews.push(review.clone());
            }
            TransitionPayload::ReviewActionRecorded { action } => {
                action.validate_integrity()?;
                let Some(review) = next
                    .reviews
                    .iter_mut()
                    .find(|review| review.review_id == action.review_id)
                else {
                    return Err("review_not_found".to_string());
                };
                if review.operation_id != action.operation_id
                    || action.case_id != next.case_id
                    || action.expected_case_generation != next.generation
                    || !next.participants.iter().any(|participant| {
                        participant.participant_id == action.reviewer_participant_id
                            && if review.schema == REVIEW_REQUEST_SCHEMA {
                                review
                                    .required_reviewer_roles
                                    .iter()
                                    .all(|role| participant.roles.contains(role))
                            } else {
                                review.reviewer_participant == action.reviewer_participant_id
                            }
                    })
                {
                    return Err("review_action_binding_or_generation_mismatch".to_string());
                }
                if !matches!(
                    review.status,
                    ReviewResolution::Pending | ReviewResolution::Deferred
                ) {
                    return Err("review_already_resolved".to_string());
                }
                review.status = match action.action {
                    ReviewActionKind::Approve => ReviewResolution::Approved,
                    ReviewActionKind::Deny => ReviewResolution::Denied,
                    ReviewActionKind::Defer => ReviewResolution::Deferred,
                };
                review.latest_action_id = Some(action.action_id.clone());
            }
            TransitionPayload::ReviewInvalidated { invalidation } => {
                let review = next
                    .reviews
                    .iter_mut()
                    .find(|review| review.review_id == invalidation.review_id)
                    .ok_or_else(|| "review_not_found".to_string())?;
                if !matches!(
                    review.status,
                    ReviewResolution::Pending | ReviewResolution::Deferred
                ) {
                    return Err("review_invalidation_requires_usable_review".to_string());
                }
                review.status = ReviewResolution::Invalidated;
                review.invalidation_reason = Some(invalidation.reason.clone());
                review.invalidation_source_ref = Some(invalidation.source_ref.clone());
                review.invalidated_at_unix_ms = Some(invalidation.invalidated_at_unix_ms);
            }
            TransitionPayload::ExecutionGrantInvalidated { invalidation } => {
                let grant = next
                    .grants
                    .iter_mut()
                    .find(|grant| grant.grant_id == invalidation.grant_id)
                    .ok_or_else(|| "execution_grant_not_found".to_string())?;
                if grant.status != GrantLifecycle::Issued {
                    return Err("execution_grant_invalidation_requires_issued".to_string());
                }
                grant.status = match invalidation.disposition {
                    GrantInvalidationDisposition::Expired => GrantLifecycle::Expired,
                    GrantInvalidationDisposition::Revoked => GrantLifecycle::Revoked,
                    GrantInvalidationDisposition::Abandoned => GrantLifecycle::Abandoned,
                };
                grant.invalidation_reason = Some(invalidation.reason.clone());
                grant.invalidation_source_ref = Some(invalidation.source_ref.clone());
                grant.invalidated_at_unix_ms = Some(invalidation.invalidated_at_unix_ms);
            }
            TransitionPayload::CaseCancellationRequested { cancellation } => {
                if next.cancellation.is_some() {
                    return Err("case_already_cancelled".to_string());
                }
                if cancellation.transition_id != transition.transition_id {
                    return Err("case_cancellation_transition_identity_mismatch".to_string());
                }
                next.cancellation = Some(cancellation.clone());
            }
            TransitionPayload::CaseClosed { closure } => {
                let cancellation = next
                    .cancellation
                    .as_ref()
                    .ok_or_else(|| "case_close_requires_cancellation".to_string())?;
                if closure.cancellation_ref != cancellation.transition_id {
                    return Err("case_closure_cancellation_ref_mismatch".to_string());
                }
                if closure.transition_id != transition.transition_id {
                    return Err("case_closure_transition_identity_mismatch".to_string());
                }
                if next.reviews.iter().any(|review| {
                    matches!(
                        review.status,
                        ReviewResolution::Pending | ReviewResolution::Deferred
                    )
                }) || next
                    .grants
                    .iter()
                    .any(|grant| grant.status == GrantLifecycle::Issued)
                    || next.effects.iter().any(|effect| {
                        matches!(
                            effect.status,
                            EffectLifecycle::Prepared | EffectLifecycle::Indeterminate
                        )
                    })
                {
                    return Err("case_close_has_unresolved_authority_or_effect".to_string());
                }
                next.lifecycle = CaseLifecycle::Closed;
                next.closure = Some(closure.clone());
            }
            TransitionPayload::CasePolicyBound { binding } => {
                binding.validate_integrity()?;
                if binding.case_id != next.case_id
                    || binding.bound_at_case_generation != transition.sequence
                    || next
                        .policy_bindings
                        .iter()
                        .any(|current| current.lineage_id == binding.lineage_id)
                {
                    return Err("case_policy_bind_state_mismatch".to_string());
                }
                next.policy_bindings.push(binding.clone());
                next.policy_bindings
                    .sort_by(|left, right| left.lineage_id.cmp(&right.lineage_id));
            }
            TransitionPayload::CasePolicyReplaced {
                prior_binding_id,
                binding,
            } => {
                binding.validate_integrity()?;
                let Some(index) = next
                    .policy_bindings
                    .iter()
                    .position(|current| current.binding_id == *prior_binding_id)
                else {
                    return Err("case_policy_replace_prior_binding_not_found".to_string());
                };
                let prior = &next.policy_bindings[index];
                if binding.case_id != next.case_id
                    || binding.lineage_id != prior.lineage_id
                    || binding.replaces_binding_id.as_deref() != Some(prior_binding_id)
                    || binding.bound_at_case_generation != transition.sequence
                {
                    return Err("case_policy_replace_state_mismatch".to_string());
                }
                next.policy_bindings[index] = binding.clone();
                next.policy_bindings
                    .sort_by(|left, right| left.lineage_id.cmp(&right.lineage_id));
            }
            TransitionPayload::CasePolicyUnbound {
                binding_id,
                lineage_id,
                ..
            } => {
                let Some(index) = next.policy_bindings.iter().position(|current| {
                    current.binding_id == *binding_id && current.lineage_id == *lineage_id
                }) else {
                    return Err("case_policy_unbind_current_binding_not_found".to_string());
                };
                next.policy_bindings.remove(index);
            }
            TransitionPayload::ReviewResolved {
                review_id,
                attempt_id,
                resolution,
                decision_ref,
                receipt_ref,
                carrier_attempted,
                execution_performed,
                ..
            } => {
                if supports_wave7_contract(&transition.schema) {
                    return Err("legacy_review_resolution_not_writable_in_v4_or_later".to_string());
                }
                let Some(review) = next
                    .reviews
                    .iter_mut()
                    .find(|review| review.review_id == *review_id)
                else {
                    return Err("review_not_found".to_string());
                };
                if review.attempt_id != *attempt_id {
                    return Err("review_attempt_mismatch".to_string());
                }
                review.status = resolution.clone();
                review.decision_ref = Some(decision_ref.clone());
                review.receipt_ref = Some(receipt_ref.clone());
                review.carrier_attempted = *carrier_attempted;
                review.execution_performed = *execution_performed;
            }
        }
        next.generation = transition.sequence;
        Ok(next)
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|error| format!("case_state_encode_failed: {error}"))
    }

    pub fn from_json(value: &str) -> Result<Self, String> {
        let mut state: Self = serde_json::from_str(value)
            .map_err(|error| format!("case_state_decode_failed: {error}"))?;
        if state.schema == CASE_STATE_SCHEMA_V1
            || state.schema == CASE_STATE_SCHEMA_V2
            || state.schema == CASE_STATE_SCHEMA_V3
            || state.schema == CASE_STATE_SCHEMA_V4
            || state.schema == CASE_STATE_SCHEMA_V5
            || state.schema == CASE_STATE_SCHEMA_V6
        {
            state.schema = CASE_STATE_SCHEMA.to_string();
        } else if state.schema != CASE_STATE_SCHEMA {
            return Err(format!("unsupported_case_state_schema: {}", state.schema));
        }
        Ok(state)
    }
}

impl Transition {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != TRANSITION_SCHEMA
            && self.schema != TRANSITION_SCHEMA_V6
            && self.schema != TRANSITION_SCHEMA_V5
            && self.schema != TRANSITION_SCHEMA_V4
            && self.schema != TRANSITION_SCHEMA_V3
            && self.schema != TRANSITION_SCHEMA_V2
            && self.schema != TRANSITION_SCHEMA_V1
        {
            return Err(format!("unsupported_transition_schema: {}", self.schema));
        }
        if self.schema == TRANSITION_SCHEMA_V1 && self.payload.is_wave3_kind() {
            return Err("wave3_transition_kind_requires_yai_transition_v2".to_string());
        }
        if matches!(
            self.schema.as_str(),
            TRANSITION_SCHEMA_V1 | TRANSITION_SCHEMA_V2
        ) && self.payload.is_wave4_kind()
        {
            return Err("wave4_transition_kind_requires_yai_transition_v3".to_string());
        }
        if !supports_wave7_contract(&self.schema) && self.payload.is_wave7_kind() {
            return Err("wave7_transition_kind_requires_yai_transition_v4".to_string());
        }
        if !supports_wave9_contract(&self.schema) && self.payload.is_wave9_kind() {
            return Err("wave9_transition_kind_requires_yai_transition_v5".to_string());
        }
        if !supports_wave10_contract(&self.schema) && self.payload.is_wave10_kind() {
            return Err("wave10_contract_requires_yai_transition_v6".to_string());
        }
        if self.schema != TRANSITION_SCHEMA && self.payload.is_wave11_kind() {
            return Err("wave11_contract_requires_yai_transition_v7".to_string());
        }
        require_value("transition_id", &self.transition_id)?;
        require_value("case_id", &self.case_id)?;
        require_value("source.component", &self.source.component)?;
        if self.sequence == 0 {
            return Err("transition_sequence_must_be_positive".to_string());
        }
        if let Some(scope) = &self.scope {
            if scope.case_id != self.case_id {
                return Err("transition_scope_case_mismatch".to_string());
            }
        }
        for causal_ref in &self.causal_refs {
            require_value("causal_ref", causal_ref)?;
        }
        match &self.payload {
            TransitionPayload::CaseOpened { .. } => {
                if self.sequence != 1 {
                    return Err("case_opened_must_be_first_transition".to_string());
                }
            }
            TransitionPayload::ParticipantBound {
                participant_id,
                role,
            } => {
                require_value("participant_id", participant_id)?;
                require_value("role", role)?;
            }
            TransitionPayload::ParticipantAdmitted {
                participant_id,
                consumer,
                view_kind,
            } => {
                require_value("participant_id", participant_id)?;
                require_value("consumer", consumer)?;
                require_value("view_kind", view_kind)?;
            }
            TransitionPayload::ProviderAttached {
                participant_id,
                provider_id,
                provider_kind,
                base_url,
                model_id,
                credential_ref,
            } => {
                require_value("participant_id", participant_id)?;
                if supports_wave7_contract(&self.schema) {
                    require_value("provider_id", provider_id)?;
                }
                require_value("provider_kind", provider_kind)?;
                require_value("base_url", base_url)?;
                require_value("model_id", model_id)?;
                require_value("credential_ref", credential_ref)?;
            }
            TransitionPayload::ProviderInvocationStarted {
                invocation_id,
                participant_id,
                provider_id,
                provider_kind,
                model_id,
                semantic_lineage,
            } => {
                require_value("invocation_id", invocation_id)?;
                require_value("participant_id", participant_id)?;
                if supports_wave7_contract(&self.schema) {
                    require_value("provider_id", provider_id)?;
                    validate_provider_lineage(semantic_lineage.as_ref())?;
                }
                require_value("provider_kind", provider_kind)?;
                require_value("model_id", model_id)?;
            }
            TransitionPayload::ProviderResultRecorded {
                result_id,
                invocation_id,
                provider_id,
                provider_kind,
                model_id,
                semantic_lineage,
                ..
            } => {
                require_value("result_id", result_id)?;
                require_value("invocation_id", invocation_id)?;
                if supports_wave7_contract(&self.schema) {
                    require_value("provider_id", provider_id)?;
                    validate_provider_lineage(semantic_lineage.as_ref())?;
                }
                require_value("provider_kind", provider_kind)?;
                require_value("model_id", model_id)?;
                require_causal_ref(&self.causal_refs, invocation_id, "provider_invocation")?;
            }
            TransitionPayload::InteractionTurnRecorded {
                turn_id,
                thread_id,
                participant_id,
                invocation_id,
                result_id,
                ..
            } => {
                require_value("turn_id", turn_id)?;
                require_value("thread_id", thread_id)?;
                require_value("participant_id", participant_id)?;
                require_value("invocation_id", invocation_id)?;
                require_value("result_id", result_id)?;
                require_causal_ref(&self.causal_refs, invocation_id, "provider_invocation")?;
                require_causal_ref(&self.causal_refs, result_id, "provider_result")?;
            }
            TransitionPayload::ModelInterpretationRecorded {
                interpretation_id,
                result_id,
                ..
            } => {
                require_value("interpretation_id", interpretation_id)?;
                require_value("result_id", result_id)?;
                require_causal_ref(&self.causal_refs, result_id, "provider_result")?;
            }
            TransitionPayload::ResourceAttached { attachment } => {
                attachment.validate()?;
                require_causal_ref(
                    &self.causal_refs,
                    &attachment.policy_owner_participant_id,
                    "resource_policy_owner",
                )?;
            }
            TransitionPayload::OperationNormalizationFailed {
                provider_result_id,
                failure,
            } => {
                require_value("provider_result_id", provider_result_id)?;
                require_value("normalization_failure.detail", &failure.detail)?;
                require_causal_ref(&self.causal_refs, provider_result_id, "provider_result")?;
            }
            TransitionPayload::OperationRecorded { operation } => {
                operation.validate()?;
                if self.scope.as_ref() != Some(&operation.scope) {
                    return Err("operation_transition_scope_mismatch".to_string());
                }
                for origin_ref in operation.origin.causal_refs() {
                    require_causal_ref(&self.causal_refs, &origin_ref, "operation_origin")?;
                }
            }
            TransitionPayload::DecisionRecorded { decision } => {
                decision.validate_integrity()?;
                require_causal_ref(&self.causal_refs, &decision.operation_id, "operation")?;
                if decision.schema == crate::effect::DECISION_SCHEMA {
                    let basis = decision
                        .decision_basis
                        .as_ref()
                        .ok_or_else(|| "policy_decision_basis_missing".to_string())?;
                    require_causal_ref(&self.causal_refs, &basis.basis_id, "decision_basis")?;
                    require_causal_ref(
                        &self.causal_refs,
                        &basis.effective_policy_id,
                        "effective_policy",
                    )?;
                }
            }
            TransitionPayload::ExecutionGrantIssued { grant } => {
                grant.validate_integrity()?;
                require_causal_ref(&self.causal_refs, &grant.operation_id, "operation")?;
                require_causal_ref(&self.causal_refs, &grant.decision_id, "decision")?;
                if grant.schema == crate::effect::EXECUTION_GRANT_SCHEMA {
                    require_causal_ref(
                        &self.causal_refs,
                        grant.decision_basis_id.as_deref().unwrap_or_default(),
                        "decision_basis",
                    )?;
                    require_causal_ref(
                        &self.causal_refs,
                        grant.effective_policy_id.as_deref().unwrap_or_default(),
                        "effective_policy",
                    )?;
                }
            }
            TransitionPayload::EffectPrepared { prepared } => {
                prepared.validate()?;
                require_causal_ref(&self.causal_refs, &prepared.operation_id, "operation")?;
                require_causal_ref(&self.causal_refs, &prepared.decision_id, "decision")?;
                require_causal_ref(&self.causal_refs, &prepared.grant_id, "execution_grant")?;
                require_causal_ref(
                    &self.causal_refs,
                    &prepared.expected_pre_observation.observation_id,
                    "pre_observation",
                )?;
            }
            TransitionPayload::EffectFinalized {
                effect_id,
                post_observation,
                receipt,
            } => {
                validate_finalization(effect_id, post_observation, receipt)?;
                require_causal_ref(&self.causal_refs, effect_id, "prepared_effect")?;
                require_causal_ref(&self.causal_refs, &receipt.receipt_id, "effect_receipt")?;
            }
            TransitionPayload::EffectIndeterminate {
                effect_id,
                reason,
                observation,
            } => {
                require_value("effect_id", effect_id)?;
                require_value("indeterminate_reason", reason)?;
                if let Some(observation) = observation {
                    observation.validate()?;
                }
                require_causal_ref(&self.causal_refs, effect_id, "prepared_effect")?;
            }
            TransitionPayload::EffectReconciled {
                effect_id,
                conclusion,
                observation,
                receipt,
            } => {
                require_value("effect_id", effect_id)?;
                observation.validate()?;
                if matches!(
                    conclusion,
                    ReconciliationConclusion::EffectObserved
                        | ReconciliationConclusion::NoEffectObserved
                ) {
                    let receipt = receipt
                        .as_ref()
                        .ok_or_else(|| "conclusive_reconciliation_requires_receipt".to_string())?;
                    validate_finalization(effect_id, observation, receipt)?;
                } else if receipt.is_some() {
                    return Err("indeterminate_reconciliation_cannot_have_receipt".to_string());
                }
                require_causal_ref(&self.causal_refs, effect_id, "prepared_effect")?;
            }
            TransitionPayload::ReviewRequested { review } => {
                review.validate_for_schema(&self.schema)?;
                if supports_wave7_contract(&self.schema) {
                    require_causal_ref(
                        &self.causal_refs,
                        &review.operation_id,
                        "review_operation",
                    )?;
                    require_causal_ref(
                        &self.causal_refs,
                        &review.initial_decision_id,
                        "review_initial_decision",
                    )?;
                    if review.schema == REVIEW_REQUEST_SCHEMA {
                        require_causal_ref(
                            &self.causal_refs,
                            &review.decision_basis_id,
                            "decision_basis",
                        )?;
                        require_causal_ref(
                            &self.causal_refs,
                            &review.effective_policy_id,
                            "effective_policy",
                        )?;
                    }
                } else {
                    require_causal_ref(&self.causal_refs, &review.attempt_id, "review_attempt")?;
                }
            }
            TransitionPayload::ReviewActionRecorded { action } => {
                action.validate_integrity()?;
                require_causal_ref(&self.causal_refs, &action.review_id, "review_request")?;
                require_causal_ref(&self.causal_refs, &action.operation_id, "review_operation")?;
            }
            TransitionPayload::ReviewInvalidated { invalidation } => {
                require_value("review_invalidation.review_id", &invalidation.review_id)?;
                require_value("review_invalidation.source_ref", &invalidation.source_ref)?;
                require_causal_ref(&self.causal_refs, &invalidation.review_id, "review_request")?;
                require_causal_ref(
                    &self.causal_refs,
                    &invalidation.source_ref,
                    "invalidation_source",
                )?;
            }
            TransitionPayload::ExecutionGrantInvalidated { invalidation } => {
                require_value("grant_invalidation.grant_id", &invalidation.grant_id)?;
                require_value("grant_invalidation.reason", &invalidation.reason)?;
                require_value("grant_invalidation.source_ref", &invalidation.source_ref)?;
                require_causal_ref(&self.causal_refs, &invalidation.grant_id, "execution_grant")?;
                require_causal_ref(
                    &self.causal_refs,
                    &invalidation.source_ref,
                    "invalidation_source",
                )?;
            }
            TransitionPayload::CaseCancellationRequested { cancellation } => {
                require_value("case_cancellation.actor_ref", &cancellation.actor_ref)?;
                require_value("case_cancellation.reason", &cancellation.reason)?;
                if cancellation.transition_id != self.transition_id {
                    return Err("case_cancellation_transition_identity_mismatch".to_string());
                }
            }
            TransitionPayload::CaseClosed { closure } => {
                require_value("case_closure.actor_ref", &closure.actor_ref)?;
                require_value("case_closure.reason", &closure.reason)?;
                require_value("case_closure.cancellation_ref", &closure.cancellation_ref)?;
                if closure.transition_id != self.transition_id {
                    return Err("case_closure_transition_identity_mismatch".to_string());
                }
                require_causal_ref(
                    &self.causal_refs,
                    &closure.cancellation_ref,
                    "case_cancellation",
                )?;
            }
            TransitionPayload::CasePolicyBound { binding } => {
                binding.validate_integrity()?;
                if binding.case_id != self.case_id
                    || binding.bound_at_case_generation != self.sequence
                {
                    return Err("case_policy_binding_transition_mismatch".to_string());
                }
                require_causal_ref(&self.causal_refs, &binding.artifact_id, "policy_artifact")?;
                require_causal_ref(
                    &self.causal_refs,
                    &binding.publication_event_id,
                    "policy_publication_event",
                )?;
            }
            TransitionPayload::CasePolicyReplaced {
                prior_binding_id,
                binding,
            } => {
                require_value("prior_binding_id", prior_binding_id)?;
                binding.validate_integrity()?;
                if binding.case_id != self.case_id
                    || binding.bound_at_case_generation != self.sequence
                    || binding.replaces_binding_id.as_deref() != Some(prior_binding_id)
                {
                    return Err("case_policy_replacement_transition_mismatch".to_string());
                }
                require_causal_ref(&self.causal_refs, prior_binding_id, "prior_policy_binding")?;
                require_causal_ref(&self.causal_refs, &binding.artifact_id, "policy_artifact")?;
                require_causal_ref(
                    &self.causal_refs,
                    &binding.publication_event_id,
                    "policy_publication_event",
                )?;
            }
            TransitionPayload::CasePolicyUnbound {
                binding_id,
                lineage_id,
                actor_ref,
                reason,
            } => {
                require_value("binding_id", binding_id)?;
                require_value("lineage_id", lineage_id)?;
                require_value("actor_ref", actor_ref)?;
                require_value("reason", reason)?;
                require_causal_ref(&self.causal_refs, binding_id, "policy_binding")?;
            }
            TransitionPayload::ReviewResolved {
                review_id,
                attempt_id,
                decision_ref,
                receipt_ref,
                ..
            } => {
                require_value("review_id", review_id)?;
                require_value("attempt_id", attempt_id)?;
                require_value("decision_ref", decision_ref)?;
                require_value("receipt_ref", receipt_ref)?;
                require_causal_ref(&self.causal_refs, review_id, "review_request")?;
                require_causal_ref(&self.causal_refs, attempt_id, "review_attempt")?;
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string(self).map_err(|error| format!("transition_encode_failed: {error}"))
    }

    pub fn from_json(value: &str) -> Result<Self, String> {
        let transition: Self = serde_json::from_str(value)
            .map_err(|error| format!("transition_decode_failed: {error}"))?;
        transition.validate()?;
        Ok(transition)
    }
}

impl ReviewState {
    pub fn seal_policy_integrity(mut self) -> Result<Self, String> {
        if self.schema != REVIEW_REQUEST_SCHEMA {
            return Err("policy_review_integrity_requires_v2".to_string());
        }
        self.review_id.clear();
        self.integrity_digest.clear();
        let digest = self.policy_integrity_digest()?;
        self.review_id = format!("review:{}", &digest["sha256:".len()..][..32]);
        self.integrity_digest = digest;
        Ok(self)
    }

    pub fn validate_policy_integrity(&self) -> Result<(), String> {
        if self.schema != REVIEW_REQUEST_SCHEMA {
            return Err("policy_review_integrity_requires_v2".to_string());
        }
        let digest = self.policy_integrity_digest()?;
        if self.integrity_digest != digest
            || self.review_id != format!("review:{}", &digest["sha256:".len()..][..32])
        {
            return Err("policy_review_request_integrity_mismatch".to_string());
        }
        Ok(())
    }

    fn policy_integrity_digest(&self) -> Result<String, String> {
        serde_json::to_vec(&serde_json::json!({
            "schema": self.schema,
            "case_id": self.case_id,
            "operation_id": self.operation_id,
            "operation_digest": self.operation_digest,
            "initial_decision_id": self.initial_decision_id,
            "decision_basis_id": self.decision_basis_id,
            "decision_basis_digest": self.decision_basis_digest,
            "effective_policy_id": self.effective_policy_id,
            "effective_policy_digest": self.effective_policy_digest,
            "policy_binding_refs": self.policy_binding_refs,
            "policy_artifact_refs": self.policy_artifact_refs,
            "required_reviewer_roles": self.required_reviewer_roles,
            "resource_attachment_id": self.resource_attachment_id,
            "normalized_target": self.normalized_target,
            "created_at_generation": self.created_at_generation,
            "requested_by_participant": self.requested_by_participant,
            "policy_reason": self.policy_reason,
        }))
        .map(|value| digest_bytes(&value))
        .map_err(|error| format!("policy_review_request_digest_encode_failed: {error}"))
    }

    fn validate_for_schema(&self, transition_schema: &str) -> Result<(), String> {
        require_value("review_id", &self.review_id)?;
        require_value("policy_reason", &self.policy_reason)?;
        if supports_wave7_contract(transition_schema) {
            let wave10 = self.schema == REVIEW_REQUEST_SCHEMA;
            if self.schema != REVIEW_REQUEST_SCHEMA && self.schema != REVIEW_REQUEST_SCHEMA_V1
                || self.status != ReviewResolution::Pending
                || self.created_at_generation == 0
                || self.latest_action_id.is_some()
                || self.effective_decision_id.is_some()
            {
                return Err("invalid_review_request_contract".to_string());
            }
            if wave10 && !supports_wave10_contract(transition_schema) {
                return Err("policy_review_requires_yai_transition_v6".to_string());
            }
            let mut required = vec![
                ("review.operation_id", self.operation_id.as_str()),
                ("review.operation_digest", self.operation_digest.as_str()),
                (
                    "review.initial_decision_id",
                    self.initial_decision_id.as_str(),
                ),
                (
                    "review.requested_by_participant",
                    self.requested_by_participant.as_str(),
                ),
                (
                    "review.resource_attachment_id",
                    self.resource_attachment_id.as_str(),
                ),
                ("review.normalized_target", self.normalized_target.as_str()),
            ];
            if wave10 {
                self.validate_policy_integrity()?;
                required.extend([
                    ("review.case_id", self.case_id.as_str()),
                    ("review.decision_basis_id", self.decision_basis_id.as_str()),
                    (
                        "review.decision_basis_digest",
                        self.decision_basis_digest.as_str(),
                    ),
                    (
                        "review.effective_policy_id",
                        self.effective_policy_id.as_str(),
                    ),
                    (
                        "review.effective_policy_digest",
                        self.effective_policy_digest.as_str(),
                    ),
                ]);
                if self.required_reviewer_roles.is_empty() || !self.reviewer_participant.is_empty()
                {
                    return Err("invalid_policy_review_eligibility_contract".to_string());
                }
            } else {
                required.push((
                    "review.reviewer_participant",
                    self.reviewer_participant.as_str(),
                ));
            }
            for (field, value) in required {
                require_value(field, value)?;
            }
            return Ok(());
        }
        require_value("attempt_id", &self.attempt_id)?;
        require_value("requested_by_participant", &self.requested_by_participant)?;
        require_value("target_participant", &self.target_participant)?;
        require_value("reviewer_participant", &self.reviewer_participant)?;
        require_value("operation_kind", &self.operation_kind)?;
        require_value("carrier_family", &self.carrier_family)?;
        require_value("target_display", &self.target_display)?;
        require_value("sandbox_path", &self.sandbox_path)?;
        require_value("target_path", &self.target_path)?;
        Ok(())
    }
}

impl ReviewAction {
    pub fn validate_integrity(&self) -> Result<(), String> {
        if self.schema != REVIEW_ACTION_SCHEMA {
            return Err("unsupported_review_action_schema".to_string());
        }
        for (field, value) in [
            ("review_action.action_id", self.action_id.as_str()),
            (
                "review_action.integrity_digest",
                self.integrity_digest.as_str(),
            ),
            ("review_action.review_id", self.review_id.as_str()),
            ("review_action.operation_id", self.operation_id.as_str()),
            ("review_action.case_id", self.case_id.as_str()),
            (
                "review_action.reviewer_participant_id",
                self.reviewer_participant_id.as_str(),
            ),
            ("review_action.reason", self.reason.as_str()),
            ("review_action.source", self.source.as_str()),
        ] {
            require_value(field, value)?;
        }
        let material = review_action_digest_material(
            &self.schema,
            &self.review_id,
            &self.operation_id,
            &self.case_id,
            &self.reviewer_participant_id,
            &self.action,
            &self.reason,
            self.expected_case_generation,
            &self.source,
        );
        let digest = crate::effect::digest_bytes(material.to_string().as_bytes());
        if digest != self.integrity_digest
            || self.action_id != format!("review-action:{}", &digest[..32])
        {
            return Err("review_action_integrity_mismatch".to_string());
        }
        Ok(())
    }
}

impl TransitionPayload {
    fn is_wave3_kind(&self) -> bool {
        matches!(
            self,
            Self::ResourceAttached { .. }
                | Self::OperationNormalizationFailed { .. }
                | Self::OperationRecorded { .. }
                | Self::DecisionRecorded { .. }
                | Self::ExecutionGrantIssued { .. }
                | Self::EffectPrepared { .. }
                | Self::EffectFinalized { .. }
                | Self::EffectIndeterminate { .. }
                | Self::EffectReconciled { .. }
        )
    }

    fn is_wave4_kind(&self) -> bool {
        matches!(self, Self::InteractionTurnRecorded { .. })
    }

    fn is_wave7_kind(&self) -> bool {
        matches!(self, Self::ReviewActionRecorded { .. })
    }

    fn is_wave9_kind(&self) -> bool {
        matches!(
            self,
            Self::CasePolicyBound { .. }
                | Self::CasePolicyReplaced { .. }
                | Self::CasePolicyUnbound { .. }
        )
    }

    fn is_wave10_kind(&self) -> bool {
        match self {
            Self::DecisionRecorded { decision } => {
                decision.schema == crate::effect::DECISION_SCHEMA
            }
            Self::ExecutionGrantIssued { grant } => {
                grant.schema == crate::effect::EXECUTION_GRANT_SCHEMA
            }
            Self::ReviewRequested { review } => review.schema == REVIEW_REQUEST_SCHEMA,
            _ => false,
        }
    }

    fn is_wave11_kind(&self) -> bool {
        matches!(
            self,
            Self::ReviewInvalidated { .. }
                | Self::ExecutionGrantInvalidated { .. }
                | Self::CaseCancellationRequested { .. }
                | Self::CaseClosed { .. }
        )
    }
}

fn supports_wave7_contract(schema: &str) -> bool {
    matches!(
        schema,
        TRANSITION_SCHEMA | TRANSITION_SCHEMA_V6 | TRANSITION_SCHEMA_V5 | TRANSITION_SCHEMA_V4
    )
}

fn supports_wave9_contract(schema: &str) -> bool {
    matches!(
        schema,
        TRANSITION_SCHEMA | TRANSITION_SCHEMA_V6 | TRANSITION_SCHEMA_V5
    )
}

fn supports_wave10_contract(schema: &str) -> bool {
    matches!(schema, TRANSITION_SCHEMA | TRANSITION_SCHEMA_V6)
}

fn validate_provider_lineage(lineage: Option<&ProviderInvocationLineage>) -> Result<(), String> {
    let lineage = lineage.ok_or_else(|| "provider_semantic_lineage_required".to_string())?;
    require_value("projection_id", &lineage.projection_id)?;
    require_value("context_frame_id", &lineage.context_frame_id)?;
    require_value("rendered_input_id", &lineage.rendered_input_id)?;
    require_value("rendered_input_digest", &lineage.rendered_input_digest)?;
    require_value("output_contract_id", &lineage.output_contract_id)?;
    require_value(
        "continuation_disposition",
        &lineage.continuation_disposition,
    )
}

fn provider_lineage_matches(
    invocation: Option<&ProviderInvocationLineage>,
    result: Option<&ProviderInvocationLineage>,
) -> bool {
    match (invocation, result) {
        (None, None) => true,
        (Some(left), Some(right)) => {
            left.projection_id == right.projection_id
                && left.context_frame_id == right.context_frame_id
                && left.case_generation == right.case_generation
                && left.rendered_input_id == right.rendered_input_id
                && left.rendered_input_digest == right.rendered_input_digest
                && left.output_contract_id == right.output_contract_id
        }
        _ => false,
    }
}

impl ResourceAttachmentState {
    pub fn validate(&self) -> Result<(), String> {
        require_value("attachment_id", &self.attachment_id)?;
        require_value("allowed_write_prefix", &self.allowed_write_prefix)?;
        require_value("policy_id", &self.policy_id)?;
        require_value(
            "policy_owner_participant_id",
            &self.policy_owner_participant_id,
        )?;
        if crate::effect::normalize_write_prefix(&self.allowed_write_prefix)?
            != self.allowed_write_prefix
            || self.max_write_bytes == 0
        {
            return Err("invalid_resource_attachment_contract".to_string());
        }
        Ok(())
    }
}

impl FilesystemObservation {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != OBSERVATION_SCHEMA {
            return Err("unsupported_filesystem_observation_schema".to_string());
        }
        require_value("observation_id", &self.observation_id)?;
        require_value(
            "observation.resource_attachment_id",
            &self.resource_attachment_id,
        )?;
        require_value("observation.relative_path", &self.relative_path)?;
        if crate::effect::normalize_relative_path(&self.relative_path)? != self.relative_path {
            return Err("observation_path_not_normalized".to_string());
        }
        match self.state {
            crate::effect::ResourceState::File if self.content_digest.is_none() => {
                Err("file_observation_requires_digest".to_string())
            }
            crate::effect::ResourceState::Unavailable if self.error.is_none() => {
                Err("unavailable_observation_requires_error".to_string())
            }
            _ => Ok(()),
        }
    }
}

impl PreparedEffect {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != PREPARED_EFFECT_SCHEMA {
            return Err("unsupported_prepared_effect_schema".to_string());
        }
        for (field, value) in [
            ("effect_id", self.effect_id.as_str()),
            ("operation_id", self.operation_id.as_str()),
            ("decision_id", self.decision_id.as_str()),
            ("grant_id", self.grant_id.as_str()),
            ("case_id", self.case_id.as_str()),
            ("participant_id", self.participant_id.as_str()),
            (
                "resource_attachment_id",
                self.resource_attachment_id.as_str(),
            ),
            ("relative_path", self.relative_path.as_str()),
            (
                "intended_content_digest",
                self.intended_content_digest.as_str(),
            ),
            ("idempotency_key", self.idempotency_key.as_str()),
            ("carrier_backend", self.carrier_backend.as_str()),
        ] {
            require_value(field, value)?;
        }
        self.expected_pre_observation.validate()?;
        if self.expected_pre_observation.resource_attachment_id != self.resource_attachment_id
            || self.expected_pre_observation.relative_path != self.relative_path
        {
            return Err("prepared_effect_observation_target_mismatch".to_string());
        }
        Ok(())
    }
}

fn validate_finalization(
    effect_id: &str,
    post_observation: &FilesystemObservation,
    receipt: &EffectReceipt,
) -> Result<(), String> {
    require_value("effect_id", effect_id)?;
    post_observation.validate()?;
    if receipt.schema != EFFECT_RECEIPT_SCHEMA
        || receipt.effect_id != effect_id
        || receipt.receipt_id.is_empty()
        || receipt.operation_id.is_empty()
        || receipt.decision_id.is_empty()
        || receipt.grant_id.is_empty()
        || receipt.resource_attachment_id != post_observation.resource_attachment_id
        || receipt.relative_path != post_observation.relative_path
        || receipt.post_observation_id != post_observation.observation_id
        || matches!(
            receipt.outcome,
            EffectOutcome::Conflict | EffectOutcome::Indeterminate
        )
    {
        return Err("invalid_effect_finalization_contract".to_string());
    }
    Ok(())
}

fn apply_finalized_effect(
    state: &mut CaseState,
    effect_index: usize,
    post_observation: &FilesystemObservation,
    receipt: &EffectReceipt,
    generation: u64,
) -> Result<(), String> {
    let intended_digest = state.effects[effect_index].intended_content_digest.clone();
    if matches!(
        receipt.outcome,
        EffectOutcome::Applied | EffectOutcome::AlreadyApplied
    ) && post_observation.content_digest.as_deref() != Some(intended_digest.as_str())
    {
        return Err("successful_receipt_post_digest_mismatch".to_string());
    }
    let grant_id = state.effects[effect_index].grant_id.clone();
    let grant = state
        .grants
        .iter_mut()
        .find(|grant| grant.grant_id == grant_id)
        .ok_or_else(|| "finalize_grant_not_materialized".to_string())?;
    if grant.status != GrantLifecycle::Prepared {
        return Err("finalize_requires_prepared_grant".to_string());
    }
    grant.status = GrantLifecycle::Finalized;
    let effect = &mut state.effects[effect_index];
    effect.status = EffectLifecycle::Finalized;
    effect.outcome = Some(receipt.outcome.clone());
    effect.post_observation_id = Some(post_observation.observation_id.clone());
    effect.receipt_id = Some(receipt.receipt_id.clone());
    effect.updated_at_generation = generation;
    Ok(())
}

pub fn replay_case(case_id: &str, transitions: &[Transition]) -> Result<CaseState, String> {
    if transitions.is_empty() {
        return Err("cannot_replay_empty_case_history".to_string());
    }
    let first = &transitions[0];
    if first.case_id != case_id {
        return Err("replay_case_mismatch".to_string());
    }
    let TransitionPayload::CaseOpened { lifecycle } = &first.payload else {
        return Err("case_history_must_start_with_case_opened".to_string());
    };
    let mut state = CaseState::new(case_id, lifecycle.clone());
    for transition in transitions {
        state = state.reduce(transition)?;
    }
    Ok(state)
}

fn upsert_participant<'a>(
    participants: &'a mut Vec<ParticipantState>,
    participant_id: &str,
) -> &'a mut ParticipantState {
    if let Some(index) = participants
        .iter()
        .position(|participant| participant.participant_id == participant_id)
    {
        return &mut participants[index];
    }
    participants.push(ParticipantState {
        participant_id: participant_id.to_string(),
        roles: Vec::new(),
        admitted_views: Vec::new(),
    });
    participants.last_mut().expect("participant just inserted")
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_string());
    }
}

fn require_value(field: &str, value: &str) -> Result<(), String> {
    if value.is_empty() {
        Err(format!("missing_required_field: {field}"))
    } else {
        Ok(())
    }
}

fn require_causal_ref(refs: &[String], required: &str, role: &str) -> Result<(), String> {
    if refs.iter().any(|reference| reference == required) {
        Ok(())
    } else {
        Err(format!("missing_causal_reference: {role}:{required}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transition(sequence: u64, payload: TransitionPayload) -> Transition {
        Transition {
            schema: TRANSITION_SCHEMA.to_string(),
            transition_id: format!("transition:{sequence}"),
            case_id: "case:test".to_string(),
            sequence,
            committed_at_unix_ms: sequence,
            source: TransitionSource::component("test"),
            scope: None,
            causal_refs: Vec::new(),
            payload,
            provenance: Vec::new(),
            summary: Some("presentation only".to_string()),
        }
    }

    #[test]
    fn transition_roundtrip_and_unknown_field_policy() {
        let value = transition(
            1,
            TransitionPayload::CaseOpened {
                lifecycle: CaseLifecycle::Open,
            },
        );
        let encoded = value.to_json().expect("encode transition");
        assert_eq!(Transition::from_json(&encoded).expect("decode"), value);

        let with_unknown = encoded.replacen('{', "{\"future_field\":\"ignored\",", 1);
        assert_eq!(
            Transition::from_json(&with_unknown).expect("additive field accepted"),
            value
        );
    }

    #[test]
    fn version_and_unknown_kind_are_rejected() {
        let encoded = transition(
            1,
            TransitionPayload::CaseOpened {
                lifecycle: CaseLifecycle::Open,
            },
        )
        .to_json()
        .expect("encode");
        assert!(
            Transition::from_json(&encoded.replace(TRANSITION_SCHEMA, "yai.transition.v99"))
                .unwrap_err()
                .contains("unsupported_transition_schema")
        );
        assert!(
            Transition::from_json(&encoded.replace("case_opened", "future_kind"))
                .unwrap_err()
                .contains("transition_decode_failed")
        );
    }

    #[test]
    fn closure_is_mechanical_not_summary_based() {
        let mut result = transition(
            2,
            TransitionPayload::ProviderResultRecorded {
                result_id: "result:1".to_string(),
                invocation_id: "invocation:1".to_string(),
                provider_id: "provider:1".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:1".to_string(),
                semantic_lineage: Some(ProviderInvocationLineage {
                    projection_id: "projection:1".to_string(),
                    context_frame_id: "context-frame:1".to_string(),
                    case_generation: 1,
                    rendered_input_id: "rendered-input:1".to_string(),
                    rendered_input_digest: "digest:1".to_string(),
                    output_contract_id: "output-contract:1".to_string(),
                    continuation_disposition: "not_provided".to_string(),
                }),
                output: "hello".to_string(),
            },
        );
        result.summary = Some("complete:true invocation:wrong".to_string());
        assert!(result
            .validate()
            .unwrap_err()
            .contains("missing_causal_reference"));
        result.causal_refs.push("invocation:1".to_string());
        result.validate().expect("typed closure is complete");
    }
}
