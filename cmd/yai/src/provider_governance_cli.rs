//! Product/admin adapters for Tenant-scoped provider governance.

use super::*;
use serde::de::{Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};
use yai_core_engine::provider_governance::{
    ProviderAdapterKind, ProviderFailoverPolicy, ProviderLocality, ProviderProbeEvidence,
    ProviderRealizationShape, ProviderTargetInput, ProviderTrustPosture,
};
use yai_core_engine::security::AuthenticatedPrincipal;

const QUALIFICATION_SUITE: &str = "yai.openai_compatible.synthetic.v1";
const EMBEDDING_QUALIFICATION_SUITE: &str = "yai.openai_compatible.embedding.synthetic.v1";
const REALIZATION_QUALIFICATION_SUITE: &str = "yai.openai_compatible.typed_content.synthetic.v2";

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
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

fn authenticated_store() -> Result<(AuthenticatedPrincipal, LmdbRecordStore), String> {
    let authenticated = security::authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    Ok((authenticated, store))
}

fn provider_target_id_by_key(
    authenticated: &AuthenticatedPrincipal,
    store: &LmdbRecordStore,
    tenant_id: &str,
    provider_key: &str,
) -> Result<String, String> {
    let mut matches = store
        .list_provider_targets_authorized(authenticated, tenant_id)?
        .into_iter()
        .filter(|target| target.provider_key == provider_key)
        .map(|target| target.target_id)
        .collect::<Vec<_>>();
    matches.sort();
    match matches.as_slice() {
        [] => Err("provider_key_not_found".to_string()),
        [target_id] => Ok(target_id.clone()),
        _ => Err("provider_key_ambiguous_use_exact_target".to_string()),
    }
}

fn provider_target_id_from_args(
    args: &[String],
    authenticated: &AuthenticatedPrincipal,
    store: &LmdbRecordStore,
) -> Result<String, String> {
    let exact = optional_arg(args, "--target");
    let tenant = optional_arg(args, "--tenant");
    let provider_key = optional_arg(args, "--provider-key");
    match (exact, tenant, provider_key) {
        (Some(target_id), None, None) => Ok(target_id),
        (None, Some(tenant_id), Some(provider_key)) => {
            provider_target_id_by_key(authenticated, store, &tenant_id, &provider_key)
        }
        (None, None, None) => Err(
            "provider_reference_required: use TARGET or --tenant TENANT --provider-key KEY"
                .to_string(),
        ),
        (Some(_), _, _) => Err("provider_reference_conflict".to_string()),
        (None, _, _) => Err("provider_key_reference_requires_tenant_and_provider_key".to_string()),
    }
}

fn parse_locality(value: &str) -> Result<ProviderLocality, String> {
    match value {
        "loopback" => Ok(ProviderLocality::Loopback),
        "private_network" => Ok(ProviderLocality::PrivateNetwork),
        "remote" => Ok(ProviderLocality::Remote),
        _ => Err("provider_locality_invalid".to_string()),
    }
}

fn parse_failover(value: &str) -> Result<ProviderFailoverPolicy, String> {
    match value {
        "none" => Ok(ProviderFailoverPolicy::None),
        "safe_only" => Ok(ProviderFailoverPolicy::SafeOnly),
        _ => Err("provider_failover_policy_invalid".to_string()),
    }
}

fn credential_ref_from_args(args: &[String]) -> Result<String, String> {
    let exact = optional_arg(args, "--credential-ref");
    let optional_env = optional_arg(args, "--credential-env");
    match (exact, optional_env) {
        (Some(reference), None) => Ok(reference),
        (None, Some(name)) => {
            if name.is_empty()
                || name.len() > 128
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                || name.as_bytes()[0].is_ascii_digit()
            {
                return Err("provider_credential_env_name_invalid".to_string());
            }
            if std::env::var(&name).is_ok_and(|value| !value.trim().is_empty()) {
                Ok(format!("env:{name}"))
            } else {
                Ok("none".to_string())
            }
        }
        (None, None) => Ok("none".to_string()),
        (Some(_), Some(_)) => Err("provider_credential_reference_conflict".to_string()),
    }
}

fn provider_add(args: &[String]) -> Result<(), String> {
    let (authenticated, store) = authenticated_store()?;
    let tenant_id = named_arg(args, "--tenant")?;
    let target = store.register_provider_target_authorized(
        &authenticated,
        ProviderTargetInput {
            tenant_id,
            provider_key: named_arg(args, "--provider-key")?,
            adapter: ProviderAdapterKind::OpenAiCompatible,
            endpoint: named_arg(args, "--endpoint")?,
            model_id: named_arg(args, "--model")?,
            credential_ref: credential_ref_from_args(args)?,
            locality: parse_locality(&named_arg(args, "--locality")?)?,
            extension_adapter_id: optional_arg(args, "--extension-adapter"),
            created_by_principal_id: authenticated.projected_principal_id(),
            created_at_unix_ms: now_ms(),
        },
    )?;
    println!("provider_target: registered");
    println!("target_id: {}", target.target_id);
    println!("tenant_id: {}", target.tenant_id);
    println!("provider_key: {}", target.provider_key);
    println!("adapter: open_ai_compatible");
    println!("endpoint: {}", target.endpoint);
    println!("model_id: {}", target.model_id);
    println!("credential_ref: {}", target.credential_ref);
    println!("locality: {:?}", target.locality);
    println!("integrity_digest: {}", target.integrity_digest);
    Ok(())
}

fn provider_list(args: &[String]) -> Result<(), String> {
    let (authenticated, store) = authenticated_store()?;
    let tenant_id = named_arg(args, "--tenant")?;
    let targets = store.list_provider_targets_authorized(&authenticated, &tenant_id)?;
    println!("provider_targets: {}", targets.len());
    for target in targets {
        println!(
            "target: {} provider_key:{} model:{} endpoint:{} locality:{:?}",
            target.target_id,
            target.provider_key,
            target.model_id,
            target.endpoint,
            target.locality
        );
    }
    Ok(())
}

fn provider_show(args: &[String]) -> Result<(), String> {
    let (authenticated, store) = authenticated_store()?;
    let target_id = provider_target_id_from_args(args, &authenticated, &store)?;
    let (target, qualification, trust, health) =
        store.provider_posture_authorized(&authenticated, &target_id)?;
    println!("provider_target: {}", target.target_id);
    println!("configuration_provider_key: {}", target.provider_key);
    println!("configuration_adapter: open_ai_compatible");
    println!("configuration_endpoint: {}", target.endpoint);
    println!("configuration_model: {}", target.model_id);
    println!("configuration_locality: {:?}", target.locality);
    println!("configuration_credential_ref: {}", target.credential_ref);
    let credential_revision =
        store.provider_credential_revision_authorized(&authenticated, &target_id)?;
    println!(
        "configuration_credential_revision: {}",
        credential_revision
            .as_ref()
            .map_or(0, |value| value.sequence)
    );
    println!(
        "qualification: {}",
        qualification.as_ref().map_or("missing", |qualification| {
            if (qualification.evidence.chat_text_envelope_valid
                || qualification.evidence.text_embedding_envelope_valid)
                && qualification.evidence.exact_model_addressed
            {
                "qualified"
            } else {
                "evidence_failed"
            }
        })
    );
    if let Some(qualification) = qualification {
        println!("qualification_id: {}", qualification.qualification_id);
        println!("qualification_run_id: {}", qualification.run_id);
        println!(
            "qualification_capabilities: {}",
            qualification
                .capabilities
                .iter()
                .map(|value| format!("{:?}@{:?}", value.capability, value.provenance))
                .collect::<Vec<_>>()
                .join(",")
        );
        println!(
            "qualification_time_unix_ms: {}",
            qualification.qualified_at_unix_ms
        );
        println!(
            "realization_shapes: {}",
            qualification
                .evidence
                .realization_shapes
                .iter()
                .map(ProviderRealizationShape::as_str)
                .collect::<Vec<_>>()
                .join(",")
        );
        println!(
            "failure_codes: {}",
            qualification.evidence.failure_codes.join(",")
        );
        println!(
            "probe_elapsed_ms: {}",
            qualification
                .evidence
                .completed_at_unix_ms
                .saturating_sub(qualification.evidence.started_at_unix_ms)
        );
    }
    println!(
        "governance: {:?}",
        trust
            .as_ref()
            .map(|value| value.posture.clone())
            .unwrap_or(ProviderTrustPosture::Unreviewed)
    );
    if let Some(trust) = trust {
        println!("governance_event_id: {}", trust.event_id);
        println!("governance_principal: {}", trust.principal_id);
    }
    println!("health: {:?}", health.effective_posture(now_ms()));
    println!("health_observed: {:?}", health.posture);
    println!("health_circuit: {:?}", health.circuit_at(now_ms()));
    println!("health_source: {}", health.source);
    println!("dimensions_collapsed: false");
    Ok(())
}

#[derive(Clone, Debug)]
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

pub(super) fn strict_json(body: &[u8]) -> Result<Value, String> {
    let mut deserializer = serde_json::Deserializer::from_slice(body);
    let value = StrictValue::deserialize(&mut deserializer)
        .map_err(|error| format!("provider_response_json_invalid: {error}"))?;
    deserializer
        .end()
        .map_err(|error| format!("provider_response_json_trailing_data: {error}"))?;
    Ok(value.0)
}

type ParsedEndpoint = super::provider_transport::ProviderEndpoint;

fn parse_http_endpoint(endpoint: &str) -> Result<ParsedEndpoint, String> {
    super::provider_transport::parse_provider_endpoint(endpoint)
}

fn api_path(endpoint: &ParsedEndpoint, suffix: &str) -> String {
    endpoint.api_path(suffix)
}

struct ProbeHttpResponse {
    status: u16,
    body: Vec<u8>,
}

fn probe_http(
    endpoint: &ParsedEndpoint,
    locality: &ProviderLocality,
    method: &str,
    path: &str,
    body: Option<&[u8]>,
    api_key: Option<&str>,
) -> Result<ProbeHttpResponse, String> {
    let mut request = body.and_then(|bytes| serde_json::from_slice::<Value>(bytes).ok());
    let stage = if path.ends_with("models") {
        "catalog"
    } else if request.as_ref().is_some_and(|v| v["tool_choice"] == "none") {
        "function_result"
    } else if request.as_ref().is_some_and(|v| v.get("tools").is_some()) {
        "function_call"
    } else if request
        .as_ref()
        .is_some_and(|v| v.get("response_format").is_some())
    {
        "json"
    } else {
        "text_or_typed_content"
    };
    if let Some(Value::Object(ref mut object)) = request {
        if path.ends_with("chat/completions") {
            object.insert("max_tokens".into(), Value::from(96));
        }
    }
    let encoded = request
        .as_ref()
        .map(|v| serde_json::to_vec(v).expect("probe JSON"));
    let started = std::time::Instant::now();
    eprintln!("provider_probe: {stage} started (synthetic input; no Case data)");
    let response = std::thread::scope(|scope| {
        let (stop, signal) = std::sync::mpsc::channel::<()>();
        scope.spawn(move || {
            while signal.recv_timeout(std::time::Duration::from_secs(5))
                == Err(std::sync::mpsc::RecvTimeoutError::Timeout)
            {
                eprintln!(
                    "provider_probe: {stage} waiting for provider, elapsed={}s",
                    started.elapsed().as_secs()
                );
            }
        });
        let result = super::provider_transport::provider_http(
            endpoint,
            Some(locality),
            method,
            path,
            encoded.as_deref().or(body).unwrap_or_default(),
            api_key,
        );
        drop(stop);
        result
    });
    match &response {
        Ok(value) => eprintln!(
            "provider_probe: {stage} HTTP {} elapsed={}ms",
            value.status,
            started.elapsed().as_millis()
        ),
        Err(error) => eprintln!(
            "provider_probe: {stage} {} elapsed={}ms",
            probe_failure_code(error),
            started.elapsed().as_millis()
        ),
    }
    let response = response?;
    Ok(ProbeHttpResponse {
        status: response.status,
        body: response.body,
    })
}

fn probe_failure_code(error: &str) -> String {
    // Stable, bounded diagnostics; never copy model prose/HTTP error messages
    // or credential bytes into evidence. Internal error category and subtype
    // carry timeout/delivery/normalization distinctions lost by the old probe.
    error
        .split(':')
        .take(2)
        .collect::<Vec<_>>()
        .join(":")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || "._:-/".contains(*c))
        .take(128)
        .collect()
}

pub(super) fn public_error_code(body: &[u8]) -> String {
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

fn probe_status_error(response: &ProbeHttpResponse) -> String {
    format!(
        "http_{}:{}",
        response.status,
        public_error_code(&response.body)
    )
}

fn credential_for(target: &yai_core_engine::provider_governance::ProviderTarget) -> Option<String> {
    target
        .credential_ref
        .strip_prefix("env:")
        .and_then(provider::env_var)
}

fn embedding_probe_shape(value: &Value, model_id: &str) -> (Option<u64>, bool) {
    let dimension = value
        .pointer("/data/0/embedding")
        .and_then(Value::as_array)
        .and_then(|values| {
            (!values.is_empty()
                && values.len() <= 4096
                && values
                    .iter()
                    .all(|value| value.as_f64().is_some_and(f64::is_finite)))
            .then_some(values.len() as u64)
        });
    let exact_response_model = value.get("model").and_then(Value::as_str) == Some(model_id);
    (dimension, exact_response_model)
}

fn probe_native_function_roundtrip(
    target: &yai_core_engine::provider_governance::ProviderTarget,
    endpoint: &ParsedEndpoint,
    api_key: Option<&str>,
) -> Result<(), String> {
    let definitions = [provider::NativeFunctionDefinition {
        name: "yai_contract_echo".into(),
        description: "Synthetic contract probe; no external operation.".into(),
        parameters: serde_json::json!({"type":"object", "properties":{"value":{"type":"string", "enum":["yai-contract"]}}, "required":["value"], "additionalProperties":false}),
    }];
    let tools = provider::native_function_tools(&definitions)?;
    let messages = serde_json::json!([
        {"role":"system", "content":"Synthetic YAI native function contract qualification. No Case data or external tools."},
        {"role":"user", "content":"Call yai_contract_echo with value yai-contract. After its result return exactly that returned value."}
    ]);
    let request = serde_json::json!({"model":target.model_id, "stream":false, "parallel_tool_calls":false,
        "messages":messages, "tools":tools, "tool_choice":{"type":"function", "function":{"name":"yai_contract_echo"}}});
    let response = probe_http(
        endpoint,
        &target.locality,
        "POST",
        &api_path(endpoint, "chat/completions"),
        Some(&serde_json::to_vec(&request).map_err(|e| e.to_string())?),
        api_key,
    )?;
    if response.status != 200 {
        return Err(probe_status_error(&response));
    }
    let reply =
        provider::decode_native_function_reply(&response.body, &target.model_id, &definitions)?;
    let call = reply
        .call
        .ok_or("provider_function_probe_native_call_required")?;
    // This local synthetic echo carries no operation authority. The second
    // response must consume the correlated result, not merely emit any text.
    let echoed = format!("yai-result-{}", now_ms());
    let mut messages = messages.as_array().unwrap().clone();
    messages.push(serde_json::json!({"role":"assistant", "content":reply.text, "tool_calls":[{
        "id":call.call_id, "type":"function", "function":{"name":call.name, "arguments":serde_json::to_string(&call.arguments).map_err(|e| e.to_string())?}}]}));
    messages
        .push(serde_json::json!({"role":"tool", "tool_call_id":call.call_id, "content":echoed}));
    let request = serde_json::json!({"model":target.model_id, "stream":false, "parallel_tool_calls":false,
        "messages":messages, "tools":tools, "tool_choice":"none"});
    let response = probe_http(
        endpoint,
        &target.locality,
        "POST",
        &api_path(endpoint, "chat/completions"),
        Some(&serde_json::to_vec(&request).map_err(|e| e.to_string())?),
        api_key,
    )?;
    if response.status != 200 {
        return Err(probe_status_error(&response));
    }
    let reply =
        provider::decode_native_function_reply(&response.body, &target.model_id, &definitions)?;
    if reply.call.is_some() || reply.text.as_deref().map(str::trim) != Some(echoed.as_str()) {
        return Err("provider_function_probe_correlated_result_not_consumed".into());
    }
    Ok(())
}

fn run_synthetic_probe(
    target: &yai_core_engine::provider_governance::ProviderTarget,
    probe_embedding: bool,
    requested_shapes: &[ProviderRealizationShape],
) -> ProviderProbeEvidence {
    let started = now_ms();
    let run_id = format!(
        "qualification-run:{}:{started}",
        yai_core_engine::context::stable_digest(&target.target_id)
    );
    let mut evidence = ProviderProbeEvidence {
        run_id,
        target_id: target.target_id.clone(),
        started_at_unix_ms: started,
        completed_at_unix_ms: started,
        transport_connected: false,
        exact_model_addressed: false,
        chat_text_envelope_valid: false,
        structured_json_object_valid: false,
        usage_accounting_observed: false,
        health_endpoint_observed: false,
        extension_telemetry_observed: false,
        text_embedding_envelope_valid: false,
        embedding_dimension: None,
        realization_shapes: Vec::new(),
        failure_codes: Vec::new(),
    };
    let endpoint = match parse_http_endpoint(&target.endpoint) {
        Ok(value) => value,
        Err(error) => {
            evidence.failure_codes.push(error);
            evidence.completed_at_unix_ms = now_ms();
            return evidence;
        }
    };
    let api_key = credential_for(target);
    if target.credential_ref != "none" && api_key.is_none() {
        evidence
            .failure_codes
            .push("credential_missing".to_string());
        evidence.completed_at_unix_ms = now_ms();
        return evidence;
    }

    match probe_http(
        &endpoint,
        &target.locality,
        "GET",
        &api_path(&endpoint, "models"),
        None,
        api_key.as_deref(),
    ) {
        Ok(response) => {
            evidence.transport_connected = true;
            if (200..300).contains(&response.status) {
                match strict_json(&response.body) {
                    Ok(value) => {
                        evidence.exact_model_addressed = value
                            .get("data")
                            .and_then(Value::as_array)
                            .is_some_and(|models| {
                                models.iter().any(|model| {
                                    model.get("id").and_then(Value::as_str)
                                        == Some(target.model_id.as_str())
                                })
                            });
                    }
                    Err(_) => evidence
                        .failure_codes
                        .push("models_response_invalid".to_string()),
                }
            }
        }
        Err(error) => evidence.failure_codes.push(probe_failure_code(&error)),
    }

    if probe_embedding {
        let embedding_body = serde_json::to_vec(&serde_json::json!({
            "model": target.model_id,
            "input": ["Synthetic YAI embedding contract probe. No Case data."],
            "encoding_format": "float"
        }))
        .expect("synthetic embedding probe serializes");
        match probe_http(
            &endpoint,
            &target.locality,
            "POST",
            &api_path(&endpoint, "embeddings"),
            Some(&embedding_body),
            api_key.as_deref(),
        ) {
            Ok(response) if (200..300).contains(&response.status) => {
                match strict_json(&response.body) {
                    Ok(value) => {
                        let (dimension, exact_response_model) =
                            embedding_probe_shape(&value, &target.model_id);
                        evidence.text_embedding_envelope_valid = dimension.is_some();
                        evidence.embedding_dimension = dimension;
                        // Exact catalog membership alone is insufficient: the
                        // embedding response must bind itself to the same model.
                        evidence.exact_model_addressed &= exact_response_model;
                        if !evidence.text_embedding_envelope_valid {
                            evidence
                                .failure_codes
                                .push("embedding_response_invalid".to_string());
                        }
                    }
                    Err(_) => evidence
                        .failure_codes
                        .push("embedding_response_invalid".to_string()),
                }
            }
            Ok(response) => evidence
                .failure_codes
                .push(format!("embedding_http_{}", response.status)),
            Err(error) => evidence.failure_codes.push(probe_failure_code(&error)),
        }
        evidence.completed_at_unix_ms = now_ms().max(evidence.started_at_unix_ms);
        evidence.failure_codes.sort();
        evidence.failure_codes.dedup();
        evidence.failure_codes.truncate(16);
        return evidence;
    }

    let text_body = serde_json::to_vec(&serde_json::json!({
        "model": target.model_id,
        "stream": false,
        "messages": [
            {"role": "system", "content": "Synthetic YAI provider contract probe. No Case data."},
            {"role": "user", "content": "Return exactly YAI_OK."}
        ]
    }))
    .expect("synthetic probe serializes");
    match probe_http(
        &endpoint,
        &target.locality,
        "POST",
        &api_path(&endpoint, "chat/completions"),
        Some(&text_body),
        api_key.as_deref(),
    ) {
        Ok(response) => {
            evidence.transport_connected = true;
            if (200..300).contains(&response.status) {
                match strict_json(&response.body) {
                    Ok(value) => {
                        evidence.chat_text_envelope_valid = value
                            .pointer("/choices/0/message/content")
                            .and_then(Value::as_str)
                            .is_some();
                        evidence.exact_model_addressed |=
                            value.get("model").and_then(Value::as_str)
                                == Some(target.model_id.as_str());
                        evidence.usage_accounting_observed = value.get("usage").is_some();
                        evidence.extension_telemetry_observed =
                            target.extension_adapter_id.as_deref() == Some("yvex.http.v1")
                                && value.get("yvex_completion_metrics").is_some();
                    }
                    Err(_) => evidence
                        .failure_codes
                        .push("chat_response_invalid".to_string()),
                }
            } else {
                evidence
                    .failure_codes
                    .push(format!("chat_http_{}", response.status));
            }
        }
        Err(error) => evidence.failure_codes.push(probe_failure_code(&error)),
    }

    let json_body = serde_json::to_vec(&serde_json::json!({
        "model": target.model_id,
        "stream": false,
        "response_format": {"type": "json_object"},
        "messages": [
            {"role": "system", "content": "Synthetic YAI provider JSON contract probe. No Case data."},
            {"role": "user", "content": "Return a JSON object with ok=true."}
        ]
    }))
    .expect("synthetic JSON probe serializes");
    if requested_shapes.is_empty()
        || requested_shapes.contains(&ProviderRealizationShape::TextToJsonObject)
    {
        if let Ok(response) = probe_http(
            &endpoint,
            &target.locality,
            "POST",
            &api_path(&endpoint, "chat/completions"),
            Some(&json_body),
            api_key.as_deref(),
        ) {
            if (200..300).contains(&response.status) {
                if let Ok(value) = strict_json(&response.body) {
                    evidence.structured_json_object_valid = value
                        .pointer("/choices/0/message/content")
                        .and_then(Value::as_str)
                        .and_then(|content| strict_json(content.as_bytes()).ok())
                        .is_some_and(|content| content.is_object());
                }
            }
        }
    }
    if target.extension_adapter_id.as_deref() == Some("yvex.http.v1") {
        if let Ok(response) = probe_http(
            &endpoint,
            &target.locality,
            "GET",
            "/health",
            None,
            api_key.as_deref(),
        ) {
            evidence.health_endpoint_observed =
                (200..300).contains(&response.status) && strict_json(&response.body).is_ok();
        }
    }
    for shape in requested_shapes {
        if *shape == ProviderRealizationShape::TextFunctionsToTextOrCall {
            match probe_native_function_roundtrip(target, &endpoint, api_key.as_deref()) {
                Ok(()) => evidence.realization_shapes.push(shape.clone()),
                Err(error) => {
                    evidence
                        .failure_codes
                        .push("native_function_roundtrip_invalid".into());
                    evidence.failure_codes.push(probe_failure_code(&error));
                }
            }
            continue;
        }
        let content = match shape {
            ProviderRealizationShape::TextFunctionsToTextOrCall => {
                unreachable!("function shape qualified separately")
            }
            ProviderRealizationShape::TextToText => Some(serde_json::json!([
                {"type":"text","text":"Synthetic YAI ordered text-part wire probe."},
                {"type":"text","text":"Synthetic YAI ordered text-part wire probe."},
                {"type":"text","text":"Return exactly YAI_OK."}
            ])),
            ProviderRealizationShape::TextToJsonObject => Some(serde_json::json!([
                {"type":"text","text":"Synthetic YAI ordered text-part JSON wire probe. Return a JSON object with probe=true."}
            ])),
            ProviderRealizationShape::AudioWavToText => Some(serde_json::json!([
                {"type":"text","text":"Synthetic YAI audio-to-text wire probe. Return text."},
                {"type":"input_audio","input_audio":{"data":"UklGRiQAAABXQVZFZm10IBAAAAABAAEAQB8AAEAfAAABAAgAZGF0YQAAAAA=","format":"wav"}}
            ])),
            ProviderRealizationShape::OrderedPngTextToText => Some(serde_json::json!([
                {"type":"text","text":"Synthetic YAI ordered image/text wire probe."},
                {"type":"image_url","image_url":{"url":"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII="}},
                {"type":"image_url","image_url":{"url":"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII="}},
                {"type":"text","text":"Preserve the declared order and return text."}
            ])),
        };
        let valid = if let Some(content) = content {
            let mut messages = vec![
                serde_json::json!({"role":"system","content":"Synthetic YAI typed-content contract probe. No Case data."}),
            ];
            provider::append_openai_parts(
                &mut messages,
                content.as_array().expect("typed probe parts"),
            );
            let mut body = serde_json::json!({
                "model": target.model_id,
                "stream": false,
                "messages": messages
            });
            if *shape == ProviderRealizationShape::TextToJsonObject {
                body["response_format"] = serde_json::json!({"type":"json_object"});
            }
            let body = serde_json::to_vec(&body).expect("synthetic typed-content probe serializes");
            let response = probe_http(
                &endpoint,
                &target.locality,
                "POST",
                &api_path(&endpoint, "chat/completions"),
                Some(&body),
                api_key.as_deref(),
            );
            match &response {
                Err(error) => evidence.failure_codes.push(probe_failure_code(error)),
                Ok(response) if !(200..300).contains(&response.status) => {
                    evidence.failure_codes.push(probe_status_error(response))
                }
                _ => {}
            }
            response
                .ok()
                .filter(|response| (200..300).contains(&response.status))
                .and_then(|response| strict_json(&response.body).ok())
                .is_some_and(|value| {
                    value
                        .pointer("/choices/0/message/content")
                        .and_then(Value::as_str)
                        .is_some_and(|content| {
                            *shape != ProviderRealizationShape::TextToJsonObject
                                || strict_json(content.as_bytes()).is_ok_and(|v| v.is_object())
                        })
                        && value.get("model").and_then(Value::as_str)
                            == Some(target.model_id.as_str())
                })
        } else {
            false
        };
        if valid {
            evidence.realization_shapes.push(shape.clone());
        } else {
            evidence
                .failure_codes
                .push(format!("realization_shape_{}_invalid", shape.as_str()));
        }
    }
    evidence.realization_shapes.sort();
    evidence.realization_shapes.dedup();
    evidence.completed_at_unix_ms = now_ms().max(evidence.started_at_unix_ms);
    evidence.failure_codes.sort();
    evidence.failure_codes.dedup();
    evidence.failure_codes.truncate(16);
    evidence
}

fn print_probe(evidence: &ProviderProbeEvidence) {
    println!("provider_probe: completed");
    println!("run_id: {}", evidence.run_id);
    println!("target_id: {}", evidence.target_id);
    println!("transport_connected: {}", evidence.transport_connected);
    println!("model_exact_addressing: {}", evidence.exact_model_addressed);
    println!("chat_text: {}", evidence.chat_text_envelope_valid);
    println!(
        "structured_json_object: {}",
        evidence.structured_json_object_valid
    );
    println!("usage_accounting: {}", evidence.usage_accounting_observed);
    println!("text_embedding: {}", evidence.text_embedding_envelope_valid);
    println!(
        "embedding_dimension: {}",
        evidence
            .embedding_dimension
            .map_or_else(|| "none".to_string(), |value| value.to_string())
    );
    println!("health_probe: {}", evidence.health_endpoint_observed);
    println!(
        "extension_compatible_telemetry: {}",
        evidence.extension_telemetry_observed
    );
    println!(
        "realization_shapes: {}",
        evidence
            .realization_shapes
            .iter()
            .map(ProviderRealizationShape::as_str)
            .collect::<Vec<_>>()
            .join(",")
    );
    println!("synthetic_input_only: true");
    println!("failure_codes: {}", evidence.failure_codes.join(","));
}

fn provider_probe(args: &[String], persist_qualification: bool) -> Result<(), String> {
    let (authenticated, store) = authenticated_store()?;
    let target_id = provider_target_id_from_args(args, &authenticated, &store)?;
    let (target, _, _, _) = store.provider_posture_authorized(&authenticated, &target_id)?;
    let probe_embedding = args.iter().any(|value| value == "--embedding");
    let requested_shapes = repeated_arg(args, "--realization-shape")
        .iter()
        .map(|value| ProviderRealizationShape::parse(value))
        .collect::<Result<Vec<_>, _>>()?;
    if probe_embedding && !requested_shapes.is_empty() {
        return Err("provider_embedding_and_realization_probe_conflict".to_string());
    }
    let admission_token = format!("probe-admission:{}:{}", std::process::id(), now_ms());
    let probe_owner = store.begin_provider_probe_authorized(
        &authenticated,
        &target.target_id,
        &admission_token,
    )?;
    let evidence = run_synthetic_probe(&target, probe_embedding, &requested_shapes);
    store.complete_provider_probe_authorized(
        &authenticated,
        &target.target_id,
        &probe_owner,
        &evidence,
    )?;
    print_probe(&evidence);
    if persist_qualification {
        let valid_until = optional_arg(args, "--valid-for-ms")
            .map(|value| {
                value
                    .parse::<u64>()
                    .map(|duration| evidence.completed_at_unix_ms.saturating_add(duration))
                    .map_err(|_| "provider_qualification_valid_for_invalid".to_string())
            })
            .transpose()?;
        let qualification = store.qualify_provider_target_authorized(
            &authenticated,
            &target.target_id,
            evidence,
            if probe_embedding {
                EMBEDDING_QUALIFICATION_SUITE
            } else if !requested_shapes.is_empty() {
                REALIZATION_QUALIFICATION_SUITE
            } else {
                QUALIFICATION_SUITE
            },
            valid_until,
        )?;
        println!("qualification: recorded");
        println!("qualification_id: {}", qualification.qualification_id);
        println!("qualification_suite: {}", qualification.suite_id);
        println!(
            "qualified_capabilities: {}",
            qualification
                .capabilities
                .iter()
                .map(|capability| format!("{:?}", capability.capability))
                .collect::<Vec<_>>()
                .join(",")
        );
    }
    Ok(())
}

/// Product onboarding uses the same synthetic probes and provider evidence
/// owner as the Advanced qualification command. It sends no Case material.
pub(super) fn qualify_case_work_target(
    store: &LmdbRecordStore,
    authenticated: &AuthenticatedPrincipal,
    target: &yai_core_engine::provider_governance::ProviderTarget,
    case_work: bool,
) -> Result<yai_core_engine::provider_governance::ProviderQualification, String> {
    let token = format!("probe-admission:{}:{}", std::process::id(), now_ms());
    let owner = store.begin_provider_probe_authorized(authenticated, &target.target_id, &token)?;
    let shapes = if case_work {
        vec![
            ProviderRealizationShape::TextToText,
            ProviderRealizationShape::TextFunctionsToTextOrCall,
            ProviderRealizationShape::TextToJsonObject,
        ]
    } else {
        vec![ProviderRealizationShape::TextToText]
    };
    let evidence = run_synthetic_probe(target, false, &shapes);
    store.complete_provider_probe_authorized(
        authenticated,
        &target.target_id,
        &owner,
        &evidence,
    )?;
    let qualified = store.qualify_provider_target_authorized(
        authenticated,
        &target.target_id,
        evidence,
        REALIZATION_QUALIFICATION_SUITE,
        None,
    )?;
    if !shapes
        .iter()
        .all(|shape| qualified.supports_realization_shape(shape))
    {
        return Err(format!("case_connect_mechanical_contract_unqualified: target={} qualification={}; required={}; proven={}; failures={}; no trust or Case binding added", target.target_id, qualified.qualification_id,
            shapes.iter().map(ProviderRealizationShape::as_str).collect::<Vec<_>>().join(","),
            qualified.evidence.realization_shapes.iter().map(ProviderRealizationShape::as_str).collect::<Vec<_>>().join(","),
            qualified.evidence.failure_codes.join(",")));
    }
    Ok(qualified)
}

fn provider_trust(args: &[String], posture: ProviderTrustPosture) -> Result<(), String> {
    let (authenticated, store) = authenticated_store()?;
    let target_id = provider_target_id_from_args(args, &authenticated, &store)?;
    let event =
        store.set_provider_trust_authorized(&authenticated, &target_id, posture, now_ms())?;
    println!("provider_trust: recorded");
    println!("event_id: {}", event.event_id);
    println!("target_id: {}", event.target_id);
    println!("posture: {:?}", event.posture);
    println!("sequence: {}", event.sequence);
    println!("principal_id: {}", event.principal_id);
    Ok(())
}

fn provider_credential_rotate(args: &[String]) -> Result<(), String> {
    let revision_label = named_arg(args, "--revision")?;
    let (authenticated, store) = authenticated_store()?;
    let target_id = provider_target_id_from_args(args, &authenticated, &store)?;
    let revision =
        store.rotate_provider_credential_authorized(&authenticated, &target_id, &revision_label)?;
    println!("provider_credential_rotation: recorded");
    println!("revision_id: {}", revision.revision_id);
    println!("target_id: {}", revision.target_id);
    println!("credential_revision: {}", revision.sequence);
    println!("revision_label: {}", revision.revision_label);
    println!("secret_persisted: false");
    Ok(())
}

fn case_provider_bind(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let participant_id = named_arg(args, "--participant")?;
    let exact_targets = repeated_arg(args, "--target");
    let provider_keys = repeated_arg(args, "--provider-key");
    let failover = parse_failover(
        &optional_arg(args, "--failover").unwrap_or_else(|| "safe_only".to_string()),
    )?;
    let max_attempts = optional_arg(args, "--max-attempts")
        .unwrap_or_else(|| "3".to_string())
        .parse::<u32>()
        .map_err(|_| "provider_max_attempts_invalid".to_string())?;
    let (authenticated, store) = authenticated_store()?;
    if !exact_targets.is_empty() && !provider_keys.is_empty() {
        return Err("case_provider_reference_families_conflict".to_string());
    }
    let targets = if provider_keys.is_empty() {
        if exact_targets.is_empty() {
            return Err("case_provider_target_or_provider_key_required".to_string());
        }
        exact_targets
    } else {
        let state = store.get_case_state_authorized(&authenticated, &case_id)?;
        let tenant_id = state
            .tenant_id
            .as_deref()
            .ok_or_else(|| "case_provider_key_requires_tenant_scoped_case".to_string())?;
        provider_keys
            .iter()
            .map(|key| provider_target_id_by_key(&authenticated, &store, tenant_id, key))
            .collect::<Result<Vec<_>, _>>()?
    };
    let binding = store.bind_case_provider_targets_authorized(
        &authenticated,
        &case_id,
        &participant_id,
        targets,
        failover,
        max_attempts,
    )?;
    println!("case_provider_binding: recorded");
    println!("binding_id: {}", binding.binding_id);
    println!("case_id: {}", binding.case_id);
    println!("participant_id: {}", binding.participant_id);
    println!("targets: {}", binding.ordered_target_ids.join(","));
    println!("failover_policy: {:?}", binding.failover_policy);
    println!("max_attempts_per_turn: {}", binding.max_attempts_per_turn);
    Ok(())
}

fn case_provider_show(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let (authenticated, store) = authenticated_store()?;
    let state = store.get_case_state_authorized(&authenticated, &case_id)?;
    println!("case_id: {}", case_id);
    if let Some(binding) = state.provider_binding {
        println!("provider_mode: governed_pool");
        println!("binding_id: {}", binding.binding_id);
        println!("participant_id: {}", binding.participant_id);
        println!("candidate_count: {}", binding.ordered_target_ids.len());
        println!("targets: {}", binding.ordered_target_ids.join(","));
        println!("failover_policy: {:?}", binding.failover_policy);
        if let Some(selection) = state.provider_selections.last() {
            println!("last_selection_id: {}", selection.selection_id);
            println!("last_selected_target: {}", selection.selected_target_id);
            println!("last_selected_model: {}", selection.selected_model_id);
        } else {
            println!("last_selection_id: none");
        }
        if let Some(outcome) = state.provider_attempt_outcomes.last() {
            println!("last_attempt_posture: {:?}", outcome.delivery);
            println!("delivery_indeterminate: {}", matches!(outcome.delivery, yai_core_engine::provider_governance::ProviderDeliveryClass::DeliveryIndeterminate));
        }
    } else if let Some(provider) = state.provider {
        println!("provider_mode: legacy_exact_pin");
        println!("participant_id: {}", provider.participant_id);
        println!("provider_id: {}", provider.provider_id);
        println!("endpoint: {}", provider.base_url);
        println!("model_id: {}", provider.model_id);
    } else {
        println!("provider_mode: unconfigured");
    }
    Ok(())
}

pub(super) fn provider_governance_command(
    operation_id: &str,
    args: &[String],
) -> Result<(), String> {
    match operation_id {
        "yai.provider.add" => provider_add(args),
        "yai.provider.list" => provider_list(args),
        "yai.provider.show" => provider_show(args),
        "yai.provider.probe" => provider_probe(args, false),
        "yai.provider.qualify" => provider_probe(args, true),
        "yai.provider.trust.approve" => provider_trust(args, ProviderTrustPosture::Approved),
        "yai.provider.trust.deny" => provider_trust(args, ProviderTrustPosture::Denied),
        "yai.provider.credential.rotate" => provider_credential_rotate(args),
        "yai.case.provider.bind" => case_provider_bind(args),
        "yai.case.provider.show" => case_provider_show(args),
        _ => Err(format!(
            "unsupported provider governance operation: {operation_id}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_function_probe_requires_real_correlated_roundtrip() {
        use std::io::{BufRead, BufReader, Read, Write};
        use std::net::TcpListener;
        for correct_result in [true, false] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            let peer = std::thread::spawn(move || {
                for step in 0..2 {
                    let (mut stream, _) = listener.accept().unwrap();
                    stream
                        .set_read_timeout(Some(std::time::Duration::from_secs(3)))
                        .unwrap();
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut first = String::new();
                    reader.read_line(&mut first).unwrap();
                    assert!(first.starts_with("POST /v1/chat/completions "), "{first}");
                    let mut length = None;
                    loop {
                        let mut line = String::new();
                        reader.read_line(&mut line).unwrap();
                        if line == "\r\n" {
                            break;
                        }
                        if let Some((key, value)) = line.split_once(':') {
                            if key.eq_ignore_ascii_case("content-length") {
                                length = Some(value.trim().parse::<usize>().unwrap());
                            }
                        }
                    }
                    let mut bytes = vec![0; length.unwrap()];
                    reader.read_exact(&mut bytes).unwrap();
                    let request: Value = serde_json::from_slice(&bytes).unwrap();
                    assert_eq!(request["model"], "whisper-vision-tools-only-a-name");
                    assert_eq!(request["parallel_tool_calls"], false);
                    assert_eq!(request["tools"][0]["function"]["name"], "yai_contract_echo");
                    let (finish, message) = if step == 0 {
                        assert_eq!(
                            request["tool_choice"]["function"]["name"],
                            "yai_contract_echo"
                        );
                        (
                            "tool_calls",
                            serde_json::json!({"role":"assistant", "content":null, "tool_calls":[{
                            "id":"call_exact", "type":"function", "function":{"name":"yai_contract_echo", "arguments":"{\"value\":\"yai-contract\"}"}}]}),
                        )
                    } else {
                        assert_eq!(request["tool_choice"], "none");
                        assert_eq!(request["messages"][2]["tool_calls"][0]["id"], "call_exact");
                        assert_eq!(request["messages"][3]["role"], "tool");
                        assert_eq!(request["messages"][3]["tool_call_id"], "call_exact");
                        let text = if correct_result {
                            request["messages"][3]["content"].clone()
                        } else {
                            serde_json::json!("unrelated text cannot qualify result consumption")
                        };
                        (
                            "stop",
                            serde_json::json!({"role":"assistant", "content":text}),
                        )
                    };
                    let body = serde_json::to_vec(
                        &serde_json::json!({"model":"whisper-vision-tools-only-a-name",
                        "choices":[{"finish_reason":finish, "message":message}]}),
                    )
                    .unwrap();
                    write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
                    stream.write_all(&body).unwrap();
                }
                2
            });
            let target = yai_core_engine::provider_governance::ProviderTarget::from_input(
                ProviderTargetInput {
                    tenant_id: "tenant:function-probe".into(),
                    provider_key: "deepseek-not-semantic-evidence".into(),
                    adapter: ProviderAdapterKind::OpenAiCompatible,
                    endpoint: format!("http://{address}/v1"),
                    model_id: "whisper-vision-tools-only-a-name".into(),
                    credential_ref: "none".into(),
                    locality: ProviderLocality::Loopback,
                    extension_adapter_id: None,
                    created_by_principal_id: "principal:test".into(),
                    created_at_unix_ms: 1,
                },
            )
            .unwrap();
            let result = probe_native_function_roundtrip(
                &target,
                &parse_http_endpoint(&target.endpoint).unwrap(),
                None,
            );
            assert_eq!(peer.join().unwrap(), 2);
            if correct_result {
                result.unwrap();
            } else {
                assert!(result.unwrap_err().contains("not_consumed"));
            }
        }
        println!("native_function_contract: provider=loopback_fixture actual_http_requests=4 correlated_result=required no_external_operation=true no_yvex=true");
    }

    #[test]
    fn strict_json_rejects_duplicate_keys() {
        assert!(strict_json(br#"{"ok":true}"#).is_ok());
        assert!(strict_json(br#"{"ok":true,"ok":false}"#)
            .unwrap_err()
            .contains("duplicate JSON key"));
        assert_eq!(
            probe_failure_code("provider_delivery_indeterminate:response_deadline:bytes=123"),
            "provider_delivery_indeterminate:response_deadline"
        );
        let rejected = ProbeHttpResponse {
            status: 400,
            body:
                br#"{"error":{"code":"invalid_request","message":"secret text must not persist"}}"#
                    .to_vec(),
        };
        assert_eq!(probe_status_error(&rejected), "http_400:invalid_request");
    }

    #[test]
    fn text_wire_lowering_preserves_distinct_order_without_claiming_media() {
        let mut messages = vec![serde_json::json!({"role":"system","content":"context"})];
        provider::append_openai_parts(
            &mut messages,
            &[
                serde_json::json!({"type":"text","text":"same"}),
                serde_json::json!({"type":"text","text":"same"}),
                serde_json::json!({"type":"text","text":"last"}),
            ],
        );
        assert_eq!(messages.len(), 4);
        assert_eq!(messages[1]["content"], "same");
        assert_eq!(messages[2]["content"], "same");
        assert_eq!(messages[3]["content"], "last");
        let media = vec![
            serde_json::json!({"type":"text","text":"first"}),
            serde_json::json!({"type":"image_url","image_url":{"url":"data:image/png;base64,AA=="}}),
        ];
        provider::append_openai_parts(&mut messages, &media);
        assert_eq!(messages[4]["content"], serde_json::json!(media));
    }

    #[test]
    fn embedding_probe_requires_exact_response_model_and_finite_bounded_vector() {
        let exact = serde_json::json!({
            "model": "encoder:exact",
            "data": [{"embedding": [1.0, 2.0]}]
        });
        assert_eq!(
            embedding_probe_shape(&exact, "encoder:exact"),
            (Some(2), true)
        );
        assert_eq!(
            embedding_probe_shape(&exact, "encoder:other"),
            (Some(2), false)
        );
        let missing = serde_json::json!({"data": [{"embedding": [1.0, 2.0]}]});
        assert_eq!(
            embedding_probe_shape(&missing, "encoder:exact"),
            (Some(2), false)
        );
        let invalid = serde_json::json!({
            "model": "encoder:exact",
            "data": [{"embedding": []}]
        });
        assert_eq!(
            embedding_probe_shape(&invalid, "encoder:exact"),
            (None, true)
        );
    }
}
