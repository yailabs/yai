use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use yai_core_engine::security::AuthenticatedPrincipal;
use yai_core_engine::store::lmdb::{LmdbRecordStore, RecordStoreStatusKind};
use yai_application::{LocalApplication, OperationRequest, ResultState, APPLICATION_PROTOCOL};

use super::output::{
    CaseView, CliData, CliError, Field, ParticipantView, ProviderBindingView, ProviderView,
    ResourceView, WorkflowView,
};
use super::parser::Invocation;
use super::registry::{registry_digest, Visibility, REGISTRY_SCHEMA};

pub(crate) fn execute(invocation: &Invocation) -> Result<CliData, CliError> {
    match invocation.descriptor.operation_id {
        operation if operation.starts_with("yai.host.") => host_operation(operation, invocation),
        "yai.application.capabilities" => application_operation(
            "application.capabilities",
            serde_json::json!({}),
        ),
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
        "yai.case.history" | "yai.case.verify" => canonical_case_inspection(invocation),
        "yai.case.context.compile" => working_state_compilation(invocation),
        "yai.case.context.expand" => working_state_expansion(invocation),
        "yai.case.context.refresh" => working_state_refresh(invocation),
        "yai.case.context.ambient" => ambient_consumer_refresh(invocation),
        "yai.case.as_of" | "yai.case.experience" | "yai.case.recall" => historical_case_inspection(invocation),
        operation if operation.starts_with("yai.case.knowledge.") => knowledge_inspection(invocation),
        "yai.case.open" | "yai.case.workbench" => {
            if invocation.json {
                return Err(CliError::usage("interactive workbench has no JSON stream; use the structured Case inspection commands"));
            }
            crate::command_adapters::dispatch_operation(
                invocation.descriptor.operation_id,
                &invocation.legacy_args(),
            )
            .map_err(|error| domain_error(classify_domain_code(&error), error))?;
            Ok(CliData::AlreadyRendered)
        }
        "yai.case.capabilities" => {
            application_operation(
                "case.capabilities",
                serde_json::json!({
                    "case_ref": invocation.positional("case")
                        .ok_or_else(|| CliError::usage("Case is required"))?,
                    "participant_ref": invocation.flag("--participant")
                        .ok_or_else(|| CliError::usage("--participant is required"))?,
                }),
            )
        }
        "yai.case.cognitive.frontier" => decision_frontier(invocation),
        "yai.case.cognitive.decision_request" => decision_request(invocation),
        operation_id @ ("yai.case.resource.import" | "yai.case.resource.request") => {
            crate::command_adapters::resource_application_command(
                operation_id,
                &invocation.legacy_args(),
            )
            .map(|value| CliData::NativeJson { value })
            .map_err(|error| domain_error(classify_domain_code(&error), error))
        }
        operation if operation.starts_with("yai.case.sources.") => {
            let value = crate::command_adapters::source_application_command(
                operation,
                &invocation.legacy_args(),
            )
            .map_err(|error| domain_error(classify_domain_code(&error), error))?;
            if invocation.json {
                Ok(CliData::NativeJson { value })
            } else {
                crate::command_adapters::render_source_inventory(&value);
                Ok(CliData::AlreadyRendered)
            }
        }
        "yai.case.show" if invocation.compatibility_syntax && !invocation.json => {
            crate::command_adapters::dispatch_operation("yai.case.show", &invocation.legacy_args())
                .map_err(|error| domain_error(classify_domain_code(&error), error))?;
            Ok(CliData::AlreadyRendered)
        }
        "yai.case.show" => case_show(invocation),
        operation_id
            if operation_id.starts_with("yai.case.memory.")
                || operation_id.starts_with("yai.case.conversation.")
                || operation_id.starts_with("yai.case.cognitive.")
                || operation_id.starts_with("yai.provider.suitability.") =>
        {
            execute_structured_legacy(operation_id, invocation)
        }
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

fn host_operation(operation: &str, invocation: &Invocation) -> Result<CliData, CliError> {
    let home = yai_home();
    let executable = std::env::current_exe()
        .map_err(|error| domain_error("host_executable_unavailable", error.to_string()))?;
    let value = match operation {
        "yai.host.status" => serde_json::to_value(
            yai_host::observe(&home).map_err(|error| domain_error("host_status_failed", error))?,
        ),
        "yai.host.start" => serde_json::to_value(
            yai_host::start(&home, &executable, &["host", "serve"])
                .map_err(|error| domain_error("host_start_failed", error))?,
        ),
        "yai.host.stop" => serde_json::to_value(
            yai_host::stop(&home).map_err(|error| domain_error("host_stop_failed", error))?,
        ),
        "yai.host.restart" => serde_json::to_value(
            yai_host::restart(&home, &executable, &["host", "serve"])
                .map_err(|error| domain_error("host_restart_failed", error))?,
        ),
        "yai.host.logs" => {
            let limit = invocation
                .flag("--lines")
                .map(str::parse::<usize>)
                .transpose()
                .map_err(|_| CliError::usage("--lines must be an integer"))?
                .unwrap_or(100);
            serde_json::to_value(serde_json::json!({
                "lines": yai_host::recent_logs(&home, limit)
                    .map_err(|error| domain_error("host_logs_failed", error))?
            }))
        }
        "yai.host.serve" => {
            if invocation.json {
                return Err(CliError::usage(
                    "the foreground Host has no finite JSON result; use `yai host status --json`",
                ));
            }
            yai_host::serve(&home).map_err(|error| domain_error("host_serve_failed", error))?;
            return Ok(CliData::AlreadyRendered);
        }
        _ => return Err(CliError::internal("unknown Host lifecycle operation")),
    }
    .map_err(|error| domain_error("host_result_encode_failed", error.to_string()))?;
    Ok(CliData::NativeJson { value })
}

fn application_operation(operation_ref: &str, input: serde_json::Value) -> Result<CliData, CliError> {
    let result = LocalApplication::from_yai_home(yai_home()).call(OperationRequest {
        protocol: APPLICATION_PROTOCOL.to_string(),
        operation_ref: operation_ref.to_string(),
        correlation_ref: format!("cli:{operation_ref}:{}", std::process::id()),
        input,
    });
    if result.result_state == ResultState::Success {
        return Ok(CliData::NativeJson { value: result.data.unwrap_or(serde_json::Value::Null) });
    }
    let detail = result.error
        .map(|error| format!("{}: {}", error.code, error.safe_message))
        .unwrap_or_else(|| format!("application operation returned {:?}", result.result_state));
    Err(domain_error("application_operation_failed", detail))
}

fn decision_frontier(invocation: &Invocation) -> Result<CliData, CliError> {
    let working = working_state_file(invocation)?;
    let case_ref = invocation.positional("case").ok_or_else(|| CliError::usage("Case is required"))?;
    if working.case_id() != case_ref {
        return Err(CliError::usage("working state and Case do not match"));
    }
    let max_candidates = invocation.flag("--max-candidates")
        .map(str::parse::<usize>).transpose()
        .map_err(|_| CliError::usage("--max-candidates must be an integer"))?
        .unwrap_or(8);
    application_operation("decision.frontier.prepare", serde_json::json!({
        "working_state": working,
        "max_candidates": max_candidates,
    }))
}

fn decision_request(invocation: &Invocation) -> Result<CliData, CliError> {
    let working = working_state_file(invocation)?;
    let case_ref = invocation.positional("case").ok_or_else(|| CliError::usage("Case is required"))?;
    if working.case_id() != case_ref {
        return Err(CliError::usage("working state and Case do not match"));
    }
    let frontier_path = invocation.flag("--frontier-file")
        .ok_or_else(|| CliError::usage("--frontier-file is required"))?;
    let frontier_bytes = fs::read(frontier_path)
        .map_err(|error| domain_error("frontier_file_unavailable", error.to_string()))?;
    if frontier_bytes.len() > 4 * 1024 * 1024 {
        return Err(CliError::usage("frontier JSON exceeds 4 MiB"));
    }
    let frontier_value: serde_json::Value = serde_json::from_slice(&frontier_bytes)
        .map_err(|error| domain_error("frontier_file_invalid", error.to_string()))?;
    let frontier = frontier_value.pointer("/data/value/frontier")
        .or_else(|| frontier_value.get("frontier"))
        .unwrap_or(&frontier_value).clone();
    let max_candidates = frontier.pointer("/request/max_candidates")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| domain_error("frontier_file_invalid", "frontier request bound missing"))?;
    let max_result_bytes = invocation.flag("--max-result-bytes")
        .map(str::parse::<usize>).transpose()
        .map_err(|_| CliError::usage("--max-result-bytes must be an integer"))?
        .unwrap_or(65_536);
    let max_compute_millis = invocation.flag("--max-compute-ms")
        .map(str::parse::<u64>).transpose()
        .map_err(|_| CliError::usage("--max-compute-ms must be an integer"))?
        .unwrap_or(5_000);
    application_operation("decision.request.prepare", serde_json::json!({
        "working_state": working,
        "frontier": frontier,
        "decision_kind": invocation.flag("--decision-kind")
            .ok_or_else(|| CliError::usage("--decision-kind is required"))?,
        "budget": {
            "max_candidates": max_candidates,
            "max_result_bytes": max_result_bytes,
            "max_compute_millis": max_compute_millis,
        },
    }))
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
    let mut effective = Invocation {
        descriptor: invocation.descriptor,
        positionals: invocation.positionals.clone(),
        flags: invocation.flags.clone(),
        json: invocation.json,
        compatibility_syntax: invocation.compatibility_syntax,
    };
    if invocation.flag("--tenant").is_none() || invocation.flag("--organization").is_none() {
        if invocation.json {
            return Err(CliError::usage("structured init requires --tenant and --organization; run `yai init` in a terminal for guided setup"));
        }
        let (tenant, organization) = crate::command_adapters::guided_initialization(
            invocation.flag("--tenant"),
            invocation.flag("--organization"),
        )
        .map_err(|e| domain_error("initialization_setup", e))?;
        effective.flags.insert("--tenant", vec![tenant]);
        effective.flags.insert("--organization", vec![organization]);
    }
    let home = yai_home();
    fs::create_dir_all(&home).map_err(|error| {
        CliError::domain(
            "initialization_failed",
            format!("cannot prepare YAI_HOME: {error}"),
        )
    })?;
    #[cfg(unix)]
    fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).map_err(|error| {
        CliError::domain(
            "initialization_failed",
            format!("cannot make YAI_HOME private: {error}"),
        )
    })?;
    for relative in ["run", "store", "log", "tmp", "cases", "sockets", "config"] {
        fs::create_dir_all(home.join(relative)).map_err(|error| {
            CliError::domain(
                "initialization_failed",
                format!("cannot prepare YAI_HOME: {error}"),
            )
        })?;
    }
    execute_structured_legacy("yai.init", &effective)
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

fn working_state_compilation(invocation: &Invocation) -> Result<CliData, CliError> {
    use yai_core_engine::semantic_state::{CompilationRequest, SemanticScope, WorkingStateRequest};
    use yai_core_engine::semantic_state::historical::HistoricalCoordinate;
    let case = load_case(invocation)?;
    let auth = AuthenticatedPrincipal::authenticate_local()
        .map_err(|e| domain_error("authentication_failed", e))?;
    let linked: Vec<_> = case.principal_participant_links.iter()
        .filter(|l| l.principal_id == auth.projected_principal_id()).collect();
    let participant = match invocation.flag("--participant") {
        Some(p) => p.to_string(),
        None if linked.len() == 1 => linked[0].participant_id.clone(),
        _ => return Err(CliError::usage("select your linked Participant with --participant")),
    };
    let number = |flag, default| -> Result<usize, CliError> {
        invocation.flag(flag).map(|s| s.parse().map_err(|_| CliError::usage(format!("{flag} must be an integer")))).unwrap_or(Ok(default))
    };
    let mut scope = SemanticScope::model(participant, yai_core_engine::context::ProjectionPurpose::Inspection);
    scope.max_items = number("--limit", scope.max_items)?;
    let mut recall_bounds = yai_core_engine::memory_hierarchy::recall::RecallBounds::default();
    recall_bounds.candidates = number("--candidates", recall_bounds.candidates)?;
    let at = invocation.flag("--at").map(|at| {
        if at.starts_with("transition:") { Ok(HistoricalCoordinate::Transition(at.into())) }
        else { at.parse().map(HistoricalCoordinate::Generation).map_err(|_| CliError::usage("--at requires an exact generation or Transition")) }
    }).transpose()?;
    let request = WorkingStateRequest {
        case_id: case.case_id, expected_generation: case.generation, at, recall_query: None,
        compilation: CompilationRequest {
            scope, intent: invocation.positionals["intent"].clone(),
            output_contract_id: yai_core_engine::context::InvocationOutputContract::NaturalLanguage.contract_id(),
            max_semantic_units: number("--units", 32768)?, max_derived_items: number("--resident-groups", 16)?,
            resource_refs: invocation.flag("--resource").map(str::to_string).into_iter().collect(),
            required_refs: invocation.flag("--require").map(str::to_string).into_iter().collect(),
            previous_item_ids: vec![], view_selection_id: None,
        },
        recall_required_refs: invocation.flag("--ref").map(str::to_string).into_iter().collect(),
        recall_bounds, max_output_bytes: number("--bytes", 1024 * 1024)?,
    };
    let content = yai_core_engine::conversation::ConversationContentStore::open_existing(&yai_home()).ok();
    let store = open_store()?;
    let result = if invocation.flags.contains_key("--paged") {
        store.compile_pageable_working_state_authorized(&auth, request, content.as_ref())
    } else { store.compile_working_state_authorized(&auth, request, content.as_ref()) }
        .map_err(|e| domain_error("working_state_unavailable", e))?;
    let lowering_started = std::time::Instant::now();
    let projection = invocation.flags.contains_key("--projection").then(|| result.lower_context())
        .transpose().map_err(|e| domain_error("working_lowering_unavailable", e))?;
    let lowering_us = lowering_started.elapsed().as_micros();
    if invocation.json {
        let mut value = serde_json::to_value(&result).map_err(|e| domain_error("working_encoding", e.to_string()))?;
        if let Some(projection) = projection {
            value["projection"] = serde_json::to_value(projection).map_err(|e| domain_error("working_encoding", e.to_string()))?;
            value["measurements"]["compatibility_lowering_us"] = serde_json::json!(lowering_us);
        }
        return Ok(CliData::NativeJson { value });
    }
    let w = &result.working_state;
    println!("CASE WORKING STATE {}", w.id());
    println!("Task: {}", w.request().intent);
    println!("Recall: {} | present execution authority; recalled history never grants permission", w.recall().unwrap().recall_id);
    for entry in w.entries() {
        println!("{} [{:?}]", entry.entry_id, entry.posture);
        println!("{}", serde_json::to_string_pretty(&entry.value).map_err(|e| domain_error("working_encoding", e.to_string()))?);
        println!("Backing: {:?}", entry.provenance);
    }
    println!("Bounds/omissions: {:?}", w.bounds());
    println!("Derived W only; no model call, no W -> E, no canonical mutation.");
    Ok(CliData::AlreadyRendered)
}

fn working_state_file(invocation: &Invocation) -> Result<yai_core_engine::semantic_state::SemanticWorkingState, CliError> {
    let file = fs::File::open(invocation.flag("--working-file").ok_or_else(|| CliError::usage("--working-file required"))?)
        .map_err(|e| domain_error("working_file_unavailable", e.to_string()))?;
    let mut bytes = Vec::new();
    file.take(4 * 1024 * 1024 + 1).read_to_end(&mut bytes)
        .map_err(|e| domain_error("working_file_unavailable", e.to_string()))?;
    if bytes.len() > 4 * 1024 * 1024 { return Err(CliError::usage("working JSON exceeds 4 MiB")); }
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| domain_error("working_file_invalid", e.to_string()))?;
    let body = value.pointer("/data/value/working_state").or_else(|| value.get("working_state")).unwrap_or(&value);
    serde_json::from_value(body.clone()).map_err(|e| domain_error("working_file_invalid", e.to_string()))
}

fn working_state_refresh(invocation: &Invocation) -> Result<CliData, CliError> {
    use yai_core_engine::semantic_state::working_recall::{WorkingRefreshRequest, WorkingRefreshBudget};
    let auth = AuthenticatedPrincipal::authenticate_local()
        .map_err(|e| domain_error("authentication_failed", e))?;
    let base = working_state_file(invocation)?;
    let mut request = WorkingRefreshRequest::new(&base);
    request.case_id = invocation.positionals["case"].clone();
    if let Some(id) = invocation.flag("--participant") { request.participant_id = id.into(); }
    if let Some(id) = invocation.flag("--base-id") { request.base_working_state_id = id.into(); }
    let number = |flag, default| -> Result<usize, CliError> {
        invocation.flag(flag).map(|s| s.parse().map_err(|_| CliError::usage(format!("{flag} must be an integer")))).unwrap_or(Ok(default))
    };
    if ["--units", "--limit", "--bytes"].iter().any(|f| invocation.flag(f).is_some()) {
        request.budget = Some(WorkingRefreshBudget {
            max_items: number("--limit", base.request().scope.max_items)?,
            max_semantic_units: number("--units", base.request().max_semantic_units)?,
            max_output_bytes: number("--bytes", base.recall().ok_or_else(|| CliError::usage("Recall-aware W required"))?.request.max_output_bytes)?,
        });
    }
    let content = yai_core_engine::conversation::ConversationContentStore::open_existing(&yai_home()).ok();
    let result = open_store()?.refresh_working_state_authorized(&auth, &base, request, content.as_ref())
        .map_err(|e| domain_error("working_refresh_unavailable", e))?;
    let started = std::time::Instant::now();
    let projection = invocation.flags.contains_key("--projection").then(|| result.lower_context())
        .transpose().map_err(|e| domain_error("working_lowering_unavailable", e))?;
    let lowering_us = started.elapsed().as_micros();
    if invocation.json {
        let mut value = serde_json::to_value(&result).map_err(|e| domain_error("working_encoding", e.to_string()))?;
        if let Some(p) = projection {
            value["projection"] = serde_json::to_value(p).map_err(|e| domain_error("working_encoding", e.to_string()))?;
            value["measurements"]["compatibility_lowering_us"] = serde_json::json!(lowering_us);
        }
        return Ok(CliData::NativeJson { value });
    }
    println!("SEMANTIC WORKING STATE REFRESH [{:?}]", result.posture);
    println!("Task preserved: {}", result.working_state.request().intent);
    println!("Current W: {}\nCurrent Recall: {}", result.working_state.id(), result.working_state.recall().unwrap().recall_id);
    println!("Full current requalification; predecessor is not authority.\nAssessment: {:?}", result.assessment);
    for entry in result.working_state.entries() {
        println!("{} [{:?}]", entry.entry_id, entry.posture);
    }
    println!("Bounds/omissions: {:?}\nNo new prompt, canonical mutation or model call.", result.working_state.bounds());
    Ok(CliData::AlreadyRendered)
}

fn ambient_consumer_refresh(invocation: &Invocation) -> Result<CliData, CliError> {
    use yai_core_engine::semantic_state::working_recall::{
        ActiveSemanticConsumerKind as Consumer, AmbientSemanticChange,
        AmbientSemanticChangeKind as Change,
    };
    let base = working_state_file(invocation)?;
    if base.case_id() != invocation.positionals["case"] {
        return Err(CliError::usage("working state and Case do not match"));
    }
    let consumer = match invocation.flag("--consumer") {
        Some("conversation") => Consumer::Conversation,
        Some("workflow") => Consumer::Workflow,
        _ => return Err(CliError::usage("--consumer requires conversation or workflow")),
    };
    let kind = match invocation.flag("--change-kind") {
        Some("transition") => Change::CanonicalTransition,
        Some("source") => Change::SourceQualification,
        Some("authority") => Change::AuthorityOrDisclosure,
        Some("backing") => Change::BackingAvailability,
        Some("recovery") => Change::ConsumerRecovery,
        Some("other") => Change::Other,
        _ => return Err(CliError::usage(
            "--change-kind requires transition, source, authority, backing, recovery or other",
        )),
    };
    let result = crate::command_adapters::refresh_active_semantic_consumer(
        base.case_id(),
        invocation.flag("--operator"),
        &base,
        consumer,
        invocation
            .flag("--consumer-ref")
            .ok_or_else(|| CliError::usage("--consumer-ref required"))?,
        vec![AmbientSemanticChange {
            kind,
            reference: invocation
                .flag("--change-ref")
                .ok_or_else(|| CliError::usage("--change-ref required"))?
                .into(),
        }],
    )
        .map_err(|error| domain_error("ambient_refresh_unavailable", error))?;
    if invocation.json {
        return Ok(CliData::NativeJson {
            value: serde_json::to_value(result)
                .map_err(|error| domain_error("ambient_refresh_encoding", error.to_string()))?,
        });
    }
    println!("ACTIVE SEMANTIC CONSUMER [{:?}]", result.freshness);
    println!("Task: {}", result.request.task_id);
    println!("Consumer: {:?} {}", result.request.consumer, result.request.consumer_ref);
    println!("Reason: {}", result.reason);
    if let Some(id) = result.current_working_state_id {
        println!("Current W: {id}");
    }
    println!(
        "Coalesced changes: {}; no prompt, provider call or canonical mutation.",
        result.measurements.coalesced_changes
    );
    Ok(CliData::AlreadyRendered)
}

fn working_state_expansion(invocation: &Invocation) -> Result<CliData, CliError> {
    use yai_core_engine::semantic_state::paging::{PageRequest, PageAction};
    let auth = AuthenticatedPrincipal::authenticate_local()
        .map_err(|e| domain_error("authentication_failed", e))?;
    let base = working_state_file(invocation)?;
    let mut request = PageRequest::new(&base, vec![invocation.flag("--ref").ok_or_else(|| CliError::usage("--ref required"))?.into()]);
    request.case_id = invocation.positionals["case"].clone();
    if let Some(id) = invocation.flag("--participant") { request.participant_id = id.into(); }
    if let Some(id) = invocation.flag("--base-id") { request.base_working_state_id = id.into(); }
    if invocation.flags.contains_key("--page-out") { request.action = PageAction::PageOut; }
    let number = |flag, default| -> Result<usize, CliError> {
        invocation.flag(flag).map(|s| s.parse().map_err(|_| CliError::usage(format!("{flag} must be an integer")))).unwrap_or(Ok(default))
    };
    request.bounds.semantic_units = number("--page-units", request.bounds.semantic_units)?;
    request.bounds.bytes = number("--page-bytes", request.bounds.bytes)?;
    request.bounds.events = number("--page-items", request.bounds.events)?;
    let content = yai_core_engine::conversation::ConversationContentStore::open_existing(&yai_home()).ok();
    let result = open_store()?.page_working_state_authorized(&auth, &base, request, content.as_ref())
        .map_err(|e| domain_error("semantic_page_unavailable", e))?;
    let projection = invocation.flags.contains_key("--projection").then(|| result.lower_context())
        .transpose().map_err(|e| domain_error("working_lowering_unavailable", e))?;
    if invocation.json {
        let mut value = serde_json::to_value(&result).map_err(|e| domain_error("page_encoding", e.to_string()))?;
        if let Some(p) = projection { value["projection"] = serde_json::to_value(p).map_err(|e| domain_error("page_encoding", e.to_string()))?; }
        return Ok(CliData::NativeJson { value });
    }
    println!("SEMANTIC PAGE {}", result.page.page_id);
    println!("Task: {}\nExpanded W: {}", result.working_state.request().intent, result.working_state.id());
    println!("Closure: {}\nCurrent authority revalidated; no global Recall discovery", result.page.closure);
    for (number, group) in result.page.groups.iter().enumerate() {
        println!("Group {}: {} events, {} experience relations; closure complete={}",
            number + 1, group.events.len(), group.relations.len(), group.closure_complete);
        for event in &group.events {
            println!("  Historical event {}: {:?}", event.event.transition_id, event.event.object_refs);
        }
        if let Some(d) = &group.documentary {
            for source in &d.sources {
                println!("  Source {} @ {} [{}]", source.source.path, source.source.revision_id, source.source.id);
            }
            for unit in &d.units {
                println!("  {:?} {}: {}", unit.unit.posture, unit.unit.id,
                    unit.unit.text.chars().take(160).collect::<String>());
            }
            println!("  Documentary relations: {}; unresolved disagreements: {} (no winner)",
                d.relations.len(), d.contradictions.len());
        }
    }
    let resident = result.working_state.resident_page_references();
    for r in result.working_state.page_references() {
        println!("{} [{}] {}", r.reference_id, if resident.contains(&r.reference_id) { "resident" } else { "deferred; authorization required on demand" }, r.family);
    }
    println!("Evicted optional groups: {:?}\nNo canonical mutation or model call.", result.evicted_references);
    Ok(CliData::AlreadyRendered)
}

fn historical_case_inspection(invocation: &Invocation) -> Result<CliData, CliError> {
    use yai_core_engine::semantic_state::historical::{HistoricalCoordinate, HistoricalRequest};
    let case = load_case(invocation)?;
    let authenticated = AuthenticatedPrincipal::authenticate_local()
        .map_err(|e| domain_error("authentication_failed", e))?;
    let participant = match invocation.flag("--participant") {
        Some(p) => p.to_string(),
        None => {
            let linked: Vec<_> = case
                .principal_participant_links
                .iter()
                .filter(|l| l.principal_id == authenticated.projected_principal_id())
                .collect();
            if linked.len() != 1 {
                return Err(CliError::usage(
                    "select your linked Participant with --participant",
                ));
            }
            linked[0].participant_id.clone()
        }
    };
    let recall = invocation.descriptor.operation_id == "yai.case.recall";
    let at = invocation
        .positionals
        .get("coordinate")
        .map(String::as_str)
        .or_else(|| if recall { Some(invocation.flag("--at").unwrap_or("current")) } else { None })
        .ok_or_else(|| CliError::usage("use: yai case as-of CASE GENERATION_OR_TRANSITION"))?;
    let experience = invocation.descriptor.operation_id == "yai.case.experience";
    let coordinate = if (experience || recall) && at == "current" {
        HistoricalCoordinate::Generation(case.generation)
    } else if at.starts_with("transition:") {
        HistoricalCoordinate::Transition(at.to_string())
    } else {
        HistoricalCoordinate::Generation(at.parse().map_err(|_| CliError::usage("coordinate must be an exact generation or Transition ID; wall-clock queries are unsupported"))?)
    };
    let mut request = HistoricalRequest::inspection(coordinate, participant);
    if let Some(limit) = invocation.flag("--limit") {
        request.max_items = limit
            .parse()
            .map_err(|_| CliError::usage("--limit must be an integer"))?;
    }
    let content =
        yai_core_engine::conversation::ConversationContentStore::open_existing(&yai_home()).ok();
    if recall {
        use yai_core_engine::memory_hierarchy::recall::RecallRequest;
        let mut recall_request = RecallRequest::integrated(&case.case_id, case.generation,
            &request.participant_id, invocation.positionals.get("query").ok_or_else(|| CliError::usage("use: yai case recall CASE QUERY"))?);
        recall_request.at = request.coordinate;
        if let Some(reference) = invocation.flag("--ref") { recall_request.required_refs.push(reference.into()); }
        if invocation.flag("--limit").is_some() { recall_request.bounds.events = request.max_items; }
        if let Some(value) = invocation.flag("--candidates") { recall_request.bounds.candidates = value.parse().map_err(|_| CliError::usage("--candidates must be an integer"))?; }
        if let Some(value) = invocation.flag("--hops") { recall_request.bounds.expansion_depth = value.parse().map_err(|_| CliError::usage("--hops must be an integer"))?; }
        let result = open_store()?.recall_trace_authorized(&authenticated, recall_request, content.as_ref())
            .map_err(|e| domain_error("recall_unavailable", e))?;
        if invocation.json {
            return Ok(CliData::NativeJson { value: serde_json::to_value(result).map_err(|e| domain_error("recall_encoding", e.to_string()))? });
        }
        let t = &result.trace;
        println!("CASE RECALL {} @{}", t.request.case_id, t.generation);
        println!("Trace: {}", t.trace_id);
        println!("Query: {} | Participant: {}", t.request.query, t.request.participant_id);
        println!("Qualified knowledge / experience / current state; not authority or automatic model context. Source closure: {}", if t.closure_complete { "complete" } else { "INCOMPLETE" });
        if let Some(d) = &t.documentary {
            for s in &d.sources {
                println!("DOCUMENT {} / {} @ {}: {:?}; admitted @{}; applicable at cut: {}; current revision: {}", s.source.logical_name, s.source.path, s.source.revision_id, s.source.status, s.admitted_at_generation, s.applicable_at_cut, s.source.current_revision);
                for u in d.units.iter().filter(|u| u.unit.source == s.source.id) {
                    println!("  {:?}: {}\n    {} at {:?}; selected: {:?}", u.unit.posture, u.unit.text, u.unit.id, u.unit.location, u.reasons);
                }
            }
            for r in &d.cross_references { println!("DOCUMENT REFERENCE {} -> {}: {}", r.unit, r.event, r.posture); }
            for c in &d.contradictions { println!("DISAGREEMENT {} / {}: {} (no winner)", c.entity, c.predicate, c.posture); }
        }
        for s in &t.segments {
            println!("SEGMENT {} ({})", s.segment_id, s.grouping);
            for id in &s.events {
                if let Some(e) = t.events.iter().find(|e| &e.event.transition_id == id) {
                    println!("  @{} {} — {}", e.event.recorded_generation, id, e.label);
                    println!("    {:?}; {}; selected: {:?}", e.event.posture, e.validity_at_cut, e.reasons);
                }
            }
        }
        for a in &t.assertions { println!("ASSERTION {} {:?} {} = {:?}; {:?}/{:?}; backing: {}", a.assertion.assertion_id, a.assertion.subject, a.assertion.predicate, a.assertion.value, a.assertion.epistemic_class, a.assertion.lifecycle, a.source_events.join(", ")); }
        for r in &t.relations { println!("RELATION {} --{:?}/{:?}--> {}", r.from_event, r.kind, r.posture, r.to_event); }
        for s in &t.source_closure { println!("SOURCE {}: {}", s.source_ref, s.posture); }
        println!("Omitted candidate groups: {}; expansion depth limited: {}; semantic units: {}; bytes: {}", t.omitted_candidates, t.expansion_stopped_at_depth, result.measurements.semantic_units, result.measurements.output_bytes);
        println!("Recording order is not causality; a recalled claim is not an admitted fact.");
        return Ok(CliData::AlreadyRendered);
    }
    if experience {
        let mut query = yai_core_engine::graph::experience::ExperienceQuery::default();
        query.from = invocation.flag("--from").map(str::to_string);
        query.to = invocation.flag("--to").map(str::to_string);
        query.include_recording_order = invocation.flags.contains_key("--recording-order");
        query.max_events = request.max_items;
        if let Some(hops) = invocation.flag("--hops") {
            query.max_hops = hops.parse().map_err(|_| CliError::usage("--hops must be an integer"))?;
        }
        // Independent bounded source qualification and output budgets. This is
        // an inspection profile, never provider token or semantic-W admission.
        request.max_items = 4096;
        request.max_bytes = 16_777_216;
        let view = open_store()?.experience_view_authorized(&authenticated, &case.case_id, request, query, content.as_ref())
            .map_err(|e| domain_error("experience_view_unavailable", e))?;
        if !invocation.json {
            println!("CASE EXPERIENCE {} @{} ({})", view.case_id, view.generation, view.result);
            println!("View: {}", view.view_id);
            println!("Participant: {} | derived, read-only; not authority or Recall", view.participant_id);
            for e in &view.events {
                println!("@{} {} [{}] {:?}; recorded={:?} observed={:?} occurrence={:?}; source_closed={}",
                    e.recorded_generation, e.transition_id, e.kind, e.posture, e.recorded_at_unix_ms,
                    e.observed_at_unix_ms, e.occurred_at_unix_ms, e.source_closed);
                println!("  objects: {}", e.object_refs.join(", "));
            }
            for r in &view.relations {
                println!("{} --{:?}/{:?}--> {} (known @{})", r.from_event, r.kind, r.posture, r.to_event, r.known_at_generation);
                for s in &r.sources { println!("  backing: {} :: {}", s.transition_id, s.field); }
            }
            println!("Recording order is not causality. No qualified path does not distinguish absent from undisclosed evidence.");
            return Ok(CliData::AlreadyRendered);
        }
        return Ok(CliData::NativeJson { value: serde_json::to_value(view)
            .map_err(|e| domain_error("experience_encoding", e.to_string()))? });
    }
    let view = open_store()?
        .historical_semantic_view_authorized(
            &authenticated,
            &case.case_id,
            request,
            content.as_ref(),
        )
        .map_err(|e| domain_error("historical_view_unavailable", e))?;
    Ok(CliData::NativeJson {
        value: serde_json::to_value(view)
            .map_err(|e| domain_error("historical_view_encoding", e.to_string()))?,
    })
}

fn canonical_case_inspection(invocation: &Invocation) -> Result<CliData, CliError> {
    let case = load_case(invocation)?;
    let authenticated = AuthenticatedPrincipal::authenticate_local()
        .map_err(|e| domain_error("authentication_failed", e))?;
    let store = open_store()?;
    // Full historical payload inspection is an operator/owner surface, not an
    // implicit disclosure channel for every model Participant in this Case.
    store
        .resolve_security_context(&authenticated, case.tenant_id.as_deref().unwrap())
        .and_then(|context| context.require_owner())
        .map_err(|e| domain_error("tenant_owner_required", e))?;
    if invocation.descriptor.operation_id == "yai.case.verify" {
        let replay = store
            .replay_case_state(&case.case_id)
            .map_err(|e| domain_error("canonical_replay_failed", e))?;
        if replay != case {
            return Err(domain_error(
                "canonical_materialization_mismatch",
                "CaseState differs from Transition Ledger replay",
            ));
        }
        return Ok(CliData::NativeJson {
            value: serde_json::json!({"case_id":case.case_id,
            "generation":case.generation,"authority":"transition_ledger","materialization":"equivalent_to_replay",
            "schema":case.schema,"mutation":"none"}),
        });
    }
    let limit = invocation
        .flag("--limit")
        .map(|value| value.parse::<usize>())
        .transpose()
        .map_err(|_| CliError::usage("--limit must be an integer from 1 to 256"))?
        .unwrap_or(32);
    if limit == 0 || limit > 256 {
        return Err(CliError::usage("--limit must be from 1 to 256"));
    }
    let history = store
        .list_case_transitions(&case.case_id)
        .map_err(|e| domain_error("canonical_history_failed", e))?;
    Ok(CliData::NativeJson {
        value: serde_json::json!({"case_id":case.case_id,
        "authority":"transition_ledger","total_transitions":history.len(),
        "transitions":&history[history.len().saturating_sub(limit)..]}),
    })
}

fn knowledge_inspection(invocation: &Invocation) -> Result<CliData, CliError> {
    use yai_core_engine::memory_hierarchy::knowledge::KnowledgeRequest;
    let case = invocation.positional("case")
        .ok_or_else(|| CliError::usage("Case is required"))?;
    let authenticated = AuthenticatedPrincipal::authenticate_local()
        .map_err(|e| domain_error("authentication_failed", e))?;
    let content = yai_core_engine::conversation::ConversationContentStore::open_existing(&yai_home()).ok();
    let mut request = KnowledgeRequest::new(case);
    request.source = invocation.flag("--source").map(str::to_string);
    request.revision = invocation.flag("--revision").map(str::to_string);
    if let Some(limit) = invocation.flag("--limit") {
        request.max_units = limit.parse()
            .map_err(|_| CliError::usage("--limit must be an integer"))?;
    }
    let result = open_store()?
        .case_knowledge_authorized(&authenticated, request, content.as_ref())
        .map_err(|e| domain_error("knowledge_unavailable", e))?;
    let view = &result.view;
    let mut value = serde_json::to_value(&result)
        .map_err(|e| domain_error("knowledge_encoding", e.to_string()))?;
    match invocation.descriptor.operation_id {
        "yai.case.knowledge.search" => {
            let query = invocation.positional("query")
                .ok_or_else(|| CliError::usage("query is required"))?;
            let hits = view.search(query, 32).map_err(|e| domain_error("knowledge_search", e))?;
            value["hits"] = serde_json::to_value(&hits).unwrap();
            if !invocation.json {
                for hit in &hits {
                    let unit = view.resolve(&hit.document_id)
                        .map_err(|e| domain_error("knowledge_reference", e))?;
                    println!("{} score={} {:?}\n  {}", unit.id, hit.score_micros,
                        unit.posture, serde_json::to_string(&unit.text).unwrap());
                }
            }
        }
        "yai.case.knowledge.resolve" => {
            let id = invocation.positional("reference")
                .ok_or_else(|| CliError::usage("reference is required"))?;
            let unit = view.resolve(id).map_err(|e| domain_error("knowledge_reference", e))?;
            value["resolved"] = serde_json::to_value(unit).unwrap();
            if !invocation.json {
                println!("{}", serde_json::to_string_pretty(unit).unwrap());
            }
        }
        "yai.case.knowledge.graph" if !invocation.json => {
            for edge in &view.relations {
                println!("{} -- {:?} --> {}\n  {} backing={:?}", edge.from,
                    edge.kind, edge.to, edge.posture, edge.backing_units);
            }
        }
        _ => {
            if !invocation.json {
                print!("{}", view.navigation());
            }
        }
    }
    if invocation.json {
        Ok(CliData::NativeJson { value })
    } else {
        Ok(CliData::AlreadyRendered)
    }
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
        && (operation_id.starts_with("yai.case.memory.")
            || operation_id.starts_with("yai.case.conversation.")
            || operation_id.starts_with("yai.case.cognitive.")
            || operation_id.starts_with("yai.provider.suitability.")
            || (operation_id.starts_with("yai.workflow.") && operation_id != "yai.workflow.input")
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
