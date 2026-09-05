use super::*;
use crate::cognitive::{
    CognitiveBindingRole, CognitiveCapability, CognitiveCapabilityRequirement, CognitivePlanRoute,
    CognitivePlanUnresolvedReason, ProviderExecutionPosture, SemanticEvidencePosture,
};
use crate::provider_governance::{
    ProviderAdapterKind, ProviderFailoverPolicy, ProviderLocality, ProviderProbeEvidence,
    ProviderTargetInput, ProviderTrustPosture,
};

fn setup_case(
    store: &LmdbRecordStore,
    owner: &AuthenticatedPrincipal,
    tenant_id: &str,
    case_id: &str,
) {
    store
        .bootstrap_local_security(owner, tenant_id, "organization:i02", 1)
        .unwrap();
    store.create_tenant_case(owner, tenant_id, case_id).unwrap();
    store
        .commit_secured_transition(
            owner,
            tenant_id,
            secured_pending(
                &format!("transition:{case_id}:participant"),
                case_id,
                1,
                &owner.projected_principal_id(),
                TransitionPayload::ParticipantBound {
                    participant_id: "participant:model".to_string(),
                    role: "model-executor".to_string(),
                },
            ),
            true,
        )
        .unwrap();
}

fn target(
    store: &LmdbRecordStore,
    owner: &AuthenticatedPrincipal,
    tenant_id: &str,
    misleading_name: &str,
    port: u16,
) -> ProviderTarget {
    let target = store
        .register_provider_target_authorized(
            owner,
            ProviderTargetInput {
                tenant_id: tenant_id.to_string(),
                provider_key: misleading_name.to_string(),
                adapter: ProviderAdapterKind::OpenAiCompatible,
                endpoint: format!("http://127.0.0.1:{port}"),
                model_id: misleading_name.to_string(),
                credential_ref: "none".to_string(),
                locality: ProviderLocality::Loopback,
                extension_adapter_id: None,
                created_by_principal_id: owner.projected_principal_id(),
                created_at_unix_ms: u64::from(port),
            },
        )
        .unwrap();
    store
        .qualify_provider_target_authorized(
            owner,
            &target.target_id,
            ProviderProbeEvidence {
                run_id: format!("fixture:i02:{port}"),
                target_id: target.target_id.clone(),
                started_at_unix_ms: u64::from(port),
                completed_at_unix_ms: u64::from(port) + 1,
                transport_connected: true,
                exact_model_addressed: true,
                chat_text_envelope_valid: true,
                structured_json_object_valid: false,
                usage_accounting_observed: false,
                health_endpoint_observed: false,
                extension_telemetry_observed: false,
                text_embedding_envelope_valid: false,
                embedding_dimension: None,
                failure_codes: Vec::new(),
            },
            "yai.i02.deterministic_provider_fixture.v1",
            None,
        )
        .unwrap();
    store
        .set_provider_trust_authorized(
            owner,
            &target.target_id,
            ProviderTrustPosture::Approved,
            u64::from(port) + 2,
        )
        .unwrap();
    target
}

fn suitability(
    store: &LmdbRecordStore,
    owner: &AuthenticatedPrincipal,
    target: &ProviderTarget,
    capability: CognitiveCapability,
) -> crate::cognitive::SemanticSuitabilityEvidence {
    store
        .record_semantic_suitability_evidence_authorized(
            owner,
            &target.target_id,
            capability,
            SemanticEvidencePosture::DeterministicFixture,
            "yai.i02.semantic_fixture.v1",
            "run:i02:semantic",
            vec!["fixture:i02:semantic".to_string()],
            "repository_fixture",
        )
        .unwrap()
}

#[test]
fn i02_native_derived_replay_and_zero_execution_contract() {
    let path = temp_store_path("i02-plan");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(20201);
    let tenant_id = "tenant:i02";
    let case_id = "case:i02-plan";
    setup_case(&store, &owner, tenant_id, case_id);
    let primary = target(&store, &owner, tenant_id, "whisper-vision-name", 20201);
    let auxiliary = target(&store, &owner, tenant_id, "deepseek-bge-name", 20202);
    store
        .bind_case_provider_targets_authorized(
            &owner,
            case_id,
            "participant:model",
            vec![primary.target_id.clone(), auxiliary.target_id.clone()],
            ProviderFailoverPolicy::SafeOnly,
            2,
        )
        .unwrap();
    let primary_conversation = suitability(
        &store,
        &owner,
        &primary,
        CognitiveCapability::PrimaryConversation,
    );
    let primary_image = suitability(
        &store,
        &owner,
        &primary,
        CognitiveCapability::ImageUnderstanding,
    );
    let auxiliary_stt = suitability(
        &store,
        &owner,
        &auxiliary,
        CognitiveCapability::SpeechToText,
    );
    let primary_binding = store
        .bind_case_cognitive_target_authorized(
            &owner,
            case_id,
            "participant:model",
            CognitiveBindingRole::Primary,
            CognitiveCapability::PrimaryConversation,
            &primary.target_id,
            &primary_conversation.evidence_id,
            false,
        )
        .unwrap();
    let auxiliary_binding = store
        .bind_case_cognitive_target_authorized(
            &owner,
            case_id,
            "participant:model",
            CognitiveBindingRole::Auxiliary,
            CognitiveCapability::SpeechToText,
            &auxiliary.target_id,
            &auxiliary_stt.evidence_id,
            false,
        )
        .unwrap();
    let image_requirement = CognitiveCapabilityRequirement::new(
        case_id,
        "participant:model",
        CognitiveCapability::ImageUnderstanding,
        "turn:multipart-image",
    )
    .unwrap();
    let image_plan = store
        .plan_case_cognitive_execution_authorized(
            &owner,
            case_id,
            "participant:model",
            &image_requirement,
        )
        .unwrap();
    assert_eq!(image_plan.route, CognitivePlanRoute::Native);
    assert_eq!(
        image_plan.selected_target_id,
        Some(primary.target_id.clone())
    );
    assert_eq!(
        image_plan.semantic_evidence_id,
        Some(primary_image.evidence_id)
    );
    let stt_requirement = CognitiveCapabilityRequirement::new(
        case_id,
        "participant:model",
        CognitiveCapability::SpeechToText,
        "turn:multipart-audio",
    )
    .unwrap();
    let stt_plan = store
        .plan_case_cognitive_execution_authorized(
            &owner,
            case_id,
            "participant:model",
            &stt_requirement,
        )
        .unwrap();
    assert_eq!(stt_plan.route, CognitivePlanRoute::Derived);
    assert_eq!(
        stt_plan.selected_target_id,
        Some(auxiliary.target_id.clone())
    );
    assert_eq!(
        stt_plan.provider_execution,
        ProviderExecutionPosture::NotPerformed
    );
    assert_ne!(image_plan.execution_lane_id, stt_plan.execution_lane_id);
    let state = store.get_case_state_authorized(&owner, case_id).unwrap();
    assert!(state.provider_selections.is_empty());
    assert!(state.last_provider_invocation.is_none());
    assert!(state.last_provider_result.is_none());
    assert!(store.verify_case_state(case_id).unwrap());
    let rebuilt = store.rebuild_case_state(case_id).unwrap();
    assert_eq!(
        rebuilt.cognitive_bindings,
        vec![primary_binding.clone(), auxiliary_binding.clone()]
    );
    let replay_plan = store
        .plan_case_cognitive_execution_authorized(
            &owner,
            case_id,
            "participant:model",
            &stt_requirement,
        )
        .unwrap();
    assert_eq!(stt_plan, replay_plan);
    println!(
        "i02_plan: native={} derived={} primary_lane={} auxiliary_lane={} provider_dispatches=0 replay=true",
        image_plan.plan_id,
        stt_plan.plan_id,
        image_plan.execution_lane_id.unwrap(),
        stt_plan.execution_lane_id.unwrap()
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn i02_binding_composition_and_name_inference_fail_closed() {
    let path = temp_store_path("i02-adversarial");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner_a = AuthenticatedPrincipal::for_test(20202);
    let owner_b = AuthenticatedPrincipal::for_test(20203);
    setup_case(&store, &owner_a, "tenant:i02-a", "case:i02-adversarial");
    store
        .bootstrap_local_security(&owner_b, "tenant:i02-b", "organization:i02-b", 1)
        .unwrap();
    let named_whisper = target(
        &store,
        &owner_a,
        "tenant:i02-a",
        "whisper-vision-deepseek-bge",
        20203,
    );
    let not_admitted = target(&store, &owner_a, "tenant:i02-a", "not-admitted", 20204);
    let other_tenant = target(&store, &owner_b, "tenant:i02-b", "other-tenant", 20205);
    store
        .bind_case_provider_targets_authorized(
            &owner_a,
            "case:i02-adversarial",
            "participant:model",
            vec![named_whisper.target_id.clone()],
            ProviderFailoverPolicy::None,
            1,
        )
        .unwrap();
    let primary = suitability(
        &store,
        &owner_a,
        &named_whisper,
        CognitiveCapability::PrimaryConversation,
    );
    store
        .bind_case_cognitive_target_authorized(
            &owner_a,
            "case:i02-adversarial",
            "participant:model",
            CognitiveBindingRole::Primary,
            CognitiveCapability::PrimaryConversation,
            &named_whisper.target_id,
            &primary.evidence_id,
            false,
        )
        .unwrap();
    let stt = CognitiveCapabilityRequirement::new(
        "case:i02-adversarial",
        "participant:model",
        CognitiveCapability::SpeechToText,
        "turn:name-is-not-proof",
    )
    .unwrap();
    let unresolved = store
        .plan_case_cognitive_execution_authorized(
            &owner_a,
            "case:i02-adversarial",
            "participant:model",
            &stt,
        )
        .unwrap();
    assert_eq!(unresolved.route, CognitivePlanRoute::Unresolved);
    let not_admitted_evidence = suitability(
        &store,
        &owner_a,
        &not_admitted,
        CognitiveCapability::SpeechToText,
    );
    assert_eq!(
        store
            .bind_case_cognitive_target_authorized(
                &owner_a,
                "case:i02-adversarial",
                "participant:model",
                CognitiveBindingRole::Auxiliary,
                CognitiveCapability::SpeechToText,
                &not_admitted.target_id,
                &not_admitted_evidence.evidence_id,
                false,
            )
            .unwrap_err(),
        "cognitive_target_not_admitted_by_provider_envelope"
    );
    let other_evidence = suitability(
        &store,
        &owner_b,
        &other_tenant,
        CognitiveCapability::SpeechToText,
    );
    let cross_tenant = store
        .bind_case_cognitive_target_authorized(
            &owner_a,
            "case:i02-adversarial",
            "participant:model",
            CognitiveBindingRole::Auxiliary,
            CognitiveCapability::SpeechToText,
            &named_whisper.target_id,
            &other_evidence.evidence_id,
            false,
        )
        .unwrap_err();
    assert_eq!(
        cross_tenant,
        "semantic_suitability_evidence_binding_mismatch"
    );
    println!(
        "i02_adversarial: misleading_name=unresolved envelope_mismatch=rejected cross_tenant_evidence={} provider_dispatches=0",
        cross_tenant
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn i02_replacement_envelope_invalidation_unbind_and_replay_are_explicit() {
    let path = temp_store_path("i02-replacement");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(20204);
    let intruder = AuthenticatedPrincipal::for_test(20205);
    let tenant_id = "tenant:i02-replacement";
    let case_id = "case:i02-replacement";
    setup_case(&store, &owner, tenant_id, case_id);
    let target_a = target(&store, &owner, tenant_id, "primary-a", 20206);
    let target_b = target(&store, &owner, tenant_id, "primary-b", 20207);
    store
        .bind_case_provider_targets_authorized(
            &owner,
            case_id,
            "participant:model",
            vec![target_a.target_id.clone(), target_b.target_id.clone()],
            ProviderFailoverPolicy::SafeOnly,
            2,
        )
        .unwrap();
    let evidence_a = suitability(
        &store,
        &owner,
        &target_a,
        CognitiveCapability::PrimaryConversation,
    );
    let evidence_b = suitability(
        &store,
        &owner,
        &target_b,
        CognitiveCapability::PrimaryConversation,
    );
    let wrong_capability = suitability(
        &store,
        &owner,
        &target_b,
        CognitiveCapability::ImageUnderstanding,
    );
    let binding_a = store
        .bind_case_cognitive_target_authorized(
            &owner,
            case_id,
            "participant:model",
            CognitiveBindingRole::Primary,
            CognitiveCapability::PrimaryConversation,
            &target_a.target_id,
            &evidence_a.evidence_id,
            false,
        )
        .unwrap();
    let requirement = CognitiveCapabilityRequirement::new(
        case_id,
        "participant:model",
        CognitiveCapability::PrimaryConversation,
        "turn:replacement",
    )
    .unwrap();
    let plan_a = store
        .plan_case_cognitive_execution_authorized(
            &owner,
            case_id,
            "participant:model",
            &requirement,
        )
        .unwrap();

    // Provider routing can make a cognitive binding unusable, but it cannot
    // silently rewrite that canonical binding to another target.
    store
        .bind_case_provider_targets_authorized(
            &owner,
            case_id,
            "participant:model",
            vec![target_b.target_id.clone()],
            ProviderFailoverPolicy::None,
            1,
        )
        .unwrap();
    let stale = store
        .plan_case_cognitive_execution_authorized(
            &owner,
            case_id,
            "participant:model",
            &requirement,
        )
        .unwrap();
    assert_eq!(stale.route, CognitivePlanRoute::Unresolved);
    assert_eq!(
        stale.unresolved_reason,
        Some(CognitivePlanUnresolvedReason::PrimaryTargetNotAdmitted)
    );
    assert_eq!(
        store
            .bind_case_cognitive_target_authorized(
                &owner,
                case_id,
                "participant:model",
                CognitiveBindingRole::Primary,
                CognitiveCapability::PrimaryConversation,
                &target_b.target_id,
                &evidence_b.evidence_id,
                false,
            )
            .unwrap_err(),
        "cognitive_binding_replacement_requires_replace"
    );
    assert_eq!(
        store
            .bind_case_cognitive_target_authorized(
                &owner,
                case_id,
                "participant:model",
                CognitiveBindingRole::Primary,
                CognitiveCapability::PrimaryConversation,
                &target_b.target_id,
                &wrong_capability.evidence_id,
                true,
            )
            .unwrap_err(),
        "semantic_suitability_evidence_binding_mismatch"
    );
    assert!(store
        .bind_case_cognitive_target_authorized(
            &intruder,
            case_id,
            "participant:model",
            CognitiveBindingRole::Primary,
            CognitiveCapability::PrimaryConversation,
            &target_b.target_id,
            &evidence_b.evidence_id,
            true,
        )
        .is_err());
    let binding_b = store
        .bind_case_cognitive_target_authorized(
            &owner,
            case_id,
            "participant:model",
            CognitiveBindingRole::Primary,
            CognitiveCapability::PrimaryConversation,
            &target_b.target_id,
            &evidence_b.evidence_id,
            true,
        )
        .unwrap();
    assert_eq!(
        binding_b.replaces_binding_id,
        Some(binding_a.binding_id.clone())
    );
    let plan_b = store
        .plan_case_cognitive_execution_authorized(
            &owner,
            case_id,
            "participant:model",
            &requirement,
        )
        .unwrap();
    assert_ne!(plan_a.execution_lane_id, plan_b.execution_lane_id);
    let idempotent = store
        .bind_case_cognitive_target_authorized(
            &owner,
            case_id,
            "participant:model",
            CognitiveBindingRole::Primary,
            CognitiveCapability::PrimaryConversation,
            &target_b.target_id,
            &evidence_b.evidence_id,
            false,
        )
        .unwrap();
    assert_eq!(binding_b, idempotent);
    let unbound = store
        .unbind_case_cognitive_target_authorized(
            &owner,
            case_id,
            "participant:model",
            CognitiveBindingRole::Primary,
            CognitiveCapability::PrimaryConversation,
            "I02 explicit unbind",
        )
        .unwrap();
    assert!(unbound.state.cognitive_bindings.is_empty());
    assert_eq!(
        store
            .plan_case_cognitive_execution_authorized(
                &owner,
                case_id,
                "participant:model",
                &requirement,
            )
            .unwrap()
            .unresolved_reason,
        Some(CognitivePlanUnresolvedReason::PrimaryBindingMissing)
    );
    let rebuilt = store.rebuild_case_state(case_id).unwrap();
    assert!(rebuilt.cognitive_bindings.is_empty());
    assert!(store.verify_case_state(case_id).unwrap());
    println!(
        "i02_replacement: explicit=true envelope_invalidation=true lane_changed=true unbound=true replay=true unauthorized_principal=rejected"
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}
