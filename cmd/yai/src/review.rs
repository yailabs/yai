//! Case-native human review boundary.
//!
//! Review is a durable admission posture for an existing Operation. This
//! module records eligible participant actions and the resulting effective
//! Decision; it never invokes a carrier or performs an external effect.

use super::*;
use crate::command_adapters::security::authenticate_local;
use yai_core_engine::admission::reviewer_is_eligible;
use yai_core_engine::case_policy::NormativeReadiness;
use yai_core_engine::effect::{DecisionOutcome, Operation};
use yai_core_engine::transition::{
    build_authenticated_review_action, CaseLifecycle, ReviewAction, ReviewActionKind,
    ReviewResolution, ReviewState, Transition, REVIEW_REQUEST_SCHEMA,
};

const REVIEW_COMPONENT: &str = "yai.human_review_boundary";
const LOCAL_OPERATOR_SOURCE: &str = "kernel_authenticated_principal_participant_link";

fn review_status_label(status: &ReviewResolution) -> &'static str {
    match status {
        ReviewResolution::Pending => "pending",
        ReviewResolution::Deferred => "deferred",
        ReviewResolution::Approved => "approved",
        ReviewResolution::Denied => "denied",
        ReviewResolution::PendingOperator => "compatibility_pending_operator",
        ReviewResolution::Quarantined => "compatibility_quarantined",
        ReviewResolution::Invalidated => "invalidated",
    }
}

fn review_is_open(status: &ReviewResolution) -> bool {
    matches!(
        status,
        ReviewResolution::Pending | ReviewResolution::Deferred
    )
}

fn action_label(action: &ReviewActionKind) -> &'static str {
    match action {
        ReviewActionKind::Approve => "approve",
        ReviewActionKind::Deny => "deny",
        ReviewActionKind::Defer => "defer",
    }
}

fn case_review(
    state: &yai_core_engine::transition::CaseState,
    review_id: &str,
) -> Result<ReviewState, String> {
    state
        .reviews
        .iter()
        .find(|review| review.review_id == review_id)
        .cloned()
        .ok_or_else(|| format!("review_not_found: {review_id}"))
}

fn operation_for_review(
    transitions: &[Transition],
    review: &ReviewState,
) -> Result<Operation, String> {
    transitions
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::OperationRecorded { operation }
                if operation.operation_id == review.operation_id =>
            {
                Some(operation.clone())
            }
            _ => None,
        })
        .ok_or_else(|| "review_operation_transition_missing".to_string())
}

fn action_by_id(transitions: &[Transition], action_id: &str) -> Option<ReviewAction> {
    transitions
        .iter()
        .find_map(|transition| match &transition.payload {
            TransitionPayload::ReviewActionRecorded { action } if action.action_id == action_id => {
                Some(action.clone())
            }
            _ => None,
        })
}

fn update_review_derivations(store: &LmdbRecordStore, case_id: &str) {
    if let Err(error) = store.materialize_graph_relations_for_case(case_id) {
        eprintln!("review_graph_derivation_failed: {error}");
    }
    if let Err(error) = store
        .list_case_transitions(case_id)
        .and_then(|transitions| {
            derive_operational_memory(case_id, &transitions)
                .and_then(|build| store.replace_case_operational_memory(&build))
        })
    {
        eprintln!("review_memory_derivation_failed: {error}");
    }
}

fn print_review(review: &ReviewState, case_id: &str) {
    println!(
        "review_schema: {}",
        if review.schema.is_empty() {
            "legacy_compatibility"
        } else {
            &review.schema
        }
    );
    println!("review_id: {}", review.review_id);
    println!("case_id: {case_id}");
    println!(
        "operation_id: {}",
        if review.operation_id.is_empty() {
            "compatibility_unmapped"
        } else {
            &review.operation_id
        }
    );
    println!(
        "initial_decision_id: {}",
        if review.initial_decision_id.is_empty() {
            "compatibility_unmapped"
        } else {
            &review.initial_decision_id
        }
    );
    println!(
        "requesting_participant: {}",
        review.requested_by_participant
    );
    if review.schema == REVIEW_REQUEST_SCHEMA {
        println!(
            "required_reviewer_roles: {}",
            review.required_reviewer_roles.join(",")
        );
        println!("decision_basis_id: {}", review.decision_basis_id);
        println!("effective_policy_id: {}", review.effective_policy_id);
    } else {
        println!("eligible_reviewer: {}", review.reviewer_participant);
    }
    println!(
        "resource_attachment_id: {}",
        if review.resource_attachment_id.is_empty() {
            "compatibility_unmapped"
        } else {
            &review.resource_attachment_id
        }
    );
    println!(
        "normalized_target: {}",
        if review.normalized_target.is_empty() {
            "compatibility_unmapped"
        } else {
            &review.normalized_target
        }
    );
    println!("reason: {}", review.policy_reason);
    println!("status: {}", review_status_label(&review.status));
    if let Some(reason) = &review.invalidation_reason {
        println!("invalidation_reason: {reason:?}");
    }
    println!(
        "latest_action_id: {}",
        review.latest_action_id.as_deref().unwrap_or("none")
    );
    println!(
        "effective_decision_id: {}",
        review.effective_decision_id.as_deref().unwrap_or("none")
    );
    println!("effect_evidence: none_owned_by_review");
    println!("operator_trust_boundary: kernel_authenticated_principal_participant_link");
}

pub(super) fn review_pending(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store.get_case_state_authorized(&authenticated, &case_id)?;
    let pending = state
        .reviews
        .iter()
        .filter(|review| review_is_open(&review.status))
        .collect::<Vec<_>>();
    println!("review_pending:");
    println!("case_id: {case_id}");
    println!("items_total: {}", pending.len());
    for review in pending {
        println!("-");
        print_review(review, &case_id);
    }
    Ok(())
}

pub(super) fn review_show(args: &[String]) -> Result<(), String> {
    let review_id = args
        .first()
        .filter(|value| !value.starts_with("--"))
        .ok_or_else(|| "review_id is required".to_string())?;
    let case_id = named_arg(args, "--case")?;
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store.get_case_state_authorized(&authenticated, &case_id)?;
    print_review(&case_review(&state, review_id)?, &case_id);
    Ok(())
}

fn commit_review_action(
    store: &LmdbRecordStore,
    authenticated: &yai_core_engine::security::AuthenticatedPrincipal,
    tenant_id: &str,
    state: &yai_core_engine::transition::CaseState,
    action: &ReviewAction,
) -> Result<yai_core_engine::transition::CaseState, String> {
    let mut pending = PendingTransition::new(
        format!("transition:{}", action.action_id),
        &state.case_id,
        state.generation,
        TransitionSource {
            component: REVIEW_COMPONENT.to_string(),
            participant_id: Some(action.reviewer_participant_id.clone()),
            principal_id: action.principal_id.clone(),
            source_ref: Some(action.action_id.clone()),
        },
        TransitionPayload::ReviewActionRecorded {
            action: action.clone(),
        },
    );
    pending.causal_refs = vec![action.review_id.clone(), action.operation_id.clone()];
    store
        .commit_secured_transition(authenticated, tenant_id, pending, false)
        .map(|commit| commit.state)
}

pub(super) fn review_resolve(args: &[String], requested: ReviewActionKind) -> Result<(), String> {
    let review_id = args
        .first()
        .filter(|value| !value.starts_with("--"))
        .ok_or_else(|| "review_id is required".to_string())?;
    let case_id = named_arg(args, "--case")?;
    if optional_arg(args, "--as").is_some() {
        return Err("reviewer_selection_by_as_is_forbidden_for_tenant_case".to_string());
    }
    let reason = named_arg(args, "--reason")?;
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let initial_state = store.get_case_state_authorized(&authenticated, &case_id)?;
    let tenant_id = initial_state
        .tenant_id
        .clone()
        .ok_or_else(|| "legacy_review_is_compatibility_only".to_string())?;
    let principal_id = authenticated.projected_principal_id();
    let reviewer = initial_state
        .principal_participant_links
        .iter()
        .find(|link| link.principal_id == principal_id && link.tenant_id == tenant_id)
        .map(|link| link.participant_id.clone())
        .ok_or_else(|| "authenticated_principal_participant_link_required".to_string())?;
    if initial_state.lifecycle != CaseLifecycle::Open {
        return Err("review_action_requires_open_case".to_string());
    }
    let initial_review = case_review(&initial_state, review_id)?;
    if initial_review.schema != REVIEW_REQUEST_SCHEMA {
        return Err("legacy_review_is_compatibility_only".to_string());
    }
    if let Some(commit) = store.invalidate_review_if_policy_unusable(&case_id, review_id)? {
        let invalidated = case_review(&commit.state, review_id)?;
        println!("review_invalidation: committed");
        println!("review_id: {review_id}");
        println!("case_generation: {}", commit.state.generation);
        println!("invalidation_reason: {:?}", invalidated.invalidation_reason);
        return Err("review_authority_invalidated".to_string());
    }
    let state = store.get_case_state_authorized(&authenticated, &case_id)?;
    let review = case_review(&state, review_id)?;
    if !reviewer_is_eligible(&state, &review, &reviewer) {
        return Err("reviewer_not_eligible_for_case_review".to_string());
    }
    let normative = store.case_policy_status(&case_id)?;
    let effective_policy = normative
        .effective_policy
        .as_ref()
        .filter(|_| {
            normative.readiness == NormativeReadiness::Ready
                && normative.validity == yai_core_engine::case_policy::PolicyValidityPosture::Valid
        })
        .ok_or_else(|| "review_policy_basis_stale".to_string())?;
    if review.effective_policy_id != effective_policy.effective_policy_id
        || review.effective_policy_digest != effective_policy.semantic_digest
    {
        return Err("review_policy_basis_stale".to_string());
    }
    let transitions = store.list_case_transitions(&case_id)?;
    if !review_is_open(&review.status) {
        let same_resolution = matches!(
            (&review.status, &requested),
            (ReviewResolution::Approved, ReviewActionKind::Approve)
                | (ReviewResolution::Denied, ReviewActionKind::Deny)
        );
        if same_resolution {
            println!("review_action: already_resolved_idempotent");
            print_review(&review, &case_id);
            return Ok(());
        }
        return Err(format!(
            "review_already_resolved: {}",
            review_status_label(&review.status)
        ));
    }
    if review.status == ReviewResolution::Deferred {
        if let Some(existing) = review
            .latest_action_id
            .as_deref()
            .and_then(|action_id| action_by_id(&transitions, action_id))
        {
            let normalized_reason = reason.split_whitespace().collect::<Vec<_>>().join(" ");
            if existing.action == requested
                && existing.reviewer_participant_id == reviewer
                && existing.reason == normalized_reason
            {
                println!("review_action: already_recorded_idempotent");
                print_review(&review, &case_id);
                return Ok(());
            }
        }
    }
    let action = build_authenticated_review_action(
        &review,
        &case_id,
        &tenant_id,
        &principal_id,
        &reviewer,
        requested.clone(),
        &reason,
        state.generation,
        LOCAL_OPERATOR_SOURCE,
    )?;
    let state_after_action =
        commit_review_action(&store, &authenticated, &tenant_id, &state, &action)?;
    if optional_arg(args, "--failpoint").as_deref() == Some("review_r3") {
        eprintln!("review_crash_injected: review_r3");
        std::process::exit(103);
    }
    let mut effective_decision_id = None;
    if requested != ReviewActionKind::Defer {
        let current_review = case_review(&state_after_action, review_id)?;
        let operation = operation_for_review(&transitions, &current_review)?;
        let (effective, commit) = store.derive_and_commit_policy_review_decision(
            &case_id,
            &operation.operation_id,
            &current_review.review_id,
            &action.action_id,
        )?;
        let state_after_decision = commit.state;
        effective_decision_id = Some(effective.decision_id.clone());
        let failpoint = optional_arg(args, "--failpoint");
        if effective.outcome == DecisionOutcome::Allow && failpoint.as_deref() == Some("review_r4")
        {
            eprintln!("review_crash_injected: review_r4");
            std::process::exit(104);
        }
        if effective.outcome == DecisionOutcome::Deny && failpoint.as_deref() == Some("review_r6") {
            eprintln!("review_crash_injected: review_r6");
            std::process::exit(106);
        }
        if state_after_decision.lifecycle != CaseLifecycle::Open {
            return Err("review_resolution_closed_case_race".to_string());
        }
    }
    update_review_derivations(&store, &case_id);
    println!("review_action: committed");
    println!("review_id: {review_id}");
    println!("case_id: {case_id}");
    println!("reviewer_participant: {reviewer}");
    println!("authenticated_principal_id: {principal_id}");
    println!("action: {}", action_label(&requested));
    println!("action_id: {}", action.action_id);
    println!(
        "effective_decision_id: {}",
        effective_decision_id.as_deref().unwrap_or("none_deferred")
    );
    println!("execution_grant: none_review_command_never_executes");
    println!("external_effect: none");
    Ok(())
}
