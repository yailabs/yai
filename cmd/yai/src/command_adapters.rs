//! Adapters from stable CLI operation IDs to existing YAI command owners.
//!
//! Purpose:
//!   Preserve proven domain and engineering handlers while the registry owns
//!   command spelling, syntax, lane selection and visibility.
//!
//! Ownership:
//!   Operation-ID adaptation only. Existing plain output remains a bounded
//!   compatibility contract for advanced/plumbing commands.
//!
//! Boundary:
//!   Does not own command paths, parsing, product help, semantic truth, daemon
//!   internals or public SDK shape. This is not an alternate or legacy CLI.
//!
//! Status:
//!   active

use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::{CStr, CString};
use std::fmt::Write as FmtWrite;
use std::fs::{self, OpenOptions};
use std::io::{IsTerminal, Read, Write};
use std::net::TcpStream;
use std::os::raw::{c_char, c_int, c_void};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::Command;

use yai_core_engine::compatibility::{
    inspect_legacy_jsonl, legacy_summary_has_marker, parse_legacy_summary_fields,
};
use yai_core_engine::context::{
    build_context_frame, compile_projection, render_openai_compatible, ContextFrame,
    ContinuationDisposition, DerivedProjectionInput, InvocationOutputContract, Projection,
    ProjectionPurpose, ProjectionRequest, ProviderContinuationReference, ProviderModelProfile,
    RenderedInput, SemanticContextArtifact,
};
use yai_core_engine::graph::GraphSummary;
use yai_core_engine::journal::{Journal, JournalInspection, JOURNAL_RECORD_SCHEMA};
use yai_core_engine::memory::{
    derive_operational_memory, retrieve_operational_memory, MemorySummary, OperationalMemoryEntry,
    OperationalMemoryLifecycle, RetrievalQualification, DEFAULT_RETRIEVAL_LIMIT,
};
use yai_core_engine::projection::ProjectionSummary;
use yai_core_engine::query::{QueryFilter, QueryResult};
use yai_core_engine::reconcile::ReconcileSummary;
use yai_core_engine::record::{Record, RecordKind};
use yai_core_engine::residency::{
    apply_residency_plan, plan_residency, ResidencyPlan, ResidencyRequest,
    DEFAULT_MAX_RESIDENT_ITEMS, DEFAULT_SEMANTIC_UNIT_BUDGET,
};
use yai_core_engine::store::lmdb::{
    GraphMaterializeReport, LmdbRecordStore, RecordStoreStatusKind, ReplayMetadata,
    RuntimeGraphEdge, RuntimeGraphLoadResult, StoredRecordEnvelope, GRAPH_RELATION_SCHEMA,
    GRAPH_RELATION_STORE_NAME, RECORD_SCHEMA,
};
use yai_core_engine::store::Store;
use yai_core_engine::transition::{
    CaseLifecycle, InterpretationAuthority, PendingTransition, ProviderInvocationLineage,
    ReviewActionKind, TransitionPayload, TransitionProvenance, TransitionSource,
};

const ANSI_RESET: &str = "\x1b[0m";
const ANSI_BOLD: &str = "\x1b[1m";
const ANSI_DIM: &str = "\x1b[2m";
const ANSI_CYAN: &str = "\x1b[36m";
const ANSI_BLUE: &str = "\x1b[34m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_MAGENTA: &str = "\x1b[35m";
const FACT_SCHEMA: &str = "yai.fact.v1";
const FACT_TABLES: &[&str] = &[
    "fact_receipt",
    "fact_decision",
    "fact_projection",
    "fact_carrier_outcome",
    "fact_divergence",
    "fact_replay",
    "fact_runtime_graph",
    "fact_model_behavior",
    "fact_policy_outcome",
    "fact_memory_quality",
    "fact_retrieval_quality",
    "fact_provider_runtime",
];
const FACT_COMMON_COLUMNS: &[&str] = &[
    "transaction_time",
    "valid_time_start",
    "valid_time_end",
    "known_at",
    "status",
    "revision_of",
    "superseded_by",
    "retracted_by",
];
const FACT_VALID_TIME_END_SENTINEL: u128 = 0;

unsafe extern "C" {
    fn linenoise(prompt: *const c_char) -> *mut c_char;
    fn linenoiseFree(ptr: *mut c_void);
    fn linenoiseHistoryAdd(line: *const c_char) -> c_int;
    fn linenoiseHistorySetMaxLen(len: c_int) -> c_int;
    #[cfg(unix)]
    fn kill(pid: c_int, sig: c_int) -> c_int;
}

fn print_info() {
    println!(concat!(
        "yai: technical YAI control command\n",
        "status: SPINE.51 Fact Plane Freeze\n",
        "ownership: Rust operational CLI plus Rust data engine\n",
        "canonical_state: LMDB yai.transition.v10 plus atomically materialized yai.case_state.v10\n",
        "effect_paths: typed filesystem.write with Case-native human review and no product review bypass\n",
        "semantic_context: typed yai.projection.v5 plus yai.context_frame.v5 derived from CaseState, Tenant domain, qualified memory, review posture and ResidencyPlan\n",
        "operational_memory: yai.operational_memory.v1 derived, provenance-bound, droppable and rebuildable\n",
        "governance_intake: immutable source content plus Tenant-scoped yai.policy_artifact.v5 lifecycle\n",
        "case_governance: Tenant-exact bindings plus yai.effective_policy.v3 and policy-bound authority admission\n",
        "security: invocation-scoped local POSIX Principal projected into immutable Tenant security domains\n",
        "provider-runtime: provider-specific rendering and real OpenAI-compatible HTTP invocation with typed frame lineage"
    ));
}

fn bool_word(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn parse_pid_arg(args: &[String]) -> Result<i32, String> {
    let value = optional_arg(args, "--pid").ok_or_else(|| "--pid is required".to_string())?;
    value
        .parse::<i32>()
        .map_err(|_| "--pid must be an integer".to_string())
}

#[cfg(unix)]
fn process_state_for_pid(pid: i32) -> &'static str {
    if pid <= 0 {
        return "not_found";
    }
    let result = unsafe { kill(pid as c_int, 0) };
    if result == 0 {
        return "running";
    }
    if let Some(errno) = std::io::Error::last_os_error().raw_os_error() {
        if errno == 3 {
            return "not_found";
        }
        if errno == 1 {
            return "permission_denied";
        }
    }
    let proc_path = PathBuf::from(format!("/proc/{pid}"));
    if PathBuf::from("/proc").is_dir() && !proc_path.exists() {
        return "not_found";
    }
    "unknown"
}

#[cfg(not(unix))]
fn process_state_for_pid(_pid: i32) -> &'static str {
    "unknown"
}

fn expected_matches_observed(expected: &str, observed: &str) -> bool {
    expected == observed || (expected == "stopped" && observed != "running")
}

fn divergence_candidate_for(expected: &str, observed: &str) -> &'static str {
    match (expected, observed) {
        ("stopped", "running") => "expected_stopped_but_running",
        ("running", "not_found") => "expected_running_but_not_found",
        _ => "unknown",
    }
}

fn process_observe(args: &[String]) -> Result<(), String> {
    let pid = parse_pid_arg(args)?;
    let state = process_state_for_pid(pid);
    println!("process_ref: process:{pid}");
    println!("pid: {pid}");
    println!("state: {state}");
    println!("owner_scope: external_observed");
    println!("carrier_family: process");
    println!("outcome: observed");
    println!("receipt_required: yes");
    println!("enforcement: none");
    println!("observation_is_enforcement: false");
    Ok(())
}

fn observe_compare_process(args: &[String]) -> Result<(), String> {
    let pid = parse_pid_arg(args)?;
    let expected =
        optional_arg(args, "--expected").ok_or_else(|| "--expected is required".to_string())?;
    if expected != "running" && expected != "stopped" {
        return Err("--expected must be running or stopped".to_string());
    }
    let observed = process_state_for_pid(pid);
    let matched = expected_matches_observed(&expected, observed);
    println!("observation_target: process");
    println!("pid: {pid}");
    println!("expected_state: {expected}");
    println!("observed_state: {observed}");
    println!("result: {}", if matched { "matched" } else { "mismatch" });
    println!("enforcement: none");
    println!("observation_is_enforcement: false");
    if !matched {
        println!(
            "divergence_candidate: {}",
            divergence_candidate_for(&expected, observed)
        );
        println!("severity: warning");
        println!("silent_repair: false");
    }
    Ok(())
}

fn process_signal(args: &[String]) -> Result<(), String> {
    let pid = parse_pid_arg(args)?;
    let signal =
        optional_arg(args, "--signal").ok_or_else(|| "--signal is required".to_string())?;
    let signal = signal.to_uppercase();
    let dry_run = args.iter().any(|arg| arg == "--dry-run");
    println!("op: process.signal");
    println!("pid: {pid}");
    println!("signal: {signal}");
    if dry_run {
        println!("dry_run: true");
        println!("carrier_family: process");
        println!("lane: process_lane");
        println!("dispatch_status: routable");
        println!("decision_required: true");
        println!("carrier_attempted: false");
        println!("expected_receipt: process_signal_receipt");
        return Ok(());
    }
    println!("decision: deny");
    println!("carrier_attempted: false");
    println!("outcome: blocked");
    println!("reason: unsafe_process_target");
    Ok(())
}

fn yai_home() -> PathBuf {
    std::env::var_os("YAI_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".yai")
        })
}

fn hot_state_path() -> PathBuf {
    yai_home().join("run").join("hot-state.json")
}

fn record_store_path() -> PathBuf {
    yai_home().join("store").join("lmdb")
}

fn facts_store_dir() -> PathBuf {
    yai_home().join("store").join("facts")
}

fn facts_store_path() -> PathBuf {
    facts_store_dir().join("yai-facts.duckdb")
}

fn replay_report_dir() -> PathBuf {
    yai_home().join("store").join("replay").join("reports")
}

fn replay_report_path(journal_identity: &str) -> PathBuf {
    replay_report_dir().join(format!("{journal_identity}.replay-report.json"))
}

fn runtime_graph_rebuild_report_dir() -> PathBuf {
    yai_home()
        .join("store")
        .join("graph")
        .join("rebuild-reports")
}

fn runtime_graph_rebuild_report_path(case_ref: &str) -> PathBuf {
    runtime_graph_rebuild_report_dir().join(format!(
        "{}.runtime-graph-rebuild-report.json",
        safe_case_ref_for_filename(case_ref)
    ))
}

fn safe_case_ref_for_filename(case_ref: &str) -> String {
    case_ref
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

struct RecordStoreStatus {
    path: PathBuf,
    backend: &'static str,
    status: &'static str,
}

fn record_store_status() -> RecordStoreStatus {
    let path = record_store_path();
    let status = LmdbRecordStore::status(&path);
    RecordStoreStatus {
        path,
        backend: status.backend,
        status: status.status.as_str(),
    }
}

fn print_store_status() {
    let status = record_store_status();
    println!("record_store_backend: {}", status.backend);
    println!("record_store_status: {}", status.status);
    println!("record_store_path: {}", status.path.display());
    println!("canonical_authority: lmdb_transaction_authority_v1");
    println!("transition_schema: yai.transition.v8");
    println!("case_state_schema: yai.case_state.v8");
    println!("legacy_record_schema: yai.record.v1");
    if status.status == "ready" {
        println!("canonical_databases: transitions_by_id,case_transition_sequence,case_state,security_principals_by_id,tenants_by_id,tenant_memberships,security_events_by_id");
        println!("indexes: records_by_id,records_by_case,records_by_kind,records_by_subject,records_by_receipt");
        println!("legacy_databases: records_by_id,records_by_case,records_by_kind,records_by_subject,records_by_receipt,legacy_compatibility_payloads");
        println!("derived_databases: graph_relations_by_id,graph_relations_by_case,graph_relations_by_kind");
    }
}

fn print_store_summary() -> Result<(), String> {
    let status = LmdbRecordStore::status(record_store_path());
    println!("record_store_backend: {}", status.backend);
    println!("record_store_status: {}", status.status.as_str());
    println!("record_store_path: {}", status.path.display());
    if status.status != RecordStoreStatusKind::Ready {
        println!("records_total: 0");
        println!("records_by_case: 0");
        println!("records_by_kind: 0");
        println!("records_by_subject: 0");
        println!("records_by_receipt: 0");
        println!("transitions_total: 0");
        println!("cases_materialized: 0");
        println!("legacy_compatibility_payloads: 0");
        return Ok(());
    }
    let store = LmdbRecordStore::open(&status.path)?;
    let summary = store.summary()?;
    let canonical = store.canonical_summary()?;
    println!("records_total: {}", summary.records_total);
    println!("records_by_case: {}", summary.records_by_case);
    println!("records_by_kind: {}", summary.records_by_kind);
    println!("records_by_subject: {}", summary.records_by_subject);
    println!("records_by_receipt: {}", summary.records_by_receipt);
    println!("transitions_total: {}", canonical.transitions_total);
    println!("cases_materialized: {}", canonical.cases_materialized);
    println!(
        "legacy_compatibility_payloads: {}",
        canonical.legacy_compatibility_payloads
    );
    Ok(())
}

#[path = "analytics.rs"]
mod analytics;
use analytics::*;

fn print_non_ready_record_store(status: &yai_core_engine::store::lmdb::RecordStoreStatus) {
    println!("record_store_backend: {}", status.backend);
    println!("record_store_status: {}", status.status.as_str());
    println!("record_store_path: {}", status.path.display());
}

fn store_record_get(args: &[String]) -> Result<(), String> {
    let record_id = args
        .first()
        .filter(|value| !value.starts_with("--"))
        .ok_or_else(|| "record id is required".to_string())?;
    let status = LmdbRecordStore::status(record_store_path());
    if status.status != RecordStoreStatusKind::Ready {
        print_non_ready_record_store(&status);
        return Ok(());
    }
    let store = LmdbRecordStore::open(&status.path)?;
    let Some(record) = store.get_record_by_id(record_id)? else {
        println!("record: not_found");
        return Ok(());
    };
    println!("schema: {}", record.schema);
    println!("record_id: {}", record.record_id);
    println!("record_kind: {}", record.record_kind);
    println!("case_ref: {}", record.case_ref);
    println!("source:");
    println!(
        "  plane: {}",
        json_string_or(&record.raw_json, "plane", "unknown")
    );
    println!(
        "  ref: {}",
        json_string_or(&record.raw_json, "ref", "unknown")
    );
    println!("payload:");
    println!(
        "  summary: {}",
        json_string_or(&record.raw_json, "summary", "unknown")
    );
    println!("envelope: {}", record.raw_json);
    Ok(())
}

fn store_record_list(args: &[String]) -> Result<(), String> {
    let case_ref = optional_arg(args, "--case");
    let record_kind = optional_arg(args, "--kind");
    let subject_ref = optional_arg(args, "--subject");
    let receipt_ref = optional_arg(args, "--receipt");
    let filter_count = [
        case_ref.is_some(),
        record_kind.is_some(),
        subject_ref.is_some(),
        receipt_ref.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count();
    if filter_count != 1 {
        return Err("provide exactly one of --case, --kind, --subject or --receipt".to_string());
    }
    let limit = parse_limit(args)?;
    let status = LmdbRecordStore::status(record_store_path());
    if status.status != RecordStoreStatusKind::Ready {
        print_non_ready_record_store(&status);
        return Ok(());
    }
    let store = LmdbRecordStore::open(&status.path)?;
    let result = if let Some(case_ref) = case_ref.as_deref() {
        let result = store.list_records_by_case(case_ref, limit)?;
        println!("filter: case");
        println!("filter_value: {case_ref}");
        result
    } else if let Some(record_kind) = record_kind.as_deref() {
        if RecordKind::from_str(record_kind).is_none() {
            return Err(format!("unknown record kind: {record_kind}"));
        }
        let result = store.list_records_by_kind(record_kind, limit)?;
        println!("filter: kind");
        println!("filter_value: {record_kind}");
        result
    } else if let Some(subject_ref) = subject_ref.as_deref() {
        let result = store.list_records_by_subject(subject_ref, limit)?;
        println!("filter: subject");
        println!("filter_value: {subject_ref}");
        result
    } else {
        let receipt_ref = receipt_ref.as_deref().unwrap_or_default();
        let result = store.list_records_by_receipt(receipt_ref, limit)?;
        println!("filter: receipt");
        println!("filter_value: {receipt_ref}");
        result
    };
    println!("records_total: {}", result.records_total);
    println!("limit: {limit}");
    if result.records.is_empty() {
        println!("records: none");
    } else {
        println!("records:");
        for record in result.records {
            println!("- record_id: {}", record.record_id);
            println!("  record_kind: {}", record.record_kind);
            println!("  case_ref: {}", record.case_ref);
        }
    }
    Ok(())
}

fn parse_limit(args: &[String]) -> Result<usize, String> {
    let limit = optional_arg(args, "--limit").unwrap_or_else(|| "20".to_string());
    let parsed = limit
        .parse::<usize>()
        .map_err(|_| format!("invalid --limit value: {limit}"))?;
    if parsed == 0 {
        return Err("--limit must be greater than zero".to_string());
    }
    Ok(parsed)
}

fn json_string_field(content: &str, key: &str) -> Option<String> {
    let marker = format!("\"{key}\":\"");
    let start = content.find(&marker)? + marker.len();
    let rest = &content[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn json_number_field(content: &str, key: &str) -> Option<String> {
    let marker = format!("\"{key}\":");
    let start = content.find(&marker)? + marker.len();
    let rest = &content[start..];
    let end = rest
        .find(|ch: char| !(ch.is_ascii_digit()))
        .unwrap_or(rest.len());
    (end > 0).then(|| rest[..end].to_string())
}

fn json_string_or(content: &str, key: &str, fallback: &str) -> String {
    json_string_field(content, key)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

struct HotSnapshotStatus {
    status: &'static str,
    reason: &'static str,
    content: Option<String>,
}

fn validate_hot_snapshot(content: &str) -> bool {
    let trimmed = content.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return false;
    }
    if json_string_field(content, "schema").as_deref() != Some("yai.hot_state.v1") {
        return false;
    }
    let required_strings = [
        "hot_state_id",
        "case_ref",
        "case_session_id",
        "case_context_id",
        "case_session_status",
        "case_world_status",
        "case_context_status",
        "projection_freshness",
        "projection_stale_reason",
    ];
    if required_strings
        .iter()
        .any(|key| json_string_field(content, key).is_none())
    {
        return false;
    }
    let required_numbers = ["case_version", "updated_at_unix_ms"];
    !required_numbers
        .iter()
        .any(|key| json_number_field(content, key).is_none())
}

fn hot_snapshot_status(path: &std::path::Path) -> HotSnapshotStatus {
    if !path.is_file() {
        return HotSnapshotStatus {
            status: "unavailable",
            reason: "missing_snapshot",
            content: None,
        };
    }
    match fs::read_to_string(path) {
        Ok(content) if validate_hot_snapshot(&content) => HotSnapshotStatus {
            status: "active",
            reason: "none",
            content: Some(content),
        },
        Ok(content) => HotSnapshotStatus {
            status: "unavailable",
            reason: "invalid_snapshot",
            content: Some(content),
        },
        Err(_) => HotSnapshotStatus {
            status: "unavailable",
            reason: "unreadable_snapshot",
            content: None,
        },
    }
}

#[derive(Clone, Debug)]
struct ProjectionFreshnessView {
    freshness: String,
    stale_reason: String,
    policy: String,
    consumer: String,
    source: String,
}

fn projection_policy_for(consumer: &str, freshness: &str, stale_reason: &str) -> &'static str {
    if freshness == "fresh" && stale_reason == "none" {
        return "usable";
    }
    if matches!(consumer, "operator" | "audit" | "debug") {
        return "refresh_recommended";
    }
    if !matches!(consumer, "model" | "agent") {
        return "unknown";
    }
    if freshness == "rebuilding" {
        return "refresh_required";
    }
    match stale_reason {
        "new_receipt_after_projection"
        | "new_decision_after_projection"
        | "new_memory_after_projection" => "refresh_required",
        "new_authority_scope_after_projection"
        | "new_divergence_after_projection"
        | "thread_changed"
        | "manual_refresh_required"
        | "unknown" => "blocked_for_model",
        _ => "blocked_for_model",
    }
}

fn projection_freshness_view(case_ref: &str, consumer: &str) -> ProjectionFreshnessView {
    let status = hot_snapshot_status(&hot_state_path());
    if status.status == "active" {
        if let Some(content) = status.content.as_deref() {
            let hot_case = json_string_or(content, "case_ref", "");
            if !case_ref.is_empty() && hot_case == case_ref {
                let freshness = json_string_or(content, "projection_freshness", "unknown");
                let stale_reason = json_string_or(content, "projection_stale_reason", "unknown");
                return ProjectionFreshnessView {
                    policy: projection_policy_for(consumer, &freshness, &stale_reason).to_string(),
                    freshness,
                    stale_reason,
                    consumer: consumer.to_string(),
                    source: "hot_state".to_string(),
                };
            }
        }
    }
    ProjectionFreshnessView {
        freshness: "fresh".to_string(),
        stale_reason: "none".to_string(),
        policy: projection_policy_for(consumer, "fresh", "none").to_string(),
        consumer: consumer.to_string(),
        source: "projection_record".to_string(),
    }
}

fn print_hot_status() -> Result<(), String> {
    let path = hot_state_path();
    let status = hot_snapshot_status(&path);
    if status.status != "active" {
        println!("hot_state: unavailable");
        println!("reason: {}", status.reason);
        println!("snapshot: {}", path.display());
        println!("snapshot_path: {}", path.display());
        println!("snapshot_status: {}", status.status);
        println!("schema: unknown");
        println!("case_session: unknown");
        println!("case_world: unknown");
        println!("case_context: unknown");
        println!("active_thread: unknown");
        println!("participant_view: unknown");
        println!("projection: unknown");
        println!("projection_policy: unknown");
        println!("freshness_policy: unknown");
        println!("stale_reason: unknown");
        return Ok(());
    }

    let content = status
        .content
        .as_deref()
        .ok_or_else(|| "valid hot snapshot was not loaded".to_string())?;
    println!("hot_state: active");
    println!("snapshot: {}", path.display());
    println!("snapshot_path: {}", path.display());
    println!("snapshot_status: active");
    println!("schema: {}", json_string_or(content, "schema", "unknown"));
    println!("case: {}", json_string_or(content, "case_ref", "unknown"));
    let case_session_status = json_string_or(content, "case_session_status", "unknown");
    let case_world_status = json_string_or(content, "case_world_status", "unknown");
    let case_context_status = json_string_or(content, "case_context_status", "unknown");
    println!("session: {case_session_status}");
    println!("case_session: {case_session_status}");
    println!("case_world: {case_world_status}");
    println!("context: {case_context_status}");
    println!("case_context: {case_context_status}");
    println!(
        "case_session_id: {}",
        json_string_or(content, "case_session_id", "unknown")
    );
    println!(
        "case_context_id: {}",
        json_string_or(content, "case_context_id", "unknown")
    );
    println!(
        "active_thread: {}",
        json_string_or(content, "active_thread_id", "none")
    );
    println!(
        "participant_view: {}",
        json_string_or(content, "participant_view_frame_id", "none")
    );
    println!(
        "case_version: {}",
        json_number_field(content, "case_version").unwrap_or_else(|| "0".to_string())
    );
    println!(
        "projection: {}",
        json_string_or(content, "projection_freshness", "unknown")
    );
    let projection_freshness = json_string_or(content, "projection_freshness", "unknown");
    let stale_reason = json_string_or(content, "projection_stale_reason", "unknown");
    let freshness_policy = projection_policy_for("model", &projection_freshness, &stale_reason);
    println!("projection_policy: {}", freshness_policy);
    println!("freshness_policy: {freshness_policy}");
    println!("projection_freshness: {}", projection_freshness);
    println!("stale_reason: {stale_reason}");
    println!("projection_stale_reason: {stale_reason}");
    println!(
        "last_record: {}",
        json_string_or(content, "last_record_id", "none")
    );
    println!(
        "last_decision: {}",
        json_string_or(content, "last_decision_id", "none")
    );
    println!(
        "last_receipt: {}",
        json_string_or(content, "last_receipt_id", "none")
    );
    println!(
        "pending_ops: {}",
        json_number_field(content, "pending_op_count").unwrap_or_else(|| "0".to_string())
    );
    println!(
        "pending_obligations: {}",
        json_number_field(content, "pending_obligation_count").unwrap_or_else(|| "0".to_string())
    );
    println!(
        "carrier_locks: {}",
        json_number_field(content, "carrier_lock_count").unwrap_or_else(|| "0".to_string())
    );
    println!(
        "updated_at: {}",
        json_number_field(content, "updated_at_unix_ms").unwrap_or_else(|| "0".to_string())
    );
    Ok(())
}

fn yai_env_file() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("YAI_ENV_FILE")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
    {
        return Some(path);
    }

    let mut dir = std::env::current_dir().ok()?;
    loop {
        let candidate = dir.join(".yai").join("env");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            break;
        }
    }

    let candidate = yai_home().join("env");
    candidate.is_file().then_some(candidate)
}

fn parse_env_assignment(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let line = line.strip_prefix("export ").unwrap_or(line).trim();
    let (key, value) = line.split_once('=')?;
    let key = key.trim();
    if key.is_empty()
        || !key
            .chars()
            .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
        || key.chars().next().is_some_and(|ch| ch.is_ascii_digit())
    {
        return None;
    }

    let value = value.trim();
    let value = if value.len() >= 2 {
        let bytes = value.as_bytes();
        if (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
        {
            &value[1..value.len() - 1]
        } else {
            value
        }
    } else {
        value
    };
    Some((key.to_string(), value.to_string()))
}

fn env_file_var(name: &str) -> Option<String> {
    let path = yai_env_file()?;
    let content = fs::read_to_string(path).ok()?;
    content
        .lines()
        .filter_map(parse_env_assignment)
        .find_map(|(key, value)| (key == name && !value.is_empty()).then_some(value))
}

fn journal_arg(args: &[String]) -> Result<PathBuf, String> {
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--journal" {
            return args
                .get(index + 1)
                .map(PathBuf::from)
                .ok_or_else(|| "--journal requires a path".to_string());
        }
        index += 1;
    }
    Err("--journal <path> is required".to_string())
}

fn named_arg(args: &[String], name: &str) -> Result<String, String> {
    let mut index = 0;
    while index < args.len() {
        if args[index] == name {
            return args
                .get(index + 1)
                .cloned()
                .ok_or_else(|| format!("{name} requires a value"));
        }
        index += 1;
    }
    Err(format!("{name} is required"))
}

fn optional_arg(args: &[String], name: &str) -> Option<String> {
    let mut index = 0;
    while index < args.len() {
        if args[index] == name {
            return args.get(index + 1).cloned();
        }
        index += 1;
    }
    None
}

fn latest_filesystem_journal() -> Option<PathBuf> {
    fn visit(dir: &std::path::Path, best: &mut Option<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit(&path, best);
            } else if path.file_name().and_then(|name| name.to_str()) == Some("journal.jsonl")
                && path
                    .components()
                    .any(|component| component.as_os_str() == "filesystem")
                && best.as_ref().is_none_or(|current| path > *current)
            {
                *best = Some(path);
            }
        }
    }

    let mut best = None;
    visit(std::path::Path::new("build/tmp"), &mut best);
    best
}

fn existing_env_path(name: &str) -> Option<PathBuf> {
    let path = env_var(name).map(PathBuf::from)?;
    if path.is_file() {
        Some(path)
    } else {
        None
    }
}

fn reject_journal_flag(args: &[String], command: &str) -> Result<(), String> {
    if args.iter().any(|arg| arg == "--journal") {
        return Err(format!(
            "{command} does not accept --journal; materialize the case first and pass journal state through YAI_JOURNAL"
        ));
    }
    Ok(())
}

fn case_journal_path(args: &[String], command: &str) -> Result<PathBuf, String> {
    reject_journal_flag(args, command)?;
    if let Some(path) = existing_env_path("YAI_JOURNAL").or_else(latest_filesystem_journal) {
        return Ok(path);
    }
    let case_id = optional_arg(args, "--case").ok_or_else(|| {
        format!("{command} requires an explicit Case when no compatibility journal is supplied")
    })?;
    let directory = yai_home()
        .join("cases")
        .join(yai_core_engine::context::stable_digest(&case_id));
    fs::create_dir_all(&directory)
        .map_err(|error| format!("failed to prepare Case compatibility directory: {error}"))?;
    let path = directory.join("compatibility.jsonl");
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| format!("failed to prepare Case compatibility journal: {error}"))?;
    Ok(path)
}

#[path = "filesystem.rs"]
mod filesystem;
use filesystem::*;

#[path = "replay.rs"]
mod replay;
use replay::*;

#[path = "review.rs"]
mod review;
use review::*;

#[path = "provider.rs"]
mod provider;
use provider::*;

#[path = "controlled_effect.rs"]
mod controlled_effect;
use controlled_effect::*;

#[path = "case_runtime.rs"]
mod case_runtime;
use case_runtime::*;

#[path = "runtime_instance.rs"]
mod runtime_instance;

#[path = "workflow.rs"]
mod workflow;
use workflow::workflow_command;

#[path = "handoff.rs"]
mod handoff;
use handoff::handoff_command;

#[path = "policy.rs"]
mod policy;
use policy::policy_command;

#[path = "case_policy.rs"]
mod case_policy;
use case_policy::case_policy_command;

#[path = "case_lifecycle.rs"]
mod case_lifecycle;
use case_lifecycle::{case_cancel, case_close};

#[path = "security.rs"]
mod security;
use security::{case_security_command, identity_command, security_command, tenant_command};

fn decision_outcome(summary: &str) -> String {
    parse_legacy_summary_fields(summary)
        .remove("decision")
        .unwrap_or_else(|| "unknown".to_string())
}

fn decision_inspect(args: &[String]) -> Result<(), String> {
    let path = journal_arg(args)?;
    let journal = Journal::load_jsonl(&path)
        .map_err(|error| format!("failed to load {}: {error}", path.display()))?;
    let projection = ProjectionSummary::from_journal("control", &journal);
    let decision = journal
        .records()
        .iter()
        .find(|record| record.kind == RecordKind::Decision);
    let basis = journal
        .records()
        .iter()
        .find(|record| record.kind == RecordKind::DecisionBasis);

    if let Some(record) = decision {
        println!("decision: {}", decision_outcome(&record.summary));
    } else {
        println!("decision: none");
    }
    if let Some(record) = basis {
        println!("basis: {}", record.summary);
    } else {
        println!("basis: none");
    }
    println!("obligations: {}", projection.obligation_count);
    println!(
        "receipt_requirements: {}",
        projection.receipt_requirement_count
    );
    Ok(())
}

fn receipt_summary(args: &[String]) -> Result<(), String> {
    let path = journal_arg(args)?;
    let journal = Journal::load_jsonl(&path)
        .map_err(|error| format!("failed to load {}: {error}", path.display()))?;
    let projection = ProjectionSummary::from_journal("receipt", &journal);
    println!("records: {}", projection.source_record_count);
    println!("receipts: {}", projection.receipt_count);
    println!(
        "filesystem_receipts: {}",
        projection.filesystem_receipt_count
    );
    println!("subject_states: {}", projection.subject_state_count);
    println!("effects: {}", projection.effect_count);
    Ok(())
}

#[path = "graph_runtime.rs"]
mod graph_runtime;
use graph_runtime::*;

#[path = "memory_cli.rs"]
mod memory_cli;
use memory_cli::*;

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn reconcile_summary(args: &[String]) -> Result<(), String> {
    let path = journal_arg(args)?;
    let journal = Journal::load_jsonl(&path)
        .map_err(|error| format!("failed to load {}: {error}", path.display()))?;
    let summary = ReconcileSummary::from_journal(&journal);

    println!("records: {}", summary.records);
    println!("divergences: {}", summary.divergences);
    println!("reconciliations: {}", summary.reconciliations);
    println!("critical: {}", summary.critical);
    println!("warnings: {}", summary.warnings);
    Ok(())
}

fn query_filter_from_args(args: &[String]) -> Result<QueryFilter, String> {
    let record_kind = optional_arg(args, "--kind")
        .map(|kind| {
            RecordKind::from_str(&kind).ok_or_else(|| format!("unknown record kind: {kind}"))
        })
        .transpose()?;
    let limit = optional_arg(args, "--limit")
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|error| format!("invalid --limit value: {error}"))
        })
        .transpose()?;

    Ok(QueryFilter {
        case_ref: optional_arg(args, "--case"),
        record_kind,
        limit,
        include_summary: true,
        ..Default::default()
    })
}

fn query_summary(args: &[String]) -> Result<(), String> {
    let path = journal_arg(args)?;
    let journal = Journal::load_jsonl(&path)
        .map_err(|error| format!("failed to load {}: {error}", path.display()))?;
    let filter = QueryFilter::default();
    let result = QueryResult::scan(&journal, &filter);
    println!("records: {}", result.records);
    println!("matched: {}", result.matched);
    println!("returned: {}", result.returned);
    println!("truncated: {}", result.truncated);
    Ok(())
}

fn query_records(args: &[String]) -> Result<(), String> {
    let path = journal_arg(args)?;
    let journal = Journal::load_jsonl(&path)
        .map_err(|error| format!("failed to load {}: {error}", path.display()))?;
    let filter = query_filter_from_args(args)?;
    let include_summary = filter.include_summary;
    let result = QueryResult::scan(&journal, &filter);

    println!("records: {}", result.records);
    println!("matched: {}", result.matched);
    println!("returned: {}", result.returned);
    println!("truncated: {}", result.truncated);
    for record in result.matched_records {
        if include_summary {
            println!("{} {} {}", record.id, record.kind.as_str(), record.summary);
        } else {
            println!("{} {}", record.id, record.kind.as_str());
        }
    }
    Ok(())
}

fn engine_summary(args: &[String]) -> Result<(), String> {
    let path = journal_arg(args)?;
    let journal = Journal::load_jsonl(&path)
        .map_err(|error| format!("failed to load {}: {error}", path.display()))?;
    let store = Store::from_journal(journal);
    let summary = store.engine_summary();
    println!("records: {}", summary.records);
    println!("receipts: {}", summary.receipts);
    println!("graph_edges: {}", summary.graph_edges);
    println!("memory_candidates: {}", summary.memory_candidates);
    println!("projections: {}", summary.projections);
    println!("divergences: {}", summary.divergences);
    Ok(())
}

#[cfg(unix)]
fn daemon_request_response(args: &[String], request: &str) -> Result<String, String> {
    let socket = named_arg(args, "--socket")?;
    let mut stream = UnixStream::connect(&socket)
        .map_err(|error| format!("failed to connect {socket}: {error}"))?;
    stream
        .write_all(format!("{request}\n").as_bytes())
        .map_err(|error| format!("failed to write request: {error}"))?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|error| format!("failed to read response: {error}"))?;
    Ok(response)
}

#[cfg(unix)]
fn daemon_request(args: &[String], request: &str) -> Result<(), String> {
    let response = daemon_request_response(args, request)?;
    print!("{response}");
    Ok(())
}

#[cfg(unix)]
fn daemon_request_and_import_records(args: &[String], request: &str) -> Result<(), String> {
    let response = daemon_request_response(args, request)?;
    print!("{response}");
    if extract_json_string_field(&response, "status").as_deref() != Some("completed") {
        return Ok(());
    }
    let journal_path = extract_json_string_field(&response, "journal_path")
        .ok_or_else(|| "daemon response did not include journal_path".to_string())?;
    import_journal_to_record_store(&PathBuf::from(journal_path))
}

#[cfg(unix)]
fn daemon_request_with_journal(args: &[String], request: &str) -> Result<(), String> {
    let journal = journal_arg(args)?;
    let line = format!(
        "{request} request_id=yai-{request} payload={}",
        journal.display()
    );
    daemon_request(args, &line)
}

#[cfg(not(unix))]
fn daemon_request(_args: &[String], _request: &str) -> Result<(), String> {
    Err("daemon IPC is only implemented on Unix in NEW.11".to_string())
}

#[cfg(not(unix))]
fn daemon_request_and_import_records(_args: &[String], _request: &str) -> Result<(), String> {
    Err("daemon IPC is only implemented on Unix in NEW.11".to_string())
}

#[cfg(not(unix))]
fn daemon_request_with_journal(_args: &[String], _request: &str) -> Result<(), String> {
    Err("daemon IPC is only implemented on Unix in NEW.13".to_string())
}

/// Dispatches an already registry-resolved operation to its existing domain
/// adapter. This match is over stable operation identity, never command text;
/// path and syntax authority remain in `cli::registry`.
pub(crate) fn dispatch_operation(operation_id: &str, args: &[String]) -> Result<(), String> {
    match operation_id {
        "yai.init" | "yai.security.bootstrap_local" => security_command(&args[1..]),
        "yai.identity.whoami" => identity_command(&args[1..]),
        "yai.tenant.list" | "yai.tenant.show" | "yai.tenant.member.add" => {
            tenant_command(&args[1..])
        }
        "yai.case.create" | "yai.case.participant.link_principal" => {
            case_security_command(&args[1..])
        }
        "yai.case.participant.role.add" => provider::case_bind_participant_role(&args[2..]),
        "yai.case.provider.attach" => case_attach_provider(&args[2..]),
        "yai.case.resource.attach_filesystem" => case_attach_filesystem(&args[2..]),
        "yai.case.resource.attach_process" => case_attach_process(&args[2..]),
        operation if operation.starts_with("yai.case.handoff.") => handoff_command(&args[2..]),
        operation if operation.starts_with("yai.case.policy.") => case_policy_porcelain(&args[2..]),
        "yai.case.run" => case_runtime_run(&args[2..]),
        "yai.case.resume" => case_runtime_resume(&args[2..]),
        "yai.case.show" => case_runtime_status(&args[2..]),
        "yai.case.stop" => case_runtime_stop(&args[2..]),
        "yai.case.cancel" => case_cancel(&args[2..]),
        "yai.case.close" => case_close(&args[2..]),
        operation if operation.starts_with("yai.workflow.") => workflow_command(&args[1..]),
        "yai.review.pending" => review_pending(&args[2..]),
        "yai.review.show" => review_show(&args[2..]),
        "yai.review.approve" => review_resolve(&args[2..], ReviewActionKind::Approve),
        "yai.review.deny" => review_resolve(&args[2..], ReviewActionKind::Deny),
        "yai.review.defer" => review_resolve(&args[2..], ReviewActionKind::Defer),
        operation if operation.starts_with("yai.policy.") => policy_command(&args[1..]),
        operation if operation.starts_with("yai.runtime.") => {
            runtime_instance::dispatch(&args[1..])
        }
        "yai.case.enter" => case_enter(&args[2..]),
        "yai.effect.filesystem_write" => controlled_filesystem_write(&args[2..]),
        "yai.effect.process_signal" => controlled_process_signal(&args[2..]),
        "yai.effect.reconcile" => controlled_effect_reconcile(&args[2..]),
        "yai.effect.inspect" => controlled_effect_inspect(&args[2..]),
        "yai.prompt" => prompt_repl(&args[1..]),
        "yai.info" => {
            print_info();
            Ok(())
        }
        "yai.store.status" => {
            print_store_status();
            Ok(())
        }
        "yai.store.summary" => print_store_summary(),
        "yai.store.record.get" => store_record_get(&args[3..]),
        "yai.store.record.list" => store_record_list(&args[3..]),
        "yai.store.tail" => store_tail(&args[2..]),
        "yai.journal.inspect" => journal_inspect(&args[2..]),
        "yai.journal.compatibility_inspect" => journal_compatibility_inspect(&args[2..]),
        "yai.journal.compatibility_import" => journal_compatibility_import(&args[2..]),
        "yai.journal.replay" => journal_replay(&args[2..]),
        "yai.journal.replay_status" => journal_replay_status(&args[2..]),
        "yai.journal.replay_report" => journal_replay_report(&args[2..]),
        "yai.projection.summary" => projection_summary(&args[2..]),
        "yai.projection.inspect" => projection_inspect(&args[2..]),
        "yai.projection.request" => projection_request(&args[2..]),
        "yai.context.inspect" => semantic_context_inspect(&args[2..]),
        "yai.control.summary" => control_summary(&args[2..]),
        "yai.decision.inspect" => decision_inspect(&args[2..]),
        "yai.receipt.summary" => receipt_summary(&args[2..]),
        "yai.reconcile.summary" => reconcile_summary(&args[2..]),
        "yai.query.summary" => query_summary(&args[2..]),
        "yai.query.records" => query_records(&args[2..]),
        "yai.engine.summary" => engine_summary(&args[2..]),
        "yai.hot.status" => print_hot_status(),
        "yai.process.observe" => process_observe(&args[2..]),
        "yai.process.signal" => process_signal(&args[2..]),
        "yai.observe.compare_process" => observe_compare_process(&args[2..]),
        "yai.carrier.fs_read" => carrier_fs_read(&args[2..]),
        "yai.graph.summary" => graph_summary(&args[2..]),
        "yai.graph.schema" => graph_schema(&args[2..]),
        "yai.graph.runtime_status" => graph_runtime_status(&args[2..]),
        "yai.graph.materialize" => graph_materialize(&args[2..]),
        "yai.graph.relations" => graph_relations(&args[2..]),
        "yai.graph.runtime_load" => graph_runtime_load(&args[2..], false),
        "yai.graph.runtime_summary" => graph_runtime_load(&args[2..], true),
        "yai.graph.rebuild" => graph_rebuild(&args[2..]),
        "yai.graph.rebuild_report" => graph_rebuild_report(&args[2..]),
        "yai.graph.fanout" => graph_fanout(&args[2..]),
        "yai.graph.fanin" => graph_fanin(&args[2..]),
        "yai.graph.neighborhood" => graph_neighborhood(&args[2..]),
        "yai.graph.path" => graph_path(&args[2..]),
        "yai.facts.status" => facts_status(&args[2..]),
        "yai.facts.schema" => facts_schema(&args[2..]),
        "yai.facts.init" => facts_init(&args[2..]),
        "yai.facts.extract" => facts_extract(&args[2..]),
        "yai.facts.summary" => facts_summary(&args[2..]),
        "yai.facts.report" => facts_report(&args[2..]),
        "yai.memory.summary" => memory_summary(&args[2..]),
        "yai.memory.rebuild" => memory_rebuild(&args[2..]),
        "yai.memory.clear" => memory_clear(&args[2..]),
        "yai.memory.list" => memory_list(&args[2..]),
        "yai.memory.show" => memory_show(&args[2..]),
        "yai.memory.provenance" => memory_provenance(&args[2..]),
        "yai.memory.retrieve" => memory_retrieve(&args[2..]),
        "yai.daemon.status" => daemon_request(&args[2..], "status"),
        "yai.daemon.info" => daemon_request(&args[2..], "info"),
        "yai.daemon.shutdown" => daemon_request(&args[2..], "shutdown"),
        "yai.daemon.run_minimum_loop" => daemon_request_and_import_records(
            &args[2..],
            "run_minimum_loop request_id=yai-minimum case_ref=case:new12-daemon subject_ref=subject:repo-test",
        ),
        "yai.daemon.run_filesystem_loop" => daemon_request_and_import_records(
            &args[2..],
            "run_filesystem_loop request_id=yai-filesystem case_ref=case:new12-filesystem subject_ref=subject:filesystem-sandbox",
        ),
        "yai.daemon.journal_summary" => {
            daemon_request_with_journal(&args[2..], "journal_summary")
        }
        "yai.daemon.projection_summary" => {
            daemon_request_with_journal(&args[2..], "projection_summary")
        }
        _ => Err(format!("registry_handler_not_resolved:{operation_id}")),
    }
}

fn case_policy_porcelain(args: &[String]) -> Result<(), String> {
    let mut normalized = args.to_vec();
    let mutation = matches!(
        args.first().map(String::as_str),
        Some("bind" | "replace" | "unbind")
    );
    if mutation && optional_arg(args, "--expected-generation").is_none() {
        let case_id = named_arg(args, "--case")?;
        let authenticated = security::authenticate_local()?;
        let state = LmdbRecordStore::open(record_store_path())?
            .get_case_state_authorized(&authenticated, &case_id)?;
        normalized.push("--expected-generation".to_string());
        normalized.push(state.generation.to_string());
    }
    case_policy_command(&normalized)
}
