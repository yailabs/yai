//! Disposable execution loop that advances durable Case state.
//!
//! The loop owns a bounded transition algorithm. It has no canonical state,
//! model memory, policy authority, or resource authority; restart always
//! resumes from the Transition ledger and materialized CaseState.

use super::*;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};
use yai_core_engine::case_policy::NormativeReadiness;
use yai_core_engine::effect::OPERATION_PROPOSAL_SCHEMA;
use yai_core_engine::store::lmdb::{CaseRuntimeAdmissionOutcome, CaseRuntimeAdmissionRequest};

const CASE_RUNTIME_CHECKPOINT_SCHEMA: &str = "yai.case_runtime_checkpoint.v1";
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CaseRuntimeCheckpoint {
    schema: String,
    run_id: String,
    case_id: String,
    participant_id: String,
    attachment_id: String,
    journal_path: String,
    task: String,
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

impl CaseRuntimeCheckpoint {
    fn stop(&mut self, status: CaseRuntimeStop, detail: impl Into<String>) {
        self.status = status;
        self.stop_detail = detail.into();
    }
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
    fs::write(&temporary, encoded)
        .map_err(|error| format!("failed to write case runtime checkpoint: {error}"))?;
    fs::rename(&temporary, &path)
        .map_err(|error| format!("failed to publish case runtime checkpoint: {error}"))?;
    Ok(())
}

fn read_checkpoint(case_id: &str) -> Result<CaseRuntimeCheckpoint, String> {
    let path = checkpoint_path(case_id);
    let encoded =
        fs::read(&path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let checkpoint: CaseRuntimeCheckpoint = serde_json::from_slice(&encoded)
        .map_err(|error| format!("invalid case runtime checkpoint: {error}"))?;
    if checkpoint.schema != CASE_RUNTIME_CHECKPOINT_SCHEMA || checkpoint.case_id != case_id {
        return Err("case_runtime_checkpoint_identity_or_schema_mismatch".to_string());
    }
    Ok(checkpoint)
}

fn initial_checkpoint(args: &[String]) -> Result<CaseRuntimeCheckpoint, String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--subject")?;
    let attachment_id = named_arg(args, "--attachment")?;
    let journal_path = case_journal_path(args, "yai case run")?;
    let task = named_arg(args, "--prompt")?;
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
    if !state
        .resources
        .iter()
        .any(|resource| resource.attachment_id == attachment_id)
    {
        return Err(format!(
            "attachment {attachment_id} is not bound to {case_id}"
        ));
    }
    let run_material = format!("{case_id}:{}:{}", state.generation, runtime_now_millis());
    Ok(CaseRuntimeCheckpoint {
        schema: CASE_RUNTIME_CHECKPOINT_SCHEMA.to_string(),
        run_id: format!(
            "case-run:{}",
            yai_core_engine::context::stable_digest(&run_material)
        ),
        case_id,
        participant_id,
        attachment_id,
        journal_path: journal_path.display().to_string(),
        task,
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
    checkpoint.stop_requested = false;
    checkpoint.status = CaseRuntimeStop::Running;
    checkpoint.stop_detail.clear();
    Ok(())
}

fn provider_args(checkpoint: &CaseRuntimeCheckpoint) -> Result<Vec<String>, String> {
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store
        .get_case_state(&checkpoint.case_id)?
        .ok_or_else(|| format!("canonical CaseState missing for {}", checkpoint.case_id))?;
    let provider = state
        .provider
        .ok_or_else(|| "Case has no attached provider".to_string())?;
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
    Ok(args)
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
            }),
            _ => None,
        })
        .ok_or_else(|| format!("canonical ProviderResult not found: {result_id}"))
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

fn run_with_admission(checkpoint: CaseRuntimeCheckpoint, args: &[String]) -> Result<(), String> {
    let owner = acquire_runtime_admission(&checkpoint)?;
    let result = run_loop(checkpoint, args, &owner);
    let release = LmdbRecordStore::open(record_store_path()).and_then(|store| {
        store
            .release_case_runtime_admission(&owner.case_id, &owner.run_id, &owner.owner_token)
            .map(|_| ())
    });
    match (result, release) {
        (Err(error), _) => Err(error),
        (Ok(()), Err(error)) => Err(error),
        (Ok(()), Ok(())) => Ok(()),
    }
}

fn run_loop(
    mut checkpoint: CaseRuntimeCheckpoint,
    args: &[String],
    admission: &RuntimeAdmissionOwner,
) -> Result<(), String> {
    std::env::set_var("YAI_JOURNAL", &checkpoint.journal_path);
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
        let normative =
            LmdbRecordStore::open(record_store_path())?.case_policy_status(&checkpoint.case_id)?;
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
            let options = RuntimeInvocationOptions {
                max_resident_items: checkpoint.max_resident_items,
                max_semantic_units: checkpoint.max_semantic_units,
                max_estimated_input_units: remaining_cost,
                retrieval_limit: checkpoint.max_resident_items.saturating_mul(4).max(1),
                previous_item_ids: checkpoint.previous_item_ids.clone(),
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
            let provider_args = provider_args(&checkpoint)?;
            let mut attempt = 0usize;
            let result = loop {
                match invoke_runtime_provider(
                    &provider_args,
                    ProjectionPurpose::FilesystemWriteProposal,
                    &checkpoint.task,
                    InvocationOutputContract::CaseRuntimeTurn {
                        schema: CASE_RUNTIME_OUTPUT_SCHEMA.to_string(),
                        operation_schema: OPERATION_PROPOSAL_SCHEMA.to_string(),
                        attachment_id: resource.attachment_id.clone(),
                        allowed_write_prefix: resource.allowed_write_prefix.clone(),
                        max_write_bytes: resource.max_write_bytes,
                    },
                    &options,
                ) {
                    Ok(result) => break result,
                    Err(error) if error.starts_with("residency_budget_below_mandatory_state") => {
                        checkpoint.stop(CaseRuntimeStop::ContextBudgetExhausted, error);
                        write_checkpoint(&checkpoint)?;
                        return print_runtime_summary(&checkpoint);
                    }
                    Err(error) if error.starts_with("provider_input_budget_exceeded") => {
                        checkpoint.stop(CaseRuntimeStop::CostBudgetExhausted, error);
                        write_checkpoint(&checkpoint)?;
                        return print_runtime_summary(&checkpoint);
                    }
                    Err(error) if attempt < checkpoint.max_provider_retries => {
                        attempt += 1;
                        checkpoint.provider_failures += 1;
                        eprintln!(
                            "provider_retry: {attempt} reason:{}",
                            runtime_error_preview(&error)
                        );
                    }
                    Err(error) => {
                        checkpoint.provider_failures += 1;
                        checkpoint.stop(CaseRuntimeStop::ProviderFailureBudgetExhausted, error);
                        write_checkpoint(&checkpoint)?;
                        return print_runtime_summary(&checkpoint);
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
            if runtime_failpoint(args).as_deref() == Some("runtime_after_provider_result") {
                exit_runtime_failpoint("runtime_after_provider_result", 91);
            }
            result
        };

        if completion_response(&provider_result.raw_output) {
            checkpoint.pending_provider_result_id = None;
            checkpoint.stop(
                CaseRuntimeStop::Completed,
                "provider returned typed completion",
            );
            write_checkpoint(&checkpoint)?;
            break;
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
    print_runtime_summary(&checkpoint)
}

fn print_runtime_summary(checkpoint: &CaseRuntimeCheckpoint) -> Result<(), String> {
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store
        .get_case_state(&checkpoint.case_id)?
        .ok_or_else(|| "canonical CaseState missing for runtime summary".to_string())?;
    println!("case_runtime_schema: {}", checkpoint.schema);
    println!("run_id: {}", checkpoint.run_id);
    println!("case_id: {}", checkpoint.case_id);
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
    run_with_admission(checkpoint, args)
}

pub(super) fn case_runtime_resume(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let mut checkpoint = read_checkpoint(&case_id)?;
    update_resume_budgets(&mut checkpoint, args)?;
    run_with_admission(checkpoint, args)
}

pub(super) fn case_runtime_status(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let checkpoint = read_checkpoint(&case_id)?;
    print_runtime_summary(&checkpoint)
}

pub(super) fn case_runtime_stop(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let mut checkpoint = read_checkpoint(&case_id)?;
    checkpoint.stop_requested = true;
    checkpoint.stop_detail = "operator stop requested".to_string();
    write_checkpoint(&checkpoint)?;
    println!("case_runtime_stop: requested");
    println!("case_id: {case_id}");
    Ok(())
}
