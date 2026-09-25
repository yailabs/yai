//! Typed first-party application projections shared by non-owning YAI clients.
//!
//! The engine remains the owner of Case truth, policy, workflow, memory and
//! provider governance. This crate authenticates a local caller and composes
//! bounded, redacted product views from those owners. It owns no persistence.

pub mod capabilities;
// Shared bounded carriers, not product operations or authority. Existing CLI
// and Application orchestration use this one implementation.
pub mod provider_transport;
pub mod provider_execution;
pub mod cognitive_execution;
pub mod fast_search;
pub mod resource_transport;
pub mod resource_execution;
pub mod runtime_execution;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use yai_core_engine::admission::reviewer_is_eligible;
use yai_core_engine::case_policy::{NormativeReadiness, PolicyValidityPosture};
use yai_core_engine::cognitive::{
    CognitiveBindingRole, CognitiveCapability, CognitiveCapabilityRequirement,
    CognitiveDecisionBudget, CognitiveDecisionFrontier, CognitiveDecisionFrontierRequest,
};
use yai_core_engine::conversation::{turns_from_history, ConversationContentStore};
use yai_core_engine::effect::access::source::{
    CaseSourceDeclaration, SourcePhase, SourceProgress, SourceRole, SOURCE_DECLARATION_SCHEMA,
    SOURCE_PROGRESS_SCHEMA,
};
use yai_core_engine::effect::access::{LocalAccessBinding, ResourceAccessContract, ResourceAction};
use yai_core_engine::effect::{DecisionOutcome, LocalProcessBinding, Operation, ProcessSignalAction};
use yai_core_engine::governance::{compile_policy_source, scope_policy_compilation};
use yai_core_engine::handoff::{HandoffData, HandoffOutcome};
use yai_core_engine::memory_hierarchy::knowledge::KnowledgeRequest;
use yai_core_engine::memory_hierarchy::recall::RecallRequest;
use yai_core_engine::provider_governance::{
    ProviderAdapterKind, ProviderFailoverPolicy, ProviderLocality, ProviderProbeEvidence,
    ProviderRealizationShape, ProviderTargetInput, ProviderTrustPosture,
};
use yai_core_engine::security::AuthenticatedPrincipal;
use yai_core_engine::semantic_state::paging::PageRequest;
use yai_core_engine::semantic_state::working_recall::{
    AmbientRefreshRequest, WorkingRefreshRequest, WorkingStateRequest,
};
use yai_core_engine::semantic_state::SemanticWorkingState;
use yai_core_engine::store::lmdb::LmdbRecordStore;
use yai_core_engine::transition::{
    build_authenticated_review_action, CaseLifecycle, CaseState, PendingTransition,
    PrincipalParticipantLink, ResourceAttachmentState, ReviewAction, ReviewActionKind,
    ReviewRequirement, ReviewResolution, ReviewState, Transition, TransitionPayload,
    TransitionScope, TransitionSource, REVIEW_REQUEST_SCHEMA,
};
use yai_core_engine::workflow::{
    WorkflowCaseBinding, WorkflowDefinitionInput, WorkflowExecutorBinding, WorkflowPlanPatchInput,
    WorkflowResourceBinding,
};

pub const INTERFACES_REVISION: &str = "bae6cdf7cf17f3e6a58c0323852c7c0efeb26147";
pub const APPLICATION_PROTOCOL: &str = "yai.studio.application.v1";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OperationRequest {
    pub protocol: String,
    pub operation_ref: String,
    pub correlation_ref: String,
    #[serde(default)]
    pub input: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OperationError {
    pub code: String,
    pub message: String,
    pub safe_message: String,
    pub result_state: ResultState,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultState {
    Success,
    Partial,
    Unauthorized,
    Stale,
    CorePending,
    NotImplemented,
    TransportUnavailable,
    Error,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OperationResult {
    pub operation_ref: String,
    pub result_state: ResultState,
    pub correlation_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<OperationError>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseCapabilitiesInput {
    pub case_ref: String,
    pub participant_ref: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionTrajectoryInspectInput {
    pub case_ref: String,
    pub participant_ref: String,
    pub decision_ref: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionTrajectoryCorpusInput {
    pub case_ref: String,
    pub participant_ref: String,
    pub max_decisions: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionFrontierPrepareInput {
    pub working_state: SemanticWorkingState,
    pub max_candidates: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRequestPrepareInput {
    pub working_state: SemanticWorkingState,
    pub frontier: CognitiveDecisionFrontier,
    pub decision_kind: String,
    pub budget: CognitiveDecisionBudget,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecallExecuteInput {
    pub request: RecallRequest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkingStateCompileInput {
    pub request: WorkingStateRequest,
    #[serde(default)]
    pub pageable: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkingStateRefreshInput {
    pub working_state: SemanticWorkingState,
    pub request: WorkingRefreshRequest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkingStatePageInput {
    pub working_state: SemanticWorkingState,
    pub request: PageRequest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AmbientRefreshAssessInput {
    pub working_state: SemanticWorkingState,
    pub request: AmbientRefreshRequest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityBootstrapInput {
    pub tenant_id: String,
    pub organization_ref: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TenantGetInput {
    pub tenant_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TenantMemberAddInput {
    pub tenant_id: String,
    pub principal_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseCreateInput {
    pub tenant_id: String,
    pub case_ref: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseTerminalInput {
    pub case_ref: String,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeInspectInput {
    pub request: KnowledgeRequest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeSearchInput {
    pub request: KnowledgeRequest,
    pub query: String,
    pub limit: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeResolveInput {
    pub request: KnowledgeRequest,
    pub unit_ref: String,
}

/// The bounded qualified view carries source/relation closure. Lexical scores
/// remain discovery scores, not epistemic confidence or current authority.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KnowledgeSearchResult {
    pub view: yai_core_engine::memory_hierarchy::knowledge::KnowledgeView,
    pub hits: Vec<yai_core_engine::memory_index::LexicalHit>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KnowledgeResolveResult {
    pub view: yai_core_engine::memory_hierarchy::knowledge::KnowledgeView,
    pub unit: yai_core_engine::memory_hierarchy::knowledge::KnowledgeUnit,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KnowledgeNavigationResult {
    pub view_ref: String,
    pub navigation: String,
}

/// A durable domain submission can be looked up even if its acknowledgement
/// was lost. Neither the IPC correlation reference nor a client attachment is
/// an execution identity.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "domain", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExecutionReference {
    CognitiveRealization { plan_ref: String },
    CognitiveComposition { request_ref: String },
    Conversation { submission_ref: String },
    RuntimeWork { submission_ref: String },
    SourceAcquisition { source_ref: String, attempt: u64 },
    ResourceRequest { submission_ref: String },
    ControlledEffect { operation_ref: String },
}

/// Exact domain attempt, not a new Application job. Progress detail and backing
/// content are intentionally excluded from operational observation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceExecutionObservation {
    pub schema: String,
    pub case_ref: String,
    pub participant_ref: String,
    pub source_ref: String,
    pub attempt: u64,
    pub progress_ref: String,
    pub posture: SourceExecutionPosture,
    pub phase: SourcePhase,
    pub current_source_phase: SourcePhase,
    pub observed_generation: u64,
}

/// Durable progress plus an exact operational carrier observation. Unresolved
/// is retained for older attempts without carrier evidence. Neither Running
/// nor DeliveryIndeterminate permits replay or supplies effect authority.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceExecutionPosture {
    Unresolved,
    Running,
    DeliveryIndeterminate,
    Completed,
    WaitingForReview,
    Refused,
    Interrupted,
    Revoked,
}

impl From<&SourcePhase> for SourceExecutionPosture {
    fn from(phase: &SourcePhase) -> Self {
        match phase {
            SourcePhase::Acquiring => Self::Unresolved,
            SourcePhase::Acquired => Self::Completed,
            SourcePhase::AwaitingReview => Self::WaitingForReview,
            SourcePhase::Denied => Self::Refused,
            SourcePhase::Inaccessible | SourcePhase::NeedsProcessing => Self::Interrupted,
            SourcePhase::Revoked => Self::Revoked,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionGetInput {
    pub case_ref: String,
    pub participant_ref: String,
    pub execution: ExecutionReference,
    #[serde(default)]
    pub include_context: bool,
    #[serde(default)]
    pub include_output: bool,
}

/// Bounded discovery of durable references. Observation remains independently
/// authorized; this projection never retains output or grants dispatch authority.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionListInput {
    pub case_ref: String,
    pub participant_ref: String,
    #[serde(default = "execution_list_limit")]
    pub limit: usize,
}
fn execution_list_limit() -> usize { 16 }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionListEntry {
    pub execution: ExecutionReference,
    pub recorded_at_unix_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionListProjection {
    pub schema: String,
    pub case_ref: String,
    pub participant_ref: String,
    pub generation: u64,
    pub entries: Vec<ExecutionListEntry>,
    pub limit: usize,
    pub scope: String,
}

/// Domain-specific projections retain their own schema and lifecycle instead
/// of normalizing materially different outcomes into a generic job state.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExecutionObservation {
    CognitiveRealization(cognitive_execution::CognitiveRealizationObservation),
    Conversation(cognitive_execution::ConversationExecutionObservation),
    RuntimeWork(RuntimeExecutionObservation),
    SourceAcquisition(SourceExecutionObservation),
    ResourceRequest(resource_execution::ResourceExecutionObservation),
    ControlledEffect(resource_execution::ControlledEffectObservation),
}

/// Advance an already durable canonical Operation, not arbitrary candidate
/// JSON. Its exact identity is the reconnect/observation reference before any
/// external dispatch. Current admission is still required by the effect owner.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectProposeInput {
    pub case_ref: String,
    pub participant_ref: String,
    pub resource_ref: String,
    pub candidate_ref: String,
    pub expected_generation: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "posture", rename_all = "snake_case")]
pub enum EffectProposalResult {
    Recorded { operation: yai_core_engine::effect::Operation },
    NormalizationRefused { failure: yai_core_engine::effect::NormalizationFailure },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectSubmitInput {
    pub case_ref: String,
    pub participant_ref: String,
    pub operation_ref: String,
    pub expected_generation: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectReconcileInput {
    pub case_ref: String,
    pub participant_ref: String,
    pub operation_ref: String,
    pub effect_ref: String,
    pub expected_generation: u64,
    /// Only the existing filesystem no-effect recovery contract can retry.
    /// Process signals remain observation-only even when this is requested.
    #[serde(default)]
    pub retry_no_effect: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceRequestInput {
    pub case_ref: String,
    pub participant_ref: String,
    pub resource_ref: String,
    pub submission_ref: String,
    pub expected_generation: u64,
    pub request: yai_core_engine::effect::access::ResourceRequest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceSubmissionResult {
    pub execution: resource_execution::ResourceExecutionObservation,
    pub outcome: Option<resource_execution::ResourceActionOutcome>,
}

/// Domain posture is preserved; queue admission is not a canonical Decision.
/// No runtime lease token, filesystem path or task text crosses this surface.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeExecutionObservation {
    pub schema: String,
    pub execution_ref: String,
    pub submission_ref: String,
    pub case_ref: String,
    pub participant_ref: String,
    pub state: yai_core_engine::store::lmdb::RuntimeWorkState,
    pub attempt_count: u32,
    pub updated_at_unix_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runner: Option<RuntimeRunnerObservation>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeRunnerObservation {
    pub run_ref: String,
    pub checkpoint_digest: String,
    pub posture: runtime_execution::CaseRuntimeStop,
    pub stop_requested: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseStopInput {
    pub case_ref: String,
    pub participant_ref: String,
    pub submission_ref: String,
    pub run_ref: String,
}

impl From<yai_core_engine::store::lmdb::RuntimeWorkItem> for RuntimeExecutionObservation {
    fn from(item: yai_core_engine::store::lmdb::RuntimeWorkItem) -> Self {
        Self {
            schema: "yai.runtime_execution_observation.v1".into(),
            execution_ref: item.work_id,
            submission_ref: item.request_id,
            case_ref: item.case_id,
            participant_ref: item.participant_id,
            state: item.state,
            attempt_count: item.attempt_count,
            updated_at_unix_ms: item.updated_at_unix_ms,
            runner: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseRunInput {
    pub case_ref: String,
    pub participant_ref: String,
    pub resource_ref: String,
    pub submission_ref: String,
    pub task: String,
    pub budgets: yai_core_engine::store::lmdb::RuntimeCaseBudgets,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseResumeInput {
    pub case_ref: String,
    pub participant_ref: String,
    pub previous_submission_ref: String,
    pub submission_ref: String,
    pub run_ref: String,
    pub checkpoint_digest: String,
    pub budgets: yai_core_engine::store::lmdb::RuntimeCaseBudgets,
}

/// Returned only after the existing runtime queue transaction commits. A true
/// `created` acknowledges queue admission, never provider/effect completion.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionSubmissionResult {
    pub created: bool,
    pub execution: RuntimeExecutionObservation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowDefineInput {
    pub definition: WorkflowDefinitionInput,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowBindInput {
    pub case_ref: String,
    pub definition_ref: String,
    #[serde(default)]
    pub executor_bindings: Vec<WorkflowExecutorBinding>,
    #[serde(default)]
    pub resource_bindings: Vec<WorkflowResourceBinding>,
    #[serde(default)]
    pub case_bindings: Vec<WorkflowCaseBinding>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowInputRecordInput {
    pub case_ref: String,
    pub node_ref: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowPatchProposeInput {
    pub case_ref: String,
    pub patch: WorkflowPlanPatchInput,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowPatchAdoptInput {
    pub case_ref: String,
    pub patch_ref: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffPendingInput { pub case_ref: String }

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffInspectInput { pub case_ref: String, pub handoff_ref: String }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HandoffPendingProjection {
    pub schema: String, pub case_ref: String, pub generation: u64,
    pub offers: Vec<yai_core_engine::handoff::HandoffOffer>,
    pub scope: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HandoffInspection {
    pub schema: String, pub case_ref: String, pub generation: u64,
    pub offer: yai_core_engine::handoff::HandoffOffer,
    pub acceptance: Option<yai_core_engine::handoff::HandoffAcceptance>,
    pub decline: Option<yai_core_engine::handoff::HandoffDecline>,
    pub result: Option<yai_core_engine::handoff::HandoffResult>,
    pub reconciliation: Option<yai_core_engine::handoff::HandoffReconciliation>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffOfferInput {
    pub source_case_ref: String,
    pub target_case_ref: String,
    pub request: HandoffData,
    #[serde(default)]
    pub required_target_roles: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffAcceptInput {
    pub target_case_ref: String,
    pub source_case_ref: String,
    pub handoff_ref: String,
    pub participant_ref: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffDeclineInput {
    pub target_case_ref: String,
    pub source_case_ref: String,
    pub handoff_ref: String,
    pub participant_ref: String,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffResultInput {
    pub target_case_ref: String,
    pub handoff_ref: String,
    pub participant_ref: String,
    pub outcome: HandoffOutcome,
    pub result: HandoffData,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffReconcileInput {
    pub source_case_ref: String,
    pub handoff_ref: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyIngestInput {
    pub tenant_id: String,
    pub source_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyArtifactLifecycleInput {
    pub artifact_ref: String,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CasePolicyBindInput {
    pub case_ref: String,
    pub artifact_ref: String,
    pub expected_generation: u64,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CasePolicyReplaceInput {
    pub case_ref: String,
    pub prior_binding_ref: String,
    pub artifact_ref: String,
    pub expected_generation: u64,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CasePolicyUnbindInput {
    pub case_ref: String,
    pub binding_ref: String,
    pub expected_generation: u64,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewResolveInput {
    pub case_ref: String,
    pub review_ref: String,
    #[serde(default)]
    pub participant_ref: Option<String>,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReviewResolveResult {
    pub review_ref: String,
    pub action: ReviewAction,
    pub effective_decision_ref: Option<String>,
    pub state: CaseState,
    pub external_effect: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParticipantRoleAddInput {
    pub case_ref: String,
    pub participant_ref: String,
    pub role: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParticipantPrincipalLinkInput {
    pub case_ref: String,
    pub participant_ref: String,
    pub principal_ref: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParticipantViewAdmitInput {
    pub case_ref: String,
    pub participant_ref: String,
    pub consumer: String,
    pub view_kind: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderModelsInput {
    pub tenant_id: String,
    pub endpoint: String,
    pub locality: ProviderLocality,
    pub credential_ref: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisteredProviderModelsInput {
    pub tenant_id: String,
    pub target_ref: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProviderModelDiscoveryInput {
    Connection(ProviderModelsInput),
    Registered(RegisteredProviderModelsInput),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderRegisterInput {
    pub tenant_id: String,
    pub provider_key: String,
    pub adapter: ProviderAdapterKind,
    pub endpoint: String,
    pub model_id: String,
    pub credential_ref: String,
    pub locality: ProviderLocality,
    #[serde(default)]
    pub extension_adapter_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderQualifyInput {
    pub target_ref: String,
    pub evidence: ProviderProbeEvidence,
    pub suite_ref: String,
    #[serde(default)]
    pub valid_until_unix_ms: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderProbeInput {
    pub target_ref: String,
    pub submission_ref: String,
    pub embedding: bool,
    pub realization_shapes: Vec<yai_core_engine::provider_governance::ProviderRealizationShape>,
    pub qualify: bool,
    #[serde(default)]
    pub valid_for_ms: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderProbeGetInput {
    pub target_ref: String,
    pub submission_ref: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderProbeExecution {
    pub schema: String,
    pub target_ref: String,
    pub submission_ref: String,
    pub created: bool,
    pub posture: String,
    pub run: yai_core_engine::provider_governance::ProviderProbeRun,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderTrustInput {
    pub target_ref: String,
    pub posture: ProviderTrustPosture,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderSuitabilityRecordInput {
    pub target_ref: String,
    pub capability: CognitiveCapability,
    pub suite_ref: String,
    pub run_ref: String,
    pub evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderCaseBindInput {
    pub case_ref: String,
    pub participant_ref: String,
    pub ordered_target_refs: Vec<String>,
    pub failover_policy: ProviderFailoverPolicy,
    pub max_attempts_per_turn: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CognitiveBindInput {
    pub case_ref: String,
    pub participant_ref: String,
    pub role: CognitiveBindingRole,
    pub capability: CognitiveCapability,
    pub candidates: Vec<CognitiveTargetReference>,
    #[serde(default)]
    pub replace: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CognitiveTargetReference {
    pub target_ref: String,
    pub semantic_evidence_ref: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CognitivePlanInput {
    pub case_ref: String,
    pub participant_ref: String,
    pub requirement: CognitiveCapabilityRequirement,
    #[serde(default)]
    pub realization_shape: Option<ProviderRealizationShape>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceImportInput {
    pub case_ref: String,
    pub definition: resource_execution::ResourceDefinition,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceAttachInput {
    pub binding: LocalAccessBinding,
    pub access: ResourceAccessContract,
    pub policy_owner_participant_ref: String,
    #[serde(default)]
    pub write_prefix: Option<String>,
    #[serde(default)]
    pub max_write_bytes: Option<usize>,
    #[serde(default)]
    pub review_requirement: ReviewRequirement,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceAttachProcessInput {
    pub case_ref: String,
    pub attachment_ref: String,
    pub pid: u32,
    pub policy_owner_participant_ref: String,
    pub actions: Vec<ProcessSignalAction>,
    pub review_requirement: ReviewRequirement,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceDeclareInput {
    pub case_ref: String,
    pub perimeter: String,
    pub logical_name: String,
    pub participant_ref: String,
    pub resource_ref: String,
    pub roles: Vec<SourceRole>,
    pub action: ResourceAction,
    #[serde(default)]
    pub bootstrap_policy: bool,
    pub media_type: String,
}

/// `source_ref` + `attempt` is the durable submission identity, known before
/// transport. Retrying it observes the admitted attempt; it never resumes it.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceAcquireInput {
    pub case_ref: String,
    pub participant_ref: String,
    pub source_ref: String,
    pub attempt: u64,
    pub expected_generation: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceResumeInput {
    pub case_ref: String,
    pub participant_ref: String,
    pub source_ref: String,
    pub attempt: u64,
    pub expected_generation: u64,
    pub previous_progress_ref: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceAdvancementPosture {
    /// This call ran the carrier; the domain phase is still authoritative.
    Returned,
    /// Duplicate submission: no carrier was called.
    ExistingAttempt,
    /// Admission committed but advancement did not yield a terminal projection.
    /// Observe the exact attempt; this is not permission to redispatch.
    ObservationRequired,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceAcquisitionSubmissionResult {
    pub schema: String,
    pub created: bool,
    pub execution: SourceExecutionObservation,
    pub advancement: SourceAdvancementPosture,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourcePublishInput {
    pub case_ref: String,
    pub source_ref: String,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRevokeInput {
    pub case_ref: String,
    pub source_ref: String,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CaseStateMutationResult {
    pub changed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition_ref: Option<String>,
    pub state: CaseState,
}

impl CaseStateMutationResult {
    fn unchanged(state: CaseState) -> Self {
        Self {
            changed: false,
            transition_ref: None,
            state,
        }
    }

    fn committed(commit: yai_core_engine::store::lmdb::CanonicalCommit) -> Self {
        Self {
            changed: true,
            transition_ref: Some(commit.transition.transition_id),
            state: commit.state,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SourcePolicyPublicationResult {
    pub state: CaseState,
    pub normative: yai_core_engine::case_policy::NormativeStatus,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CaseUpdate {
    pub protocol: String,
    pub event_ref: String,
    pub event_type: String,
    pub event_family: String,
    pub case_ref: String,
    pub generation: u64,
    pub sequence: u64,
    pub cursor: String,
    pub affected_views: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct LocalApplication {
    home_path: PathBuf,
    store_path: PathBuf,
}

impl Default for LocalApplication {
    fn default() -> Self {
        Self::from_yai_home(default_yai_home())
    }
}

impl LocalApplication {
    pub fn from_yai_home(home: impl AsRef<Path>) -> Self {
        let home_path = home.as_ref().to_path_buf();
        Self {
            store_path: home_path.join("store").join("lmdb"),
            home_path,
        }
    }

    pub fn call(&self, request: OperationRequest) -> OperationResult {
        if request.protocol != APPLICATION_PROTOCOL {
            return failure(
                &request,
                ResultState::Error,
                "version_mismatch",
                "Studio application protocol version is not supported.",
            );
        }
        match self.call_inner(&request) {
            Ok(data) => success(&request, data),
            Err(error) => map_error(&request, &error),
        }
    }

    pub fn visible_generations(&self) -> Result<BTreeMap<String, u64>, String> {
        let (store, auth) = self.open()?;
        Ok(store
            .list_case_states_authorized(&auth, None, 1024)?
            .into_iter()
            .map(|state| (state.case_id, state.generation))
            .collect())
    }

    pub fn update_for(&self, case_ref: &str, generation: u64, sequence: u64) -> CaseUpdate {
        CaseUpdate {
            protocol: APPLICATION_PROTOCOL.to_string(),
            event_ref: format!("case-update:{case_ref}:{generation}"),
            event_type: "case_projection_invalidated".to_string(),
            event_family: "case".to_string(),
            case_ref: case_ref.to_string(),
            generation,
            sequence,
            cursor: format!("{case_ref}:{generation}:{sequence}"),
            affected_views: vec![
                "overview".to_string(),
                "environment".to_string(),
                "knowledge".to_string(),
                "memory".to_string(),
                "authority".to_string(),
                "work".to_string(),
                "compute".to_string(),
                "conversation".to_string(),
            ],
        }
    }

    fn open(&self) -> Result<(LmdbRecordStore, AuthenticatedPrincipal), String> {
        let auth = AuthenticatedPrincipal::authenticate_local()?;
        let store = LmdbRecordStore::open(&self.store_path)?;
        Ok((store, auth))
    }

    fn execution_observation(&self, store: &LmdbRecordStore, auth: &AuthenticatedPrincipal,
        input: ExecutionGetInput) -> Result<Value, String> {
        if input.include_context && !matches!(&input.execution,
            ExecutionReference::Conversation {..} | ExecutionReference::CognitiveComposition {..}) {
            return Err("execution_context_domain_not_supported".into());
        }
        if input.include_output && !matches!(&input.execution, ExecutionReference::ResourceRequest {..}) {
            return Err("execution_output_domain_not_supported".into());
        }
        match input.execution {
            ExecutionReference::ControlledEffect { operation_ref } => {
                encode_result("execution_get", ExecutionObservation::ControlledEffect(
                    resource_execution::observe_controlled_effect(store, auth,
                        &input.case_ref, &input.participant_ref, &operation_ref)?))
            }
            ExecutionReference::ResourceRequest { submission_ref } => {
                let state = store.get_case_state_authorized(auth, &input.case_ref)?;
                let principal = auth.projected_principal_id();
                let history = store.list_case_transitions(&state.case_id)?;
                let operation = history.iter().find_map(|t| match &t.payload {
                    TransitionPayload::OperationRecorded { operation }
                        if operation.participant_id == input.participant_ref
                            && matches!(&operation.origin,
                                yai_core_engine::effect::OperationOrigin::ParticipantRequest { request_id, principal_id, .. }
                                if request_id == &submission_ref && principal_id == &principal) => Some(operation),
                    _ => None,
                }).ok_or("resource_execution_not_visible")?;
                encode_result("execution_get", ExecutionObservation::ResourceRequest(
                    resource_execution::observe_with_output(store, auth, &input.case_ref,
                        &input.participant_ref, &operation.operation_id, input.include_output)?))
            }
            ExecutionReference::CognitiveRealization { plan_ref } => {
                encode_result("execution_get", ExecutionObservation::CognitiveRealization(
                    cognitive_execution::observe_realization(&self.home_path, auth, store,
                        &input.case_ref, &input.participant_ref, &plan_ref)?))
            }
            ExecutionReference::CognitiveComposition { request_ref } => {
                let mut observed = cognitive_execution::observe_cognitive_request(&self.home_path, auth, store,
                    &input.case_ref, &input.participant_ref, &request_ref)?;
                if input.include_context { cognitive_execution::include_prepared_context(&self.home_path, auth, store, &mut observed)?; }
                encode_result("execution_get", ExecutionObservation::Conversation(observed))
            }
            ExecutionReference::Conversation { submission_ref } => {
                let mut observed = cognitive_execution::observe_submission(&self.home_path, auth, store,
                    &input.case_ref, &input.participant_ref, &submission_ref)?;
                if input.include_context { cognitive_execution::include_prepared_context(&self.home_path, auth, store, &mut observed)?; }
                encode_result("execution_get", ExecutionObservation::Conversation(observed))
            }
            ExecutionReference::RuntimeWork { submission_ref } => {
                let item = store.observe_runtime_submission_authorized(
                    &auth, &input.case_ref, &input.participant_ref, &submission_ref,
                )?;
                encode_result("execution_get", ExecutionObservation::RuntimeWork(
                    runtime_execution_observation(&self.home_path, item)?))
            }
            ExecutionReference::SourceAcquisition { source_ref, attempt } => {
                encode_result("execution_get", ExecutionObservation::SourceAcquisition(
                    resource_execution::source::observe_execution(
                        &self.home_path, store, auth, &input.case_ref,
                        &input.participant_ref, &source_ref, attempt)?))
            }
        }
    }

    fn call_inner(&self, request: &OperationRequest) -> Result<Value, String> {
        if !capabilities::APPLICATION_OPERATIONS
            .iter()
            .any(|operation| operation.operation_id == request.operation_ref)
        {
            return Err("operation_not_implemented".to_string());
        }
        if request.operation_ref == "application.capabilities" {
            AuthenticatedPrincipal::authenticate_local()?;
            capabilities::validate_capability_catalog()?;
            return serde_json::to_value(capabilities::capability_catalog())
                .map_err(|error| format!("application_capabilities_encode:{error}"));
        }
        let (store, auth) = self.open()?;
        match request.operation_ref.as_str() {
            "system.status" | "runtime.readiness" => Ok(json!({
                "protocol": APPLICATION_PROTOCOL,
                "interfaces_revision": INTERFACES_REVISION,
                "transport": "tauri-local-bridge",
                "local": true,
                "authenticated_principal": auth.projected_principal_id(),
                "runtime_state": "available"
            })),
            "case.list" | "case.recent" => case_list(&store, &auth),
            "case.open" => {
                let case_ref = input_case_ref(request)?;
                case_open(&store, &auth, case_ref)
            }
            "case.capabilities" => {
                let input: CaseCapabilitiesInput = decode_input(request)?;
                let view = store.case_capability_view_authorized(
                    &auth,
                    &input.case_ref,
                    &input.participant_ref,
                )?;
                serde_json::to_value(view)
                    .map_err(|error| format!("case_capabilities_encode:{error}"))
            }
            "decision.trajectory.inspect" => {
                let input: DecisionTrajectoryInspectInput = decode_input(request)?;
                let content = ConversationContentStore::open_existing(&self.home_path).ok();
                encode_result("decision_trajectory", store.decision_trajectory_authorized(
                    &auth, &input.case_ref, &input.participant_ref,
                    &input.decision_ref, content.as_ref())?)
            }
            "decision.trajectory.corpus" | "decision.trajectory.evaluate" => {
                let input: DecisionTrajectoryCorpusInput = decode_input(request)?;
                let content = ConversationContentStore::open_existing(&self.home_path).ok();
                let corpus = store.decision_trajectory_corpus_authorized(
                    &auth, &input.case_ref, &input.participant_ref,
                    input.max_decisions, content.as_ref())?;
                if request.operation_ref == "decision.trajectory.evaluate" {
                    encode_result("decision_trajectory_evaluation",
                        yai_core_engine::semantic_state::historical::trajectory::evaluate(&corpus)?)
                } else {
                    encode_result("decision_trajectory_corpus", corpus)
                }
            }
            "case.summary" => {
                let case_ref = input_case_ref(request)?;
                let content = ConversationContentStore::open_existing(&self.home_path).ok();
                case_snapshot(
                    &store,
                    &auth,
                    case_ref,
                    content.as_ref(),
                    request
                        .input
                        .get("expected_generation")
                        .and_then(Value::as_u64),
                )
            }
            "identity.bootstrap" => {
                let input: IdentityBootstrapInput = decode_input(request)?;
                encode_result(
                    "identity_bootstrap",
                    store.bootstrap_local_security(
                        &auth,
                        &input.tenant_id,
                        &input.organization_ref,
                        now_unix_ms()?,
                    )?,
                )
            }
            "identity.current" => encode_result(
                "identity_current",
                json!({
                    "principal": store.enrolled_principal(&auth)?,
                    "tenants": store.list_principal_tenants(&auth)?,
                    "authentication": auth.binding(),
                }),
            ),
            "tenant.list" => encode_result("tenant_list", store.list_principal_tenants(&auth)?),
            "tenant.get" => {
                let input: TenantGetInput = decode_input(request)?;
                let context = store.resolve_security_context(&auth, &input.tenant_id)?;
                let tenant = store
                    .get_tenant(&input.tenant_id)?
                    .ok_or_else(|| "tenant_not_visible".to_string())?;
                encode_result(
                    "tenant_get",
                    json!({ "tenant": tenant, "membership": context.membership() }),
                )
            }
            "tenant.member.add" => {
                let input: TenantMemberAddInput = decode_input(request)?;
                encode_result(
                    "tenant_member_add",
                    store.add_tenant_member(
                        &auth,
                        &input.tenant_id,
                        &input.principal_id,
                        now_unix_ms()?,
                    )?,
                )
            }
            "knowledge.inspect" | "knowledge.navigation" => {
                let input: KnowledgeInspectInput = decode_input(request)?;
                let content = ConversationContentStore::open_existing(&self.home_path).ok();
                let result = store.case_knowledge_authorized(&auth, input.request, content.as_ref())?;
                if request.operation_ref == "knowledge.navigation" {
                    let navigation = result.view.navigation();
                    encode_result("knowledge_navigation", KnowledgeNavigationResult {
                        view_ref: result.view.id, navigation,
                    })
                } else {
                    encode_result("knowledge_inspect", result.view)
                }
            }
            "knowledge.search" => {
                let input: KnowledgeSearchInput = decode_input(request)?;
                if input.limit == 0 || input.limit > 128 || input.query.len() > 2048 {
                    return Err("knowledge_query_bound".into());
                }
                let content = ConversationContentStore::open_existing(&self.home_path).ok();
                let result = store.case_knowledge_authorized(&auth, input.request, content.as_ref())?;
                let hits = result.view.search(&input.query, input.limit)?;
                encode_result("knowledge_search", KnowledgeSearchResult { view: result.view, hits })
            }
            "knowledge.resolve" => {
                let input: KnowledgeResolveInput = decode_input(request)?;
                if input.unit_ref.is_empty() || input.unit_ref.len() > 256 {
                    return Err("knowledge_reference_unavailable".into());
                }
                let content = ConversationContentStore::open_existing(&self.home_path).ok();
                let result = store.case_knowledge_authorized(&auth, input.request, content.as_ref())?;
                let unit = result.view.resolve(&input.unit_ref)?.clone();
                encode_result("knowledge_resolve", KnowledgeResolveResult { view: result.view, unit })
            }
            "effect.reconcile" => {
                let input: EffectReconcileInput = decode_input(request)?;
                let existing = resource_execution::observe_controlled_effect(&store, &auth,
                    &input.case_ref, &input.participant_ref, &input.operation_ref)?;
                let progress = existing.progress.as_ref().ok_or("execution_not_visible")?;
                if progress.effect_id.as_deref() != Some(input.effect_ref.as_str()) {
                    return Err("execution_not_visible".into());
                }
                if progress.status == resource_execution::ControlledEffectTurnStatus::Finalized {
                    return encode_result("effect_reconcile", existing);
                }
                if existing.observed_generation != input.expected_generation {
                    return Err("controlled_effect_reconciliation_stale".into());
                }
                struct ApplicationEffectHooks;
                impl resource_execution::ControlledEffectHooks for ApplicationEffectHooks {}
                resource_execution::reconcile_controlled_effect(&store, &auth, &input.case_ref,
                    Some(&input.effect_ref), input.retry_no_effect, &mut ApplicationEffectHooks)?;
                encode_result("effect_reconcile", resource_execution::observe_controlled_effect(
                    &store, &auth, &input.case_ref, &input.participant_ref, &input.operation_ref)?)
            }
            "effect.propose" => {
                let input: EffectProposeInput = decode_input(request)?;
                let result = store.propose_controlled_candidate_authorized(&auth, &input.case_ref,
                    &input.participant_ref, &input.resource_ref, &input.candidate_ref, Some(input.expected_generation))?;
                encode_result("effect_propose", match result {
                    Ok(operation) => EffectProposalResult::Recorded { operation },
                    Err(failure) => EffectProposalResult::NormalizationRefused { failure },
                })
            }
            "effect.submit" => {
                let input: EffectSubmitInput = decode_input(request)?;
                let operation = store.controlled_operation_authorized(&auth, &input.case_ref,
                    &input.participant_ref, &input.operation_ref)?;
                let existing = resource_execution::observe_controlled_effect(&store, &auth,
                    &input.case_ref, &input.participant_ref, &input.operation_ref)?;
                // PREPARE is the irreversible dispatch boundary. Retry only
                // observes it, including when the old acknowledgement was lost.
                if existing.progress.as_ref().is_some_and(|p| p.effect_id.is_some()) {
                    return encode_result("effect_submit", existing);
                }
                if existing.observed_generation != input.expected_generation {
                    return Err("controlled_effect_submission_stale".into());
                }
                struct ApplicationEffectHooks;
                impl resource_execution::ControlledEffectHooks for ApplicationEffectHooks {}
                resource_execution::advance_controlled_operation(&auth, &mut ApplicationEffectHooks,
                    &store, &operation)?;
                encode_result("effect_submit", resource_execution::observe_controlled_effect(
                    &store, &auth, &input.case_ref, &input.participant_ref, &input.operation_ref)?)
            }
            "execution.list" => {
                let input: ExecutionListInput = decode_input(request)?;
                if !(1..=32).contains(&input.limit) {
                    return Err("execution_list_limit_invalid".into());
                }
                let state = cognitive_execution::authorized_conversation_case(
                    &auth, &store, &input.case_ref, &input.participant_ref)?;
                let principal = auth.projected_principal_id();
                let mut candidates = Vec::new();
                for transition in store.list_case_transitions(&input.case_ref)? {
                    let reference = match transition.payload {
                        TransitionPayload::OperationRecorded { operation }
                            if operation.participant_id == input.participant_ref => {
                            match operation.origin {
                                yai_core_engine::effect::OperationOrigin::ParticipantRequest { request_id, principal_id, .. }
                                    if operation.resource_request.is_some() && principal_id == principal =>
                                    Some(ExecutionReference::ResourceRequest { submission_ref: request_id }),
                                _ if operation.resource_request.is_none() =>
                                    Some(ExecutionReference::ControlledEffect { operation_ref: operation.operation_id }),
                                _ => None,
                            }
                        }
                        TransitionPayload::CaseSourceProgressed { progress } =>
                            Some(ExecutionReference::SourceAcquisition { source_ref: progress.source_id, attempt: progress.attempt }),
                        TransitionPayload::ConversationExecutionIntentRecorded { request }
                            if request.participant_id == input.participant_ref =>
                            Some(ExecutionReference::CognitiveComposition { request_ref: request.request_id }),
                        TransitionPayload::ProviderSelectionRecorded { selection }
                            if selection.participant_id == input.participant_ref =>
                            selection.logical_turn_id.strip_prefix("cognitive-realization:")
                                .map(|plan| ExecutionReference::CognitiveRealization { plan_ref: plan.into() }),
                        _ => None,
                    };
                    if let Some(execution) = reference {
                        candidates.push(ExecutionListEntry { execution, recorded_at_unix_ms: transition.committed_at_unix_ms });
                    }
                }
                for item in store.list_runtime_work_authorized(&auth)? {
                    if item.case_id == input.case_ref && item.participant_id == input.participant_ref {
                        candidates.push(ExecutionListEntry {
                            execution: ExecutionReference::RuntimeWork { submission_ref: item.request_id },
                            recorded_at_unix_ms: item.enqueued_at_unix_ms,
                        });
                    }
                }
                candidates.sort_by_key(|entry| std::cmp::Reverse(entry.recorded_at_unix_ms));
                let mut seen = std::collections::BTreeSet::new();
                let mut entries = Vec::new();
                // Bound expensive current-authority observations. This is recent
                // discovery, not a complete ledger or a new execution index.
                for entry in candidates.into_iter().filter(|entry| seen.insert(
                    serde_json::to_string(&entry.execution).expect("execution reference serializes")
                )).take(128) {
                    let observe = OperationRequest {
                        protocol: APPLICATION_PROTOCOL.into(), operation_ref: "execution.get".into(),
                        correlation_ref: request.correlation_ref.clone(),
                        input: json!({"case_ref": input.case_ref, "participant_ref": input.participant_ref,
                            "execution": entry.execution}),
                    };
                    match self.execution_observation(&store, &auth, ExecutionGetInput {
                        case_ref: input.case_ref.clone(), participant_ref: input.participant_ref.clone(),
                        execution: entry.execution.clone(), include_context: false, include_output: false,
                    }) {
                        Ok(_) => entries.push(entry),
                        Err(error) => match map_error(&observe, &error).result_state {
                            ResultState::Unauthorized | ResultState::Stale => continue,
                            _ => return Err(error),
                        },
                    }
                    if entries.len() == input.limit { break; }
                }
                let current = cognitive_execution::authorized_conversation_case(
                    &auth, &store, &input.case_ref, &input.participant_ref)?;
                if current.generation != state.generation { return Err("execution_list_stale".into()); }
                encode_result("execution_list", ExecutionListProjection {
                    schema: "yai.execution_list_projection.v1".into(),
                    case_ref: input.case_ref, participant_ref: input.participant_ref,
                    generation: state.generation, entries, limit: input.limit,
                    scope: "recent_visible_references_bounded_128_candidates_not_complete_history".into(),
                })
            }
            "execution.get" => {
                let input: ExecutionGetInput = decode_input(request)?;
                self.execution_observation(&store, &auth, input)
            }
            "cognitive.realization.prepare" => {
                let input: cognitive_execution::CognitiveRealizationPrepareInput = decode_input(request)?;
                encode_result("cognitive_realization_prepare", cognitive_execution::prepare_realization(
                    &self.home_path, &auth, &store, input)?)
            }
            "cognitive.realize" => {
                let input: cognitive_execution::CognitiveRealizeInput = decode_input(request)?;
                encode_result("cognitive_realize", cognitive_execution::submit_realization(
                    &self.home_path, &auth, &store, input)?)
            }
            "cognitive.compose" => {
                let input: cognitive_execution::CognitiveComposeInput = decode_input(request)?;
                encode_result("cognitive_compose", cognitive_execution::submit_composition(
                    &self.home_path, &auth, &store, input)?)
            }
            "conversation.send" => {
                let input: cognitive_execution::ConversationSendInput = decode_input(request)?;
                encode_result("conversation_send", cognitive_execution::submit_conversation(
                    &self.home_path, &auth, &store, input)?)
            }
            "case.stop" => {
                let input: CaseStopInput = decode_input(request)?;
                let item = store.observe_runtime_submission_authorized(
                    &auth, &input.case_ref, &input.participant_ref, &input.submission_ref,
                )?;
                let observed = runtime_execution_observation(&self.home_path, item.clone())?;
                let runner = observed.runner.as_ref().ok_or("case_runtime_not_started")?;
                if runner.run_ref != input.run_ref {
                    return Err("case_runtime_stop_checkpoint_stale".into());
                }
                if !item.state.is_terminal() {
                    runtime_execution::request_checkpoint_stop_at(
                        &runtime_execution::checkpoint_path_for(&self.home_path, &input.case_ref),
                        &input.case_ref, &input.run_ref, Some(&item.work_id),
                    )?;
                }
                encode_result("case_stop", runtime_execution_observation(&self.home_path, item)?)
            }
            "resource.request" => {
                let input: ResourceRequestInput = decode_input(request)?;
                let operation = store.record_participant_resource_request(&auth,
                    &input.case_ref, &input.participant_ref, &input.resource_ref,
                    &input.submission_ref, input.expected_generation, input.request)?;
                // Canonical Operation admission precedes all dispatch. A lost
                // response is resolved by the same submission identity; the
                // existing PREPARE owner forbids replay of uncertain effects.
                let before = resource_execution::observe(&store, &auth, &input.case_ref,
                    &input.participant_ref, &operation.operation_id)?;
                let outcome = if matches!(before.posture,
                    resource_execution::ResourceExecutionPosture::Completed { .. }
                    | resource_execution::ResourceExecutionPosture::EffectRecorded { .. }
                    | resource_execution::ResourceExecutionPosture::Refused { .. }
                    | resource_execution::ResourceExecutionPosture::PreparedOrIndeterminate { .. }) {
                    None
                } else {
                    resource_execution::advance(&self.home_path, &store, &auth, &operation).ok()
                };
                let execution = resource_execution::observe(&store, &auth, &input.case_ref,
                    &input.participant_ref, &operation.operation_id)?;
                encode_result("resource_request", ResourceSubmissionResult { execution, outcome })
            }
            "case.resume" => {
                let input: CaseResumeInput = decode_input(request)?;
                let previous = store.observe_runtime_submission_authorized(&auth, &input.case_ref,
                    &input.participant_ref, &input.previous_submission_ref)?;
                let resume = yai_core_engine::store::lmdb::RuntimeWorkResume {
                    work_id: previous.work_id.clone(), run_id: input.run_ref,
                    checkpoint_digest: input.checkpoint_digest,
                };
                // A retry observes the already admitted continuation even after
                // its checkpoint has advanced. The queue checks exact identity.
                match store.observe_runtime_submission_authorized(&auth, &input.case_ref,
                    &input.participant_ref, &input.submission_ref) {
                    Ok(_) => {},
                    Err(error) if error == "execution_not_visible" => {
                        let checkpoint = runtime_execution::read_checkpoint_at(
                            &runtime_execution::checkpoint_path_for(&self.home_path, &input.case_ref), &input.case_ref)?;
                        let mut candidate = previous.clone();
                        candidate.resume_from = Some(resume.clone());
                        candidate.budgets = input.budgets.clone();
                        runtime_execution::resumed_work_checkpoint(&checkpoint, &candidate)?;
                    },
                    Err(error) => return Err(error),
                }
                let submitted = store.submit_runtime_work(&auth, &yai_core_engine::store::lmdb::RuntimeWorkSubmission {
                    request_id: input.submission_ref, tenant_id: previous.tenant_id,
                    case_id: input.case_ref, participant_id: input.participant_ref,
                    attachment_id: previous.attachment_id, journal_path: previous.journal_path,
                    task: previous.task, budgets: input.budgets, resume_from: Some(resume),
                    failpoint: None, now_unix_ms: now_unix_ms()?,
                })?;
                encode_result("case_resume", ExecutionSubmissionResult {
                    created: submitted.created, execution: submitted.item.into(),
                })
            }
            "case.run" => {
                let input: CaseRunInput = decode_input(request)?;
                let state = store.get_case_state_authorized(&auth, &input.case_ref)?;
                let tenant = state.tenant_id.as_deref().ok_or("case_not_visible")?;
                store.resolve_security_context(&auth, tenant)?.require_owner()?;
                // Compatibility output location is selected below presentation,
                // never supplied by the IPC caller. It carries no authority.
                let directory = self.home_path.join("cases")
                    .join(yai_core_engine::context::stable_digest(&input.case_ref));
                std::fs::create_dir_all(&directory).map_err(|e| format!("case_journal_directory:{e}"))?;
                let journal = directory.join("compatibility.jsonl");
                std::fs::OpenOptions::new().create(true).append(true).open(&journal)
                    .map_err(|e| format!("case_journal_open:{e}"))?;
                let submitted = store.submit_runtime_work(&auth, &yai_core_engine::store::lmdb::RuntimeWorkSubmission {
                    request_id: input.submission_ref,
                    tenant_id: tenant.into(), case_id: input.case_ref,
                    participant_id: input.participant_ref, attachment_id: input.resource_ref,
                    journal_path: journal.display().to_string(), task: input.task, budgets: input.budgets,
                    resume_from: None, failpoint: None, now_unix_ms: now_unix_ms()?,
                })?;
                encode_result("case_run", ExecutionSubmissionResult {
                    created: submitted.created,
                    execution: submitted.item.into(),
                })
            }
            "case.create" => {
                let input: CaseCreateInput = decode_input(request)?;
                encode_result(
                    "case_create",
                    store.create_tenant_case(&auth, &input.tenant_id, &input.case_ref)?,
                )
            }
            "case.cancel" => {
                let input: CaseTerminalInput = decode_input(request)?;
                encode_result(
                    "case_cancel",
                    store.cancel_tenant_case(&auth, &input.case_ref, &input.reason)?,
                )
            }
            "case.close" => {
                let input: CaseTerminalInput = decode_input(request)?;
                encode_result(
                    "case_close",
                    store.close_tenant_case(&auth, &input.case_ref, &input.reason)?,
                )
            }
            "material.read" => {
                let case_ref = input_case_ref(request)?;
                material_read(
                    &store,
                    &auth,
                    &self.home_path,
                    case_ref,
                    request
                        .input
                        .get("source_ref")
                        .and_then(Value::as_str)
                        .ok_or("source_ref_invalid")?,
                    request.input.get("revision_ref").and_then(Value::as_str),
                    request
                        .input
                        .get("path")
                        .and_then(Value::as_str)
                        .ok_or("material_path_invalid")?,
                    request
                        .input
                        .get("expected_generation")
                        .and_then(Value::as_u64),
                )
            }
            "decision.frontier.prepare" => {
                let input: DecisionFrontierPrepareInput = decode_input(request)?;
                let frontier_request = CognitiveDecisionFrontierRequest::new(
                    &input.working_state,
                    input.max_candidates,
                )?;
                let content = ConversationContentStore::open_existing(&self.home_path).ok();
                let qualification = store.derive_cognitive_decision_frontier_authorized(
                    &auth,
                    &input.working_state,
                    frontier_request,
                    content.as_ref(),
                )?;
                serde_json::to_value(qualification)
                    .map_err(|error| format!("decision_frontier_encode:{error}"))
            }
            "decision.request.prepare" => {
                let input: DecisionRequestPrepareInput = decode_input(request)?;
                let content = ConversationContentStore::open_existing(&self.home_path).ok();
                let prepared = store.prepare_cognitive_decision_request_from_frontier_authorized(
                    &auth,
                    &input.working_state,
                    &input.frontier,
                    input.decision_kind,
                    input.budget,
                    content.as_ref(),
                )?;
                serde_json::to_value(prepared)
                    .map_err(|error| format!("decision_request_encode:{error}"))
            }
            "semantic.fast_search.prepare" => {
                let input: fast_search::FastSearchPrepareInput = decode_input(request)?;
                let prepared = fast_search::prepare(&self.home_path, &auth, &store, input)?;
                serde_json::to_value(prepared)
                    .map_err(|error| format!("fast_search_prepare_encode:{error}"))
            }
            "semantic.recall" => {
                let input: RecallExecuteInput = decode_input(request)?;
                let content = ConversationContentStore::open_existing(&self.home_path).ok();
                let recalled =
                    store.recall_trace_authorized(&auth, input.request, content.as_ref())?;
                serde_json::to_value(recalled)
                    .map_err(|error| format!("semantic_recall_encode:{error}"))
            }
            "semantic.working_state.compile" => {
                let input: WorkingStateCompileInput = decode_input(request)?;
                let content = ConversationContentStore::open_existing(&self.home_path).ok();
                let qualified = if input.pageable {
                    store.compile_pageable_working_state_authorized(
                        &auth,
                        input.request,
                        content.as_ref(),
                    )?
                } else {
                    store.compile_working_state_authorized(
                        &auth,
                        input.request,
                        content.as_ref(),
                    )?
                };
                serde_json::to_value(qualified)
                    .map_err(|error| format!("working_state_compile_encode:{error}"))
            }
            "semantic.working_state.refresh" => {
                let input: WorkingStateRefreshInput = decode_input(request)?;
                let content = ConversationContentStore::open_existing(&self.home_path).ok();
                let refreshed = store.refresh_working_state_authorized(
                    &auth,
                    &input.working_state,
                    input.request,
                    content.as_ref(),
                )?;
                serde_json::to_value(refreshed)
                    .map_err(|error| format!("working_state_refresh_encode:{error}"))
            }
            "semantic.working_state.page" => {
                let input: WorkingStatePageInput = decode_input(request)?;
                let content = ConversationContentStore::open_existing(&self.home_path).ok();
                let page = store.page_working_state_authorized(
                    &auth,
                    &input.working_state,
                    input.request,
                    content.as_ref(),
                )?;
                serde_json::to_value(page)
                    .map_err(|error| format!("working_state_page_encode:{error}"))
            }
            "semantic.ambient_refresh.assess" => {
                let input: AmbientRefreshAssessInput = decode_input(request)?;
                let content = ConversationContentStore::open_existing(&self.home_path).ok();
                let refresh = store.refresh_active_semantic_consumer_authorized(
                    &auth,
                    &input.working_state,
                    input.request,
                    content.as_ref(),
                )?;
                serde_json::to_value(refresh)
                    .map_err(|error| format!("ambient_refresh_encode:{error}"))
            }
            "workflow.define" => {
                let input: WorkflowDefineInput = decode_input(request)?;
                encode_result(
                    "workflow_define",
                    store.define_workflow(&auth, input.definition, now_unix_ms()?)?,
                )
            }
            "workflow.bind" => {
                let input: WorkflowBindInput = decode_input(request)?;
                encode_result(
                    "workflow_bind",
                    store.bind_case_workflow_composed(
                        &auth,
                        &input.case_ref,
                        &input.definition_ref,
                        input.executor_bindings,
                        input.resource_bindings,
                        input.case_bindings,
                        now_unix_ms()?,
                    )?,
                )
            }
            "workflow.input.record" => {
                let input: WorkflowInputRecordInput = decode_input(request)?;
                encode_result(
                    "workflow_input",
                    store.record_workflow_human_input(
                        &auth,
                        &input.case_ref,
                        &input.node_ref,
                        &input.value,
                        now_unix_ms()?,
                    )?,
                )
            }
            "workflow.patch.propose" => {
                let input: WorkflowPatchProposeInput = decode_input(request)?;
                encode_result(
                    "workflow_patch_propose",
                    store.propose_workflow_plan_patch_human(
                        &auth,
                        &input.case_ref,
                        input.patch,
                        now_unix_ms()?,
                    )?,
                )
            }
            "workflow.patch.adopt" => {
                let input: WorkflowPatchAdoptInput = decode_input(request)?;
                encode_result(
                    "workflow_patch_adopt",
                    store.adopt_workflow_plan_patch(
                        &auth,
                        &input.case_ref,
                        &input.patch_ref,
                        now_unix_ms()?,
                    )?,
                )
            }
            "handoff.pending" => {
                let input: HandoffPendingInput = decode_input(request)?;
                let state = store.get_case_state_authorized(&auth, &input.case_ref)?;
                let offers = store.list_pending_case_handoffs_authorized(&auth, &input.case_ref)?;
                if store.get_case_state_authorized(&auth, &input.case_ref)?.generation != state.generation {
                    return Err("handoff_observation_stale".into());
                }
                encode_result("handoff_pending", HandoffPendingProjection {
                    schema: "yai.handoff_pending_projection.v1".into(), case_ref: input.case_ref,
                    generation: state.generation, offers,
                    scope: "pending_incoming_from_at_most_1024_authorized_tenant_cases".into(),
                })
            }
            "handoff.inspect" => {
                let input: HandoffInspectInput = decode_input(request)?;
                let state = store.get_case_state_authorized(&auth, &input.case_ref)?;
                let local_offer = state.handoff_offers.iter().find(|v| v.handoff_id == input.handoff_ref).cloned();
                let acceptance = state.handoff_acceptances.iter().find(|v| v.handoff_id == input.handoff_ref).cloned();
                let decline = state.handoff_declines.iter().find(|v| v.handoff_id == input.handoff_ref).cloned();
                let result = state.handoff_results.iter().find(|v| v.handoff_id == input.handoff_ref).cloned();
                let reconciliation = state.handoff_reconciliations.iter().find(|v| v.handoff_id == input.handoff_ref).cloned();
                let source = acceptance.as_ref().map(|v| &v.source_case_id)
                    .or_else(|| decline.as_ref().map(|v| &v.source_case_id))
                    .or_else(|| result.as_ref().map(|v| &v.source_case_id));
                let offer = if let Some(offer) = local_offer { offer }
                    else if let Some(source) = source {
                        store.get_case_state_authorized(&auth, source)?.handoff_offers.into_iter()
                            .find(|v| v.handoff_id == input.handoff_ref).ok_or("handoff_not_visible")?
                    } else {
                        store.list_pending_case_handoffs_authorized(&auth, &input.case_ref)?.into_iter()
                            .find(|v| v.handoff_id == input.handoff_ref).ok_or("handoff_not_visible")?
                    };
                if store.get_case_state_authorized(&auth, &input.case_ref)?.generation != state.generation {
                    return Err("handoff_observation_stale".into());
                }
                encode_result("handoff_inspect", HandoffInspection {
                    schema: "yai.handoff_inspection.v1".into(), case_ref: input.case_ref,
                    generation: state.generation, offer, acceptance, decline, result, reconciliation,
                })
            }
            "handoff.offer" => {
                let input: HandoffOfferInput = decode_input(request)?;
                encode_result(
                    "handoff_offer",
                    store.offer_case_handoff(
                        &auth,
                        &input.source_case_ref,
                        &input.target_case_ref,
                        input.request,
                        input.required_target_roles,
                        now_unix_ms()?,
                    )?,
                )
            }
            "handoff.accept" => {
                let input: HandoffAcceptInput = decode_input(request)?;
                encode_result(
                    "handoff_accept",
                    store.accept_case_handoff(
                        &auth,
                        &input.target_case_ref,
                        &input.source_case_ref,
                        &input.handoff_ref,
                        &input.participant_ref,
                        now_unix_ms()?,
                    )?,
                )
            }
            "handoff.decline" => {
                let input: HandoffDeclineInput = decode_input(request)?;
                encode_result(
                    "handoff_decline",
                    store.decline_case_handoff(
                        &auth,
                        &input.target_case_ref,
                        &input.source_case_ref,
                        &input.handoff_ref,
                        &input.participant_ref,
                        &input.reason,
                        now_unix_ms()?,
                    )?,
                )
            }
            "handoff.result.record" => {
                let input: HandoffResultInput = decode_input(request)?;
                encode_result(
                    "handoff_result",
                    store.record_case_handoff_result(
                        &auth,
                        &input.target_case_ref,
                        &input.handoff_ref,
                        input.outcome,
                        input.result,
                        input.evidence_refs,
                        &input.participant_ref,
                        now_unix_ms()?,
                    )?,
                )
            }
            "handoff.reconcile" => {
                let input: HandoffReconcileInput = decode_input(request)?;
                encode_result(
                    "handoff_reconcile",
                    store.reconcile_case_handoff(
                        &auth,
                        &input.source_case_ref,
                        &input.handoff_ref,
                        now_unix_ms()?,
                    )?,
                )
            }
            "policy.ingest" => {
                let input: PolicyIngestInput = decode_input(request)?;
                let context = store.resolve_security_context(&auth, &input.tenant_id)?;
                context.require_owner()?;
                let tenant = store
                    .get_tenant(&input.tenant_id)?
                    .ok_or_else(|| "tenant_not_visible".to_string())?;
                let compilation = scope_policy_compilation(
                    &compile_policy_source(&input.source_bytes)?,
                    &input.tenant_id,
                    &tenant.organization_ref,
                )?;
                encode_result(
                    "policy_ingest",
                    store.ingest_tenant_policy_compilation(
                        &auth,
                        &input.tenant_id,
                        &compilation,
                    )?,
                )
            }
            "policy.validate" => {
                let input: PolicyArtifactLifecycleInput = decode_input(request)?;
                encode_result(
                    "policy_validate",
                    store.validate_tenant_policy_artifact(
                        &auth,
                        &input.artifact_ref,
                        &input.reason,
                    )?,
                )
            }
            "policy.publish" => {
                let input: PolicyArtifactLifecycleInput = decode_input(request)?;
                encode_result(
                    "policy_publish",
                    store.publish_tenant_policy_artifact(
                        &auth,
                        &input.artifact_ref,
                        &input.reason,
                    )?,
                )
            }
            "policy.retire" => {
                let input: PolicyArtifactLifecycleInput = decode_input(request)?;
                encode_result(
                    "policy_retire",
                    store.retire_tenant_policy_artifact(
                        &auth,
                        &input.artifact_ref,
                        &input.reason,
                    )?,
                )
            }
            "policy.revoke" => {
                let input: PolicyArtifactLifecycleInput = decode_input(request)?;
                encode_result(
                    "policy_revoke",
                    store.revoke_tenant_policy_artifact(
                        &auth,
                        &input.artifact_ref,
                        &input.reason,
                    )?,
                )
            }
            "policy.case.bind" => {
                let input: CasePolicyBindInput = decode_input(request)?;
                encode_result(
                    "policy_case_bind",
                    store.bind_tenant_case_policy(
                        &auth,
                        &input.case_ref,
                        &input.artifact_ref,
                        input.expected_generation,
                        &input.reason,
                    )?,
                )
            }
            "policy.case.replace" => {
                let input: CasePolicyReplaceInput = decode_input(request)?;
                encode_result(
                    "policy_case_replace",
                    store.replace_tenant_case_policy(
                        &auth,
                        &input.case_ref,
                        &input.prior_binding_ref,
                        &input.artifact_ref,
                        input.expected_generation,
                        &input.reason,
                    )?,
                )
            }
            "policy.case.unbind" => {
                let input: CasePolicyUnbindInput = decode_input(request)?;
                encode_result(
                    "policy_case_unbind",
                    store.unbind_tenant_case_policy(
                        &auth,
                        &input.case_ref,
                        &input.binding_ref,
                        input.expected_generation,
                        &input.reason,
                    )?,
                )
            }
            "participant.principal.link" => {
                let input: ParticipantPrincipalLinkInput = decode_input(request)?;
                let state = store
                    .get_case_state(&input.case_ref)?
                    .ok_or_else(|| "case_not_visible".to_string())?;
                let tenant_id = state.tenant_id.clone().ok_or_else(|| {
                    "legacy_unscoped_case_cannot_accept_principal_link".to_string()
                })?;
                store
                    .resolve_security_context(&auth, &tenant_id)?
                    .require_owner()?;
                let principal_ref = if input.principal_ref == "self" {
                    auth.projected_principal_id()
                } else {
                    input.principal_ref
                };
                if state.principal_participant_links.iter().any(|existing| {
                    existing.principal_id == principal_ref
                        && existing.participant_id == input.participant_ref
                        && existing.tenant_id == tenant_id
                }) {
                    return encode_result(
                        "participant_principal_link",
                        CaseStateMutationResult::unchanged(state),
                    );
                }
                let link = PrincipalParticipantLink::new(
                    &input.case_ref,
                    &tenant_id,
                    &principal_ref,
                    &input.participant_ref,
                    &auth.projected_principal_id(),
                    now_unix_ms()?,
                )?;
                let mut pending = PendingTransition::new(
                    format!("transition:{}", link.link_id),
                    &input.case_ref,
                    state.generation,
                    TransitionSource {
                        component: "yai.application.participant".to_string(),
                        participant_id: None,
                        principal_id: Some(auth.projected_principal_id()),
                        source_ref: Some(link.link_id.clone()),
                    },
                    TransitionPayload::ParticipantPrincipalLinked { link: link.clone() },
                );
                pending.causal_refs = vec![link.principal_id.clone(), link.participant_id.clone()];
                encode_result(
                    "participant_principal_link",
                    CaseStateMutationResult::committed(
                        store.commit_secured_transition(&auth, &tenant_id, pending, true)?,
                    ),
                )
            }
            "participant.role.add" => {
                let input: ParticipantRoleAddInput = decode_input(request)?;
                validate_identifier("participant_role", &input.role)?;
                let state = store
                    .get_case_state(&input.case_ref)?
                    .ok_or_else(|| "case_not_visible".to_string())?;
                let tenant_id = state.tenant_id.clone().ok_or_else(|| {
                    "legacy_unscoped_case_cannot_accept_new_participant".to_string()
                })?;
                store
                    .resolve_security_context(&auth, &tenant_id)?
                    .require_owner()?;
                if state.participants.iter().any(|participant| {
                    participant.participant_id == input.participant_ref
                        && participant.roles.contains(&input.role)
                }) {
                    return encode_result(
                        "participant_role_add",
                        CaseStateMutationResult::unchanged(state),
                    );
                }
                let mut pending = PendingTransition::new(
                    format!(
                        "transition:participant-role:{}:{}:{}",
                        canonical_id_component(&input.case_ref),
                        canonical_id_component(&input.participant_ref),
                        canonical_id_component(&input.role)
                    ),
                    &input.case_ref,
                    state.generation,
                    TransitionSource {
                        component: "yai.application.participant".to_string(),
                        participant_id: None,
                        principal_id: Some(auth.projected_principal_id()),
                        source_ref: Some(format!(
                            "participant-role:{}:{}",
                            input.participant_ref, input.role
                        )),
                    },
                    TransitionPayload::ParticipantBound {
                        participant_id: input.participant_ref.clone(),
                        role: input.role,
                    },
                );
                pending.causal_refs = vec![input.participant_ref];
                encode_result(
                    "participant_role_add",
                    CaseStateMutationResult::committed(
                        store.commit_secured_transition(&auth, &tenant_id, pending, true)?,
                    ),
                )
            }
            "participant.view.admit" => {
                let input: ParticipantViewAdmitInput = decode_input(request)?;
                if input.consumer != "model" || input.view_kind != "model_context" {
                    return Err("participant_view_contract_unsupported".to_string());
                }
                let state = store
                    .get_case_state(&input.case_ref)?
                    .ok_or_else(|| "case_not_visible".to_string())?;
                let tenant_id = state.tenant_id.clone().ok_or_else(|| {
                    "legacy_unscoped_case_cannot_admit_participant_view".to_string()
                })?;
                store
                    .resolve_security_context(&auth, &tenant_id)?
                    .require_owner()?;
                let participant = state
                    .participants
                    .iter()
                    .find(|participant| participant.participant_id == input.participant_ref)
                    .ok_or_else(|| "participant_view_requires_bound_participant".to_string())?;
                if participant.admitted_views.iter().any(|view| {
                    view.consumer == input.consumer && view.view_kind == input.view_kind
                }) {
                    return encode_result(
                        "participant_view_admit",
                        CaseStateMutationResult::unchanged(state),
                    );
                }
                let mut pending = PendingTransition::new(
                    format!(
                        "transition:participant-view:{}:{}:{}:{}",
                        canonical_id_component(&input.case_ref),
                        canonical_id_component(&input.participant_ref),
                        canonical_id_component(&input.consumer),
                        canonical_id_component(&input.view_kind)
                    ),
                    &input.case_ref,
                    state.generation,
                    TransitionSource {
                        component: "yai.application.participant".to_string(),
                        participant_id: None,
                        principal_id: Some(auth.projected_principal_id()),
                        source_ref: Some(format!(
                            "participant-view:{}:{}:{}",
                            input.participant_ref, input.consumer, input.view_kind
                        )),
                    },
                    TransitionPayload::ParticipantAdmitted {
                        participant_id: input.participant_ref.clone(),
                        consumer: input.consumer,
                        view_kind: input.view_kind,
                    },
                );
                pending.causal_refs = vec![input.participant_ref];
                encode_result(
                    "participant_view_admit",
                    CaseStateMutationResult::committed(
                        store.commit_secured_transition(&auth, &tenant_id, pending, true)?,
                    ),
                )
            }
            "provider.inventory" => {
                let input: TenantGetInput = decode_input(request)?;
                let targets = store.list_provider_targets_authorized(&auth, &input.tenant_id)?;
                let total = targets.len();
                let targets = targets.into_iter().take(128)
                    .map(|target| provider_target_projection(&store, &auth, target)).collect::<Vec<_>>();
                Ok(json!({"tenant_ref":input.tenant_id, "targets":targets,
                    "total_visible_targets":total, "limit":128, "omitted":total.saturating_sub(128),
                    "case_usage":"not_projected"}))
            }
            "provider.models" => {
                let input: ProviderModelDiscoveryInput = decode_input(request)?;
                let (connection, target_ref) = match input {
                    ProviderModelDiscoveryInput::Connection(input) => (input, None),
                    ProviderModelDiscoveryInput::Registered(input) => {
                        store.resolve_security_context(&auth, &input.tenant_id)?.require_owner()?;
                        let target = store.list_provider_targets_authorized(&auth, &input.tenant_id)?
                            .into_iter().find(|target| target.target_id == input.target_ref)
                            .ok_or("provider_target_not_found")?;
                        (ProviderModelsInput { tenant_id: input.tenant_id, endpoint: target.endpoint,
                            locality: target.locality, credential_ref: target.credential_ref }, Some(input.target_ref))
                    }
                };
                store.resolve_security_context(&auth, &connection.tenant_id)?.require_owner()?;
                let models = provider_execution::discover_provider_models(&connection.endpoint, &connection.locality,
                    &connection.credential_ref, |key| provider_execution::credential_from_profile(&self.home_path, key))?;
                encode_result("provider_models", json!({ "models": models, "scope": "currently_exposed", "authority": "provider_metadata_only", "target_ref":target_ref, "observed_at_unix_ms":now_unix_ms()? }))
            }
            "provider.register" => {
                let input: ProviderRegisterInput = decode_input(request)?;
                let target = ProviderTargetInput {
                    tenant_id: input.tenant_id,
                    provider_key: input.provider_key,
                    adapter: input.adapter,
                    endpoint: input.endpoint,
                    model_id: input.model_id,
                    credential_ref: input.credential_ref,
                    locality: input.locality,
                    extension_adapter_id: input.extension_adapter_id,
                    created_by_principal_id: auth.projected_principal_id(),
                    created_at_unix_ms: now_unix_ms()?,
                };
                encode_result(
                    "provider_register",
                    store.register_provider_target_authorized(&auth, target)?,
                )
            }
            "provider.probe" => {
                let input: ProviderProbeInput = decode_input(request)?;
                encode_result("provider_probe", provider_execution::probes::submit(&self.home_path, &store, &auth,
                    yai_core_engine::provider_governance::ProviderProbeRequest {
                        target_id: input.target_ref, submission_ref: input.submission_ref,
                        embedding: input.embedding, realization_shapes: input.realization_shapes,
                        qualify: input.qualify, valid_for_ms: input.valid_for_ms,
                    })?)
            }
            "provider.probe.get" => {
                let input: ProviderProbeGetInput = decode_input(request)?;
                encode_result("provider_probe", provider_execution::probes::observe(&self.home_path, &store, &auth,
                    &input.target_ref, &input.submission_ref, false)?)
            }
            "provider.qualify" => {
                let input: ProviderQualifyInput = decode_input(request)?;
                encode_result(
                    "provider_qualify",
                    store.qualify_provider_target_authorized(
                        &auth,
                        &input.target_ref,
                        input.evidence,
                        &input.suite_ref,
                        input.valid_until_unix_ms,
                    )?,
                )
            }
            "provider.suitability.record" => {
                let input: ProviderSuitabilityRecordInput = decode_input(request)?;
                encode_result("provider_suitability", store.record_semantic_suitability_evidence_authorized(
                    &auth, &input.target_ref, input.capability,
                    yai_core_engine::cognitive::SemanticEvidencePosture::OperatorAttested,
                    &input.suite_ref, &input.run_ref, input.evidence_refs,
                    "authenticated_operator_attestation")?)
            }
            "provider.trust.set" => {
                let input: ProviderTrustInput = decode_input(request)?;
                encode_result(
                    "provider_trust",
                    store.set_provider_trust_authorized(
                        &auth,
                        &input.target_ref,
                        input.posture,
                        now_unix_ms()?,
                    )?,
                )
            }
            "provider.case.bind" => {
                let input: ProviderCaseBindInput = decode_input(request)?;
                encode_result(
                    "provider_case_bind",
                    store.bind_case_provider_targets_authorized(
                        &auth,
                        &input.case_ref,
                        &input.participant_ref,
                        input.ordered_target_refs,
                        input.failover_policy,
                        input.max_attempts_per_turn,
                    )?,
                )
            }
            "resource.import" => {
                let input: ResourceImportInput = decode_input(request)?;
                encode_result("resource_import", resource_execution::import_definition_result(
                    &store, &auth, &input.case_ref, input.definition,
                )?)
            }
            "resource.attach" => {
                let input: ResourceAttachInput = decode_input(request)?;
                if input.write_prefix.is_some() != input.max_write_bytes.is_some() {
                    return Err("resource_write_envelope_incomplete".to_string());
                }
                if input.access.configuration_digest != input.binding.digest() {
                    return Err("resource_access_configuration_digest_mismatch".to_string());
                }
                encode_result("resource_attach", resource_execution::attach(
                    &store, &auth, &input.binding, input.access,
                    &input.policy_owner_participant_ref,
                    input.write_prefix.as_deref().zip(input.max_write_bytes),
                    input.review_requirement,
                )?)
            }
            "resource.attach_process" => {
                let input: ResourceAttachProcessInput = decode_input(request)?;
                let state = store.get_case_state_authorized(&auth, &input.case_ref)?;
                let tenant = state.tenant_id.as_deref().ok_or("resource_attachment_requires_tenant")?;
                store.resolve_security_context(&auth, tenant)?.require_owner()?;
                if input.pid == std::process::id() {
                    return Err("cannot_attach_current_yai_process".into());
                }
                let binding = LocalProcessBinding::capture(&input.case_ref, &input.attachment_ref, input.pid)?;
                let mut actions = input.actions;
                actions.sort_by_key(ProcessSignalAction::as_str);
                actions.dedup();
                let attachment = ResourceAttachmentState {
                    attachment_id: input.attachment_ref.clone(),
                    kind: yai_core_engine::transition::ResourceKind::Process,
                    allowed_write_prefix: String::new(),
                    max_write_bytes: 0,
                    policy_id: format!("policy:process-signal:{}", input.attachment_ref),
                    policy_owner_participant_id: input.policy_owner_participant_ref.clone(),
                    review_requirement: input.review_requirement,
                    process_signal_actions: actions,
                    access: None,
                };
                attachment.validate()?;
                if let Some(existing) = state.resources.iter().find(|r| r.attachment_id == input.attachment_ref) {
                    if existing != &attachment || store.get_local_process_binding(&input.case_ref, &input.attachment_ref)?.as_ref() != Some(&binding) {
                        return Err("process_attachment_is_immutable".into());
                    }
                    return encode_result("resource_attach_process", CaseStateMutationResult::unchanged(state));
                }
                let mut pending = PendingTransition::new(
                    format!("transition:process-resource-attached:{}:{}", canonical_id_component(&input.case_ref), input.attachment_ref),
                    &input.case_ref,
                    state.generation,
                    TransitionSource {
                        component: "yai.application.resource".into(),
                        participant_id: None,
                        principal_id: Some(auth.projected_principal_id()),
                        source_ref: Some(format!("process-resource-attached:{}", input.attachment_ref)),
                    },
                    TransitionPayload::ResourceAttached { attachment: attachment.clone() },
                );
                pending.scope = Some(TransitionScope {
                    case_id: input.case_ref,
                    participant_refs: vec![input.policy_owner_participant_ref.clone()],
                    resource_refs: vec![input.attachment_ref],
                    policy_refs: vec![attachment.policy_id],
                });
                pending.causal_refs = vec![input.policy_owner_participant_ref];
                encode_result("resource_attach_process", CaseStateMutationResult::committed(
                    store.commit_tenant_process_attachment(&auth, tenant, pending, &binding)?
                ))
            }
            "source.acquire" | "source.resume" => {
                let (input, resume) = if request.operation_ref == "source.resume" {
                    let value: SourceResumeInput = decode_input(request)?;
                    (SourceAcquireInput { case_ref: value.case_ref, participant_ref: value.participant_ref,
                        source_ref: value.source_ref, attempt: value.attempt, expected_generation: value.expected_generation },
                        Some(value.previous_progress_ref))
                } else {
                    (decode_input::<SourceAcquireInput>(request)?, None)
                };
                let (_, source) = store.case_source_authorized(&auth, &input.case_ref, &input.source_ref)?;
                if source.declaration.source_id != input.source_ref
                    || source.declaration.participant_id != input.participant_ref || input.attempt == 0 {
                    return Err("source_not_visible".into());
                }
                let repeated = if resume.is_none() {
                    match store.observe_source_attempt_authorized(&auth, &input.case_ref,
                        &input.participant_ref, &input.source_ref, input.attempt) {
                        Ok(_) => true,
                        Err(error) if error == "execution_not_visible" => false,
                        Err(error) => return Err(error),
                    }
                } else {
                    source.progress.as_ref().is_some_and(|p| p.attempt == input.attempt
                        && p.phase == SourcePhase::Acquiring
                        && p.previous_progress_id.as_ref() == resume.as_ref())
                };
                let carrier = if repeated { None } else { Some(resource_execution::source::acquire_carrier(
                    &self.home_path, &store, &auth, &input.case_ref, &input.source_ref, input.attempt)?) };
                let created = if repeated { false } else if let Some(prior) = resume {
                    store.resume_source_attempt_authorized(&auth, &input.case_ref, &input.participant_ref,
                        &input.source_ref, input.attempt, input.expected_generation, &prior)?
                } else {
                    store.begin_source_attempt_authorized(
                        &auth, &input.case_ref, &input.participant_ref, &input.source_ref,
                        input.attempt, input.expected_generation,
                    )?
                };
                let advancement = if !created {
                    SourceAdvancementPosture::ExistingAttempt
                } else if resource_execution::source::advance_admitted(
                    &self.home_path, &store, &auth, &input.case_ref, &input.source_ref, input.attempt,
                    carrier.as_ref().ok_or("source_carrier_missing")?,
                ).is_ok() {
                    SourceAdvancementPosture::Returned
                } else {
                    SourceAdvancementPosture::ObservationRequired
                };
                drop(carrier);
                // Requalify observation even after dispatch: the authenticated
                // caller cannot retain disclosure merely by having submitted.
                let execution = resource_execution::source::observe_execution(
                    &self.home_path, &store, &auth, &input.case_ref,
                    &input.participant_ref, &input.source_ref, input.attempt)?;
                encode_result("source_acquire", SourceAcquisitionSubmissionResult {
                    schema: "yai.source_acquisition_submission_result.v1".into(), created, advancement,
                    execution,
                })
            }
            "source.declare" => {
                let input: SourceDeclareInput = decode_input(request)?;
                let state = store.get_case_state_authorized(&auth, &input.case_ref)?;
                let tenant_id = state
                    .tenant_id
                    .as_deref()
                    .ok_or_else(|| "source_requires_tenant".to_string())?;
                store
                    .resolve_security_context(&auth, tenant_id)?
                    .require_owner()?;
                let binding = store
                    .get_local_access_binding(&input.case_ref, &input.resource_ref)?
                    .ok_or_else(|| "source_resource_not_attached".to_string())?;
                let declaration = CaseSourceDeclaration {
                    schema: SOURCE_DECLARATION_SCHEMA.to_string(),
                    source_id: String::new(),
                    case_id: input.case_ref,
                    perimeter: input.perimeter,
                    logical_name: input.logical_name,
                    participant_id: input.participant_ref,
                    declared_by_principal_id: auth.projected_principal_id(),
                    resource_attachment_id: input.resource_ref,
                    configuration_digest: binding.digest(),
                    roles: input.roles,
                    action: input.action,
                    bootstrap_policy: input.bootstrap_policy,
                    media_type: input.media_type,
                }
                .seal()?;
                let prior_generation = state.generation;
                let transition_ref = format!("transition:{}", declaration.source_id);
                let state = store.declare_case_source(&auth, declaration)?;
                encode_result(
                    "source_declare",
                    CaseStateMutationResult {
                        changed: state.generation != prior_generation,
                        transition_ref: (state.generation != prior_generation)
                            .then_some(transition_ref),
                        state,
                    },
                )
            }
            "source.publish" => {
                let input: SourcePublishInput = decode_input(request)?;
                store.publish_case_source_policy_authorized(
                    &auth,
                    &input.case_ref,
                    &input.source_ref,
                    &input.reason,
                )?;
                encode_result(
                    "source_publish",
                    SourcePolicyPublicationResult {
                        state: store.get_case_state_authorized(&auth, &input.case_ref)?,
                        normative: store.case_policy_status(&input.case_ref)?,
                    },
                )
            }
            "source.revoke" => {
                let input: SourceRevokeInput = decode_input(request)?;
                let (_, source) =
                    store.case_source_authorized(&auth, &input.case_ref, &input.source_ref)?;
                if source
                    .progress
                    .as_ref()
                    .is_some_and(|progress| progress.phase == SourcePhase::Revoked)
                {
                    return encode_result(
                        "source_revoke",
                        CaseStateMutationResult::unchanged(
                            store.get_case_state_authorized(&auth, &input.case_ref)?,
                        ),
                    );
                }
                let progress = SourceProgress {
                    schema: SOURCE_PROGRESS_SCHEMA.to_string(),
                    progress_id: String::new(),
                    source_id: source.declaration.source_id,
                    previous_progress_id: source
                        .progress
                        .as_ref()
                        .map(|item| item.progress_id.clone()),
                    attempt: source.progress.as_ref().map_or(1, |item| item.attempt),
                    phase: SourcePhase::Revoked,
                    revision: None,
                    decision_ref: None,
                    detail: input.reason,
                }
                .seal()?;
                let transition_ref = format!("transition:{}", progress.progress_id);
                encode_result(
                    "source_revoke",
                    CaseStateMutationResult {
                        changed: true,
                        transition_ref: Some(transition_ref),
                        state: store.progress_case_source(
                            &auth,
                            &input.case_ref,
                            &input.source_ref,
                            progress,
                        )?,
                    },
                )
            }
            "cognitive.binding.set" => {
                let input: CognitiveBindInput = decode_input(request)?;
                let candidates = input
                    .candidates
                    .into_iter()
                    .map(|candidate| (candidate.target_ref, candidate.semantic_evidence_ref))
                    .collect();
                encode_result(
                    "cognitive_binding",
                    store.bind_case_cognitive_candidates_authorized(
                        &auth,
                        &input.case_ref,
                        &input.participant_ref,
                        input.role,
                        input.capability,
                        candidates,
                        input.replace,
                    )?,
                )
            }
            "cognitive.plan" => {
                let input: CognitivePlanInput = decode_input(request)?;
                encode_result(
                    "cognitive_plan",
                    store.plan_case_cognitive_execution_for_shape_authorized(
                        &auth,
                        &input.case_ref,
                        &input.participant_ref,
                        &input.requirement,
                        input.realization_shape.as_ref(),
                    )?,
                )
            }
            "review.approve" => {
                let input: ReviewResolveInput = decode_input(request)?;
                encode_result(
                    "review_approve",
                    resolve_review_action(&store, &auth, input, ReviewActionKind::Approve)?,
                )
            }
            "review.defer" => {
                let input: ReviewResolveInput = decode_input(request)?;
                encode_result(
                    "review_defer",
                    resolve_review_action(&store, &auth, input, ReviewActionKind::Defer)?,
                )
            }
            "review.deny" => {
                let input: ReviewResolveInput = decode_input(request)?;
                encode_result(
                    "review_deny",
                    resolve_review_action(&store, &auth, input, ReviewActionKind::Deny)?,
                )
            }
            "events.subscribe" | "events.resume" => Ok(json!({
                "transport": "tauri-event-bridge",
                "stream_state": "subscribed",
                "resync": "case.summary",
                "resume_posture": "snapshot_required_after_attach_or_gap",
                "heartbeat_interval_ms": 1000,
                "resume_token": request.input.get("resume_token").and_then(Value::as_str)
            })),
            "events.heartbeat" => {
                let case_ref = input_case_ref(request)?;
                let state = store.get_case_state_authorized(&auth, case_ref)?;
                Ok(json!({
                    "transport": "tauri-event-bridge",
                    "stream_state": "live",
                    "case_ref": state.case_id,
                    "generation": state.generation,
                    "cursor": format!("{}:{}:heartbeat", state.case_id, state.generation),
                    "resync": "case.summary"
                }))
            }
            _ => unreachable!("operation references were validated before opening the store"),
        }
    }
}

fn default_yai_home() -> PathBuf {
    std::env::var_os("YAI_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".yai")
        })
}

fn input_case_ref(request: &OperationRequest) -> Result<&str, String> {
    request
        .input
        .get("case_ref")
        .and_then(Value::as_str)
        .filter(|value| value.starts_with("case:"))
        .ok_or_else(|| "case_ref_invalid".to_string())
}

fn decode_input<T: for<'de> Deserialize<'de>>(request: &OperationRequest) -> Result<T, String> {
    serde_json::from_value(request.input.clone())
        .map_err(|error| format!("operation_input_invalid:{error}"))
}

fn encode_result<T: Serialize>(label: &str, value: T) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|error| format!("{label}_encode:{error}"))
}

fn runtime_execution_observation(home: &std::path::Path, item: yai_core_engine::store::lmdb::RuntimeWorkItem)
    -> Result<RuntimeExecutionObservation, String> {
    let path = runtime_execution::checkpoint_path_for(home, &item.case_id);
    let mut observed = RuntimeExecutionObservation::from(item.clone());
    if path.is_file() {
        let checkpoint = runtime_execution::read_checkpoint_at(&path, &item.case_id)?;
        if checkpoint.work_item_id.as_deref() == Some(item.work_id.as_str()) {
            runtime_execution::validate_checkpoint_work_identity(&checkpoint, &item)?;
            observed.runner = Some(RuntimeRunnerObservation { checkpoint_digest: runtime_execution::checkpoint_digest(&checkpoint)?, run_ref:checkpoint.run_id,
                posture:checkpoint.status, stop_requested:checkpoint.stop_requested });
        }
    }
    Ok(observed)
}

fn now_unix_ms() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis().min(u64::MAX as u128) as u64)
        .map_err(|error| format!("system_clock_before_unix_epoch:{error}"))
}

fn canonical_id_component(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, ':' | '-' | '_' | '.') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn validate_identifier(label: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty()
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._:-".contains(character))
    {
        return Err(format!("{label}_identifier_invalid"));
    }
    Ok(())
}

fn review_for(state: &CaseState, review_ref: &str) -> Result<ReviewState, String> {
    state
        .reviews
        .iter()
        .find(|review| review.review_id == review_ref)
        .cloned()
        .ok_or_else(|| "review_not_found".to_string())
}

fn operation_for_review(
    transitions: &[Transition],
    review: &ReviewState,
) -> Result<Operation, String> {
    transitions
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::OperationRecorded { operation }
                if operation.operation_id == review.operation_id =>
            {
                Some(operation.clone())
            }
            _ => None,
        })
        .ok_or_else(|| "review_operation_transition_missing".to_string())
}

fn review_action_by_id(transitions: &[Transition], action_ref: &str) -> Option<ReviewAction> {
    transitions
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::ReviewActionRecorded { action }
                if action.action_id == action_ref =>
            {
                Some(action.clone())
            }
            _ => None,
        })
}

fn resolve_review_action(
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    input: ReviewResolveInput,
    requested: ReviewActionKind,
) -> Result<ReviewResolveResult, String> {
    let initial = store.get_case_state_authorized(auth, &input.case_ref)?;
    let tenant_id = initial
        .tenant_id
        .clone()
        .ok_or_else(|| "legacy_review_is_compatibility_only".to_string())?;
    if initial.lifecycle != CaseLifecycle::Open {
        return Err("review_action_requires_open_case".to_string());
    }
    let principal_id = auth.projected_principal_id();
    let reviewer = initial
        .principal_participant_links
        .iter()
        .find(|link| {
            link.principal_id == principal_id
                && link.tenant_id == tenant_id
                && input
                    .participant_ref
                    .as_deref()
                    .is_none_or(|selected| link.participant_id == selected)
        })
        .map(|link| link.participant_id.clone())
        .ok_or_else(|| "authenticated_principal_participant_link_required".to_string())?;
    let initial_review = review_for(&initial, &input.review_ref)?;
    if initial_review.schema != REVIEW_REQUEST_SCHEMA {
        return Err("legacy_review_is_compatibility_only".to_string());
    }
    if store
        .invalidate_review_if_policy_unusable(&input.case_ref, &input.review_ref)?
        .is_some()
    {
        return Err("review_authority_invalidated".to_string());
    }
    let state = store.get_case_state_authorized(auth, &input.case_ref)?;
    let review = review_for(&state, &input.review_ref)?;
    if !reviewer_is_eligible(&state, &review, &reviewer) {
        return Err("reviewer_not_eligible_for_case_review".to_string());
    }
    let normative = store.case_policy_status(&input.case_ref)?;
    let effective = normative
        .effective_policy
        .as_ref()
        .filter(|_| {
            normative.readiness == NormativeReadiness::Ready
                && normative.validity == PolicyValidityPosture::Valid
        })
        .ok_or_else(|| "review_policy_basis_stale".to_string())?;
    if review.effective_policy_id != effective.effective_policy_id
        || review.effective_policy_digest != effective.semantic_digest
    {
        return Err("review_policy_basis_stale".to_string());
    }
    let transitions = store.list_case_transitions(&input.case_ref)?;
    if !matches!(
        review.status,
        ReviewResolution::Pending | ReviewResolution::Deferred
    ) {
        let same = matches!(
            (&review.status, &requested),
            (ReviewResolution::Approved, ReviewActionKind::Approve)
                | (ReviewResolution::Denied, ReviewActionKind::Deny)
        );
        if !same {
            return Err("review_already_resolved".to_string());
        }
        let action = review
            .latest_action_id
            .as_deref()
            .and_then(|id| review_action_by_id(&transitions, id))
            .ok_or_else(|| "review_action_missing".to_string())?;
        return Ok(ReviewResolveResult {
            review_ref: review.review_id,
            action,
            effective_decision_ref: review.effective_decision_id,
            state,
            external_effect: false,
        });
    }
    if review.status == ReviewResolution::Deferred {
        if let Some(existing) = review
            .latest_action_id
            .as_deref()
            .and_then(|id| review_action_by_id(&transitions, id))
        {
            let reason = input
                .reason
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            if existing.action == requested
                && existing.reviewer_participant_id == reviewer
                && existing.reason == reason
            {
                return Ok(ReviewResolveResult {
                    review_ref: review.review_id,
                    action: existing,
                    effective_decision_ref: review.effective_decision_id,
                    state,
                    external_effect: false,
                });
            }
        }
    }
    let action = build_authenticated_review_action(
        &review,
        &input.case_ref,
        &tenant_id,
        &principal_id,
        &reviewer,
        requested.clone(),
        &input.reason,
        state.generation,
        "kernel_authenticated_principal_participant_link",
    )?;
    let mut pending = PendingTransition::new(
        format!("transition:{}", action.action_id),
        &input.case_ref,
        state.generation,
        TransitionSource {
            component: "yai.application.review".to_string(),
            participant_id: Some(reviewer),
            principal_id: Some(principal_id),
            source_ref: Some(action.action_id.clone()),
        },
        TransitionPayload::ReviewActionRecorded {
            action: action.clone(),
        },
    );
    pending.causal_refs = vec![action.review_id.clone(), action.operation_id.clone()];
    let after_action = store
        .commit_secured_transition(auth, &tenant_id, pending, false)?
        .state;
    let (effective_decision_ref, final_state) = if requested == ReviewActionKind::Defer {
        (None, after_action)
    } else {
        let current_review = review_for(&after_action, &input.review_ref)?;
        let operation = operation_for_review(&transitions, &current_review)?;
        let (decision, commit) = store.derive_and_commit_policy_review_decision(
            &input.case_ref,
            &operation.operation_id,
            &current_review.review_id,
            &action.action_id,
        )?;
        if !matches!(
            decision.outcome,
            DecisionOutcome::Allow | DecisionOutcome::Deny
        ) {
            return Err("review_effective_decision_invalid".to_string());
        }
        (Some(decision.decision_id), commit.state)
    };
    Ok(ReviewResolveResult {
        review_ref: input.review_ref,
        action,
        effective_decision_ref,
        state: final_state,
        external_effect: false,
    })
}

fn material_read(
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    home: &Path,
    case_ref: &str,
    source_ref: &str,
    revision_ref: Option<&str>,
    path: &str,
    expected_generation: Option<u64>,
) -> Result<Value, String> {
    let state = store.get_case_state_authorized(auth, case_ref)?;
    if expected_generation.is_some_and(|expected| expected != state.generation) {
        return Err(format!(
            "stale_generation:expected={}:actual={}",
            expected_generation.unwrap_or_default(),
            state.generation
        ));
    }
    let content = ConversationContentStore::open_existing(home)
        .map_err(|_| "source_backing_unavailable".to_string())?;
    let resolved =
        store.resolve_case_source_authorized(auth, case_ref, source_ref, revision_ref, &content)?;
    let (item, bytes) = resolved
        .items
        .into_iter()
        .find(|(item, _)| item.path == path)
        .ok_or("material_not_found")?;
    let current = store.get_case_state_authorized(auth, case_ref)?;
    if current.generation != state.generation {
        return Err(format!(
            "stale_generation:expected={}:actual={}",
            state.generation, current.generation
        ));
    }
    let media_type = resolved.declaration.media_type;
    let utf8 = String::from_utf8(bytes.clone()).ok();
    Ok(json!({
        "case_ref": case_ref,
        "generation": state.generation,
        "source_ref": resolved.declaration.source_id,
        "revision_ref": resolved.revision.revision_id,
        "path": item.path,
        "digest": item.digest,
        "bytes": item.bytes,
        "media_type": media_type,
        "encoding": if utf8.is_some() { "utf-8" } else { "base64" },
        "content": utf8.unwrap_or_else(|| encode_base64(&bytes))
    }))
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let value = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        output.push(TABLE[((value >> 18) & 63) as usize] as char);
        output.push(TABLE[((value >> 12) & 63) as usize] as char);
        output.push(if chunk.len() > 1 {
            TABLE[((value >> 6) & 63) as usize] as char
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            TABLE[(value & 63) as usize] as char
        } else {
            '='
        });
    }
    output
}

fn source_kind(action: &ResourceAction) -> &'static str {
    match action {
        ResourceAction::Discover { .. } => "discovery",
        ResourceAction::DatabaseQuery { .. } => "database_query",
        ResourceAction::HttpFetch { .. } => "http_fetch",
        _ => "unsupported",
    }
}

/// Produce a human presentation label without changing canonical Case identity.
///
/// Case refs remain the durable lookup key and stay available in technical
/// details. The application boundary owns this bounded fallback until YAI has
/// a qualified persistent display-name contract.
fn case_display_name(case_ref: &str) -> String {
    // First-party operational Case label; canonical lookup identity is unchanged.
    if case_ref == "case:yai-enterprise-launch" {
        return "YAI — Primo rilascio aziendale".into();
    }
    let Some(slug) = case_ref.strip_prefix("case:") else {
        return case_ref.to_string();
    };
    let words = slug
        .split(['-', '_'])
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>();
    if words.is_empty() {
        case_ref.to_string()
    } else {
        words.join(" ")
    }
}

fn case_list(store: &LmdbRecordStore, auth: &AuthenticatedPrincipal) -> Result<Value, String> {
    let mut cases = Vec::new();
    for state in store.list_case_states_authorized(auth, None, 1024)? {
        let history = store.list_case_transitions(&state.case_id)?;
        let updated_at_unix_ms = history
            .last()
            .map(|transition| transition.committed_at_unix_ms);
        cases.push(json!({
            "case_ref": state.case_id,
            "display_name": case_display_name(&state.case_id),
            "case_status": lifecycle(&state.lifecycle),
            "generation": state.generation,
            "updated_at_unix_ms": updated_at_unix_ms,
            "participant_count": state.participants.len(),
            "source_count": state.sources.len(),
            "resource_count": state.resources.len(),
            "pending_review_count": pending_reviews(&state),
            "summary_available": true
        }));
    }
    Ok(json!({ "cases": cases, "authority": "yai.application" }))
}

fn case_open(
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    case_ref: &str,
) -> Result<Value, String> {
    let state = store.get_case_state_authorized(auth, case_ref)?;
    let principal = auth.projected_principal_id();
    let participant = state
        .principal_participant_links
        .iter()
        .find(|link| link.principal_id == principal)
        .map(|link| link.participant_id.clone())
        .ok_or_else(|| "authenticated_principal_participant_link_required".to_string())?;
    let history = store.list_case_transitions(case_ref)?;
    let thread_ref = turns_from_history(case_ref, &history)
        .into_iter()
        .rev()
        .find(|turn| turn.participant_id == participant)
        .map(|turn| turn.thread_id.clone());
    Ok(json!({
        "case_ref": case_ref,
        "case_status": lifecycle(&state.lifecycle),
        "generation": state.generation,
        "authenticated_principal": principal,
        "participant_ref": participant,
        "thread_ref": thread_ref,
        "attachment": "ephemeral",
        "changed_active_case": false,
        "receipt_ref": Value::Null,
        "event_refs": []
    }))
}

fn case_snapshot(
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    case_ref: &str,
    content: Option<&ConversationContentStore>,
    expected_generation: Option<u64>,
) -> Result<Value, String> {
    let state = store.get_case_state_authorized(auth, case_ref)?;
    if expected_generation.is_some_and(|expected| expected != state.generation) {
        return Err(format!(
            "stale_generation:expected={}:actual={}",
            expected_generation.unwrap_or_default(),
            state.generation
        ));
    }
    let history = store.list_case_transitions(case_ref)?;
    let graph = store.list_graph_relations_by_case(case_ref, 512)?;
    let participant_ref = case_open(store, auth, case_ref)?["participant_ref"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let participants = state
        .participants
        .iter()
        .map(|participant| {
            json!({
                "id": participant.participant_id,
                "roles": participant.roles,
                "is_current": participant.participant_id == participant_ref
            })
        })
        .collect::<Vec<_>>();
    let sources = state.sources.iter().map(|source| json!({
        "id": source.declaration.source_id,
        "label": source.declaration.logical_name,
        "kind": source_kind(&source.declaration.action),
        "perimeter": source.declaration.perimeter,
        "media_type": source.declaration.media_type,
        "roles": source.declaration.roles,
        "resource_ref": source.declaration.resource_attachment_id,
        "attempt": source.progress.as_ref().map(|progress| progress.attempt),
        "progress_ref": source.progress.as_ref().map(|progress| progress.progress_id.clone()),
        "posture": source.progress.as_ref().map(|progress| format!("{:?}", progress.phase).to_lowercase()),
        "revision_ref": source.progress.as_ref().and_then(|progress| progress.revision.as_ref()).map(|revision| revision.revision_id.clone()),
        "items": source.progress.as_ref().and_then(|progress| progress.revision.as_ref()).map(|revision| revision.items.len())
    })).collect::<Vec<_>>();
    let files = state
        .sources
        .iter()
        .flat_map(|source| {
            source.progress.iter().flat_map(move |progress| {
                progress.revision.iter().flat_map(move |revision| {
                    revision.items.iter().map(move |item| json!({
                "id": format!("source-file:{}:{}", source.declaration.source_id, item.digest),
                "source_ref": source.declaration.source_id,
                "source_label": source.declaration.logical_name,
                "revision_ref": revision.revision_id,
                "path": item.path,
                "digest": item.digest,
                "bytes": item.bytes,
                "media_type": source.declaration.media_type,
                "backing": item.backing
            }))
                })
            })
        })
        .collect::<Vec<_>>();
    let resources = state
        .resources
        .iter()
        .map(|resource| {
            json!({
                "id": resource.attachment_id,
                "kind": format!("{:?}", resource.kind).to_lowercase(),
                "policy_ref": resource.policy_id,
                "review_requirement": format!("{:?}", resource.review_requirement).to_lowercase(),
                "allowed_write_prefix": resource.allowed_write_prefix,
                "max_write_bytes": resource.max_write_bytes,
                "configuration_digest": resource.access.as_ref().map(|access| access.configuration_digest.clone()),
                "operations": resource.access.as_ref().map(|access| access.operations.iter().map(|operation| operation.operation_name()).collect::<Vec<_>>()).unwrap_or_default(),
                "read_prefixes": resource.access.as_ref().map(|access| access.read_prefixes.clone()).unwrap_or_default(),
                "names": resource.access.as_ref().map(|access| access.names.clone()).unwrap_or_default(),
                "max_output_bytes": resource.access.as_ref().map(|access| access.max_output_bytes),
                "max_items": resource.access.as_ref().map(|access| access.max_items)
            })
        })
        .collect::<Vec<_>>();
    let timeline = history
        .iter()
        .rev()
        .take(160)
        .rev()
        .map(timeline_entry)
        .collect::<Vec<_>>();
    let relations = graph
        .relations
        .iter()
        .map(|relation| {
            json!({
                "id": relation.relation_id,
                "from": relation.from_ref,
                "to": relation.to_ref,
                "kind": relation.edge_kind,
                "from_kind": relation.from_kind,
                "to_kind": relation.to_kind,
                "source_ref": relation.source_record_id,
                "confidence": relation.confidence
            })
        })
        .collect::<Vec<_>>();
    let authority = authority_projection(&state);
    let workflow = workflow_projection(store, auth, &state)?;
    let compute = compute_projection(store, auth, &state)?;
    let knowledge = knowledge_projection(store, auth, case_ref, content);
    let conversation = turns_from_history(case_ref, &history)
        .into_iter()
        .map(|turn| {
            json!({
                "id": turn.turn_id,
                "thread_ref": turn.thread_id,
                "participant_ref": turn.participant_id,
                "generation": turn.base_generation + 1,
                "execution_request_ref": history.iter().find_map(|transition| match &transition.payload {
                    TransitionPayload::ConversationExecutionIntentRecorded { request }
                        if request.source_turn_id == turn.turn_id && request.participant_id == participant_ref => Some(&request.request_id),
                    _ => None,
                }),
                "parts": turn.ordered_parts.iter().map(|part| json!({
                    "part_ref": part.part_id,
                    "modality": part.object.modality,
                    "media_type": part.object.media_type,
                    "text": part.object.inline_text
                })).collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();
    let attention = attention_projection(&state, &workflow, &compute);
    Ok(json!({
        "case": {
            "case_ref": state.case_id,
            "display_name": case_display_name(&state.case_id),
            "case_status": lifecycle(&state.lifecycle),
            "generation": state.generation,
            "tenant_ref": state.tenant_id,
            "participant_ref": participant_ref,
            "updated_at_unix_ms": history.last().map(|transition| transition.committed_at_unix_ms)
        },
        "overview": { "attention": attention, "participants": participants },
        "environment": { "sources": sources, "files": files, "resources": resources, "artifacts": state.admitted_content },
        "knowledge": knowledge,
        "memory": { "authority": "transition_ledger_and_derived_graph", "timeline": timeline, "relations": relations, "generation": state.generation },
        "authority": authority,
        "work": workflow,
        "compute": compute,
        "conversation": { "read_only": true, "turns": conversation },
        "freshness": { "generation": state.generation, "resync_operation": "case.summary" }
    }))
}

fn knowledge_projection(
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    case_ref: &str,
    content: Option<&ConversationContentStore>,
) -> Value {
    // Use the qualified Knowledge profile's bounded default. A lower ad-hoc
    // presentation cap can suppress the entire deterministic derivation when
    // the exact document structure exceeds that cap, which incorrectly makes
    // available Case knowledge appear empty in Studio.
    let request = KnowledgeRequest::new(case_ref);
    match store.case_knowledge_authorized(auth, request, content) {
        Ok(result) => {
            let view = result.view;
            let status = if view.sources.is_empty() {
                "empty"
            } else {
                "available"
            };
            json!({
                "status": status,
                "message": if status == "empty" { "No qualified knowledge has been derived for this Case." } else { "Qualified source-grounded derivation from YAI." },
                "profile": view.profile,
                "source_closure": view.source_closure,
                "sources": view.sources.into_iter().map(|source| json!({
                    "id": source.id,
                    "source_ref": source.source_id,
                    "label": source.logical_name,
                    "revision_ref": source.revision_id,
                    "path": source.path,
                    "digest": source.digest,
                    "media_type": source.media_type,
                    "extractor": source.extractor,
                    "status": format!("{:?}", source.status).to_lowercase(),
                    "detail": source.detail
                })).collect::<Vec<_>>(),
                "units": view.units.into_iter().map(|unit| json!({
                    "id": unit.id,
                    "source_ref": unit.source,
                    "parent_ref": unit.parent,
                    "kind": unit.kind,
                    "text": unit.text,
                    "posture": format!("{:?}", unit.posture).to_lowercase(),
                    "entity_ref": unit.entity,
                    "predicate": unit.predicate,
                    "value": unit.value,
                    "references": unit.references,
                    "topics": unit.topics
                })).collect::<Vec<_>>(),
                "entities": view.entities,
                "topics": view.topics,
                "contradictions": view.contradictions,
                "relations": view.relations.into_iter().map(|relation| json!({
                    "id": relation.id,
                    "from": relation.from,
                    "to": relation.to,
                    "kind": format!("{:?}", relation.kind).to_lowercase(),
                    "posture": relation.posture,
                    "backing_units": relation.backing_units
                })).collect::<Vec<_>>(),
                "counts": {
                    "sources": result.measurements.source_documents,
                    "units": result.measurements.units,
                    "entities": result.measurements.entities,
                    "claims": result.measurements.claims,
                    "relations": result.measurements.relations,
                    "contradictions": result.measurements.contradictions
                }
            })
        }
        Err(error) => {
            let status = if error.contains("backing") {
                "backing_unavailable"
            } else {
                "unavailable"
            };
            json!({
                "status": status,
                "message": if status == "backing_unavailable" { "Qualified source backing is unavailable." } else { "Knowledge is unavailable to the current principal." },
                "sources": [], "units": [], "entities": [], "topics": [], "contradictions": [], "relations": []
            })
        }
    }
}

fn timeline_entry(transition: &Transition) -> Value {
    json!({
        "id": transition.transition_id,
        "sequence": transition.sequence,
        "committed_at_unix_ms": transition.committed_at_unix_ms,
        "kind": transition.payload.kind(),
        "participant_ref": transition.source.participant_id,
        "component": transition.source.component,
        "causal_refs": transition.causal_refs,
        "summary": transition.summary
    })
}

fn authority_projection(state: &CaseState) -> Value {
    let policies = state
        .policy_bindings
        .iter()
        .map(|binding| {
            json!({
                "id": binding.binding_id,
                "policy_key": binding.policy_key,
                "lineage_ref": binding.lineage_id,
                "artifact_ref": binding.artifact_id,
                "source_ref": binding.source_id,
                "owner_ref": binding.owner_ref,
                "version": binding.artifact_version,
                "bound_at_generation": binding.bound_at_case_generation,
                "reason": binding.reason
            })
        })
        .collect::<Vec<_>>();
    let reviews = state
        .reviews
        .iter()
        .map(|review| {
            json!({
                "id": review.review_id,
                "status": review.status,
                "operation_ref": review.operation_id,
                "policy_ref": review.effective_policy_id,
                "decision_ref": review.effective_decision_id,
                "evidence_ref": review.receipt_ref,
                "required_roles": review.required_reviewer_roles
            })
        })
        .collect::<Vec<_>>();
    let grants = state
        .grants
        .iter()
        .map(|grant| serde_json::to_value(grant).unwrap_or(Value::Null))
        .collect::<Vec<_>>();
    json!({
        "policies": policies,
        "reviews": reviews,
        "grants": grants,
        "last_decision": state.last_decision,
        "empty": state.policy_bindings.is_empty() && state.reviews.is_empty() && state.grants.is_empty()
    })
}

fn workflow_projection(
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    state: &CaseState,
) -> Result<Value, String> {
    let Some(binding) = &state.workflow_binding else {
        return Ok(
            json!({ "status": "empty", "message": "No workflow is configured for this Case.", "nodes": [], "edges": [] }),
        );
    };
    let resolution = store.workflow_status_authorized(auth, &state.case_id)?;
    let definition =
        store.get_workflow_definition_authorized(auth, &binding.workflow_definition_id)?;
    if resolution.case_generation != state.generation {
        return Err("workflow_observation_stale".to_string());
    }
    let edges = resolution
        .effective_edges
        .iter()
        .enumerate()
        .map(|(index, edge)| {
            json!({
                "id": format!("workflow-edge:{index}:{}:{}", edge.from, edge.to),
                "from": edge.from,
                "to": edge.to,
                "kind": format!("{:?}", edge.kind).to_lowercase()
            })
        })
        .collect::<Vec<_>>();
    Ok(
        json!({ "status": "available", "definition": definition, "resolution": resolution, "effective_nodes": resolution.effective_nodes, "nodes": resolution.nodes, "edges": edges }),
    )
}


fn provider_target_projection(
    store: &LmdbRecordStore, auth: &AuthenticatedPrincipal,
    target: yai_core_engine::provider_governance::ProviderTarget,
) -> Value {
    let posture = store
        .provider_posture_authorized(auth, &target.target_id)
        .ok()
        .map(|(_, qualification, trust, health)| {
            json!({
                "qualification": qualification.map(|value| json!({
                    "id": value.qualification_id,
                    "suite_id": value.suite_id,
                    "run_id": value.run_id,
                    "qualified_at_unix_ms": value.qualified_at_unix_ms,
                    "valid_until_unix_ms": value.valid_until_unix_ms,
                    "capabilities": value.capabilities.into_iter().map(|capability| json!({
                        "capability": format!("{:?}", capability.capability).to_lowercase(),
                        "provenance": format!("{:?}", capability.provenance).to_lowercase(),
                        "evidence_refs": capability.evidence_refs,
                        "verified_minimum": capability.verified_minimum
                    })).collect::<Vec<_>>()
                })),
                "trust": trust.map(|value| json!({
                    "event_ref": value.event_id,
                    "posture": format!("{:?}", value.posture).to_lowercase(),
                    "recorded_at_unix_ms": value.recorded_at_unix_ms
                })),
                "health": {
                    "posture": format!("{:?}", health.posture).to_lowercase(),
                    "circuit": format!("{:?}", health.circuit).to_lowercase(),
                    "consecutive_failures": health.consecutive_failures,
                    "observed_at_unix_ms": health.observed_at_unix_ms,
                    "failure_class": health.failure_class
                }
            })
        });
    json!({
        "id": target.target_id,
        "provider_key": target.provider_key,
        "extension_adapter_id": target.extension_adapter_id,
        "adapter": target.adapter,
        "model_id": target.model_id,
        "locality": target.locality,
        "endpoint": sanitized_endpoint(&target.endpoint),
        "posture": posture,
        "management": "not_exposed",
        "semantic_evidence": store.list_semantic_suitability_evidence_authorized(auth, &target.target_id, Some(CognitiveCapability::PrimaryConversation)).unwrap_or_default()
    })
}

fn compute_projection(
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    state: &CaseState,
) -> Result<Value, String> {
    let Some(tenant_ref) = state.tenant_id.as_deref() else {
        return Ok(
            json!({ "status": "unavailable", "message": "Provider posture requires a Tenant-scoped Case.", "targets": [] }),
        );
    };
    let targets = store.list_provider_targets_authorized(auth, tenant_ref)?;
    let bound = state
        .provider_binding
        .as_ref()
        .map(|binding| {
            binding
                .ordered_target_ids
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let projected = targets
        .into_iter()
        .filter(|target| bound.contains(&target.target_id))
        .map(|target| provider_target_projection(store, auth, target))
        .collect::<Vec<_>>();
    Ok(json!({
        "status": if projected.is_empty() { "empty" } else { "available" },
        "message": if projected.is_empty() { "No provider is connected to this Case." } else { "Provider facts are reported by YAI." },
        "targets": projected,
        "cognitive_bindings": state.cognitive_bindings
    }))
}

fn attention_projection(state: &CaseState, workflow: &Value, compute: &Value) -> Vec<Value> {
    let mut attention = Vec::new();
    for review in state.reviews.iter().filter(|review| {
        matches!(
            review.status,
            ReviewResolution::Pending | ReviewResolution::Deferred
        )
    }) {
        attention.push(json!({ "kind": "review", "title": "Review required", "detail": review.policy_reason, "ref": review.review_id }));
    }
    if workflow["status"] == "empty" {
        attention.push(json!({ "kind": "workflow", "title": "No workflow configured", "detail": "Work remains available through other Case operations." }));
    }
    if compute["status"] == "empty" {
        attention.push(json!({ "kind": "provider", "title": "No provider connected", "detail": "Inference is unavailable for this Case." }));
    }
    attention
}

fn pending_reviews(state: &CaseState) -> usize {
    state
        .reviews
        .iter()
        .filter(|review| {
            matches!(
                review.status,
                ReviewResolution::Pending | ReviewResolution::Deferred
            )
        })
        .count()
}

fn lifecycle(value: &CaseLifecycle) -> &'static str {
    match value {
        CaseLifecycle::Open => "open",
        CaseLifecycle::Closed => "closed",
    }
}

fn sanitized_endpoint(endpoint: &str) -> String {
    endpoint
        .split_once("://")
        .map(|(scheme, rest)| {
            let authority = rest.split('/').next().unwrap_or(rest);
            let host = authority.rsplit('@').next().unwrap_or(authority);
            format!("{scheme}://{host}")
        })
        .unwrap_or_else(|| "configured".to_string())
}

fn success(request: &OperationRequest, data: Value) -> OperationResult {
    OperationResult {
        operation_ref: request.operation_ref.clone(),
        result_state: ResultState::Success,
        correlation_ref: request.correlation_ref.clone(),
        data: Some(data),
        error: None,
    }
}

fn map_error(request: &OperationRequest, error: &str) -> OperationResult {
    let (state, safe) = if error == "operation_not_implemented" {
        (
            ResultState::NotImplemented,
            "This operation is not implemented by the local YAI host.",
        )
    } else if error == "cognitive_realization_acknowledgement_pending_observe_exact_plan" {
        (ResultState::CorePending, "Submission acknowledgement is pending. Observe the exact plan reference; do not create a replacement submission.")
    } else if matches!(error, "application_execution_capacity_pending" | "execution_carrier_active_requires_observation") {
        (ResultState::CorePending, "Execution capacity is occupied. Observe an existing submission before retrying.")
    } else if matches!(error, "runtime_instance_not_running" | "runtime_instance_not_accepting_work") {
        (ResultState::CorePending, "The runtime is not accepting work. No new execution was submitted.")
    } else if error == "capability_view_participant_not_admitted" {
        (ResultState::Unauthorized,
            "This Participant is not admitted to inspect the Case capabilities.")
    } else if error == "historical_scope_unavailable"
        || error == "provider_probe_submission_not_authorized"
        || error.contains("not_visible")
        || error.contains("authentication")
        || error.contains("principal")
        || error.contains("owner")
    {
        (
            ResultState::Unauthorized,
            "This Case is not available to the authenticated local principal.",
        )
    } else if error.contains("stale") {
        (
            ResultState::Stale,
            "The Case changed. Studio must resynchronize before continuing.",
        )
    } else if request.operation_ref == "resource.attach_process"
        && error.starts_with("process_identity_stat_unavailable:")
    {
        (ResultState::Error, "The selected process is unavailable. Verify its current PID; no Resource was attached.")
    } else if error.contains("not found")
        || error.contains("No such file")
        || error.contains("lmdb")
    {
        (
            ResultState::TransportUnavailable,
            "The local YAI application store is unavailable.",
        )
    } else {
        (
            ResultState::Error,
            "YAI could not produce this application projection.",
        )
    };
    failure(request, state, error, safe)
}

fn failure(
    request: &OperationRequest,
    state: ResultState,
    code: &str,
    safe: &str,
) -> OperationResult {
    OperationResult {
        operation_ref: request.operation_ref.clone(),
        result_state: state,
        correlation_ref: request.correlation_ref.clone(),
        data: None,
        error: Some(OperationError {
            code: code.to_string(),
            message: code.to_string(),
            safe_message: safe.to_string(),
            result_state: state,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_execution_acknowledgement_is_not_completion_or_safe_redispatch() {
        let request = OperationRequest { protocol:APPLICATION_PROTOCOL.into(),
            operation_ref:"cognitive.realize".into(), correlation_ref:"test:pending".into(), input:Value::Null };
        let result = map_error(&request, "cognitive_realization_acknowledgement_pending_observe_exact_plan");
        assert_eq!(result.result_state, ResultState::CorePending);
        assert!(result.data.is_none());
        assert!(result.error.unwrap().safe_message.contains("Observe the exact plan"));
    }

    #[test]
    fn unavailable_historical_participant_scope_is_an_authorization_refusal() {
        let request = OperationRequest { protocol: APPLICATION_PROTOCOL.into(),
            operation_ref: "decision.trajectory.inspect".into(),
            correlation_ref: "test:hidden-history".into(), input: Value::Null };
        let result = map_error(&request, "historical_scope_unavailable");
        assert_eq!(result.result_state, ResultState::Unauthorized);
        assert!(result.data.is_none());
    }

    #[test]
    fn unsupported_operation_is_honest() {
        let app = LocalApplication::from_yai_home("/path/does/not/matter");
        let result = app.call(OperationRequest {
            protocol: APPLICATION_PROTOCOL.into(),
            operation_ref: "provider.telemetry.fake".into(),
            correlation_ref: "test:1".into(),
            input: Value::Null,
        });
        assert_eq!(result.result_state, ResultState::NotImplemented);
        assert!(result.data.is_none());
    }

    #[test]
    fn capability_discovery_needs_no_case_store_and_leaks_no_case_state() {
        let app = LocalApplication::from_yai_home("/path/does/not/matter");
        let result = app.call(OperationRequest {
            protocol: APPLICATION_PROTOCOL.into(),
            operation_ref: "application.capabilities".into(),
            correlation_ref: "test:capabilities".into(),
            input: json!({}),
        });
        assert_eq!(result.result_state, ResultState::Success);
        let data = result.data.unwrap();
        assert_eq!(data["schema"], capabilities::CAPABILITY_CATALOG_SCHEMA);
        assert_eq!(data["capabilities"].as_array().unwrap().len(), 44);
        assert!(data.get("cases").is_none());
        assert!(data.get("resources").is_none());
    }

    #[test]
    fn version_mismatch_fails_before_store_access() {
        let app = LocalApplication::from_yai_home("/path/does/not/matter");
        let result = app.call(OperationRequest {
            protocol: "future".into(),
            operation_ref: "case.list".into(),
            correlation_ref: "test:2".into(),
            input: Value::Null,
        });
        assert_eq!(result.error.unwrap().code, "version_mismatch");
    }

    #[test]
    fn endpoint_projection_omits_path_and_userinfo() {
        assert_eq!(
            sanitized_endpoint("https://operator:secret@example.test/v1/chat?key=no"),
            "https://example.test"
        );
        assert_eq!(sanitized_endpoint("opaque-local-target"), "configured");
    }

    #[test]
    fn case_presentation_label_preserves_canonical_identity_separately() {
        assert_eq!(case_display_name("case:yai-enterprise-launch"),
            "YAI — Primo rilascio aziendale");
        assert_eq!(
            case_display_name("case:studio-live-qualification"),
            "Studio Live Qualification"
        );
        assert_eq!(case_display_name("not-a-case-ref"), "not-a-case-ref");
    }

    #[test]
    fn exact_material_transport_encodes_binary_without_coercing_it_to_text() {
        assert_eq!(encode_base64(b"YAI"), "WUFJ");
        assert_eq!(encode_base64(&[0, 255, 16, 32]), "AP8QIA==");
    }

    #[test]
    fn source_kind_comes_from_the_typed_action() {
        assert_eq!(
            source_kind(&ResourceAction::Discover {
                path: "studio".into()
            }),
            "discovery"
        );
        assert_eq!(
            source_kind(&ResourceAction::DatabaseQuery {
                name: "inventory".into()
            }),
            "database_query"
        );
        assert_eq!(
            source_kind(&ResourceAction::HttpFetch {
                name: "documentation".into()
            }),
            "http_fetch"
        );
    }

    #[test]
    fn capability_catalog_is_deterministic_and_surface_complete() {
        capabilities::validate_capability_catalog().unwrap();
        let first = serde_json::to_vec(&capabilities::capability_catalog()).unwrap();
        let second = serde_json::to_vec(&capabilities::capability_catalog()).unwrap();
        assert_eq!(first, second);
        for (id, impact) in [
            ("case.stop", capabilities::StateImpact::OperationalMutation),
            ("case.cancel", capabilities::StateImpact::CanonicalMutation),
            ("source.acquire", capabilities::StateImpact::ExternalEffect),
            ("source.resume", capabilities::StateImpact::ExternalEffect),
        ] {
            assert_eq!(capabilities::APPLICATION_OPERATIONS.iter().find(|op| op.operation_id == id).unwrap().impact, impact);
        }
        assert!(capabilities::CAPABILITIES.iter().all(|capability| {
            capability.disposition != capabilities::CapabilityDisposition::InternalMechanic
                || (capability.parent_capability_id.is_some() && capability.rationale.is_some())
        }));
    }

    #[test]
    fn product_application_debt_is_exact_and_contract_blocked() {
        use capabilities::{ApplicationPosture, CapabilityDisposition};

        let product = capabilities::CAPABILITIES
            .iter()
            .filter(|capability| {
                matches!(
                    capability.disposition,
                    CapabilityDisposition::ProductRead | CapabilityDisposition::ProductAction
                )
            })
            .collect::<Vec<_>>();
        let ready = product
            .iter()
            .filter(|capability| capability.application_posture == ApplicationPosture::Ready)
            .count();
        let deferred = product
            .iter()
            .filter(|capability| capability.application_posture == ApplicationPosture::Deferred)
            .count();
        assert_eq!(product.len(), 34);
        assert_eq!(ready, 33);
        assert_eq!(deferred, 0);
        assert_eq!(capabilities::APPLICATION_BLOCKERS.len(), deferred);
        assert!(capabilities::APPLICATION_BLOCKERS.iter().all(|blocker| {
            !blocker.missing_contract.contains("wrapper")
                && !blocker.missing_contract.contains("CLI owner")
                && !blocker.missing_contract.trim().is_empty()
        }));
    }

    #[test]
    fn public_application_inputs_are_bidirectionally_typed() {
        fn typed<T: Serialize + for<'de> Deserialize<'de>>() {}

        typed::<DecisionTrajectoryInspectInput>();
        typed::<DecisionTrajectoryCorpusInput>();

        typed::<cognitive_execution::CognitiveComposeInput>();
        typed::<cognitive_execution::CognitiveRealizationPrepareInput>();
        typed::<cognitive_execution::CognitiveRealizeInput>();
        typed::<cognitive_execution::CognitiveRealizationObservation>();

        typed::<CaseCapabilitiesInput>();
        typed::<EffectProposeInput>();
        typed::<EffectProposalResult>();
        typed::<DecisionFrontierPrepareInput>();
        typed::<DecisionRequestPrepareInput>();
        typed::<fast_search::FastSearchPrepareInput>();
        typed::<fast_search::FastSearchPrepareResult>();
        typed::<RecallExecuteInput>();
        typed::<WorkingStateCompileInput>();
        typed::<WorkingStateRefreshInput>();
        typed::<WorkingStatePageInput>();
        typed::<AmbientRefreshAssessInput>();
        typed::<IdentityBootstrapInput>();
        typed::<TenantGetInput>();
        typed::<TenantMemberAddInput>();
        typed::<CaseCreateInput>();
        typed::<KnowledgeInspectInput>();
        typed::<KnowledgeSearchInput>();
        typed::<KnowledgeResolveInput>();
        typed::<KnowledgeSearchResult>();
        typed::<KnowledgeResolveResult>();
        typed::<KnowledgeNavigationResult>();
        typed::<CaseTerminalInput>();
        typed::<ExecutionGetInput>();
        typed::<ExecutionListInput>();
        typed::<ResourceRequestInput>();
        typed::<ResourceSubmissionResult>();
        typed::<resource_execution::ResourceExecutionObservation>();
        typed::<ExecutionObservation>();
        typed::<SourceExecutionObservation>();
        typed::<CaseRunInput>();
        typed::<CaseResumeInput>();
        typed::<cognitive_execution::ConversationSendInput>();
        typed::<cognitive_execution::ConversationSubmissionResult>();
        typed::<ExecutionSubmissionResult>();
        typed::<RuntimeExecutionObservation>();
        typed::<WorkflowDefineInput>();
        typed::<WorkflowBindInput>();
        typed::<WorkflowInputRecordInput>();
        typed::<WorkflowPatchProposeInput>();
        typed::<WorkflowPatchAdoptInput>();
        typed::<HandoffPendingInput>();
        typed::<HandoffInspectInput>();
        typed::<HandoffOfferInput>();
        typed::<HandoffAcceptInput>();
        typed::<HandoffDeclineInput>();
        typed::<HandoffResultInput>();
        typed::<HandoffReconcileInput>();
        typed::<PolicyIngestInput>();
        typed::<PolicyArtifactLifecycleInput>();
        typed::<CasePolicyBindInput>();
        typed::<CasePolicyReplaceInput>();
        typed::<CasePolicyUnbindInput>();
        typed::<ReviewResolveInput>();
        typed::<ParticipantRoleAddInput>();
        typed::<ParticipantPrincipalLinkInput>();
        typed::<ParticipantViewAdmitInput>();
        typed::<ProviderModelsInput>();
        typed::<ProviderModelDiscoveryInput>();
        typed::<ProviderRegisterInput>();
        typed::<ProviderQualifyInput>();
        typed::<ProviderProbeInput>();
        typed::<ProviderProbeGetInput>();
        typed::<ProviderTrustInput>();
        typed::<ProviderSuitabilityRecordInput>();
        typed::<ProviderCaseBindInput>();
        typed::<CognitiveBindInput>();
        typed::<CognitiveTargetReference>();
        typed::<CognitivePlanInput>();
        typed::<ResourceImportInput>();
        typed::<ResourceAttachInput>();
        typed::<ResourceAttachProcessInput>();
        typed::<SourceDeclareInput>();
        typed::<SourceAcquireInput>();
        typed::<CaseStopInput>();
        typed::<SourceResumeInput>();
        typed::<SourceAcquisitionSubmissionResult>();
        typed::<SourcePublishInput>();
        typed::<SourceRevokeInput>();
    }

    #[test]
    fn committed_capability_matrix_matches_the_code_owned_catalog() {
        assert_eq!(
            capabilities::render_capability_matrix(),
            include_str!("../../../docs/reference/application-capabilities.md")
        );
    }
}
