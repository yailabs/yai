//! Derived, disclosure-qualified Decision history. Never a Decision or dataset owner.
use super::{
    digest, HistoricalEvidence, HistoricalMaterialization, HistoricalNormative,
    HistoricalSemanticView, SourceClosure,
};
use crate::effect::{Decision, Operation, OperationOrigin};
use crate::graph::experience::{ExperienceRelation, ExperienceView, RelationKind};
use crate::transition::TransitionPayload;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const TRAJECTORY_SCHEMA: &str = "yai.cognitive_decision_trajectory.v1";
pub const CORPUS_SCHEMA: &str = "yai.cognitive_decision_corpus.v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoricalCandidatePosture {
    /// Reserved for exact recorded/reconstructible typed Frontier membership.
    ExactReconstructed,
    /// Only the selected canonical Operation is established. Other alternatives are unknown.
    Partial,
    /// Even the selected Operation cannot be disclosed or reconstructed.
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PreDecisionEvidence {
    pub cut_generation: u64,
    pub state_then: HistoricalMaterialization,
    pub known_by_then: Vec<HistoricalEvidence>,
    pub normative_then: HistoricalNormative,
    pub content_backing: Vec<SourceClosure>,
    pub unsupported_families: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TrajectoryReadiness {
    pub pre_state_available: bool,
    pub working_state_available: bool,
    pub candidate_set_available: bool,
    pub decision_basis_available: bool,
    pub consequence_available: bool,
    pub correction_available: bool,
    pub historical_distribution_available: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CognitiveDecisionTrajectory {
    pub schema: String,
    pub trajectory_id: String,
    pub case_id: String,
    pub participant_id: String,
    pub decision_transition_id: String,
    pub decision_generation: u64,
    pub pre_decision: PreDecisionEvidence,
    pub task_context: Option<String>,
    pub candidate_posture: HistoricalCandidatePosture,
    pub exact_candidates: Vec<crate::cognitive::CognitiveDecisionCandidate>,
    /// A selected Operation is not an exact historical Frontier.
    pub selected_operation: Option<Operation>,
    pub historical_decision_request_id: Option<String>,
    pub historical_distribution_id: Option<String>,
    pub decision: Decision,
    pub related_evidence: Vec<HistoricalEvidence>,
    /// Only exact qualified non-chronological edges, with typed source provenance.
    pub related_relations: Vec<ExperienceRelation>,
    pub correction_decisions: Vec<String>,
    pub missingness: Vec<String>,
    pub readiness: TrajectoryReadiness,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CognitiveDecisionCorpus {
    pub schema: String,
    pub case_id: String,
    pub participant_id: String,
    pub trajectories: Vec<CognitiveDecisionTrajectory>,
    pub omitted_visible_decisions: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TrajectoryEvaluation {
    pub schema: String,
    pub trajectory_count: usize,
    pub exact_candidate_set_count: usize,
    pub partial_candidate_set_count: usize,
    pub historical_working_state_count: usize,
    pub consequence_link_count: usize,
    pub correction_count: usize,
    pub missing_backing_count: usize,
    pub temporal_leakage_violations: usize,
    pub false_causality_violations: usize,
    pub cross_case_leakage_violations: usize,
    pub serialized_bytes: usize,
}

/// The selected Decision must already be present in a current-disclosure-qualified
/// historical view. No raw Transition or current policy is accepted as input.
pub(crate) fn derive(
    pre: HistoricalSemanticView,
    current: &HistoricalSemanticView,
    graph: &ExperienceView,
    decision_id: &str,
) -> Result<CognitiveDecisionTrajectory, String> {
    if pre.case_id != current.case_id
        || graph.case_id != current.case_id
        || pre.request.participant_id != current.request.participant_id
        || graph.participant_id != current.request.participant_id
        || pre.current_generation != current.current_generation
    {
        return Err("trajectory_qualification_mismatch".into());
    }
    let (decision_event, decision) = current
        .known_by_then
        .iter()
        .find_map(|event| {
            if let TransitionPayload::DecisionRecorded { decision } = &event.payload {
                (decision.decision_id == decision_id).then_some((event, decision))
            } else {
                None
            }
        })
        .ok_or("decision_not_visible")?;
    if pre.generation.checked_add(1) != Some(decision_event.recorded_generation)
        || pre
            .known_by_then
            .iter()
            .any(|event| event.recorded_generation >= decision_event.recorded_generation)
    {
        return Err("trajectory_temporal_cut_invalid".into());
    }
    let selected_operation = pre.known_by_then.iter().find_map(|event| {
        if let TransitionPayload::OperationRecorded { operation } = &event.payload {
            (operation.operation_id == decision.operation_id).then_some(operation.clone())
        } else {
            None
        }
    });
    let task_context = selected_operation.as_ref().and_then(|operation| {
        if let OperationOrigin::WorkflowDeterministicProposal {
            workflow_execution_id,
            ..
        } = &operation.origin
        {
            Some(workflow_execution_id.clone())
        } else {
            None
        }
    });
    let candidate_posture = if selected_operation.is_some() {
        HistoricalCandidatePosture::Partial
    } else {
        HistoricalCandidatePosture::Unavailable
    };
    let by_id: BTreeMap<_, _> = current
        .known_by_then
        .iter()
        .map(|event| (event.transition_id.as_str(), event))
        .collect();
    let mut outgoing: BTreeMap<&str, Vec<&ExperienceRelation>> = BTreeMap::new();
    for relation in &graph.relations {
        if relation.kind == RelationKind::RecordedBefore {
            continue;
        }
        outgoing
            .entry(&relation.from_event)
            .or_default()
            .push(relation);
    }
    let mut reached = BTreeSet::from([decision_event.transition_id.as_str()]);
    let mut queue = VecDeque::from([decision_event.transition_id.as_str()]);
    let mut relation_ids = BTreeSet::new();
    while let Some(from) = queue.pop_front() {
        // A review-derived later Decision is explicit correction evidence, but
        // its own effect lineage belongs to that later Decision's trajectory.
        if from != decision_event.transition_id.as_str()
            && by_id.get(from).is_some_and(|event| {
                matches!(&event.payload, TransitionPayload::DecisionRecorded { .. })
            })
        {
            continue;
        }
        for relation in outgoing.get(from).into_iter().flatten() {
            if !by_id.contains_key(relation.to_event.as_str()) {
                continue;
            }
            relation_ids.insert(relation.relation_id.as_str());
            if reached.insert(relation.to_event.as_str()) {
                queue.push_back(relation.to_event.as_str());
            }
        }
    }
    let related_relations: Vec<_> = graph
        .relations
        .iter()
        .filter(|relation| relation_ids.contains(relation.relation_id.as_str()))
        .cloned()
        .collect();
    let related_evidence: Vec<_> = current
        .known_by_then
        .iter()
        .filter(|event| {
            event.recorded_generation > decision_event.recorded_generation
                && reached.contains(event.transition_id.as_str())
        })
        .cloned()
        .collect();
    let correction_decisions: Vec<_> = related_relations
        .iter()
        .filter(|relation| relation.kind == RelationKind::ReviewReevaluation)
        .filter_map(|relation| by_id.get(relation.to_event.as_str()))
        .filter_map(|event| {
            if let TransitionPayload::DecisionRecorded { decision } = &event.payload {
                Some(decision.decision_id.clone())
            } else {
                None
            }
        })
        .collect();
    let consequence_available = related_relations.iter().any(|relation| {
        matches!(
            relation.kind,
            RelationKind::PreparedConsequence | RelationKind::DecisionObservation
        )
    });
    let mut missingness = vec![
        "historical_task_conditioned_recall_and_w_not_recorded".into(),
        "historical_cognitive_decision_request_not_recorded".into(),
        "historical_cognitive_distribution_not_recorded".into(),
        "counterfactual_candidate_frontier_not_reconstructible".into(),
    ];
    if graph
        .events
        .iter()
        .any(|event| event.transition_id == decision_event.transition_id && !event.source_closed)
    {
        missingness.push("decision_relation_backing_unavailable".into());
    }
    if selected_operation.is_none() {
        missingness.push("selected_operation_unavailable".into());
    }
    for missing in &pre.normative_then.missing {
        missingness.push(format!("historical_policy:{missing}"));
    }
    for closure in &pre.normative_then.source_closure {
        if closure.posture != "exact_immutable_artifact"
            && closure.posture != "exact_binding_publication_anchor"
            && closure.posture != "exact_source_available"
        {
            missingness.push(format!(
                "historical_policy_backing:{}:{}",
                closure.source_ref, closure.posture
            ));
        }
    }
    for closure in &pre.content_backing {
        if closure.posture != "exact_original_available" {
            missingness.push(format!(
                "content_backing:{}:{}",
                closure.source_ref, closure.posture
            ));
        }
    }
    let readiness = TrajectoryReadiness {
        pre_state_available: true,
        working_state_available: false,
        candidate_set_available: false,
        decision_basis_available: decision.decision_basis.is_some(),
        consequence_available,
        correction_available: !correction_decisions.is_empty(),
        historical_distribution_available: false,
    };
    let pre_decision = PreDecisionEvidence {
        cut_generation: pre.generation,
        state_then: pre.state_then,
        known_by_then: pre.known_by_then,
        normative_then: pre.normative_then,
        content_backing: pre.content_backing,
        unsupported_families: pre.unsupported_families,
    };
    let trajectory_id = format!(
        "decision-trajectory:{}",
        digest(&(
            TRAJECTORY_SCHEMA,
            &current.case_id,
            &current.request.participant_id,
            decision_id,
            &pre_decision,
            &related_relations,
            &related_evidence,
            &missingness
        ))
    );
    Ok(CognitiveDecisionTrajectory {
        schema: TRAJECTORY_SCHEMA.into(),
        trajectory_id,
        case_id: current.case_id.clone(),
        participant_id: current.request.participant_id.clone(),
        decision_transition_id: decision_event.transition_id.clone(),
        decision_generation: decision_event.recorded_generation,
        pre_decision,
        task_context,
        candidate_posture,
        exact_candidates: Vec::new(),
        selected_operation,
        historical_decision_request_id: None,
        historical_distribution_id: None,
        decision: decision.clone(),
        related_evidence,
        related_relations,
        correction_decisions,
        missingness,
        readiness,
    })
}

pub fn evaluate(corpus: &CognitiveDecisionCorpus) -> Result<TrajectoryEvaluation, String> {
    let mut result = TrajectoryEvaluation {
        schema: "yai.cognitive_decision_trajectory_evaluation.v1".into(),
        trajectory_count: corpus.trajectories.len(),
        exact_candidate_set_count: 0,
        partial_candidate_set_count: 0,
        historical_working_state_count: 0,
        consequence_link_count: 0,
        correction_count: 0,
        missing_backing_count: 0,
        temporal_leakage_violations: 0,
        false_causality_violations: 0,
        cross_case_leakage_violations: 0,
        serialized_bytes: serde_json::to_vec(corpus)
            .map_err(|error| error.to_string())?
            .len(),
    };
    for trajectory in &corpus.trajectories {
        result.exact_candidate_set_count += usize::from(
            trajectory.candidate_posture == HistoricalCandidatePosture::ExactReconstructed,
        );
        result.partial_candidate_set_count +=
            usize::from(trajectory.candidate_posture == HistoricalCandidatePosture::Partial);
        result.historical_working_state_count +=
            usize::from(trajectory.readiness.working_state_available);
        result.consequence_link_count += usize::from(trajectory.readiness.consequence_available);
        result.correction_count += usize::from(trajectory.readiness.correction_available);
        result.missing_backing_count += usize::from(trajectory.missingness.iter().any(|item| {
            item.starts_with("content_backing:")
                || item.starts_with("historical_policy:")
                || item.starts_with("historical_policy_backing:")
        }));
        result.temporal_leakage_violations += trajectory
            .pre_decision
            .known_by_then
            .iter()
            .filter(|event| event.recorded_generation >= trajectory.decision_generation)
            .count();
        result.false_causality_violations += trajectory
            .related_relations
            .iter()
            .filter(|relation| relation.kind == RelationKind::RecordedBefore)
            .count();
        result.cross_case_leakage_violations += usize::from(trajectory.case_id != corpus.case_id);
    }
    Ok(result)
}
