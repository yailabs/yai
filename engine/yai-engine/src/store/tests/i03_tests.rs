use super::*;
use crate::cognitive::{
    CognitiveBindingRole, CognitiveCapability, CognitiveCapabilityRequirement, CognitivePlanRoute,
    SemanticEvidencePosture,
};
use crate::provider_governance::{
    CapabilityProvenance, ProviderAdapterKind, ProviderCapability, ProviderCapabilityRequirement,
    ProviderFailoverPolicy, ProviderLocality, ProviderProbeEvidence, ProviderRealizationShape,
    ProviderRequirement, ProviderTargetInput, ProviderTrustPosture,
};
use crate::transition::{ProviderInvocationGovernance, ProviderInvocationLineage};

pub(super) fn i03_setup_case(
    store: &LmdbRecordStore,
    owner: &AuthenticatedPrincipal,
    tenant_id: &str,
    case_id: &str,
) {
    store
        .bootstrap_local_security(owner, tenant_id, "organization:i03", 1)
        .unwrap();
    store.create_tenant_case(owner, tenant_id, case_id).unwrap();
    let participant = store
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
    let principal_id = owner.projected_principal_id();
    let link = crate::transition::PrincipalParticipantLink::new(
        case_id,
        tenant_id,
        &principal_id,
        "participant:model",
        &principal_id,
        2,
    )
    .unwrap();
    let mut pending = secured_pending(
        &format!("transition:{case_id}:participant-link"),
        case_id,
        participant.state.generation,
        &principal_id,
        TransitionPayload::ParticipantPrincipalLinked { link },
    );
    pending.causal_refs = vec![principal_id, "participant:model".to_string()];
    store
        .commit_secured_transition(owner, tenant_id, pending, true)
        .unwrap();
}

pub(super) fn i03_target(
    store: &LmdbRecordStore,
    owner: &AuthenticatedPrincipal,
    tenant_id: &str,
    name: &str,
    port: u16,
    realization_shapes: Vec<ProviderRealizationShape>,
) -> ProviderTarget {
    let target = store
        .register_provider_target_authorized(
            owner,
            ProviderTargetInput {
                tenant_id: tenant_id.to_string(),
                provider_key: name.to_string(),
                adapter: ProviderAdapterKind::OpenAiCompatible,
                endpoint: format!("http://127.0.0.1:{port}"),
                model_id: name.to_string(),
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
                run_id: format!("fixture:i03:{port}"),
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
                realization_shapes,
                failure_codes: Vec::new(),
            },
            "yai.i03.typed_provider_fixture.v1",
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

pub(super) fn i03_suitability(
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
            "yai.i03.semantic_fixture.v1",
            &format!("run:i03:{}", target.provider_key),
            vec![format!("fixture:i03:{}", target.provider_key)],
            "repository_fixture",
        )
        .unwrap()
}

pub(super) fn i03_provider_requirement(source: &str) -> ProviderRequirement {
    ProviderRequirement::new(
        &format!("cognitive_realization:{source}"),
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
    .unwrap()
}

#[test]
fn i03_exact_realization_selection_never_substitutes_provider_target() {
    let path = temp_store_path("i03-exact-selection");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(20301);
    let tenant_id = "tenant:i03";
    let case_id = "case:i03-exact-selection";
    i03_setup_case(&store, &owner, tenant_id, case_id);
    let primary = i03_target(
        &store,
        &owner,
        tenant_id,
        "whisper-name-primary",
        20301,
        vec![ProviderRealizationShape::TextToText],
    );
    let auxiliary = i03_target(
        &store,
        &owner,
        tenant_id,
        "not-whisper-auxiliary",
        20302,
        vec![ProviderRealizationShape::AudioWavToText],
    );
    let fallback = i03_target(
        &store,
        &owner,
        tenant_id,
        "whisper-looking-fallback",
        20303,
        vec![ProviderRealizationShape::AudioWavToText],
    );
    store
        .bind_case_provider_targets_authorized(
            &owner,
            case_id,
            "participant:model",
            vec![
                primary.target_id.clone(),
                auxiliary.target_id.clone(),
                fallback.target_id.clone(),
            ],
            ProviderFailoverPolicy::SafeOnly,
            3,
        )
        .unwrap();
    let primary_evidence = i03_suitability(
        &store,
        &owner,
        &primary,
        CognitiveCapability::PrimaryConversation,
    );
    let auxiliary_evidence = i03_suitability(
        &store,
        &owner,
        &auxiliary,
        CognitiveCapability::SpeechToText,
    );
    store
        .bind_case_cognitive_target_authorized(
            &owner,
            case_id,
            "participant:model",
            CognitiveBindingRole::Primary,
            CognitiveCapability::PrimaryConversation,
            &primary.target_id,
            &primary_evidence.evidence_id,
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
            &auxiliary_evidence.evidence_id,
            false,
        )
        .unwrap();
    let requirement = CognitiveCapabilityRequirement::new(
        case_id,
        "participant:model",
        CognitiveCapability::SpeechToText,
        "conversation-source:i03-audio",
    )
    .unwrap();
    let plan = store
        .plan_case_cognitive_execution_authorized(
            &owner,
            case_id,
            "participant:model",
            &requirement,
        )
        .unwrap();
    assert_eq!(plan.route, CognitivePlanRoute::Derived);
    assert_eq!(
        plan.selected_target_id.as_deref(),
        Some(auxiliary.target_id.as_str())
    );
    let provider_requirement = i03_provider_requirement(&requirement.requirement_id);
    let mismatched_shape = store
        .select_case_provider_exact_authorized(
            &owner,
            &plan,
            &provider_requirement,
            &ProviderRealizationShape::OrderedPngTextToText,
            &format!("cognitive-realization:{}", plan.plan_id),
            &BTreeSet::from(["none".to_string()]),
            &[],
        )
        .unwrap_err();
    assert_eq!(
        mismatched_shape,
        "cognitive_realization_capability_shape_mismatch"
    );
    assert!(store
        .get_case_state(case_id)
        .unwrap()
        .unwrap()
        .provider_selections
        .is_empty());
    let outcome = store
        .select_case_provider_exact_authorized(
            &owner,
            &plan,
            &provider_requirement,
            &ProviderRealizationShape::AudioWavToText,
            &format!("cognitive-realization:{}", plan.plan_id),
            &BTreeSet::from(["none".to_string()]),
            &[],
        )
        .unwrap();
    let selection = match outcome {
        ProviderSelectionStoreOutcome::Selected { selection, .. } => selection,
        other => panic!("unexpected exact selection: {other:?}"),
    };
    assert_eq!(selection.selected_target_id, auxiliary.target_id);
    assert_ne!(selection.selected_target_id, fallback.target_id);
    let transitions = store.list_case_transitions(case_id).unwrap();
    let selection_transition = transitions
        .iter()
        .find(|transition| matches!(&transition.payload, TransitionPayload::ProviderSelectionRecorded { selection: item } if item.selection_id == selection.selection_id))
        .unwrap();
    assert!(selection_transition.causal_refs.contains(&plan.plan_id));
    assert!(selection_transition
        .causal_refs
        .contains(&auxiliary_binding.binding_id));
    assert!(selection_transition
        .causal_refs
        .contains(&auxiliary_evidence.evidence_id));
    let stale_error = store
        .select_case_provider_exact_authorized(
            &owner,
            &plan,
            &provider_requirement,
            &ProviderRealizationShape::AudioWavToText,
            "cognitive-realization:stale-plan",
            &BTreeSet::from(["none".to_string()]),
            &[],
        )
        .unwrap_err();
    assert_eq!(stale_error, "cognitive_realization_plan_stale");
    assert_eq!(
        store
            .get_case_state(case_id)
            .unwrap()
            .unwrap()
            .provider_selections
            .len(),
        1
    );
    let fallback_evidence =
        i03_suitability(&store, &owner, &fallback, CognitiveCapability::SpeechToText);
    store
        .bind_case_cognitive_target_authorized(
            &owner,
            case_id,
            "participant:model",
            CognitiveBindingRole::Auxiliary,
            CognitiveCapability::SpeechToText,
            &fallback.target_id,
            &fallback_evidence.evidence_id,
            true,
        )
        .unwrap();
    let current = store.get_case_state(case_id).unwrap().unwrap();
    let mut invocation = PendingTransition::new(
        "transition:i03-stale-cognitive-invocation",
        case_id,
        current.generation,
        TransitionSource {
            component: "yai.cognitive_realization".to_string(),
            participant_id: Some("participant:model".to_string()),
            principal_id: Some(owner.projected_principal_id()),
            source_ref: Some(selection.selection_id.clone()),
        },
        TransitionPayload::ProviderInvocationStarted {
            invocation_id: "invocation:i03-stale-cognitive-binding".to_string(),
            participant_id: "participant:model".to_string(),
            provider_id: auxiliary.target_id.clone(),
            provider_kind: "openai_compatible".to_string(),
            model_id: auxiliary.model_id.clone(),
            semantic_lineage: Some(ProviderInvocationLineage {
                projection_id: "projection:i03-stale-cognitive-binding".to_string(),
                context_frame_id: "context-frame:i03-stale-cognitive-binding".to_string(),
                case_generation: current.generation,
                rendered_input_id: "rendered-input:i03-stale-cognitive-binding".to_string(),
                rendered_input_digest: "sha256:i03-stale-cognitive-binding".to_string(),
                output_contract_id: "output-contract:natural-language".to_string(),
                continuation_disposition: "not_provided".to_string(),
            }),
            governance: Some(ProviderInvocationGovernance {
                selection_id: selection.selection_id.clone(),
                target_id: auxiliary.target_id.clone(),
                logical_turn_id: selection.logical_turn_id.clone(),
                attempt_number: selection.attempt_number,
            }),
        },
    );
    invocation.causal_refs = vec![selection.selection_id.clone()];
    assert_eq!(
        store.commit_transition(invocation).unwrap_err(),
        "provider_invocation_cognitive_binding_stale"
    );
    println!(
        "i03_exact_selection: plan={} selected={} fallback={} substituted=false capability_shape_mismatch=rejected stale_plan=rejected stale_binding_dispatch=rejected",
        plan.plan_id, selection.selected_target_id, fallback.target_id
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn i03_semantic_suitability_without_wire_shape_refuses_before_selection() {
    let path = temp_store_path("i03-mechanical-refusal");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(20302);
    let tenant_id = "tenant:i03-refusal";
    let case_id = "case:i03-mechanical-refusal";
    i03_setup_case(&store, &owner, tenant_id, case_id);
    let primary = i03_target(
        &store,
        &owner,
        tenant_id,
        "speech-looking-primary",
        20304,
        vec![ProviderRealizationShape::TextToText],
    );
    let auxiliary = i03_target(
        &store,
        &owner,
        tenant_id,
        "whisper-looking-without-wire-proof",
        20305,
        Vec::new(),
    );
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
    let primary_evidence = i03_suitability(
        &store,
        &owner,
        &primary,
        CognitiveCapability::PrimaryConversation,
    );
    let auxiliary_evidence = i03_suitability(
        &store,
        &owner,
        &auxiliary,
        CognitiveCapability::SpeechToText,
    );
    store
        .bind_case_cognitive_target_authorized(
            &owner,
            case_id,
            "participant:model",
            CognitiveBindingRole::Primary,
            CognitiveCapability::PrimaryConversation,
            &primary.target_id,
            &primary_evidence.evidence_id,
            false,
        )
        .unwrap();
    store
        .bind_case_cognitive_target_authorized(
            &owner,
            case_id,
            "participant:model",
            CognitiveBindingRole::Auxiliary,
            CognitiveCapability::SpeechToText,
            &auxiliary.target_id,
            &auxiliary_evidence.evidence_id,
            false,
        )
        .unwrap();
    let requirement = CognitiveCapabilityRequirement::new(
        case_id,
        "participant:model",
        CognitiveCapability::SpeechToText,
        "conversation-source:i03-refusal",
    )
    .unwrap();
    let plan = store
        .plan_case_cognitive_execution_authorized(
            &owner,
            case_id,
            "participant:model",
            &requirement,
        )
        .unwrap();
    assert_eq!(plan.route, CognitivePlanRoute::Derived);
    let error = store
        .select_case_provider_exact_authorized(
            &owner,
            &plan,
            &i03_provider_requirement(&requirement.requirement_id),
            &ProviderRealizationShape::AudioWavToText,
            &format!("cognitive-realization:{}", plan.plan_id),
            &BTreeSet::from(["none".to_string()]),
            &[],
        )
        .unwrap_err();
    assert_eq!(error, "cognitive_realization_shape_not_qualified");
    assert!(store
        .get_case_state(case_id)
        .unwrap()
        .unwrap()
        .provider_selections
        .is_empty());
    println!(
        "i03_mechanical_refusal: semantic_suitability=present wire_shape=absent provider_selection=absent"
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn i03_transition_v14_store_marker_upgrades_to_v15() {
    let path = temp_store_path("i03-transition-v14-upgrade");
    let store = LmdbRecordStore::open(&path).unwrap();
    let mut txn = store.env.begin_rw_txn().unwrap();
    txn.put(
        store.schema_meta,
        &"meta:canonical_transition_schema",
        &TRANSITION_SCHEMA_V14,
        WriteFlags::empty(),
    )
    .unwrap();
    txn.commit().unwrap();
    drop(store);

    let reopened = LmdbRecordStore::open(&path).unwrap();
    let txn = reopened.env.begin_ro_txn().unwrap();
    assert_eq!(
        txn.get(reopened.schema_meta, &"meta:canonical_transition_schema")
            .unwrap(),
        TRANSITION_SCHEMA.as_bytes()
    );
    println!("i03_schema_upgrade: transition_v14_to_v15=pass");
    drop(txn);
    drop(reopened);
    fs::remove_dir_all(path).unwrap();
}
