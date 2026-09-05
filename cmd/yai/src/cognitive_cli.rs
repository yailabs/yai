//! Advanced operator surfaces for I02 semantic cognitive planning.
//!
//! These commands mutate only provider-owned suitability evidence or canonical
//! Case cognitive bindings. Planning is read-only and cannot dispatch a model.

use super::*;
use serde_json::json;
use yai_core_engine::cognitive::{
    assess_lane_continuation, CognitiveBindingRole, CognitiveCapability,
    CognitiveCapabilityRequirement, LaneContinuationReference, SemanticEvidencePosture,
};
use yai_core_engine::security::AuthenticatedPrincipal;

fn authenticated_store() -> Result<(AuthenticatedPrincipal, LmdbRecordStore), String> {
    let authenticated = security::authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    Ok((authenticated, store))
}

fn repeated_arg(args: &[String], name: &str) -> Vec<String> {
    args.iter()
        .enumerate()
        .filter_map(|(index, value)| {
            (value == name)
                .then(|| args.get(index + 1).cloned())
                .flatten()
        })
        .collect()
}

fn json_requested(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--json")
}

fn evidence_record(args: &[String]) -> Result<(), String> {
    let target_id = named_arg(args, "--target")?;
    let capability = CognitiveCapability::parse(&named_arg(args, "--capability")?)?;
    let provenance_refs = repeated_arg(args, "--evidence-ref");
    let (authenticated, store) = authenticated_store()?;
    let evidence = store.record_semantic_suitability_evidence_authorized(
        &authenticated,
        &target_id,
        capability,
        SemanticEvidencePosture::OperatorAttested,
        &named_arg(args, "--suite")?,
        &named_arg(args, "--run")?,
        provenance_refs,
        "authenticated_operator_attestation",
    )?;
    if json_requested(args) {
        println!(
            "{}",
            serde_json::to_string(&json!({
                "schema": "yai.cli.semantic_suitability_evidence_result.v1",
                "evidence": evidence,
                "mechanically_qualified": false,
                "provider_execution": "not_performed"
            }))
            .map_err(|error| format!("semantic_suitability_cli_encode_failed: {error}"))?
        );
    } else {
        println!("semantic_suitability_evidence: recorded");
        println!("evidence_id: {}", evidence.evidence_id);
        println!("target_id: {}", evidence.target_id);
        println!("capability: {}", evidence.capability.as_str());
        println!("posture: operator_attested");
        println!("mechanically_qualified: false");
        println!("provider_execution: not_performed");
    }
    Ok(())
}

fn evidence_show(args: &[String]) -> Result<(), String> {
    let target_id = named_arg(args, "--target")?;
    let capability = optional_arg(args, "--capability")
        .map(|value| CognitiveCapability::parse(&value))
        .transpose()?;
    let (authenticated, store) = authenticated_store()?;
    let evidence = store.list_semantic_suitability_evidence_authorized(
        &authenticated,
        &target_id,
        capability,
    )?;
    if json_requested(args) {
        println!(
            "{}",
            serde_json::to_string(&json!({
                "schema": "yai.cli.semantic_suitability_evidence_collection.v1",
                "target_id": target_id,
                "count": evidence.len(),
                "evidence": evidence,
                "provider_execution": "not_performed"
            }))
            .map_err(|error| format!("semantic_suitability_cli_encode_failed: {error}"))?
        );
    } else {
        println!("target_id: {target_id}");
        println!("semantic_suitability_evidence_count: {}", evidence.len());
        for item in evidence {
            println!(
                "evidence: {} capability:{} posture:{:?} target_digest:{}",
                item.evidence_id,
                item.capability.as_str(),
                item.posture,
                item.target_digest
            );
        }
        println!("provider_execution: not_performed");
    }
    Ok(())
}

fn cognitive_bind(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--participant")?;
    let role = CognitiveBindingRole::parse(&named_arg(args, "--role")?)?;
    let capability = CognitiveCapability::parse(&named_arg(args, "--capability")?)?;
    let (authenticated, store) = authenticated_store()?;
    let binding = store.bind_case_cognitive_target_authorized(
        &authenticated,
        &case_id,
        &participant_id,
        role,
        capability,
        &named_arg(args, "--target")?,
        &named_arg(args, "--evidence")?,
        args.iter().any(|arg| arg == "--replace"),
    )?;
    if json_requested(args) {
        println!(
            "{}",
            serde_json::to_string(&json!({
                "schema": "yai.cli.case_cognitive_binding_result.v1",
                "binding": binding,
                "provider_execution": "not_performed"
            }))
            .map_err(|error| format!("cognitive_binding_cli_encode_failed: {error}"))?
        );
    } else {
        println!("case_cognitive_binding: recorded");
        println!("binding_id: {}", binding.binding_id);
        println!("case_id: {}", binding.case_id);
        println!("participant_id: {}", binding.participant_id);
        println!("role: {}", binding.role.as_str());
        println!("capability: {}", binding.capability.as_str());
        println!("target_id: {}", binding.target_id);
        println!("semantic_evidence_id: {}", binding.semantic_evidence_id);
        println!("provider_execution: not_performed");
    }
    Ok(())
}

fn cognitive_unbind(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--participant")?;
    let role = CognitiveBindingRole::parse(&named_arg(args, "--role")?)?;
    let capability = CognitiveCapability::parse(&named_arg(args, "--capability")?)?;
    let (authenticated, store) = authenticated_store()?;
    let commit = store.unbind_case_cognitive_target_authorized(
        &authenticated,
        &case_id,
        &participant_id,
        role,
        capability,
        &named_arg(args, "--reason")?,
    )?;
    if json_requested(args) {
        println!(
            "{}",
            serde_json::to_string(&json!({
                "schema": "yai.cli.case_cognitive_unbind_result.v1",
                "transition_id": commit.transition.transition_id,
                "case_id": commit.state.case_id,
                "generation": commit.state.generation,
                "provider_execution": "not_performed"
            }))
            .map_err(|error| format!("cognitive_unbind_cli_encode_failed: {error}"))?
        );
    } else {
        println!("case_cognitive_binding: unbound");
        println!("transition_id: {}", commit.transition.transition_id);
        println!("case_id: {}", commit.state.case_id);
        println!("generation: {}", commit.state.generation);
        println!("provider_execution: not_performed");
    }
    Ok(())
}

fn cognitive_show(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--participant")?;
    let (authenticated, store) = authenticated_store()?;
    let state = store.get_case_state_authorized(&authenticated, &case_id)?;
    let mut bindings = state
        .cognitive_bindings
        .into_iter()
        .filter(|binding| binding.participant_id == participant_id)
        .collect::<Vec<_>>();
    bindings.sort_by(|left, right| left.binding_id.cmp(&right.binding_id));
    if json_requested(args) {
        println!(
            "{}",
            serde_json::to_string(&json!({
                "schema": "yai.cli.case_cognitive_binding_collection.v1",
                "case_id": case_id,
                "participant_id": participant_id,
                "case_generation": state.generation,
                "count": bindings.len(),
                "bindings": bindings,
                "provider_execution": "not_performed"
            }))
            .map_err(|error| format!("cognitive_show_cli_encode_failed: {error}"))?
        );
    } else {
        println!("case_id: {case_id}");
        println!("participant_id: {participant_id}");
        println!("case_generation: {}", state.generation);
        println!("cognitive_bindings: {}", bindings.len());
        for binding in bindings {
            println!(
                "binding: {} role:{} capability:{} target:{} evidence:{}",
                binding.binding_id,
                binding.role.as_str(),
                binding.capability.as_str(),
                binding.target_id,
                binding.semantic_evidence_id
            );
        }
        println!("provider_execution: not_performed");
    }
    Ok(())
}

fn continuation_from_args(args: &[String]) -> Result<Option<LaneContinuationReference>, String> {
    let lane = optional_arg(args, "--continuation-lane");
    let target = optional_arg(args, "--continuation-target");
    let runtime = optional_arg(args, "--continuation-runtime");
    let opaque = optional_arg(args, "--continuation-ref");
    if lane.is_none() && target.is_none() && runtime.is_none() && opaque.is_none() {
        return Ok(None);
    }
    match (lane, target, runtime, opaque) {
        (Some(execution_lane_id), Some(target_id), Some(runtime_id), Some(opaque_reference)) => {
            Ok(Some(LaneContinuationReference {
                execution_lane_id,
                target_id,
                runtime_id,
                opaque_reference,
            }))
        }
        _ => Err("lane_continuation_requires_all_fields".to_string()),
    }
}

fn cognitive_plan(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--participant")?;
    let capability = CognitiveCapability::parse(&named_arg(args, "--capability")?)?;
    let source_ref = named_arg(args, "--source")?;
    let requirement =
        CognitiveCapabilityRequirement::new(&case_id, &participant_id, capability, &source_ref)?;
    let continuation = continuation_from_args(args)?;
    let (authenticated, store) = authenticated_store()?;
    let plan = store.plan_case_cognitive_execution_authorized(
        &authenticated,
        &case_id,
        &participant_id,
        &requirement,
    )?;
    let continuation_posture = assess_lane_continuation(&plan, continuation.as_ref());
    if json_requested(args) {
        println!(
            "{}",
            serde_json::to_string(&json!({
                "schema": "yai.cli.cognitive_execution_plan_result.v1",
                "requirement": requirement,
                "plan": plan,
                "continuation_posture": continuation_posture,
                "continuation_opaque_value_rendered": false
            }))
            .map_err(|error| format!("cognitive_plan_cli_encode_failed: {error}"))?
        );
    } else {
        println!("cognitive_execution_plan: {}", plan.plan_id);
        println!("requirement_id: {}", requirement.requirement_id);
        println!("capability: {}", plan.capability.as_str());
        println!("route: {:?}", plan.route);
        println!("role: {:?}", plan.role);
        println!(
            "selected_target_id: {}",
            plan.selected_target_id.as_deref().unwrap_or("none")
        );
        println!(
            "execution_lane_id: {}",
            plan.execution_lane_id.as_deref().unwrap_or("none")
        );
        println!("continuation_posture: {:?}", continuation_posture);
        println!("provider_realization: deferred_to_execution_adapter");
        println!("provider_execution: not_performed");
    }
    Ok(())
}

pub(super) fn cognitive_command(operation_id: &str, args: &[String]) -> Result<(), String> {
    match operation_id {
        "yai.provider.suitability.record" => evidence_record(args),
        "yai.provider.suitability.show" => evidence_show(args),
        "yai.case.cognitive.bind" => cognitive_bind(args),
        "yai.case.cognitive.unbind" => cognitive_unbind(args),
        "yai.case.cognitive.show" => cognitive_show(args),
        "yai.case.cognitive.plan" => cognitive_plan(args),
        _ => Err(format!("unsupported cognitive operation: {operation_id}")),
    }
}
