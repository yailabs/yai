//! Host-independent Case conversation application semantics.
//!
//! This controller is deliberately not a terminal REPL. It accepts typed
//! actions and ordered semantic content, commits the user turn first, and then
//! reuses YAI's Projection/Context/provider-governance invocation boundary.
//! The REPLAI terminal frontend and a graphical client can
//! adapt to this boundary without owning Case or conversation truth.

// The terminal adapter consumes this application boundary through typed actions.
// Some non-terminal multipart actions are also exercised directly in qualification.
#![allow(dead_code)]

use super::cognitive_execution::{execute_composition, CognitiveCompositionOutcome};
use super::*;
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use yai_core_engine::cognitive::CognitiveCapability;
use yai_core_engine::conversation::{
    find_turn, turns_from_history, CognitiveCompositionPrerequisite, CognitiveCompositionRequest,
    ContentModality, ContentPartProvenance, ConversationContentStore, ConversationDraft,
    ConversationTurn, CONVERSATION_DRAFT_SCHEMA,
};
use yai_core_engine::security::AuthenticatedPrincipal;
use yai_core_engine::transition::{CaseState, TransitionScope};

#[path = "conversation_controller/setup.rs"]
mod setup;
#[path = "conversation_controller/work.rs"]
mod work;
#[path = "conversation_controller/workflow_host.rs"]
mod workflow_host;
pub(super) use setup::{admit_workbench_participants, scoped_name, ProviderConnection};

#[derive(Clone, Copy, Debug)]
pub(super) enum CaseInspection {
    Status,
    History,
    Participants,
    Resources,
    Artifacts,
    Policy,
    Reviews,
    Provider,
    Capabilities,
    Workflow,
    Effects,
    Memory,
    Graph,
    Handoffs,
    Verify,
}

pub(super) enum CaseHandoffAction {
    Offer {
        target: String,
        required_role: String,
        text: String,
    },
    Accept {
        source: String,
        handoff_id: String,
    },
    Result {
        handoff_id: String,
        outcome: yai_core_engine::handoff::HandoffOutcome,
        evidence_ref: String,
        text: String,
    },
    Reconcile {
        handoff_id: String,
    },
}

static CONTROLLER_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ConversationInputPart {
    Text {
        text: String,
    },
    Bytes {
        modality: ContentModality,
        media_type: String,
        bytes: Vec<u8>,
    },
}

/// Pre-SEND semantic choice. Ordinals address ordered application parts, never
/// filenames or terminal syntax. No prerequisite is inferred when omitted.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct ConversationExecutionInput {
    pub prerequisite: Option<(CognitiveCapability, Vec<u16>)>,
    pub executor_participant_id: Option<String>,
    pub work_limits: Option<yai_core_engine::conversation::CaseWorkLimits>,
    pub workflow_execution_id: Option<String>,
}

impl ConversationExecutionInput {
    fn bind(&self, turn: &ConversationTurn) -> Result<CognitiveCompositionRequest, String> {
        let prerequisite = self
            .prerequisite
            .as_ref()
            .map(|(capability, ordinals)| {
                let source_part_ids = ordinals
                    .iter()
                    .map(|ordinal| {
                        turn.ordered_parts
                            .get(usize::from(*ordinal))
                            .map(|part| part.part_id.clone())
                            .ok_or_else(|| "conversation_intent_ordinal_not_found".to_string())
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok::<_, String>(CognitiveCompositionPrerequisite {
                    capability: capability.clone(),
                    source_part_ids,
                })
            })
            .transpose()?;
        let request = CognitiveCompositionRequest::for_executor(
            turn,
            self.executor_participant_id
                .as_deref()
                .unwrap_or(&turn.participant_id),
            CognitiveCapability::PrimaryConversation,
            turn.ordered_parts
                .iter()
                .map(|part| part.part_id.clone())
                .collect(),
            prerequisite,
        )?;
        let request = match &self.work_limits {
            Some(limits) => request.with_work_limits(limits.clone()),
            None => Ok(request),
        }?;
        match &self.workflow_execution_id {
            Some(id) => request.with_workflow_execution(id),
            None => Ok(request),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ConversationAction {
    Submit { parts: Vec<ConversationInputPart> },
    Retry { turn_id: String },
    NewThread,
    ListThreads,
    UseThread { thread_id: String },
    Inspect,
    Cancel,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ConversationExecutionPosture {
    Completed,
    ProviderUnavailable,
    ProviderFailed,
    DeliveryIndeterminate,
    Unresolved,
    CancelledBeforeDispatch,
    AwaitingReview,
    BudgetExhausted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub(super) enum ConversationApplicationEvent {
    TurnCommitted {
        turn_id: String,
        thread_id: String,
        generation: u64,
    },
    ProviderSelected {
        selection_id: String,
        target_id: String,
        attempt_number: u32,
    },
    ProviderResultRecorded {
        invocation_id: String,
        result_id: String,
    },
    ExecutionUnavailable {
        posture: ConversationExecutionPosture,
        detail: String,
    },
    ActiveThreadChanged {
        thread_id: String,
        durable: bool,
    },
    CancellationRequested,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct ConversationThreadSummary {
    pub thread_id: String,
    pub committed_turn_count: usize,
    pub first_generation: u64,
    pub last_generation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct ConversationControllerStatus {
    pub case_id: String,
    pub participant_id: String,
    pub active_thread_id: String,
    pub active_thread_durable: bool,
    pub case_generation: u64,
    pub committed_threads: Vec<ConversationThreadSummary>,
    pub terminal_frontend: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct ConversationExecutionResult {
    pub turn_id: String,
    pub posture: ConversationExecutionPosture,
    pub output: Option<String>,
    pub selection_id: Option<String>,
    pub invocation_id: Option<String>,
    pub provider_result_id: Option<String>,
    pub projection_id: Option<String>,
    pub context_frame_id: Option<String>,
    pub intent: Option<CognitiveCompositionRequest>,
    pub cognition: Option<CognitiveCompositionOutcome>,
    pub work: Option<work::CaseWorkOutcome>,
    pub events: Vec<ConversationApplicationEvent>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct ConversationSubmissionResult {
    pub turn: ConversationTurn,
    pub transition_id: String,
    pub generation: u64,
    pub draft_discarded: bool,
    pub execution: ConversationExecutionResult,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub(super) enum ConversationActionResult {
    Submission {
        value: Box<ConversationSubmissionResult>,
    },
    Execution {
        value: Box<ConversationExecutionResult>,
    },
    Threads {
        values: Vec<ConversationThreadSummary>,
    },
    Status {
        value: ConversationControllerStatus,
    },
    ThreadSelected {
        thread_id: String,
        durable: bool,
    },
    CancellationRequested,
}

#[derive(Clone, Default)]
pub(super) struct ConversationCancellation {
    requested: Arc<AtomicBool>,
}

impl ConversationCancellation {
    pub(super) fn request(&self) {
        self.requested.store(true, Ordering::SeqCst);
    }

    fn reset(&self) {
        self.requested.store(false, Ordering::SeqCst);
    }

    fn is_requested(&self) -> bool {
        self.requested.load(Ordering::SeqCst)
    }
}

pub(super) struct AuthorizedConversationCase {
    pub(super) store: LmdbRecordStore,
    pub(super) state: CaseState,
    pub(super) authenticated: AuthenticatedPrincipal,
    pub(super) principal_id: String,
    pub(super) tenant_id: String,
}

#[derive(Clone, Debug)]
pub(super) struct ConversationCommitResult {
    pub(super) turn: ConversationTurn,
    pub(super) transition_id: String,
    pub(super) generation: u64,
    pub(super) draft_discarded: bool,
}

pub(super) struct ConversationController {
    case_id: String,
    participant_id: String,
    executor_participant_id: Option<String>,
    active_thread_id: String,
    cancellation: ConversationCancellation,
}

impl ConversationController {
    pub(super) fn inspect_operation(
        &self,
        operation_id: &str,
    ) -> Result<serde_json::Value, String> {
        let a = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        a.store
            .resolve_security_context(&a.authenticated, &a.tenant_id)?
            .require_owner()?;
        let history = a.store.list_case_transitions(&self.case_id)?;
        let operation = history
            .iter()
            .find_map(|t| match &t.payload {
                TransitionPayload::OperationRecorded { operation }
                    if operation.operation_id == operation_id =>
                {
                    Some(operation)
                }
                _ => None,
            })
            .ok_or("operation_not_in_case")?;
        Ok(
            serde_json::json!({"operation":operation,"causal_transitions":history.iter()
            .filter(|t| t.causal_refs.iter().any(|r|r == operation_id))
            .map(|t| serde_json::json!({"id":t.transition_id,"kind":t.payload.kind(),"generation":t.sequence})).collect::<Vec<_>>()}),
        )
    }

    /// Scoped maintenance of disposable access structures, not a semantic write.
    /// The caller never needs to edit LMDB or delete arbitrary filesystem paths.
    pub(super) fn rebuild_case_views(&self) -> Result<serde_json::Value, String> {
        let a = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        a.store
            .resolve_security_context(&a.authenticated, &a.tenant_id)?
            .require_owner()?;
        let history = a.store.list_case_transitions(&self.case_id)?;
        if a.store.replay_case_state(&self.case_id)? != a.state {
            return Err("case_materialization_does_not_match_replay".into());
        }
        let memory = yai_core_engine::memory::derive_operational_memory(&self.case_id, &history)?;
        let hierarchy =
            yai_core_engine::memory_hierarchy::build_memory_hierarchy(&a.state, &history, &memory)?;
        a.store.clear_case_operational_memory(&self.case_id)?;
        a.store.replace_case_operational_memory(&memory)?;
        let graph = a.store.rebuild_graph_relations_for_case(&self.case_id)?;
        let rebuilt = a.store.rebuild_case_state(&self.case_id)?;
        if rebuilt != a.state || a.store.list_case_transitions(&self.case_id)? != history {
            return Err("case_changed_during_derived_rebuild".into());
        }
        Ok(
            serde_json::json!({"case":self.case_id,"generation":a.state.generation,
            "canonical_history_unchanged":true,"replay_equal":true,
            "operational_entries":memory.entries.len(),"hierarchy_digest":hierarchy.manifest.hierarchy_digest,
            "episodes":hierarchy.episodes.len(),"semantic_assertions":hierarchy.assertions.len(),
            "hierarchy_cache":"absent_by_design","graph":{"relations_seen":graph.relations_seen,
                "relations_written":graph.relations_written,"relations_skipped":graph.relations_skipped},
            "embedding_index":"not_rebuilt_requires_explicit_encoder_profile"}),
        )
    }

    pub(super) fn handoff(&self, action: CaseHandoffAction) -> Result<serde_json::Value, String> {
        use yai_core_engine::handoff::{HandoffData, HandoffDataKind};
        let a = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .min(u128::from(u64::MAX)) as u64;
        let text_data = |value| HandoffData {
            kind: HandoffDataKind::Text,
            value,
        };
        let commit = match action {
            CaseHandoffAction::Offer {
                target,
                required_role,
                text,
            } => a.store.offer_case_handoff(
                &a.authenticated,
                &self.case_id,
                &target,
                text_data(text),
                vec![required_role],
                now,
            )?,
            CaseHandoffAction::Accept { source, handoff_id } => a.store.accept_case_handoff(
                &a.authenticated,
                &self.case_id,
                &source,
                &handoff_id,
                &self.participant_id,
                now,
            )?,
            CaseHandoffAction::Result {
                handoff_id,
                outcome,
                evidence_ref,
                text,
            } => a.store.record_case_handoff_result(
                &a.authenticated,
                &self.case_id,
                &handoff_id,
                outcome,
                text_data(text),
                vec![evidence_ref],
                &self.participant_id,
                now,
            )?,
            CaseHandoffAction::Reconcile { handoff_id } => {
                a.store
                    .reconcile_case_handoff(&a.authenticated, &self.case_id, &handoff_id, now)?
            }
        };
        Ok(
            serde_json::json!({"case":self.case_id,"generation":commit.state.generation,
            "offers":commit.state.handoff_offers,"acceptances":commit.state.handoff_acceptances,
            "results":commit.state.handoff_results,"reconciliations":commit.state.handoff_reconciliations,
            "authority_transferred":false}),
        )
    }

    /// Operator workbench view. This is NOT a model disclosure bypass: the
    /// authenticated Tenant owner is required before inspecting full history.
    pub(super) fn inspect_case(
        &self,
        section: CaseInspection,
    ) -> Result<serde_json::Value, String> {
        use serde_json::json;
        let authorized = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        authorized
            .store
            .resolve_security_context(&authorized.authenticated, &authorized.tenant_id)?
            .require_owner()?;
        let state = &authorized.state;
        let store = &authorized.store;
        let value = match section {
            CaseInspection::Status => {
                json!({"case":state.case_id,"generation":state.generation,"tenant":state.tenant_id,"lifecycle":state.lifecycle,
                "operator":self.participant_id,"executor":self.executor_participant_id,"participants":state.participants.len(),"resources":state.resources.len(),
                "pending_reviews":state.reviews.iter().filter(|r|matches!(r.status,yai_core_engine::transition::ReviewResolution::Pending | yai_core_engine::transition::ReviewResolution::Deferred)).count()})
            }
            CaseInspection::History => {
                let history = store.list_case_transitions(&self.case_id)?;
                json!({"authority":"transition_ledger","total":history.len(),"tail":history.iter().rev().take(48).collect::<Vec<_>>().into_iter().rev().map(|t|json!({"sequence":t.sequence,"id":t.transition_id,"kind":t.payload.kind(),"causal_refs":t.causal_refs})).collect::<Vec<_>>()})
            }
            CaseInspection::Participants => json!(state.participants),
            CaseInspection::Resources => json!(state.resources),
            CaseInspection::Artifacts => json!(state.admitted_content),
            CaseInspection::Policy => json!(store.case_policy_status(&self.case_id)?),
            CaseInspection::Reviews => json!(state.reviews),
            CaseInspection::Provider => {
                json!({"routing_envelope":state.provider_binding,"cognitive_bindings":state.cognitive_bindings})
            }
            CaseInspection::Capabilities => json!(store.case_capability_view_authorized(
                &authorized.authenticated,
                &self.case_id,
                self.executor_participant_id
                    .as_deref()
                    .unwrap_or(&self.participant_id)
            )?),
            CaseInspection::Workflow => {
                json!({"status":store.workflow_status_authorized(&authorized.authenticated, &self.case_id)?,
                    "patches":state.workflow_plan_patches,"amendments":state.workflow_amendments})
            }
            CaseInspection::Effects => json!(state.effects),
            CaseInspection::Memory => {
                let history = store.list_case_transitions(&self.case_id)?;
                let memory =
                    yai_core_engine::memory::derive_operational_memory(&self.case_id, &history)?;
                let hierarchy = yai_core_engine::memory_hierarchy::build_memory_hierarchy(
                    state, &history, &memory,
                )?;
                json!({"authority":"derived","operational_count":memory.entries.len(),
                    "operational":memory.entries.iter().rev().take(24).collect::<Vec<_>>(),
                    "hierarchy":hierarchy.manifest,"episodes":hierarchy.episodes.iter().rev().take(12).collect::<Vec<_>>(),
                    "semantic_assertions":hierarchy.assertions.iter().rev().take(12).collect::<Vec<_>>()})
            }
            CaseInspection::Graph => {
                let relations = store.list_graph_relations_by_case(&self.case_id, 48)?;
                json!({"authority":"derived","total":relations.relations_total,"relations":relations.relations.iter().map(|r|json!({"id":r.relation_id,"from":r.from_ref,"to":r.to_ref,"kind":r.edge_kind,"source":r.source_record_id,"confidence":r.confidence})).collect::<Vec<_>>()})
            }
            CaseInspection::Handoffs => {
                json!({"offers":state.handoff_offers,"acceptances":state.handoff_acceptances,
                "results":state.handoff_results,"reconciliations":state.handoff_reconciliations,
                "pending_incoming":store.list_pending_case_handoffs_authorized(&authorized.authenticated,&self.case_id)?})
            }
            CaseInspection::Verify => {
                json!({"case":self.case_id,"generation":state.generation,"replay_equal":store.replay_case_state(&self.case_id)? == *state})
            }
        };
        Ok(value)
    }

    pub(super) fn review_action(
        &self,
        review_id: &str,
        reviewer: &str,
        action: yai_core_engine::transition::ReviewActionKind,
        reason: &str,
    ) -> Result<(), String> {
        let authorized = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        super::review::resolve_review_action(
            &authorized.store,
            &authorized.authenticated,
            &self.case_id,
            review_id,
            Some(reviewer),
            action,
            reason,
            None,
        )
    }

    pub(super) fn commit_work(
        &mut self,
        text: String,
        limits: yai_core_engine::conversation::CaseWorkLimits,
    ) -> Result<ConversationCommitResult, String> {
        self.commit_parts_with_intent(
            vec![ConversationInputPart::Text { text }],
            &ConversationExecutionInput {
                executor_participant_id: self.executor_participant_id.clone(),
                work_limits: Some(limits),
                ..Default::default()
            },
        )
    }

    pub(super) fn request_resource(
        &self,
        resource: &str,
        action: yai_core_engine::effect::access::ResourceAction,
    ) -> Result<serde_json::Value, String> {
        let authorized = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        let access = authorized
            .state
            .resources
            .iter()
            .find(|r| r.attachment_id == resource)
            .and_then(|r| r.access.as_ref())
            .ok_or("resource_not_attached")?;
        let request = yai_core_engine::effect::access::ResourceRequest {
            schema: yai_core_engine::effect::access::RESOURCE_REQUEST_SCHEMA.into(),
            configuration_digest: access.configuration_digest.clone(),
            action,
        };
        let operation = authorized.store.record_participant_resource_request(
            &authorized.authenticated,
            &self.case_id,
            &self.participant_id,
            resource,
            &fresh_controller_id(
                "resource-request",
                &self.case_id,
                &self.participant_id,
                authorized.state.generation,
            ),
            authorized.state.generation,
            request,
        )?;
        serde_json::to_value(controlled_effect::access::advance(
            &authorized.store,
            &authorized.authenticated,
            &operation,
        )?)
        .map_err(|e| e.to_string())
    }
    pub(super) fn open(case_id: &str, participant_id: Option<&str>) -> Result<Self, String> {
        let authenticated = security::authenticate_local()?;
        let principal_id = authenticated.projected_principal_id();
        let store = LmdbRecordStore::open(record_store_path())?;
        let state = store.get_case_state_authorized(&authenticated, case_id)?;
        let participant_id =
            resolve_conversation_participant(&state, &principal_id, participant_id)?;
        let transitions = store.list_case_transitions(case_id)?;
        let latest_thread = turns_from_history(case_id, &transitions)
            .into_iter()
            .rev()
            .find(|turn| turn.participant_id == participant_id)
            .map(|turn| turn.thread_id.clone());
        let active_thread_id = latest_thread.unwrap_or_else(|| {
            fresh_controller_thread_id(case_id, &participant_id, state.generation)
        });
        Ok(Self {
            case_id: case_id.to_string(),
            participant_id,
            executor_participant_id: None,
            active_thread_id,
            cancellation: ConversationCancellation::default(),
        })
    }

    pub(super) fn cancellation(&self) -> ConversationCancellation {
        self.cancellation.clone()
    }

    /// Frontend-local selection becomes exact durable delegation only at SEND.
    /// It grants neither the Principal identity nor human review authority.
    pub(super) fn select_executor(&mut self, executor: &str) -> Result<(), String> {
        let authorized = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        if !authorized.state.participants.iter().any(|participant| {
            participant.participant_id == executor
                && participant
                    .admitted_views
                    .iter()
                    .any(|view| view.consumer == "model" && view.view_kind == "model_context")
        }) {
            return Err("conversation_executor_model_view_not_admitted".into());
        }
        self.executor_participant_id = Some(executor.into());
        Ok(())
    }

    pub(super) fn apply(
        &mut self,
        action: ConversationAction,
    ) -> Result<ConversationActionResult, String> {
        match action {
            ConversationAction::Submit { parts } => {
                self.submit(parts)
                    .map(|value| ConversationActionResult::Submission {
                        value: Box::new(value),
                    })
            }
            ConversationAction::Retry { turn_id } => {
                self.retry(&turn_id)
                    .map(|value| ConversationActionResult::Execution {
                        value: Box::new(value),
                    })
            }
            ConversationAction::NewThread => {
                let state =
                    authorized_conversation_case(&self.case_id, &self.participant_id)?.state;
                self.active_thread_id = fresh_controller_thread_id(
                    &self.case_id,
                    &self.participant_id,
                    state.generation,
                );
                Ok(ConversationActionResult::ThreadSelected {
                    thread_id: self.active_thread_id.clone(),
                    durable: false,
                })
            }
            ConversationAction::ListThreads => self
                .threads()
                .map(|values| ConversationActionResult::Threads { values }),
            ConversationAction::UseThread { thread_id } => {
                if !self
                    .threads()?
                    .iter()
                    .any(|thread| thread.thread_id == thread_id)
                {
                    return Err("conversation_thread_not_in_committed_history".to_string());
                }
                self.active_thread_id = thread_id.clone();
                Ok(ConversationActionResult::ThreadSelected {
                    thread_id,
                    durable: true,
                })
            }
            ConversationAction::Inspect => self
                .status()
                .map(|value| ConversationActionResult::Status { value }),
            ConversationAction::Cancel => {
                self.cancellation.request();
                Ok(ConversationActionResult::CancellationRequested)
            }
        }
    }

    fn submit(
        &mut self,
        parts: Vec<ConversationInputPart>,
    ) -> Result<ConversationSubmissionResult, String> {
        let committed = self.commit_parts(parts)?;
        let mut events = vec![ConversationApplicationEvent::TurnCommitted {
            turn_id: committed.turn.turn_id.clone(),
            thread_id: committed.turn.thread_id.clone(),
            generation: committed.generation,
        }];
        let execution = self.execute_turn(&committed.turn, &mut events)?;
        Ok(ConversationSubmissionResult {
            turn: committed.turn,
            transition_id: committed.transition_id,
            generation: committed.generation,
            draft_discarded: committed.draft_discarded,
            execution,
        })
    }

    /// Commit the host-normalized ordered parts without dispatching a model.
    /// Frontends may use this boundary to acknowledge SEND immediately and
    /// schedule [`Self::execute_committed_turn`] independently.
    pub(super) fn commit_parts(
        &mut self,
        parts: Vec<ConversationInputPart>,
    ) -> Result<ConversationCommitResult, String> {
        let intent = ConversationExecutionInput {
            executor_participant_id: self.executor_participant_id.clone(),
            ..ConversationExecutionInput::default()
        };
        self.commit_parts_with_intent(parts, &intent)
    }

    pub(super) fn commit_parts_with_intent(
        &mut self,
        parts: Vec<ConversationInputPart>,
        intent: &ConversationExecutionInput,
    ) -> Result<ConversationCommitResult, String> {
        if parts.is_empty() {
            return Err("conversation_turn_requires_content".to_string());
        }
        self.cancellation.reset();
        let authorized = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        authorized
            .store
            .resolve_security_context(&authorized.authenticated, &authorized.tenant_id)?
            .require_owner()?;
        let draft_id = fresh_controller_draft_id(
            &self.case_id,
            &self.participant_id,
            authorized.state.generation,
        );
        let mut draft = ConversationDraft {
            schema: CONVERSATION_DRAFT_SCHEMA.to_string(),
            draft_id,
            case_id: self.case_id.clone(),
            tenant_id: authorized.tenant_id,
            thread_id: self.active_thread_id.clone(),
            participant_id: self.participant_id.clone(),
            principal_id: authorized.principal_id,
            base_generation: authorized.state.generation,
            parts: Vec::new(),
        };
        let content_store = conversation_content_store()?;
        content_store.create_draft(&draft)?;
        let principal_id = draft.principal_id.clone();
        let stage_result = parts.into_iter().try_for_each(|part| match part {
            ConversationInputPart::Text { text } => content_store
                .stage_bytes(
                    &mut draft,
                    ContentModality::Text,
                    "text/plain;charset=utf-8",
                    text.as_bytes(),
                    ContentPartProvenance::Original {
                        imported_by_principal_id: principal_id.clone(),
                    },
                )
                .map(|_| ()),
            ConversationInputPart::Bytes {
                modality,
                media_type,
                bytes,
            } => content_store
                .stage_bytes(
                    &mut draft,
                    modality,
                    &media_type,
                    &bytes,
                    ContentPartProvenance::Original {
                        imported_by_principal_id: principal_id.clone(),
                    },
                )
                .map(|_| ()),
        });
        if let Err(error) = stage_result {
            let _ = content_store.discard_draft(&draft.case_id, &draft.draft_id);
            return Err(error);
        }
        commit_conversation_draft_with_intent(&draft, Some(intent))
    }

    fn retry(&mut self, turn_id: &str) -> Result<ConversationExecutionResult, String> {
        // Retry is a new application request over the same canonical Turn, so
        // an earlier cancellation request does not permanently poison it.
        self.cancellation.reset();
        self.execute_committed_turn(turn_id)
    }

    /// Execute one already committed Turn through the shared semantic/provider
    /// seam. This path has no ResourceAttachment, Workflow, Effect, or
    /// operational-runtime admission requirement.
    pub(super) fn execute_committed_turn(
        &mut self,
        turn_id: &str,
    ) -> Result<ConversationExecutionResult, String> {
        let authorized = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        let transitions = authorized.store.list_case_transitions(&self.case_id)?;
        let turn = find_turn(&self.case_id, turn_id, &transitions)
            .ok_or_else(|| "conversation_turn_not_found".to_string())?;
        if turn.participant_id != self.participant_id {
            return Err("conversation_turn_not_visible".to_string());
        }
        verify_conversation_turn(turn)?;
        let mut events = Vec::new();
        let result = self.execute_turn(turn, &mut events)?;
        if result.posture == ConversationExecutionPosture::Completed
            && result
                .intent
                .as_ref()
                .is_some_and(|intent| intent.workflow_execution_id.is_some())
        {
            authorized.store.advance_workflow_passive_progress(
                &authorized.authenticated,
                &self.case_id,
                128,
            )?;
        }
        Ok(result)
    }

    fn execute_turn(
        &self,
        turn: &ConversationTurn,
        events: &mut Vec<ConversationApplicationEvent>,
    ) -> Result<ConversationExecutionResult, String> {
        self.execute_turn_with_failpoint(turn, events, None)
    }

    fn execute_turn_with_failpoint(
        &self,
        turn: &ConversationTurn,
        events: &mut Vec<ConversationApplicationEvent>,
        failpoint: Option<&str>,
    ) -> Result<ConversationExecutionResult, String> {
        verify_conversation_turn(turn)?;
        let authorized = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        let history = authorized.store.list_case_transitions(&self.case_id)?;
        let intent = history
            .iter()
            .find_map(|transition| match &transition.payload {
                TransitionPayload::ConversationExecutionIntentRecorded { request }
                    if request.source_turn_id == turn.turn_id =>
                {
                    Some(request.clone())
                }
                _ => None,
            })
            .unwrap_or(ConversationExecutionInput::default().bind(turn)?);
        intent.validate(turn)?;
        let intent = authorized
            .store
            .record_conversation_execution_intent_authorized(&authorized.authenticated, intent)?;
        // A changed source closure/target must not hide any possibly delivered
        // work on this Turn. The exact I03 guard remains active below this gate.
        let unsafe_prior = history.iter().any(|transition| {
            let TransitionPayload::ProviderInvocationStarted { invocation_id, governance, .. } = &transition.payload else {
                return false;
            };
            if !transition.causal_refs.contains(&turn.turn_id) { return false; }
            let completed = history.iter().any(|item| matches!(&item.payload,
                TransitionPayload::ProviderResultRecorded { invocation_id: owner, .. } if owner == invocation_id));
            if completed { return false; }
            !history.iter().any(|item| matches!(&item.payload,
                TransitionPayload::ProviderAttemptOutcomeRecorded { outcome }
                if governance.as_ref().is_some_and(|g| g.selection_id == outcome.selection_id) && outcome.retry_safe()))
        });
        if !unsafe_prior && intent.work_limits.is_some() {
            return match work::execute(self, &authorized, turn, &intent, events, failpoint) {
                Ok(result) => Ok(result),
                Err(error) => {
                    let posture = if error.contains("indeterminate") {
                        ConversationExecutionPosture::DeliveryIndeterminate
                    } else if error.contains("cancelled_before_dispatch") {
                        ConversationExecutionPosture::CancelledBeforeDispatch
                    } else {
                        ConversationExecutionPosture::Unresolved
                    };
                    let mut result = unavailable_execution(turn, posture, error, events);
                    result.intent = Some(intent);
                    Ok(result)
                }
            };
        }
        let result = if unsafe_prior {
            Err("conversation_prior_delivery_indeterminate_requires_resolution".to_string())
        } else if intent.workflow_execution_id.is_some() {
            return match workflow_host::execute_single(
                self,
                &authorized,
                turn,
                &intent,
                events,
                failpoint,
            ) {
                Ok(result) => Ok(result),
                Err(error) => {
                    let posture = if error.contains("indeterminate") {
                        ConversationExecutionPosture::DeliveryIndeterminate
                    } else if error.contains("cancelled_before_dispatch") {
                        ConversationExecutionPosture::CancelledBeforeDispatch
                    } else {
                        ConversationExecutionPosture::Unresolved
                    };
                    let mut result = unavailable_execution(turn, posture, error, events);
                    result.intent = Some(intent);
                    Ok(result)
                }
            };
        } else {
            execute_composition(
                &authorized.authenticated,
                &authorized.store,
                &conversation_content_store()?,
                turn,
                &intent,
                &|| self.cancellation.is_requested(),
                failpoint,
            )
        };
        match result {
            Ok(cognition) => {
                let execution = cognition
                    .primary
                    .execution
                    .as_ref()
                    .ok_or_else(|| "conversation_primary_result_missing".to_string())?;
                let history = authorized.store.list_case_transitions(&self.case_id)?;
                let lineage = history
                    .iter()
                    .find_map(|transition| match &transition.payload {
                        TransitionPayload::ProviderInvocationStarted {
                            invocation_id,
                            semantic_lineage,
                            ..
                        } if invocation_id == &execution.invocation_id => semantic_lineage.as_ref(),
                        _ => None,
                    });
                events.push(ConversationApplicationEvent::ProviderSelected {
                    selection_id: execution.selection.selection_id.clone(),
                    target_id: execution.selection.selected_target_id.clone(),
                    attempt_number: execution.selection.attempt_number,
                });
                events.push(ConversationApplicationEvent::ProviderResultRecorded {
                    invocation_id: execution.invocation_id.clone(),
                    result_id: execution.result_id.clone(),
                });
                Ok(ConversationExecutionResult {
                    turn_id: turn.turn_id.clone(),
                    posture: ConversationExecutionPosture::Completed,
                    output: Some(execution.output.clone()),
                    selection_id: Some(execution.selection.selection_id.clone()),
                    invocation_id: Some(execution.invocation_id.clone()),
                    provider_result_id: Some(execution.result_id.clone()),
                    projection_id: lineage.map(|value| value.projection_id.clone()),
                    context_frame_id: lineage.map(|value| value.context_frame_id.clone()),
                    intent: Some(intent),
                    cognition: Some(cognition),
                    work: None,
                    events: events.clone(),
                })
            }
            Err(error) => {
                let posture = if error.contains("cancelled_before_dispatch") {
                    ConversationExecutionPosture::CancelledBeforeDispatch
                } else if error.contains("delivery_indeterminate")
                    || error.contains("DeliveryIndeterminate")
                {
                    ConversationExecutionPosture::DeliveryIndeterminate
                } else if error.contains("cognitive_realization_failed")
                    || error.contains("derived_text_invalid")
                {
                    ConversationExecutionPosture::ProviderFailed
                } else if authorized.state.cognitive_bindings.is_empty() {
                    ConversationExecutionPosture::ProviderUnavailable
                } else {
                    ConversationExecutionPosture::Unresolved
                };
                let mut result = unavailable_execution(turn, posture, error, events);
                result.intent = Some(intent);
                Ok(result)
            }
        }
    }

    fn threads(&self) -> Result<Vec<ConversationThreadSummary>, String> {
        let authorized = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        let transitions = authorized.store.list_case_transitions(&self.case_id)?;
        Ok(derive_thread_summaries(
            &self.case_id,
            &self.participant_id,
            &transitions,
        ))
    }

    pub(super) fn latest_turn_id(&self) -> Result<String, String> {
        let a = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        let history = a.store.list_case_transitions(&self.case_id)?;
        turns_from_history(&self.case_id, &history)
            .into_iter()
            .rev()
            .find(|t| {
                t.participant_id == self.participant_id && t.thread_id == self.active_thread_id
            })
            .map(|t| t.turn_id.clone())
            .ok_or("no_committed_turn_in_current_thread".into())
    }

    fn status(&self) -> Result<ConversationControllerStatus, String> {
        let authorized = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        let committed_threads = self.threads()?;
        let active_thread_durable = committed_threads
            .iter()
            .any(|thread| thread.thread_id == self.active_thread_id);
        Ok(ConversationControllerStatus {
            case_id: self.case_id.clone(),
            participant_id: self.participant_id.clone(),
            active_thread_id: self.active_thread_id.clone(),
            active_thread_durable,
            case_generation: authorized.state.generation,
            committed_threads,
            terminal_frontend: "replai_native",
        })
    }
}

fn unavailable_execution(
    turn: &ConversationTurn,
    posture: ConversationExecutionPosture,
    detail: String,
    events: &mut Vec<ConversationApplicationEvent>,
) -> ConversationExecutionResult {
    events.push(ConversationApplicationEvent::ExecutionUnavailable {
        posture: posture.clone(),
        detail,
    });
    ConversationExecutionResult {
        turn_id: turn.turn_id.clone(),
        posture,
        output: None,
        selection_id: None,
        invocation_id: None,
        provider_result_id: None,
        projection_id: None,
        context_frame_id: None,
        intent: None,
        cognition: None,
        work: None,
        events: events.clone(),
    }
}

fn derive_thread_summaries(
    case_id: &str,
    participant_id: &str,
    transitions: &[yai_core_engine::transition::Transition],
) -> Vec<ConversationThreadSummary> {
    let mut threads = BTreeMap::<String, ConversationThreadSummary>::new();
    for turn in turns_from_history(case_id, transitions)
        .into_iter()
        .filter(|turn| turn.participant_id == participant_id)
    {
        let generation = turn.base_generation.saturating_add(1);
        let entry =
            threads
                .entry(turn.thread_id.clone())
                .or_insert_with(|| ConversationThreadSummary {
                    thread_id: turn.thread_id.clone(),
                    committed_turn_count: 0,
                    first_generation: generation,
                    last_generation: generation,
                });
        entry.committed_turn_count = entry.committed_turn_count.saturating_add(1);
        entry.first_generation = entry.first_generation.min(generation);
        entry.last_generation = entry.last_generation.max(generation);
    }
    let mut result = threads.into_values().collect::<Vec<_>>();
    result.sort_by(|left, right| {
        left.first_generation
            .cmp(&right.first_generation)
            .then_with(|| left.thread_id.cmp(&right.thread_id))
    });
    result
}

fn resolve_conversation_participant(
    state: &CaseState,
    principal_id: &str,
    requested: Option<&str>,
) -> Result<String, String> {
    let tenant_id = state
        .tenant_id
        .as_deref()
        .ok_or_else(|| "legacy_unscoped_case_cannot_own_conversation_content".to_string())?;
    let mut eligible = state
        .participants
        .iter()
        .filter(|participant| {
            state.principal_participant_links.iter().any(|link| {
                link.tenant_id == tenant_id
                    && link.principal_id == principal_id
                    && link.participant_id == participant.participant_id
            })
        })
        .map(|participant| participant.participant_id.clone())
        .collect::<Vec<_>>();
    eligible.sort();
    eligible.dedup();
    if let Some(requested) = requested {
        return eligible
            .iter()
            .any(|candidate| candidate == requested)
            .then(|| requested.to_string())
            .ok_or_else(|| "conversation_participant_not_admitted_for_model_context".to_string());
    }
    match eligible.as_slice() {
        [participant] => Ok(participant.clone()),
        [] => Err("conversation_participant_not_admitted_for_model_context".to_string()),
        _ => Err(format!(
            "conversation_participant_selection_required:{}",
            eligible.join(",")
        )),
    }
}

fn fresh_controller_thread_id(case_id: &str, participant_id: &str, generation: u64) -> String {
    fresh_controller_id("conversation-thread", case_id, participant_id, generation)
}

fn fresh_controller_draft_id(case_id: &str, participant_id: &str, generation: u64) -> String {
    let digest = fresh_controller_digest(case_id, participant_id, generation);
    format!("chat-draft-{}", &digest[..32])
}

fn fresh_controller_id(
    prefix: &str,
    case_id: &str,
    participant_id: &str,
    generation: u64,
) -> String {
    format!(
        "{prefix}:{}",
        fresh_controller_digest(case_id, participant_id, generation)
    )
}

fn fresh_controller_digest(case_id: &str, participant_id: &str, generation: u64) -> String {
    let sequence = CONTROLLER_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    yai_core_engine::effect::digest_bytes(
        format!(
            "yai.conversation_controller.local.v1\0{case_id}\0{participant_id}\0{generation}\0{}\0{sequence}\0{now}",
            std::process::id()
        )
        .as_bytes(),
    )
    .trim_start_matches("sha256:")
    .to_string()
}

pub(super) fn authorized_conversation_case(
    case_id: &str,
    participant_id: &str,
) -> Result<AuthorizedConversationCase, String> {
    let authenticated = security::authenticate_local()?;
    let principal_id = authenticated.projected_principal_id();
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store.get_case_state_authorized(&authenticated, case_id)?;
    let tenant_id = state
        .tenant_id
        .clone()
        .ok_or_else(|| "legacy_unscoped_case_cannot_own_conversation_content".to_string())?;
    if !state
        .participants
        .iter()
        .any(|participant| participant.participant_id == participant_id)
        || !state.principal_participant_links.iter().any(|link| {
            link.participant_id == participant_id
                && link.principal_id == principal_id
                && link.tenant_id == tenant_id
        })
    {
        return Err("conversation_participant_not_linked_to_authenticated_principal".to_string());
    }
    Ok(AuthorizedConversationCase {
        store,
        state,
        authenticated,
        principal_id,
        tenant_id,
    })
}

pub(super) fn conversation_content_store() -> Result<ConversationContentStore, String> {
    ConversationContentStore::open(&yai_home())
}

pub(super) fn verify_conversation_turn(turn: &ConversationTurn) -> Result<(), String> {
    turn.validate()?;
    let store = conversation_content_store()?;
    for part in &turn.ordered_parts {
        store.verify_object(&part.object)?;
    }
    Ok(())
}

pub(super) fn commit_conversation_draft(
    draft: &ConversationDraft,
) -> Result<ConversationCommitResult, String> {
    commit_conversation_draft_with_intent(draft, None)
}

fn commit_conversation_draft_with_intent(
    draft: &ConversationDraft,
    intent: Option<&ConversationExecutionInput>,
) -> Result<ConversationCommitResult, String> {
    if draft.parts.is_empty() {
        return Err("conversation_turn_requires_content".to_string());
    }
    let authorized = authorized_conversation_case(&draft.case_id, &draft.participant_id)?;
    if authorized.state.generation != draft.base_generation {
        return Err("conversation_draft_case_generation_stale".to_string());
    }
    if authorized.tenant_id != draft.tenant_id || authorized.principal_id != draft.principal_id {
        return Err("conversation_draft_security_scope_changed".to_string());
    }
    authorized
        .store
        .resolve_security_context(&authorized.authenticated, &authorized.tenant_id)?
        .require_owner()?;
    let content_store = conversation_content_store()?;
    let objects = content_store.publish_draft(draft)?;
    let turn = ConversationTurn::build(
        &draft.case_id,
        &draft.tenant_id,
        &draft.thread_id,
        &draft.participant_id,
        &draft.principal_id,
        draft.base_generation,
        objects,
    )?;
    verify_conversation_turn(&turn)?;
    let mut causal_refs = vec![turn.participant_id.clone()];
    causal_refs.extend(
        turn.ordered_parts
            .iter()
            .map(|part| part.object.object_id.clone()),
    );
    causal_refs.sort();
    causal_refs.dedup();
    let pending = PendingTransition {
        transition_id: format!("transition:{}", turn.turn_id),
        case_id: turn.case_id.clone(),
        expected_generation: turn.base_generation,
        source: TransitionSource {
            component: "yai.case_conversation".to_string(),
            participant_id: Some(turn.participant_id.clone()),
            principal_id: Some(turn.submitted_by_principal_id.clone()),
            source_ref: Some(turn.turn_id.clone()),
        },
        scope: Some(TransitionScope {
            case_id: turn.case_id.clone(),
            participant_refs: vec![turn.participant_id.clone()],
            resource_refs: Vec::new(),
            policy_refs: Vec::new(),
        }),
        causal_refs,
        payload: TransitionPayload::ConversationTurnCommitted { turn: turn.clone() },
        provenance: Vec::new(),
        summary: Some(format!(
            "multipart conversation turn with {} ordered content parts",
            turn.ordered_parts.len()
        )),
    };
    let (transition_id, generation) = if let Some(intent) = intent {
        let request = intent.bind(&turn)?;
        let (first, second) = authorized.store.commit_conversation_submission_authorized(
            &authorized.authenticated,
            &authorized.tenant_id,
            pending,
            request,
        )?;
        (first.transition.transition_id, second.state.generation)
    } else {
        let commit = authorized.store.commit_secured_transition(
            &authorized.authenticated,
            &authorized.tenant_id,
            pending,
            false,
        )?;
        (commit.transition.transition_id, commit.state.generation)
    };
    let draft_discarded = content_store
        .discard_draft(&draft.case_id, &draft.draft_id)
        .is_ok();
    Ok(ConversationCommitResult {
        turn,
        transition_id,
        generation,
        draft_discarded,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;

    struct TestHome {
        path: PathBuf,
        prior: Option<std::ffi::OsString>,
    }

    impl TestHome {
        fn enter(label: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "yai-conversation-controller-{label}-{}-{}",
                std::process::id(),
                CONTROLLER_SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            let prior = std::env::var_os("YAI_HOME");
            std::env::set_var("YAI_HOME", &path);
            Self { path, prior }
        }
    }

    impl Drop for TestHome {
        fn drop(&mut self) {
            if let Some(prior) = self.prior.take() {
                std::env::set_var("YAI_HOME", prior);
            } else {
                std::env::remove_var("YAI_HOME");
            }
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    fn prepare_case(case_id: &str, tenant_id: &str, participant_id: &str) {
        security::security_command(&strings(&[
            "bootstrap-local",
            "--tenant",
            tenant_id,
            "--organization",
            "organization:conversation-host-test",
        ]))
        .unwrap();
        security::case_security_command(&strings(&[
            "create", "--case", case_id, "--tenant", tenant_id,
        ]))
        .unwrap();
        provider::case_bind_participant_role(&strings(&[
            "--case",
            case_id,
            "--participant",
            participant_id,
            "--role",
            "model-executor",
        ]))
        .unwrap();
        security::case_security_command(&strings(&[
            "principal",
            "link",
            "--case",
            case_id,
            "--principal",
            "self",
            "--participant",
            participant_id,
        ]))
        .unwrap();
        provider::case_admit_participant_view(&strings(&[
            "--case",
            case_id,
            "--participant",
            participant_id,
            "--consumer",
            "model",
            "--view",
            "model_context",
        ]))
        .unwrap();
    }

    fn submit_text(
        controller: &mut ConversationController,
        text: &str,
    ) -> ConversationSubmissionResult {
        match controller
            .apply(ConversationAction::Submit {
                parts: vec![ConversationInputPart::Text {
                    text: text.to_string(),
                }],
            })
            .unwrap()
        {
            ConversationActionResult::Submission { value } => *value,
            other => panic!("unexpected action result: {other:?}"),
        }
    }

    #[test]
    #[ignore = "mutates isolated YAI_HOME; exercised by smoke-conversation-interaction-host"]
    fn post_i01_host_commits_before_provider_and_derives_threads_only_from_turns() {
        let _home = TestHome::enter("commit-thread");
        let case_id = "case:conversation-host-commit";
        let tenant_id = "tenant:conversation-host-commit";
        let participant_id = "participant:conversation-host";
        prepare_case(case_id, tenant_id, participant_id);

        let mut controller = ConversationController::open(case_id, None).unwrap();
        let initial = match controller.apply(ConversationAction::Inspect).unwrap() {
            ConversationActionResult::Status { value } => value,
            other => panic!("unexpected action result: {other:?}"),
        };
        assert!(!initial.active_thread_durable);
        assert!(initial.committed_threads.is_empty());
        assert_eq!(initial.terminal_frontend, "replai_native");

        let first = submit_text(&mut controller, "canonical before provider availability");
        assert_eq!(
            first.execution.posture,
            ConversationExecutionPosture::ProviderUnavailable
        );
        assert!(matches!(
            first.execution.events.first(),
            Some(ConversationApplicationEvent::TurnCommitted { .. })
        ));
        verify_conversation_turn(&first.turn).unwrap();

        let threads = match controller.apply(ConversationAction::ListThreads).unwrap() {
            ConversationActionResult::Threads { values } => values,
            other => panic!("unexpected action result: {other:?}"),
        };
        assert_eq!(threads.len(), 1);
        assert_eq!(threads[0].committed_turn_count, 1);
        let durable_thread = threads[0].thread_id.clone();

        let ephemeral = match controller.apply(ConversationAction::NewThread).unwrap() {
            ConversationActionResult::ThreadSelected { thread_id, durable } => {
                assert!(!durable);
                thread_id
            }
            other => panic!("unexpected action result: {other:?}"),
        };
        assert_ne!(ephemeral, durable_thread);
        assert_eq!(controller.threads().unwrap().len(), 1);

        let reopened = ConversationController::open(case_id, Some(participant_id)).unwrap();
        assert_eq!(reopened.active_thread_id, durable_thread);
        assert_eq!(reopened.threads().unwrap().len(), 1);

        let selected = controller
            .apply(ConversationAction::UseThread {
                thread_id: durable_thread.clone(),
            })
            .unwrap();
        assert!(matches!(
            selected,
            ConversationActionResult::ThreadSelected { durable: true, .. }
        ));
        assert_eq!(
            controller
                .apply(ConversationAction::UseThread {
                    thread_id: ephemeral,
                })
                .unwrap_err(),
            "conversation_thread_not_in_committed_history"
        );

        let cancellation = controller.cancellation();
        cancellation.request();
        let cancelled = controller
            .execute_committed_turn(&first.turn.turn_id)
            .unwrap();
        assert_eq!(
            cancelled.posture,
            ConversationExecutionPosture::CancelledBeforeDispatch
        );
        assert!(cancelled.provider_result_id.is_none());

        let before_media_generation = authorized_conversation_case(case_id, participant_id)
            .unwrap()
            .state
            .generation;
        let media = match controller
            .apply(ConversationAction::Submit {
                parts: vec![
                    ConversationInputPart::Text {
                        text: "ordered text".to_string(),
                    },
                    ConversationInputPart::Bytes {
                        modality: ContentModality::Image,
                        media_type: "image/png".to_string(),
                        bytes: b"bounded-image-fixture".to_vec(),
                    },
                    ConversationInputPart::Bytes {
                        modality: ContentModality::Image,
                        media_type: "image/png".to_string(),
                        bytes: b"bounded-image-fixture".to_vec(),
                    },
                ],
            })
            .unwrap()
        {
            ConversationActionResult::Submission { value } => *value,
            other => panic!("unexpected action result: {other:?}"),
        };
        assert_eq!(media.generation, before_media_generation + 2);
        assert_eq!(media.turn.ordered_parts.len(), 3);
        assert_eq!(
            media.turn.ordered_parts[1].object.object_id,
            media.turn.ordered_parts[2].object.object_id
        );
        assert_ne!(
            media.turn.ordered_parts[1].part_id,
            media.turn.ordered_parts[2].part_id
        );
        assert_eq!(
            media.execution.posture,
            ConversationExecutionPosture::ProviderUnavailable
        );
        let state = authorized_conversation_case(case_id, participant_id)
            .unwrap()
            .state;
        assert!(state.resources.is_empty());
    }

    fn read_http_request(stream: &mut std::net::TcpStream) -> String {
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut bytes = Vec::new();
        let mut buffer = [0u8; 2048];
        let header_end;
        loop {
            let read = stream.read(&mut buffer).unwrap();
            assert!(read > 0);
            bytes.extend_from_slice(&buffer[..read]);
            if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                header_end = index + 4;
                break;
            }
        }
        let headers = String::from_utf8_lossy(&bytes[..header_end]);
        let content_length = headers
            .lines()
            .find_map(|line| {
                line.to_ascii_lowercase()
                    .strip_prefix("content-length:")
                    .map(str::trim)
                    .and_then(|value| value.parse::<usize>().ok())
            })
            .unwrap_or(0);
        while bytes.len() < header_end + content_length {
            let read = stream.read(&mut buffer).unwrap();
            assert!(read > 0);
            bytes.extend_from_slice(&buffer[..read]);
        }
        String::from_utf8(bytes).unwrap()
    }

    fn start_provider(request_count: usize) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            for _ in 0..request_count {
                let (mut stream, _) = listener.accept().unwrap();
                let request = read_http_request(&mut stream);
                let first_line = request.lines().next().unwrap_or_default();
                let body = if first_line.starts_with("GET ") {
                    serde_json::json!({"object":"list","data":[{"id":"model:conversation-host"}]})
                        .to_string()
                } else if request.contains("response_format") {
                    serde_json::json!({
                        "model":"model:conversation-host",
                        "choices":[{"message":{"role":"assistant","content":"{\"ok\":true}"}}],
                        "usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}
                    })
                    .to_string()
                } else {
                    serde_json::json!({
                        "model":"model:conversation-host",
                        "choices":[{"message":{"role":"assistant","content":"HOST_REPLY"}}],
                        "usage":{"prompt_tokens":2,"completion_tokens":1,"total_tokens":3}
                    })
                    .to_string()
                };
                write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                )
                .unwrap();
                stream.flush().unwrap();
            }
        });
        (format!("http://{address}/v1"), handle)
    }

    #[test]
    #[ignore = "requires loopback sockets and isolated YAI_HOME; exercised by smoke-conversation-interaction-host"]
    fn post_i01_host_reuses_governed_semantic_execution_without_operational_runtime() {
        let _home = TestHome::enter("provider-execution");
        let case_id = "case:conversation-host-provider";
        let tenant_id = "tenant:conversation-host-provider";
        let participant_id = "participant:conversation-host";
        prepare_case(case_id, tenant_id, participant_id);
        let (endpoint, server) = start_provider(5);

        provider_governance_cli::provider_governance_command(
            "yai.provider.add",
            &[
                "--tenant".to_string(),
                tenant_id.to_string(),
                "--provider-key".to_string(),
                "conversation-host-fixture".to_string(),
                "--endpoint".to_string(),
                endpoint,
                "--model".to_string(),
                "model:conversation-host".to_string(),
                "--credential-ref".to_string(),
                "none".to_string(),
                "--locality".to_string(),
                "loopback".to_string(),
            ],
        )
        .unwrap();
        let authenticated = security::authenticate_local().unwrap();
        let store = LmdbRecordStore::open(record_store_path()).unwrap();
        let target = store
            .list_provider_targets_authorized(&authenticated, tenant_id)
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        provider_governance_cli::provider_governance_command(
            "yai.provider.qualify",
            &strings(&[
                "--target",
                &target.target_id,
                "--realization-shape",
                "text_to_text",
            ]),
        )
        .unwrap();
        provider_governance_cli::provider_governance_command(
            "yai.provider.trust.approve",
            &strings(&["--target", &target.target_id]),
        )
        .unwrap();
        provider_governance_cli::provider_governance_command(
            "yai.case.provider.bind",
            &strings(&[
                "--case",
                case_id,
                "--participant",
                participant_id,
                "--target",
                &target.target_id,
                "--failover",
                "safe_only",
                "--max-attempts",
                "1",
            ]),
        )
        .unwrap();

        let evidence = store
            .record_semantic_suitability_evidence_authorized(
                &authenticated,
                &target.target_id,
                CognitiveCapability::PrimaryConversation,
                yai_core_engine::cognitive::SemanticEvidencePosture::OperatorAttested,
                "fixture:i06-host",
                "fixture:i06-run",
                vec!["evidence:deterministic-fixture".into()],
                "deterministic_loopback_fixture",
            )
            .unwrap();
        store
            .bind_case_cognitive_target_authorized(
                &authenticated,
                case_id,
                participant_id,
                yai_core_engine::cognitive::CognitiveBindingRole::Primary,
                CognitiveCapability::PrimaryConversation,
                &target.target_id,
                &evidence.evidence_id,
                false,
            )
            .unwrap();
        let mut controller = ConversationController::open(case_id, None).unwrap();
        let submission = submit_text(&mut controller, "natural Case conversation");
        assert_eq!(
            submission.execution.posture,
            ConversationExecutionPosture::Completed
        );
        assert_eq!(submission.execution.output.as_deref(), Some("HOST_REPLY"));
        assert!(matches!(
            submission.execution.events.first(),
            Some(ConversationApplicationEvent::TurnCommitted { .. })
        ));

        let retry = match controller
            .apply(ConversationAction::Retry {
                turn_id: submission.turn.turn_id.clone(),
            })
            .unwrap()
        {
            ConversationActionResult::Execution { value } => value,
            other => panic!("unexpected action result: {other:?}"),
        };
        assert_eq!(retry.posture, ConversationExecutionPosture::Completed);
        let state = authorized_conversation_case(case_id, participant_id)
            .unwrap()
            .state;
        assert!(state.resources.is_empty());
        let transitions = store.list_case_transitions(case_id).unwrap();
        let turns = turns_from_history(case_id, &transitions);
        assert_eq!(turns.len(), 1, "retry must not duplicate the user turn");
        let turn_index = transitions
            .iter()
            .position(|transition| {
                matches!(
                    &transition.payload,
                    TransitionPayload::ConversationTurnCommitted { turn }
                        if turn.turn_id == submission.turn.turn_id
                )
            })
            .unwrap();
        let invocation_indices = transitions
            .iter()
            .enumerate()
            .filter_map(|(index, transition)| {
                matches!(
                    transition.payload,
                    TransitionPayload::ProviderInvocationStarted { .. }
                )
                .then_some(index)
                .filter(|_| transition.causal_refs.contains(&submission.turn.turn_id))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            invocation_indices.len(),
            1,
            "retry reuses the recorded result"
        );
        assert_eq!(
            retry.provider_result_id,
            submission.execution.provider_result_id
        );
        assert!(invocation_indices.iter().all(|index| *index > turn_index));
        server.join().unwrap();
    }

    struct CognitiveFixture {
        process: std::process::Child,
        target: String,
        log: PathBuf,
    }

    impl Drop for CognitiveFixture {
        fn drop(&mut self) {
            let _ = self.process.kill();
            let _ = self.process.wait();
        }
    }

    impl CognitiveFixture {
        fn new(name: &str, mode: &str, shapes: &[&str]) -> Self {
            use std::io::BufRead;
            let log = yai_home().join(format!("{name}.jsonl"));
            let mut process = std::process::Command::new("python3")
                .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
                .args([
                    "tests/fixtures/provider_governance_server.py",
                    "--mode",
                    mode,
                    "--model",
                    name,
                    "--log",
                    log.to_str().unwrap(),
                ])
                .stdout(std::process::Stdio::piped())
                .spawn()
                .unwrap();
            let mut port = String::new();
            std::io::BufReader::new(process.stdout.take().unwrap())
                .read_line(&mut port)
                .unwrap();
            let endpoint = format!("http://127.0.0.1:{}", port.trim());
            provider_governance_cli::provider_governance_command(
                "yai.provider.add",
                &strings(&[
                    "--tenant",
                    "tenant:i06-host",
                    "--provider-key",
                    name,
                    "--endpoint",
                    &endpoint,
                    "--model",
                    name,
                    "--locality",
                    "loopback",
                ]),
            )
            .unwrap();
            let auth = security::authenticate_local().unwrap();
            let store = LmdbRecordStore::open(record_store_path()).unwrap();
            let target = store
                .list_provider_targets_authorized(&auth, "tenant:i06-host")
                .unwrap()
                .into_iter()
                .find(|target| target.model_id == name)
                .unwrap()
                .target_id;
            let mut args = strings(&["--target", &target]);
            for shape in shapes {
                args.extend(strings(&["--realization-shape", shape]));
            }
            provider_governance_cli::provider_governance_command("yai.provider.qualify", &args)
                .unwrap();
            provider_governance_cli::provider_governance_command(
                "yai.provider.trust.approve",
                &strings(&["--target", &target]),
            )
            .unwrap();
            Self {
                process,
                target,
                log,
            }
        }

        fn count(&self) -> usize {
            fs::read_to_string(&self.log)
                .unwrap_or_default()
                .lines()
                .filter(|line| {
                    !serde_json::from_str::<serde_json::Value>(line).unwrap()["synthetic"]
                        .as_bool()
                        .unwrap()
                })
                .count()
        }

        fn bind(&self, capability: CognitiveCapability, replace: bool) {
            let auth = security::authenticate_local().unwrap();
            let store = LmdbRecordStore::open(record_store_path()).unwrap();
            let evidence = store
                .record_semantic_suitability_evidence_authorized(
                    &auth,
                    &self.target,
                    capability.clone(),
                    yai_core_engine::cognitive::SemanticEvidencePosture::OperatorAttested,
                    "fixture:i06-host",
                    &format!("fixture:{}", self.target),
                    vec!["evidence:i06-fixture".into()],
                    "deterministic_loopback_test",
                )
                .unwrap();
            let role = if capability == CognitiveCapability::PrimaryConversation {
                yai_core_engine::cognitive::CognitiveBindingRole::Primary
            } else {
                yai_core_engine::cognitive::CognitiveBindingRole::Auxiliary
            };
            store
                .bind_case_cognitive_target_authorized(
                    &auth,
                    "case:i06-host",
                    "participant:model",
                    role,
                    capability,
                    &self.target,
                    &evidence.evidence_id,
                    replace,
                )
                .unwrap();
        }
    }

    fn i06_audio() -> Vec<ConversationInputPart> {
        let audio = std::process::Command::new("base64")
            .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
            .args([
                "--decode",
                "tests/fixtures/conversation/i03-audio.wav.base64",
            ])
            .output()
            .unwrap();
        assert!(audio.status.success());
        vec![
            ConversationInputPart::Text {
                text: "before original audio".into(),
            },
            ConversationInputPart::Bytes {
                modality: ContentModality::Audio,
                media_type: "audio/wav".into(),
                bytes: audio.stdout,
            },
            ConversationInputPart::Text {
                text: "after original audio".into(),
            },
        ]
    }

    #[test]
    #[ignore = "canonical stores, actual loopback provider and confined resource; smoke-case-capability-realization"]
    fn golden_native_capability_result_restarts_into_exact_governed_resource_request() {
        use yai_core_engine::context::InvocationOutputContract;
        use yai_core_engine::effect::access::{
            AccessKind, LocalAccessBinding, ResourceAccessContract, ResourceAddress,
        };
        use yai_core_engine::governance::{compile_policy_source, scope_policy_compilation};
        const CASE: &str = "case:i06-host";
        const TENANT: &str = "tenant:i06-host";
        const MODEL: &str = "participant:model";
        let _home = TestHome::enter("native-capability");
        prepare_case(CASE, TENANT, MODEL);
        let fixture = CognitiveFixture::new(
            "whisper-vision-name-grants-nothing",
            "capabilities",
            &["text_to_text"],
        );
        let auth = security::authenticate_local().unwrap();
        let store = LmdbRecordStore::open(record_store_path()).unwrap();
        store
            .bind_case_provider_targets_authorized(
                &auth,
                CASE,
                MODEL,
                vec![fixture.target.clone()],
                yai_core_engine::provider_governance::ProviderFailoverPolicy::SafeOnly,
                1,
            )
            .unwrap();
        fixture.bind(CognitiveCapability::PrimaryConversation, false);
        let workspace = yai_home().join("workspace");
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::write(
            workspace.join("src/retry.txt"),
            "real source evidence, not an assistant assertion",
        )
        .unwrap();
        let binding = LocalAccessBinding {
            schema: yai_core_engine::effect::access::LOCAL_ACCESS_BINDING_SCHEMA.into(),
            case_id: CASE.into(),
            attachment_id: "resource:workspace".into(),
            address: ResourceAddress::Filesystem {
                root: yai_core_engine::effect::LocalFilesystemBinding::new(
                    CASE,
                    "resource:workspace",
                    &workspace,
                )
                .unwrap(),
            },
        };
        controlled_effect::access::attach(
            &store,
            &auth,
            &binding,
            ResourceAccessContract {
                schema: yai_core_engine::effect::access::RESOURCE_ACCESS_SCHEMA.into(),
                configuration_digest: binding.digest(),
                participant_ids: vec![MODEL.into()],
                operations: vec![AccessKind::FilesystemRead],
                read_prefixes: vec!["src".into()],
                names: vec![],
                max_output_bytes: 8192,
                max_items: 16,
            },
            MODEL,
            None,
        )
        .unwrap();
        let bytes = serde_json::to_vec(&serde_json::json!({
            "schema":"yai.policy_source_input.v4","policy_key":"capability-read","source_version":"1","owner_ref":"organization:conversation-host-test",
            "source_origin":{"source_system":"contract-test","source_uri":"test://native-capability/policy"},"validity":{"mode":"unbounded"},
            "rules":[{"kind":"operation_restriction","rule_id":"read","operation_kind":"filesystem.read","resource_kind":"filesystem","effect":"allow","reason":"read the exact admitted subtree"}]
        })).unwrap();
        let compilation = scope_policy_compilation(
            &compile_policy_source(&bytes).unwrap(),
            TENANT,
            "organization:conversation-host-test",
        )
        .unwrap();
        let artifact = &compilation.artifact.artifact_id;
        store
            .ingest_tenant_policy_compilation(&auth, TENANT, &compilation)
            .unwrap();
        store
            .validate_tenant_policy_artifact(&auth, artifact, "test validation")
            .unwrap();
        store
            .publish_tenant_policy_artifact(&auth, artifact, "test publication")
            .unwrap();
        store
            .bind_tenant_case_policy(
                &auth,
                CASE,
                artifact,
                store.get_case_state(CASE).unwrap().unwrap().generation,
                "test exact binding",
            )
            .unwrap();
        let mut controller = ConversationController::open(CASE, Some(MODEL)).unwrap();
        let committed = controller
            .commit_parts(vec![ConversationInputPart::Text {
                text: "Inspect the admitted retry source".into(),
            }])
            .unwrap();
        let content = conversation_content_store().unwrap();
        let parts = super::super::cognitive_execution::source_parts(&content, &committed.turn, &[])
            .unwrap();
        let view = store
            .case_capability_view_authorized(&auth, CASE, MODEL)
            .unwrap();
        assert_eq!(
            fixture.count(),
            0,
            "SEND and capability inspection dispatch no provider"
        );
        let realize = |store: &LmdbRecordStore, view, failpoint| {
            super::super::cognitive_execution::realize_cognitive(
                &auth,
                store,
                &content,
                &committed.turn,
                MODEL,
                CognitiveCapability::PrimaryConversation,
                parts.clone(),
                vec![],
                "test:exact-native-source",
                Some(yai_core_engine::cognitive::CognitivePlanRoute::Native),
                None,
                failpoint,
                std::slice::from_ref(&committed.turn.turn_id),
                &|| false,
                InvocationOutputContract::CaseCapabilities {
                    view: Box::new(view),
                    catalogs: vec![],
                    feedback_result_ids: vec![],
                },
            )
        };
        assert!(realize(&store, view.clone(), None)
            .unwrap_err()
            .contains("shape_not_qualified"));
        assert_eq!(
            fixture.count(),
            0,
            "semantic suitability is not function realization evidence"
        );
        provider_governance_cli::provider_governance_command(
            "yai.provider.qualify",
            &strings(&[
                "--target",
                &fixture.target,
                "--realization-shape",
                "text_functions_to_text_or_call",
            ]),
        )
        .unwrap();
        let failure = realize(&store, view, Some("after-provider-result")).unwrap_err();
        assert!(
            failure.contains("failpoint_after_provider_result"),
            "{failure}"
        );
        assert_eq!(fixture.count(), 1);
        let history = store.list_case_transitions(CASE).unwrap();
        assert!(!history
            .iter()
            .any(|t| matches!(t.payload, TransitionPayload::OperationRecorded { .. })));
        let result_id = history
            .iter()
            .find_map(|t| match &t.payload {
                TransitionPayload::ProviderResultRecorded { result_id, .. } => {
                    Some(result_id.clone())
                }
                _ => None,
            })
            .unwrap();
        let turn_index = history
            .iter()
            .position(|t| {
                matches!(
                    t.payload,
                    TransitionPayload::ConversationTurnCommitted { .. }
                )
            })
            .unwrap();
        assert!(
            history
                .iter()
                .position(|t| matches!(
                    t.payload,
                    TransitionPayload::ProviderInvocationStarted { .. }
                ))
                .unwrap()
                > turn_index
        );
        drop(store);
        let store = LmdbRecordStore::open(record_store_path()).unwrap();
        let operation = store
            .record_provider_capability_request(&auth, CASE, &result_id)
            .unwrap();
        assert_eq!(operation.participant_id, MODEL);
        let outcome = controlled_effect::access::advance(&store, &auth, &operation).unwrap();
        let controlled_effect::access::ResourceActionOutcome::Observed {
            observation,
            reused: false,
        } = outcome
        else {
            panic!("{outcome:?}")
        };
        assert_eq!(
            observation.result["text"],
            "real source evidence, not an assistant assertion"
        );
        assert_eq!(
            fixture.count(),
            1,
            "result recovery never redispatches the model"
        );
        assert_eq!(
            store
                .record_provider_capability_request(&auth, CASE, &result_id)
                .unwrap(),
            operation
        );
        assert!(matches!(
            controlled_effect::access::advance(&store, &auth, &operation).unwrap(),
            controlled_effect::access::ResourceActionOutcome::Observed { reused: true, .. }
        ));
        let history = store.list_case_transitions(CASE).unwrap();
        assert_eq!(
            history
                .iter()
                .filter(|t| matches!(t.payload, TransitionPayload::OperationRecorded { .. }))
                .count(),
            1
        );
        assert_eq!(turns_from_history(CASE, &history), vec![&committed.turn]);
        let projection = yai_core_engine::context::compile_projection(
            &store.get_case_state(CASE).unwrap().unwrap(),
            &history,
            &yai_core_engine::context::ProjectionRequest::model(
                MODEL,
                yai_core_engine::context::ProjectionPurpose::Conversation,
            ),
            &yai_core_engine::context::DerivedProjectionInput::default(),
        )
        .unwrap();
        assert!(serde_json::to_string(&projection)
            .unwrap()
            .contains(&observation.observation_id));
        assert_eq!(
            store.replay_case_state(CASE).unwrap(),
            store.get_case_state(CASE).unwrap().unwrap()
        );
        println!(
            "capability_identifiers: {}",
            serde_json::json!({"turn_id":committed.turn.turn_id,"provider_result_id":result_id,"operation_id":operation.operation_id,"observation_id":observation.observation_id,"case_generation":store.get_case_state(CASE).unwrap().unwrap().generation})
        );
        println!("capability_realization: provider_mode=loopback_fixture dispatches=1 canonical_turns=1 operations=1 source=real_confined_file restart_result_reused=true observation_projected=true replay_equal=true no_yvex=true");
        // Exercise the SAME application host as normal conversation, with an
        // explicit immutable work budget. Crash after the real resource result,
        // then reconstruct native tool-result feedback rather than repeat work.
        let work_turn = controller
            .commit_parts_with_intent(
                vec![ConversationInputPart::Text {
                    text: "Inspect source using the admitted capability and report the evidence"
                        .into(),
                }],
                &ConversationExecutionInput {
                    work_limits: Some(yai_core_engine::conversation::CaseWorkLimits {
                        invocations: 4,
                        operations: 3,
                        effects: 0,
                        max_input_units: 8192,
                    }),
                    ..Default::default()
                },
            )
            .unwrap();
        let paused = controller
            .execute_turn_with_failpoint(
                &work_turn.turn,
                &mut Vec::new(),
                Some("after-capability-outcome"),
            )
            .unwrap();
        assert_eq!(paused.posture, ConversationExecutionPosture::Unresolved);
        assert_eq!(
            fixture.count(),
            2,
            "resource completion preceded the simulated crash"
        );
        drop(controller);
        drop(store);
        let mut controller = ConversationController::open(CASE, Some(MODEL)).unwrap();
        let resumed = controller.retry(&work_turn.turn.turn_id).unwrap();
        assert_eq!(
            resumed.posture,
            ConversationExecutionPosture::Completed,
            "{resumed:?}"
        );
        assert_eq!(
            resumed.output.as_deref(),
            Some("case-work-complete: exact source observation received")
        );
        assert_eq!(resumed.work.as_ref().unwrap().steps.len(), 2);
        assert_eq!(
            fixture.count(),
            3,
            "one next cognition, zero duplicated read request"
        );
        let repeated = controller.retry(&work_turn.turn.turn_id).unwrap();
        assert_eq!(repeated.posture, ConversationExecutionPosture::Completed);
        assert_eq!(
            fixture.count(),
            3,
            "completed work never dispatches on retry"
        );
        let store = LmdbRecordStore::open(record_store_path()).unwrap();
        let history = store.list_case_transitions(CASE).unwrap();
        assert_eq!(turns_from_history(CASE, &history).len(), 2);
        assert_eq!(
            history
                .iter()
                .filter(|t| matches!(t.payload, TransitionPayload::OperationRecorded { .. }))
                .count(),
            2
        );
        assert_eq!(
            store.replay_case_state(CASE).unwrap(),
            store.get_case_state(CASE).unwrap().unwrap()
        );
        println!(
            "bounded_case_work: {}",
            serde_json::to_string(&resumed).unwrap()
        );
        let prior = history
            .iter()
            .find(|t| {
                matches!(&t.payload,
            TransitionPayload::ProviderInvocationStarted {invocation_id,..}
                if Some(invocation_id) == resumed.invocation_id.as_ref())
            })
            .unwrap();
        let mut payload = prior.payload.clone();
        if let TransitionPayload::ProviderInvocationStarted { invocation_id, .. } = &mut payload {
            *invocation_id = "invocation:forged-repeat-completed-step".into();
        }
        let before = store.get_case_state(CASE).unwrap().unwrap();
        let mut duplicate = PendingTransition::new(
            "transition:forged-repeat-completed-step",
            CASE,
            before.generation,
            prior.source.clone(),
            payload,
        );
        duplicate.causal_refs = prior.causal_refs.clone();
        duplicate.scope = prior.scope.clone();
        let refused = store
            .commit_secured_transition(&auth, TENANT, duplicate, false)
            .unwrap_err();
        assert!(
            refused.contains("case_work_step_completed_requires_result_reuse"),
            "{refused}"
        );
        assert_eq!(store.get_case_state(CASE).unwrap().unwrap(), before);
        let bounded = controller
            .commit_work(
                "Inspect with exactly one model invocation".into(),
                yai_core_engine::conversation::CaseWorkLimits {
                    invocations: 1,
                    operations: 1,
                    effects: 0,
                    max_input_units: 8192,
                },
            )
            .unwrap();
        let exhausted = controller
            .execute_committed_turn(&bounded.turn.turn_id)
            .unwrap();
        assert_eq!(
            exhausted.posture,
            ConversationExecutionPosture::BudgetExhausted,
            "{exhausted:?}"
        );
        let calls = fixture.count();
        assert_eq!(
            controller.retry(&bounded.turn.turn_id).unwrap().posture,
            ConversationExecutionPosture::BudgetExhausted
        );
        assert_eq!(
            fixture.count(),
            calls,
            "restart/retry cannot replenish immutable work budget"
        );
        println!("case_work_adversarial: completed_step_raw_redispatch=rejected state_unchanged=true invocation_budget=1 retry_budget_reset=false");
    }

    #[test]
    #[ignore = "real loopback provider and isolated YAI_HOME; smoke-conversation-executor-delegation"]
    fn golden_turn_author_delegates_execution_without_identity_or_review_transfer() {
        const CASE: &str = "case:i06-host";
        const TENANT: &str = "tenant:i06-host";
        const HUMAN: &str = "participant:operator";
        const MODEL: &str = "participant:model";
        let _home = TestHome::enter("golden-delegation");
        security::security_command(&strings(&[
            "bootstrap-local",
            "--tenant",
            TENANT,
            "--organization",
            "organization:golden-host-test",
        ]))
        .unwrap();
        security::case_security_command(&strings(&["create", "--case", CASE, "--tenant", TENANT]))
            .unwrap();
        for (participant, role) in [(HUMAN, "human-reviewer"), (MODEL, "model-executor")] {
            provider::case_bind_participant_role(&strings(&[
                "--case",
                CASE,
                "--participant",
                participant,
                "--role",
                role,
            ]))
            .unwrap();
        }
        security::case_security_command(&strings(&[
            "principal",
            "link",
            "--case",
            CASE,
            "--principal",
            "self",
            "--participant",
            HUMAN,
        ]))
        .unwrap();
        let mut controller = ConversationController::open(CASE, Some(HUMAN)).unwrap();
        assert!(controller
            .select_executor(MODEL)
            .unwrap_err()
            .contains("view_not_admitted"));
        provider::case_admit_participant_view(&strings(&[
            "--case",
            CASE,
            "--participant",
            MODEL,
            "--consumer",
            "model",
            "--view",
            "model_context",
        ]))
        .unwrap();
        controller.select_executor(MODEL).unwrap();
        let primary = CognitiveFixture::new("not-an-agent", "full", &["text_to_text"]);
        let auxiliary = CognitiveFixture::new("not-a-transcriber", "full", &["audio_wav_to_text"]);
        let auth = security::authenticate_local().unwrap();
        let store = LmdbRecordStore::open(record_store_path()).unwrap();
        store
            .bind_case_provider_targets_authorized(
                &auth,
                CASE,
                MODEL,
                vec![primary.target.clone(), auxiliary.target.clone()],
                yai_core_engine::provider_governance::ProviderFailoverPolicy::SafeOnly,
                1,
            )
            .unwrap();
        primary.bind(CognitiveCapability::PrimaryConversation, false);
        auxiliary.bind(CognitiveCapability::SpeechToText, false);
        let committed = controller
            .commit_parts(vec![ConversationInputPart::Text {
                text: "Human-authored input".into(),
            }])
            .unwrap();
        assert_eq!(committed.turn.participant_id, HUMAN);
        assert_eq!(primary.count(), 0, "SEND has no provider execution");
        let history = store.list_case_transitions(CASE).unwrap();
        let intent = history
            .iter()
            .find_map(|t| match &t.payload {
                TransitionPayload::ConversationExecutionIntentRecorded { request } => Some(request),
                _ => None,
            })
            .unwrap();
        assert_eq!(intent.participant_id, MODEL);
        assert_eq!(
            intent.schema,
            yai_core_engine::conversation::DELEGATED_COMPOSITION_REQUEST_SCHEMA
        );
        let result = controller
            .execute_committed_turn(&committed.turn.turn_id)
            .unwrap();
        assert_eq!(
            result.posture,
            ConversationExecutionPosture::Completed,
            "{result:?}"
        );
        assert_eq!(
            result
                .cognition
                .as_ref()
                .unwrap()
                .primary
                .plan
                .participant_id,
            MODEL
        );
        assert_eq!(primary.count(), 1);
        let mut reopened = ConversationController::open(CASE, Some(HUMAN)).unwrap();
        let retry = reopened.retry(&committed.turn.turn_id).unwrap();
        assert_eq!(retry.provider_result_id, result.provider_result_id);
        assert_eq!(primary.count(), 1);
        assert!(
            ConversationController::open(CASE, Some(MODEL)).is_err(),
            "delegation cannot impersonate model as human"
        );
        let explicit = ConversationExecutionInput {
            workflow_execution_id: None,
            work_limits: None,
            prerequisite: Some((CognitiveCapability::SpeechToText, vec![1])),
            executor_participant_id: Some(MODEL.into()),
        };
        let media = reopened
            .commit_parts_with_intent(i06_audio(), &explicit)
            .unwrap();
        let interrupted = reopened
            .execute_turn_with_failpoint(&media.turn, &mut vec![], Some("after-prerequisite"))
            .unwrap();
        assert_eq!(
            interrupted.posture,
            ConversationExecutionPosture::Unresolved
        );
        assert_eq!((auxiliary.count(), primary.count()), (1, 1));
        let mut reopened = ConversationController::open(CASE, Some(HUMAN)).unwrap();
        let resumed = reopened.retry(&media.turn.turn_id).unwrap();
        assert_eq!(
            resumed.posture,
            ConversationExecutionPosture::Completed,
            "{resumed:?}"
        );
        assert_eq!((auxiliary.count(), primary.count()), (1, 2));
        let state = store.get_case_state(CASE).unwrap().unwrap();
        assert_eq!(state.principal_participant_links.len(), 1);
        assert_eq!(state.principal_participant_links[0].participant_id, HUMAN);
        assert!(state
            .participants
            .iter()
            .find(|p| p.participant_id == HUMAN)
            .unwrap()
            .admitted_views
            .is_empty());
        let history = store.list_case_transitions(CASE).unwrap();
        let derived = yai_core_engine::conversation::derived_content_from_history(CASE, &history);
        assert_eq!(derived.len(), 1);
        assert_eq!(derived[0].source_turn_id, media.turn.turn_id);
        assert_eq!(
            yai_core_engine::transition::replay_case(CASE, &history).unwrap(),
            state
        );
        assert_eq!(
            find_turn(CASE, &media.turn.turn_id, &history).unwrap(),
            &media.turn
        );
        println!(
            "GOLDEN_DELEGATED_HOST:{}",
            serde_json::json!({"case":CASE,"author":HUMAN,"executor":MODEL,
            "turn":committed.turn.turn_id,"intent":result.intent,"result":result.provider_result_id,
            "derived":derived[0].derived_content_id,"resumed_result":resumed.provider_result_id,
            "provider_mode":"loopback_fixture","golden_lifecycle":"not_yet_qualified"})
        );
    }

    #[test]
    #[ignore = "real loopback HTTP and isolated YAI_HOME; smoke-conversation-cognitive-host"]
    fn i06_typed_host_native_composed_restart_and_fail_closed() {
        const CASE: &str = "case:i06-host";
        if let Ok(turn_id) = std::env::var("YAI_I06_HOST_RESUME") {
            let mut controller =
                ConversationController::open(CASE, Some("participant:model")).unwrap();
            let result = controller.execute_committed_turn(&turn_id).unwrap();
            assert_eq!(result.posture, ConversationExecutionPosture::Completed);
            let cognition = result.cognition.as_ref().unwrap();
            assert_eq!(cognition.route, "composed");
            assert!(cognition.prerequisite.as_ref().unwrap().recovered);
            assert!(!cognition.primary.recovered);
            println!("I06_RESUMED:{}", serde_json::to_string(&result).unwrap());
            return;
        }
        let _home = TestHome::enter("i06-typed-host");
        prepare_case(CASE, "tenant:i06-host", "participant:model");
        let primary =
            CognitiveFixture::new("whisper-vision-name-text-only", "full", &["text_to_text"]);
        let native = CognitiveFixture::new(
            "plain-native",
            "full",
            &[
                "text_to_text",
                "audio_wav_to_text",
                "ordered_png_text_to_text",
            ],
        );
        let aux = CognitiveFixture::new("not-whisper", "full", &["audio_wav_to_text"]);
        let drop_peer = CognitiveFixture::new(
            "deepseek-strongest-name",
            "drop_realization",
            &["audio_wav_to_text"],
        );
        let malformed = CognitiveFixture::new(
            "speech-name",
            "malformed_realization",
            &["audio_wav_to_text"],
        );
        let empty =
            CognitiveFixture::new("vision-name", "empty_realization", &["audio_wav_to_text"]);
        let auth = security::authenticate_local().unwrap();
        let store = LmdbRecordStore::open(record_store_path()).unwrap();
        store
            .bind_case_provider_targets_authorized(
                &auth,
                CASE,
                "participant:model",
                [&primary, &native, &aux, &drop_peer, &malformed, &empty]
                    .iter()
                    .map(|peer| peer.target.clone())
                    .collect(),
                yai_core_engine::provider_governance::ProviderFailoverPolicy::SafeOnly,
                3,
            )
            .unwrap();
        native.bind(CognitiveCapability::PrimaryConversation, false);
        aux.bind(CognitiveCapability::SpeechToText, false);
        let mut controller = ConversationController::open(CASE, Some("participant:model")).unwrap();
        let commit = controller.commit_parts(i06_audio()).unwrap();
        let direct = controller
            .execute_committed_turn(&commit.turn.turn_id)
            .unwrap();
        assert_eq!(direct.posture, ConversationExecutionPosture::Completed);
        assert_eq!(direct.cognition.as_ref().unwrap().route, "direct");
        assert!(direct.intent.as_ref().unwrap().prerequisite.is_none());
        assert_eq!((native.count(), aux.count()), (1, 0));
        // Repeated equal images retain distinct ordered positions through the same host.
        let image = std::process::Command::new("base64")
            .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
            .args([
                "--decode",
                "tests/fixtures/conversation/i03-image.png.base64",
            ])
            .output()
            .unwrap()
            .stdout;
        let parts = vec![
            ConversationInputPart::Text {
                text: "two distinct images".into(),
            },
            ConversationInputPart::Bytes {
                modality: ContentModality::Image,
                media_type: "image/png".into(),
                bytes: image.clone(),
            },
            ConversationInputPart::Bytes {
                modality: ContentModality::Image,
                media_type: "image/png".into(),
                bytes: image,
            },
        ];
        let images = controller.commit_parts(parts).unwrap();
        let image_result = controller
            .execute_committed_turn(&images.turn.turn_id)
            .unwrap();
        assert_eq!(
            image_result.posture,
            ConversationExecutionPosture::Completed
        );
        assert_eq!(
            image_result
                .cognition
                .as_ref()
                .unwrap()
                .closure
                .delivery_count,
            3
        );
        assert_ne!(
            images.turn.ordered_parts[1].part_id,
            images.turn.ordered_parts[2].part_id
        );
        primary.bind(CognitiveCapability::PrimaryConversation, true);
        let unsupported = controller.commit_parts(i06_audio()).unwrap();
        let result = controller
            .execute_committed_turn(&unsupported.turn.turn_id)
            .unwrap();
        assert_eq!(result.posture, ConversationExecutionPosture::Unresolved);
        assert_eq!(
            (primary.count(), aux.count()),
            (0, 0),
            "MIME/name cannot invent a prerequisite"
        );
        // Retry retains the exact default intent but replans after a legitimate
        // binding change makes the original source directly realizable.
        native.bind(CognitiveCapability::PrimaryConversation, true);
        let retried_native = controller.retry(&unsupported.turn.turn_id).unwrap();
        assert_eq!(
            retried_native.posture,
            ConversationExecutionPosture::Completed
        );
        assert_eq!(retried_native.intent, result.intent);
        assert_eq!(retried_native.cognition.as_ref().unwrap().route, "direct");
        assert_eq!((native.count(), aux.count()), (3, 0));
        primary.bind(CognitiveCapability::PrimaryConversation, true);
        let explicit = ConversationExecutionInput {
            workflow_execution_id: None,
            work_limits: None,
            prerequisite: Some((CognitiveCapability::SpeechToText, vec![1])),
            executor_participant_id: None,
        };
        let committed = controller
            .commit_parts_with_intent(i06_audio(), &explicit)
            .unwrap();
        let interrupted = controller
            .execute_turn_with_failpoint(&committed.turn, &mut vec![], Some("after-prerequisite"))
            .unwrap();
        assert_eq!(
            interrupted.posture,
            ConversationExecutionPosture::Unresolved
        );
        assert_eq!((primary.count(), aux.count()), (0, 1));
        let history = store.list_case_transitions(CASE).unwrap();
        let derived = yai_core_engine::conversation::derived_content_from_history(CASE, &history);
        assert_eq!(derived.len(), 1);
        assert_eq!(
            derived[0].source_part_ids,
            vec![committed.turn.ordered_parts[1].part_id.clone()]
        );
        let original_derivation = derived[0].clone();
        let original_turns = turns_from_history(CASE, &history)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        drop(controller);
        store.rebuild_case_state(CASE).unwrap();
        // A fresh OS process exercises the host, not the Advanced compose CLI.
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["command_adapters::conversation_controller::tests::i06_typed_host_native_composed_restart_and_fail_closed",
                "--exact", "--ignored", "--nocapture", "--test-threads=1"])
            .env("YAI_I06_HOST_RESUME", &committed.turn.turn_id).output().unwrap();
        println!("{}", String::from_utf8_lossy(&child.stdout));
        assert!(
            child.status.success(),
            "{}",
            String::from_utf8_lossy(&child.stderr)
        );
        assert_eq!((primary.count(), aux.count()), (1, 1));
        let mut controller = ConversationController::open(CASE, Some("participant:model")).unwrap();
        let retry = controller.retry(&committed.turn.turn_id).unwrap();
        assert_eq!(retry.posture, ConversationExecutionPosture::Completed);
        assert!(retry.cognition.as_ref().unwrap().primary.recovered);
        assert_eq!((primary.count(), aux.count()), (1, 1));
        let history = store.list_case_transitions(CASE).unwrap();
        assert_eq!(
            turns_from_history(CASE, &history)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>(),
            original_turns
        );
        assert_eq!(
            yai_core_engine::conversation::derived_content_from_history(CASE, &history),
            vec![&original_derivation]
        );
        for peer in [&drop_peer, &malformed, &empty] {
            peer.bind(CognitiveCapability::SpeechToText, true);
            let committed = controller
                .commit_parts_with_intent(i06_audio(), &explicit)
                .unwrap();
            let result = controller
                .execute_committed_turn(&committed.turn.turn_id)
                .unwrap();
            assert_ne!(result.posture, ConversationExecutionPosture::Completed);
            assert_eq!(primary.count(), 1);
            assert_eq!(peer.count(), 1);
            let history = store.list_case_transitions(CASE).unwrap();
            assert_eq!(
                yai_core_engine::conversation::derived_content_from_history(CASE, &history).len(),
                1
            );
            if peer.target == drop_peer.target {
                assert_eq!(
                    result.posture,
                    ConversationExecutionPosture::DeliveryIndeterminate
                );
                aux.bind(CognitiveCapability::SpeechToText, true);
                let retry = controller.retry(&committed.turn.turn_id).unwrap();
                assert_eq!(
                    retry.posture,
                    ConversationExecutionPosture::DeliveryIndeterminate
                );
                assert_eq!((primary.count(), aux.count(), drop_peer.count()), (1, 1, 1));
            }
        }
        println!("i06_host: audio_and_repeated_images_native=true no_mime_inference=true explicit_composition=true process_restart_reuses_auxiliary=true retry_same_turn_and_intent=true failed_prerequisite_blocks_primary=true uncertain_delivery_no_cross_target=true");
    }
}
