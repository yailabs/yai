//! Shared bounded synthetic provider qualification. No Case material or trust is supplied here.
//! CLI and typed Application carriers retain governance and durable submission ownership.
use super::{strict_json, public_error_code, catalog_models};
use crate::provider_execution as provider;
use serde_json::Value;
use yai_core_engine::provider_governance::{ProviderLocality, ProviderProbeEvidence, ProviderRealizationShape};
#[cfg(test)]
use yai_core_engine::provider_governance::{ProviderTargetInput, ProviderAdapterKind};
use std::time::{SystemTime, UNIX_EPOCH};

pub const QUALIFICATION_SUITE: &str = "yai.openai_compatible.synthetic.v1";
pub const EMBEDDING_QUALIFICATION_SUITE: &str = "yai.openai_compatible.embedding.synthetic.v1";
pub const REALIZATION_QUALIFICATION_SUITE: &str = "yai.openai_compatible.typed_content.synthetic.v2";

use yai_core_engine::security::AuthenticatedPrincipal;
use yai_core_engine::store::lmdb::LmdbRecordStore;
use yai_core_engine::provider_governance::{ProviderProbeRequest};
use std::path::{Path, PathBuf};

fn carrier_path(home: &Path, target: &str, submission: &str) -> PathBuf {
    home.join("run/provider-probes").join(format!("{}.lock",
        yai_core_engine::context::stable_digest(&format!("{target}\0{submission}"))))
}

pub fn observe(home: &Path, store: &LmdbRecordStore, auth: &AuthenticatedPrincipal,
    target: &str, submission: &str, created: bool) -> Result<crate::ProviderProbeExecution, String> {
    let first = store.provider_probe_run_authorized(auth, target, submission)?;
    let running = crate::runtime_execution::execution_carrier_running(&carrier_path(home, target, submission))?;
    // Recheck current authority and terminal completion after the liveness read.
    let run = store.provider_probe_run_authorized(auth, target, submission)?;
    let posture = if run.failure_code.is_some() { "failed" }
        else if run.evidence.is_some() { "completed" }
        else if running && first.owner == run.owner { "running" } else { "interrupted" };
    Ok(crate::ProviderProbeExecution { schema: "yai.provider_probe_execution.v1".into(),
        target_ref: target.into(), submission_ref: submission.into(), created, posture: posture.into(), run })
}

pub fn submit(home: &Path, store: &LmdbRecordStore, auth: &AuthenticatedPrincipal,
    request: ProviderProbeRequest) -> Result<crate::ProviderProbeExecution, String> {
    request.validate()?;
    // Resolve Tenant owner authority before touching operational lock files.
    let (target, _, _, _) = store.provider_posture_authorized(auth, &request.target_id)?;
    store.resolve_security_context(auth, &target.tenant_id)?.require_owner()?;
    match store.provider_probe_run_authorized(auth, &request.target_id, &request.submission_ref) {
        Ok(_) => { let (_, created) = store.begin_provider_probe_run_authorized(auth, request.clone())?;
            return observe(home, store, auth, &request.target_id, &request.submission_ref, created); }
        Err(error) if error == "provider_probe_run_not_found" => {}
        Err(error) => return Err(error),
    }
    let lock = crate::runtime_execution::acquire_execution_carrier(&carrier_path(home, &request.target_id, &request.submission_ref))?;
    let permit = crate::runtime_execution::admit_application_carrier()?;
    let (run, created) = store.begin_provider_probe_run_authorized(auth, request.clone())?;
    if !created { drop(lock); drop(permit); return observe(home, store, auth, &request.target_id, &request.submission_ref, false); }
    let carrier_home = home.to_path_buf(); let carrier_auth = auth.clone(); let carrier_run = run.clone();
    let spawned = std::thread::Builder::new().name("yai-provider-probe".into()).spawn(move || {
        let _lock = lock; let _permit = permit;
        let work = || -> Result<(), String> {
            let store = LmdbRecordStore::open(carrier_home.join("store/lmdb"))?;
            let (target, _, _, _) = store.provider_posture_authorized(&carrier_auth, &carrier_run.request.target_id)?;
            store.resolve_security_context(&carrier_auth, &target.tenant_id)?.require_owner()?;
            let credential = target.credential_ref.strip_prefix("env:")
                .and_then(|name| super::credential_from_profile(&carrier_home, name));
            let mut evidence = run_synthetic_probe(&target, carrier_run.request.embedding,
                &carrier_run.request.realization_shapes, credential);
            evidence.run_id = carrier_run.request.submission_ref.clone();
            let suite = if carrier_run.request.embedding { EMBEDDING_QUALIFICATION_SUITE }
                else if carrier_run.request.realization_shapes.is_empty() { QUALIFICATION_SUITE }
                else { REALIZATION_QUALIFICATION_SUITE };
            store.complete_provider_probe_run_authorized(&carrier_auth, &target.target_id,
                &carrier_run.request.submission_ref, &carrier_run.owner, evidence, suite)?;
            Ok(())
        };
        if !matches!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)), Ok(Ok(()))) {
            if let Ok(store) = LmdbRecordStore::open(carrier_home.join("store/lmdb")) {
                let _ = store.fail_provider_probe_run_authorized(&carrier_auth, &carrier_run.request.target_id,
                    &carrier_run.request.submission_ref, &carrier_run.owner);
            }
        }
    });
    if spawned.is_err() {
        store.fail_provider_probe_run_authorized(auth, &request.target_id, &request.submission_ref, &run.owner)?;
    }
    observe(home, store, auth, &request.target_id, &request.submission_ref, true)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

type ParsedEndpoint = crate::provider_transport::ProviderEndpoint;

fn parse_http_endpoint(endpoint: &str) -> Result<ParsedEndpoint, String> {
    crate::provider_transport::parse_provider_endpoint(endpoint)
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
        "model catalog"
    } else if request.as_ref().is_some_and(|v| v["tool_choice"] == "none") {
        "function result consumption"
    } else if request.as_ref().is_some_and(|v| v.get("tools").is_some()) {
        "native function call"
    } else if request
        .as_ref()
        .is_some_and(|v| v.get("response_format").is_some())
    {
        "JSON response"
    } else {
        "text/content response"
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
    eprintln!("\n[YAI connection]\n  Checking {stage} with synthetic input, not Case data.\n[/YAI connection]");
    let response = std::thread::scope(|scope| {
        let (stop, signal) = std::sync::mpsc::channel::<()>();
        scope.spawn(move || {
            while signal.recv_timeout(std::time::Duration::from_secs(5))
                == Err(std::sync::mpsc::RecvTimeoutError::Timeout)
            {
                eprintln!(
                    "\n[YAI connection]\n  Waiting for {stage} ({}s).\n[/YAI connection]",
                    started.elapsed().as_secs()
                );
            }
        });
        let result = crate::provider_transport::provider_http(
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
            "\n[YAI connection]\n  Received {stage}: HTTP {}, {}ms.\n[/YAI connection]",
            value.status,
            started.elapsed().as_millis()
        ),
        Err(_) => eprintln!(
            "\n[YAI connection]\n  Could not complete {stage} after {}ms. Exact failure retained in qualification evidence.\n[/YAI connection]",
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

fn probe_status_error(response: &ProbeHttpResponse) -> String {
    format!(
        "http_{}:{}",
        response.status,
        public_error_code(&response.body)
    )
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

pub fn run_synthetic_probe(
    target: &yai_core_engine::provider_governance::ProviderTarget,
    probe_embedding: bool,
    requested_shapes: &[ProviderRealizationShape],
    api_key: Option<String>,
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
                match catalog_models(&response.body) {
                    Ok(models) => {
                        evidence.exact_model_addressed = models.contains(&target.model_id);
                    }
                    Err(_) => evidence
                        .failure_codes
                        .push("models_response_invalid".to_string()),
                }
            }
        }
        Err(error) => evidence.failure_codes.push(probe_failure_code(&error)),
    }

    if !evidence.exact_model_addressed {
        evidence
            .failure_codes
            .push("exact_model_not_in_current_catalog".into());
        evidence.completed_at_unix_ms = now_ms().max(evidence.started_at_unix_ms);
        evidence.failure_codes.sort();
        evidence.failure_codes.dedup();
        return evidence;
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
                        let exact_response_model = value.get("model").and_then(Value::as_str)
                            == Some(target.model_id.as_str());
                        evidence.exact_model_addressed &= exact_response_model;
                        if !exact_response_model { evidence.failure_codes.push("chat_response_model_mismatch".into()); }
                        evidence.chat_text_envelope_valid = exact_response_model && value
                            .pointer("/choices/0/message/content")
                            .and_then(Value::as_str)
                            .is_some();
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
                    let exact_response_model = value.get("model").and_then(Value::as_str)
                        == Some(target.model_id.as_str());
                    if !exact_response_model { evidence.failure_codes.push("json_response_model_mismatch".into()); }
                    evidence.structured_json_object_valid = exact_response_model && value
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn synthetic_probe_rejects_wrong_or_missing_response_model() {
        use std::io::{BufRead, BufReader, Read, Write};
        use std::net::TcpListener;
        for response_model in [Some("probe-exact"), Some("another-model"), None] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let endpoint = format!("http://{}", listener.local_addr().unwrap());
            let peer = std::thread::spawn(move || {
                for index in 0..3 {
                    let (mut stream, _) = listener.accept().unwrap();
                    stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut line = String::new(); reader.read_line(&mut line).unwrap();
                    assert!(line.starts_with(if index == 0 { "GET /v1/models " } else { "POST /v1/chat/completions " }));
                    let mut length = 0;
                    loop { line.clear(); reader.read_line(&mut line).unwrap(); if line == "\r\n" { break; }
                        if let Some((key, value)) = line.split_once(':') { if key.eq_ignore_ascii_case("content-length") { length = value.trim().parse::<usize>().unwrap(); } }
                    }
                    let mut body = vec![0;length];reader.read_exact(&mut body).unwrap();
                    let mut response = if index == 0 { serde_json::json!({"data":[{"id":"probe-exact"}]}) }
                        else { serde_json::json!({"choices":[{"message":{"content":if index == 1 { "YAI_OK" } else { "{\"ok\":true}" }}}]}) };
                    if index != 0 { if let Some(model) = response_model { response["model"] = serde_json::json!(model); } }
                    let bytes = serde_json::to_vec(&response).unwrap();
                    write!(stream,"HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",bytes.len()).unwrap();
                    stream.write_all(&bytes).unwrap();
                }
            });
            let target = yai_core_engine::provider_governance::ProviderTarget::from_input(ProviderTargetInput {
                tenant_id:"tenant:probe".into(),provider_key:"controlled".into(),adapter:ProviderAdapterKind::OpenAiCompatible,
                endpoint,model_id:"probe-exact".into(),credential_ref:"none".into(),locality:ProviderLocality::Loopback,
                extension_adapter_id:None,created_by_principal_id:"principal:probe".into(),created_at_unix_ms:1,
            }).unwrap();
            let evidence = run_synthetic_probe(&target,false,&[],None);
            peer.join().unwrap();
            let exact = response_model == Some("probe-exact");
            assert_eq!(evidence.exact_model_addressed,exact);
            assert_eq!(evidence.chat_text_envelope_valid,exact);
            assert_eq!(evidence.structured_json_object_valid,exact);
            if !exact { assert!(evidence.failure_codes.contains(&"chat_response_model_mismatch".into())); }
        }
    }


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
    #[test]
    fn catalog_discovery_is_bounded_exact_and_not_capability_evidence() {
        assert_eq!(
            catalog_models(br#"{"data":[{"id":"vision"},{"id":"bge"}]}"#).unwrap(),
            vec!["bge", "vision"]
        );
        for invalid in [
            br#"{"data":[]}"#.as_slice(),
            br#"{"data":[{"id":"x"},{"id":"x"}]}"#,
            br#"{"data":[{"id":""}]}"#,
            br#"{"data":[{"id":"name\nspoof"}]}"#,
            br#"{"data":[{"id":"x"}],"has_more":true}"#,
            br#"{"data":[{"id":"x"}],"next":"opaque-page"}"#,
            br#"{"data":[{"id":"x","id":"y"}]}"#,
            br#"{"data":{}}"#,
        ] {
            assert!(catalog_models(invalid).is_err());
        }
        assert!(catalog_models(
            &serde_json::to_vec(
                &serde_json::json!({"data":vec![serde_json::json!({"id":"x"});129]})
            )
            .unwrap()
        )
        .is_err());
        use crate::provider_transport::endpoint_locality;
        assert_eq!(
            endpoint_locality("http://127.0.0.1:18001").unwrap(),
            Some(ProviderLocality::Loopback)
        );
        assert_eq!(
            endpoint_locality("http://localhost:8001").unwrap(),
            Some(ProviderLocality::Loopback)
        );
        assert_eq!(
            endpoint_locality("http://[::1]:8001").unwrap(),
            Some(ProviderLocality::Loopback)
        );
        assert_eq!(
            endpoint_locality("http://192.168.1.61:8001").unwrap(),
            Some(ProviderLocality::PrivateNetwork)
        );
        assert_eq!(
            endpoint_locality("https://8.8.8.8").unwrap(),
            Some(ProviderLocality::Remote)
        );
        assert_eq!(endpoint_locality("https://provider.example").unwrap(), None);
        for endpoint in [
            "http://8.8.8.8",
            "http://user:secret@localhost",
            "http://localhost?token=x",
            "http://127.0.0.1/path?token=x",
            "http://0.0.0.0",
        ] {
            assert!(endpoint_locality(endpoint).is_err(), "{endpoint}");
        }
    }
}
