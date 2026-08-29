//! YAI - control CLI
//!
//! Purpose:
//!   Provide the local technical control command for runtime inspection, daemon
//!   requests and record-plane operator views.
//!
//! Ownership:
//!   Command parsing and user-facing text output for `yai`.
//!
//! Boundary:
//!   Does not own core data-plane truth, daemon internals or public SDK shape.
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

const VERSION: &str = env!("CARGO_PKG_VERSION");
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
        "canonical_state: LMDB yai.transition.v6 plus atomically materialized yai.case_state.v6\n",
        "effect_paths: typed filesystem.write with Case-native human review and no product review bypass\n",
        "semantic_context: typed yai.projection.v4 plus yai.context_frame.v4 derived from CaseState, qualified memory, review posture and ResidencyPlan\n",
        "operational_memory: yai.operational_memory.v1 derived, provenance-bound, droppable and rebuildable\n",
        "governance_intake: immutable yai.policy_source_artifact.v3 plus typed yai.policy_ir.v2 and owner-scoped yai.policy_artifact.v3 lifecycle\n",
        "case_governance: exact bindings plus yai.effective_policy.v2 and policy-bound DecisionBasis/Decision/review/ExecutionGrant admission\n",
        "provider-runtime: provider-specific rendering and real OpenAI-compatible HTTP invocation with typed frame lineage"
    ));
}

fn print_doctor() {
    let yai_home = yai_home();
    let run_dir = yai_home.join("run");
    let store_dir = yai_home.join("store");
    let log_dir = yai_home.join("log");
    let tmp_dir = yai_home.join("tmp");
    let cases_dir = yai_home.join("cases");
    let sockets_dir = yai_home.join("sockets");
    let config_dir = yai_home.join("config");
    let socket = run_dir.join("yaid.sock");
    let hot_state_path = run_dir.join("hot-state.json");
    let record_status = record_store_status();
    let yai_path = std::env::current_exe()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let yaid_path = find_on_path("yaid").unwrap_or_else(|| "not found on PATH".to_string());
    let yaid_found = if find_on_path("yaid").is_some() {
        "found"
    } else {
        "missing"
    };
    let path_status = path_contains_current_bin().unwrap_or(false);
    let runtime_layout_ok = [
        &run_dir,
        &store_dir,
        &log_dir,
        &tmp_dir,
        &cases_dir,
        &sockets_dir,
        &config_dir,
    ]
    .iter()
    .all(|path| path.is_dir());
    let hot_status = hot_snapshot_status(&hot_state_path);

    println!("yai doctor: ok");
    println!("public_semantics: canonical docs plus executable command behavior");
    println!("rust_role: yai operational CLI and data engine");
    println!("journal_mode: legacy compatibility/export only");
    println!("binary_path: {yai_path}");
    println!("yaid_path: {yaid_path}");
    println!("yaid_found: {yaid_found}");
    println!("yai_version: {VERSION}");
    println!("YAI_HOME: {}", yai_home.display());
    println!("YAI_HOME_status: {}", path_state(&yai_home));
    println!("run_dir: {}", path_state_with_path(&run_dir));
    println!("store_dir: {}", path_state_with_path(&store_dir));
    println!("log_dir: {}", path_state_with_path(&log_dir));
    println!("tmp_dir: {}", path_state_with_path(&tmp_dir));
    println!("cases_dir: {}", path_state_with_path(&cases_dir));
    println!("sockets_dir: {}", path_state_with_path(&sockets_dir));
    println!("config_dir: {}", path_state_with_path(&config_dir));
    println!(
        "env_file: {}",
        yai_env_file()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "not found".to_string())
    );
    println!(
        "PATH_status: {}",
        if path_status {
            "current binary dir present"
        } else {
            "warning current binary dir not on PATH"
        }
    );
    println!("daemon_socket_default: {}", socket.display());
    println!(
        "socket_default_status: {}",
        if socket.exists() {
            "present"
        } else {
            "not_present"
        }
    );
    println!("hot_state_path: {}", hot_state_path.display());
    println!("hot_state_status: {}", hot_status.status);
    println!("hot_state_schema_status: {}", hot_status.schema_status);
    println!("hot_state_readable: {}", hot_status.readable);
    println!("record_store_path: {}", record_status.path.display());
    println!("record_store_status: {}", record_status.status);
    println!("record_store_backend: {}", record_status.backend);
    if let Some(content) = hot_status.content.as_deref() {
        println!(
            "case_session_status: {}",
            json_string_or(content, "case_session_status", "unknown")
        );
        println!(
            "case_context_status: {}",
            json_string_or(content, "case_context_status", "unknown")
        );
        let projection_freshness = json_string_or(content, "projection_freshness", "unknown");
        let stale_reason = json_string_or(content, "projection_stale_reason", "unknown");
        println!("projection_freshness: {projection_freshness}");
        println!("stale_reason: {stale_reason}");
        println!(
            "freshness_policy: {}",
            projection_policy_for("model", &projection_freshness, &stale_reason)
        );
    }
    println!(
        "runtime_layout_status: {}",
        if runtime_layout_ok {
            "ok"
        } else {
            "incomplete"
        }
    );
}

fn bool_word(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn print_usage() {
    println!("usage: yai [--version|info|doctor]");
    println!("       yai store status");
    println!("       yai store summary");
    println!("       yai store record get <record_id>");
    println!("       yai store record list --case <case_ref> [--limit <N>]");
    println!("       yai store record list --kind <record_kind> [--limit <N>]");
    println!("       yai store record list --subject <subject_ref> [--limit <N>]");
    println!("       yai store record list --receipt <receipt_ref> [--limit <N>]");
    println!("       yai store tail --journal <path>");
    println!("       yai journal inspect --path <journal.jsonl> [--show-errors]");
    println!("       yai journal compatibility-inspect --path <journal.jsonl>");
    println!("       yai journal compatibility-import --path <journal.jsonl> --target <isolated-lmdb> [--dry-run]");
    println!("       yai journal replay --path <journal.jsonl> [--dry-run]");
    println!("       yai journal replay-status --path <journal.jsonl>");
    println!("       yai journal replay-report --path <journal.jsonl>");
    println!("       yai projection summary --journal <path>");
    println!("       yai projection inspect --journal <path> [--consumer model|operator|audit|debug|agent]");
    println!("       yai projection request --journal <path> --consumer <consumer> --kind <kind>");
    println!("       yai context inspect --id <projection|frame|rendered-input-id>");
    println!("       yai case enter --case <case_ref> --subject <subject_ref> [--consumer model] [--kind model_context] [--shell zsh]");
    println!("       yai case attach-provider --case <case_ref> --subject <subject_ref> --base-url <url> --model <model> [--provider-id <id>] [--api-key-env <env>] [--shell zsh]");
    println!("       yai case attach-filesystem --case <case_ref> --attachment <id> --root <existing-dir> --allow-prefix <relative-dir> --policy-owner <participant> [--require-review] [--policy-id <id>] [--max-bytes <N>]");
    println!("       yai case bind-participant-role --case <case_ref> --participant <participant> --role <role> --as <actor-ref>");
    println!("       yai case policy bind --case <case_ref> --artifact <id> --expected-generation <N> --as <participant> [--reason <reason>]");
    println!("       yai case policy replace --case <case_ref> --binding <id> --artifact <id> --expected-generation <N> --as <participant> [--reason <reason>]");
    println!("       yai case policy unbind --case <case_ref> --binding <id> --expected-generation <N> --as <participant> --reason <reason>");
    println!("       yai case policy status|rebuild --case <case_ref>");
    println!("       yai case run --case <case_ref> --subject <participant> --attachment <id> --prompt <task> [--max-invocations <N>] [--max-operations <N>] [--max-semantic-units <N>] [--max-estimated-input-units <N>]");
    println!("       yai case resume --case <case_ref> [budget overrides]");
    println!("       yai case status --case <case_ref>");
    println!("       yai case stop --case <case_ref>");
    println!("       yai policy ingest <source.json> --as <operator-ref>");
    println!("       yai policy inspect <source-id|artifact-id>");
    println!("       yai policy validate <artifact-id> --as <operator-ref> [--reason <reason>]");
    println!("       yai policy publish <artifact-id> --as <operator-ref> [--reason <reason>]");
    println!("       yai policy retire <artifact-id> --as <operator-ref> --reason <reason>");
    println!("       yai policy list");
    println!("       yai effect filesystem-write --case <case_ref> --subject <provider-participant> --attachment <id> --prompt <text> --base-url <url> --model <model> [--failpoint <name>]");
    println!("       yai effect reconcile --case <case_ref> [--effect <effect-id>] [--retry]");
    println!("       yai effect inspect --case <case_ref> --effect <effect-id>");
    println!("       yai prompt [--once <text>] [--dry-run] [--language-mode auto|none] [--case <case_ref>] [--subject <subject_ref>]");
    println!("       yai prompt [--dry-run] [--language-mode auto|none] [--case <case_ref>] [--subject <subject_ref>] < prompt.txt");
    println!("       yai control summary --journal <path>");
    println!("       yai review pending --case <case_ref>");
    println!("       yai review show <review_id> --case <case_ref>");
    println!("       yai review approve|deny|defer <review_id> --case <case_ref> --as <participant> --reason <reason>");
    println!("       yai decision inspect --journal <path>");
    println!("       yai receipt summary --journal <path>");
    println!("       yai graph summary --journal <path>");
    println!("       yai graph schema");
    println!("       yai graph runtime-status");
    println!("       yai graph materialize --case <case_ref>");
    println!("       yai graph relations --case <case_ref> [--limit <N>]");
    println!("       yai graph runtime-load --case <case_ref>");
    println!("       yai graph runtime-summary --case <case_ref>");
    println!("       yai graph rebuild --case <case_ref> --from graph-relations");
    println!("       yai graph rebuild --case <case_ref> --from journal --path <journal.jsonl>");
    println!("       yai graph rebuild-report --case <case_ref>");
    println!(
        "       yai graph fanout --case <case_ref> --node <ref> [--edge-kind <kind>] [--limit <N>]"
    );
    println!(
        "       yai graph fanin --case <case_ref> --node <ref> [--edge-kind <kind>] [--limit <N>]"
    );
    println!("       yai graph neighborhood --case <case_ref> --node <ref> [--depth <1|2>] [--limit <N>]");
    println!("       yai graph path --case <case_ref> --from <ref> --to <ref> [--max-depth <N>]");
    println!("       yai facts status");
    println!("       yai facts schema");
    println!("       yai facts init");
    println!("       yai facts extract --case <case_ref> --kind receipt|decision|projection|model_behavior|policy_outcome|carrier_outcome|divergence|memory_quality|core|behavior|operational|all");
    println!("       yai facts summary --case <case_ref>");
    println!("       yai facts report --case <case_ref> [--section receipts|decisions|projections|policy|carriers|divergence|memory|model] [--format plain]");
    println!(
        "       yai memory summary --journal <path>  # legacy MemoryCandidate compatibility only"
    );
    println!("       yai memory rebuild --case <case_ref> [--dry-run]");
    println!("       yai memory clear --case <case_ref>");
    println!("       yai memory list --case <case_ref> [--include-superseded] [--limit <N>]");
    println!("       yai memory show <memory_id>");
    println!("       yai memory provenance <memory_id>");
    println!("       yai memory retrieve --case <case_ref> --participant <participant_id> --purpose conversation|filesystem_write_proposal|effect_consequence|inspection [--resource <attachment>] [--kind <kind>] [--causal-ref <ref>] [--include-superseded] [--limit <N>]");
    println!("       yai reconcile summary --journal <path>");
    println!("       yai query summary --journal <path>");
    println!("       yai query records --journal <path> [--kind <record_kind>] [--case <case_ref>] [--limit <N>]");
    println!("       yai engine summary --journal <path>");
    println!("       yai hot status");
    println!("       yai daemon status --socket <path>");
    println!("       yai daemon info --socket <path>");
    println!("       yai daemon shutdown --socket <path>");
    println!("       yai daemon run-minimum-loop --socket <path>");
    println!("       yai daemon run-filesystem-loop --socket <path>");
    println!("       yai daemon journal-summary --socket <path> --journal <path>");
    println!("       yai daemon projection-summary --socket <path> --journal <path>");
    println!("       yai process observe --pid <pid>");
    println!("       yai process signal --pid <pid> --signal TERM|KILL [--dry-run]");
    println!("       yai observe process --pid <pid>");
    println!("       yai observe compare-process --pid <pid> --expected running|stopped");
    println!("       yai carrier fs-read --sandbox <sandbox> --path <path>");
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

fn observed_result_for_state(state: &str) -> &'static str {
    match state {
        "running" => "matched",
        "not_found" => "not_found",
        "permission_denied" => "permission_denied",
        "unknown" => "unknown",
        _ => "unknown",
    }
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
    Ok(())
}

fn observe_process(args: &[String]) -> Result<(), String> {
    let pid = parse_pid_arg(args)?;
    let state = process_state_for_pid(pid);
    println!("observation_target: process");
    println!("pid: {pid}");
    println!("result: {}", observed_result_for_state(state));
    println!("observed_state: {state}");
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
    println!("transition_schema: yai.transition.v1");
    println!("case_state_schema: yai.case_state.v1");
    println!("legacy_record_schema: yai.record.v1");
    if status.status == "ready" {
        println!("canonical_databases: transitions_by_id,case_transition_sequence,case_state");
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
    schema_status: &'static str,
    readable: &'static str,
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
            schema_status: "missing",
            readable: "no",
            content: None,
        };
    }
    match fs::read_to_string(path) {
        Ok(content) if validate_hot_snapshot(&content) => HotSnapshotStatus {
            status: "active",
            reason: "none",
            schema_status: "valid",
            readable: "yes",
            content: Some(content),
        },
        Ok(content) => HotSnapshotStatus {
            status: "unavailable",
            reason: "invalid_snapshot",
            schema_status: "invalid",
            readable: "yes",
            content: Some(content),
        },
        Err(_) => HotSnapshotStatus {
            status: "unavailable",
            reason: "unreadable_snapshot",
            schema_status: "unknown",
            readable: "no",
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
    println!("case: {}", json_string_or(&content, "case_ref", "unknown"));
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

fn find_on_path(name: &str) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate.display().to_string());
        }
    }
    None
}

fn path_state(path: &std::path::Path) -> &'static str {
    if path.is_dir() {
        "ok"
    } else if path.exists() {
        "not_directory"
    } else {
        "missing"
    }
}

fn path_state_with_path(path: &std::path::Path) -> String {
    format!("{} {}", path.display(), path_state(path))
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

fn path_contains_current_bin() -> Result<bool, String> {
    let current = std::env::current_exe()
        .map_err(|error| format!("failed to resolve current executable: {error}"))?;
    let Some(parent) = current.parent() else {
        return Ok(false);
    };
    let Some(path) = std::env::var_os("PATH") else {
        return Ok(false);
    };
    Ok(std::env::split_paths(&path).any(|entry| entry == parent))
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
                && best.as_ref().map_or(true, |current| path > *current)
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
    existing_env_path("YAI_JOURNAL")
        .or_else(latest_filesystem_journal)
        .ok_or_else(|| {
            "YAI_JOURNAL is required; materialize the case before using case-native commands"
                .to_string()
        })
}

mod filesystem;
use filesystem::*;

mod replay;
use replay::*;

mod review;
use review::*;

mod provider;
use provider::*;

mod controlled_effect;
use controlled_effect::*;

mod case_runtime;
use case_runtime::*;

mod policy;
use policy::policy_command;

mod case_policy;
use case_policy::case_policy_command;

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

mod graph_runtime;
use graph_runtime::*;

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

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let result = match args.first().map(String::as_str) {
        None if env_var("YAI_CASE_REF").is_some() => {
            if let Err(error) = prompt_repl(&[]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("--version") | Some("version") => println!("yai {}", VERSION),
        Some("info") => print_info(),
        Some("doctor") => print_doctor(),
        Some("store") if args.get(1).map(String::as_str) == Some("status") => print_store_status(),
        Some("store") if args.get(1).map(String::as_str) == Some("summary") => {
            if let Err(error) = print_store_summary() {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("store")
            if args.get(1).map(String::as_str) == Some("record")
                && args.get(2).map(String::as_str) == Some("get") =>
        {
            if let Err(error) = store_record_get(&args[3..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("store")
            if args.get(1).map(String::as_str) == Some("record")
                && args.get(2).map(String::as_str) == Some("list") =>
        {
            if let Err(error) = store_record_list(&args[3..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("store") if args.get(1).map(String::as_str) == Some("tail") => {
            if let Err(error) = store_tail(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("journal") if args.get(1).map(String::as_str) == Some("inspect") => {
            if let Err(error) = journal_inspect(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("journal")
            if args.get(1).map(String::as_str) == Some("compatibility-inspect") =>
        {
            if let Err(error) = journal_compatibility_inspect(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("journal")
            if args.get(1).map(String::as_str) == Some("compatibility-import") =>
        {
            if let Err(error) = journal_compatibility_import(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("journal") if args.get(1).map(String::as_str) == Some("replay") => {
            if let Err(error) = journal_replay(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("journal") if args.get(1).map(String::as_str) == Some("replay-status") => {
            if let Err(error) = journal_replay_status(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("journal") if args.get(1).map(String::as_str) == Some("replay-report") => {
            if let Err(error) = journal_replay_report(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("projection") if args.get(1).map(String::as_str) == Some("summary") => {
            if let Err(error) = projection_summary(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("projection") if args.get(1).map(String::as_str) == Some("inspect") => {
            if let Err(error) = projection_inspect(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("projection") if args.get(1).map(String::as_str) == Some("request") => {
            if let Err(error) = projection_request(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("context") if args.get(1).map(String::as_str) == Some("inspect") => {
            if let Err(error) = semantic_context_inspect(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("case") if args.get(1).map(String::as_str) == Some("enter") => {
            if let Err(error) = case_enter(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("case") if args.get(1).map(String::as_str) == Some("attach-provider") => {
            if let Err(error) = case_attach_provider(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("case") if args.get(1).map(String::as_str) == Some("attach-filesystem") => {
            if let Err(error) = case_attach_filesystem(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("case") if args.get(1).map(String::as_str) == Some("bind-participant-role") => {
            if let Err(error) = provider::case_bind_participant_role(&args[2..]) {
                eprintln!("error: {error}");
                std::process::exit(2);
            }
        }
        Some("case") if args.get(1).map(String::as_str) == Some("policy") => {
            if let Err(error) = case_policy_command(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("case") if args.get(1).map(String::as_str) == Some("run") => {
            if let Err(error) = case_runtime_run(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("case") if args.get(1).map(String::as_str) == Some("resume") => {
            if let Err(error) = case_runtime_resume(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("case") if args.get(1).map(String::as_str) == Some("status") => {
            if let Err(error) = case_runtime_status(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("case") if args.get(1).map(String::as_str) == Some("stop") => {
            if let Err(error) = case_runtime_stop(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("policy") => {
            if let Err(error) = policy_command(&args[1..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("effect") if args.get(1).map(String::as_str) == Some("filesystem-write") => {
            if let Err(error) = controlled_filesystem_write(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("effect") if args.get(1).map(String::as_str) == Some("reconcile") => {
            if let Err(error) = controlled_effect_reconcile(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("effect") if args.get(1).map(String::as_str) == Some("inspect") => {
            if let Err(error) = controlled_effect_inspect(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("prompt") => {
            if let Err(error) = prompt_repl(&args[1..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("control") if args.get(1).map(String::as_str) == Some("summary") => {
            if let Err(error) = control_summary(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("review") if args.get(1).map(String::as_str) == Some("pending") => {
            if let Err(error) = review_pending(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("review") if args.get(1).map(String::as_str) == Some("show") => {
            if let Err(error) = review_show(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("review") if args.get(1).map(String::as_str) == Some("approve") => {
            if let Err(error) = review_resolve(&args[2..], ReviewActionKind::Approve) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("review") if args.get(1).map(String::as_str) == Some("deny") => {
            if let Err(error) = review_resolve(&args[2..], ReviewActionKind::Deny) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("review") if args.get(1).map(String::as_str) == Some("defer") => {
            if let Err(error) = review_resolve(&args[2..], ReviewActionKind::Defer) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("decision") if args.get(1).map(String::as_str) == Some("inspect") => {
            if let Err(error) = decision_inspect(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("receipt") if args.get(1).map(String::as_str) == Some("summary") => {
            if let Err(error) = receipt_summary(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("graph") if args.get(1).map(String::as_str) == Some("summary") => {
            if let Err(error) = graph_summary(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("graph") if args.get(1).map(String::as_str) == Some("schema") => {
            if let Err(error) = graph_schema(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("graph") if args.get(1).map(String::as_str) == Some("runtime-status") => {
            if let Err(error) = graph_runtime_status(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("graph") if args.get(1).map(String::as_str) == Some("runtime-load") => {
            if let Err(error) = graph_runtime_load(&args[2..], false) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("graph") if args.get(1).map(String::as_str) == Some("runtime-summary") => {
            if let Err(error) = graph_runtime_load(&args[2..], true) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("graph") if args.get(1).map(String::as_str) == Some("rebuild") => {
            if let Err(error) = graph_rebuild(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("graph") if args.get(1).map(String::as_str) == Some("rebuild-report") => {
            if let Err(error) = graph_rebuild_report(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("graph") if args.get(1).map(String::as_str) == Some("materialize") => {
            if let Err(error) = graph_materialize(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("graph") if args.get(1).map(String::as_str) == Some("relations") => {
            if let Err(error) = graph_relations(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("graph") if args.get(1).map(String::as_str) == Some("fanout") => {
            if let Err(error) = graph_fanout(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("graph") if args.get(1).map(String::as_str) == Some("fanin") => {
            if let Err(error) = graph_fanin(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("graph") if args.get(1).map(String::as_str) == Some("neighborhood") => {
            if let Err(error) = graph_neighborhood(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("graph") if args.get(1).map(String::as_str) == Some("path") => {
            if let Err(error) = graph_path(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("facts") if args.get(1).map(String::as_str) == Some("status") => {
            if let Err(error) = facts_status(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("facts") if args.get(1).map(String::as_str) == Some("schema") => {
            if let Err(error) = facts_schema(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("facts") if args.get(1).map(String::as_str) == Some("init") => {
            if let Err(error) = facts_init(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("facts") if args.get(1).map(String::as_str) == Some("extract") => {
            if let Err(error) = facts_extract(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("facts") if args.get(1).map(String::as_str) == Some("summary") => {
            if let Err(error) = facts_summary(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("facts") if args.get(1).map(String::as_str) == Some("report") => {
            if let Err(error) = facts_report(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("memory") if args.get(1).map(String::as_str) == Some("summary") => {
            if let Err(error) = memory_summary(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("memory") if args.get(1).map(String::as_str) == Some("rebuild") => {
            if let Err(error) = memory_rebuild(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("memory") if args.get(1).map(String::as_str) == Some("clear") => {
            if let Err(error) = memory_clear(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("memory") if args.get(1).map(String::as_str) == Some("list") => {
            if let Err(error) = memory_list(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("memory") if args.get(1).map(String::as_str) == Some("show") => {
            if let Err(error) = memory_show(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("memory") if args.get(1).map(String::as_str) == Some("provenance") => {
            if let Err(error) = memory_provenance(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("memory") if args.get(1).map(String::as_str) == Some("retrieve") => {
            if let Err(error) = memory_retrieve(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("reconcile") if args.get(1).map(String::as_str) == Some("summary") => {
            if let Err(error) = reconcile_summary(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("query") if args.get(1).map(String::as_str) == Some("summary") => {
            if let Err(error) = query_summary(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("query") if args.get(1).map(String::as_str) == Some("records") => {
            if let Err(error) = query_records(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("engine") if args.get(1).map(String::as_str) == Some("summary") => {
            if let Err(error) = engine_summary(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("hot") if args.get(1).map(String::as_str) == Some("status") => {
            if let Err(error) = print_hot_status() {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("daemon") if args.get(1).map(String::as_str) == Some("status") => {
            if let Err(error) = daemon_request(&args[2..], "status") {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("daemon") if args.get(1).map(String::as_str) == Some("info") => {
            if let Err(error) = daemon_request(&args[2..], "info") {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("daemon") if args.get(1).map(String::as_str) == Some("shutdown") => {
            if let Err(error) = daemon_request(&args[2..], "shutdown") {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("daemon") if args.get(1).map(String::as_str) == Some("run-minimum-loop") => {
            if let Err(error) = daemon_request_and_import_records(
                &args[2..],
                "run_minimum_loop request_id=yai-minimum case_ref=case:new12-daemon subject_ref=subject:repo-test",
            ) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("daemon") if args.get(1).map(String::as_str) == Some("run-filesystem-loop") => {
            if let Err(error) = daemon_request_and_import_records(
                &args[2..],
                "run_filesystem_loop request_id=yai-filesystem case_ref=case:new12-filesystem subject_ref=subject:filesystem-sandbox",
            ) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("daemon") if args.get(1).map(String::as_str) == Some("journal-summary") => {
            if let Err(error) = daemon_request_with_journal(&args[2..], "journal_summary") {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("daemon") if args.get(1).map(String::as_str) == Some("projection-summary") => {
            if let Err(error) = daemon_request_with_journal(&args[2..], "projection_summary") {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("carrier") if args.get(1).map(String::as_str) == Some("fs-read") => {
            if let Err(error) = carrier_fs_read(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("process") if args.get(1).map(String::as_str) == Some("observe") => {
            if let Err(error) = process_observe(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("process") if args.get(1).map(String::as_str) == Some("signal") => {
            if let Err(error) = process_signal(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("observe") if args.get(1).map(String::as_str) == Some("process") => {
            if let Err(error) = observe_process(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some("observe") if args.get(1).map(String::as_str) == Some("compare-process") => {
            if let Err(error) = observe_compare_process(&args[2..]) {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
        Some(_) => {
            print_usage();
            std::process::exit(2);
        }
        None => print_usage(),
    };
    result
}
