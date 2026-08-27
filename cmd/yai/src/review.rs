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

fn print_review_next_commands(indent: &str, review_id: &str) {
    println!("{indent}next_commands:");
    println!("{indent}  approve: yai control approve {review_id} --reason \"...\"");
    println!("{indent}  deny: yai control deny {review_id} --reason \"...\"");
    println!("{indent}  defer: yai control defer {review_id} --reason \"...\"");
    println!("{indent}  quarantine: yai control quarantine {review_id} --reason \"...\"");
}

fn review_status_label(status: &ReviewResolution) -> &'static str {
    match status {
        ReviewResolution::PendingOperator => "pending_operator",
        ReviewResolution::Approved => "approved",
        ReviewResolution::Denied => "denied",
        ReviewResolution::Deferred => "deferred",
        ReviewResolution::Quarantined => "quarantined",
    }
}

fn receipt_status_for_review_status(status: &ReviewResolution) -> &'static str {
    match status {
        ReviewResolution::Approved => "executed",
        ReviewResolution::Denied => "blocked",
        ReviewResolution::Deferred => "deferred",
        ReviewResolution::Quarantined => "quarantined",
        ReviewResolution::PendingOperator => "none",
    }
}

fn review_is_unresolved(status: &ReviewResolution) -> bool {
    matches!(status, ReviewResolution::PendingOperator)
}

fn review_is_resolvable(status: &ReviewResolution) -> bool {
    matches!(
        status,
        ReviewResolution::PendingOperator | ReviewResolution::Deferred
    )
}

fn legacy_review_from_record(record: &StoredRecordEnvelope) -> Option<ReviewState> {
    let LegacyDecodeOutcome::Promoted(legacy) = decode_legacy_record(&record.raw_json) else {
        return None;
    };
    if legacy.record_kind != RecordKind::ReviewRequest {
        return None;
    }
    let status = match legacy.compatibility_value("status")? {
        "pending_operator" => ReviewResolution::PendingOperator,
        "approved" => ReviewResolution::Approved,
        "denied" => ReviewResolution::Denied,
        "deferred" => ReviewResolution::Deferred,
        "quarantined" => ReviewResolution::Quarantined,
        _ => return None,
    };
    Some(ReviewState {
        review_id: legacy
            .compatibility_value("review_id")
            .unwrap_or(&legacy.record_id)
            .to_string(),
        attempt_id: legacy.attempt_id.clone(),
        requested_by_participant: legacy.subject_ref.clone(),
        target_participant: legacy
            .compatibility_value("target_subject")
            .unwrap_or(REVIEW_TARGET_SUBJECT)
            .to_string(),
        reviewer_participant: legacy
            .compatibility_value("review_authority_subject")
            .unwrap_or(REVIEWER_SUBJECT)
            .to_string(),
        operation_kind: legacy
            .compatibility_value("operation_kind")
            .unwrap_or("fs.write")
            .to_string(),
        carrier_family: legacy
            .compatibility_value("carrier_family")
            .unwrap_or("filesystem")
            .to_string(),
        target_display: legacy
            .compatibility_value("target")
            .unwrap_or(REVIEW_TARGET_DISPLAY)
            .to_string(),
        sandbox_path: legacy
            .compatibility_value("sandbox_path")
            .unwrap_or("")
            .to_string(),
        target_path: legacy
            .compatibility_value("target_path")
            .unwrap_or("")
            .to_string(),
        policy_reason: legacy
            .compatibility_value("policy_reason")
            .unwrap_or(REVIEW_POLICY_REASON)
            .to_string(),
        status,
        carrier_attempted: legacy.compatibility_value("carrier_attempted") == Some("true"),
        execution_performed: legacy.compatibility_value("execution_performed") == Some("true"),
        decision_ref: (!legacy.decision_id.is_empty()).then_some(legacy.decision_id.clone()),
        receipt_ref: (!legacy.receipt_id.is_empty()).then_some(legacy.receipt_id.clone()),
    })
}

fn load_reviews_for_case(case_ref: &str) -> Result<Vec<ReviewState>, String> {
    let status = LmdbRecordStore::status(record_store_path());
    if status.status != RecordStoreStatusKind::Ready {
        return Ok(Vec::new());
    }
    let store = LmdbRecordStore::open(&status.path)?;
    if let Some(state) = store.get_case_state(case_ref)? {
        return Ok(state.reviews);
    }
    let result = store.list_records_by_kind("review_request", usize::MAX)?;
    Ok(result
        .records
        .into_iter()
        .filter(|record| record.case_ref == case_ref)
        .filter_map(|record| legacy_review_from_record(&record))
        .collect())
}

fn first_open_review_for_case(case_ref: &str) -> Result<Option<ReviewState>, String> {
    Ok(load_reviews_for_case(case_ref)?
        .into_iter()
        .find(|review| review_is_unresolved(&review.status)))
}

fn review_request_state(store: &LmdbRecordStore, review_id: &str) -> Result<ReviewState, String> {
    if let Some(state) = store.get_case_state(REVIEW_CASE_REF)? {
        if let Some(review) = state
            .reviews
            .into_iter()
            .find(|review| review.review_id == review_id)
        {
            return Ok(review);
        }
    }
    let record_id = if review_id == REVIEW_ID {
        REVIEW_REQUEST_RECORD_ID.to_string()
    } else {
        format!("rec:{review_id}")
    };
    let Some(record) = store.get_record_by_id(&record_id)? else {
        return Err(format!("review_not_found: {review_id}"));
    };
    legacy_review_from_record(&record).ok_or_else(|| format!("review_not_found: {review_id}"))
}

fn review_transition_source(participant_id: Option<&str>, source_ref: &str) -> TransitionSource {
    TransitionSource {
        component: "yai.review_boundary".to_string(),
        participant_id: participant_id.map(ToString::to_string),
        source_ref: Some(source_ref.to_string()),
    }
}

fn ensure_review_case_authority(store: &LmdbRecordStore) -> Result<(), String> {
    let mut state = if let Some(state) = store.get_case_state(REVIEW_CASE_REF)? {
        state
    } else {
        store
            .commit_transition(PendingTransition::new(
                "transition:review-case-open",
                REVIEW_CASE_REF,
                0,
                review_transition_source(None, "filesystem-review-loop"),
                TransitionPayload::CaseOpened {
                    lifecycle: CaseLifecycle::Open,
                },
            ))?
            .state
    };
    for (participant_id, role) in [
        (REVIEW_REQUESTED_BY, "requesting_participant"),
        (REVIEW_TARGET_SUBJECT, "filesystem_resource_participant"),
        (REVIEW_PROMPT_SURFACE_SUBJECT, "prompt_surface"),
        (REVIEWER_SUBJECT, "operator_reviewer"),
    ] {
        if state
            .participants
            .iter()
            .any(|participant| participant.participant_id == participant_id)
        {
            continue;
        }
        state = store
            .commit_transition(PendingTransition::new(
                format!(
                    "transition:review-participant:{}",
                    participant_id.replace(':', "-")
                ),
                REVIEW_CASE_REF,
                state.generation,
                review_transition_source(Some(participant_id), "filesystem-review-loop"),
                TransitionPayload::ParticipantBound {
                    participant_id: participant_id.to_string(),
                    role: role.to_string(),
                },
            ))?
            .state;
    }
    if state
        .reviews
        .iter()
        .any(|review| review.review_id == REVIEW_ID)
    {
        return Ok(());
    }
    let review = ReviewState {
        review_id: REVIEW_ID.to_string(),
        attempt_id: REVIEW_ATTEMPT_ID.to_string(),
        requested_by_participant: REVIEW_REQUESTED_BY.to_string(),
        target_participant: REVIEW_TARGET_SUBJECT.to_string(),
        reviewer_participant: REVIEWER_SUBJECT.to_string(),
        operation_kind: "fs.write".to_string(),
        carrier_family: "filesystem".to_string(),
        target_display: REVIEW_TARGET_DISPLAY.to_string(),
        sandbox_path: review_sandbox_dir().display().to_string(),
        target_path: reviewed_write_path().display().to_string(),
        policy_reason: REVIEW_POLICY_REASON.to_string(),
        status: ReviewResolution::PendingOperator,
        carrier_attempted: false,
        execution_performed: false,
        decision_ref: None,
        receipt_ref: None,
    };
    let mut pending = PendingTransition::new(
        "transition:review-request:new12-fs-write-review",
        REVIEW_CASE_REF,
        state.generation,
        review_transition_source(Some(REVIEW_REQUESTED_BY), REVIEW_REQUEST_RECORD_ID),
        TransitionPayload::ReviewRequested { review },
    );
    pending.causal_refs.push(REVIEW_ATTEMPT_ID.to_string());
    pending.summary = Some("Fixed filesystem fixture requires operator review".to_string());
    store.commit_transition(pending)?;
    Ok(())
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

    let canonical_store = LmdbRecordStore::open(record_store_path())?;
    ensure_review_case_authority(&canonical_store)?;

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
    let items: Vec<_> = load_reviews_for_case(&case_ref)?
        .into_iter()
        .filter(|review| review_is_resolvable(&review.status))
        .collect();
    println!("control_pending:");
    println!("case_ref: {case_ref}");
    println!("items_total: {}", items.len());
    if items.is_empty() {
        println!("items: none");
    } else {
        println!("items:");
        for review in items {
            println!("- review_id: {}", review.review_id);
            println!("  attempt_id: {}", review.attempt_id);
            println!("  operation_kind: {}", review.operation_kind);
            println!("  carrier_family: {}", review.carrier_family);
            println!("  target: {}", review.target_display);
            println!("  status: {}", review_status_label(&review.status));
            println!("  reason: {}", review.policy_reason);
            println!("  carrier_attempted: {}", review.carrier_attempted);
            println!("  execution_performed: {}", review.execution_performed);
            println!("  allowed_actions:");
            println!("    - approve");
            println!("    - deny");
            println!("    - defer");
            println!("    - quarantine");
            print_review_next_commands("  ", &review.review_id);
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
    let review = review_request_state(&store, review_id)?;
    println!("control_review:");
    println!("review_id: {}", review.review_id);
    println!("case_ref: {REVIEW_CASE_REF}");
    println!("attempt_id: {}", review.attempt_id);
    println!("requested_by_subject: {}", review.requested_by_participant);
    println!("review_authority_subject: {}", review.reviewer_participant);
    println!("prompt_surface_subject: {REVIEW_PROMPT_SURFACE_SUBJECT}");
    println!("operation_kind: {}", review.operation_kind);
    println!("carrier_family: {}", review.carrier_family);
    println!("target: {}", review.target_display);
    println!("policy_reason: {}", review.policy_reason);
    println!("status: {}", review_status_label(&review.status));
    println!("receipt_required: yes");
    println!("carrier_attempted: {}", review.carrier_attempted);
    println!("execution_performed: {}", review.execution_performed);
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
    let review = review_request_state(&store, review_id)?;
    if !review_is_resolvable(&review.status) {
        return Err(format!(
            "review_not_resolvable: {}",
            review_status_label(&review.status)
        ));
    }
    let resolution = match action {
        "approve" => ReviewResolution::Approved,
        "deny" => ReviewResolution::Denied,
        "defer" => ReviewResolution::Deferred,
        "quarantine" => ReviewResolution::Quarantined,
        _ => return Err(format!("unsupported_review_action: {action}")),
    };
    let resolution_status = review_status_label(&resolution);
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
        let sandbox = review.sandbox_path.clone();
        let target = review.target_path.clone();
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

    let state = store
        .get_case_state(REVIEW_CASE_REF)?
        .ok_or_else(|| format!("canonical CaseState missing for {REVIEW_CASE_REF}"))?;
    let decision_ref = format!("decision:new12-fs-review-{action}");
    let receipt_ref = format!("receipt:new12-fs-review-{receipt_status}");
    let mut transition = PendingTransition::new(
        format!(
            "transition:review-resolution:{}:{}",
            action,
            state.generation + 1
        ),
        REVIEW_CASE_REF,
        state.generation,
        review_transition_source(Some(REVIEWER_SUBJECT), review_id),
        TransitionPayload::ReviewResolved {
            review_id: review_id.to_string(),
            attempt_id: review.attempt_id.clone(),
            resolution,
            reason: safe_reason.clone(),
            decision_ref,
            receipt_ref,
            carrier_attempted,
            execution_performed,
        },
    );
    transition.causal_refs = vec![review_id.to_string(), review.attempt_id.clone()];
    transition.summary = Some(format!("Fixture review resolved as {resolution_status}"));
    store.commit_transition(transition)?;

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

    let Some(review) = first_open_review_for_case(&case_ref)? else {
        println!("interactive_review:");
        println!("case_ref: {case_ref}");
        println!("items_total: 0");
        println!("status: no_pending_reviews");
        return Ok(());
    };
    let review_id = review.review_id.clone();
    println!("PENDING REVIEW");
    println!();
    println!("review_id: {review_id}");
    println!("case: {case_ref}");
    println!("operation: {}", review.operation_kind);
    println!("target: {}", review.target_display);
    println!("carrier: {}", review.carrier_family);
    println!("policy: {}", review.policy_reason);
    println!("carrier_attempted: {}", review.carrier_attempted);
    println!("execution_performed: {}", review.execution_performed);
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
        for review in load_reviews_for_case(&case_ref)? {
            let review_id = review.review_id.clone();
            let status = review_status_label(&review.status);
            let event_key = format!("{review_id}:{status}");
            if !seen.insert(event_key) {
                continue;
            }
            if review_is_unresolved(&review.status) {
                println!(
                    "[control] {status} {review_id} {} {}",
                    review.operation_kind, review.target_display
                );
            } else {
                println!(
                    "[control] {status} {review_id} receipt:{}",
                    receipt_status_for_review_status(&review.status)
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
            if let Ok(review) = review_request_state(&store, &review_id) {
                let review_status = review_status_label(&review.status).to_string();
                last_status = review_status.clone();
                if !review_is_unresolved(&review.status) {
                    println!("control_wait:");
                    println!("review_id: {review_id}");
                    println!("status: {review_status}");
                    println!("resolved: yes");
                    println!("timeout: false");
                    println!(
                        "receipt_status: {}",
                        receipt_status_for_review_status(&review.status)
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
