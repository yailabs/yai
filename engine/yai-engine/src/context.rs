//! Provider-independent semantic projection and invocation framing.
//!
//! This module owns one pure derivation boundary:
//! `CaseState + canonical Transition history -> Projection -> ContextFrame`.
//! Projection and frame values are immutable, bounded and rebuildable. They do
//! not own Case continuity, provider wire formats, tokenization or runtime/KV
//! continuation state.

use crate::effect::{DecisionOutcome, EffectOutcome};
use crate::transition::{
    CaseLifecycle, CaseState, EffectLifecycle, ResourceKind, ReviewRequirement, ReviewResolution,
    Transition, TransitionPayload,
};
use serde::{Deserialize, Serialize};

pub const PROJECTION_SCHEMA: &str = "yai.projection.v4";
pub const CONTEXT_FRAME_SCHEMA: &str = "yai.context_frame.v4";
pub const RENDERED_INPUT_SCHEMA: &str = "yai.rendered_input.v4";
pub const DEFAULT_MAX_PROJECTION_ITEMS: usize = 48;
pub const DEFAULT_MAX_PROVIDER_CLAIMS: usize = 6;
pub const DEFAULT_MAX_INTERACTION_TURNS: usize = 8;
pub const DEFAULT_MAX_CLAIM_CHARS: usize = 320;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionPurpose {
    Conversation,
    FilesystemWriteProposal,
    EffectConsequence,
    Inspection,
}

impl ProjectionPurpose {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Conversation => "conversation",
            Self::FilesystemWriteProposal => "filesystem_write_proposal",
            Self::EffectConsequence => "effect_consequence",
            Self::Inspection => "inspection",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectionVisibility {
    pub consumer: String,
    pub view_kind: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityPosture {
    CommittedOperationalFact,
    ObservedResourceState,
    ControlState,
    DerivedMemory,
    ProviderClaim,
    Unresolved,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceKind {
    Transition,
    Observation,
    EffectReceipt,
    CaseStateGeneration,
    DerivedMemory,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticProvenance {
    pub kind: ProvenanceKind,
    pub source_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ProjectedValue {
    CaseLifecycle {
        lifecycle: CaseLifecycle,
    },
    ParticipantBinding {
        participant_id: String,
        roles: Vec<String>,
        admitted_consumer: String,
        admitted_view_kind: String,
    },
    ProviderBinding {
        provider_id: String,
        provider_kind: String,
        model_id: String,
    },
    ResourceAttachment {
        attachment_id: String,
        resource_kind: ResourceKind,
        allowed_write_prefix: String,
        max_write_bytes: usize,
        review_requirement: ReviewRequirement,
    },
    DecisionOutcome {
        operation_id: String,
        decision_id: String,
        outcome: DecisionOutcome,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        decision_basis_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        effective_policy_id: Option<String>,
    },
    ReviewPosture {
        review_id: String,
        operation_id: String,
        reviewer_participant_id: String,
        status: ReviewResolution,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        latest_action_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        effective_decision_id: Option<String>,
    },
    ResourceConsequence {
        operation_id: String,
        effect_id: String,
        relative_path: String,
        lifecycle: EffectLifecycle,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        outcome: Option<EffectOutcome>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        content_digest: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        receipt_id: Option<String>,
    },
    ProviderClaim {
        result_id: String,
        invocation_id: String,
        preview: String,
    },
    InteractionTurn {
        turn_id: String,
        thread_id: String,
        operator_input: String,
        result_id: String,
    },
    DerivedMemory {
        memory_ref: String,
        semantic_kind: String,
        memory_posture: String,
        description: String,
        lifecycle: String,
        score: i64,
        ranking_reasons: Vec<String>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectionEntry {
    pub entry_id: String,
    pub posture: AuthorityPosture,
    pub value: ProjectedValue,
    pub provenance: Vec<SemanticProvenance>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectionBounds {
    pub max_items: usize,
    pub selected_items: usize,
    pub omitted_items: usize,
    pub history_transitions_considered: usize,
    pub graph_available: bool,
    pub memory_available: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieval_id: Option<String>,
    #[serde(default)]
    pub retrieval_candidates: usize,
    #[serde(default)]
    pub retrieval_selected: usize,
    #[serde(default)]
    pub retrieval_omitted: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub residency_plan_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_unit_budget: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_semantic_units: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Projection {
    pub schema: String,
    pub projection_id: String,
    pub case_id: String,
    pub case_generation: u64,
    pub participant_id: String,
    pub purpose: ProjectionPurpose,
    pub visibility: ProjectionVisibility,
    pub entries: Vec<ProjectionEntry>,
    pub bounds: ProjectionBounds,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionRequest {
    pub participant_id: String,
    pub purpose: ProjectionPurpose,
    pub consumer: String,
    pub view_kind: String,
    pub max_items: usize,
    pub max_provider_claims: usize,
    pub max_interaction_turns: usize,
}

impl ProjectionRequest {
    pub fn model(participant_id: impl Into<String>, purpose: ProjectionPurpose) -> Self {
        Self {
            participant_id: participant_id.into(),
            purpose,
            consumer: "model".to_string(),
            view_kind: "model_context".to_string(),
            max_items: DEFAULT_MAX_PROJECTION_ITEMS,
            max_provider_claims: DEFAULT_MAX_PROVIDER_CLAIMS,
            max_interaction_turns: DEFAULT_MAX_INTERACTION_TURNS,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DerivedMemoryInput {
    pub memory_ref: String,
    pub semantic_kind: String,
    pub memory_posture: String,
    pub description: String,
    pub lifecycle: String,
    pub score: i64,
    pub ranking_reasons: Vec<String>,
    pub transition_refs: Vec<String>,
    pub observation_refs: Vec<String>,
    pub receipt_refs: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DerivedProjectionInput {
    pub graph_available: bool,
    pub memory_available: bool,
    pub memory: Vec<DerivedMemoryInput>,
    pub retrieval_id: Option<String>,
    pub retrieval_candidates: usize,
    pub retrieval_omitted: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "contract", rename_all = "snake_case")]
pub enum InvocationOutputContract {
    NaturalLanguage,
    FilesystemWriteProposal {
        schema: String,
        attachment_id: String,
        allowed_write_prefix: String,
        max_write_bytes: usize,
    },
    CaseRuntimeTurn {
        schema: String,
        operation_schema: String,
        attachment_id: String,
        allowed_write_prefix: String,
        max_write_bytes: usize,
    },
}

impl InvocationOutputContract {
    pub fn contract_id(&self) -> String {
        format!(
            "output-contract:{}",
            stable_digest(&serde_json::to_string(self).expect("serializable output contract"))
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContextFrame {
    pub schema: String,
    pub frame_id: String,
    pub projection_id: String,
    pub case_id: String,
    pub case_generation: u64,
    pub participant_id: String,
    pub purpose: ProjectionPurpose,
    pub task: String,
    pub semantic_instructions: Vec<String>,
    pub entries: Vec<ProjectionEntry>,
    pub output_contract: InvocationOutputContract,
    pub model_independent_constraints: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderModelProfile {
    pub provider_id: String,
    pub provider_kind: String,
    pub model_id: String,
    pub structured_output_supported: bool,
    pub continuation_supported: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderContinuationReference {
    pub provider_id: String,
    pub runtime_id: String,
    pub opaque_reference: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContinuationDisposition {
    NotProvided,
    Used,
    InvalidatedAndRetried,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RenderedInputMetadata {
    pub schema: String,
    pub rendered_input_id: String,
    pub context_frame_id: String,
    pub provider_id: String,
    pub model_id: String,
    pub content_digest: String,
    pub content_chars: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderedInput {
    pub metadata: RenderedInputMetadata,
    pub system_content: String,
    pub user_content: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "artifact_kind", content = "artifact", rename_all = "snake_case")]
pub enum SemanticContextArtifact {
    Projection(Projection),
    ContextFrame(ContextFrame),
    RenderedInputMetadata(RenderedInputMetadata),
    ResidencyPlan(crate::residency::ResidencyPlan),
}

impl SemanticContextArtifact {
    pub fn id(&self) -> &str {
        match self {
            Self::Projection(value) => &value.projection_id,
            Self::ContextFrame(value) => &value.frame_id,
            Self::RenderedInputMetadata(value) => &value.rendered_input_id,
            Self::ResidencyPlan(value) => &value.plan_id,
        }
    }

    pub fn case_id(&self) -> Option<&str> {
        match self {
            Self::Projection(value) => Some(&value.case_id),
            Self::ContextFrame(value) => Some(&value.case_id),
            Self::RenderedInputMetadata(_) => None,
            Self::ResidencyPlan(value) => Some(&value.request.case_id),
        }
    }
}

pub fn compile_projection(
    state: &CaseState,
    transitions: &[Transition],
    request: &ProjectionRequest,
    derived: &DerivedProjectionInput,
) -> Result<Projection, String> {
    if request.max_items == 0 {
        return Err("projection_max_items_must_be_positive".to_string());
    }
    if state.generation != transitions.last().map(|item| item.sequence).unwrap_or(0) {
        return Err("projection_history_generation_mismatch".to_string());
    }
    let participant = state
        .participants
        .iter()
        .find(|item| item.participant_id == request.participant_id)
        .ok_or_else(|| "projection_participant_not_bound".to_string())?;
    let admitted = participant
        .admitted_views
        .iter()
        .any(|view| view.consumer == request.consumer && view.view_kind == request.view_kind);
    if !admitted {
        return Err("projection_view_not_admitted".to_string());
    }

    let mut mandatory = Vec::new();
    mandatory.push(ProjectionEntry {
        entry_id: "case:lifecycle".to_string(),
        posture: AuthorityPosture::CommittedOperationalFact,
        value: ProjectedValue::CaseLifecycle {
            lifecycle: state.lifecycle.clone(),
        },
        provenance: provenance_for_latest(transitions, |payload| {
            matches!(payload, TransitionPayload::CaseOpened { .. })
        }),
    });
    mandatory.push(ProjectionEntry {
        entry_id: format!("participant:{}", request.participant_id),
        posture: AuthorityPosture::CommittedOperationalFact,
        value: ProjectedValue::ParticipantBinding {
            participant_id: request.participant_id.clone(),
            roles: participant.roles.clone(),
            admitted_consumer: request.consumer.clone(),
            admitted_view_kind: request.view_kind.clone(),
        },
        provenance: provenance_for_latest(transitions, |payload| {
            matches!(
                payload,
                TransitionPayload::ParticipantAdmitted { participant_id, consumer, view_kind }
                    if participant_id == &request.participant_id
                        && consumer == &request.consumer
                        && view_kind == &request.view_kind
            )
        }),
    });
    if let Some(provider) = &state.provider {
        if provider.participant_id == request.participant_id {
            mandatory.push(ProjectionEntry {
                entry_id: "provider:binding".to_string(),
                posture: AuthorityPosture::CommittedOperationalFact,
                value: ProjectedValue::ProviderBinding {
                    provider_id: provider.provider_id.clone(),
                    provider_kind: provider.provider_kind.clone(),
                    model_id: provider.model_id.clone(),
                },
                provenance: provenance_for_latest(transitions, |payload| {
                    matches!(payload, TransitionPayload::ProviderAttached { participant_id, .. } if participant_id == &request.participant_id)
                }),
            });
        }
    }
    for resource in &state.resources {
        mandatory.push(ProjectionEntry {
            entry_id: format!("resource:{}", resource.attachment_id),
            posture: AuthorityPosture::CommittedOperationalFact,
            value: ProjectedValue::ResourceAttachment {
                attachment_id: resource.attachment_id.clone(),
                resource_kind: resource.kind.clone(),
                allowed_write_prefix: resource.allowed_write_prefix.clone(),
                max_write_bytes: resource.max_write_bytes,
                review_requirement: resource.review_requirement.clone(),
            },
            provenance: provenance_for_latest(transitions, |payload| {
                matches!(payload, TransitionPayload::ResourceAttached { attachment } if attachment.attachment_id == resource.attachment_id)
            }),
        });
    }
    if let Some(decision) = &state.last_decision {
        mandatory.push(ProjectionEntry {
            entry_id: format!("decision:{}", decision.decision_id),
            posture: AuthorityPosture::ControlState,
            value: ProjectedValue::DecisionOutcome {
                operation_id: decision.operation_id.clone(),
                decision_id: decision.decision_id.clone(),
                outcome: decision.outcome.clone(),
                decision_basis_id: decision.decision_basis_id.clone(),
                effective_policy_id: decision.effective_policy_id.clone(),
            },
            provenance: provenance_for_latest(transitions, |payload| {
                matches!(payload, TransitionPayload::DecisionRecorded { decision: item } if item.decision_id == decision.decision_id)
            }),
        });
    }
    for review in state.reviews.iter().filter(|review| {
        !review.operation_id.is_empty()
            && (review.requested_by_participant == request.participant_id
                || review.reviewer_participant == request.participant_id
                || state.participants.iter().any(|participant| {
                    participant.participant_id == request.participant_id
                        && !review.required_reviewer_roles.is_empty()
                        && review
                            .required_reviewer_roles
                            .iter()
                            .all(|role| participant.roles.contains(role))
                }))
            && matches!(
                review.status,
                ReviewResolution::Pending
                    | ReviewResolution::PendingOperator
                    | ReviewResolution::Deferred
            )
    }) {
        mandatory.push(ProjectionEntry {
            entry_id: format!("review:{}", review.review_id),
            posture: AuthorityPosture::Unresolved,
            value: ProjectedValue::ReviewPosture {
                review_id: review.review_id.clone(),
                operation_id: review.operation_id.clone(),
                reviewer_participant_id: review.reviewer_participant.clone(),
                status: review.status.clone(),
                latest_action_id: review.latest_action_id.clone(),
                effective_decision_id: review.effective_decision_id.clone(),
            },
            provenance: provenance_for_latest(transitions, |payload| match payload {
                TransitionPayload::ReviewRequested { review: item } => {
                    item.review_id == review.review_id
                }
                TransitionPayload::ReviewActionRecorded { action } => {
                    action.review_id == review.review_id
                }
                _ => false,
            }),
        });
    }
    let mut selected_effects = state
        .effects
        .iter()
        .rev()
        .filter(|effect| effect.status != EffectLifecycle::Finalized)
        .collect::<Vec<_>>();
    selected_effects.extend(
        state
            .effects
            .iter()
            .rev()
            .filter(|effect| effect.status == EffectLifecycle::Finalized)
            .take(4),
    );
    selected_effects.sort_by_key(|effect| effect.updated_at_generation);
    selected_effects.dedup_by(|left, right| left.effect_id == right.effect_id);
    let omitted_historical_effects = state.effects.len().saturating_sub(selected_effects.len());
    for effect in selected_effects {
        let posture = match effect.status {
            EffectLifecycle::Finalized => AuthorityPosture::ObservedResourceState,
            EffectLifecycle::Prepared | EffectLifecycle::Indeterminate => {
                AuthorityPosture::Unresolved
            }
        };
        let mut provenance = provenance_for_latest(transitions, |payload| match payload {
            TransitionPayload::EffectPrepared { prepared } => {
                prepared.effect_id == effect.effect_id
            }
            TransitionPayload::EffectFinalized { effect_id, .. }
            | TransitionPayload::EffectIndeterminate { effect_id, .. }
            | TransitionPayload::EffectReconciled { effect_id, .. } => {
                effect_id == &effect.effect_id
            }
            _ => false,
        });
        if let Some(observation_id) = effect.post_observation_id.as_ref() {
            provenance.push(SemanticProvenance {
                kind: ProvenanceKind::Observation,
                source_ref: observation_id.clone(),
            });
        }
        if let Some(receipt_id) = effect.receipt_id.as_ref() {
            provenance.push(SemanticProvenance {
                kind: ProvenanceKind::EffectReceipt,
                source_ref: receipt_id.clone(),
            });
        }
        mandatory.push(ProjectionEntry {
            entry_id: format!("effect:{}", effect.effect_id),
            posture,
            value: ProjectedValue::ResourceConsequence {
                operation_id: effect.operation_id.clone(),
                effect_id: effect.effect_id.clone(),
                relative_path: effect.relative_path.clone(),
                lifecycle: effect.status.clone(),
                outcome: effect.outcome.clone(),
                content_digest: effect
                    .post_observation_id
                    .as_ref()
                    .map(|_| effect.intended_content_digest.clone()),
                receipt_id: effect.receipt_id.clone(),
            },
            provenance,
        });
    }
    if mandatory.len() > request.max_items {
        return Err(format!(
            "projection_budget_below_mandatory_state: required={} max={}",
            mandatory.len(),
            request.max_items
        ));
    }

    let mut optional = Vec::new();
    for transition in transitions.iter().rev() {
        match &transition.payload {
            TransitionPayload::InteractionTurnRecorded {
                turn_id,
                thread_id,
                participant_id,
                invocation_id: _,
                result_id,
                operator_input,
            } if participant_id == &request.participant_id
                && optional
                    .iter()
                    .filter(|entry: &&ProjectionEntry| {
                        matches!(entry.value, ProjectedValue::InteractionTurn { .. })
                    })
                    .count()
                    < request.max_interaction_turns =>
            {
                optional.push(ProjectionEntry {
                    entry_id: format!("turn:{turn_id}"),
                    posture: AuthorityPosture::ProviderClaim,
                    value: ProjectedValue::InteractionTurn {
                        turn_id: turn_id.clone(),
                        thread_id: thread_id.clone(),
                        operator_input: bounded_text(operator_input, DEFAULT_MAX_CLAIM_CHARS),
                        result_id: result_id.clone(),
                    },
                    provenance: transition_provenance(transition),
                });
            }
            TransitionPayload::ProviderResultRecorded {
                result_id,
                invocation_id,
                output,
                ..
            } if provider_invocation_participant(transitions, invocation_id)
                == Some(request.participant_id.as_str())
                && optional
                    .iter()
                    .filter(|entry: &&ProjectionEntry| {
                        matches!(entry.value, ProjectedValue::ProviderClaim { .. })
                    })
                    .count()
                    < request.max_provider_claims =>
            {
                optional.push(ProjectionEntry {
                    entry_id: format!("provider-claim:{result_id}"),
                    posture: AuthorityPosture::ProviderClaim,
                    value: ProjectedValue::ProviderClaim {
                        result_id: result_id.clone(),
                        invocation_id: invocation_id.clone(),
                        preview: bounded_text(output, DEFAULT_MAX_CLAIM_CHARS),
                    },
                    provenance: transition_provenance(transition),
                });
            }
            _ => {}
        }
    }
    optional.reverse();
    // Retrieval is score-descending. Optional selection keeps the tail, so
    // reverse here to ensure higher-ranked entries survive a Projection budget.
    for memory in derived.memory.iter().rev() {
        optional.push(ProjectionEntry {
            entry_id: format!("memory:{}", memory.memory_ref),
            posture: AuthorityPosture::DerivedMemory,
            value: ProjectedValue::DerivedMemory {
                memory_ref: memory.memory_ref.clone(),
                semantic_kind: memory.semantic_kind.clone(),
                memory_posture: memory.memory_posture.clone(),
                description: bounded_text(&memory.description, DEFAULT_MAX_CLAIM_CHARS),
                lifecycle: memory.lifecycle.clone(),
                score: memory.score,
                ranking_reasons: memory.ranking_reasons.clone(),
            },
            provenance: memory
                .transition_refs
                .iter()
                .map(|source_ref| SemanticProvenance {
                    kind: ProvenanceKind::Transition,
                    source_ref: source_ref.clone(),
                })
                .chain(
                    memory
                        .observation_refs
                        .iter()
                        .map(|source_ref| SemanticProvenance {
                            kind: ProvenanceKind::Observation,
                            source_ref: source_ref.clone(),
                        }),
                )
                .chain(
                    memory
                        .receipt_refs
                        .iter()
                        .map(|source_ref| SemanticProvenance {
                            kind: ProvenanceKind::EffectReceipt,
                            source_ref: source_ref.clone(),
                        }),
                )
                .chain(std::iter::once(SemanticProvenance {
                    kind: ProvenanceKind::DerivedMemory,
                    source_ref: memory.memory_ref.clone(),
                }))
                .collect(),
        });
    }

    let available = request.max_items - mandatory.len();
    let omitted_items = optional
        .len()
        .saturating_sub(available)
        .saturating_add(omitted_historical_effects)
        .saturating_add(derived.retrieval_omitted);
    let keep_from = optional.len().saturating_sub(available);
    let mut entries = mandatory;
    entries.extend(optional.into_iter().skip(keep_from));
    let bounds = ProjectionBounds {
        max_items: request.max_items,
        selected_items: entries.len(),
        omitted_items,
        history_transitions_considered: transitions.len(),
        graph_available: derived.graph_available,
        memory_available: derived.memory_available,
        retrieval_id: derived.retrieval_id.clone(),
        retrieval_candidates: derived.retrieval_candidates,
        retrieval_selected: derived.memory.len(),
        retrieval_omitted: derived.retrieval_omitted,
        residency_plan_id: None,
        semantic_unit_budget: None,
        selected_semantic_units: None,
    };
    let mut projection = Projection {
        schema: PROJECTION_SCHEMA.to_string(),
        projection_id: String::new(),
        case_id: state.case_id.clone(),
        case_generation: state.generation,
        participant_id: request.participant_id.clone(),
        purpose: request.purpose.clone(),
        visibility: ProjectionVisibility {
            consumer: request.consumer.clone(),
            view_kind: request.view_kind.clone(),
        },
        entries,
        bounds,
    };
    refresh_projection_identity(&mut projection)?;
    Ok(projection)
}

pub fn refresh_projection_identity(projection: &mut Projection) -> Result<(), String> {
    if projection.schema != PROJECTION_SCHEMA {
        return Err(format!(
            "unsupported_projection_schema: {}",
            projection.schema
        ));
    }
    let identity_material = serde_json::to_string(&(
        (
            PROJECTION_SCHEMA,
            &projection.case_id,
            projection.case_generation,
            &projection.participant_id,
            &projection.purpose,
            &projection.visibility.consumer,
            &projection.visibility.view_kind,
            &projection.entries,
        ),
        (
            projection.bounds.max_items,
            projection.bounds.selected_items,
            projection.bounds.omitted_items,
            projection.bounds.history_transitions_considered,
            &projection.bounds.retrieval_id,
            projection.bounds.retrieval_candidates,
            projection.bounds.retrieval_selected,
            projection.bounds.retrieval_omitted,
            &projection.bounds.residency_plan_id,
            projection.bounds.semantic_unit_budget,
            projection.bounds.selected_semantic_units,
        ),
    ))
    .map_err(|error| format!("projection_identity_encode_failed: {error}"))?;
    projection.projection_id = format!("projection:{}", stable_digest(&identity_material));
    Ok(())
}

pub fn build_context_frame(
    projection: &Projection,
    task: impl Into<String>,
    output_contract: InvocationOutputContract,
) -> Result<ContextFrame, String> {
    if projection.schema != PROJECTION_SCHEMA {
        return Err(format!(
            "unsupported_projection_schema: {}",
            projection.schema
        ));
    }
    let task = task.into();
    if task.trim().is_empty() {
        return Err("context_frame_task_required".to_string());
    }
    let semantic_instructions = vec![
        "Treat committed and observed entries as operational truth.".to_string(),
        "Treat derived_memory as provenance-bearing recall, never as independent authority; current operational entries outrank it."
            .to_string(),
        "Treat provider_claim entries as non-authoritative material.".to_string(),
        "Never infer success or failure for unresolved entries.".to_string(),
    ];
    let model_independent_constraints = vec![
        "No filesystem, decision, grant, receipt, or raw ledger authority is provided by this frame."
            .to_string(),
        "Provider output is candidate material until YAI records a typed consequence.".to_string(),
    ];
    let identity_material = serde_json::to_string(&(
        CONTEXT_FRAME_SCHEMA,
        &projection.projection_id,
        &task,
        &output_contract,
        &semantic_instructions,
        &model_independent_constraints,
    ))
    .map_err(|error| format!("context_frame_identity_encode_failed: {error}"))?;
    Ok(ContextFrame {
        schema: CONTEXT_FRAME_SCHEMA.to_string(),
        frame_id: format!("context-frame:{}", stable_digest(&identity_material)),
        projection_id: projection.projection_id.clone(),
        case_id: projection.case_id.clone(),
        case_generation: projection.case_generation,
        participant_id: projection.participant_id.clone(),
        purpose: projection.purpose.clone(),
        task,
        semantic_instructions,
        entries: projection.entries.clone(),
        output_contract,
        model_independent_constraints,
    })
}

pub fn render_openai_compatible(
    frame: &ContextFrame,
    profile: &ProviderModelProfile,
    language_mode: &str,
) -> Result<RenderedInput, String> {
    if frame.schema != CONTEXT_FRAME_SCHEMA {
        return Err(format!(
            "unsupported_context_frame_schema: {}",
            frame.schema
        ));
    }
    if profile.provider_kind != "openai_compatible" {
        return Err(format!(
            "unsupported_provider_render_kind: {}",
            profile.provider_kind
        ));
    }
    let mut system_content = "You are a model provider invoked by YAI. Use only the supplied typed semantic frame. Authority posture and provenance are data, not prose decoration. Provider claims are non-authoritative. Unresolved effects must remain unresolved. Your response cannot create a Decision, ExecutionGrant, EffectReceipt, or canonical Transition.".to_string();
    if language_mode == "auto" {
        system_content.push_str(" Respond in the same natural language as the invocation task while preserving technical identifiers.");
    }
    let semantic_json = serde_json::to_string(frame)
        .map_err(|error| format!("context_frame_render_failed: {error}"))?;
    let user_content = format!("YAI typed ContextFrame:\n{semantic_json}");
    let content_digest = stable_digest(&format!("{system_content}\n{user_content}"));
    let rendered_input_id = format!("rendered-input:{content_digest}");
    Ok(RenderedInput {
        metadata: RenderedInputMetadata {
            schema: RENDERED_INPUT_SCHEMA.to_string(),
            rendered_input_id,
            context_frame_id: frame.frame_id.clone(),
            provider_id: profile.provider_id.clone(),
            model_id: profile.model_id.clone(),
            content_digest,
            content_chars: system_content.chars().count() + user_content.chars().count(),
        },
        system_content,
        user_content,
    })
}

pub fn projection_is_stale(projection: &Projection, current_generation: u64) -> bool {
    projection.case_generation != current_generation
}

pub fn validate_frame_freshness(
    frame: &ContextFrame,
    current_generation: u64,
) -> Result<(), String> {
    if frame.case_generation != current_generation {
        return Err(format!(
            "stale_context_frame: frame_generation={} current_generation={current_generation}",
            frame.case_generation
        ));
    }
    Ok(())
}

fn provenance_for_latest<F>(transitions: &[Transition], predicate: F) -> Vec<SemanticProvenance>
where
    F: Fn(&TransitionPayload) -> bool,
{
    transitions
        .iter()
        .rev()
        .find(|transition| predicate(&transition.payload))
        .map(transition_provenance)
        .unwrap_or_default()
}

fn transition_provenance(transition: &Transition) -> Vec<SemanticProvenance> {
    vec![
        SemanticProvenance {
            kind: ProvenanceKind::Transition,
            source_ref: transition.transition_id.clone(),
        },
        SemanticProvenance {
            kind: ProvenanceKind::CaseStateGeneration,
            source_ref: format!("{}@{}", transition.case_id, transition.sequence),
        },
    ]
}

fn provider_invocation_participant<'a>(
    transitions: &'a [Transition],
    invocation_id: &str,
) -> Option<&'a str> {
    transitions
        .iter()
        .rev()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::ProviderInvocationStarted {
                invocation_id: candidate,
                participant_id,
                ..
            } if candidate == invocation_id => Some(participant_id.as_str()),
            _ => None,
        })
}

fn bounded_text(value: &str, max_chars: usize) -> String {
    let compact = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut output = compact.chars().take(max_chars).collect::<String>();
    if compact.chars().count() > max_chars {
        output.push_str("...");
    }
    output
}

pub fn stable_digest(value: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transition::{
        AdmittedView, DecisionState, EffectState, ParticipantState, ProviderAttachmentState,
        ReviewState, TransitionSource, REVIEW_REQUEST_SCHEMA, TRANSITION_SCHEMA,
    };

    fn transition(sequence: u64, payload: TransitionPayload) -> Transition {
        Transition {
            schema: TRANSITION_SCHEMA.to_string(),
            transition_id: format!("transition:{sequence}"),
            case_id: "case:context".to_string(),
            sequence,
            committed_at_unix_ms: sequence,
            source: TransitionSource::component("test"),
            scope: None,
            causal_refs: Vec::new(),
            payload,
            provenance: Vec::new(),
            summary: None,
        }
    }

    fn state(generation: u64) -> CaseState {
        let mut state = CaseState::new("case:context", CaseLifecycle::Open);
        state.generation = generation;
        state.participants = vec![ParticipantState {
            participant_id: "participant:model".to_string(),
            roles: vec!["model_provider".to_string()],
            admitted_views: vec![AdmittedView {
                consumer: "model".to_string(),
                view_kind: "model_context".to_string(),
            }],
        }];
        state.provider = Some(ProviderAttachmentState {
            participant_id: "participant:model".to_string(),
            provider_id: "provider:a".to_string(),
            provider_kind: "openai_compatible".to_string(),
            base_url: "http://127.0.0.1".to_string(),
            model_id: "model:a".to_string(),
            credential_ref: "env:NONE".to_string(),
        });
        state
    }

    #[test]
    fn projection_rebuild_is_deterministic_and_frame_is_task_specific() {
        let history = vec![transition(
            1,
            TransitionPayload::CaseOpened {
                lifecycle: CaseLifecycle::Open,
            },
        )];
        let request =
            ProjectionRequest::model("participant:model", ProjectionPurpose::Conversation);
        let first = compile_projection(
            &state(1),
            &history,
            &request,
            &DerivedProjectionInput::default(),
        )
        .unwrap();
        let second = compile_projection(
            &state(1),
            &history,
            &request,
            &DerivedProjectionInput::default(),
        )
        .unwrap();
        assert_eq!(first, second);
        let answer =
            build_context_frame(&first, "answer", InvocationOutputContract::NaturalLanguage)
                .unwrap();
        let summarize = build_context_frame(
            &first,
            "summarize",
            InvocationOutputContract::NaturalLanguage,
        )
        .unwrap();
        assert_ne!(answer.frame_id, summarize.frame_id);
        assert_eq!(answer.projection_id, summarize.projection_id);
    }

    #[test]
    fn pending_human_review_is_mandatory_unresolved_context_with_provenance() {
        let review = ReviewState {
            review_id: "review:context".to_string(),
            schema: REVIEW_REQUEST_SCHEMA.to_string(),
            integrity_digest: String::new(),
            case_id: "case:context".to_string(),
            operation_id: "operation:context".to_string(),
            operation_digest: "sha256:operation".to_string(),
            initial_decision_id: "decision:require-review".to_string(),
            decision_basis_id: "decision-basis:context".to_string(),
            decision_basis_digest: "sha256:basis".to_string(),
            effective_policy_id: "effective-policy:context".to_string(),
            effective_policy_digest: "sha256:effective".to_string(),
            policy_binding_refs: vec!["case-policy-binding:context".to_string()],
            policy_artifact_refs: vec!["policy-artifact:context".to_string()],
            required_reviewer_roles: vec!["reviewer".to_string()],
            resource_attachment_id: "workspace".to_string(),
            normalized_target: "allowed/reviewed.txt".to_string(),
            created_at_generation: 1,
            latest_action_id: None,
            effective_decision_id: None,
            attempt_id: String::new(),
            requested_by_participant: "participant:model".to_string(),
            target_participant: String::new(),
            reviewer_participant: String::new(),
            operation_kind: String::new(),
            carrier_family: String::new(),
            target_display: String::new(),
            sandbox_path: String::new(),
            target_path: String::new(),
            policy_reason: "resource policy requires review".to_string(),
            status: ReviewResolution::Pending,
            carrier_attempted: false,
            execution_performed: false,
            decision_ref: None,
            receipt_ref: None,
        };
        let history = vec![
            transition(
                1,
                TransitionPayload::CaseOpened {
                    lifecycle: CaseLifecycle::Open,
                },
            ),
            transition(
                2,
                TransitionPayload::ReviewRequested {
                    review: review.clone(),
                },
            ),
        ];
        let mut materialized = state(2);
        materialized.reviews.push(review);
        let projection = compile_projection(
            &materialized,
            &history,
            &ProjectionRequest {
                max_items: 4,
                ..ProjectionRequest::model(
                    "participant:model",
                    ProjectionPurpose::FilesystemWriteProposal,
                )
            },
            &DerivedProjectionInput::default(),
        )
        .expect("compile pending review context at tight budget");
        let review_entry = projection
            .entries
            .iter()
            .find(|entry| matches!(entry.value, ProjectedValue::ReviewPosture { .. }))
            .expect("pending review is mandatory");
        assert_eq!(review_entry.posture, AuthorityPosture::Unresolved);
        assert!(review_entry.provenance.iter().any(|source| {
            source.kind == ProvenanceKind::Transition && source.source_ref == "transition:2"
        }));
        assert!(!projection
            .entries
            .iter()
            .any(|entry| matches!(entry.value, ProjectedValue::ResourceConsequence { .. })));
    }

    #[test]
    fn projection_visibility_fails_before_render() {
        let history = vec![transition(
            1,
            TransitionPayload::CaseOpened {
                lifecycle: CaseLifecycle::Open,
            },
        )];
        let request =
            ProjectionRequest::model("participant:not-admitted", ProjectionPurpose::Conversation);
        assert_eq!(
            compile_projection(
                &state(1),
                &history,
                &request,
                &DerivedProjectionInput::default(),
            )
            .unwrap_err(),
            "projection_participant_not_bound"
        );
    }

    #[test]
    fn graph_and_memory_availability_do_not_own_required_semantics() {
        let history = vec![transition(
            1,
            TransitionPayload::CaseOpened {
                lifecycle: CaseLifecycle::Open,
            },
        )];
        let request =
            ProjectionRequest::model("participant:model", ProjectionPurpose::Conversation);
        let without_derived = compile_projection(
            &state(1),
            &history,
            &request,
            &DerivedProjectionInput::default(),
        )
        .unwrap();
        let available_but_empty = compile_projection(
            &state(1),
            &history,
            &request,
            &DerivedProjectionInput {
                graph_available: true,
                memory_available: true,
                memory: Vec::new(),
                ..DerivedProjectionInput::default()
            },
        )
        .unwrap();
        assert_eq!(
            without_derived.projection_id,
            available_but_empty.projection_id
        );
        assert_eq!(without_derived.entries, available_but_empty.entries);
        assert!(!without_derived.bounds.graph_available);
        assert!(available_but_empty.bounds.graph_available);
    }

    #[test]
    fn large_history_is_bounded_and_reports_omissions() {
        let mut history = vec![transition(
            1,
            TransitionPayload::CaseOpened {
                lifecycle: CaseLifecycle::Open,
            },
        )];
        for index in 1..=50 {
            let invocation_sequence = index * 2;
            let result_sequence = invocation_sequence + 1;
            history.push(transition(
                invocation_sequence,
                TransitionPayload::ProviderInvocationStarted {
                    invocation_id: format!("invocation:{index}"),
                    participant_id: "participant:model".to_string(),
                    provider_id: "provider:a".to_string(),
                    provider_kind: "openai_compatible".to_string(),
                    model_id: "model:a".to_string(),
                    semantic_lineage: None,
                },
            ));
            history.push(transition(
                result_sequence,
                TransitionPayload::ProviderResultRecorded {
                    result_id: format!("result:{index}"),
                    invocation_id: format!("invocation:{index}"),
                    provider_id: "provider:a".to_string(),
                    provider_kind: "openai_compatible".to_string(),
                    model_id: "model:a".to_string(),
                    semantic_lineage: None,
                    output: format!("provider claim {index}"),
                },
            ));
        }
        let mut request =
            ProjectionRequest::model("participant:model", ProjectionPurpose::Conversation);
        request.max_items = 10;
        request.max_provider_claims = 100;
        let mut materialized = state(101);
        for index in 0..100 {
            materialized.effects.push(EffectState {
                effect_id: format!("effect:{index}"),
                operation_id: format!("operation:{index}"),
                decision_id: format!("decision:{index}"),
                grant_id: format!("grant:{index}"),
                resource_attachment_id: "workspace".to_string(),
                relative_path: format!("allowed/{index}.txt"),
                intended_content_digest: format!("digest:{index}"),
                pre_observation_id: format!("observation:{index}:pre"),
                post_observation_id: Some(format!("observation:{index}:post")),
                receipt_id: Some(format!("receipt:{index}")),
                outcome: Some(EffectOutcome::Applied),
                status: EffectLifecycle::Finalized,
                prepared_at_generation: index,
                updated_at_generation: index,
            });
        }
        let projection = compile_projection(
            &materialized,
            &history,
            &request,
            &DerivedProjectionInput::default(),
        )
        .unwrap();
        assert!(projection.entries.len() <= 10);
        assert!(projection.bounds.omitted_items > 0);
        assert_eq!(projection.bounds.history_transitions_considered, 101);
    }

    #[test]
    fn denied_provider_claim_is_not_promoted_to_resource_truth() {
        let history = vec![
            transition(
                1,
                TransitionPayload::CaseOpened {
                    lifecycle: CaseLifecycle::Open,
                },
            ),
            transition(
                2,
                TransitionPayload::ProviderInvocationStarted {
                    invocation_id: "invocation:false-claim".to_string(),
                    participant_id: "participant:model".to_string(),
                    provider_id: "provider:a".to_string(),
                    provider_kind: "openai_compatible".to_string(),
                    model_id: "model:a".to_string(),
                    semantic_lineage: None,
                },
            ),
            transition(
                3,
                TransitionPayload::ProviderResultRecorded {
                    result_id: "result:false-claim".to_string(),
                    invocation_id: "invocation:false-claim".to_string(),
                    provider_id: "provider:a".to_string(),
                    provider_kind: "openai_compatible".to_string(),
                    model_id: "model:a".to_string(),
                    semantic_lineage: None,
                    output: "I created hello.txt".to_string(),
                },
            ),
        ];
        let mut materialized = state(3);
        materialized.last_decision = Some(DecisionState {
            decision_id: "decision:deny".to_string(),
            decision_digest: "decision-digest".to_string(),
            operation_id: "operation:false-claim".to_string(),
            operation_digest: "operation-digest".to_string(),
            outcome: DecisionOutcome::Deny,
            policy_id: Some("policy:test".to_string()),
            decision_basis_id: None,
            effective_policy_id: None,
            recorded_at_generation: 3,
        });
        let projection = compile_projection(
            &materialized,
            &history,
            &ProjectionRequest::model("participant:model", ProjectionPurpose::EffectConsequence),
            &DerivedProjectionInput::default(),
        )
        .unwrap();
        assert!(projection.entries.iter().any(|entry| {
            entry.posture == AuthorityPosture::ProviderClaim
                && matches!(entry.value, ProjectedValue::ProviderClaim { .. })
        }));
        assert!(projection.entries.iter().any(|entry| matches!(
            entry.value,
            ProjectedValue::DecisionOutcome {
                outcome: DecisionOutcome::Deny,
                ..
            }
        )));
        assert!(!projection
            .entries
            .iter()
            .any(|entry| matches!(entry.value, ProjectedValue::ResourceConsequence { .. })));
    }

    #[test]
    fn indeterminate_effect_remains_explicitly_unresolved() {
        let history = vec![transition(
            1,
            TransitionPayload::CaseOpened {
                lifecycle: CaseLifecycle::Open,
            },
        )];
        let mut materialized = state(1);
        materialized.effects.push(EffectState {
            effect_id: "effect:uncertain".to_string(),
            operation_id: "operation:uncertain".to_string(),
            decision_id: "decision:allow".to_string(),
            grant_id: "grant:one".to_string(),
            resource_attachment_id: "workspace".to_string(),
            relative_path: "allowed/uncertain.txt".to_string(),
            intended_content_digest: "digest:intended".to_string(),
            pre_observation_id: "observation:pre".to_string(),
            post_observation_id: None,
            receipt_id: None,
            outcome: None,
            status: EffectLifecycle::Indeterminate,
            prepared_at_generation: 1,
            updated_at_generation: 1,
        });
        let projection = compile_projection(
            &materialized,
            &history,
            &ProjectionRequest::model("participant:model", ProjectionPurpose::EffectConsequence),
            &DerivedProjectionInput::default(),
        )
        .unwrap();
        assert!(projection.entries.iter().any(|entry| {
            entry.posture == AuthorityPosture::Unresolved
                && matches!(
                    entry.value,
                    ProjectedValue::ResourceConsequence {
                        lifecycle: EffectLifecycle::Indeterminate,
                        outcome: None,
                        ..
                    }
                )
        }));
    }

    #[test]
    fn participant_visibility_excludes_other_participant_and_wrong_view() {
        let history = vec![transition(
            1,
            TransitionPayload::CaseOpened {
                lifecycle: CaseLifecycle::Open,
            },
        )];
        let mut materialized = state(1);
        materialized.participants.push(ParticipantState {
            participant_id: "participant:operator".to_string(),
            roles: vec!["operator-secret-role".to_string()],
            admitted_views: vec![AdmittedView {
                consumer: "operator".to_string(),
                view_kind: "operator_context".to_string(),
            }],
        });
        let projection = compile_projection(
            &materialized,
            &history,
            &ProjectionRequest::model("participant:model", ProjectionPurpose::Conversation),
            &DerivedProjectionInput::default(),
        )
        .unwrap();
        let serialized = serde_json::to_string(&projection).unwrap();
        assert!(!serialized.contains("operator-secret-role"));
        assert_eq!(
            compile_projection(
                &materialized,
                &history,
                &ProjectionRequest::model("participant:operator", ProjectionPurpose::Conversation,),
                &DerivedProjectionInput::default(),
            )
            .unwrap_err(),
            "projection_view_not_admitted"
        );
    }

    #[test]
    fn frame_staleness_and_render_identity_are_explicit() {
        let history = vec![transition(
            1,
            TransitionPayload::CaseOpened {
                lifecycle: CaseLifecycle::Open,
            },
        )];
        let projection = compile_projection(
            &state(1),
            &history,
            &ProjectionRequest::model("participant:model", ProjectionPurpose::Conversation),
            &DerivedProjectionInput::default(),
        )
        .unwrap();
        assert!(!projection_is_stale(&projection, 1));
        assert!(projection_is_stale(&projection, 2));
        let frame = build_context_frame(
            &projection,
            "answer current state",
            InvocationOutputContract::NaturalLanguage,
        )
        .unwrap();
        let rendered = render_openai_compatible(
            &frame,
            &ProviderModelProfile {
                provider_id: "provider:a".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:a".to_string(),
                structured_output_supported: false,
                continuation_supported: false,
            },
            "none",
        )
        .unwrap();
        assert_eq!(
            validate_frame_freshness(&frame, 2).unwrap_err(),
            "stale_context_frame: frame_generation=1 current_generation=2"
        );
        assert_ne!(projection.projection_id, frame.frame_id);
        assert_ne!(frame.frame_id, rendered.metadata.rendered_input_id);
        assert!(!rendered.user_content.contains("token_ids"));
        assert!(!rendered.user_content.contains("kv_cache"));
    }

    #[test]
    fn qualified_memory_enters_frame_with_typed_posture_and_full_provenance() {
        let history = vec![transition(
            1,
            TransitionPayload::CaseOpened {
                lifecycle: CaseLifecycle::Open,
            },
        )];
        let projection = compile_projection(
            &state(1),
            &history,
            &ProjectionRequest::model("participant:model", ProjectionPurpose::Conversation),
            &DerivedProjectionInput {
                graph_available: false,
                memory_available: true,
                memory: vec![DerivedMemoryInput {
                    memory_ref: "memory:effect".to_string(),
                    semantic_kind: "resource_effect".to_string(),
                    memory_posture: "finalized_observed_consequence".to_string(),
                    description: "workspace/hello.txt was observed at digest abc".to_string(),
                    lifecycle: "active".to_string(),
                    score: 165,
                    ranking_reasons: vec![
                        "finalized_observed_consequence:+50".to_string(),
                        "direct_resource_match:+100".to_string(),
                    ],
                    transition_refs: vec!["transition:effect-finalized".to_string()],
                    observation_refs: vec!["observation:post".to_string()],
                    receipt_refs: vec!["receipt:effect".to_string()],
                }],
                retrieval_id: Some("retrieval:test".to_string()),
                retrieval_candidates: 5,
                retrieval_omitted: 4,
            },
        )
        .expect("compile with qualified memory");
        assert_eq!(
            projection.bounds.retrieval_id.as_deref(),
            Some("retrieval:test")
        );
        assert_eq!(projection.bounds.retrieval_selected, 1);
        assert_eq!(projection.bounds.retrieval_omitted, 4);
        let memory = projection
            .entries
            .iter()
            .find(|entry| entry.entry_id == "memory:memory:effect")
            .expect("projected memory");
        assert!(matches!(
            &memory.value,
            ProjectedValue::DerivedMemory {
                semantic_kind,
                memory_posture,
                ..
            } if semantic_kind == "resource_effect"
                && memory_posture == "finalized_observed_consequence"
        ));
        assert!(memory
            .provenance
            .iter()
            .any(|item| item.kind == ProvenanceKind::Observation));
        assert!(memory
            .provenance
            .iter()
            .any(|item| item.kind == ProvenanceKind::EffectReceipt));
        let frame = build_context_frame(
            &projection,
            "continue from observed consequence",
            InvocationOutputContract::NaturalLanguage,
        )
        .expect("frame carries memory");
        assert_eq!(frame.entries, projection.entries);
    }

    #[test]
    fn unknown_projection_and_frame_versions_fail_closed() {
        let history = vec![transition(
            1,
            TransitionPayload::CaseOpened {
                lifecycle: CaseLifecycle::Open,
            },
        )];
        let mut projection = compile_projection(
            &state(1),
            &history,
            &ProjectionRequest::model("participant:model", ProjectionPurpose::Conversation),
            &DerivedProjectionInput::default(),
        )
        .unwrap();
        projection.schema = "yai.projection.v99".to_string();
        assert_eq!(
            build_context_frame(
                &projection,
                "answer current state",
                InvocationOutputContract::NaturalLanguage,
            )
            .unwrap_err(),
            "unsupported_projection_schema: yai.projection.v99"
        );

        projection.schema = PROJECTION_SCHEMA.to_string();
        let mut frame = build_context_frame(
            &projection,
            "answer current state",
            InvocationOutputContract::NaturalLanguage,
        )
        .unwrap();
        frame.schema = "yai.context_frame.v99".to_string();
        assert_eq!(
            render_openai_compatible(
                &frame,
                &ProviderModelProfile {
                    provider_id: "provider:a".to_string(),
                    provider_kind: "openai_compatible".to_string(),
                    model_id: "model:a".to_string(),
                    structured_output_supported: false,
                    continuation_supported: false,
                },
                "none",
            )
            .unwrap_err(),
            "unsupported_context_frame_schema: yai.context_frame.v99"
        );
    }
}
