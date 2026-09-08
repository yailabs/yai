//! Operator setup actions compose existing owners. Provider/policy preparation
//! retains independent commits; the bounded Participant profile is atomic.
use super::*;
use serde_json::{json, Value};
use yai_core_engine::cognitive::{CognitiveBindingRole, SemanticEvidencePosture};
use yai_core_engine::governance::{compile_policy_source, scope_policy_compilation};
use yai_core_engine::provider_governance::{
    ProviderAdapterKind, ProviderFailoverPolicy, ProviderLocality, ProviderRealizationShape,
    ProviderTargetInput, ProviderTrustPosture,
};

pub(crate) struct ProviderConnection<'a> {
    pub endpoint: &'a str,
    pub model: &'a str,
    pub locality: ProviderLocality,
    pub credential_ref: &'a str,
    pub trust_approved: bool,
    pub suitability_ref: Option<&'a str>,
    pub replace: bool,
    pub expected_generation: Option<u64>,
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}

/// A short reference is only spelling for an exact ID, never a persisted alias
/// or an ambient selected Case. Resolution runs over authorized visible state.
pub(crate) fn scoped_name(prefix: &str, value: &str) -> Result<String, String> {
    let value = value.trim();
    let value = value.strip_prefix(prefix).unwrap_or(value);
    if value.is_empty()
        || value.len() > 160
        || !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._:-".contains(c))
    {
        return Err("setup_identifier_invalid".into());
    }
    Ok(format!("{prefix}{value}"))
}

/// Compose existing canonical facts after the human has approved this exact
/// generation/profile. No policy, provider trust, grant or resource is created.
pub(crate) fn admit_workbench_participants(
    store: &LmdbRecordStore,
    authenticated: &yai_core_engine::security::AuthenticatedPrincipal,
    state: &yai_core_engine::transition::CaseState,
    operator: &str,
    executor: &str,
) -> Result<yai_core_engine::transition::CaseState, String> {
    use yai_core_engine::transition::{
        PendingTransition, PrincipalParticipantLink, TransitionSource,
    };
    let tenant = state.tenant_id.as_deref().ok_or("setup_requires_tenant")?;
    store
        .resolve_security_context(authenticated, tenant)?
        .require_owner()?;
    let principal = authenticated.projected_principal_id();
    if operator == executor
        || state.principal_participant_links.iter().any(|link| {
            link.participant_id == executor
                || (link.principal_id == principal && link.participant_id != operator)
                || (link.participant_id == operator && link.principal_id != principal)
        })
    {
        return Err("setup_identity_conflict_no_authority_transfer".into());
    }
    let mut payloads = Vec::new();
    for (id, roles) in [
        (
            operator,
            &["operation-proposer", "operation-reviewer", "workflow-input"][..],
        ),
        (executor, &["model-executor", "operation-proposer"][..]),
    ] {
        for role in roles {
            if !state
                .participants
                .iter()
                .any(|p| p.participant_id == id && p.roles.iter().any(|r| r == role))
            {
                payloads.push(TransitionPayload::ParticipantBound {
                    participant_id: id.into(),
                    role: (*role).into(),
                });
            }
        }
    }
    if !state.participants.iter().any(|p| {
        p.participant_id == executor
            && p.admitted_views
                .iter()
                .any(|v| v.consumer == "model" && v.view_kind == "model_context")
    }) {
        payloads.push(TransitionPayload::ParticipantAdmitted {
            participant_id: executor.into(),
            consumer: "model".into(),
            view_kind: "model_context".into(),
        });
    }
    if !state
        .principal_participant_links
        .iter()
        .any(|p| p.principal_id == principal && p.participant_id == operator)
    {
        payloads.push(TransitionPayload::ParticipantPrincipalLinked {
            link: PrincipalParticipantLink::new(
                &state.case_id,
                tenant,
                &principal,
                operator,
                &principal,
                now_unix_ms(),
            )?,
        });
    }
    let current = store.get_case_state_authorized(authenticated, &state.case_id)?;
    if current.generation != state.generation {
        return Err("stale_case_generation: inspect setup again".into());
    }
    if payloads.is_empty() {
        return Ok(current);
    }
    let changes = payloads
        .into_iter()
        .enumerate()
        .map(|(offset, payload)| {
            let digest = yai_core_engine::effect::digest_bytes(
                &serde_json::to_vec(&payload).expect("typed payload"),
            );
            let mut change = PendingTransition::new(
                format!(
                    "transition:participant-setup:{}:{}:{}",
                    state.case_id, state.generation, digest
                ),
                &state.case_id,
                state.generation + offset as u64,
                TransitionSource {
                    component: "yai.case_participant_setup".into(),
                    participant_id: None,
                    principal_id: Some(principal.clone()),
                    source_ref: Some(format!("participant-setup:{operator}:{executor}")),
                },
                payload,
            );
            change.causal_refs = vec![principal.clone(), operator.into(), executor.into()];
            change
        })
        .collect();
    store.commit_participant_setup_authorized(authenticated, tenant, changes)
}

/// Explicit operator file input, not natural-text path recognition. Bounded
/// regular-file reads avoid FIFO/device hangs at administrator setup surfaces.
pub(super) fn read_input(path: &Path) -> Result<Vec<u8>, String> {
    use std::io::Read;
    use std::os::unix::fs::OpenOptionsExt;
    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)
        .map_err(|e| format!("case_setup_input_open:{e}"))?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("case_setup_regular_file_required".into());
    }
    let mut bytes = Vec::new();
    file.take(65_537)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 65_536 {
        return Err("case_setup_input_bound_exceeded".into());
    }
    Ok(bytes)
}

impl ConversationController {
    /// Read-only setup context; capture before catalog I/O and human approval.
    pub(crate) fn provider_connection_state(&self) -> Result<(u64, Option<String>), String> {
        let a = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        a.store
            .resolve_security_context(&a.authenticated, &a.tenant_id)?
            .require_owner()?;
        let executor = self
            .executor_participant_id
            .as_deref()
            .ok_or("case_connect_select_executor_first")?;
        let primary = a
            .state
            .cognitive_bindings
            .iter()
            .find(|b| b.participant_id == executor && b.role == CognitiveBindingRole::Primary)
            .map(|b| b.binding_id.clone());
        Ok((a.state.generation, primary))
    }

    pub(crate) fn discover_provider_models(
        &self,
        endpoint: &str,
        locality: &ProviderLocality,
        credential_ref: &str,
    ) -> Result<Vec<String>, String> {
        self.provider_connection_state()?;
        super::super::provider_governance_cli::discover_provider_models(
            endpoint,
            locality,
            credential_ref,
        )
    }

    pub(crate) fn attach_resource(&self, path: &Path) -> Result<Value, String> {
        let a = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        a.store
            .resolve_security_context(&a.authenticated, &a.tenant_id)?
            .require_owner()?;
        let input = serde_json::from_slice(&read_input(path)?)
            .map_err(|e| format!("resource_definition:{e}"))?;
        let state = controlled_effect::access::import_definition(
            &a.store,
            &a.authenticated,
            &self.case_id,
            input,
        )?;
        Ok(
            json!({"case":state.case_id,"generation":state.generation,"resources":state.resources.len(),"attachment_is_permission":false}),
        )
    }

    pub(crate) fn publish_policy(&self, path: &Path, reason: &str) -> Result<Value, String> {
        let a = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        a.store
            .resolve_security_context(&a.authenticated, &a.tenant_id)?
            .require_owner()?;
        let tenant = a
            .store
            .get_tenant(&a.tenant_id)?
            .ok_or("tenant_not_visible")?;
        let compilation = scope_policy_compilation(
            &compile_policy_source(&read_input(path)?)?,
            &a.tenant_id,
            &tenant.organization_ref,
        )?;
        let intake = a.store.ingest_tenant_policy_compilation(
            &a.authenticated,
            &a.tenant_id,
            &compilation,
        )?;
        let artifact = &intake.view.artifact.artifact_id;
        a.store
            .validate_tenant_policy_artifact(&a.authenticated, artifact, reason)?;
        a.store
            .publish_tenant_policy_artifact(&a.authenticated, artifact, reason)?;
        a.store.bind_tenant_case_policy(
            &a.authenticated,
            &self.case_id,
            artifact,
            a.state.generation,
            reason,
        )?;
        Ok(json!({"artifact_id":artifact,"policy":a.store.case_policy_status(&self.case_id)?}))
    }

    pub(crate) fn connect_provider(&self, input: ProviderConnection<'_>) -> Result<Value, String> {
        if !input.trust_approved
            || input
                .suitability_ref
                .is_some_and(|value| !value.starts_with("evidence:") || value.len() > 240)
        {
            return Err("case_connect_requires_explicit_trust_and_evidence_reference".into());
        }
        let a = authorized_conversation_case(&self.case_id, &self.participant_id)?;
        if input
            .expected_generation
            .is_some_and(|generation| generation != a.state.generation)
        {
            return Err(
                "case_connect_approval_stale: inspect current Case and confirm again".into(),
            );
        }
        a.store
            .resolve_security_context(&a.authenticated, &a.tenant_id)?
            .require_owner()?;
        let executor = self
            .executor_participant_id
            .as_deref()
            .ok_or("case_connect_select_executor_first")?;
        if !input.replace
            && a.state
                .cognitive_bindings
                .iter()
                .any(|b| b.participant_id == executor && b.role == CognitiveBindingRole::Primary)
        {
            return Err("case_connect_existing_primary_requires_explicit_replace".into());
        }
        if a.state
            .provider_binding
            .as_ref()
            .is_some_and(|b| b.participant_id != executor)
        {
            return Err("case_connect_provider_envelope_participant_conflict".into());
        }
        let key = yai_core_engine::effect::digest_bytes(
            &serde_json::to_vec(&(input.endpoint, input.model)).map_err(|e| e.to_string())?,
        );
        let target = a.store.register_provider_target_authorized(
            &a.authenticated,
            ProviderTargetInput {
                tenant_id: a.tenant_id.clone(),
                provider_key: format!("case-connect-{}", &key[7..39]),
                adapter: ProviderAdapterKind::OpenAiCompatible,
                endpoint: input.endpoint.into(),
                model_id: input.model.into(),
                credential_ref: input.credential_ref.into(),
                locality: input.locality,
                extension_adapter_id: None,
                created_by_principal_id: a.principal_id.clone(),
                created_at_unix_ms: now_unix_ms(),
            },
        )?;
        let qualification = super::super::provider_governance_cli::qualify_connection_target(
            &a.store,
            &a.authenticated,
            &target,
        )?;
        if a.store
            .get_case_state_authorized(&a.authenticated, &self.case_id)?
            .generation
            != a.state.generation
        {
            return Err("case_connect_case_changed_during_probe: target qualification retained; re-inspect before retry".into());
        }
        a.store.set_provider_trust_authorized(
            &a.authenticated,
            &target.target_id,
            ProviderTrustPosture::Approved,
            now_unix_ms(),
        )?;
        // The UI approval itself supplies an operator attestation, not an
        // invented evaluator run. Its exact provenance is generated, not typed
        // by the human. The qualification run remains independently referenced.
        let generated_ref = format!(
            "evidence:operator-connect:{}",
            &yai_core_engine::effect::digest_bytes(
                &serde_json::to_vec(&(
                    &self.case_id,
                    executor,
                    &a.principal_id,
                    a.state.generation,
                    &target.target_id,
                    &qualification.qualification_id
                ))
                .map_err(|e| e.to_string())?
            )[7..]
        );
        let evidence = a.store.record_semantic_suitability_evidence_authorized(
            &a.authenticated,
            &target.target_id,
            CognitiveCapability::PrimaryConversation,
            SemanticEvidencePosture::OperatorAttested,
            "yai.operator.case_connection.v1",
            &qualification.qualification_id,
            vec![input.suitability_ref.unwrap_or(&generated_ref).into()],
            "authenticated_operator_attestation",
        )?;
        let mut targets = a
            .state
            .provider_binding
            .as_ref()
            .map(|b| b.ordered_target_ids.clone())
            .unwrap_or_default();
        if !targets.contains(&target.target_id) {
            targets.push(target.target_id.clone());
        }
        let envelope = a.store.bind_case_provider_targets_authorized(
            &a.authenticated,
            &self.case_id,
            executor,
            targets,
            ProviderFailoverPolicy::SafeOnly,
            1,
        )?;
        let binding = a.store.bind_case_cognitive_candidates_authorized(
            &a.authenticated,
            &self.case_id,
            executor,
            CognitiveBindingRole::Primary,
            CognitiveCapability::PrimaryConversation,
            vec![(target.target_id.clone(), evidence.evidence_id.clone())],
            input.replace,
        )?;
        Ok(
            json!({"target_id":target.target_id,"endpoint":target.endpoint,"model":target.model_id,"qualification_id":qualification.qualification_id,
            "connection":"connected",
            "capabilities":{
                "text":qualification.supports_realization_shape(&ProviderRealizationShape::TextToText),
                "native_functions":qualification.supports_realization_shape(&ProviderRealizationShape::TextFunctionsToTextOrCall),
                "json_object":qualification.supports_realization_shape(&ProviderRealizationShape::TextToJsonObject)
            },
            "qualification_failures":qualification.evidence.failure_codes,
            "realization_shapes":qualification.evidence.realization_shapes,
            "semantic_posture":"operator_attested","evidence_id":evidence.evidence_id,"provider_binding":envelope.binding_id,"cognitive_binding":binding.binding_id,
            "policy":"pinned","trust":"explicit_operator_approval","case_continuity":"preserved"}),
        )
    }
}
