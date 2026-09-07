//! Public native-function lowering of a derived Case capability view.
//! This adapter owns neither a tool registry nor Operation admission.

use super::{decode_native_function_reply, native_function_tools, NativeFunctionDefinition};
use serde_json::{json, Value};
use yai_core_engine::admission::{
    operation_kind_name, CaseCapabilityAction, CaseCapabilityOutput, CaseCapabilityProposal,
    CaseCapabilityView, CASE_CAPABILITY_OUTPUT_SCHEMA,
};
use yai_core_engine::effect::access::{AccessKind, ResourceAction, ResourceObservation};
use yai_core_engine::effect::{digest_bytes, OperationKind};
use yai_core_engine::effect::{DecisionOutcome, OperationOrigin};
use yai_core_engine::transition::{Transition, TransitionPayload};

/// Reconstruct the public assistant/function-result exchange from canonical
/// request and outcome lineage. These messages are disposable wire lowering;
/// no provider conversation/session becomes a Case owner.
pub(super) fn feedback(
    view: &CaseCapabilityView,
    result_ids: &[String],
    history: &[Transition],
) -> Result<Vec<Value>, String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut messages = Vec::new();
    if result_ids.len() > 24 {
        return Err("capability_feedback_bound".into());
    }
    for result_id in result_ids {
        if !seen.insert(result_id) {
            return Err("capability_feedback_duplicate".into());
        }
        let (invocation_id, output) = history
            .iter()
            .find_map(|t| match &t.payload {
                TransitionPayload::ProviderResultRecorded {
                    result_id: id,
                    invocation_id,
                    output,
                    ..
                } if id == result_id && t.case_id == view.case_id => Some((invocation_id, output)),
                _ => None,
            })
            .ok_or("capability_feedback_result_not_canonical")?;
        let governance = history
            .iter()
            .find_map(|t| match &t.payload {
                TransitionPayload::ProviderInvocationStarted {
                    invocation_id: id,
                    participant_id,
                    governance: Some(g),
                    ..
                } if id == invocation_id && participant_id == &view.participant_id => Some(g),
                _ => None,
            })
            .ok_or("capability_feedback_participant_mismatch")?;
        if !history.iter().any(|t|matches!(&t.payload,
            TransitionPayload::ProviderSelectionRecorded {selection} if selection.selection_id == governance.selection_id
                && t.causal_refs.contains(&format!("provider-normalization-contract:{CASE_CAPABILITY_OUTPUT_SCHEMA}"))
                && t.causal_refs.contains(&"provider-realization-shape:text_functions_to_text_or_call".to_string()))) {
            return Err("capability_feedback_native_lineage_required".into());
        }
        let output: CaseCapabilityOutput =
            serde_json::from_str(output).map_err(|_| "capability_feedback_output_invalid")?;
        if output.schema != CASE_CAPABILITY_OUTPUT_SCHEMA {
            return Err("capability_feedback_schema_invalid".into());
        }
        let call = output
            .request
            .ok_or("capability_feedback_request_missing")?;
        let operation = history.iter().find_map(|t|match &t.payload {
            TransitionPayload::OperationRecorded {operation} if operation.participant_id == view.participant_id
                && matches!(&operation.origin,OperationOrigin::ProviderResult {provider_result_id,..} if provider_result_id == result_id)=>Some(operation),_=>None,
        }).ok_or("capability_feedback_operation_missing")?;
        let terminal = history
            .iter()
            .rev()
            .find(|t| match &t.payload {
                TransitionPayload::ResourceObservationRecorded { observation } => {
                    observation.operation_id == operation.operation_id
                }
                TransitionPayload::ResourceEffectFinalized { receipt, .. } => {
                    receipt.operation_id == operation.operation_id
                }
                TransitionPayload::EffectFinalized { receipt, .. } => {
                    receipt.operation_id == operation.operation_id
                }
                TransitionPayload::CaseContentAdmitted { admission } => {
                    admission.operation_id == operation.operation_id
                }
                TransitionPayload::DecisionRecorded { decision } => {
                    decision.operation_id == operation.operation_id
                        && decision.outcome == DecisionOutcome::Deny
                }
                _ => false,
            })
            .ok_or("capability_feedback_terminal_outcome_missing")?;
        let denied = matches!(
            &terminal.payload,
            TransitionPayload::DecisionRecorded { .. }
        );
        if !denied
            && !view.entries.iter().any(|entry| {
                entry.resource.attachment_id == call.resource_id
                    && entry
                        .resource
                        .access
                        .as_ref()
                        .is_some_and(|a| a.configuration_digest == call.configuration_digest)
            })
        {
            return Err("capability_feedback_resource_no_longer_visible".into());
        }
        let arguments = match &call.action {
            CaseCapabilityAction::FilesystemWrite { path, content } => {
                json!({"path":path,"content":content})
            }
            CaseCapabilityAction::Resource {
                action: ResourceAction::McpToolCall { arguments, .. },
            } => arguments.clone(),
            CaseCapabilityAction::Resource {
                action: ResourceAction::McpResourceRead { .. } | ResourceAction::McpCatalog,
            } => json!({}),
            CaseCapabilityAction::Resource { action } => {
                let mut value = serde_json::to_value(action).map_err(|e| e.to_string())?;
                value
                    .as_object_mut()
                    .ok_or("capability_feedback_arguments_invalid")?
                    .remove("action");
                value
            }
        };
        messages.push(json!({"role":"assistant","content":null,"tool_calls":[{
            "id":call.provider_call_id,"type":"function","function":{
                "name":call.provider_function_name,"arguments":serde_json::to_string(&arguments).map_err(|e|e.to_string())?
            }
        }]}));
        let result = json!({"provider_result_id":result_id,"operation_id":operation.operation_id,
            "transition_id":terminal.transition_id,"posture":if denied {"denied"} else {"recorded_outcome"},
            "material_is_authority":false,"outcome":terminal.payload});
        messages.push(json!({"role":"tool","tool_call_id":call.provider_call_id,
            "content":serde_json::to_string(&result).map_err(|e|e.to_string())?}));
    }
    if serde_json::to_vec(&messages)
        .map_err(|e| e.to_string())?
        .len()
        > 256 * 1024
    {
        return Err("capability_feedback_bytes_exceeded".into());
    }
    Ok(messages)
}

pub(super) struct OfferedCapability {
    pub definition: NativeFunctionDefinition,
    resource_id: String,
    configuration_digest: String,
    template: Value,
    filesystem_write: bool,
}

fn object(properties: Value) -> Value {
    let required = properties
        .as_object()
        .expect("schema properties")
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    json!({"type":"object", "properties":properties, "required":required, "additionalProperties":false})
}

/// No network or filesystem access. MCP schemas originate only from exact
/// canonical observations supplied by the application and are checked again
/// against the live catalog by the resource carrier before tools/call.
pub(super) fn offer(
    view: &CaseCapabilityView,
    catalogs: &[ResourceObservation],
) -> Result<Vec<OfferedCapability>, String> {
    if catalogs.len() > 16 {
        return Err("capability_catalog_bound".into());
    }
    for catalog in catalogs {
        catalog.validate()?;
        if catalog.case_id != view.case_id
            || catalog.participant_id != view.participant_id
            || catalog.kind != AccessKind::McpCatalog
        {
            return Err("capability_catalog_scope_mismatch".into());
        }
    }
    let mut offered = Vec::new();
    for entry in &view.entries {
        let resource = &entry.resource;
        let Some(access) = &resource.access else {
            continue;
        };
        let path = json!({"type":"string", "minLength":1,"maxLength":1024});
        let names = json!({"type":"string", "enum":access.names});
        let mut shapes = Vec::new();
        match &entry.operation_kind {
            OperationKind::FilesystemWrite => shapes.push((
                "write".to_string(), object(json!({"path":path,"content":{"type":"string","minLength":1,"maxLength":resource.max_write_bytes}})), json!({}), true)),
            OperationKind::ResourceAccess(kind) => match kind {
                AccessKind::FilesystemRead | AccessKind::Discover => shapes.push((
                    "path".into(), object(json!({"path":path})),
                    json!({"action":if *kind == AccessKind::Discover {"discover"} else {"filesystem_read"}}), false)),
                AccessKind::FilesystemSearch => shapes.push(("search".into(),
                    object(json!({"path":path,"needle":{"type":"string","minLength":1,"maxLength":256}})),json!({"action":"filesystem_search"}),false)),
                AccessKind::AdmitContent => shapes.push(("admit".into(),
                    object(json!({"path":path,"candidate_digest":{"type":"string","pattern":"^sha256:[a-f0-9]{64}$"}})),json!({"action":"admit_content"}),false)),
                AccessKind::ContentRead => shapes.push(("content".into(),
                    object(json!({"admission_id":{"type":"string","pattern":"^case-content:[a-f0-9]{32}$"}})),json!({"action":"content_read"}),false)),
                AccessKind::ProcessRun | AccessKind::DatabaseQuery | AccessKind::DatabaseMutation | AccessKind::HttpFetch if !access.names.is_empty() => {
                    let action = match kind {
                        AccessKind::ProcessRun => "process_run", AccessKind::DatabaseQuery => "database_query",
                        AccessKind::DatabaseMutation => "database_mutation", _ => "http_fetch",
                    };
                    shapes.push(("named".into(),object(json!({"name":names})),json!({"action":action}),false));
                }
                AccessKind::McpCatalog => shapes.push(("catalog".into(),object(json!({})),json!({"action":"mcp_catalog"}),false)),
                AccessKind::McpToolCall | AccessKind::McpResourceRead => {
                    let Some(catalog) = catalogs.iter().rev().find(|c|
                        c.resource_attachment_id == resource.attachment_id && c.configuration_digest == access.configuration_digest
                    ) else { continue; };
                    let digest = catalog.result["catalog_digest"].as_str().ok_or("capability_catalog_digest_missing")?;
                    let collection = if *kind == AccessKind::McpToolCall { "tools" } else { "resources" };
                    let definitions = catalog.result[collection].as_object().ok_or("capability_catalog_definitions_missing")?;
                    for (name, definition) in definitions {
                        if !access.names.contains(name) { continue; }
                        if *kind == AccessKind::McpToolCall {
                            shapes.push((name.clone(), definition["inputSchema"].clone(),
                                json!({"action":"mcp_tool_call", "name":name,"catalog_digest":digest}),false));
                        } else {
                            shapes.push((name.clone(),object(json!({})),json!({"action":"mcp_resource_read","uri":name,"catalog_digest":digest}),false));
                        }
                    }
                }
                _ => {}
            },
            OperationKind::ProcessSignal => {} // existing separate controlled vertical
        }
        for (name, parameters, template, filesystem_write) in shapes {
            let identity = serde_json::to_vec(&(
                &view.view_id,
                &resource.attachment_id,
                &entry.operation_kind,
                &name,
                &parameters,
                &template,
            ))
            .map_err(|e| e.to_string())?;
            let hash = digest_bytes(&identity);
            let hash = hash.trim_start_matches("sha256:");
            offered.push(OfferedCapability {
                definition: NativeFunctionDefinition {
                    name: format!("yai_{}", &hash[..48]),
                    description: format!("Request {} on Case resource {} ({name}). Candidate only: current YAI policy may deny or require human review. No authority is conferred.", operation_kind_name(&entry.operation_kind), resource.attachment_id),
                    parameters,
                },
                resource_id: resource.attachment_id.clone(),
                configuration_digest: access.configuration_digest.clone(),
                template,
                filesystem_write,
            });
        }
    }
    native_function_tools(
        &offered
            .iter()
            .map(|o| o.definition.clone())
            .collect::<Vec<_>>(),
    )?;
    Ok(offered)
}

pub(super) fn decode(
    body: &str,
    exact_model: &str,
    view: &CaseCapabilityView,
    offered: &[OfferedCapability],
) -> Result<String, String> {
    let definitions = offered
        .iter()
        .map(|o| o.definition.clone())
        .collect::<Vec<_>>();
    let reply = decode_native_function_reply(body.as_bytes(), exact_model, &definitions)?;
    let request = reply
        .call
        .map(|call| {
            let offered = offered
                .iter()
                .find(|o| o.definition.name == call.name)
                .ok_or("capability_not_offered")?;
            let action = if offered.filesystem_write {
                CaseCapabilityAction::FilesystemWrite {
                    path: call.arguments["path"]
                        .as_str()
                        .ok_or("capability_path_missing")?
                        .into(),
                    content: call.arguments["content"]
                        .as_str()
                        .ok_or("capability_content_missing")?
                        .into(),
                }
            } else {
                let mut action = offered.template.clone();
                let object = action
                    .as_object_mut()
                    .ok_or("capability_template_invalid")?;
                if object.get("action").is_some_and(|a| a == "mcp_tool_call") {
                    object.insert("arguments".into(), call.arguments.clone());
                } else {
                    for (key, value) in call
                        .arguments
                        .as_object()
                        .ok_or("capability_arguments_invalid")?
                    {
                        if object.insert(key.clone(), value.clone()).is_some() {
                            return Err("capability_template_override".into());
                        }
                    }
                }
                let action: ResourceAction = serde_json::from_value(action)
                    .map_err(|e| format!("capability_action_invalid:{e}"))?;
                CaseCapabilityAction::Resource { action }
            };
            Ok::<_, String>(CaseCapabilityProposal {
                provider_call_id: call.call_id,
                provider_function_name: call.name,
                resource_id: offered.resource_id.clone(),
                configuration_digest: offered.configuration_digest.clone(),
                action,
            })
        })
        .transpose()?;
    serde_json::to_string(&CaseCapabilityOutput {
        schema: CASE_CAPABILITY_OUTPUT_SCHEMA.into(),
        capability_view_id: view.view_id.clone(),
        text: reply.text,
        request,
    })
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use yai_core_engine::admission::CaseResourceCapability;
    use yai_core_engine::effect::access::{ResourceAccessContract, RESOURCE_ACCESS_SCHEMA};
    use yai_core_engine::transition::{ResourceAttachmentState, ResourceKind, ReviewRequirement};

    #[test]
    fn native_case_capability_lowering_never_parses_prose_or_accepts_unoffered_arguments() {
        let view = CaseCapabilityView {
            schema: "yai.case_capability_view.v1".into(),
            view_id: "case-capability-view:unit-only".into(),
            case_id: "case:unit".into(),
            tenant_id: "tenant:unit".into(),
            case_generation: 10,
            participant_id: "participant:model".into(),
            effective_policy_id: "policy:unit".into(),
            effective_policy_digest: digest_bytes(b"unit-policy"),
            exclusions: vec![],
            entries: vec![CaseResourceCapability {
                resource: ResourceAttachmentState {
                    attachment_id: "resource:workspace".into(),
                    kind: ResourceKind::Filesystem,
                    allowed_write_prefix: String::new(),
                    max_write_bytes: 0,
                    policy_id: "policy:envelope".into(),
                    policy_owner_participant_id: "participant:human".into(),
                    review_requirement: ReviewRequirement::Automatic,
                    process_signal_actions: vec![],
                    access: Some(ResourceAccessContract {
                        schema: RESOURCE_ACCESS_SCHEMA.into(),
                        configuration_digest: digest_bytes(b"exact binding"),
                        participant_ids: vec!["participant:model".into()],
                        operations: vec![AccessKind::FilesystemRead],
                        read_prefixes: vec!["src".into()],
                        names: vec![],
                        max_output_bytes: 8192,
                        max_items: 16,
                    }),
                },
                operation_kind: OperationKind::ResourceAccess(AccessKind::FilesystemRead),
                policy_constraints: vec![],
                requires_current_decision: true,
            }],
        };
        let offered = offer(&view, &[]).unwrap();
        assert_eq!(offered.len(), 1);
        assert_eq!(
            offered[0].definition,
            offer(&view, &[]).unwrap()[0].definition
        );
        let response = |arguments: Value| {
            json!({"model":"whisper-vision-not-authority","choices":[{"finish_reason":"tool_calls","message":{
            "role":"assistant","content":null,"tool_calls":[{"id":"call-exact","type":"function","function":{
                "name":offered[0].definition.name,"arguments":arguments.to_string(),
            }}],
        }}]}).to_string()
        };
        let packet = decode(
            &response(json!({"path":"src/retry.txt"})),
            "whisper-vision-not-authority",
            &view,
            &offered,
        )
        .unwrap();
        let parsed: CaseCapabilityOutput = serde_json::from_str(&packet).unwrap();
        assert_eq!(parsed.capability_view_id, view.view_id);
        assert_eq!(
            parsed.request.as_ref().unwrap().resource_id,
            "resource:workspace"
        );
        assert!(
            matches!(&parsed.request.unwrap().action,CaseCapabilityAction::Resource {action:ResourceAction::FilesystemRead {path}} if path == "src/retry.txt")
        );
        assert!(decode(
            &response(json!({"path":"src/retry.txt","endpoint":"http://elsewhere"})),
            "whisper-vision-not-authority",
            &view,
            &offered
        )
        .is_err());
        let prose = json!({"model":"whisper-vision-not-authority","choices":[{"finish_reason":"stop","message":{"role":"assistant","content":packet}}]}).to_string();
        let parsed: CaseCapabilityOutput = serde_json::from_str(
            &decode(&prose, "whisper-vision-not-authority", &view, &offered).unwrap(),
        )
        .unwrap();
        assert!(
            parsed.request.is_none(),
            "serialized operation-looking prose is still only text"
        );
        assert!(parsed.text.is_some());
        let mut empty = view.clone();
        empty.entries.clear();
        assert!(
            offer(&empty, &[]).is_err(),
            "no ambient fallback tool surface"
        );
    }
}
