//! Disposable execution loop that advances durable Case state.
//!
//! The loop owns a bounded transition algorithm. It has no canonical state,
//! model memory, policy authority, or resource authority; restart always
//! resumes from the Transition ledger and materialized CaseState.

use super::*;
use crate::command_adapters::controlled_effect::{
    advance_controlled_workflow_deterministic, ControlledEffectTurnStatus,
};
use crate::command_adapters::security::authenticate_local;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::io::Write;
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};
use yai_core_engine::case_policy::{NormativeReadiness, PolicyValidityPosture};
use yai_core_engine::conversation::{find_turn, ContentModality, ConversationContentStore};
use yai_core_engine::effect::OPERATION_PROPOSAL_SCHEMA;
use yai_core_engine::provider_governance::{
    ProviderDeliveryClass, ProviderRequirement, ProviderSelection,
};
use yai_core_engine::store::lmdb::{
    CaseRuntimeAdmissionOutcome, CaseRuntimeAdmissionRequest, RuntimeWorkItem, RuntimeWorkState,
};

const CASE_RUNTIME_CHECKPOINT_SCHEMA: &str = "yai.case_runtime_checkpoint.v3";
const CASE_RUNTIME_CHECKPOINT_SCHEMA_V2: &str = "yai.case_runtime_checkpoint.v2";
const CASE_RUNTIME_CHECKPOINT_SCHEMA_V1: &str = "yai.case_runtime_checkpoint.v1";
const CASE_RUNTIME_OUTPUT_SCHEMA: &str = "yai.case_runtime_turn.v1";
const CASE_RUNTIME_ADMISSION_TTL_MS: u64 = 30 * 60 * 1000;

fn runtime_failpoint(args: &[String]) -> Option<String> {
    optional_arg(args, "--failpoint")
}

fn exit_runtime_failpoint(name: &str, code: i32) -> ! {
    eprintln!("case_runtime_crash_injected: {name}");
    std::process::exit(code)
}

fn runtime_now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn runtime_error_preview(value: &str) -> String {
    value.chars().take(160).collect()
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CaseRuntimeStop {
    Running,
    Completed,
    Denied,
    AwaitingReview,
    IndeterminateEffect,
    WaitingProvider,
    DeliveryIndeterminate,
    ProviderFailureBudgetExhausted,
    InvocationBudgetExhausted,
    OperationBudgetExhausted,
    ContextBudgetExhausted,
    CostBudgetExhausted,
    OperatorStopped,
    MalformedProviderResult,
    FatalInvariantViolation,
    NormativeUnconfigured,
    NormativeBlocked,
    PolicyNotYetValid,
    PolicyRefreshRequired,
    PolicyStale,
    PolicyExpired,
    PolicyRevoked,
    PolicyValidityUnavailable,
    Cancelled,
    Closed,
}

impl CaseRuntimeStop {
    pub(super) fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Denied => "denied",
            Self::AwaitingReview => "awaiting_review",
            Self::IndeterminateEffect => "indeterminate_effect",
            Self::WaitingProvider => "waiting_provider",
            Self::DeliveryIndeterminate => "delivery_indeterminate",
            Self::ProviderFailureBudgetExhausted => "provider_failure_budget_exhausted",
            Self::InvocationBudgetExhausted => "invocation_budget_exhausted",
            Self::OperationBudgetExhausted => "operation_budget_exhausted",
            Self::ContextBudgetExhausted => "context_budget_exhausted",
            Self::CostBudgetExhausted => "cost_budget_exhausted",
            Self::OperatorStopped => "operator_stopped",
            Self::MalformedProviderResult => "malformed_provider_result",
            Self::FatalInvariantViolation => "fatal_invariant_violation",
            Self::NormativeUnconfigured => "normative_unconfigured",
            Self::NormativeBlocked => "normative_blocked",
            Self::PolicyNotYetValid => "policy_not_yet_valid",
            Self::PolicyRefreshRequired => "policy_refresh_required",
            Self::PolicyStale => "policy_stale",
            Self::PolicyExpired => "policy_expired",
            Self::PolicyRevoked => "policy_revoked",
            Self::PolicyValidityUnavailable => "policy_validity_unavailable",
            Self::Cancelled => "cancelled",
            Self::Closed => "closed",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CaseRuntimeCheckpoint {
    schema: String,
    run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    runtime_instance_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    work_item_id: Option<String>,
    case_id: String,
    participant_id: String,
    attachment_id: String,
    journal_path: String,
    task: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    input_turn_id: Option<String>,
    status: CaseRuntimeStop,
    stop_detail: String,
    stop_requested: bool,
    invocations: usize,
    operations: usize,
    provider_failures: usize,
    cumulative_estimated_input_units: usize,
    actual_input_tokens: Option<u64>,
    actual_output_tokens: Option<u64>,
    actual_total_tokens: Option<u64>,
    cumulative_provider_latency_ms: u64,
    max_invocations: usize,
    max_operations: usize,
    max_semantic_units: usize,
    max_resident_items: usize,
    max_cumulative_estimated_input_units: usize,
    max_provider_retries: usize,
    max_runtime_ms: Option<u64>,
    stop_on_deny: bool,
    continue_after_malformed: bool,
    previous_item_ids: Vec<String>,
    last_residency_plan_id: Option<String>,
    last_projection_id: Option<String>,
    last_context_frame_id: Option<String>,
    last_projection_selected_items: usize,
    last_projection_omitted_items: usize,
    last_semantic_units: usize,
    last_provider_result_id: Option<String>,
    pending_provider_result_id: Option<String>,
    last_operation_id: Option<String>,
    last_decision_id: Option<String>,
    #[serde(default)]
    last_review_id: Option<String>,
    last_effect_id: Option<String>,
    last_receipt_id: Option<String>,
    last_effect_outcome: Option<String>,
}

#[derive(Clone, Debug)]
pub(super) struct CaseRuntimeReport {
    pub case_id: String,
    pub run_id: String,
    pub work_item_id: Option<String>,
    pub status: CaseRuntimeStop,
    pub detail: String,
}

impl From<&CaseRuntimeCheckpoint> for CaseRuntimeReport {
    fn from(checkpoint: &CaseRuntimeCheckpoint) -> Self {
        Self {
            case_id: checkpoint.case_id.clone(),
            run_id: checkpoint.run_id.clone(),
            work_item_id: checkpoint.work_item_id.clone(),
            status: checkpoint.status.clone(),
            detail: checkpoint.stop_detail.clone(),
        }
    }
}

impl CaseRuntimeCheckpoint {
    fn stop(&mut self, status: CaseRuntimeStop, detail: impl Into<String>) {
        self.status = status;
        self.stop_detail = detail.into();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CheckpointResumeIntent {
    DirectOperator,
    RuntimeWork,
}

fn checkpoint_is_never_resumable(status: &CaseRuntimeStop) -> bool {
    matches!(
        status,
        CaseRuntimeStop::Completed
            | CaseRuntimeStop::Denied
            | CaseRuntimeStop::Cancelled
            | CaseRuntimeStop::Closed
            | CaseRuntimeStop::FatalInvariantViolation
            | CaseRuntimeStop::DeliveryIndeterminate
    )
}

fn authorize_checkpoint_resume(
    checkpoint: &mut CaseRuntimeCheckpoint,
    intent: CheckpointResumeIntent,
) -> Result<bool, String> {
    if checkpoint_is_never_resumable(&checkpoint.status) {
        return match intent {
            CheckpointResumeIntent::DirectOperator => Err(format!(
                "case_runtime_terminal_checkpoint_cannot_resume: {}",
                checkpoint.status.as_str()
            )),
            CheckpointResumeIntent::RuntimeWork => Ok(false),
        };
    }
    if intent == CheckpointResumeIntent::RuntimeWork
        && matches!(
            checkpoint.status,
            CaseRuntimeStop::ProviderFailureBudgetExhausted
                | CaseRuntimeStop::InvocationBudgetExhausted
                | CaseRuntimeStop::OperationBudgetExhausted
                | CaseRuntimeStop::ContextBudgetExhausted
                | CaseRuntimeStop::CostBudgetExhausted
                | CaseRuntimeStop::OperatorStopped
                | CaseRuntimeStop::MalformedProviderResult
        )
    {
        return Ok(false);
    }
    checkpoint.stop_requested = false;
    checkpoint.status = CaseRuntimeStop::Running;
    checkpoint.stop_detail.clear();
    Ok(true)
}

fn validate_checkpoint_work_identity(
    checkpoint: &CaseRuntimeCheckpoint,
    item: &RuntimeWorkItem,
) -> Result<(), String> {
    if checkpoint.work_item_id.as_deref() != Some(item.work_id.as_str()) {
        return Err("case_runtime_checkpoint_owned_by_other_work".to_string());
    }
    if checkpoint.runtime_instance_id.as_deref() != item.runtime_instance_id.as_deref() {
        return Err("case_runtime_checkpoint_instance_mismatch".to_string());
    }
    Ok(())
}

fn parse_positive(args: &[String], name: &str, default: usize) -> Result<usize, String> {
    optional_arg(args, name)
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|error| format!("invalid {name}: {error}"))
                .and_then(|value| {
                    (value > 0)
                        .then_some(value)
                        .ok_or_else(|| format!("{name} must be positive"))
                })
        })
        .transpose()
        .map(|value| value.unwrap_or(default))
}

fn parse_optional_u64(args: &[String], name: &str) -> Result<Option<u64>, String> {
    optional_arg(args, name)
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|error| format!("invalid {name}: {error}"))
                .and_then(|value| {
                    (value > 0)
                        .then_some(value)
                        .ok_or_else(|| format!("{name} must be positive"))
                })
        })
        .transpose()
}

fn checkpoint_path(case_id: &str) -> PathBuf {
    yai_home().join("run").join("case-runtime").join(format!(
        "{}.json",
        yai_core_engine::context::stable_digest(case_id)
    ))
}

fn write_checkpoint(checkpoint: &CaseRuntimeCheckpoint) -> Result<(), String> {
    let path = checkpoint_path(&checkpoint.case_id);
    write_checkpoint_at(&path, checkpoint)
}

fn write_checkpoint_at(path: &Path, checkpoint: &CaseRuntimeCheckpoint) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "case runtime checkpoint has no parent".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("failed to create case runtime directory: {error}"))?;
    let temporary = parent.join(format!(
        ".{}.{}.tmp",
        yai_core_engine::context::stable_digest(&checkpoint.run_id),
        std::process::id()
    ));
    let encoded = serde_json::to_vec_pretty(checkpoint)
        .map_err(|error| format!("failed to encode case runtime checkpoint: {error}"))?;
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|error| format!("failed to create case runtime checkpoint: {error}"))?;
    file.write_all(&encoded)
        .map_err(|error| format!("failed to write case runtime checkpoint: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("failed to sync case runtime checkpoint: {error}"))?;
    drop(file);
    fs::rename(&temporary, path)
        .map_err(|error| format!("failed to publish case runtime checkpoint: {error}"))?;
    fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| format!("failed to sync case runtime directory: {error}"))?;
    Ok(())
}

fn read_checkpoint(case_id: &str) -> Result<CaseRuntimeCheckpoint, String> {
    let path = checkpoint_path(case_id);
    read_checkpoint_at(&path, case_id)
}

fn read_checkpoint_at(path: &Path, case_id: &str) -> Result<CaseRuntimeCheckpoint, String> {
    let encoded =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let checkpoint: CaseRuntimeCheckpoint = serde_json::from_slice(&encoded)
        .map_err(|error| format!("invalid case runtime checkpoint: {error}"))?;
    if (checkpoint.schema != CASE_RUNTIME_CHECKPOINT_SCHEMA
        && checkpoint.schema != CASE_RUNTIME_CHECKPOINT_SCHEMA_V2
        && checkpoint.schema != CASE_RUNTIME_CHECKPOINT_SCHEMA_V1)
        || checkpoint.case_id != case_id
    {
        return Err("case_runtime_checkpoint_identity_or_schema_mismatch".to_string());
    }
    Ok(checkpoint)
}

fn initial_checkpoint(args: &[String]) -> Result<CaseRuntimeCheckpoint, String> {
    let journal_path = case_journal_path(args, "yai case run")?;
    initial_checkpoint_with_journal(args, journal_path, None, None)
}

fn initial_checkpoint_with_journal(
    args: &[String],
    journal_path: PathBuf,
    runtime_instance_id: Option<String>,
    work_item_id: Option<String>,
) -> Result<CaseRuntimeCheckpoint, String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--subject")?;
    let attachment_id = named_arg(args, "--attachment")?;
    let prompt = optional_arg(args, "--prompt");
    let input_turn_id = optional_arg(args, "--input-turn");
    if prompt.is_some() == input_turn_id.is_some() {
        return Err("case_run_requires_exactly_one_of_prompt_or_input_turn".to_string());
    }
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store
        .get_case_state(&case_id)?
        .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
    if !state
        .participants
        .iter()
        .any(|participant| participant.participant_id == participant_id)
    {
        return Err(format!(
            "participant {participant_id} is not bound to {case_id}"
        ));
    }
    let mut resolved_input_turn_id = None;
    let task = if let Some(turn_id) = input_turn_id.as_deref() {
        let transitions = store.list_case_transitions(&case_id)?;
        let turn = if turn_id == "latest" {
            yai_core_engine::conversation::turns_from_history(&case_id, &transitions)
                .into_iter()
                .rev()
                .find(|turn| turn.participant_id == participant_id)
        } else {
            find_turn(&case_id, turn_id, &transitions)
        }
        .ok_or_else(|| "case_run_input_conversation_turn_not_found".to_string())?;
        if turn.participant_id != participant_id {
            return Err("case_run_input_conversation_turn_participant_mismatch".to_string());
        }
        resolved_input_turn_id = Some(turn.turn_id.clone());
        let content_store = ConversationContentStore::open(&yai_home())?;
        let mut text = Vec::new();
        for part in &turn.ordered_parts {
            content_store.verify_object(&part.object)?;
            if part.object.modality != ContentModality::Text {
                return Err("conversation_turn_requires_typed_media_provider_adapter".to_string());
            }
            text.push(content_store.read_text(&part.object)?);
        }
        text.join("\n\n")
    } else {
        prompt.expect("exclusive prompt contract checked")
    };
    if !state
        .resources
        .iter()
        .any(|resource| resource.attachment_id == attachment_id)
    {
        return Err(format!(
            "attachment {attachment_id} is not bound to {case_id}"
        ));
    }
    let run_material = format!(
        "{case_id}:{}:{}:{}",
        state.generation,
        runtime_now_millis(),
        work_item_id.as_deref().unwrap_or("direct")
    );
    Ok(CaseRuntimeCheckpoint {
        schema: CASE_RUNTIME_CHECKPOINT_SCHEMA.to_string(),
        run_id: format!(
            "case-run:{}",
            yai_core_engine::context::stable_digest(&run_material)
        ),
        runtime_instance_id,
        work_item_id,
        case_id,
        participant_id,
        attachment_id,
        journal_path: journal_path.display().to_string(),
        task,
        // Persist the exact identity, never the moving `latest` alias.
        input_turn_id: resolved_input_turn_id,
        status: CaseRuntimeStop::Running,
        stop_detail: String::new(),
        stop_requested: false,
        invocations: 0,
        operations: 0,
        provider_failures: 0,
        cumulative_estimated_input_units: 0,
        actual_input_tokens: Some(0),
        actual_output_tokens: Some(0),
        actual_total_tokens: Some(0),
        cumulative_provider_latency_ms: 0,
        max_invocations: parse_positive(args, "--max-invocations", 8)?,
        max_operations: parse_positive(args, "--max-operations", 8)?,
        max_semantic_units: parse_positive(
            args,
            "--max-semantic-units",
            DEFAULT_SEMANTIC_UNIT_BUDGET,
        )?,
        max_resident_items: parse_positive(
            args,
            "--max-resident-items",
            DEFAULT_MAX_RESIDENT_ITEMS,
        )?,
        max_cumulative_estimated_input_units: parse_positive(
            args,
            "--max-estimated-input-units",
            DEFAULT_SEMANTIC_UNIT_BUDGET * 16,
        )?,
        max_provider_retries: optional_arg(args, "--max-provider-retries")
            .map(|value| {
                value
                    .parse::<usize>()
                    .map_err(|error| format!("invalid --max-provider-retries: {error}"))
            })
            .transpose()?
            .unwrap_or(1),
        max_runtime_ms: parse_optional_u64(args, "--max-runtime-ms")?,
        stop_on_deny: args.iter().any(|value| value == "--stop-on-deny"),
        continue_after_malformed: args
            .iter()
            .any(|value| value == "--continue-after-malformed"),
        previous_item_ids: Vec::new(),
        last_residency_plan_id: None,
        last_projection_id: None,
        last_context_frame_id: None,
        last_projection_selected_items: 0,
        last_projection_omitted_items: 0,
        last_semantic_units: 0,
        last_provider_result_id: None,
        pending_provider_result_id: None,
        last_operation_id: None,
        last_decision_id: None,
        last_review_id: None,
        last_effect_id: None,
        last_receipt_id: None,
        last_effect_outcome: None,
    })
}

fn update_resume_budgets(
    checkpoint: &mut CaseRuntimeCheckpoint,
    args: &[String],
) -> Result<(), String> {
    if let Some(value) = optional_arg(args, "--max-invocations") {
        checkpoint.max_invocations = value
            .parse()
            .map_err(|error| format!("invalid --max-invocations: {error}"))?;
    }
    if let Some(value) = optional_arg(args, "--max-operations") {
        checkpoint.max_operations = value
            .parse()
            .map_err(|error| format!("invalid --max-operations: {error}"))?;
    }
    if let Some(value) = optional_arg(args, "--max-semantic-units") {
        checkpoint.max_semantic_units = value
            .parse()
            .map_err(|error| format!("invalid --max-semantic-units: {error}"))?;
    }
    if let Some(value) = optional_arg(args, "--max-estimated-input-units") {
        checkpoint.max_cumulative_estimated_input_units = value
            .parse()
            .map_err(|error| format!("invalid --max-estimated-input-units: {error}"))?;
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub(super) struct RuntimeProviderRoute {
    pub(super) args: Vec<String>,
    pub(super) selection: Option<ProviderSelection>,
}

#[allow(clippy::too_many_arguments)]
fn provider_route(
    checkpoint: &CaseRuntimeCheckpoint,
    plan_patch: bool,
    logical_turn_id: &str,
    attempt_number: u32,
    attempted_targets: &BTreeSet<String>,
    prior_attempt_retry_safe: bool,
) -> Result<Option<RuntimeProviderRoute>, String> {
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store
        .get_case_state(&checkpoint.case_id)?
        .ok_or_else(|| format!("canonical CaseState missing for {}", checkpoint.case_id))?;
    if state.provider_binding.is_some() {
        let requirement = if plan_patch {
            ProviderRequirement::plan_patch()?
        } else {
            ProviderRequirement::text("case_runtime_turn")?
        };
        let route = governed_provider_route_for_attempt(
            &checkpoint.case_id,
            &checkpoint.participant_id,
            &requirement,
            logical_turn_id,
            attempt_number,
            attempted_targets,
            prior_attempt_retry_safe,
        )?;
        return Ok(Some(RuntimeProviderRoute {
            args: route.args,
            selection: Some(route.selection),
        }));
    }
    let Some(provider) = state.provider else {
        return Ok(None);
    };
    let mut args = vec![
        "--case".to_string(),
        checkpoint.case_id.clone(),
        "--subject".to_string(),
        checkpoint.participant_id.clone(),
        "--base-url".to_string(),
        provider.base_url,
        "--provider-id".to_string(),
        provider.provider_id,
        "--model".to_string(),
        provider.model_id,
    ];
    if let Some(environment) = provider.credential_ref.strip_prefix("env:") {
        args.push("--api-key-env".to_string());
        args.push(environment.to_string());
    }
    Ok(Some(RuntimeProviderRoute {
        args,
        selection: None,
    }))
}

fn ensure_memory_fresh(case_id: &str) -> Result<usize, String> {
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store
        .get_case_state(case_id)?
        .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
    if store
        .operational_memory_manifest(case_id)?
        .is_some_and(|manifest| manifest.is_current(case_id, state.generation))
    {
        return store
            .list_operational_memory(case_id)
            .map(|entries| entries.len());
    }
    let transitions = store.list_case_transitions(case_id)?;
    let build = derive_operational_memory(case_id, &transitions)?;
    let count = build.entries.len();
    store.replace_case_operational_memory(&build)?;
    Ok(count)
}

fn add_usage(total: &mut Option<u64>, value: Option<u64>) {
    match (total.as_mut(), value) {
        (Some(total), Some(value)) => *total = total.saturating_add(value),
        (_, None) => *total = None,
        (None, Some(_)) => {}
    }
}

fn completion_response(output: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(output)
        .ok()
        .is_some_and(|value| {
            value.get("schema").and_then(|value| value.as_str()) == Some(CASE_RUNTIME_OUTPUT_SCHEMA)
                && value.get("outcome").and_then(|value| value.as_str()) == Some("complete")
        })
}

fn load_canonical_provider_result(
    case_id: &str,
    result_id: &str,
) -> Result<ControlledProviderResult, String> {
    let store = LmdbRecordStore::open(record_store_path())?;
    let transitions = store.list_case_transitions(case_id)?;
    transitions
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::ProviderResultRecorded {
                result_id: stored_result_id,
                invocation_id,
                provider_id,
                model_id,
                semantic_lineage,
                output,
                ..
            } if stored_result_id == result_id => Some(ControlledProviderResult {
                invocation_id: invocation_id.clone(),
                result_id: stored_result_id.clone(),
                raw_output: output.clone(),
                provider_id: provider_id.clone(),
                model_id: model_id.clone(),
                projection_id: semantic_lineage
                    .as_ref()
                    .map(|lineage| lineage.projection_id.clone())
                    .unwrap_or_default(),
                context_frame_id: semantic_lineage
                    .as_ref()
                    .map(|lineage| lineage.context_frame_id.clone())
                    .unwrap_or_default(),
                residency_plan_id: String::new(),
                resident_item_ids: Vec::new(),
                projection_selected_items: 0,
                projection_omitted_items: 0,
                semantic_units: 0,
                estimated_input_units: 0,
                usage: ProviderUsageTelemetry::default(),
                request_bytes_written: 0,
            }),
            _ => None,
        })
        .ok_or_else(|| format!("canonical ProviderResult not found: {result_id}"))
}

fn current_workflow_execution_id(
    checkpoint: &CaseRuntimeCheckpoint,
) -> Result<Option<String>, String> {
    let Some(work_item_id) = checkpoint.work_item_id.as_deref() else {
        return Ok(None);
    };
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let items = store.list_runtime_work_authorized(&authenticated)?;
    let item = items
        .into_iter()
        .find(|item| item.work_id == work_item_id)
        .ok_or_else(|| "runtime_work_item_missing_for_checkpoint".to_string())?;
    if item.case_id != checkpoint.case_id {
        return Err("runtime_work_checkpoint_case_mismatch".to_string());
    }
    Ok(item.workflow.map(|workflow| workflow.workflow_execution_id))
}

fn current_workflow_plan_patch_contract(
    checkpoint: &CaseRuntimeCheckpoint,
) -> Result<Option<(String, InvocationOutputContract)>, String> {
    let Some(execution_id) = current_workflow_execution_id(checkpoint)? else {
        return Ok(None);
    };
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let (contract, topology_digest) = store.workflow_model_output_contract_authorized(
        &authenticated,
        &checkpoint.case_id,
        &execution_id,
    )?;
    if contract != yai_core_engine::workflow::ModelWorkOutputContract::PlanPatch {
        return Ok(None);
    }
    Ok(Some((
        execution_id,
        InvocationOutputContract::WorkflowPlanPatch {
            schema: yai_core_engine::workflow::WORKFLOW_PLAN_PATCH_SCHEMA.to_string(),
            base_effective_topology_digest: topology_digest,
            max_operations: yai_core_engine::workflow::MAX_WORKFLOW_PATCH_OPERATIONS,
        },
    )))
}

fn advance_and_check_workflow_completion(
    case_id: &str,
    workflow_execution_id: &str,
) -> Result<bool, String> {
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let resolution = store.advance_workflow_passive_progress(&authenticated, case_id, 128)?;
    Ok(resolution.nodes.iter().any(|node| {
        node.execution_id.as_deref() == Some(workflow_execution_id)
            && node.posture == yai_core_engine::workflow::WorkflowNodePosture::Satisfied
    }))
}

struct RuntimeAdmissionOwner {
    case_id: String,
    run_id: String,
    owner_token: String,
    owner_pid: u32,
}

#[cfg(unix)]
fn local_process_alive(pid: u32) -> bool {
    unsafe { kill(pid as c_int, 0) == 0 }
}

#[cfg(not(unix))]
fn local_process_alive(_pid: u32) -> bool {
    true
}

fn runtime_admission_request(owner: &RuntimeAdmissionOwner) -> CaseRuntimeAdmissionRequest {
    CaseRuntimeAdmissionRequest {
        case_id: owner.case_id.clone(),
        run_id: owner.run_id.clone(),
        owner_token: owner.owner_token.clone(),
        owner_pid: owner.owner_pid,
        now_unix_ms: runtime_now_millis().min(u64::MAX as u128) as u64,
        lease_duration_ms: CASE_RUNTIME_ADMISSION_TTL_MS,
    }
}

fn acquire_runtime_admission(
    checkpoint: &CaseRuntimeCheckpoint,
) -> Result<RuntimeAdmissionOwner, String> {
    let store = LmdbRecordStore::open(record_store_path())?;
    let existing = store.get_case_runtime_admission(&checkpoint.case_id)?;
    let allow_reclaim = existing.as_ref().is_some_and(|admission| {
        admission.expires_at_unix_ms <= runtime_now_millis().min(u64::MAX as u128) as u64
            || !local_process_alive(admission.owner_pid)
    });
    let owner_pid = std::process::id();
    let token_material = format!(
        "{}:{}:{}:{}",
        checkpoint.case_id,
        checkpoint.run_id,
        owner_pid,
        runtime_now_millis()
    );
    let owner = RuntimeAdmissionOwner {
        case_id: checkpoint.case_id.clone(),
        run_id: checkpoint.run_id.clone(),
        owner_token: format!(
            "run-owner:{}",
            yai_core_engine::context::stable_digest(&token_material)
        ),
        owner_pid,
    };
    let (outcome, admission) =
        store.acquire_case_runtime_admission(&runtime_admission_request(&owner), allow_reclaim)?;
    println!(
        "runtime_admission: {}",
        match outcome {
            CaseRuntimeAdmissionOutcome::Acquired => "acquired",
            CaseRuntimeAdmissionOutcome::Renewed => "renewed",
            CaseRuntimeAdmissionOutcome::Reclaimed => "reclaimed_stale",
        }
    );
    println!("runtime_admission_owner_pid: {}", admission.owner_pid);
    Ok(owner)
}

fn renew_runtime_admission(owner: &RuntimeAdmissionOwner) -> Result<(), String> {
    let store = LmdbRecordStore::open(record_store_path())?;
    let (outcome, _) =
        store.acquire_case_runtime_admission(&runtime_admission_request(owner), false)?;
    if outcome != CaseRuntimeAdmissionOutcome::Renewed {
        return Err("case_runtime_admission_lost".to_string());
    }
    Ok(())
}

fn release_runtime_admission(owner: &RuntimeAdmissionOwner) -> Result<(), String> {
    LmdbRecordStore::open(record_store_path())?
        .release_case_runtime_admission(&owner.case_id, &owner.run_id, &owner.owner_token)
        .map(|_| ())
}

fn run_with_admission(
    checkpoint: CaseRuntimeCheckpoint,
    args: &[String],
) -> Result<CaseRuntimeCheckpoint, String> {
    let owner = acquire_runtime_admission(&checkpoint)?;
    let result = run_loop(checkpoint, args, &owner);
    let release = release_runtime_admission(&owner);
    match (result, release) {
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Ok(checkpoint), Ok(())) => Ok(checkpoint),
    }
}

fn run_loop(
    mut checkpoint: CaseRuntimeCheckpoint,
    args: &[String],
    admission: &RuntimeAdmissionOwner,
) -> Result<CaseRuntimeCheckpoint, String> {
    let started = Instant::now();
    write_checkpoint(&checkpoint)?;
    loop {
        renew_runtime_admission(admission)?;
        checkpoint = read_checkpoint(&checkpoint.case_id)?;
        if checkpoint.stop_requested {
            checkpoint.stop(
                CaseRuntimeStop::OperatorStopped,
                "operator requested stop between governed iterations",
            );
            write_checkpoint(&checkpoint)?;
            break;
        }
        if checkpoint.invocations >= checkpoint.max_invocations {
            checkpoint.stop(
                CaseRuntimeStop::InvocationBudgetExhausted,
                "no provider invocation issued after budget exhaustion",
            );
            write_checkpoint(&checkpoint)?;
            break;
        }
        if checkpoint.operations >= checkpoint.max_operations {
            checkpoint.stop(
                CaseRuntimeStop::OperationBudgetExhausted,
                "no provider invocation or effect issued after operation budget exhaustion",
            );
            write_checkpoint(&checkpoint)?;
            break;
        }
        if checkpoint.cumulative_estimated_input_units
            >= checkpoint.max_cumulative_estimated_input_units
        {
            checkpoint.stop(
                CaseRuntimeStop::CostBudgetExhausted,
                "estimated provider input budget exhausted before invocation",
            );
            write_checkpoint(&checkpoint)?;
            break;
        }
        if checkpoint
            .max_runtime_ms
            .is_some_and(|limit| started.elapsed() >= Duration::from_millis(limit))
        {
            checkpoint.stop(
                CaseRuntimeStop::OperatorStopped,
                "runtime wall-clock bound reached",
            );
            write_checkpoint(&checkpoint)?;
            break;
        }

        match reconcile_case_before_invocation(&checkpoint.case_id, true)? {
            CaseReconciliationStatus::Unresolved { effect_ids } => {
                checkpoint.stop(
                    CaseRuntimeStop::IndeterminateEffect,
                    format!("unresolved effects: {}", effect_ids.join(",")),
                );
                write_checkpoint(&checkpoint)?;
                break;
            }
            CaseReconciliationStatus::Clean | CaseReconciliationStatus::Reconciled { .. } => {}
        }
        let store = LmdbRecordStore::open(record_store_path())?;
        let state = store
            .get_case_state(&checkpoint.case_id)?
            .ok_or_else(|| format!("canonical CaseState missing for {}", checkpoint.case_id))?;
        if state.lifecycle == CaseLifecycle::Closed {
            checkpoint.stop(
                CaseRuntimeStop::Closed,
                "Case is durably closed; provider was not invoked",
            );
            write_checkpoint(&checkpoint)?;
            break;
        }
        if state.cancellation.is_some() {
            checkpoint.stop(
                CaseRuntimeStop::Cancelled,
                "Case has a durable cancellation barrier; provider was not invoked",
            );
            write_checkpoint(&checkpoint)?;
            break;
        }
        let normative = store.case_policy_status(&checkpoint.case_id)?;
        match normative.readiness {
            NormativeReadiness::Ready => {}
            NormativeReadiness::Unconfigured => {
                checkpoint.stop(
                    CaseRuntimeStop::NormativeUnconfigured,
                    "Case has no exact published policy binding; provider was not invoked",
                );
                write_checkpoint(&checkpoint)?;
                break;
            }
            NormativeReadiness::Blocked => {
                checkpoint.stop(
                    CaseRuntimeStop::NormativeBlocked,
                    format!(
                        "normative materialization blocked: missing={} conflicts={}",
                        normative.missing.join(","),
                        normative.blocking_conflicts.join(",")
                    ),
                );
                write_checkpoint(&checkpoint)?;
                break;
            }
        }
        let temporal_stop = match normative.validity {
            PolicyValidityPosture::Valid => None,
            PolicyValidityPosture::NotYetValid => Some((
                CaseRuntimeStop::PolicyNotYetValid,
                "Case policy is not yet valid",
            )),
            PolicyValidityPosture::RefreshRequired => Some((
                CaseRuntimeStop::PolicyRefreshRequired,
                "Case policy requires explicit replacement/refresh",
            )),
            PolicyValidityPosture::Stale => Some((
                CaseRuntimeStop::PolicyStale,
                "Case is pinned to stale policy material",
            )),
            PolicyValidityPosture::Expired => {
                Some((CaseRuntimeStop::PolicyExpired, "Case policy has expired"))
            }
            PolicyValidityPosture::Revoked => Some((
                CaseRuntimeStop::PolicyRevoked,
                "Case policy has been revoked",
            )),
            PolicyValidityPosture::Unavailable => Some((
                CaseRuntimeStop::PolicyValidityUnavailable,
                "Case policy validity cannot be established",
            )),
        };
        if let Some((stop, detail)) = temporal_stop {
            checkpoint.stop(stop, format!("{detail}; provider was not invoked"));
            write_checkpoint(&checkpoint)?;
            break;
        }
        if let Err(error) = ensure_memory_fresh(&checkpoint.case_id) {
            eprintln!("runtime_memory_fallback_to_canonical: {error}");
        }

        let provider_result = if let Some(result_id) = checkpoint.pending_provider_result_id.clone()
        {
            load_canonical_provider_result(&checkpoint.case_id, &result_id)?
        } else {
            let remaining_cost = checkpoint
                .max_cumulative_estimated_input_units
                .saturating_sub(checkpoint.cumulative_estimated_input_units);
            let plan_patch_contract = current_workflow_plan_patch_contract(&checkpoint)?;
            let options = SemanticInvocationOptions {
                max_resident_items: checkpoint.max_resident_items,
                max_semantic_units: checkpoint.max_semantic_units,
                max_estimated_input_units: remaining_cost,
                retrieval_limit: checkpoint.max_resident_items.saturating_mul(4).max(1),
                previous_item_ids: checkpoint.previous_item_ids.clone(),
                workflow_execution_id: current_workflow_execution_id(&checkpoint)?,
                conversation_turn_id: checkpoint.input_turn_id.clone(),
            };
            let store = LmdbRecordStore::open(record_store_path())?;
            let state = store.get_case_state(&checkpoint.case_id)?.ok_or_else(|| {
                "canonical CaseState missing before runtime invocation".to_string()
            })?;
            let resource = state
                .resources
                .iter()
                .find(|resource| resource.attachment_id == checkpoint.attachment_id)
                .ok_or_else(|| "runtime resource attachment missing".to_string())?;
            let logical_turn_id = format!(
                "provider-turn:{}:{}:{}",
                yai_core_engine::context::stable_digest(&checkpoint.case_id),
                yai_core_engine::context::stable_digest(&checkpoint.run_id),
                checkpoint.invocations + 1
            );
            let mut attempt = 0usize;
            let mut attempted_targets = BTreeSet::new();
            let mut prior_attempt_retry_safe = true;
            let result = loop {
                let (purpose, output_contract) = match &plan_patch_contract {
                    Some((_, contract)) => (
                        ProjectionPurpose::WorkflowPlanPatchProposal,
                        contract.clone(),
                    ),
                    None => (
                        ProjectionPurpose::FilesystemWriteProposal,
                        InvocationOutputContract::CaseRuntimeTurn {
                            schema: CASE_RUNTIME_OUTPUT_SCHEMA.to_string(),
                            operation_schema: OPERATION_PROPOSAL_SCHEMA.to_string(),
                            attachment_id: resource.attachment_id.clone(),
                            allowed_write_prefix: resource.allowed_write_prefix.clone(),
                            max_write_bytes: resource.max_write_bytes,
                        },
                    ),
                };
                let route = match provider_route(
                    &checkpoint,
                    plan_patch_contract.is_some(),
                    &logical_turn_id,
                    (attempt + 1).try_into().unwrap_or(u32::MAX),
                    &attempted_targets,
                    prior_attempt_retry_safe,
                ) {
                    Ok(Some(route)) => route,
                    Ok(None) => {
                        checkpoint.stop(
                            CaseRuntimeStop::ProviderFailureBudgetExhausted,
                            "Case has no attached or governed provider",
                        );
                        write_checkpoint(&checkpoint)?;
                        return Ok(checkpoint);
                    }
                    Err(error) if error.starts_with("provider_waiting:") => {
                        checkpoint.stop(CaseRuntimeStop::WaitingProvider, error);
                        write_checkpoint(&checkpoint)?;
                        return Ok(checkpoint);
                    }
                    Err(error) => return Err(error),
                };
                match invoke_semantic_provider_with_journal(
                    &route.args,
                    purpose,
                    &checkpoint.task,
                    output_contract,
                    &options,
                    Path::new(&checkpoint.journal_path),
                ) {
                    Ok(result) => {
                        if let Some(selection) = &route.selection {
                            let outcome = governed_attempt_outcome(
                                selection,
                                None,
                                Some(result.request_bytes_written),
                            )?;
                            let authenticated = authenticate_local()?;
                            let store = LmdbRecordStore::open(record_store_path())?;
                            let outcome_id = outcome.outcome_id.clone();
                            store.record_provider_attempt_outcome_authorized(
                                &authenticated,
                                &checkpoint.case_id,
                                outcome,
                            )?;
                            store.record_provider_attempt_health_authorized(
                                &authenticated,
                                &checkpoint.case_id,
                                &outcome_id,
                            )?;
                        }
                        break result;
                    }
                    Err(error) if error.contains("residency_budget_below_mandatory_state") => {
                        checkpoint.stop(CaseRuntimeStop::ContextBudgetExhausted, error);
                        write_checkpoint(&checkpoint)?;
                        return Ok(checkpoint);
                    }
                    Err(error) if error.contains("provider_input_budget_exceeded") => {
                        checkpoint.stop(CaseRuntimeStop::CostBudgetExhausted, error);
                        write_checkpoint(&checkpoint)?;
                        return Ok(checkpoint);
                    }
                    Err(error) => {
                        checkpoint.provider_failures += 1;
                        if let Some(selection) = &route.selection {
                            let outcome = governed_attempt_outcome(selection, Some(&error), None)?;
                            let retry_safe = outcome.retry_safe();
                            let delivery_indeterminate =
                                outcome.delivery == ProviderDeliveryClass::DeliveryIndeterminate;
                            let authenticated = authenticate_local()?;
                            let store = LmdbRecordStore::open(record_store_path())?;
                            let outcome_id = outcome.outcome_id.clone();
                            store.record_provider_attempt_outcome_authorized(
                                &authenticated,
                                &checkpoint.case_id,
                                outcome,
                            )?;
                            store.record_provider_attempt_health_authorized(
                                &authenticated,
                                &checkpoint.case_id,
                                &outcome_id,
                            )?;
                            attempted_targets.insert(selection.selected_target_id.clone());
                            prior_attempt_retry_safe = retry_safe;
                            if delivery_indeterminate {
                                checkpoint.stop(CaseRuntimeStop::DeliveryIndeterminate, error);
                                write_checkpoint(&checkpoint)?;
                                return Ok(checkpoint);
                            }
                            if retry_safe && attempt < checkpoint.max_provider_retries {
                                attempt += 1;
                                eprintln!(
                                    "provider_safe_failover: attempt={} reason:{}",
                                    attempt + 1,
                                    runtime_error_preview(&error)
                                );
                                continue;
                            }
                            checkpoint.stop(CaseRuntimeStop::ProviderFailureBudgetExhausted, error);
                            write_checkpoint(&checkpoint)?;
                            return Ok(checkpoint);
                        }
                        if attempt < checkpoint.max_provider_retries {
                            attempt += 1;
                            eprintln!(
                                "provider_retry: {attempt} reason:{}",
                                runtime_error_preview(&error)
                            );
                            continue;
                        }
                        checkpoint.stop(CaseRuntimeStop::ProviderFailureBudgetExhausted, error);
                        write_checkpoint(&checkpoint)?;
                        return Ok(checkpoint);
                    }
                }
            };
            checkpoint.invocations += 1;
            checkpoint.cumulative_estimated_input_units = checkpoint
                .cumulative_estimated_input_units
                .saturating_add(result.estimated_input_units);
            add_usage(
                &mut checkpoint.actual_input_tokens,
                result.usage.input_tokens,
            );
            add_usage(
                &mut checkpoint.actual_output_tokens,
                result.usage.output_tokens,
            );
            add_usage(
                &mut checkpoint.actual_total_tokens,
                result.usage.total_tokens,
            );
            checkpoint.cumulative_provider_latency_ms = checkpoint
                .cumulative_provider_latency_ms
                .saturating_add(result.usage.latency_ms);
            checkpoint.previous_item_ids = result.resident_item_ids.clone();
            checkpoint.last_residency_plan_id = Some(result.residency_plan_id.clone());
            checkpoint.last_projection_id = Some(result.projection_id.clone());
            checkpoint.last_context_frame_id = Some(result.context_frame_id.clone());
            checkpoint.last_projection_selected_items = result.projection_selected_items;
            checkpoint.last_projection_omitted_items = result.projection_omitted_items;
            checkpoint.last_semantic_units = result.semantic_units;
            checkpoint.last_provider_result_id = Some(result.result_id.clone());
            checkpoint.pending_provider_result_id = Some(result.result_id.clone());
            write_checkpoint(&checkpoint)?;
            if runtime_failpoint(args).as_deref() == Some("runtime_after_provider_result")
                && checkpoint.invocations == 1
            {
                exit_runtime_failpoint("runtime_after_provider_result", 91);
            }
            result
        };

        if let Some((workflow_execution_id, _)) = current_workflow_plan_patch_contract(&checkpoint)?
        {
            let authenticated = authenticate_local()?;
            let store = LmdbRecordStore::open(record_store_path())?;
            let proposal = store.propose_workflow_plan_patch_from_provider_result(
                &authenticated,
                &checkpoint.case_id,
                &provider_result.result_id,
                runtime_now_millis().min(u64::MAX as u128) as u64,
            );
            checkpoint.pending_provider_result_id = None;
            let satisfied =
                advance_and_check_workflow_completion(&checkpoint.case_id, &workflow_execution_id)?;
            match proposal {
                Ok(commit) if satisfied => checkpoint.stop(
                    CaseRuntimeStop::Completed,
                    format!(
                        "strict Workflow PlanPatch candidate recorded; patch_id={}",
                        commit
                            .state
                            .workflow_plan_patches
                            .last()
                            .map(|patch| patch.patch_id.as_str())
                            .unwrap_or("unknown")
                    ),
                ),
                Ok(_) => {
                    checkpoint.stop_detail =
                        "strict Workflow PlanPatch candidate recorded; completion predicate remains false"
                            .to_string();
                }
                Err(error) if error.starts_with("workflow_model_plan_patch_invalid") => {
                    if satisfied {
                        checkpoint.stop(
                            CaseRuntimeStop::Completed,
                            format!(
                                "ProviderResult satisfied Workflow but strict PlanPatch candidate was rejected: {error}"
                            ),
                        );
                    } else {
                        checkpoint.stop(CaseRuntimeStop::MalformedProviderResult, error);
                    }
                }
                Err(error) => return Err(error),
            }
            write_checkpoint(&checkpoint)?;
            if checkpoint.status != CaseRuntimeStop::Running {
                break;
            }
            continue;
        }

        if completion_response(&provider_result.raw_output) {
            checkpoint.pending_provider_result_id = None;
            let workflow_execution_id = current_workflow_execution_id(&checkpoint)?;
            if let Some(workflow_execution_id) = workflow_execution_id {
                if advance_and_check_workflow_completion(
                    &checkpoint.case_id,
                    &workflow_execution_id,
                )? {
                    checkpoint.stop(
                        CaseRuntimeStop::Completed,
                        "canonical workflow completion predicate satisfied",
                    );
                } else {
                    checkpoint.stop_detail =
                        "provider claimed completion but workflow predicate remains false"
                            .to_string();
                }
            } else {
                checkpoint.stop(
                    CaseRuntimeStop::Completed,
                    "provider returned typed completion",
                );
            }
            write_checkpoint(&checkpoint)?;
            if checkpoint.status != CaseRuntimeStop::Running {
                break;
            }
            continue;
        }
        let outcome = advance_controlled_filesystem_candidate(
            args,
            &checkpoint.case_id,
            &checkpoint.participant_id,
            &checkpoint.attachment_id,
            &provider_result,
        )?;
        if outcome.status != ControlledEffectTurnStatus::AwaitingReview {
            checkpoint.pending_provider_result_id = None;
        }
        checkpoint.last_operation_id = outcome.operation_id.clone();
        checkpoint.last_decision_id = outcome.decision_id.clone();
        checkpoint.last_review_id = outcome.review_id.clone();
        checkpoint.last_effect_id = outcome.effect_id.clone();
        checkpoint.last_receipt_id = outcome.receipt_id.clone();
        checkpoint.last_effect_outcome = outcome
            .outcome
            .as_ref()
            .map(|outcome| format!("{outcome:?}"));
        match outcome.status {
            ControlledEffectTurnStatus::NormalizationRejected => {
                if !checkpoint.continue_after_malformed {
                    checkpoint.stop(
                        CaseRuntimeStop::MalformedProviderResult,
                        "provider candidate could not be normalized",
                    );
                }
            }
            ControlledEffectTurnStatus::Denied => {
                if checkpoint.stop_on_deny {
                    checkpoint.stop(CaseRuntimeStop::Denied, "typed Decision denied operation");
                }
            }
            ControlledEffectTurnStatus::AwaitingReview => {
                checkpoint.stop(
                    CaseRuntimeStop::AwaitingReview,
                    format!(
                        "human participant action required for {}",
                        outcome.review_id.as_deref().unwrap_or("unknown_review")
                    ),
                );
            }
            ControlledEffectTurnStatus::Finalized => {
                checkpoint.operations += 1;
            }
            ControlledEffectTurnStatus::Indeterminate => {
                checkpoint.stop(
                    CaseRuntimeStop::IndeterminateEffect,
                    "effect outcome remains unresolved",
                );
            }
        }
        if checkpoint.status == CaseRuntimeStop::Running {
            let workflow_execution_id = current_workflow_execution_id(&checkpoint)?;
            if let Some(workflow_execution_id) = workflow_execution_id {
                if advance_and_check_workflow_completion(
                    &checkpoint.case_id,
                    &workflow_execution_id,
                )? {
                    checkpoint.stop(
                        CaseRuntimeStop::Completed,
                        "canonical workflow completion predicate satisfied",
                    );
                }
            }
        }
        write_checkpoint(&checkpoint)?;
        if runtime_failpoint(args).as_deref() == Some("runtime_after_finalized_before_memory")
            && outcome.status == ControlledEffectTurnStatus::Finalized
        {
            let store = LmdbRecordStore::open(record_store_path())?;
            store.clear_case_operational_memory(&checkpoint.case_id)?;
            exit_runtime_failpoint("runtime_after_finalized_before_memory", 92);
        }
        if checkpoint.status != CaseRuntimeStop::Running {
            break;
        }
        if runtime_failpoint(args).as_deref() == Some("runtime_between_iterations") {
            exit_runtime_failpoint("runtime_between_iterations", 93);
        }
    }
    Ok(checkpoint)
}

fn print_runtime_summary(checkpoint: &CaseRuntimeCheckpoint) -> Result<(), String> {
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store
        .get_case_state(&checkpoint.case_id)?
        .ok_or_else(|| "canonical CaseState missing for runtime summary".to_string())?;
    println!("case_runtime_schema: {}", checkpoint.schema);
    println!("run_id: {}", checkpoint.run_id);
    println!("case_id: {}", checkpoint.case_id);
    println!(
        "input_conversation_turn_id: {}",
        checkpoint
            .input_turn_id
            .as_deref()
            .unwrap_or("legacy_prompt")
    );
    println!("case_generation: {}", state.generation);
    println!("runtime_status: {:?}", checkpoint.status);
    println!("stop_detail: {}", checkpoint.stop_detail);
    println!("stop_requested: {}", checkpoint.stop_requested);
    println!("invocations: {}", checkpoint.invocations);
    println!("operations: {}", checkpoint.operations);
    println!("provider_failures: {}", checkpoint.provider_failures);
    println!(
        "estimated_input_units: {}",
        checkpoint.cumulative_estimated_input_units
    );
    println!(
        "actual_input_tokens: {}",
        checkpoint
            .actual_input_tokens
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unavailable".to_string())
    );
    println!(
        "actual_total_tokens: {}",
        checkpoint
            .actual_total_tokens
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unavailable".to_string())
    );
    println!(
        "residency_plan_id: {}",
        checkpoint
            .last_residency_plan_id
            .as_deref()
            .unwrap_or("none")
    );
    println!(
        "projection_id: {}",
        checkpoint.last_projection_id.as_deref().unwrap_or("none")
    );
    println!(
        "context_frame_id: {}",
        checkpoint
            .last_context_frame_id
            .as_deref()
            .unwrap_or("none")
    );
    println!("resident_items: {}", checkpoint.previous_item_ids.len());
    println!(
        "projection_selected_items: {}",
        checkpoint.last_projection_selected_items
    );
    println!(
        "projection_omitted_items: {}",
        checkpoint.last_projection_omitted_items
    );
    println!("semantic_units: {}", checkpoint.last_semantic_units);
    println!(
        "provider_latency_ms: {}",
        checkpoint.cumulative_provider_latency_ms
    );
    println!(
        "last_provider_result_id: {}",
        checkpoint
            .last_provider_result_id
            .as_deref()
            .unwrap_or("none")
    );
    println!(
        "last_effect_id: {}",
        checkpoint.last_effect_id.as_deref().unwrap_or("none")
    );
    println!(
        "last_review_id: {}",
        checkpoint.last_review_id.as_deref().unwrap_or("none")
    );
    println!(
        "last_effect_outcome: {}",
        checkpoint.last_effect_outcome.as_deref().unwrap_or("none")
    );
    if let Some(admission) = store.get_case_runtime_admission(&checkpoint.case_id)? {
        println!("runtime_admission_status: active");
        println!("runtime_admission_run_id: {}", admission.run_id);
        println!("runtime_admission_owner_pid: {}", admission.owner_pid);
        println!(
            "runtime_admission_expires_at_unix_ms: {}",
            admission.expires_at_unix_ms
        );
    } else {
        println!("runtime_admission_status: none");
    }
    Ok(())
}

pub(super) fn case_runtime_run(args: &[String]) -> Result<(), String> {
    let checkpoint = initial_checkpoint(args)?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let authenticated = authenticate_local()?;
    store
        .resolve_security_context(
            &authenticated,
            store
                .get_case_state_authorized(&authenticated, &checkpoint.case_id)?
                .tenant_id
                .as_deref()
                .ok_or_else(|| "legacy_unscoped_case_cannot_start_new_runtime".to_string())?,
        )?
        .require_owner()?;
    let path = checkpoint_path(&checkpoint.case_id);
    if path.exists() {
        let existing = read_checkpoint(&checkpoint.case_id)?;
        if existing.status == CaseRuntimeStop::Running {
            return Err(format!(
                "Case runtime already exists at {}; use `yai case resume`",
                path.display()
            ));
        }
    }
    let checkpoint = run_with_admission(checkpoint, args)?;
    print_runtime_summary(&checkpoint)
}

pub(super) fn case_runtime_resume(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let authenticated = authenticate_local()?;
    let state = store.get_case_state_authorized(&authenticated, &case_id)?;
    store
        .resolve_security_context(
            &authenticated,
            state
                .tenant_id
                .as_deref()
                .ok_or_else(|| "legacy_unscoped_case_cannot_resume_runtime".to_string())?,
        )?
        .require_owner()?;
    let mut checkpoint = read_checkpoint(&case_id)?;
    update_resume_budgets(&mut checkpoint, args)?;
    authorize_checkpoint_resume(&mut checkpoint, CheckpointResumeIntent::DirectOperator)?;
    let checkpoint = run_with_admission(checkpoint, args)?;
    print_runtime_summary(&checkpoint)
}

fn runtime_work_args(item: &RuntimeWorkItem) -> Vec<String> {
    let mut args = vec![
        "--case".to_string(),
        item.case_id.clone(),
        "--subject".to_string(),
        item.participant_id.clone(),
        "--attachment".to_string(),
        item.attachment_id.clone(),
        "--prompt".to_string(),
        item.task.clone(),
        "--max-invocations".to_string(),
        item.budgets.max_invocations.to_string(),
        "--max-operations".to_string(),
        item.budgets.max_operations.to_string(),
        "--max-semantic-units".to_string(),
        item.budgets.max_semantic_units.to_string(),
        "--max-resident-items".to_string(),
        item.budgets.max_resident_items.to_string(),
        "--max-estimated-input-units".to_string(),
        item.budgets.max_estimated_input_units.to_string(),
        "--max-provider-retries".to_string(),
        item.budgets.max_provider_retries.to_string(),
    ];
    if let Some(max_runtime_ms) = item.budgets.max_runtime_ms {
        args.push("--max-runtime-ms".to_string());
        args.push(max_runtime_ms.to_string());
    }
    if item.budgets.stop_on_deny {
        args.push("--stop-on-deny".to_string());
    }
    if item.budgets.continue_after_malformed {
        args.push("--continue-after-malformed".to_string());
    }
    if let Some(failpoint) = &item.failpoint {
        args.push("--failpoint".to_string());
        args.push(failpoint.clone());
    }
    args
}

/// Reusable execution boundary shared by direct Case commands and the
/// multi-Case RuntimeInstance. It is the only Case advancement algorithm.
pub(super) fn execute_runtime_work(item: &RuntimeWorkItem) -> Result<CaseRuntimeReport, String> {
    item.validate_integrity()?;
    if item
        .workflow
        .as_ref()
        .is_some_and(|workflow| workflow.workflow_node_kind == "deterministic_work")
    {
        return execute_deterministic_runtime_work(item);
    }
    let runtime_instance_id = item
        .runtime_instance_id
        .clone()
        .ok_or_else(|| "runtime_work_not_bound_to_instance".to_string())?;
    let args = runtime_work_args(item);
    let journal_path = PathBuf::from(&item.journal_path);
    let mut checkpoint = match read_checkpoint(&item.case_id) {
        Ok(mut existing) if existing.work_item_id.as_deref() == Some(item.work_id.as_str()) => {
            if existing.runtime_instance_id.as_deref() != Some(runtime_instance_id.as_str()) {
                return Err("case_runtime_checkpoint_instance_mismatch".to_string());
            }
            update_resume_budgets(&mut existing, &args)?;
            existing.schema = CASE_RUNTIME_CHECKPOINT_SCHEMA.to_string();
            if !authorize_checkpoint_resume(&mut existing, CheckpointResumeIntent::RuntimeWork)? {
                return Ok(CaseRuntimeReport::from(&existing));
            }
            existing
        }
        Ok(existing)
            if matches!(
                existing.status,
                CaseRuntimeStop::Running
                    | CaseRuntimeStop::AwaitingReview
                    | CaseRuntimeStop::IndeterminateEffect
            ) =>
        {
            return Err("case_runtime_checkpoint_owned_by_other_work".to_string());
        }
        Ok(_) => initial_checkpoint_with_journal(
            &args,
            journal_path,
            Some(runtime_instance_id),
            Some(item.work_id.clone()),
        )?,
        Err(_) if !checkpoint_path(&item.case_id).exists() => initial_checkpoint_with_journal(
            &args,
            journal_path,
            Some(runtime_instance_id),
            Some(item.work_id.clone()),
        )?,
        Err(error) => return Err(error),
    };
    checkpoint.task = item.task.clone();
    let checkpoint = run_with_admission(checkpoint, &args)?;
    Ok(CaseRuntimeReport::from(&checkpoint))
}

fn execute_deterministic_runtime_work(item: &RuntimeWorkItem) -> Result<CaseRuntimeReport, String> {
    let runtime_instance_id = item
        .runtime_instance_id
        .clone()
        .ok_or_else(|| "runtime_work_not_bound_to_instance".to_string())?;
    let args = runtime_work_args(item);
    let mut checkpoint = initial_checkpoint_with_journal(
        &args,
        PathBuf::from(&item.journal_path),
        Some(runtime_instance_id),
        Some(item.work_id.clone()),
    )?;
    checkpoint.task = item.task.clone();
    write_checkpoint(&checkpoint)?;
    let owner = acquire_runtime_admission(&checkpoint)?;
    let result = (|| {
        renew_runtime_admission(&owner)?;
        let outcome = advance_controlled_workflow_deterministic(&args, item)?;
        checkpoint.operations = usize::from(outcome.operation_id.is_some());
        checkpoint.last_operation_id = outcome.operation_id;
        checkpoint.last_decision_id = outcome.decision_id;
        checkpoint.last_review_id = outcome.review_id;
        checkpoint.last_effect_id = outcome.effect_id;
        checkpoint.last_receipt_id = outcome.receipt_id;
        checkpoint.last_effect_outcome = outcome.outcome.map(|value| format!("{value:?}"));
        match outcome.status {
            ControlledEffectTurnStatus::Finalized => checkpoint.stop(
                CaseRuntimeStop::Completed,
                "workflow deterministic effect finalized; provider_invocations=0",
            ),
            ControlledEffectTurnStatus::Denied => checkpoint.stop(
                CaseRuntimeStop::Denied,
                "workflow deterministic operation denied; provider_invocations=0",
            ),
            ControlledEffectTurnStatus::AwaitingReview => checkpoint.stop(
                CaseRuntimeStop::AwaitingReview,
                "workflow deterministic operation awaiting existing Review",
            ),
            ControlledEffectTurnStatus::Indeterminate => checkpoint.stop(
                CaseRuntimeStop::IndeterminateEffect,
                "workflow deterministic effect truth indeterminate",
            ),
            ControlledEffectTurnStatus::NormalizationRejected => checkpoint.stop(
                CaseRuntimeStop::FatalInvariantViolation,
                "canonical deterministic template normalization rejected",
            ),
        }
        write_checkpoint(&checkpoint)?;
        Ok(CaseRuntimeReport::from(&checkpoint))
    })();
    let release = release_runtime_admission(&owner);
    match (result, release) {
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Ok(report), Ok(())) => Ok(report),
    }
}

/// Reconstructs operational WorkItem posture from the exact checkpoint owned
/// by a stale Running item. This is recovery evidence, not Case authority.
pub(super) fn recover_runtime_work_from_checkpoint(
    item: &RuntimeWorkItem,
) -> Result<Option<(RuntimeWorkState, String)>, String> {
    let path = checkpoint_path(&item.case_id);
    if !path.exists() {
        return Ok(None);
    }
    let checkpoint = read_checkpoint(&item.case_id)?;
    checkpoint_recovery_posture(&checkpoint, item)
}

/// Converges the noncanonical checkpoint after the store has mechanically
/// proven the exact workflow execution satisfied. The store proof must happen
/// first; this helper never decides workflow truth itself.
pub(super) fn repair_workflow_checkpoint_completed(item: &RuntimeWorkItem) -> Result<bool, String> {
    if item.workflow.is_none() {
        return Err("runtime_work_is_not_workflow_attributed".to_string());
    }
    let path = checkpoint_path(&item.case_id);
    if !path.exists() {
        return Ok(false);
    }
    let mut checkpoint = read_checkpoint(&item.case_id)?;
    validate_checkpoint_work_identity(&checkpoint, item)?;
    if checkpoint.status == CaseRuntimeStop::Completed {
        return Ok(false);
    }
    let admission = acquire_runtime_admission(&checkpoint)?;
    checkpoint.stop(
        CaseRuntimeStop::Completed,
        "canonical workflow satisfaction recovered without Case re-execution",
    );
    let write_result = write_checkpoint(&checkpoint);
    let release_result = release_runtime_admission(&admission);
    write_result?;
    release_result?;
    Ok(true)
}

fn checkpoint_recovery_posture(
    checkpoint: &CaseRuntimeCheckpoint,
    item: &RuntimeWorkItem,
) -> Result<Option<(RuntimeWorkState, String)>, String> {
    if checkpoint.work_item_id.as_deref() != Some(item.work_id.as_str())
        && checkpoint_is_never_resumable(&checkpoint.status)
    {
        // A newly claimed later WorkItem may crash before it publishes its own
        // checkpoint. The old terminal checkpoint is not evidence for the new
        // work; treat it exactly as no current-work checkpoint.
        return Ok(None);
    }
    validate_checkpoint_work_identity(checkpoint, item)?;
    let state = runtime_work_state_for_checkpoint(&checkpoint.status);
    Ok(Some((
        state,
        format!(
            "checkpoint_recovery: status={}; run_id={}; detail={}",
            checkpoint.status.as_str(),
            checkpoint.run_id,
            checkpoint.stop_detail
        ),
    )))
}

fn runtime_work_state_for_checkpoint(status: &CaseRuntimeStop) -> RuntimeWorkState {
    match status {
        CaseRuntimeStop::Running => RuntimeWorkState::Queued,
        CaseRuntimeStop::Completed => RuntimeWorkState::Completed,
        CaseRuntimeStop::Denied => RuntimeWorkState::Denied,
        CaseRuntimeStop::AwaitingReview => RuntimeWorkState::WaitingReview,
        CaseRuntimeStop::IndeterminateEffect => RuntimeWorkState::WaitingEffect,
        CaseRuntimeStop::WaitingProvider => RuntimeWorkState::WaitingProvider,
        CaseRuntimeStop::DeliveryIndeterminate => RuntimeWorkState::DeliveryIndeterminate,
        CaseRuntimeStop::NormativeUnconfigured
        | CaseRuntimeStop::NormativeBlocked
        | CaseRuntimeStop::PolicyNotYetValid
        | CaseRuntimeStop::PolicyRefreshRequired
        | CaseRuntimeStop::PolicyStale
        | CaseRuntimeStop::PolicyExpired
        | CaseRuntimeStop::PolicyRevoked
        | CaseRuntimeStop::PolicyValidityUnavailable => RuntimeWorkState::Blocked,
        CaseRuntimeStop::Cancelled | CaseRuntimeStop::Closed | CaseRuntimeStop::OperatorStopped => {
            RuntimeWorkState::Cancelled
        }
        CaseRuntimeStop::ProviderFailureBudgetExhausted
        | CaseRuntimeStop::InvocationBudgetExhausted
        | CaseRuntimeStop::OperationBudgetExhausted
        | CaseRuntimeStop::ContextBudgetExhausted
        | CaseRuntimeStop::CostBudgetExhausted
        | CaseRuntimeStop::MalformedProviderResult
        | CaseRuntimeStop::FatalInvariantViolation => RuntimeWorkState::Failed,
    }
}

pub(super) fn case_runtime_status(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let authenticated = authenticate_local()?;
    let state = store.get_case_state_authorized(&authenticated, &case_id)?;
    if !checkpoint_path(&case_id).is_file() {
        println!("case_id: {case_id}");
        println!("case_generation: {}", state.generation);
        println!("case_lifecycle: {:?}", state.lifecycle);
        println!("runtime_status: NeverStarted");
        println!("runtime_admission_status: none");
        println!("stop_requested: false");
        println!("invocations: 0");
        println!("operations: 0");
        return Ok(());
    }
    let checkpoint = read_checkpoint(&case_id)?;
    print_runtime_summary(&checkpoint)
}

pub(super) fn latest_case_context_artifact_id(case_id: &str, kind: &str) -> Result<String, String> {
    if !checkpoint_path(case_id).is_file() {
        return Err("case_context_latest_unavailable_case_never_started".to_string());
    }
    let checkpoint = read_checkpoint(case_id)?;
    match kind {
        "projection" => checkpoint
            .last_projection_id
            .ok_or_else(|| "case_context_latest_projection_unavailable".to_string()),
        "context-frame" => checkpoint
            .last_context_frame_id
            .ok_or_else(|| "case_context_latest_frame_unavailable".to_string()),
        _ => Err("case_context_kind_invalid".to_string()),
    }
}

pub(super) fn case_runtime_stop(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let authenticated = authenticate_local()?;
    let state = store.get_case_state_authorized(&authenticated, &case_id)?;
    store
        .resolve_security_context(
            &authenticated,
            state.tenant_id.as_deref().ok_or_else(|| {
                "legacy_unscoped_case_runtime_stop_is_compatibility_only".to_string()
            })?,
        )?
        .require_owner()?;
    let path = checkpoint_path(&case_id);
    if !path.is_file() {
        println!("case_runtime_stop: no_active_execution");
        println!("case_id: {case_id}");
        return Ok(());
    }
    let mut checkpoint = read_checkpoint_at(&path, &case_id)?;
    checkpoint.stop_requested = true;
    checkpoint.stop_detail = "operator stop requested".to_string();
    write_checkpoint(&checkpoint)?;
    println!("case_runtime_stop: requested");
    println!("case_id: {case_id}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checkpoint(status: CaseRuntimeStop, run_id: &str, work_id: &str) -> CaseRuntimeCheckpoint {
        CaseRuntimeCheckpoint {
            schema: CASE_RUNTIME_CHECKPOINT_SCHEMA.to_string(),
            run_id: run_id.to_string(),
            runtime_instance_id: Some("runtime-instance:local-default".to_string()),
            work_item_id: Some(work_id.to_string()),
            case_id: "case:h13-checkpoint".to_string(),
            participant_id: "participant:model".to_string(),
            attachment_id: "resource:workspace".to_string(),
            journal_path: "/tmp/h13-journal.jsonl".to_string(),
            task: "bounded task".to_string(),
            input_turn_id: None,
            status,
            stop_detail: "test posture".to_string(),
            stop_requested: false,
            invocations: 1,
            operations: 1,
            provider_failures: 0,
            cumulative_estimated_input_units: 1,
            actual_input_tokens: Some(1),
            actual_output_tokens: Some(1),
            actual_total_tokens: Some(2),
            cumulative_provider_latency_ms: 1,
            max_invocations: 2,
            max_operations: 2,
            max_semantic_units: 128,
            max_resident_items: 16,
            max_cumulative_estimated_input_units: 1024,
            max_provider_retries: 1,
            max_runtime_ms: Some(10_000),
            stop_on_deny: true,
            continue_after_malformed: false,
            previous_item_ids: Vec::new(),
            last_residency_plan_id: None,
            last_projection_id: None,
            last_context_frame_id: None,
            last_projection_selected_items: 0,
            last_projection_omitted_items: 0,
            last_semantic_units: 0,
            last_provider_result_id: None,
            pending_provider_result_id: None,
            last_operation_id: None,
            last_decision_id: None,
            last_review_id: None,
            last_effect_id: None,
            last_receipt_id: None,
            last_effect_outcome: None,
        }
    }

    fn work_item(work_id: &str) -> RuntimeWorkItem {
        RuntimeWorkItem {
            schema: yai_core_engine::store::lmdb::RUNTIME_WORK_ITEM_SCHEMA.to_string(),
            work_id: work_id.to_string(),
            integrity_digest: "not-used-by-identity-test".to_string(),
            request_id: "request:h13".to_string(),
            request_digest: "digest:h13".to_string(),
            principal_id: "principal:h13".to_string(),
            tenant_id: "tenant:h13".to_string(),
            case_id: "case:h13-checkpoint".to_string(),
            participant_id: "participant:model".to_string(),
            attachment_id: "resource:workspace".to_string(),
            journal_path: "/tmp/h13-journal.jsonl".to_string(),
            task: "bounded task".to_string(),
            budgets: yai_core_engine::store::lmdb::RuntimeCaseBudgets {
                max_invocations: 2,
                max_operations: 2,
                max_semantic_units: 128,
                max_resident_items: 16,
                max_estimated_input_units: 1024,
                max_provider_retries: 1,
                max_runtime_ms: Some(10_000),
                stop_on_deny: true,
                continue_after_malformed: false,
            },
            failpoint: None,
            workflow: None,
            enqueue_sequence: 1,
            state: RuntimeWorkState::Running,
            attempt_count: 1,
            runtime_instance_id: Some("runtime-instance:local-default".to_string()),
            runtime_owner_token: Some("owner:h13".to_string()),
            worker_id: Some("worker:0".to_string()),
            last_stop_reason: "dispatched".to_string(),
            enqueued_at_unix_ms: 1,
            updated_at_unix_ms: 2,
        }
    }

    #[test]
    fn h13_terminal_checkpoint_cannot_be_reset_to_running() {
        for status in [
            CaseRuntimeStop::Completed,
            CaseRuntimeStop::Denied,
            CaseRuntimeStop::Cancelled,
            CaseRuntimeStop::Closed,
            CaseRuntimeStop::FatalInvariantViolation,
        ] {
            let mut terminal = checkpoint(status.clone(), "run:terminal", "work:terminal");
            assert!(!authorize_checkpoint_resume(
                &mut terminal,
                CheckpointResumeIntent::RuntimeWork
            )
            .unwrap());
            assert_eq!(terminal.status, status);
        }
        let mut waiting = checkpoint(
            CaseRuntimeStop::AwaitingReview,
            "run:waiting",
            "work:waiting",
        );
        assert!(
            authorize_checkpoint_resume(&mut waiting, CheckpointResumeIntent::RuntimeWork).unwrap()
        );
        assert_eq!(waiting.status, CaseRuntimeStop::Running);
    }

    #[test]
    fn h13_checkpoint_postures_map_to_operational_recovery_without_reinterpretation() {
        assert_eq!(
            runtime_work_state_for_checkpoint(&CaseRuntimeStop::Completed),
            RuntimeWorkState::Completed
        );
        assert_eq!(
            runtime_work_state_for_checkpoint(&CaseRuntimeStop::Denied),
            RuntimeWorkState::Denied
        );
        assert_eq!(
            runtime_work_state_for_checkpoint(&CaseRuntimeStop::AwaitingReview),
            RuntimeWorkState::WaitingReview
        );
        assert_eq!(
            runtime_work_state_for_checkpoint(&CaseRuntimeStop::IndeterminateEffect),
            RuntimeWorkState::WaitingEffect
        );
        assert_eq!(
            runtime_work_state_for_checkpoint(&CaseRuntimeStop::PolicyStale),
            RuntimeWorkState::Blocked
        );
        assert_eq!(
            runtime_work_state_for_checkpoint(&CaseRuntimeStop::OperatorStopped),
            RuntimeWorkState::Cancelled
        );
        let exact = checkpoint(CaseRuntimeStop::Completed, "run:exact", "work:exact");
        let exact_item = work_item("work:exact");
        assert_eq!(
            checkpoint_recovery_posture(&exact, &exact_item)
                .unwrap()
                .map(|value| value.0),
            Some(RuntimeWorkState::Completed)
        );
    }

    #[test]
    fn h13_checkpoint_publish_is_atomic_and_stale_temp_is_ignored() {
        let directory = std::env::temp_dir().join(format!(
            "yai-h13-checkpoint-{}-{}",
            std::process::id(),
            runtime_now_millis()
        ));
        let path = directory.join("checkpoint.json");
        let first = checkpoint(CaseRuntimeStop::Completed, "run:first", "work:first");
        write_checkpoint_at(&path, &first).expect("publish first checkpoint");
        fs::write(directory.join(".stale.tmp"), b"{partial")
            .expect("materialize stale temp residue");
        let loaded = read_checkpoint_at(&path, &first.case_id).expect("read final checkpoint");
        assert_eq!(loaded.run_id, "run:first");

        let second = checkpoint(CaseRuntimeStop::Denied, "run:second", "work:second");
        let third = checkpoint(CaseRuntimeStop::AwaitingReview, "run:third", "work:third");
        let path_two = path.clone();
        let path_three = path.clone();
        let writer_two = std::thread::spawn(move || write_checkpoint_at(&path_two, &second));
        let writer_three = std::thread::spawn(move || write_checkpoint_at(&path_three, &third));
        writer_two.join().unwrap().expect("second atomic writer");
        writer_three.join().unwrap().expect("third atomic writer");
        let loaded = read_checkpoint_at(&path, "case:h13-checkpoint")
            .expect("concurrent publication leaves complete JSON");
        assert!(matches!(
            loaded.status,
            CaseRuntimeStop::Denied | CaseRuntimeStop::AwaitingReview
        ));
        fs::remove_dir_all(directory).expect("remove checkpoint test directory");
    }

    #[test]
    fn h13_checkpoint_from_another_work_is_rejected() {
        let checkpoint = checkpoint(CaseRuntimeStop::AwaitingReview, "run:a", "work:a");
        let item = work_item("work:b");
        assert_eq!(
            validate_checkpoint_work_identity(&checkpoint, &item).unwrap_err(),
            "case_runtime_checkpoint_owned_by_other_work"
        );
    }

    #[test]
    fn h13_old_terminal_checkpoint_is_not_adopted_by_newly_claimed_work() {
        let terminal = checkpoint(CaseRuntimeStop::Completed, "run:a", "work:a");
        let item = work_item("work:b");
        assert_eq!(checkpoint_recovery_posture(&terminal, &item).unwrap(), None);
    }
}
