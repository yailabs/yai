//! Local tenant-fair multi-Case RuntimeInstance.
//!
//! This module owns durable operational scheduling only. Case Transitions,
//! policy, security, Decisions, Grants, and effect truth remain downstream.

use super::*;
use crate::case_runtime::{
    execute_runtime_work, recover_runtime_work_from_checkpoint, CaseRuntimeReport, CaseRuntimeStop,
};
use crate::provider::validate_journal_case_binding;
use crate::security::authenticate_local;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use yai_core_engine::case_policy::{NormativeReadiness, PolicyValidityPosture};
use yai_core_engine::store::lmdb::{
    runtime_process_identity_is_live, RuntimeCaseBudgets, RuntimeInstanceAcquireOutcome,
    RuntimeInstanceAcquireRequest, RuntimeInstanceConfig, RuntimeInstanceLifecycle,
    RuntimeWorkItem, RuntimeWorkState, RuntimeWorkSubmission,
};
use yai_core_engine::transition::{CaseLifecycle, ReviewResolution};

const INSTANCE_LEASE_MS: u64 = 5_000;
const HEARTBEAT_INTERVAL_MS: u64 = 1_000;
const PARKED_RECHECK_MS: u64 = 1_000;
const SCHEDULER_TICK_MS: u64 = 50;

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}

fn positive_arg(args: &[String], name: &str, default: usize) -> Result<usize, String> {
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

fn optional_positive_u64(args: &[String], name: &str) -> Result<Option<u64>, String> {
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

fn instance_config(args: &[String]) -> Result<RuntimeInstanceConfig, String> {
    let config = RuntimeInstanceConfig {
        workers: positive_arg(args, "--workers", 2)?,
        max_active_per_tenant: positive_arg(args, "--max-active-per-tenant", 1)?,
        max_queued_per_tenant: positive_arg(args, "--max-queued-per-tenant", 32)?,
        max_queued_total: positive_arg(args, "--max-queued-total", 128)?,
    };
    config.validate()?;
    Ok(config)
}

fn work_budgets(args: &[String]) -> Result<RuntimeCaseBudgets, String> {
    let budgets = RuntimeCaseBudgets {
        max_invocations: positive_arg(args, "--max-invocations", 8)?,
        max_operations: positive_arg(args, "--max-operations", 8)?,
        max_semantic_units: positive_arg(
            args,
            "--max-semantic-units",
            DEFAULT_SEMANTIC_UNIT_BUDGET,
        )?,
        max_resident_items: positive_arg(args, "--max-resident-items", DEFAULT_MAX_RESIDENT_ITEMS)?,
        max_estimated_input_units: positive_arg(
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
        max_runtime_ms: optional_positive_u64(args, "--max-runtime-ms")?,
        stop_on_deny: args.iter().any(|value| value == "--stop-on-deny"),
        continue_after_malformed: args
            .iter()
            .any(|value| value == "--continue-after-malformed"),
    };
    budgets.validate()?;
    Ok(budgets)
}

fn instance_token(principal_id: &str) -> String {
    format!(
        "runtime-owner:{}",
        yai_core_engine::context::stable_digest(&format!(
            "{}:{}:{}",
            principal_id,
            std::process::id(),
            now_unix_ms()
        ))
    )
}

fn local_process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        unsafe { kill(pid as c_int, 0) == 0 }
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
        true
    }
}

fn runtime_submit(args: &[String]) -> Result<(), String> {
    let tenant_id = named_arg(args, "--tenant")?;
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--subject")?;
    let attachment_id = named_arg(args, "--attachment")?;
    let task = named_arg(args, "--prompt")?;
    let request_id = optional_arg(args, "--idempotency-key").unwrap_or_else(|| {
        format!(
            "request:{}",
            yai_core_engine::context::stable_digest(&format!(
                "{}:{}:{}",
                std::process::id(),
                now_unix_ms(),
                task
            ))
        )
    });
    let journal_path = case_journal_path(args, "yai runtime submit")?;
    validate_journal_case_binding(&journal_path, &case_id)?;
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let outcome = store.submit_runtime_work(
        &authenticated,
        &RuntimeWorkSubmission {
            request_id,
            tenant_id,
            case_id,
            participant_id,
            attachment_id,
            journal_path: journal_path.display().to_string(),
            task,
            budgets: work_budgets(args)?,
            failpoint: optional_arg(args, "--failpoint"),
            now_unix_ms: now_unix_ms(),
        },
    )?;
    println!("runtime_work_submission: accepted");
    println!("created: {}", outcome.created);
    print_work_item(&outcome.item);
    Ok(())
}

fn print_work_item(item: &RuntimeWorkItem) {
    println!("work_id: {}", item.work_id);
    println!("request_id: {}", item.request_id);
    println!("tenant_id: {}", item.tenant_id);
    println!("case_id: {}", item.case_id);
    println!("state: {:?}", item.state);
    println!("enqueue_sequence: {}", item.enqueue_sequence);
    println!("attempt_count: {}", item.attempt_count);
    println!("worker_id: {}", item.worker_id.as_deref().unwrap_or("none"));
    println!("stop_reason: {}", item.last_stop_reason);
}

fn runtime_status(_args: &[String]) -> Result<(), String> {
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let instance = store
        .get_runtime_instance_authorized(&authenticated)?
        .ok_or_else(|| "runtime_instance_missing".to_string())?;
    let items = store.list_runtime_work_authorized(&authenticated)?;
    let mut by_state = BTreeMap::<String, usize>::new();
    let mut per_tenant = BTreeMap::<String, (usize, usize)>::new();
    for item in &items {
        *by_state.entry(format!("{:?}", item.state)).or_default() += 1;
        let counters = per_tenant.entry(item.tenant_id.clone()).or_default();
        if matches!(item.state, RuntimeWorkState::Running) {
            counters.0 += 1;
        }
        if item.state.is_queued_capacity() {
            counters.1 += 1;
        }
    }
    println!("runtime_instance_schema: {}", instance.schema);
    println!("runtime_instance_id: {}", instance.instance_id);
    println!("integrity_digest: {}", instance.integrity_digest);
    println!("authenticated_principal_id: {}", instance.principal_id);
    println!("pid: {}", instance.owner_pid);
    println!(
        "owner_process_identity: {}",
        instance.owner_process_identity
    );
    println!(
        "owner_pid_alive: {}",
        local_process_alive(instance.owner_pid)
    );
    println!(
        "owner_process_identity_live: {}",
        runtime_process_identity_is_live(instance.owner_pid, &instance.owner_process_identity)
    );
    println!("state: {:?}", instance.lifecycle);
    println!("workers: {}", instance.config.workers);
    println!(
        "max_active_per_tenant: {}",
        instance.config.max_active_per_tenant
    );
    println!(
        "max_queued_per_tenant: {}",
        instance.config.max_queued_per_tenant
    );
    println!("max_queued_total: {}", instance.config.max_queued_total);
    println!("heartbeat_at_unix_ms: {}", instance.heartbeat_at_unix_ms);
    println!(
        "lease_expires_at_unix_ms: {}",
        instance.lease_expires_at_unix_ms
    );
    println!("recovered_items: {}", instance.recovered_items);
    println!(
        "last_dispatched_tenant: {}",
        instance.last_dispatched_tenant.as_deref().unwrap_or("none")
    );
    println!(
        "drain_requested_at_unix_ms: {}",
        instance
            .drain_requested_at_unix_ms
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string())
    );
    println!("work_items_total: {}", items.len());
    for (state, count) in by_state {
        println!("state_count: {state}={count}");
    }
    for (tenant, (active, queued)) in per_tenant {
        println!("tenant_metrics: {tenant} active={active} queued={queued}");
    }
    Ok(())
}

fn runtime_queue(_args: &[String]) -> Result<(), String> {
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    store
        .get_runtime_instance_authorized(&authenticated)?
        .ok_or_else(|| "runtime_instance_missing".to_string())?;
    let items = store.list_runtime_work_authorized(&authenticated)?;
    println!("runtime_queue_items: {}", items.len());
    for item in items {
        println!("---");
        print_work_item(&item);
    }
    Ok(())
}

fn runtime_stop(_args: &[String]) -> Result<(), String> {
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let instance = store.request_runtime_instance_drain(&authenticated, now_unix_ms())?;
    println!("runtime_stop: drain_requested");
    println!("runtime_instance_id: {}", instance.instance_id);
    println!("state: drain_requested");
    Ok(())
}

fn has_pending_review(item: &RuntimeWorkItem, store: &LmdbRecordStore) -> Result<bool, String> {
    let state = store
        .get_case_state(&item.case_id)?
        .ok_or_else(|| "runtime_work_case_missing".to_string())?;
    Ok(state.reviews.iter().any(|review| {
        matches!(
            review.status,
            ReviewResolution::Pending | ReviewResolution::PendingOperator
        )
    }))
}

fn work_policy_is_usable(item: &RuntimeWorkItem, store: &LmdbRecordStore) -> Result<bool, String> {
    let status = store.case_policy_status(&item.case_id)?;
    Ok(status.readiness == NormativeReadiness::Ready
        && status.validity == PolicyValidityPosture::Valid)
}

fn case_runtime_admission_is_available(
    item: &RuntimeWorkItem,
    store: &LmdbRecordStore,
) -> Result<bool, String> {
    Ok(store
        .get_case_runtime_admission(&item.case_id)?
        .is_none_or(|admission| {
            admission.expires_at_unix_ms <= now_unix_ms()
                || !local_process_alive(admission.owner_pid)
        }))
}

fn recovery_sweep(
    authenticated: &yai_core_engine::security::AuthenticatedPrincipal,
    owner_token: &str,
) -> Result<usize, String> {
    let store = LmdbRecordStore::open(record_store_path())?;
    let items = store.list_runtime_work_authorized(authenticated)?;
    let mut recovered = 0usize;
    for item in items.into_iter().filter(|item| !item.state.is_terminal()) {
        let state = store
            .get_case_state(&item.case_id)?
            .ok_or_else(|| "runtime_work_case_missing".to_string())?;
        let target = if state.lifecycle == CaseLifecycle::Closed || state.cancellation.is_some() {
            Some((
                RuntimeWorkState::Cancelled,
                "case_cancelled_or_closed".to_string(),
            ))
        } else {
            match item.state {
                RuntimeWorkState::Running => {
                    recover_runtime_work_from_checkpoint(&item)?.or_else(|| {
                        Some((
                            RuntimeWorkState::Queued,
                            "stale_running_work_without_checkpoint".to_string(),
                        ))
                    })
                }
                RuntimeWorkState::WaitingEffect => Some((
                    RuntimeWorkState::Queued,
                    "canonical_effect_reconciliation_required".to_string(),
                )),
                RuntimeWorkState::WaitingReview if !has_pending_review(&item, &store)? => Some((
                    RuntimeWorkState::Queued,
                    "review_resolved_requeue".to_string(),
                )),
                RuntimeWorkState::Blocked if work_policy_is_usable(&item, &store)? => {
                    if item
                        .last_stop_reason
                        .contains("case_runtime_admission_active")
                    {
                        case_runtime_admission_is_available(&item, &store)?.then_some((
                            RuntimeWorkState::Queued,
                            "case_runtime_admission_available".to_string(),
                        ))
                    } else {
                        Some((
                            RuntimeWorkState::Queued,
                            "policy_repaired_requeue".to_string(),
                        ))
                    }
                }
                _ => None,
            }
        };
        if let Some((target, reason)) = target {
            store.update_runtime_work_state(
                authenticated,
                owner_token,
                &item.work_id,
                None,
                target,
                &reason,
                now_unix_ms(),
            )?;
            recovered += 1;
        }
    }
    Ok(recovered)
}

#[derive(Clone, Debug)]
struct ActiveWork {
    case_id: String,
    tenant_id: String,
    resource_root: Option<PathBuf>,
}

fn roots_overlap(left: &Path, right: &Path) -> bool {
    left.starts_with(right) || right.starts_with(left)
}

fn work_resource_root(item: &RuntimeWorkItem) -> Result<Option<PathBuf>, String> {
    let store = LmdbRecordStore::open(record_store_path())?;
    Ok(store
        .get_local_filesystem_binding(&item.case_id, &item.attachment_id)?
        .map(|binding| PathBuf::from(binding.canonical_root)))
}

fn resource_dispatchable(root: Option<&Path>, active: &HashMap<String, ActiveWork>) -> bool {
    active.values().all(|work| {
        matches!((root, work.resource_root.as_deref()), (Some(left), Some(right)) if !roots_overlap(left, right))
    })
}

fn select_fair_work(
    items: &[RuntimeWorkItem],
    active: &HashMap<String, ActiveWork>,
    last_tenant: Option<&str>,
    max_active_per_tenant: usize,
) -> Result<Option<(RuntimeWorkItem, Option<PathBuf>, String)>, String> {
    select_fair_work_with_roots(
        items,
        active,
        last_tenant,
        max_active_per_tenant,
        work_resource_root,
    )
}

fn select_fair_work_with_roots<F>(
    items: &[RuntimeWorkItem],
    active: &HashMap<String, ActiveWork>,
    last_tenant: Option<&str>,
    max_active_per_tenant: usize,
    resolve_root: F,
) -> Result<Option<(RuntimeWorkItem, Option<PathBuf>, String)>, String>
where
    F: Fn(&RuntimeWorkItem) -> Result<Option<PathBuf>, String>,
{
    let active_cases: std::collections::HashSet<&str> =
        active.values().map(|work| work.case_id.as_str()).collect();
    let mut tenant_active = BTreeMap::<String, usize>::new();
    for work in active.values() {
        *tenant_active.entry(work.tenant_id.clone()).or_default() += 1;
    }
    let mut by_tenant = BTreeMap::<String, Vec<&RuntimeWorkItem>>::new();
    for item in items
        .iter()
        .filter(|item| matches!(item.state, RuntimeWorkState::Queued))
    {
        by_tenant
            .entry(item.tenant_id.clone())
            .or_default()
            .push(item);
    }
    for values in by_tenant.values_mut() {
        values.sort_by_key(|item| item.enqueue_sequence);
    }
    let tenants: Vec<String> = by_tenant.keys().cloned().collect();
    if tenants.is_empty() {
        return Ok(None);
    }
    let start = last_tenant
        .and_then(|last| tenants.iter().position(|tenant| tenant == last))
        .map(|index| (index + 1) % tenants.len())
        .unwrap_or(0);
    for offset in 0..tenants.len() {
        let tenant = &tenants[(start + offset) % tenants.len()];
        if tenant_active.get(tenant).copied().unwrap_or(0) >= max_active_per_tenant {
            continue;
        }
        let Some(candidate) = by_tenant[tenant].iter().find(|item| {
            !active_cases.contains(item.case_id.as_str())
                && !items.iter().any(|other| {
                    other.work_id != item.work_id
                        && other.case_id == item.case_id
                        && !other.state.is_terminal()
                        && other.enqueue_sequence < item.enqueue_sequence
                })
        }) else {
            continue;
        };
        let root = resolve_root(candidate)?;
        if resource_dispatchable(root.as_deref(), active) {
            let reason = format!(
                "tenant_round_robin tenant={} fifo_sequence={} resource_relation={}",
                tenant,
                candidate.enqueue_sequence,
                if active.is_empty() {
                    "no_active_conflict"
                } else {
                    "disjoint"
                }
            );
            return Ok(Some(((*candidate).clone(), root, reason)));
        }
    }
    Ok(None)
}

fn report_work_state(report: &CaseRuntimeReport) -> RuntimeWorkState {
    match report.status {
        CaseRuntimeStop::Completed => RuntimeWorkState::Completed,
        CaseRuntimeStop::Denied => RuntimeWorkState::Denied,
        CaseRuntimeStop::AwaitingReview => RuntimeWorkState::WaitingReview,
        CaseRuntimeStop::IndeterminateEffect => RuntimeWorkState::WaitingEffect,
        CaseRuntimeStop::PolicyNotYetValid
        | CaseRuntimeStop::PolicyRefreshRequired
        | CaseRuntimeStop::PolicyStale
        | CaseRuntimeStop::PolicyExpired
        | CaseRuntimeStop::PolicyRevoked
        | CaseRuntimeStop::PolicyValidityUnavailable
        | CaseRuntimeStop::NormativeUnconfigured
        | CaseRuntimeStop::NormativeBlocked => RuntimeWorkState::Blocked,
        CaseRuntimeStop::Cancelled | CaseRuntimeStop::Closed | CaseRuntimeStop::OperatorStopped => {
            RuntimeWorkState::Cancelled
        }
        CaseRuntimeStop::Running => RuntimeWorkState::Failed,
        CaseRuntimeStop::ProviderFailureBudgetExhausted
        | CaseRuntimeStop::InvocationBudgetExhausted
        | CaseRuntimeStop::OperationBudgetExhausted
        | CaseRuntimeStop::ContextBudgetExhausted
        | CaseRuntimeStop::CostBudgetExhausted
        | CaseRuntimeStop::MalformedProviderResult
        | CaseRuntimeStop::FatalInvariantViolation => RuntimeWorkState::Failed,
    }
}

struct WorkerResult {
    worker_id: String,
    work_id: String,
    outcome: WorkerOutcome,
}

enum WorkerOutcome {
    Result(Result<CaseRuntimeReport, String>),
    Panicked(String),
}

fn runtime_serve(args: &[String]) -> Result<(), String> {
    let config = instance_config(args)?;
    let authenticated = authenticate_local()?;
    let principal_id = authenticated.projected_principal_id();
    let owner_token = instance_token(&principal_id);
    let store = LmdbRecordStore::open(record_store_path())?;
    let (acquire, starting) = store.acquire_runtime_instance(
        &authenticated,
        &RuntimeInstanceAcquireRequest {
            owner_pid: std::process::id(),
            owner_token: owner_token.clone(),
            now_unix_ms: now_unix_ms(),
            lease_duration_ms: INSTANCE_LEASE_MS,
            config: config.clone(),
        },
        true,
    )?;
    println!("runtime_instance_schema: {}", starting.schema);
    println!("runtime_instance_id: {}", starting.instance_id);
    println!("integrity_digest: {}", starting.integrity_digest);
    println!("authenticated_principal_id: {principal_id}");
    println!("pid: {}", starting.owner_pid);
    println!(
        "owner_process_identity: {}",
        starting.owner_process_identity
    );
    println!("state: starting");
    println!(
        "instance_admission: {}",
        match acquire {
            RuntimeInstanceAcquireOutcome::Acquired => "acquired",
            RuntimeInstanceAcquireOutcome::Renewed => "renewed",
            RuntimeInstanceAcquireOutcome::Reclaimed => "reclaimed_stale",
        }
    );
    let recovered = recovery_sweep(&authenticated, &owner_token)?;
    let running = store.activate_runtime_instance(
        &authenticated,
        &owner_token,
        now_unix_ms(),
        INSTANCE_LEASE_MS,
        recovered,
    )?;
    println!("state: running");
    println!("workers: {}", config.workers);
    println!("max_active_per_tenant: {}", config.max_active_per_tenant);
    println!("max_queued_per_tenant: {}", config.max_queued_per_tenant);
    println!("max_queued_total: {}", config.max_queued_total);
    println!("recovered_items: {recovered}");
    println!("heartbeat_at_unix_ms: {}", running.heartbeat_at_unix_ms);
    if let Some(delay_ms) = optional_positive_u64(args, "--startup-dispatch-delay-ms")? {
        println!("startup_dispatch_delay_ms: {delay_ms}");
        thread::sleep(Duration::from_millis(delay_ms));
    }

    let (job_tx, job_rx) = mpsc::channel::<(String, RuntimeWorkItem)>();
    let job_rx = Arc::new(Mutex::new(job_rx));
    let (result_tx, result_rx) = mpsc::channel::<WorkerResult>();
    let mut handles = Vec::new();
    for _ in 0..config.workers {
        let receiver = Arc::clone(&job_rx);
        let sender = result_tx.clone();
        let worker_failpoint = optional_arg(args, "--failpoint");
        handles.push(thread::spawn(move || loop {
            let job = receiver.lock().ok().and_then(|rx| rx.recv().ok());
            let Some((worker_id, item)) = job else {
                break;
            };
            println!(
                "runtime_worker_event: started timestamp_unix_ms={} worker_id={} work_id={} tenant_id={} case_id={}",
                now_unix_ms(), worker_id, item.work_id, item.tenant_id, item.case_id
            );
            let result = catch_unwind(AssertUnwindSafe(|| {
                if worker_failpoint.as_deref() == Some("worker_panic_before_case_runtime") {
                    panic!("worker_panic_before_case_runtime");
                }
                execute_runtime_work(&item)
            }));
            let outcome = match result {
                Ok(result) => WorkerOutcome::Result(result),
                Err(payload) => WorkerOutcome::Panicked(
                    payload
                        .downcast_ref::<&str>()
                        .map(|value| (*value).to_string())
                        .or_else(|| payload.downcast_ref::<String>().cloned())
                        .unwrap_or_else(|| "unknown_worker_panic".to_string()),
                ),
            };
            println!(
                "runtime_worker_event: stopped timestamp_unix_ms={} worker_id={} work_id={} status={}",
                now_unix_ms(),
                worker_id,
                item.work_id,
                match &outcome {
                    WorkerOutcome::Result(Ok(report)) => report.status.as_str(),
                    WorkerOutcome::Result(Err(_)) => "worker_error",
                    WorkerOutcome::Panicked(_) => "worker_panicked",
                }
            );
            if sender
                .send(WorkerResult {
                    worker_id,
                    work_id: item.work_id,
                    outcome,
                })
                .is_err()
            {
                break;
            }
        }));
    }
    drop(result_tx);

    let mut available: VecDeque<String> = (0..config.workers)
        .map(|index| format!("worker:{index}"))
        .collect();
    let mut active = HashMap::<String, ActiveWork>::new();
    let mut resource_block_diagnostics = HashSet::<String>::new();
    let mut last_tenant = running.last_dispatched_tenant.clone();
    let mut last_heartbeat = now_unix_ms();
    let mut last_parked_check = now_unix_ms();
    loop {
        while let Ok(completion) = result_rx.try_recv() {
            active.remove(&completion.work_id);
            available.push_back(completion.worker_id.clone());
            match completion.outcome {
                WorkerOutcome::Result(Ok(report)) => {
                    debug_assert_eq!(
                        report.work_item_id.as_deref(),
                        Some(completion.work_id.as_str())
                    );
                    let state = report_work_state(&report);
                    if optional_arg(args, "--failpoint").as_deref()
                        == Some("after_case_runtime_terminal_before_workitem_terminal_commit")
                        && state.is_terminal()
                    {
                        eprintln!(
                            "runtime_instance_crash_injected: after_case_runtime_terminal_before_workitem_terminal_commit work_id={} checkpoint_status={}",
                            completion.work_id,
                            report.status.as_str()
                        );
                        std::process::exit(122);
                    }
                    if optional_arg(args, "--failpoint").as_deref()
                        == Some("after_case_runtime_awaiting_review_before_workitem_state_commit")
                        && state == RuntimeWorkState::WaitingReview
                    {
                        eprintln!(
                            "runtime_instance_crash_injected: after_case_runtime_awaiting_review_before_workitem_state_commit work_id={}",
                            completion.work_id
                        );
                        std::process::exit(123);
                    }
                    if optional_arg(args, "--failpoint").as_deref()
                        == Some("after_case_runtime_waiting_effect_before_workitem_state_commit")
                        && state == RuntimeWorkState::WaitingEffect
                    {
                        eprintln!(
                            "runtime_instance_crash_injected: after_case_runtime_waiting_effect_before_workitem_state_commit work_id={}",
                            completion.work_id
                        );
                        std::process::exit(124);
                    }
                    store.update_runtime_work_state(
                        &authenticated,
                        &owner_token,
                        &completion.work_id,
                        Some(&completion.worker_id),
                        state,
                        &format!(
                            "{}: {}; run_id={}; case_id={}",
                            report.status.as_str(),
                            report.detail,
                            report.run_id,
                            report.case_id
                        ),
                        now_unix_ms(),
                    )?;
                }
                WorkerOutcome::Result(Err(error)) => {
                    let blocked_by_case_owner = error.contains("case_runtime_admission_active");
                    store.update_runtime_work_state(
                        &authenticated,
                        &owner_token,
                        &completion.work_id,
                        Some(&completion.worker_id),
                        if blocked_by_case_owner {
                            RuntimeWorkState::Blocked
                        } else {
                            RuntimeWorkState::Failed
                        },
                        &format!("worker_error: {error}"),
                        now_unix_ms(),
                    )?;
                }
                WorkerOutcome::Panicked(detail) => {
                    eprintln!(
                        "runtime_worker_panic: worker_id={} work_id={} detail={}",
                        completion.worker_id, completion.work_id, detail
                    );
                    store.fail_runtime_instance_closed(
                        &authenticated,
                        &owner_token,
                        now_unix_ms(),
                        INSTANCE_LEASE_MS,
                        &format!(
                            "worker_panic: worker_id={} work_id={}",
                            completion.worker_id, completion.work_id
                        ),
                    )?;
                    return Err(format!(
                        "runtime_instance_degraded_by_worker_panic: worker_id={} work_id={}",
                        completion.worker_id, completion.work_id
                    ));
                }
            }
        }

        let now = now_unix_ms();
        if now.saturating_sub(last_heartbeat) >= HEARTBEAT_INTERVAL_MS {
            store.heartbeat_runtime_instance(
                &authenticated,
                &owner_token,
                now,
                INSTANCE_LEASE_MS,
            )?;
            last_heartbeat = now;
        }
        let instance = store
            .get_runtime_instance_authorized(&authenticated)?
            .ok_or_else(|| "runtime_instance_missing_during_serve".to_string())?;
        let draining = matches!(instance.lifecycle, RuntimeInstanceLifecycle::Draining)
            || instance.drain_requested_at_unix_ms.is_some();
        if draining && !matches!(instance.lifecycle, RuntimeInstanceLifecycle::Draining) {
            store.begin_runtime_instance_drain(
                &authenticated,
                &owner_token,
                now,
                INSTANCE_LEASE_MS,
            )?;
        }

        if now.saturating_sub(last_parked_check) >= PARKED_RECHECK_MS {
            let items = store.list_runtime_work_authorized(&authenticated)?;
            for item in items.into_iter().filter(|item| {
                matches!(
                    item.state,
                    RuntimeWorkState::WaitingReview
                        | RuntimeWorkState::WaitingEffect
                        | RuntimeWorkState::Blocked
                        | RuntimeWorkState::Queued
                )
            }) {
                let case_state = store
                    .get_case_state(&item.case_id)?
                    .ok_or_else(|| "runtime_work_case_missing".to_string())?;
                let target = if case_state.lifecycle == CaseLifecycle::Closed
                    || case_state.cancellation.is_some()
                {
                    Some((RuntimeWorkState::Cancelled, "case_cancelled_or_closed"))
                } else {
                    match item.state {
                        RuntimeWorkState::WaitingReview if !has_pending_review(&item, &store)? => {
                            Some((RuntimeWorkState::Queued, "review_resolved_requeue"))
                        }
                        RuntimeWorkState::WaitingEffect
                            if now.saturating_sub(item.updated_at_unix_ms) >= PARKED_RECHECK_MS =>
                        {
                            Some((RuntimeWorkState::Queued, "effect_reconciliation_retry"))
                        }
                        RuntimeWorkState::Blocked if work_policy_is_usable(&item, &store)? => {
                            if item
                                .last_stop_reason
                                .contains("case_runtime_admission_active")
                            {
                                case_runtime_admission_is_available(&item, &store)?.then_some((
                                    RuntimeWorkState::Queued,
                                    "case_runtime_admission_available",
                                ))
                            } else {
                                Some((RuntimeWorkState::Queued, "policy_repaired_requeue"))
                            }
                        }
                        _ => None,
                    }
                };
                if let Some((target, reason)) = target {
                    store.update_runtime_work_state(
                        &authenticated,
                        &owner_token,
                        &item.work_id,
                        None,
                        target,
                        reason,
                        now,
                    )?;
                }
            }
            last_parked_check = now;
        }

        if !draining {
            loop {
                let Some(worker_id) = available.pop_front() else {
                    break;
                };
                let items = store.list_runtime_work_authorized(&authenticated)?;
                let selected = select_fair_work(
                    &items,
                    &active,
                    last_tenant.as_deref(),
                    config.max_active_per_tenant,
                )?;
                let Some((candidate, root, dispatch_reason)) = selected else {
                    for candidate in items
                        .iter()
                        .filter(|item| matches!(item.state, RuntimeWorkState::Queued))
                    {
                        let root = work_resource_root(candidate)?;
                        if !active.is_empty()
                            && !resource_dispatchable(root.as_deref(), &active)
                            && resource_block_diagnostics.insert(candidate.work_id.clone())
                        {
                            println!(
                                "runtime_dispatch_blocked: work_id={} reason=serialized_due_to_resource_overlap_or_unknown_relation",
                                candidate.work_id
                            );
                        }
                    }
                    available.push_front(worker_id);
                    break;
                };
                let dispatch_principal = authenticate_local()?;
                if dispatch_principal.projected_principal_id() != principal_id {
                    return Err("runtime_instance_kernel_principal_changed".to_string());
                }
                let claimed = match store.claim_runtime_work(
                    &dispatch_principal,
                    &owner_token,
                    &candidate.work_id,
                    &worker_id,
                    now_unix_ms(),
                ) {
                    Ok(claimed) => claimed,
                    Err(error)
                        if error == "runtime_case_already_active"
                            || error == "runtime_tenant_active_capacity_exhausted" =>
                    {
                        available.push_front(worker_id);
                        break;
                    }
                    Err(error) => return Err(error),
                };
                println!(
                    "runtime_dispatch: work_id={} worker_id={} reason={}",
                    claimed.work_id, worker_id, dispatch_reason
                );
                resource_block_diagnostics.remove(&claimed.work_id);
                last_tenant = Some(claimed.tenant_id.clone());
                active.insert(
                    claimed.work_id.clone(),
                    ActiveWork {
                        case_id: claimed.case_id.clone(),
                        tenant_id: claimed.tenant_id.clone(),
                        resource_root: root,
                    },
                );
                if optional_arg(args, "--failpoint").as_deref()
                    == Some("after_work_running_before_case_admission")
                {
                    eprintln!(
                        "runtime_instance_crash_injected: after_work_running_before_case_admission"
                    );
                    std::process::exit(121);
                }
                if job_tx.send((worker_id, claimed)).is_err() {
                    return Err("runtime_worker_channel_closed".to_string());
                }
            }
        }
        if draining && active.is_empty() {
            break;
        }
        thread::sleep(Duration::from_millis(SCHEDULER_TICK_MS));
    }
    drop(job_tx);
    for handle in handles {
        handle
            .join()
            .map_err(|_| "runtime_worker_thread_panicked".to_string())?;
    }
    let stopped = store.stop_runtime_instance(&authenticated, &owner_token, now_unix_ms())?;
    println!("runtime_instance_id: {}", stopped.instance_id);
    println!("state: stopped");
    Ok(())
}

pub(super) fn dispatch(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("serve") => runtime_serve(&args[1..]),
        Some("submit") => runtime_submit(&args[1..]),
        Some("status") => runtime_status(&args[1..]),
        Some("queue") => runtime_queue(&args[1..]),
        Some("stop") => runtime_stop(&args[1..]),
        _ => Err("usage: yai runtime <serve|submit|status|queue|stop> ...".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(tenant: &str, case_id: &str, sequence: u64) -> RuntimeWorkItem {
        RuntimeWorkItem {
            schema: "yai.runtime_work_item.v1".to_string(),
            work_id: format!("work:{tenant}:{case_id}:{sequence}"),
            integrity_digest: String::new(),
            request_id: format!("request:{sequence}"),
            request_digest: "digest".to_string(),
            principal_id: "principal:test".to_string(),
            tenant_id: tenant.to_string(),
            case_id: case_id.to_string(),
            participant_id: "participant:model".to_string(),
            attachment_id: "resource:fs".to_string(),
            journal_path: "/tmp/journal".to_string(),
            task: "task".to_string(),
            budgets: RuntimeCaseBudgets {
                max_invocations: 1,
                max_operations: 1,
                max_semantic_units: 1,
                max_resident_items: 1,
                max_estimated_input_units: 1,
                max_provider_retries: 0,
                max_runtime_ms: None,
                stop_on_deny: false,
                continue_after_malformed: false,
            },
            failpoint: None,
            enqueue_sequence: sequence,
            state: RuntimeWorkState::Queued,
            attempt_count: 0,
            runtime_instance_id: None,
            runtime_owner_token: None,
            worker_id: None,
            last_stop_reason: String::new(),
            enqueued_at_unix_ms: 1,
            updated_at_unix_ms: 1,
        }
    }

    #[test]
    fn tenant_round_robin_and_fifo_are_deterministic() {
        let items = vec![
            item("tenant:a", "case:a1", 1),
            item("tenant:a", "case:a2", 2),
            item("tenant:b", "case:b1", 3),
        ];
        let active = HashMap::new();
        let roots =
            |item: &RuntimeWorkItem| Ok(Some(PathBuf::from(format!("/tmp/{}", item.case_id))));
        let first = select_fair_work_with_roots(&items, &active, None, 2, roots)
            .unwrap()
            .unwrap();
        assert_eq!(first.0.work_id, "work:tenant:a:case:a1:1");
        let second = select_fair_work_with_roots(&items, &active, Some("tenant:a"), 2, roots)
            .unwrap()
            .unwrap();
        assert_eq!(second.0.work_id, "work:tenant:b:case:b1:3");

        let mut tenant_limited = HashMap::new();
        tenant_limited.insert(
            "work:active-a".to_string(),
            ActiveWork {
                case_id: "case:a0".to_string(),
                tenant_id: "tenant:a".to_string(),
                resource_root: Some(PathBuf::from("/tmp/case-a0")),
            },
        );
        let selected = select_fair_work_with_roots(&items, &tenant_limited, None, 1, roots)
            .unwrap()
            .unwrap();
        assert_eq!(selected.0.work_id, "work:tenant:b:case:b1:3");
    }

    #[test]
    fn overlapping_or_unknown_resources_serialize() {
        let mut active = HashMap::new();
        active.insert(
            "work:a".to_string(),
            ActiveWork {
                case_id: "case:a".to_string(),
                tenant_id: "tenant:a".to_string(),
                resource_root: Some(PathBuf::from("/data")),
            },
        );
        assert!(!resource_dispatchable(
            Some(Path::new("/data/sub")),
            &active
        ));
        assert!(!resource_dispatchable(None, &active));
        assert!(resource_dispatchable(Some(Path::new("/other")), &active));
    }

    #[test]
    fn h13_durable_cursor_prevents_restart_bias_and_repeated_crash_starvation() {
        let items = vec![
            item("tenant:a", "case:a1", 1),
            item("tenant:a", "case:a2", 2),
            item("tenant:a", "case:a3", 3),
            item("tenant:a", "case:a4", 4),
            item("tenant:b", "case:b1", 5),
            item("tenant:b", "case:b2", 6),
        ];
        let active = HashMap::new();
        let roots = |_item: &RuntimeWorkItem| Ok(Some(PathBuf::from("/tmp/disjoint")));
        let after_a_restart =
            select_fair_work_with_roots(&items, &active, Some("tenant:a"), 4, roots)
                .unwrap()
                .unwrap();
        assert_eq!(after_a_restart.0.tenant_id, "tenant:b");
        let after_b_restart =
            select_fair_work_with_roots(&items, &active, Some("tenant:b"), 4, roots)
                .unwrap()
                .unwrap();
        assert_eq!(after_b_restart.0.tenant_id, "tenant:a");
    }

    #[test]
    fn h13_scheduler_terminal_history_scale_is_characterized() {
        for terminal_count in [100usize, 1_000, 5_000] {
            let mut items = Vec::with_capacity(terminal_count + 2);
            for sequence in 1..=terminal_count {
                let mut terminal = item("tenant:history", "case:history", sequence as u64);
                terminal.state = RuntimeWorkState::Completed;
                items.push(terminal);
            }
            items.push(item("tenant:a", "case:active-a", terminal_count as u64 + 1));
            items.push(item("tenant:b", "case:active-b", terminal_count as u64 + 2));
            let started = std::time::Instant::now();
            let selected = select_fair_work_with_roots(
                &items,
                &HashMap::new(),
                Some("tenant:a"),
                2,
                |_item| Ok(Some(PathBuf::from("/tmp/disjoint"))),
            )
            .unwrap()
            .unwrap();
            let elapsed = started.elapsed();
            assert_eq!(selected.0.tenant_id, "tenant:b");
            println!(
                "h13_scheduler_scale: terminal_items={terminal_count} total_items={} selector_elapsed_us={} idle_tick_ms={SCHEDULER_TICK_MS}",
                items.len(),
                elapsed.as_micros()
            );
        }
    }
}
