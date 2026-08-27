//! Fixed controlled-filesystem review transition family.

use super::*;

const REVIEW_CASE_REF: &str = "case:new12-filesystem";
const REVIEW_ID: &str = "review:new12-fs-write-review";
const REVIEW_PENDING_ID: &str = "pending:new12-fs-write-review";
const REVIEW_ATTEMPT_ID: &str = "attempt:new12-fs-reviewed-write";
const REVIEW_OUTSIDE_ATTEMPT_ID: &str = "attempt:new12-fs-outside-write";
const REVIEW_REQUEST_RECORD_ID: &str = "rec:review:new12-fs-write-review";
const REVIEW_PENDING_RECORD_ID: &str = "rec:pending:new12-fs-write-review";
const REVIEW_REQUESTED_BY: &str = "subject:llm-provider";
const REVIEW_TARGET_SUBJECT: &str = "subject:filesystem-sandbox";
const REVIEWER_SUBJECT: &str = "subject:operator-reviewer";
const REVIEW_PROMPT_SURFACE_SUBJECT: &str = "subject:linenoise-terminal";
const REVIEW_TARGET_DISPLAY: &str = "sandbox/reviewed-output.txt";
const REVIEW_POLICY_REASON: &str = "mutative_operation_requires_review";

fn control_journal_path() -> PathBuf {
    yai_home()
        .join("store")
        .join("control")
        .join("review.jsonl")
}

fn review_sandbox_dir() -> PathBuf {
    yai_home()
        .join("tmp")
        .join("filesystem-review-loop")
        .join("sandbox")
}

fn reviewed_write_path() -> PathBuf {
    review_sandbox_dir().join("reviewed-output.txt")
}

fn review_record_summary(status: &str, resolved_at: &str) -> String {
    let sandbox = review_sandbox_dir().display().to_string();
    let target = reviewed_write_path().display().to_string();
    format!(
        "review_id:{REVIEW_ID} pending_id:{REVIEW_PENDING_ID} attempt_id:{REVIEW_ATTEMPT_ID} requested_by_subject:{REVIEW_REQUESTED_BY} target_subject:{REVIEW_TARGET_SUBJECT} operation_kind:fs.write carrier_family:filesystem target:{REVIEW_TARGET_DISPLAY} gate_outcome:require_review status:{status} reason:{REVIEW_POLICY_REASON} policy_reason:{REVIEW_POLICY_REASON} reason_required:yes created_at:spine44a resolved_at:{resolved_at} authority_scope:local-dev prompt_surface_subject:{REVIEW_PROMPT_SURFACE_SUBJECT} review_authority_subject:{REVIEWER_SUBJECT} subject:linenoise-terminal_prompt_surface:true operator_reviewer_authority:true sandbox_path:{sandbox} target_path:{target} carrier_attempted:false execution_performed:false"
    )
}

fn pending_record_summary(status: &str) -> String {
    format!(
        "pending_id:{REVIEW_PENDING_ID} review_id:{REVIEW_ID} attempt_id:{REVIEW_ATTEMPT_ID} operation_kind:fs.write carrier_family:filesystem target:{REVIEW_TARGET_DISPLAY} status:{status} reason:{REVIEW_POLICY_REASON} carrier_attempted:false execution_performed:false"
    )
}

fn persist_control_records(records: &[Record]) -> Result<(), String> {
    let journal_path = control_journal_path();
    if let Some(parent) = journal_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&journal_path)
        .map_err(|error| format!("failed to open {}: {error}", journal_path.display()))?;
    let store = LmdbRecordStore::open(record_store_path())?;
    for record in records {
        file.write_all(record.to_jsonl().as_bytes())
            .map_err(|error| format!("failed to write {}: {error}", journal_path.display()))?;
        let source_ref = format!("{}#{}", journal_path.display(), record.id);
        store.append_record(record, &source_ref)?;
    }
    Ok(())
}

fn review_summary_value(summary: &str, key: &str) -> String {
    let prefix = format!("{key}:");
    summary
        .split_whitespace()
        .find_map(|part| part.strip_prefix(&prefix))
        .unwrap_or("")
        .to_string()
}

fn review_summary_value_or(summary: &str, key: &str, fallback: &str) -> String {
    let value = review_summary_value(summary, key);
    if value.is_empty() {
        fallback.to_string()
    } else {
        value
    }
}

fn print_review_next_commands(indent: &str, review_id: &str) {
    println!("{indent}next_commands:");
    println!("{indent}  approve: yai control approve {review_id} --reason \"...\"");
    println!("{indent}  deny: yai control deny {review_id} --reason \"...\"");
    println!("{indent}  defer: yai control defer {review_id} --reason \"...\"");
    println!("{indent}  quarantine: yai control quarantine {review_id} --reason \"...\"");
}

fn receipt_status_for_review_status(status: &str) -> &str {
    match status {
        "approved" => "executed",
        "denied" => "blocked",
        "deferred" => "deferred",
        "quarantined" => "quarantined",
        _ => "none",
    }
}

fn review_is_unresolved(status: &str) -> bool {
    matches!(status, "pending_operator")
}

fn load_review_summaries_for_case(case_ref: &str) -> Result<Vec<String>, String> {
    let status = LmdbRecordStore::status(record_store_path());
    if status.status != RecordStoreStatusKind::Ready {
        return Ok(Vec::new());
    }
    let store = LmdbRecordStore::open(&status.path)?;
    let result = store.list_records_by_kind("review_request", usize::MAX)?;
    let mut items = Vec::new();
    for record in result.records {
        if record.case_ref == case_ref {
            items.push(json_string_or(&record.raw_json, "summary", ""));
        }
    }
    Ok(items)
}

fn first_open_review_for_case(case_ref: &str) -> Result<Option<String>, String> {
    Ok(load_review_summaries_for_case(case_ref)?
        .into_iter()
        .find(|summary| review_is_unresolved(&review_summary_value(summary, "status"))))
}

fn review_request_record(
    store: &LmdbRecordStore,
    review_id: &str,
) -> Result<Option<yai_core_engine::store::lmdb::StoredRecordEnvelope>, String> {
    let record_id = if review_id == REVIEW_ID {
        REVIEW_REQUEST_RECORD_ID.to_string()
    } else {
        format!("rec:{review_id}")
    };
    store.get_record_by_id(&record_id)
}

fn control_pending_record(
    store: &LmdbRecordStore,
) -> Result<Option<yai_core_engine::store::lmdb::StoredRecordEnvelope>, String> {
    store.get_record_by_id(REVIEW_PENDING_RECORD_ID)
}

fn review_request_summary(store: &LmdbRecordStore, review_id: &str) -> Result<String, String> {
    let Some(record) = review_request_record(store, review_id)? else {
        return Err(format!("review_not_found: {review_id}"));
    };
    Ok(json_string_or(&record.raw_json, "summary", ""))
}

#[cfg(unix)]
pub(super) fn daemon_filesystem_review_loop(args: &[String]) -> Result<(), String> {
    let status_response = daemon_request_response(args, "status")?;
    if extract_json_string_field(&status_response, "status").as_deref() != Some("ok") {
        return Err("daemon socket did not report ok status".to_string());
    }
    let sandbox = review_sandbox_dir();
    let target = reviewed_write_path();
    fs::create_dir_all(&sandbox)
        .map_err(|error| format!("failed to create {}: {error}", sandbox.display()))?;
    if target.exists() {
        fs::remove_file(&target)
            .map_err(|error| format!("failed to remove {}: {error}", target.display()))?;
    }

    persist_control_records(&[
        Record::from_parts(
            "rec:review-case",
            REVIEW_CASE_REF,
            RecordKind::Case,
            "subject:none",
            "",
            "",
            "",
            "case:opened operator review loop",
        ),
        Record::from_parts(
            "rec:review-fs-subject",
            REVIEW_CASE_REF,
            RecordKind::SubjectBinding,
            REVIEW_TARGET_SUBJECT,
            "",
            "",
            "",
            "subject:filesystem-sandbox bound for review loop",
        ),
        Record::from_parts(
            "rec:review-terminal-subject",
            REVIEW_CASE_REF,
            RecordKind::SubjectBinding,
            "subject:linenoise-terminal",
            "",
            "",
            "",
            "subject:linenoise-terminal is prompt surface and owns no approval authority",
        ),
        Record::from_parts(
            "rec:review-operator-subject",
            REVIEW_CASE_REF,
            RecordKind::SubjectBinding,
            REVIEWER_SUBJECT,
            "",
            "",
            "",
            "subject:operator-reviewer owns local-dev operator reviewer authority",
        ),
        Record::from_parts(
            "rec:review-terminal-authority",
            REVIEW_CASE_REF,
            RecordKind::AuthorityScope,
            "subject:linenoise-terminal",
            "",
            "",
            "",
            "authority_scope:terminal prompt_surface only no:approval_authority",
        ),
        Record::from_parts(
            "rec:review-operator-authority",
            REVIEW_CASE_REF,
            RecordKind::AuthorityScope,
            REVIEWER_SUBJECT,
            "",
            "",
            "",
            "authority_scope:local-dev operator reviewer authority approve deny defer quarantine",
        ),
        Record::from_parts(
            "rec:new12-fs-outside-write-attempt",
            REVIEW_CASE_REF,
            RecordKind::Attempt,
            REVIEW_TARGET_SUBJECT,
            REVIEW_OUTSIDE_ATTEMPT_ID,
            "",
            "",
            "op:fs.write path:outside-sandbox/forbidden.txt sandbox:outside",
        ),
        Record::from_parts(
            "rec:new12-fs-outside-write-decision",
            REVIEW_CASE_REF,
            RecordKind::Decision,
            REVIEW_TARGET_SUBJECT,
            REVIEW_OUTSIDE_ATTEMPT_ID,
            "decision:new12-fs-outside-write-deny",
            "",
            "decision:deny reason:outside_sandbox carrier_attempted:false execution_performed:false",
        ),
        Record::from_parts(
            "rec:new12-fs-outside-write-receipt",
            REVIEW_CASE_REF,
            RecordKind::FilesystemReceipt,
            REVIEW_TARGET_SUBJECT,
            REVIEW_OUTSIDE_ATTEMPT_ID,
            "decision:new12-fs-outside-write-deny",
            "receipt:new12-fs-outside-write-blocked",
            "fs:write status:blocked receipt_status:blocked sandbox:outside carrier_attempted:false execution_performed:false",
        ),
        Record::from_parts(
            "rec:new12-fs-reviewed-write-attempt",
            REVIEW_CASE_REF,
            RecordKind::Attempt,
            REVIEW_TARGET_SUBJECT,
            REVIEW_ATTEMPT_ID,
            "",
            "",
            "op:fs.write mutative path:sandbox/reviewed-output.txt gate_outcome:require_review",
        ),
        Record::from_parts(
            "rec:new12-fs-reviewed-write-gate",
            REVIEW_CASE_REF,
            RecordKind::GateResult,
            REVIEW_TARGET_SUBJECT,
            REVIEW_ATTEMPT_ID,
            "decision:new12-fs-reviewed-write-gate",
            "",
            "gate_outcome:require_review rule:mutative_operation_requires_review carrier_attempted:false execution_performed:false",
        ),
        Record::from_parts(
            REVIEW_REQUEST_RECORD_ID,
            REVIEW_CASE_REF,
            RecordKind::ReviewRequest,
            REVIEW_REQUESTED_BY,
            REVIEW_ATTEMPT_ID,
            "decision:new12-fs-reviewed-write-gate",
            "",
            review_record_summary("pending_operator", "none"),
        ),
        Record::from_parts(
            REVIEW_PENDING_RECORD_ID,
            REVIEW_CASE_REF,
            RecordKind::ControlPending,
            REVIEWER_SUBJECT,
            REVIEW_ATTEMPT_ID,
            "decision:new12-fs-reviewed-write-gate",
            "",
            pending_record_summary("pending_operator"),
        ),
    ])?;

    println!("filesystem_review_loop: completed");
    println!("case_ref: {REVIEW_CASE_REF}");
    println!("outside_sandbox_attempt: blocked");
    println!("outside_sandbox_status: blocked");
    println!("outside_sandbox_carrier_attempted: false");
    println!("outside_sandbox_execution_performed: false");
    println!("outside_sandbox_receipt_status: blocked");
    println!("review_required_attempt: pending_operator");
    println!("review_required: yes");
    println!("review_id: {REVIEW_ID}");
    println!("status: pending_operator");
    println!("carrier_attempted: false");
    println!("execution_performed: false");
    print_review_next_commands("", REVIEW_ID);
    Ok(())
}

#[cfg(not(unix))]
pub(super) fn daemon_filesystem_review_loop(_args: &[String]) -> Result<(), String> {
    Err("daemon IPC is only implemented on Unix in SPINE.44A".to_string())
}

pub(super) fn control_pending(args: &[String]) -> Result<(), String> {
    let case_ref = named_arg(args, "--case")?;
    let status = LmdbRecordStore::status(record_store_path());
    if status.status != RecordStoreStatusKind::Ready {
        print_non_ready_record_store(&status);
        return Ok(());
    }
    let store = LmdbRecordStore::open(&status.path)?;
    let result = store.list_records_by_kind("review_request", usize::MAX)?;
    let mut items = Vec::new();
    for record in result.records {
        if record.case_ref != case_ref {
            continue;
        }
        let summary = json_string_or(&record.raw_json, "summary", "");
        let status = review_summary_value(&summary, "status");
        if matches!(status.as_str(), "pending_operator" | "deferred") {
            items.push(summary);
        }
    }
    println!("control_pending:");
    println!("case_ref: {case_ref}");
    println!("items_total: {}", items.len());
    if items.is_empty() {
        println!("items: none");
    } else {
        println!("items:");
        for summary in items {
            let review_id = review_summary_value(&summary, "review_id");
            println!("- review_id: {review_id}");
            println!(
                "  attempt_id: {}",
                review_summary_value(&summary, "attempt_id")
            );
            println!(
                "  operation_kind: {}",
                review_summary_value(&summary, "operation_kind")
            );
            println!(
                "  carrier_family: {}",
                review_summary_value(&summary, "carrier_family")
            );
            println!(
                "  target: {}",
                review_summary_value_or(&summary, "target", REVIEW_TARGET_DISPLAY)
            );
            println!("  status: {}", review_summary_value(&summary, "status"));
            println!(
                "  reason: {}",
                review_summary_value_or(&summary, "reason", REVIEW_POLICY_REASON)
            );
            println!(
                "  carrier_attempted: {}",
                review_summary_value(&summary, "carrier_attempted")
            );
            println!(
                "  execution_performed: {}",
                review_summary_value(&summary, "execution_performed")
            );
            println!("  allowed_actions:");
            println!("    - approve");
            println!("    - deny");
            println!("    - defer");
            println!("    - quarantine");
            print_review_next_commands("  ", &review_id);
        }
    }
    Ok(())
}

pub(super) fn control_show(args: &[String]) -> Result<(), String> {
    let review_id = args
        .first()
        .filter(|value| !value.starts_with("--"))
        .ok_or_else(|| "review_id is required".to_string())?;
    let status = LmdbRecordStore::status(record_store_path());
    if status.status != RecordStoreStatusKind::Ready {
        print_non_ready_record_store(&status);
        return Ok(());
    }
    let store = LmdbRecordStore::open(&status.path)?;
    let summary = review_request_summary(&store, review_id)?;
    println!("control_review:");
    println!("review_id: {}", review_summary_value(&summary, "review_id"));
    println!("case_ref: {REVIEW_CASE_REF}");
    println!(
        "attempt_id: {}",
        review_summary_value(&summary, "attempt_id")
    );
    println!(
        "requested_by_subject: {}",
        review_summary_value(&summary, "requested_by_subject")
    );
    println!("review_authority_subject: {REVIEWER_SUBJECT}");
    println!("prompt_surface_subject: {REVIEW_PROMPT_SURFACE_SUBJECT}");
    println!(
        "operation_kind: {}",
        review_summary_value(&summary, "operation_kind")
    );
    println!(
        "carrier_family: {}",
        review_summary_value(&summary, "carrier_family")
    );
    println!(
        "target: {}",
        review_summary_value_or(&summary, "target", REVIEW_TARGET_DISPLAY)
    );
    println!(
        "policy_reason: {}",
        review_summary_value_or(&summary, "policy_reason", REVIEW_POLICY_REASON)
    );
    println!("status: {}", review_summary_value(&summary, "status"));
    println!("receipt_required: yes");
    if let Some(pending) = control_pending_record(&store)? {
        let pending_summary = json_string_or(&pending.raw_json, "summary", "");
        println!(
            "carrier_attempted: {}",
            review_summary_value(&pending_summary, "carrier_attempted")
        );
        println!(
            "execution_performed: {}",
            review_summary_value(&pending_summary, "execution_performed")
        );
    }
    println!("allowed_actions:");
    println!("- approve");
    println!("- deny");
    println!("- defer");
    println!("- quarantine");
    println!("subject:linenoise-terminal is prompt surface only");
    println!("subject:operator-reviewer is local-dev review authority");
    println!("operator reviewer authority: local-dev");
    Ok(())
}

pub(super) fn control_resolve(args: &[String], action: &str) -> Result<(), String> {
    let review_id = args
        .first()
        .filter(|value| !value.starts_with("--"))
        .ok_or_else(|| "review_id is required".to_string())?;
    let reason = named_arg(args, "--reason")?;
    let status = LmdbRecordStore::status(record_store_path());
    if status.status != RecordStoreStatusKind::Ready {
        print_non_ready_record_store(&status);
        return Ok(());
    }
    let store = LmdbRecordStore::open(&status.path)?;
    let summary = review_request_summary(&store, review_id)?;
    let current_status = review_summary_value(&summary, "status");
    if !matches!(current_status.as_str(), "pending_operator" | "deferred") {
        return Err(format!("review_not_resolvable: {current_status}"));
    }
    let resolution_status = match action {
        "approve" => "approved",
        "deny" => "denied",
        "defer" => "deferred",
        "quarantine" => "quarantined",
        _ => return Err(format!("unsupported_review_action: {action}")),
    };
    let decision = match action {
        "approve" => "allow_with_constraints",
        "deny" => "deny",
        "defer" => "defer",
        "quarantine" => "quarantine",
        _ => "unknown",
    };
    let receipt_status = match action {
        "approve" => "executed",
        "deny" => "blocked",
        "defer" => "deferred",
        "quarantine" => "quarantined",
        _ => "unknown",
    };
    let carrier_attempted = action == "approve";
    let execution_performed = action == "approve";
    let resolved_at = unix_timestamp_string();
    let safe_reason = reason.replace('\n', " ");
    let mut records = vec![
        Record::from_parts(
            REVIEW_REQUEST_RECORD_ID,
            REVIEW_CASE_REF,
            RecordKind::ReviewRequest,
            REVIEW_REQUESTED_BY,
            REVIEW_ATTEMPT_ID,
            format!("decision:new12-fs-review-{action}"),
            format!("receipt:new12-fs-review-{receipt_status}"),
            review_record_summary(resolution_status, &resolved_at),
        ),
        Record::from_parts(
            REVIEW_PENDING_RECORD_ID,
            REVIEW_CASE_REF,
            RecordKind::ControlPending,
            REVIEWER_SUBJECT,
            REVIEW_ATTEMPT_ID,
            format!("decision:new12-fs-review-{action}"),
            format!("receipt:new12-fs-review-{receipt_status}"),
            format!(
                "pending_id:{REVIEW_PENDING_ID} review_id:{REVIEW_ID} attempt_id:{REVIEW_ATTEMPT_ID} operation_kind:fs.write carrier_family:filesystem target:{REVIEW_TARGET_DISPLAY} status:{resolution_status} reason:{REVIEW_POLICY_REASON} carrier_attempted:{} execution_performed:{}",
                carrier_attempted,
                execution_performed
            ),
        ),
        Record::from_parts(
            format!("rec:review-decision:new12-fs-write-review-{action}"),
            REVIEW_CASE_REF,
            RecordKind::ReviewDecision,
            REVIEWER_SUBJECT,
            REVIEW_ATTEMPT_ID,
            format!("decision:new12-fs-review-{action}"),
            "",
            format!(
                "review_id:{review_id} reviewer_subject:{REVIEWER_SUBJECT} action:{action} reason:{} authority_scope:local-dev result:{resolution_status}",
                safe_reason
            ),
        ),
        Record::from_parts(
            format!("rec:new12-fs-review-final-decision-{action}"),
            REVIEW_CASE_REF,
            RecordKind::Decision,
            REVIEWER_SUBJECT,
            REVIEW_ATTEMPT_ID,
            format!("decision:new12-fs-review-{action}"),
            "",
            format!("decision:{decision} review_id:{review_id} authority_scope:local-dev"),
        ),
    ];

    if action == "approve" {
        let sandbox = review_summary_value(&summary, "sandbox_path");
        let target = review_summary_value(&summary, "target_path");
        if !path_inside_sandbox(&sandbox, &target) {
            return Err("review target path is outside sandbox".to_string());
        }
        if let Some(parent) = Path::new(&target).parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }
        fs::write(&target, "approved reviewed filesystem write\n")
            .map_err(|error| format!("failed to write {target}: {error}"))?;
        records.push(Record::from_parts(
            "rec:new12-fs-review-dispatch-approve",
            REVIEW_CASE_REF,
            RecordKind::CarrierRequest,
            REVIEW_TARGET_SUBJECT,
            REVIEW_ATTEMPT_ID,
            "decision:new12-fs-review-approve",
            "",
            "dispatch:filesystem status:dispatched carrier_attempted:true execution_performed:true",
        ));
    }

    records.push(Record::from_parts(
        format!("rec:new12-fs-review-receipt-{action}"),
        REVIEW_CASE_REF,
        RecordKind::FilesystemReceipt,
        REVIEW_TARGET_SUBJECT,
        REVIEW_ATTEMPT_ID,
        format!("decision:new12-fs-review-{action}"),
        format!("receipt:new12-fs-review-{receipt_status}"),
        format!(
            "fs:write status:{receipt_status} review_id:{review_id} carrier_attempted:{} execution_performed:{}",
            carrier_attempted, execution_performed
        ),
    ));

    persist_control_records(&records)?;
    println!("review_resolution:");
    println!("review_id: {review_id}");
    println!("action: {action}");
    println!("status: {resolution_status}");
    println!("decision: {decision}");
    println!("carrier_family: filesystem");
    println!("carrier_attempted: {carrier_attempted}");
    println!("execution_performed: {execution_performed}");
    println!("receipt_status: {receipt_status}");
    if action == "defer" {
        println!("pending_condition: operator_or_policy_followup");
    }
    if action == "quarantine" {
        println!("quarantine_scope: case");
    }
    Ok(())
}

pub(super) fn control_review_interactive(args: &[String]) -> Result<(), String> {
    let case_ref = named_arg(args, "--case")?;
    if !args.iter().any(|arg| arg == "--interactive") {
        return Err("--interactive is required for yai control review".to_string());
    }
    if !io::stdin().is_terminal() {
        println!("interactive_review: unavailable");
        println!("reason: not_a_tty");
        println!("use: yai control pending --case {case_ref}");
        std::process::exit(2);
    }

    let Some(summary) = first_open_review_for_case(&case_ref)? else {
        println!("interactive_review:");
        println!("case_ref: {case_ref}");
        println!("items_total: 0");
        println!("status: no_pending_reviews");
        return Ok(());
    };
    let review_id = review_summary_value(&summary, "review_id");
    println!("PENDING REVIEW");
    println!();
    println!("review_id: {review_id}");
    println!("case: {case_ref}");
    println!(
        "operation: {}",
        review_summary_value_or(&summary, "operation_kind", "fs.write")
    );
    println!(
        "target: {}",
        review_summary_value_or(&summary, "target", REVIEW_TARGET_DISPLAY)
    );
    println!(
        "carrier: {}",
        review_summary_value_or(&summary, "carrier_family", "filesystem")
    );
    println!(
        "policy: {}",
        review_summary_value_or(&summary, "policy_reason", REVIEW_POLICY_REASON)
    );
    println!(
        "carrier_attempted: {}",
        review_summary_value_or(&summary, "carrier_attempted", "false")
    );
    println!(
        "execution_performed: {}",
        review_summary_value_or(&summary, "execution_performed", "false")
    );
    println!();
    println!("Actions:");
    println!("  [a] approve");
    println!("  [d] deny");
    println!("  [f] defer");
    println!("  [q] quarantine");
    println!("  [s] skip");
    println!("  [x] exit");
    print!("choice> ");
    io::stdout()
        .flush()
        .map_err(|error| format!("failed to flush prompt: {error}"))?;
    let mut choice = String::new();
    io::stdin()
        .read_line(&mut choice)
        .map_err(|error| format!("failed to read choice: {error}"))?;
    let action = match choice.trim() {
        "a" => "approve",
        "d" => "deny",
        "f" => "defer",
        "q" => "quarantine",
        "s" => {
            println!("interactive_review:");
            println!("case_ref: {case_ref}");
            println!("review_id: {review_id}");
            println!("status: skipped");
            return Ok(());
        }
        "x" => {
            println!("interactive_review:");
            println!("case_ref: {case_ref}");
            println!("status: exited");
            return Ok(());
        }
        other => return Err(format!("invalid_review_choice: {other}")),
    };
    print!("reason> ");
    io::stdout()
        .flush()
        .map_err(|error| format!("failed to flush reason prompt: {error}"))?;
    let mut reason = String::new();
    io::stdin()
        .read_line(&mut reason)
        .map_err(|error| format!("failed to read reason: {error}"))?;
    let reason = if reason.trim().is_empty() {
        "interactive review".to_string()
    } else {
        reason.trim().to_string()
    };
    let resolve_args = vec![review_id, "--reason".to_string(), reason];
    control_resolve(&resolve_args, action)
}

pub(super) fn control_watch(args: &[String]) -> Result<(), String> {
    let case_ref = named_arg(args, "--case")?;
    let interval_ms = optional_arg(args, "--interval-ms")
        .unwrap_or_else(|| "1000".to_string())
        .parse::<u64>()
        .map_err(|_| "--interval-ms must be a positive integer".to_string())?
        .max(1);
    let max_events = optional_arg(args, "--max-events")
        .unwrap_or_else(|| "1".to_string())
        .parse::<usize>()
        .map_err(|_| "--max-events must be a positive integer".to_string())?
        .max(1);
    println!("control_watch:");
    println!("case_ref: {case_ref}");
    println!("interval_ms: {interval_ms}");
    println!("mode: polling");

    let mut events_seen = 0usize;
    let mut seen = HashSet::new();
    for attempt in 0..2 {
        for summary in load_review_summaries_for_case(&case_ref)? {
            let review_id = review_summary_value(&summary, "review_id");
            let status = review_summary_value(&summary, "status");
            let event_key = format!("{review_id}:{status}");
            if !seen.insert(event_key) {
                continue;
            }
            let operation = review_summary_value_or(&summary, "operation_kind", "fs.write");
            let target = review_summary_value_or(&summary, "target", REVIEW_TARGET_DISPLAY);
            if review_is_unresolved(&status) {
                println!("[control] {status} {review_id} {operation} {target}");
            } else {
                println!(
                    "[control] {status} {review_id} receipt:{}",
                    receipt_status_for_review_status(&status)
                );
            }
            events_seen += 1;
            if events_seen >= max_events {
                println!("control_watch:");
                println!("status: completed");
                println!("events_seen: {events_seen}");
                return Ok(());
            }
        }
        if events_seen > 0 {
            break;
        }
        if attempt == 0 {
            std::thread::sleep(std::time::Duration::from_millis(interval_ms));
        }
    }
    println!("control_watch:");
    println!("status: completed");
    println!("events_seen: {events_seen}");
    Ok(())
}

pub(super) fn control_wait(args: &[String]) -> Result<(), String> {
    let review_id = args
        .first()
        .filter(|value| !value.starts_with("--"))
        .ok_or_else(|| "review_id is required".to_string())?
        .to_string();
    let timeout_seconds = named_arg(args, "--timeout")?
        .parse::<u64>()
        .map_err(|_| "--timeout must be a positive integer".to_string())?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_seconds);
    let mut last_status = "not_found".to_string();
    loop {
        let status = LmdbRecordStore::status(record_store_path());
        if status.status == RecordStoreStatusKind::Ready {
            let store = LmdbRecordStore::open(&status.path)?;
            if let Some(record) = review_request_record(&store, &review_id)? {
                let summary = json_string_or(&record.raw_json, "summary", "");
                let review_status = review_summary_value_or(&summary, "status", "not_found");
                last_status = review_status.clone();
                if !review_is_unresolved(&review_status) {
                    println!("control_wait:");
                    println!("review_id: {review_id}");
                    println!("status: {review_status}");
                    println!("resolved: yes");
                    println!("timeout: false");
                    println!(
                        "receipt_status: {}",
                        receipt_status_for_review_status(&review_status)
                    );
                    return Ok(());
                }
            }
        }
        if std::time::Instant::now() >= deadline {
            println!("control_wait:");
            println!("review_id: {review_id}");
            println!("status: {last_status}");
            println!("resolved: no");
            println!("timeout: true");
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
