//! Finite Case work advancement over the existing cognitive and effect seams.
//! Every cursor, counter and feedback exchange is rebuilt from canonical facts.
//! This is process-local control, not an Agent, Workflow or execution owner.
use super::super::cognitive_execution::{realize_cognitive, source_parts, RecordedExecution};
use super::super::controlled_effect::{
    self, access::ResourceActionOutcome, ControlledEffectTurnStatus,
};
use super::*;
use serde_json::{json, Value};
use yai_core_engine::admission::{
    CaseCapabilityAction, CaseCapabilityOutput, CASE_CAPABILITY_OUTPUT_SCHEMA,
};
use yai_core_engine::cognitive::{
    cognitive_execution_lane_for_target, cognitive_provider_requirement,
    CognitiveCapabilityRequirement, CognitivePlanRoute,
};
use yai_core_engine::context::InvocationOutputContract;
use yai_core_engine::effect::OperationKind;
use yai_core_engine::transition::Transition;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct CaseWorkStep {
    ordinal: u16,
    plan_id: String,
    lane_id: String,
    target_id: String,
    provider_result_id: String,
    operation_id: Option<String>,
    outcome: Option<Value>,
    recovered: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct CaseWorkOutcome {
    pub request_id: String,
    pub steps: Vec<CaseWorkStep>,
}

impl CaseWorkOutcome {
    /// Compact operator presentation; exact structured outcomes remain in the
    /// application result and canonical resource/provider lineage, not stdout.
    pub(crate) fn operator_summary(&self) -> String {
        let mut lines = vec![format!("case_work: {}", self.request_id)];
        for step in &self.steps {
            let posture = step
                .outcome
                .as_ref()
                .and_then(|v| v.get("posture").or_else(|| v.get("status")))
                .and_then(Value::as_str)
                .unwrap_or("provider_result");
            lines.push(format!(
                "  {}. {} reused={} {}",
                step.ordinal + 1,
                posture,
                step.recovered,
                step.operation_id
                    .as_deref()
                    .unwrap_or(&step.provider_result_id)
            ));
            if let Some(id) = step
                .outcome
                .as_ref()
                .and_then(|v| v.get("review_id"))
                .and_then(Value::as_str)
            {
                lines.push(format!("review_id: {id}"));
            }
        }
        if let Some(last) = self.steps.last() {
            lines.push(format!(
                "target_id: {}\nlane_id: {}\nplan_id: {}\nprovider_result_id: {}",
                last.target_id, last.lane_id, last.plan_id, last.provider_result_id
            ));
        }
        lines.join("\n")
    }
}

/// A completed request is recovered before consulting fresh target evidence.
/// A different binding may serve the NEXT step, never repeat a completed one.
fn completed_step(
    history: &[Transition],
    requirement_id: &str,
) -> Result<Option<(RecordedExecution, String)>, String> {
    for selected in history.iter().rev() {
        let TransitionPayload::ProviderSelectionRecorded { selection } = &selected.payload else {
            continue;
        };
        if selection.requirement_id != requirement_id {
            continue;
        }
        if !selected.causal_refs.contains(&format!(
            "provider-normalization-contract:{CASE_CAPABILITY_OUTPUT_SCHEMA}"
        )) {
            return Err("case_work_step_contract_mismatch".into());
        }
        let invocation = history.iter().find_map(|t| match &t.payload {
            TransitionPayload::ProviderInvocationStarted {
                invocation_id,
                governance: Some(g),
                ..
            } if g.selection_id == selection.selection_id => Some(invocation_id),
            _ => None,
        });
        let Some(invocation_id) = invocation else {
            continue;
        };
        let result = history.iter().find_map(|t| match &t.payload {
            TransitionPayload::ProviderResultRecorded {
                result_id,
                invocation_id: id,
                output,
                ..
            } if id == invocation_id => Some((result_id, output)),
            _ => None,
        });
        let Some((result_id, output)) = result else {
            if history.iter().any(|t|matches!(&t.payload,TransitionPayload::ProviderAttemptOutcomeRecorded {outcome} if outcome.selection_id == selection.selection_id && outcome.retry_safe())) {continue;}
            return Err("case_work_prior_delivery_indeterminate".into());
        };
        let binding = history
            .iter()
            .find_map(|t| match &t.payload {
                TransitionPayload::CaseCognitiveBindingRecorded { binding }
                    if selected.causal_refs.contains(&binding.binding_id) =>
                {
                    Some(binding)
                }
                _ => None,
            })
            .ok_or("case_work_historical_binding_missing")?;
        let lane = cognitive_execution_lane_for_target(
            &selected.case_id,
            &selection.participant_id,
            binding,
            &selection.selected_target_id,
        )?;
        return Ok(Some((
            RecordedExecution {
                plan_id: selection
                    .logical_turn_id
                    .strip_prefix("cognitive-realization:")
                    .ok_or("case_work_exact_plan_missing")?
                    .into(),
                selection: selection.clone(),
                invocation_id: invocation_id.clone(),
                result_id: result_id.clone(),
                output: output.clone(),
            },
            lane,
        )));
    }
    Ok(None)
}

pub(super) fn execute(
    controller: &ConversationController,
    authorized: &AuthorizedConversationCase,
    turn: &ConversationTurn,
    intent: &CognitiveCompositionRequest,
    events: &mut Vec<ConversationApplicationEvent>,
    failpoint: Option<&str>,
) -> Result<ConversationExecutionResult, String> {
    intent.validate(turn)?;
    let limits = intent
        .work_limits
        .as_ref()
        .ok_or("case_work_intent_required")?;
    let store = &authorized.store;
    if intent.workflow_execution_id.is_some() {
        store.revalidate_conversation_workflow_intent_authorized(
            &authorized.authenticated,
            intent,
        )?;
    }
    let content = conversation_content_store()?;
    let mut steps = Vec::new();
    let mut feedback_ids = Vec::new();
    let mut effects = 0u16;
    let mut result = unavailable_execution(
        turn,
        ConversationExecutionPosture::BudgetExhausted,
        "case_work_budget_exhausted".into(),
        &mut Vec::new(),
    );
    result.intent = Some(intent.clone());
    for ordinal in 0..limits.invocations {
        let history = store.list_case_transitions(&turn.case_id)?;
        let source = intent.work_step_source(ordinal)?;
        let requirement = CognitiveCapabilityRequirement::new(
            &turn.case_id,
            &intent.participant_id,
            CognitiveCapability::PrimaryConversation,
            &source,
        )?;
        let requirement = cognitive_provider_requirement(&requirement)?;
        let recorded = completed_step(&history, &requirement.requirement_id)?;
        let (execution, lane, recovered) = if let Some((execution, lane)) = recorded {
            (execution, lane, true)
        } else {
            let selection_ids = history
                .iter()
                .filter_map(|t| match &t.payload {
                    TransitionPayload::ProviderSelectionRecorded { selection }
                        if t.causal_refs.contains(&intent.request_id) =>
                    {
                        Some(selection.selection_id.as_str())
                    }
                    _ => None,
                })
                .collect::<std::collections::BTreeSet<_>>();
            let invocation_count = history.iter().filter(|t|matches!(&t.payload,TransitionPayload::ProviderInvocationStarted {governance:Some(g),..} if selection_ids.contains(g.selection_id.as_str()))).count();
            if invocation_count >= usize::from(limits.invocations) {
                break;
            }
            let view = store.case_capability_view_authorized(
                &authorized.authenticated,
                &turn.case_id,
                &intent.participant_id,
            )?;
            let catalogs = history
                .iter()
                .filter_map(|t| match &t.payload {
                    TransitionPayload::ResourceObservationRecorded { observation }
                        if observation.participant_id == intent.participant_id
                            && observation.kind
                                == yai_core_engine::effect::access::AccessKind::McpCatalog =>
                    {
                        Some(observation.clone())
                    }
                    _ => None,
                })
                .rev()
                .take(16)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            let realized = realize_cognitive(
                &authorized.authenticated,
                store,
                &content,
                turn,
                &intent.participant_id,
                CognitiveCapability::PrimaryConversation,
                source_parts(&content, turn, &intent.source_part_ids)?,
                vec![],
                &source,
                Some(CognitivePlanRoute::Native),
                None,
                failpoint,
                &[
                    turn.turn_id.clone(),
                    intent.request_id.clone(),
                    source.clone(),
                ],
                &|| controller.cancellation.is_requested(),
                InvocationOutputContract::CaseCapabilities {
                    view: Box::new(view),
                    catalogs,
                    feedback_result_ids: feedback_ids.clone(),
                },
            )?;
            let lane = realized
                .plan
                .execution_lane_id
                .clone()
                .ok_or("case_work_lane_missing")?;
            (
                realized.execution.ok_or("case_work_result_missing")?,
                lane,
                realized.recovered,
            )
        };
        let output: CaseCapabilityOutput =
            serde_json::from_str(&execution.output).map_err(|_| "case_work_output_invalid")?;
        if output.schema != CASE_CAPABILITY_OUTPUT_SCHEMA {
            return Err("case_work_output_contract_mismatch".into());
        }
        events.push(ConversationApplicationEvent::ProviderSelected {
            selection_id: execution.selection.selection_id.clone(),
            target_id: execution.selection.selected_target_id.clone(),
            attempt_number: execution.selection.attempt_number,
        });
        events.push(ConversationApplicationEvent::ProviderResultRecorded {
            invocation_id: execution.invocation_id.clone(),
            result_id: execution.result_id.clone(),
        });
        result.selection_id = Some(execution.selection.selection_id.clone());
        result.invocation_id = Some(execution.invocation_id.clone());
        result.provider_result_id = Some(execution.result_id.clone());
        let current = store.list_case_transitions(&turn.case_id)?;
        if let Some(lineage) = current.iter().find_map(|t| match &t.payload {
            TransitionPayload::ProviderInvocationStarted {
                invocation_id,
                semantic_lineage,
                ..
            } if invocation_id == &execution.invocation_id => semantic_lineage.as_ref(),
            _ => None,
        }) {
            result.projection_id = Some(lineage.projection_id.clone());
            result.context_frame_id = Some(lineage.context_frame_id.clone());
        }
        let mut step = CaseWorkStep {
            ordinal,
            plan_id: execution.plan_id,
            lane_id: lane,
            target_id: execution.selection.selected_target_id,
            provider_result_id: execution.result_id.clone(),
            operation_id: None,
            outcome: None,
            recovered,
        };
        let Some(proposal) = output.request else {
            result.output = output.text;
            result.posture = ConversationExecutionPosture::Completed;
            steps.push(step);
            break;
        };
        let is_effect = match &proposal.action {
            CaseCapabilityAction::FilesystemWrite { .. } => true,
            CaseCapabilityAction::Resource { action } => action.kind().is_external_effect(),
        };
        if ordinal >= limits.operations || (is_effect && effects >= limits.effects) {
            steps.push(step);
            break;
        }
        let operation = store.record_provider_capability_request(
            &authorized.authenticated,
            &turn.case_id,
            &execution.result_id,
        )?;
        effects += u16::from(is_effect);
        step.operation_id = Some(operation.operation_id.clone());
        if controller.cancellation.is_requested() {
            return Err("conversation_cancelled_before_dispatch".into());
        }
        let (outcome, posture) = match operation.kind {
            OperationKind::ResourceAccess(_) => {
                let value = controlled_effect::access::advance(
                    store,
                    &authorized.authenticated,
                    &operation,
                )?;
                let posture = match &value {
                    ResourceActionOutcome::AwaitingReview { .. } => {
                        Some(ConversationExecutionPosture::AwaitingReview)
                    }
                    ResourceActionOutcome::Indeterminate { .. } => {
                        Some(ConversationExecutionPosture::DeliveryIndeterminate)
                    }
                    ResourceActionOutcome::Unsupported { .. } => {
                        Some(ConversationExecutionPosture::Unresolved)
                    }
                    _ => None,
                };
                (
                    serde_json::to_value(value).map_err(|e| e.to_string())?,
                    posture,
                )
            }
            OperationKind::FilesystemWrite => {
                let value =
                    controlled_effect::advance_case_filesystem_operation(store, &operation)?;
                let posture = match value.status {
                    ControlledEffectTurnStatus::AwaitingReview => {
                        Some(ConversationExecutionPosture::AwaitingReview)
                    }
                    ControlledEffectTurnStatus::Indeterminate => {
                        Some(ConversationExecutionPosture::DeliveryIndeterminate)
                    }
                    ControlledEffectTurnStatus::NormalizationRejected => {
                        Some(ConversationExecutionPosture::Unresolved)
                    }
                    _ => None,
                };
                (
                    json!({"status":format!("{:?}",value.status),"operation_id":value.operation_id,"decision_id":value.decision_id,"review_id":value.review_id,"effect_id":value.effect_id,"receipt_id":value.receipt_id}),
                    posture,
                )
            }
            _ => return Err("case_work_operation_kind_not_supported".into()),
        };
        step.outcome = Some(outcome);
        steps.push(step);
        if let Some(posture) = posture {
            result.posture = posture;
            break;
        }
        feedback_ids.push(execution.result_id);
        if failpoint == Some("after-capability-outcome") {
            return Err("case_work_failpoint_after_capability_outcome".into());
        }
    }
    result.work = Some(CaseWorkOutcome {
        request_id: intent.request_id.clone(),
        steps,
    });
    result.events = events.clone();
    Ok(result)
}
