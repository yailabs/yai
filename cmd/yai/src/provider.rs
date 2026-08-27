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

fn render_model_context_records(
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

fn render_model_context_view(journal: &Journal, case_ref: Option<&str>) -> String {
    let projection = ProjectionSummary::from_journal("model", &journal);
    let case_ref = case_ref
        .or_else(|| (!projection.case_ref.is_empty()).then_some(projection.case_ref.as_str()));

    let mut output = String::new();
    let _ = writeln!(output, "case_ref: {}", case_ref.unwrap_or("unknown"));
    let _ = writeln!(output, "case_world: materialized");
    let _ = writeln!(output, "case_context: active");
    let _ = writeln!(output, "consumer: model");
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

    render_model_context_records(
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
    render_model_context_records(
        &mut output,
        "Subjects",
        &journal,
        case_ref,
        &[RecordKind::SubjectBinding],
    );
    render_model_context_records(
        &mut output,
        "Policy",
        &journal,
        case_ref,
        &[RecordKind::PolicyRule],
    );
    render_model_context_records(
        &mut output,
        "Projection Rules",
        &journal,
        case_ref,
        &[RecordKind::ProjectionRule],
    );
    render_model_context_records(
        &mut output,
        "Authority Scopes",
        &journal,
        case_ref,
        &[RecordKind::AuthorityScope],
    );
    render_model_context_records(
        &mut output,
        "Decisions",
        &journal,
        case_ref,
        &[RecordKind::Decision],
    );
    render_model_context_records(
        &mut output,
        "Filesystem Receipts",
        &journal,
        case_ref,
        &[RecordKind::FilesystemReceipt],
    );
    render_model_context_records(
        &mut output,
        "Memory",
        &journal,
        case_ref,
        &[RecordKind::MemoryCandidate],
    );
    render_model_context_records(
        &mut output,
        "Graph",
        &journal,
        case_ref,
        &[RecordKind::GraphEdge],
    );
    render_model_context_records(
        &mut output,
        "Projection Records",
        &journal,
        case_ref,
        &[RecordKind::ProjectionRequest, RecordKind::ProjectionResult],
    );
    render_model_context_records(
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

fn print_model_context_view(journal: &Journal, case_ref: Option<&str>) {
    print!("{}", render_model_context_view(journal, case_ref));
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
            && record.summary.contains(&format!("consumer:{consumer}"))
            && record.summary.contains(&format!("kind:{kind}"))
            && record.summary.contains("redaction:summary_only")
    });
    if !projection_available {
        return Err(format!(
            "no governed {consumer}/{kind} projection is available for case {case_ref}"
        ));
    }

    let already_admitted = journal.records().iter().any(|record| {
        record.case_ref == case_ref
            && record.kind == RecordKind::SubjectState
            && record.subject_ref == subject_ref
            && record.summary.contains("case_entry:admitted")
            && record.summary.contains(&format!("consumer:{consumer}"))
            && record.summary.contains(&format!("kind:{kind}"))
    });

    if !already_admitted {
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
    print_model_context_view(&journal, Some(&case_ref));
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
        "provider_attachment:attached provider:openai_compatible base_url:{base_url} model:{model} api_key_env:{api_key_env} prompt_surface:vendored_linenoise context:case_projection_graph_memory"
    );
    let already_attached = journal.records().iter().any(|record| {
        record.case_ref == case_ref
            && record.kind == RecordKind::SubjectState
            && record.subject_ref == subject_ref
            && record.summary == provider_summary
    });

    if !already_attached {
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
    println!("case_session: active");
    println!("case_context: active");
    println!("authority_scope: model interpretation_only");
    println!("provider_base_url: {base_url}");
    println!("provider_model: {model}");
    println!("api_key_env: {api_key_env}");
    Ok(())
}

struct ProviderConfig {
    base_url: String,
    model: String,
    api_key: Option<String>,
    language_mode: String,
}

struct PromptSession {
    journal_path: PathBuf,
    case_ref: String,
    subject_ref: String,
    provider: ProviderConfig,
    active_thread_id: String,
    participant_view: String,
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
            && record.summary.contains("prompt_transcript_retention:")
    }) {
        if record
            .summary
            .contains("prompt_transcript_retention:enabled")
        {
            enabled = true;
        } else if record
            .summary
            .contains("prompt_transcript_retention:disabled")
        {
            enabled = false;
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
    let prefix = format!("{key}:");
    summary.split_whitespace().find_map(|part| {
        part.strip_prefix(&prefix)
            .map(|value| value.trim_matches(',').to_string())
            .filter(|value| !value.is_empty())
    })
}

fn active_thread_id(journal: &Journal, case_ref: &str) -> Option<String> {
    let mut active = None;
    for record in journal.records().iter().filter(|record| {
        record.case_ref == case_ref && record.kind == RecordKind::InteractionThread
    }) {
        if record.summary.contains("state:active") {
            if let Some(thread_id) = summary_token(&record.summary, "thread_id") {
                active = Some(thread_id);
            }
        }
    }
    active
}

fn thread_turn_count(journal: &Journal, case_ref: &str, thread_id: &str) -> usize {
    let needle = format!("thread_id:{thread_id}");
    journal
        .records()
        .iter()
        .filter(|record| {
            record.case_ref == case_ref
                && record.kind == RecordKind::InteractionTurn
                && record.summary.contains(&needle)
        })
        .count()
}

fn latest_frame_id(journal: &Journal, case_ref: &str, thread_id: &str) -> Option<String> {
    let needle = format!("thread_id:{thread_id}");
    let mut frame_id = None;
    for record in journal.records().iter().filter(|record| {
        record.case_ref == case_ref
            && record.kind == RecordKind::ParticipantViewFrame
            && record.summary.contains(&needle)
    }) {
        frame_id = summary_token(&record.summary, "frame_id").or_else(|| Some(record.id.clone()));
    }
    frame_id
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
    let needle = format!("thread_id:{thread_id}");
    let mut count = 0usize;
    let _ = writeln!(output, "## Active Interaction Thread");
    let _ = writeln!(output, "interaction_thread: {thread_id}");
    let _ = writeln!(output, "journal_role: replay_audit_not_chat_memory");
    for record in journal.records().iter().filter(|record| {
        record.case_ref == case_ref
            && record.kind == RecordKind::InteractionTurn
            && record.summary.contains(&needle)
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

fn render_participant_view(journal: &Journal, case_ref: &str, thread_id: &str) -> String {
    let mut view = render_model_context_view(journal, Some(case_ref));
    view.push('\n');
    view.push_str(&render_thread_context(journal, case_ref, thread_id));
    view
}

fn prompt_session_from_args(args: &[String]) -> Result<PromptSession, String> {
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
    let journal = Journal::load_jsonl(&journal_path)
        .map_err(|error| format!("failed to load {}: {error}", journal_path.display()))?;
    let admitted = journal.records().iter().any(|record| {
        record.case_ref == case_ref
            && record.kind == RecordKind::SubjectState
            && record.subject_ref == subject_ref
            && record.summary.contains("case_entry:admitted")
    });
    if !admitted {
        return Err(format!(
            "{subject_ref} has not entered {case_ref}; run `yai case enter` first"
        ));
    }
    let attached = journal.records().iter().any(|record| {
        record.case_ref == case_ref
            && record.kind == RecordKind::SubjectState
            && record.subject_ref == subject_ref
            && record.summary.contains("provider_attachment:attached")
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

    Ok(PromptSession {
        journal_path,
        case_ref: case_ref.clone(),
        subject_ref,
        provider: ProviderConfig {
            base_url,
            model,
            api_key,
            language_mode,
        },
        active_thread_id: active_thread_id.clone(),
        participant_view: render_participant_view(&journal, &case_ref, &active_thread_id),
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

fn provider_chat_completion(
    config: &ProviderConfig,
    participant_view: &str,
    prompt: &str,
) -> Result<String, String> {
    let url = parse_http_url(&config.base_url)?;
    let mut system_prompt = "You are the case-bound model provider subject inside YAI. Answer only from the supplied YAI participant view and the operator prompt. Use a natural, direct style: start with the answer, avoid boilerplate openings, and do not begin every response with phrases like `according to the current projection`. Speak in terms of authority granted by the current case projection when authority matters, especially for refusals, missing access, proposed actions or boundary explanations. In those cases, prefer bounded authority language such as `the participant view does not grant authority to...` over absolute physical-capability claims. For ordinary factual answers, cite the visible record refs or record kinds when useful instead of repeating the authority preamble. Format for terminal reading with short paragraphs or compact bullets, and keep technical identifiers unchanged. Your outputs are claims/proposals/model interpretations, not authoritative case state, YAI decisions, policy rules, receipts or filesystem effects. subject:linenoise-terminal is only a display/input surface and never owns execution or decision authority. Any proposed action must become an op_attempt and pass through control/carrier before any effect. Do not claim raw journal, filesystem, shell, environment, API-key or out-of-case memory access unless explicitly provided by the participant view.".to_string();
    if config.language_mode == "auto" {
        system_prompt.push_str(" Respond in the same natural language as the operator prompt. Do not translate technical identifiers, record kinds, subject refs, case refs, decision ids, receipt ids, command names or code spans.");
    }
    let user_content =
        format!("YAI participant view:\n{participant_view}\n\nOperator prompt:\n{prompt}");
    let body = format!(
        "{{\"model\":\"{}\",\"stream\":false,\"messages\":[{{\"role\":\"system\",\"content\":\"{}\"}},{{\"role\":\"user\",\"content\":\"{}\"}}]}}",
        json_escape(&config.model),
        json_escape(&system_prompt),
        json_escape(&user_content)
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
    if !headers.starts_with("HTTP/1.1 2") && !headers.starts_with("HTTP/1.0 2") {
        return Err(format!(
            "provider returned non-2xx response: {}",
            compact_text(&String::from_utf8_lossy(body_bytes), 240)
        ));
    }
    let lower_headers = headers.to_ascii_lowercase();
    let body_bytes = if lower_headers.contains("transfer-encoding: chunked") {
        decode_chunked_body(body_bytes)?
    } else {
        body_bytes.to_vec()
    };
    let body_text = String::from_utf8_lossy(&body_bytes).to_string();
    extract_json_string_field(&body_text, "content").ok_or_else(|| {
        format!(
            "provider response did not contain message content: {}",
            compact_text(&body_text, 240)
        )
    })
}

fn redact_sensitive(value: &str, session: &PromptSession) -> String {
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

fn transcript_text(value: &str, session: &PromptSession) -> String {
    redact_sensitive(value, session)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn append_model_prompt_attempt(session: &PromptSession, prompt: &str) -> Result<String, String> {
    let journal = Journal::load_jsonl(&session.journal_path)
        .map_err(|error| format!("failed to load {}: {error}", session.journal_path.display()))?;
    let sequence = journal.count() + 1;
    let attempt_id = format!("attempt:model-prompt-{sequence}");
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
    append_record_to_journal(&session.journal_path, &record)?;
    Ok(attempt_id)
}

fn append_model_output_receipt(
    session: &PromptSession,
    attempt_id: &str,
    output: &str,
) -> Result<(), String> {
    let journal = Journal::load_jsonl(&session.journal_path)
        .map_err(|error| format!("failed to load {}: {error}", session.journal_path.display()))?;
    let sequence = journal.count() + 1;
    let receipt_id = format!("receipt:model-output-{sequence}");
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
    append_record_to_journal(&session.journal_path, &record)
}

fn append_model_interpretation_record(
    session: &PromptSession,
    attempt_id: &str,
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
    append_record_to_journal(&session.journal_path, &record)?;
    Ok(summary)
}

fn append_participant_view_frame(session: &PromptSession) -> Result<String, String> {
    let journal = Journal::load_jsonl(&session.journal_path)
        .map_err(|error| format!("failed to load {}: {error}", session.journal_path.display()))?;
    let sequence = journal.count() + 1;
    let previous_frame_id =
        latest_frame_id(&journal, &session.case_ref, &session.active_thread_id).unwrap_or_default();
    let frame_id = format!("frame:participant-view-{sequence}");
    let included_turn_count =
        thread_turn_count(&journal, &session.case_ref, &session.active_thread_id);
    let included_memory_count = journal
        .records()
        .iter()
        .filter(|record| {
            record.case_ref == session.case_ref && record.kind == RecordKind::MemoryCandidate
        })
        .count();
    let included_receipt_count = journal
        .records()
        .iter()
        .filter(|record| {
            record.case_ref == session.case_ref
                && matches!(
                    record.kind,
                    RecordKind::Receipt | RecordKind::EffectReceipt | RecordKind::FilesystemReceipt
                )
        })
        .count();
    let projection_id = journal
        .records()
        .iter()
        .rev()
        .find(|record| {
            record.case_ref == session.case_ref && record.kind == RecordKind::ProjectionResult
        })
        .map(|record| record.id.as_str())
        .unwrap_or("projection:current");
    let freshness = projection_freshness_view(&session.case_ref, "model");
    let summary = format!(
        "participant_view_frame frame_id:{frame_id} case_ref:{} thread_id:{} projection_id:{projection_id} previous_frame_id:{} delta_since_frame_id:{} included_turn_count:{included_turn_count} included_memory_count:{included_memory_count} included_receipt_count:{included_receipt_count} redaction:summary_only freshness:{} freshness_source:{} stale_reason_at_build:{} freshness_policy:{} source:active_thread_plus_case_projection",
        session.case_ref,
        session.active_thread_id,
        if previous_frame_id.is_empty() { "none" } else { previous_frame_id.as_str() },
        if previous_frame_id.is_empty() { "none" } else { previous_frame_id.as_str() },
        freshness.freshness,
        freshness.source,
        freshness.stale_reason,
        freshness.policy
    );
    let record = Record::from_parts(
        &frame_id,
        &session.case_ref,
        RecordKind::ParticipantViewFrame,
        &session.subject_ref,
        "",
        "",
        "",
        summary,
    );
    append_record_to_journal(&session.journal_path, &record)?;
    Ok(frame_id)
}

fn append_interaction_turn(
    session: &PromptSession,
    attempt_id: &str,
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
    append_record_to_journal(&session.journal_path, &record)?;
    Ok(record_id)
}

fn prompt_attempt_summary(session: &PromptSession, prompt: &str) -> String {
    if session.transcript_enabled {
        format!(
            "op:model.prompt.submit prompt_surface:vendored_linenoise context:participant_view_frame thread_id:{} transcript_retention:full_redacted_case_local prompt_text:{}",
            session.active_thread_id,
            transcript_text(prompt, session)
        )
    } else {
        format!(
            "op:model.prompt.submit prompt_surface:vendored_linenoise context:participant_view_frame thread_id:{} transcript_retention:preview_only prompt_preview:{}",
            session.active_thread_id,
            compact_text(prompt, 120)
        )
    }
}

fn model_output_summary(session: &PromptSession, output: &str) -> String {
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
    session: &PromptSession,
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

fn append_memory_proposal(session: &PromptSession, note: Option<&str>) -> Result<String, String> {
    let journal = Journal::load_jsonl(&session.journal_path)
        .map_err(|error| format!("failed to load {}: {error}", session.journal_path.display()))?;
    let prompt_attempts = journal
        .records()
        .iter()
        .filter(|record| {
            record.case_ref == session.case_ref
                && record.kind == RecordKind::Attempt
                && record.summary.contains("op:model.prompt.submit")
        })
        .count();
    let model_outputs = journal
        .records()
        .iter()
        .filter(|record| {
            record.case_ref == session.case_ref
                && record.kind == RecordKind::EffectReceipt
                && record.summary.contains("model.output")
        })
        .count();
    let basis_records = prompt_attempts + model_outputs;
    let sequence = journal.count() + 1;
    let note = note
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!(" note:{}", compact_text(value, 120)))
        .unwrap_or_default();
    let summary = format!(
        "memory:operational scope:case source:prompt_session transcript_retention:{} basis_records:{} basis_prompt_attempts:{} basis_model_outputs:{} summary:prompt session tested case boundary and model/provider residue{}",
        transcript_retention_label(session.transcript_enabled),
        basis_records,
        prompt_attempts,
        model_outputs,
        note
    );
    let record = Record::from_parts(
        format!(
            "prompt-memory:{}:{sequence}",
            session.subject_ref.replace(':', "-")
        ),
        &session.case_ref,
        RecordKind::MemoryCandidate,
        &session.subject_ref,
        "",
        "",
        "",
        summary.clone(),
    );
    append_record_to_journal(&session.journal_path, &record)?;
    Ok(summary)
}

fn run_prompt_once(session: &mut PromptSession, prompt: &str, dry_run: bool) -> Result<(), String> {
    let colors = color_enabled();
    let freshness = projection_freshness_view(&session.case_ref, "model");
    if dry_run {
        println!("model_prompt: dry_run");
        println!("case_ref: {}", session.case_ref);
        println!("case_session: active");
        println!("case_context: active");
        println!("interaction_thread: {}", session.active_thread_id);
        println!("participant_view_frame: would_build");
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
        println!("context_source: interaction_thread_plus_projection_frame");
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
    let frame_id = append_participant_view_frame(session)?;
    let attempt_id = append_model_prompt_attempt(session, prompt)?;
    let output = provider_chat_completion(&session.provider, &session.participant_view, prompt)?;
    println!();
    print_cli_section(colors, "MODEL", &session.provider.model, ANSI_MAGENTA);
    print_model_output(colors, &output);
    println!();
    append_model_output_receipt(session, &attempt_id, &output)?;
    let interpretation_summary = append_model_interpretation_record(session, &attempt_id, &output)?;
    let turn_id = append_interaction_turn(session, &attempt_id, prompt, &output)?;
    let journal = Journal::load_jsonl(&session.journal_path)
        .map_err(|error| format!("failed to load {}: {error}", session.journal_path.display()))?;
    session.participant_view =
        render_participant_view(&journal, &session.case_ref, &session.active_thread_id);
    println!("participant_view_frame: {frame_id}");
    println!("interaction_turn: {turn_id}");
    println!(
        "model_interpretation: {}",
        compact_text(&interpretation_summary, 120)
    );
    Ok(())
}

fn handle_prompt_command(session: &mut PromptSession, command: &str) -> Result<bool, String> {
    if command == "/refresh" {
        let journal = Journal::load_jsonl(&session.journal_path).map_err(|error| {
            format!("failed to load {}: {error}", session.journal_path.display())
        })?;
        session.participant_view =
            render_participant_view(&journal, &session.case_ref, &session.active_thread_id);
        session.transcript_enabled =
            transcript_retention_enabled(&journal, &session.case_ref, &session.subject_ref);
        let frame_id = append_participant_view_frame(session)?;
        println!("case_prompt: refreshed");
        println!("participant_view_frame: {frame_id}");
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
        println!("participant_view: active_thread_plus_projection");
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
        session.participant_view =
            render_participant_view(&journal, &session.case_ref, &session.active_thread_id);
        println!("interaction_thread: new active");
        println!("thread_id: {thread_id}");
        println!("participant_view: empty thread");
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
        session.participant_view =
            render_participant_view(&journal, &session.case_ref, &session.active_thread_id);
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
        session.participant_view =
            render_participant_view(&journal, &session.case_ref, &session.active_thread_id);
        println!("interaction_thread: archived");
        println!("thread_id: {thread_id}");
        println!("active_thread_id: {}", session.active_thread_id);
        return Ok(true);
    }

    if command == "/transcript on" {
        let summary = append_transcript_retention_state(session, true)?;
        session.transcript_enabled = true;
        let _ = writeln!(session.participant_view, "## Prompt Session State");
        let _ = writeln!(
            session.participant_view,
            "- kind:subject_state subject_ref:{} summary:{}",
            session.subject_ref, summary
        );
        let _ = writeln!(session.participant_view);
        println!("prompt_transcript_retention: enabled");
        println!("transcript_retention: full_redacted_case_local");
        println!("full_transcript: on_explicit_redacted_case_local");
        println!("redaction: secret_redacted");
        return Ok(true);
    }

    if command == "/transcript off" {
        let summary = append_transcript_retention_state(session, false)?;
        session.transcript_enabled = false;
        let _ = writeln!(session.participant_view, "## Prompt Session State");
        let _ = writeln!(
            session.participant_view,
            "- kind:subject_state subject_ref:{} summary:{}",
            session.subject_ref, summary
        );
        let _ = writeln!(session.participant_view);
        println!("prompt_transcript_retention: disabled");
        println!("transcript_retention: preview_only");
        println!("full_transcript: off");
        return Ok(true);
    }

    if command == "/memory propose" || command.starts_with("/memory propose ") {
        let note = command.strip_prefix("/memory propose").map(str::trim);
        let summary = append_memory_proposal(session, note)?;
        let _ = writeln!(session.participant_view, "## Prompt Session Memory");
        let _ = writeln!(
            session.participant_view,
            "- kind:memory_candidate subject_ref:{} summary:{}",
            session.subject_ref, summary
        );
        let _ = writeln!(session.participant_view);
        println!("memory_proposal: accepted");
        println!("record_kind: memory_candidate");
        println!(
            "transcript_retention: {}",
            transcript_retention_label(session.transcript_enabled)
        );
        return Ok(true);
    }

    if command.starts_with('/') {
        println!("unknown_command: {command}");
        println!("commands: /thread status /thread new [label] /thread list /thread use <thread_id> /thread archive <thread_id> /refresh /transcript on /transcript off /transcript status /memory propose /exit");
        return Ok(true);
    }

    Ok(false)
}

pub(super) fn prompt_repl(args: &[String]) -> Result<(), String> {
    let dry_run = args.iter().any(|arg| arg == "--dry-run");
    let once = optional_arg(args, "--once");
    let mut session = prompt_session_from_args(args)?;
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
    println!("context_source: interaction_thread_plus_projection_frame");
    println!(
        "transcript_retention: {}",
        transcript_retention_label(session.transcript_enabled)
    );
    println!("commands: /thread status /thread new [label] /thread list /thread use <thread_id> /thread archive <thread_id> /refresh /transcript on /transcript off /transcript status /memory propose /exit");

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
