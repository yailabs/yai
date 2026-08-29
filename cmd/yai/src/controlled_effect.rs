//! Product-owned controlled filesystem transition family.
//!
//! This module owns the one constitutional filesystem.write orchestration:
//! provider candidate normalization, deterministic admission, prepared-effect
//! recovery, the grant-validated Rust carrier, and typed consequence views.
//! It is deliberately not a generic carrier registry or policy engine.

use super::*;
use yai_core_engine::admission::{
    build_policy_review_request, evaluate_filesystem_admission, resolve_policy_review_decision,
    EvidenceContext,
};
use yai_core_engine::case_policy::NormativeReadiness;
use yai_core_engine::effect::{
    build_effect_receipt, classify_reconciliation, execute_filesystem_write,
    issue_policy_execution_grant, normalize_filesystem_write_candidate, normalize_write_prefix,
    observe_filesystem, prepare_effect, validate_finalized_effect_chain, CarrierFailpoint,
    CarrierResult, Decision, DecisionOutcome, EffectOutcome, ExecutionGrant, FilesystemObservation,
    LocalFilesystemBinding, NormalizationContext, Operation, PreparedEffect,
    ReconciliationConclusion, ResourceState, EXECUTION_GRANT_SCHEMA, OPERATION_PROPOSAL_SCHEMA,
};
use yai_core_engine::transition::{
    CaseState, EffectLifecycle, PendingTransition, ResourceAttachmentState, ResourceKind,
    ReviewRequirement, ReviewResolution, ReviewState, Transition, TransitionPayload,
    TransitionScope, TransitionSource, REVIEW_REQUEST_SCHEMA,
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
        review_requirement: review_requirement.clone(),
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
    let participant_id = decision
        .decision_basis
        .as_ref()
        .map(|basis| basis.proposer_participant_id.as_str())
        .or_else(|| {
            decision
                .source
                .as_ref()
                .map(|source| source.owner_participant_id.as_str())
        });
    let mut causal_refs = vec![decision.operation_id.clone()];
    if let Some(basis) = &decision.decision_basis {
        causal_refs.push(basis.basis_id.clone());
        causal_refs.push(basis.effective_policy_id.clone());
        causal_refs.extend(basis.policy_binding_refs.iter().cloned());
        causal_refs.extend(basis.policy_artifact_refs.iter().cloned());
        if let Some(action) = &basis.review_action_ref {
            causal_refs.push(action.clone());
        }
    } else {
        causal_refs.extend(decision.basis_refs.iter().cloned());
    }
    commit_effect_transition(
        store,
        case_id,
        participant_id,
        &format!("decision:{}", decision.decision_id),
        TransitionPayload::DecisionRecorded {
            decision: decision.clone(),
        },
        None,
        causal_refs,
    )
}

fn commit_review_request(
    store: &LmdbRecordStore,
    case_id: &str,
    review: &ReviewState,
) -> Result<CaseState, String> {
    let mut causal_refs = vec![
        review.operation_id.clone(),
        review.initial_decision_id.clone(),
    ];
    if review.schema == REVIEW_REQUEST_SCHEMA {
        causal_refs.push(review.decision_basis_id.clone());
        causal_refs.push(review.effective_policy_id.clone());
        causal_refs.extend(review.policy_binding_refs.iter().cloned());
        causal_refs.extend(review.policy_artifact_refs.iter().cloned());
    }
    commit_effect_transition(
        store,
        case_id,
        Some(&review.requested_by_participant),
        &format!("review-request:{}", review.review_id),
        TransitionPayload::ReviewRequested {
            review: review.clone(),
        },
        None,
        causal_refs,
    )
}

fn commit_grant(store: &LmdbRecordStore, grant: &ExecutionGrant) -> Result<CaseState, String> {
    let mut causal_refs = vec![grant.operation_id.clone(), grant.decision_id.clone()];
    if grant.schema == EXECUTION_GRANT_SCHEMA {
        if let Some(basis_id) = &grant.decision_basis_id {
            causal_refs.push(basis_id.clone());
        }
        if let Some(effective_policy_id) = &grant.effective_policy_id {
            causal_refs.push(effective_policy_id.clone());
        }
        causal_refs.extend(grant.policy_binding_refs.iter().cloned());
        causal_refs.extend(grant.policy_artifact_refs.iter().cloned());
        if let Some(action_id) = &grant.review_action_ref {
            causal_refs.push(action_id.clone());
        }
    }
    commit_effect_transition(
        store,
        &grant.case_id,
        Some(&grant.participant_id),
        &format!("grant:{}", grant.grant_id),
        TransitionPayload::ExecutionGrantIssued {
            grant: grant.clone(),
        },
        None,
        causal_refs,
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
    match store
        .list_case_transitions(case_id)
        .and_then(|transitions| {
            derive_operational_memory(case_id, &transitions)
                .and_then(|build| store.replace_case_operational_memory(&build).map(|_| build))
        }) {
        Ok(build) => println!("derived_memory_entries: {}", build.entries.len()),
        Err(error) => eprintln!("derived_memory_failed_canonical_state_preserved: {error}"),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ControlledEffectTurnStatus {
    NormalizationRejected,
    Denied,
    AwaitingReview,
    Finalized,
    Indeterminate,
}

#[derive(Clone, Debug)]
pub(super) struct ControlledEffectTurnResult {
    pub status: ControlledEffectTurnStatus,
    pub operation_id: Option<String>,
    pub decision_id: Option<String>,
    pub review_id: Option<String>,
    pub effect_id: Option<String>,
    pub receipt_id: Option<String>,
    pub outcome: Option<EffectOutcome>,
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
    let state = store
        .get_case_state(case_id)?
        .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
    let resource = resource_for_case(&state, attachment_id)?;
    let normative = store.case_policy_status(case_id)?;
    let effective_policy = normative
        .effective_policy
        .clone()
        .filter(|_| normative.readiness == NormativeReadiness::Ready)
        .ok_or_else(|| format!("normative_case_not_ready: {:?}", normative.readiness))?;
    let binding = store
        .get_local_filesystem_binding(case_id, attachment_id)?
        .ok_or_else(|| format!("local binding missing for {case_id}/{attachment_id}"))?;
    let existing = store.list_case_transitions(case_id)?;
    if existing.iter().any(|transition| {
        matches!(
            &transition.payload,
            TransitionPayload::OperationNormalizationFailed { provider_result_id, .. }
                if provider_result_id == &provider_result.result_id
        )
    }) {
        return Ok(ControlledEffectTurnResult {
            status: ControlledEffectTurnStatus::NormalizationRejected,
            operation_id: None,
            decision_id: None,
            review_id: None,
            effect_id: None,
            receipt_id: None,
            outcome: None,
        });
    }
    let existing_operation = existing
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::OperationRecorded { operation }
                if matches!(
                    &operation.origin,
                    yai_core_engine::effect::OperationOrigin::ProviderResult {
                        provider_result_id,
                        ..
                    } if provider_result_id == &provider_result.result_id
                ) =>
            {
                Some(operation.clone())
            }
            _ => None,
        });
    let (operation, state_after_operation) = if let Some(operation) = existing_operation {
        let state = store
            .get_case_state(case_id)?
            .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
        (operation, state)
    } else {
        let state_after_provider = store
            .get_case_state(case_id)?
            .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
        let context = NormalizationContext {
            case_id,
            participant_id,
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
                        case_id,
                        participant_id,
                        &provider_result.result_id,
                        failure.clone(),
                    )?;
                    update_derived_after_commit(&store, case_id, args);
                    println!("operation_normalization: rejected");
                    println!("normalization_code: {:?}", failure.code);
                    println!("external_effect: none");
                    return Ok(ControlledEffectTurnResult {
                        status: ControlledEffectTurnStatus::NormalizationRejected,
                        operation_id: None,
                        decision_id: None,
                        review_id: None,
                        effect_id: None,
                        receipt_id: None,
                        outcome: None,
                    });
                }
            };
        let state = commit_operation(&store, &operation)?;
        (operation, state)
    };
    println!("operation_normalization: accepted");
    println!("operation_id: {}", operation.operation_id);
    println!("operation_kind: filesystem.write");

    let existing_decisions = existing
        .iter()
        .filter_map(|transition| match &transition.payload {
            TransitionPayload::DecisionRecorded { decision }
                if decision.operation_id == operation.operation_id =>
            {
                Some(decision.clone())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let existing_effective = existing_decisions
        .iter()
        .rev()
        .find(|decision| decision.outcome != DecisionOutcome::RequireReview)
        .cloned();
    let (decision, state_after_decision) = if let Some(decision) = existing_effective {
        let state = store
            .get_case_state(case_id)?
            .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
        (decision, state)
    } else {
        let (initial, state_after_initial) = if let Some(decision) = existing_decisions
            .iter()
            .find(|decision| decision.outcome == DecisionOutcome::RequireReview)
            .cloned()
        {
            let state = store
                .get_case_state(case_id)?
                .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
            (decision, state)
        } else {
            let canonical_provider_lineage = existing.iter().any(|transition| {
                matches!(
                    &transition.payload,
                    TransitionPayload::ProviderResultRecorded { result_id, invocation_id, .. }
                        if result_id == &provider_result.result_id
                            && invocation_id == &provider_result.invocation_id
                )
            }) && existing.iter().any(|transition| {
                matches!(
                    &transition.payload,
                    TransitionPayload::ProviderInvocationStarted { invocation_id, .. }
                        if invocation_id == &provider_result.invocation_id
                )
            });
            let evidence = EvidenceContext {
                canonical_provider_result_id: canonical_provider_lineage
                    .then(|| provider_result.result_id.clone()),
                canonical_provider_invocation_id: canonical_provider_lineage
                    .then(|| provider_result.invocation_id.clone()),
                review_action_id: None,
                review_reason: None,
            };
            let decision = evaluate_filesystem_admission(
                &operation,
                &state_after_operation,
                &resource,
                &effective_policy,
                &evidence,
            )?;
            let state = commit_decision(&store, case_id, &decision)?;
            (decision, state)
        };
        if initial.outcome != DecisionOutcome::RequireReview {
            (initial, state_after_initial)
        } else {
            if matches!(
                failpoint(args).as_deref(),
                Some("review_after_require_decision" | "review_r1")
            ) {
                exit_at_failpoint("review_after_require_decision", 94);
            }
            let review = if let Some(review) = state_after_initial
                .reviews
                .iter()
                .find(|review| review.operation_id == operation.operation_id)
                .cloned()
            {
                review
            } else {
                let review = build_policy_review_request(
                    &operation,
                    &initial,
                    state_after_initial.generation,
                )?;
                commit_review_request(&store, case_id, &review)?;
                if matches!(
                    failpoint(args).as_deref(),
                    Some("review_after_request" | "review_r2")
                ) {
                    exit_at_failpoint("review_after_request", 95);
                }
                review
            };
            if matches!(
                review.status,
                ReviewResolution::Pending
                    | ReviewResolution::PendingOperator
                    | ReviewResolution::Deferred
            ) {
                update_derived_after_commit(&store, case_id, args);
                println!("decision: require_review");
                println!("review_id: {}", review.review_id);
                println!("execution_grant: none");
                println!("external_effect: none");
                return Ok(ControlledEffectTurnResult {
                    status: ControlledEffectTurnStatus::AwaitingReview,
                    operation_id: Some(operation.operation_id),
                    decision_id: Some(initial.decision_id),
                    review_id: Some(review.review_id),
                    effect_id: None,
                    receipt_id: None,
                    outcome: None,
                });
            }
            if review.status == ReviewResolution::Quarantined {
                return Err("legacy_quarantined_review_is_not_executable".to_string());
            }
            let action_id = review
                .latest_action_id
                .as_ref()
                .ok_or_else(|| "resolved review is missing its human action".to_string())?;
            let action = existing
                .iter()
                .find_map(|transition| match &transition.payload {
                    TransitionPayload::ReviewActionRecorded { action }
                        if action.action_id == *action_id =>
                    {
                        Some(action.clone())
                    }
                    _ => None,
                })
                .ok_or_else(|| "review action transition is missing".to_string())?;
            let current = store
                .get_case_state(case_id)?
                .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
            let current_normative = store.case_policy_status(case_id)?;
            let current_effective = current_normative
                .effective_policy
                .as_ref()
                .filter(|_| current_normative.readiness == NormativeReadiness::Ready)
                .ok_or_else(|| "review_policy_basis_stale".to_string())?;
            let effective = resolve_policy_review_decision(
                &operation,
                &current,
                &resource,
                current_effective,
                &review,
                &action,
            )?;
            let state = commit_decision(&store, case_id, &effective)?;
            if effective.outcome == DecisionOutcome::Allow
                && matches!(
                    failpoint(args).as_deref(),
                    Some("review_after_allow_decision" | "review_r4")
                )
            {
                exit_at_failpoint("review_after_allow_decision", 96);
            }
            if effective.outcome == DecisionOutcome::Deny
                && matches!(
                    failpoint(args).as_deref(),
                    Some("review_after_deny_decision" | "review_r6")
                )
            {
                exit_at_failpoint("review_after_deny_decision", 97);
            }
            (effective, state)
        }
    };
    println!("decision_id: {}", decision.decision_id);
    println!("decision_reason: {}", decision.reason);
    if let Some(basis) = &decision.decision_basis {
        println!("decision_basis_id: {}", basis.basis_id);
        println!("effective_policy_id: {}", basis.effective_policy_id);
        println!(
            "matched_policy_rules: {}",
            basis.matched_rule_refs.join(",")
        );
        println!(
            "authority_requirements: {}",
            serde_json::to_string(&basis.authority)
                .map_err(|error| format!("authority_render_failed: {error}"))?
        );
        println!(
            "evidence_obligations: {}",
            serde_json::to_string(&basis.obligations)
                .map_err(|error| format!("obligation_render_failed: {error}"))?
        );
    }
    println!(
        "decision: {}",
        match decision.outcome {
            DecisionOutcome::Allow => "allow",
            DecisionOutcome::Deny => "deny",
            DecisionOutcome::RequireReview => "require_review",
        }
    );
    if decision.outcome == DecisionOutcome::Deny {
        update_derived_after_commit(&store, case_id, args);
        println!("execution_grant: none");
        println!("external_effect: none");
        return Ok(ControlledEffectTurnResult {
            status: ControlledEffectTurnStatus::Denied,
            operation_id: Some(operation.operation_id.clone()),
            decision_id: Some(decision.decision_id),
            review_id: state_after_decision
                .reviews
                .iter()
                .find(|review| review.operation_id == operation.operation_id)
                .map(|review| review.review_id.clone()),
            effect_id: None,
            receipt_id: None,
            outcome: None,
        });
    }

    let existing_grant = existing
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::ExecutionGrantIssued { grant }
                if grant.operation_id == operation.operation_id =>
            {
                Some(grant.clone())
            }
            _ => None,
        });
    let grant = if let Some(grant) = existing_grant {
        grant
    } else {
        if state_after_decision
            .last_decision
            .as_ref()
            .is_none_or(|current| current.decision_id != decision.decision_id)
        {
            return Err("execution_grant_requires_latest_case_decision".to_string());
        }
        let grant =
            issue_policy_execution_grant(&operation, &decision, state_after_decision.generation)?;
        commit_grant(&store, &grant)?;
        grant
    };
    println!("execution_grant_id: {}", grant.grant_id);
    println!(
        "execution_grant_decision_basis_id: {}",
        grant.decision_basis_id.as_deref().unwrap_or("none")
    );
    if matches!(
        failpoint(args).as_deref(),
        Some("after_grant_before_prepare" | "review_r5")
    ) {
        exit_at_failpoint("after_grant_before_prepare", 84);
    }

    let existing_prepared = existing
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::EffectPrepared { prepared }
                if prepared.operation_id == operation.operation_id =>
            {
                Some(prepared.clone())
            }
            _ => None,
        });
    let (prepared, state_after_prepare) = if let Some(prepared) = existing_prepared {
        let current = store
            .get_case_state(case_id)?
            .ok_or_else(|| format!("canonical CaseState missing for {case_id}"))?;
        if let Some(effect) = current
            .effects
            .iter()
            .find(|effect| effect.effect_id == prepared.effect_id)
        {
            if effect.status == EffectLifecycle::Finalized {
                return Ok(ControlledEffectTurnResult {
                    status: ControlledEffectTurnStatus::Finalized,
                    operation_id: Some(operation.operation_id),
                    decision_id: Some(decision.decision_id),
                    review_id: None,
                    effect_id: Some(prepared.effect_id),
                    receipt_id: effect.receipt_id.clone(),
                    outcome: effect.outcome.clone(),
                });
            }
            if effect.status == EffectLifecycle::Indeterminate {
                return Ok(ControlledEffectTurnResult {
                    status: ControlledEffectTurnStatus::Indeterminate,
                    operation_id: Some(operation.operation_id),
                    decision_id: Some(decision.decision_id),
                    review_id: None,
                    effect_id: Some(prepared.effect_id),
                    receipt_id: None,
                    outcome: effect.outcome.clone(),
                });
            }
        }
        (prepared, current)
    } else {
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
        let state = commit_prepare(&store, &prepared)?;
        (prepared, state)
    };
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
        update_derived_after_commit(&store, case_id, args);
        println!("effect_state: indeterminate");
        println!("case_generation: {}", state.generation);
        return Ok(ControlledEffectTurnResult {
            status: ControlledEffectTurnStatus::Indeterminate,
            operation_id: Some(operation.operation_id),
            decision_id: Some(decision.decision_id),
            review_id: None,
            effect_id: Some(prepared.effect_id),
            receipt_id: None,
            outcome: Some(result.outcome),
        });
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
    update_derived_after_commit(&store, case_id, args);
    let transitions = store.list_case_transitions(case_id)?;
    validate_finalized_effect_chain(&transitions, &prepared.effect_id)?;
    println!("effect_chain_closure: valid");
    Ok(ControlledEffectTurnResult {
        status: ControlledEffectTurnStatus::Finalized,
        operation_id: Some(operation.operation_id),
        decision_id: Some(decision.decision_id),
        review_id: None,
        effect_id: Some(prepared.effect_id),
        receipt_id: Some(receipt.receipt_id),
        outcome: Some(result.outcome),
    })
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
    let normative = store.case_policy_status(&case_id)?;
    if normative.readiness != NormativeReadiness::Ready {
        println!("normative_readiness: {:?}", normative.readiness);
        println!("provider_invocations: 0");
        println!("execution_grants: 0");
        println!("external_effect: none");
        return Err(format!(
            "normative_case_not_ready: {:?}",
            normative.readiness
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum CaseReconciliationStatus {
    Clean,
    Reconciled { count: usize },
    Unresolved { effect_ids: Vec<String> },
}

/// Reconcile every currently unresolved filesystem effect before a new model
/// invocation. This is intentionally a synchronous Case boundary, not a
/// background scheduler.
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
        .map(|effect| effect.effect_id.clone())
        .collect::<Vec<_>>();
    drop(store);
    if unresolved.is_empty() {
        return Ok(CaseReconciliationStatus::Clean);
    }
    for effect_id in &unresolved {
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
