//! Optional public-wire capacity extension. It never changes W, authority,
//! provider selection, request bytes, or retry policy. Generic providers remain
//! generic; an unsupported capacity contract stays explicitly unknown.
use super::ProviderConfig;
use crate::provider_transport::{provider_http, ProviderEndpoint};
use serde_json::Value;
use yai_core_engine::context::ProviderCapacityObservation;

pub(super) struct Assessment {
    pub capacity: Option<ProviderCapacityObservation>,
    pub refusal: Option<&'static str>,
}

fn text(value: &Value, key: &str) -> Result<String, String> {
    value[key].as_str().filter(|s| !s.is_empty() && s.len() <= 256)
        .map(str::to_string).ok_or_else(|| format!("capacity_field_invalid:{key}"))
}
fn number(value: &Value, key: &str) -> Result<u64, String> {
    value[key].as_u64().ok_or_else(|| format!("capacity_field_invalid:{key}"))
}
fn boolean(value: &Value, key: &str) -> Result<bool, String> {
    value[key].as_bool().ok_or_else(|| format!("capacity_field_invalid:{key}"))
}

pub(super) fn advertised(row: &Value, model: &str) -> Result<ProviderCapacityObservation, String> {
    let limits = &row["yvex_capacity"];
    if row["yvex_profile"] != "yvex.openai.compat.v3"
        || limits["schema"] != "yvex.execution.capacity.v1"
        || limits["input_accounting"] != "exact_tokenizer_including_template_and_tools"
        || limits["resource_reservation"] != false {
        return Err("capacity_contract_invalid".into());
    }
    Ok(ProviderCapacityObservation {
        public_contract: "yvex.openai.compat.v3".into(), observation_kind: "public_model_catalog".into(),
        model_id: model.into(), engine_generation: number(row,"engine_generation")?,
        runtime_binding_identity: text(row,"runtime_binding_identity")?,
        runtime_model_identity: text(row,"runtime_model_identity")?,
        capacity_plan_identity: text(row,"capacity_plan_identity")?,
        tokenizer_identity: None, prompt_identity: None, provider_request_identity: None,
        http_body_limit_bytes: number(limits,"http_body_bytes")?, input_tokens: None,
        input_capacity_tokens: number(limits,"runtime_input_tokens")?,
        sequence_capacity_tokens: number(limits,"runtime_sequence_tokens")?,
        requested_output_tokens: None, effective_output_tokens: None,
        full_requested_output_fits: None, token_capacity_compatible: None,
        resource_reservation: false, execution_or_resources_qualified: false,
    })
}

fn qualify_response(value: &Value, model: &str, expected: &ProviderCapacityObservation) -> Result<ProviderCapacityObservation, String> {
    if value["object"] != "yvex.execution.preflight" || value["model"] != model
        || value["scope"] != "complete_stateless_chat_request"
        || value["execution_or_resources_qualified"] != false {
        return Err("capacity_preflight_identity_mismatch".into());
    }
    let mut result = advertised(value, model)?;
    if &result != expected { return Err("capacity_deployment_changed".into()); }
    result.observation_kind = "exact_request_preflight".into();
    result.tokenizer_identity = Some(text(value,"tokenizer_identity")?);
    result.prompt_identity = Some(text(value,"prompt_identity")?);
    result.provider_request_identity = Some(text(value,"provider_request_identity")?);
    let input = number(value,"input_tokens")?;
    let output = number(value,"effective_output_tokens")?;
    let compatible = boolean(value,"token_capacity_compatible")?;
    let input_exceeded = boolean(value,"input_capacity_exceeded")?;
    let output_exceeded = boolean(value,"output_capacity_exceeded")?;
    if compatible && (input_exceeded || output_exceeded || input > result.input_capacity_tokens
        || input.checked_add(output).is_none_or(|n| n > result.sequence_capacity_tokens)) {
        return Err("capacity_preflight_inconsistent".into());
    }
    result.input_tokens = Some(input);
    result.requested_output_tokens = Some(number(value,"requested_output_tokens")?);
    result.effective_output_tokens = Some(output);
    result.full_requested_output_fits = Some(boolean(value,"full_requested_output_fits")?);
    result.token_capacity_compatible = Some(compatible);
    Ok(result)
}

pub(super) fn assess(config: &ProviderConfig, endpoint: &ProviderEndpoint, body: &[u8]) -> Result<Assessment, String> {
    if config.extension_adapter_id.as_deref() != Some("yvex.http.v1") {
        return Ok(Assessment {capacity: None, refusal: None});
    }
    let prefix = endpoint.path.strip_suffix("/chat/completions").ok_or("capacity_endpoint_not_supported")?;
    let catalog = provider_http(endpoint, config.governed_locality.as_ref(), "GET", &format!("{prefix}/models"), &[], config.api_key.as_deref())?;
    if catalog.status != 200 { return Err("capacity_catalog_unavailable".into()); }
    let catalog: Value = serde_json::from_slice(&catalog.body).map_err(|_| "capacity_catalog_invalid")?;
    let rows = catalog["data"].as_array().filter(|rows| rows.len() <= 128).ok_or("capacity_catalog_invalid")?;
    let rows = rows.iter().filter(|row| row["id"].as_str() == Some(config.model.as_str())).collect::<Vec<_>>();
    if rows.len() != 1 { return Err("capacity_exact_model_unavailable".into()); }
    let row = rows[0];
    if row["yvex_profile"] != "yvex.openai.compat.v3" {
        return Ok(Assessment {capacity: None, refusal: None});
    }
    let expected = advertised(row, &config.model)?;
    if body.len() as u64 > expected.http_body_limit_bytes {
        return Ok(Assessment {capacity: Some(expected), refusal: Some("http_body_capacity_exceeded")});
    }
    let route = format!("{}/preflight", endpoint.path);
    if row["yvex_capacity"]["preflight"].as_str() != Some(route.as_str()) {
        return Err("capacity_preflight_route_mismatch".into());
    }
    // Exact bytes, same authenticated origin/locality. No redirect, remote URL
    // from model metadata, inference, or reservation is performed by preflight.
    let response = provider_http(endpoint, config.governed_locality.as_ref(), "POST", &route, body, config.api_key.as_deref())?;
    if response.status != 200 { return Err("capacity_preflight_unavailable".into()); }
    let value: Value = serde_json::from_slice(&response.body).map_err(|_| "capacity_preflight_invalid")?;
    let capacity = qualify_response(&value, &config.model, &expected)?;
    let refusal = (capacity.token_capacity_compatible != Some(true)).then_some("token_capacity_exceeded");
    Ok(Assessment {capacity: Some(capacity), refusal})
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use yai_core_engine::context::{RenderedInput, RenderedInputMetadata};

    fn row() -> Value { json!({"id":"controlled-model","yvex_profile":"yvex.openai.compat.v3",
        "engine_generation":1,"runtime_binding_identity":"binding:one","runtime_model_identity":"runtime:one",
        "capacity_plan_identity":"capacity:one","yvex_capacity":{"schema":"yvex.execution.capacity.v1",
        "input_accounting":"exact_tokenizer_including_template_and_tools","resource_reservation":false,
        "http_body_bytes":1048576,"runtime_input_tokens":16,"runtime_sequence_tokens":32,
        "preflight":"/v1/chat/completions/preflight"}}) }
    fn response(compatible: bool) -> Value {
        let mut value = row();
        value.as_object_mut().unwrap().extend(json!({"object":"yvex.execution.preflight","model":"controlled-model",
            "scope":"complete_stateless_chat_request","execution_or_resources_qualified":false,
            "input_tokens":if compatible {10} else {40},"requested_output_tokens":0,
            "effective_output_tokens":if compatible {22} else {0},"full_requested_output_fits":false,
            "input_capacity_exceeded":!compatible,"output_capacity_exceeded":false,
            "token_capacity_compatible":compatible,"tokenizer_identity":"tokenizer:one",
            "prompt_identity":"prompt:one","provider_request_identity":"request:one"}).as_object().unwrap().clone());
        value
    }
    fn config(endpoint: String) -> ProviderConfig { ProviderConfig {
        provider_id:"target:controlled".into(),base_url:endpoint,model:"controlled-model".into(),api_key:None,
        language_mode:"auto".into(),continuation_supported:false,continuation_ref:None,governance:None,
        governed_locality:Some(yai_core_engine::provider_governance::ProviderLocality::Loopback),
        extension_adapter_id:Some("yvex.http.v1".into()),
    } }
    fn rendered() -> RenderedInput { RenderedInput {
        metadata:RenderedInputMetadata {schema:"yai.rendered_input.v7".into(),rendered_input_id:"rendered:test".into(),
            context_frame_id:"frame:test".into(),provider_id:"target:controlled".into(),model_id:"controlled-model".into(),
            content_digest:"digest:sentinel".into(),content_chars:24},
        system_content:"MANDATORY_SENTINEL".into(),user_content:"EVIDENCE_SENTINEL".into(),
    } }

    #[test]
    fn capacity_requires_exact_deployment_and_consistent_token_accounting() {
        let expected = advertised(&row(),"controlled-model").unwrap();
        assert_eq!(expected.input_tokens,None);
        let qualified = qualify_response(&response(true),"controlled-model",&expected).unwrap();
        assert_eq!(qualified.input_tokens,Some(10));
        assert_eq!(qualified.effective_output_tokens,Some(22));
        assert!(!qualified.resource_reservation && !qualified.execution_or_resources_qualified);
        for (field,value) in [("model",json!("different-model")),("engine_generation",json!(2)),
            ("capacity_plan_identity",json!("capacity:changed")),("input_tokens",json!(100)),
            ("effective_output_tokens",json!(30)),("tokenizer_identity",Value::Null)] {
            let mut changed=response(true);changed[field]=value;
            assert!(qualify_response(&changed,"controlled-model",&expected).is_err(),"{field}");
        }
        assert_eq!(qualify_response(&response(false),"controlled-model",&expected).unwrap().token_capacity_compatible,Some(false));
    }

    #[test]
    fn complete_wire_budget_refuses_ordinary_text_before_any_network() {
        let observations=std::cell::RefCell::new(Vec::new());
        let observer=|value| {observations.borrow_mut().push(value);Ok(())};
        let error=super::super::provider_http_request_with_functions(&config("http://127.0.0.1:9/v1/chat/completions".into()),
            &rendered(),None,false,None,None,Some(1),Some(&observer)).unwrap_err();
        assert_eq!(error,"provider_not_dispatched:complete_wire_input_budget_exceeded");
        let values=observations.borrow();assert_eq!(values.len(),1);assert!(values[0].bytes>4);
        assert!(values[0].capacity.is_none());assert!(values[0].digest.starts_with("sha256:"));
    }

    #[test]
    fn exact_preflight_bytes_are_dispatched_unchanged_or_never_dispatched() {
        for compatible in [false,true] {
            let listener=TcpListener::bind("127.0.0.1:0").unwrap();
            listener.set_nonblocking(true).unwrap();
            let address=listener.local_addr().unwrap();
            let server=std::thread::spawn(move || {
                let mut bodies=Vec::new();
                for index in 0..if compatible {3} else {2} {
                    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
                    let (mut stream,_)=loop {
                        match listener.accept() {
                            Ok(connection)=>break connection,
                            Err(error) if error.kind()==std::io::ErrorKind::WouldBlock=>{
                                assert!(std::time::Instant::now()<deadline,"missing request {index}");
                                std::thread::sleep(std::time::Duration::from_millis(10));
                            }
                            Err(error)=>panic!("accept failed: {error}"),
                        }
                    };
                    stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
                    let mut bytes=Vec::new();let mut chunk=[0;4096];
                    let body_start=loop {
                        let count=stream.read(&mut chunk).unwrap();assert!(count>0);bytes.extend_from_slice(&chunk[..count]);
                        if let Some(end)=bytes.windows(4).position(|w|w==b"\r\n\r\n") {break end+4;}
                    };
                    let headers=String::from_utf8(bytes[..body_start].to_vec()).unwrap();
                    let length=headers.lines().find_map(|line|line.to_ascii_lowercase().strip_prefix("content-length:").map(|n|n.trim().parse::<usize>().unwrap())).unwrap_or(0);
                    while bytes.len()<body_start+length {let count=stream.read(&mut chunk).unwrap();assert!(count>0);bytes.extend_from_slice(&chunk[..count]);}
                    bodies.push(bytes[body_start..body_start+length].to_vec());
                    let value=match index {
                        0=>{assert!(headers.starts_with("GET /v1/models "));json!({"data":[row()]})},
                        1=>{assert!(headers.starts_with("POST /v1/chat/completions/preflight "));response(compatible)},
                        _=>{assert!(headers.starts_with("POST /v1/chat/completions "));json!({"choices":[{"message":{"content":"controlled"}}]})},
                    }.to_string();
                    write!(stream,"HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",value.len(),value).unwrap();
                }
                bodies
            });
            let result=super::super::provider_http_request(&config(format!("http://{address}/v1/chat/completions")),&rendered(),None,false,None);
            let bodies=server.join().unwrap();
            let wire:Value=serde_json::from_slice(&bodies[1]).unwrap();
            assert_eq!(wire["messages"][0]["content"],"MANDATORY_SENTINEL");
            assert_eq!(wire["messages"][1]["content"],"EVIDENCE_SENTINEL");
            if compatible {assert_eq!(result.unwrap().0,200);assert_eq!(bodies[1],bodies[2]);}
            else {assert_eq!(result.unwrap_err(),"provider_not_dispatched:token_capacity_exceeded");assert_eq!(bodies.len(),2);}
        }
    }
}
