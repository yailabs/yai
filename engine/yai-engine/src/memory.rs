//! Rebuildable operational memory and qualified retrieval.
//!
//! Canonical Transition history is the only input authority. Memory entries
//! compact typed operational residue for later selection; retrieval qualifies
//! them by Case, participant, view, generation and explicit resource/causal
//! refs before ranking. Neither operation mutates canonical history.

use crate::compatibility::legacy_summary_value;
use crate::context::{stable_digest, ProjectionPurpose};
use crate::effect::{
    DecisionOutcome, EffectOutcome, NormalizationFailureCode, Operation, PreparedEffect,
    ReconciliationConclusion,
};
use crate::journal::Journal;
use crate::record::RecordKind;
use crate::transition::{CaseState, Transition, TransitionPayload};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const OPERATIONAL_MEMORY_SCHEMA: &str = "yai.operational_memory.v1";
pub const OPERATIONAL_MEMORY_DERIVATION: &str = "yai.operational_memory.derivation.v1";
pub const OPERATIONAL_MEMORY_MANIFEST_SCHEMA: &str = "yai.operational_memory_manifest.v1";
pub const RETRIEVAL_SET_SCHEMA: &str = "yai.retrieval_set.v1";
pub const DEFAULT_RETRIEVAL_LIMIT: usize = 8;
pub const MAX_MEMORY_DESCRIPTION_CHARS: usize = 320;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationalMemoryKind {
    ResourceEffect,
    Decision,
    Review,
    UnresolvedEffect,
    NormalizationFailure,
    ProviderClaim,
}

impl OperationalMemoryKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ResourceEffect => "resource_effect",
            Self::Decision => "decision",
            Self::Review => "review",
            Self::UnresolvedEffect => "unresolved_effect",
            Self::NormalizationFailure => "normalization_failure",
            Self::ProviderClaim => "provider_claim",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "resource_effect" => Some(Self::ResourceEffect),
            "decision" => Some(Self::Decision),
            "review" => Some(Self::Review),
            "unresolved_effect" => Some(Self::UnresolvedEffect),
            "normalization_failure" => Some(Self::NormalizationFailure),
            "provider_claim" => Some(Self::ProviderClaim),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationalMemoryPosture {
    FinalizedObservedConsequence,
    DecisionControlHistory,
    ProviderOriginatedClaim,
    Unresolved,
}

impl OperationalMemoryPosture {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FinalizedObservedConsequence => "finalized_observed_consequence",
            Self::DecisionControlHistory => "decision_control_history",
            Self::ProviderOriginatedClaim => "provider_originated_claim",
            Self::Unresolved => "unresolved",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationalMemoryLifecycle {
    Active,
    Superseded,
}

impl OperationalMemoryLifecycle {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Superseded => "superseded",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OperationalMemoryVisibility {
    pub participant_ids: Vec<String>,
    pub consumer: String,
    pub view_kind: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OperationalMemoryProvenance {
    pub transition_ids: Vec<String>,
    #[serde(default)]
    pub observation_ids: Vec<String>,
    #[serde(default)]
    pub effect_receipt_ids: Vec<String>,
    #[serde(default)]
    pub causal_refs: Vec<String>,
    pub generation_start: u64,
    pub generation_end: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum OperationalMemoryValue {
    ResourceEffect {
        operation_id: String,
        effect_id: String,
        resource_attachment_id: String,
        relative_path: String,
        outcome: EffectOutcome,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        content_digest: Option<String>,
        receipt_id: String,
    },
    Decision {
        operation_id: String,
        decision_id: String,
        outcome: DecisionOutcome,
        resource_attachment_id: String,
        relative_path: String,
        reason: String,
    },
    Review {
        review_id: String,
        operation_id: String,
        resource_attachment_id: String,
        relative_path: String,
        reviewer_participant_id: String,
        status: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        action_id: Option<String>,
    },
    UnresolvedEffect {
        operation_id: String,
        effect_id: String,
        resource_attachment_id: String,
        relative_path: String,
        state: String,
        reason: String,
    },
    NormalizationFailure {
        provider_result_id: String,
        code: String,
        detail: String,
    },
    ProviderClaim {
        result_id: String,
        invocation_id: String,
        provider_id: String,
        model_id: String,
        preview: String,
    },
}

impl OperationalMemoryValue {
    pub fn resource_refs(&self) -> Vec<String> {
        match self {
            Self::ResourceEffect {
                resource_attachment_id,
                ..
            }
            | Self::Decision {
                resource_attachment_id,
                ..
            }
            | Self::Review {
                resource_attachment_id,
                ..
            }
            | Self::UnresolvedEffect {
                resource_attachment_id,
                ..
            } => vec![resource_attachment_id.clone()],
            Self::NormalizationFailure { .. } | Self::ProviderClaim { .. } => Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OperationalMemoryEntry {
    pub schema: String,
    pub memory_id: String,
    pub case_id: String,
    pub derivation_version: String,
    pub semantic_kind: OperationalMemoryKind,
    pub posture: OperationalMemoryPosture,
    pub value: OperationalMemoryValue,
    pub description: String,
    pub provenance: OperationalMemoryProvenance,
    pub visibility: OperationalMemoryVisibility,
    pub derived_at_generation: u64,
    pub lifecycle: OperationalMemoryLifecycle,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<String>,
}

impl OperationalMemoryEntry {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != OPERATIONAL_MEMORY_SCHEMA {
            return Err(format!(
                "unsupported_operational_memory_schema: {}",
                self.schema
            ));
        }
        require_memory_value("memory_id", &self.memory_id)?;
        require_memory_value("case_id", &self.case_id)?;
        if self.derivation_version != OPERATIONAL_MEMORY_DERIVATION {
            return Err(format!(
                "unsupported_memory_derivation: {}",
                self.derivation_version
            ));
        }
        if self.provenance.transition_ids.is_empty() {
            return Err("memory_provenance_transition_required".to_string());
        }
        if self.provenance.generation_start == 0
            || self.provenance.generation_start > self.provenance.generation_end
            || self.provenance.generation_end > self.derived_at_generation
        {
            return Err("memory_provenance_generation_invalid".to_string());
        }
        if self.visibility.participant_ids.is_empty() {
            return Err("memory_visibility_participant_required".to_string());
        }
        require_memory_value("memory_visibility.consumer", &self.visibility.consumer)?;
        require_memory_value("memory_visibility.view_kind", &self.visibility.view_kind)?;
        require_memory_value("memory_description", &self.description)?;
        if self.description.chars().count() > MAX_MEMORY_DESCRIPTION_CHARS + 3 {
            return Err("memory_description_too_large".to_string());
        }
        if self.lifecycle == OperationalMemoryLifecycle::Active && self.superseded_by.is_some() {
            return Err("active_memory_cannot_have_superseder".to_string());
        }
        if self.lifecycle == OperationalMemoryLifecycle::Superseded
            && self.superseded_by.as_deref().unwrap_or_default().is_empty()
        {
            return Err("superseded_memory_requires_replacement".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OperationalMemoryManifest {
    pub schema: String,
    pub case_id: String,
    pub derivation_version: String,
    pub source_generation: u64,
    pub memory_ids: Vec<String>,
}

impl OperationalMemoryManifest {
    pub fn is_current(&self, case_id: &str, generation: u64) -> bool {
        self.schema == OPERATIONAL_MEMORY_MANIFEST_SCHEMA
            && self.case_id == case_id
            && self.derivation_version == OPERATIONAL_MEMORY_DERIVATION
            && self.source_generation == generation
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OperationalMemoryBuild {
    pub manifest: OperationalMemoryManifest,
    pub entries: Vec<OperationalMemoryEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RetrievalQualification {
    pub case_id: String,
    pub participant_id: String,
    pub consumer: String,
    pub view_kind: String,
    pub purpose: ProjectionPurpose,
    pub case_generation: u64,
    #[serde(default)]
    pub resource_refs: Vec<String>,
    #[serde(default)]
    pub semantic_kinds: Vec<OperationalMemoryKind>,
    #[serde(default)]
    pub causal_refs: Vec<String>,
    pub max_results: usize,
    pub include_superseded: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RetrievedOperationalMemory {
    pub memory: OperationalMemoryEntry,
    pub score: i64,
    pub ranking_reasons: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct RetrievalRejections {
    pub wrong_case: usize,
    pub future_or_stale_derivation: usize,
    pub participant_visibility: usize,
    pub view_not_admitted: usize,
    pub lifecycle: usize,
    pub semantic_kind: usize,
    pub resource_qualification: usize,
    pub causal_qualification: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RetrievalSet {
    pub schema: String,
    pub retrieval_id: String,
    pub qualification: RetrievalQualification,
    pub source_memory_count: usize,
    pub qualified_count: usize,
    pub selected_count: usize,
    pub omitted_count: usize,
    pub selected: Vec<RetrievedOperationalMemory>,
    pub rejections: RetrievalRejections,
}

#[derive(Clone)]
struct InvocationOrigin {
    participant_id: String,
    transition_id: String,
    generation: u64,
}

#[derive(Clone)]
struct ProviderResultOrigin {
    invocation_id: String,
    transition_id: String,
    generation: u64,
}

#[derive(Clone)]
struct OperationOriginEntry {
    operation: Operation,
    transition_id: String,
    generation: u64,
}

#[derive(Clone)]
struct DecisionOriginEntry {
    transition_id: String,
}

#[derive(Clone)]
struct ReviewOriginEntry {
    review: crate::transition::ReviewState,
    transition_id: String,
    generation: u64,
}

#[derive(Clone)]
struct PreparedOriginEntry {
    prepared: PreparedEffect,
    transition_id: String,
    generation: u64,
}

pub fn derive_operational_memory(
    case_id: &str,
    transitions: &[Transition],
) -> Result<OperationalMemoryBuild, String> {
    require_memory_value("case_id", case_id)?;
    validate_history_identity(case_id, transitions)?;
    let derived_at_generation = transitions.last().map(|item| item.sequence).unwrap_or(0);
    let mut invocations = BTreeMap::<String, InvocationOrigin>::new();
    let mut results = BTreeMap::<String, ProviderResultOrigin>::new();
    let mut operations = BTreeMap::<String, OperationOriginEntry>::new();
    let mut decisions = BTreeMap::<String, DecisionOriginEntry>::new();
    let mut reviews = BTreeMap::<String, ReviewOriginEntry>::new();
    let mut prepared_effects = BTreeMap::<String, PreparedOriginEntry>::new();
    let mut entries = Vec::new();

    for transition in transitions {
        match &transition.payload {
            TransitionPayload::ProviderInvocationStarted {
                invocation_id,
                participant_id,
                ..
            } => {
                invocations.insert(
                    invocation_id.clone(),
                    InvocationOrigin {
                        participant_id: participant_id.clone(),
                        transition_id: transition.transition_id.clone(),
                        generation: transition.sequence,
                    },
                );
            }
            TransitionPayload::ProviderResultRecorded {
                result_id,
                invocation_id,
                provider_id,
                model_id,
                output,
                ..
            } => {
                let invocation = invocations
                    .get(invocation_id)
                    .ok_or_else(|| "memory_provider_result_without_invocation".to_string())?;
                entries.push(build_entry(
                    case_id,
                    OperationalMemoryKind::ProviderClaim,
                    OperationalMemoryPosture::ProviderOriginatedClaim,
                    OperationalMemoryValue::ProviderClaim {
                        result_id: result_id.clone(),
                        invocation_id: invocation_id.clone(),
                        provider_id: provider_id.clone(),
                        model_id: model_id.clone(),
                        preview: bounded_memory_text(output),
                    },
                    format!(
                        "Provider {provider_id} model {model_id} returned non-authoritative material: {}",
                        bounded_memory_text(output)
                    ),
                    provenance(
                        vec![invocation.transition_id.clone(), transition.transition_id.clone()],
                        Vec::new(),
                        Vec::new(),
                        vec![invocation_id.clone(), result_id.clone()],
                        invocation.generation,
                        transition.sequence,
                    ),
                    vec![invocation.participant_id.clone()],
                    derived_at_generation,
                )?);
                results.insert(
                    result_id.clone(),
                    ProviderResultOrigin {
                        invocation_id: invocation_id.clone(),
                        transition_id: transition.transition_id.clone(),
                        generation: transition.sequence,
                    },
                );
            }
            TransitionPayload::OperationNormalizationFailed {
                provider_result_id,
                failure,
            } => {
                let result = results
                    .get(provider_result_id)
                    .ok_or_else(|| "memory_normalization_without_provider_result".to_string())?;
                let invocation = invocations
                    .get(&result.invocation_id)
                    .ok_or_else(|| "memory_normalization_without_invocation".to_string())?;
                let code = normalization_code_label(&failure.code).to_string();
                entries.push(build_entry(
                    case_id,
                    OperationalMemoryKind::NormalizationFailure,
                    OperationalMemoryPosture::DecisionControlHistory,
                    OperationalMemoryValue::NormalizationFailure {
                        provider_result_id: provider_result_id.clone(),
                        code: code.clone(),
                        detail: bounded_memory_text(&failure.detail),
                    },
                    format!("Provider proposal was rejected during normalization: {code}"),
                    provenance(
                        vec![
                            result.transition_id.clone(),
                            transition.transition_id.clone(),
                        ],
                        Vec::new(),
                        Vec::new(),
                        vec![provider_result_id.clone()],
                        result.generation,
                        transition.sequence,
                    ),
                    vec![invocation.participant_id.clone()],
                    derived_at_generation,
                )?);
            }
            TransitionPayload::OperationRecorded { operation } => {
                operations.insert(
                    operation.operation_id.clone(),
                    OperationOriginEntry {
                        operation: operation.clone(),
                        transition_id: transition.transition_id.clone(),
                        generation: transition.sequence,
                    },
                );
            }
            TransitionPayload::DecisionRecorded { decision } => {
                let operation = operations
                    .get(&decision.operation_id)
                    .ok_or_else(|| "memory_decision_without_operation".to_string())?;
                entries.push(build_entry(
                    case_id,
                    OperationalMemoryKind::Decision,
                    OperationalMemoryPosture::DecisionControlHistory,
                    OperationalMemoryValue::Decision {
                        operation_id: decision.operation_id.clone(),
                        decision_id: decision.decision_id.clone(),
                        outcome: decision.outcome.clone(),
                        resource_attachment_id: operation.operation.resource_attachment_id.clone(),
                        relative_path: operation.operation.filesystem_write.relative_path.clone(),
                        reason: bounded_memory_text(&decision.reason),
                    },
                    format!(
                        "filesystem.write to {}/{} was {} by Decision {}: {}",
                        operation.operation.resource_attachment_id,
                        operation.operation.filesystem_write.relative_path,
                        decision_outcome_label(&decision.outcome),
                        decision.decision_id,
                        bounded_memory_text(&decision.reason)
                    ),
                    provenance(
                        vec![
                            operation.transition_id.clone(),
                            transition.transition_id.clone(),
                        ],
                        Vec::new(),
                        Vec::new(),
                        vec![decision.operation_id.clone(), decision.decision_id.clone()],
                        operation.generation,
                        transition.sequence,
                    ),
                    vec![operation.operation.participant_id.clone()],
                    derived_at_generation,
                )?);
                decisions.insert(
                    decision.decision_id.clone(),
                    DecisionOriginEntry {
                        transition_id: transition.transition_id.clone(),
                    },
                );
            }
            TransitionPayload::ReviewRequested { review } if !review.operation_id.is_empty() => {
                let operation = operations
                    .get(&review.operation_id)
                    .ok_or_else(|| "memory_review_without_operation".to_string())?;
                entries.push(build_entry(
                    case_id,
                    OperationalMemoryKind::Review,
                    OperationalMemoryPosture::Unresolved,
                    OperationalMemoryValue::Review {
                        review_id: review.review_id.clone(),
                        operation_id: review.operation_id.clone(),
                        resource_attachment_id: review.resource_attachment_id.clone(),
                        relative_path: review.normalized_target.clone(),
                        reviewer_participant_id: review.reviewer_participant.clone(),
                        status: "pending".to_string(),
                        action_id: None,
                    },
                    format!(
                        "Operation {} awaits review {} by participant {}",
                        review.operation_id, review.review_id, review.reviewer_participant
                    ),
                    provenance(
                        vec![
                            operation.transition_id.clone(),
                            transition.transition_id.clone(),
                        ],
                        Vec::new(),
                        Vec::new(),
                        vec![
                            review.operation_id.clone(),
                            review.initial_decision_id.clone(),
                            review.review_id.clone(),
                        ],
                        operation.generation,
                        transition.sequence,
                    ),
                    vec![
                        operation.operation.participant_id.clone(),
                        review.reviewer_participant.clone(),
                    ],
                    derived_at_generation,
                )?);
                reviews.insert(
                    review.review_id.clone(),
                    ReviewOriginEntry {
                        review: review.clone(),
                        transition_id: transition.transition_id.clone(),
                        generation: transition.sequence,
                    },
                );
            }
            TransitionPayload::ReviewActionRecorded { action } => {
                let review = reviews
                    .get(&action.review_id)
                    .ok_or_else(|| "memory_review_action_without_request".to_string())?;
                let operation = operations
                    .get(&action.operation_id)
                    .ok_or_else(|| "memory_review_action_without_operation".to_string())?;
                let status = match action.action {
                    crate::transition::ReviewActionKind::Approve => "approved",
                    crate::transition::ReviewActionKind::Deny => "denied",
                    crate::transition::ReviewActionKind::Defer => "deferred",
                };
                let posture = if action.action == crate::transition::ReviewActionKind::Defer {
                    OperationalMemoryPosture::Unresolved
                } else {
                    OperationalMemoryPosture::DecisionControlHistory
                };
                entries.push(build_entry(
                    case_id,
                    OperationalMemoryKind::Review,
                    posture,
                    OperationalMemoryValue::Review {
                        review_id: action.review_id.clone(),
                        operation_id: action.operation_id.clone(),
                        resource_attachment_id: review.review.resource_attachment_id.clone(),
                        relative_path: review.review.normalized_target.clone(),
                        reviewer_participant_id: action.reviewer_participant_id.clone(),
                        status: status.to_string(),
                        action_id: Some(action.action_id.clone()),
                    },
                    format!(
                        "Review {} for Operation {} was {} by participant {}: {}",
                        action.review_id,
                        action.operation_id,
                        status,
                        action.reviewer_participant_id,
                        bounded_memory_text(&action.reason)
                    ),
                    provenance(
                        vec![
                            review.transition_id.clone(),
                            transition.transition_id.clone(),
                        ],
                        Vec::new(),
                        Vec::new(),
                        vec![
                            action.review_id.clone(),
                            action.operation_id.clone(),
                            action.action_id.clone(),
                        ],
                        review.generation,
                        transition.sequence,
                    ),
                    vec![
                        operation.operation.participant_id.clone(),
                        action.reviewer_participant_id.clone(),
                    ],
                    derived_at_generation,
                )?);
            }
            TransitionPayload::EffectPrepared { prepared } => {
                let operation = operations
                    .get(&prepared.operation_id)
                    .ok_or_else(|| "memory_prepare_without_operation".to_string())?;
                let decision = decisions
                    .get(&prepared.decision_id)
                    .ok_or_else(|| "memory_prepare_without_decision".to_string())?;
                entries.push(unresolved_entry(
                    case_id,
                    prepared,
                    "prepared",
                    "carrier result not yet finalized",
                    vec![
                        operation.transition_id.clone(),
                        decision.transition_id.clone(),
                        transition.transition_id.clone(),
                    ],
                    vec![prepared.expected_pre_observation.observation_id.clone()],
                    operation.generation,
                    transition.sequence,
                    derived_at_generation,
                )?);
                prepared_effects.insert(
                    prepared.effect_id.clone(),
                    PreparedOriginEntry {
                        prepared: prepared.clone(),
                        transition_id: transition.transition_id.clone(),
                        generation: transition.sequence,
                    },
                );
            }
            TransitionPayload::EffectIndeterminate {
                effect_id,
                reason,
                observation,
            } => {
                let prepared = prepared_effects
                    .get(effect_id)
                    .ok_or_else(|| "memory_indeterminate_without_prepare".to_string())?;
                let mut observation_ids = vec![prepared
                    .prepared
                    .expected_pre_observation
                    .observation_id
                    .clone()];
                if let Some(observation) = observation {
                    observation_ids.push(observation.observation_id.clone());
                }
                entries.push(unresolved_entry(
                    case_id,
                    &prepared.prepared,
                    "indeterminate",
                    reason,
                    vec![
                        prepared.transition_id.clone(),
                        transition.transition_id.clone(),
                    ],
                    observation_ids,
                    prepared.generation,
                    transition.sequence,
                    derived_at_generation,
                )?);
            }
            TransitionPayload::EffectFinalized {
                effect_id,
                post_observation,
                receipt,
            } => {
                let prepared = prepared_effects
                    .get(effect_id)
                    .ok_or_else(|| "memory_finalize_without_prepare".to_string())?;
                entries.push(finalized_entry(
                    case_id,
                    prepared,
                    transition,
                    post_observation.content_digest.clone(),
                    receipt.outcome.clone(),
                    &post_observation.observation_id,
                    &receipt.receipt_id,
                    derived_at_generation,
                )?);
            }
            TransitionPayload::EffectReconciled {
                effect_id,
                conclusion,
                observation,
                receipt,
            } => {
                let prepared = prepared_effects
                    .get(effect_id)
                    .ok_or_else(|| "memory_reconcile_without_prepare".to_string())?;
                if let Some(receipt) = receipt {
                    entries.push(finalized_entry(
                        case_id,
                        prepared,
                        transition,
                        observation.content_digest.clone(),
                        receipt.outcome.clone(),
                        &observation.observation_id,
                        &receipt.receipt_id,
                        derived_at_generation,
                    )?);
                } else {
                    entries.push(unresolved_entry(
                        case_id,
                        &prepared.prepared,
                        reconciliation_label(conclusion),
                        "reconciliation did not establish a terminal outcome",
                        vec![
                            prepared.transition_id.clone(),
                            transition.transition_id.clone(),
                        ],
                        vec![
                            prepared
                                .prepared
                                .expected_pre_observation
                                .observation_id
                                .clone(),
                            observation.observation_id.clone(),
                        ],
                        prepared.generation,
                        transition.sequence,
                        derived_at_generation,
                    )?);
                }
            }
            _ => {}
        }
    }

    apply_effect_supersession(&mut entries);
    apply_review_supersession(&mut entries);
    apply_resource_state_supersession(&mut entries);
    entries.sort_by(|left, right| {
        left.provenance
            .generation_end
            .cmp(&right.provenance.generation_end)
            .then_with(|| left.memory_id.cmp(&right.memory_id))
    });
    for entry in &entries {
        entry.validate()?;
        validate_memory_provenance(entry, transitions)?;
    }
    let memory_ids = entries
        .iter()
        .map(|entry| entry.memory_id.clone())
        .collect();
    Ok(OperationalMemoryBuild {
        manifest: OperationalMemoryManifest {
            schema: OPERATIONAL_MEMORY_MANIFEST_SCHEMA.to_string(),
            case_id: case_id.to_string(),
            derivation_version: OPERATIONAL_MEMORY_DERIVATION.to_string(),
            source_generation: derived_at_generation,
            memory_ids,
        },
        entries,
    })
}

pub fn validate_memory_provenance(
    entry: &OperationalMemoryEntry,
    transitions: &[Transition],
) -> Result<(), String> {
    entry.validate()?;
    let source = transitions
        .iter()
        .map(|transition| (transition.transition_id.as_str(), transition))
        .collect::<BTreeMap<_, _>>();
    for transition_id in &entry.provenance.transition_ids {
        let transition = source
            .get(transition_id.as_str())
            .ok_or_else(|| format!("memory_source_transition_missing: {transition_id}"))?;
        if transition.case_id != entry.case_id {
            return Err("memory_source_case_mismatch".to_string());
        }
    }
    let observation_ids = transitions
        .iter()
        .flat_map(transition_observation_ids)
        .collect::<BTreeSet<_>>();
    for observation_id in &entry.provenance.observation_ids {
        if !observation_ids.contains(observation_id.as_str()) {
            return Err(format!(
                "memory_source_observation_missing: {observation_id}"
            ));
        }
    }
    let receipt_ids = transitions
        .iter()
        .filter_map(transition_receipt_id)
        .collect::<BTreeSet<_>>();
    for receipt_id in &entry.provenance.effect_receipt_ids {
        if !receipt_ids.contains(receipt_id.as_str()) {
            return Err(format!("memory_source_receipt_missing: {receipt_id}"));
        }
    }
    Ok(())
}

pub fn retrieve_operational_memory(
    state: &CaseState,
    entries: &[OperationalMemoryEntry],
    qualification: RetrievalQualification,
) -> Result<RetrievalSet, String> {
    if qualification.max_results == 0 {
        return Err("retrieval_max_results_must_be_positive".to_string());
    }
    if qualification.case_id != state.case_id || qualification.case_generation != state.generation {
        return Err("retrieval_case_generation_mismatch".to_string());
    }
    let participant = state
        .participants
        .iter()
        .find(|participant| participant.participant_id == qualification.participant_id)
        .ok_or_else(|| "retrieval_participant_not_bound".to_string())?;
    if !participant.admitted_views.iter().any(|view| {
        view.consumer == qualification.consumer && view.view_kind == qualification.view_kind
    }) {
        return Err("retrieval_view_not_admitted".to_string());
    }

    let mut rejections = RetrievalRejections::default();
    let mut qualified = Vec::new();
    for entry in entries {
        if entry.case_id != qualification.case_id {
            rejections.wrong_case += 1;
            continue;
        }
        if entry.derivation_version != OPERATIONAL_MEMORY_DERIVATION
            || entry.derived_at_generation != qualification.case_generation
        {
            rejections.future_or_stale_derivation += 1;
            continue;
        }
        if !entry
            .visibility
            .participant_ids
            .iter()
            .any(|participant_id| participant_id == &qualification.participant_id)
        {
            rejections.participant_visibility += 1;
            continue;
        }
        if entry.visibility.consumer != qualification.consumer
            || entry.visibility.view_kind != qualification.view_kind
        {
            rejections.view_not_admitted += 1;
            continue;
        }
        if !qualification.include_superseded
            && entry.lifecycle != OperationalMemoryLifecycle::Active
        {
            rejections.lifecycle += 1;
            continue;
        }
        if !qualification.semantic_kinds.is_empty()
            && !qualification.semantic_kinds.contains(&entry.semantic_kind)
        {
            rejections.semantic_kind += 1;
            continue;
        }
        let entry_resources = entry.value.resource_refs();
        if !qualification.resource_refs.is_empty()
            && !entry_resources
                .iter()
                .any(|value| qualification.resource_refs.contains(value))
        {
            rejections.resource_qualification += 1;
            continue;
        }
        if !qualification.causal_refs.is_empty()
            && !entry
                .provenance
                .causal_refs
                .iter()
                .any(|value| qualification.causal_refs.contains(value))
        {
            rejections.causal_qualification += 1;
            continue;
        }
        let (score, ranking_reasons) = rank_memory(entry, &qualification);
        qualified.push(RetrievedOperationalMemory {
            memory: entry.clone(),
            score,
            ranking_reasons,
        });
    }
    qualified.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| {
                right
                    .memory
                    .provenance
                    .generation_end
                    .cmp(&left.memory.provenance.generation_end)
            })
            .then_with(|| left.memory.memory_id.cmp(&right.memory.memory_id))
    });
    let qualified_count = qualified.len();
    let selected = qualified
        .into_iter()
        .take(qualification.max_results)
        .collect::<Vec<_>>();
    let omitted_count = qualified_count.saturating_sub(selected.len());
    let identity = serde_json::to_string(&(
        RETRIEVAL_SET_SCHEMA,
        &qualification,
        selected
            .iter()
            .map(|item| (&item.memory.memory_id, item.score, &item.ranking_reasons))
            .collect::<Vec<_>>(),
        omitted_count,
    ))
    .map_err(|error| format!("retrieval_identity_encode_failed: {error}"))?;
    Ok(RetrievalSet {
        schema: RETRIEVAL_SET_SCHEMA.to_string(),
        retrieval_id: format!("retrieval:{}", stable_digest(&identity)),
        qualification,
        source_memory_count: entries.len(),
        qualified_count,
        selected_count: selected.len(),
        omitted_count,
        selected,
        rejections,
    })
}

fn rank_memory(
    entry: &OperationalMemoryEntry,
    qualification: &RetrievalQualification,
) -> (i64, Vec<String>) {
    let mut score = 0i64;
    let mut reasons = Vec::new();
    if entry.lifecycle == OperationalMemoryLifecycle::Active {
        score += 30;
        reasons.push("active:+30".to_string());
    }
    match entry.posture {
        OperationalMemoryPosture::FinalizedObservedConsequence => {
            score += 50;
            reasons.push("finalized_observed_consequence:+50".to_string());
        }
        OperationalMemoryPosture::Unresolved => {
            score += 40;
            reasons.push("unresolved_operational_state:+40".to_string());
        }
        OperationalMemoryPosture::DecisionControlHistory => {
            score += 25;
            reasons.push("decision_control_history:+25".to_string());
        }
        OperationalMemoryPosture::ProviderOriginatedClaim => {
            reasons.push("provider_claim:+0".to_string());
        }
    }
    let purpose_score = match qualification.purpose {
        ProjectionPurpose::FilesystemWriteProposal => match entry.semantic_kind {
            OperationalMemoryKind::Review => 65,
            OperationalMemoryKind::Decision => 60,
            OperationalMemoryKind::ResourceEffect => 50,
            OperationalMemoryKind::NormalizationFailure => 40,
            OperationalMemoryKind::UnresolvedEffect => 35,
            OperationalMemoryKind::ProviderClaim => 5,
        },
        ProjectionPurpose::EffectConsequence => match entry.semantic_kind {
            OperationalMemoryKind::UnresolvedEffect => 70,
            OperationalMemoryKind::Review => 60,
            OperationalMemoryKind::ResourceEffect => 65,
            OperationalMemoryKind::Decision => 45,
            OperationalMemoryKind::NormalizationFailure => 20,
            OperationalMemoryKind::ProviderClaim => 5,
        },
        ProjectionPurpose::Conversation => match entry.semantic_kind {
            OperationalMemoryKind::ResourceEffect => 50,
            OperationalMemoryKind::Review => 45,
            OperationalMemoryKind::Decision => 35,
            OperationalMemoryKind::UnresolvedEffect => 35,
            OperationalMemoryKind::NormalizationFailure => 20,
            OperationalMemoryKind::ProviderClaim => 5,
        },
        ProjectionPurpose::Inspection => 30,
    };
    score += purpose_score;
    reasons.push(format!("purpose_match:+{purpose_score}"));
    if !qualification.resource_refs.is_empty() {
        score += 100;
        reasons.push("direct_resource_match:+100".to_string());
    }
    if !qualification.causal_refs.is_empty() {
        score += 120;
        reasons.push("direct_causal_match:+120".to_string());
    }
    let age = qualification
        .case_generation
        .saturating_sub(entry.provenance.generation_end);
    let recency_score = if age <= 10 {
        20
    } else if age <= 50 {
        10
    } else {
        0
    };
    score += recency_score;
    reasons.push(format!("recency_age_{age}:+{recency_score}"));
    (score, reasons)
}

fn build_entry(
    case_id: &str,
    semantic_kind: OperationalMemoryKind,
    posture: OperationalMemoryPosture,
    value: OperationalMemoryValue,
    description: String,
    provenance: OperationalMemoryProvenance,
    mut participant_ids: Vec<String>,
    derived_at_generation: u64,
) -> Result<OperationalMemoryEntry, String> {
    participant_ids.sort();
    participant_ids.dedup();
    let identity = serde_json::to_string(&(
        OPERATIONAL_MEMORY_SCHEMA,
        OPERATIONAL_MEMORY_DERIVATION,
        case_id,
        &semantic_kind,
        &value,
        &provenance.transition_ids,
    ))
    .map_err(|error| format!("memory_identity_encode_failed: {error}"))?;
    let entry = OperationalMemoryEntry {
        schema: OPERATIONAL_MEMORY_SCHEMA.to_string(),
        memory_id: format!("memory:{}", stable_digest(&identity)),
        case_id: case_id.to_string(),
        derivation_version: OPERATIONAL_MEMORY_DERIVATION.to_string(),
        semantic_kind,
        posture,
        value,
        description: bounded_memory_text(&description),
        provenance,
        visibility: OperationalMemoryVisibility {
            participant_ids,
            consumer: "model".to_string(),
            view_kind: "model_context".to_string(),
        },
        derived_at_generation,
        lifecycle: OperationalMemoryLifecycle::Active,
        superseded_by: None,
    };
    entry.validate()?;
    Ok(entry)
}

fn unresolved_entry(
    case_id: &str,
    prepared: &PreparedEffect,
    state: &str,
    reason: &str,
    transition_ids: Vec<String>,
    observation_ids: Vec<String>,
    generation_start: u64,
    generation_end: u64,
    derived_at_generation: u64,
) -> Result<OperationalMemoryEntry, String> {
    build_entry(
        case_id,
        OperationalMemoryKind::UnresolvedEffect,
        OperationalMemoryPosture::Unresolved,
        OperationalMemoryValue::UnresolvedEffect {
            operation_id: prepared.operation_id.clone(),
            effect_id: prepared.effect_id.clone(),
            resource_attachment_id: prepared.resource_attachment_id.clone(),
            relative_path: prepared.relative_path.clone(),
            state: state.to_string(),
            reason: bounded_memory_text(reason),
        },
        format!(
            "Effect {} on {}/{} remains {}: {}",
            prepared.effect_id,
            prepared.resource_attachment_id,
            prepared.relative_path,
            state,
            bounded_memory_text(reason)
        ),
        provenance(
            transition_ids,
            observation_ids,
            Vec::new(),
            vec![
                prepared.operation_id.clone(),
                prepared.decision_id.clone(),
                prepared.grant_id.clone(),
                prepared.effect_id.clone(),
            ],
            generation_start,
            generation_end,
        ),
        vec![prepared.participant_id.clone()],
        derived_at_generation,
    )
}

fn finalized_entry(
    case_id: &str,
    prepared: &PreparedOriginEntry,
    transition: &Transition,
    content_digest: Option<String>,
    outcome: EffectOutcome,
    post_observation_id: &str,
    receipt_id: &str,
    derived_at_generation: u64,
) -> Result<OperationalMemoryEntry, String> {
    build_entry(
        case_id,
        OperationalMemoryKind::ResourceEffect,
        OperationalMemoryPosture::FinalizedObservedConsequence,
        OperationalMemoryValue::ResourceEffect {
            operation_id: prepared.prepared.operation_id.clone(),
            effect_id: prepared.prepared.effect_id.clone(),
            resource_attachment_id: prepared.prepared.resource_attachment_id.clone(),
            relative_path: prepared.prepared.relative_path.clone(),
            outcome: outcome.clone(),
            content_digest: content_digest.clone(),
            receipt_id: receipt_id.to_string(),
        },
        format!(
            "Observed filesystem.write consequence on {}/{}: outcome={} digest={}",
            prepared.prepared.resource_attachment_id,
            prepared.prepared.relative_path,
            effect_outcome_label(&outcome),
            content_digest.as_deref().unwrap_or("none")
        ),
        provenance(
            vec![
                prepared.transition_id.clone(),
                transition.transition_id.clone(),
            ],
            vec![
                prepared
                    .prepared
                    .expected_pre_observation
                    .observation_id
                    .clone(),
                post_observation_id.to_string(),
            ],
            vec![receipt_id.to_string()],
            vec![
                prepared.prepared.operation_id.clone(),
                prepared.prepared.decision_id.clone(),
                prepared.prepared.grant_id.clone(),
                prepared.prepared.effect_id.clone(),
                receipt_id.to_string(),
            ],
            prepared.generation,
            transition.sequence,
        ),
        vec![prepared.prepared.participant_id.clone()],
        derived_at_generation,
    )
}

fn apply_effect_supersession(entries: &mut [OperationalMemoryEntry]) {
    let mut by_effect = BTreeMap::<String, Vec<usize>>::new();
    for (index, entry) in entries.iter().enumerate() {
        let effect_id = match &entry.value {
            OperationalMemoryValue::ResourceEffect { effect_id, .. }
            | OperationalMemoryValue::UnresolvedEffect { effect_id, .. } => Some(effect_id),
            _ => None,
        };
        if let Some(effect_id) = effect_id {
            by_effect.entry(effect_id.clone()).or_default().push(index);
        }
    }
    for indexes in by_effect.values_mut() {
        indexes.sort_by_key(|index| entries[*index].provenance.generation_end);
        if let Some(latest) = indexes.last().copied() {
            let replacement = entries[latest].memory_id.clone();
            for index in indexes.iter().copied().filter(|index| *index != latest) {
                entries[index].lifecycle = OperationalMemoryLifecycle::Superseded;
                entries[index].superseded_by = Some(replacement.clone());
            }
        }
    }
}

fn apply_review_supersession(entries: &mut [OperationalMemoryEntry]) {
    let mut by_review = BTreeMap::<String, Vec<usize>>::new();
    for (index, entry) in entries.iter().enumerate() {
        if let OperationalMemoryValue::Review { review_id, .. } = &entry.value {
            by_review.entry(review_id.clone()).or_default().push(index);
        }
    }
    for indexes in by_review.values_mut() {
        indexes.sort_by_key(|index| entries[*index].provenance.generation_end);
        if let Some(latest) = indexes.last().copied() {
            let replacement = entries[latest].memory_id.clone();
            for index in indexes.iter().copied().filter(|index| *index != latest) {
                entries[index].lifecycle = OperationalMemoryLifecycle::Superseded;
                entries[index].superseded_by = Some(replacement.clone());
            }
        }
    }
}

fn apply_resource_state_supersession(entries: &mut [OperationalMemoryEntry]) {
    let mut current = BTreeMap::<(String, String), usize>::new();
    let mut ordered = entries
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| match &entry.value {
            OperationalMemoryValue::ResourceEffect {
                resource_attachment_id,
                relative_path,
                outcome: EffectOutcome::Applied | EffectOutcome::AlreadyApplied,
                ..
            } => Some((
                entry.provenance.generation_end,
                index,
                (resource_attachment_id.clone(), relative_path.clone()),
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    ordered.sort_by_key(|(generation, _, _)| *generation);
    for (_, index, key) in ordered {
        if let Some(previous) = current.insert(key, index) {
            let replacement = entries[index].memory_id.clone();
            entries[previous].lifecycle = OperationalMemoryLifecycle::Superseded;
            entries[previous].superseded_by = Some(replacement);
        }
    }
}

fn provenance(
    transition_ids: Vec<String>,
    observation_ids: Vec<String>,
    effect_receipt_ids: Vec<String>,
    causal_refs: Vec<String>,
    generation_start: u64,
    generation_end: u64,
) -> OperationalMemoryProvenance {
    OperationalMemoryProvenance {
        transition_ids: unique(transition_ids),
        observation_ids: unique(observation_ids),
        effect_receipt_ids: unique(effect_receipt_ids),
        causal_refs: unique(causal_refs),
        generation_start,
        generation_end,
    }
}

fn unique(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    values
        .into_iter()
        .filter(|value| seen.insert(value.clone()))
        .collect()
}

fn validate_history_identity(case_id: &str, transitions: &[Transition]) -> Result<(), String> {
    let mut expected = 1u64;
    for transition in transitions {
        if transition.case_id != case_id {
            return Err("memory_history_case_mismatch".to_string());
        }
        if transition.sequence != expected {
            return Err(format!(
                "memory_history_sequence_mismatch: expected={expected} actual={}",
                transition.sequence
            ));
        }
        expected += 1;
    }
    Ok(())
}

fn transition_observation_ids(transition: &Transition) -> Vec<&str> {
    match &transition.payload {
        TransitionPayload::EffectPrepared { prepared } => {
            vec![prepared.expected_pre_observation.observation_id.as_str()]
        }
        TransitionPayload::EffectFinalized {
            post_observation, ..
        } => vec![post_observation.observation_id.as_str()],
        TransitionPayload::EffectIndeterminate { observation, .. } => observation
            .as_ref()
            .map(|observation| vec![observation.observation_id.as_str()])
            .unwrap_or_default(),
        TransitionPayload::EffectReconciled { observation, .. } => {
            vec![observation.observation_id.as_str()]
        }
        _ => Vec::new(),
    }
}

fn transition_receipt_id(transition: &Transition) -> Option<&str> {
    match &transition.payload {
        TransitionPayload::EffectFinalized { receipt, .. } => Some(receipt.receipt_id.as_str()),
        TransitionPayload::EffectReconciled { receipt, .. } => {
            receipt.as_ref().map(|receipt| receipt.receipt_id.as_str())
        }
        _ => None,
    }
}

fn require_memory_value(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{field}_required"))
    } else {
        Ok(())
    }
}

fn bounded_memory_text(value: &str) -> String {
    let compact = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut output = compact
        .chars()
        .take(MAX_MEMORY_DESCRIPTION_CHARS)
        .collect::<String>();
    if compact.chars().count() > MAX_MEMORY_DESCRIPTION_CHARS {
        output.push_str("...");
    }
    output
}

fn normalization_code_label(code: &NormalizationFailureCode) -> &'static str {
    match code {
        NormalizationFailureCode::MalformedJson => "malformed_json",
        NormalizationFailureCode::UnsupportedSchema => "unsupported_schema",
        NormalizationFailureCode::UnsupportedOperation => "unsupported_operation",
        NormalizationFailureCode::AttachmentMismatch => "attachment_mismatch",
        NormalizationFailureCode::InvalidRelativePath => "invalid_relative_path",
        NormalizationFailureCode::EmptyContent => "empty_content",
        NormalizationFailureCode::PayloadTooLarge => "payload_too_large",
    }
}

fn decision_outcome_label(outcome: &DecisionOutcome) -> &'static str {
    match outcome {
        DecisionOutcome::Allow => "allowed",
        DecisionOutcome::Deny => "denied",
        DecisionOutcome::RequireReview => "requires_review",
    }
}

fn effect_outcome_label(outcome: &EffectOutcome) -> &'static str {
    match outcome {
        EffectOutcome::Applied => "applied",
        EffectOutcome::AlreadyApplied => "already_applied",
        EffectOutcome::NoEffect => "no_effect",
        EffectOutcome::FailedNoEffect => "failed_no_effect",
        EffectOutcome::Conflict => "conflict",
        EffectOutcome::Indeterminate => "indeterminate",
    }
}

fn reconciliation_label(conclusion: &ReconciliationConclusion) -> &'static str {
    match conclusion {
        ReconciliationConclusion::EffectObserved => "effect_observed",
        ReconciliationConclusion::NoEffectObserved => "no_effect_observed",
        ReconciliationConclusion::Conflict => "conflict",
        ReconciliationConclusion::StillIndeterminate => "still_indeterminate",
    }
}

// Historical MemoryCandidate compatibility. These helpers never feed the
// typed provider path; they remain only for old journal inspection/tests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemorySummary {
    pub records: usize,
    pub memory_candidates: usize,
    pub operational: usize,
    pub decision: usize,
    pub subject: usize,
    pub error: usize,
    pub recovery: usize,
}

impl MemorySummary {
    pub fn from_journal(journal: &Journal) -> Self {
        let mut summary = Self {
            records: journal.count(),
            memory_candidates: 0,
            operational: 0,
            decision: 0,
            subject: 0,
            error: 0,
            recovery: 0,
        };
        for record in journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::MemoryCandidate)
        {
            summary.memory_candidates += 1;
            match legacy_summary_value(&record.summary, "memory").as_deref() {
                Some("operational") => summary.operational += 1,
                Some("decision") => summary.decision += 1,
                Some("subject") => summary.subject += 1,
                Some("error") => summary.error += 1,
                Some("recovery") => summary.recovery += 1,
                _ => {}
            }
        }
        summary
    }
}

pub fn derive_memory_note(journal: &Journal) -> String {
    format!("memory:candidate records:{}", journal.count())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::{
        normalize_filesystem_write_candidate, record_filesystem_decision, EffectReceipt,
        FilesystemObservation, NormalizationContext, PreparedEffect, ResourceState,
        EFFECT_RECEIPT_SCHEMA, FILESYSTEM_CARRIER_BACKEND, OBSERVATION_SCHEMA,
        PREPARED_EFFECT_SCHEMA,
    };
    use crate::transition::{
        replay_case, CaseLifecycle, ProviderInvocationLineage, ResourceAttachmentState,
        ResourceKind, TransitionSource, TRANSITION_SCHEMA,
    };

    const CASE: &str = "case:memory";
    const PARTICIPANT_A: &str = "participant:a";
    const PARTICIPANT_B: &str = "participant:b";

    fn transition(sequence: u64, payload: TransitionPayload) -> Transition {
        Transition {
            schema: TRANSITION_SCHEMA.to_string(),
            transition_id: format!("transition:{sequence}"),
            case_id: CASE.to_string(),
            sequence,
            committed_at_unix_ms: sequence,
            source: TransitionSource::component("memory-test"),
            scope: None,
            causal_refs: Vec::new(),
            payload,
            provenance: Vec::new(),
            summary: None,
        }
    }

    fn resource() -> ResourceAttachmentState {
        ResourceAttachmentState {
            attachment_id: "workspace".to_string(),
            kind: ResourceKind::Filesystem,
            allowed_write_prefix: "allowed".to_string(),
            max_write_bytes: 1024,
            policy_id: "policy:workspace".to_string(),
            policy_owner_participant_id: PARTICIPANT_A.to_string(),
            review_requirement: crate::transition::ReviewRequirement::Automatic,
        }
    }

    fn lineage(case_generation: u64) -> ProviderInvocationLineage {
        ProviderInvocationLineage {
            projection_id: format!("projection:{case_generation}"),
            context_frame_id: format!("context-frame:{case_generation}"),
            case_generation,
            rendered_input_id: format!("rendered-input:{case_generation}"),
            rendered_input_digest: format!("digest:{case_generation}"),
            output_contract_id: "output-contract:test".to_string(),
            continuation_disposition: "not_provided".to_string(),
        }
    }

    fn admitted_history() -> Vec<Transition> {
        let mut history = vec![
            transition(
                1,
                TransitionPayload::CaseOpened {
                    lifecycle: CaseLifecycle::Open,
                },
            ),
            transition(
                2,
                TransitionPayload::ParticipantBound {
                    participant_id: PARTICIPANT_A.to_string(),
                    role: "model_provider".to_string(),
                },
            ),
            transition(
                3,
                TransitionPayload::ParticipantAdmitted {
                    participant_id: PARTICIPANT_A.to_string(),
                    consumer: "model".to_string(),
                    view_kind: "model_context".to_string(),
                },
            ),
            transition(
                4,
                TransitionPayload::ParticipantBound {
                    participant_id: PARTICIPANT_B.to_string(),
                    role: "observer".to_string(),
                },
            ),
            transition(
                5,
                TransitionPayload::ParticipantAdmitted {
                    participant_id: PARTICIPANT_B.to_string(),
                    consumer: "model".to_string(),
                    view_kind: "model_context".to_string(),
                },
            ),
            transition(
                6,
                TransitionPayload::ProviderAttached {
                    participant_id: PARTICIPANT_A.to_string(),
                    provider_id: "provider:a".to_string(),
                    provider_kind: "openai_compatible".to_string(),
                    base_url: "http://127.0.0.1:1".to_string(),
                    model_id: "model:a".to_string(),
                    credential_ref: "none".to_string(),
                },
            ),
        ];
        let mut attachment = transition(
            7,
            TransitionPayload::ResourceAttached {
                attachment: resource(),
            },
        );
        attachment.causal_refs = vec![PARTICIPANT_A.to_string()];
        history.push(attachment);
        history
    }

    fn false_claim_denied_history() -> Vec<Transition> {
        let mut history = admitted_history();
        history.push(transition(
            8,
            TransitionPayload::ProviderInvocationStarted {
                invocation_id: "invocation:false-claim".to_string(),
                participant_id: PARTICIPANT_A.to_string(),
                provider_id: "provider:a".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:a".to_string(),
                semantic_lineage: Some(lineage(7)),
            },
        ));
        history.push(transition(
            9,
            TransitionPayload::ProviderResultRecorded {
                result_id: "result:false-claim".to_string(),
                invocation_id: "invocation:false-claim".to_string(),
                provider_id: "provider:a".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:a".to_string(),
                semantic_lineage: Some(lineage(7)),
                output: "I wrote outside/claimed.txt successfully".to_string(),
            },
        ));
        let operation = normalize_filesystem_write_candidate(
            r#"{"schema":"yai.operation_proposal.filesystem_write.v1","operation":"filesystem.write","resource":"workspace","path":"outside/claimed.txt","content":"not written"}"#,
            &NormalizationContext {
                case_id: CASE,
                participant_id: PARTICIPANT_A,
                provider_result_id: "result:false-claim",
                provider_invocation_id: "invocation:false-claim",
                case_generation: 9,
                resource: &resource(),
            },
        )
        .expect("typed operation");
        history.push(transition(
            10,
            TransitionPayload::OperationRecorded {
                operation: operation.clone(),
            },
        ));
        let decision = record_filesystem_decision(
            &operation,
            &resource(),
            10,
            DecisionOutcome::Deny,
            "outside configured write prefix",
        )
        .expect("typed denial");
        history.push(transition(
            11,
            TransitionPayload::DecisionRecorded { decision },
        ));
        history
    }

    fn qualification(
        state: &CaseState,
        participant_id: &str,
        max_results: usize,
    ) -> RetrievalQualification {
        RetrievalQualification {
            case_id: CASE.to_string(),
            participant_id: participant_id.to_string(),
            consumer: "model".to_string(),
            view_kind: "model_context".to_string(),
            purpose: ProjectionPurpose::Conversation,
            case_generation: state.generation,
            resource_refs: Vec::new(),
            semantic_kinds: Vec::new(),
            causal_refs: Vec::new(),
            max_results,
            include_superseded: false,
        }
    }

    fn unresolved_history() -> Vec<Transition> {
        let mut history = admitted_history();
        history.push(transition(
            8,
            TransitionPayload::ProviderInvocationStarted {
                invocation_id: "invocation:effect".to_string(),
                participant_id: PARTICIPANT_A.to_string(),
                provider_id: "provider:a".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:a".to_string(),
                semantic_lineage: Some(lineage(7)),
            },
        ));
        history.push(transition(
            9,
            TransitionPayload::ProviderResultRecorded {
                result_id: "result:effect".to_string(),
                invocation_id: "invocation:effect".to_string(),
                provider_id: "provider:a".to_string(),
                provider_kind: "openai_compatible".to_string(),
                model_id: "model:a".to_string(),
                semantic_lineage: Some(lineage(7)),
                output: "structured proposal".to_string(),
            },
        ));
        let operation = normalize_filesystem_write_candidate(
            r#"{"schema":"yai.operation_proposal.filesystem_write.v1","operation":"filesystem.write","resource":"workspace","path":"allowed/reconcile.txt","content":"observed"}"#,
            &NormalizationContext {
                case_id: CASE,
                participant_id: PARTICIPANT_A,
                provider_result_id: "result:effect",
                provider_invocation_id: "invocation:effect",
                case_generation: 9,
                resource: &resource(),
            },
        )
        .expect("typed operation");
        history.push(transition(
            10,
            TransitionPayload::OperationRecorded {
                operation: operation.clone(),
            },
        ));
        let decision = record_filesystem_decision(
            &operation,
            &resource(),
            10,
            DecisionOutcome::Allow,
            "inside configured write prefix",
        )
        .expect("typed allow");
        history.push(transition(
            11,
            TransitionPayload::DecisionRecorded {
                decision: decision.clone(),
            },
        ));
        let pre = FilesystemObservation {
            schema: OBSERVATION_SCHEMA.to_string(),
            observation_id: "observation:pre".to_string(),
            resource_attachment_id: "workspace".to_string(),
            relative_path: "allowed/reconcile.txt".to_string(),
            state: ResourceState::Absent,
            content_digest: None,
            size_bytes: None,
            error: None,
            observed_at_unix_ms: 1,
        };
        history.push(transition(
            12,
            TransitionPayload::EffectPrepared {
                prepared: PreparedEffect {
                    schema: PREPARED_EFFECT_SCHEMA.to_string(),
                    effect_id: "effect:reconcile".to_string(),
                    operation_id: operation.operation_id,
                    decision_id: decision.decision_id,
                    grant_id: "grant:reconcile".to_string(),
                    case_id: CASE.to_string(),
                    participant_id: PARTICIPANT_A.to_string(),
                    resource_attachment_id: "workspace".to_string(),
                    relative_path: "allowed/reconcile.txt".to_string(),
                    expected_pre_observation: pre,
                    intended_content_digest: "digest:observed".to_string(),
                    idempotency_key: "effect-key:reconcile".to_string(),
                    carrier_backend: FILESYSTEM_CARRIER_BACKEND.to_string(),
                },
            },
        ));
        history.push(transition(
            13,
            TransitionPayload::EffectIndeterminate {
                effect_id: "effect:reconcile".to_string(),
                reason: "reply lost".to_string(),
                observation: None,
            },
        ));
        history
    }

    #[test]
    fn denied_provider_claim_never_becomes_successful_operational_memory() {
        let history = false_claim_denied_history();
        let first = derive_operational_memory(CASE, &history).expect("derive memory");
        let second = derive_operational_memory(CASE, &history).expect("derive twice");
        assert_eq!(first, second, "derivation must be deterministic/idempotent");
        assert!(first.entries.iter().any(|entry| {
            entry.semantic_kind == OperationalMemoryKind::ProviderClaim
                && entry.posture == OperationalMemoryPosture::ProviderOriginatedClaim
        }));
        assert!(first.entries.iter().any(|entry| {
            matches!(
                entry.value,
                OperationalMemoryValue::Decision {
                    outcome: DecisionOutcome::Deny,
                    ..
                }
            )
        }));
        assert!(!first
            .entries
            .iter()
            .any(|entry| { entry.semantic_kind == OperationalMemoryKind::ResourceEffect }));
        for entry in &first.entries {
            validate_memory_provenance(entry, &history).expect("complete provenance");
        }
    }

    #[test]
    fn visibility_is_filtered_before_ranking() {
        let history = false_claim_denied_history();
        let mut state = replay_case(CASE, &history[..5]).expect("replay participant state");
        state.generation = history.len() as u64;
        let build = derive_operational_memory(CASE, &history).expect("derive memory");
        let allowed = retrieve_operational_memory(
            &state,
            &build.entries,
            qualification(&state, PARTICIPANT_A, 8),
        )
        .expect("allowed retrieval");
        assert!(!allowed.selected.is_empty());
        assert!(allowed
            .selected
            .iter()
            .all(|item| !item.ranking_reasons.is_empty()));

        let denied = retrieve_operational_memory(
            &state,
            &build.entries,
            qualification(&state, PARTICIPANT_B, 8),
        )
        .expect("qualified empty retrieval");
        assert!(denied.selected.is_empty());
        assert_eq!(
            denied.rejections.participant_visibility,
            build.entries.len()
        );

        let mut mixed_cases = build.entries.clone();
        let mut foreign = mixed_cases[0].clone();
        foreign.case_id = "case:foreign".to_string();
        mixed_cases.push(foreign);
        let qualified = retrieve_operational_memory(
            &state,
            &mixed_cases,
            qualification(&state, PARTICIPANT_A, 8),
        )
        .expect("cross-Case entry filtered");
        assert_eq!(qualified.rejections.wrong_case, 1);
    }

    #[test]
    fn large_history_retrieval_is_bounded_explainable_and_pure() {
        let mut history = admitted_history();
        let mut sequence = history.len() as u64;
        for index in 0..250 {
            sequence += 1;
            history.push(transition(
                sequence,
                TransitionPayload::ProviderInvocationStarted {
                    invocation_id: format!("invocation:{index}"),
                    participant_id: PARTICIPANT_A.to_string(),
                    provider_id: "provider:a".to_string(),
                    provider_kind: "openai_compatible".to_string(),
                    model_id: "model:a".to_string(),
                    semantic_lineage: Some(lineage(sequence - 1)),
                },
            ));
            sequence += 1;
            history.push(transition(
                sequence,
                TransitionPayload::ProviderResultRecorded {
                    result_id: format!("result:{index}"),
                    invocation_id: format!("invocation:{index}"),
                    provider_id: "provider:a".to_string(),
                    provider_kind: "openai_compatible".to_string(),
                    model_id: "model:a".to_string(),
                    semantic_lineage: Some(lineage(sequence - 2)),
                    output: format!("non-authoritative provider claim {index}"),
                },
            ));
        }
        let ledger_before = history.clone();
        let mut state = replay_case(CASE, &history[..5]).expect("replay participant state");
        state.generation = history.len() as u64;
        let build = derive_operational_memory(CASE, &history).expect("derive large memory");
        assert_eq!(build.entries.len(), 250);
        let retrieval = retrieve_operational_memory(
            &state,
            &build.entries,
            qualification(&state, PARTICIPANT_A, 7),
        )
        .expect("bounded retrieval");
        assert_eq!(retrieval.selected_count, 7);
        assert_eq!(retrieval.omitted_count, 243);
        assert!(retrieval.selected.iter().all(|item| item
            .ranking_reasons
            .iter()
            .any(|reason| reason.starts_with("recency_"))));
        assert_eq!(history, ledger_before, "query may not mutate ledger input");
    }

    #[test]
    fn superseded_resource_state_cannot_outrank_latest_memory() {
        let mut entries = ["digest:a", "digest:b", "digest:c"]
            .iter()
            .enumerate()
            .map(|(index, digest)| {
                let generation = index as u64 + 1;
                build_entry(
                    CASE,
                    OperationalMemoryKind::ResourceEffect,
                    OperationalMemoryPosture::FinalizedObservedConsequence,
                    OperationalMemoryValue::ResourceEffect {
                        operation_id: format!("operation:{generation}"),
                        effect_id: format!("effect:{generation}"),
                        resource_attachment_id: "workspace".to_string(),
                        relative_path: "allowed/auth.py".to_string(),
                        outcome: EffectOutcome::Applied,
                        content_digest: Some((*digest).to_string()),
                        receipt_id: format!("receipt:{generation}"),
                    },
                    format!("observed digest {digest}"),
                    provenance(
                        vec![format!("transition:{generation}")],
                        vec![format!("observation:{generation}")],
                        vec![format!("receipt:{generation}")],
                        vec![format!("effect:{generation}")],
                        generation,
                        generation,
                    ),
                    vec![PARTICIPANT_A.to_string()],
                    3,
                )
                .expect("memory entry")
            })
            .collect::<Vec<_>>();
        apply_resource_state_supersession(&mut entries);
        assert_eq!(
            entries
                .iter()
                .filter(|entry| entry.lifecycle == OperationalMemoryLifecycle::Active)
                .count(),
            1
        );
        let mut state = replay_case(CASE, &admitted_history()[..5]).expect("participant state");
        state.generation = 3;
        let mut request = qualification(&state, PARTICIPANT_A, 8);
        request.resource_refs = vec!["workspace".to_string()];
        let selected = retrieve_operational_memory(&state, &entries, request)
            .expect("current resource retrieval");
        assert_eq!(selected.selected_count, 1);
        assert!(matches!(
            &selected.selected[0].memory.value,
            OperationalMemoryValue::ResourceEffect {
                content_digest: Some(digest),
                ..
            } if digest == "digest:c"
        ));
    }

    #[test]
    fn indeterminate_memory_stays_unresolved_until_receipted_reconciliation() {
        let mut history = unresolved_history();
        let unresolved = derive_operational_memory(CASE, &history).expect("derive unresolved");
        assert!(unresolved.entries.iter().any(|entry| {
            entry.semantic_kind == OperationalMemoryKind::UnresolvedEffect
                && entry.posture == OperationalMemoryPosture::Unresolved
                && entry.lifecycle == OperationalMemoryLifecycle::Active
        }));
        assert!(!unresolved
            .entries
            .iter()
            .any(|entry| entry.semantic_kind == OperationalMemoryKind::ResourceEffect));

        let post = FilesystemObservation {
            schema: OBSERVATION_SCHEMA.to_string(),
            observation_id: "observation:reconciled".to_string(),
            resource_attachment_id: "workspace".to_string(),
            relative_path: "allowed/reconcile.txt".to_string(),
            state: ResourceState::File,
            content_digest: Some("digest:observed".to_string()),
            size_bytes: Some(8),
            error: None,
            observed_at_unix_ms: 2,
        };
        history.push(transition(
            14,
            TransitionPayload::EffectReconciled {
                effect_id: "effect:reconcile".to_string(),
                conclusion: ReconciliationConclusion::EffectObserved,
                observation: post,
                receipt: Some(EffectReceipt {
                    schema: EFFECT_RECEIPT_SCHEMA.to_string(),
                    receipt_id: "receipt:reconciled".to_string(),
                    effect_id: "effect:reconcile".to_string(),
                    operation_id: unresolved_history()
                        .iter()
                        .find_map(|transition| match &transition.payload {
                            TransitionPayload::OperationRecorded { operation } => {
                                Some(operation.operation_id.clone())
                            }
                            _ => None,
                        })
                        .expect("operation identity"),
                    decision_id: unresolved_history()
                        .iter()
                        .find_map(|transition| match &transition.payload {
                            TransitionPayload::DecisionRecorded { decision } => {
                                Some(decision.decision_id.clone())
                            }
                            _ => None,
                        })
                        .expect("decision identity"),
                    grant_id: "grant:reconcile".to_string(),
                    resource_attachment_id: "workspace".to_string(),
                    relative_path: "allowed/reconcile.txt".to_string(),
                    pre_observation_id: "observation:pre".to_string(),
                    post_observation_id: "observation:reconciled".to_string(),
                    outcome: EffectOutcome::AlreadyApplied,
                    carrier_backend: FILESYSTEM_CARRIER_BACKEND.to_string(),
                    carrier_attempted: false,
                    mutation_performed: false,
                    completed_at_unix_ms: 2,
                }),
            },
        ));
        let reconciled = derive_operational_memory(CASE, &history).expect("derive reconciled");
        assert!(reconciled.entries.iter().any(|entry| {
            entry.semantic_kind == OperationalMemoryKind::ResourceEffect
                && entry.posture == OperationalMemoryPosture::FinalizedObservedConsequence
                && entry.lifecycle == OperationalMemoryLifecycle::Active
        }));
        assert!(!reconciled.entries.iter().any(|entry| {
            entry.semantic_kind == OperationalMemoryKind::UnresolvedEffect
                && entry.lifecycle == OperationalMemoryLifecycle::Active
        }));
    }
}
