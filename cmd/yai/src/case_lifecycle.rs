//! CLI-only surface for durable Case cancellation and terminal closure.

use super::*;
use crate::security::{authenticate_local, reject_spoofed_as};
use yai_core_engine::transition::{EffectLifecycle, GrantLifecycle, ReviewResolution};

fn print_terminal_posture(store: &LmdbRecordStore, case_id: &str) -> Result<(), String> {
    let state = store
        .get_case_state(case_id)?
        .ok_or_else(|| format!("case_state_not_found: {case_id}"))?;
    let pending_reviews = state
        .reviews
        .iter()
        .filter(|review| {
            matches!(
                review.status,
                ReviewResolution::Pending | ReviewResolution::Deferred
            )
        })
        .count();
    let issued_grants = state
        .grants
        .iter()
        .filter(|grant| grant.status == GrantLifecycle::Issued)
        .count();
    let unresolved_effects = state
        .effects
        .iter()
        .filter(|effect| {
            matches!(
                effect.status,
                EffectLifecycle::Prepared | EffectLifecycle::Indeterminate
            )
        })
        .count();
    println!("case_id: {case_id}");
    println!("case_generation: {}", state.generation);
    println!("case_lifecycle: {:?}", state.lifecycle);
    println!("case_cancelled: {}", state.cancellation.is_some());
    if let Some(cancellation) = &state.cancellation {
        println!("cancellation_transition: {}", cancellation.transition_id);
        println!(
            "cancellation_requested_at: {}",
            cancellation.requested_at_unix_ms
        );
    }
    if let Some(closure) = &state.closure {
        println!("closure_transition: {}", closure.transition_id);
        println!("closed_at: {}", closure.closed_at_unix_ms);
    }
    println!("usable_pending_reviews: {pending_reviews}");
    println!("usable_issued_grants: {issued_grants}");
    println!("unresolved_effects: {unresolved_effects}");
    Ok(())
}

pub(super) fn case_cancel(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let authenticated = authenticate_local()?;
    reject_spoofed_as(args, &authenticated.projected_principal_id())?;
    let reason = named_arg(args, "--reason")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    store.get_case_state_authorized(&authenticated, &case_id)?;
    let outcome = store.cancel_tenant_case(&authenticated, &case_id, &reason)?;
    println!(
        "case_cancel: {}",
        if outcome.changed {
            "cancelled"
        } else {
            "already_cancelled_idempotent"
        }
    );
    println!("invalidated_reviews: {}", outcome.invalidated_reviews);
    println!("abandoned_grants: {}", outcome.abandoned_grants);
    for commit in &outcome.commits {
        println!(
            "cancellation_commit: transition_id={} generation={} kind={}",
            commit.transition.transition_id,
            commit.transition.sequence,
            commit.transition.payload.kind()
        );
    }
    print_terminal_posture(&store, &case_id)
}

pub(super) fn case_close(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let authenticated = authenticate_local()?;
    reject_spoofed_as(args, &authenticated.projected_principal_id())?;
    let reason = named_arg(args, "--reason")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    store.get_case_state_authorized(&authenticated, &case_id)?;
    let outcome = store.close_tenant_case(&authenticated, &case_id, &reason)?;
    println!(
        "case_close: {}",
        if outcome.changed {
            "closed"
        } else {
            "already_closed_idempotent"
        }
    );
    if let Some(commit) = &outcome.commit {
        println!("closure_transition: {}", commit.transition.transition_id);
        println!("closure_generation: {}", commit.transition.sequence);
    }
    print_terminal_posture(&store, &case_id)
}
