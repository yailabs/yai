//! Product-owned controlled filesystem transition family.
//!
//! CLI candidate presentation, normalization entry points and consequence views.
//! Canonical effect advancement/reconciliation are shared Application operations;
//! admission, PREPARE and carriers retain their existing engine owners.

use super::*;
use crate::command_adapters::security::authenticate_local;
use yai_core_engine::case_policy::{NormativeReadiness, PolicyValidityPosture};
use yai_core_engine::effect::{
    normalize_write_prefix,
    validate_finalized_effect_chain, DecisionOutcome,
    LocalFilesystemBinding, LocalProcessBinding,
    Operation, OperationKind,
    ProcessSignalAction, OPERATION_PROPOSAL_SCHEMA,
    PROCESS_SIGNAL_PROPOSAL_SCHEMA,
};
use yai_core_engine::transition::{
    CaseState, EffectLifecycle, PendingTransition, ResourceAttachmentState, ResourceKind,
    ReviewRequirement, TransitionPayload,
    TransitionScope, TransitionSource,
};

pub(super) use yai_application::resource_execution::{
    load_effect_chain,
    ControlledEffectTurnStatus, ControlledEffectTurnResult,
};
use yai_application::resource_execution::{
    CONTROLLED_EFFECT_COMPONENT,
};

#[path = "controlled_effect/access.rs"]
pub(super) mod access;

#[path = "controlled_effect/source.rs"]
pub(super) mod source;


fn parse_max_bytes(args: &[String]) -> Result<usize, String> {
    optional_arg(args, "--max-bytes")
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|error| format!("invalid --max-bytes: {error}"))
        })
        .transpose()
        .map(|value| value.unwrap_or(yai_core_engine::effect::DEFAULT_MAX_WRITE_BYTES))
}

pub(super) fn case_attach_filesystem(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let attachment_id = named_arg(args, "--attachment")?;
    let root = PathBuf::from(named_arg(args, "--root")?);
    let allowed_write_prefix = normalize_write_prefix(&named_arg(args, "--allow-prefix")?)?;
    let policy_owner = named_arg(args, "--policy-owner")?;
    let policy_id = optional_arg(args, "--policy-id")
        .unwrap_or_else(|| format!("policy:filesystem-prefix:{attachment_id}"));
    let review_requirement = if args.iter().any(|arg| arg == "--require-review") {
        ReviewRequirement::RequireReview
    } else {
        ReviewRequirement::Automatic
    };
    let max_write_bytes = parse_max_bytes(args)?;
    if max_write_bytes == 0 {
        return Err("--max-bytes must be positive".to_string());
    }

    let store = LmdbRecordStore::open(record_store_path())?;
    let authenticated = authenticate_local()?;
    let state = store.get_case_state_authorized(&authenticated, &case_id)?;
    let tenant_id = state
        .tenant_id
        .clone()
        .ok_or_else(|| "legacy_unscoped_case_cannot_attach_filesystem".to_string())?;
    store
        .resolve_security_context(&authenticated, &tenant_id)?
        .require_owner()?;
    if !state
        .participants
        .iter()
        .any(|participant| participant.participant_id == policy_owner)
    {
        return Err(format!(
            "policy owner {policy_owner} is not bound to {case_id}"
        ));
    }
    let binding = LocalFilesystemBinding::new(&case_id, &attachment_id, &root)?;
    let prefix_path = Path::new(&binding.canonical_root).join(&allowed_write_prefix);
    let canonical_prefix = fs::canonicalize(&prefix_path)
        .map_err(|error| format!("allowed write prefix must already exist: {error}"))?;
    if !canonical_prefix.is_dir()
        || !canonical_prefix.starts_with(Path::new(&binding.canonical_root))
    {
        return Err("allowed write prefix is not a directory inside the binding root".to_string());
    }
    let attachment = ResourceAttachmentState {
        attachment_id: attachment_id.clone(),
        kind: ResourceKind::Filesystem,
        allowed_write_prefix: allowed_write_prefix.clone(),
        max_write_bytes,
        policy_id: policy_id.clone(),
        policy_owner_participant_id: policy_owner.clone(),
        review_requirement: review_requirement.clone(),
        process_signal_actions: Vec::new(),
        access: None,
    };
    attachment.validate()?;

    if let Some(existing) = state
        .resources
        .iter()
        .find(|resource| resource.attachment_id == attachment_id)
    {
        if existing != &attachment {
            return Err("resource attachment already exists with a different contract".to_string());
        }
        store.put_tenant_local_filesystem_binding(&authenticated, &binding)?;
        println!("filesystem_attachment: already_attached");
    } else {
        let mut pending = PendingTransition::new(
            format!(
                "transition:resource-attached:{}:{attachment_id}",
                yai_core_engine::context::stable_digest(&case_id)
            ),
            &case_id,
            state.generation,
            TransitionSource {
                component: CONTROLLED_EFFECT_COMPONENT.to_string(),
                participant_id: None,
                principal_id: Some(authenticated.projected_principal_id()),
                source_ref: Some(format!(
                    "resource-attached:{}:{attachment_id}",
                    yai_core_engine::context::stable_digest(&case_id)
                )),
            },
            TransitionPayload::ResourceAttached {
                attachment: attachment.clone(),
            },
        );
        pending.scope = Some(TransitionScope {
            case_id: case_id.clone(),
            participant_refs: vec![policy_owner.clone()],
            resource_refs: vec![attachment_id.clone()],
            policy_refs: vec![policy_id.clone()],
        });
        pending.causal_refs = vec![policy_owner.clone()];
        store.commit_tenant_resource_attachment(&authenticated, &tenant_id, pending, &binding)?;
        println!("filesystem_attachment: attached");
    }
    println!("case_id: {case_id}");
    println!("attachment_id: {attachment_id}");
    println!("logical_kind: filesystem");
    println!("allowed_write_prefix: {allowed_write_prefix}");
    println!("policy_id: {policy_id}");
    println!("policy_owner: {policy_owner}");
    println!(
        "review_requirement: {}",
        match review_requirement {
            ReviewRequirement::Automatic => "automatic",
            ReviewRequirement::RequireReview => "require_review",
        }
    );
    println!("local_binding: configured_noncanonical");
    println!("local_root: {}", binding.canonical_root);
    Ok(())
}

pub(super) fn case_attach_process(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let attachment_id = named_arg(args, "--attachment")?;
    let pid = named_arg(args, "--pid")?
        .parse::<u32>()
        .map_err(|error| format!("invalid --pid: {error}"))?;
    let policy_owner = named_arg(args, "--policy-owner")?;
    let policy_id = optional_arg(args, "--policy-id")
        .unwrap_or_else(|| format!("policy:process-signal:{attachment_id}"));
    let review_requirement = if args.iter().any(|arg| arg == "--require-review") {
        ReviewRequirement::RequireReview
    } else {
        ReviewRequirement::Automatic
    };
    let mut actions = optional_arg(args, "--actions")
        .unwrap_or_else(|| "terminate".to_string())
        .split(',')
        .map(|value| match value.trim() {
            "terminate" => Ok(ProcessSignalAction::Terminate),
            "suspend" => Ok(ProcessSignalAction::Suspend),
            "resume" => Ok(ProcessSignalAction::Resume),
            other => Err(format!("unsupported process action: {other}")),
        })
        .collect::<Result<Vec<_>, _>>()?;
    actions.sort_by_key(ProcessSignalAction::as_str);
    actions.dedup();
    if actions.is_empty() {
        return Err("at least one process action is required".to_string());
    }

    let store = LmdbRecordStore::open(record_store_path())?;
    let authenticated = authenticate_local()?;
    let state = store.get_case_state_authorized(&authenticated, &case_id)?;
    let tenant_id = state
        .tenant_id
        .clone()
        .ok_or_else(|| "legacy_unscoped_case_cannot_attach_process".to_string())?;
    store
        .resolve_security_context(&authenticated, &tenant_id)?
        .require_owner()?;
    if !state
        .participants
        .iter()
        .any(|participant| participant.participant_id == policy_owner)
    {
        return Err(format!(
            "policy owner {policy_owner} is not bound to {case_id}"
        ));
    }
    let binding = LocalProcessBinding::capture(&case_id, &attachment_id, pid)?;
    if binding.process.pid == std::process::id() {
        return Err("cannot_attach_current_yai_process".to_string());
    }
    let attachment = ResourceAttachmentState {
        attachment_id: attachment_id.clone(),
        kind: ResourceKind::Process,
        allowed_write_prefix: String::new(),
        max_write_bytes: 0,
        policy_id: policy_id.clone(),
        policy_owner_participant_id: policy_owner.clone(),
        review_requirement: review_requirement.clone(),
        process_signal_actions: actions.clone(),
        access: None,
    };
    attachment.validate()?;
    if let Some(existing) = state
        .resources
        .iter()
        .find(|resource| resource.attachment_id == attachment_id)
    {
        if existing != &attachment
            || store
                .get_local_process_binding(&case_id, &attachment_id)?
                .as_ref()
                != Some(&binding)
        {
            return Err("process_attachment_is_immutable".to_string());
        }
        println!("process_attachment: already_attached");
    } else {
        let mut pending = PendingTransition::new(
            format!(
                "transition:process-resource-attached:{}:{attachment_id}",
                yai_core_engine::context::stable_digest(&case_id)
            ),
            &case_id,
            state.generation,
            TransitionSource {
                component: CONTROLLED_EFFECT_COMPONENT.to_string(),
                participant_id: None,
                principal_id: Some(authenticated.projected_principal_id()),
                source_ref: Some(format!("process-resource-attached:{attachment_id}")),
            },
            TransitionPayload::ResourceAttached {
                attachment: attachment.clone(),
            },
        );
        pending.scope = Some(TransitionScope {
            case_id: case_id.clone(),
            participant_refs: vec![policy_owner.clone()],
            resource_refs: vec![attachment_id.clone()],
            policy_refs: vec![policy_id.clone()],
        });
        pending.causal_refs = vec![policy_owner.clone()];
        store.commit_tenant_process_attachment(&authenticated, &tenant_id, pending, &binding)?;
        println!("process_attachment: attached");
    }
    println!("case_id: {case_id}");
    println!("attachment_id: {attachment_id}");
    println!("logical_kind: process");
    println!("process_pid: {}", binding.process.pid);
    println!("process_boot_id: {}", binding.process.boot_id);
    println!("process_start_ticks: {}", binding.process.start_ticks);
    println!(
        "allowed_actions: {}",
        actions
            .iter()
            .map(ProcessSignalAction::as_str)
            .collect::<Vec<_>>()
            .join(",")
    );
    println!("policy_id: {policy_id}");
    println!("policy_owner: {policy_owner}");
    println!("local_binding: exact_process_birth_identity");
    Ok(())
}

fn resource_for_case(
    state: &CaseState,
    attachment_id: &str,
) -> Result<ResourceAttachmentState, String> {
    state
        .resources
        .iter()
        .find(|resource| resource.attachment_id == attachment_id)
        .cloned()
        .ok_or_else(|| {
            format!(
                "resource attachment {attachment_id} is not bound to {}",
                state.case_id
            )
        })
}

fn provider_args_for_case(args: &[String], case_id: &str, participant_id: &str) -> Vec<String> {
    let mut result = args.to_vec();
    if optional_arg(&result, "--case").is_none() {
        result.push("--case".to_string());
        result.push(case_id.to_string());
    }
    if optional_arg(&result, "--subject").is_none() {
        result.push("--subject".to_string());
        result.push(participant_id.to_string());
    }
    result
}

fn replace_named_arg(args: &mut Vec<String>, name: &str, value: &str) {
    if let Some(index) = args.iter().position(|item| item == name) {
        if index + 1 < args.len() {
            args[index + 1] = value.to_string();
            return;
        }
    }
    args.push(name.to_string());
    args.push(value.to_string());
}

fn remove_named_arg(args: &mut Vec<String>, name: &str, takes_value: bool) {
    if let Some(index) = args.iter().position(|item| item == name) {
        args.remove(index);
        if takes_value && index < args.len() {
            args.remove(index);
        }
    }
}

fn second_turn_provider_args(
    args: &[String],
    case_id: &str,
    participant_id: &str,
) -> Result<Vec<String>, String> {
    let mut result = provider_args_for_case(args, case_id, participant_id);
    let second_base_url = optional_arg(args, "--second-base-url");
    let second_provider_id = optional_arg(args, "--second-provider-id");
    let second_model = optional_arg(args, "--second-model");
    if second_base_url.is_none() && second_provider_id.is_none() && second_model.is_none() {
        return Ok(result);
    }
    let base_url = second_base_url
        .or_else(|| optional_arg(args, "--base-url"))
        .ok_or_else(|| "second provider requires --second-base-url or --base-url".to_string())?;
    let provider_id = second_provider_id
        .or_else(|| optional_arg(args, "--provider-id"))
        .unwrap_or_else(|| "provider:openai-compatible".to_string());
    let model = second_model
        .or_else(|| optional_arg(args, "--model"))
        .ok_or_else(|| "second provider requires --second-model or --model".to_string())?;
    let mut attach = vec![
        "--case".to_string(),
        case_id.to_string(),
        "--subject".to_string(),
        participant_id.to_string(),
        "--base-url".to_string(),
        base_url.clone(),
        "--provider-id".to_string(),
        provider_id.clone(),
        "--model".to_string(),
        model.clone(),
    ];
    if let Some(api_key_env) = optional_arg(args, "--api-key-env") {
        attach.push("--api-key-env".to_string());
        attach.push(api_key_env);
    }
    case_attach_provider(&attach)?;
    replace_named_arg(&mut result, "--base-url", &base_url);
    replace_named_arg(&mut result, "--provider-id", &provider_id);
    replace_named_arg(&mut result, "--model", &model);
    remove_named_arg(&mut result, "--continuation-ref", true);
    remove_named_arg(&mut result, "--provider-runtime-id", true);
    remove_named_arg(&mut result, "--continuation-capable", false);
    Ok(result)
}



fn failpoint(args: &[String]) -> Option<String> {
    optional_arg(args, "--failpoint")
}

fn exit_at_failpoint(name: &str, code: i32) -> ! {
    eprintln!("controlled_effect_crash_injected: {name}");
    std::process::exit(code)
}

fn update_derived_after_commit(store: &LmdbRecordStore, case_id: &str, args: &[String]) {
    update_derived_after_commit_with_reporting(store, case_id, args, true)
}

fn update_derived_after_commit_with_reporting(
    store: &LmdbRecordStore,
    case_id: &str,
    args: &[String],
    verbose: bool,
) {
    if args.iter().any(|arg| arg == "--inject-derived-failure") {
        eprintln!("derived_update: injected_failure_canonical_state_preserved");
        return;
    }
    match store.materialize_graph_relations_for_case(case_id) {
        Ok(report) => {
            if verbose {
                println!("derived_graph_edges: {}", report.relations_written);
            }
        }
        Err(error) => eprintln!("derived_update_failed_canonical_state_preserved: {error}"),
    }
    match store
        .list_case_transitions(case_id)
        .and_then(|transitions| {
            derive_operational_memory(case_id, &transitions)
                .and_then(|build| store.replace_case_operational_memory(&build).map(|_| build))
        }) {
        Ok(build) => {
            if verbose {
                println!("derived_memory_entries: {}", build.entries.len());
            }
        }
        Err(error) => eprintln!("derived_memory_failed_canonical_state_preserved: {error}"),
    }
}

/// Advance provider-originated candidate material through the one typed
/// filesystem effect chain. The caller owns iteration; this function owns no
/// process-local continuity and performs no provider invocation.
pub(super) fn advance_controlled_filesystem_candidate(
    args: &[String],
    case_id: &str,
    participant_id: &str,
    attachment_id: &str,
    provider_result: &ControlledProviderResult,
) -> Result<ControlledEffectTurnResult, String> {
    let store = LmdbRecordStore::open(record_store_path())?;
    match store.propose_controlled_candidate_authorized(&authenticate_local()?, case_id,
        participant_id, attachment_id, &provider_result.result_id, None)? {
        Ok(operation) => advance_canonical_controlled_operation(args, &store, &operation),
        Err(failure) => {
            update_derived_after_commit(&store, case_id, args);
            println!("operation_normalization: rejected");
            println!("normalization_code: {:?}", failure.code);
            println!("external_effect: none");
            Ok(ControlledEffectTurnResult {
                status: ControlledEffectTurnStatus::NormalizationRejected,
                operation_id: None, decision_id: None, review_id: None,
                effect_id: None, receipt_id: None, outcome: None,
            })
        }
    }
}

/// Shared carrier/authority advancement for an already canonical Operation.
/// The normalizer (legacy typed proposal, Workflow or native capability result)
/// does not choose a different execution path after canonical admission.
pub(super) fn advance_canonical_controlled_operation(
    args: &[String],
    store: &LmdbRecordStore,
    operation: &Operation,
) -> Result<ControlledEffectTurnResult, String> {
    advance_canonical_controlled_operation_with_reporting(args, store, operation, true)
}

/// Application caller consumes typed outcomes, not engineering stdout.
pub(super) fn advance_case_filesystem_operation(
    store: &LmdbRecordStore,
    operation: &Operation,
) -> Result<ControlledEffectTurnResult, String> {
    if operation.kind != OperationKind::FilesystemWrite {
        return Err("case_filesystem_operation_kind_required".into());
    }
    advance_canonical_controlled_operation_with_reporting(&[], store, operation, false)
}

fn advance_canonical_controlled_operation_with_reporting(
    args: &[String],
    store: &LmdbRecordStore,
    operation: &Operation,
    verbose: bool,
) -> Result<ControlledEffectTurnResult, String> {
    struct CliEffectHooks<'a> { args: &'a [String], verbose: bool, failpoint: Option<String> }
    impl yai_application::resource_execution::ControlledEffectHooks for CliEffectHooks<'_> {
        fn report(&mut self, message: String) {
            if self.verbose { println!("{message}"); }
        }
        fn failpoint(&self) -> Option<&str> { self.failpoint.as_deref() }
        fn interrupt(&mut self, name: &str, code: i32) -> Result<(), String> {
            exit_at_failpoint(name, code)
        }
        fn after_commit(&mut self, store: &LmdbRecordStore, case: &str) {
            update_derived_after_commit_with_reporting(store, case, self.args, self.verbose);
        }
    }
    let mut hooks = CliEffectHooks { args, verbose, failpoint: failpoint(args) };
    yai_application::resource_execution::advance_controlled_operation(
        &authenticate_local()?, &mut hooks, store, operation)
}

pub(super) fn advance_controlled_workflow_deterministic(
    args: &[String],
    item: &yai_core_engine::store::lmdb::RuntimeWorkItem,
) -> Result<ControlledEffectTurnResult, String> {
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let proposal_existed = store
        .get_case_state_authorized(&authenticated, &item.case_id)?
        .workflow_deterministic_proposals
        .iter()
        .any(|proposal| {
            item.workflow
                .as_ref()
                .is_some_and(|workflow| proposal.execution_id == workflow.workflow_execution_id)
        });
    let proposal = store.record_workflow_deterministic_proposal(&authenticated, item)?;
    if !proposal_existed
        && failpoint(args).as_deref() == Some("after_workflow_deterministic_proposal")
    {
        eprintln!(
            "workflow_deterministic_proposal_id: {}",
            proposal.proposal_id
        );
        exit_at_failpoint("after_workflow_deterministic_proposal", 90);
    }
    let operation_existed = store
        .list_case_transitions(&item.case_id)?
        .iter()
        .any(|transition| {
            matches!(
                &transition.payload,
                TransitionPayload::OperationRecorded { operation }
                    if matches!(
                        &operation.origin,
                        yai_core_engine::effect::OperationOrigin::WorkflowDeterministicProposal {
                            proposal_id,
                            ..
                        } if proposal_id == &proposal.proposal_id
                    )
            )
        });
    let operation = store.record_workflow_deterministic_operation_from_proposal(
        &authenticated,
        item,
        &proposal,
    )?;
    if !operation_existed
        && failpoint(args).as_deref() == Some("after_workflow_deterministic_operation")
    {
        eprintln!(
            "workflow_deterministic_operation_id: {}",
            operation.operation_id
        );
        exit_at_failpoint("after_workflow_deterministic_operation", 92);
    }
    let source = ControlledProviderResult {
        invocation_id: item
            .workflow
            .as_ref()
            .map(|workflow| workflow.workflow_execution_id.clone())
            .ok_or_else(|| "runtime_workflow_context_missing".to_string())?,
        result_id: proposal.proposal_id,
        raw_output: String::new(),
        provider_id: "provider:none".to_string(),
        model_id: "model:none".to_string(),
        projection_id: String::new(),
        context_frame_id: String::new(),
        residency_plan_id: String::new(),
        resident_item_ids: Vec::new(),
        projection_selected_items: 0,
        projection_omitted_items: 0,
        semantic_units: 0,
        estimated_input_units: 0,
        usage: Default::default(),
        request_bytes_written: 0,
    };
    advance_controlled_filesystem_candidate(
        args,
        &item.case_id,
        &item.participant_id,
        &item.attachment_id,
        &source,
    )
}

pub(super) fn controlled_filesystem_write(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--subject")?;
    let attachment_id = named_arg(args, "--attachment")?;
    let prompt = named_arg(args, "--prompt")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let authenticated = authenticate_local()?;
    let state = store.get_case_state_authorized(&authenticated, &case_id)?;
    if state.lifecycle == CaseLifecycle::Closed || state.cancellation.is_some() {
        println!("case_lifecycle: {:?}", state.lifecycle);
        println!("case_cancelled: {}", state.cancellation.is_some());
        println!("provider_invocations: 0");
        println!("execution_grants: 0");
        println!("external_effect: none");
        return Err(if state.lifecycle == CaseLifecycle::Closed {
            "case_closed_new_effect_forbidden".to_string()
        } else {
            "case_cancelled_new_effect_forbidden".to_string()
        });
    }
    let resource = resource_for_case(&state, &attachment_id)?;
    let normative = store.case_policy_status(&case_id)?;
    if normative.readiness != NormativeReadiness::Ready
        || normative.validity != PolicyValidityPosture::Valid
    {
        println!("normative_readiness: {:?}", normative.readiness);
        println!("policy_validity: {:?}", normative.validity);
        println!("provider_invocations: 0");
        println!("execution_grants: 0");
        println!("external_effect: none");
        return Err(format!(
            "normative_case_not_authoritative: readiness={:?} validity={:?}",
            normative.readiness, normative.validity
        ));
    }
    drop(store);
    let provider_args = provider_args_for_case(args, &case_id, &participant_id);
    let provider_result = invoke_controlled_provider(
        &provider_args,
        ProjectionPurpose::FilesystemWriteProposal,
        &prompt,
        InvocationOutputContract::FilesystemWriteProposal {
            schema: OPERATION_PROPOSAL_SCHEMA.to_string(),
            attachment_id: resource.attachment_id.clone(),
            allowed_write_prefix: resource.allowed_write_prefix.clone(),
            max_write_bytes: resource.max_write_bytes,
        },
    )?;
    println!("provider_invocation_id: {}", provider_result.invocation_id);
    println!("provider_result_id: {}", provider_result.result_id);
    println!("provider_id: {}", provider_result.provider_id);
    println!("provider_model: {}", provider_result.model_id);
    println!("provider_projection_id: {}", provider_result.projection_id);
    println!(
        "provider_context_frame_id: {}",
        provider_result.context_frame_id
    );
    println!("provider_result_authority: non_authoritative_candidate_material");

    let outcome = advance_controlled_filesystem_candidate(
        args,
        &case_id,
        &participant_id,
        &attachment_id,
        &provider_result,
    )?;
    if matches!(
        outcome.status,
        ControlledEffectTurnStatus::NormalizationRejected
            | ControlledEffectTurnStatus::AwaitingReview
            | ControlledEffectTurnStatus::Indeterminate
    ) {
        return Ok(());
    }
    let second_provider_args = second_turn_provider_args(args, &case_id, &participant_id)?;
    let second = invoke_controlled_provider(
        &second_provider_args,
        ProjectionPurpose::EffectConsequence,
        match outcome.status {
            ControlledEffectTurnStatus::Denied => {
                "Report the committed controlled filesystem denial from this view."
            }
            ControlledEffectTurnStatus::Finalized => {
                "Report the observed controlled filesystem consequence from this view."
            }
            ControlledEffectTurnStatus::NormalizationRejected
            | ControlledEffectTurnStatus::AwaitingReview
            | ControlledEffectTurnStatus::Indeterminate => unreachable!(),
        },
        InvocationOutputContract::NaturalLanguage,
    )?;
    println!("second_provider_invocation_id: {}", second.invocation_id);
    println!("second_provider_result_id: {}", second.result_id);
    println!(
        "second_turn_consequence: {}",
        match outcome.status {
            ControlledEffectTurnStatus::Denied => "committed_denial_no_effect",
            ControlledEffectTurnStatus::Finalized => "observed_reality_from_canonical_state",
            ControlledEffectTurnStatus::NormalizationRejected
            | ControlledEffectTurnStatus::AwaitingReview
            | ControlledEffectTurnStatus::Indeterminate => unreachable!(),
        }
    );
    Ok(())
}

pub(super) fn controlled_process_signal(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--subject")?;
    let attachment_id = named_arg(args, "--attachment")?;
    let prompt = named_arg(args, "--prompt")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let authenticated = authenticate_local()?;
    let state = store.get_case_state_authorized(&authenticated, &case_id)?;
    if state.lifecycle == CaseLifecycle::Closed || state.cancellation.is_some() {
        println!("provider_invocations: 0");
        println!("execution_grants: 0");
        println!("physical_signal: none");
        return Err(if state.lifecycle == CaseLifecycle::Closed {
            "case_closed_new_effect_forbidden".to_string()
        } else {
            "case_cancelled_new_effect_forbidden".to_string()
        });
    }
    let resource = resource_for_case(&state, &attachment_id)?;
    if resource.kind != ResourceKind::Process {
        return Err("process_signal_requires_process_attachment".to_string());
    }
    let normative = store.case_policy_status(&case_id)?;
    if normative.readiness != NormativeReadiness::Ready
        || normative.validity != PolicyValidityPosture::Valid
    {
        println!("provider_invocations: 0");
        println!("execution_grants: 0");
        println!("physical_signal: none");
        return Err(format!(
            "normative_case_not_authoritative: readiness={:?} validity={:?}",
            normative.readiness, normative.validity
        ));
    }
    drop(store);
    let provider_args = provider_args_for_case(args, &case_id, &participant_id);
    let provider_result = invoke_controlled_provider(
        &provider_args,
        ProjectionPurpose::ProcessSignalProposal,
        &prompt,
        InvocationOutputContract::ProcessSignalProposal {
            schema: PROCESS_SIGNAL_PROPOSAL_SCHEMA.to_string(),
            attachment_id: resource.attachment_id.clone(),
            allowed_actions: resource
                .process_signal_actions
                .iter()
                .map(|action| action.as_str().to_string())
                .collect(),
        },
    )?;
    println!("provider_invocation_id: {}", provider_result.invocation_id);
    println!("provider_result_id: {}", provider_result.result_id);
    println!("provider_id: {}", provider_result.provider_id);
    println!("provider_model: {}", provider_result.model_id);
    println!("provider_projection_id: {}", provider_result.projection_id);
    println!(
        "provider_context_frame_id: {}",
        provider_result.context_frame_id
    );
    println!("provider_result_authority: non_authoritative_candidate_material");
    let outcome = advance_controlled_filesystem_candidate(
        args,
        &case_id,
        &participant_id,
        &attachment_id,
        &provider_result,
    )?;
    if matches!(
        outcome.status,
        ControlledEffectTurnStatus::NormalizationRejected
            | ControlledEffectTurnStatus::AwaitingReview
            | ControlledEffectTurnStatus::Indeterminate
    ) {
        return Ok(());
    }
    let second_provider_args = second_turn_provider_args(args, &case_id, &participant_id)?;
    let second = invoke_controlled_provider(
        &second_provider_args,
        ProjectionPurpose::EffectConsequence,
        match outcome.status {
            ControlledEffectTurnStatus::Denied => {
                "Report the committed process-signal denial from this Case view."
            }
            ControlledEffectTurnStatus::Finalized => {
                "Report exactly what the kernel carrier and observation recorded."
            }
            _ => unreachable!(),
        },
        InvocationOutputContract::NaturalLanguage,
    )?;
    println!("second_provider_invocation_id: {}", second.invocation_id);
    println!("second_provider_result_id: {}", second.result_id);
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum CaseReconciliationStatus {
    Clean,
    Reconciled { count: usize },
    Unresolved { effect_ids: Vec<String> },
}

/// Reconcile every currently unresolved filesystem effect before a new model
/// invocation. Process signals are deliberately not retried: after a lost
/// acknowledgement the kernel side effect is not generally replay-safe, so
/// the unresolved process effect remains parked with its resource fence. This
/// is intentionally a synchronous Case boundary, not a background scheduler.
pub(super) fn reconcile_case_before_invocation(
    case_id: &str,
    retry_no_effect: bool,
) -> Result<CaseReconciliationStatus, String> {
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store
        .get_case_state(case_id)?
        .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
    let unresolved = state
        .effects
        .iter()
        .filter(|effect| {
            matches!(
                effect.status,
                EffectLifecycle::Prepared | EffectLifecycle::Indeterminate
            )
        })
        .map(|effect| (effect.effect_id.clone(), effect.kind.clone()))
        .collect::<Vec<_>>();
    drop(store);
    if unresolved.is_empty() {
        return Ok(CaseReconciliationStatus::Clean);
    }
    for (effect_id, kind) in &unresolved {
        if *kind == OperationKind::ProcessSignal {
            continue;
        }
        let mut args = vec![
            "--case".to_string(),
            case_id.to_string(),
            "--effect".to_string(),
            effect_id.clone(),
        ];
        if retry_no_effect {
            args.push("--retry".to_string());
        }
        controlled_effect_reconcile(&args)?;
    }
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store
        .get_case_state(case_id)?
        .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
    let still_unresolved = state
        .effects
        .iter()
        .filter(|effect| {
            matches!(
                effect.status,
                EffectLifecycle::Prepared | EffectLifecycle::Indeterminate
            )
        })
        .map(|effect| effect.effect_id.clone())
        .collect::<Vec<_>>();
    if still_unresolved.is_empty() {
        Ok(CaseReconciliationStatus::Reconciled {
            count: unresolved.len(),
        })
    } else {
        Ok(CaseReconciliationStatus::Unresolved {
            effect_ids: still_unresolved,
        })
    }
}

pub(super) fn controlled_effect_reconcile(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let requested_effect = optional_arg(args, "--effect");
    let retry = args.iter().any(|arg| arg == "--retry");
    let store = LmdbRecordStore::open(record_store_path())?;
    struct Report;
    impl yai_application::resource_execution::ControlledEffectHooks for Report {
        fn report(&mut self, message: String) { println!("{message}"); }
    }
    yai_application::resource_execution::reconcile_controlled_effect(
        &store, &authenticate_local()?, &case_id, requested_effect.as_deref(), retry, &mut Report)
}

pub(super) fn controlled_effect_inspect(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let effect_id = named_arg(args, "--effect")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store
        .get_case_state(&case_id)?
        .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
    if state.tenant_id.is_some() {
        let authenticated = authenticate_local()?;
        store.get_case_state_authorized(&authenticated, &case_id)?;
    }
    let transitions = store.list_case_transitions(&case_id)?;
    let chain = load_effect_chain(&transitions, &effect_id)?;
    let effect = state
        .effects
        .iter()
        .find(|effect| effect.effect_id == effect_id)
        .ok_or_else(|| "materialized effect state missing".to_string())?;
    println!("effect_chain:");
    println!("case_id: {case_id}");
    println!("operation_id: {}", chain.operation.operation_id);
    println!("decision_id: {}", chain.decision.decision_id);
    println!("decision: {:?}", chain.decision.outcome);
    println!("grant_id: {}", chain.grant.grant_id);
    println!("prepare: {}", chain.prepared.effect_id);
    println!(
        "pre_observation: {} {:?}",
        chain.prepared.expected_pre_observation.observation_id,
        chain.prepared.expected_pre_observation.state
    );
    println!("carrier: {}", chain.prepared.carrier_backend);
    println!("state: {:?}", effect.status);
    println!("outcome: {:?}", effect.outcome);
    println!(
        "post_observation: {}",
        effect.post_observation_id.as_deref().unwrap_or("none")
    );
    println!(
        "receipt: {}",
        effect.receipt_id.as_deref().unwrap_or("none")
    );
    if effect.status == EffectLifecycle::Finalized {
        validate_finalized_effect_chain(&transitions, &effect_id)?;
        println!("closure: valid");
    } else {
        println!("closure: unresolved");
    }
    Ok(())
}
