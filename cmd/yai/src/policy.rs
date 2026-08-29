//! Case-independent governance source intake and PolicyArtifact CLI.
//!
//! This surface compiles constrained source bytes and persists immutable
//! artifacts/lifecycle events. It never binds a Case or emits authority.

use super::*;
use yai_core_engine::governance::{
    compile_policy_source, NormalizedPolicyRule, ParsedPolicyFact, PolicyArtifactView,
    PolicyLifecycleState, PolicySourceArtifact, PolicyValidationStatus,
};

fn state_label(state: &PolicyLifecycleState) -> &'static str {
    match state {
        PolicyLifecycleState::Candidate => "candidate",
        PolicyLifecycleState::Validated => "validated",
        PolicyLifecycleState::Published => "published",
        PolicyLifecycleState::Superseded => "superseded",
        PolicyLifecycleState::Retired => "retired",
    }
}

fn validation_label(status: &PolicyValidationStatus) -> &'static str {
    match status {
        PolicyValidationStatus::Qualified => "qualified",
        PolicyValidationStatus::Blocked => "blocked",
    }
}

fn fact_kind(fact: &ParsedPolicyFact) -> &'static str {
    match fact {
        ParsedPolicyFact::OperationRestriction { .. } => "operation_restriction",
        ParsedPolicyFact::ReviewRequirement { .. } => "review_requirement",
        ParsedPolicyFact::EvidenceObligation { .. } => "evidence_obligation",
    }
}

fn normalized_kind(rule: &NormalizedPolicyRule) -> &'static str {
    match rule {
        NormalizedPolicyRule::OperationRestriction { .. } => "operation_restriction",
        NormalizedPolicyRule::ReviewRequirement { .. } => "review_requirement",
        NormalizedPolicyRule::EvidenceObligation { .. } => "evidence_obligation",
    }
}

fn print_source(source: &PolicySourceArtifact) {
    println!("policy_source_schema: {}", source.schema);
    println!("source_id: {}", source.source_id);
    println!("source_digest: {}", source.content_digest);
    println!("source_format: {}", source.source_format);
    println!("policy_key: {}", source.policy_key);
    println!("source_version: {}", source.source_version);
    println!("owner_ref: {}", source.owner_ref);
    println!("source_bytes_retained: {}", source.content_utf8.len());
    println!("source_payload_display: withheld_by_default");
}

fn print_artifact(view: &PolicyArtifactView) {
    let artifact = &view.artifact;
    println!("policy_artifact_schema: {}", artifact.schema);
    println!("artifact_id: {}", artifact.artifact_id);
    println!("policy_key: {}", artifact.policy_key);
    println!("artifact_version: {}", artifact.artifact_version);
    println!("owner_ref: {}", artifact.owner_ref);
    println!("source_id: {}", artifact.source_id);
    println!("source_digest: {}", artifact.source_digest);
    println!("parsed_schema: {}", artifact.parsed.schema);
    println!("parsed_digest: {}", artifact.parsed.parsed_digest);
    println!("parsed_facts: {}", artifact.parsed.facts.len());
    for fact in &artifact.parsed.facts {
        println!(
            "parsed_fact: kind={} fact_id={} rule_id={} source_location={}",
            fact_kind(fact),
            fact.fact_id(),
            fact.rule_id(),
            fact.source_location()
        );
    }
    println!("policy_ir_schema: {}", artifact.policy_ir.schema);
    println!("policy_ir_digest: {}", artifact.policy_ir.ir_digest);
    println!("normalized_rules: {}", artifact.policy_ir.rules.len());
    for rule in &artifact.policy_ir.rules {
        println!(
            "normalized_rule: kind={} rule_id={} source_facts={} source_locations={}",
            normalized_kind(rule),
            rule.rule_id(),
            rule.provenance().fact_refs.join(","),
            rule.provenance().source_locations.join(",")
        );
    }
    println!("unresolved_items: {}", artifact.policy_ir.unresolved.len());
    for unresolved in &artifact.policy_ir.unresolved {
        println!(
            "unresolved: code={} kind={} location={} detail={}",
            unresolved.code, unresolved.source_kind, unresolved.source_location, unresolved.detail
        );
    }
    println!("conflicts: {}", artifact.policy_ir.conflicts.len());
    for conflict in &artifact.policy_ir.conflicts {
        println!(
            "conflict: code={} selector={} source_facts={}",
            conflict.code,
            conflict.selector,
            conflict.source_fact_refs.join(",")
        );
    }
    println!(
        "validation_status: {}",
        validation_label(&artifact.validation.status)
    );
    println!(
        "validation_blockers: {}",
        artifact.validation.blockers.len()
    );
    for blocker in &artifact.validation.blockers {
        println!("validation_blocker: {blocker}");
    }
    println!("lifecycle: {}", state_label(&view.lifecycle));
    println!("runtime_consumable: {}", view.runtime_consumable);
    println!(
        "superseded_by: {}",
        view.superseded_by.as_deref().unwrap_or("none")
    );
    println!("lifecycle_events: {}", view.lifecycle_events.len());
    for event in &view.lifecycle_events {
        println!(
            "lifecycle_event: sequence={} event_id={} action={:?} actor={} related_artifact={}",
            event.sequence,
            event.event_id,
            event.action,
            event.actor_ref,
            event.related_artifact_id.as_deref().unwrap_or("none")
        );
    }
    println!("case_binding: absent_wave_8");
    println!("effective_policy: absent_wave_8");
    println!("decision_or_grant: never_emitted_by_policy_authoring");
}

fn positional(args: &[String], label: &str) -> Result<String, String> {
    args.first()
        .filter(|value| !value.starts_with("--"))
        .cloned()
        .ok_or_else(|| format!("{label} is required"))
}

fn policy_ingest(args: &[String]) -> Result<(), String> {
    let source_path = PathBuf::from(positional(args, "policy source path")?);
    let actor_ref = named_arg(args, "--as")?;
    let bytes = fs::read(&source_path)
        .map_err(|error| format!("failed to read {}: {error}", source_path.display()))?;
    let compilation = compile_policy_source(&bytes)?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let outcome = store.ingest_policy_compilation(&compilation, &actor_ref)?;
    println!(
        "policy_ingest: {}",
        if outcome.artifact_created {
            "candidate_created"
        } else {
            "existing_idempotent"
        }
    );
    println!("source_created: {}", outcome.source_created);
    println!("artifact_created: {}", outcome.artifact_created);
    println!("input_path: {}", source_path.display());
    print_source(&compilation.source);
    print_artifact(&outcome.view);
    Ok(())
}

fn policy_inspect(args: &[String]) -> Result<(), String> {
    let identity = positional(args, "policy source/artifact id")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    if identity.starts_with("policy-source:") {
        let source = store
            .get_policy_source(&identity)?
            .ok_or_else(|| format!("policy_source_not_found: {identity}"))?;
        print_source(&source);
    } else {
        let view = store
            .policy_artifact_view(&identity)?
            .ok_or_else(|| format!("policy_artifact_not_found: {identity}"))?;
        print_artifact(&view);
    }
    Ok(())
}

fn policy_validate(args: &[String]) -> Result<(), String> {
    let artifact_id = positional(args, "policy artifact id")?;
    let actor_ref = named_arg(args, "--as")?;
    let reason = optional_arg(args, "--reason")
        .unwrap_or_else(|| "deterministic policy qualification passed".to_string());
    let store = LmdbRecordStore::open(record_store_path())?;
    let outcome = store.validate_policy_artifact(&artifact_id, &actor_ref, &reason)?;
    println!(
        "policy_validate: {}",
        if outcome.changed {
            "validated"
        } else {
            "already_validated"
        }
    );
    print_artifact(&outcome.view);
    Ok(())
}

fn policy_publish(args: &[String]) -> Result<(), String> {
    let artifact_id = positional(args, "policy artifact id")?;
    let actor_ref = named_arg(args, "--as")?;
    let reason = optional_arg(args, "--reason")
        .unwrap_or_else(|| "trusted local operator published qualified artifact".to_string());
    let store = LmdbRecordStore::open(record_store_path())?;
    let outcome = store.publish_policy_artifact(&artifact_id, &actor_ref, &reason)?;
    println!(
        "policy_publish: {}",
        if outcome.changed {
            "published"
        } else {
            "already_published"
        }
    );
    print_artifact(&outcome.view);
    Ok(())
}

fn policy_retire(args: &[String]) -> Result<(), String> {
    let artifact_id = positional(args, "policy artifact id")?;
    let actor_ref = named_arg(args, "--as")?;
    let reason = named_arg(args, "--reason")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let outcome = store.retire_policy_artifact(&artifact_id, &actor_ref, &reason)?;
    println!(
        "policy_retire: {}",
        if outcome.changed {
            "retired"
        } else {
            "already_retired"
        }
    );
    print_artifact(&outcome.view);
    Ok(())
}

fn policy_list() -> Result<(), String> {
    let store = LmdbRecordStore::open(record_store_path())?;
    let views = store.list_policy_artifact_views()?;
    println!("policy_artifacts: {}", views.len());
    for view in views {
        println!(
            "policy_artifact: id={} policy_key={} version={} lifecycle={} runtime_consumable={}",
            view.artifact.artifact_id,
            view.artifact.policy_key,
            view.artifact.artifact_version,
            state_label(&view.lifecycle),
            view.runtime_consumable
        );
    }
    Ok(())
}

pub(super) fn policy_command(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("ingest") => policy_ingest(&args[1..]),
        Some("inspect") => policy_inspect(&args[1..]),
        Some("validate") => policy_validate(&args[1..]),
        Some("publish") => policy_publish(&args[1..]),
        Some("retire") => policy_retire(&args[1..]),
        Some("list") => policy_list(),
        Some(other) => Err(format!("unknown policy command: {other}")),
        None => Err("policy command is required".to_string()),
    }
}
