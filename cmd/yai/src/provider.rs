//! Case-scoped projection construction and OpenAI-compatible provider invocation.

use super::*;

pub(super) fn projection_summary(args: &[String]) -> Result<(), String> {
    let path = journal_arg(args)?;
    let journal = Journal::load_jsonl(&path)
        .map_err(|error| format!("failed to load {}: {error}", path.display()))?;
    let projection = ProjectionSummary::from_journal("operator", &journal);
    println!("records: {}", projection.source_record_count);
    if !projection.case_ref.is_empty() {
        println!("case: {}", projection.case_ref);
    }
    println!("receipts: {}", projection.receipt_count);
    println!("subjects: {}", projection.subject_count);
    Ok(())
}

pub(super) fn projection_inspect(args: &[String]) -> Result<(), String> {
    let path = journal_arg(args)?;
    let consumer = optional_arg(args, "--consumer").unwrap_or_else(|| "model".to_string());
    let journal = Journal::load_jsonl(&path)
        .map_err(|error| format!("failed to load {}: {error}", path.display()))?;
    let projection = ProjectionSummary::from_journal("projection", &journal);
    let freshness = projection_freshness_view(&projection.case_ref, &consumer);
    println!("records: {}", projection.source_record_count);
    if !projection.case_ref.is_empty() {
        println!("case: {}", projection.case_ref);
    }
    println!("case_domains: {}", projection.case_domain_count);
    println!("case_attachments: {}", projection.case_attachment_count);
    println!("case_bindings: {}", projection.case_binding_count);
    println!(
        "interaction_threads: {}",
        projection.interaction_thread_count
    );
    println!("interaction_turns: {}", projection.interaction_turn_count);
    println!(
        "participant_view_frames: {}",
        projection.participant_view_frame_count
    );
    println!(
        "projection_requests: {}",
        projection.projection_request_count
    );
    println!("projection_results: {}", projection.projection_result_count);
    println!("projection_rules: {}", projection.projection_rule_count);
    println!("authority_scopes: {}", projection.authority_scope_count);
    println!(
        "model_interpretations: {}",
        projection.model_interpretation_count
    );
    println!("operator: {}", projection.operator_projection_count);
    println!("model: {}", projection.model_projection_count);
    println!("audit: {}", projection.audit_projection_count);
    println!(
        "redacted_or_limited: {}",
        projection.limited_projection_count
    );
    println!("consumer: {}", freshness.consumer);
    println!("projection_freshness: {}", freshness.freshness);
    println!("stale_reason: {}", freshness.stale_reason);
    println!("freshness_policy: {}", freshness.policy);
    println!("freshness_source: {}", freshness.source);
    println!("source: {}", freshness.source);
    Ok(())
}

fn default_redaction_for_consumer(consumer: &str) -> &'static str {
    match consumer {
        "model" | "agent" => "summary_only",
        "debug" => "refs_only",
        _ => "none",
    }
}

pub(super) fn projection_request(args: &[String]) -> Result<(), String> {
    let path = journal_arg(args)?;
    let consumer = named_arg(args, "--consumer")?;
    let kind = named_arg(args, "--kind")?;
    let journal = Journal::load_jsonl(&path)
        .map_err(|error| format!("failed to load {}: {error}", path.display()))?;
    let projection = ProjectionSummary::from_journal(&consumer, &journal);
    println!("projection_request: preview");
    println!("consumer: {consumer}");
    println!("kind: {kind}");
    println!("redaction: {}", default_redaction_for_consumer(&consumer));
    println!("freshness: fresh");
    println!("source_records: {}", projection.source_record_count);
    println!(
        "source_receipts: {}",
        projection.receipt_count + projection.filesystem_receipt_count
    );
    println!("source_memory: {}", projection.memory_candidate_count);
    println!("source_divergences: {}", projection.divergence_count);
    Ok(())
}

pub(super) fn semantic_context_inspect(args: &[String]) -> Result<(), String> {
    let artifact_id = optional_arg(args, "--id")
        .or_else(|| optional_arg(args, "--projection"))
        .or_else(|| optional_arg(args, "--frame"))
        .ok_or_else(|| "context inspect requires --id, --projection, or --frame".to_string())?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let artifact = store
        .get_semantic_context_artifact(&artifact_id)?
        .ok_or_else(|| format!("semantic context artifact not found: {artifact_id}"))?;
    match artifact {
        SemanticContextArtifact::Projection(projection) => {
            println!("artifact_kind: projection");
            println!("schema: {}", projection.schema);
            println!("projection_id: {}", projection.projection_id);
            println!("case_id: {}", projection.case_id);
            println!("case_generation: {}", projection.case_generation);
            println!("participant_id: {}", projection.participant_id);
            println!("purpose: {}", projection.purpose.as_str());
            println!("visibility_consumer: {}", projection.visibility.consumer);
            println!("visibility_view_kind: {}", projection.visibility.view_kind);
            println!("selected_items: {}", projection.bounds.selected_items);
            println!("omitted_items: {}", projection.bounds.omitted_items);
            println!("graph_available: {}", projection.bounds.graph_available);
            println!("memory_available: {}", projection.bounds.memory_available);
            println!(
                "retrieval_id: {}",
                projection.bounds.retrieval_id.as_deref().unwrap_or("none")
            );
            println!(
                "retrieval_candidates: {}",
                projection.bounds.retrieval_candidates
            );
            println!(
                "retrieval_selected: {}",
                projection.bounds.retrieval_selected
            );
            println!("retrieval_omitted: {}", projection.bounds.retrieval_omitted);
            if let Some(state) = store.get_case_state(&projection.case_id)? {
                println!(
                    "stale: {}",
                    yai_core_engine::context::projection_is_stale(&projection, state.generation)
                );
            }
            for entry in projection.entries {
                println!(
                    "entry: {} posture:{:?} value:{:?}",
                    entry.entry_id, entry.posture, entry.value
                );
                for provenance in entry.provenance {
                    println!(
                        "provenance: entry={} kind:{:?} source_ref:{}",
                        entry.entry_id, provenance.kind, provenance.source_ref
                    );
                }
            }
        }
        SemanticContextArtifact::ContextFrame(frame) => {
            println!("artifact_kind: context_frame");
            println!("schema: {}", frame.schema);
            println!("frame_id: {}", frame.frame_id);
            println!("projection_id: {}", frame.projection_id);
            println!("case_id: {}", frame.case_id);
            println!("case_generation: {}", frame.case_generation);
            println!("participant_id: {}", frame.participant_id);
            println!("purpose: {}", frame.purpose.as_str());
            println!("task: {}", compact_text(&frame.task, 160));
            println!(
                "output_contract_id: {}",
                frame.output_contract.contract_id()
            );
            println!("selected_items: {}", frame.entries.len());
            if let Some(state) = store.get_case_state(&frame.case_id)? {
                println!("stale: {}", frame.case_generation != state.generation);
            }
        }
        SemanticContextArtifact::RenderedInputMetadata(metadata) => {
            println!("artifact_kind: rendered_input_metadata");
            println!("schema: {}", metadata.schema);
            println!("rendered_input_id: {}", metadata.rendered_input_id);
            println!("context_frame_id: {}", metadata.context_frame_id);
            println!("provider_id: {}", metadata.provider_id);
            println!("model_id: {}", metadata.model_id);
            println!("content_digest: {}", metadata.content_digest);
            println!("content_chars: {}", metadata.content_chars);
            println!("full_render_persisted: false");
        }
    }
    Ok(())
}

pub(super) fn control_summary(args: &[String]) -> Result<(), String> {
    let path = journal_arg(args)?;
    let journal = Journal::load_jsonl(&path)
        .map_err(|error| format!("failed to load {}: {error}", path.display()))?;
    let projection = ProjectionSummary::from_journal("control", &journal);
    println!("records: {}", projection.source_record_count);
    println!("decisions: {}", projection.decision_count);
    println!("rules: {}", projection.policy_rule_count);
    println!("gates: {}", projection.gate_count);
    println!("obligations: {}", projection.obligation_count);
    println!(
        "receipt_requirements: {}",
        projection.receipt_requirement_count
    );
    Ok(())
}

fn record_in_case(record: &Record, case_ref: Option<&str>) -> bool {
    case_ref
        .map(|expected| record.case_ref == expected)
        .unwrap_or(true)
}

fn display_field<'a>(value: &'a str, fallback: &'static str) -> &'a str {
    if value.is_empty() {
        fallback
    } else {
        value
    }
}

fn render_legacy_case_entry_records(
    output: &mut String,
    title: &str,
    journal: &Journal,
    case_ref: Option<&str>,
    kinds: &[RecordKind],
) {
    let _ = writeln!(output, "## {title}");
    let mut count = 0usize;
    for record in journal
        .records()
        .iter()
        .filter(|record| record_in_case(record, case_ref) && kinds.contains(&record.kind))
    {
        let _ = writeln!(
            output,
            "- {} kind:{} subject_ref:{} attempt_id:{} decision_id:{} receipt_id:{} summary:{}",
            record.id,
            record.kind.as_str(),
            display_field(&record.subject_ref, "subject:none"),
            display_field(&record.attempt_id, "none"),
            display_field(&record.decision_id, "none"),
            display_field(&record.receipt_id, "none"),
            record.summary
        );
        count += 1;
    }
    if count == 0 {
        let _ = writeln!(output, "- none");
    }
    let _ = writeln!(output);
}

fn render_legacy_case_entry_preview(journal: &Journal, case_ref: Option<&str>) -> String {
    let projection = ProjectionSummary::from_journal("model", &journal);
    let case_ref = case_ref
        .or_else(|| (!projection.case_ref.is_empty()).then_some(projection.case_ref.as_str()));

    let mut output = String::new();
    let _ = writeln!(output, "case_ref: {}", case_ref.unwrap_or("unknown"));
    let _ = writeln!(output, "case_world: materialized");
    let _ = writeln!(output, "case_context: active");
    let _ = writeln!(output, "consumer: model");
    let _ = writeln!(
        output,
        "authority: legacy_compatibility_preview_not_provider_input"
    );
    let _ = writeln!(output, "projection_kind: model_context");
    let _ = writeln!(output, "redaction: summary_only");
    let _ = writeln!(output, "source: case_projection_graph_memory");
    let _ = writeln!(output, "raw_journal_access: not_provided");
    let _ = writeln!(output, "filesystem_access: not_provided");
    let _ = writeln!(output, "decision_authority: not_provided");
    let _ = writeln!(output, "receipt_authority: not_provided");
    let _ = writeln!(
        output,
        "terminal_authority: prompt_surface_only_no_decision_authority"
    );
    let _ = writeln!(output, "records: {}", projection.source_record_count);
    let _ = writeln!(output, "case_domains: {}", projection.case_domain_count);
    let _ = writeln!(
        output,
        "case_attachments: {}",
        projection.case_attachment_count
    );
    let _ = writeln!(output, "case_bindings: {}", projection.case_binding_count);
    let _ = writeln!(
        output,
        "interaction_threads: {}",
        projection.interaction_thread_count
    );
    let _ = writeln!(
        output,
        "interaction_turns: {}",
        projection.interaction_turn_count
    );
    let _ = writeln!(
        output,
        "participant_view_frames: {}",
        projection.participant_view_frame_count
    );
    let _ = writeln!(
        output,
        "model_projection_records: {}",
        projection.model_projection_count
    );
    let _ = writeln!(
        output,
        "operator_projection_records: {}",
        projection.operator_projection_count
    );
    let _ = writeln!(
        output,
        "filesystem_receipts: {}",
        projection.filesystem_receipt_count
    );
    let _ = writeln!(
        output,
        "memory_candidates: {}",
        projection.memory_candidate_count
    );
    let _ = writeln!(
        output,
        "projection_rules: {}",
        projection.projection_rule_count
    );
    let _ = writeln!(
        output,
        "authority_scopes: {}",
        projection.authority_scope_count
    );
    let _ = writeln!(
        output,
        "model_interpretations: {}",
        projection.model_interpretation_count
    );
    let _ = writeln!(output, "graph_edges: {}", projection.graph_edge_count);
    let _ = writeln!(output);

    render_legacy_case_entry_records(
        &mut output,
        "Case World",
        &journal,
        case_ref,
        &[
            RecordKind::CaseDomain,
            RecordKind::CaseAttachment,
            RecordKind::CaseBinding,
        ],
    );
    render_legacy_case_entry_records(
        &mut output,
        "Subjects",
        &journal,
        case_ref,
        &[RecordKind::SubjectBinding],
    );
    render_legacy_case_entry_records(
        &mut output,
        "Policy",
        &journal,
        case_ref,
        &[RecordKind::PolicyRule],
    );
    render_legacy_case_entry_records(
        &mut output,
        "Projection Rules",
        &journal,
        case_ref,
        &[RecordKind::ProjectionRule],
    );
    render_legacy_case_entry_records(
        &mut output,
        "Authority Scopes",
        &journal,
        case_ref,
        &[RecordKind::AuthorityScope],
    );
    render_legacy_case_entry_records(
        &mut output,
        "Decisions",
        &journal,
        case_ref,
        &[RecordKind::Decision],
    );
    render_legacy_case_entry_records(
        &mut output,
        "Filesystem Receipts",
        &journal,
        case_ref,
        &[RecordKind::FilesystemReceipt],
    );
    render_legacy_case_entry_records(
        &mut output,
        "Memory",
        &journal,
        case_ref,
        &[RecordKind::MemoryCandidate],
    );
    render_legacy_case_entry_records(
        &mut output,
        "Graph",
        &journal,
        case_ref,
        &[RecordKind::GraphEdge],
    );
    render_legacy_case_entry_records(
        &mut output,
        "Projection Records",
        &journal,
        case_ref,
        &[RecordKind::ProjectionRequest, RecordKind::ProjectionResult],
    );
    render_legacy_case_entry_records(
        &mut output,
        "Model Interpretations",
        &journal,
        case_ref,
        &[RecordKind::ModelInterpretation],
    );
    let _ = writeln!(output, "## Authority Boundaries");
    let _ = writeln!(
        output,
        "- case_domain, case_attachment and case_binding records define the operational case world visible to this participant."
    );
    let _ = writeln!(
        output,
        "- subject:linenoise-terminal is a vendored prompt surface only; it does not generate decisions, authorize writes, mutate receipts or own provider semantics."
    );
    let _ = writeln!(
        output,
        "- subject:llm-provider may produce claims, proposals and model_interpretation records; those are not authoritative state until checked against decisions, receipts, graph and memory."
    );
    let _ = writeln!(
        output,
        "- filesystem decisions are represented by decision records; existing decisions/receipts are historical residue, not mutable by model output."
    );
    let _ = writeln!(
        output,
        "- When answering, state authority granted by the current projection, not physical capability of the host process."
    );
    let _ = writeln!(
        output,
        "- raw_journal_access, filesystem_access, decision_authority and receipt_authority are not provided to the model participant view."
    );
    let _ = writeln!(output);
    output
}

fn print_legacy_case_entry_preview(journal: &Journal, case_ref: Option<&str>) {
    print!("{}", render_legacy_case_entry_preview(journal, case_ref));
}

fn append_case_entry_record(
    path: &PathBuf,
    journal: &Journal,
    case_ref: &str,
    subject_ref: &str,
    consumer: &str,
    kind: &str,
) -> Result<(), String> {
    let record_id = format!(
        "case-entry:{}:{}",
        subject_ref.replace(':', "-"),
        journal.count() + 1
    );
    let record = Record::from_parts(
        record_id,
        case_ref,
        RecordKind::SubjectState,
        subject_ref,
        "",
        "",
        "",
        format!(
            "case_entry:admitted consumer:{consumer} kind:{kind} redaction:summary_only raw_journal_access:not_provided filesystem_access:not_provided"
        ),
    );
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|error| format!("failed to append {}: {error}", path.display()))?;
    file.write_all(record.to_jsonl().as_bytes())
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    Ok(())
}

fn shell_quote(value: &str) -> String {
    let mut quoted = String::from("'");
    for ch in value.chars() {
        if ch == '\'' {
            quoted.push_str("'\\''");
        } else {
            quoted.push(ch);
        }
    }
    quoted.push('\'');
    quoted
}

fn print_case_enter_shell(path: &PathBuf, case_ref: &str, subject_ref: &str) {
    let prompt_flag = format!("[yai:{case_ref}]");
    println!("printf '%s\\n' {}", shell_quote("case_entry: accepted"));
    println!(
        "printf '%s\\n' {}",
        shell_quote("case_entry_status: shell_scoped")
    );
    println!("printf '%s\\n' {}", shell_quote("case_session: active"));
    println!("printf '%s\\n' {}", shell_quote("case_world: materialized"));
    println!("printf '%s\\n' {}", shell_quote("case_context: active"));
    println!(
        "printf '%s\\n' {}",
        shell_quote(&format!("case_ref: {case_ref}"))
    );
    println!(
        "printf '%s\\n' {}",
        shell_quote(&format!("subject_ref: {subject_ref}"))
    );
    println!(
        "printf '%s\\n' {}",
        shell_quote("participant_view: model_context")
    );
    println!("printf '%s\\n' {}", shell_quote("consumer: model"));
    println!("printf '%s\\n' {}", shell_quote("redaction: summary_only"));
    println!(
        "printf '%s\\n' {}",
        shell_quote("raw_journal_access: not_provided")
    );
    println!(
        "printf '%s\\n' {}",
        shell_quote("filesystem_access: not_provided")
    );
    println!(
        "export YAI_JOURNAL={}",
        shell_quote(&path.display().to_string())
    );
    println!("export YAI_CASE_REF={}", shell_quote(case_ref));
    println!("export YAI_SUBJECT_REF={}", shell_quote(subject_ref));
    println!("export YAI_CASE_PROMPT_FLAG={}", shell_quote(&prompt_flag));
    println!("if [ -z \"${{YAI_PROMPT_BASE+x}}\" ]; then export YAI_PROMPT_BASE=\"${{PROMPT:-${{PS1:-}}}}\"; fi");
    println!("if [ -z \"${{YAI_RPROMPT_BASE+x}}\" ]; then export YAI_RPROMPT_BASE=\"${{RPROMPT:-}}\"; fi");
    println!("export PROMPT=\"$YAI_CASE_PROMPT_FLAG $YAI_PROMPT_BASE\"");
    println!("export PS1=\"$PROMPT\"");
    println!("export RPROMPT=\"$YAI_RPROMPT_BASE\"");
}

pub(super) fn case_enter(args: &[String]) -> Result<(), String> {
    let path = case_journal_path(args, "yai case enter")?;
    let case_ref = named_arg(args, "--case")?;
    let subject_ref = named_arg(args, "--subject")?;
    let consumer = optional_arg(args, "--consumer").unwrap_or_else(|| "model".to_string());
    let kind = optional_arg(args, "--kind").unwrap_or_else(|| "model_context".to_string());
    let shell = optional_arg(args, "--shell");
    let journal = Journal::load_jsonl(&path)
        .map_err(|error| format!("failed to load {}: {error}", path.display()))?;

    let subject_bound = journal.records().iter().any(|record| {
        record.case_ref == case_ref
            && record.kind == RecordKind::SubjectBinding
            && record.subject_ref == subject_ref
    });
    if !subject_bound {
        return Err(format!(
            "subject {subject_ref} is not bound to case {case_ref}"
        ));
    }

    let projection_available = journal.records().iter().any(|record| {
        record.case_ref == case_ref
            && record.kind == RecordKind::ProjectionResult
            && legacy_summary_is(record, "consumer", &consumer)
            && legacy_summary_is(record, "kind", &kind)
            && legacy_summary_is(record, "redaction", "summary_only")
    });
    if !projection_available {
        return Err(format!(
            "no governed {consumer}/{kind} projection is available for case {case_ref}"
        ));
    }

    let store = LmdbRecordStore::open(record_store_path())?;
    let mut state = ensure_canonical_case(&store, &journal, &path, &case_ref)?;
    let already_admitted = state
        .participants
        .iter()
        .find(|participant| participant.participant_id == subject_ref)
        .map(|participant| {
            participant
                .admitted_views
                .iter()
                .any(|view| view.consumer == consumer && view.view_kind == kind)
        })
        .unwrap_or(false);
    if !already_admitted {
        let mut admission = PendingTransition::new(
            format!(
                "transition:case-admission:{}:{}:{}:{}",
                canonical_id_component(&case_ref),
                canonical_id_component(&subject_ref),
                canonical_id_component(&consumer),
                canonical_id_component(&kind)
            ),
            &case_ref,
            state.generation,
            provider_source(Some(&subject_ref), &path.display().to_string()),
            TransitionPayload::ParticipantAdmitted {
                participant_id: subject_ref.clone(),
                consumer: consumer.clone(),
                view_kind: kind.clone(),
            },
        );
        admission.summary = Some(format!(
            "Participant admitted to {consumer}/{kind} view for compatibility command output"
        ));
        state = store.commit_transition(admission)?.state;
        debug_assert!(state.generation > 0);
    }

    let legacy_already_admitted = journal.records().iter().any(|record| {
        record.case_ref == case_ref
            && record.kind == RecordKind::SubjectState
            && record.subject_ref == subject_ref
            && legacy_summary_is(record, "case_entry", "admitted")
            && legacy_summary_is(record, "consumer", &consumer)
            && legacy_summary_is(record, "kind", &kind)
    });

    if !legacy_already_admitted {
        append_case_entry_record(&path, &journal, &case_ref, &subject_ref, &consumer, &kind)?;
    }

    let journal = Journal::load_jsonl(&path)
        .map_err(|error| format!("failed to reload {}: {error}", path.display()))?;

    if let Some(shell) = shell.as_deref() {
        if shell != "zsh" && shell != "sh" {
            return Err(format!("unsupported shell: {shell}"));
        }
        print_case_enter_shell(&path, &case_ref, &subject_ref);
        return Ok(());
    }

    println!("case_entry: accepted");
    println!(
        "case_entry_status: {}",
        if already_admitted {
            "already_admitted"
        } else {
            "admitted"
        }
    );
    println!("case_session: active");
    println!("case_world: materialized");
    println!("case_context: active");
    println!("case_ref: {case_ref}");
    println!("subject_ref: {subject_ref}");
    println!("participant_view: {kind}");
    println!("consumer: {consumer}");
    println!("redaction: summary_only");
    println!("raw_journal_access: not_provided");
    println!("filesystem_access: not_provided");
    println!("authority_scope: model interpretation_only");
    println!();
    print_legacy_case_entry_preview(&journal, Some(&case_ref));
    Ok(())
}

fn append_record_to_journal(path: &PathBuf, record: &Record) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|error| format!("failed to append {}: {error}", path.display()))?;
    file.write_all(record.to_jsonl().as_bytes())
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    Ok(())
}

fn legacy_summary_value(record: &Record, key: &str) -> Option<String> {
    parse_legacy_summary_fields(&record.summary).remove(key)
}

fn legacy_summary_is(record: &Record, key: &str, expected: &str) -> bool {
    legacy_summary_value(record, key).as_deref() == Some(expected)
}

fn canonical_id_component(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, ':' | '-' | '_' | '.') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn provider_source(participant_id: Option<&str>, source_ref: &str) -> TransitionSource {
    TransitionSource {
        component: "yai.provider_boundary".to_string(),
        participant_id: participant_id.map(ToString::to_string),
        source_ref: Some(source_ref.to_string()),
    }
}

fn legacy_provenance(
    path: &Path,
    record_id: Option<&str>,
    promotion: &str,
) -> TransitionProvenance {
    TransitionProvenance {
        origin_schema: "yai.store.record.v0".to_string(),
        origin_ref: path.display().to_string(),
        legacy_record_id: record_id.map(ToString::to_string),
        promotion: promotion.to_string(),
    }
}

fn ensure_canonical_case(
    store: &LmdbRecordStore,
    journal: &Journal,
    journal_path: &Path,
    case_id: &str,
) -> Result<yai_core_engine::transition::CaseState, String> {
    if let Some(state) = store.get_case_state(case_id)? {
        return Ok(state);
    }
    let mut open = PendingTransition::new(
        format!("transition:case-open:{}", canonical_id_component(case_id)),
        case_id,
        0,
        provider_source(None, &journal_path.display().to_string()),
        TransitionPayload::CaseOpened {
            lifecycle: CaseLifecycle::Open,
        },
    );
    open.provenance.push(legacy_provenance(
        journal_path,
        None,
        "compatibility_bootstrap",
    ));
    open.summary = Some("Case opened from readable legacy journal input".to_string());
    let mut state = store.commit_transition(open)?.state;

    let mut seen = HashSet::new();
    for record in journal.records().iter().filter(|record| {
        record.case_ref == case_id
            && record.kind == RecordKind::SubjectBinding
            && !record.subject_ref.is_empty()
            && record.subject_ref != "subject:none"
    }) {
        if !seen.insert(record.subject_ref.clone()) {
            continue;
        }
        let mut binding = PendingTransition::new(
            format!(
                "transition:participant-bound:{}:{}",
                canonical_id_component(case_id),
                canonical_id_component(&record.subject_ref)
            ),
            case_id,
            state.generation,
            provider_source(Some(&record.subject_ref), &record.id),
            TransitionPayload::ParticipantBound {
                participant_id: record.subject_ref.clone(),
                role: "legacy_subject_binding".to_string(),
            },
        );
        binding.provenance.push(legacy_provenance(
            journal_path,
            Some(&record.id),
            "lossless_structural_promotion",
        ));
        state = store.commit_transition(binding)?.state;
    }
    Ok(state)
}

fn promote_provider_compatibility_state(
    store: &LmdbRecordStore,
    journal: &Journal,
    journal_path: &Path,
    case_id: &str,
    participant_id: &str,
) -> Result<yai_core_engine::transition::CaseState, String> {
    let mut state = ensure_canonical_case(store, journal, journal_path, case_id)?;
    let participant_bound = state
        .participants
        .iter()
        .any(|participant| participant.participant_id == participant_id);
    if !participant_bound {
        return Err(format!(
            "canonical participant {participant_id} is not bound to {case_id}"
        ));
    }

    let admitted = state
        .participants
        .iter()
        .find(|participant| participant.participant_id == participant_id)
        .map(|participant| !participant.admitted_views.is_empty())
        .unwrap_or(false);
    if !admitted {
        if let Some(record) = journal.records().iter().find(|record| {
            record.case_ref == case_id
                && record.kind == RecordKind::SubjectState
                && record.subject_ref == participant_id
                && legacy_summary_is(record, "case_entry", "admitted")
        }) {
            let consumer =
                legacy_summary_value(record, "consumer").unwrap_or_else(|| "model".to_string());
            let view_kind =
                legacy_summary_value(record, "kind").unwrap_or_else(|| "model_context".to_string());
            let mut admission = PendingTransition::new(
                format!("transition:{}", canonical_id_component(&record.id)),
                case_id,
                state.generation,
                provider_source(Some(participant_id), &record.id),
                TransitionPayload::ParticipantAdmitted {
                    participant_id: participant_id.to_string(),
                    consumer,
                    view_kind,
                },
            );
            admission.provenance.push(legacy_provenance(
                journal_path,
                Some(&record.id),
                "compatibility_summary_promotion",
            ));
            state = store.commit_transition(admission)?.state;
        }
    }

    if state.provider.is_none() {
        if let Some(record) = journal.records().iter().find(|record| {
            record.case_ref == case_id
                && record.kind == RecordKind::SubjectState
                && record.subject_ref == participant_id
                && legacy_summary_is(record, "provider_attachment", "attached")
        }) {
            let mut attached = PendingTransition::new(
                format!("transition:{}", canonical_id_component(&record.id)),
                case_id,
                state.generation,
                provider_source(Some(participant_id), &record.id),
                TransitionPayload::ProviderAttached {
                    participant_id: participant_id.to_string(),
                    provider_id: legacy_summary_value(record, "provider_id")
                        .unwrap_or_else(|| "provider:openai-compatible".to_string()),
                    provider_kind: legacy_summary_value(record, "provider")
                        .unwrap_or_else(|| "openai_compatible".to_string()),
                    base_url: legacy_summary_value(record, "base_url").unwrap_or_default(),
                    model_id: legacy_summary_value(record, "model").unwrap_or_default(),
                    credential_ref: format!(
                        "env:{}",
                        legacy_summary_value(record, "api_key_env")
                            .unwrap_or_else(|| "OPENCODE_LLM_API_KEY".to_string())
                    ),
                },
            );
            attached.provenance.push(legacy_provenance(
                journal_path,
                Some(&record.id),
                "compatibility_summary_promotion",
            ));
            state = store.commit_transition(attached)?.state;
        }
    }
    Ok(state)
}

fn compact_text(value: &str, max_chars: usize) -> String {
    let compact = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut result = String::new();
    for ch in compact.chars().take(max_chars) {
        result.push(ch);
    }
    if compact.chars().count() > max_chars {
        result.push_str("...");
    }
    result
}

fn print_provider_attach_shell(
    case_ref: &str,
    subject_ref: &str,
    provider_id: &str,
    base_url: &str,
    model: &str,
    api_key_env: &str,
    status: &str,
) {
    println!(
        "printf '%s\\n' {}",
        shell_quote("provider_attachment: accepted")
    );
    println!(
        "printf '%s\\n' {}",
        shell_quote(&format!("provider_attachment_status: {status}"))
    );
    println!(
        "printf '%s\\n' {}",
        shell_quote(&format!("case_ref: {case_ref}"))
    );
    println!(
        "printf '%s\\n' {}",
        shell_quote(&format!("subject_ref: {subject_ref}"))
    );
    println!(
        "printf '%s\\n' {}",
        shell_quote(&format!("provider_id: {provider_id}"))
    );
    println!(
        "printf '%s\\n' {}",
        shell_quote(&format!("provider_base_url: {base_url}"))
    );
    println!(
        "printf '%s\\n' {}",
        shell_quote(&format!("provider_model: {model}"))
    );
    println!("printf '%s\\n' {}", shell_quote("case_session: active"));
    println!("printf '%s\\n' {}", shell_quote("case_context: active"));
    println!(
        "printf '%s\\n' {}",
        shell_quote("authority_scope: model interpretation_only")
    );
    println!("export YAI_PROVIDER_BASE_URL={}", shell_quote(base_url));
    println!("export YAI_PROVIDER_ID={}", shell_quote(provider_id));
    println!("export YAI_PROVIDER_MODEL={}", shell_quote(model));
    println!(
        "export YAI_PROVIDER_SUBJECT_REF={}",
        shell_quote(subject_ref)
    );
    println!(
        "export YAI_PROVIDER_API_KEY_ENV={}",
        shell_quote(api_key_env)
    );
}

pub(super) fn case_attach_provider(args: &[String]) -> Result<(), String> {
    let path = case_journal_path(args, "yai case attach-provider")?;
    let case_ref = named_arg(args, "--case")?;
    let subject_ref = named_arg(args, "--subject")?;
    let base_url = named_arg(args, "--base-url")?;
    let model = named_arg(args, "--model")?;
    let provider_id = optional_arg(args, "--provider-id")
        .unwrap_or_else(|| "provider:openai-compatible".to_string());
    let api_key_env =
        optional_arg(args, "--api-key-env").unwrap_or_else(|| "OPENCODE_LLM_API_KEY".to_string());
    let shell = optional_arg(args, "--shell");
    let journal = Journal::load_jsonl(&path)
        .map_err(|error| format!("failed to load {}: {error}", path.display()))?;

    let subject_bound = journal.records().iter().any(|record| {
        record.case_ref == case_ref
            && record.kind == RecordKind::SubjectBinding
            && record.subject_ref == subject_ref
    });
    if !subject_bound {
        return Err(format!(
            "subject {subject_ref} is not bound to case {case_ref}"
        ));
    }

    let provider_summary = format!(
        "provider_attachment:attached provider_id:{provider_id} provider:openai_compatible base_url:{base_url} model:{model} api_key_env:{api_key_env} prompt_surface:vendored_linenoise context:typed_projection_context_frame"
    );
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = ensure_canonical_case(&store, &journal, &path, &case_ref)?;
    let already_attached = state.provider.as_ref().is_some_and(|provider| {
        provider.participant_id == subject_ref
            && (provider.provider_id.is_empty() || provider.provider_id == provider_id)
            && provider.provider_kind == "openai_compatible"
            && provider.base_url == base_url
            && provider.model_id == model
            && provider.credential_ref == format!("env:{api_key_env}")
    });
    let legacy_already_attached = journal.records().iter().any(|record| {
        record.case_ref == case_ref
            && record.kind == RecordKind::SubjectState
            && record.subject_ref == subject_ref
            && record.summary == provider_summary
    });

    let record = Record::from_parts(
        format!(
            "provider-attachment:{}:{}",
            subject_ref.replace(':', "-"),
            journal.count() + 1
        ),
        &case_ref,
        RecordKind::SubjectState,
        &subject_ref,
        "",
        "",
        "",
        provider_summary,
    );
    if !already_attached {
        let mut attached = PendingTransition::new(
            format!("transition:{}", canonical_id_component(&record.id)),
            &case_ref,
            state.generation,
            provider_source(Some(&subject_ref), &record.id),
            TransitionPayload::ProviderAttached {
                participant_id: subject_ref.clone(),
                provider_id: provider_id.clone(),
                provider_kind: "openai_compatible".to_string(),
                base_url: base_url.clone(),
                model_id: model.clone(),
                credential_ref: format!("env:{api_key_env}"),
            },
        );
        attached.summary = Some("OpenAI-compatible provider attachment".to_string());
        store.commit_transition(attached)?;
    }
    if !legacy_already_attached {
        append_record_to_journal(&path, &record)?;
    }

    let status = if already_attached {
        "already_attached"
    } else {
        "attached"
    };
    if let Some(shell) = shell.as_deref() {
        if shell != "zsh" && shell != "sh" {
            return Err(format!("unsupported shell: {shell}"));
        }
        print_provider_attach_shell(
            &case_ref,
            &subject_ref,
            &provider_id,
            &base_url,
            &model,
            &api_key_env,
            status,
        );
        return Ok(());
    }

    println!("provider_attachment: accepted");
    println!("provider_attachment_status: {status}");
    println!("case_ref: {case_ref}");
    println!("subject_ref: {subject_ref}");
    println!("provider_id: {provider_id}");
    println!("case_session: active");
    println!("case_context: active");
    println!("authority_scope: model interpretation_only");
    println!("provider_base_url: {base_url}");
    println!("provider_model: {model}");
    println!("api_key_env: {api_key_env}");
    Ok(())
}

struct ProviderConfig {
    provider_id: String,
    base_url: String,
    model: String,
    api_key: Option<String>,
    language_mode: String,
    continuation_supported: bool,
    continuation_ref: Option<ProviderContinuationReference>,
}

struct PromptRuntime {
    journal_path: PathBuf,
    case_ref: String,
    subject_ref: String,
    provider: ProviderConfig,
    active_thread_id: String,
    legacy_status_notes: String,
    transcript_enabled: bool,
}

const DEFAULT_THREAD_ID: &str = "thread:default";

pub(super) fn env_var(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| env_file_var(name))
}

fn color_enabled() -> bool {
    std::io::stdout().is_terminal()
        && env_var("NO_COLOR").is_none()
        && env_var("YAI_NO_COLOR").is_none()
        && env_var("TERM").as_deref() != Some("dumb")
}

fn paint(enabled: bool, code: &str, value: &str) -> String {
    if enabled {
        format!("{code}{value}{ANSI_RESET}")
    } else {
        value.to_string()
    }
}

fn transcript_retention_enabled(journal: &Journal, case_ref: &str, subject_ref: &str) -> bool {
    let mut enabled = false;
    for record in journal.records().iter().filter(|record| {
        record.case_ref == case_ref
            && record.kind == RecordKind::SubjectState
            && record.subject_ref == subject_ref
            && legacy_summary_value(record, "prompt_transcript_retention").is_some()
    }) {
        match legacy_summary_value(record, "prompt_transcript_retention").as_deref() {
            Some("enabled") => enabled = true,
            Some("disabled") => enabled = false,
            _ => {}
        }
    }
    enabled
}

fn transcript_retention_label(enabled: bool) -> &'static str {
    if enabled {
        "full_redacted_case_local"
    } else {
        "preview_only"
    }
}

fn summary_token(summary: &str, key: &str) -> Option<String> {
    parse_legacy_summary_fields(summary).remove(key)
}

fn active_thread_id(journal: &Journal, case_ref: &str) -> Option<String> {
    let mut active = None;
    for record in journal.records().iter().filter(|record| {
        record.case_ref == case_ref && record.kind == RecordKind::InteractionThread
    }) {
        if legacy_summary_is(record, "state", "active") {
            if let Some(thread_id) = summary_token(&record.summary, "thread_id") {
                active = Some(thread_id);
            }
        }
    }
    active
}

fn thread_turn_count(journal: &Journal, case_ref: &str, thread_id: &str) -> usize {
    journal
        .records()
        .iter()
        .filter(|record| {
            record.case_ref == case_ref
                && record.kind == RecordKind::InteractionTurn
                && legacy_summary_is(record, "thread_id", thread_id)
        })
        .count()
}

fn append_thread_record(
    journal_path: &PathBuf,
    case_ref: &str,
    subject_ref: &str,
    thread_id: &str,
    label: &str,
    state: &str,
) -> Result<(), String> {
    let journal = Journal::load_jsonl(journal_path)
        .map_err(|error| format!("failed to load {}: {error}", journal_path.display()))?;
    let sequence = journal.count() + 1;
    let summary = format!(
        "interaction_thread:{thread_id} thread_id:{thread_id} state:{state} label:{} journal_role:replay_audit active_context:thread_plus_projection",
        compact_text(label, 48)
    );
    let record = Record::from_parts(
        format!(
            "interaction-thread:{}:{sequence}",
            thread_id.replace(':', "-")
        ),
        case_ref,
        RecordKind::InteractionThread,
        subject_ref,
        "",
        "",
        "",
        summary,
    );
    append_record_to_journal(journal_path, &record)
}

fn ensure_default_thread(
    journal_path: &PathBuf,
    journal: &Journal,
    case_ref: &str,
    subject_ref: &str,
) -> Result<String, String> {
    if let Some(thread_id) = active_thread_id(journal, case_ref) {
        return Ok(thread_id);
    }
    append_thread_record(
        journal_path,
        case_ref,
        subject_ref,
        DEFAULT_THREAD_ID,
        "default",
        "active",
    )?;
    Ok(DEFAULT_THREAD_ID.to_string())
}

fn render_thread_context(journal: &Journal, case_ref: &str, thread_id: &str) -> String {
    let mut output = String::new();
    let mut count = 0usize;
    let _ = writeln!(output, "## Active Interaction Thread");
    let _ = writeln!(output, "interaction_thread: {thread_id}");
    let _ = writeln!(output, "journal_role: replay_audit_not_chat_memory");
    for record in journal.records().iter().filter(|record| {
        record.case_ref == case_ref
            && record.kind == RecordKind::InteractionTurn
            && legacy_summary_is(record, "thread_id", thread_id)
    }) {
        count += 1;
        let _ = writeln!(
            output,
            "- kind:interaction_turn record_id:{} summary:{}",
            record.id, record.summary
        );
    }
    let _ = writeln!(output, "included_turn_count: {count}");
    if count == 0 {
        let _ = writeln!(output, "thread_state: empty");
    }
    output
}

fn prompt_runtime_from_args(args: &[String]) -> Result<PromptRuntime, String> {
    let journal_path = case_journal_path(args, "yai prompt")?;
    let case_ref = optional_arg(args, "--case")
        .or_else(|| env_var("YAI_CASE_REF"))
        .ok_or_else(|| "YAI_CASE_REF is required; run `yai case enter` first".to_string())?;
    let subject_ref = optional_arg(args, "--subject")
        .or_else(|| env_var("YAI_PROVIDER_SUBJECT_REF"))
        .or_else(|| env_var("YAI_SUBJECT_REF"))
        .unwrap_or_else(|| "subject:llm-provider".to_string());
    let base_url = optional_arg(args, "--base-url")
        .or_else(|| env_var("YAI_PROVIDER_BASE_URL"))
        .or_else(|| env_var("YAI_LLM_BASE_URL"))
        .ok_or_else(|| {
            "provider base URL missing; run `yai case attach-provider` or export YAI_PROVIDER_BASE_URL"
                .to_string()
        })?;
    let model = optional_arg(args, "--model")
        .or_else(|| env_var("YAI_PROVIDER_MODEL"))
        .or_else(|| env_var("YAI_LLM_MODEL"))
        .ok_or_else(|| {
            "provider model missing; run `yai case attach-provider` or export YAI_PROVIDER_MODEL"
                .to_string()
        })?;
    let provider_id = optional_arg(args, "--provider-id")
        .or_else(|| env_var("YAI_PROVIDER_ID"))
        .unwrap_or_else(|| "provider:openai-compatible".to_string());
    let api_key_env = optional_arg(args, "--api-key-env")
        .or_else(|| env_var("YAI_PROVIDER_API_KEY_ENV"))
        .unwrap_or_else(|| "OPENCODE_LLM_API_KEY".to_string());
    let api_key = env_var("YAI_PROVIDER_API_KEY")
        .or_else(|| env_var(&api_key_env))
        .or_else(|| env_var("OPENCODE_LLM_API_KEY"));
    let language_mode = optional_arg(args, "--language-mode")
        .or_else(|| env_var("YAI_PROVIDER_LANGUAGE_MODE"))
        .unwrap_or_else(|| "none".to_string());
    if language_mode != "none" && language_mode != "auto" {
        return Err("--language-mode must be auto or none".to_string());
    }
    let continuation_supported = args.iter().any(|value| value == "--continuation-capable")
        || env_var("YAI_PROVIDER_CONTINUATION_CAPABLE").as_deref() == Some("1");
    let continuation_ref = optional_arg(args, "--continuation-ref")
        .or_else(|| env_var("YAI_PROVIDER_CONTINUATION_REF"))
        .map(|opaque_reference| ProviderContinuationReference {
            provider_id: provider_id.clone(),
            runtime_id: optional_arg(args, "--provider-runtime-id")
                .or_else(|| env_var("YAI_PROVIDER_RUNTIME_ID"))
                .unwrap_or_else(|| "runtime:unspecified".to_string()),
            opaque_reference,
        });
    let journal = Journal::load_jsonl(&journal_path)
        .map_err(|error| format!("failed to load {}: {error}", journal_path.display()))?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = promote_provider_compatibility_state(
        &store,
        &journal,
        &journal_path,
        &case_ref,
        &subject_ref,
    )?;
    let admitted = state
        .participants
        .iter()
        .find(|participant| participant.participant_id == subject_ref)
        .map(|participant| !participant.admitted_views.is_empty())
        .unwrap_or(false);
    if !admitted {
        return Err(format!(
            "{subject_ref} has not entered {case_ref}; run `yai case enter` first"
        ));
    }
    let attached = state.provider.as_ref().is_some_and(|provider| {
        provider.participant_id == subject_ref
            && (provider.provider_id.is_empty() || provider.provider_id == provider_id)
            && provider.provider_kind == "openai_compatible"
            && provider.model_id == model
    });
    if !attached {
        return Err(format!(
            "{subject_ref} has no provider attachment in {case_ref}; run `yai case attach-provider` first"
        ));
    }
    let transcript_enabled = transcript_retention_enabled(&journal, &case_ref, &subject_ref);
    let active_thread_id = ensure_default_thread(&journal_path, &journal, &case_ref, &subject_ref)?;
    let journal = Journal::load_jsonl(&journal_path)
        .map_err(|error| format!("failed to load {}: {error}", journal_path.display()))?;

    Ok(PromptRuntime {
        journal_path,
        case_ref: case_ref.clone(),
        subject_ref,
        provider: ProviderConfig {
            provider_id,
            base_url,
            model,
            api_key,
            language_mode,
            continuation_supported,
            continuation_ref,
        },
        active_thread_id: active_thread_id.clone(),
        legacy_status_notes: render_thread_context(&journal, &case_ref, &active_thread_id),
        transcript_enabled,
    })
}

fn linenoise_read_line(prompt: &str) -> Result<Option<String>, String> {
    let prompt = CString::new(prompt).map_err(|_| "prompt contains a NUL byte".to_string())?;
    let ptr = unsafe { linenoise(prompt.as_ptr()) };
    if ptr.is_null() {
        return Ok(None);
    }
    let line = unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe {
        linenoiseFree(ptr.cast::<c_void>());
    }
    Ok(Some(line))
}

fn prompt_label(case_ref: &str, colors: bool) -> String {
    if colors {
        format!(
            "{}{}{}({}{}{})> ",
            ANSI_BOLD, ANSI_CYAN, "yai", ANSI_YELLOW, case_ref, ANSI_RESET
        )
    } else {
        format!("yai({case_ref})> ")
    }
}

fn terminal_width() -> usize {
    env_var("COLUMNS")
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|width| *width >= 50)
        .map(|width| width.min(120))
        .unwrap_or(92)
}

fn print_cli_section(colors: bool, label: &str, detail: &str, color: &str) {
    let width = terminal_width();
    let title = if detail.is_empty() {
        label.to_string()
    } else {
        format!("{label} {detail}")
    };
    let plain = format!("-- {title} ");
    let line = if plain.len() >= width {
        plain
    } else {
        format!("{}{}", plain, "-".repeat(width - plain.len()))
    };
    println!("{}", paint(colors, color, &line));
}

fn wrap_words(text: &str, indent: &str, width: usize) -> Vec<String> {
    let available = width.saturating_sub(indent.len()).max(24);
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() > available {
            lines.push(format!("{indent}{current}"));
            current.clear();
            current.push_str(word);
        } else {
            current.push(' ');
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(format!("{indent}{current}"));
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn ordered_list_marker(line: &str) -> Option<usize> {
    let (prefix, rest) = line.split_once(". ")?;
    if prefix.is_empty() || prefix.chars().any(|ch| !ch.is_ascii_digit()) || rest.is_empty() {
        return None;
    }
    Some(prefix.len() + 2)
}

fn print_wrapped_text(colors: bool, line: &str, width: usize) {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        println!();
        return;
    }

    if trimmed.starts_with("```") {
        println!("{}", paint(colors, ANSI_DIM, trimmed));
        return;
    }

    if let Some(title) = trimmed.strip_prefix("### ") {
        println!();
        println!("{}", paint(colors, ANSI_BOLD, title));
        return;
    }
    if let Some(title) = trimmed.strip_prefix("## ") {
        println!();
        println!("{}", paint(colors, ANSI_BOLD, title));
        return;
    }
    if let Some(title) = trimmed.strip_prefix("# ") {
        println!();
        println!("{}", paint(colors, ANSI_BOLD, title));
        return;
    }

    if let Some(rest) = trimmed
        .strip_prefix("* ")
        .or_else(|| trimmed.strip_prefix("- "))
    {
        let bullet = paint(colors, ANSI_GREEN, "-");
        let first_indent = format!("  {bullet} ");
        let next_indent = "    ";
        let wrapped = wrap_words(rest, "", width.saturating_sub(4));
        for (index, item) in wrapped.iter().enumerate() {
            if index == 0 {
                println!("{first_indent}{item}");
            } else {
                println!("{next_indent}{item}");
            }
        }
        return;
    }

    if let Some(marker_len) = ordered_list_marker(trimmed) {
        let marker = &trimmed[..marker_len];
        let rest = &trimmed[marker_len..];
        let marker = paint(colors, ANSI_GREEN, marker.trim_end());
        let first_indent = format!("  {marker} ");
        let next_indent = "    ";
        let wrapped = wrap_words(rest, "", width.saturating_sub(4));
        for (index, item) in wrapped.iter().enumerate() {
            if index == 0 {
                println!("{first_indent}{item}");
            } else {
                println!("{next_indent}{item}");
            }
        }
        return;
    }

    for item in wrap_words(trimmed, "  ", width) {
        println!("{item}");
    }
}

fn print_model_output(colors: bool, output: &str) {
    let width = terminal_width();
    for line in output.lines() {
        print_wrapped_text(colors, line, width);
    }
}

pub(super) fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => {
                let _ = write!(escaped, "\\u{:04x}", ch as u32);
            }
            ch => escaped.push(ch),
        }
    }
    escaped
}

fn parse_json_string_at(bytes: &[u8], mut index: usize) -> Option<(String, usize)> {
    if bytes.get(index).copied()? != b'"' {
        return None;
    }
    index += 1;
    let mut output = String::new();
    while index < bytes.len() {
        let byte = bytes[index];
        index += 1;
        match byte {
            b'"' => return Some((output, index)),
            b'\\' => {
                let escaped = *bytes.get(index)?;
                index += 1;
                match escaped {
                    b'"' => output.push('"'),
                    b'\\' => output.push('\\'),
                    b'/' => output.push('/'),
                    b'b' => output.push('\u{0008}'),
                    b'f' => output.push('\u{000c}'),
                    b'n' => output.push('\n'),
                    b'r' => output.push('\r'),
                    b't' => output.push('\t'),
                    b'u' => {
                        let hex = std::str::from_utf8(bytes.get(index..index + 4)?).ok()?;
                        let value = u16::from_str_radix(hex, 16).ok()?;
                        if let Some(ch) = char::from_u32(value as u32) {
                            output.push(ch);
                        }
                        index += 4;
                    }
                    other => output.push(other as char),
                }
            }
            other => output.push(other as char),
        }
    }
    None
}

pub(super) fn extract_json_string_field(source: &str, field: &str) -> Option<String> {
    let needle = format!("\"{field}\"");
    let bytes = source.as_bytes();
    let mut start = 0usize;
    while let Some(relative) = source.get(start..)?.find(&needle) {
        let mut index = start + relative + needle.len();
        while bytes.get(index).copied()?.is_ascii_whitespace() {
            index += 1;
        }
        if bytes.get(index).copied()? != b':' {
            start = index;
            continue;
        }
        index += 1;
        while bytes.get(index).copied()?.is_ascii_whitespace() {
            index += 1;
        }
        if let Some((value, _)) = parse_json_string_at(bytes, index) {
            return Some(value);
        }
        start = index;
    }
    None
}

struct HttpUrl {
    host: String,
    port: u16,
    path: String,
}

fn parse_http_url(url: &str) -> Result<HttpUrl, String> {
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| "only http:// provider URLs are supported in this carrier".to_string())?;
    let (authority, path) = rest
        .split_once('/')
        .map(|(authority, path)| (authority, format!("/{path}")))
        .unwrap_or((rest, "/".to_string()));
    let (host, port) = authority
        .rsplit_once(':')
        .map(|(host, port)| {
            port.parse::<u16>()
                .map(|port| (host.to_string(), port))
                .map_err(|error| format!("invalid provider port: {error}"))
        })
        .transpose()?
        .unwrap_or_else(|| (authority.to_string(), 80));
    Ok(HttpUrl { host, port, path })
}

fn decode_chunked_body(body: &[u8]) -> Result<Vec<u8>, String> {
    let mut index = 0usize;
    let mut decoded = Vec::new();
    loop {
        let Some(line_end) = body[index..].windows(2).position(|pair| pair == b"\r\n") else {
            return Err("invalid chunked response".to_string());
        };
        let size_line = std::str::from_utf8(&body[index..index + line_end])
            .map_err(|error| format!("invalid chunk header: {error}"))?;
        let size_text = size_line.split(';').next().unwrap_or(size_line).trim();
        let size = usize::from_str_radix(size_text, 16)
            .map_err(|error| format!("invalid chunk size: {error}"))?;
        index += line_end + 2;
        if size == 0 {
            break;
        }
        let chunk_end = index + size;
        if chunk_end + 2 > body.len() {
            return Err("truncated chunked response".to_string());
        }
        decoded.extend_from_slice(&body[index..chunk_end]);
        index = chunk_end + 2;
    }
    Ok(decoded)
}

struct ProviderTransportResult {
    output: String,
    continuation_disposition: ContinuationDisposition,
}

fn provider_http_request(
    config: &ProviderConfig,
    rendered: &RenderedInput,
    continuation: Option<&ProviderContinuationReference>,
) -> Result<(bool, String), String> {
    let url = parse_http_url(&config.base_url)?;
    if let Some(reference) = continuation {
        if reference.provider_id != config.provider_id {
            return Err("provider_continuation_provider_mismatch".to_string());
        }
        if !config.continuation_supported {
            return Err("provider_continuation_not_supported".to_string());
        }
    }
    let continuation_field = continuation
        .map(|reference| {
            format!(
                ",\"yai_provider_continuation\":{{\"runtime_id\":\"{}\",\"reference\":\"{}\"}}",
                json_escape(&reference.runtime_id),
                json_escape(&reference.opaque_reference)
            )
        })
        .unwrap_or_default();
    let body = format!(
        "{{\"model\":\"{}\",\"stream\":false,\"messages\":[{{\"role\":\"system\",\"content\":\"{}\"}},{{\"role\":\"user\",\"content\":\"{}\"}}]{} }}",
        json_escape(&config.model),
        json_escape(&rendered.system_content),
        json_escape(&rendered.user_content),
        continuation_field
    );
    let auth = config
        .api_key
        .as_deref()
        .map(|key| format!("Authorization: Bearer {key}\r\n"))
        .unwrap_or_default();
    let request = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nAccept: application/json\r\n{}Content-Length: {}\r\nConnection: close\r\n\r\n{}",
        url.path,
        url.host,
        auth,
        body.len(),
        body
    );
    let mut stream = TcpStream::connect((url.host.as_str(), url.port))
        .map_err(|error| format!("failed to connect provider: {error}"))?;
    stream
        .write_all(request.as_bytes())
        .map_err(|error| format!("failed to write provider request: {error}"))?;
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .map_err(|error| format!("failed to read provider response: {error}"))?;
    let split = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| "invalid HTTP provider response".to_string())?;
    let headers = String::from_utf8_lossy(&response[..split]).to_string();
    let body_bytes = &response[split + 4..];
    let success = headers.starts_with("HTTP/1.1 2") || headers.starts_with("HTTP/1.0 2");
    let lower_headers = headers.to_ascii_lowercase();
    let body_bytes = if lower_headers.contains("transfer-encoding: chunked") {
        decode_chunked_body(body_bytes)?
    } else {
        body_bytes.to_vec()
    };
    Ok((success, String::from_utf8_lossy(&body_bytes).to_string()))
}

fn provider_chat_completion(
    config: &ProviderConfig,
    rendered: &RenderedInput,
) -> Result<ProviderTransportResult, String> {
    let continuation = config.continuation_ref.as_ref();
    let (success, mut body_text) = provider_http_request(config, rendered, continuation)?;
    let disposition = if success {
        if continuation.is_some() {
            ContinuationDisposition::Used
        } else {
            ContinuationDisposition::NotProvided
        }
    } else if continuation.is_some()
        && body_text
            .to_ascii_lowercase()
            .contains("invalid_continuation")
    {
        let (retry_success, retry_body) = provider_http_request(config, rendered, None)?;
        if !retry_success {
            return Err(format!(
                "provider continuation retry returned non-2xx response: {}",
                compact_text(&retry_body, 240)
            ));
        }
        body_text = retry_body;
        ContinuationDisposition::InvalidatedAndRetried
    } else {
        return Err(format!(
            "provider returned non-2xx response: {}",
            compact_text(&body_text, 240)
        ));
    };
    let output = extract_json_string_field(&body_text, "content").ok_or_else(|| {
        format!(
            "provider response did not contain message content: {}",
            compact_text(&body_text, 240)
        )
    })?;
    Ok(ProviderTransportResult {
        output,
        continuation_disposition: disposition,
    })
}

fn redact_sensitive(value: &str, session: &PromptRuntime) -> String {
    if let Some(api_key) = session
        .provider
        .api_key
        .as_deref()
        .filter(|api_key| !api_key.is_empty())
    {
        value.replace(api_key, "[redacted:api_key]")
    } else {
        value.to_string()
    }
}

fn transcript_text(value: &str, session: &PromptRuntime) -> String {
    redact_sensitive(value, session)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

struct ProviderInvocationRefs {
    attempt_id: String,
    invocation_id: String,
}

struct SemanticInvocation {
    projection: Projection,
    frame: ContextFrame,
    rendered: RenderedInput,
    output_contract_id: String,
}

fn continuation_disposition_label(value: &ContinuationDisposition) -> &'static str {
    match value {
        ContinuationDisposition::NotProvided => "not_provided",
        ContinuationDisposition::Used => "used",
        ContinuationDisposition::InvalidatedAndRetried => "invalidated_and_retried",
    }
}

fn compile_semantic_invocation(
    session: &PromptRuntime,
    purpose: ProjectionPurpose,
    task: &str,
    output_contract: InvocationOutputContract,
) -> Result<SemanticInvocation, String> {
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store
        .get_case_state(&session.case_ref)?
        .ok_or_else(|| format!("canonical CaseState missing for {}", session.case_ref))?;
    let transitions = store.list_case_transitions(&session.case_ref)?;
    let request = ProjectionRequest::model(&session.subject_ref, purpose);
    let resource_refs = match &output_contract {
        InvocationOutputContract::FilesystemWriteProposal { attachment_id, .. } => {
            vec![attachment_id.clone()]
        }
        InvocationOutputContract::NaturalLanguage => Vec::new(),
    };
    let memory_entries = match store.operational_memory_manifest(&session.case_ref) {
        Ok(Some(manifest)) if manifest.is_current(&session.case_ref, state.generation) => {
            match store.list_operational_memory(&session.case_ref) {
                Ok(entries) => Some(entries),
                Err(error) => {
                    eprintln!("warning: derived operational memory unavailable: {error}");
                    None
                }
            }
        }
        Ok(_) => match derive_operational_memory(&session.case_ref, &transitions) {
            Ok(build) => {
                if let Err(error) = store.replace_case_operational_memory(&build) {
                    eprintln!("warning: derived operational memory was not persisted: {error}");
                }
                Some(build.entries)
            }
            Err(error) => {
                eprintln!("warning: operational memory derivation failed; using canonical fallback: {error}");
                None
            }
        },
        Err(error) => {
            eprintln!(
                "warning: operational memory store unavailable; using canonical fallback: {error}"
            );
            None
        }
    };
    let retrieval = memory_entries.as_ref().and_then(|entries| {
        match retrieve_operational_memory(
            &state,
            entries,
            RetrievalQualification {
                case_id: session.case_ref.clone(),
                participant_id: session.subject_ref.clone(),
                consumer: request.consumer.clone(),
                view_kind: request.view_kind.clone(),
                purpose: request.purpose.clone(),
                case_generation: state.generation,
                resource_refs,
                semantic_kinds: Vec::new(),
                causal_refs: Vec::new(),
                max_results: DEFAULT_RETRIEVAL_LIMIT,
                include_superseded: false,
            },
        ) {
            Ok(retrieval) => Some(retrieval),
            Err(error) => {
                eprintln!(
                    "warning: qualified memory retrieval failed; using canonical fallback: {error}"
                );
                None
            }
        }
    });
    let derived = retrieval
        .as_ref()
        .map(|retrieval| DerivedProjectionInput {
            graph_available: false,
            memory_available: true,
            memory: retrieval
                .selected
                .iter()
                .map(|item| yai_core_engine::context::DerivedMemoryInput {
                    memory_ref: item.memory.memory_id.clone(),
                    semantic_kind: item.memory.semantic_kind.as_str().to_string(),
                    memory_posture: item.memory.posture.as_str().to_string(),
                    description: item.memory.description.clone(),
                    lifecycle: item.memory.lifecycle.as_str().to_string(),
                    score: item.score,
                    ranking_reasons: item.ranking_reasons.clone(),
                    transition_refs: item.memory.provenance.transition_ids.clone(),
                    observation_refs: item.memory.provenance.observation_ids.clone(),
                    receipt_refs: item.memory.provenance.effect_receipt_ids.clone(),
                })
                .collect(),
            retrieval_id: Some(retrieval.retrieval_id.clone()),
            retrieval_candidates: retrieval.qualified_count,
            retrieval_omitted: retrieval.omitted_count,
        })
        .unwrap_or_default();
    let projection = compile_projection(&state, &transitions, &request, &derived)?;
    let output_contract_id = output_contract.contract_id();
    let frame = build_context_frame(&projection, task, output_contract)?;
    let profile = ProviderModelProfile {
        provider_id: session.provider.provider_id.clone(),
        provider_kind: "openai_compatible".to_string(),
        model_id: session.provider.model.clone(),
        structured_output_supported: matches!(
            frame.output_contract,
            InvocationOutputContract::FilesystemWriteProposal { .. }
        ),
        continuation_supported: session.provider.continuation_supported,
    };
    let rendered = render_openai_compatible(&frame, &profile, &session.provider.language_mode)?;
    store
        .put_semantic_context_artifact(&SemanticContextArtifact::Projection(projection.clone()))?;
    store.put_semantic_context_artifact(&SemanticContextArtifact::ContextFrame(frame.clone()))?;
    store.put_semantic_context_artifact(&SemanticContextArtifact::RenderedInputMetadata(
        rendered.metadata.clone(),
    ))?;
    Ok(SemanticInvocation {
        projection,
        frame,
        rendered,
        output_contract_id,
    })
}

fn invocation_lineage(
    semantic: &SemanticInvocation,
    disposition: ContinuationDisposition,
) -> ProviderInvocationLineage {
    ProviderInvocationLineage {
        projection_id: semantic.projection.projection_id.clone(),
        context_frame_id: semantic.frame.frame_id.clone(),
        case_generation: semantic.projection.case_generation,
        rendered_input_id: semantic.rendered.metadata.rendered_input_id.clone(),
        rendered_input_digest: semantic.rendered.metadata.content_digest.clone(),
        output_contract_id: semantic.output_contract_id.clone(),
        continuation_disposition: continuation_disposition_label(&disposition).to_string(),
    }
}

/// Result of one provider invocation made through the real case/provider
/// boundary for a controlled effect turn. The raw output remains
/// non-authoritative candidate material until normalization succeeds.
pub(super) struct ControlledProviderResult {
    pub invocation_id: String,
    pub result_id: String,
    pub raw_output: String,
    pub provider_id: String,
    pub model_id: String,
    pub projection_id: String,
    pub context_frame_id: String,
}

pub(super) fn invoke_controlled_provider(
    args: &[String],
    purpose: ProjectionPurpose,
    task: &str,
    output_contract: InvocationOutputContract,
) -> Result<ControlledProviderResult, String> {
    let session = prompt_runtime_from_args(args)?;
    let semantic = compile_semantic_invocation(&session, purpose, task, output_contract)?;
    let requested_disposition = if session.provider.continuation_ref.is_some() {
        ContinuationDisposition::Used
    } else {
        ContinuationDisposition::NotProvided
    };
    let invocation = append_model_prompt_attempt(
        &session,
        task,
        invocation_lineage(&semantic, requested_disposition),
    )?;
    let transport = provider_chat_completion(&session.provider, &semantic.rendered)?;
    let result_lineage = invocation_lineage(&semantic, transport.continuation_disposition.clone());
    let result_id = append_model_output_receipt(
        &session,
        &invocation.attempt_id,
        &invocation.invocation_id,
        &transport.output,
        result_lineage,
    )?;
    append_model_interpretation_record(
        &session,
        &invocation.attempt_id,
        &result_id,
        &transport.output,
    )?;
    append_interaction_turn(
        &session,
        &invocation.attempt_id,
        &invocation.invocation_id,
        &result_id,
        task,
        &transport.output,
    )?;
    Ok(ControlledProviderResult {
        invocation_id: invocation.invocation_id,
        result_id,
        raw_output: transport.output,
        provider_id: session.provider.provider_id,
        model_id: session.provider.model,
        projection_id: semantic.projection.projection_id,
        context_frame_id: semantic.frame.frame_id,
    })
}

fn append_model_prompt_attempt(
    session: &PromptRuntime,
    prompt: &str,
    semantic_lineage: ProviderInvocationLineage,
) -> Result<ProviderInvocationRefs, String> {
    let journal = Journal::load_jsonl(&session.journal_path)
        .map_err(|error| format!("failed to load {}: {error}", session.journal_path.display()))?;
    let sequence = journal.count() + 1;
    let attempt_id = format!("attempt:model-prompt-{sequence}");
    let invocation_id = format!("invocation:model-prompt-{sequence}");
    let record = Record::from_parts(
        format!(
            "model-prompt:{}:{sequence}",
            session.subject_ref.replace(':', "-")
        ),
        &session.case_ref,
        RecordKind::Attempt,
        &session.subject_ref,
        &attempt_id,
        "",
        "",
        prompt_attempt_summary(session, prompt),
    );
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store
        .get_case_state(&session.case_ref)?
        .ok_or_else(|| format!("canonical CaseState missing for {}", session.case_ref))?;
    if semantic_lineage.case_generation != state.generation {
        return Err(format!(
            "stale_context_frame: frame_generation={} current_generation={}",
            semantic_lineage.case_generation, state.generation
        ));
    }
    let mut pending = PendingTransition::new(
        format!("transition:{}", canonical_id_component(&record.id)),
        &session.case_ref,
        state.generation,
        provider_source(Some(&session.subject_ref), &record.id),
        TransitionPayload::ProviderInvocationStarted {
            invocation_id: invocation_id.clone(),
            participant_id: session.subject_ref.clone(),
            provider_id: session.provider.provider_id.clone(),
            provider_kind: "openai_compatible".to_string(),
            model_id: session.provider.model.clone(),
            semantic_lineage: Some(semantic_lineage.clone()),
        },
    );
    pending.summary = Some(prompt_attempt_summary(session, prompt));
    store.commit_transition(pending)?;
    append_record_to_journal(&session.journal_path, &record)?;
    Ok(ProviderInvocationRefs {
        attempt_id,
        invocation_id,
    })
}

fn append_model_output_receipt(
    session: &PromptRuntime,
    attempt_id: &str,
    invocation_id: &str,
    output: &str,
    semantic_lineage: ProviderInvocationLineage,
) -> Result<String, String> {
    let journal = Journal::load_jsonl(&session.journal_path)
        .map_err(|error| format!("failed to load {}: {error}", session.journal_path.display()))?;
    let sequence = journal.count() + 1;
    let receipt_id = format!("receipt:model-output-{sequence}");
    let result_id = format!("provider-result:model-output-{sequence}");
    let record = Record::from_parts(
        format!(
            "model-output:{}:{sequence}",
            session.subject_ref.replace(':', "-")
        ),
        &session.case_ref,
        RecordKind::EffectReceipt,
        &session.subject_ref,
        attempt_id,
        "",
        &receipt_id,
        model_output_summary(session, output),
    );
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store
        .get_case_state(&session.case_ref)?
        .ok_or_else(|| format!("canonical CaseState missing for {}", session.case_ref))?;
    let mut pending = PendingTransition::new(
        format!("transition:{}", canonical_id_component(&record.id)),
        &session.case_ref,
        state.generation,
        provider_source(Some(&session.subject_ref), &record.id),
        TransitionPayload::ProviderResultRecorded {
            result_id: result_id.clone(),
            invocation_id: invocation_id.to_string(),
            provider_id: session.provider.provider_id.clone(),
            provider_kind: "openai_compatible".to_string(),
            model_id: session.provider.model.clone(),
            semantic_lineage: Some(semantic_lineage),
            output: output.to_string(),
        },
    );
    pending.causal_refs.push(invocation_id.to_string());
    pending.summary = Some(model_output_summary(session, output));
    store.commit_transition(pending)?;
    append_record_to_journal(&session.journal_path, &record)?;
    Ok(result_id)
}

fn append_model_interpretation_record(
    session: &PromptRuntime,
    attempt_id: &str,
    result_id: &str,
    output: &str,
) -> Result<String, String> {
    let journal = Journal::load_jsonl(&session.journal_path)
        .map_err(|error| format!("failed to load {}: {error}", session.journal_path.display()))?;
    let sequence = journal.count() + 1;
    let summary = format!(
        "model_interpretation:observed source:provider_output authority:not_authoritative_state output_is:claim_or_proposal check_against:decisions_receipts_graph preview:{}",
        compact_text(output, 140)
    );
    let record = Record::from_parts(
        format!(
            "model-interpretation:{}:{sequence}",
            session.subject_ref.replace(':', "-")
        ),
        &session.case_ref,
        RecordKind::ModelInterpretation,
        &session.subject_ref,
        attempt_id,
        "",
        "",
        summary.clone(),
    );
    let interpretation_id = format!("interpretation:{}", canonical_id_component(&record.id));
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store
        .get_case_state(&session.case_ref)?
        .ok_or_else(|| format!("canonical CaseState missing for {}", session.case_ref))?;
    let mut pending = PendingTransition::new(
        format!("transition:{}", canonical_id_component(&record.id)),
        &session.case_ref,
        state.generation,
        provider_source(Some(&session.subject_ref), &record.id),
        TransitionPayload::ModelInterpretationRecorded {
            interpretation_id,
            result_id: result_id.to_string(),
            authority: InterpretationAuthority::NonAuthoritative,
        },
    );
    pending.causal_refs.push(result_id.to_string());
    pending.summary = Some(summary.clone());
    store.commit_transition(pending)?;
    append_record_to_journal(&session.journal_path, &record)?;
    Ok(summary)
}

fn append_interaction_turn(
    session: &PromptRuntime,
    attempt_id: &str,
    invocation_id: &str,
    result_id: &str,
    prompt: &str,
    output: &str,
) -> Result<String, String> {
    let journal = Journal::load_jsonl(&session.journal_path)
        .map_err(|error| format!("failed to load {}: {error}", session.journal_path.display()))?;
    let sequence = journal.count() + 1;
    let record_id = format!(
        "interaction-turn:{}:{sequence}",
        session.active_thread_id.replace(':', "-")
    );
    let summary = format!(
        "interaction_turn:{sequence} thread_id:{} attempt_id:{attempt_id} prompt_preview:{} output_preview:{} transcript_retention:{} lane:selected_thread audit:journal_retained",
        session.active_thread_id,
        compact_text(prompt, 100),
        compact_text(output, 120),
        transcript_retention_label(session.transcript_enabled)
    );
    let record = Record::from_parts(
        &record_id,
        &session.case_ref,
        RecordKind::InteractionTurn,
        &session.subject_ref,
        attempt_id,
        "",
        "",
        summary,
    );
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store
        .get_case_state(&session.case_ref)?
        .ok_or_else(|| format!("canonical CaseState missing for {}", session.case_ref))?;
    let turn_id = format!("turn:{}", canonical_id_component(&record_id));
    let mut pending = PendingTransition::new(
        format!("transition:{}", canonical_id_component(&record_id)),
        &session.case_ref,
        state.generation,
        provider_source(Some(&session.subject_ref), &record_id),
        TransitionPayload::InteractionTurnRecorded {
            turn_id,
            thread_id: session.active_thread_id.clone(),
            participant_id: session.subject_ref.clone(),
            invocation_id: invocation_id.to_string(),
            result_id: result_id.to_string(),
            operator_input: compact_text(prompt, 2048),
        },
    );
    pending.causal_refs = vec![invocation_id.to_string(), result_id.to_string()];
    store.commit_transition(pending)?;
    append_record_to_journal(&session.journal_path, &record)?;
    Ok(record_id)
}

fn prompt_attempt_summary(session: &PromptRuntime, prompt: &str) -> String {
    if session.transcript_enabled {
        format!(
            "op:model.prompt.submit prompt_surface:vendored_linenoise context:typed_context_frame thread_id:{} transcript_retention:full_redacted_case_local prompt_text:{}",
            session.active_thread_id,
            transcript_text(prompt, session)
        )
    } else {
        format!(
            "op:model.prompt.submit prompt_surface:vendored_linenoise context:typed_context_frame thread_id:{} transcript_retention:preview_only prompt_preview:{}",
            session.active_thread_id,
            compact_text(prompt, 120)
        )
    }
}

fn model_output_summary(session: &PromptRuntime, output: &str) -> String {
    if session.transcript_enabled {
        format!(
            "model.output status:observed provider:openai_compatible model:{} output_chars:{} transcript_retention:full_redacted_case_local output_text:{}",
            session.provider.model,
            output.chars().count(),
            transcript_text(output, session)
        )
    } else {
        format!(
            "model.output status:observed provider:openai_compatible model:{} output_chars:{} transcript_retention:preview_only output_preview:{}",
            session.provider.model,
            output.chars().count(),
            compact_text(output, 160)
        )
    }
}

fn append_transcript_retention_state(
    session: &PromptRuntime,
    enabled: bool,
) -> Result<String, String> {
    let journal = Journal::load_jsonl(&session.journal_path)
        .map_err(|error| format!("failed to load {}: {error}", session.journal_path.display()))?;
    let sequence = journal.count() + 1;
    let state = if enabled { "enabled" } else { "disabled" };
    let full_transcript = if enabled { "on_explicit" } else { "off" };
    let summary = format!(
        "prompt_transcript_retention:{state} scope:case_local redaction:secret_redacted prompt_preview:on provider_output_preview:on full_transcript:{full_transcript} memory_candidate:derived_not_raw_chat"
    );
    let record = Record::from_parts(
        format!(
            "prompt-retention:{}:{sequence}",
            session.subject_ref.replace(':', "-")
        ),
        &session.case_ref,
        RecordKind::SubjectState,
        &session.subject_ref,
        "",
        "",
        "",
        summary.clone(),
    );
    append_record_to_journal(&session.journal_path, &record)?;
    Ok(summary)
}

fn run_prompt_once(session: &mut PromptRuntime, prompt: &str, dry_run: bool) -> Result<(), String> {
    let colors = color_enabled();
    let freshness = projection_freshness_view(&session.case_ref, "model");
    let semantic = compile_semantic_invocation(
        session,
        ProjectionPurpose::Conversation,
        prompt,
        InvocationOutputContract::NaturalLanguage,
    )?;
    if dry_run {
        println!("model_prompt: dry_run");
        println!("case_ref: {}", session.case_ref);
        println!("case_session: active");
        println!("case_context: active");
        println!("interaction_thread: {}", session.active_thread_id);
        println!("projection_id: {}", semantic.projection.projection_id);
        println!("context_frame_id: {}", semantic.frame.frame_id);
        println!("context_frame_schema: {}", semantic.frame.schema);
        println!("projection_freshness: {}", freshness.freshness);
        println!("stale_reason: {}", freshness.stale_reason);
        println!("freshness_policy: {}", freshness.policy);
        println!("freshness_source: {}", freshness.source);
        if freshness.policy == "refresh_required" || freshness.policy == "blocked_for_model" {
            println!("refresh_required: true");
        }
        println!("subject_ref: {}", session.subject_ref);
        println!("provider_base_url: {}", session.provider.base_url);
        println!("provider_model: {}", session.provider.model);
        println!("context_source: typed_projection_plus_context_frame");
        println!(
            "transcript_retention: {}",
            transcript_retention_label(session.transcript_enabled)
        );
        println!("raw_journal_access: not_provided");
        println!("filesystem_access: not_provided");
        println!("decision_authority: not_provided");
        println!("receipt_authority: not_provided");
        println!("prompt_preview: {}", compact_text(prompt, 160));
        return Ok(());
    }

    if freshness.policy == "refresh_required" || freshness.policy == "blocked_for_model" {
        println!("projection: stale");
        println!("stale_reason: {}", freshness.stale_reason);
        println!("freshness_policy: {}", freshness.policy);
        println!("refresh_required: true");
    }
    let requested_disposition = if session.provider.continuation_ref.is_some() {
        ContinuationDisposition::Used
    } else {
        ContinuationDisposition::NotProvided
    };
    let invocation = append_model_prompt_attempt(
        session,
        prompt,
        invocation_lineage(&semantic, requested_disposition),
    )?;
    let transport = provider_chat_completion(&session.provider, &semantic.rendered)?;
    let output = transport.output;
    println!();
    print_cli_section(colors, "MODEL", &session.provider.model, ANSI_MAGENTA);
    print_model_output(colors, &output);
    println!();
    let result_id = append_model_output_receipt(
        session,
        &invocation.attempt_id,
        &invocation.invocation_id,
        &output,
        invocation_lineage(&semantic, transport.continuation_disposition.clone()),
    )?;
    let interpretation_summary =
        append_model_interpretation_record(session, &invocation.attempt_id, &result_id, &output)?;
    let turn_id = append_interaction_turn(
        session,
        &invocation.attempt_id,
        &invocation.invocation_id,
        &result_id,
        prompt,
        &output,
    )?;
    println!("projection_id: {}", semantic.projection.projection_id);
    println!("context_frame_id: {}", semantic.frame.frame_id);
    println!(
        "continuation_disposition: {}",
        continuation_disposition_label(&transport.continuation_disposition)
    );
    println!("interaction_turn: {turn_id}");
    println!(
        "model_interpretation: {}",
        compact_text(&interpretation_summary, 120)
    );
    Ok(())
}

fn handle_prompt_command(session: &mut PromptRuntime, command: &str) -> Result<bool, String> {
    if command == "/refresh" {
        let journal = Journal::load_jsonl(&session.journal_path).map_err(|error| {
            format!("failed to load {}: {error}", session.journal_path.display())
        })?;
        session.legacy_status_notes =
            render_thread_context(&journal, &session.case_ref, &session.active_thread_id);
        session.transcript_enabled =
            transcript_retention_enabled(&journal, &session.case_ref, &session.subject_ref);
        println!("case_prompt: refreshed");
        println!("semantic_context: rebuild_on_next_invocation");
        println!("interaction_thread: {}", session.active_thread_id);
        println!(
            "transcript_retention: {}",
            transcript_retention_label(session.transcript_enabled)
        );
        return Ok(true);
    }

    if command == "/transcript status" {
        println!(
            "transcript_retention: {}",
            transcript_retention_label(session.transcript_enabled)
        );
        println!(
            "prompt_transcript_retention: {}",
            if session.transcript_enabled {
                "enabled"
            } else {
                "disabled"
            }
        );
        println!("prompt_preview: on");
        println!("provider_output_preview: on");
        println!(
            "full_transcript: {}",
            if session.transcript_enabled {
                "on_explicit_redacted_case_local"
            } else {
                "off"
            }
        );
        println!("memory_candidate: derived_not_raw_chat");
        return Ok(true);
    }

    if command == "/thread status" {
        let journal = Journal::load_jsonl(&session.journal_path).map_err(|error| {
            format!("failed to load {}: {error}", session.journal_path.display())
        })?;
        println!("interaction_thread: {}", session.active_thread_id);
        println!(
            "thread_turn_count: {}",
            thread_turn_count(&journal, &session.case_ref, &session.active_thread_id)
        );
        println!("semantic_context: typed_projection_plus_active_thread_metadata");
        println!("journal_role: replay_audit_not_chat_memory");
        return Ok(true);
    }

    if command == "/thread list" {
        let journal = Journal::load_jsonl(&session.journal_path).map_err(|error| {
            format!("failed to load {}: {error}", session.journal_path.display())
        })?;
        let mut seen = Vec::<String>::new();
        for record in journal.records().iter().filter(|record| {
            record.case_ref == session.case_ref && record.kind == RecordKind::InteractionThread
        }) {
            if let Some(thread_id) = summary_token(&record.summary, "thread_id") {
                if !seen.iter().any(|value| value == &thread_id) {
                    println!(
                        "interaction_thread: {} turns:{}",
                        thread_id,
                        thread_turn_count(&journal, &session.case_ref, &thread_id)
                    );
                    seen.push(thread_id);
                }
            }
        }
        if seen.is_empty() {
            println!("interaction_thread: none");
        }
        return Ok(true);
    }

    if command == "/thread new" || command.starts_with("/thread new ") {
        let label = command
            .strip_prefix("/thread new")
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("thread");
        let journal = Journal::load_jsonl(&session.journal_path).map_err(|error| {
            format!("failed to load {}: {error}", session.journal_path.display())
        })?;
        let thread_id = format!("thread:{}", journal.count() + 1);
        append_thread_record(
            &session.journal_path,
            &session.case_ref,
            &session.subject_ref,
            &thread_id,
            label,
            "active",
        )?;
        let journal = Journal::load_jsonl(&session.journal_path).map_err(|error| {
            format!("failed to load {}: {error}", session.journal_path.display())
        })?;
        session.active_thread_id = thread_id.clone();
        session.legacy_status_notes =
            render_thread_context(&journal, &session.case_ref, &session.active_thread_id);
        println!("interaction_thread: new active");
        println!("thread_id: {thread_id}");
        println!("semantic_context: new_thread_projection_rebuild_required");
        println!("journal:audit retained");
        return Ok(true);
    }

    if let Some(thread_id) = command.strip_prefix("/thread use ").map(str::trim) {
        if thread_id.is_empty() {
            println!("thread_use: missing_thread_id");
            return Ok(true);
        }
        append_thread_record(
            &session.journal_path,
            &session.case_ref,
            &session.subject_ref,
            thread_id,
            thread_id,
            "active",
        )?;
        let journal = Journal::load_jsonl(&session.journal_path).map_err(|error| {
            format!("failed to load {}: {error}", session.journal_path.display())
        })?;
        session.active_thread_id = thread_id.to_string();
        session.legacy_status_notes =
            render_thread_context(&journal, &session.case_ref, &session.active_thread_id);
        println!("interaction_thread: restored previous");
        println!("thread_id: {thread_id}");
        println!(
            "thread_turn_count: {}",
            thread_turn_count(&journal, &session.case_ref, &session.active_thread_id)
        );
        return Ok(true);
    }

    if let Some(thread_id) = command.strip_prefix("/thread archive ").map(str::trim) {
        if thread_id.is_empty() {
            println!("thread_archive: missing_thread_id");
            return Ok(true);
        }
        append_thread_record(
            &session.journal_path,
            &session.case_ref,
            &session.subject_ref,
            thread_id,
            thread_id,
            "archived",
        )?;
        if session.active_thread_id == thread_id {
            session.active_thread_id = DEFAULT_THREAD_ID.to_string();
            append_thread_record(
                &session.journal_path,
                &session.case_ref,
                &session.subject_ref,
                DEFAULT_THREAD_ID,
                "default",
                "active",
            )?;
        }
        let journal = Journal::load_jsonl(&session.journal_path).map_err(|error| {
            format!("failed to load {}: {error}", session.journal_path.display())
        })?;
        session.legacy_status_notes =
            render_thread_context(&journal, &session.case_ref, &session.active_thread_id);
        println!("interaction_thread: archived");
        println!("thread_id: {thread_id}");
        println!("active_thread_id: {}", session.active_thread_id);
        return Ok(true);
    }

    if command == "/transcript on" {
        let summary = append_transcript_retention_state(session, true)?;
        session.transcript_enabled = true;
        let _ = writeln!(session.legacy_status_notes, "## Prompt Runtime State");
        let _ = writeln!(
            session.legacy_status_notes,
            "- kind:subject_state subject_ref:{} summary:{}",
            session.subject_ref, summary
        );
        let _ = writeln!(session.legacy_status_notes);
        println!("prompt_transcript_retention: enabled");
        println!("transcript_retention: full_redacted_case_local");
        println!("full_transcript: on_explicit_redacted_case_local");
        println!("redaction: secret_redacted");
        return Ok(true);
    }

    if command == "/transcript off" {
        let summary = append_transcript_retention_state(session, false)?;
        session.transcript_enabled = false;
        let _ = writeln!(session.legacy_status_notes, "## Prompt Runtime State");
        let _ = writeln!(
            session.legacy_status_notes,
            "- kind:subject_state subject_ref:{} summary:{}",
            session.subject_ref, summary
        );
        let _ = writeln!(session.legacy_status_notes);
        println!("prompt_transcript_retention: disabled");
        println!("transcript_retention: preview_only");
        println!("full_transcript: off");
        return Ok(true);
    }

    if command == "/memory propose" || command.starts_with("/memory propose ") {
        println!("memory_proposal: retired");
        println!("authority: compatibility_only");
        println!("replacement: canonical Transition-derived operational memory");
        println!("canonical_history_mutated: no");
        return Ok(true);
    }

    if command.starts_with('/') {
        println!("unknown_command: {command}");
        println!("commands: /thread status /thread new [label] /thread list /thread use <thread_id> /thread archive <thread_id> /refresh /transcript on /transcript off /transcript status /exit");
        return Ok(true);
    }

    Ok(false)
}

pub(super) fn prompt_repl(args: &[String]) -> Result<(), String> {
    let dry_run = args.iter().any(|arg| arg == "--dry-run");
    let once = optional_arg(args, "--once");
    let mut session = prompt_runtime_from_args(args)?;
    let colors = color_enabled();
    if let Some(prompt) = once {
        if handle_prompt_command(&mut session, prompt.trim())? {
            return Ok(());
        }
        return run_prompt_once(&mut session, &prompt, dry_run);
    }

    let mut stdin = std::io::stdin();
    if !stdin.is_terminal() {
        let mut prompt = String::new();
        stdin
            .read_to_string(&mut prompt)
            .map_err(|error| format!("failed to read prompt from stdin: {error}"))?;
        let prompt = prompt.trim();
        if prompt.is_empty() {
            return Err("prompt stdin is empty".to_string());
        }
        if handle_prompt_command(&mut session, prompt)? {
            return Ok(());
        }
        return run_prompt_once(&mut session, prompt, dry_run);
    }

    unsafe {
        let _ = linenoiseHistorySetMaxLen(200);
    }
    println!("case_prompt: entered");
    println!("case_ref: {}", session.case_ref);
    println!("case_session: active");
    println!("case_context: active");
    println!("interaction_thread: {}", session.active_thread_id);
    println!("subject_ref: {}", session.subject_ref);
    println!("provider_model: {}", session.provider.model);
    println!("context_source: typed_projection_plus_context_frame");
    println!(
        "transcript_retention: {}",
        transcript_retention_label(session.transcript_enabled)
    );
    println!("commands: /thread status /thread new [label] /thread list /thread use <thread_id> /thread archive <thread_id> /refresh /transcript on /transcript off /transcript status /exit");

    loop {
        println!();
        print_cli_section(colors, "QUESTION", &session.case_ref, ANSI_BLUE);
        let prompt = prompt_label(&session.case_ref, colors);
        let Some(line) = linenoise_read_line(&prompt)? else {
            break;
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "/exit" || trimmed == "/quit" {
            break;
        }
        if handle_prompt_command(&mut session, trimmed)? {
            continue;
        }
        if let Ok(history_line) = CString::new(trimmed) {
            unsafe {
                let _ = linenoiseHistoryAdd(history_line.as_ptr());
            }
        }
        if let Err(error) = run_prompt_once(&mut session, trimmed, dry_run) {
            eprintln!("{error}");
        }
    }
    Ok(())
}
