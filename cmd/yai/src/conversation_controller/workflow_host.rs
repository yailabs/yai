//! Workflow is a consumer of the same Case conversation host. Canonical node
//! readiness/progression and amendment authority remain in the engine owner.
use super::*;
use serde::Deserialize;
use serde_json::{json, Value};
use yai_core_engine::context::InvocationOutputContract;
use yai_core_engine::workflow::{
    ModelWorkOutputContract, WorkflowCaseBinding, WorkflowDefinitionInput, WorkflowExecutorBinding,
    WorkflowNodeKind, WorkflowResourceBinding,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowSetup {
    definition: WorkflowDefinitionInput,
    executor_bindings: Vec<WorkflowExecutorBinding>,
    #[serde(default)]
    resource_bindings: Vec<WorkflowResourceBinding>,
    #[serde(default)]
    case_bindings: Vec<WorkflowCaseBinding>,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}

impl ConversationController {
    pub(crate) fn configure_workflow(&self, path: &Path) -> Result<Value, String> {
        let a = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        a.store
            .resolve_security_context(&a.authenticated, &a.tenant_id)?
            .require_owner()?;
        let input: WorkflowSetup =
            serde_json::from_slice(&setup::read_input(path)?).map_err(|e| e.to_string())?;
        if input.definition.tenant_id != a.tenant_id {
            return Err("workflow_setup_tenant_mismatch".into());
        }
        let definition = a
            .store
            .define_workflow(&a.authenticated, input.definition, now_ms())?;
        let commit = a.store.bind_case_workflow_composed(
            &a.authenticated,
            &self.case_id,
            &definition.workflow_definition_id,
            input.executor_bindings,
            input.resource_bindings,
            input.case_bindings,
            now_ms(),
        )?;
        Ok(
            json!({"workflow_binding":commit.state.workflow_binding,"generation":commit.state.generation}),
        )
    }

    /// Commit exactly one immutable task/intent for one existing Workflow node
    /// execution. A crash after start but before SEND resumes that same node;
    /// concurrent SENDs are excluded by the canonical intent admission check.
    pub(crate) fn commit_workflow_node(
        &mut self,
        node_id: &str,
    ) -> Result<ConversationCommitResult, String> {
        let a = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        a.store
            .advance_workflow_passive_progress(&a.authenticated, &self.case_id, 128)?;
        let (execution, node, participant) = a.store.start_workflow_model_work_authorized(
            &a.authenticated,
            &self.case_id,
            node_id,
            now_ms(),
        )?;
        let history = a.store.list_case_transitions(&self.case_id)?;
        if let Some(request) = history.iter().find_map(|t| match &t.payload {
            TransitionPayload::ConversationExecutionIntentRecorded { request }
                if request.workflow_execution_id.as_ref() == Some(&execution.execution_id) =>
            {
                Some(request)
            }
            _ => None,
        }) {
            let transition = history.iter().find(|t| matches!(&t.payload,
                TransitionPayload::ConversationTurnCommitted { turn } if turn.turn_id == request.source_turn_id)).ok_or("workflow_turn_missing")?;
            let TransitionPayload::ConversationTurnCommitted { turn } = &transition.payload else {
                unreachable!()
            };
            if turn.participant_id != self.participant_id {
                return Err("workflow_turn_author_mismatch".into());
            }
            return Ok(ConversationCommitResult {
                turn: turn.clone(),
                transition_id: transition.transition_id.clone(),
                generation: transition.sequence,
                draft_discarded: true,
            });
        }
        let WorkflowNodeKind::ModelWork {
            task,
            budgets,
            output_contract,
            ..
        } = node.kind
        else {
            unreachable!()
        };
        self.commit_parts_with_intent(
            vec![ConversationInputPart::Text { text: task }],
            &ConversationExecutionInput {
                executor_participant_id: Some(participant),
                workflow_execution_id: Some(execution.execution_id),
                work_limits: if output_contract == ModelWorkOutputContract::CaseWork {
                    Some(budgets.case_work_limits()?)
                } else {
                    None
                },
                ..Default::default()
            },
        )
    }

    pub(crate) fn workflow_input(&self, node: &str, text: &str) -> Result<Value, String> {
        let a = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        let commit = a.store.record_workflow_human_input(
            &a.authenticated,
            &self.case_id,
            node,
            text,
            now_ms(),
        )?;
        Ok(json!({"workflow_input":"committed","generation":commit.state.generation}))
    }

    pub(crate) fn workflow_advance(&self) -> Result<Value, String> {
        let a = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        Ok(json!(a.store.advance_workflow_passive_progress(
            &a.authenticated,
            &self.case_id,
            128
        )?))
    }

    pub(crate) fn workflow_patch_validate(&self, patch: &str) -> Result<Value, String> {
        let a = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        Ok(
            json!({"valid":true,"adopted":false,"preview":a.store.validate_workflow_plan_patch_authorized(&a.authenticated, &self.case_id, patch)?}),
        )
    }

    pub(crate) fn workflow_patch_adopt(&self, patch: &str) -> Result<Value, String> {
        let a = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        let commit =
            a.store
                .adopt_workflow_plan_patch(&a.authenticated, &self.case_id, patch, now_ms())?;
        Ok(
            json!({"adopted":true,"generation":commit.state.generation,"amendments":commit.state.workflow_amendments}),
        )
    }
}

/// Single-result Workflow contracts retain their established meaning while
/// dispatch now uses exact cognitive planning/realization and immutable SEND.
pub(super) fn execute_single(
    controller: &ConversationController,
    a: &AuthorizedConversationCase,
    turn: &ConversationTurn,
    intent: &CognitiveCompositionRequest,
    events: &mut Vec<ConversationApplicationEvent>,
    failpoint: Option<&str>,
) -> Result<ConversationExecutionResult, String> {
    use crate::command_adapters::cognitive_execution::{realize_cognitive, source_parts};
    let execution_id = intent
        .workflow_execution_id
        .as_deref()
        .ok_or("workflow_execution_required")?;
    a.store
        .revalidate_conversation_workflow_intent_authorized(&a.authenticated, intent)?;
    let (contract, digest) = a.store.workflow_model_output_contract_authorized(
        &a.authenticated,
        &turn.case_id,
        execution_id,
    )?;
    let history = a.store.list_case_transitions(&turn.case_id)?;
    let recovered = history.iter().find_map(|t| match &t.payload {
        TransitionPayload::ProviderResultRecorded {
            result_id,
            invocation_id,
            output,
            ..
        } if t.causal_refs.iter().any(|r| r == execution_id)
            && history.iter().any(|i| {
                matches!(&i.payload,
                TransitionPayload::ProviderInvocationStarted { invocation_id: id, .. }
                    if id == invocation_id && i.causal_refs.contains(&turn.turn_id))
            }) =>
        {
            Some((result_id.clone(), invocation_id.clone(), output.clone()))
        }
        _ => None,
    });
    let (result_id, invocation_id, output) = if let Some(recorded) = recovered {
        recorded
    } else {
        let output_contract = match contract {
            ModelWorkOutputContract::Text => InvocationOutputContract::NaturalLanguage,
            ModelWorkOutputContract::PlanPatch => InvocationOutputContract::WorkflowPlanPatch {
                schema: yai_core_engine::workflow::WORKFLOW_PLAN_PATCH_SCHEMA.into(),
                base_effective_topology_digest: digest,
                max_operations: yai_core_engine::workflow::MAX_WORKFLOW_PATCH_OPERATIONS,
            },
            ModelWorkOutputContract::CaseWork => {
                return Err("workflow_case_work_requires_finite_host".into())
            }
        };
        let content = conversation_content_store()?;
        let realized = realize_cognitive(
            &a.authenticated,
            &a.store,
            &content,
            turn,
            &intent.participant_id,
            CognitiveCapability::PrimaryConversation,
            source_parts(&content, turn, &intent.source_part_ids)?,
            vec![],
            &intent.request_id,
            Some(yai_core_engine::cognitive::CognitivePlanRoute::Native),
            None,
            failpoint,
            &[turn.turn_id.clone(), intent.request_id.clone()],
            &|| controller.cancellation.is_requested(),
            output_contract,
        )?;
        let e = realized
            .execution
            .ok_or("workflow_provider_result_missing")?;
        (e.result_id, e.invocation_id, e.output)
    };
    if contract == ModelWorkOutputContract::PlanPatch {
        a.store.propose_workflow_plan_patch_from_provider_result(
            &a.authenticated,
            &turn.case_id,
            &result_id,
            now_ms(),
        )?;
    }
    let mut result = unavailable_execution(
        turn,
        ConversationExecutionPosture::Completed,
        String::new(),
        &mut vec![],
    );
    result.output = Some(output);
    result.provider_result_id = Some(result_id.clone());
    result.invocation_id = Some(invocation_id.clone());
    result.intent = Some(intent.clone());
    events.push(ConversationApplicationEvent::ProviderResultRecorded {
        invocation_id,
        result_id,
    });
    result.events = events.clone();
    Ok(result)
}
