use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

use yai_core_engine::security::AuthenticatedPrincipal;
use yai_core_engine::store::lmdb::{LmdbRecordStore, RecordStoreStatusKind};

use super::output::{
    CaseView, CliData, CliError, Field, ParticipantView, ProviderBindingView, ProviderView,
    ResourceView, WorkflowView,
};
use super::parser::Invocation;
use super::registry::{registry_digest, Visibility, REGISTRY_SCHEMA};

pub(crate) fn execute(invocation: &Invocation) -> Result<CliData, CliError> {
    match invocation.descriptor.operation_id {
        "yai.meta.version" => version(),
        "yai.meta.completion" => completion(invocation),
        "yai.doctor" => doctor(),
        "yai.init" => init(invocation),
        "yai.identity.whoami" => identity(),
        "yai.tenant.list" => tenant_list(),
        "yai.tenant.show" => tenant_show(invocation),
        "yai.case.list" => case_list(invocation),
        "yai.case.participant.list" => participant_list(invocation),
        "yai.case.resource.list" => resource_list(invocation),
        "yai.case.show" if invocation.compatibility_syntax && !invocation.json => {
            crate::command_adapters::dispatch_operation("yai.case.show", &invocation.legacy_args())
                .map_err(|error| domain_error(classify_domain_code(&error), error))?;
            Ok(CliData::AlreadyRendered)
        }
        "yai.case.show" => case_show(invocation),
        operation_id if invocation.descriptor.visibility == Visibility::Product => {
            execute_structured_legacy(operation_id, invocation)
        }
        _ => {
            crate::command_adapters::dispatch_operation(
                invocation.descriptor.operation_id,
                &invocation.legacy_args(),
            )
            .map_err(|error| domain_error(classify_domain_code(&error), error))?;
            Ok(CliData::AlreadyRendered)
        }
    }
}

fn version() -> Result<CliData, CliError> {
    let mut fields = BTreeMap::new();
    fields.insert("Binary".to_string(), env!("CARGO_PKG_VERSION").to_string());
    fields.insert("CLI registry".to_string(), REGISTRY_SCHEMA.to_string());
    fields.insert("Registry digest".to_string(), registry_digest());
    fields.insert(
        "Transition schema".to_string(),
        yai_core_engine::transition::TRANSITION_SCHEMA.to_string(),
    );
    fields.insert(
        "CaseState schema".to_string(),
        yai_core_engine::transition::CASE_STATE_SCHEMA.to_string(),
    );
    Ok(CliData::Object {
        title: "YAI VERSION".to_string(),
        fields: map_fields(fields),
    })
}

fn init(invocation: &Invocation) -> Result<CliData, CliError> {
    let home = yai_home();
    for relative in ["run", "store", "log", "tmp", "cases", "sockets", "config"] {
        fs::create_dir_all(home.join(relative)).map_err(|error| {
            CliError::domain(
                "initialization_failed",
                format!("cannot prepare YAI_HOME: {error}"),
            )
        })?;
    }
    execute_structured_legacy("yai.init", invocation)
}

fn doctor() -> Result<CliData, CliError> {
    let home = yai_home();
    let required = ["run", "store", "log", "tmp", "cases", "sockets", "config"];
    let layout_ready = required.iter().all(|relative| home.join(relative).is_dir());
    let status = LmdbRecordStore::status(record_store_path());
    let local_identity = if status.status == RecordStoreStatusKind::Ready {
        AuthenticatedPrincipal::authenticate_local()
            .ok()
            .and_then(|authenticated| {
                LmdbRecordStore::open(record_store_path())
                    .ok()
                    .map(|store| (authenticated, store))
            })
            .and_then(|(authenticated, store)| store.enrolled_principal(&authenticated).ok())
            .is_some()
    } else {
        false
    };
    let posture = if !layout_ready
        || matches!(
            status.status,
            RecordStoreStatusKind::Missing | RecordStoreStatusKind::NotInitialized
        ) {
        "NOT_INITIALIZED"
    } else if status.status == RecordStoreStatusKind::Unavailable {
        "FAILED"
    } else if !local_identity {
        "DEGRADED"
    } else {
        "OK"
    };
    let checks = vec![
        field(
            "Binary",
            std::env::current_exe()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|_| "unavailable".to_string()),
        ),
        field("YAI_HOME", home.display().to_string()),
        field(
            "Runtime layout",
            if layout_ready { "ready" } else { "missing" },
        ),
        field("Storage", status.status.as_str()),
        field("Storage backend", status.backend),
        field(
            "Local identity",
            if local_identity {
                "enrolled"
            } else {
                "not_enrolled"
            },
        ),
        field("RuntimeInstance", runtime_posture(&home)),
        field("Legacy daemon", "optional"),
    ];
    let remediation = (posture != "OK")
        .then(|| "run `yai init --tenant <TENANT> --organization <ORGANIZATION>`".to_string());
    Ok(CliData::Doctor {
        posture: posture.to_string(),
        checks,
        remediation,
    })
}

fn identity() -> Result<CliData, CliError> {
    let authenticated = AuthenticatedPrincipal::authenticate_local()
        .map_err(|error| domain_error("authentication_failed", error))?;
    let store = open_store()?;
    let principal = store
        .enrolled_principal(&authenticated)
        .map_err(|error| domain_error("principal_not_enrolled", error))?;
    let relations = store
        .list_principal_tenants(&authenticated)
        .map_err(|error| domain_error("tenant_membership_unavailable", error))?;
    Ok(CliData::Object {
        title: "IDENTITY".to_string(),
        fields: vec![
            field("Principal", principal.principal_id),
            field("Authentication", principal.authentication_method),
            field("Binding", authenticated.binding().binding_ref.clone()),
            field(
                "Tenants",
                relations
                    .into_iter()
                    .map(|relation| relation.tenant.tenant_id)
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        ],
    })
}

fn tenant_list() -> Result<CliData, CliError> {
    let authenticated = AuthenticatedPrincipal::authenticate_local()
        .map_err(|error| domain_error("authentication_failed", error))?;
    let relations = open_store()?
        .list_principal_tenants(&authenticated)
        .map_err(|error| domain_error("tenant_list_failed", error))?;
    Ok(CliData::Collection {
        columns: vec![
            "TENANT".to_string(),
            "ORGANIZATION".to_string(),
            "MEMBERSHIP".to_string(),
        ],
        rows: relations
            .into_iter()
            .map(|relation| {
                vec![
                    relation.tenant.tenant_id,
                    relation.tenant.organization_ref,
                    enum_name(&relation.membership),
                ]
            })
            .collect(),
    })
}

fn tenant_show(invocation: &Invocation) -> Result<CliData, CliError> {
    let tenant_id = invocation
        .positional("tenant")
        .ok_or_else(|| CliError::usage("missing Tenant identity"))?;
    let authenticated = AuthenticatedPrincipal::authenticate_local()
        .map_err(|error| domain_error("authentication_failed", error))?;
    let store = open_store()?;
    let context = store
        .resolve_security_context(&authenticated, tenant_id)
        .map_err(|error| domain_error("tenant_not_visible", error))?;
    let tenant = store
        .get_tenant(tenant_id)
        .map_err(|error| domain_error("tenant_read_failed", error))?
        .ok_or_else(|| domain_error("tenant_not_found", "Tenant does not exist"))?;
    Ok(CliData::Object {
        title: "TENANT".to_string(),
        fields: vec![
            field("ID", tenant.tenant_id),
            field("Organization", tenant.organization_ref),
            field("Owner Principal", tenant.owner_principal_id),
            field("Current Principal", context.principal_id()),
            field("Membership", enum_name(context.membership())),
        ],
    })
}

fn case_list(invocation: &Invocation) -> Result<CliData, CliError> {
    let authenticated = AuthenticatedPrincipal::authenticate_local()
        .map_err(|error| domain_error("authentication_failed", error))?;
    let store = open_store()?;
    let states = store
        .list_case_states_authorized(&authenticated, invocation.flag("--tenant"), 1024)
        .map_err(|error| domain_error("case_list_failed", error))?;
    let rows = states
        .into_iter()
        .map(|state| {
            let attention = if state.cancellation.is_some() {
                "cancelled"
            } else if state
                .effects
                .iter()
                .any(|effect| effect.receipt_id.is_none())
            {
                "effect"
            } else if state
                .reviews
                .iter()
                .any(|review| format!("{:?}", review.status).starts_with("Pending"))
            {
                "review"
            } else {
                "none"
            };
            vec![
                state.case_id,
                state.tenant_id.unwrap_or_else(|| "unscoped".to_string()),
                enum_name(&state.lifecycle),
                if state.workflow_binding.is_some() {
                    "bound".to_string()
                } else {
                    "none".to_string()
                },
                attention.to_string(),
            ]
        })
        .collect();
    Ok(CliData::Collection {
        columns: ["CASE", "TENANT", "LIFECYCLE", "WORKFLOW", "ATTENTION"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        rows,
    })
}

fn case_show(invocation: &Invocation) -> Result<CliData, CliError> {
    let case_id = invocation
        .positional("case")
        .ok_or_else(|| CliError::usage("missing Case identity"))?;
    let authenticated = AuthenticatedPrincipal::authenticate_local()
        .map_err(|error| domain_error("authentication_failed", error))?;
    let store = open_store()?;
    let state = store
        .get_case_state_authorized(&authenticated, case_id)
        .map_err(|error| domain_error("case_not_found_or_not_visible", error))?;
    let execution = case_execution_posture(case_id);
    let workflow = if state.workflow_binding.is_some() {
        let resolution = store
            .workflow_status_authorized(&authenticated, case_id)
            .map_err(|error| domain_error("workflow_resolution_failed", error))?;
        Some(WorkflowView {
            effective_revision: resolution.effective_revision,
            effective_topology_digest: resolution.effective_topology_digest.clone(),
            amendment_count: resolution.amendment_ids.len(),
            definition_id: resolution.workflow_definition_id,
            binding_id: resolution.workflow_binding_id,
            completed: resolution.completed,
            satisfied: resolution.satisfied_count,
            skipped: resolution.skipped_count,
            active: resolution.active_count,
            waiting: resolution.waiting_count,
            ready_nodes: resolution
                .ready_work
                .into_iter()
                .map(|work| work.node_id)
                .collect(),
        })
    } else {
        None
    };
    let pending_reviews = state
        .reviews
        .iter()
        .filter(|review| format!("{:?}", review.status).starts_with("Pending"))
        .count();
    let unresolved_effects = state
        .effects
        .iter()
        .filter(|effect| effect.receipt_id.is_none())
        .count();
    let finalized_effects = state
        .effects
        .iter()
        .filter(|effect| effect.receipt_id.is_some())
        .count();
    let reconciled_handoffs = state
        .handoff_reconciliations
        .iter()
        .map(|value| value.handoff_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let open_handoff_offers = state
        .handoff_offers
        .iter()
        .filter(|offer| !reconciled_handoffs.contains(offer.handoff_id.as_str()))
        .count();
    let provider_binding = state.provider_binding.as_ref().map(|binding| {
        let last_selection = state.provider_selections.last();
        let last_attempt = state.provider_attempt_outcomes.last();
        ProviderBindingView {
            mode: "governed_pool".to_string(),
            binding_id: binding.binding_id.clone(),
            participant_id: binding.participant_id.clone(),
            candidate_count: binding.ordered_target_ids.len(),
            failover_policy: enum_name(&binding.failover_policy),
            last_selection_id: last_selection.map(|selection| selection.selection_id.clone()),
            last_selected_target: last_selection
                .map(|selection| selection.selected_target_id.clone()),
            last_selected_model: last_selection
                .map(|selection| selection.selected_model_id.clone()),
            last_attempt_posture: last_attempt.map(|outcome| enum_name(&outcome.delivery)),
            delivery_indeterminate: last_attempt.is_some_and(|outcome| {
                outcome.delivery
                    == yai_core_engine::provider_governance::ProviderDeliveryClass::DeliveryIndeterminate
            }),
        }
    });
    let case = CaseView {
        case_id: state.case_id,
        tenant_id: state.tenant_id.unwrap_or_else(|| "unscoped".to_string()),
        generation: state.generation,
        lifecycle: enum_name(&state.lifecycle),
        execution,
        participants: state
            .participants
            .into_iter()
            .map(|participant| ParticipantView {
                participant_id: participant.participant_id,
                roles: participant.roles,
            })
            .collect(),
        provider: state.provider.map(|provider| ProviderView {
            participant_id: provider.participant_id,
            provider_id: if provider.provider_id.is_empty() {
                provider.provider_kind.clone()
            } else {
                provider.provider_id
            },
            provider_kind: provider.provider_kind,
            endpoint: provider.base_url,
            model_id: provider.model_id,
        }),
        provider_binding,
        resources: state
            .resources
            .into_iter()
            .map(|resource| ResourceView {
                resource_id: resource.attachment_id,
                kind: enum_name(&resource.kind),
                policy_id: resource.policy_id,
                policy_owner_participant_id: resource.policy_owner_participant_id,
                review_requirement: enum_name(&resource.review_requirement),
            })
            .collect(),
        workflow,
        policy_bindings: state.policy_bindings.len(),
        pending_reviews,
        unresolved_effects,
        finalized_effects,
        open_handoff_offers,
        handoff_acceptances: state.handoff_acceptances.len(),
        handoff_results: state.handoff_results.len(),
        handoff_reconciliations: state.handoff_reconciliations.len(),
    };
    Ok(CliData::Case {
        case: Box::new(case),
    })
}

fn participant_list(invocation: &Invocation) -> Result<CliData, CliError> {
    let state = load_case(invocation)?;
    Ok(CliData::Collection {
        columns: vec![
            "PARTICIPANT".to_string(),
            "ROLES".to_string(),
            "VIEWS".to_string(),
        ],
        rows: state
            .participants
            .into_iter()
            .map(|participant| {
                vec![
                    participant.participant_id,
                    participant.roles.join(","),
                    participant
                        .admitted_views
                        .into_iter()
                        .map(|view| format!("{}/{}", view.consumer, view.view_kind))
                        .collect::<Vec<_>>()
                        .join(","),
                ]
            })
            .collect(),
    })
}

fn resource_list(invocation: &Invocation) -> Result<CliData, CliError> {
    let state = load_case(invocation)?;
    Ok(CliData::Collection {
        columns: vec![
            "RESOURCE".to_string(),
            "KIND".to_string(),
            "POLICY".to_string(),
            "OWNER".to_string(),
            "REVIEW".to_string(),
        ],
        rows: state
            .resources
            .into_iter()
            .map(|resource| {
                vec![
                    resource.attachment_id,
                    enum_name(&resource.kind),
                    resource.policy_id,
                    resource.policy_owner_participant_id,
                    enum_name(&resource.review_requirement),
                ]
            })
            .collect(),
    })
}

fn load_case(invocation: &Invocation) -> Result<yai_core_engine::transition::CaseState, CliError> {
    let case_id = invocation
        .positional("case")
        .ok_or_else(|| CliError::usage("missing Case identity"))?;
    let authenticated = AuthenticatedPrincipal::authenticate_local()
        .map_err(|error| domain_error("authentication_failed", error))?;
    open_store()?
        .get_case_state_authorized(&authenticated, case_id)
        .map_err(|error| domain_error("case_not_found_or_not_visible", error))
}

fn completion(invocation: &Invocation) -> Result<CliData, CliError> {
    let shell = invocation.positional("shell").unwrap_or_default();
    let paths = super::registry::REGISTRY
        .iter()
        .filter(|descriptor| descriptor.visibility != Visibility::Removed)
        .map(|descriptor| descriptor.path.join(" "))
        .collect::<Vec<_>>();
    let script = match shell {
        "bash" => format!(
            "_yai_commands='{}'\ncomplete -W \"$_yai_commands\" yai\n",
            paths.join(" ")
        ),
        "zsh" => format!(
            "#compdef yai\n_arguments '1:command:({})'\n",
            paths.join(" ")
        ),
        "fish" => paths
            .iter()
            .map(|path| format!("complete -c yai -a '{}'\n", path))
            .collect(),
        _ => {
            return Err(CliError::usage(
                "completion shell must be bash, zsh, or fish",
            ))
        }
    };
    Ok(CliData::Completion {
        shell: shell.to_string(),
        script,
    })
}

fn execute_structured_legacy(
    operation_id: &str,
    invocation: &Invocation,
) -> Result<CliData, CliError> {
    if operation_id == "yai.runtime.serve" && !invocation.json {
        crate::command_adapters::dispatch_operation(operation_id, &invocation.legacy_args())
            .map_err(|error| domain_error(classify_domain_code(&error), error))?;
        return Ok(CliData::AlreadyRendered);
    }
    let mut args = invocation.legacy_args();
    let native_json = invocation.json
        && ((operation_id.starts_with("yai.workflow.") && operation_id != "yai.workflow.input")
            || operation_id.starts_with("yai.case.handoff."));
    if native_json {
        args.push("--json".to_string());
    }
    let output =
        capture_stdout(|| crate::command_adapters::dispatch_operation(operation_id, &args))
            .map_err(|error| domain_error(classify_domain_code(&error), error))?;
    if native_json {
        let value = serde_json::from_str(output.trim())
            .map_err(|error| domain_error("invalid_handler_json", error.to_string()))?;
        return Ok(CliData::NativeJson { value });
    }
    let lines = output.lines().map(str::to_string).collect::<Vec<_>>();
    let mut fields = Vec::new();
    for line in &lines {
        if let Some((name, value)) = line.split_once(':') {
            fields.push(field(humanize(name), value.trim()));
        }
    }
    let message = fields
        .first()
        .map(|field| format!("{} {}", field.name, field.value))
        .unwrap_or_else(|| "Command completed".to_string());
    Ok(if !invocation.json || fields.is_empty() {
        CliData::Compatibility { lines }
    } else {
        CliData::Message { message, fields }
    })
}

#[cfg(unix)]
fn capture_stdout<F>(operation: F) -> Result<String, String>
where
    F: FnOnce() -> Result<(), String>,
{
    use std::fs::File;
    use std::os::fd::FromRawFd;

    unsafe extern "C" {
        fn pipe(fds: *mut i32) -> i32;
        fn dup(fd: i32) -> i32;
        fn dup2(oldfd: i32, newfd: i32) -> i32;
        fn close(fd: i32) -> i32;
    }
    std::io::stdout()
        .flush()
        .map_err(|error| error.to_string())?;
    let mut fds = [0_i32; 2];
    if unsafe { pipe(fds.as_mut_ptr()) } != 0 {
        return Err(format!(
            "stdout_capture_pipe_failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    let saved = unsafe { dup(1) };
    if saved < 0 || unsafe { dup2(fds[1], 1) } < 0 {
        unsafe {
            close(fds[0]);
            close(fds[1]);
        }
        return Err(format!(
            "stdout_capture_redirect_failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    unsafe {
        close(fds[1]);
    }
    let reader = std::thread::spawn(move || {
        let mut file = unsafe { File::from_raw_fd(fds[0]) };
        let mut output = String::new();
        file.read_to_string(&mut output)
            .map(|_| output)
            .map_err(|error| error.to_string())
    });
    let result = operation();
    let flush_result = std::io::stdout().flush().map_err(|error| error.to_string());
    unsafe {
        dup2(saved, 1);
        close(saved);
    }
    let output = reader
        .join()
        .map_err(|_| "stdout_capture_reader_panicked".to_string())??;
    flush_result?;
    result?;
    Ok(output)
}

#[cfg(not(unix))]
fn capture_stdout<F>(operation: F) -> Result<String, String>
where
    F: FnOnce() -> Result<(), String>,
{
    operation()?;
    Ok(String::new())
}

fn open_store() -> Result<LmdbRecordStore, CliError> {
    let status = LmdbRecordStore::status(record_store_path());
    if status.status != RecordStoreStatusKind::Ready {
        let mut error = CliError::domain("not_initialized", "YAI environment is not initialized");
        error.remediation =
            Some("run `yai init --tenant <TENANT> --organization <ORGANIZATION>`".to_string());
        return Err(error);
    }
    LmdbRecordStore::open(record_store_path())
        .map_err(|error| domain_error("storage_unavailable", error))
}

fn classify_domain_code(error: &str) -> &'static str {
    if error.contains("not_visible") || error.contains("not found") {
        "not_found"
    } else if error.contains("denied")
        || error.contains("requires_owner")
        || error.contains("unauthorized")
    {
        "denied"
    } else if error.contains("conflict")
        || error.contains("generation")
        || error.contains("already")
    {
        "conflict"
    } else if error.contains("provider") {
        "provider_unavailable"
    } else if error.contains("review") {
        "review_state"
    } else if error.contains("resource_temporarily_owned") {
        "resource_busy"
    } else {
        "operation_failed"
    }
}

fn domain_error(code: &'static str, error: impl Into<String>) -> CliError {
    CliError::domain(code, error)
}

fn map_fields(fields: BTreeMap<String, String>) -> Vec<Field> {
    fields
        .into_iter()
        .map(|(name, value)| Field { name, value })
        .collect()
}

fn field(name: impl Into<String>, value: impl Into<String>) -> Field {
    Field {
        name: name.into(),
        value: value.into(),
    }
}

fn humanize(value: &str) -> String {
    value.trim().replace('_', " ")
}

fn enum_name<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".to_string())
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

fn record_store_path() -> PathBuf {
    yai_home().join("store").join("lmdb")
}

fn checkpoint_path(case_id: &str) -> PathBuf {
    yai_home().join("run").join("case-runtime").join(format!(
        "{}.json",
        yai_core_engine::context::stable_digest(case_id)
    ))
}

fn case_execution_posture(case_id: &str) -> String {
    let path = checkpoint_path(case_id);
    if !path.is_file() {
        return "never_started".to_string();
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
        .and_then(|value| {
            value
                .get("status")
                .and_then(|status| status.as_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "runtime_state_unavailable".to_string())
}

fn runtime_posture(home: &std::path::Path) -> &'static str {
    if home.join("run/runtime-instance.json").is_file() {
        "state_available"
    } else {
        "not_running"
    }
}
