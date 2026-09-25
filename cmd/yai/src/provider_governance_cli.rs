//! Product/admin adapters for Tenant-scoped provider governance.

use super::*;
use std::time::{SystemTime, UNIX_EPOCH};
use yai_core_engine::provider_governance::{
    ProviderAdapterKind, ProviderFailoverPolicy, ProviderLocality, ProviderProbeEvidence,
    ProviderRealizationShape, ProviderTargetInput, ProviderTrustPosture,
};
use yai_core_engine::security::AuthenticatedPrincipal;

use yai_core_engine::provider_governance::{ProviderProbeRequest, ProviderProbeRun};

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

pub(super) use yai_application::provider_execution::strict_json;
pub(super) fn discover_provider_models(endpoint: &str, locality: &ProviderLocality, credential_ref: &str) -> Result<Vec<String>, String> {
    yai_application::provider_execution::discover_provider_models(endpoint, locality, credential_ref, |key| std::env::var(key).ok())
}

fn await_provider_probe(
    store: &LmdbRecordStore,
    authenticated: &AuthenticatedPrincipal,
    request: ProviderProbeRequest,
) -> Result<ProviderProbeRun, String> {
    let home = yai_home();
    let mut result = yai_application::provider_execution::probes::submit(&home, store, authenticated, request.clone())?;
    println!("probe_submission: {}", request.submission_ref);
    while result.posture == "running" {
        std::thread::sleep(std::time::Duration::from_millis(100));
        result = yai_application::provider_execution::probes::observe(&home, store, authenticated,
            &request.target_id, &request.submission_ref, false)?;
    }
    if result.posture != "completed" {
        return Err(format!("provider_probe_{}: submission={}", result.posture, request.submission_ref));
    }
    Ok(result.run)
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
    let valid_for_ms = optional_arg(args, "--valid-for-ms")
        .map(|value| value.parse::<u64>().map_err(|_| "provider_qualification_valid_for_invalid".to_string()))
        .transpose()?;
    let run = await_provider_probe(&store, &authenticated, ProviderProbeRequest {
        target_id:target.target_id, submission_ref:optional_arg(args, "--submission-ref")
            .unwrap_or_else(|| format!("probe-admission:{}:{}", std::process::id(), now_ms())),
        embedding:probe_embedding, realization_shapes:requested_shapes, qualify:persist_qualification, valid_for_ms,
    })?;
    print_probe(run.evidence.as_ref().ok_or("provider_probe_evidence_missing")?);
    if let Some(qualification) = run.qualification {
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
pub(super) fn qualify_connection_target(
    store: &LmdbRecordStore,
    authenticated: &AuthenticatedPrincipal,
    target: &yai_core_engine::provider_governance::ProviderTarget,
) -> Result<yai_core_engine::provider_governance::ProviderQualification, String> {
    let run = await_provider_probe(store, authenticated, ProviderProbeRequest {
        target_id:target.target_id.clone(), submission_ref:format!("probe-admission:{}:{}", std::process::id(), now_ms()),
        embedding:false, realization_shapes:vec![ProviderRealizationShape::TextToText,
            ProviderRealizationShape::TextFunctionsToTextOrCall, ProviderRealizationShape::TextToJsonObject],
        qualify:true, valid_for_ms:None,
    })?;
    let qualified = run.qualification.ok_or("provider_probe_qualification_missing")?;
    // A connection is not blanket permission to use all provider contracts.
    // Keep independently proven shapes; exact execution checks still require
    // each requested shape. Failed function/JSON probes never qualify tools.
    if !qualified.supports_realization_shape(&ProviderRealizationShape::TextToText) {
        return Err(format!("case_connect_mechanical_contract_unqualified: target={} qualification={}; required={}; proven={}; failures={}; no trust or Case binding added", target.target_id, qualified.qualification_id,
            ProviderRealizationShape::TextToText.as_str(),
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
