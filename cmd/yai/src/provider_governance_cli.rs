//! Product/admin adapters for Tenant-scoped provider governance.

use super::*;
use serde::de::{Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};
use yai_core_engine::provider_governance::{
    ProviderAdapterKind, ProviderFailoverPolicy, ProviderLocality, ProviderProbeEvidence,
    ProviderTargetInput, ProviderTrustPosture,
};
use yai_core_engine::security::AuthenticatedPrincipal;

const QUALIFICATION_SUITE: &str = "yai.openai_compatible.synthetic.v1";

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
            credential_ref: optional_arg(args, "--credential-ref")
                .unwrap_or_else(|| "none".to_string()),
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
    let target_id = named_arg(args, "--target")?;
    let (authenticated, store) = authenticated_store()?;
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
            if qualification.evidence.chat_text_envelope_valid
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

fn strict_json(body: &[u8]) -> Result<Value, String> {
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
    let response = super::provider_transport::provider_http(
        endpoint,
        Some(locality),
        method,
        path,
        body.unwrap_or_default(),
        api_key,
    )?;
    Ok(ProbeHttpResponse {
        status: response.status,
        body: response.body,
    })
}

fn credential_for(target: &yai_core_engine::provider_governance::ProviderTarget) -> Option<String> {
    target
        .credential_ref
        .strip_prefix("env:")
        .and_then(provider::env_var)
}

fn run_synthetic_probe(
    target: &yai_core_engine::provider_governance::ProviderTarget,
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
        Err(error) => evidence.failure_codes.push(
            error
                .split(':')
                .next()
                .unwrap_or("transport_failure")
                .to_string(),
        ),
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
        Err(error) => evidence.failure_codes.push(
            error
                .split(':')
                .next()
                .unwrap_or("transport_failure")
                .to_string(),
        ),
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
    println!("health_probe: {}", evidence.health_endpoint_observed);
    println!(
        "extension_compatible_telemetry: {}",
        evidence.extension_telemetry_observed
    );
    println!("synthetic_input_only: true");
    println!("failure_codes: {}", evidence.failure_codes.join(","));
}

fn provider_probe(args: &[String], persist_qualification: bool) -> Result<(), String> {
    let target_id = named_arg(args, "--target")?;
    let (authenticated, store) = authenticated_store()?;
    let (target, _, _, _) = store.provider_posture_authorized(&authenticated, &target_id)?;
    let admission_token = format!("probe-admission:{}:{}", std::process::id(), now_ms());
    let probe_owner = store.begin_provider_probe_authorized(
        &authenticated,
        &target.target_id,
        &admission_token,
    )?;
    let evidence = run_synthetic_probe(&target);
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
            QUALIFICATION_SUITE,
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

fn provider_trust(args: &[String], posture: ProviderTrustPosture) -> Result<(), String> {
    let target_id = named_arg(args, "--target")?;
    let (authenticated, store) = authenticated_store()?;
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
    let target_id = named_arg(args, "--target")?;
    let revision_label = named_arg(args, "--revision")?;
    let (authenticated, store) = authenticated_store()?;
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
    let targets = repeated_arg(args, "--target");
    let failover = parse_failover(
        &optional_arg(args, "--failover").unwrap_or_else(|| "safe_only".to_string()),
    )?;
    let max_attempts = optional_arg(args, "--max-attempts")
        .unwrap_or_else(|| "3".to_string())
        .parse::<u32>()
        .map_err(|_| "provider_max_attempts_invalid".to_string())?;
    let (authenticated, store) = authenticated_store()?;
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
    fn strict_json_rejects_duplicate_keys() {
        assert!(strict_json(br#"{"ok":true}"#).is_ok());
        assert!(strict_json(br#"{"ok":true,"ok":false}"#)
            .unwrap_err()
            .contains("duplicate JSON key"));
    }
}
