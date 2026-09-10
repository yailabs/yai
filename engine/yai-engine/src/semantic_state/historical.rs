//! Read-only, prefix-qualified Case meaning. Not Working State, Recall or authority.
use super::AuthorityPosture;
use crate::case_policy::{CasePolicyBinding, EffectivePolicy, NormativeStatus};
use crate::effect::digest_bytes;
use crate::transition::{
    replay_case, CaseLifecycle, CaseState, ParticipantState, ResourceAttachmentState, Transition,
    TransitionPayload,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const HISTORICAL_VIEW_SCHEMA: &str = "yai.historical_semantic_view.v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum HistoricalCoordinate {
    Generation(u64),
    Transition(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HistoricalRequest {
    pub coordinate: HistoricalCoordinate,
    pub participant_id: String,
    pub consumer: String,
    pub view_kind: String,
    pub max_items: usize,
    pub max_bytes: usize,
}

impl HistoricalRequest {
    pub fn inspection(coordinate: HistoricalCoordinate, participant: impl Into<String>) -> Self {
        Self {
            coordinate,
            participant_id: participant.into(),
            consumer: "operator".into(),
            view_kind: "case_inspection".into(),
            max_items: 256,
            max_bytes: 1_048_576,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HistoricalMaterialization {
    pub lifecycle: CaseLifecycle,
    pub cancellation: Option<crate::transition::CaseCancellationState>,
    pub closure: Option<crate::transition::CaseClosureState>,
    pub participant: Option<ParticipantState>,
    pub principal_association: Option<crate::transition::PrincipalParticipantLink>,
    pub policy_bindings: Vec<CasePolicyBinding>,
    pub resources: Vec<ResourceAttachmentState>,
    pub reviews: Vec<crate::transition::ReviewState>,
    pub grants: Vec<crate::transition::GrantState>,
    pub effects: Vec<crate::transition::EffectState>,
}

/// These are recorded payloads, never a new semantic mutation API. Only the
/// explicitly scoped families in `visible_evidence` can construct an item.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HistoricalEvidence {
    pub transition_id: String,
    pub recorded_generation: u64,
    pub recorded_at_unix_ms: Option<u64>,
    pub observed_at_unix_ms: Option<u64>,
    /// No current producer supplies a general exact occurrence-time contract.
    pub occurred_at_unix_ms: Option<u64>,
    pub posture: AuthorityPosture,
    pub payload: TransitionPayload,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceClosure {
    pub source_ref: String,
    pub posture: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HistoricalNormative {
    /// Exact bound-artifact composition, NOT a claim of arbitrary-time validity.
    pub effective_policy: Option<EffectivePolicy>,
    pub readiness: crate::case_policy::NormativeReadiness,
    pub blocking_conflicts: Vec<String>,
    pub missing: Vec<String>,
    pub temporal_posture: String,
    pub source_closure: Vec<SourceClosure>,
}

/// Current normative meaning without pretending a sampled wall clock is part
/// of a reproducible historical identity. DecisionBasis retains its exact time.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CurrentNormativeMeaning {
    pub readiness: crate::case_policy::NormativeReadiness,
    pub validity: crate::case_policy::PolicyValidityPosture,
    pub binding_validity: BTreeMap<String, crate::case_policy::BindingValidity>,
    pub effective_policy: Option<EffectivePolicy>,
    pub missing: Vec<String>,
    pub blocking_conflicts: Vec<String>,
    pub catalog_drift: BTreeMap<String, crate::case_policy::PolicyCatalogDrift>,
    pub persisted_authority_floor_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HistoricalComparison {
    pub key: String,
    pub category: String,
    pub historical_digest: Option<String>,
    pub current_digest: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HistoricalSemanticView {
    pub schema: String,
    pub view_id: String,
    pub case_id: String,
    pub generation: u64,
    pub coordinate_transition_id: String,
    pub source_history_digest: String,
    pub current_generation: u64,
    pub current_scope_digest: String,
    pub request: HistoricalRequest,
    pub state_then: HistoricalMaterialization,
    pub state_source_transitions: Vec<String>,
    pub known_by_then: Vec<HistoricalEvidence>,
    pub normative_then: HistoricalNormative,
    /// Current catalog/validity, explicitly separate from the historical view.
    pub normative_now: CurrentNormativeMeaning,
    pub comparison_to_now: Vec<HistoricalComparison>,
    pub content_backing: Vec<SourceClosure>,
    pub supported_families: Vec<String>,
    pub unsupported_families: Vec<String>,
    pub transitions_scanned: usize,
    pub transitions_replayed: usize,
    pub semantic_items: usize,
}

pub(crate) fn digest<T: Serialize>(value: &T) -> String {
    digest_bytes(&serde_json::to_vec(value).expect("typed historical value serializes"))
}

pub(crate) fn prefix<'a>(
    history: &'a [Transition],
    coordinate: &HistoricalCoordinate,
) -> Result<&'a [Transition], String> {
    let generation = match coordinate {
        HistoricalCoordinate::Generation(n) => *n,
        HistoricalCoordinate::Transition(id) => history
            .iter()
            .find(|t| &t.transition_id == id)
            .map(|t| t.sequence)
            .ok_or("historical_coordinate_invalid")?,
    };
    if generation == 0
        || generation > history.len() as u64
        || generation > history.last().map_or(0, |t| t.sequence)
    {
        return Err("historical_coordinate_invalid".into());
    }
    Ok(&history[..usize::try_from(generation).map_err(|_| "historical_coordinate_invalid")?])
}

pub(crate) fn validate_scope(
    current: &CaseState,
    request: &HistoricalRequest,
) -> Result<(), String> {
    if request.max_items == 0
        || request.max_items > 4096
        || request.max_bytes == 0
        || request.max_bytes > 16_777_216
    {
        return Err("historical_bounds_invalid".into());
    }
    let p = current
        .participants
        .iter()
        .find(|p| p.participant_id == request.participant_id)
        .ok_or("historical_scope_unavailable")?;
    // Operator inspection uses the existing current Principal/Participant link
    // checked by the transactional public reader; it is not model disclosure.
    if request.consumer == "operator" && request.view_kind == "case_inspection" {
        return Ok(());
    }
    if !p
        .admitted_views
        .iter()
        .any(|v| v.consumer == request.consumer && v.view_kind == request.view_kind)
    {
        return Err("historical_scope_unavailable".into());
    }
    Ok(())
}

fn resource_visible(resource: &ResourceAttachmentState, participant: &str) -> bool {
    resource
        .access
        .as_ref()
        .map_or(resource.policy_owner_participant_id == participant, |a| {
            a.participant_ids.iter().any(|p| p == participant)
        })
}

// Exact current envelope, including configuration and disclosure. Reusing an
// attachment ID or path does not restore access to the previous attachment.
fn resource_current(
    resource: &ResourceAttachmentState,
    current: &CaseState,
    participant: &str,
) -> bool {
    resource_visible(resource, participant)
        && current
            .resources
            .iter()
            .any(|r| r == resource && resource_visible(r, participant))
}

fn resource_epochs(history: &[Transition]) -> BTreeMap<String, u64> {
    let mut last = BTreeMap::new();
    let mut epochs = BTreeMap::new();
    for t in history {
        if let TransitionPayload::ResourceAttached { attachment } = &t.payload {
            if last
                .get(&attachment.attachment_id)
                .is_some_and(|r| *r != attachment)
            {
                epochs.insert(attachment.attachment_id.clone(), t.sequence);
            }
            last.insert(attachment.attachment_id.clone(), attachment);
        }
    }
    epochs
}

fn materialization(
    state: &CaseState,
    current: &CaseState,
    participant: &str,
    epochs: &BTreeMap<String, u64>,
    evidence: &[HistoricalEvidence],
) -> HistoricalMaterialization {
    let operations: BTreeSet<_> = evidence
        .iter()
        .filter_map(|e| match &e.payload {
            TransitionPayload::OperationRecorded { operation } => Some(&operation.operation_id),
            _ => None,
        })
        .collect();
    HistoricalMaterialization {
        lifecycle: state.lifecycle.clone(),
        cancellation: state.cancellation.clone(),
        closure: state.closure.clone(),
        principal_association: state
            .principal_participant_links
            .iter()
            .find(|link| link.participant_id == participant)
            .cloned(),
        participant: state
            .participants
            .iter()
            .find(|p| p.participant_id == participant)
            .cloned(),
        policy_bindings: state.policy_bindings.clone(),
        resources: state
            .resources
            .iter()
            .filter(|r| {
                state.generation >= epochs.get(&r.attachment_id).copied().unwrap_or(0)
                    && resource_current(r, current, participant)
            })
            .cloned()
            .collect(),
        reviews: state
            .reviews
            .iter()
            .filter(|r| operations.contains(&r.operation_id))
            .cloned()
            .collect(),
        grants: state
            .grants
            .iter()
            .filter(|g| operations.contains(&g.operation_id))
            .cloned()
            .collect(),
        effects: state
            .effects
            .iter()
            .filter(|e| operations.contains(&e.operation_id))
            .cloned()
            .collect(),
    }
}

/// Traversal never uses a later observation to populate an earlier knowledge
/// cut. Resource envelopes are tracked at each occurrence, not looked up only
/// at the final prefix state (which could reveal a superseded attachment).
fn visible_evidence(
    history: &[Transition],
    current: &CaseState,
    participant: &str,
    epochs: &BTreeMap<String, u64>,
) -> Vec<HistoricalEvidence> {
    use TransitionPayload as P;
    let mut resources = BTreeMap::<String, ResourceAttachmentState>::new();
    let mut operations = BTreeSet::new();
    let mut decisions = BTreeSet::new();
    let mut reviews = BTreeSet::new();
    let mut grants = BTreeSet::new();
    let mut effects = BTreeSet::new();
    let mut invocations = BTreeSet::new();
    let mut results = BTreeSet::new();
    let mut output = Vec::new();
    for t in history {
        let resource_ok = |id: &str| {
            t.sequence >= epochs.get(id).copied().unwrap_or(0)
                && resources
                    .get(id)
                    .is_some_and(|r| resource_current(r, current, participant))
        };
        let mut observed = None;
        let posture = match &t.payload {
            P::CasePolicyBound { .. }
            | P::CasePolicyReplaced { .. }
            | P::CasePolicyUnbound { .. } => Some(AuthorityPosture::ControlState),
            P::ResourceAttached { attachment } => {
                resources.insert(attachment.attachment_id.clone(), attachment.clone());
                None
            }
            P::OperationRecorded { operation }
                if operation.participant_id == participant
                    && resource_ok(&operation.resource_attachment_id) =>
            {
                operations.insert(operation.operation_id.clone());
                Some(AuthorityPosture::Unresolved)
            }
            P::DecisionRecorded { decision } if operations.contains(&decision.operation_id) => {
                decisions.insert(decision.decision_id.clone());
                Some(AuthorityPosture::ControlState)
            }
            P::ReviewRequested { review } if operations.contains(&review.operation_id) => {
                reviews.insert(review.review_id.clone());
                Some(AuthorityPosture::ControlState)
            }
            P::ReviewActionRecorded { action } if reviews.contains(&action.review_id) => {
                Some(AuthorityPosture::ControlState)
            }
            P::ReviewInvalidated { invalidation } if reviews.contains(&invalidation.review_id) => {
                Some(AuthorityPosture::ControlState)
            }
            P::ExecutionGrantIssued { grant } if operations.contains(&grant.operation_id) => {
                grants.insert(grant.grant_id.clone());
                Some(AuthorityPosture::ControlState)
            }
            P::ExecutionGrantInvalidated { invalidation }
                if grants.contains(&invalidation.grant_id) =>
            {
                Some(AuthorityPosture::ControlState)
            }
            P::ResourceObservationRecorded { observation }
            | P::ResourceEffectFinalized { observation, .. }
                if observation.participant_id == participant
                    && operations.contains(&observation.operation_id)
                    && decisions.contains(&observation.decision_id)
                    && resource_ok(&observation.resource_attachment_id)
                    && resources[&observation.resource_attachment_id]
                        .access
                        .as_ref()
                        .is_some_and(|a| {
                            a.configuration_digest == observation.configuration_digest
                        }) =>
            {
                observed = Some(observation.observed_at_unix_ms);
                Some(AuthorityPosture::ObservedResourceState)
            }
            P::EffectPrepared { prepared } if operations.contains(&prepared.operation_id) => {
                effects.insert(prepared.effect_id.clone());
                Some(AuthorityPosture::Unresolved)
            }
            P::EffectFinalized { effect_id, .. } | P::EffectReconciled { effect_id, .. }
                if effects.contains(effect_id) =>
            {
                Some(AuthorityPosture::CommittedOperationalFact)
            }
            P::EffectIndeterminate { effect_id, .. } if effects.contains(effect_id) => {
                Some(AuthorityPosture::Unresolved)
            }
            P::CaseContentAdmitted { admission }
                if admission.participant_ids.iter().any(|p| p == participant)
                    && resource_ok(&admission.source_resource_id)
                    && resources[&admission.source_resource_id]
                        .access
                        .as_ref()
                        .is_some_and(|a| {
                            a.configuration_digest == admission.source_configuration_digest
                        }) =>
            {
                Some(AuthorityPosture::CommittedApplicationContent)
            }
            P::ConversationTurnCommitted { turn } if turn.participant_id == participant => {
                Some(AuthorityPosture::CommittedApplicationContent)
            }
            P::ProviderInvocationStarted {
                invocation_id,
                participant_id,
                ..
            } if participant_id == participant
                && epochs.values().all(|changed| *changed <= t.sequence) =>
            {
                invocations.insert(invocation_id.clone());
                None
            }
            P::ProviderResultRecorded {
                invocation_id,
                result_id,
                ..
            } if invocations.contains(invocation_id) => {
                results.insert(result_id.clone());
                Some(AuthorityPosture::ProviderClaim)
            }
            P::ModelInterpretationRecorded { result_id, .. } if results.contains(result_id) => {
                Some(AuthorityPosture::ProviderClaim)
            }
            _ => None,
        };
        if let Some(posture) = posture {
            output.push(HistoricalEvidence {
                transition_id: t.transition_id.clone(),
                recorded_generation: t.sequence,
                recorded_at_unix_ms: (t.committed_at_unix_ms != 0)
                    .then_some(t.committed_at_unix_ms),
                observed_at_unix_ms: observed,
                occurred_at_unix_ms: None,
                posture,
                payload: t.payload.clone(),
            });
        }
    }
    output
}

fn comparison_items(
    state: &HistoricalMaterialization,
    evidence: &[HistoricalEvidence],
) -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    values.insert("case:lifecycle".into(), digest(&state.lifecycle));
    if let Some(c) = &state.cancellation {
        values.insert("case:cancellation".into(), digest(c));
    }
    if let Some(c) = &state.closure {
        values.insert("case:closure".into(), digest(c));
    }
    if let Some(link) = &state.principal_association {
        values.insert("participant:principal-association".into(), digest(link));
    }
    if let Some(p) = &state.participant {
        values.insert(p.participant_id.clone(), digest(p));
    }
    for b in &state.policy_bindings {
        values.insert(format!("policy-lineage:{}", b.lineage_id), digest(b));
    }
    for r in &state.resources {
        values.insert(format!("resource:{}", r.attachment_id), digest(r));
    }
    for r in &state.reviews {
        values.insert(r.review_id.clone(), digest(r));
    }
    for g in &state.grants {
        values.insert(g.grant_id.clone(), digest(g));
    }
    for e in &state.effects {
        values.insert(e.effect_id.clone(), digest(e));
    }
    for e in evidence {
        values.insert(e.transition_id.clone(), digest(e));
    }
    values
}

pub(crate) fn reconstruct(
    current: &CaseState,
    history: &[Transition],
    mut request: HistoricalRequest,
    normative_then: HistoricalNormative,
    normative_now: NormativeStatus,
) -> Result<HistoricalSemanticView, String> {
    validate_scope(current, &request)?;
    if replay_case(&current.case_id, history)? != *current {
        return Err("historical_replay_mismatch".into());
    }
    let cut = prefix(history, &request.coordinate)?;
    let then = replay_case(&current.case_id, cut)?;
    request.coordinate = HistoricalCoordinate::Generation(then.generation);
    let epochs = resource_epochs(history);
    let known_by_then = visible_evidence(cut, current, &request.participant_id, &epochs);
    let known_now = visible_evidence(history, current, &request.participant_id, &epochs);
    let state_then = materialization(
        &then,
        current,
        &request.participant_id,
        &epochs,
        &known_by_then,
    );
    let state_now = materialization(
        current,
        current,
        &request.participant_id,
        &epochs,
        &known_now,
    );
    let state_source_transitions = cut
        .iter()
        .filter(|t| match &t.payload {
            TransitionPayload::TenantCaseOpened { .. }
            | TransitionPayload::CaseOpened { .. }
            | TransitionPayload::CaseCancellationRequested { .. }
            | TransitionPayload::CaseClosed { .. } => true,
            TransitionPayload::ParticipantPrincipalLinked { link } => {
                link.participant_id == request.participant_id
            }
            TransitionPayload::ParticipantBound { participant_id, .. }
            | TransitionPayload::ParticipantAdmitted { participant_id, .. } => {
                participant_id == &request.participant_id
            }
            TransitionPayload::CasePolicyBound { binding }
            | TransitionPayload::CasePolicyReplaced { binding, .. } => state_then
                .policy_bindings
                .iter()
                .any(|b| b.binding_id == binding.binding_id),
            TransitionPayload::ResourceAttached { attachment } => {
                state_then.resources.contains(attachment)
            }
            _ => known_by_then
                .iter()
                .any(|e| e.transition_id == t.transition_id),
        })
        .map(|t| t.transition_id.clone())
        .collect();
    let old = comparison_items(&state_then, &known_by_then);
    let new = comparison_items(&state_now, &known_now);
    let keys: BTreeSet<_> = old.keys().chain(new.keys()).cloned().collect();
    let comparison_to_now = keys
        .into_iter()
        .map(|key| {
            let historical_digest = old.get(&key).cloned();
            let current_digest = new.get(&key).cloned();
            let category = match (&historical_digest, &current_digest) {
                (None, Some(_)) => "not_yet_existing_at_coordinate",
                (Some(_), None) => "removed_from_current_state",
                (Some(a), Some(b)) if a == b && key.starts_with("transition:") => {
                    "still_recorded_not_current_authority"
                }
                (Some(a), Some(b)) if a == b => "unchanged_current_materialization",
                _ if key.starts_with("policy-lineage:") => "superseded",
                _ => "changed",
            }
            .into();
            HistoricalComparison {
                key,
                category,
                historical_digest,
                current_digest,
            }
        })
        .collect();
    // Sampled wall clock is diagnostic, not identity. Actual temporal validity
    // and the committed authority floor remain bound into the current posture.
    let normative_now = CurrentNormativeMeaning {
        readiness: normative_now.readiness,
        validity: normative_now.validity,
        binding_validity: normative_now.binding_validity,
        effective_policy: normative_now.effective_policy,
        missing: normative_now.missing,
        blocking_conflicts: normative_now.blocking_conflicts,
        catalog_drift: normative_now.catalog_drift,
        persisted_authority_floor_unix_ms: normative_now.persisted_authority_floor_unix_ms,
    };
    let semantic_items = old.len() + new.len();
    if semantic_items > request.max_items {
        return Err(format!(
            "historical_output_item_budget_exceeded:required={semantic_items}"
        ));
    }
    let mut view = HistoricalSemanticView {
        schema: HISTORICAL_VIEW_SCHEMA.into(),
        view_id: String::new(),
        case_id: current.case_id.clone(),
        generation: then.generation,
        coordinate_transition_id: cut.last().unwrap().transition_id.clone(),
        source_history_digest: digest(&history),
        current_generation: current.generation,
        current_scope_digest: digest(&(
            current.tenant_id.clone(),
            &request.participant_id,
            &request.consumer,
            &request.view_kind,
            &current.participants,
            &current.resources,
            &current.principal_participant_links,
        )),
        request,
        state_then,
        state_source_transitions,
        known_by_then,
        normative_then,
        normative_now,
        comparison_to_now,
        content_backing: Vec::new(),
        supported_families: [
            "case_lifecycle",
            "requesting_participant",
            "policy_bindings_and_exact_artifacts",
            "own_operations_decisions_reviews_grants",
            "currently_disclosed_resources_observations_content",
            "filesystem_effect_history",
            "own_conversation_turns_and_provider_claims",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
        unsupported_families: [
            "general_event_time",
            "wall_clock_as_of",
            "global_catalog_cut_at_arbitrary_case_generation",
            "workflow_handoff_projection",
            "process_effect_projection",
            "derived_episodes_assertions",
            "general_recall",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
        transitions_scanned: history.len(),
        transitions_replayed: history.len() + 2 * cut.len(),
        semantic_items,
    };
    view.seal()?;
    Ok(view)
}

impl HistoricalSemanticView {
    pub(crate) fn seal(&mut self) -> Result<(), String> {
        self.view_id.clear();
        self.view_id = format!("historical-view:{}", digest(self));
        if serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > self.request.max_bytes {
            return Err("historical_output_byte_budget_exceeded".into());
        }
        Ok(())
    }
}
