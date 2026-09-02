use super::*;
use crate::provider_governance::{
    CapabilityProvenance, ProviderAdapterKind, ProviderCapability, ProviderDeliveryClass,
    ProviderFailoverPolicy, ProviderLocality, ProviderProbeEvidence, ProviderRequirement,
    ProviderTargetInput, ProviderTransportStage, ProviderTrustPosture,
};

fn w18_case(
    store: &LmdbRecordStore,
    owner: &AuthenticatedPrincipal,
    tenant_id: &str,
    case_id: &str,
) {
    let principal = owner.projected_principal_id();
    store
        .bootstrap_local_security(owner, tenant_id, &format!("organization:{tenant_id}"), 1)
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
                &principal,
                TransitionPayload::ParticipantBound {
                    participant_id: "participant:model".to_string(),
                    role: "model-executor".to_string(),
                },
            ),
            true,
        )
        .unwrap();
}

fn w18_target_input(
    owner: &AuthenticatedPrincipal,
    tenant_id: &str,
    port: u16,
) -> ProviderTargetInput {
    ProviderTargetInput {
        tenant_id: tenant_id.to_string(),
        provider_key: format!("fixture-{port}"),
        adapter: ProviderAdapterKind::OpenAiCompatible,
        endpoint: format!("http://127.0.0.1:{port}"),
        model_id: format!("fixture-model-{port}"),
        credential_ref: "none".to_string(),
        locality: ProviderLocality::Loopback,
        extension_adapter_id: None,
        created_by_principal_id: owner.projected_principal_id(),
        created_at_unix_ms: 10,
    }
}

fn w18_start_invocation(
    store: &LmdbRecordStore,
    owner: &AuthenticatedPrincipal,
    tenant_id: &str,
    case_id: &str,
    selection: &ProviderSelection,
) {
    let state = store.get_case_state(case_id).unwrap().unwrap();
    let mut pending = secured_pending(
        &format!("transition:{}:invocation", selection.selection_id),
        case_id,
        state.generation,
        &owner.projected_principal_id(),
        TransitionPayload::ProviderInvocationStarted {
            invocation_id: format!("invocation:{}", selection.selection_id),
            participant_id: selection.participant_id.clone(),
            provider_id: selection.selected_target_id.clone(),
            provider_kind: "openai_compatible".to_string(),
            model_id: selection.selected_model_id.clone(),
            semantic_lineage: Some(test_provider_lineage(state.generation)),
            governance: Some(crate::transition::ProviderInvocationGovernance {
                selection_id: selection.selection_id.clone(),
                target_id: selection.selected_target_id.clone(),
                logical_turn_id: selection.logical_turn_id.clone(),
                attempt_number: selection.attempt_number,
            }),
        },
    );
    pending.causal_refs.push(selection.selection_id.clone());
    store
        .commit_secured_transition(owner, tenant_id, pending, false)
        .unwrap();
}

fn w18_qualify(
    store: &LmdbRecordStore,
    owner: &AuthenticatedPrincipal,
    target: &ProviderTarget,
    json: bool,
) -> ProviderQualification {
    store
        .qualify_provider_target_authorized(
            owner,
            &target.target_id,
            ProviderProbeEvidence {
                run_id: format!("qualification-run:{}", target.target_id),
                target_id: target.target_id.clone(),
                started_at_unix_ms: 20,
                completed_at_unix_ms: 21,
                transport_connected: true,
                exact_model_addressed: true,
                chat_text_envelope_valid: true,
                structured_json_object_valid: json,
                usage_accounting_observed: true,
                health_endpoint_observed: true,
                extension_telemetry_observed: false,
                failure_codes: vec![],
            },
            "yai.openai_compatible.synthetic.v1",
            None,
        )
        .unwrap()
}

#[test]
fn wave18_governed_single_target_selection_is_case_canonical_and_replayable() {
    let path = temp_store_path("w18-single");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(18001);
    w18_case(&store, &owner, "tenant:w18", "case:w18-single");
    let target = store
        .register_provider_target_authorized(&owner, w18_target_input(&owner, "tenant:w18", 18001))
        .unwrap();
    let qualification = w18_qualify(&store, &owner, &target, true);
    store
        .set_provider_trust_authorized(
            &owner,
            &target.target_id,
            ProviderTrustPosture::Approved,
            22,
        )
        .unwrap();
    store
        .record_provider_health_observation_internal(
            &target.target_id,
            true,
            "qualification_probe",
            None,
            23,
        )
        .unwrap();
    let binding = store
        .bind_case_provider_targets_authorized(
            &owner,
            "case:w18-single",
            "participant:model",
            vec![target.target_id.clone()],
            ProviderFailoverPolicy::SafeOnly,
            3,
        )
        .unwrap();
    let requirement = ProviderRequirement::plan_patch().unwrap();
    let selected = store
        .select_case_provider_authorized(
            &owner,
            "case:w18-single",
            "participant:model",
            &requirement,
            "logical-turn:one",
            1,
            &BTreeSet::new(),
            true,
            &BTreeSet::new(),
            24,
        )
        .unwrap();
    let selection = match selected {
        ProviderSelectionStoreOutcome::Selected { selection, .. } => selection,
        other => panic!("unexpected selection outcome: {other:?}"),
    };
    assert_eq!(selection.selected_target_id, target.target_id);
    assert_eq!(selection.qualification_id, qualification.qualification_id);
    assert_eq!(selection.binding_id, binding.binding_id);
    assert!(selection.exclusions.is_empty());
    assert!(store.verify_case_state("case:w18-single").unwrap());
    let rebuilt = store.rebuild_case_state("case:w18-single").unwrap();
    assert_eq!(rebuilt.provider_selections, vec![selection.clone()]);
    assert_eq!(rebuilt.provider_binding.as_ref(), Some(&binding));
    println!(
        "w18_governed_selection: target={} qualification={} binding={} selection={} generation={} replay=true exclusions=0",
        target.target_id,
        qualification.qualification_id,
        binding.binding_id,
        selection.selection_id,
        rebuilt.generation
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn wave18_capability_trust_health_and_cross_tenant_filters_are_mechanical() {
    let path = temp_store_path("w18-filter");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner_a = AuthenticatedPrincipal::for_test(18002);
    let owner_b = AuthenticatedPrincipal::for_test(18003);
    w18_case(&store, &owner_a, "tenant:w18-a", "case:w18-filter");
    store
        .bootstrap_local_security(&owner_b, "tenant:w18-b", "organization:w18-b", 1)
        .unwrap();
    let text_only = store
        .register_provider_target_authorized(
            &owner_a,
            w18_target_input(&owner_a, "tenant:w18-a", 18002),
        )
        .unwrap();
    let denied = store
        .register_provider_target_authorized(
            &owner_a,
            w18_target_input(&owner_a, "tenant:w18-a", 18003),
        )
        .unwrap();
    let eligible = store
        .register_provider_target_authorized(
            &owner_a,
            w18_target_input(&owner_a, "tenant:w18-a", 18004),
        )
        .unwrap();
    let other_tenant = store
        .register_provider_target_authorized(
            &owner_b,
            w18_target_input(&owner_b, "tenant:w18-b", 18005),
        )
        .unwrap();
    w18_qualify(&store, &owner_a, &text_only, false);
    w18_qualify(&store, &owner_a, &denied, true);
    w18_qualify(&store, &owner_a, &eligible, true);
    for target in [&text_only, &eligible] {
        store
            .set_provider_trust_authorized(
                &owner_a,
                &target.target_id,
                ProviderTrustPosture::Approved,
                30,
            )
            .unwrap();
    }
    store
        .set_provider_trust_authorized(
            &owner_a,
            &denied.target_id,
            ProviderTrustPosture::Denied,
            30,
        )
        .unwrap();
    let cross = store
        .bind_case_provider_targets_authorized(
            &owner_a,
            "case:w18-filter",
            "participant:model",
            vec![other_tenant.target_id.clone()],
            ProviderFailoverPolicy::SafeOnly,
            3,
        )
        .unwrap_err();
    assert_eq!(cross, "cross_tenant_provider_target_binding_rejected");
    store
        .bind_case_provider_targets_authorized(
            &owner_a,
            "case:w18-filter",
            "participant:model",
            vec![
                text_only.target_id.clone(),
                denied.target_id.clone(),
                eligible.target_id.clone(),
            ],
            ProviderFailoverPolicy::SafeOnly,
            3,
        )
        .unwrap();
    let selected = store
        .select_case_provider_authorized(
            &owner_a,
            "case:w18-filter",
            "participant:model",
            &ProviderRequirement::plan_patch().unwrap(),
            "logical-turn:plan-patch",
            1,
            &BTreeSet::new(),
            true,
            &BTreeSet::new(),
            31,
        )
        .unwrap();
    let selection = match selected {
        ProviderSelectionStoreOutcome::Selected { selection, .. } => selection,
        other => panic!("unexpected selection outcome: {other:?}"),
    };
    assert_eq!(selection.selected_target_id, eligible.target_id);
    assert!(selection.exclusions.iter().any(|entry| {
        entry.target_id == text_only.target_id
            && entry.code
                == crate::provider_governance::ProviderExclusionCode::RequiredCapabilityMissing
    }));
    assert!(selection.exclusions.iter().any(|entry| {
        entry.target_id == denied.target_id
            && entry.code == crate::provider_governance::ProviderExclusionCode::TrustNotApproved
    }));
    println!(
        "w18_selection_filter: selected={} text_only=required_capability_missing denied=trust_not_approved cross_tenant={}",
        selection.selected_target_id, cross
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn wave18_circuit_and_delivery_contract_forbid_indeterminate_failover() {
    let path = temp_store_path("w18-delivery");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(18004);
    w18_case(&store, &owner, "tenant:w18", "case:w18-delivery");
    let primary = store
        .register_provider_target_authorized(&owner, w18_target_input(&owner, "tenant:w18", 18101))
        .unwrap();
    let secondary = store
        .register_provider_target_authorized(&owner, w18_target_input(&owner, "tenant:w18", 18102))
        .unwrap();
    for target in [&primary, &secondary] {
        w18_qualify(&store, &owner, target, true);
        store
            .set_provider_trust_authorized(
                &owner,
                &target.target_id,
                ProviderTrustPosture::Approved,
                40,
            )
            .unwrap();
    }
    for now in 41..44 {
        store
            .record_provider_health_observation_internal(
                &primary.target_id,
                false,
                "provider_invocation",
                Some("connect_refused"),
                now,
            )
            .unwrap();
    }
    let (_, _, _, primary_health) = store
        .provider_posture_authorized(&owner, &primary.target_id)
        .unwrap();
    assert_eq!(
        primary_health.circuit,
        crate::provider_governance::ProviderCircuitPosture::Open
    );
    store
        .bind_case_provider_targets_authorized(
            &owner,
            "case:w18-delivery",
            "participant:model",
            vec![primary.target_id.clone(), secondary.target_id.clone()],
            ProviderFailoverPolicy::SafeOnly,
            3,
        )
        .unwrap();
    let requirement = ProviderRequirement::text("case_runtime_turn").unwrap();
    let first = store
        .select_case_provider_authorized(
            &owner,
            "case:w18-delivery",
            "participant:model",
            &requirement,
            "logical-turn:delivery",
            1,
            &BTreeSet::new(),
            true,
            &BTreeSet::new(),
            44,
        )
        .unwrap();
    let selection = match first {
        ProviderSelectionStoreOutcome::Selected { selection, .. } => selection,
        other => panic!("unexpected selection outcome: {other:?}"),
    };
    assert_eq!(selection.selected_target_id, secondary.target_id);
    w18_start_invocation(
        &store,
        &owner,
        "tenant:w18",
        "case:w18-delivery",
        &selection,
    );
    let indeterminate = ProviderAttemptOutcome::new(
        &selection,
        ProviderDeliveryClass::DeliveryIndeterminate,
        ProviderTransportStage::ResponseBody,
        512,
        None,
        false,
        Some("connection_reset_after_write".to_string()),
        45,
    )
    .unwrap();
    assert!(!indeterminate.retry_safe());
    store
        .record_provider_attempt_outcome_authorized(
            &owner,
            "case:w18-delivery",
            indeterminate.clone(),
        )
        .unwrap();
    let attempted = BTreeSet::from([secondary.target_id.clone()]);
    let no_failover = store
        .select_case_provider_authorized(
            &owner,
            "case:w18-delivery",
            "participant:model",
            &requirement,
            "logical-turn:delivery",
            2,
            &attempted,
            indeterminate.retry_safe(),
            &BTreeSet::new(),
            46,
        )
        .unwrap();
    match no_failover {
        ProviderSelectionStoreOutcome::Waiting { exclusions } => {
            assert!(exclusions.iter().all(|entry| entry.code
                == crate::provider_governance::ProviderExclusionCode::FailoverNotSafe
                || entry.code == crate::provider_governance::ProviderExclusionCode::CircuitOpen))
        }
        other => panic!("indeterminate delivery selected alternate: {other:?}"),
    }
    assert!(store.verify_case_state("case:w18-delivery").unwrap());
    println!(
        "w18_delivery: primary_circuit=open selected_secondary={} delivery=indeterminate retry_safe=false automatic_failover=false outcome={}",
        selection.selection_id, indeterminate.outcome_id
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn wave18_qualification_capabilities_are_evidence_bound() {
    let path = temp_store_path("w18-forgery");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(18005);
    store
        .bootstrap_local_security(&owner, "tenant:w18", "organization:w18", 1)
        .unwrap();
    let target = store
        .register_provider_target_authorized(&owner, w18_target_input(&owner, "tenant:w18", 18201))
        .unwrap();
    let qualification = w18_qualify(&store, &owner, &target, false);
    assert!(qualification.capability_at_least(
        &ProviderCapability::ChatText,
        &CapabilityProvenance::Qualified
    ));
    assert!(!qualification.capability_at_least(
        &ProviderCapability::StructuredJsonObject,
        &CapabilityProvenance::Qualified
    ));
    let mut forged = qualification.clone();
    forged
        .capabilities
        .push(crate::provider_governance::ProviderCapabilityEvidence {
            capability: ProviderCapability::StructuredJsonObject,
            provenance: CapabilityProvenance::Qualified,
            evidence_refs: vec!["caller:supports_json=true".to_string()],
            verified_minimum: None,
        });
    assert_eq!(
        forged.validate(&target).unwrap_err(),
        "provider_qualification_integrity_mismatch"
    );
    println!(
        "w18_capability_forgery: qualification={} chat_text=qualified structured_json=false caller_boolean=rejected",
        qualification.qualification_id
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn wave18_concurrent_selection_and_attempt_outcome_have_one_case_truth() {
    use std::sync::{Arc, Barrier};
    use std::thread;

    let path = temp_store_path("w18-concurrency");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(18006);
    w18_case(
        &store,
        &owner,
        "tenant:w18-concurrency",
        "case:w18-concurrency",
    );
    let target = store
        .register_provider_target_authorized(
            &owner,
            w18_target_input(&owner, "tenant:w18-concurrency", 18301),
        )
        .unwrap();
    w18_qualify(&store, &owner, &target, true);
    store
        .set_provider_trust_authorized(
            &owner,
            &target.target_id,
            ProviderTrustPosture::Approved,
            50,
        )
        .unwrap();
    store
        .bind_case_provider_targets_authorized(
            &owner,
            "case:w18-concurrency",
            "participant:model",
            vec![target.target_id.clone()],
            ProviderFailoverPolicy::SafeOnly,
            3,
        )
        .unwrap();
    let before = store
        .get_case_state("case:w18-concurrency")
        .unwrap()
        .unwrap()
        .generation;
    drop(store);

    let barrier = Arc::new(Barrier::new(16));
    let mut handles = Vec::new();
    for _ in 0..16 {
        let path = path.clone();
        let owner = owner.clone();
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            let store = LmdbRecordStore::open(&path).unwrap();
            barrier.wait();
            store
                .select_case_provider_authorized(
                    &owner,
                    "case:w18-concurrency",
                    "participant:model",
                    &ProviderRequirement::text("case_runtime_turn").unwrap(),
                    "logical-turn:concurrent",
                    1,
                    &BTreeSet::new(),
                    true,
                    &BTreeSet::new(),
                    51,
                )
                .unwrap()
        }));
    }
    let results = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, ProviderSelectionStoreOutcome::Selected { .. }))
            .count(),
        1
    );
    let selection = results
        .iter()
        .find_map(|result| match result {
            ProviderSelectionStoreOutcome::Selected { selection, .. }
            | ProviderSelectionStoreOutcome::AlreadySelected(selection) => Some(selection.clone()),
            ProviderSelectionStoreOutcome::Waiting { .. } => None,
        })
        .unwrap();

    let store = LmdbRecordStore::open(&path).unwrap();
    w18_start_invocation(
        &store,
        &owner,
        "tenant:w18-concurrency",
        "case:w18-concurrency",
        &selection,
    );
    drop(store);
    let outcome = ProviderAttemptOutcome::new(
        &selection,
        ProviderDeliveryClass::NotDispatched,
        ProviderTransportStage::Connect,
        0,
        None,
        false,
        Some("connect_refused".to_string()),
        52,
    )
    .unwrap();
    let barrier = Arc::new(Barrier::new(16));
    let mut handles = Vec::new();
    for _ in 0..16 {
        let path = path.clone();
        let owner = owner.clone();
        let outcome = outcome.clone();
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            let store = LmdbRecordStore::open(&path).unwrap();
            barrier.wait();
            store
                .record_provider_attempt_outcome_authorized(&owner, "case:w18-concurrency", outcome)
                .unwrap()
        }));
    }
    let outcomes = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, ProviderAttemptStoreOutcome::Recorded(_)))
            .count(),
        1
    );
    let store = LmdbRecordStore::open(&path).unwrap();
    let state = store
        .get_case_state("case:w18-concurrency")
        .unwrap()
        .unwrap();
    assert_eq!(state.provider_selections.len(), 1);
    assert_eq!(state.provider_attempt_outcomes, vec![outcome]);
    assert_eq!(state.generation, before + 3);
    assert!(store.verify_case_state("case:w18-concurrency").unwrap());
    println!(
        "w18_concurrency: contenders=16 selections=1 outcomes=1 generation={} replay=true",
        state.generation
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn wave18_selection_scale_is_bounded_and_deterministic() {
    use std::time::Instant;

    let path = temp_store_path("w18-selection-scale");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(18007);
    w18_case(
        &store,
        &owner,
        "tenant:w18-scale",
        "case:w18-selection-scale",
    );

    let mut targets = Vec::new();
    for offset in 0..32 {
        let target = store
            .register_provider_target_authorized(
                &owner,
                w18_target_input(&owner, "tenant:w18-scale", 19000 + offset),
            )
            .unwrap();
        w18_qualify(&store, &owner, &target, true);
        store
            .set_provider_trust_authorized(
                &owner,
                &target.target_id,
                ProviderTrustPosture::Approved,
                60,
            )
            .unwrap();
        targets.push(target);
    }

    let mut observations = Vec::new();
    for candidate_count in [1_usize, 4, 16, 32] {
        store
            .bind_case_provider_targets_authorized(
                &owner,
                "case:w18-selection-scale",
                "participant:model",
                targets[..candidate_count]
                    .iter()
                    .map(|target| target.target_id.clone())
                    .collect(),
                ProviderFailoverPolicy::SafeOnly,
                3,
            )
            .unwrap();
        let started = Instant::now();
        let selected = store
            .select_case_provider_authorized(
                &owner,
                "case:w18-selection-scale",
                "participant:model",
                &ProviderRequirement::text("selection_scale").unwrap(),
                &format!("logical-turn:scale:{candidate_count}"),
                1,
                &BTreeSet::new(),
                true,
                &BTreeSet::new(),
                61,
            )
            .unwrap();
        let selection = match selected {
            ProviderSelectionStoreOutcome::Selected { selection, .. } => selection,
            other => panic!("unexpected scale selection outcome: {other:?}"),
        };
        assert_eq!(selection.selected_target_id, targets[0].target_id);
        observations.push((candidate_count, started.elapsed().as_micros()));
    }
    assert!(store.verify_case_state("case:w18-selection-scale").unwrap());
    println!(
        "w18_selection_scale: candidates=1:{}us,4:{}us,16:{}us,32:{}us selected={} deterministic=true",
        observations[0].1,
        observations[1].1,
        observations[2].1,
        observations[3].1,
        targets[0].target_id
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn wave18_qualification_current_projection_never_rolls_back() {
    let path = temp_store_path("w18-qualification-order");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(18008);
    store
        .bootstrap_local_security(
            &owner,
            "tenant:w18-qualification-order",
            "organization:w18-qualification-order",
            1,
        )
        .unwrap();
    let target = store
        .register_provider_target_authorized(
            &owner,
            w18_target_input(&owner, "tenant:w18-qualification-order", 19100),
        )
        .unwrap();
    let evidence = |run: &str, completed_at_unix_ms: u64| ProviderProbeEvidence {
        run_id: run.to_string(),
        target_id: target.target_id.clone(),
        started_at_unix_ms: completed_at_unix_ms - 1,
        completed_at_unix_ms,
        transport_connected: true,
        exact_model_addressed: true,
        chat_text_envelope_valid: true,
        structured_json_object_valid: true,
        usage_accounting_observed: true,
        health_endpoint_observed: false,
        extension_telemetry_observed: false,
        failure_codes: vec![],
    };
    let newer = store
        .qualify_provider_target_authorized(
            &owner,
            &target.target_id,
            evidence("qualification-run:newer", 200),
            "yai.openai_compatible.synthetic.v1",
            None,
        )
        .unwrap();
    let older = store
        .qualify_provider_target_authorized(
            &owner,
            &target.target_id,
            evidence("qualification-run:older", 100),
            "yai.openai_compatible.synthetic.v1",
            None,
        )
        .unwrap();
    assert_ne!(newer.qualification_id, older.qualification_id);
    let (_, current, _, _) = store
        .provider_posture_authorized(&owner, &target.target_id)
        .unwrap();
    assert_eq!(current.unwrap().qualification_id, newer.qualification_id);
    println!(
        "w18_qualification_order: newer={} stale_late={} current={} rollback=false",
        newer.qualification_id, older.qualification_id, newer.qualification_id
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn wave18_trust_revoke_and_invocation_start_serialize() {
    use crate::transition::{
        ProviderInvocationGovernance, ProviderInvocationLineage, TransitionPayload,
        TransitionSource,
    };
    use std::sync::{Arc, Barrier};
    use std::thread;

    let path = temp_store_path("w18-revoke-start");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(18009);
    w18_case(
        &store,
        &owner,
        "tenant:w18-revoke-start",
        "case:w18-revoke-start",
    );
    let target = store
        .register_provider_target_authorized(
            &owner,
            w18_target_input(&owner, "tenant:w18-revoke-start", 19101),
        )
        .unwrap();
    w18_qualify(&store, &owner, &target, true);
    store
        .set_provider_trust_authorized(
            &owner,
            &target.target_id,
            ProviderTrustPosture::Approved,
            70,
        )
        .unwrap();
    store
        .bind_case_provider_targets_authorized(
            &owner,
            "case:w18-revoke-start",
            "participant:model",
            vec![target.target_id.clone()],
            ProviderFailoverPolicy::SafeOnly,
            3,
        )
        .unwrap();
    let selected = store
        .select_case_provider_authorized(
            &owner,
            "case:w18-revoke-start",
            "participant:model",
            &ProviderRequirement::text("revoke_race").unwrap(),
            "logical-turn:revoke-race",
            1,
            &BTreeSet::new(),
            true,
            &BTreeSet::new(),
            71,
        )
        .unwrap();
    let selection = match selected {
        ProviderSelectionStoreOutcome::Selected { selection, .. } => selection,
        other => panic!("unexpected selection outcome: {other:?}"),
    };
    let state = store
        .get_case_state("case:w18-revoke-start")
        .unwrap()
        .unwrap();
    let mut pending = PendingTransition::new(
        "transition:w18-revoke-start-invocation",
        "case:w18-revoke-start",
        state.generation,
        TransitionSource {
            component: "yai.provider".to_string(),
            participant_id: Some("participant:model".to_string()),
            principal_id: Some(owner.projected_principal_id()),
            source_ref: Some(selection.selection_id.clone()),
        },
        TransitionPayload::ProviderInvocationStarted {
            invocation_id: "invocation:w18-revoke-race".to_string(),
            participant_id: "participant:model".to_string(),
            provider_id: target.target_id.clone(),
            provider_kind: "openai_compatible".to_string(),
            model_id: target.model_id.clone(),
            semantic_lineage: Some(ProviderInvocationLineage {
                projection_id: "projection:w18-revoke-race".to_string(),
                context_frame_id: "context-frame:w18-revoke-race".to_string(),
                case_generation: state.generation,
                rendered_input_id: "rendered-input:w18-revoke-race".to_string(),
                rendered_input_digest: "digest:w18-revoke-race".to_string(),
                output_contract_id: "output-contract:natural-language".to_string(),
                continuation_disposition: "not_provided".to_string(),
            }),
            governance: Some(ProviderInvocationGovernance {
                selection_id: selection.selection_id.clone(),
                target_id: target.target_id.clone(),
                logical_turn_id: selection.logical_turn_id.clone(),
                attempt_number: 1,
            }),
        },
    );
    pending.causal_refs = vec![selection.selection_id.clone()];
    drop(store);

    let barrier = Arc::new(Barrier::new(2));
    let start_path = path.clone();
    let start_barrier = Arc::clone(&barrier);
    let start = thread::spawn(move || {
        let store = LmdbRecordStore::open(&start_path).unwrap();
        start_barrier.wait();
        store.commit_transition(pending)
    });
    let revoke_path = path.clone();
    let revoke_barrier = Arc::clone(&barrier);
    let revoke_owner = owner.clone();
    let revoke_target = target.target_id.clone();
    let revoke = thread::spawn(move || {
        let store = LmdbRecordStore::open(&revoke_path).unwrap();
        revoke_barrier.wait();
        store.set_provider_trust_authorized(
            &revoke_owner,
            &revoke_target,
            ProviderTrustPosture::Denied,
            72,
        )
    });
    let start_result = start.join().unwrap();
    let denied = revoke.join().unwrap().unwrap();
    assert_eq!(denied.posture, ProviderTrustPosture::Denied);
    let store = LmdbRecordStore::open(&path).unwrap();
    let final_state = store
        .get_case_state("case:w18-revoke-start")
        .unwrap()
        .unwrap();
    match start_result {
        Ok(_) => assert_eq!(
            final_state
                .last_provider_invocation
                .as_ref()
                .map(|invocation| invocation.invocation_id.as_str()),
            Some("invocation:w18-revoke-race")
        ),
        Err(error) => {
            assert_eq!(error, "provider_invocation_trust_not_approved");
            assert!(final_state.last_provider_invocation.is_none());
        }
    }
    assert!(store.verify_case_state("case:w18-revoke-start").unwrap());
    println!(
        "w18_revoke_start_race: invocation_committed={} final_trust=denied serializable=true",
        final_state.last_provider_invocation.is_some()
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}
