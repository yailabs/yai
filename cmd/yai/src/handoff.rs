//! Case-centric product adapter for the typed same-Tenant Handoff protocol.
//!
//! The engine owns validation and canonical facts. This module only translates
//! the W16 registry invocation into one bounded protocol operation and renders
//! its typed result.

use super::*;
use crate::command_adapters::security::authenticate_local;
use std::time::{SystemTime, UNIX_EPOCH};
use yai_core_engine::handoff::{HandoffData, HandoffDataKind, HandoffOutcome};

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string_pretty(value)
            .map_err(|error| format!("handoff_json_render_failed: {error}"))?
    );
    Ok(())
}

fn data(args: &[String]) -> Result<HandoffData, String> {
    let kind = match optional_arg(args, "--kind").as_deref().unwrap_or("text") {
        "text" => HandoffDataKind::Text,
        "json" => HandoffDataKind::Json,
        _ => return Err("--kind must be text or json".to_string()),
    };
    let value = named_arg(args, "--value")?;
    let value = HandoffData { kind, value };
    value.validate()?;
    Ok(value)
}

fn repeated(args: &[String], flag: &str) -> Result<Vec<String>, String> {
    let mut values = Vec::new();
    let mut index = 0usize;
    while index < args.len() {
        if args[index] == flag {
            values.push(
                args.get(index + 1)
                    .ok_or_else(|| format!("missing value for {flag}"))?
                    .clone(),
            );
            index += 2;
        } else {
            index += 1;
        }
    }
    Ok(values)
}

fn offer(args: &[String]) -> Result<(), String> {
    let source_case_id = named_arg(args, "--case")?;
    let target_case_id = named_arg(args, "--target")?;
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let commit = store.offer_case_handoff(
        &authenticated,
        &source_case_id,
        &target_case_id,
        data(args)?,
        repeated(args, "--role")?,
        now_unix_ms(),
    )?;
    let offer = commit
        .state
        .handoff_offers
        .last()
        .ok_or_else(|| "handoff_offer_missing_after_commit".to_string())?;
    if args.iter().any(|arg| arg == "--json") {
        return print_json(offer);
    }
    println!("handoff: offered");
    println!("handoff_id: {}", offer.handoff_id);
    println!("source_case: {}", offer.source_case_id);
    println!("target_case: {}", offer.target_case_id);
    println!("request_digest: {}", offer.request.digest());
    Ok(())
}

fn pending(args: &[String]) -> Result<(), String> {
    let target_case_id = named_arg(args, "--case")?;
    let authenticated = authenticate_local()?;
    let offers = LmdbRecordStore::open(record_store_path())?
        .list_pending_case_handoffs_authorized(&authenticated, &target_case_id)?;
    if args.iter().any(|arg| arg == "--json") {
        return print_json(&offers);
    }
    println!("pending_handoffs: {}", offers.len());
    for offer in offers {
        println!(
            "handoff: {} source={} target={} roles={}",
            offer.handoff_id,
            offer.source_case_id,
            offer.target_case_id,
            offer.required_target_roles.join(",")
        );
    }
    Ok(())
}

fn show(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let handoff_id = named_arg(args, "--handoff")?;
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store.get_case_state_authorized(&authenticated, &case_id)?;
    let local_offer = state
        .handoff_offers
        .iter()
        .find(|value| value.handoff_id == handoff_id)
        .cloned();
    let source_case_id = state
        .handoff_acceptances
        .iter()
        .find(|value| value.handoff_id == handoff_id)
        .map(|value| value.source_case_id.as_str())
        .or_else(|| {
            state
                .handoff_declines
                .iter()
                .find(|value| value.handoff_id == handoff_id)
                .map(|value| value.source_case_id.as_str())
        })
        .or_else(|| {
            state
                .handoff_results
                .iter()
                .find(|value| value.handoff_id == handoff_id)
                .map(|value| value.source_case_id.as_str())
        });
    let offer = if let Some(offer) = local_offer {
        offer
    } else if let Some(source_case_id) = source_case_id {
        store
            .get_case_state_authorized(&authenticated, source_case_id)?
            .handoff_offers
            .into_iter()
            .find(|value| value.handoff_id == handoff_id)
            .ok_or_else(|| "handoff_source_offer_missing".to_string())?
    } else {
        store
            .list_pending_case_handoffs_authorized(&authenticated, &case_id)?
            .into_iter()
            .find(|value| value.handoff_id == handoff_id)
            .ok_or_else(|| "handoff_not_visible".to_string())?
    };
    let value = serde_json::json!({
        "offer": offer,
        "acceptance": state.handoff_acceptances.iter().find(|value| value.handoff_id == handoff_id),
        "decline": state.handoff_declines.iter().find(|value| value.handoff_id == handoff_id),
        "result": state.handoff_results.iter().find(|value| value.handoff_id == handoff_id),
        "reconciliation": state.handoff_reconciliations.iter().find(|value| value.handoff_id == handoff_id),
    });
    print_json(&value)
}

fn accept(args: &[String]) -> Result<(), String> {
    let target_case_id = named_arg(args, "--case")?;
    let source_case_id = named_arg(args, "--source")?;
    let handoff_id = named_arg(args, "--handoff")?;
    let participant_id = named_arg(args, "--participant")?;
    let authenticated = authenticate_local()?;
    let commit = LmdbRecordStore::open(record_store_path())?.accept_case_handoff(
        &authenticated,
        &target_case_id,
        &source_case_id,
        &handoff_id,
        &participant_id,
        now_unix_ms(),
    )?;
    let acceptance = commit
        .state
        .handoff_acceptances
        .last()
        .ok_or_else(|| "handoff_acceptance_missing_after_commit".to_string())?;
    if args.iter().any(|arg| arg == "--json") {
        return print_json(acceptance);
    }
    println!("handoff: accepted");
    println!("handoff_id: {}", acceptance.handoff_id);
    println!("acceptance_id: {}", acceptance.acceptance_id);
    println!("target_case: {}", acceptance.target_case_id);
    Ok(())
}

fn decline(args: &[String]) -> Result<(), String> {
    let target_case_id = named_arg(args, "--case")?;
    let source_case_id = named_arg(args, "--source")?;
    let handoff_id = named_arg(args, "--handoff")?;
    let participant_id = named_arg(args, "--participant")?;
    let reason = named_arg(args, "--reason")?;
    let authenticated = authenticate_local()?;
    let commit = LmdbRecordStore::open(record_store_path())?.decline_case_handoff(
        &authenticated,
        &target_case_id,
        &source_case_id,
        &handoff_id,
        &participant_id,
        &reason,
        now_unix_ms(),
    )?;
    let decline = commit
        .state
        .handoff_declines
        .last()
        .ok_or_else(|| "handoff_decline_missing_after_commit".to_string())?;
    if args.iter().any(|arg| arg == "--json") {
        return print_json(decline);
    }
    println!("handoff: declined");
    println!("handoff_id: {}", decline.handoff_id);
    println!("decline_id: {}", decline.decline_id);
    Ok(())
}

fn result(args: &[String]) -> Result<(), String> {
    let target_case_id = named_arg(args, "--case")?;
    let handoff_id = named_arg(args, "--handoff")?;
    let participant_id = named_arg(args, "--participant")?;
    let outcome = match named_arg(args, "--outcome")?.as_str() {
        "succeeded" => HandoffOutcome::Succeeded,
        "failed" => HandoffOutcome::Failed,
        "cancelled" => HandoffOutcome::Cancelled,
        _ => return Err("--outcome must be succeeded, failed, or cancelled".to_string()),
    };
    let authenticated = authenticate_local()?;
    let commit = LmdbRecordStore::open(record_store_path())?.record_case_handoff_result(
        &authenticated,
        &target_case_id,
        &handoff_id,
        outcome,
        data(args)?,
        repeated(args, "--evidence")?,
        &participant_id,
        now_unix_ms(),
    )?;
    let result = commit
        .state
        .handoff_results
        .last()
        .ok_or_else(|| "handoff_result_missing_after_commit".to_string())?;
    if args.iter().any(|arg| arg == "--json") {
        return print_json(result);
    }
    println!("handoff: result_recorded");
    println!("handoff_id: {}", result.handoff_id);
    println!("result_id: {}", result.result_id);
    println!("outcome: {:?}", result.outcome);
    Ok(())
}

fn reconcile(args: &[String]) -> Result<(), String> {
    let source_case_id = named_arg(args, "--case")?;
    let handoff_id = named_arg(args, "--handoff")?;
    let authenticated = authenticate_local()?;
    let commit = LmdbRecordStore::open(record_store_path())?.reconcile_case_handoff(
        &authenticated,
        &source_case_id,
        &handoff_id,
        now_unix_ms(),
    )?;
    let reconciliation = commit
        .state
        .handoff_reconciliations
        .iter()
        .find(|value| value.handoff_id == handoff_id)
        .ok_or_else(|| "handoff_reconciliation_missing_after_commit".to_string())?;
    if args.iter().any(|arg| arg == "--json") {
        return print_json(reconciliation);
    }
    println!("handoff: reconciled");
    println!("handoff_id: {}", reconciliation.handoff_id);
    println!("reconciliation_id: {}", reconciliation.reconciliation_id);
    println!("outcome: {:?}", reconciliation.outcome);
    Ok(())
}

pub(super) fn handoff_command(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("offer") => offer(&args[1..]),
        Some("pending") => pending(&args[1..]),
        Some("show") => show(&args[1..]),
        Some("accept") => accept(&args[1..]),
        Some("decline") => decline(&args[1..]),
        Some("result") => result(&args[1..]),
        Some("reconcile") => reconcile(&args[1..]),
        _ => Err(
            "usage: yai case handoff <offer|pending|show|accept|decline|result|reconcile>"
                .to_string(),
        ),
    }
}
