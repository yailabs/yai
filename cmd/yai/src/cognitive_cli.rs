//! Advanced operator surfaces for I02 semantic cognitive planning.
//!
//! These commands mutate only provider-owned suitability evidence or canonical
//! Case cognitive bindings. Planning is read-only and cannot dispatch a model.

use super::*;
use serde_json::json;
use yai_core_engine::cognitive::{
    assess_lane_continuation, CognitiveBindingRole, CognitiveCapability,
    CognitiveCapabilityRequirement, CognitivePlanRoute, LaneContinuationReference,
    SemanticEvidencePosture,
};
use yai_core_engine::context::{InvocationOutputContract, ProjectionPurpose};
use yai_core_engine::conversation::{
    derived_content_from_history, find_turn, normalize_provider_derived_text, ContentModality,
    ConversationContentStore, ConversationDerivedContent, PROVIDER_DERIVED_TEXT_NORMALIZER,
};
use yai_core_engine::effect::digest_bytes;
use yai_core_engine::provider_governance::{
    CapabilityProvenance, ProviderCapability, ProviderCapabilityRequirement,
    ProviderRealizationShape, ProviderRequirement,
};
use yai_core_engine::security::AuthenticatedPrincipal;
use yai_core_engine::transition::TransitionPayload;

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

fn source_parts(
    store: &ConversationContentStore,
    turn: &yai_core_engine::conversation::ConversationTurn,
    requested: &[String],
) -> Result<Vec<provider::ProviderWireInputPart>, String> {
    let selected = if requested.is_empty() {
        turn.ordered_parts.iter().collect::<Vec<_>>()
    } else {
        let requested_count = requested.len();
        let requested = requested.iter().collect::<std::collections::BTreeSet<_>>();
        if requested.len() != requested_count {
            return Err("cognitive_realization_duplicate_source_part".to_string());
        }
        let parts = turn
            .ordered_parts
            .iter()
            .filter(|part| requested.contains(&part.part_id))
            .collect::<Vec<_>>();
        if parts.len() != requested.len() {
            return Err("cognitive_realization_source_part_not_found".to_string());
        }
        parts
    };
    selected
        .into_iter()
        .map(|part| {
            Ok(provider::ProviderWireInputPart {
                source_part_id: part.part_id.clone(),
                modality: part.object.modality.clone(),
                media_type: part.object.media_type.clone(),
                bytes: store.read_bytes(&part.object)?,
            })
        })
        .collect()
}

fn realization_shape(
    capability: &CognitiveCapability,
    parts: &[provider::ProviderWireInputPart],
) -> Result<ProviderRealizationShape, String> {
    if parts.is_empty() {
        return Err("cognitive_realization_source_parts_required".to_string());
    }
    match capability {
        CognitiveCapability::PrimaryConversation
            if parts
                .iter()
                .all(|part| part.modality == ContentModality::Text) =>
        {
            Ok(ProviderRealizationShape::TextToText)
        }
        CognitiveCapability::SpeechToText
            if parts.len() == 1
                && parts[0].modality == ContentModality::Audio
                && matches!(parts[0].media_type.as_str(), "audio/wav" | "audio/x-wav") =>
        {
            Ok(ProviderRealizationShape::AudioWavToText)
        }
        CognitiveCapability::ImageUnderstanding
            if parts
                .iter()
                .any(|part| part.modality == ContentModality::Image)
                && parts.iter().all(|part| {
                    part.modality == ContentModality::Text
                        || (part.modality == ContentModality::Image
                            && part.media_type == "image/png")
                }) =>
        {
            Ok(ProviderRealizationShape::OrderedPngTextToText)
        }
        _ => Err("cognitive_realization_content_shape_mismatch".to_string()),
    }
}

fn provider_requirement(
    cognitive: &CognitiveCapabilityRequirement,
) -> Result<ProviderRequirement, String> {
    ProviderRequirement::new(
        &format!("cognitive_realization:{}", cognitive.requirement_id),
        vec![
            ProviderCapabilityRequirement {
                capability: ProviderCapability::ChatText,
                minimum_provenance: CapabilityProvenance::Qualified,
            },
            ProviderCapabilityRequirement {
                capability: ProviderCapability::ModelExactAddressing,
                minimum_provenance: CapabilityProvenance::Qualified,
            },
        ],
        None,
    )
}

fn source_identity(turn_id: &str, part_ids: &[String]) -> Result<String, String> {
    let bytes = serde_json::to_vec(&(turn_id, part_ids))
        .map_err(|error| format!("cognitive_realization_source_encode_failed: {error}"))?;
    Ok(format!("conversation-source:{}", digest_bytes(&bytes)))
}

struct RecordedExecution {
    plan_id: String,
    selection: yai_core_engine::provider_governance::ProviderSelection,
    invocation_id: String,
    result_id: String,
    output: String,
}

fn recorded_execution(
    transitions: &[yai_core_engine::transition::Transition],
    provider_requirement_id: &str,
    current_binding_id: &str,
    current_target_id: &str,
) -> Result<Option<RecordedExecution>, String> {
    for transition in transitions.iter().rev() {
        let TransitionPayload::ProviderSelectionRecorded { selection } = &transition.payload else {
            continue;
        };
        if selection.requirement_id != provider_requirement_id
            || selection.selected_target_id != current_target_id
            || !transition
                .causal_refs
                .iter()
                .any(|value| value == current_binding_id)
            || !transition.causal_refs.contains(&format!(
                "provider-normalization-contract:{PROVIDER_DERIVED_TEXT_NORMALIZER}"
            ))
            || !selection
                .logical_turn_id
                .starts_with("cognitive-realization:")
        {
            continue;
        }
        let invocation = transitions
            .iter()
            .find_map(|candidate| match &candidate.payload {
                TransitionPayload::ProviderInvocationStarted {
                    invocation_id,
                    governance: Some(governance),
                    ..
                } if governance.selection_id == selection.selection_id => {
                    Some(invocation_id.clone())
                }
                _ => None,
            });
        let Some(invocation_id) = invocation else {
            continue;
        };
        let result = transitions
            .iter()
            .find_map(|candidate| match &candidate.payload {
                TransitionPayload::ProviderResultRecorded {
                    result_id,
                    invocation_id: owner,
                    output,
                    ..
                } if owner == &invocation_id => Some((result_id.clone(), output.clone())),
                _ => None,
            });
        let Some((result_id, output)) = result else {
            let outcome = transitions
                .iter()
                .find_map(|candidate| match &candidate.payload {
                    TransitionPayload::ProviderAttemptOutcomeRecorded { outcome }
                        if outcome.selection_id == selection.selection_id =>
                    {
                        Some(outcome)
                    }
                    _ => None,
                });
            return match outcome {
                Some(outcome) if outcome.retry_safe() => Ok(None),
                Some(outcome) => Err(format!(
                    "cognitive_realization_existing_attempt_not_retry_safe:{:?}",
                    outcome.delivery
                )),
                None => Err(
                    "cognitive_realization_delivery_indeterminate_existing_invocation".to_string(),
                ),
            };
        };
        return Ok(Some(RecordedExecution {
            plan_id: selection
                .logical_turn_id
                .trim_start_matches("cognitive-realization:")
                .to_string(),
            selection: selection.clone(),
            invocation_id,
            result_id,
            output,
        }));
    }
    Ok(None)
}

#[allow(clippy::too_many_arguments)]
fn publish_derived(
    store: &LmdbRecordStore,
    authenticated: &AuthenticatedPrincipal,
    content_store: &ConversationContentStore,
    turn: &yai_core_engine::conversation::ConversationTurn,
    part_ids: Vec<String>,
    capability: CognitiveCapability,
    shape: ProviderRealizationShape,
    plan_id: &str,
    lane_id: &str,
    binding_id: &str,
    semantic_evidence_id: &str,
    target_id: &str,
    target_digest: &str,
    execution: &RecordedExecution,
) -> Result<ConversationDerivedContent, String> {
    let normalized = normalize_provider_derived_text(&execution.output)?;
    let object = content_store.publish_derived_text(&turn.tenant_id, &turn.case_id, &normalized)?;
    let derived = ConversationDerivedContent::new(
        turn,
        part_ids,
        capability,
        shape,
        plan_id,
        lane_id,
        binding_id,
        semantic_evidence_id,
        target_id,
        target_digest,
        &execution.selection.qualification_id,
        &execution.selection.selection_id,
        &execution.invocation_id,
        &execution.result_id,
        object,
    )?;
    content_store.verify_object(&derived.object)?;
    store.record_conversation_derived_content_authorized(authenticated, derived)
}

fn cognitive_realize(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--participant")?;
    let turn_id = named_arg(args, "--turn")?;
    let capability = CognitiveCapability::parse(&named_arg(args, "--capability")?)?;
    let requested_parts = repeated_arg(args, "--part");
    let continuation = continuation_from_args(args)?;
    let (authenticated, store) = authenticated_store()?;
    let state = store.get_case_state_authorized(&authenticated, &case_id)?;
    let principal_id = authenticated.projected_principal_id();
    if !state.principal_participant_links.iter().any(|link| {
        state.tenant_id.as_deref() == Some(link.tenant_id.as_str())
            && link.participant_id == participant_id
            && link.principal_id == principal_id
    }) {
        return Err("cognitive_realization_principal_participant_mismatch".to_string());
    }
    let transitions = store.list_case_transitions(&case_id)?;
    let turn = find_turn(&case_id, &turn_id, &transitions)
        .ok_or_else(|| "cognitive_realization_turn_not_found".to_string())?;
    if turn.participant_id != participant_id {
        return Err("cognitive_realization_turn_participant_mismatch".to_string());
    }
    let content_store = ConversationContentStore::open(&yai_home())?;
    let wire_parts = source_parts(&content_store, turn, &requested_parts)?;
    let part_ids = wire_parts
        .iter()
        .map(|part| part.source_part_id.clone())
        .collect::<Vec<_>>();
    let shape = realization_shape(&capability, &wire_parts)?;
    let cognitive_requirement = CognitiveCapabilityRequirement::new(
        &case_id,
        &participant_id,
        capability.clone(),
        &source_identity(&turn_id, &part_ids)?,
    )?;
    let plan = store.plan_case_cognitive_execution_authorized(
        &authenticated,
        &case_id,
        &participant_id,
        &cognitive_requirement,
    )?;
    if plan.route == CognitivePlanRoute::Unresolved {
        return Err(format!(
            "cognitive_realization_plan_unresolved:{:?}",
            plan.unresolved_reason
        ));
    }
    let binding_id = plan
        .selected_binding_id
        .as_deref()
        .ok_or_else(|| "cognitive_realization_binding_missing".to_string())?;
    let target_id = plan
        .selected_target_id
        .as_deref()
        .ok_or_else(|| "cognitive_realization_target_missing".to_string())?;
    let evidence_id = plan
        .semantic_evidence_id
        .as_deref()
        .ok_or_else(|| "cognitive_realization_semantic_evidence_missing".to_string())?;
    let lane_id = plan
        .execution_lane_id
        .as_deref()
        .ok_or_else(|| "cognitive_realization_lane_missing".to_string())?;
    let continuation_posture = assess_lane_continuation(&plan, continuation.as_ref());
    if !matches!(
        continuation_posture,
        yai_core_engine::cognitive::LaneContinuationPosture::Compatible
            | yai_core_engine::cognitive::LaneContinuationPosture::NotProvidedSemanticReconstruction
    ) {
        return Err(format!(
            "cognitive_realization_continuation_rejected:{continuation_posture:?}"
        ));
    }
    let provider_requirement = provider_requirement(&cognitive_requirement)?;
    let (target, _, _, _) = store.provider_posture_authorized(&authenticated, target_id)?;
    if let Some(derived) = derived_content_from_history(&case_id, &transitions)
        .into_iter()
        .find(|derived| {
            derived.source_turn_id == turn_id
                && derived.source_part_ids == part_ids
                && derived.capability == capability
                && derived.cognitive_binding_id == binding_id
        })
    {
        content_store.verify_object(&derived.object)?;
        return render_realization(args, &plan, Some(derived), None, "already_published", false);
    }
    let recorded = recorded_execution(
        &transitions,
        &provider_requirement.requirement_id,
        binding_id,
        target_id,
    )?;
    let (execution, recovered) = if let Some(recorded) = recorded {
        (recorded, true)
    } else {
        let logical_turn_id = format!("cognitive-realization:{}", plan.plan_id);
        let route = provider::governed_provider_route_for_exact_plan(
            &plan,
            &provider_requirement,
            &shape,
            &logical_turn_id,
            continuation.as_ref(),
        )?;
        let options = provider::SemanticInvocationOptions {
            conversation_turn_id: Some(turn.turn_id.clone()),
            ..provider::SemanticInvocationOptions::default()
        };
        let task = format!(
            "Realize the explicit YAI cognitive capability {} over committed Turn {} and source parts [{}]. Return bounded text only; the result remains non-authoritative provider output.",
            capability.as_str(),
            turn.turn_id,
            part_ids.join(",")
        );
        let result = match provider::invoke_semantic_provider_typed(
            &route.args,
            ProjectionPurpose::Conversation,
            &task,
            InvocationOutputContract::NaturalLanguage,
            &options,
            &wire_parts,
        ) {
            Ok(result) => {
                provider::record_governed_provider_outcome(
                    &case_id,
                    &route.selection,
                    None,
                    Some(result.request_bytes_written),
                )?;
                result
            }
            Err(error) => {
                let outcome = provider::record_governed_provider_outcome(
                    &case_id,
                    &route.selection,
                    Some(&error),
                    None,
                )?;
                return Err(format!(
                    "cognitive_realization_failed:delivery={:?}:derived_content=false:{error}",
                    outcome.delivery
                ));
            }
        };
        let recorded = RecordedExecution {
            plan_id: plan.plan_id.clone(),
            selection: route.selection,
            invocation_id: result.invocation_id,
            result_id: result.result_id,
            output: result.raw_output,
        };
        if optional_arg(args, "--failpoint").as_deref() == Some("after-provider-result") {
            return Err(format!(
                "cognitive_realization_failpoint_after_provider_result:{}",
                recorded.result_id
            ));
        }
        (recorded, false)
    };
    if plan.route == CognitivePlanRoute::Native {
        return render_realization(
            args,
            &plan,
            None,
            Some(&execution),
            if recovered {
                "recovered_result"
            } else {
                "completed"
            },
            recovered,
        );
    }
    let derived = publish_derived(
        &store,
        &authenticated,
        &content_store,
        turn,
        part_ids,
        capability,
        shape,
        &execution.plan_id,
        lane_id,
        binding_id,
        evidence_id,
        target_id,
        &target.integrity_digest,
        &execution,
    )?;
    render_realization(
        args,
        &plan,
        Some(&derived),
        Some(&execution),
        if recovered {
            "recovered_without_dispatch"
        } else {
            "completed"
        },
        recovered,
    )
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
        "yai.case.cognitive.derived.show" => cognitive_derived_show(args),
        _ => Err(format!("unsupported cognitive operation: {operation_id}")),
    }
}
