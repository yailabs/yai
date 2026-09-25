//! Shared provider execution preparation and public wire lowering.
//! Case/W and authorization remain in engine owners; no CLI arguments or global
//! profile selection enter this reusable execution boundary.

mod preflight;
pub mod probes;

/// Explicit-profile credential lookup for resident application carriers.
/// Never search the Host's working directory: it is not the client's profile.
pub fn credential_from_profile(home: &std::path::Path, name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty()).or_else(|| {
        let path = std::env::var_os("YAI_ENV_FILE").map(std::path::PathBuf::from)
            .filter(|path| path.is_file()).unwrap_or_else(|| home.join("env"));
        let content = std::fs::read_to_string(path).ok()?;
        content.lines().filter_map(parse_env_assignment)
            .find_map(|(key, value)| (key == name && !value.is_empty()).then_some(value))
    })
}

pub fn parse_env_assignment(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let line = line.strip_prefix("export ").unwrap_or(line).trim();
    let (key, value) = line.split_once('=')?;
    let key = key.trim();
    if key.is_empty()
        || !key
            .chars()
            .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
        || key.chars().next().is_some_and(|ch| ch.is_ascii_digit())
    {
        return None;
    }

    let value = value.trim();
    let value = if value.len() >= 2 {
        let bytes = value.as_bytes();
        if (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
        {
            &value[1..value.len() - 1]
        } else {
            value
        }
    } else {
        value
    };
    Some((key.to_string(), value.to_string()))
}

pub struct ProviderInvocationRefs {
    pub attempt_id: String,
    pub invocation_id: String,
}

pub struct ControlledProviderResult {
    pub invocation_id: String,
    pub result_id: String,
    pub raw_output: String,
    pub provider_id: String,
    pub model_id: String,
    pub projection_id: String,
    pub context_frame_id: String,
    pub residency_plan_id: String,
    pub resident_item_ids: Vec<String>,
    pub projection_selected_items: usize,
    pub projection_omitted_items: usize,
    pub semantic_units: usize,
    pub estimated_input_units: usize,
    pub usage: ProviderUsageTelemetry,
    pub request_bytes_written: u64,
}

pub fn invocation_lineage(
    semantic: &SemanticInvocation,
    disposition: ContinuationDisposition,
) -> ProviderInvocationLineage {
    ProviderInvocationLineage {
        projection_id: semantic.projection.projection_id.clone(),
        context_frame_id: semantic.frame.frame_id.clone(),
        case_generation: semantic.projection.case_generation,
        rendered_input_id: semantic.rendered.metadata.rendered_input_id.clone(),
        rendered_input_digest: semantic.rendered.metadata.content_digest.clone(),
        output_contract_id: semantic.output_contract_id.clone(),
        continuation_disposition: continuation_disposition_label(&disposition).to_string(),
    }
}

fn continuation_disposition_label(value: &ContinuationDisposition) -> &'static str {
    match value {
        ContinuationDisposition::NotProvided => "not_provided",
        ContinuationDisposition::Used => "used",
        ContinuationDisposition::InvalidatedAndRetried => "invalidated_and_retried",
    }
}

/// One execution chain, shared by terminal and Application clients. The caller
/// supplies canonical admission/result commits; transport never owns authority.
#[allow(clippy::too_many_arguments)]
pub fn execute_prepared(
    store: &LmdbRecordStore,
    config: &ProviderConfig,
    semantic: &SemanticInvocation,
    options: &SemanticInvocationOptions,
    typed_parts: Option<&[ProviderWireInputPart]>,
    admit: impl FnOnce(yai_core_engine::transition::ProviderInvocationLineage) -> Result<ProviderInvocationRefs, String>,
    record: impl FnOnce(&ProviderInvocationRefs, yai_core_engine::transition::ProviderInvocationLineage, &str) -> Result<String, String>,
) -> Result<(ProviderInvocationRefs, ControlledProviderResult), String> {
    let requested_disposition = if config.continuation_ref.is_some() {
        ContinuationDisposition::Used
    } else {
        ContinuationDisposition::NotProvided
    };
    let invocation = admit(invocation_lineage(semantic, requested_disposition))
        .map_err(|error| format!("provider_not_dispatched:invocation_start:{error}"))?;
    let structured_json = matches!(
        semantic.frame.output_contract,
        InvocationOutputContract::MemoryConsolidation { .. }
            | InvocationOutputContract::WorkflowPlanPatch { .. }
    );
    let observe = |wire: WireObservation| {
        let observation = yai_core_engine::context::ProviderInputObservation {
            schema: "yai.provider_input_observation.v1".into(),
            observation_id: provider_input_observation_id(&invocation.invocation_id),
            invocation_id: invocation.invocation_id.clone(),
            case_id: semantic.frame.case_id.clone(), case_generation: semantic.frame.case_generation,
            participant_id: semantic.frame.participant_id.clone(),
            rendered_input_id: semantic.rendered.metadata.rendered_input_id.clone(),
            target_id: config.provider_id.clone(), model_id: config.model.clone(),
            serialized_request_digest: wire.digest, serialized_request_bytes: wire.bytes,
            observed_at_unix_ms: crate::now_unix_ms()?, capacity: wire.capacity, refusal: wire.refusal,
        };
        store.put_semantic_context_artifact(&SemanticContextArtifact::ProviderInputObservation(observation))
    };
    let transport = provider_chat_completion_observed(
        store,
        config,
        &semantic.rendered,
        structured_json,
        typed_parts,
        &semantic.frame.output_contract,
        Some(options.max_estimated_input_units),
        Some(&observe),
    )?;
    let result_lineage = invocation_lineage(semantic, transport.continuation_disposition.clone());
    let result_id = record(&invocation, result_lineage, &transport.output)
    .map_err(|error| {
        format!(
            "provider_delivery_indeterminate:result_commit:bytes={}:{}",
            transport.request_bytes_written, error
        )
    })?;
    let result = ControlledProviderResult {
        invocation_id: invocation.invocation_id.clone(),
        result_id,
        raw_output: transport.output,
        provider_id: config.provider_id.clone(),
        model_id: config.model.clone(),
        projection_id: semantic.projection.projection_id.clone(),
        context_frame_id: semantic.frame.frame_id.clone(),
        residency_plan_id: semantic.residency.plan_id.clone(),
        resident_item_ids: semantic.residency.selected_item_ids.clone(),
        projection_selected_items: semantic.projection.bounds.selected_items,
        projection_omitted_items: semantic.projection.bounds.omitted_items,
        semantic_units: semantic.residency.selected_semantic_units,
        estimated_input_units: semantic.rendered.metadata.content_chars.div_ceil(4),
        usage: transport.usage,
        request_bytes_written: transport.request_bytes_written,
    };
    Ok((invocation, result))
}
use std::collections::{BTreeSet, HashSet};
use std::fmt;
use std::time::Instant;
use serde::{Deserialize, Deserializer};
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};
use yai_core_engine::context::{build_context_frame, render_openai_compatible, ContextFrame,
    ContinuationDisposition, InvocationOutputContract, Projection, ProjectionPurpose,
    ProjectionRequest, ProviderContinuationReference, ProviderModelProfile, RenderedInput,
    SemanticContextArtifact};
use yai_core_engine::provider_governance::{ProviderLocality, ProviderSelection, ProviderRequirement, ProviderAttemptOutcome, ProviderDeliveryClass, ProviderTransportStage};
use yai_core_engine::store::lmdb::ProviderSelectionStoreOutcome;
use yai_core_engine::transition::{ProviderInvocationGovernance, ProviderInvocationLineage, TransitionPayload};
use yai_core_engine::residency::ResidencyPlan;
use yai_core_engine::memory::DEFAULT_RETRIEVAL_LIMIT;
use yai_core_engine::store::lmdb::LmdbRecordStore;
#[path = "provider_execution/capabilities.rs"]
pub mod capabilities;
pub fn governed_provider_route_for_exact_plan(
    authenticated: &yai_core_engine::security::AuthenticatedPrincipal,
    store: &LmdbRecordStore,
    credential: &dyn Fn(&str) -> Option<String>,
    plan: &yai_core_engine::cognitive::CognitiveExecutionPlan,
    requirement: &ProviderRequirement,
    realization_shape: &yai_core_engine::provider_governance::ProviderRealizationShape,
    logical_turn_id: &str,
    continuation: Option<&yai_core_engine::cognitive::LaneContinuationReference>,
    realization_causal_refs: &[String],
) -> Result<(ProviderConfig, ProviderSelection), String> {
    use yai_core_engine::cognitive::{assess_lane_continuation, LaneContinuationPosture};
    let target_id = plan
        .selected_target_id
        .as_deref()
        .ok_or_else(|| "cognitive_realization_plan_unresolved".to_string())?;
    let (target, _, _, _) = store.provider_posture_authorized(&authenticated, target_id)?;
    let mut available_credentials = BTreeSet::new();
    if target.credential_ref == "none"
        || target
            .credential_ref
            .strip_prefix("env:")
            .is_some_and(|name| credential(name).is_some())
    {
        available_credentials.insert(target.credential_ref.clone());
    }
    if !matches!(
        assess_lane_continuation(plan, continuation),
        LaneContinuationPosture::NotProvidedSemanticReconstruction
            | LaneContinuationPosture::Compatible
    ) {
        return Err("cognitive_realization_continuation_incompatible".to_string());
    }
    let selection = match store.select_case_provider_exact_authorized(
        &authenticated,
        plan,
        requirement,
        realization_shape,
        logical_turn_id,
        &available_credentials,
        realization_causal_refs,
    )? {
        ProviderSelectionStoreOutcome::Selected { selection, .. }
        | ProviderSelectionStoreOutcome::AlreadySelected(selection) => selection,
        ProviderSelectionStoreOutcome::Waiting { exclusions } => {
            return Err(format!(
                "cognitive_realization_target_unavailable:{}",
                exclusions
                    .iter()
                    .map(|entry| format!("{}:{:?}", entry.target_id, entry.code))
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }
    };
    if selection.selected_target_id != target_id {
        return Err("cognitive_realization_exact_target_substitution".to_string());
    }
    let config = ProviderConfig {
        provider_id: target.target_id.clone(),
        base_url: { let endpoint = target.endpoint.trim_end_matches('/');
            if endpoint.ends_with("/v1") { format!("{endpoint}/chat/completions") }
            else { format!("{endpoint}/v1/chat/completions") } },
        model: target.model_id, api_key: target.credential_ref.strip_prefix("env:").and_then(credential),
        language_mode: "auto".into(), continuation_supported: continuation.is_some(),
        continuation_ref: continuation.map(|value| ProviderContinuationReference {
            provider_id: target.target_id, runtime_id: value.runtime_id.clone(), opaque_reference: value.opaque_reference.clone(),
        }),
        governance: Some(ProviderInvocationGovernance {
            selection_id: selection.selection_id.clone(), target_id: selection.selected_target_id.clone(),
            logical_turn_id: selection.logical_turn_id.clone(), attempt_number: selection.attempt_number,
        }),
        governed_locality: Some(target.locality),
        extension_adapter_id: target.extension_adapter_id,
    };
    Ok((config, selection))
}

pub fn governed_attempt_outcome(
    selection: &ProviderSelection,
    error: Option<&str>,
    successful_request_bytes: Option<u64>,
) -> Result<ProviderAttemptOutcome, String> {
    let error_bytes = error
        .and_then(|value| value.split("bytes=").nth(1))
        .and_then(|value| value.split(|ch: char| !ch.is_ascii_digit()).next())
        .and_then(|value| value.parse::<u64>().ok())
        .or_else(|| {
            error
                .and_then(|value| value.split("partial_write:").nth(1))
                .and_then(|value| value.split(':').next())
                .and_then(|value| value.parse::<u64>().ok())
        });
    let error_status = error
        .and_then(|value| value.split("status=").nth(1))
        .and_then(|value| value.split(|ch: char| !ch.is_ascii_digit()).next())
        .and_then(|value| value.parse::<u16>().ok());
    let (delivery, stage, bytes, status, failure) = match error {
        None => (
            ProviderDeliveryClass::ResultReceived,
            ProviderTransportStage::Completed,
            successful_request_bytes.unwrap_or(1),
            Some(200),
            None,
        ),
        Some(error) if error.starts_with("provider_not_dispatched:connect") => (
            ProviderDeliveryClass::NotDispatched,
            ProviderTransportStage::Connect,
            0,
            None,
            Some("connect_failure".to_string()),
        ),
        Some(error) if error.starts_with("provider_not_dispatched:capacity_preflight:")
            || matches!(error, "provider_not_dispatched:token_capacity_exceeded"
                | "provider_not_dispatched:http_body_capacity_exceeded"
                | "provider_not_dispatched:complete_wire_input_budget_exceeded") => (
            ProviderDeliveryClass::NotDispatched,
            ProviderTransportStage::RequestSerialized,
            0,
            None,
            Some("request_capacity_refused".to_string()),
        ),
        Some(error) if error.starts_with("provider_not_dispatched:") => (
            ProviderDeliveryClass::NotDispatched,
            ProviderTransportStage::RequestWriting,
            0,
            None,
            Some("write_before_bytes_failure".to_string()),
        ),
        Some(error) if error.starts_with("provider_remote_response:") => {
            let status = error
                .split(':')
                .nth(1)
                .and_then(|value| value.parse::<u16>().ok());
            (
                ProviderDeliveryClass::DeliveryIndeterminate,
                ProviderTransportStage::ResponseBody,
                error_bytes.unwrap_or(1),
                status,
                Some("remote_response_rejected".to_string()),
            )
        }
        Some(error) if error.starts_with("provider_response_invalid:") => {
            let stage = if error.contains("header") || error.contains("content_length") {
                ProviderTransportStage::ResponseHeaders
            } else if error.contains("body") || error.contains("transfer_encoding") {
                ProviderTransportStage::ResponseBody
            } else {
                ProviderTransportStage::JsonParse
            };
            (
                ProviderDeliveryClass::ResponseInvalid,
                stage,
                error_bytes.unwrap_or(1),
                error_status,
                Some("response_schema_invalid".to_string()),
            )
        }
        Some(error) if error.starts_with("provider_delivery_indeterminate:") => (
            ProviderDeliveryClass::DeliveryIndeterminate,
            ProviderTransportStage::ResponseBody,
            error_bytes.unwrap_or(1),
            None,
            Some("delivery_or_response_unknown".to_string()),
        ),
        Some(_) => (
            ProviderDeliveryClass::NotDispatched,
            ProviderTransportStage::RequestWriting,
            0,
            None,
            Some("local_pre_dispatch_failure".to_string()),
        ),
    };
    ProviderAttemptOutcome::new(
        selection,
        delivery,
        stage,
        bytes,
        status,
        false,
        failure,
        crate::now_unix_ms()?,
    )
}


#[allow(clippy::too_many_arguments)]
pub fn invoke_cognitive(
    home: &std::path::Path,
    authenticated: &yai_core_engine::security::AuthenticatedPrincipal,
    store: &LmdbRecordStore,
    config: &ProviderConfig,
    selection: &ProviderSelection,
    purpose: ProjectionPurpose,
    task: &str,
    output_contract: InvocationOutputContract,
    options: &SemanticInvocationOptions,
    parts: &[ProviderWireInputPart],
) -> Result<ControlledProviderResult, String> {
    let execution = (|| {
    validate_provider_wire_parts(parts)
        .map_err(|error| format!("provider_not_dispatched:local_wire:{error}"))?;
    let semantic = compile_semantic_invocation(home, authenticated, store, &selection.case_id,
        &selection.participant_id, config, purpose, task, output_contract, options)
        .map_err(|error| format!("provider_not_dispatched:local_projection:{error}"))?;
    let key = yai_core_engine::context::stable_digest(&selection.selection_id);
    let invocation_id = format!("invocation:cognitive:{key}");
    let source = |reference: &str| yai_core_engine::transition::TransitionSource {
        component: "yai.provider_boundary".into(), participant_id: Some(selection.participant_id.clone()),
        principal_id: None, source_ref: Some(reference.into()),
    };
    execute_prepared(store, config, &semantic, options, Some(parts), |lineage| {
        let mut pending = yai_core_engine::transition::PendingTransition::new(
            format!("transition:cognitive-invocation:{key}"), &selection.case_id, lineage.case_generation,
            source(&invocation_id), TransitionPayload::ProviderInvocationStarted {
                invocation_id: invocation_id.clone(), participant_id: selection.participant_id.clone(),
                provider_id: config.provider_id.clone(), provider_kind: "openai_compatible".into(),
                model_id: config.model.clone(), semantic_lineage: Some(lineage), governance: config.governance.clone(),
            });
        pending.causal_refs.push(selection.selection_id.clone());
        pending.causal_refs.extend(options.conversation_turn_id.iter().cloned());
        pending.causal_refs.extend(options.workflow_execution_id.iter().cloned());
        let working = semantic.projection.bounds.working_state_id.as_deref().ok_or("cognitive_working_state_missing")?;
        let content = yai_core_engine::conversation::ConversationContentStore::open_existing(home).ok();
        store.commit_cognitive_invocation_authorized(authenticated, pending, working, content.as_ref())?;
        Ok(ProviderInvocationRefs { attempt_id: format!("attempt:cognitive:{key}"), invocation_id: invocation_id.clone() })
    }, |invocation, lineage, output| {
        let state = store.get_case_state(&selection.case_id)?.ok_or("cognitive_case_missing")?;
        let result_id = format!("provider-result:cognitive:{key}");
        let mut pending = yai_core_engine::transition::PendingTransition::new(
            format!("transition:cognitive-result:{key}"), &selection.case_id, state.generation,
            source(&result_id), TransitionPayload::ProviderResultRecorded {
                result_id: result_id.clone(), invocation_id: invocation.invocation_id.clone(),
                provider_id: config.provider_id.clone(), provider_kind: "openai_compatible".into(),
                model_id: config.model.clone(), semantic_lineage: Some(lineage), output: output.into(),
            });
        pending.causal_refs.push(invocation.invocation_id.clone());
        pending.causal_refs.extend(options.workflow_execution_id.iter().cloned());
        store.commit_transition(pending)?;
        Ok(result_id)
    })
    })();
    let outcome = governed_attempt_outcome(selection, execution.as_ref().err().map(String::as_str),
        execution.as_ref().ok().map(|(_, result)| result.request_bytes_written))?;
    store.record_provider_attempt_outcome_authorized(authenticated, &selection.case_id, outcome.clone())?;
    store.record_provider_attempt_health_authorized(authenticated, &selection.case_id, &outcome.outcome_id)?;
    execution.map(|(_, result)| result).map_err(|error|
        format!("cognitive_realization_failed:delivery={:?}:derived_content=false:{error}", outcome.delivery))
}

pub struct ProviderConfig {
    pub provider_id: String,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    pub language_mode: String,
    pub continuation_supported: bool,
    pub continuation_ref: Option<ProviderContinuationReference>,
    pub governance: Option<ProviderInvocationGovernance>,
    pub governed_locality: Option<ProviderLocality>,
    pub extension_adapter_id: Option<String>,
}

pub fn provider_input_observation_id(invocation_id: &str) -> String {
    format!("provider-input:{}", yai_core_engine::context::stable_digest(invocation_id))
}

struct WireObservation {
    digest: String,
    bytes: usize,
    capacity: Option<yai_core_engine::context::ProviderCapacityObservation>,
    refusal: Option<String>,
}
type WireObserver<'a> = Option<&'a dyn Fn(WireObservation) -> Result<(), String>>;


pub struct ProviderTransportResult {
    pub output: String,
    pub response_model_id: Option<String>,
    pub continuation_disposition: ContinuationDisposition,
    pub usage: ProviderUsageTelemetry,
    pub request_bytes_written: u64,
}

#[derive(Debug)]
pub struct DecodedProviderResponse {
    pub output: String,
    pub response_model_id: Option<String>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

#[derive(Clone, Debug, Default)]
pub struct ProviderUsageTelemetry {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub latency_ms: u64,
}

/// Adapter-local wire contracts, not a registry or permission to execute.
/// Application definitions are derived and every requested action must still
/// cross current Case authority independently of this response decoder.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativeFunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativeFunctionCall {
    pub call_id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativeFunctionReply {
    pub text: Option<String>,
    pub call: Option<NativeFunctionCall>,
}

pub fn native_function_tools(
    definitions: &[NativeFunctionDefinition],
) -> Result<serde_json::Value, String> {
    if definitions.is_empty()
        || definitions.len() > 32
        || serde_json::to_vec(definitions)
            .map_err(|e| e.to_string())?
            .len()
            > 32768
    {
        return Err("provider_function_definition_bound".into());
    }
    let mut names = BTreeSet::new();
    let mut tools = Vec::new();
    for definition in definitions {
        if definition.name.is_empty()
            || definition.name.len() > 64
            || !definition
                .name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
            || definition.description.is_empty()
            || definition.description.len() > 1024
            || !names.insert(&definition.name)
            || definition.parameters["type"] != "object"
        {
            return Err("provider_function_definition_invalid".into());
        }
        super::resource_transport::schema_validator(&definition.parameters)
            .map_err(|_| "provider_function_parameter_schema_invalid")?;
        tools.push(serde_json::json!({"type":"function", "function":definition}));
    }
    Ok(serde_json::Value::Array(tools))
}

/// Only the public Chat Completions envelope is decoded as a function request.
/// Prose is never parsed as a call, even if it contains well-formed JSON.
pub fn decode_native_function_reply(
    body: &[u8],
    exact_model: &str,
    definitions: &[NativeFunctionDefinition],
) -> Result<NativeFunctionReply, String> {
    native_function_tools(definitions)?;
    if body.len() > 131072 {
        return Err("provider_function_response_bound".into());
    }
    let value = strict_json(body)?;
    if value["model"] != exact_model || value["choices"].as_array().is_none_or(|v| v.len() != 1) {
        return Err("provider_function_exact_response_mismatch".into());
    }
    let message = &value["choices"][0]["message"];
    if message["role"] != "assistant"
        || message.get("function_call").is_some()
        || message.get("refusal").is_some_and(|v| !v.is_null())
    {
        return Err("provider_function_response_kind_invalid".into());
    }
    let text = match message.get("content") {
        None | Some(serde_json::Value::Null) => None,
        Some(serde_json::Value::String(text)) if text.len() <= 65536 => Some(text.clone()),
        _ => return Err("provider_function_text_shape_invalid".into()),
    };
    let calls = match message.get("tool_calls") {
        None | Some(serde_json::Value::Null) => &[][..],
        Some(serde_json::Value::Array(calls)) if calls.len() <= 1 => calls.as_slice(),
        _ => return Err("provider_function_call_bound".into()),
    };
    if calls.is_empty() {
        if value["choices"][0]["finish_reason"] != "stop"
            || text.as_deref().is_none_or(|v| v.trim().is_empty())
        {
            return Err("provider_function_text_not_complete".into());
        }
        return Ok(NativeFunctionReply { text, call: None });
    }
    if value["choices"][0]["finish_reason"] != "tool_calls" || calls[0]["type"] != "function" {
        return Err("provider_function_call_not_complete".into());
    }
    let call = &calls[0];
    let id = call["id"]
        .as_str()
        .filter(|s| {
            !s.is_empty()
                && s.len() <= 128
                && s.bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"_-:.".contains(&c))
        })
        .ok_or("provider_function_call_id_invalid")?;
    let name = call["function"]["name"]
        .as_str()
        .ok_or("provider_function_name_missing")?;
    let definition = definitions
        .iter()
        .find(|d| d.name == name)
        .ok_or("provider_function_not_offered")?;
    let arguments = call["function"]["arguments"]
        .as_str()
        .filter(|s| s.len() <= 16384)
        .ok_or("provider_function_arguments_shape_invalid")?;
    let arguments = strict_json(arguments.as_bytes())?;
    if !super::resource_transport::schema_validator(&definition.parameters)?.is_valid(&arguments) {
        return Err("provider_function_arguments_contract_mismatch".into());
    }
    Ok(NativeFunctionReply {
        text,
        call: Some(NativeFunctionCall {
            call_id: id.into(),
            name: name.into(),
            arguments,
        }),
    })
}

pub fn decode_provider_response(body: &str) -> Result<DecodedProviderResponse, String> {
    let value: serde_json::Value = serde_json::from_str(body)
        .map_err(|error| format!("provider response was not valid JSON: {error}"))?;
    let message = value
        .pointer("/choices/0/message")
        .ok_or_else(|| "provider response did not contain message".to_string())?;
    if message
        .get("tool_calls")
        .and_then(|value| value.as_array())
        .is_some_and(|calls| !calls.is_empty())
        || message.get("function_call").is_some()
    {
        return Err(
            "provider tool/function call is candidate material for W22 and is not executable"
                .to_string(),
        );
    }
    let output = value
        .pointer("/choices/0/message/content")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "provider response did not contain message content".to_string())?
        .to_string();
    let usage = value.get("usage");
    let response_model_id = value
        .get("model")
        .and_then(|value| value.as_str())
        .map(ToString::to_string);
    let input_tokens = usage.and_then(|usage| {
        usage
            .get("prompt_tokens")
            .or_else(|| usage.get("input_tokens"))
            .and_then(|value| value.as_u64())
    });
    let output_tokens = usage.and_then(|usage| {
        usage
            .get("completion_tokens")
            .or_else(|| usage.get("output_tokens"))
            .and_then(|value| value.as_u64())
    });
    let total_tokens = usage
        .and_then(|usage| usage.get("total_tokens"))
        .and_then(|value| value.as_u64());
    Ok(DecodedProviderResponse {
        output,
        response_model_id,
        input_tokens,
        output_tokens,
        total_tokens,
    })
}

#[derive(Clone, Debug)]
pub struct ProviderWireInputPart {
    pub source_part_id: String,
    pub modality: yai_core_engine::conversation::ContentModality,
    pub media_type: String,
    pub bytes: Vec<u8>,
}

impl ProviderWireInputPart {
    fn validate(&self) -> Result<(), String> {
        use yai_core_engine::conversation::ContentModality;
        if self.source_part_id.is_empty()
            || self.source_part_id.len() > 256
            || self.bytes.is_empty()
            || self.bytes.len() > 1_200_000
        {
            return Err("provider_typed_input_part_size_invalid".to_string());
        }
        match self.modality {
            ContentModality::Text => {
                std::str::from_utf8(&self.bytes)
                    .map_err(|_| "provider_typed_text_invalid_utf8".to_string())?;
            }
            ContentModality::Image if self.media_type == "image/png" => {
                if !self.bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
                    return Err("provider_typed_png_signature_invalid".to_string());
                }
            }
            ContentModality::Audio
                if matches!(self.media_type.as_str(), "audio/wav" | "audio/x-wav") =>
            {
                if self.bytes.len() < 12
                    || &self.bytes[..4] != b"RIFF"
                    || &self.bytes[8..12] != b"WAVE"
                {
                    return Err("provider_typed_wav_signature_invalid".to_string());
                }
            }
            _ => return Err("provider_typed_input_shape_not_supported".to_string()),
        }
        Ok(())
    }

    fn to_openai_content(&self) -> Result<serde_json::Value, String> {
        use yai_core_engine::conversation::ContentModality;
        self.validate()?;
        match self.modality {
            ContentModality::Text => Ok(serde_json::json!({
                "type":"text",
                "text": std::str::from_utf8(&self.bytes)
                    .map_err(|_| "provider_typed_text_invalid_utf8".to_string())?
            })),
            ContentModality::Image if self.media_type == "image/png" => Ok(serde_json::json!({
                "type":"image_url",
                "image_url":{"url":format!("data:{};base64,{}", self.media_type, encode_base64(&self.bytes))}
            })),
            ContentModality::Audio
                if matches!(self.media_type.as_str(), "audio/wav" | "audio/x-wav") =>
            {
                Ok(serde_json::json!({
                    "type":"input_audio",
                    "input_audio":{"data":encode_base64(&self.bytes),"format":"wav"}
                }))
            }
            _ => Err("provider_typed_input_shape_not_supported".to_string()),
        }
    }
}

pub fn validate_provider_wire_parts(parts: &[ProviderWireInputPart]) -> Result<(), String> {
    if parts.is_empty() || parts.len() > 16 {
        return Err("provider_typed_input_part_count_invalid".to_string());
    }
    let mut total = 0usize;
    for part in parts {
        part.validate()?;
        total = total
            .checked_add(part.bytes.len())
            .ok_or_else(|| "provider_typed_input_total_size_invalid".to_string())?;
    }
    if total > 4_000_000 {
        return Err("provider_typed_input_total_size_invalid".to_string());
    }
    Ok(())
}

pub fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        output.push(TABLE[(first >> 2) as usize] as char);
        output.push(TABLE[(((first & 0x03) << 4) | (second >> 4)) as usize] as char);
        output.push(if chunk.len() > 1 {
            TABLE[(((second & 0x0f) << 2) | (third >> 6)) as usize] as char
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            TABLE[(third & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    output
}

pub fn provider_http_request(
    config: &ProviderConfig,
    rendered: &RenderedInput,
    continuation: Option<&ProviderContinuationReference>,
    structured_json: bool,
    typed_parts: Option<&[ProviderWireInputPart]>,
) -> Result<(u16, String, usize), String> {
    provider_http_request_with_functions(
        config,
        rendered,
        continuation,
        structured_json,
        typed_parts,
        None,
        None,
        None,
    )
}

struct NativeFunctionExchange<'a> {
    definitions: &'a [NativeFunctionDefinition],
    feedback: &'a [serde_json::Value],
}

/// Wire lowering, not conversation normalization. Text parts become distinct
/// ordered user messages (including repeated equal text); media retains the
/// ordered content-array representation. Canonical part IDs and provenance
/// remain in the invocation's exact source closure, never recovered from wire.
pub fn append_openai_parts(
    messages: &mut Vec<serde_json::Value>,
    parts: &[serde_json::Value],
) {
    if parts
        .iter()
        .all(|part| part["type"] == "text" && part["text"].is_string())
    {
        messages.extend(
            parts
                .iter()
                .map(|part| serde_json::json!({"role":"user", "content":part["text"]})),
        );
    } else if !parts.is_empty() {
        messages.push(serde_json::json!({"role":"user", "content":parts}));
    }
}

#[allow(clippy::too_many_arguments)]
fn provider_http_request_with_functions(
    config: &ProviderConfig,
    rendered: &RenderedInput,
    continuation: Option<&ProviderContinuationReference>,
    structured_json: bool,
    typed_parts: Option<&[ProviderWireInputPart]>,
    native: Option<NativeFunctionExchange<'_>>,
    max_input_units: Option<usize>,
    observer: WireObserver<'_>,
) -> Result<(u16, String, usize), String> {
    let functions = native.as_ref().map(|n| n.definitions);
    let feedback = native.as_ref().map_or(&[][..], |n| n.feedback);
    let endpoint = crate::provider_transport::parse_provider_endpoint(&config.base_url)?;
    if let Some(reference) = continuation {
        if reference.provider_id != config.provider_id {
            return Err("provider_continuation_provider_mismatch".to_string());
        }
        if !config.continuation_supported {
            return Err("provider_continuation_not_supported".to_string());
        }
    }
    let mut messages =
        vec![serde_json::json!({"role":"system", "content":rendered.system_content})];
    if let Some(parts) = typed_parts {
        let mut content = vec![serde_json::json!({"type":"text", "text":rendered.user_content})];
        for part in parts {
            content.push(part.to_openai_content()?);
        }
        append_openai_parts(&mut messages, &content);
    } else {
        messages.push(serde_json::json!({"role":"user", "content":rendered.user_content}));
    }
    let mut body = serde_json::json!({
        "model": config.model,
        "stream": false,
        "messages": messages
    });
    let object = body.as_object_mut().expect("provider request object");
    if !feedback.is_empty() {
        if functions.is_none() {
            return Err("provider_feedback_requires_native_functions".into());
        }
        object
            .get_mut("messages")
            .and_then(serde_json::Value::as_array_mut)
            .expect("provider messages")
            .extend_from_slice(feedback);
    }
    if let Some(functions) = functions {
        if structured_json {
            return Err("provider_output_contract_conflict".into());
        }
        object.insert("tools".into(), native_function_tools(functions)?);
        object.insert("parallel_tool_calls".into(), serde_json::Value::Bool(false));
        object.insert("tool_choice".into(), serde_json::json!("auto"));
    }
    if let Some(reference) = continuation {
        object.insert(
            "yai_provider_continuation".to_string(),
            serde_json::json!({
                "runtime_id": reference.runtime_id,
                "reference": reference.opaque_reference
            }),
        );
    }
    if structured_json {
        object.insert(
            "response_format".to_string(),
            serde_json::json!({"type": "json_object"}),
        );
    }
    let body = serde_json::to_vec(&body)
        .map_err(|error| format!("provider_request_encode_failed: {error}"))?;
    let assessment = if max_input_units.is_some_and(|limit| body.len().div_ceil(4) > limit) {
        Ok(preflight::Assessment {capacity: None, refusal: Some("complete_wire_input_budget_exceeded")})
    } else { preflight::assess(config, &endpoint, &body) };
    let (capacity, refusal) = match assessment {
        Ok(value) => (value.capacity, value.refusal.map(str::to_string)),
        Err(error) => (None, Some(format!("capacity_preflight:{error}"))),
    };
    if let Some(observe) = observer {
        observe(WireObservation {digest: yai_core_engine::effect::digest_bytes(&body), bytes: body.len(),
            capacity, refusal: refusal.clone()})
            .map_err(|_| "provider_not_dispatched:input_observation_unavailable")?;
    }
    if let Some(refusal) = refusal { return Err(format!("provider_not_dispatched:{refusal}")); }
    let response = crate::provider_transport::provider_http(
        &endpoint,
        config.governed_locality.as_ref(),
        "POST",
        &endpoint.path,
        &body,
        config.api_key.as_deref(),
    )?;
    Ok((
        response.status,
        String::from_utf8(response.body).map_err(|_| {
            format!(
                "provider_response_invalid:body_utf8:status={}:bytes={}",
                response.status, response.request_bytes_written
            )
        })?,
        response.request_bytes_written,
    ))
}

pub fn provider_chat_completion(
    store: &LmdbRecordStore,
    config: &ProviderConfig,
    rendered: &RenderedInput,
    structured_json: bool,
    typed_parts: Option<&[ProviderWireInputPart]>,
    output_contract: &InvocationOutputContract,
    max_input_units: Option<usize>,
) -> Result<ProviderTransportResult, String> {
    provider_chat_completion_observed(store, config, rendered, structured_json, typed_parts,
        output_contract, max_input_units, None)
}

#[allow(clippy::too_many_arguments)]
fn provider_chat_completion_observed(
    store: &LmdbRecordStore,
    config: &ProviderConfig,
    rendered: &RenderedInput,
    structured_json: bool,
    typed_parts: Option<&[ProviderWireInputPart]>,
    output_contract: &InvocationOutputContract,
    max_input_units: Option<usize>,
    observer: WireObserver<'_>,
) -> Result<ProviderTransportResult, String> {
    let started = Instant::now();
    let continuation = config.continuation_ref.as_ref();
    let offered = match output_contract {
        InvocationOutputContract::CaseCapabilities { view, catalogs, .. } => {
            Some(capabilities::offer(view, catalogs)?)
        }
        _ => None,
    };
    let definitions = offered.as_ref().map(|offered| {
        offered
            .iter()
            .map(|o| o.definition.clone())
            .collect::<Vec<_>>()
    });
    let (status, body_text, request_bytes_written) = if let Some(definitions) = &definitions {
        let feedback = if let InvocationOutputContract::CaseCapabilities {
            view,
            feedback_result_ids,
            ..
        } = output_contract
        {
            let history =
                store.list_case_transitions(&view.case_id)?;
            capabilities::feedback(view, feedback_result_ids, &history)?
        } else {
            Vec::new()
        };
        provider_http_request_with_functions(
            config,
            rendered,
            continuation,
            structured_json,
            typed_parts,
            Some(NativeFunctionExchange {
                definitions,
                feedback: &feedback,
            }),
            max_input_units,
            observer,
        )?
    } else {
        provider_http_request_with_functions(config, rendered, continuation, structured_json, typed_parts, None, max_input_units, observer)?
    };
    let success = (200..300).contains(&status);
    let disposition = if success {
        if continuation.is_some() {
            ContinuationDisposition::Used
        } else {
            ContinuationDisposition::NotProvided
        }
    } else {
        let rejection = public_error_code(body_text.as_bytes());
        return Err(format!(
            "provider_remote_response:{status}:bytes={request_bytes_written}:provider_code={rejection}"
        ));
    };
    let decoded = if let (InvocationOutputContract::CaseCapabilities { view, .. }, Some(offered)) =
        (output_contract, offered)
    {
        capabilities::decode(&body_text, &config.model, view, &offered).map(|output| {
            DecodedProviderResponse {
                output,
                response_model_id: Some(config.model.clone()),
                input_tokens: None,
                output_tokens: None,
                total_tokens: None,
            }
        })
    } else {
        decode_provider_response(&body_text)
    }
    .map_err(|error| {
        format!("provider_response_invalid:status={status}:bytes={request_bytes_written}:{error}")
    })?;
    if typed_parts.is_some() && decoded.response_model_id.as_deref() != Some(config.model.as_str())
    {
        return Err(format!(
            "provider_response_invalid:status={status}:bytes={request_bytes_written}:exact_model_mismatch"
        ));
    }
    Ok(ProviderTransportResult {
        output: decoded.output,
        response_model_id: decoded.response_model_id,
        continuation_disposition: disposition,
        usage: ProviderUsageTelemetry {
            input_tokens: decoded.input_tokens,
            output_tokens: decoded.output_tokens,
            total_tokens: decoded.total_tokens,
            latency_ms: started.elapsed().as_millis().try_into().unwrap_or(u64::MAX),
        },
        request_bytes_written: request_bytes_written.try_into().unwrap_or(u64::MAX),
    })
}


pub struct SemanticInvocation {
    pub projection: Projection,
    pub residency: ResidencyPlan,
    pub frame: ContextFrame,
    pub rendered: RenderedInput,
    pub output_contract_id: String,
}

#[derive(Clone, Debug)]
pub struct SemanticInvocationOptions {
    pub max_resident_items: usize,
    pub max_semantic_units: usize,
    pub max_estimated_input_units: usize,
    pub retrieval_limit: usize,
    pub previous_item_ids: Vec<String>,
    pub workflow_execution_id: Option<String>,
    /// Canonical user Turn that causally precedes this execution. This is an
    /// application-content reference, not a provider routing instruction.
    pub conversation_turn_id: Option<String>,
}

impl Default for SemanticInvocationOptions {
    fn default() -> Self {
        Self {
            // W3 accounts for typed Recall/source closure as well as current
            // control. Explicit Workflow/work budgets still override these
            // bounded conversation defaults; no automatic target-capacity bump.
            max_resident_items: 64,
            max_semantic_units: 32_768,
            max_estimated_input_units: 65_536,
            retrieval_limit: DEFAULT_RETRIEVAL_LIMIT,
            previous_item_ids: Vec::new(),
            workflow_execution_id: None,
            conversation_turn_id: None,
        }
    }
}


pub fn compile_semantic_invocation(
    home: &std::path::Path,
    auth: &yai_core_engine::security::AuthenticatedPrincipal,
    store: &LmdbRecordStore,
    case_ref: &str,
    participant_ref: &str,
    provider: &ProviderConfig,
    purpose: ProjectionPurpose,
    task: &str,
    output_contract: InvocationOutputContract,
    options: &SemanticInvocationOptions,
) -> Result<SemanticInvocation, String> {
    let is_memory_consolidation = purpose == ProjectionPurpose::MemoryConsolidation;
    let state = store
        .get_case_state(case_ref)?
        .ok_or_else(|| format!("canonical CaseState missing for {}", case_ref))?;
    let transitions = store.list_case_transitions(case_ref)?;
    let mut request = ProjectionRequest::model(participant_ref, purpose);
    if let Some(governance) = &provider.governance {
        let selection_is_canonical = state.provider_selections.iter().any(|selection| {
            selection.selection_id == governance.selection_id
                && selection.selected_target_id == governance.target_id
                && selection.logical_turn_id == governance.logical_turn_id
                && selection.attempt_number == governance.attempt_number
                && selection.participant_id == participant_ref
        });
        if !selection_is_canonical {
            return Err("provider_selection_projection_admission_invalid".to_string());
        }
        let native_selected = transitions.iter().any(|t|matches!(&t.payload,
            TransitionPayload::ProviderSelectionRecorded {selection} if selection.selection_id == governance.selection_id
                && t.causal_refs.contains(&"provider-realization-shape:text_functions_to_text_or_call".to_string())));
        if native_selected
            != matches!(
                &output_contract,
                InvocationOutputContract::CaseCapabilities { .. }
            )
        {
            return Err("provider_selected_output_contract_mismatch".into());
        }
    }
    request.max_items = options.max_resident_items;
    request.max_provider_claims = if is_memory_consolidation { 0 } else { 6 };
    request.max_interaction_turns = if is_memory_consolidation { 0 } else { 8 };
    let resource_refs = match &output_contract {
        InvocationOutputContract::CaseCapabilities {
            view,
            catalogs,
            feedback_result_ids,
        } => {
            if view.case_id != state.case_id || view.participant_id != participant_ref {
                return Err("provider_capability_view_scope_mismatch".into());
            }
            let selection_id = provider
                .governance
                .as_ref()
                .ok_or("provider_capability_governance_required")?
                .selection_id
                .as_str();
            let selection = transitions.iter().find(|t| matches!(&t.payload,
                TransitionPayload::ProviderSelectionRecorded {selection} if selection.selection_id == selection_id
            )).ok_or("provider_capability_selection_missing")?;
            if !selection.causal_refs.contains(&view.view_id)
                || !selection.causal_refs.contains(
                    &"provider-realization-shape:text_functions_to_text_or_call".to_string(),
                )
            {
                return Err("provider_capability_view_not_selected".into());
            }
            // Validate schemas and catalog provenance before InvocationStarted.
            capabilities::offer(view, catalogs)?;
            capabilities::feedback(view, feedback_result_ids, &transitions)?;
            for catalog in catalogs {
                if !transitions.iter().any(|t|matches!(&t.payload,
                    TransitionPayload::ResourceObservationRecorded {observation} if observation == catalog)) {
                    return Err("provider_capability_catalog_not_canonical".into());
                }
            }
            view.entries
                .iter()
                .map(|entry| entry.resource.attachment_id.clone())
                .collect()
        }
        InvocationOutputContract::FilesystemWriteProposal { attachment_id, .. }
        | InvocationOutputContract::ProcessSignalProposal { attachment_id, .. }
        | InvocationOutputContract::CaseRuntimeTurn { attachment_id, .. } => {
            vec![attachment_id.clone()]
        }
        InvocationOutputContract::NaturalLanguage
        | InvocationOutputContract::WorkflowPlanPatch { .. }
        | InvocationOutputContract::MemoryConsolidation { .. } => Vec::new(),
    };
    let output_contract_id = output_contract.contract_id();
    let profile = ProviderModelProfile {
        provider_id: provider.provider_id.clone(),
        provider_kind: "openai_compatible".to_string(),
        model_id: provider.model.clone(),
        structured_output_supported: matches!(
            &output_contract,
            InvocationOutputContract::CaseCapabilities { .. }
                | InvocationOutputContract::FilesystemWriteProposal { .. }
                | InvocationOutputContract::ProcessSignalProposal { .. }
                | InvocationOutputContract::CaseRuntimeTurn { .. }
                | InvocationOutputContract::WorkflowPlanPatch { .. }
                | InvocationOutputContract::MemoryConsolidation { .. }
        ),
        continuation_supported: provider.continuation_supported,
    };
    let content =
        yai_core_engine::conversation::ConversationContentStore::open_existing(home).ok();
    let recall_query = if let Some(turn_id) = &options.conversation_turn_id {
        let turn = transitions
            .iter()
            .find_map(|t| match &t.payload {
                TransitionPayload::ConversationTurnCommitted { turn }
                    if &turn.turn_id == turn_id => Some(turn),
                _ => None,
            })
            .ok_or("working_execution_turn_unavailable")?;
        yai_core_engine::conversation::authorize_turn_execution(
            &state, &transitions, turn, participant_ref, &auth.projected_principal_id(),
        )?;
        let text = turn.ordered_parts.iter()
            .filter_map(|p| p.object.inline_text.as_deref())
            .collect::<Vec<_>>().join("\n");
        if text.trim().is_empty() { None } else { Some(text) }
    } else {
        None
    };
    let compilation = yai_core_engine::semantic_state::CompilationRequest {
        scope: request,
        intent: task.to_string(),
        output_contract_id: output_contract_id.clone(),
        max_semantic_units: options.max_semantic_units,
        max_derived_items: options.retrieval_limit,
        resource_refs,
        required_refs: options.conversation_turn_id.iter().cloned().collect(),
        previous_item_ids: options.previous_item_ids.clone(),
        view_selection_id: provider
            .governance
            .as_ref()
            .map(|g| g.selection_id.clone()),
    };
    // Deliberate compatibility contracts, never an error fallback: historical
    // ProviderAttached pins and the explicit W20 consolidation operation keep
    // their established S-only interpretation. Normal governed Conversation /
    // Workflow execution always takes current Recall-aware compilation below.
    let legacy_pinned = provider.governance.is_none()
        && state.provider_binding.is_none()
        && state.provider.is_some();
    let (working, mut projection) = if legacy_pinned || is_memory_consolidation {
        let source = store.compose_cognitive_state(&state, &transitions)?;
        let working = source.compile(&compilation)?;
        let projection = working.lower_context(&source, &compilation)?;
        (working, projection)
    } else {
        let qualified = store.compile_working_state_authorized(
            &auth,
            yai_core_engine::semantic_state::WorkingStateRequest {
                case_id: state.case_id.clone(),
                expected_generation: state.generation,
                compilation,
                recall_query,
                at: None,
                recall_required_refs: vec![],
                recall_bounds: yai_core_engine::memory_hierarchy::recall::RecallBounds {
                    // Discovery's qualified closure may use the execution
                    // envelope; W still reserves mandatory current state and
                    // atomically omits optional evidence to fit that envelope.
                    semantic_units: options.max_semantic_units,
                    ..Default::default()
                },
                max_output_bytes: 4 * 1024 * 1024,
            },
            content.as_ref(),
        )?;
        let projection = qualified.lower_context()?;
        (qualified.working_state, projection)
    };
    let residency = working.residency_report(
        &projection,
        profile.provider_id.clone(),
        profile.model_id.clone(),
    )?;
    projection.bounds.residency_plan_id = Some(residency.plan_id.clone());
    yai_core_engine::context::refresh_projection_identity(&mut projection)?;
    let frame = build_context_frame(&projection, task, output_contract)?;
    let rendered = render_openai_compatible(&frame, &profile, &provider.language_mode)?;
    let estimated_input_units = rendered.metadata.content_chars.div_ceil(4);
    if estimated_input_units > options.max_estimated_input_units {
        return Err(format!(
            "provider_input_budget_exceeded: estimated_units={estimated_input_units} max_units={}",
            options.max_estimated_input_units
        ));
    }
    store.put_semantic_context_artifact(&SemanticContextArtifact::WorkingState(working))?;
    store
        .put_semantic_context_artifact(&SemanticContextArtifact::Projection(projection.clone()))?;
    store.put_semantic_context_artifact(&SemanticContextArtifact::ContextFrame(frame.clone()))?;
    store.put_semantic_context_artifact(&SemanticContextArtifact::RenderedInputMetadata(
        rendered.metadata.clone(),
    ))?;
    store.put_semantic_context_artifact(&SemanticContextArtifact::ResidencyPlan(
        residency.clone(),
    ))?;
    Ok(SemanticInvocation {
        projection,
        residency,
        frame,
        rendered,
        output_contract_id,
    })
}


struct StrictValue(Value);

impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StrictVisitor;
        impl<'de> Visitor<'de> for StrictVisitor {
            type Value = StrictValue;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("JSON without duplicate object keys")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Bool(value)))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Number(value.into())))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Number(value.into())))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                serde_json::Number::from_f64(value)
                    .map(|number| StrictValue(Value::Number(number)))
                    .ok_or_else(|| E::custom("non-finite JSON number"))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::String(value.to_string())))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::String(value)))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Null))
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Null))
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = Vec::new();
                while let Some(value) = sequence.next_element::<StrictValue>()? {
                    values.push(value.0);
                }
                Ok(StrictValue(Value::Array(values)))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut values = Map::new();
                let mut keys = HashSet::new();
                while let Some(key) = map.next_key::<String>()? {
                    if !keys.insert(key.clone()) {
                        return Err(serde::de::Error::custom(format!(
                            "duplicate JSON key: {key}"
                        )));
                    }
                    let value = map.next_value::<StrictValue>()?;
                    values.insert(key, value.0);
                }
                Ok(StrictValue(Value::Object(values)))
            }
        }
        deserializer.deserialize_any(StrictVisitor)
    }
}

pub fn strict_json(body: &[u8]) -> Result<Value, String> {
    let mut deserializer = serde_json::Deserializer::from_slice(body);
    let value = StrictValue::deserialize(&mut deserializer)
        .map_err(|error| format!("provider_response_json_invalid: {error}"))?;
    deserializer
        .end()
        .map_err(|error| format!("provider_response_json_trailing_data: {error}"))?;
    Ok(value.0)
}


pub fn public_error_code(body: &[u8]) -> String {
    let code = strict_json(body).ok().and_then(|v| {
        v.pointer("/error/code")
            .or_else(|| v.pointer("/error/type"))
            .and_then(Value::as_str)
            .map(str::to_owned)
    });
    code.filter(|s| {
        s.len() <= 64
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || "_-".contains(c))
    })
    .unwrap_or_else(|| "provider_rejected_request".into())
}

/// Ephemeral public catalog, not semantic evidence or target admission. The
/// application caller must authorize metadata access before entering here.
pub fn discover_provider_models(
    endpoint: &str,
    locality: &yai_core_engine::provider_governance::ProviderLocality,
    credential_ref: &str,
    credential: impl Fn(&str) -> Option<String>,
) -> Result<Vec<String>, String> {
    let endpoint =
        yai_core_engine::provider_governance::normalize_provider_endpoint(endpoint, locality)?;
    let endpoint = crate::provider_transport::parse_provider_endpoint(&endpoint)?;
    let credential = if credential_ref == "none" {
        None
    } else {
        let key = credential_ref
            .strip_prefix("env:")
            .filter(|key| {
                !key.is_empty()
                    && key.len() <= 128
                    && key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
            })
            .ok_or("provider_credential_reference_invalid")?;
        Some(credential(key).ok_or("provider_credential_unavailable")?)
    };
    let response = crate::provider_transport::provider_http(
        &endpoint,
        Some(locality),
        "GET",
        &endpoint.api_path("models"),
        &[],
        credential.as_deref(),
    )?;
    if matches!(response.status, 401 | 403) {
        return Err("provider_catalog_auth_required".into());
    }
    if response.status != 200 {
        return Err(format!(
            "provider_catalog_http_{}:{}",
            response.status,
            public_error_code(&response.body)
        ));
    }
    catalog_models(&response.body)
}

pub fn catalog_models(body: &[u8]) -> Result<Vec<String>, String> {
    let value = strict_json(body).map_err(|_| "provider_catalog_invalid")?;
    let rows = value
        .get("data")
        .and_then(Value::as_array)
        .ok_or("provider_catalog_invalid")?;
    if rows.len() > 128
        || value.get("has_more").and_then(Value::as_bool) == Some(true)
        || value.get("next").is_some_and(|next| !next.is_null())
    {
        return Err("provider_catalog_incomplete_or_over_bound".into());
    }
    let mut models = Vec::new();
    for row in rows {
        let id = row
            .get("id")
            .and_then(Value::as_str)
            .ok_or("provider_catalog_invalid")?;
        if id.is_empty()
            || id.len() > yai_core_engine::provider_governance::MAX_PROVIDER_MODEL_ID_BYTES
            || !id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "._:-/".contains(c))
            || models.iter().any(|known| known == id)
        {
            return Err("provider_catalog_invalid_or_duplicate_model".into());
        }
        models.push(id.to_owned());
    }
    models.sort();
    if models.is_empty() {
        return Err("provider_catalog_empty: endpoint exposes no models; no qualification or binding performed".into());
    }
    Ok(models)
}
