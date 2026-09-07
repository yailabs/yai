//! Advanced operator surfaces for cognitive planning, exact realization, and
//! bounded composition. Planning remains read-only; realization and
//! composition reuse the governed provider/Case owners without acquiring
//! independent runtime authority.

use super::cognitive_execution::*;
use super::*;
use serde_json::json;
use yai_core_engine::cognitive::{
    assess_lane_continuation, CognitiveBindingRole, CognitiveCapability,
    CognitiveCapabilityRequirement, LaneContinuationReference, SemanticEvidencePosture,
};
use yai_core_engine::conversation::{
    derived_content_from_history, find_turn, CognitiveCompositionPrerequisite,
    CognitiveCompositionRequest, CognitiveSourceClosure, ConversationContentStore,
    ConversationDerivedContent,
};
use yai_core_engine::provider_governance::ProviderRealizationShape;
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
    let mut candidates = vec![(named_arg(args, "--target")?, named_arg(args, "--evidence")?)];
    for alternative in repeated_arg(args, "--alternative") {
        let (target, evidence) = alternative
            .split_once('=')
            .ok_or("cognitive_alternative_requires_target_equals_evidence")?;
        candidates.push((target.to_string(), evidence.to_string()));
    }
    let binding = store.bind_case_cognitive_candidates_authorized(
        &authenticated,
        &case_id,
        &participant_id,
        role,
        capability,
        candidates,
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
        println!(
            "target_policy: {}",
            if binding.target_policy.is_some() {
                "ordered_eligible"
            } else {
                "pinned"
            }
        );
        for (order, candidate) in binding.candidates().iter().enumerate() {
            println!(
                "candidate: {} preference:{} evidence:{}",
                candidate.target_id, order, candidate.semantic_evidence_id
            );
        }
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
            println!(
                "target_policy: {}",
                if binding.target_policy.is_some() {
                    "ordered_eligible"
                } else {
                    "pinned"
                }
            );
            for (order, candidate) in binding.candidates().iter().enumerate() {
                println!(
                    "candidate: {} preference:{} evidence:{}",
                    candidate.target_id, order, candidate.semantic_evidence_id
                );
            }
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
    let shape = optional_arg(args, "--shape")
        .map(|value| ProviderRealizationShape::parse(&value))
        .transpose()?;
    let plan = store.plan_case_cognitive_execution_for_shape_authorized(
        &authenticated,
        &case_id,
        &participant_id,
        &requirement,
        shape.as_ref(),
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
        if let Some(arbitration) = &plan.arbitration {
            println!(
                "arbitration_snapshot: {}",
                arbitration.evidence_snapshot_digest
            );
            for candidate in &arbitration.candidates {
                println!(
                    "candidate: {} role:{:?} preference:{} exclusions:{:?}",
                    candidate.target_id, candidate.role, candidate.preference, candidate.exclusions
                );
            }
        }
        println!("provider_realization: deferred_to_execution_adapter");
        println!("provider_execution: not_performed");
    }
    Ok(())
}

fn cognitive_realize(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--participant")?;
    let turn_id = named_arg(args, "--turn")?;
    let capability = CognitiveCapability::parse(&named_arg(args, "--capability")?)?;
    let requested_parts = repeated_arg(args, "--part");
    let continuation = continuation_from_args(args)?;
    let (authenticated, store) = authenticated_store()?;
    let transitions = store.list_case_transitions(&case_id)?;
    let turn = find_turn(&case_id, &turn_id, &transitions)
        .ok_or_else(|| "cognitive_realization_turn_not_found".to_string())?;
    let content_store = ConversationContentStore::open(&yai_home())?;
    let wire_parts = source_parts(&content_store, turn, &requested_parts)?;
    let part_ids = wire_parts
        .iter()
        .map(|part| part.source_part_id.clone())
        .collect::<Vec<_>>();
    let source_ref = source_identity(&turn_id, &part_ids)?;
    let causal_refs = vec![turn_id.clone(), source_ref.clone()];
    let outcome = realize_cognitive(
        &authenticated,
        &store,
        &content_store,
        turn,
        &participant_id,
        capability,
        wire_parts,
        part_ids,
        &source_ref,
        None,
        continuation.as_ref(),
        optional_arg(args, "--failpoint").as_deref(),
        &causal_refs,
        &|| false,
        yai_core_engine::context::InvocationOutputContract::NaturalLanguage,
    )?;
    render_realization(
        args,
        &outcome.plan,
        outcome.derived.as_ref(),
        outcome.execution.as_ref(),
        outcome.posture,
        outcome.recovered,
    )
}

fn cognitive_compose(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--participant")?;
    let turn_id = named_arg(args, "--turn")?;
    let goal = CognitiveCapability::parse(&named_arg(args, "--goal")?)?;
    let requested_parts = repeated_arg(args, "--part");
    let prerequisite_capability = optional_arg(args, "--prerequisite")
        .map(|value| CognitiveCapability::parse(&value))
        .transpose()?;
    let prerequisite_parts = repeated_arg(args, "--prerequisite-part");
    if prerequisite_capability.is_none() && !prerequisite_parts.is_empty() {
        return Err("cognitive_composition_prerequisite_capability_required".to_string());
    }
    if prerequisite_capability.is_some() && prerequisite_parts.is_empty() {
        return Err("cognitive_composition_prerequisite_parts_required".to_string());
    }
    let (authenticated, store) = authenticated_store()?;
    let transitions = store.list_case_transitions(&case_id)?;
    let turn = find_turn(&case_id, &turn_id, &transitions)
        .ok_or_else(|| "cognitive_composition_turn_not_found".to_string())?;
    let selected_part_ids = if requested_parts.is_empty() {
        turn.ordered_parts
            .iter()
            .map(|part| part.part_id.clone())
            .collect()
    } else {
        requested_parts
    };
    let prerequisite = prerequisite_capability.map(|capability| CognitiveCompositionPrerequisite {
        capability,
        source_part_ids: prerequisite_parts,
    });
    let request = CognitiveCompositionRequest::new(
        turn,
        &participant_id,
        goal.clone(),
        selected_part_ids,
        prerequisite,
    )?;
    let content_store = ConversationContentStore::open(&yai_home())?;
    let outcome = execute_composition(
        &authenticated,
        &store,
        &content_store,
        turn,
        &request,
        &|| false,
        optional_arg(args, "--failpoint").as_deref(),
    )?;
    render_composition(
        args,
        &outcome.request,
        &outcome.closure,
        outcome.route,
        outcome.prerequisite.as_ref(),
        &outcome.primary,
    )
}

fn render_composition(
    args: &[String],
    request: &CognitiveCompositionRequest,
    closure: &CognitiveSourceClosure,
    route: &str,
    prerequisite: Option<&CognitiveRealizationOutcome>,
    primary: &CognitiveRealizationOutcome,
) -> Result<(), String> {
    let value = json!({
        "schema":"yai.cognitive_composition_result.v1",
        "composition_request":request,
        "composition_route":route,
        "source_closure":closure,
        "prerequisite":prerequisite.map(|outcome| json!({
            "plan_id":outcome.plan.plan_id,
            "route":outcome.plan.route,
            "execution_lane_id":outcome.plan.execution_lane_id,
            "provider_result_id":outcome.execution.as_ref().map(|value| value.result_id.as_str()).or_else(|| outcome.derived.as_ref().map(|value| value.provider_result_id.as_str())),
            "derived_content_id":outcome.derived.as_ref().map(|value| value.derived_content_id.as_str()),
            "provider_execution_performed_now":realization_performed_now(outcome),
            "posture":outcome.posture
        })),
        "primary":{
            "plan_id":primary.plan.plan_id,
            "route":primary.plan.route,
            "execution_lane_id":primary.plan.execution_lane_id,
            "provider_result_id":primary.execution.as_ref().map(|value| value.result_id.as_str()),
            "provider_execution_performed_now":realization_performed_now(primary),
            "posture":primary.posture
        },
        "conversation_turn_mutated":false,
        "composition_owner":"none_derived_control",
        "authority":"provider_candidate_material"
    });
    if json_requested(args) {
        println!(
            "{}",
            serde_json::to_string(&value)
                .map_err(|error| format!("cognitive_composition_encode_failed: {error}"))?
        );
    } else {
        println!("cognitive_composition: completed");
        println!("composition_request_id: {}", request.request_id);
        println!("composition_route: {route}");
        println!("source_closure_id: {}", closure.closure_id);
        println!("delivery_parts: {}", closure.delivery_count);
        println!(
            "prerequisite_provider_result_id: {}",
            prerequisite
                .and_then(|outcome| {
                    outcome
                        .execution
                        .as_ref()
                        .map(|value| value.result_id.as_str())
                        .or_else(|| {
                            outcome
                                .derived
                                .as_ref()
                                .map(|value| value.provider_result_id.as_str())
                        })
                })
                .unwrap_or("none")
        );
        println!(
            "primary_provider_result_id: {}",
            primary
                .execution
                .as_ref()
                .map(|value| value.result_id.as_str())
                .unwrap_or("none")
        );
        println!("conversation_turn_mutated: no");
        println!("composition_owner: none_derived_control");
    }
    Ok(())
}

fn render_realization(
    args: &[String],
    plan: &yai_core_engine::cognitive::CognitiveExecutionPlan,
    derived: Option<&ConversationDerivedContent>,
    execution: Option<&RecordedExecution>,
    posture: &str,
    recovered: bool,
) -> Result<(), String> {
    let execution_plan_id = execution
        .map(|value| value.plan_id.as_str())
        .or_else(|| derived.map(|value| value.plan_id.as_str()))
        .unwrap_or(plan.plan_id.as_str());
    let value = json!({
        "schema":"yai.cognitive_provider_realization_result.v1",
        "posture":posture,
        "plan_id":execution_plan_id,
        "validation_plan_id":plan.plan_id,
        "route":plan.route,
        "execution_lane_id":plan.execution_lane_id,
        "selected_target_id":plan.selected_target_id,
        "provider_selection_id":execution.map(|value| value.selection.selection_id.as_str()).or_else(|| derived.map(|value| value.provider_selection_id.as_str())),
        "provider_invocation_id":execution.map(|value| value.invocation_id.as_str()).or_else(|| derived.map(|value| value.provider_invocation_id.as_str())),
        "provider_result_id":execution.map(|value| value.result_id.as_str()).or_else(|| derived.map(|value| value.provider_result_id.as_str())),
        "provider_execution_performed_now":execution.is_some() && !recovered,
        "recovered_from_recorded_result":recovered,
        "derived_content":derived,
        "conversation_turn_mutated":false,
        "authority":"provider_candidate_material"
    });
    if json_requested(args) {
        println!(
            "{}",
            serde_json::to_string(&value)
                .map_err(|error| format!("cognitive_realization_encode_failed: {error}"))?
        );
    } else {
        println!("cognitive_realization: {posture}");
        println!("plan_id: {execution_plan_id}");
        println!("validation_plan_id: {}", plan.plan_id);
        println!("route: {:?}", plan.route);
        println!(
            "execution_lane_id: {}",
            plan.execution_lane_id.as_deref().unwrap_or("none")
        );
        println!(
            "provider_result_id: {}",
            value["provider_result_id"].as_str().unwrap_or("none")
        );
        println!(
            "derived_content_id: {}",
            derived
                .map(|value| value.derived_content_id.as_str())
                .unwrap_or("none")
        );
        println!("conversation_turn_mutated: no");
        println!("authority: provider_candidate_material");
    }
    Ok(())
}

fn cognitive_derived_show(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--participant")?;
    let (authenticated, store) = authenticated_store()?;
    let state = store.get_case_state_authorized(&authenticated, &case_id)?;
    if !state
        .participants
        .iter()
        .any(|item| item.participant_id == participant_id)
    {
        return Err("cognitive_derived_content_participant_not_bound".to_string());
    }
    let principal_id = authenticated.projected_principal_id();
    if !state.principal_participant_links.iter().any(|link| {
        state.tenant_id.as_deref() == Some(link.tenant_id.as_str())
            && link.participant_id == participant_id
            && link.principal_id == principal_id
    }) {
        return Err("cognitive_derived_content_principal_participant_mismatch".to_string());
    }
    let transitions = store.list_case_transitions(&case_id)?;
    let content_store = ConversationContentStore::open(&yai_home())?;
    let mut items = derived_content_from_history(&case_id, &transitions);
    items.retain(|item| {
        find_turn(&case_id, &item.source_turn_id, &transitions)
            .is_some_and(|turn| turn.participant_id == participant_id)
    });
    for item in &items {
        let turn = find_turn(&case_id, &item.source_turn_id, &transitions)
            .ok_or_else(|| "cognitive_derived_content_source_turn_missing".to_string())?;
        item.validate(turn)?;
        content_store.verify_object(&item.object)?;
    }
    items.sort_by(|left, right| left.derived_content_id.cmp(&right.derived_content_id));
    if json_requested(args) {
        println!(
            "{}",
            serde_json::to_string(&json!({
                "schema":"yai.conversation_derived_content_collection.v1",
                "case_id":case_id,
                "participant_id":participant_id,
                "count":items.len(),
                "items":items,
                "content_integrity":"verified"
            }))
            .map_err(|error| format!("cognitive_derived_content_encode_failed: {error}"))?
        );
    } else {
        println!("case_id: {case_id}");
        println!("derived_content_count: {}", items.len());
        for item in items {
            println!(
                "derived: {} capability={} source_turn={} provider_result={} object={}",
                item.derived_content_id,
                item.capability.as_str(),
                item.source_turn_id,
                item.provider_result_id,
                item.object.object_id
            );
        }
        println!("content_integrity: verified");
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
        "yai.case.cognitive.realize" => cognitive_realize(args),
        "yai.case.cognitive.compose" => cognitive_compose(args),
        "yai.case.cognitive.derived.show" => cognitive_derived_show(args),
        _ => Err(format!("unsupported cognitive operation: {operation_id}")),
    }
}
