//! Pure semantic residency planning for one provider invocation.
//!
//! A plan selects already-qualified Projection entries under explicit item and
//! semantic-size budgets. It owns no state: prior-frame membership is only an
//! optimization signal, and deleting every plan leaves Case continuity intact.

use crate::context::{
    refresh_projection_identity, AuthorityPosture, ProjectedValue, Projection, ProjectionEntry,
    ProjectionPurpose,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const RESIDENCY_PLAN_SCHEMA: &str = "yai.residency_plan.v1";
pub const DEFAULT_MAX_RESIDENT_ITEMS: usize = 24;
pub const DEFAULT_SEMANTIC_UNIT_BUDGET: usize = 4096;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResidencyClass {
    MandatoryCurrent,
    ObservedConsequence,
    DerivedMemory,
    ProviderClaim,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResidencyDisposition {
    Pinned,
    Retained,
    Reintroduced,
    Omitted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResidencyDecision {
    pub item_id: String,
    pub class: ResidencyClass,
    pub disposition: ResidencyDisposition,
    pub semantic_units: usize,
    pub score: i64,
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResidencyRequest {
    pub case_id: String,
    pub case_generation: u64,
    pub participant_id: String,
    pub purpose: ProjectionPurpose,
    pub provider_id: String,
    pub model_id: String,
    pub max_items: usize,
    pub max_semantic_units: usize,
    #[serde(default)]
    pub resource_refs: Vec<String>,
    #[serde(default)]
    pub previous_item_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResidencyPlan {
    pub schema: String,
    pub plan_id: String,
    pub request: ResidencyRequest,
    pub source_projection_id: String,
    pub source_item_count: usize,
    pub source_semantic_units: usize,
    pub selected_item_ids: Vec<String>,
    pub selected_semantic_units: usize,
    pub omitted_item_count: usize,
    pub decisions: Vec<ResidencyDecision>,
}

impl ResidencyPlan {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != RESIDENCY_PLAN_SCHEMA {
            return Err(format!(
                "unsupported_residency_plan_schema: {}",
                self.schema
            ));
        }
        if self.request.max_items == 0 || self.request.max_semantic_units == 0 {
            return Err("residency_budget_must_be_positive".to_string());
        }
        if self.selected_item_ids.len() > self.request.max_items
            || self.selected_semantic_units > self.request.max_semantic_units
        {
            return Err("residency_plan_exceeds_budget".to_string());
        }
        let selected = self.selected_item_ids.iter().collect::<BTreeSet<_>>();
        if selected.len() != self.selected_item_ids.len() {
            return Err("residency_duplicate_selected_item".to_string());
        }
        let decision_selected = self
            .decisions
            .iter()
            .filter(|decision| decision.disposition != ResidencyDisposition::Omitted)
            .map(|decision| &decision.item_id)
            .collect::<BTreeSet<_>>();
        if selected != decision_selected {
            return Err("residency_selection_decision_mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Clone)]
struct Candidate {
    entry: ProjectionEntry,
    class: ResidencyClass,
    semantic_units: usize,
    score: i64,
    reasons: Vec<String>,
    previous: bool,
    source_order: usize,
}

pub fn plan_residency(
    projection: &Projection,
    request: ResidencyRequest,
) -> Result<ResidencyPlan, String> {
    if request.max_items == 0 || request.max_semantic_units == 0 {
        return Err("residency_budget_must_be_positive".to_string());
    }
    if projection.case_id != request.case_id
        || projection.case_generation != request.case_generation
        || projection.participant_id != request.participant_id
        || projection.purpose != request.purpose
    {
        return Err("residency_projection_request_mismatch".to_string());
    }
    let previous = request.previous_item_ids.iter().collect::<BTreeSet<_>>();
    let mut candidates = projection
        .entries
        .iter()
        .cloned()
        .enumerate()
        .map(|(source_order, entry)| candidate(entry, source_order, &previous, &request))
        .collect::<Result<Vec<_>, _>>()?;
    let source_semantic_units = candidates
        .iter()
        .map(|candidate| candidate.semantic_units)
        .sum();

    let mandatory = candidates
        .iter()
        .filter(|candidate| {
            matches!(
                candidate.class,
                ResidencyClass::MandatoryCurrent | ResidencyClass::ObservedConsequence
            )
        })
        .collect::<Vec<_>>();
    let mandatory_units = mandatory
        .iter()
        .map(|candidate| candidate.semantic_units)
        .sum::<usize>();
    if mandatory.len() > request.max_items || mandatory_units > request.max_semantic_units {
        return Err(format!(
            "residency_budget_below_mandatory_state: required_items={} max_items={} required_units={} max_units={}",
            mandatory.len(),
            request.max_items,
            mandatory_units,
            request.max_semantic_units
        ));
    }

    let mut selected = mandatory
        .iter()
        .map(|candidate| candidate.entry.entry_id.clone())
        .collect::<BTreeSet<_>>();
    let mut selected_units = mandatory_units;
    let mut selected_count = mandatory.len();
    let mut optional = candidates
        .iter_mut()
        .filter(|candidate| {
            !matches!(
                candidate.class,
                ResidencyClass::MandatoryCurrent | ResidencyClass::ObservedConsequence
            )
        })
        .collect::<Vec<_>>();
    optional.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.source_order.cmp(&right.source_order))
            .then_with(|| left.entry.entry_id.cmp(&right.entry.entry_id))
    });
    for candidate in optional {
        if selected_count < request.max_items
            && selected_units.saturating_add(candidate.semantic_units) <= request.max_semantic_units
        {
            selected.insert(candidate.entry.entry_id.clone());
            selected_count += 1;
            selected_units += candidate.semantic_units;
        }
    }

    candidates.sort_by_key(|candidate| candidate.source_order);
    let mut decisions = Vec::with_capacity(candidates.len());
    let mut selected_item_ids = Vec::with_capacity(selected.len());
    for candidate in candidates {
        let is_selected = selected.contains(&candidate.entry.entry_id);
        let disposition = if is_selected
            && matches!(
                candidate.class,
                ResidencyClass::MandatoryCurrent | ResidencyClass::ObservedConsequence
            ) {
            ResidencyDisposition::Pinned
        } else if is_selected && candidate.previous {
            ResidencyDisposition::Retained
        } else if is_selected {
            ResidencyDisposition::Reintroduced
        } else {
            ResidencyDisposition::Omitted
        };
        let mut reasons = candidate.reasons;
        reasons.push(match disposition {
            ResidencyDisposition::Pinned => "mandatory_current_truth".to_string(),
            ResidencyDisposition::Retained => "selected_previous_residency".to_string(),
            ResidencyDisposition::Reintroduced => "selected_for_current_invocation".to_string(),
            ResidencyDisposition::Omitted => "omitted_by_item_or_semantic_budget".to_string(),
        });
        if is_selected {
            selected_item_ids.push(candidate.entry.entry_id.clone());
        }
        decisions.push(ResidencyDecision {
            item_id: candidate.entry.entry_id,
            class: candidate.class,
            disposition,
            semantic_units: candidate.semantic_units,
            score: candidate.score,
            reasons,
        });
    }
    let identity = serde_json::to_string(&(
        RESIDENCY_PLAN_SCHEMA,
        &request,
        &projection.projection_id,
        &selected_item_ids,
        selected_units,
        &decisions,
    ))
    .map_err(|error| format!("residency_identity_encode_failed: {error}"))?;
    let plan = ResidencyPlan {
        schema: RESIDENCY_PLAN_SCHEMA.to_string(),
        plan_id: format!("residency:{}", crate::context::stable_digest(&identity)),
        request,
        source_projection_id: projection.projection_id.clone(),
        source_item_count: projection.entries.len(),
        source_semantic_units,
        selected_item_ids,
        selected_semantic_units: selected_units,
        omitted_item_count: projection.entries.len().saturating_sub(selected.len()),
        decisions,
    };
    plan.validate()?;
    Ok(plan)
}

pub fn apply_residency_plan(
    projection: &Projection,
    plan: &ResidencyPlan,
) -> Result<Projection, String> {
    plan.validate()?;
    if projection.projection_id != plan.source_projection_id
        || projection.case_id != plan.request.case_id
        || projection.case_generation != plan.request.case_generation
    {
        return Err("residency_plan_source_projection_mismatch".to_string());
    }
    let selected = plan.selected_item_ids.iter().collect::<BTreeSet<_>>();
    let mut output = projection.clone();
    output
        .entries
        .retain(|entry| selected.contains(&entry.entry_id));
    if output.entries.len() != selected.len() {
        return Err("residency_plan_selected_item_missing".to_string());
    }
    output.bounds.max_items = plan.request.max_items;
    output.bounds.selected_items = output.entries.len();
    output.bounds.omitted_items = output
        .bounds
        .omitted_items
        .saturating_add(plan.omitted_item_count);
    output.bounds.residency_plan_id = Some(plan.plan_id.clone());
    output.bounds.semantic_unit_budget = Some(plan.request.max_semantic_units);
    output.bounds.selected_semantic_units = Some(plan.selected_semantic_units);
    refresh_projection_identity(&mut output)?;
    Ok(output)
}

fn candidate(
    entry: ProjectionEntry,
    source_order: usize,
    previous: &BTreeSet<&String>,
    request: &ResidencyRequest,
) -> Result<Candidate, String> {
    let encoded = serde_json::to_string(&entry)
        .map_err(|error| format!("residency_entry_encode_failed: {error}"))?;
    let semantic_units = encoded.chars().count().div_ceil(4).max(1);
    let (class, mut score, mut reasons) = classify(&entry);
    if !request.resource_refs.is_empty() && class == ResidencyClass::DerivedMemory {
        score += 40;
        reasons.push("qualified_resource_focus:+40".to_string());
    }
    match (&request.purpose, &entry.value) {
        (
            ProjectionPurpose::FilesystemWriteProposal,
            ProjectedValue::DerivedMemory { semantic_kind, .. },
        ) if matches!(semantic_kind.as_str(), "decision" | "normalization_failure") => {
            score += 35;
            reasons.push("write_proposal_control_experience:+35".to_string());
        }
        (
            ProjectionPurpose::EffectConsequence,
            ProjectedValue::DerivedMemory { semantic_kind, .. },
        ) if matches!(
            semantic_kind.as_str(),
            "resource_effect" | "unresolved_effect"
        ) =>
        {
            score += 45;
            reasons.push("effect_consequence_experience:+45".to_string());
        }
        (ProjectionPurpose::Conversation, ProjectedValue::ProviderClaim { .. }) => {
            score += 10;
            reasons.push("conversation_claim_context:+10".to_string());
        }
        _ => {}
    }
    let was_previous = previous.contains(&entry.entry_id);
    if was_previous {
        score += 20;
        reasons.push("previous_frame_presence:+20".to_string());
    }
    Ok(Candidate {
        entry,
        class,
        semantic_units,
        score,
        reasons,
        previous: was_previous,
        source_order,
    })
}

fn classify(entry: &ProjectionEntry) -> (ResidencyClass, i64, Vec<String>) {
    match (&entry.posture, &entry.value) {
        (AuthorityPosture::ObservedResourceState, _) => (
            ResidencyClass::ObservedConsequence,
            900,
            vec!["observed_resource_consequence:+900".to_string()],
        ),
        (AuthorityPosture::Unresolved, _) => (
            ResidencyClass::MandatoryCurrent,
            1000,
            vec!["unresolved_state_pinned:+1000".to_string()],
        ),
        (AuthorityPosture::CommittedOperationalFact, _) | (AuthorityPosture::ControlState, _) => (
            ResidencyClass::MandatoryCurrent,
            1000,
            vec!["current_operational_state_pinned:+1000".to_string()],
        ),
        (AuthorityPosture::DerivedMemory, ProjectedValue::DerivedMemory { score, .. }) => (
            ResidencyClass::DerivedMemory,
            200 + *score,
            vec![format!("qualified_memory:+{}", 200 + *score)],
        ),
        _ => (
            ResidencyClass::ProviderClaim,
            10,
            vec!["provider_claim_optional:+10".to_string()],
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{
        build_context_frame, compile_projection, DerivedMemoryInput, DerivedProjectionInput,
        InvocationOutputContract, ProjectionBounds, ProjectionRequest, ProjectionVisibility,
        ProvenanceKind, SemanticProvenance, PROJECTION_SCHEMA,
    };
    use crate::memory::{
        derive_operational_memory, retrieve_operational_memory, RetrievalQualification,
    };
    use crate::transition::{
        replay_case, CaseLifecycle, CaseState, ProviderInvocationLineage, Transition,
        TransitionPayload, TransitionSource, TRANSITION_SCHEMA,
    };

    fn entry(index: usize, posture: AuthorityPosture) -> ProjectionEntry {
        let value = match posture {
            AuthorityPosture::CommittedOperationalFact => ProjectedValue::CaseLifecycle {
                lifecycle: CaseLifecycle::Open,
            },
            AuthorityPosture::ObservedResourceState => ProjectedValue::ResourceConsequence {
                operation_id: format!("operation:{index}"),
                effect_id: format!("effect:{index}"),
                relative_path: format!("allowed/{index}.txt"),
                lifecycle: crate::transition::EffectLifecycle::Finalized,
                outcome: Some(crate::effect::EffectOutcome::Applied),
                content_digest: Some(format!("digest:{index}")),
                receipt_id: Some(format!("receipt:{index}")),
            },
            AuthorityPosture::DerivedMemory => ProjectedValue::DerivedMemory {
                memory_ref: format!("memory:{index}"),
                semantic_kind: "resource_effect".to_string(),
                memory_posture: "finalized_observed_consequence".to_string(),
                description: format!("derived operational consequence {index}"),
                lifecycle: "active".to_string(),
                score: index as i64,
                ranking_reasons: vec!["resource".to_string()],
            },
            _ => ProjectedValue::ProviderClaim {
                result_id: format!("result:{index}"),
                invocation_id: format!("invocation:{index}"),
                preview: format!("claim {index}"),
            },
        };
        ProjectionEntry {
            entry_id: format!("entry:{index}"),
            posture,
            value,
            provenance: vec![SemanticProvenance {
                kind: ProvenanceKind::Transition,
                source_ref: format!("transition:{index}"),
            }],
        }
    }

    fn projection(entries: Vec<ProjectionEntry>) -> Projection {
        Projection {
            schema: PROJECTION_SCHEMA.to_string(),
            projection_id: "projection:candidate".to_string(),
            case_id: "case:runtime".to_string(),
            case_generation: 42,
            participant_id: "participant:model".to_string(),
            purpose: ProjectionPurpose::Conversation,
            visibility: ProjectionVisibility {
                consumer: "model".to_string(),
                view_kind: "model_context".to_string(),
            },
            bounds: ProjectionBounds {
                max_items: entries.len(),
                selected_items: entries.len(),
                omitted_items: 0,
                history_transitions_considered: 42,
                graph_available: false,
                memory_available: true,
                retrieval_id: None,
                retrieval_candidates: entries.len(),
                retrieval_selected: entries.len(),
                retrieval_omitted: 0,
                residency_plan_id: None,
                semantic_unit_budget: None,
                selected_semantic_units: None,
            },
            entries,
        }
    }

    fn request(max_items: usize, max_semantic_units: usize) -> ResidencyRequest {
        ResidencyRequest {
            case_id: "case:runtime".to_string(),
            case_generation: 42,
            participant_id: "participant:model".to_string(),
            purpose: ProjectionPurpose::Conversation,
            provider_id: "provider:a".to_string(),
            model_id: "model:a".to_string(),
            max_items,
            max_semantic_units,
            resource_refs: Vec::new(),
            previous_item_ids: Vec::new(),
        }
    }

    #[test]
    fn mandatory_truth_is_pinned_before_optional_memory() {
        let projection = projection(vec![
            entry(0, AuthorityPosture::CommittedOperationalFact),
            entry(1, AuthorityPosture::ObservedResourceState),
            entry(2, AuthorityPosture::DerivedMemory),
            entry(3, AuthorityPosture::ProviderClaim),
        ]);
        let plan = plan_residency(&projection, request(3, 10_000)).expect("plan");
        assert_eq!(plan.selected_item_ids.len(), 3);
        assert!(plan.selected_item_ids.contains(&"entry:0".to_string()));
        assert!(plan.selected_item_ids.contains(&"entry:1".to_string()));
        assert!(plan.selected_item_ids.contains(&"entry:2".to_string()));
        assert!(!plan.selected_item_ids.contains(&"entry:3".to_string()));
        let applied = apply_residency_plan(&projection, &plan).expect("apply");
        assert_eq!(applied.bounds.residency_plan_id, Some(plan.plan_id));
        assert_eq!(applied.entries.len(), 3);
    }

    #[test]
    fn planning_is_deterministic_and_reacts_to_previous_residency() {
        let projection = projection(vec![
            entry(0, AuthorityPosture::CommittedOperationalFact),
            entry(1, AuthorityPosture::DerivedMemory),
            entry(2, AuthorityPosture::DerivedMemory),
        ]);
        let first = plan_residency(&projection, request(2, 10_000)).expect("first");
        let second = plan_residency(&projection, request(2, 10_000)).expect("second");
        assert_eq!(first, second);
        let mut changed = request(2, 10_000);
        changed.previous_item_ids = vec!["entry:1".to_string()];
        let retained = plan_residency(&projection, changed).expect("retained");
        assert!(retained.decisions.iter().any(|decision| {
            decision.item_id == "entry:1" && decision.disposition == ResidencyDisposition::Retained
        }));
    }

    #[test]
    fn planning_reacts_to_purpose_and_qualified_resource_focus() {
        let projection = projection(vec![
            entry(0, AuthorityPosture::CommittedOperationalFact),
            ProjectionEntry {
                entry_id: "entry:decision-memory".to_string(),
                posture: AuthorityPosture::DerivedMemory,
                value: ProjectedValue::DerivedMemory {
                    memory_ref: "memory:decision".to_string(),
                    semantic_kind: "decision".to_string(),
                    memory_posture: "decision_control_history".to_string(),
                    description: "prior bounded denial".to_string(),
                    lifecycle: "active".to_string(),
                    score: 5,
                    ranking_reasons: vec!["decision".to_string()],
                },
                provenance: vec![SemanticProvenance {
                    kind: ProvenanceKind::Transition,
                    source_ref: "transition:decision".to_string(),
                }],
            },
            entry(2, AuthorityPosture::DerivedMemory),
        ]);
        let conversation = plan_residency(&projection, request(2, 10_000)).expect("conversation");
        let mut focused_request = request(2, 10_000);
        focused_request.purpose = ProjectionPurpose::FilesystemWriteProposal;
        focused_request.resource_refs = vec!["workspace".to_string()];
        let mut focused_projection = projection.clone();
        focused_projection.purpose = ProjectionPurpose::FilesystemWriteProposal;
        let focused = plan_residency(&focused_projection, focused_request).expect("focused");
        assert_ne!(conversation.plan_id, focused.plan_id);
        assert!(focused.decisions.iter().any(|decision| {
            decision.item_id == "entry:decision-memory"
                && decision
                    .reasons
                    .iter()
                    .any(|reason| reason == "write_proposal_control_experience:+35")
                && decision
                    .reasons
                    .iter()
                    .any(|reason| reason == "qualified_resource_focus:+40")
        }));
    }

    #[test]
    fn impossible_mandatory_budget_fails_explicitly() {
        let projection = projection(vec![
            entry(0, AuthorityPosture::CommittedOperationalFact),
            entry(1, AuthorityPosture::ObservedResourceState),
        ]);
        let error = plan_residency(&projection, request(1, 10_000))
            .expect_err("mandatory budget must fail");
        assert!(error.contains("residency_budget_below_mandatory_state"));
    }

    #[test]
    fn hundred_iteration_planning_remains_bounded() {
        let mut previous = Vec::new();
        for iteration in 0..128 {
            let mut entries = vec![entry(0, AuthorityPosture::CommittedOperationalFact)];
            entries.extend(
                (1..=iteration + 32).map(|index| entry(index, AuthorityPosture::DerivedMemory)),
            );
            let projection = projection(entries);
            let mut request = request(8, 10_000);
            request.previous_item_ids = previous;
            let plan = plan_residency(&projection, request).expect("bounded plan");
            assert!(plan.selected_item_ids.len() <= 8);
            assert!(plan.selected_semantic_units <= 10_000);
            previous = plan.selected_item_ids;
        }
    }

    #[test]
    fn hundred_iteration_case_state_memory_context_endurance() {
        fn committed(
            sequence: u64,
            payload: TransitionPayload,
            causal_refs: Vec<String>,
        ) -> Transition {
            Transition {
                schema: TRANSITION_SCHEMA.to_string(),
                transition_id: format!("transition:endurance:{sequence}"),
                case_id: "case:endurance".to_string(),
                sequence,
                committed_at_unix_ms: sequence,
                source: TransitionSource::component("test.case_runtime"),
                scope: None,
                causal_refs,
                payload,
                provenance: Vec::new(),
                summary: None,
            }
        }

        let lineage = |index: usize, generation: u64| ProviderInvocationLineage {
            projection_id: format!("projection:{index}"),
            context_frame_id: format!("frame:{index}"),
            case_generation: generation,
            rendered_input_id: format!("rendered:{index}"),
            rendered_input_digest: format!("digest:{index}"),
            output_contract_id: "output-contract:endurance".to_string(),
            continuation_disposition: "not_provided".to_string(),
        };
        let mut history = Vec::new();
        let initial = [
            TransitionPayload::CaseOpened {
                lifecycle: CaseLifecycle::Open,
            },
            TransitionPayload::ParticipantBound {
                participant_id: "participant:model".to_string(),
                role: "model_provider".to_string(),
            },
            TransitionPayload::ParticipantAdmitted {
                participant_id: "participant:model".to_string(),
                consumer: "model".to_string(),
                view_kind: "model_context".to_string(),
            },
            TransitionPayload::ProviderAttached {
                participant_id: "participant:model".to_string(),
                provider_id: "provider:endurance".to_string(),
                provider_kind: "openai_compatible".to_string(),
                base_url: "http://127.0.0.1:1/v1/chat/completions".to_string(),
                model_id: "model:endurance".to_string(),
                credential_ref: "env:YAI_TEST_KEY".to_string(),
            },
        ];
        let mut state = CaseState::new("case:endurance", CaseLifecycle::Open);
        for payload in initial {
            let transition = committed(history.len() as u64 + 1, payload, Vec::new());
            state = state.reduce(&transition).expect("initial reduction");
            history.push(transition);
        }
        for index in 0..128usize {
            let invocation_id = format!("invocation:{index}");
            let result_id = format!("result:{index}");
            let invocation_lineage = lineage(index, state.generation);
            let invocation = committed(
                history.len() as u64 + 1,
                TransitionPayload::ProviderInvocationStarted {
                    invocation_id: invocation_id.clone(),
                    participant_id: "participant:model".to_string(),
                    provider_id: "provider:endurance".to_string(),
                    provider_kind: "openai_compatible".to_string(),
                    model_id: "model:endurance".to_string(),
                    semantic_lineage: Some(invocation_lineage.clone()),
                    governance: None,
                },
                Vec::new(),
            );
            state = state.reduce(&invocation).expect("invocation reduction");
            history.push(invocation);
            let result = committed(
                history.len() as u64 + 1,
                TransitionPayload::ProviderResultRecorded {
                    result_id: result_id.clone(),
                    invocation_id: invocation_id.clone(),
                    provider_id: "provider:endurance".to_string(),
                    provider_kind: "openai_compatible".to_string(),
                    model_id: "model:endurance".to_string(),
                    semantic_lineage: Some(invocation_lineage),
                    output: format!("bounded provider result {index}"),
                },
                vec![invocation_id.clone()],
            );
            state = state.reduce(&result).expect("result reduction");
            history.push(result);
            let turn = committed(
                history.len() as u64 + 1,
                TransitionPayload::InteractionTurnRecorded {
                    turn_id: format!("turn:{index}"),
                    thread_id: "thread:endurance".to_string(),
                    participant_id: "participant:model".to_string(),
                    invocation_id: invocation_id.clone(),
                    result_id: result_id.clone(),
                    operator_input: format!("bounded task {index}"),
                },
                vec![invocation_id, result_id],
            );
            state = state.reduce(&turn).expect("turn reduction");
            history.push(turn);
        }
        assert_eq!(
            state,
            replay_case("case:endurance", &history).expect("replay")
        );

        let memory = derive_operational_memory("case:endurance", &history).expect("memory");
        assert!(memory.entries.len() >= 128);
        let retrieval = retrieve_operational_memory(
            &state,
            &memory.entries,
            RetrievalQualification {
                case_id: "case:endurance".to_string(),
                participant_id: "participant:model".to_string(),
                consumer: "model".to_string(),
                view_kind: "model_context".to_string(),
                purpose: ProjectionPurpose::Conversation,
                case_generation: state.generation,
                resource_refs: Vec::new(),
                semantic_kinds: Vec::new(),
                causal_refs: Vec::new(),
                max_results: 16,
                include_superseded: false,
            },
        )
        .expect("retrieval");
        assert_eq!(retrieval.selected_count, 16);
        let derived = DerivedProjectionInput {
            graph_available: false,
            memory_available: true,
            memory: retrieval
                .selected
                .iter()
                .map(|item| DerivedMemoryInput {
                    memory_ref: item.memory.memory_id.clone(),
                    semantic_kind: item.memory.semantic_kind.as_str().to_string(),
                    memory_posture: item.memory.posture.as_str().to_string(),
                    description: item.memory.description.clone(),
                    lifecycle: item.memory.lifecycle.as_str().to_string(),
                    score: item.score,
                    ranking_reasons: item.ranking_reasons.clone(),
                    transition_refs: item.memory.provenance.transition_ids.clone(),
                    observation_refs: item.memory.provenance.observation_ids.clone(),
                    receipt_refs: item.memory.provenance.effect_receipt_ids.clone(),
                })
                .collect(),
            retrieval_id: Some(retrieval.retrieval_id),
            retrieval_candidates: retrieval.qualified_count,
            retrieval_omitted: retrieval.omitted_count,
        };
        let mut projection_request =
            ProjectionRequest::model("participant:model", ProjectionPurpose::Conversation);
        projection_request.max_items = 256;
        projection_request.max_provider_claims = 128;
        projection_request.max_interaction_turns = 128;
        let candidate = compile_projection(&state, &history, &projection_request, &derived)
            .expect("candidate projection");
        let plan = plan_residency(
            &candidate,
            ResidencyRequest {
                case_id: state.case_id.clone(),
                case_generation: state.generation,
                participant_id: "participant:model".to_string(),
                purpose: ProjectionPurpose::Conversation,
                provider_id: "provider:endurance".to_string(),
                model_id: "model:endurance".to_string(),
                max_items: 12,
                max_semantic_units: 4096,
                resource_refs: Vec::new(),
                previous_item_ids: Vec::new(),
            },
        )
        .expect("residency");
        let bounded = apply_residency_plan(&candidate, &plan).expect("bounded projection");
        let frame = build_context_frame(
            &bounded,
            "continue the durable Case",
            InvocationOutputContract::NaturalLanguage,
        )
        .expect("frame");
        assert!(history.len() > 380);
        assert!(memory.entries.len() >= 128);
        assert!(candidate.entries.len() > frame.entries.len());
        assert!(frame.entries.len() <= 12);
        assert!(serde_json::to_string(&frame).unwrap().len() < 32_000);
    }
}
