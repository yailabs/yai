//! Case-native policy binding and derived normative-state CLI.

use super::*;
use yai_core_engine::case_policy::{
    EffectivePolicyRule, NormativeReadiness, NormativeStatus, PolicyCatalogDrift,
};
use yai_core_engine::store::lmdb::CasePolicyMutationOutcome;

fn required_generation(args: &[String]) -> Result<u64, String> {
    named_arg(args, "--expected-generation")?
        .parse::<u64>()
        .map_err(|error| format!("invalid --expected-generation: {error}"))
}

fn readiness_label(value: &NormativeReadiness) -> &'static str {
    match value {
        NormativeReadiness::Unconfigured => "unconfigured",
        NormativeReadiness::Ready => "ready",
        NormativeReadiness::Blocked => "blocked",
    }
}

fn drift_label(value: &PolicyCatalogDrift) -> String {
    match value {
        PolicyCatalogDrift::Current => "current".to_string(),
        PolicyCatalogDrift::Superseded {
            current_artifact_id,
        } => {
            format!("superseded:current={current_artifact_id}")
        }
        PolicyCatalogDrift::Retired => "retired".to_string(),
        PolicyCatalogDrift::NoCurrentPublishedArtifact => {
            "no_current_published_artifact".to_string()
        }
    }
}

fn print_normative_status(
    store: &LmdbRecordStore,
    case_id: &str,
    status: &NormativeStatus,
) -> Result<(), String> {
    let state = store
        .get_case_state(case_id)?
        .ok_or_else(|| format!("case_state_not_found: {case_id}"))?;
    let transitions = store.list_case_transitions(case_id)?;
    let decisions = transitions
        .iter()
        .filter(|item| matches!(item.payload, TransitionPayload::DecisionRecorded { .. }))
        .count();
    let grants = transitions
        .iter()
        .filter(|item| matches!(item.payload, TransitionPayload::ExecutionGrantIssued { .. }))
        .count();
    let effects = transitions
        .iter()
        .filter(|item| matches!(item.payload, TransitionPayload::EffectPrepared { .. }))
        .count();
    println!("case_id: {case_id}");
    println!("case_generation: {}", state.generation);
    println!("transition_count: {}", transitions.len());
    println!(
        "normative_readiness: {}",
        readiness_label(&status.readiness)
    );
    println!("active_policy_bindings: {}", state.policy_bindings.len());
    for binding in &state.policy_bindings {
        println!(
            "policy_binding: binding_id={} lineage_id={} owner_ref={} policy_key={} artifact_id={} version={} publication_event={} bound_generation={}",
            binding.binding_id,
            binding.lineage_id,
            binding.owner_ref,
            binding.policy_key,
            binding.artifact_id,
            binding.artifact_version,
            binding.publication_event_id,
            binding.bound_at_case_generation
        );
    }
    if let Some(effective) = &status.effective_policy {
        println!("effective_policy_id: {}", effective.effective_policy_id);
        println!("effective_policy_digest: {}", effective.semantic_digest);
        println!("materializer_version: {}", effective.materializer_version);
        println!("effective_input_rules: {}", effective.input_rule_count);
        println!("effective_output_rules: {}", effective.rules.len());
        println!("effective_merged_rules: {}", effective.merged_rule_count);
        println!(
            "effective_resolved_conflicts: {}",
            effective.resolved_conflict_count
        );
        let provenance_count = effective
            .rules
            .iter()
            .map(|rule| match rule {
                EffectivePolicyRule::OperationRestriction { contributions, .. }
                | EffectivePolicyRule::ReviewRequirement { contributions, .. }
                | EffectivePolicyRule::EvidenceObligation { contributions, .. } => {
                    contributions.len()
                }
                EffectivePolicyRule::AuthorityRequirement { contributions, .. } => {
                    contributions.len()
                }
            })
            .sum::<usize>();
        println!("effective_provenance_contributions: {provenance_count}");
        for rule in &effective.rules {
            println!(
                "effective_rule: {}",
                serde_json::to_string(rule)
                    .map_err(|error| format!("effective_rule_render_failed: {error}"))?
            );
        }
    } else {
        println!("effective_policy_id: none");
    }
    println!("blocking_conflicts: {}", status.blocking_conflicts.len());
    for conflict in &status.blocking_conflicts {
        println!("blocking_conflict: {conflict}");
    }
    println!("missing_inputs: {}", status.missing.len());
    for missing in &status.missing {
        println!("missing_input: {missing}");
    }
    for (lineage, drift) in &status.catalog_drift {
        println!(
            "catalog_drift: lineage_id={lineage} status={}",
            drift_label(drift)
        );
    }
    println!("decision_count: {decisions}");
    println!("execution_grant_count: {grants}");
    println!("prepared_effect_count: {effects}");
    println!("authority_emitted_by_case_policy: false");
    Ok(())
}

fn print_mutation(
    store: &LmdbRecordStore,
    case_id: &str,
    action: &str,
    outcome: CasePolicyMutationOutcome,
) -> Result<(), String> {
    println!(
        "case_policy_{action}: {}",
        if outcome.changed {
            "committed"
        } else {
            "unchanged_idempotent"
        }
    );
    if let Some(commit) = &outcome.commit {
        println!("transition_id: {}", commit.transition.transition_id);
        println!("case_generation: {}", commit.state.generation);
    }
    println!(
        "derived_cache: {}",
        outcome.derived_cache_error.as_deref().unwrap_or("stored")
    );
    print_normative_status(store, case_id, &outcome.status)
}

fn bind(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let artifact_id = named_arg(args, "--artifact")?;
    let actor = named_arg(args, "--as")?;
    let reason =
        optional_arg(args, "--reason").unwrap_or_else(|| "bind exact published policy".to_string());
    let store = LmdbRecordStore::open(record_store_path())?;
    let outcome = store.bind_case_policy(
        &case_id,
        &artifact_id,
        required_generation(args)?,
        &actor,
        &reason,
    )?;
    print_mutation(&store, &case_id, "bind", outcome)
}

fn replace(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let binding_id = named_arg(args, "--binding")?;
    let artifact_id = named_arg(args, "--artifact")?;
    let actor = named_arg(args, "--as")?;
    let reason = optional_arg(args, "--reason")
        .unwrap_or_else(|| "replace exact policy binding".to_string());
    let store = LmdbRecordStore::open(record_store_path())?;
    let outcome = store.replace_case_policy(
        &case_id,
        &binding_id,
        &artifact_id,
        required_generation(args)?,
        &actor,
        &reason,
    )?;
    print_mutation(&store, &case_id, "replace", outcome)
}

fn unbind(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let binding_id = named_arg(args, "--binding")?;
    let actor = named_arg(args, "--as")?;
    let reason = named_arg(args, "--reason")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let outcome = store.unbind_case_policy(
        &case_id,
        &binding_id,
        required_generation(args)?,
        &actor,
        &reason,
    )?;
    print_mutation(&store, &case_id, "unbind", outcome)
}

fn status(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let status = store.case_policy_status(&case_id)?;
    print_normative_status(&store, &case_id, &status)
}

fn rebuild(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let before = store.list_case_transitions(&case_id)?.len();
    let status = store.rebuild_effective_policy(&case_id)?;
    let after = store.list_case_transitions(&case_id)?.len();
    println!("effective_policy_rebuild: completed");
    println!("canonical_transitions_before: {before}");
    println!("canonical_transitions_after: {after}");
    print_normative_status(&store, &case_id, &status)
}

pub(super) fn case_policy_command(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("bind") => bind(&args[1..]),
        Some("replace") => replace(&args[1..]),
        Some("unbind") => unbind(&args[1..]),
        Some("status") => status(&args[1..]),
        Some("rebuild") => rebuild(&args[1..]),
        Some(other) => Err(format!("unknown case policy command: {other}")),
        None => Err("case policy command is required".to_string()),
    }
}
