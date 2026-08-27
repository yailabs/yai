//! Journal inspection, replay, metadata, and diagnostic reporting boundary.

use super::*;

pub(super) fn store_tail(args: &[String]) -> Result<(), String> {
    let path = journal_arg(args)?;
    let contents = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let mut count = 0usize;
    for line in contents.lines() {
        println!("{line}");
        count += 1;
    }
    println!("records: {count}");
    Ok(())
}

pub(super) fn journal_inspect(args: &[String]) -> Result<(), String> {
    let path = PathBuf::from(named_arg(args, "--path")?);
    let show_errors = args.iter().any(|arg| arg == "--show-errors");
    println!("journal_path: {}", path.display());
    println!("parser_policy: diagnostic");
    println!("lmdb_write: no");
    if !path.exists() {
        println!("journal_status: missing");
        println!("lines_total: 0");
        println!("valid_entries: 0");
        println!("invalid_entries: 0");
        println!("unsupported_entries: 0");
        println!("duplicate_entries: 0");
        println!("replay_ready: no");
        return Ok(());
    }
    if !path.is_file() {
        println!("journal_status: unavailable");
        println!("lines_total: 0");
        println!("valid_entries: 0");
        println!("invalid_entries: 0");
        println!("unsupported_entries: 0");
        println!("duplicate_entries: 0");
        println!("replay_ready: no");
        return Ok(());
    }

    let inspection = Journal::inspect_jsonl(&path)
        .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
    println!("journal_status: readable");
    println!("lines_total: {}", inspection.lines_total);
    println!("valid_entries: {}", inspection.valid_entries);
    println!("invalid_entries: {}", inspection.invalid_entries);
    println!("unsupported_entries: {}", inspection.unsupported_entries);
    println!("duplicate_entries: {}", inspection.duplicate_entries);
    println!("replay_ready: {}", bool_word(inspection.replay_ready()));
    if show_errors {
        for diagnostic in inspection.diagnostics {
            println!("line: {}", diagnostic.line_number);
            println!("entry_status: {}", diagnostic.entry_status.as_str());
            println!("record_id: {}", diagnostic.record_id);
            println!("record_kind: {}", diagnostic.record_kind);
            println!("schema: {}", diagnostic.schema);
            println!("error_code: {}", diagnostic.error_code);
            println!("error_message: {}", diagnostic.error_message);
            println!("action: {}", diagnostic.action);
        }
    }
    Ok(())
}

fn print_legacy_corpus_report(
    path: &Path,
    report: &yai_core_engine::compatibility::LegacyCorpusReport,
) {
    println!("legacy_compatibility:");
    println!("source_path: {}", path.display());
    println!("input_authority: compatibility_only");
    println!("lines_total: {}", report.lines_total);
    println!("losslessly_promoted: {}", report.losslessly_promoted);
    println!(
        "promoted_with_compatibility_metadata: {}",
        report.promoted_with_metadata
    );
    println!("preserved_opaque: {}", report.preserved_opaque);
    println!("rejected_malformed: {}", report.rejected_malformed);
    println!("repeated_record_ids: {}", report.repeated_record_ids);
    println!("canonical_transitions_written: 0");
}

pub(super) fn journal_compatibility_inspect(args: &[String]) -> Result<(), String> {
    let path = PathBuf::from(named_arg(args, "--path")?);
    let contents = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let report = inspect_legacy_jsonl(&contents);
    print_legacy_corpus_report(&path, &report);
    println!("mode: inspect");
    Ok(())
}

pub(super) fn journal_compatibility_import(args: &[String]) -> Result<(), String> {
    let path = PathBuf::from(named_arg(args, "--path")?);
    let target = PathBuf::from(named_arg(args, "--target")?);
    let dry_run = args.iter().any(|argument| argument == "--dry-run");
    if target == record_store_path() {
        return Err("compatibility import target must be isolated from the live store".to_string());
    }
    let contents = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let inspection = inspect_legacy_jsonl(&contents);
    print_legacy_corpus_report(&path, &inspection);
    println!("target_path: {}", target.display());
    if dry_run {
        println!("mode: dry_run");
        println!("target_written: no");
        return Ok(());
    }
    let store = LmdbRecordStore::open(&target)?;
    let report = store.import_legacy_compatibility(&contents, &path.display().to_string())?;
    let stored_total = store.legacy_compatibility_payload_count()?;
    println!("mode: import");
    println!("target_written: yes");
    println!("payloads_written: {}", report.payloads_written);
    println!("payloads_duplicate: {}", report.payloads_duplicate);
    println!("payloads_stored_total: {stored_total}");
    println!(
        "validation: {}",
        if stored_total >= report.payloads_written {
            "passed"
        } else {
            "failed"
        }
    );
    Ok(())
}

pub(super) fn journal_replay(args: &[String]) -> Result<(), String> {
    let path = PathBuf::from(named_arg(args, "--path")?);
    let dry_run = args.iter().any(|arg| arg == "--dry-run");
    let lmdb_path = record_store_path();
    if !path.exists() {
        let profile = replay_profile_for_missing(&path);
        let started_at = unix_timestamp_string();
        let completed_at = unix_timestamp_string();
        let report_path = write_replay_report(&ReplayReport {
            report_id: replay_report_id(&profile.journal_identity),
            journal_identity: profile.journal_identity.clone(),
            journal_path: path.display().to_string(),
            lmdb_path: lmdb_path.display().to_string(),
            record_schema: profile.record_schema.clone(),
            journal_schema: profile.journal_schema.clone(),
            compatibility: profile.compatibility.clone(),
            started_at,
            completed_at,
            lines_total: 0,
            valid_entries: 0,
            invalid_entries: 0,
            unsupported_entries: 0,
            duplicate_entries: 0,
            records_seen: 0,
            records_written: 0,
            records_duplicate: 0,
            records_skipped: 0,
            cursor_line: 0,
            replay_status: "failed".to_string(),
            replay_ready: false,
            idempotent: false,
            errors: vec![ReplayReportIssue::new(
                0,
                "missing_journal",
                "journal path is missing",
            )],
            warnings: Vec::new(),
            summary: "Replay failed before materialization because the journal path is missing."
                .to_string(),
        })?;
        println!("journal_replay: failed");
        println!("journal_path: {}", path.display());
        println!("journal_identity: {}", profile.journal_identity);
        println!("lmdb_path: {}", lmdb_path.display());
        println!("record_schema: {}", profile.record_schema);
        println!("journal_schema: {}", profile.journal_schema);
        println!("compatibility: {}", profile.compatibility);
        println!("lines_total: 0");
        println!("valid_entries: 0");
        println!("invalid_entries: 0");
        println!("unsupported_entries: 0");
        println!("duplicate_entries: 0");
        println!("records_seen: 0");
        println!("records_written: 0");
        println!("records_duplicate: 0");
        println!("records_skipped: 0");
        println!("cursor_line: 0");
        println!("replay_status: failed");
        println!("replay_ready: no");
        println!("replay_report: {}", report_path.display());
        println!("reason: missing_journal");
        return Ok(());
    }
    if !path.is_file() {
        let profile = replay_profile_for_missing(&path);
        let started_at = unix_timestamp_string();
        let completed_at = unix_timestamp_string();
        let report_path = write_replay_report(&ReplayReport {
            report_id: replay_report_id(&profile.journal_identity),
            journal_identity: profile.journal_identity.clone(),
            journal_path: path.display().to_string(),
            lmdb_path: lmdb_path.display().to_string(),
            record_schema: profile.record_schema.clone(),
            journal_schema: profile.journal_schema.clone(),
            compatibility: profile.compatibility.clone(),
            started_at,
            completed_at,
            lines_total: 0,
            valid_entries: 0,
            invalid_entries: 0,
            unsupported_entries: 0,
            duplicate_entries: 0,
            records_seen: 0,
            records_written: 0,
            records_duplicate: 0,
            records_skipped: 0,
            cursor_line: 0,
            replay_status: "failed".to_string(),
            replay_ready: false,
            idempotent: false,
            errors: vec![ReplayReportIssue::new(
                0,
                "journal_unavailable",
                "journal path is not a regular file",
            )],
            warnings: Vec::new(),
            summary:
                "Replay failed before materialization because the journal path is unavailable."
                    .to_string(),
        })?;
        println!("journal_replay: failed");
        println!("journal_path: {}", path.display());
        println!("journal_identity: {}", profile.journal_identity);
        println!("lmdb_path: {}", lmdb_path.display());
        println!("record_schema: {}", profile.record_schema);
        println!("journal_schema: {}", profile.journal_schema);
        println!("compatibility: {}", profile.compatibility);
        println!("lines_total: 0");
        println!("valid_entries: 0");
        println!("invalid_entries: 0");
        println!("unsupported_entries: 0");
        println!("duplicate_entries: 0");
        println!("records_seen: 0");
        println!("records_written: 0");
        println!("records_duplicate: 0");
        println!("records_skipped: 0");
        println!("cursor_line: 0");
        println!("replay_status: failed");
        println!("replay_ready: no");
        println!("replay_report: {}", report_path.display());
        println!("reason: journal_unavailable");
        return Ok(());
    }

    let inspection = Journal::inspect_jsonl(&path)
        .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
    let contents = fs::read_to_string(&path).map_err(|error| {
        format!(
            "failed to read {} for replay identity: {error}",
            path.display()
        )
    })?;
    let profile = replay_profile_for_inspection(&path, &contents, &inspection);
    if !inspection.replay_ready() {
        let metadata = replay_metadata_from_failure(&path, &profile, &inspection);
        persist_replay_metadata(&lmdb_path, metadata.clone())?;
        let report_path = write_replay_report(&replay_report_from_metadata(
            &metadata,
            &inspection,
            inspection.valid_entries,
            false,
            replay_report_issues_from_inspection(&inspection),
            Vec::new(),
            replay_summary(&metadata.status, 0, 0, inspection.invalid_entries),
        ))?;
        println!("journal_replay: failed");
        println!("journal_path: {}", path.display());
        println!("journal_identity: {}", profile.journal_identity);
        println!("lmdb_path: {}", lmdb_path.display());
        println!("record_schema: {}", profile.record_schema);
        println!("journal_schema: {}", profile.journal_schema);
        println!("supported_schema: {}", RECORD_SCHEMA);
        println!("compatibility: {}", profile.compatibility);
        println!("lines_total: {}", inspection.lines_total);
        println!("valid_entries: {}", inspection.valid_entries);
        println!("invalid_entries: {}", inspection.invalid_entries);
        println!("unsupported_entries: {}", inspection.unsupported_entries);
        println!("duplicate_entries: {}", inspection.duplicate_entries);
        println!("records_seen: {}", inspection.valid_entries);
        println!("records_written: 0");
        println!("records_duplicate: 0");
        println!("records_skipped: {}", inspection.lines_total);
        println!("cursor_line: 0");
        println!(
            "replay_status: {}",
            if inspection.invalid_entries == 0 && profile.compatibility == "schema_mismatch" {
                "incompatible"
            } else {
                "failed"
            }
        );
        println!("replay_ready: no");
        println!("replay_report: {}", report_path.display());
        println!("reason: {}", replay_failure_reason(&inspection));
        return Ok(());
    }

    if dry_run {
        let cursor_line = current_replay_metadata(&lmdb_path, &profile.journal_identity)?
            .map(|metadata| metadata.cursor_line)
            .unwrap_or(0);
        println!("journal_replay: dry_run");
        println!("journal_path: {}", path.display());
        println!("journal_identity: {}", profile.journal_identity);
        println!("lmdb_path: {}", lmdb_path.display());
        println!("record_schema: {}", profile.record_schema);
        println!("journal_schema: {}", profile.journal_schema);
        println!("compatibility: {}", profile.compatibility);
        println!("lines_total: {}", inspection.lines_total);
        println!("valid_entries: {}", inspection.valid_entries);
        println!("invalid_entries: {}", inspection.invalid_entries);
        println!("unsupported_entries: {}", inspection.unsupported_entries);
        println!("duplicate_entries: {}", inspection.duplicate_entries);
        println!("records_to_write: {}", inspection.valid_entries);
        println!("cursor_line: {cursor_line}");
        println!("would_update_cursor: yes");
        println!("would_write_lmdb: yes");
        println!("replay_ready: yes");
        return Ok(());
    }

    let journal = Journal::load_jsonl(&path)
        .map_err(|error| format!("journal replay failed to load journal: {error}"))?;
    let store = LmdbRecordStore::open(&lmdb_path)
        .map_err(|error| format!("journal replay failed to open LMDB: {error}"))?;
    let started_at = unix_timestamp_string();
    store.put_replay_metadata(&replay_metadata_in_progress(
        &path,
        &profile,
        &inspection,
        &started_at,
    ))?;
    let report = store.import_journal_with_report(&journal, &path.display().to_string())?;
    let metadata = replay_metadata_from_report(
        &path,
        &profile,
        &inspection,
        &report,
        &started_at,
        &unix_timestamp_string(),
    );
    store.put_replay_metadata(&metadata)?;
    let idempotent = report.records_seen > 0
        && report.records_written == 0
        && report.records_duplicate == report.records_seen;
    let report_path = write_replay_report(&replay_report_from_metadata(
        &metadata,
        &inspection,
        report.records_seen,
        idempotent,
        Vec::new(),
        Vec::new(),
        replay_summary(
            &metadata.status,
            report.records_written,
            report.records_duplicate,
            inspection.invalid_entries,
        ),
    ))?;
    let status = LmdbRecordStore::status(&lmdb_path);
    println!("journal_replay: completed");
    println!("journal_path: {}", path.display());
    println!("journal_identity: {}", profile.journal_identity);
    println!("lmdb_path: {}", lmdb_path.display());
    println!("record_schema: {}", profile.record_schema);
    println!("journal_schema: {}", profile.journal_schema);
    println!("compatibility: {}", profile.compatibility);
    println!("lines_total: {}", inspection.lines_total);
    println!("lines_replayed: {}", metadata.lines_replayed);
    println!("valid_entries: {}", inspection.valid_entries);
    println!("invalid_entries: {}", inspection.invalid_entries);
    println!("unsupported_entries: {}", inspection.unsupported_entries);
    println!("duplicate_entries: {}", inspection.duplicate_entries);
    println!("records_seen: {}", report.records_seen);
    println!("records_written: {}", report.records_written);
    println!("records_duplicate: {}", report.records_duplicate);
    println!("records_skipped: {}", report.records_skipped);
    println!("cursor_line: {}", metadata.cursor_line);
    println!("replay_status: {}", metadata.status);
    println!("replay_ready: yes");
    println!("replay_report: {}", report_path.display());
    println!("record_store_status: {}", status.status.as_str());
    println!("idempotent: {}", bool_word(idempotent));
    Ok(())
}

pub(super) fn journal_replay_status(args: &[String]) -> Result<(), String> {
    let path = PathBuf::from(named_arg(args, "--path")?);
    let lmdb_path = record_store_path();
    if !path.exists() || !path.is_file() {
        let profile = replay_profile_for_missing(&path);
        println!("journal_replay_status:");
        println!("journal_path: {}", path.display());
        println!("journal_identity: {}", profile.journal_identity);
        println!("lmdb_path: {}", lmdb_path.display());
        println!("record_schema: {}", profile.record_schema);
        println!("journal_schema: {}", profile.journal_schema);
        println!("cursor_line: 0");
        println!("replay_status: not_started");
        println!("records_written: 0");
        println!("records_duplicate: 0");
        println!("compatibility: {}", profile.compatibility);
        return Ok(());
    }

    let inspection = Journal::inspect_jsonl(&path)
        .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
    let contents = fs::read_to_string(&path).map_err(|error| {
        format!(
            "failed to read {} for replay identity: {error}",
            path.display()
        )
    })?;
    let profile = replay_profile_for_inspection(&path, &contents, &inspection);
    let metadata = current_replay_metadata(&lmdb_path, &profile.journal_identity)?;
    println!("journal_replay_status:");
    println!("journal_path: {}", path.display());
    println!("journal_identity: {}", profile.journal_identity);
    println!("lmdb_path: {}", lmdb_path.display());
    match metadata {
        Some(metadata) => {
            println!("record_schema: {}", metadata.record_schema);
            println!("journal_schema: {}", metadata.journal_schema);
            println!("cursor_line: {}", metadata.cursor_line);
            println!("replay_status: {}", metadata.status);
            println!("lines_total: {}", metadata.lines_total);
            println!("lines_replayed: {}", metadata.lines_replayed);
            println!("records_written: {}", metadata.records_written);
            println!("records_duplicate: {}", metadata.records_duplicate);
            println!("records_skipped: {}", metadata.records_skipped);
            println!("invalid_entries: {}", metadata.invalid_entries);
            println!("unsupported_entries: {}", metadata.unsupported_entries);
            println!("compatibility: {}", metadata.compatibility);
        }
        None => {
            println!("record_schema: {}", profile.record_schema);
            println!("journal_schema: {}", profile.journal_schema);
            println!("cursor_line: 0");
            println!("replay_status: not_started");
            println!("lines_total: {}", inspection.lines_total);
            println!("lines_replayed: 0");
            println!("records_written: 0");
            println!("records_duplicate: 0");
            println!("records_skipped: 0");
            println!("invalid_entries: {}", inspection.invalid_entries);
            println!("unsupported_entries: {}", inspection.unsupported_entries);
            println!("compatibility: {}", profile.compatibility);
        }
    }
    Ok(())
}

pub(super) fn journal_replay_report(args: &[String]) -> Result<(), String> {
    let path = PathBuf::from(named_arg(args, "--path")?);
    let (profile, inspection) = replay_profile_and_inspection_for_report(&path)?;
    let report_path = replay_report_path(&profile.journal_identity);
    if !report_path.is_file() {
        println!("replay_report_schema: yai.replay_report.v1");
        println!("replay_report: not_found");
        println!("journal_identity: {}", profile.journal_identity);
        println!("journal_path: {}", path.display());
        println!("lmdb_path: {}", record_store_path().display());
        println!("compatibility: {}", profile.compatibility);
        println!("replay_status: not_started");
        return Ok(());
    }

    let report = fs::read_to_string(&report_path)
        .map_err(|error| format!("failed to read {}: {error}", report_path.display()))?;
    println!(
        "replay_report_schema: {}",
        json_string_or(&report, "report_schema", "yai.replay_report.v1")
    );
    println!("replay_report: {}", report_path.display());
    println!(
        "report_id: {}",
        json_string_or(&report, "report_id", "unknown")
    );
    println!(
        "journal_identity: {}",
        json_string_or(&report, "journal_identity", &profile.journal_identity)
    );
    println!(
        "journal_path: {}",
        json_string_or(&report, "journal_path", &path.display().to_string())
    );
    println!(
        "lmdb_path: {}",
        json_string_or(
            &report,
            "lmdb_path",
            &record_store_path().display().to_string()
        )
    );
    println!(
        "record_schema: {}",
        json_string_or(&report, "record_schema", RECORD_SCHEMA)
    );
    println!(
        "journal_schema: {}",
        json_string_or(&report, "journal_schema", &profile.journal_schema)
    );
    println!(
        "compatibility: {}",
        json_string_or(&report, "compatibility", &profile.compatibility)
    );
    println!(
        "replay_status: {}",
        json_string_or(&report, "replay_status", "unknown")
    );
    print_report_number(&report, "lines_total", inspection.lines_total);
    print_report_number(&report, "valid_entries", inspection.valid_entries);
    print_report_number(&report, "invalid_entries", inspection.invalid_entries);
    print_report_number(
        &report,
        "unsupported_entries",
        inspection.unsupported_entries,
    );
    print_report_number(&report, "duplicate_entries", inspection.duplicate_entries);
    print_report_number(&report, "records_seen", 0);
    print_report_number(&report, "records_written", 0);
    print_report_number(&report, "records_duplicate", 0);
    print_report_number(&report, "records_skipped", 0);
    print_report_number(&report, "cursor_line", 0);
    println!(
        "replay_ready: {}",
        json_string_or(&report, "replay_ready", "no")
    );
    println!(
        "idempotent: {}",
        json_string_or(&report, "idempotent", "no")
    );
    println!(
        "summary: {}",
        json_string_or(&report, "summary", "Replay report summary unavailable.")
    );
    if !inspection.replay_ready() && !inspection.diagnostics.is_empty() {
        println!("errors:");
        for diagnostic in inspection.diagnostics {
            println!("- line: {}", diagnostic.line_number);
            println!("  status: {}", diagnostic.entry_status.as_str());
            println!("  error_code: {}", diagnostic.error_code);
        }
    }
    Ok(())
}

pub(super) fn replay_failure_reason(inspection: &JournalInspection) -> String {
    if inspection.invalid_entries > 0 {
        return "invalid_json".to_string();
    }
    inspection
        .diagnostics
        .first()
        .map(|diagnostic| diagnostic.error_code.clone())
        .unwrap_or_else(|| "not_replay_ready".to_string())
}

#[derive(Clone, Debug)]
pub(super) struct ReplayProfile {
    pub(super) journal_identity: String,
    record_schema: String,
    journal_schema: String,
    compatibility: String,
}

pub(super) fn replay_profile_for_missing(path: &std::path::Path) -> ReplayProfile {
    ReplayProfile {
        journal_identity: journal_identity(path, ""),
        record_schema: RECORD_SCHEMA.to_string(),
        journal_schema: "unknown".to_string(),
        compatibility: "unknown".to_string(),
    }
}

pub(super) fn replay_profile_for_inspection(
    path: &std::path::Path,
    contents: &str,
    inspection: &JournalInspection,
) -> ReplayProfile {
    let mut observed_schema = if inspection.valid_entries > 0 {
        JOURNAL_RECORD_SCHEMA.to_string()
    } else {
        "unknown".to_string()
    };
    let mut compatibility = if inspection.valid_entries > 0 {
        "compatible".to_string()
    } else {
        "unknown".to_string()
    };

    for diagnostic in &inspection.diagnostics {
        if diagnostic.error_code == "invalid_schema" || diagnostic.error_code == "missing_record_id"
        {
            observed_schema = diagnostic.schema.clone();
            if observed_schema == "none" {
                compatibility = "unknown".to_string();
            } else {
                compatibility = "schema_mismatch".to_string();
            }
            break;
        }
    }

    let record_schema = if compatibility == "schema_mismatch" {
        observed_schema.clone()
    } else {
        RECORD_SCHEMA.to_string()
    };

    ReplayProfile {
        journal_identity: journal_identity(path, contents),
        record_schema,
        journal_schema: observed_schema,
        compatibility,
    }
}

fn current_replay_metadata(
    lmdb_path: &std::path::Path,
    journal_identity: &str,
) -> Result<Option<ReplayMetadata>, String> {
    if LmdbRecordStore::status(lmdb_path).status != RecordStoreStatusKind::Ready {
        return Ok(None);
    }
    let store = LmdbRecordStore::open(lmdb_path)
        .map_err(|error| format!("failed to open LMDB for replay status: {error}"))?;
    store.replay_metadata(journal_identity)
}

fn persist_replay_metadata(
    lmdb_path: &std::path::Path,
    metadata: ReplayMetadata,
) -> Result<(), String> {
    let store = LmdbRecordStore::open(lmdb_path)
        .map_err(|error| format!("failed to open LMDB for replay metadata: {error}"))?;
    store.put_replay_metadata(&metadata)
}

pub(super) fn replay_metadata_in_progress(
    path: &std::path::Path,
    profile: &ReplayProfile,
    inspection: &JournalInspection,
    started_at: &str,
) -> ReplayMetadata {
    ReplayMetadata {
        replay_id: format!("replay:{}", profile.journal_identity),
        journal_identity: profile.journal_identity.clone(),
        journal_path: path.display().to_string(),
        record_schema: profile.record_schema.clone(),
        journal_schema: profile.journal_schema.clone(),
        started_at: started_at.to_string(),
        completed_at: "none".to_string(),
        lines_total: inspection.lines_total,
        lines_replayed: 0,
        records_written: 0,
        records_duplicate: 0,
        records_skipped: 0,
        invalid_entries: inspection.invalid_entries,
        unsupported_entries: inspection.unsupported_entries,
        cursor_line: 0,
        status: "in_progress".to_string(),
        compatibility: profile.compatibility.clone(),
    }
}

pub(super) fn replay_metadata_from_report(
    path: &std::path::Path,
    profile: &ReplayProfile,
    inspection: &JournalInspection,
    report: &yai_core_engine::store::lmdb::JournalImportReport,
    started_at: &str,
    completed_at: &str,
) -> ReplayMetadata {
    ReplayMetadata {
        replay_id: format!("replay:{}", profile.journal_identity),
        journal_identity: profile.journal_identity.clone(),
        journal_path: path.display().to_string(),
        record_schema: profile.record_schema.clone(),
        journal_schema: profile.journal_schema.clone(),
        started_at: started_at.to_string(),
        completed_at: completed_at.to_string(),
        lines_total: inspection.lines_total,
        lines_replayed: inspection.valid_entries,
        records_written: report.records_written,
        records_duplicate: report.records_duplicate,
        records_skipped: report.records_skipped,
        invalid_entries: inspection.invalid_entries,
        unsupported_entries: inspection.unsupported_entries,
        cursor_line: inspection.lines_total,
        status: "completed".to_string(),
        compatibility: profile.compatibility.clone(),
    }
}

fn replay_metadata_from_failure(
    path: &std::path::Path,
    profile: &ReplayProfile,
    inspection: &JournalInspection,
) -> ReplayMetadata {
    ReplayMetadata {
        replay_id: format!("replay:{}", profile.journal_identity),
        journal_identity: profile.journal_identity.clone(),
        journal_path: path.display().to_string(),
        record_schema: profile.record_schema.clone(),
        journal_schema: profile.journal_schema.clone(),
        started_at: unix_timestamp_string(),
        completed_at: unix_timestamp_string(),
        lines_total: inspection.lines_total,
        lines_replayed: 0,
        records_written: 0,
        records_duplicate: 0,
        records_skipped: inspection.lines_total,
        invalid_entries: inspection.invalid_entries,
        unsupported_entries: inspection.unsupported_entries,
        cursor_line: 0,
        status: if inspection.invalid_entries == 0 && profile.compatibility == "schema_mismatch" {
            "incompatible".to_string()
        } else {
            "failed".to_string()
        },
        compatibility: profile.compatibility.clone(),
    }
}

#[derive(Clone, Debug)]
struct ReplayReportIssue {
    line: usize,
    status: String,
    message: String,
}

impl ReplayReportIssue {
    fn new(line: usize, status: &str, message: &str) -> Self {
        Self {
            line,
            status: status.to_string(),
            message: message.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
struct ReplayReport {
    report_id: String,
    journal_identity: String,
    journal_path: String,
    lmdb_path: String,
    record_schema: String,
    journal_schema: String,
    compatibility: String,
    started_at: String,
    completed_at: String,
    lines_total: usize,
    valid_entries: usize,
    invalid_entries: usize,
    unsupported_entries: usize,
    duplicate_entries: usize,
    records_seen: usize,
    records_written: usize,
    records_duplicate: usize,
    records_skipped: usize,
    cursor_line: usize,
    replay_status: String,
    replay_ready: bool,
    idempotent: bool,
    errors: Vec<ReplayReportIssue>,
    warnings: Vec<ReplayReportIssue>,
    summary: String,
}

fn replay_profile_and_inspection_for_report(
    path: &Path,
) -> Result<(ReplayProfile, JournalInspection), String> {
    if !path.exists() || !path.is_file() {
        return Ok((
            replay_profile_for_missing(path),
            JournalInspection::default(),
        ));
    }
    let inspection = Journal::inspect_jsonl(path)
        .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read {} for replay identity: {error}",
            path.display()
        )
    })?;
    Ok((
        replay_profile_for_inspection(path, &contents, &inspection),
        inspection,
    ))
}

fn replay_report_id(journal_identity: &str) -> String {
    format!("replay-report:{journal_identity}")
}

fn replay_report_from_metadata(
    metadata: &ReplayMetadata,
    inspection: &JournalInspection,
    records_seen: usize,
    idempotent: bool,
    errors: Vec<ReplayReportIssue>,
    warnings: Vec<ReplayReportIssue>,
    summary: String,
) -> ReplayReport {
    ReplayReport {
        report_id: replay_report_id(&metadata.journal_identity),
        journal_identity: metadata.journal_identity.clone(),
        journal_path: metadata.journal_path.clone(),
        lmdb_path: record_store_path().display().to_string(),
        record_schema: metadata.record_schema.clone(),
        journal_schema: metadata.journal_schema.clone(),
        compatibility: metadata.compatibility.clone(),
        started_at: metadata.started_at.clone(),
        completed_at: metadata.completed_at.clone(),
        lines_total: metadata.lines_total,
        valid_entries: inspection.valid_entries,
        invalid_entries: metadata.invalid_entries,
        unsupported_entries: metadata.unsupported_entries,
        duplicate_entries: inspection.duplicate_entries,
        records_seen,
        records_written: metadata.records_written,
        records_duplicate: metadata.records_duplicate,
        records_skipped: metadata.records_skipped,
        cursor_line: metadata.cursor_line,
        replay_status: metadata.status.clone(),
        replay_ready: inspection.replay_ready(),
        idempotent,
        errors,
        warnings,
        summary,
    }
}

fn replay_report_issues_from_inspection(inspection: &JournalInspection) -> Vec<ReplayReportIssue> {
    inspection
        .diagnostics
        .iter()
        .map(|diagnostic| ReplayReportIssue {
            line: diagnostic.line_number,
            status: diagnostic.entry_status.as_str().to_string(),
            message: diagnostic.error_code.clone(),
        })
        .collect()
}

fn replay_summary(
    replay_status: &str,
    records_written: usize,
    records_duplicate: usize,
    invalid_entries: usize,
) -> String {
    if replay_status == "completed" && records_duplicate > 0 && records_written == 0 {
        return format!(
            "Replay completed idempotently with {records_duplicate} duplicate records and no new writes."
        );
    }
    if replay_status == "completed" {
        return format!("Replay completed with {records_written} records written.");
    }
    format!("Replay failed with {invalid_entries} invalid entries and no durable writes.")
}

fn write_replay_report(report: &ReplayReport) -> Result<PathBuf, String> {
    let report_path = replay_report_path(&report.journal_identity);
    fs::create_dir_all(replay_report_dir()).map_err(|error| {
        format!(
            "failed to create replay report dir {}: {error}",
            replay_report_dir().display()
        )
    })?;
    fs::write(&report_path, replay_report_json(report))
        .map_err(|error| format!("failed to write {}: {error}", report_path.display()))?;
    Ok(report_path)
}

fn replay_report_json(report: &ReplayReport) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"report_schema\":\"yai.replay_report.v1\",\n",
            "  \"report_id\":\"{}\",\n",
            "  \"journal_identity\":\"{}\",\n",
            "  \"journal_path\":\"{}\",\n",
            "  \"lmdb_path\":\"{}\",\n",
            "  \"record_schema\":\"{}\",\n",
            "  \"journal_schema\":\"{}\",\n",
            "  \"compatibility\":\"{}\",\n",
            "  \"started_at\":\"{}\",\n",
            "  \"completed_at\":\"{}\",\n            \"duration_ms\":{},\n",
            "  \"lines_total\":{},\n",
            "  \"valid_entries\":{},\n",
            "  \"invalid_entries\":{},\n",
            "  \"unsupported_entries\":{},\n",
            "  \"duplicate_entries\":{},\n",
            "  \"records_seen\":{},\n",
            "  \"records_written\":{},\n",
            "  \"records_duplicate\":{},\n",
            "  \"records_skipped\":{},\n",
            "  \"cursor_line\":{},\n",
            "  \"replay_status\":\"{}\",\n",
            "  \"replay_ready\":\"{}\",\n",
            "  \"idempotent\":\"{}\",\n",
            "  \"errors\":[{}],\n",
            "  \"warnings\":[{}],\n",
            "  \"summary\":\"{}\"\n",
            "}}\n"
        ),
        json_escape(&report.report_id),
        json_escape(&report.journal_identity),
        json_escape(&report.journal_path),
        json_escape(&report.lmdb_path),
        json_escape(&report.record_schema),
        json_escape(&report.journal_schema),
        json_escape(&report.compatibility),
        json_escape(&report.started_at),
        json_escape(&report.completed_at),
        replay_duration_ms(&report.started_at, &report.completed_at),
        report.lines_total,
        report.valid_entries,
        report.invalid_entries,
        report.unsupported_entries,
        report.duplicate_entries,
        report.records_seen,
        report.records_written,
        report.records_duplicate,
        report.records_skipped,
        report.cursor_line,
        json_escape(&report.replay_status),
        bool_word(report.replay_ready),
        bool_word(report.idempotent),
        replay_report_issues_json(&report.errors),
        replay_report_issues_json(&report.warnings),
        json_escape(&report.summary)
    )
}

fn replay_report_issues_json(issues: &[ReplayReportIssue]) -> String {
    issues
        .iter()
        .map(|issue| {
            format!(
                "{{\"line\":{},\"status\":\"{}\",\"message\":\"{}\"}}",
                issue.line,
                json_escape(&issue.status),
                json_escape(&issue.message)
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn replay_duration_ms(started_at: &str, completed_at: &str) -> u64 {
    let started = started_at.parse::<u64>().unwrap_or(0);
    let completed = completed_at.parse::<u64>().unwrap_or(started);
    completed.saturating_sub(started) * 1000
}

pub(super) fn print_report_number(report: &str, key: &str, fallback: usize) {
    println!(
        "{key}: {}",
        json_number_field(report, key).unwrap_or_else(|| fallback.to_string())
    );
}

fn journal_identity(path: &std::path::Path, contents: &str) -> String {
    let seed = format!("{}|{}", path.display(), contents);
    format!("journal:{:016x}", fnv1a64(seed.as_bytes()))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub(super) fn unix_timestamp_string() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

pub(super) fn unix_time_ms_now() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

pub(super) fn import_journal_to_record_store(journal_path: &std::path::Path) -> Result<(), String> {
    let journal = Journal::load_jsonl(journal_path)
        .map_err(|error| format!("record store import failed to load journal: {error}"))?;
    let store = LmdbRecordStore::open(record_store_path()).map_err(|error| {
        format!(
            "record store import failed after journal write remained at {}: {error}",
            journal_path.display()
        )
    })?;
    store
        .import_journal(&journal, &journal_path.display().to_string())
        .map_err(|error| {
            format!(
                "record store import failed after journal write remained at {}: {error}",
                journal_path.display()
            )
        })
}
