//! No network: canonical LMDB, explicit deterministic qualification evidence.
use super::i03_tests::{i03_provider_requirement, i03_setup_case, i03_suitability, i03_target};
use super::*;
use crate::cognitive::{
    assess_lane_continuation, CognitiveCandidateExclusion, CognitivePlanRoute,
    LaneContinuationPosture, LaneContinuationReference,
};
use crate::provider_governance::{ProviderFailoverPolicy, ProviderTrustPosture};
use crate::transition::{ProviderInvocationGovernance, ProviderInvocationLineage};

#[test]
fn arbitration_changes_between_selection_and_invocation_fail_closed() {
    let path = temp_store_path("i05-admission-race");
    let owner = AuthenticatedPrincipal::for_test(20521);
    let store = LmdbRecordStore::open(&path).unwrap();
    let case = "case:i05-race";
    let tenant = "tenant:i05-race";
    let participant = "participant:model";
    i03_setup_case(&store, &owner, tenant, case);
    let a = i03_target(
        &store,
        &owner,
        tenant,
        "preferred",
        20521,
        vec![ProviderRealizationShape::TextToText],
    );
    let b = i03_target(
        &store,
        &owner,
        tenant,
        "second",
        20522,
        vec![ProviderRealizationShape::TextToText],
    );
    let ea = i03_suitability(&store, &owner, &a, CognitiveCapability::PrimaryConversation);
    let eb = i03_suitability(&store, &owner, &b, CognitiveCapability::PrimaryConversation);
    store
        .bind_case_provider_targets_authorized(
            &owner,
            case,
            participant,
            vec![a.target_id.clone(), b.target_id.clone()],
            ProviderFailoverPolicy::SafeOnly,
            2,
        )
        .unwrap();
    let binding = store
        .bind_case_cognitive_candidates_authorized(
            &owner,
            case,
            participant,
            CognitiveBindingRole::Primary,
            CognitiveCapability::PrimaryConversation,
            vec![
                (a.target_id.clone(), ea.evidence_id),
                (b.target_id.clone(), eb.evidence_id),
            ],
            false,
        )
        .unwrap();
    // Independently produced operational failure: suitability/policy remain intact.
    for _ in 0..3 {
        store
            .record_provider_health_observation_internal(
                &a.target_id,
                false,
                "i05_deterministic_fixture",
                Some("fixture_connection_failure"),
                0,
            )
            .unwrap();
    }
    let req = CognitiveCapabilityRequirement::new(
        case,
        participant,
        CognitiveCapability::PrimaryConversation,
        "source:race",
    )
    .unwrap();
    let plan = store
        .plan_case_cognitive_execution_for_shape_authorized(
            &owner,
            case,
            participant,
            &req,
            Some(&ProviderRealizationShape::TextToText),
        )
        .unwrap();
    assert_eq!(plan.selected_target_id.as_ref(), Some(&b.target_id));
    assert!(plan.arbitration.as_ref().unwrap().candidates[0]
        .exclusions
        .contains(&CognitiveCandidateExclusion::CircuitOpen));
    let selected = store
        .select_case_provider_exact_authorized(
            &owner,
            &plan,
            &i03_provider_requirement(&req.requirement_id),
            &ProviderRealizationShape::TextToText,
            &format!("cognitive-realization:{}", plan.plan_id),
            &BTreeSet::new(),
            &[],
        )
        .unwrap();
    let selection = match selected {
        ProviderSelectionStoreOutcome::Selected { selection, .. } => selection,
        other => panic!("{other:?}"),
    };
    // Recovery before invocation makes A eligible again. B may not dispatch
    // under a stale arbitration, even though B itself is still fully qualified.
    store
        .record_provider_health_observation_internal(
            &a.target_id,
            true,
            "i05_deterministic_fixture",
            None,
            0,
        )
        .unwrap();
    let state = store.get_case_state(case).unwrap().unwrap();
    assert_eq!(state.cognitive_bindings, vec![binding]);
    let fresh = store
        .plan_case_cognitive_execution_for_shape_authorized(
            &owner,
            case,
            participant,
            &req,
            Some(&ProviderRealizationShape::TextToText),
        )
        .unwrap();
    assert_eq!(fresh.selected_target_id.as_ref(), Some(&a.target_id));
    let mut invocation = PendingTransition::new(
        "transition:i05-stale-arbitration",
        case,
        state.generation,
        TransitionSource {
            component: "yai.cognitive_realization".into(),
            participant_id: Some(participant.into()),
            principal_id: Some(owner.projected_principal_id()),
            source_ref: Some(selection.selection_id.clone()),
        },
        TransitionPayload::ProviderInvocationStarted {
            invocation_id: "invocation:i05-stale-arbitration".into(),
            participant_id: participant.into(),
            provider_id: b.target_id.clone(),
            provider_kind: "openai_compatible".into(),
            model_id: b.model_id,
            semantic_lineage: Some(ProviderInvocationLineage {
                projection_id: "projection:i05-race".into(),
                context_frame_id: "context-frame:i05-race".into(),
                case_generation: state.generation,
                rendered_input_id: "rendered-input:i05-race".into(),
                rendered_input_digest: "sha256:i05-race".into(),
                output_contract_id: "output-contract:natural-language".into(),
                continuation_disposition: "not_provided".into(),
            }),
            governance: Some(ProviderInvocationGovernance {
                selection_id: selection.selection_id.clone(),
                target_id: b.target_id,
                logical_turn_id: selection.logical_turn_id,
                attempt_number: 1,
            }),
        },
    );
    invocation.causal_refs = vec![selection.selection_id];
    assert_eq!(
        store.commit_transition(invocation).unwrap_err(),
        "provider_invocation_arbitration_snapshot_stale"
    );
    assert_eq!(state, store.get_case_state(case).unwrap().unwrap());
    println!("i05_no_provider: health_excludes_without_semantic_mutation=true recovery_rearbitrates=true selection_to_invocation_race=blocked zero_invocations=true");
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn arbitration_replay_shape_trust_envelope_and_pinning() {
    let path = temp_store_path("i05-arbitration");
    let owner = AuthenticatedPrincipal::for_test(20501);
    let store = LmdbRecordStore::open(&path).unwrap();
    let case = "case:i05";
    let tenant = "tenant:i05";
    let participant = "participant:model";
    i03_setup_case(&store, &owner, tenant, case);
    let first = i03_target(
        &store,
        &owner,
        tenant,
        "vision-whisper-best-name",
        20501,
        vec![ProviderRealizationShape::TextToText],
    );
    let second = i03_target(
        &store,
        &owner,
        tenant,
        "plain-target",
        20502,
        vec![
            ProviderRealizationShape::TextToText,
            ProviderRealizationShape::AudioWavToText,
        ],
    );
    store
        .bind_case_provider_targets_authorized(
            &owner,
            case,
            participant,
            vec![first.target_id.clone(), second.target_id.clone()],
            ProviderFailoverPolicy::SafeOnly,
            2,
        )
        .unwrap();
    let a = i03_suitability(
        &store,
        &owner,
        &first,
        CognitiveCapability::PrimaryConversation,
    );
    let b = i03_suitability(
        &store,
        &owner,
        &second,
        CognitiveCapability::PrimaryConversation,
    );
    let candidates = vec![
        (first.target_id.clone(), a.evidence_id.clone()),
        (second.target_id.clone(), b.evidence_id.clone()),
    ];
    let binding = store
        .bind_case_cognitive_candidates_authorized(
            &owner,
            case,
            participant,
            CognitiveBindingRole::Primary,
            CognitiveCapability::PrimaryConversation,
            candidates.clone(),
            false,
        )
        .unwrap();
    assert_eq!(
        binding.schema,
        crate::cognitive::CASE_COGNITIVE_BINDING_SCHEMA_V2
    );
    let requirement = CognitiveCapabilityRequirement::new(
        case,
        participant,
        CognitiveCapability::PrimaryConversation,
        "source:i05",
    )
    .unwrap();
    let plan = |store: &LmdbRecordStore, shape: &ProviderRealizationShape| {
        store
            .plan_case_cognitive_execution_for_shape_authorized(
                &owner,
                case,
                participant,
                &requirement,
                Some(shape),
            )
            .unwrap()
    };
    let text = plan(&store, &ProviderRealizationShape::TextToText);
    assert_eq!(text.selected_target_id.as_ref(), Some(&first.target_id));
    let audio = plan(&store, &ProviderRealizationShape::AudioWavToText);
    assert_eq!(audio.selected_target_id.as_ref(), Some(&second.target_id));
    assert!(audio.arbitration.as_ref().unwrap().candidates[0]
        .exclusions
        .contains(&CognitiveCandidateExclusion::MechanicalShapeUnsupported));
    assert_ne!(text.execution_lane_id, audio.execution_lane_id);
    let continuation = LaneContinuationReference {
        execution_lane_id: text.execution_lane_id.clone().unwrap(),
        target_id: first.target_id.clone(),
        runtime_id: "opaque:runtime".into(),
        opaque_reference: "opaque".into(),
    };
    assert_eq!(
        assess_lane_continuation(&audio, Some(&continuation)),
        LaneContinuationPosture::RejectedCrossLane
    );
    assert_eq!(
        assess_lane_continuation(&audio, None),
        LaneContinuationPosture::NotProvidedSemanticReconstruction
    );
    let before = store.get_case_state(case).unwrap().unwrap();
    assert_eq!(
        before.cognitive_bindings,
        store.rebuild_case_state(case).unwrap().cognitive_bindings
    );
    drop(store);
    let store = LmdbRecordStore::open(&path).unwrap();
    assert_eq!(text, plan(&store, &ProviderRealizationShape::TextToText));
    store
        .set_provider_trust_authorized(&owner, &first.target_id, ProviderTrustPosture::Denied, 1)
        .unwrap();
    let next = plan(&store, &ProviderRealizationShape::TextToText);
    assert_eq!(next.selected_target_id.as_ref(), Some(&second.target_id));
    assert!(next.arbitration.as_ref().unwrap().candidates[0]
        .exclusions
        .contains(&CognitiveCandidateExclusion::TrustNotApproved));
    let stale = store
        .select_case_provider_exact_authorized(
            &owner,
            &text,
            &i03_provider_requirement(&requirement.requirement_id),
            &ProviderRealizationShape::TextToText,
            &format!("cognitive-realization:{}", text.plan_id),
            &BTreeSet::new(),
            &[],
        )
        .unwrap_err();
    assert_eq!(stale, "cognitive_realization_arbitration_snapshot_stale");
    assert!(store
        .get_case_state(case)
        .unwrap()
        .unwrap()
        .provider_selections
        .is_empty());
    store
        .set_provider_trust_authorized(&owner, &first.target_id, ProviderTrustPosture::Approved, 1)
        .unwrap();
    assert_eq!(
        plan(&store, &ProviderRealizationShape::TextToText)
            .selected_target_id
            .as_ref(),
        Some(&first.target_id)
    );
    // Removal is an eligibility change, not an implicit rewrite of the policy.
    store
        .bind_case_provider_targets_authorized(
            &owner,
            case,
            participant,
            vec![second.target_id.clone()],
            ProviderFailoverPolicy::SafeOnly,
            1,
        )
        .unwrap();
    let removed = plan(&store, &ProviderRealizationShape::TextToText);
    assert_eq!(removed.selected_target_id.as_ref(), Some(&second.target_id));
    assert!(removed.arbitration.as_ref().unwrap().candidates[0]
        .exclusions
        .contains(&CognitiveCandidateExclusion::ProviderEnvelopeMismatch));
    assert_eq!(
        store
            .get_case_state(case)
            .unwrap()
            .unwrap()
            .cognitive_bindings,
        vec![binding.clone()]
    );
    // Restore envelope explicitly and pin: no alternative remains implicit.
    store
        .bind_case_provider_targets_authorized(
            &owner,
            case,
            participant,
            vec![first.target_id.clone(), second.target_id.clone()],
            ProviderFailoverPolicy::SafeOnly,
            2,
        )
        .unwrap();
    let pinned = store
        .bind_case_cognitive_target_authorized(
            &owner,
            case,
            participant,
            CognitiveBindingRole::Primary,
            CognitiveCapability::PrimaryConversation,
            &first.target_id,
            &a.evidence_id,
            true,
        )
        .unwrap();
    assert!(pinned.target_policy.is_none());
    assert_eq!(
        plan(&store, &ProviderRealizationShape::AudioWavToText).route,
        CognitivePlanRoute::Unresolved
    );
    println!("i05_no_provider: replay=exact preferred={} shape_fallback={} trust_and_envelope=excluded pinned=no_fallback stale_plan=no_selection lane_isolated=true", first.target_id, second.target_id);
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn arbitration_policy_security_bounds_and_historical_readers() {
    let path = temp_store_path("i05-policy-security");
    let owner = AuthenticatedPrincipal::for_test(20511);
    let store = LmdbRecordStore::open(&path).unwrap();
    i03_setup_case(&store, &owner, "tenant:i05-security", "case:i05-security");
    let a = i03_target(
        &store,
        &owner,
        "tenant:i05-security",
        "deepseek-name",
        20511,
        vec![ProviderRealizationShape::TextToText],
    );
    let b = i03_target(
        &store,
        &owner,
        "tenant:i05-security",
        "whisper-name",
        20512,
        vec![ProviderRealizationShape::TextToText],
    );
    let ea = i03_suitability(&store, &owner, &a, CognitiveCapability::PrimaryConversation);
    let eb = i03_suitability(&store, &owner, &b, CognitiveCapability::SpeechToText);
    store
        .bind_case_provider_targets_authorized(
            &owner,
            "case:i05-security",
            "participant:model",
            vec![a.target_id.clone(), b.target_id.clone()],
            ProviderFailoverPolicy::SafeOnly,
            2,
        )
        .unwrap();
    let before = store.get_case_state("case:i05-security").unwrap().unwrap();
    for candidates in [
        vec![],
        vec![(a.target_id.clone(), ea.evidence_id.clone()); 9],
        vec![(a.target_id.clone(), ea.evidence_id.clone()); 2],
        vec![
            (a.target_id.clone(), ea.evidence_id.clone()),
            (b.target_id.clone(), eb.evidence_id.clone()),
        ],
        vec![(b.target_id.clone(), ea.evidence_id.clone())],
    ] {
        assert!(store
            .bind_case_cognitive_candidates_authorized(
                &owner,
                "case:i05-security",
                "participant:model",
                CognitiveBindingRole::Primary,
                CognitiveCapability::PrimaryConversation,
                candidates,
                false
            )
            .is_err());
        assert_eq!(
            before,
            store.get_case_state("case:i05-security").unwrap().unwrap()
        );
    }
    let eb = i03_suitability(&store, &owner, &b, CognitiveCapability::PrimaryConversation);
    let candidates = vec![
        (a.target_id.clone(), ea.evidence_id.clone()),
        (b.target_id.clone(), eb.evidence_id.clone()),
    ];
    let spoof = AuthenticatedPrincipal::for_test(99905);
    assert!(store
        .bind_case_cognitive_candidates_authorized(
            &spoof,
            "case:i05-security",
            "participant:model",
            CognitiveBindingRole::Primary,
            CognitiveCapability::PrimaryConversation,
            candidates.clone(),
            false
        )
        .is_err());
    assert!(store
        .bind_case_cognitive_candidates_authorized(
            &owner,
            "case:i05-security",
            "participant:hidden",
            CognitiveBindingRole::Primary,
            CognitiveCapability::PrimaryConversation,
            candidates.clone(),
            false
        )
        .is_err());
    let binding = store
        .bind_case_cognitive_candidates_authorized(
            &owner,
            "case:i05-security",
            "participant:model",
            CognitiveBindingRole::Primary,
            CognitiveCapability::PrimaryConversation,
            candidates.clone(),
            false,
        )
        .unwrap();
    let after = store.get_case_state("case:i05-security").unwrap().unwrap();
    assert_eq!(
        binding,
        store
            .bind_case_cognitive_candidates_authorized(
                &owner,
                "case:i05-security",
                "participant:model",
                CognitiveBindingRole::Primary,
                CognitiveCapability::PrimaryConversation,
                candidates,
                false
            )
            .unwrap()
    );
    assert_eq!(
        after,
        store.get_case_state("case:i05-security").unwrap().unwrap()
    );
    let history = store.list_case_transitions("case:i05-security").unwrap();
    let mut old = history.last().unwrap().clone();
    old.schema = TRANSITION_SCHEMA_V15.to_string();
    assert_eq!(
        old.validate().unwrap_err(),
        "arbitrated_binding_requires_yai_transition_v16"
    );
    let mut corrupt = binding.clone();
    corrupt.target_id = b.target_id.clone();
    assert!(corrupt.validate().is_err());
    // Old marker upgrade, without rewriting any canonical row.
    let mut txn = store.env.begin_rw_txn().unwrap();
    txn.put(
        store.schema_meta,
        &"meta:canonical_transition_schema",
        &TRANSITION_SCHEMA_V15,
        WriteFlags::empty(),
    )
    .unwrap();
    txn.put(
        store.schema_meta,
        &"meta:case_state_schema",
        &CASE_STATE_SCHEMA_V13,
        WriteFlags::empty(),
    )
    .unwrap();
    txn.commit().unwrap();
    drop(store);
    let store = LmdbRecordStore::open(&path).unwrap();
    assert_eq!(
        history,
        store.list_case_transitions("case:i05-security").unwrap()
    );
    assert_eq!(
        vec![binding],
        store
            .rebuild_case_state("case:i05-security")
            .unwrap()
            .cognitive_bindings
    );
    println!("i05_no_provider: policy_bounds=8 duplicate=reject wrong_capability=reject wrong_target=reject spoofed_principal=reject hidden_participant=reject atomic_failure=true v15_v13_markers=upgraded history=unchanged");
    drop(store);
    fs::remove_dir_all(path).unwrap();
}
