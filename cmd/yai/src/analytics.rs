//! Derived DuckDB fact extraction and reporting boundary.

use super::*;

const FACT_INIT_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS fact_receipt (
  fact_id TEXT PRIMARY KEY,
  case_ref TEXT,
  subject_ref TEXT,
  receipt_id TEXT,
  attempt_id TEXT,
  decision_id TEXT,
  receipt_kind TEXT,
  receipt_status TEXT,
  carrier_family TEXT,
  carrier_outcome TEXT,
  carrier_attempted BOOLEAN,
  execution_performed BOOLEAN,
  guarantee_mode TEXT,
  asserted_by_event_ref TEXT,
  source_record_refs TEXT,
  source_graph_refs TEXT,
  evidence_refs TEXT,
  transaction_time BIGINT,
  valid_time_start BIGINT,
  valid_time_end BIGINT,
  known_at BIGINT,
  status TEXT,
  revision_of TEXT,
  superseded_by TEXT,
  retracted_by TEXT,
  confidence DOUBLE,
  authority_scope TEXT,
  source_record_id TEXT,
  source_record_kind TEXT,
  source_schema TEXT,
  fact_schema TEXT,
  created_at_unix_ms BIGINT
);

CREATE TABLE IF NOT EXISTS fact_decision (
  fact_id TEXT PRIMARY KEY,
  case_ref TEXT,
  subject_ref TEXT,
  decision_id TEXT,
  attempt_id TEXT,
  decision_outcome TEXT,
  gate_outcome TEXT,
  policy_ref TEXT,
  requires_review BOOLEAN,
  review_id TEXT,
  asserted_by_event_ref TEXT,
  source_record_refs TEXT,
  source_graph_refs TEXT,
  evidence_refs TEXT,
  transaction_time BIGINT,
  valid_time_start BIGINT,
  valid_time_end BIGINT,
  known_at BIGINT,
  status TEXT,
  revision_of TEXT,
  superseded_by TEXT,
  retracted_by TEXT,
  confidence DOUBLE,
  authority_scope TEXT,
  source_record_id TEXT,
  source_record_kind TEXT,
  source_schema TEXT,
  fact_schema TEXT,
  created_at_unix_ms BIGINT
);

CREATE TABLE IF NOT EXISTS fact_projection (
  fact_id TEXT PRIMARY KEY,
  case_ref TEXT,
  subject_ref TEXT,
  projection_id TEXT,
  projection_kind TEXT,
  consumer TEXT,
  freshness TEXT,
  freshness_source TEXT,
  stale_reason TEXT,
  redaction TEXT,
  source_records BIGINT,
  source_receipts BIGINT,
  source_memory BIGINT,
  asserted_by_event_ref TEXT,
  source_record_refs TEXT,
  source_graph_refs TEXT,
  evidence_refs TEXT,
  transaction_time BIGINT,
  valid_time_start BIGINT,
  valid_time_end BIGINT,
  known_at BIGINT,
  status TEXT,
  revision_of TEXT,
  superseded_by TEXT,
  retracted_by TEXT,
  confidence DOUBLE,
  authority_scope TEXT,
  source_record_id TEXT,
  source_record_kind TEXT,
  source_schema TEXT,
  fact_schema TEXT,
  created_at_unix_ms BIGINT
);

CREATE TABLE IF NOT EXISTS fact_carrier_outcome (
  fact_id TEXT PRIMARY KEY,
  case_ref TEXT,
  subject_ref TEXT,
  carrier_family TEXT,
  carrier_mode TEXT,
  carrier_status TEXT,
  requested_outcome TEXT,
  effective_outcome TEXT,
  execution_available BOOLEAN,
  execution_performed BOOLEAN,
  carrier_attempted BOOLEAN,
  receipt_required BOOLEAN,
  receipt_posture TEXT,
  divergence_candidate TEXT,
  asserted_by_event_ref TEXT,
  source_record_refs TEXT,
  source_graph_refs TEXT,
  evidence_refs TEXT,
  transaction_time BIGINT,
  valid_time_start BIGINT,
  valid_time_end BIGINT,
  known_at BIGINT,
  status TEXT,
  revision_of TEXT,
  superseded_by TEXT,
  retracted_by TEXT,
  confidence DOUBLE,
  authority_scope TEXT,
  source_record_id TEXT,
  source_record_kind TEXT,
  source_schema TEXT,
  fact_schema TEXT,
  created_at_unix_ms BIGINT
);

CREATE TABLE IF NOT EXISTS fact_divergence (
  fact_id TEXT PRIMARY KEY,
  case_ref TEXT,
  subject_ref TEXT,
  divergence_id TEXT,
  divergence_kind TEXT,
  severity TEXT,
  decision_id TEXT,
  receipt_id TEXT,
  attempt_id TEXT,
  carrier_family TEXT,
  expected_state TEXT,
  observed_state TEXT,
  asserted_by_event_ref TEXT,
  source_record_refs TEXT,
  source_graph_refs TEXT,
  evidence_refs TEXT,
  transaction_time BIGINT,
  valid_time_start BIGINT,
  valid_time_end BIGINT,
  known_at BIGINT,
  status TEXT,
  revision_of TEXT,
  superseded_by TEXT,
  retracted_by TEXT,
  confidence DOUBLE,
  authority_scope TEXT,
  source_record_id TEXT,
  source_record_kind TEXT,
  source_schema TEXT,
  fact_schema TEXT,
  created_at_unix_ms BIGINT
);

CREATE TABLE IF NOT EXISTS fact_replay (
  fact_id TEXT PRIMARY KEY,
  case_ref TEXT,
  subject_ref TEXT,
  journal_identity TEXT,
  journal_path TEXT,
  replay_status TEXT,
  compatibility TEXT,
  record_schema TEXT,
  journal_schema TEXT,
  cursor_line BIGINT,
  lines_total BIGINT,
  records_seen BIGINT,
  records_written BIGINT,
  records_duplicate BIGINT,
  records_skipped BIGINT,
  invalid_entries BIGINT,
  report_ref TEXT,
  asserted_by_event_ref TEXT,
  source_record_refs TEXT,
  source_graph_refs TEXT,
  evidence_refs TEXT,
  transaction_time BIGINT,
  valid_time_start BIGINT,
  valid_time_end BIGINT,
  known_at BIGINT,
  status TEXT,
  revision_of TEXT,
  superseded_by TEXT,
  retracted_by TEXT,
  confidence DOUBLE,
  authority_scope TEXT,
  source_record_id TEXT,
  source_record_kind TEXT,
  source_schema TEXT,
  fact_schema TEXT,
  created_at_unix_ms BIGINT
);

CREATE TABLE IF NOT EXISTS fact_runtime_graph (
  fact_id TEXT PRIMARY KEY,
  case_ref TEXT,
  subject_ref TEXT,
  source_mode TEXT,
  nodes_total BIGINT,
  edges_total BIGINT,
  relations_seen BIGINT,
  relations_written BIGINT,
  relations_duplicate BIGINT,
  outgoing_index_entries BIGINT,
  incoming_index_entries BIGINT,
  runtime_generation BIGINT,
  rebuild_status TEXT,
  report_ref TEXT,
  asserted_by_event_ref TEXT,
  source_record_refs TEXT,
  source_graph_refs TEXT,
  evidence_refs TEXT,
  transaction_time BIGINT,
  valid_time_start BIGINT,
  valid_time_end BIGINT,
  known_at BIGINT,
  status TEXT,
  revision_of TEXT,
  superseded_by TEXT,
  retracted_by TEXT,
  confidence DOUBLE,
  authority_scope TEXT,
  source_record_id TEXT,
  source_record_kind TEXT,
  source_schema TEXT,
  fact_schema TEXT,
  created_at_unix_ms BIGINT
);

CREATE TABLE IF NOT EXISTS fact_model_behavior (
  fact_id TEXT PRIMARY KEY,
  case_ref TEXT,
  subject_ref TEXT,
  model_ref TEXT,
  provider_ref TEXT,
  model_output_id TEXT,
  behavior_kind TEXT,
  unsupported_claim BOOLEAN,
  authority_overclaim BOOLEAN,
  refusal BOOLEAN,
  tool_call_proposed BOOLEAN,
  filesystem_operation_proposed BOOLEAN,
  review_required BOOLEAN,
  output_chars BIGINT,
  asserted_by_event_ref TEXT,
  source_record_refs TEXT,
  source_graph_refs TEXT,
  evidence_refs TEXT,
  transaction_time BIGINT,
  valid_time_start BIGINT,
  valid_time_end BIGINT,
  known_at BIGINT,
  status TEXT,
  revision_of TEXT,
  superseded_by TEXT,
  retracted_by TEXT,
  confidence DOUBLE,
  authority_scope TEXT,
  source_record_id TEXT,
  source_record_kind TEXT,
  source_schema TEXT,
  fact_schema TEXT,
  created_at_unix_ms BIGINT
);

CREATE TABLE IF NOT EXISTS fact_policy_outcome (
  fact_id TEXT PRIMARY KEY,
  case_ref TEXT,
  subject_ref TEXT,
  policy_ref TEXT,
  policy_kind TEXT,
  operation_kind TEXT,
  decision_id TEXT,
  attempt_id TEXT,
  review_id TEXT,
  policy_outcome TEXT,
  requires_review BOOLEAN,
  blocked BOOLEAN,
  approved BOOLEAN,
  denied BOOLEAN,
  deferred BOOLEAN,
  quarantined BOOLEAN,
  asserted_by_event_ref TEXT,
  source_record_refs TEXT,
  source_graph_refs TEXT,
  evidence_refs TEXT,
  transaction_time BIGINT,
  valid_time_start BIGINT,
  valid_time_end BIGINT,
  known_at BIGINT,
  status TEXT,
  revision_of TEXT,
  superseded_by TEXT,
  retracted_by TEXT,
  confidence DOUBLE,
  authority_scope TEXT,
  source_record_id TEXT,
  source_record_kind TEXT,
  source_schema TEXT,
  fact_schema TEXT,
  created_at_unix_ms BIGINT
);

CREATE TABLE IF NOT EXISTS fact_memory_quality (
  fact_id TEXT PRIMARY KEY,
  case_ref TEXT,
  subject_ref TEXT,
  memory_ref TEXT,
  memory_kind TEXT,
  memory_scope TEXT,
  basis_record_count BIGINT,
  basis_receipt_count BIGINT,
  basis_edge_count BIGINT,
  freshness TEXT,
  quality_status TEXT,
  requires_review BOOLEAN,
  asserted_by_event_ref TEXT,
  source_record_refs TEXT,
  source_graph_refs TEXT,
  evidence_refs TEXT,
  transaction_time BIGINT,
  valid_time_start BIGINT,
  valid_time_end BIGINT,
  known_at BIGINT,
  status TEXT,
  revision_of TEXT,
  superseded_by TEXT,
  retracted_by TEXT,
  confidence DOUBLE,
  authority_scope TEXT,
  source_record_id TEXT,
  source_record_kind TEXT,
  source_schema TEXT,
  fact_schema TEXT,
  created_at_unix_ms BIGINT
);

CREATE TABLE IF NOT EXISTS fact_retrieval_quality (
  fact_id TEXT PRIMARY KEY,
  case_ref TEXT,
  subject_ref TEXT,
  provider_ref TEXT,
  provider_kind TEXT,
  query_ref TEXT,
  results_returned BIGINT,
  results_selected BIGINT,
  results_rejected BIGINT,
  latency_ms BIGINT,
  cost DOUBLE,
  provenance_quality TEXT,
  scope_violation_count BIGINT,
  duplicate_count BIGINT,
  selected_for_context BOOLEAN,
  promoted_to_case_material BOOLEAN,
  asserted_by_event_ref TEXT,
  source_record_refs TEXT,
  source_graph_refs TEXT,
  evidence_refs TEXT,
  transaction_time BIGINT,
  valid_time_start BIGINT,
  valid_time_end BIGINT,
  known_at BIGINT,
  status TEXT,
  revision_of TEXT,
  superseded_by TEXT,
  retracted_by TEXT,
  confidence DOUBLE,
  authority_scope TEXT,
  source_record_id TEXT,
  source_record_kind TEXT,
  source_schema TEXT,
  fact_schema TEXT,
  created_at_unix_ms BIGINT
);

CREATE TABLE IF NOT EXISTS fact_provider_runtime (
  fact_id TEXT PRIMARY KEY,
  case_ref TEXT,
  subject_ref TEXT,
  asserted_by_event_ref TEXT,
  source_record_refs TEXT,
  source_graph_refs TEXT,
  evidence_refs TEXT,
  transaction_time BIGINT,
  valid_time_start BIGINT,
  valid_time_end BIGINT,
  known_at BIGINT,
  status TEXT,
  revision_of TEXT,
  superseded_by TEXT,
  retracted_by TEXT,
  confidence DOUBLE,
  authority_scope TEXT,
  source_record_id TEXT,
  source_record_kind TEXT,
  source_schema TEXT,
  fact_schema TEXT,
  created_at_unix_ms BIGINT
);

ALTER TABLE fact_policy_outcome ADD COLUMN IF NOT EXISTS policy_ref TEXT;
ALTER TABLE fact_policy_outcome ADD COLUMN IF NOT EXISTS policy_kind TEXT;
ALTER TABLE fact_policy_outcome ADD COLUMN IF NOT EXISTS operation_kind TEXT;
ALTER TABLE fact_policy_outcome ADD COLUMN IF NOT EXISTS decision_id TEXT;
ALTER TABLE fact_policy_outcome ADD COLUMN IF NOT EXISTS attempt_id TEXT;
ALTER TABLE fact_policy_outcome ADD COLUMN IF NOT EXISTS review_id TEXT;
ALTER TABLE fact_policy_outcome ADD COLUMN IF NOT EXISTS policy_outcome TEXT;
ALTER TABLE fact_policy_outcome ADD COLUMN IF NOT EXISTS requires_review BOOLEAN;
ALTER TABLE fact_policy_outcome ADD COLUMN IF NOT EXISTS blocked BOOLEAN;
ALTER TABLE fact_policy_outcome ADD COLUMN IF NOT EXISTS approved BOOLEAN;
ALTER TABLE fact_policy_outcome ADD COLUMN IF NOT EXISTS denied BOOLEAN;
ALTER TABLE fact_policy_outcome ADD COLUMN IF NOT EXISTS deferred BOOLEAN;
ALTER TABLE fact_policy_outcome ADD COLUMN IF NOT EXISTS quarantined BOOLEAN;
ALTER TABLE fact_memory_quality ADD COLUMN IF NOT EXISTS memory_ref TEXT;
ALTER TABLE fact_memory_quality ADD COLUMN IF NOT EXISTS memory_kind TEXT;
ALTER TABLE fact_memory_quality ADD COLUMN IF NOT EXISTS memory_scope TEXT;
ALTER TABLE fact_memory_quality ADD COLUMN IF NOT EXISTS basis_record_count BIGINT;
ALTER TABLE fact_memory_quality ADD COLUMN IF NOT EXISTS basis_receipt_count BIGINT;
ALTER TABLE fact_memory_quality ADD COLUMN IF NOT EXISTS basis_edge_count BIGINT;
ALTER TABLE fact_memory_quality ADD COLUMN IF NOT EXISTS freshness TEXT;
ALTER TABLE fact_memory_quality ADD COLUMN IF NOT EXISTS quality_status TEXT;
ALTER TABLE fact_memory_quality ADD COLUMN IF NOT EXISTS requires_review BOOLEAN;
"#;

pub(super) fn facts_status(args: &[String]) -> Result<(), String> {
    if !args.is_empty() {
        return Err("usage: yai facts status".to_string());
    }
    let path = facts_store_path();
    let status = if path.is_file() {
        "ready"
    } else {
        "not_initialized"
    };
    println!("fact_plane:");
    println!("backend: duckdb");
    println!("status: {status}");
    println!("facts_path: {}", path.display());
    println!("schema: {FACT_SCHEMA}");
    println!("bitemporal: yes");
    if status == "ready" {
        let counts = fact_counts(None)?;
        println!("tables: {}", FACT_TABLES.len());
        println!("facts_extracted: {}", counts.total);
        println!("fact_receipt: {}", counts.receipt);
        println!("fact_decision: {}", counts.decision);
        println!("fact_projection: {}", counts.projection);
        println!("fact_carrier_outcome: {}", counts.carrier_outcome);
        println!("fact_divergence: {}", counts.divergence);
        println!("fact_model_behavior: {}", counts.model_behavior);
        println!("fact_policy_outcome: {}", counts.policy_outcome);
        println!("fact_memory_quality: {}", counts.memory_quality);
    }
    println!("facts_are_truth: false");
    println!("operational_truth: false");
    Ok(())
}

pub(super) fn facts_schema(args: &[String]) -> Result<(), String> {
    if !args.is_empty() {
        return Err("usage: yai facts schema".to_string());
    }
    println!("fact_schema: {FACT_SCHEMA}");
    println!("bitemporal: yes");
    println!("facts_are_truth: false");
    println!("tables:");
    for table in FACT_TABLES {
        println!("- {table}");
    }
    println!("common_columns:");
    for column in FACT_COMMON_COLUMNS {
        println!("- {column}");
    }
    println!("extraction:");
    println!("  facts_extracted: 0");
    println!(
        "  extraction_status: receipt_decision_projection_model_behavior_policy_outcome_memory_divergence_carrier_active"
    );
    println!("  valid_time_end_sentinel: 0");
    Ok(())
}

pub(super) fn facts_init(args: &[String]) -> Result<(), String> {
    if !args.is_empty() {
        return Err("usage: yai facts init".to_string());
    }
    let dir = facts_store_dir();
    let path = facts_store_path();
    fs::create_dir_all(&dir).map_err(|err| {
        format!(
            "failed to create facts store directory {}: {err}",
            dir.display()
        )
    })?;
    let output = Command::new("duckdb")
        .arg(&path)
        .arg("-c")
        .arg(FACT_INIT_SQL)
        .output()
        .map_err(|err| format!("duckdb executable unavailable: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        return Err(format!("duckdb facts init failed: {detail}"));
    }
    println!("facts_init:");
    println!("backend: duckdb");
    println!("status: ready");
    println!("facts_path: {}", path.display());
    println!("schema: {FACT_SCHEMA}");
    println!("bitemporal: yes");
    println!("tables_created: {}", FACT_TABLES.len());
    println!("facts_extracted: 0");
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FactExtractKind {
    Receipt,
    Decision,
    Projection,
    CarrierOutcome,
    Divergence,
    ModelBehavior,
    PolicyOutcome,
    MemoryQuality,
}

impl FactExtractKind {
    fn from_arg(value: &str) -> Option<Self> {
        match value {
            "receipt" => Some(Self::Receipt),
            "decision" => Some(Self::Decision),
            "projection" => Some(Self::Projection),
            "carrier_outcome" => Some(Self::CarrierOutcome),
            "divergence" => Some(Self::Divergence),
            "model_behavior" => Some(Self::ModelBehavior),
            "policy_outcome" => Some(Self::PolicyOutcome),
            "memory_quality" => Some(Self::MemoryQuality),
            _ => None,
        }
    }

    fn output_kind(self) -> &'static str {
        match self {
            Self::Receipt => "receipt",
            Self::Decision => "decision",
            Self::Projection => "projection",
            Self::CarrierOutcome => "carrier_outcome",
            Self::Divergence => "divergence",
            Self::ModelBehavior => "model_behavior",
            Self::PolicyOutcome => "policy_outcome",
            Self::MemoryQuality => "memory_quality",
        }
    }

    fn table(self) -> &'static str {
        match self {
            Self::Receipt => "fact_receipt",
            Self::Decision => "fact_decision",
            Self::Projection => "fact_projection",
            Self::CarrierOutcome => "fact_carrier_outcome",
            Self::Divergence => "fact_divergence",
            Self::ModelBehavior => "fact_model_behavior",
            Self::PolicyOutcome => "fact_policy_outcome",
            Self::MemoryQuality => "fact_memory_quality",
        }
    }

    fn fact_id_prefix(self) -> &'static str {
        match self {
            Self::Receipt => "fact:receipt:",
            Self::Decision => "fact:decision:",
            Self::Projection => "fact:projection:",
            Self::CarrierOutcome => "fact:carrier_outcome:",
            Self::Divergence => "fact:divergence:",
            Self::ModelBehavior => "fact:model_behavior:",
            Self::PolicyOutcome => "fact:policy_outcome:",
            Self::MemoryQuality => "fact:memory_quality:",
        }
    }
}

#[derive(Clone, Debug, Default)]
struct FactExtractionStats {
    records_scanned: usize,
    facts_written: usize,
    facts_duplicate: usize,
    facts_skipped: usize,
}

#[derive(Clone, Debug, Default)]
struct FactCounts {
    receipt: usize,
    decision: usize,
    projection: usize,
    carrier_outcome: usize,
    divergence: usize,
    model_behavior: usize,
    policy_outcome: usize,
    memory_quality: usize,
    total: usize,
}

fn ensure_facts_ready() -> Result<(), String> {
    let path = facts_store_path();
    if path.is_file() {
        return Ok(());
    }
    Err(format!(
        "fact plane is not initialized: run yai facts init; facts_path: {}",
        path.display()
    ))
}

fn duckdb_run(args: &[&str]) -> Result<String, String> {
    let output = Command::new("duckdb")
        .args(args)
        .output()
        .map_err(|err| format!("duckdb executable unavailable: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        return Err(format!("duckdb command failed: {detail}"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn duckdb_query_csv(sql: &str) -> Result<String, String> {
    let path = facts_store_path();
    let path_string = path.display().to_string();
    duckdb_run(&[&path_string, "-csv", "-noheader", "-c", sql])
}

fn duckdb_exec_sql(sql: &str) -> Result<(), String> {
    let path = facts_store_path();
    let path_string = path.display().to_string();
    duckdb_run(&[&path_string, "-c", sql]).map(|_| ())
}

fn duckdb_count(sql: &str) -> Result<usize, String> {
    duckdb_query_csv(sql)?
        .trim()
        .parse::<usize>()
        .map_err(|err| format!("invalid duckdb count output for `{sql}`: {err}"))
}

fn csv_field_value(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed[1..trimmed.len() - 1].replace("\"\"", "\"")
    } else {
        trimmed.to_string()
    }
}

fn duckdb_group_counts(sql: &str) -> Result<Vec<(String, usize)>, String> {
    duckdb_query_csv(sql)?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let mut parts = line.splitn(2, ',');
            let key = csv_field_value(parts.next().unwrap_or("unknown"));
            let count = parts
                .next()
                .unwrap_or("0")
                .trim()
                .parse::<usize>()
                .map_err(|err| format!("invalid duckdb group count output for `{sql}`: {err}"))?;
            Ok((key, count))
        })
        .collect()
}

fn sql_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn sql_bool(value: bool) -> &'static str {
    if value {
        "TRUE"
    } else {
        "FALSE"
    }
}

fn fact_counts(case_ref: Option<&str>) -> Result<FactCounts, String> {
    ensure_facts_ready()?;
    let where_clause = case_ref
        .map(|case_ref| format!(" WHERE case_ref = {}", sql_quote(case_ref)))
        .unwrap_or_default();
    let receipt = duckdb_count(&format!("SELECT count(*) FROM fact_receipt{where_clause};"))?;
    let decision = duckdb_count(&format!(
        "SELECT count(*) FROM fact_decision{where_clause};"
    ))?;
    let projection = duckdb_count(&format!(
        "SELECT count(*) FROM fact_projection{where_clause};"
    ))?;
    let carrier_outcome = duckdb_count(&format!(
        "SELECT count(*) FROM fact_carrier_outcome{where_clause};"
    ))?;
    let divergence = duckdb_count(&format!(
        "SELECT count(*) FROM fact_divergence{where_clause};"
    ))?;
    let model_behavior = duckdb_count(&format!(
        "SELECT count(*) FROM fact_model_behavior{where_clause};"
    ))?;
    let policy_outcome = duckdb_count(&format!(
        "SELECT count(*) FROM fact_policy_outcome{where_clause};"
    ))?;
    let memory_quality = duckdb_count(&format!(
        "SELECT count(*) FROM fact_memory_quality{where_clause};"
    ))?;
    let total = FACT_TABLES
        .iter()
        .map(|table| duckdb_count(&format!("SELECT count(*) FROM {table}{where_clause};")))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .sum();
    Ok(FactCounts {
        receipt,
        decision,
        projection,
        carrier_outcome,
        divergence,
        model_behavior,
        policy_outcome,
        memory_quality,
        total,
    })
}

fn existing_fact_ids(table: &str, case_ref: &str) -> Result<HashSet<String>, String> {
    let query = format!(
        "SELECT fact_id FROM {table} WHERE case_ref = {};",
        sql_quote(case_ref)
    );
    Ok(duckdb_query_csv(&query)?
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect())
}

fn summary_token_value(summary: &str, key: &str) -> String {
    let prefix = format!("{key}:");
    summary
        .split_whitespace()
        .find_map(|part| part.strip_prefix(&prefix))
        .unwrap_or("")
        .to_string()
}

fn summary_token_value_or(summary: &str, key: &str, fallback: &str) -> String {
    let value = summary_token_value(summary, key);
    if value.is_empty() {
        fallback.to_string()
    } else {
        value
    }
}

fn summary_bool(summary: &str, key: &str, fallback: bool) -> bool {
    match summary_token_value(summary, key).as_str() {
        "true" | "yes" => true,
        "false" | "no" => false,
        _ => fallback,
    }
}

fn summary_number(summary: &str, key: &str) -> usize {
    summary_token_value(summary, key).parse().unwrap_or(0)
}

fn source_record_summary(record: &StoredRecordEnvelope) -> String {
    json_string_or(&record.raw_json, "summary", "")
}

fn source_record_subject_ref(record: &StoredRecordEnvelope) -> String {
    json_string_or(&record.raw_json, "subject_ref", "")
}

fn source_record_attempt_id(record: &StoredRecordEnvelope) -> String {
    json_string_or(&record.raw_json, "attempt_id", "")
}

fn source_record_decision_id(record: &StoredRecordEnvelope) -> String {
    json_string_or(&record.raw_json, "decision_id", "")
}

fn source_record_receipt_id(record: &StoredRecordEnvelope) -> String {
    json_string_or(&record.raw_json, "receipt_id", "")
}

fn source_valid_time_start(record: &StoredRecordEnvelope, transaction_time: u128) -> u128 {
    json_number_field(&record.raw_json, "created_at_unix_ms")
        .and_then(|value| value.parse::<u128>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(transaction_time)
}

fn fact_common_sql_values(record: &StoredRecordEnvelope, transaction_time: u128) -> String {
    let valid_time_start = source_valid_time_start(record, transaction_time);
    [
        sql_quote(""),
        sql_quote(&record.record_id),
        sql_quote(""),
        sql_quote(""),
        transaction_time.to_string(),
        valid_time_start.to_string(),
        FACT_VALID_TIME_END_SENTINEL.to_string(),
        transaction_time.to_string(),
        sql_quote("current"),
        sql_quote(""),
        sql_quote(""),
        sql_quote(""),
        "1.0".to_string(),
        sql_quote(&summary_token_value(
            &source_record_summary(record),
            "authority_scope",
        )),
        sql_quote(&record.record_id),
        sql_quote(&record.record_kind),
        sql_quote(&record.schema),
        sql_quote(FACT_SCHEMA),
        transaction_time.to_string(),
    ]
    .join(", ")
}

fn receipt_status_from_summary(summary: &str) -> String {
    let explicit = summary_token_value(summary, "receipt_status");
    if !explicit.is_empty() {
        return explicit;
    }
    let status = summary_token_value(summary, "status");
    if !status.is_empty() {
        return status;
    }
    summary_token_value(summary, "receipt")
}

fn receipt_carrier_family(record: &StoredRecordEnvelope, summary: &str) -> String {
    let explicit = summary_token_value(summary, "carrier_family");
    if !explicit.is_empty() {
        return explicit;
    }
    match record.record_kind.as_str() {
        "filesystem_receipt" => "filesystem".to_string(),
        "process_receipt" => "process".to_string(),
        "carrier_receipt" => "unknown".to_string(),
        _ => String::new(),
    }
}

fn source_record_text(record: &StoredRecordEnvelope) -> String {
    format!("{} {}", record.record_kind, source_record_summary(record)).to_lowercase()
}

fn text_contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn model_behavior_kind(record: &StoredRecordEnvelope, text: &str) -> &'static str {
    match record.record_kind.as_str() {
        "model_output" => "provider_output_observed",
        "model_interpretation" => "model_interpretation_observed",
        "effect_receipt" if text.contains("model.output") => "provider_output_observed",
        _ if text.contains("output_is:claim_or_proposal") => "claim_or_proposal",
        _ if text_contains_any(
            text,
            &[
                "raw_journal_access",
                "raw journal",
                "filesystem_access",
                "directly the file",
            ],
        ) =>
        {
            "raw_access_request"
        }
        _ if text_contains_any(
            text,
            &[
                "change decision",
                "mutate decision",
                "approve action",
                "approve its own",
                "decision_mutation_request",
            ],
        ) =>
        {
            "decision_mutation_request"
        }
        _ if text_contains_any(
            text,
            &[
                "fs.write",
                "fs.read",
                "filesystem write",
                "filesystem_operation_proposed",
                "tool_call",
            ],
        ) =>
        {
            "filesystem_operation_proposal"
        }
        _ if text_contains_any(text, &["refusal:true", "boundary_refusal"]) => "boundary_refusal",
        _ if text_contains_any(text, &["provider_attachment:", "case_entry:"]) => {
            "claim_or_proposal"
        }
        _ => "unknown",
    }
}

fn model_behavior_flags(
    record: &StoredRecordEnvelope,
    text: &str,
) -> (bool, bool, bool, bool, bool, bool) {
    let summary = source_record_summary(record);
    let unsupported_claim = summary_bool(&summary, "unsupported_claim", false)
        || text.contains("unsupported_claim:true");
    let authority_overclaim = summary_bool(&summary, "authority_overclaim", false)
        || text_contains_any(
            text,
            &[
                "authority_overclaim:true",
                "decision_mutation_request",
                "change decision",
                "mutate decision",
                "approve action",
                "approve its own",
                "policy engine",
                "receipt authority",
            ],
        );
    let refusal = summary_bool(&summary, "refusal", false)
        || text_contains_any(text, &["refusal:true", "boundary_refusal"]);
    let tool_call_proposed = summary_bool(&summary, "tool_call_proposed", false)
        || text_contains_any(text, &["tool_call", "tool execution"]);
    let filesystem_operation_proposed =
        summary_bool(&summary, "filesystem_operation_proposed", false)
            || text_contains_any(
                text,
                &[
                    "fs.write",
                    "fs.read",
                    "filesystem write",
                    "filesystem_operation_proposal",
                    "filesystem_operation_proposed:true",
                ],
            );
    let review_required = summary_bool(&summary, "review_required", false)
        || text_contains_any(
            text,
            &["require_review", "pending_operator", "review_required"],
        );
    (
        unsupported_claim,
        authority_overclaim,
        refusal,
        tool_call_proposed,
        filesystem_operation_proposed,
        review_required,
    )
}

fn policy_outcome_kind(record: &StoredRecordEnvelope, text: &str) -> &'static str {
    match record.record_kind.as_str() {
        "policy_rule" => "policy_rule_defined",
        "authority_scope" => "authority_scope_defined",
        "projection_rule" => "projection_rule_defined",
        "decision" if text_contains_any(text, &["allow_with_constraints", "decision:allow"]) => {
            "decision_allowed"
        }
        "decision" if text_contains_any(text, &["decision:deny", "status:blocked", "blocked"]) => {
            "decision_denied"
        }
        "decision" if text.contains("require_review") => "review_required",
        "review_request" if text.contains("approved") => "review_approved",
        "review_request" if text.contains("denied") => "review_denied",
        "review_request" if text.contains("deferred") => "review_deferred",
        "review_request" if text.contains("quarantined") => "review_quarantined",
        "review_request" => "review_required",
        "review_decision" if text.contains("action:approve") => "review_approved",
        "review_decision" if text.contains("action:deny") => "review_denied",
        "review_decision" if text.contains("action:defer") => "review_deferred",
        "review_decision" if text.contains("action:quarantine") => "review_quarantined",
        "control_pending" if text.contains("approved") => "review_approved",
        "control_pending" if text.contains("denied") => "review_denied",
        "control_pending" if text.contains("deferred") => "review_deferred",
        "control_pending" if text.contains("quarantined") => "review_quarantined",
        "control_pending" => "review_required",
        "carrier_outcome" if text.contains("blocked") => "carrier_blocked",
        "divergence" => "divergence_detected",
        _ => "unknown",
    }
}

fn policy_outcome_flags(text: &str, outcome: &str) -> (bool, bool, bool, bool, bool, bool) {
    let requires_review = outcome == "review_required"
        || text_contains_any(text, &["require_review", "pending_operator"]);
    let blocked =
        outcome == "decision_denied" || outcome == "carrier_blocked" || text.contains("blocked");
    let approved = outcome == "review_approved" || text.contains("approved");
    let denied = outcome == "review_denied"
        || outcome == "decision_denied"
        || text.contains("denied")
        || text.contains("decision:deny");
    let deferred = outcome == "review_deferred" || text.contains("deferred");
    let quarantined = outcome == "review_quarantined" || text.contains("quarantined");
    (
        requires_review,
        blocked,
        approved,
        denied,
        deferred,
        quarantined,
    )
}

fn carrier_status_from_summary(record: &StoredRecordEnvelope, summary: &str) -> String {
    let explicit = summary_token_value(summary, "carrier_status");
    if !explicit.is_empty() {
        return explicit;
    }
    let receipt_status = receipt_status_from_summary(summary);
    if !receipt_status.is_empty() {
        return receipt_status;
    }
    match record.record_kind.as_str() {
        "divergence" => "divergence_candidate".to_string(),
        _ => "unknown".to_string(),
    }
}

fn carrier_effective_outcome(record: &StoredRecordEnvelope, summary: &str) -> String {
    summary_token_value_or(
        summary,
        "effective_outcome",
        &summary_token_value_or(
            summary,
            "carrier_outcome",
            &carrier_status_from_summary(record, summary),
        ),
    )
}

fn carrier_execution_flags(summary: &str, outcome: &str) -> (bool, bool, bool) {
    let execution_available = !matches!(
        outcome,
        "blocked" | "deferred" | "quarantined" | "not_attempted" | "unknown"
    );
    let execution_performed = summary_bool(
        summary,
        "execution_performed",
        matches!(outcome, "executed" | "observed"),
    );
    let carrier_attempted = summary_bool(
        summary,
        "carrier_attempted",
        matches!(outcome, "executed" | "observed" | "failed" | "mismatch"),
    );
    (execution_available, execution_performed, carrier_attempted)
}

fn divergence_kind(record: &StoredRecordEnvelope, summary: &str) -> String {
    summary_token_value_or(
        summary,
        "divergence_kind",
        &summary_token_value_or(summary, "divergence", record.record_kind.as_str()),
    )
}

fn divergence_severity(summary: &str) -> String {
    summary_token_value_or(summary, "severity", "unknown")
}

fn memory_quality_status(summary: &str) -> String {
    let explicit = summary_token_value(summary, "quality_status");
    if !explicit.is_empty() {
        return explicit;
    }
    let basis_records = summary_number(summary, "basis_records");
    let basis_receipts = summary_number(summary, "basis_receipts");
    let basis_edges = summary_number(summary, "basis_edges");
    if basis_records > 0 || basis_receipts > 0 || basis_edges > 0 {
        "basis_present".to_string()
    } else {
        "candidate_observed".to_string()
    }
}

fn source_record_matches_fact_kind(record: &StoredRecordEnvelope, kind: FactExtractKind) -> bool {
    match kind {
        FactExtractKind::Receipt => matches!(
            record.record_kind.as_str(),
            "receipt"
                | "filesystem_receipt"
                | "effect_receipt"
                | "carrier_receipt"
                | "process_receipt"
        ),
        FactExtractKind::Decision => {
            matches!(record.record_kind.as_str(), "decision" | "review_decision")
        }
        FactExtractKind::Projection => matches!(
            record.record_kind.as_str(),
            "projection_result" | "projection_request" | "participant_view_frame"
        ),
        FactExtractKind::CarrierOutcome => matches!(
            record.record_kind.as_str(),
            "carrier_outcome"
                | "carrier_request"
                | "filesystem_receipt"
                | "effect_receipt"
                | "process_receipt"
                | "carrier_receipt"
                | "divergence"
        ),
        FactExtractKind::Divergence => matches!(
            record.record_kind.as_str(),
            "divergence"
                | "carrier_consistency"
                | "reconcile_report"
                | "runtime_graph_rebuild_report"
                | "replay_report"
        ),
        FactExtractKind::ModelBehavior => {
            let summary = source_record_summary(record);
            matches!(
                record.record_kind.as_str(),
                "model_output"
                    | "model_interpretation"
                    | "interaction_turn"
                    | "participant_view_frame"
            ) || (record.record_kind == "effect_receipt" && summary.contains("model.output"))
                || (record.record_kind == "subject_state"
                    && (summary.contains("provider_attachment:")
                        || summary.contains("case_entry:admitted")))
        }
        FactExtractKind::PolicyOutcome => matches!(
            record.record_kind.as_str(),
            "policy_rule"
                | "authority_scope"
                | "projection_rule"
                | "decision"
                | "review_request"
                | "review_decision"
                | "control_pending"
                | "carrier_outcome"
                | "divergence"
        ),
        FactExtractKind::MemoryQuality => matches!(
            record.record_kind.as_str(),
            "memory_candidate" | "memory_unit" | "memory_consolidation"
        ),
    }
}

fn fact_insert_sql(
    kind: FactExtractKind,
    record: &StoredRecordEnvelope,
    transaction_time: u128,
) -> String {
    let summary = source_record_summary(record);
    let fact_id = format!("{}{}", kind.fact_id_prefix(), record.record_id);
    let subject_ref = source_record_subject_ref(record);
    match kind {
        FactExtractKind::Receipt => {
            let receipt_status = receipt_status_from_summary(&summary);
            let carrier_family = receipt_carrier_family(record, &summary);
            let carrier_outcome =
                summary_token_value_or(&summary, "carrier_outcome", &receipt_status);
            let carrier_attempted = summary_bool(&summary, "carrier_attempted", false);
            let execution_performed = summary_bool(&summary, "execution_performed", false);
            format!(
                "INSERT INTO fact_receipt (fact_id, case_ref, subject_ref, receipt_id, attempt_id, decision_id, receipt_kind, receipt_status, carrier_family, carrier_outcome, carrier_attempted, execution_performed, guarantee_mode, asserted_by_event_ref, source_record_refs, source_graph_refs, evidence_refs, transaction_time, valid_time_start, valid_time_end, known_at, status, revision_of, superseded_by, retracted_by, confidence, authority_scope, source_record_id, source_record_kind, source_schema, fact_schema, created_at_unix_ms) VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {});",
                sql_quote(&fact_id),
                sql_quote(&record.case_ref),
                sql_quote(&subject_ref),
                sql_quote(&source_record_receipt_id(record)),
                sql_quote(&source_record_attempt_id(record)),
                sql_quote(&source_record_decision_id(record)),
                sql_quote(&record.record_kind),
                sql_quote(&receipt_status),
                sql_quote(&carrier_family),
                sql_quote(&carrier_outcome),
                sql_bool(carrier_attempted),
                sql_bool(execution_performed),
                sql_quote(&summary_token_value(&summary, "guarantee_mode")),
                fact_common_sql_values(record, transaction_time)
            )
        }
        FactExtractKind::Decision => {
            let decision_outcome = summary_token_value_or(
                &summary,
                "decision_outcome",
                &summary_token_value(&summary, "decision"),
            );
            let gate_outcome = summary_token_value_or(
                &summary,
                "gate_outcome",
                if decision_outcome == "require_review" {
                    "require_review"
                } else {
                    ""
                },
            );
            let review_id = summary_token_value(&summary, "review_id");
            let requires_review = summary.contains("require_review") || !review_id.is_empty();
            let policy_ref = summary_token_value_or(
                &summary,
                "policy_ref",
                &summary_token_value(&summary, "rule"),
            );
            format!(
                "INSERT INTO fact_decision (fact_id, case_ref, subject_ref, decision_id, attempt_id, decision_outcome, gate_outcome, policy_ref, requires_review, review_id, asserted_by_event_ref, source_record_refs, source_graph_refs, evidence_refs, transaction_time, valid_time_start, valid_time_end, known_at, status, revision_of, superseded_by, retracted_by, confidence, authority_scope, source_record_id, source_record_kind, source_schema, fact_schema, created_at_unix_ms) VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {});",
                sql_quote(&fact_id),
                sql_quote(&record.case_ref),
                sql_quote(&subject_ref),
                sql_quote(&source_record_decision_id(record)),
                sql_quote(&source_record_attempt_id(record)),
                sql_quote(&decision_outcome),
                sql_quote(&gate_outcome),
                sql_quote(&policy_ref),
                sql_bool(requires_review),
                sql_quote(&review_id),
                fact_common_sql_values(record, transaction_time)
            )
        }
        FactExtractKind::Projection => {
            let projection_id =
                summary_token_value_or(&summary, "projection_id", &record.record_id);
            let projection_kind = summary_token_value_or(
                &summary,
                "projection_kind",
                match record.record_kind.as_str() {
                    "projection_request" => "request",
                    "projection_result" => "result",
                    "participant_view_frame" => "participant_view_frame",
                    _ => "",
                },
            );
            let freshness = summary_token_value_or(
                &summary,
                "freshness",
                &summary_token_value(&summary, "projection_freshness"),
            );
            format!(
                "INSERT INTO fact_projection (fact_id, case_ref, subject_ref, projection_id, projection_kind, consumer, freshness, freshness_source, stale_reason, redaction, source_records, source_receipts, source_memory, asserted_by_event_ref, source_record_refs, source_graph_refs, evidence_refs, transaction_time, valid_time_start, valid_time_end, known_at, status, revision_of, superseded_by, retracted_by, confidence, authority_scope, source_record_id, source_record_kind, source_schema, fact_schema, created_at_unix_ms) VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {});",
                sql_quote(&fact_id),
                sql_quote(&record.case_ref),
                sql_quote(&subject_ref),
                sql_quote(&projection_id),
                sql_quote(&projection_kind),
                sql_quote(&summary_token_value(&summary, "consumer")),
                sql_quote(&freshness),
                sql_quote(&summary_token_value(&summary, "freshness_source")),
                sql_quote(&summary_token_value(&summary, "stale_reason")),
                sql_quote(&summary_token_value(&summary, "redaction")),
                summary_number(&summary, "source_records"),
                summary_number(&summary, "source_receipts"),
                summary_number(&summary, "source_memory"),
                fact_common_sql_values(record, transaction_time)
            )
        }
        FactExtractKind::CarrierOutcome => {
            let carrier_family = receipt_carrier_family(record, &summary);
            let carrier_status = carrier_status_from_summary(record, &summary);
            let effective_outcome = carrier_effective_outcome(record, &summary);
            let requested_outcome =
                summary_token_value_or(&summary, "requested_outcome", &effective_outcome);
            let (execution_available, execution_performed, carrier_attempted) =
                carrier_execution_flags(&summary, &effective_outcome);
            let receipt_required = summary_bool(&summary, "receipt_required", true);
            let receipt_posture = summary_token_value_or(
                &summary,
                "receipt_posture",
                if source_record_receipt_id(record).is_empty() {
                    "unknown"
                } else {
                    "present"
                },
            );
            format!(
                "INSERT INTO fact_carrier_outcome (fact_id, case_ref, subject_ref, carrier_family, carrier_mode, carrier_status, requested_outcome, effective_outcome, execution_available, execution_performed, carrier_attempted, receipt_required, receipt_posture, divergence_candidate, asserted_by_event_ref, source_record_refs, source_graph_refs, evidence_refs, transaction_time, valid_time_start, valid_time_end, known_at, status, revision_of, superseded_by, retracted_by, confidence, authority_scope, source_record_id, source_record_kind, source_schema, fact_schema, created_at_unix_ms) VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {});",
                sql_quote(&fact_id),
                sql_quote(&record.case_ref),
                sql_quote(&subject_ref),
                sql_quote(&carrier_family),
                sql_quote(&summary_token_value(&summary, "carrier_mode")),
                sql_quote(&carrier_status),
                sql_quote(&requested_outcome),
                sql_quote(&effective_outcome),
                sql_bool(execution_available),
                sql_bool(execution_performed),
                sql_bool(carrier_attempted),
                sql_bool(receipt_required),
                sql_quote(&receipt_posture),
                sql_quote(&summary_token_value(&summary, "divergence_candidate")),
                fact_common_sql_values(record, transaction_time)
            )
        }
        FactExtractKind::Divergence => {
            let divergence_id =
                summary_token_value_or(&summary, "divergence_id", &record.record_id);
            let divergence_kind = divergence_kind(record, &summary);
            format!(
                "INSERT INTO fact_divergence (fact_id, case_ref, subject_ref, divergence_id, divergence_kind, severity, decision_id, receipt_id, attempt_id, carrier_family, expected_state, observed_state, asserted_by_event_ref, source_record_refs, source_graph_refs, evidence_refs, transaction_time, valid_time_start, valid_time_end, known_at, status, revision_of, superseded_by, retracted_by, confidence, authority_scope, source_record_id, source_record_kind, source_schema, fact_schema, created_at_unix_ms) VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {});",
                sql_quote(&fact_id),
                sql_quote(&record.case_ref),
                sql_quote(&subject_ref),
                sql_quote(&divergence_id),
                sql_quote(&divergence_kind),
                sql_quote(&divergence_severity(&summary)),
                sql_quote(&source_record_decision_id(record)),
                sql_quote(&source_record_receipt_id(record)),
                sql_quote(&source_record_attempt_id(record)),
                sql_quote(&receipt_carrier_family(record, &summary)),
                sql_quote(&summary_token_value(&summary, "expected_state")),
                sql_quote(&summary_token_value(&summary, "observed_state")),
                fact_common_sql_values(record, transaction_time)
            )
        }
        FactExtractKind::ModelBehavior => {
            let text = source_record_text(record);
            let behavior_kind = model_behavior_kind(record, &text);
            let (
                unsupported_claim,
                authority_overclaim,
                refusal,
                tool_call_proposed,
                filesystem_operation_proposed,
                review_required,
            ) = model_behavior_flags(record, &text);
            let model_ref = summary_token_value_or(
                &summary,
                "model_ref",
                &summary_token_value(&summary, "model"),
            );
            let provider_ref = summary_token_value_or(
                &summary,
                "provider_ref",
                &summary_token_value(&summary, "provider"),
            );
            let model_output_fallback =
                if record.record_kind == "effect_receipt" && summary.contains("model.output") {
                    source_record_receipt_id(record)
                } else {
                    record.record_id.clone()
                };
            let model_output_id =
                summary_token_value_or(&summary, "model_output_id", &model_output_fallback);
            format!(
                "INSERT INTO fact_model_behavior (fact_id, case_ref, subject_ref, model_ref, provider_ref, model_output_id, behavior_kind, unsupported_claim, authority_overclaim, refusal, tool_call_proposed, filesystem_operation_proposed, review_required, output_chars, asserted_by_event_ref, source_record_refs, source_graph_refs, evidence_refs, transaction_time, valid_time_start, valid_time_end, known_at, status, revision_of, superseded_by, retracted_by, confidence, authority_scope, source_record_id, source_record_kind, source_schema, fact_schema, created_at_unix_ms) VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {});",
                sql_quote(&fact_id),
                sql_quote(&record.case_ref),
                sql_quote(&subject_ref),
                sql_quote(&model_ref),
                sql_quote(&provider_ref),
                sql_quote(&model_output_id),
                sql_quote(behavior_kind),
                sql_bool(unsupported_claim),
                sql_bool(authority_overclaim),
                sql_bool(refusal),
                sql_bool(tool_call_proposed),
                sql_bool(filesystem_operation_proposed),
                sql_bool(review_required),
                summary_number(&summary, "output_chars"),
                fact_common_sql_values(record, transaction_time)
            )
        }
        FactExtractKind::PolicyOutcome => {
            let text = source_record_text(record);
            let policy_outcome = policy_outcome_kind(record, &text);
            let (requires_review, blocked, approved, denied, deferred, quarantined) =
                policy_outcome_flags(&text, policy_outcome);
            let policy_ref = summary_token_value_or(
                &summary,
                "policy_ref",
                &summary_token_value(&summary, "rule"),
            );
            let policy_kind = summary_token_value_or(
                &summary,
                "policy_kind",
                match record.record_kind.as_str() {
                    "policy_rule" => "policy_rule",
                    "authority_scope" => "authority_scope",
                    "projection_rule" => "projection_rule",
                    _ => "",
                },
            );
            let operation_kind = summary_token_value_or(
                &summary,
                "operation_kind",
                &summary_token_value(&summary, "op"),
            );
            let review_id = summary_token_value(&summary, "review_id");
            format!(
                "INSERT INTO fact_policy_outcome (fact_id, case_ref, subject_ref, policy_ref, policy_kind, operation_kind, decision_id, attempt_id, review_id, policy_outcome, requires_review, blocked, approved, denied, deferred, quarantined, asserted_by_event_ref, source_record_refs, source_graph_refs, evidence_refs, transaction_time, valid_time_start, valid_time_end, known_at, status, revision_of, superseded_by, retracted_by, confidence, authority_scope, source_record_id, source_record_kind, source_schema, fact_schema, created_at_unix_ms) VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {});",
                sql_quote(&fact_id),
                sql_quote(&record.case_ref),
                sql_quote(&subject_ref),
                sql_quote(&policy_ref),
                sql_quote(&policy_kind),
                sql_quote(&operation_kind),
                sql_quote(&source_record_decision_id(record)),
                sql_quote(&source_record_attempt_id(record)),
                sql_quote(&review_id),
                sql_quote(policy_outcome),
                sql_bool(requires_review),
                sql_bool(blocked),
                sql_bool(approved),
                sql_bool(denied),
                sql_bool(deferred),
                sql_bool(quarantined),
                fact_common_sql_values(record, transaction_time)
            )
        }
        FactExtractKind::MemoryQuality => {
            let memory_ref = summary_token_value_or(&summary, "memory_ref", &record.record_id);
            let memory_kind = summary_token_value_or(
                &summary,
                "memory_kind",
                &summary_token_value(&summary, "memory"),
            );
            let memory_scope = summary_token_value_or(
                &summary,
                "memory_scope",
                &summary_token_value(&summary, "scope"),
            );
            let quality_status = memory_quality_status(&summary);
            format!(
                "INSERT INTO fact_memory_quality (fact_id, case_ref, subject_ref, memory_ref, memory_kind, memory_scope, basis_record_count, basis_receipt_count, basis_edge_count, freshness, quality_status, requires_review, asserted_by_event_ref, source_record_refs, source_graph_refs, evidence_refs, transaction_time, valid_time_start, valid_time_end, known_at, status, revision_of, superseded_by, retracted_by, confidence, authority_scope, source_record_id, source_record_kind, source_schema, fact_schema, created_at_unix_ms) VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {});",
                sql_quote(&fact_id),
                sql_quote(&record.case_ref),
                sql_quote(&subject_ref),
                sql_quote(&memory_ref),
                sql_quote(&memory_kind),
                sql_quote(&memory_scope),
                summary_number(&summary, "basis_records"),
                summary_number(&summary, "basis_receipts"),
                summary_number(&summary, "basis_edges"),
                sql_quote(&summary_token_value_or(&summary, "freshness", "unknown")),
                sql_quote(&quality_status),
                sql_bool(summary_bool(&summary, "requires_review", false)),
                fact_common_sql_values(record, transaction_time)
            )
        }
    }
}

fn extract_fact_kind(
    store: &LmdbRecordStore,
    case_ref: &str,
    kind: FactExtractKind,
) -> Result<FactExtractionStats, String> {
    ensure_facts_ready()?;
    let records = store.list_records_by_case(case_ref, usize::MAX)?;
    let existing = existing_fact_ids(kind.table(), case_ref)?;
    let transaction_time = unix_time_ms_now();
    let mut stats = FactExtractionStats {
        records_scanned: records.records_total,
        ..Default::default()
    };
    let mut inserts = Vec::new();
    for record in records.records {
        if !source_record_matches_fact_kind(&record, kind) {
            stats.facts_skipped += 1;
            continue;
        }
        let fact_id = format!("{}{}", kind.fact_id_prefix(), record.record_id);
        if existing.contains(&fact_id) {
            stats.facts_duplicate += 1;
            continue;
        }
        inserts.push(fact_insert_sql(kind, &record, transaction_time));
        stats.facts_written += 1;
    }
    if !inserts.is_empty() {
        duckdb_exec_sql(&inserts.join("\n"))?;
    }
    Ok(stats)
}

pub(super) fn facts_extract(args: &[String]) -> Result<(), String> {
    let case_ref = named_arg(args, "--case")?;
    let kind_arg = named_arg(args, "--kind")?;
    let status = LmdbRecordStore::status(record_store_path());
    if status.status != RecordStoreStatusKind::Ready {
        print_non_ready_record_store(&status);
        return Ok(());
    }
    let store = LmdbRecordStore::open(&status.path)?;
    if kind_arg == "core" {
        let receipt = extract_fact_kind(&store, &case_ref, FactExtractKind::Receipt)?;
        let decision = extract_fact_kind(&store, &case_ref, FactExtractKind::Decision)?;
        let projection = extract_fact_kind(&store, &case_ref, FactExtractKind::Projection)?;
        println!("facts_extract:");
        println!("case_ref: {case_ref}");
        println!("kind: core");
        println!("status: completed");
        println!("fact_receipt_written: {}", receipt.facts_written);
        println!("fact_decision_written: {}", decision.facts_written);
        println!("fact_projection_written: {}", projection.facts_written);
        println!(
            "facts_duplicate: {}",
            receipt.facts_duplicate + decision.facts_duplicate + projection.facts_duplicate
        );
        println!("facts_are_truth: false");
        return Ok(());
    }
    if kind_arg == "behavior" {
        let model_behavior = extract_fact_kind(&store, &case_ref, FactExtractKind::ModelBehavior)?;
        let policy_outcome = extract_fact_kind(&store, &case_ref, FactExtractKind::PolicyOutcome)?;
        println!("facts_extract:");
        println!("case_ref: {case_ref}");
        println!("kind: behavior");
        println!("status: completed");
        println!(
            "fact_model_behavior_written: {}",
            model_behavior.facts_written
        );
        println!(
            "fact_policy_outcome_written: {}",
            policy_outcome.facts_written
        );
        println!(
            "facts_duplicate: {}",
            model_behavior.facts_duplicate + policy_outcome.facts_duplicate
        );
        println!("facts_are_truth: false");
        return Ok(());
    }
    if kind_arg == "operational" {
        let carrier_outcome =
            extract_fact_kind(&store, &case_ref, FactExtractKind::CarrierOutcome)?;
        let divergence = extract_fact_kind(&store, &case_ref, FactExtractKind::Divergence)?;
        let memory_quality = extract_fact_kind(&store, &case_ref, FactExtractKind::MemoryQuality)?;
        println!("facts_extract:");
        println!("case_ref: {case_ref}");
        println!("kind: operational");
        println!("status: completed");
        println!(
            "fact_carrier_outcome_written: {}",
            carrier_outcome.facts_written
        );
        println!("fact_divergence_written: {}", divergence.facts_written);
        println!(
            "fact_memory_quality_written: {}",
            memory_quality.facts_written
        );
        println!(
            "facts_duplicate: {}",
            carrier_outcome.facts_duplicate
                + divergence.facts_duplicate
                + memory_quality.facts_duplicate
        );
        println!("facts_are_truth: false");
        return Ok(());
    }
    if kind_arg == "all" {
        let receipt = extract_fact_kind(&store, &case_ref, FactExtractKind::Receipt)?;
        let decision = extract_fact_kind(&store, &case_ref, FactExtractKind::Decision)?;
        let projection = extract_fact_kind(&store, &case_ref, FactExtractKind::Projection)?;
        let model_behavior = extract_fact_kind(&store, &case_ref, FactExtractKind::ModelBehavior)?;
        let policy_outcome = extract_fact_kind(&store, &case_ref, FactExtractKind::PolicyOutcome)?;
        let carrier_outcome =
            extract_fact_kind(&store, &case_ref, FactExtractKind::CarrierOutcome)?;
        let divergence = extract_fact_kind(&store, &case_ref, FactExtractKind::Divergence)?;
        let memory_quality = extract_fact_kind(&store, &case_ref, FactExtractKind::MemoryQuality)?;
        println!("facts_extract:");
        println!("case_ref: {case_ref}");
        println!("kind: all");
        println!("status: completed");
        println!("fact_receipt_written: {}", receipt.facts_written);
        println!("fact_decision_written: {}", decision.facts_written);
        println!("fact_projection_written: {}", projection.facts_written);
        println!(
            "fact_model_behavior_written: {}",
            model_behavior.facts_written
        );
        println!(
            "fact_policy_outcome_written: {}",
            policy_outcome.facts_written
        );
        println!(
            "fact_carrier_outcome_written: {}",
            carrier_outcome.facts_written
        );
        println!("fact_divergence_written: {}", divergence.facts_written);
        println!(
            "fact_memory_quality_written: {}",
            memory_quality.facts_written
        );
        println!(
            "facts_duplicate: {}",
            receipt.facts_duplicate
                + decision.facts_duplicate
                + projection.facts_duplicate
                + model_behavior.facts_duplicate
                + policy_outcome.facts_duplicate
                + carrier_outcome.facts_duplicate
                + divergence.facts_duplicate
                + memory_quality.facts_duplicate
        );
        println!("facts_are_truth: false");
        return Ok(());
    }
    let kind = FactExtractKind::from_arg(&kind_arg)
        .ok_or_else(|| format!("unsupported facts extract kind: {kind_arg}"))?;
    let stats = extract_fact_kind(&store, &case_ref, kind)?;
    println!("facts_extract:");
    println!("case_ref: {case_ref}");
    println!("kind: {}", kind.output_kind());
    println!("status: completed");
    println!("records_scanned: {}", stats.records_scanned);
    println!("facts_written: {}", stats.facts_written);
    println!("facts_duplicate: {}", stats.facts_duplicate);
    println!("facts_skipped: {}", stats.facts_skipped);
    if kind == FactExtractKind::Divergence && stats.facts_written == 0 && stats.facts_duplicate == 0
    {
        println!("no_divergence_records: true");
    }
    println!("table: {}", kind.table());
    println!("schema: {FACT_SCHEMA}");
    println!("facts_are_truth: false");
    if kind == FactExtractKind::MemoryQuality {
        println!("memory_is_truth: false");
    }
    Ok(())
}

pub(super) fn facts_summary(args: &[String]) -> Result<(), String> {
    let case_ref = named_arg(args, "--case")?;
    let counts = fact_counts(Some(&case_ref))?;
    println!("facts_summary:");
    println!("case_ref: {case_ref}");
    println!("fact_receipt: {}", counts.receipt);
    println!("fact_decision: {}", counts.decision);
    println!("fact_projection: {}", counts.projection);
    println!("fact_carrier_outcome: {}", counts.carrier_outcome);
    println!("fact_divergence: {}", counts.divergence);
    println!("fact_model_behavior: {}", counts.model_behavior);
    println!("fact_policy_outcome: {}", counts.policy_outcome);
    println!("fact_memory_quality: {}", counts.memory_quality);
    println!("facts_total: {}", counts.total);
    println!("facts_are_truth: false");
    println!("memory_is_truth: false");
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FactReportSection {
    Receipts,
    Decisions,
    Projections,
    Policy,
    Carriers,
    Divergence,
    Memory,
    Model,
}

impl FactReportSection {
    fn from_arg(value: &str) -> Option<Self> {
        match value {
            "receipts" => Some(Self::Receipts),
            "decisions" => Some(Self::Decisions),
            "projections" => Some(Self::Projections),
            "policy" => Some(Self::Policy),
            "carriers" => Some(Self::Carriers),
            "divergence" => Some(Self::Divergence),
            "memory" => Some(Self::Memory),
            "model" => Some(Self::Model),
            _ => None,
        }
    }
}

fn fact_report_where(case_ref: &str) -> String {
    format!("case_ref = {}", sql_quote(case_ref))
}

fn fact_report_total(table: &str, case_ref: &str) -> Result<usize, String> {
    duckdb_count(&format!(
        "SELECT count(*) FROM {table} WHERE {};",
        fact_report_where(case_ref)
    ))
}

fn fact_report_count(table: &str, case_ref: &str, predicate: &str) -> Result<usize, String> {
    duckdb_count(&format!(
        "SELECT count(*) FROM {table} WHERE {} AND {predicate};",
        fact_report_where(case_ref)
    ))
}

fn fact_report_group(
    table: &str,
    case_ref: &str,
    column: &str,
) -> Result<Vec<(String, usize)>, String> {
    duckdb_group_counts(&format!(
        "SELECT coalesce(nullif({column}, ''), 'unknown') AS value, count(*) FROM {table} WHERE {} GROUP BY value ORDER BY value;",
        fact_report_where(case_ref)
    ))
}

fn print_group_lines(table: &str, case_ref: &str, column: &str) -> Result<(), String> {
    for (key, count) in fact_report_group(table, case_ref, column)? {
        println!("    {key}: {count}");
    }
    Ok(())
}

fn print_bool_line(
    table: &str,
    case_ref: &str,
    output_key: &str,
    column: &str,
) -> Result<(), String> {
    println!(
        "    {output_key}: {}",
        fact_report_count(table, case_ref, &format!("{column} = TRUE"))?
    );
    Ok(())
}

fn print_receipts_report(case_ref: &str) -> Result<(), String> {
    println!("  receipts:");
    println!(
        "    total: {}",
        fact_report_total("fact_receipt", case_ref)?
    );
    print_group_lines("fact_receipt", case_ref, "receipt_status")?;
    print_group_lines("fact_receipt", case_ref, "carrier_family")?;
    print_bool_line(
        "fact_receipt",
        case_ref,
        "execution_performed",
        "execution_performed",
    )?;
    print_bool_line(
        "fact_receipt",
        case_ref,
        "carrier_attempted",
        "carrier_attempted",
    )?;
    Ok(())
}

fn print_decisions_report(case_ref: &str) -> Result<(), String> {
    println!("  decisions:");
    println!(
        "    total: {}",
        fact_report_total("fact_decision", case_ref)?
    );
    print_group_lines("fact_decision", case_ref, "decision_outcome")?;
    print_bool_line(
        "fact_decision",
        case_ref,
        "requires_review",
        "requires_review",
    )?;
    println!(
        "    review_id: {}",
        fact_report_count("fact_decision", case_ref, "review_id <> ''")?
    );
    Ok(())
}

fn print_projections_report(case_ref: &str) -> Result<(), String> {
    println!("  projections:");
    println!(
        "    total: {}",
        fact_report_total("fact_projection", case_ref)?
    );
    print_group_lines("fact_projection", case_ref, "consumer")?;
    print_group_lines("fact_projection", case_ref, "projection_kind")?;
    print_group_lines("fact_projection", case_ref, "freshness")?;
    print_group_lines("fact_projection", case_ref, "redaction")?;
    println!(
        "    model_visible: {}",
        fact_report_count("fact_projection", case_ref, "consumer = 'model'")?
    );
    println!(
        "    operator_visible: {}",
        fact_report_count("fact_projection", case_ref, "consumer = 'operator'")?
    );
    Ok(())
}

fn print_policy_report(case_ref: &str) -> Result<(), String> {
    println!("  policy:");
    println!(
        "    total: {}",
        fact_report_total("fact_policy_outcome", case_ref)?
    );
    print_group_lines("fact_policy_outcome", case_ref, "policy_outcome")?;
    print_bool_line(
        "fact_policy_outcome",
        case_ref,
        "review_required",
        "requires_review",
    )?;
    print_bool_line("fact_policy_outcome", case_ref, "approved", "approved")?;
    print_bool_line("fact_policy_outcome", case_ref, "denied", "denied")?;
    print_bool_line("fact_policy_outcome", case_ref, "deferred", "deferred")?;
    print_bool_line(
        "fact_policy_outcome",
        case_ref,
        "quarantined",
        "quarantined",
    )?;
    Ok(())
}

fn print_carriers_report(case_ref: &str) -> Result<(), String> {
    println!("  carriers:");
    println!(
        "    total: {}",
        fact_report_total("fact_carrier_outcome", case_ref)?
    );
    print_group_lines("fact_carrier_outcome", case_ref, "carrier_family")?;
    print_group_lines("fact_carrier_outcome", case_ref, "effective_outcome")?;
    print_bool_line(
        "fact_carrier_outcome",
        case_ref,
        "carrier_attempted",
        "carrier_attempted",
    )?;
    print_bool_line(
        "fact_carrier_outcome",
        case_ref,
        "execution_performed",
        "execution_performed",
    )?;
    print_bool_line(
        "fact_carrier_outcome",
        case_ref,
        "receipt_required",
        "receipt_required",
    )?;
    Ok(())
}

fn print_divergence_report(case_ref: &str) -> Result<(), String> {
    println!("  divergence:");
    let total = fact_report_total("fact_divergence", case_ref)?;
    println!("    total: {total}");
    if total == 0 {
        println!("    status: none_observed");
        return Ok(());
    }
    print_group_lines("fact_divergence", case_ref, "divergence_kind")?;
    print_group_lines("fact_divergence", case_ref, "severity")?;
    Ok(())
}

fn print_memory_report(case_ref: &str) -> Result<(), String> {
    println!("  memory:");
    println!(
        "    total: {}",
        fact_report_total("fact_memory_quality", case_ref)?
    );
    println!("    memory_is_truth: false");
    println!(
        "    candidates: {}",
        fact_report_count(
            "fact_memory_quality",
            case_ref,
            "source_record_kind = 'memory_candidate'"
        )?
    );
    print_group_lines("fact_memory_quality", case_ref, "memory_kind")?;
    print_group_lines("fact_memory_quality", case_ref, "memory_scope")?;
    println!(
        "    basis_record_count: {}",
        duckdb_count(&format!(
            "SELECT coalesce(sum(basis_record_count), 0) FROM fact_memory_quality WHERE {};",
            fact_report_where(case_ref)
        ))?
    );
    println!(
        "    basis_receipt_count: {}",
        duckdb_count(&format!(
            "SELECT coalesce(sum(basis_receipt_count), 0) FROM fact_memory_quality WHERE {};",
            fact_report_where(case_ref)
        ))?
    );
    println!(
        "    basis_edge_count: {}",
        duckdb_count(&format!(
            "SELECT coalesce(sum(basis_edge_count), 0) FROM fact_memory_quality WHERE {};",
            fact_report_where(case_ref)
        ))?
    );
    print_bool_line(
        "fact_memory_quality",
        case_ref,
        "requires_review",
        "requires_review",
    )?;
    Ok(())
}

fn print_model_report(case_ref: &str) -> Result<(), String> {
    println!("  model:");
    let total = fact_report_total("fact_model_behavior", case_ref)?;
    println!("    total: {total}");
    if total == 0 {
        println!("    status: no_model_records");
        return Ok(());
    }
    print_group_lines("fact_model_behavior", case_ref, "behavior_kind")?;
    print_bool_line(
        "fact_model_behavior",
        case_ref,
        "authority_overclaim",
        "authority_overclaim",
    )?;
    print_bool_line(
        "fact_model_behavior",
        case_ref,
        "unsupported_claim",
        "unsupported_claim",
    )?;
    print_bool_line(
        "fact_model_behavior",
        case_ref,
        "filesystem_operation_proposed",
        "filesystem_operation_proposed",
    )?;
    print_bool_line(
        "fact_model_behavior",
        case_ref,
        "review_required",
        "review_required",
    )?;
    Ok(())
}

fn print_fact_report_section(section: FactReportSection, case_ref: &str) -> Result<(), String> {
    match section {
        FactReportSection::Receipts => print_receipts_report(case_ref),
        FactReportSection::Decisions => print_decisions_report(case_ref),
        FactReportSection::Projections => print_projections_report(case_ref),
        FactReportSection::Policy => print_policy_report(case_ref),
        FactReportSection::Carriers => print_carriers_report(case_ref),
        FactReportSection::Divergence => print_divergence_report(case_ref),
        FactReportSection::Memory => print_memory_report(case_ref),
        FactReportSection::Model => print_model_report(case_ref),
    }
}

pub(super) fn facts_report(args: &[String]) -> Result<(), String> {
    let case_ref = named_arg(args, "--case")?;
    let format = optional_arg(args, "--format").unwrap_or_else(|| "plain".to_string());
    if format != "plain" {
        return Err("unsupported facts report format: use --format plain".to_string());
    }
    ensure_facts_ready()?;
    let sections = if let Some(section) = optional_arg(args, "--section") {
        vec![FactReportSection::from_arg(&section)
            .ok_or_else(|| format!("unsupported facts report section: {section}"))?]
    } else {
        vec![
            FactReportSection::Receipts,
            FactReportSection::Decisions,
            FactReportSection::Projections,
            FactReportSection::Policy,
            FactReportSection::Carriers,
            FactReportSection::Divergence,
            FactReportSection::Memory,
            FactReportSection::Model,
        ]
    };
    println!("facts_report:");
    println!("case_ref: {case_ref}");
    println!("schema: {FACT_SCHEMA}");
    println!("facts_are_truth: false");
    println!();
    println!("sections:");
    for section in sections {
        print_fact_report_section(section, &case_ref)?;
    }
    Ok(())
}
