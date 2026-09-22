//! Shared operational checkpoint contract for the existing bounded Case runner.
//! This is rebuild/recovery metadata, never Case authority or a second scheduler.
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::Path};
use yai_core_engine::store::lmdb::RuntimeWorkItem;

pub fn checkpoint_path_for(home: &Path, case: &str) -> std::path::PathBuf {
    home.join("run").join("case-runtime").join(format!("{}.json",
        yai_core_engine::context::stable_digest(case)))
}

pub const CASE_RUNTIME_CHECKPOINT_SCHEMA: &str = "yai.case_runtime_checkpoint.v3";
pub const CASE_RUNTIME_CHECKPOINT_SCHEMA_V2: &str = "yai.case_runtime_checkpoint.v2";
pub const CASE_RUNTIME_CHECKPOINT_SCHEMA_V1: &str = "yai.case_runtime_checkpoint.v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseRuntimeStop {
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
    pub fn as_str(&self) -> &'static str {
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
pub struct CaseRuntimeCheckpoint {
    pub schema: String,
    pub run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_instance_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_item_id: Option<String>,
    pub case_id: String,
    pub participant_id: String,
    pub attachment_id: String,
    pub journal_path: String,
    pub task: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_turn_id: Option<String>,
    pub status: CaseRuntimeStop,
    pub stop_detail: String,
    pub stop_requested: bool,
    pub invocations: usize,
    pub operations: usize,
    pub provider_failures: usize,
    pub cumulative_estimated_input_units: usize,
    pub actual_input_tokens: Option<u64>,
    pub actual_output_tokens: Option<u64>,
    pub actual_total_tokens: Option<u64>,
    pub cumulative_provider_latency_ms: u64,
    pub max_invocations: usize,
    pub max_operations: usize,
    pub max_semantic_units: usize,
    pub max_resident_items: usize,
    pub max_cumulative_estimated_input_units: usize,
    pub max_provider_retries: usize,
    pub max_runtime_ms: Option<u64>,
    pub stop_on_deny: bool,
    pub continue_after_malformed: bool,
    pub previous_item_ids: Vec<String>,
    pub last_residency_plan_id: Option<String>,
    pub last_projection_id: Option<String>,
    pub last_context_frame_id: Option<String>,
    pub last_projection_selected_items: usize,
    pub last_projection_omitted_items: usize,
    pub last_semantic_units: usize,
    pub last_provider_result_id: Option<String>,
    pub pending_provider_result_id: Option<String>,
    pub last_operation_id: Option<String>,
    pub last_decision_id: Option<String>,
    #[serde(default)]
    pub last_review_id: Option<String>,
    pub last_effect_id: Option<String>,
    pub last_receipt_id: Option<String>,
    pub last_effect_outcome: Option<String>,
}

#[derive(Clone, Debug)]
pub struct CaseRuntimeReport {
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
    pub fn stop(&mut self, status: CaseRuntimeStop, detail: impl Into<String>) {
        self.status = status;
        self.stop_detail = detail.into();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckpointResumeIntent {
    DirectOperator,
    RuntimeWork,
}

pub fn checkpoint_is_never_resumable(status: &CaseRuntimeStop) -> bool {
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

pub fn authorize_checkpoint_resume(
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

pub fn validate_checkpoint_work_identity(
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

pub fn write_checkpoint_at(path: &Path, checkpoint: &CaseRuntimeCheckpoint) -> Result<(), String> {
    let _lock = checkpoint_lock(path)?;
    let mut merged = checkpoint.clone();
    if path.exists() {
        let current = read_checkpoint_at(path, &checkpoint.case_id)?;
        if current.run_id == checkpoint.run_id && current.work_item_id == checkpoint.work_item_id {
            merged.stop_requested |= current.stop_requested;
        }
    }
    publish_checkpoint_at(path, &merged)
}

/// Only an explicitly admitted operator resume may clear the sticky stop bit.
/// Ordinary worker progress uses `write_checkpoint_at` and cannot clear it.
pub fn write_resumed_checkpoint_at(path: &Path, checkpoint: &CaseRuntimeCheckpoint) -> Result<(), String> {
    let _lock = checkpoint_lock(path)?;
    let current = read_checkpoint_at(path, &checkpoint.case_id)?;
    if current.run_id != checkpoint.run_id || current.work_item_id != checkpoint.work_item_id
        || checkpoint_is_never_resumable(&current.status) {
        return Err("case_runtime_resume_checkpoint_stale_or_terminal".into());
    }
    publish_checkpoint_at(path, checkpoint)
}

/// Operational control only. Callers must establish current Case authority.
/// Reading and updating under the publication lock preserves worker progress.
pub fn request_checkpoint_stop_at(path: &Path, case: &str, expected_run: &str, expected_work: Option<&str>) -> Result<CaseRuntimeCheckpoint, String> {
    let _lock = checkpoint_lock(path)?;
    let mut checkpoint = read_checkpoint_at(path, case)?;
    if checkpoint.run_id != expected_run || expected_work.is_some_and(|work| checkpoint.work_item_id.as_deref() != Some(work)) {
        return Err("case_runtime_stop_checkpoint_stale".into());
    }
    if !checkpoint.stop_requested {
        checkpoint.stop_requested = true;
        publish_checkpoint_at(path, &checkpoint)?;
    }
    Ok(checkpoint)
}

fn checkpoint_lock(path: &Path) -> Result<fs::File, String> {
    let parent = path.parent().ok_or("case_runtime_checkpoint_parent_missing")?;
    fs::create_dir_all(parent).map_err(|e| format!("case_runtime_directory:{e}"))?;
    let mut options = fs::OpenOptions::new();
    options.create(true).read(true).write(true);
    #[cfg(unix)] {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    // The stable lock inode is not removed after use: unlinking it could let
    // two processes lock different inodes for the same checkpoint.
    let lock = options.open(path.with_extension("lock")).map_err(|e| format!("case_runtime_lock_open:{e}"))?;
    lock.lock().map_err(|e| format!("case_runtime_lock:{e}"))?;
    Ok(lock)
}

fn publish_checkpoint_at(path: &Path, checkpoint: &CaseRuntimeCheckpoint) -> Result<(), String> {
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

pub fn read_checkpoint_at(path: &Path, case_id: &str) -> Result<CaseRuntimeCheckpoint, String> {
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
