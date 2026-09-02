use std::io::IsTerminal;

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct Field {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ParticipantView {
    pub participant_id: String,
    pub roles: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ProviderView {
    pub participant_id: String,
    pub provider_id: String,
    pub provider_kind: String,
    pub endpoint: String,
    pub model_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ProviderBindingView {
    pub mode: String,
    pub binding_id: String,
    pub participant_id: String,
    pub candidate_count: usize,
    pub failover_policy: String,
    pub last_selection_id: Option<String>,
    pub last_selected_target: Option<String>,
    pub last_selected_model: Option<String>,
    pub last_attempt_posture: Option<String>,
    pub delivery_indeterminate: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ResourceView {
    pub resource_id: String,
    pub kind: String,
    pub policy_id: String,
    pub policy_owner_participant_id: String,
    pub review_requirement: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct WorkflowView {
    pub definition_id: String,
    pub binding_id: String,
    pub completed: bool,
    pub satisfied: usize,
    pub skipped: usize,
    pub active: usize,
    pub waiting: usize,
    pub ready_nodes: Vec<String>,
    pub effective_revision: u32,
    pub effective_topology_digest: String,
    pub amendment_count: usize,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct CaseView {
    pub case_id: String,
    pub tenant_id: String,
    pub generation: u64,
    pub lifecycle: String,
    pub execution: String,
    pub participants: Vec<ParticipantView>,
    pub provider: Option<ProviderView>,
    pub provider_binding: Option<ProviderBindingView>,
    pub resources: Vec<ResourceView>,
    pub workflow: Option<WorkflowView>,
    pub policy_bindings: usize,
    pub pending_reviews: usize,
    pub unresolved_effects: usize,
    pub finalized_effects: usize,
    pub open_handoff_offers: usize,
    pub handoff_acceptances: usize,
    pub handoff_results: usize,
    pub handoff_reconciliations: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum CliData {
    Message {
        message: String,
        fields: Vec<Field>,
    },
    Object {
        title: String,
        fields: Vec<Field>,
    },
    Collection {
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Doctor {
        posture: String,
        checks: Vec<Field>,
        remediation: Option<String>,
    },
    Case {
        case: Box<CaseView>,
    },
    Completion {
        shell: String,
        script: String,
    },
    NativeJson {
        value: serde_json::Value,
    },
    Compatibility {
        lines: Vec<String>,
    },
    AlreadyRendered,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct CliError {
    #[serde(skip)]
    pub category: ErrorCategory,
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ErrorCategory {
    Usage,
    Domain,
    Internal,
}

impl CliError {
    pub(crate) fn exit_code(&self) -> i32 {
        match self.category {
            ErrorCategory::Usage => 2,
            ErrorCategory::Domain => 3,
            ErrorCategory::Internal => 4,
        }
    }

    pub(crate) fn usage(message: impl Into<String>) -> Self {
        Self {
            category: ErrorCategory::Usage,
            code: "usage_error",
            message: message.into(),
            field: None,
            remediation: Some("use `yai help`".to_string()),
        }
    }

    pub(crate) fn domain(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            category: ErrorCategory::Domain,
            code,
            message: message.into(),
            field: None,
            remediation: None,
        }
    }

    pub(crate) fn internal(message: impl Into<String>) -> Self {
        Self {
            category: ErrorCategory::Internal,
            code: "internal_error",
            message: message.into(),
            field: None,
            remediation: None,
        }
    }

    pub(crate) fn unknown_command(command: String, suggestion: Option<String>) -> Self {
        Self {
            category: ErrorCategory::Usage,
            code: "unknown_command",
            message: format!("unknown command: {command}"),
            field: None,
            remediation: Some(
                suggestion
                    .map(|path| format!("did you mean `yai {path}`?"))
                    .unwrap_or_else(|| "use `yai help`".to_string()),
            ),
        }
    }

    pub(crate) fn removed(
        _operation_id: &'static str,
        message: String,
        remediation: String,
    ) -> Self {
        Self {
            category: ErrorCategory::Usage,
            code: "removed_command",
            message,
            field: None,
            remediation: Some(remediation),
        }
    }
}

#[derive(Serialize)]
struct SuccessEnvelope<'a> {
    schema: &'static str,
    operation_id: &'a str,
    status: &'static str,
    data: &'a CliData,
}

#[derive(Serialize)]
struct ErrorEnvelope<'a> {
    schema: &'static str,
    operation_id: Option<&'a str>,
    status: &'static str,
    #[serde(flatten)]
    error: &'a CliError,
}

pub(crate) fn render_result(operation_id: &str, json: bool, data: CliData) {
    if matches!(data, CliData::AlreadyRendered) {
        return;
    }
    if json {
        let envelope = SuccessEnvelope {
            schema: "yai.cli.result.v1",
            operation_id,
            status: "ok",
            data: &data,
        };
        println!(
            "{}",
            serde_json::to_string(&envelope).expect("CLI result serializes")
        );
        return;
    }
    render_human(&data);
}

pub(crate) fn render_error(operation_id: Option<&str>, json: bool, error: CliError) {
    if json {
        let envelope = ErrorEnvelope {
            schema: "yai.cli.error.v1",
            operation_id,
            status: "error",
            error: &error,
        };
        eprintln!(
            "{}",
            serde_json::to_string(&envelope).expect("CLI error serializes")
        );
        return;
    }
    eprintln!("yai: {}", error.message);
    if let Some(remediation) = error.remediation {
        eprintln!("hint: {remediation}");
    }
}

fn render_human(data: &CliData) {
    let styled = std::io::stdout().is_terminal()
        && std::env::var_os("NO_COLOR").is_none()
        && std::env::var_os("TERM").as_deref() != Some(std::ffi::OsStr::new("dumb"));
    match data {
        CliData::Message { message, fields } => {
            println!("{}", success(message, styled));
            render_fields(fields, styled);
        }
        CliData::Object { title, fields } => {
            println!("{}", heading(title, styled));
            render_fields(fields, styled);
        }
        CliData::Collection { columns, rows } => render_table(columns, rows, styled),
        CliData::Doctor {
            posture,
            checks,
            remediation,
        } => {
            println!("{}", heading("YAI DOCTOR", styled));
            println!("{:<18} {}", "Posture", posture_style(posture, styled));
            render_fields(checks, styled);
            if let Some(remediation) = remediation {
                println!("\n{} {remediation}", warning("Next", styled));
            }
        }
        CliData::Case { case } => render_case(case, styled),
        CliData::Completion { script, .. } => print!("{script}"),
        CliData::NativeJson { value } => println!(
            "{}",
            serde_json::to_string_pretty(value).expect("native JSON serializes")
        ),
        CliData::Compatibility { lines } => {
            for line in lines {
                println!("{line}");
            }
        }
        CliData::AlreadyRendered => {}
    }
}

fn render_case(case: &CaseView, styled: bool) {
    println!("{}", heading("CASE", styled));
    render_fields(
        &[
            Field {
                name: "ID".to_string(),
                value: case.case_id.clone(),
            },
            Field {
                name: "Tenant".to_string(),
                value: case.tenant_id.clone(),
            },
            Field {
                name: "Lifecycle".to_string(),
                value: case.lifecycle.clone(),
            },
            Field {
                name: "Generation".to_string(),
                value: case.generation.to_string(),
            },
        ],
        styled,
    );
    println!("\n{}", heading("EXECUTION", styled));
    render_fields(
        &[Field {
            name: "Posture".to_string(),
            value: case.execution.clone(),
        }],
        styled,
    );
    if let Some(workflow) = &case.workflow {
        println!("\n{}", heading("WORKFLOW", styled));
        render_fields(
            &[
                Field {
                    name: "Definition".to_string(),
                    value: workflow.definition_id.clone(),
                },
                Field {
                    name: "Completed".to_string(),
                    value: workflow.completed.to_string(),
                },
                Field {
                    name: "Revision".to_string(),
                    value: workflow.effective_revision.to_string(),
                },
                Field {
                    name: "Topology digest".to_string(),
                    value: workflow.effective_topology_digest.clone(),
                },
                Field {
                    name: "Amendments".to_string(),
                    value: workflow.amendment_count.to_string(),
                },
                Field {
                    name: "Satisfied".to_string(),
                    value: workflow.satisfied.to_string(),
                },
                Field {
                    name: "Active".to_string(),
                    value: workflow.active.to_string(),
                },
                Field {
                    name: "Waiting".to_string(),
                    value: workflow.waiting.to_string(),
                },
            ],
            styled,
        );
    }
    if let Some(binding) = &case.provider_binding {
        println!("\n{}", heading("PROVIDER ROUTING", styled));
        render_fields(
            &[
                Field {
                    name: "Mode".to_string(),
                    value: binding.mode.clone(),
                },
                Field {
                    name: "Binding".to_string(),
                    value: binding.binding_id.clone(),
                },
                Field {
                    name: "Participant".to_string(),
                    value: binding.participant_id.clone(),
                },
                Field {
                    name: "Candidates".to_string(),
                    value: binding.candidate_count.to_string(),
                },
                Field {
                    name: "Failover".to_string(),
                    value: binding.failover_policy.clone(),
                },
                Field {
                    name: "Last target".to_string(),
                    value: binding
                        .last_selected_target
                        .clone()
                        .unwrap_or_else(|| "none".to_string()),
                },
                Field {
                    name: "Last model".to_string(),
                    value: binding
                        .last_selected_model
                        .clone()
                        .unwrap_or_else(|| "none".to_string()),
                },
                Field {
                    name: "Last attempt".to_string(),
                    value: binding
                        .last_attempt_posture
                        .clone()
                        .unwrap_or_else(|| "none".to_string()),
                },
                Field {
                    name: "Delivery indeterminate".to_string(),
                    value: binding.delivery_indeterminate.to_string(),
                },
            ],
            styled,
        );
    }
    if let Some(provider) = &case.provider {
        println!("\n{}", heading("MODEL", styled));
        render_fields(
            &[
                Field {
                    name: "Participant".to_string(),
                    value: provider.participant_id.clone(),
                },
                Field {
                    name: "Provider".to_string(),
                    value: provider.provider_id.clone(),
                },
                Field {
                    name: "Endpoint".to_string(),
                    value: provider.endpoint.clone(),
                },
                Field {
                    name: "Model".to_string(),
                    value: provider.model_id.clone(),
                },
            ],
            styled,
        );
    }
    println!("\n{}", heading("RESOURCES", styled));
    if case.resources.is_empty() {
        println!("none");
    } else {
        for resource in &case.resources {
            println!(
                "{}  {}  policy={}",
                resource.resource_id, resource.kind, resource.policy_id
            );
        }
    }
    println!("\n{}", heading("GOVERNANCE", styled));
    render_fields(
        &[
            Field {
                name: "Policy bindings".to_string(),
                value: case.policy_bindings.to_string(),
            },
            Field {
                name: "Pending reviews".to_string(),
                value: case.pending_reviews.to_string(),
            },
        ],
        styled,
    );
    println!("\n{}", heading("EFFECTS", styled));
    render_fields(
        &[
            Field {
                name: "Unresolved".to_string(),
                value: case.unresolved_effects.to_string(),
            },
            Field {
                name: "Finalized".to_string(),
                value: case.finalized_effects.to_string(),
            },
        ],
        styled,
    );
    if case.open_handoff_offers != 0
        || case.handoff_acceptances != 0
        || case.handoff_results != 0
        || case.handoff_reconciliations != 0
    {
        println!("\n{}", heading("HANDOFFS", styled));
        render_fields(
            &[
                Field {
                    name: "Open offers".to_string(),
                    value: case.open_handoff_offers.to_string(),
                },
                Field {
                    name: "Accepted".to_string(),
                    value: case.handoff_acceptances.to_string(),
                },
                Field {
                    name: "Results".to_string(),
                    value: case.handoff_results.to_string(),
                },
                Field {
                    name: "Reconciled".to_string(),
                    value: case.handoff_reconciliations.to_string(),
                },
            ],
            styled,
        );
    }
}

fn render_fields(fields: &[Field], styled: bool) {
    let width = fields
        .iter()
        .map(|field| field.name.len())
        .max()
        .unwrap_or(0)
        .min(24);
    for field in fields {
        println!(
            "{:<width$} {}",
            field.name,
            secondary(&field.value, styled),
            width = width
        );
    }
}

fn render_table(columns: &[String], rows: &[Vec<String>], styled: bool) {
    if columns.is_empty() {
        return;
    }
    let terminal_width = std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(120)
        .max(40);
    let available = terminal_width.saturating_sub(columns.len().saturating_sub(1) * 2);
    let per_column = (available / columns.len()).max(8);
    let widths = columns
        .iter()
        .enumerate()
        .map(|(index, column)| {
            rows.iter()
                .filter_map(|row| row.get(index))
                .map(String::len)
                .chain(std::iter::once(column.len()))
                .max()
                .unwrap_or(column.len())
                .min(per_column)
        })
        .collect::<Vec<_>>();
    for (index, column) in columns.iter().enumerate() {
        if index > 0 {
            print!("  ");
        }
        let label = truncate(column, widths[index]);
        let padded = format!("{label:<width$}", width = widths[index]);
        print!("{}", heading(&padded, styled));
    }
    println!();
    for row in rows {
        for (index, width) in widths.iter().enumerate() {
            if index > 0 {
                print!("  ");
            }
            let value = row.get(index).map(String::as_str).unwrap_or("");
            print!("{:<width$}", truncate(value, *width), width = width);
        }
        println!();
    }
}

fn truncate(value: &str, width: usize) -> String {
    if value.chars().count() <= width {
        return value.to_string();
    }
    if width < 2 {
        return "…".to_string();
    }
    value.chars().take(width - 1).collect::<String>() + "…"
}

fn heading(value: &str, styled: bool) -> String {
    style(value, "\x1b[1m", styled)
}

fn success(value: &str, styled: bool) -> String {
    style(value, "\x1b[32m", styled)
}

fn warning(value: &str, styled: bool) -> String {
    style(value, "\x1b[33m", styled)
}

fn secondary(value: &str, styled: bool) -> String {
    style(value, "\x1b[2m", styled)
}

fn posture_style(value: &str, styled: bool) -> String {
    let code = match value {
        "OK" => "\x1b[32m",
        "WARNING" | "DEGRADED" => "\x1b[33m",
        _ => "\x1b[31m",
    };
    style(value, code, styled)
}

fn style(value: &str, code: &str, styled: bool) -> String {
    if styled {
        format!("{code}{value}\x1b[0m")
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structured_data_is_typed_and_ansi_free() {
        let data = CliData::Object {
            title: "CASE".to_string(),
            fields: vec![Field {
                name: "ID".to_string(),
                value: "case:test".to_string(),
            }],
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("case:test"));
        assert!(!json.contains("\\u001b"));
    }

    #[test]
    fn truncation_never_changes_machine_data() {
        let exact = "case:0123456789";
        assert_eq!(truncate(exact, 8), "case:01…");
        let data = CliData::Message {
            message: "ok".to_string(),
            fields: vec![Field {
                name: "ID".to_string(),
                value: exact.to_string(),
            }],
        };
        assert!(serde_json::to_string(&data).unwrap().contains(exact));
    }
}
