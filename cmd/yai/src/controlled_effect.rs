//! Product-owned controlled filesystem transition family.
//!
//! This module owns the one constitutional filesystem.write orchestration:
//! provider candidate normalization, deterministic admission, prepared-effect
//! recovery, the grant-validated Rust carrier, and typed consequence views.
//! It is deliberately not a generic carrier registry or policy engine.

use super::*;
use yai_core_engine::effect::{
    build_effect_receipt, classify_reconciliation, decide_filesystem_write,
    execute_filesystem_write, issue_execution_grant, normalize_filesystem_write_candidate,
    normalize_review_filesystem_write, normalize_write_prefix, observe_filesystem, prepare_effect,
    record_filesystem_decision, validate_finalized_effect_chain, CarrierFailpoint, CarrierResult,
    Decision, DecisionOutcome, EffectOutcome, ExecutionGrant, FilesystemObservation,
    LocalFilesystemBinding, NormalizationContext, Operation, PreparedEffect,
    ReconciliationConclusion, ResourceState, OPERATION_PROPOSAL_SCHEMA,
};
use yai_core_engine::transition::{
    CaseState, EffectLifecycle, PendingTransition, ResourceAttachmentState, ResourceKind,
    ReviewResolution, ReviewState, Transition, TransitionPayload, TransitionScope,
    TransitionSource,
};

const CONTROLLED_EFFECT_COMPONENT: &str = "yai.controlled_filesystem_effect";

fn id_component(value: &str) -> String {
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

fn effect_source(participant_id: Option<&str>, source_ref: &str) -> TransitionSource {
    TransitionSource {
        component: CONTROLLED_EFFECT_COMPONENT.to_string(),
        participant_id: participant_id.map(ToString::to_string),
        source_ref: Some(source_ref.to_string()),
    }
}

fn commit_effect_transition(
    store: &LmdbRecordStore,
    case_id: &str,
    participant_id: Option<&str>,
    label: &str,
    payload: TransitionPayload,
    scope: Option<TransitionScope>,
    causal_refs: Vec<String>,
) -> Result<CaseState, String> {
    let generation = store
        .get_case_state(case_id)?
        .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?
        .generation;
    let transition_id = format!(
        "transition:controlled-effect:{}:{:020}:{}",
        id_component(case_id),
        generation + 1,
        id_component(label)
    );
    let mut pending = PendingTransition::new(
        transition_id,
        case_id,
        generation,
        effect_source(participant_id, label),
        payload,
    );
    pending.scope = scope;
    pending.causal_refs = causal_refs;
    store.commit_transition(pending).map(|commit| commit.state)
}

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
    let max_write_bytes = parse_max_bytes(args)?;
    if max_write_bytes == 0 {
        return Err("--max-bytes must be positive".to_string());
    }

    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store
        .get_case_state(&case_id)?
        .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
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
        store.put_local_filesystem_binding(&binding)?;
        println!("filesystem_attachment: already_attached");
    } else {
        commit_effect_transition(
            &store,
            &case_id,
            Some(&policy_owner),
            &format!("resource-attached:{attachment_id}"),
            TransitionPayload::ResourceAttached {
                attachment: attachment.clone(),
            },
            Some(TransitionScope {
                case_id: case_id.clone(),
                participant_refs: vec![policy_owner.clone()],
                resource_refs: vec![attachment_id.clone()],
                policy_refs: vec![policy_id.clone()],
            }),
            vec![policy_owner.clone()],
        )?;
        store.put_local_filesystem_binding(&binding)?;
        println!("filesystem_attachment: attached");
    }
    println!("case_id: {case_id}");
    println!("attachment_id: {attachment_id}");
    println!("logical_kind: filesystem");
    println!("allowed_write_prefix: {allowed_write_prefix}");
    println!("policy_id: {policy_id}");
    println!("policy_owner: {policy_owner}");
    println!("local_binding: configured_noncanonical");
    println!("local_root: {}", binding.canonical_root);
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

fn commit_normalization_failure(
    store: &LmdbRecordStore,
    case_id: &str,
    participant_id: &str,
    provider_result_id: &str,
    failure: yai_core_engine::effect::NormalizationFailure,
) -> Result<CaseState, String> {
    commit_effect_transition(
        store,
        case_id,
        Some(participant_id),
        "operation-normalization-failed",
        TransitionPayload::OperationNormalizationFailed {
            provider_result_id: provider_result_id.to_string(),
            failure,
        },
        None,
        vec![provider_result_id.to_string()],
    )
}

fn commit_operation(store: &LmdbRecordStore, operation: &Operation) -> Result<CaseState, String> {
    commit_effect_transition(
        store,
        &operation.case_id,
        Some(&operation.participant_id),
        &format!("operation:{}", operation.operation_id),
        TransitionPayload::OperationRecorded {
            operation: operation.clone(),
        },
        Some(operation.scope.clone()),
        operation.origin.causal_refs(),
    )
}

fn commit_decision(
    store: &LmdbRecordStore,
    case_id: &str,
    decision: &Decision,
) -> Result<CaseState, String> {
    commit_effect_transition(
        store,
        case_id,
        Some(&decision.source.owner_participant_id),
        &format!("decision:{}", decision.decision_id),
        TransitionPayload::DecisionRecorded {
            decision: decision.clone(),
        },
        None,
        vec![decision.operation_id.clone()],
    )
}

fn commit_grant(store: &LmdbRecordStore, grant: &ExecutionGrant) -> Result<CaseState, String> {
    commit_effect_transition(
        store,
        &grant.case_id,
        Some(&grant.participant_id),
        &format!("grant:{}", grant.grant_id),
        TransitionPayload::ExecutionGrantIssued {
            grant: grant.clone(),
        },
        None,
        vec![grant.operation_id.clone(), grant.decision_id.clone()],
    )
}

fn commit_prepare(store: &LmdbRecordStore, prepared: &PreparedEffect) -> Result<CaseState, String> {
    commit_effect_transition(
        store,
        &prepared.case_id,
        Some(&prepared.participant_id),
        &format!("prepare:{}", prepared.effect_id),
        TransitionPayload::EffectPrepared {
            prepared: prepared.clone(),
        },
        None,
        vec![
            prepared.operation_id.clone(),
            prepared.decision_id.clone(),
            prepared.grant_id.clone(),
            prepared.expected_pre_observation.observation_id.clone(),
        ],
    )
}

fn commit_indeterminate(
    store: &LmdbRecordStore,
    prepared: &PreparedEffect,
    reason: String,
    observation: Option<FilesystemObservation>,
) -> Result<CaseState, String> {
    commit_effect_transition(
        store,
        &prepared.case_id,
        Some(&prepared.participant_id),
        &format!("indeterminate:{}", prepared.effect_id),
        TransitionPayload::EffectIndeterminate {
            effect_id: prepared.effect_id.clone(),
            reason,
            observation,
        },
        None,
        vec![prepared.effect_id.clone()],
    )
}

fn commit_finalize(
    store: &LmdbRecordStore,
    prepared: &PreparedEffect,
    result: &CarrierResult,
) -> Result<CaseState, String> {
    let receipt = build_effect_receipt(prepared, result);
    commit_effect_transition(
        store,
        &prepared.case_id,
        Some(&prepared.participant_id),
        &format!("finalize:{}", prepared.effect_id),
        TransitionPayload::EffectFinalized {
            effect_id: prepared.effect_id.clone(),
            post_observation: result.post_observation.clone(),
            receipt: receipt.clone(),
        },
        None,
        vec![prepared.effect_id.clone(), receipt.receipt_id],
    )
}

fn failpoint(args: &[String]) -> Option<String> {
    optional_arg(args, "--failpoint")
}

fn exit_at_failpoint(name: &str, code: i32) -> ! {
    eprintln!("controlled_effect_crash_injected: {name}");
    std::process::exit(code)
}

fn update_derived_after_commit(store: &LmdbRecordStore, case_id: &str, args: &[String]) {
    if args.iter().any(|arg| arg == "--inject-derived-failure") {
        eprintln!("derived_update: injected_failure_canonical_state_preserved");
        return;
    }
    match store.materialize_graph_relations_for_case(case_id) {
        Ok(report) => println!("derived_graph_edges: {}", report.relations_written),
        Err(error) => eprintln!("derived_update_failed_canonical_state_preserved: {error}"),
    }
}

pub(super) fn controlled_filesystem_write(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--subject")?;
    let attachment_id = named_arg(args, "--attachment")?;
    let prompt = named_arg(args, "--prompt")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store
        .get_case_state(&case_id)?
        .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
    let resource = resource_for_case(&state, &attachment_id)?;
    if resource.policy_owner_participant_id == participant_id {
        return Err("operation participant cannot own its admission policy".to_string());
    }
    let binding = store
        .get_local_filesystem_binding(&case_id, &attachment_id)?
        .ok_or_else(|| format!("local binding missing for {case_id}/{attachment_id}"))?;
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

    let store = LmdbRecordStore::open(record_store_path())?;
    let state_after_provider = store
        .get_case_state(&case_id)?
        .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
    let context = NormalizationContext {
        case_id: &case_id,
        participant_id: &participant_id,
        provider_result_id: &provider_result.result_id,
        provider_invocation_id: &provider_result.invocation_id,
        case_generation: state_after_provider.generation,
        resource: &resource,
    };
    let operation =
        match normalize_filesystem_write_candidate(&provider_result.raw_output, &context) {
            Ok(operation) => operation,
            Err(failure) => {
                commit_normalization_failure(
                    &store,
                    &case_id,
                    &participant_id,
                    &provider_result.result_id,
                    failure.clone(),
                )?;
                println!("operation_normalization: rejected");
                println!("normalization_code: {:?}", failure.code);
                println!("external_effect: none");
                return Ok(());
            }
        };
    let state_after_operation = commit_operation(&store, &operation)?;
    println!("operation_normalization: accepted");
    println!("operation_id: {}", operation.operation_id);
    println!("operation_kind: filesystem.write");

    let decision = decide_filesystem_write(&operation, &resource, state_after_operation.generation);
    let state_after_decision = commit_decision(&store, &case_id, &decision)?;
    println!("decision_id: {}", decision.decision_id);
    println!(
        "decision: {}",
        match decision.outcome {
            DecisionOutcome::Allow => "allow",
            DecisionOutcome::Deny => "deny",
        }
    );
    if decision.outcome == DecisionOutcome::Deny {
        let second_provider_args = second_turn_provider_args(args, &case_id, &participant_id)?;
        drop(store);
        let second = invoke_controlled_provider(
            &second_provider_args,
            ProjectionPurpose::EffectConsequence,
            "Report the committed controlled filesystem outcome from this view.",
            InvocationOutputContract::NaturalLanguage,
        )?;
        println!("execution_grant: none");
        println!("external_effect: none");
        println!("second_provider_invocation_id: {}", second.invocation_id);
        println!("second_turn_consequence: committed_denial_no_effect");
        return Ok(());
    }

    let grant = issue_execution_grant(&operation, &decision, state_after_decision.generation)?;
    commit_grant(&store, &grant)?;
    println!("execution_grant_id: {}", grant.grant_id);
    if failpoint(args).as_deref() == Some("after_grant_before_prepare") {
        exit_at_failpoint("after_grant_before_prepare", 84);
    }

    let pre_observation = observe_filesystem(
        &binding,
        &resource,
        &operation.filesystem_write.relative_path,
        format!("observation:{}:pre", grant.grant_id),
    );
    if pre_observation.state == ResourceState::Unavailable {
        return Err(format!(
            "pre_effect_observation_unavailable: {}",
            pre_observation.error.as_deref().unwrap_or("unknown")
        ));
    }
    let prepared = prepare_effect(&operation, &decision, &grant, pre_observation)?;
    let state_after_prepare = commit_prepare(&store, &prepared)?;
    println!("effect_id: {}", prepared.effect_id);
    println!("effect_state: prepared_durable_before_mutation");
    if failpoint(args).as_deref() == Some("after_prepare_before_effect") {
        exit_at_failpoint("after_prepare_before_effect", 85);
    }

    let carrier_failpoint = match failpoint(args).as_deref() {
        Some("carrier_failure") => CarrierFailpoint::FailBeforeMutation,
        Some("after_effect_before_finalize") => CarrierFailpoint::CrashAfterVisibleEffect,
        _ => CarrierFailpoint::None,
    };
    let result = execute_filesystem_write(
        &operation,
        &decision,
        &grant,
        &prepared,
        &state_after_prepare,
        &binding,
        &resource,
        carrier_failpoint,
    )?;
    if result.crash_injected_after_effect {
        exit_at_failpoint("after_effect_before_finalize", 86);
    }
    if matches!(
        result.outcome,
        EffectOutcome::Conflict | EffectOutcome::Indeterminate
    ) {
        let state = commit_indeterminate(
            &store,
            &prepared,
            result.detail.clone(),
            Some(result.post_observation),
        )?;
        println!("effect_state: indeterminate");
        println!("case_generation: {}", state.generation);
        return Ok(());
    }
    let receipt = build_effect_receipt(&prepared, &result);
    if failpoint(args).as_deref() == Some("after_receipt_before_finalize") {
        eprintln!("prepared_receipt_id: {}", receipt.receipt_id);
        exit_at_failpoint("after_receipt_before_finalize", 87);
    }
    commit_finalize(&store, &prepared, &result)?;
    println!("effect_receipt_id: {}", receipt.receipt_id);
    println!("effect_outcome: {:?}", result.outcome);
    println!("effect_state: finalized");
    update_derived_after_commit(&store, &case_id, args);

    let transitions = store.list_case_transitions(&case_id)?;
    validate_finalized_effect_chain(&transitions, &prepared.effect_id)?;
    let second_provider_args = second_turn_provider_args(args, &case_id, &participant_id)?;
    drop(store);
    let second = invoke_controlled_provider(
        &second_provider_args,
        ProjectionPurpose::EffectConsequence,
        "Report the observed controlled filesystem consequence from this view.",
        InvocationOutputContract::NaturalLanguage,
    )?;
    println!("second_provider_invocation_id: {}", second.invocation_id);
    println!("second_provider_result_id: {}", second.result_id);
    println!("second_turn_consequence: observed_reality_from_canonical_state");
    println!("effect_chain_closure: valid");
    Ok(())
}

pub(super) struct CompatibilityReviewEffect {
    pub operation_id: String,
    pub decision_id: String,
    pub grant_id: String,
    pub effect_id: String,
    pub receipt_id: String,
}

/// Migrates the existing fixture-bound approved review write onto the same
/// typed Grant/PREPARE/carrier/FINALIZE path. Its review origin is explicit and
/// cannot be mistaken for provider-originated candidate material.
pub(super) fn execute_approved_review_filesystem_write(
    case_id: &str,
    review: &ReviewState,
    operator_reason: &str,
) -> Result<CompatibilityReviewEffect, String> {
    if review.status != ReviewResolution::PendingOperator {
        return Err("review effect requires a pending operator review".to_string());
    }
    let sandbox = fs::canonicalize(&review.sandbox_path)
        .map_err(|error| format!("failed to canonicalize review sandbox: {error}"))?;
    let root = sandbox
        .parent()
        .ok_or_else(|| "review sandbox has no binding root".to_string())?;
    let prefix = sandbox
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "review sandbox prefix is not UTF-8".to_string())?;
    let target = Path::new(&review.target_path);
    let filename = target
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "review target filename is not UTF-8".to_string())?;
    let relative_path = format!("{prefix}/{filename}");
    let attachment_id = format!("review-resource:{}", id_component(&review.review_id));
    let resource = ResourceAttachmentState {
        attachment_id: attachment_id.clone(),
        kind: ResourceKind::Filesystem,
        allowed_write_prefix: prefix.to_string(),
        max_write_bytes: 4096,
        policy_id: format!("policy:review:{}", id_component(&review.review_id)),
        policy_owner_participant_id: review.reviewer_participant.clone(),
    };
    resource.validate()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let mut state = store
        .get_case_state(case_id)?
        .ok_or_else(|| "canonical review CaseState missing".to_string())?;
    if !state
        .resources
        .iter()
        .any(|existing| existing.attachment_id == attachment_id)
    {
        state = commit_effect_transition(
            &store,
            case_id,
            Some(&review.reviewer_participant),
            &format!("resource-attached:{attachment_id}"),
            TransitionPayload::ResourceAttached {
                attachment: resource.clone(),
            },
            Some(TransitionScope {
                case_id: case_id.to_string(),
                participant_refs: vec![review.reviewer_participant.clone()],
                resource_refs: vec![attachment_id.clone()],
                policy_refs: vec![resource.policy_id.clone()],
            }),
            vec![review.reviewer_participant.clone()],
        )?;
    }
    let binding = LocalFilesystemBinding::new(case_id, &attachment_id, root)?;
    store.put_local_filesystem_binding(&binding)?;
    let operation = normalize_review_filesystem_write(
        case_id,
        &review.requested_by_participant,
        &review.review_id,
        &review.attempt_id,
        state.generation,
        &resource,
        &relative_path,
        "approved reviewed filesystem write\n",
    )
    .map_err(|failure| format!("review_operation_normalization_failed: {}", failure.detail))?;
    let state = commit_operation(&store, &operation)?;
    let decision = record_filesystem_decision(
        &operation,
        &resource,
        state.generation,
        DecisionOutcome::Allow,
        operator_reason,
    )?;
    let state = commit_decision(&store, case_id, &decision)?;
    let grant = issue_execution_grant(&operation, &decision, state.generation)?;
    commit_grant(&store, &grant)?;
    let pre = observe_filesystem(
        &binding,
        &resource,
        &relative_path,
        format!("observation:{}:pre", grant.grant_id),
    );
    let prepared = prepare_effect(&operation, &decision, &grant, pre)?;
    let state = commit_prepare(&store, &prepared)?;
    let result = execute_filesystem_write(
        &operation,
        &decision,
        &grant,
        &prepared,
        &state,
        &binding,
        &resource,
        CarrierFailpoint::None,
    )?;
    if !matches!(
        result.outcome,
        EffectOutcome::Applied | EffectOutcome::AlreadyApplied
    ) {
        commit_indeterminate(
            &store,
            &prepared,
            result.detail,
            Some(result.post_observation),
        )?;
        return Err("review filesystem effect did not establish intended post-state".to_string());
    }
    let receipt = build_effect_receipt(&prepared, &result);
    commit_finalize(&store, &prepared, &result)?;
    let transitions = store.list_case_transitions(case_id)?;
    validate_finalized_effect_chain(&transitions, &prepared.effect_id)?;
    Ok(CompatibilityReviewEffect {
        operation_id: operation.operation_id,
        decision_id: decision.decision_id,
        grant_id: grant.grant_id,
        effect_id: prepared.effect_id,
        receipt_id: receipt.receipt_id,
    })
}

#[derive(Clone)]
struct StoredEffectChain {
    operation: Operation,
    decision: Decision,
    grant: ExecutionGrant,
    prepared: PreparedEffect,
}

fn load_effect_chain(
    transitions: &[Transition],
    effect_id: &str,
) -> Result<StoredEffectChain, String> {
    let prepared = transitions
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::EffectPrepared { prepared } if prepared.effect_id == effect_id => {
                Some(prepared.clone())
            }
            _ => None,
        })
        .ok_or_else(|| format!("effect PREPARE not found: {effect_id}"))?;
    let operation = transitions
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::OperationRecorded { operation }
                if operation.operation_id == prepared.operation_id =>
            {
                Some(operation.clone())
            }
            _ => None,
        })
        .ok_or_else(|| "prepared effect operation missing".to_string())?;
    let decision = transitions
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::DecisionRecorded { decision }
                if decision.decision_id == prepared.decision_id =>
            {
                Some(decision.clone())
            }
            _ => None,
        })
        .ok_or_else(|| "prepared effect decision missing".to_string())?;
    let grant = transitions
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::ExecutionGrantIssued { grant }
                if grant.grant_id == prepared.grant_id =>
            {
                Some(grant.clone())
            }
            _ => None,
        })
        .ok_or_else(|| "prepared effect grant missing".to_string())?;
    Ok(StoredEffectChain {
        operation,
        decision,
        grant,
        prepared,
    })
}

fn reconciliation_result(
    chain: &StoredEffectChain,
    observation: FilesystemObservation,
    conclusion: &ReconciliationConclusion,
) -> CarrierResult {
    CarrierResult {
        outcome: match conclusion {
            ReconciliationConclusion::EffectObserved => EffectOutcome::AlreadyApplied,
            ReconciliationConclusion::NoEffectObserved => EffectOutcome::NoEffect,
            ReconciliationConclusion::Conflict => EffectOutcome::Conflict,
            ReconciliationConclusion::StillIndeterminate => EffectOutcome::Indeterminate,
        },
        post_observation: observation,
        carrier_attempted: false,
        mutation_performed: false,
        crash_injected_after_effect: false,
        detail: format!("reconciled prepared effect {}", chain.prepared.effect_id),
    }
}

pub(super) fn controlled_effect_reconcile(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let requested_effect = optional_arg(args, "--effect");
    let retry = args.iter().any(|arg| arg == "--retry");
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store
        .get_case_state(&case_id)?
        .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
    let effect_state = if let Some(effect_id) = requested_effect.as_deref() {
        state
            .effects
            .iter()
            .find(|effect| effect.effect_id == effect_id)
    } else {
        state.effects.iter().find(|effect| {
            matches!(
                effect.status,
                EffectLifecycle::Prepared | EffectLifecycle::Indeterminate
            )
        })
    }
    .ok_or_else(|| "no matching prepared or finalized effect".to_string())?;
    if effect_state.status == EffectLifecycle::Finalized {
        println!("reconciliation: already_finalized");
        println!("effect_id: {}", effect_state.effect_id);
        println!(
            "receipt_id: {}",
            effect_state.receipt_id.as_deref().unwrap_or("none")
        );
        return Ok(());
    }
    let transitions = store.list_case_transitions(&case_id)?;
    let chain = load_effect_chain(&transitions, &effect_state.effect_id)?;
    let resource = resource_for_case(&state, &chain.prepared.resource_attachment_id)?;
    let binding = store
        .get_local_filesystem_binding(&case_id, &resource.attachment_id)?
        .ok_or_else(|| "local filesystem binding unavailable for reconciliation".to_string())?;
    let observation = observe_filesystem(
        &binding,
        &resource,
        &chain.prepared.relative_path,
        format!("observation:{}:reconcile", chain.prepared.effect_id),
    );
    let mut conclusion = classify_reconciliation(&chain.prepared, &observation);

    if conclusion == ReconciliationConclusion::NoEffectObserved
        && retry
        && effect_state.status == EffectLifecycle::Prepared
    {
        let result = execute_filesystem_write(
            &chain.operation,
            &chain.decision,
            &chain.grant,
            &chain.prepared,
            &state,
            &binding,
            &resource,
            CarrierFailpoint::None,
        )?;
        conclusion = if matches!(
            result.outcome,
            EffectOutcome::Applied | EffectOutcome::AlreadyApplied
        ) {
            ReconciliationConclusion::EffectObserved
        } else if matches!(
            result.outcome,
            EffectOutcome::FailedNoEffect | EffectOutcome::NoEffect
        ) {
            ReconciliationConclusion::NoEffectObserved
        } else if result.outcome == EffectOutcome::Conflict {
            ReconciliationConclusion::Conflict
        } else {
            ReconciliationConclusion::StillIndeterminate
        };
        return commit_reconciliation(
            &store,
            &chain,
            conclusion,
            result.post_observation.clone(),
            Some(result),
        );
    }
    commit_reconciliation(&store, &chain, conclusion, observation, None)
}

fn commit_reconciliation(
    store: &LmdbRecordStore,
    chain: &StoredEffectChain,
    conclusion: ReconciliationConclusion,
    observation: FilesystemObservation,
    carrier_result: Option<CarrierResult>,
) -> Result<(), String> {
    let receipt = if matches!(
        conclusion,
        ReconciliationConclusion::EffectObserved | ReconciliationConclusion::NoEffectObserved
    ) {
        let result = carrier_result
            .unwrap_or_else(|| reconciliation_result(chain, observation.clone(), &conclusion));
        Some(build_effect_receipt(&chain.prepared, &result))
    } else {
        None
    };
    let refs = vec![chain.prepared.effect_id.clone()];
    let state = commit_effect_transition(
        store,
        &chain.prepared.case_id,
        Some(&chain.prepared.participant_id),
        &format!("reconcile:{}", chain.prepared.effect_id),
        TransitionPayload::EffectReconciled {
            effect_id: chain.prepared.effect_id.clone(),
            conclusion: conclusion.clone(),
            observation,
            receipt,
        },
        None,
        refs,
    )?;
    println!("reconciliation: {:?}", conclusion);
    println!("effect_id: {}", chain.prepared.effect_id);
    println!(
        "effect_state: {:?}",
        state
            .effects
            .iter()
            .find(|effect| effect.effect_id == chain.prepared.effect_id)
            .map(|effect| &effect.status)
    );
    Ok(())
}

pub(super) fn controlled_effect_inspect(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let effect_id = named_arg(args, "--effect")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let transitions = store.list_case_transitions(&case_id)?;
    let chain = load_effect_chain(&transitions, &effect_id)?;
    let state = store
        .get_case_state(&case_id)?
        .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
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
