//! Host-independent Case conversation application semantics.
//!
//! This controller is deliberately not a terminal REPL. It accepts typed
//! actions and ordered semantic content, commits the user turn first, and then
//! reuses YAI's Projection/Context/provider-governance invocation boundary.
//! A future terminal frontend (including Replia) and a graphical client can
//! adapt to this boundary without owning Case or conversation truth.

// The production consumer is intentionally the pending external frontend
// adapter. Until Replia exists, qualification exercises this seam directly
// instead of adding a temporary private terminal implementation.
#![allow(dead_code)]

use super::*;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use yai_core_engine::conversation::{
    find_turn, turns_from_history, ContentModality, ContentPartProvenance,
    ConversationContentStore, ConversationDraft, ConversationTurn, CONVERSATION_DRAFT_SCHEMA,
};
use yai_core_engine::provider_governance::{ProviderDeliveryClass, ProviderRequirement};
use yai_core_engine::security::AuthenticatedPrincipal;
use yai_core_engine::transition::{CaseState, TransitionScope};

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
    TypedMediaAdapterPending,
    CancelledBeforeDispatch,
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
        value: ConversationExecutionResult,
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
    active_thread_id: String,
    cancellation: ConversationCancellation,
}

impl ConversationController {
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
            active_thread_id,
            cancellation: ConversationCancellation::default(),
        })
    }

    pub(super) fn cancellation(&self) -> ConversationCancellation {
        self.cancellation.clone()
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
            ConversationAction::Retry { turn_id } => self
                .retry(&turn_id)
                .map(|value| ConversationActionResult::Execution { value }),
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
        commit_conversation_draft(&draft)
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
        self.execute_turn(turn, &mut events)
    }

    fn execute_turn(
        &self,
        turn: &ConversationTurn,
        events: &mut Vec<ConversationApplicationEvent>,
    ) -> Result<ConversationExecutionResult, String> {
        verify_conversation_turn(turn)?;
        let task = match turn.provider_text_input() {
            Ok(task) => task,
            Err(error) if error == "conversation_turn_requires_typed_media_provider_adapter" => {
                return Ok(unavailable_execution(
                    turn,
                    ConversationExecutionPosture::TypedMediaAdapterPending,
                    error,
                    events,
                ));
            }
            Err(error) => return Err(error),
        };
        if self.cancellation.is_requested() {
            return Ok(unavailable_execution(
                turn,
                ConversationExecutionPosture::CancelledBeforeDispatch,
                "conversation_cancellation_observed_before_provider_dispatch".to_string(),
                events,
            ));
        }
        let state = authorized_conversation_case(&self.case_id, &self.participant_id)?.state;
        let Some(binding) = state.provider_binding.as_ref() else {
            return Ok(unavailable_execution(
                turn,
                ConversationExecutionPosture::ProviderUnavailable,
                "case_governed_provider_binding_required".to_string(),
                events,
            ));
        };
        if binding.participant_id != self.participant_id {
            return Ok(unavailable_execution(
                turn,
                ConversationExecutionPosture::ProviderUnavailable,
                "case_provider_binding_participant_mismatch".to_string(),
                events,
            ));
        }
        let requirement = ProviderRequirement::text("conversation")?;
        let logical_turn_id = format!(
            "conversation-execution:{}:{}",
            turn.turn_id, state.generation
        );
        let mut attempted_targets = BTreeSet::new();
        let mut prior_attempt_retry_safe = false;
        for attempt_number in 1..=binding.max_attempts_per_turn {
            if self.cancellation.is_requested() {
                return Ok(unavailable_execution(
                    turn,
                    ConversationExecutionPosture::CancelledBeforeDispatch,
                    "conversation_cancellation_observed_before_provider_dispatch".to_string(),
                    events,
                ));
            }
            let route = match provider::governed_provider_route_for_attempt(
                &self.case_id,
                &self.participant_id,
                &requirement,
                &logical_turn_id,
                attempt_number,
                &attempted_targets,
                prior_attempt_retry_safe,
            ) {
                Ok(route) => route,
                Err(error) => {
                    return Ok(unavailable_execution(
                        turn,
                        ConversationExecutionPosture::ProviderUnavailable,
                        error,
                        events,
                    ));
                }
            };
            events.push(ConversationApplicationEvent::ProviderSelected {
                selection_id: route.selection.selection_id.clone(),
                target_id: route.selection.selected_target_id.clone(),
                attempt_number,
            });
            if self.cancellation.is_requested() {
                return Ok(unavailable_execution(
                    turn,
                    ConversationExecutionPosture::CancelledBeforeDispatch,
                    "conversation_cancellation_observed_after_selection_before_provider_dispatch"
                        .to_string(),
                    events,
                ));
            }
            let options = provider::SemanticInvocationOptions {
                conversation_turn_id: Some(turn.turn_id.clone()),
                ..provider::SemanticInvocationOptions::default()
            };
            match provider::invoke_semantic_provider(
                &route.args,
                ProjectionPurpose::Conversation,
                &task,
                InvocationOutputContract::NaturalLanguage,
                &options,
            ) {
                Ok(result) => {
                    provider::record_governed_provider_outcome(
                        &self.case_id,
                        &route.selection,
                        None,
                        Some(result.request_bytes_written),
                    )?;
                    events.push(ConversationApplicationEvent::ProviderResultRecorded {
                        invocation_id: result.invocation_id.clone(),
                        result_id: result.result_id.clone(),
                    });
                    return Ok(ConversationExecutionResult {
                        turn_id: turn.turn_id.clone(),
                        posture: ConversationExecutionPosture::Completed,
                        output: Some(result.raw_output),
                        selection_id: Some(route.selection.selection_id),
                        invocation_id: Some(result.invocation_id),
                        provider_result_id: Some(result.result_id),
                        projection_id: Some(result.projection_id),
                        context_frame_id: Some(result.context_frame_id),
                        events: events.clone(),
                    });
                }
                Err(error) => {
                    let outcome = provider::record_governed_provider_outcome(
                        &self.case_id,
                        &route.selection,
                        Some(&error),
                        None,
                    )?;
                    let delivery_indeterminate =
                        outcome.delivery == ProviderDeliveryClass::DeliveryIndeterminate;
                    let retry_safe = outcome.retry_safe();
                    attempted_targets.insert(route.selection.selected_target_id.clone());
                    if !delivery_indeterminate
                        && retry_safe
                        && attempt_number < binding.max_attempts_per_turn
                    {
                        prior_attempt_retry_safe = true;
                        continue;
                    }
                    let posture = if delivery_indeterminate {
                        ConversationExecutionPosture::DeliveryIndeterminate
                    } else {
                        ConversationExecutionPosture::ProviderFailed
                    };
                    return Ok(unavailable_execution(turn, posture, error, events));
                }
            }
        }
        Ok(unavailable_execution(
            turn,
            ConversationExecutionPosture::ProviderFailed,
            "conversation_provider_attempt_bound_exhausted".to_string(),
            events,
        ))
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
            terminal_frontend: "awaiting_replia_integration",
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
            participant
                .admitted_views
                .iter()
                .any(|view| view.consumer == "model" && view.view_kind == "model_context")
                && state.principal_participant_links.iter().any(|link| {
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
    let commit = authorized.store.commit_secured_transition(
        &authorized.authenticated,
        &authorized.tenant_id,
        pending,
        false,
    )?;
    let draft_discarded = content_store
        .discard_draft(&draft.case_id, &draft.draft_id)
        .is_ok();
    Ok(ConversationCommitResult {
        turn,
        transition_id: commit.transition.transition_id,
        generation: commit.state.generation,
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
        assert_eq!(initial.terminal_frontend, "awaiting_replia_integration");

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
        assert_eq!(media.generation, before_media_generation + 1);
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
            ConversationExecutionPosture::TypedMediaAdapterPending
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
            &strings(&["--target", &target.target_id]),
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
        assert_eq!(invocation_indices.len(), 2);
        assert!(invocation_indices.iter().all(|index| *index > turn_index));
        server.join().unwrap();
    }
}
