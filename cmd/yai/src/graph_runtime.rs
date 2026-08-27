//! LMDB graph materialization, RuntimeGraph rebuild, and bounded query behavior.

use super::*;

pub(super) fn graph_summary(args: &[String]) -> Result<(), String> {
    let path = journal_arg(args)?;
    let journal = Journal::load_jsonl(&path)
        .map_err(|error| format!("failed to load {}: {error}", path.display()))?;
    let projection = ProjectionSummary::from_journal("graph", &journal);
    let graph = GraphSummary::from_summaries(
        journal
            .records()
            .iter()
            .filter(|record| record.kind == RecordKind::GraphEdge)
            .map(|record| record.summary.as_str()),
    );

    println!("graph_edges: {}", projection.graph_edge_count);
    println!("case_binds_subject: {}", graph.case_binds_subject);
    println!("op_targets_subject: {}", graph.op_targets_subject);
    println!("decision_controls_op: {}", graph.decision_controls_op);
    println!("receipt_records_effect: {}", graph.receipt_records_effect);
    println!("receipt_updates_subject: {}", graph.receipt_updates_subject);
    Ok(())
}

const GRAPH_NODE_KINDS: &[&str] = &[
    "case",
    "subject",
    "operation",
    "attempt",
    "decision",
    "review_request",
    "review_decision",
    "control_pending",
    "dispatch",
    "carrier",
    "receipt",
    "effect",
    "observation",
    "divergence",
    "policy",
    "projection",
    "memory_candidate",
    "model_output",
    "model_interpretation",
    "record",
    "unknown",
];

const GRAPH_EDGE_KINDS: &[&str] = &[
    "belongs_to_case",
    "subject_participates_in_case",
    "attempt_targets_subject",
    "decision_controls_attempt",
    "review_request_for_attempt",
    "review_decision_resolves_request",
    "control_pending_blocks_attempt",
    "review_resolution_produces_receipt",
    "dispatch_routes_decision",
    "carrier_realizes_dispatch",
    "receipt_records_effect",
    "observation_checks_effect",
    "divergence_describes_mismatch",
    "policy_constrains_subject",
    "policy_constrains_operation",
    "projection_exposes_record",
    "model_output_produces_interpretation",
    "memory_derived_from_receipt",
    "record_materializes_node",
    "derived_from",
    "supports",
    "contradicts",
    "unknown",
];

pub(super) fn graph_schema(args: &[String]) -> Result<(), String> {
    if !args.is_empty() {
        return Err("usage: yai graph schema".to_string());
    }
    println!("graph_schema:");
    println!("  node_kinds:");
    for kind in GRAPH_NODE_KINDS {
        println!("  - {kind}");
    }
    println!();
    println!("  edge_kinds:");
    for kind in GRAPH_EDGE_KINDS {
        println!("  - {kind}");
    }
    println!();
    println!("graph_persistence:");
    println!("  status: active_minimal");
    println!("  durable_truth: typed_relations");
    println!("  relation_write_path: active_minimal");
    println!("  graph_store: {GRAPH_RELATION_STORE_NAME}");
    println!("runtime_graph:");
    println!("  status: active_minimal");
    println!("  role: in_memory_active_case_working_set");
    println!("  working_set: per_command_ephemeral");
    println!("  resident_service: planned");
    println!("  source: graph_relations");
    println!("  hnsw: future_candidate_index");
    println!("  context_compiler: future_consumer");
    Ok(())
}

pub(super) fn graph_runtime_status(args: &[String]) -> Result<(), String> {
    if !args.is_empty() {
        return Err("usage: yai graph runtime-status".to_string());
    }
    println!("runtime_graph:");
    println!("  status: active_minimal");
    println!("  role: in_memory_active_case_working_set");
    println!("  working_set: per_command_ephemeral");
    println!("  resident_service: planned");
    println!("  source: graph_relations");
    println!("  durable_truth: graph_persistence");
    println!("  hnsw: future_candidate_index");
    println!("  context_compiler: future_consumer");
    println!("  relation_write_path: active_minimal");
    println!("  graph_store: {GRAPH_RELATION_STORE_NAME}");
    println!("  graph_persistence: durable_typed_relations");
    println!("  implementation_claim: ephemeral_working_set_only");
    Ok(())
}

pub(super) fn graph_runtime_load(args: &[String], summary_only: bool) -> Result<(), String> {
    let case_ref = named_arg(args, "--case")?;
    let status = LmdbRecordStore::status(record_store_path());
    if status.status != RecordStoreStatusKind::Ready {
        print_non_ready_record_store(&status);
        return Ok(());
    }
    let store = LmdbRecordStore::open(&status.path)?;
    let graph = store.load_runtime_graph_for_case(&case_ref)?;
    if summary_only {
        println!("runtime_graph_summary:");
    } else {
        println!("runtime_graph_load:");
    }
    println!("case_ref: {}", graph.case_ref);
    println!("source: {}", graph.source);
    println!("nodes_total: {}", graph.nodes_total);
    println!("edges_total: {}", graph.edges_total);
    println!("outgoing_index_entries: {}", graph.outgoing_index_entries);
    println!("incoming_index_entries: {}", graph.incoming_index_entries);
    println!("generation: {}", graph.generation);
    println!("dirty: {}", yes_no(graph.dirty));
    println!("stale: {}", yes_no(graph.stale));
    println!("durable_truth: {}", graph.durable_truth);
    println!("resident: false");
    println!("resident_service: planned");
    println!("hnsw: future_candidate_index");
    println!("context_compiler: future_consumer");
    Ok(())
}

#[derive(Clone, Debug)]
struct RuntimeGraphRebuildReport {
    case_ref: String,
    source_mode: String,
    journal_path: String,
    journal_identity: String,
    lmdb_path: String,
    graph_relation_source: String,
    records_seen: usize,
    records_written: usize,
    records_duplicate: usize,
    relations_seen: usize,
    relations_written: usize,
    relations_duplicate: usize,
    relations_skipped: usize,
    nodes_total: usize,
    edges_total: usize,
    outgoing_index_entries: usize,
    incoming_index_entries: usize,
    runtime_generation: usize,
    dirty: bool,
    stale: bool,
    journal_replay_status: String,
    graph_materialize_status: String,
    runtime_graph_status: String,
    rebuild_status: String,
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl RuntimeGraphRebuildReport {
    fn empty(case_ref: &str, source_mode: &str) -> Self {
        Self {
            case_ref: case_ref.to_string(),
            source_mode: source_mode.to_string(),
            journal_path: String::new(),
            journal_identity: String::new(),
            lmdb_path: record_store_path().display().to_string(),
            graph_relation_source: GRAPH_RELATION_STORE_NAME.to_string(),
            records_seen: 0,
            records_written: 0,
            records_duplicate: 0,
            relations_seen: 0,
            relations_written: 0,
            relations_duplicate: 0,
            relations_skipped: 0,
            nodes_total: 0,
            edges_total: 0,
            outgoing_index_entries: 0,
            incoming_index_entries: 0,
            runtime_generation: 0,
            dirty: false,
            stale: false,
            journal_replay_status: "not_applicable".to_string(),
            graph_materialize_status: "not_started".to_string(),
            runtime_graph_status: "not_started".to_string(),
            rebuild_status: "not_started".to_string(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn with_runtime_graph(mut self, graph: &RuntimeGraphLoadResult) -> Self {
        self.nodes_total = graph.nodes_total;
        self.edges_total = graph.edges_total;
        self.outgoing_index_entries = graph.outgoing_index_entries;
        self.incoming_index_entries = graph.incoming_index_entries;
        self.runtime_generation = graph.generation;
        self.dirty = graph.dirty;
        self.stale = graph.stale;
        self.runtime_graph_status = "completed".to_string();
        if graph.edges_total == 0 {
            self.warnings
                .push("no_graph_relations_for_case".to_string());
        }
        self
    }
}

pub(super) fn graph_rebuild(args: &[String]) -> Result<(), String> {
    let case_ref = named_arg(args, "--case")?;
    let source_mode = named_arg(args, "--from")?;
    match source_mode.as_str() {
        "graph-relations" => graph_rebuild_from_graph_relations(&case_ref),
        "journal" => {
            let path = PathBuf::from(named_arg(args, "--path")?);
            graph_rebuild_from_journal(&case_ref, &path)
        }
        other => Err(format!("unsupported_rebuild_source: {other}")),
    }
}

fn graph_rebuild_from_graph_relations(case_ref: &str) -> Result<(), String> {
    let status = LmdbRecordStore::status(record_store_path());
    if status.status != RecordStoreStatusKind::Ready {
        let mut report = RuntimeGraphRebuildReport::empty(case_ref, "graph_relations");
        report.rebuild_status = "missing_source".to_string();
        report.errors.push("record_store_not_ready".to_string());
        let report_path = write_runtime_graph_rebuild_report(&report)?;
        print_runtime_graph_rebuild(&report, &report_path);
        return Ok(());
    }
    let store = LmdbRecordStore::open(&status.path)?;
    let graph = store.load_runtime_graph_for_case(case_ref)?;
    let mut report =
        RuntimeGraphRebuildReport::empty(case_ref, "graph_relations").with_runtime_graph(&graph);
    report.relations_seen = graph.edges_total;
    report.graph_materialize_status = "not_applicable".to_string();
    report.rebuild_status = "completed".to_string();
    let report_path = write_runtime_graph_rebuild_report(&report)?;
    print_runtime_graph_rebuild(&report, &report_path);
    Ok(())
}

fn graph_rebuild_from_journal(case_ref: &str, path: &Path) -> Result<(), String> {
    let mut report = RuntimeGraphRebuildReport::empty(case_ref, "journal");
    report.journal_path = path.display().to_string();
    if !path.exists() || !path.is_file() {
        let profile = replay_profile_for_missing(path);
        report.journal_identity = profile.journal_identity;
        report.journal_replay_status = "failed".to_string();
        report.rebuild_status = "missing_source".to_string();
        report.errors.push("missing_journal".to_string());
        let report_path = write_runtime_graph_rebuild_report(&report)?;
        print_runtime_graph_rebuild(&report, &report_path);
        println!("reason: missing_journal");
        return Ok(());
    }

    let inspection = Journal::inspect_jsonl(path)
        .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read {} for runtime graph rebuild identity: {error}",
            path.display()
        )
    })?;
    let profile = replay_profile_for_inspection(path, &contents, &inspection);
    report.journal_identity = profile.journal_identity.clone();
    report.records_seen = inspection.valid_entries;

    if !inspection.replay_ready() {
        let reason = replay_failure_reason(&inspection);
        report.journal_replay_status = "failed".to_string();
        report.rebuild_status = "failed".to_string();
        report.errors.push(reason.clone());
        let report_path = write_runtime_graph_rebuild_report(&report)?;
        print_runtime_graph_rebuild(&report, &report_path);
        println!("reason: {reason}");
        return Ok(());
    }

    let journal = Journal::load_jsonl(path)
        .map_err(|error| format!("runtime graph rebuild failed to load journal: {error}"))?;
    let store = LmdbRecordStore::open(record_store_path())
        .map_err(|error| format!("runtime graph rebuild failed to open LMDB: {error}"))?;
    let started_at = unix_timestamp_string();
    store.put_replay_metadata(&replay_metadata_in_progress(
        path,
        &profile,
        &inspection,
        &started_at,
    ))?;
    let import_report = store.import_journal_with_report(&journal, &path.display().to_string())?;
    let metadata = replay_metadata_from_report(
        path,
        &profile,
        &inspection,
        &import_report,
        &started_at,
        &unix_timestamp_string(),
    );
    store.put_replay_metadata(&metadata)?;

    let materialize_report = store.materialize_graph_relations_for_case(case_ref)?;
    let graph = store.load_runtime_graph_for_case(case_ref)?;

    report.records_seen = import_report.records_seen;
    report.records_written = import_report.records_written;
    report.records_duplicate = import_report.records_duplicate;
    apply_materialize_report(&mut report, &materialize_report);
    report = report.with_runtime_graph(&graph);
    report.journal_replay_status = "completed".to_string();
    report.graph_materialize_status = "completed".to_string();
    report.rebuild_status = "completed".to_string();
    let report_path = write_runtime_graph_rebuild_report(&report)?;
    print_runtime_graph_rebuild(&report, &report_path);
    Ok(())
}

fn apply_materialize_report(
    report: &mut RuntimeGraphRebuildReport,
    materialize_report: &GraphMaterializeReport,
) {
    report.relations_seen = materialize_report.relations_seen;
    report.relations_written = materialize_report.relations_written;
    report.relations_duplicate = materialize_report.relations_duplicate;
    report.relations_skipped = materialize_report.relations_skipped;
}

pub(super) fn graph_rebuild_report(args: &[String]) -> Result<(), String> {
    let case_ref = named_arg(args, "--case")?;
    let report_path = runtime_graph_rebuild_report_path(&case_ref);
    if !report_path.is_file() {
        println!("runtime_graph_rebuild_report:");
        println!("report_schema: yai.runtime_graph_rebuild_report.v1");
        println!("rebuild_report: not_found");
        println!("case_ref: {case_ref}");
        println!("rebuild_status: not_started");
        return Ok(());
    }
    let report = fs::read_to_string(&report_path)
        .map_err(|error| format!("failed to read {}: {error}", report_path.display()))?;
    println!("runtime_graph_rebuild_report:");
    println!(
        "report_schema: {}",
        json_string_or(
            &report,
            "report_schema",
            "yai.runtime_graph_rebuild_report.v1"
        )
    );
    println!("rebuild_report: {}", report_path.display());
    println!(
        "case_ref: {}",
        json_string_or(&report, "case_ref", &case_ref)
    );
    println!(
        "source_mode: {}",
        json_string_or(&report, "source_mode", "unknown")
    );
    println!(
        "journal_identity: {}",
        json_string_or(&report, "journal_identity", "none")
    );
    println!(
        "journal_replay_status: {}",
        json_string_or(&report, "journal_replay_status", "unknown")
    );
    println!(
        "graph_materialize_status: {}",
        json_string_or(&report, "graph_materialize_status", "unknown")
    );
    println!(
        "runtime_graph_status: {}",
        json_string_or(&report, "runtime_graph_status", "unknown")
    );
    print_report_number(&report, "records_seen", 0);
    print_report_number(&report, "relations_seen", 0);
    print_report_number(&report, "relations_written", 0);
    print_report_number(&report, "relations_duplicate", 0);
    print_report_number(&report, "nodes_total", 0);
    print_report_number(&report, "edges_total", 0);
    print_report_number(&report, "outgoing_index_entries", 0);
    print_report_number(&report, "incoming_index_entries", 0);
    println!(
        "rebuild_status: {}",
        json_string_or(&report, "rebuild_status", "unknown")
    );
    if report.contains("\"no_graph_relations_for_case\"") {
        println!("warnings:");
        println!("- no_graph_relations_for_case");
    }
    if report.contains("\"invalid_json\"") {
        println!("errors:");
        println!("- invalid_json");
    }
    Ok(())
}

fn print_runtime_graph_rebuild(report: &RuntimeGraphRebuildReport, report_path: &Path) {
    println!("runtime_graph_rebuild:");
    println!("case_ref: {}", report.case_ref);
    println!("source_mode: {}", report.source_mode);
    if !report.journal_path.is_empty() {
        println!("journal_path: {}", report.journal_path);
    }
    if !report.journal_identity.is_empty() {
        println!("journal_identity: {}", report.journal_identity);
    }
    println!("lmdb_path: {}", report.lmdb_path);
    println!("graph_relation_source: {}", report.graph_relation_source);
    println!("journal_replay_status: {}", report.journal_replay_status);
    println!(
        "graph_materialize_status: {}",
        report.graph_materialize_status
    );
    println!("runtime_graph_status: {}", report.runtime_graph_status);
    println!("records_seen: {}", report.records_seen);
    println!("records_written: {}", report.records_written);
    println!("records_duplicate: {}", report.records_duplicate);
    println!("relations_seen: {}", report.relations_seen);
    println!("relations_written: {}", report.relations_written);
    println!("relations_duplicate: {}", report.relations_duplicate);
    println!("relations_skipped: {}", report.relations_skipped);
    println!("nodes_total: {}", report.nodes_total);
    println!("edges_total: {}", report.edges_total);
    println!("outgoing_index_entries: {}", report.outgoing_index_entries);
    println!("incoming_index_entries: {}", report.incoming_index_entries);
    println!("runtime_generation: {}", report.runtime_generation);
    println!("dirty: {}", yes_no(report.dirty));
    println!("stale: {}", yes_no(report.stale));
    println!("rebuild_status: {}", report.rebuild_status);
    println!("report_schema: yai.runtime_graph_rebuild_report.v1");
    println!("rebuild_report: {}", report_path.display());
    if !report.warnings.is_empty() {
        println!("warnings:");
        for warning in &report.warnings {
            println!("- {warning}");
        }
    }
    if !report.errors.is_empty() {
        println!("errors:");
        for error in &report.errors {
            println!("- {error}");
        }
    }
}

fn write_runtime_graph_rebuild_report(
    report: &RuntimeGraphRebuildReport,
) -> Result<PathBuf, String> {
    fs::create_dir_all(runtime_graph_rebuild_report_dir()).map_err(|error| {
        format!(
            "failed to create RuntimeGraph rebuild report dir {}: {error}",
            runtime_graph_rebuild_report_dir().display()
        )
    })?;
    let report_path = runtime_graph_rebuild_report_path(&report.case_ref);
    fs::write(&report_path, runtime_graph_rebuild_report_json(report))
        .map_err(|error| format!("failed to write {}: {error}", report_path.display()))?;
    Ok(report_path)
}

fn runtime_graph_rebuild_report_json(report: &RuntimeGraphRebuildReport) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"report_schema\":\"yai.runtime_graph_rebuild_report.v1\",\n",
            "  \"case_ref\":\"{}\",\n",
            "  \"source_mode\":\"{}\",\n",
            "  \"journal_path\":\"{}\",\n",
            "  \"journal_identity\":\"{}\",\n",
            "  \"lmdb_path\":\"{}\",\n",
            "  \"graph_relation_source\":\"{}\",\n",
            "  \"records_seen\":{},\n",
            "  \"records_written\":{},\n",
            "  \"records_duplicate\":{},\n",
            "  \"relations_seen\":{},\n",
            "  \"relations_written\":{},\n",
            "  \"relations_duplicate\":{},\n",
            "  \"relations_skipped\":{},\n",
            "  \"nodes_total\":{},\n",
            "  \"edges_total\":{},\n",
            "  \"outgoing_index_entries\":{},\n",
            "  \"incoming_index_entries\":{},\n",
            "  \"runtime_generation\":{},\n",
            "  \"dirty\":\"{}\",\n",
            "  \"stale\":\"{}\",\n",
            "  \"journal_replay_status\":\"{}\",\n",
            "  \"graph_materialize_status\":\"{}\",\n",
            "  \"runtime_graph_status\":\"{}\",\n",
            "  \"rebuild_status\":\"{}\",\n",
            "  \"errors\":[{}],\n",
            "  \"warnings\":[{}]\n",
            "}}\n"
        ),
        json_escape(&report.case_ref),
        json_escape(&report.source_mode),
        json_escape(&report.journal_path),
        json_escape(&report.journal_identity),
        json_escape(&report.lmdb_path),
        json_escape(&report.graph_relation_source),
        report.records_seen,
        report.records_written,
        report.records_duplicate,
        report.relations_seen,
        report.relations_written,
        report.relations_duplicate,
        report.relations_skipped,
        report.nodes_total,
        report.edges_total,
        report.outgoing_index_entries,
        report.incoming_index_entries,
        report.runtime_generation,
        yes_no(report.dirty),
        yes_no(report.stale),
        json_escape(&report.journal_replay_status),
        json_escape(&report.graph_materialize_status),
        json_escape(&report.runtime_graph_status),
        json_escape(&report.rebuild_status),
        json_string_array(&report.errors),
        json_string_array(&report.warnings),
    )
}

fn json_string_array(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(",")
}

pub(super) fn graph_materialize(args: &[String]) -> Result<(), String> {
    let case_ref = named_arg(args, "--case")?;
    let status = LmdbRecordStore::status(record_store_path());
    if status.status != RecordStoreStatusKind::Ready {
        print_non_ready_record_store(&status);
        return Ok(());
    }
    let store = LmdbRecordStore::open(&status.path)?;
    let report = store.materialize_graph_relations_for_case(&case_ref)?;
    println!("graph_materialize:");
    println!("case_ref: {case_ref}");
    println!("source: lmdb_records");
    println!("relations_seen: {}", report.relations_seen);
    println!("relations_written: {}", report.relations_written);
    println!("relations_duplicate: {}", report.relations_duplicate);
    println!("relations_skipped: {}", report.relations_skipped);
    println!("schema: {GRAPH_RELATION_SCHEMA}");
    println!("graph_store: {GRAPH_RELATION_STORE_NAME}");
    println!("runtime_graph_updated: false");
    Ok(())
}

pub(super) fn graph_relations(args: &[String]) -> Result<(), String> {
    let case_ref = named_arg(args, "--case")?;
    let limit = parse_limit(args)?;
    let status = LmdbRecordStore::status(record_store_path());
    if status.status != RecordStoreStatusKind::Ready {
        print_non_ready_record_store(&status);
        return Ok(());
    }
    let store = LmdbRecordStore::open(&status.path)?;
    let result = store.list_graph_relations_by_case(&case_ref, limit)?;
    println!("graph_relations:");
    println!("case_ref: {case_ref}");
    println!("relations_total: {}", result.relations_total);
    println!("limit: {limit}");
    if result.relations.is_empty() {
        println!("relations: none");
    } else {
        println!("relations:");
        for relation in result.relations {
            println!("- relation_id: {}", relation.relation_id);
            println!("  edge_kind: {}", relation.edge_kind);
            println!("  from_ref: {}", relation.from_ref);
            println!("  to_ref: {}", relation.to_ref);
            println!("  source_record_id: {}", relation.source_record_id);
        }
    }
    Ok(())
}

fn graph_query_limit(args: &[String]) -> Result<(usize, bool), String> {
    let limit = parse_limit(args)?;
    Ok((limit.min(200), limit > 200))
}

fn graph_query_depth(
    args: &[String],
    name: &str,
    default: usize,
    max: usize,
) -> Result<(usize, bool), String> {
    let raw = optional_arg(args, name).unwrap_or_else(|| default.to_string());
    let parsed = raw
        .parse::<usize>()
        .map_err(|_| format!("invalid {name} value: {raw}"))?;
    if parsed == 0 {
        return Err(format!("{name} must be greater than zero"));
    }
    Ok((parsed.min(max), parsed > max))
}

fn runtime_graph_for_query(case_ref: &str) -> Result<Option<RuntimeGraphLoadResult>, String> {
    let status = LmdbRecordStore::status(record_store_path());
    if status.status != RecordStoreStatusKind::Ready {
        print_non_ready_record_store(&status);
        return Ok(None);
    }
    let store = LmdbRecordStore::open(&status.path)?;
    Ok(Some(store.load_runtime_graph_for_case(case_ref)?))
}

fn edge_matches_kind(edge: &RuntimeGraphEdge, edge_kind: &Option<String>) -> bool {
    edge_kind
        .as_ref()
        .map(|kind| edge.edge_kind == *kind)
        .unwrap_or(true)
}

fn runtime_node_kind(graph: &RuntimeGraphLoadResult, node_ref: &str) -> String {
    graph
        .nodes
        .iter()
        .find(|node| node.node_ref == node_ref)
        .map(|node| node.node_kind.clone())
        .unwrap_or_else(|| "unknown".to_string())
}

fn print_graph_edges(edges: &[RuntimeGraphEdge]) {
    if edges.is_empty() {
        println!("edges: none");
    } else {
        println!("edges:");
        for edge in edges {
            println!("- edge_kind: {}", edge.edge_kind);
            println!("  from_ref: {}", edge.from_ref);
            println!("  to_ref: {}", edge.to_ref);
            println!("  relation_id: {}", edge.relation_id);
        }
    }
}

pub(super) fn graph_fanout(args: &[String]) -> Result<(), String> {
    let case_ref = named_arg(args, "--case")?;
    let node_ref = named_arg(args, "--node")?;
    let edge_kind = optional_arg(args, "--edge-kind");
    let (limit, limit_clamped) = graph_query_limit(args)?;
    let Some(graph) = runtime_graph_for_query(&case_ref)? else {
        return Ok(());
    };
    let edges: Vec<RuntimeGraphEdge> = graph
        .edges
        .iter()
        .filter(|edge| edge.from_ref == node_ref && edge_matches_kind(edge, &edge_kind))
        .take(limit)
        .cloned()
        .collect();
    println!("graph_fanout:");
    println!("case_ref: {case_ref}");
    println!("node: {node_ref}");
    println!("edges_total: {}", edges.len());
    println!("limit: {limit}");
    if limit_clamped {
        println!("limit_clamped: yes");
    }
    if let Some(kind) = edge_kind {
        println!("edge_kind_filter: {kind}");
    }
    print_graph_edges(&edges);
    Ok(())
}

pub(super) fn graph_fanin(args: &[String]) -> Result<(), String> {
    let case_ref = named_arg(args, "--case")?;
    let node_ref = named_arg(args, "--node")?;
    let edge_kind = optional_arg(args, "--edge-kind");
    let (limit, limit_clamped) = graph_query_limit(args)?;
    let Some(graph) = runtime_graph_for_query(&case_ref)? else {
        return Ok(());
    };
    let edges: Vec<RuntimeGraphEdge> = graph
        .edges
        .iter()
        .filter(|edge| edge.to_ref == node_ref && edge_matches_kind(edge, &edge_kind))
        .take(limit)
        .cloned()
        .collect();
    println!("graph_fanin:");
    println!("case_ref: {case_ref}");
    println!("node: {node_ref}");
    println!("edges_total: {}", edges.len());
    println!("limit: {limit}");
    if limit_clamped {
        println!("limit_clamped: yes");
    }
    if let Some(kind) = edge_kind {
        println!("edge_kind_filter: {kind}");
    }
    print_graph_edges(&edges);
    Ok(())
}

pub(super) fn graph_neighborhood(args: &[String]) -> Result<(), String> {
    let case_ref = named_arg(args, "--case")?;
    let node_ref = named_arg(args, "--node")?;
    let edge_kind = optional_arg(args, "--edge-kind");
    let (depth, depth_clamped) = graph_query_depth(args, "--depth", 1, 2)?;
    let (limit, limit_clamped) = graph_query_limit(args)?;
    let Some(graph) = runtime_graph_for_query(&case_ref)? else {
        return Ok(());
    };

    let mut seen_nodes = HashSet::new();
    let mut nodes = Vec::new();
    let mut seen_edges = HashSet::new();
    let mut edges = Vec::new();
    let mut queue = VecDeque::new();
    seen_nodes.insert(node_ref.clone());
    nodes.push(node_ref.clone());
    queue.push_back((node_ref.clone(), 0usize));

    while let Some((current, current_depth)) = queue.pop_front() {
        if current_depth >= depth || edges.len() >= limit {
            continue;
        }
        for edge in graph
            .edges
            .iter()
            .filter(|edge| edge_matches_kind(edge, &edge_kind))
            .filter(|edge| edge.from_ref == current || edge.to_ref == current)
        {
            if edges.len() >= limit {
                break;
            }
            if seen_edges.insert(edge.relation_id.clone()) {
                edges.push(edge.clone());
            }
            for next in [&edge.from_ref, &edge.to_ref] {
                if seen_nodes.insert(next.clone()) {
                    nodes.push(next.clone());
                    queue.push_back((next.clone(), current_depth + 1));
                }
            }
        }
    }

    println!("graph_neighborhood:");
    println!("case_ref: {case_ref}");
    println!("node: {node_ref}");
    println!("depth: {depth}");
    if depth_clamped {
        println!("depth_clamped: yes");
    }
    println!("limit: {limit}");
    if limit_clamped {
        println!("limit_clamped: yes");
    }
    if let Some(kind) = edge_kind {
        println!("edge_kind_filter: {kind}");
    }
    println!("nodes_total: {}", nodes.len());
    println!("edges_total: {}", edges.len());
    if nodes.is_empty() {
        println!("nodes: none");
    } else {
        println!("nodes:");
        for node in &nodes {
            println!("- ref: {node}");
            println!("  kind: {}", runtime_node_kind(&graph, node));
        }
    }
    print_graph_edges(&edges);
    Ok(())
}

pub(super) fn graph_path(args: &[String]) -> Result<(), String> {
    let case_ref = named_arg(args, "--case")?;
    let from_ref = named_arg(args, "--from")?;
    let to_ref = named_arg(args, "--to")?;
    let (max_depth, depth_clamped) = graph_query_depth(args, "--max-depth", 4, 6)?;
    let Some(graph) = runtime_graph_for_query(&case_ref)? else {
        return Ok(());
    };

    let mut outgoing: HashMap<String, Vec<RuntimeGraphEdge>> = HashMap::new();
    for edge in &graph.edges {
        outgoing
            .entry(edge.from_ref.clone())
            .or_default()
            .push(edge.clone());
    }

    let mut found: Option<Vec<RuntimeGraphEdge>> = None;
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    visited.insert(from_ref.clone());
    queue.push_back((from_ref.clone(), Vec::<RuntimeGraphEdge>::new()));

    while let Some((current, path)) = queue.pop_front() {
        if current == to_ref {
            found = Some(path);
            break;
        }
        if path.len() >= max_depth {
            continue;
        }
        for edge in outgoing.get(&current).into_iter().flatten() {
            if visited.insert(edge.to_ref.clone()) {
                let mut next_path = path.clone();
                next_path.push(edge.clone());
                queue.push_back((edge.to_ref.clone(), next_path));
            }
        }
    }

    println!("graph_path:");
    println!("case_ref: {case_ref}");
    println!("from_ref: {from_ref}");
    println!("to_ref: {to_ref}");
    println!("max_depth: {max_depth}");
    if depth_clamped {
        println!("max_depth_clamped: yes");
    }
    match found {
        Some(edges) => {
            println!("path_status: found");
            println!("hops: {}", edges.len());
            print_graph_edges(&edges);
        }
        None => {
            println!("path_status: not_found");
            println!("hops: 0");
            println!("edges: none");
        }
    }
    Ok(())
}
