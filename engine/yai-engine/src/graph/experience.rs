//! Qualified, disposable access to relations between recorded Case experiences.
//! The historical reader owns source qualification and current disclosure. This
//! module owns neither history nor admission, and never interprets prose/causal_refs.
use crate::effect::OperationOrigin;
use crate::semantic_state::historical::{digest, HistoricalEvidence, HistoricalSemanticView};
use crate::semantic_state::AuthorityPosture;
use crate::transition::TransitionPayload as P;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const EXPERIENCE_SCHEMA: &str = "yai.experience_relations.v1";

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationKind {
    RecordedBefore,
    ProposalOrigin,
    OperationDecision,
    PolicyBasis,
    EvidenceSupport,
    DecisionReview,
    ReviewAction,
    ReviewReevaluation,
    DecisionGrant,
    GrantPreparation,
    PreparedConsequence,
    DecisionObservation,
    ObservationContent,
    ResultInterpretation,
    PolicyReplacement,
    PolicyUnbinding,
    ReviewInvalidation,
    GrantInvalidation,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationPosture {
    RecordingOrder,
    StructuralReference,
    RecordedNormativeSupport,
    RecordedEvidenceSupport,
    ObservedConsequence,
    Lifecycle,
}

impl RelationKind {
    fn posture(&self) -> RelationPosture {
        use RelationKind::*;
        match self {
            RecordedBefore => RelationPosture::RecordingOrder,
            PolicyBasis => RelationPosture::RecordedNormativeSupport,
            EvidenceSupport => RelationPosture::RecordedEvidenceSupport,
            PreparedConsequence | DecisionObservation => RelationPosture::ObservedConsequence,
            PolicyReplacement | PolicyUnbinding | ReviewInvalidation | GrantInvalidation => {
                RelationPosture::Lifecycle
            }
            _ => RelationPosture::StructuralReference,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExperienceEvent {
    pub transition_id: String,
    pub kind: String,
    /// Only identities owned by this payload, never arbitrary embedded JSON IDs.
    pub object_refs: Vec<String>,
    pub posture: AuthorityPosture,
    pub recorded_generation: u64,
    pub recorded_at_unix_ms: Option<u64>,
    pub observed_at_unix_ms: Option<u64>,
    pub occurred_at_unix_ms: Option<u64>,
    /// Missing exact backing does not erase recorded existence or mint an edge.
    pub source_closed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RelationSource {
    pub transition_id: String,
    /// Typed payload field, not a purported byte span in an external document.
    pub field: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExperienceRelation {
    pub relation_id: String,
    pub from_event: String,
    pub to_event: String,
    pub kind: RelationKind,
    pub posture: RelationPosture,
    pub known_at_generation: u64,
    pub sources: Vec<RelationSource>,
}

/// Scoped structural slices, not replacement MemoryEpisode records. Membership
/// follows the existing operation/result/turn structural identities. Evidence
/// and policy links cross slices and never union them into a giant Episode.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExperienceEpisodeSlice {
    pub slice_id: String,
    pub transition_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExperienceQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub include_recording_order: bool,
    pub max_hops: usize,
    pub max_events: usize,
    pub max_relations: usize,
    pub max_bytes: usize,
}

impl Default for ExperienceQuery {
    fn default() -> Self {
        Self {
            from: None,
            to: None,
            include_recording_order: false,
            max_hops: 16,
            max_events: 256,
            max_relations: 1024,
            max_bytes: 1_048_576,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExperienceView {
    pub schema: String,
    pub view_id: String,
    pub case_id: String,
    pub generation: u64,
    pub participant_id: String,
    pub current_disclosure_digest: String,
    pub query: ExperienceQuery,
    pub events: Vec<ExperienceEvent>,
    pub relations: Vec<ExperienceRelation>,
    pub episode_slices: Vec<ExperienceEpisodeSlice>,
    /// All-graph, qualified directed path, or no path within this profile/budget.
    /// Absence does not reveal whether an undisclosed intermediate exists.
    pub result: String,
    pub limitations: Vec<String>,
}

fn event(e: &HistoricalEvidence, closed: bool) -> ExperienceEvent {
    let refs = match &e.payload {
        P::CasePolicyBound { binding } | P::CasePolicyReplaced { binding, .. } => {
            vec![binding.binding_id.clone()]
        }
        P::OperationRecorded { operation } => vec![operation.operation_id.clone()],
        P::DecisionRecorded { decision } => {
            let mut r = vec![decision.decision_id.clone()];
            if let Some(b) = &decision.decision_basis {
                r.push(b.basis_id.clone());
            }
            r
        }
        P::ReviewRequested { review } => vec![review.review_id.clone()],
        P::ReviewActionRecorded { action } => vec![action.action_id.clone()],
        P::ExecutionGrantIssued { grant } => vec![grant.grant_id.clone()],
        P::EffectPrepared { prepared } => vec![prepared.effect_id.clone()],
        P::ResourceEffectPrepared { prepared } => vec![prepared.effect_id.clone()],
        P::EffectFinalized {
            receipt,
            post_observation,
            ..
        } => vec![
            receipt.receipt_id.clone(),
            post_observation.observation_id.clone(),
        ],
        P::EffectReconciled {
            receipt,
            observation,
            ..
        } => receipt
            .iter()
            .map(|r| r.receipt_id.clone())
            .chain(std::iter::once(observation.observation_id.clone()))
            .collect(),
        P::ResourceObservationRecorded { observation } => vec![observation.observation_id.clone()],
        P::ResourceEffectFinalized {
            receipt,
            observation,
            ..
        } => vec![
            receipt.receipt_id.clone(),
            observation.observation_id.clone(),
        ],
        P::CaseContentAdmitted { admission } => vec![admission.object.object_id.clone()],
        P::ConversationTurnCommitted { turn } => vec![turn.turn_id.clone()],
        P::ProviderResultRecorded { result_id, .. } => vec![result_id.clone()],
        P::ModelInterpretationRecorded {
            interpretation_id, ..
        } => vec![interpretation_id.clone()],
        _ => Vec::new(),
    };
    let observed = match &e.payload {
        P::EffectFinalized {
            post_observation, ..
        } => Some(post_observation.observed_at_unix_ms),
        P::EffectReconciled { observation, .. } => Some(observation.observed_at_unix_ms),
        _ => e.observed_at_unix_ms,
    };
    ExperienceEvent {
        transition_id: e.transition_id.clone(),
        kind: e.payload.kind().into(),
        object_refs: refs,
        posture: e.posture.clone(),
        recorded_generation: e.recorded_generation,
        recorded_at_unix_ms: e.recorded_at_unix_ms,
        observed_at_unix_ms: observed,
        occurred_at_unix_ms: e.occurred_at_unix_ms,
        source_closed: closed,
    }
}

fn closed(e: &HistoricalEvidence, h: &HistoricalSemanticView) -> bool {
    let artifact = |id: &str| {
        h.normative_then
            .source_closure
            .iter()
            .any(|s| s.source_ref == id && s.posture == "exact_immutable_artifact")
    };
    let content = |id: &str| {
        h.content_backing
            .iter()
            .any(|s| s.source_ref == id && s.posture == "exact_original_available")
    };
    match &e.payload {
        P::CasePolicyBound { binding } | P::CasePolicyReplaced { binding, .. } => {
            artifact(&binding.artifact_id)
        }
        P::DecisionRecorded { decision } => decision
            .decision_basis
            .as_ref()
            .is_none_or(|b| b.policy_artifact_refs.iter().all(|id| artifact(id))),
        P::CaseContentAdmitted { admission } => content(&admission.object.object_id),
        P::ConversationTurnCommitted { turn } => turn
            .ordered_parts
            .iter()
            .all(|p| content(&p.object.object_id)),
        _ => true,
    }
}

// Typed lifecycle key only: never examine arbitrary result/body/summary JSON.
fn episode_key(e: &HistoricalEvidence) -> Option<String> {
    match &e.payload {
        P::OperationRecorded { operation } => Some(operation.operation_id.clone()),
        P::DecisionRecorded { decision } => Some(decision.operation_id.clone()),
        P::ReviewRequested { review } => Some(review.operation_id.clone()),
        P::ReviewActionRecorded { action } => Some(action.operation_id.clone()),
        P::ExecutionGrantIssued { grant } => Some(grant.operation_id.clone()),
        P::EffectPrepared { prepared } => Some(prepared.operation_id.clone()),
        P::ResourceEffectPrepared { prepared } => Some(prepared.operation_id.clone()),
        P::EffectFinalized { receipt, .. } => Some(receipt.operation_id.clone()),
        P::ResourceObservationRecorded { observation }
        | P::ResourceEffectFinalized { observation, .. } => Some(observation.operation_id.clone()),
        P::ProviderResultRecorded { result_id, .. }
        | P::ModelInterpretationRecorded { result_id, .. } => Some(result_id.clone()),
        P::ConversationTurnCommitted { turn } => Some(turn.turn_id.clone()),
        _ => None,
    }
}

/// Only the store's qualified historical reader calls this constructor. A raw
/// cached graph, arbitrary Transition causal_refs or model relation proposal
/// cannot construct a publicly qualified experience view.
pub(crate) fn derive(
    h: &HistoricalSemanticView,
    query: ExperienceQuery,
    current_disclosure_digest: &str,
) -> Result<ExperienceView, String> {
    if query.from.is_some() != query.to.is_some()
        || query.max_hops == 0
        || query.max_hops > 64
        || query.max_events == 0
        || query.max_events > 4096
        || query.max_relations == 0
        || query.max_relations > 32768
        || query.max_bytes == 0
        || query.max_bytes > 16_777_216
    {
        return Err("experience_bounds_or_path_invalid".into());
    }
    let events: Vec<_> = h
        .known_by_then
        .iter()
        .map(|e| event(e, closed(e, h)))
        .collect();
    let mut aliases = BTreeMap::<String, usize>::new();
    let mut shared_payloads = BTreeSet::new();
    for (i, e) in events.iter().enumerate() {
        for id in std::iter::once(&e.transition_id).chain(e.object_refs.iter()) {
            if let Some(prior) = aliases.get(id).copied() {
                // Immutable payload reuse is not duplicate admission identity.
                // Keep both events; a payload alone cannot choose which event
                // supplied evidence. Exact Transition anchors remain available.
                if e.kind == "case_content_admitted" && events[prior].kind == e.kind {
                    shared_payloads.insert(id.clone());
                } else {
                    return Err("experience_ambiguous_source_identity".into());
                }
            } else {
                aliases.insert(id.clone(), i);
            }
        }
    }
    for id in shared_payloads { aliases.remove(&id); }
    let mut relations = BTreeMap::new();
    let mut add = |from: &str, to: usize, kind: RelationKind, field: &str| {
        let Some(&a) = aliases.get(from) else {
            return;
        }; // unavailable OR undisclosed; no distinguishing counter
        if a == to || !events[a].source_closed || !events[to].source_closed {
            return;
        }
        if events[a].recorded_generation >= events[to].recorded_generation {
            return;
        }
        let sources = vec![
            RelationSource {
                transition_id: events[a].transition_id.clone(),
                field: "typed_object_identity".into(),
            },
            RelationSource {
                transition_id: events[to].transition_id.clone(),
                field: field.into(),
            },
        ];
        let id = format!(
            "experience-relation:{}",
            digest(&(EXPERIENCE_SCHEMA, &h.case_id, &kind, &sources))
        );
        relations.insert(
            id.clone(),
            ExperienceRelation {
                relation_id: id,
                from_event: events[a].transition_id.clone(),
                to_event: events[to].transition_id.clone(),
                posture: kind.posture(),
                kind,
                known_at_generation: events[to].recorded_generation,
                sources,
            },
        );
    };
    for (i, e) in h.known_by_then.iter().enumerate() {
        use RelationKind as K;
        if i > 0 {
            add(
                &events[i - 1].transition_id,
                i,
                K::RecordedBefore,
                "recorded_generation; not occurrence order or adjacency",
            );
        }
        match &e.payload {
            P::OperationRecorded { operation } => if let OperationOrigin::ProviderResult { provider_result_id, .. } = &operation.origin {
                add(provider_result_id, i, K::ProposalOrigin, "operation.origin.provider_result_id");
            },
            P::DecisionRecorded { decision } => {
                add(&decision.operation_id, i, K::OperationDecision, "decision.operation_id");
                if let Some(b) = &decision.decision_basis {
                    for id in &b.policy_binding_refs { add(id, i, K::PolicyBasis, "decision.decision_basis.policy_binding_refs"); }
                    for o in &b.obligations { for id in &o.evidence_refs {
                        add(id, i, K::EvidenceSupport, "decision.decision_basis.obligations.evidence_refs");
                    }}
                    if let Some(id) = &b.review_action_ref { add(id, i, K::ReviewReevaluation, "decision.decision_basis.review_action_ref"); }
                }
            }
            P::ReviewRequested { review } => add(&review.initial_decision_id, i, K::DecisionReview, "review.initial_decision_id"),
            P::ReviewActionRecorded { action } => add(&action.review_id, i, K::ReviewAction, "action.review_id"),
            P::ExecutionGrantIssued { grant } => add(&grant.decision_id, i, K::DecisionGrant, "grant.decision_id"),
            P::EffectPrepared { prepared } => add(&prepared.grant_id, i, K::GrantPreparation, "prepared.grant_id"),
            P::ResourceEffectPrepared { prepared } => add(&prepared.grant_id, i, K::GrantPreparation, "prepared.grant_id"),
            P::EffectFinalized { effect_id, .. } | P::EffectReconciled { effect_id, .. } | P::ResourceEffectFinalized { effect_id, .. } =>
                add(effect_id, i, K::PreparedConsequence, "effect_id; receipt/observation records outcome, not necessarily successful mutation"),
            P::ResourceObservationRecorded { observation } => add(&observation.decision_id, i, K::DecisionObservation, "observation.decision_id"),
            P::CaseContentAdmitted { admission } => add(&admission.discovery_observation_id, i, K::ObservationContent, "admission.discovery_observation_id"),
            P::ModelInterpretationRecorded { result_id, .. } => add(result_id, i, K::ResultInterpretation, "result_id; interpretation remains a provider claim"),
            P::CasePolicyReplaced { prior_binding_id, .. } => add(prior_binding_id, i, K::PolicyReplacement, "prior_binding_id; not physical causality"),
            P::CasePolicyUnbound { binding_id, .. } => add(binding_id, i, K::PolicyUnbinding, "binding_id; historical Decisions unchanged"),
            P::ReviewInvalidated { invalidation } => add(&invalidation.review_id, i, K::ReviewInvalidation, "invalidation.review_id"),
            P::ExecutionGrantInvalidated { invalidation } => add(&invalidation.grant_id, i, K::GrantInvalidation, "invalidation.grant_id"),
            _ => (),
        }
    }
    let mut all: Vec<_> = relations.into_values().collect();
    // Stable semantic tie-breaking, not digest-order accident. An exact
    // lifecycle link wins over auxiliary support when both reach the same node.
    // Both remain available in full inspection, with their distinct meaning.
    all.sort_by_key(|r| {
        (
            r.known_at_generation,
            r.kind == RelationKind::EvidenceSupport,
            r.kind.clone(),
            r.from_event.clone(),
        )
    });
    let (keep, edges, result) = if let (Some(from), Some(to)) = (&query.from, &query.to) {
        let (&start, &end) = aliases
            .get(from)
            .zip(aliases.get(to))
            .ok_or("experience_anchor_unavailable")?;
        if !events[start].source_closed || !events[end].source_closed {
            return Err("experience_anchor_backing_unavailable".into());
        }
        let mut queue = VecDeque::from([(start, 0usize)]);
        let mut visited = BTreeSet::from([start]);
        let mut previous = BTreeMap::new();
        let mut outgoing: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for (index, r) in all.iter().enumerate() {
            if r.kind != RelationKind::RecordedBefore || query.include_recording_order {
                outgoing
                    .entry(aliases[&r.from_event])
                    .or_default()
                    .push(index);
            }
        }
        while let Some((node, depth)) = queue.pop_front() {
            if node == end {
                break;
            }
            if depth == query.max_hops {
                continue;
            }
            for &edge in outgoing.get(&node).into_iter().flatten() {
                let next = aliases[&all[edge].to_event];
                if visited.insert(next) {
                    previous.insert(next, (node, edge));
                    queue.push_back((next, depth + 1));
                }
            }
        }
        if visited.contains(&end) {
            let mut nodes = BTreeSet::from([end]);
            let mut edges = Vec::new();
            let mut at = end;
            while at != start {
                let &(p, e) = &previous[&at];
                edges.push(all[e].clone());
                nodes.insert(p);
                at = p;
            }
            edges.reverse();
            (nodes, edges, "qualified_path")
        } else {
            (
                BTreeSet::from([start, end]),
                Vec::new(),
                "no_qualified_path_within_profile_and_hops",
            )
        }
    } else {
        ((0..events.len()).collect(), all, "qualified_relations")
    };
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    // Derive slice identity from all visible members, then select the output
    // intersection. Neither hidden members nor arbitrary payload fields enter it.
    for e in &h.known_by_then {
        if let Some(key) = episode_key(e) {
            groups.entry(key).or_default().push(e.transition_id.clone());
        }
    }
    let visible_ids: BTreeSet<_> = keep
        .iter()
        .map(|i| events[*i].transition_id.clone())
        .collect();
    let slices = groups
        .into_values()
        .filter_map(|ids| {
            let selected: Vec<_> = ids
                .iter()
                .filter(|id| visible_ids.contains(*id))
                .cloned()
                .collect();
            (!selected.is_empty()).then(|| ExperienceEpisodeSlice {
                slice_id: format!(
                    "experience-episode-slice:{}",
                    digest(&(EXPERIENCE_SCHEMA, &h.case_id, &ids))
                ),
                transition_ids: selected,
            })
        })
        .collect();
    let mut view = ExperienceView {
        schema: EXPERIENCE_SCHEMA.into(),
        view_id: String::new(),
        case_id: h.case_id.clone(),
        generation: h.generation,
        participant_id: h.request.participant_id.clone(),
        current_disclosure_digest: current_disclosure_digest.into(),
        query,
        events: keep.into_iter().map(|i| events[i].clone()).collect(),
        relations: edges,
        episode_slices: slices,
        result: result.into(),
        limitations: vec![
            "derived inspection; never authority, Recall or causal discovery".into(),
            "recording order is not event order; occurrence time unsupported".into(),
            "typed structural/support/outcome relations, not universal physical causality".into(),
            "no inferred or model-proposed causal edges; no arbitrary causal_refs edges".into(),
            "current disclosure; unavailable and hidden paths indistinguishable".into(),
            "scoped operation/result/turn Episode slices; W20 records unchanged".into(),
            "general contradiction/assertion/Workflow/Handoff traversal outside initial profile"
                .into(),
        ],
    };
    if view.events.len() > view.query.max_events || view.relations.len() > view.query.max_relations
    {
        return Err("experience_output_budget_exceeded".into());
    }
    view.view_id = format!("experience-view:{}", digest(&view));
    if serde_json::to_vec(&view).map_err(|e| e.to_string())?.len() > view.query.max_bytes {
        return Err("experience_output_budget_exceeded".into());
    }
    Ok(view)
}
