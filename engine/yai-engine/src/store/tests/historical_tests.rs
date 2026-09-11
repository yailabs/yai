use super::*;
use crate::semantic_state::historical::{
    HistoricalCoordinate, HistoricalRequest, HistoricalSemanticView,
};

#[path = "experience_tests.rs"]
mod experience_tests;

#[path = "recall_tests.rs"]
mod recall_tests;

fn request(generation: u64) -> HistoricalRequest {
    let mut r = HistoricalRequest::inspection(HistoricalCoordinate::Generation(generation), HUMAN);
    r.consumer = "model".into();
    r.view_kind = "model_context".into();
    r
}

fn view(world: &World, generation: u64) -> HistoricalSemanticView {
    world
        .store
        .historical_semantic_view_authorized(&world.owner, CASE, request(generation), None)
        .unwrap()
}

fn replace(world: &World, version: &str, effect: &str, review: bool) -> (String, u64) {
    let state = world.store.get_case_state(CASE).unwrap().unwrap();
    let source = world
        .store
        .get_policy_source(&state.policy_bindings[0].source_id)
        .unwrap()
        .unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&source.content_utf8).unwrap();
    value["source_version"] = version.into();
    value["rules"][0]["effect"] = effect.into();
    value["rules"].as_array_mut().unwrap().push(serde_json::json!({
        "kind":"review_requirement", "rule_id":"review-current", "operation_kind":"filesystem.read",
        "resource_kind":"filesystem", "required":review, "reason":"historical review oracle"
    }));
    // Each version is independently compiled from one bounded exact source.
    let rules = value["rules"].as_array_mut().unwrap();
    let last = rules.pop().unwrap();
    rules.retain(|r| r["rule_id"] != "review-current");
    rules.push(last);
    rules.retain(|r| r["rule_id"] != "reviewer-current");
    if review {
        rules.push(
            serde_json::json!({"kind":"authority_requirement", "rule_id":"reviewer-current",
        "operation_kind":"filesystem.read", "resource_kind":"filesystem", "subject":"reviewer",
        "required_role":"operation-reviewer", "reason":"explicit current eligible reviewer"}),
        );
    }
    let compilation = scope_policy_compilation(
        &compile_policy_source(&serde_json::to_vec(&value).unwrap()).unwrap(),
        TENANT,
        "organization:resource-contract",
    )
    .unwrap();
    let artifact = compilation.artifact.artifact_id.clone();
    world
        .store
        .ingest_tenant_policy_compilation(&world.owner, TENANT, &compilation)
        .unwrap();
    world
        .store
        .validate_tenant_policy_artifact(&world.owner, &artifact, "exact historical test source")
        .unwrap();
    world
        .store
        .publish_tenant_policy_artifact(&world.owner, &artifact, "explicit test publication")
        .unwrap();
    world
        .store
        .replace_tenant_case_policy(
            &world.owner,
            CASE,
            &state.policy_bindings[0].binding_id,
            &artifact,
            state.generation,
            "replace normative version",
        )
        .unwrap();
    (
        artifact,
        world
            .store
            .get_case_state(CASE)
            .unwrap()
            .unwrap()
            .generation,
    )
}

#[test]
fn historical_policy_chronology_late_observation_and_current_permission() {
    let world = World::new();
    let state = world.store.get_case_state(CASE).unwrap().unwrap();
    world
        .store
        .commit_secured_transition(
            &world.owner,
            TENANT,
            secured_pending(
                "transition:historical-reviewer-role",
                CASE,
                state.generation,
                &world.owner.projected_principal_id(),
                TransitionPayload::ParticipantBound {
                    participant_id: HUMAN.into(),
                    role: "operation-reviewer".into(),
                },
            ),
            true,
        )
        .unwrap();
    let p1 = world.store.get_case_state(CASE).unwrap().unwrap();
    let before = view(&world, p1.policy_bindings[0].bound_at_case_generation - 1);
    assert!(before.state_then.policy_bindings.is_empty());
    assert!(before.normative_then.effective_policy.is_none());
    let allowed = world.operation("request:historical-allow", "src/retry.txt");
    let (d1, c1) = world
        .store
        .derive_and_commit_policy_decision(CASE, &allowed.operation_id)
        .unwrap();
    assert_eq!(d1.outcome, DecisionOutcome::Allow);
    let read = world
        .store
        .admit_resource_read_authorized(&world.owner, CASE, &allowed.operation_id)
        .unwrap();
    let mut result = world.result(&read);
    result["reported_earlier_occurrence"] =
        serde_json::json!({"case_generation":1,"description":"older constraint discovered now"});
    let observation = world
        .store
        .record_resource_observation_authorized(&world.owner, &read, result)
        .unwrap();
    let observation_generation = world
        .store
        .get_case_state(CASE)
        .unwrap()
        .unwrap()
        .generation;
    assert!(
        !serde_json::to_string(&view(&world, c1.state.generation).known_by_then)
            .unwrap()
            .contains(&observation.observation_id)
    );
    let late = view(&world, observation_generation);
    let recorded = late.known_by_then.iter().find(|e| matches!(&e.payload,
        TransitionPayload::ResourceObservationRecorded { observation: o } if o.observation_id == observation.observation_id)).unwrap();
    assert_eq!(
        recorded.observed_at_unix_ms,
        Some(observation.observed_at_unix_ms)
    );
    assert_eq!(recorded.recorded_generation, observation_generation);
    assert_eq!(recorded.occurred_at_unix_ms, None); // result JSON is not a timestamp authority

    let (_, review_generation) = replace(&world, "2", "allow", true);
    let proposed = world.operation("request:historical-review", "src/retry.txt");
    let (review_decision, reviewed) = world
        .store
        .derive_and_commit_policy_decision(CASE, &proposed.operation_id)
        .unwrap();
    assert_eq!(
        review_decision.outcome,
        DecisionOutcome::RequireReview,
        "{review_decision:?}"
    );
    let review =
        build_policy_review_request(&proposed, &review_decision, reviewed.state.generation)
            .unwrap();
    let mut pending = secured_pending(
        "transition:historical-review",
        CASE,
        reviewed.state.generation,
        &world.owner.projected_principal_id(),
        TransitionPayload::ReviewRequested {
            review: review.clone(),
        },
    );
    pending.scope = Some(proposed.scope.clone());
    pending.causal_refs = vec![
        proposed.operation_id.clone(),
        review_decision.decision_id.clone(),
        review.decision_basis_id.clone(),
        review.effective_policy_id.clone(),
    ];
    world
        .store
        .commit_secured_transition(&world.owner, TENANT, pending, true)
        .unwrap();
    let requested_generation = world
        .store
        .get_case_state(CASE)
        .unwrap()
        .unwrap()
        .generation;
    let action = crate::transition::build_authenticated_review_action(
        &review,
        CASE,
        TENANT,
        &world.owner.projected_principal_id(),
        HUMAN,
        ReviewActionKind::Approve,
        "review exact historical operation",
        requested_generation,
        "historical-oracle",
    )
    .unwrap();
    let mut pending = secured_pending(
        "transition:historical-review-action",
        CASE,
        requested_generation,
        &world.owner.projected_principal_id(),
        TransitionPayload::ReviewActionRecorded {
            action: action.clone(),
        },
    );
    pending.scope = Some(proposed.scope.clone());
    pending.causal_refs = vec![review.review_id.clone(), proposed.operation_id.clone()];
    world
        .store
        .commit_secured_transition(&world.owner, TENANT, pending, true)
        .unwrap();
    let (approved, _) = world
        .store
        .derive_and_commit_policy_review_decision(
            CASE,
            &proposed.operation_id,
            &review.review_id,
            &action.action_id,
        )
        .unwrap();
    assert_eq!(approved.outcome, DecisionOutcome::Allow);
    assert_eq!(
        view(&world, requested_generation).state_then.reviews[0].status,
        ReviewResolution::Pending
    );
    assert_eq!(
        view(
            &world,
            world
                .store
                .get_case_state(CASE)
                .unwrap()
                .unwrap()
                .generation
        )
        .state_then
        .reviews[0]
            .status,
        ReviewResolution::Approved
    );
    let (p2, replaced) = replace(&world, "3", "deny", false);
    let forbidden = world.operation("request:historical-deny", "src/retry.txt");
    let (d2, c2) = world
        .store
        .derive_and_commit_policy_decision(CASE, &forbidden.operation_id)
        .unwrap();
    assert_eq!(d2.outcome, DecisionOutcome::Deny);
    let old = view(&world, c1.state.generation);
    assert_eq!(
        old.normative_then
            .effective_policy
            .as_ref()
            .unwrap()
            .effective_policy_id,
        d1.decision_basis.as_ref().unwrap().effective_policy_id
    );
    assert_eq!(
        view(&world, c2.state.generation)
            .normative_then
            .effective_policy
            .as_ref()
            .unwrap()
            .effective_policy_id,
        d2.decision_basis.as_ref().unwrap().effective_policy_id
    );
    assert!(old.known_by_then.iter().any(|e| matches!(&e.payload,
        TransitionPayload::DecisionRecorded { decision } if decision == &d1)));
    assert!(old
        .comparison_to_now
        .iter()
        .any(|c| c.category == "superseded"));
    assert!(old
        .comparison_to_now
        .iter()
        .any(|c| c.category == "not_yet_existing_at_coordinate"));
    world
        .store
        .revoke_tenant_policy_artifact(&world.owner, &p2, "current permission revoked")
        .unwrap();
    let revoked = view(&world, c1.state.generation);
    assert_eq!(
        revoked.normative_now.validity,
        PolicyValidityPosture::Revoked
    );
    assert_eq!(revoked.normative_then, old.normative_then);
    assert_ne!(revoked.view_id, old.view_id);
    assert!(world
        .store
        .admit_resource_read_authorized(&world.owner, CASE, &forbidden.operation_id)
        .is_err());
    let state = world.store.get_case_state(CASE).unwrap().unwrap();
    world
        .store
        .unbind_tenant_case_policy(
            &world.owner,
            CASE,
            &state.policy_bindings[0].binding_id,
            state.generation,
            "retain history without permission",
        )
        .unwrap();
    let final_generation = world
        .store
        .get_case_state(CASE)
        .unwrap()
        .unwrap()
        .generation;
    let final_view = view(&world, c1.state.generation);
    assert_eq!(
        final_view.normative_now.readiness,
        NormativeReadiness::Unconfigured
    );
    assert_eq!(final_view.normative_then, old.normative_then);
    assert!(view(&world, final_generation)
        .state_then
        .policy_bindings
        .is_empty());
    assert!(world.store.verify_case_state(CASE).unwrap());
    let experience = world.store.experience_view_authorized(&world.owner, CASE, request(final_generation),
        crate::graph::experience::ExperienceQuery { from: Some(review_decision.decision_id.clone()), to: Some(approved.decision_id.clone()), ..Default::default() }, None).unwrap();
    use crate::graph::experience::RelationKind as RK;
    assert_eq!(experience.relations.iter().map(|r| r.kind.clone()).collect::<Vec<_>>(),
        vec![RK::DecisionReview, RK::ReviewAction, RK::ReviewReevaluation]);
    println!("experience_review review={} action={} approved={} qualified_edges=3 old_policy_no_current_authority=true", review.review_id, action.action_id, approved.decision_id);
    println!("historical_decisions p1_artifact={} d1={} basis1={} effective1={} d2={} basis2={} effective2={} review_action={} historical_view={}",
        p1.policy_bindings[0].artifact_id, d1.decision_id, d1.decision_basis.as_ref().unwrap().basis_id,
        d1.decision_basis.as_ref().unwrap().effective_policy_id, d2.decision_id,
        d2.decision_basis.as_ref().unwrap().basis_id, d2.decision_basis.as_ref().unwrap().effective_policy_id,
        action.action_id, final_view.view_id);
    println!("historical_policy case={CASE} before={} p1={} d1={} late={} review_policy={} review={} p2={} d2={} unbound={} exact_basis=true old_allow_not_permission=true model_calls=0",
        p1.policy_bindings[0].bound_at_case_generation-1, p1.policy_bindings[0].bound_at_case_generation, c1.state.generation, observation_generation, review_generation,
        review.review_id, replaced, c2.state.generation, final_generation);
    world.finish();
}

#[test]
fn historical_scope_coordinates_missing_source_restart_and_cache_invariance() {
    let world = World::new();
    let op = world.operation("request:historical-source", "src/retry.txt");
    world
        .store
        .derive_and_commit_policy_decision(CASE, &op.operation_id)
        .unwrap();
    let current = world.store.get_case_state(CASE).unwrap().unwrap();
    let history = world.store.list_case_transitions(CASE).unwrap();
    let before = view(&world, current.generation);
    for generation in [0, current.generation + 1, u64::MAX] {
        assert!(world
            .store
            .historical_semantic_view_authorized(&world.owner, CASE, request(generation), None)
            .unwrap_err()
            .contains("coordinate_invalid"));
    }
    let mut r = request(current.generation);
    r.coordinate = HistoricalCoordinate::Transition(history.last().unwrap().transition_id.clone());
    let exact = world
        .store
        .historical_semantic_view_authorized(&world.owner, CASE, r.clone(), None)
        .unwrap();
    assert_eq!(exact.state_then, before.state_then);
    assert_eq!(exact.view_id, before.view_id);
    assert_eq!(exact.known_by_then, before.known_by_then);
    r.coordinate = HistoricalCoordinate::Transition("transition:other-case".into());
    assert!(world
        .store
        .historical_semantic_view_authorized(&world.owner, CASE, r, None)
        .is_err());
    assert!(world
        .store
        .historical_semantic_view_authorized(&world.outsider, CASE, request(1), None)
        .is_err());
    for p in ["participant:model", "participant:absent"] {
        let mut other = request(1);
        other.participant_id = p.into();
        assert_eq!(
            world
                .store
                .historical_semantic_view_authorized(&world.owner, CASE, other, None)
                .unwrap_err(),
            "historical_scope_unavailable"
        );
    }
    let mut wrong_view = request(1);
    wrong_view.view_kind = "undisclosed".into();
    assert!(world
        .store
        .historical_semantic_view_authorized(&world.owner, CASE, wrong_view, None)
        .is_err());
    for bytes in [false, true] {
        let mut tiny = request(current.generation);
        if bytes {
            tiny.max_bytes = 1
        } else {
            tiny.max_items = 1
        }
        assert!(world
            .store
            .historical_semantic_view_authorized(&world.owner, CASE, tiny, None)
            .unwrap_err()
            .contains("budget_exceeded"));
    }
    let memory = derive_operational_memory(CASE, &history).unwrap();
    world
        .store
        .replace_case_operational_memory(&memory)
        .unwrap();
    world.store.rebuild_graph_relations_for_case(CASE).unwrap();
    assert_eq!(view(&world, current.generation), before);
    world.store.drop_effective_policy(CASE).unwrap();
    world.store.clear_case_operational_memory(CASE).unwrap();
    world.store.clear_semantic_context_artifacts().unwrap();
    world.store.clear_graph_relations_for_case(CASE).unwrap();
    assert_eq!(view(&world, current.generation), before);
    world.store.rebuild_graph_relations_for_case(CASE).unwrap();
    world.store.rebuild_effective_policy(CASE).unwrap();
    assert_eq!(view(&world, current.generation), before);
    world
        .store
        .replace_case_operational_memory(&memory)
        .unwrap();
    let World {
        path,
        store,
        owner,
        outsider,
        binding,
        artifact_id,
    } = world;
    drop(store);
    let world = World {
        store: LmdbRecordStore::open(path.join("store")).unwrap(),
        path,
        owner,
        outsider,
        binding,
        artifact_id,
    };
    assert_eq!(view(&world, current.generation), before);
    world
        .store
        .discard_policy_source_for_test(&current.policy_bindings[0].source_id)
        .unwrap();
    let missing = view(&world, current.generation);
    assert_ne!(missing.view_id, before.view_id);
    assert_eq!(
        missing.normative_then.effective_policy,
        before.normative_then.effective_policy
    );
    assert!(missing
        .normative_then
        .source_closure
        .iter()
        .any(|s| s.posture == "original_source_unavailable_artifact_retained"));
    assert_eq!(history, world.store.list_case_transitions(CASE).unwrap());
    assert_eq!(current, world.store.replay_case_state(CASE).unwrap());
    println!("historical_recovery exact_transition=true invalid_future_refused=true current_identity_and_disclosure=true bounded_refusal=true original_loss_explicit=true replay=true caches_independent=true reads_append=0");
    world.finish();
}

#[test]
fn historical_unbound_policy_without_decision_retains_exact_missingness() {
    let world = World::new();
    let bound = world.store.get_case_state(CASE).unwrap().unwrap();
    let binding = &bound.policy_bindings[0];
    world
        .store
        .unbind_tenant_case_policy(
            &world.owner,
            CASE,
            &binding.binding_id,
            bound.generation,
            "unused policy remains historical evidence",
        )
        .unwrap();
    let current = world.store.get_case_state(CASE).unwrap().unwrap();
    let history = world.store.list_case_transitions(CASE).unwrap();
    let present = view(&world, current.generation);
    assert!(present.state_then.policy_bindings.is_empty());
    assert!(present.known_by_then.iter().any(|e| matches!(&e.payload,
        TransitionPayload::CasePolicyBound { binding: b } if b == binding)));
    assert!(!present
        .known_by_then
        .iter()
        .any(|e| matches!(e.payload, TransitionPayload::DecisionRecorded { .. })));
    assert!(present
        .normative_then
        .source_closure
        .iter()
        .any(|s| s.source_ref == binding.artifact_id && s.posture == "exact_immutable_artifact"));
    world
        .store
        .discard_policy_source_for_test(&binding.source_id)
        .unwrap();
    let source_lost = view(&world, current.generation);
    assert!(source_lost
        .normative_then
        .source_closure
        .iter()
        .any(|s| s.source_ref == binding.source_id
            && s.posture == "original_source_unavailable_artifact_retained"));
    world
        .store
        .discard_policy_artifact_for_test(&binding.artifact_id)
        .unwrap();
    let artifact_lost = view(&world, current.generation);
    assert_ne!(artifact_lost.view_id, source_lost.view_id);
    assert_eq!(artifact_lost.known_by_then, present.known_by_then);
    assert!(artifact_lost
        .normative_then
        .source_closure
        .iter()
        .any(|s| s.source_ref == binding.artifact_id
            && s.posture == "artifact_unavailable_recorded_reference_retained"));
    let old = view(&world, bound.generation);
    assert_eq!(old.state_then.policy_bindings, bound.policy_bindings);
    assert_eq!(old.normative_then.readiness, NormativeReadiness::Blocked);
    assert!(old.normative_then.effective_policy.is_none());
    assert!(old
        .normative_then
        .missing
        .iter()
        .any(|s| s.contains("exact_artifact_unavailable")));
    assert_eq!(history, world.store.list_case_transitions(CASE).unwrap());
    assert_eq!(current, world.store.replay_case_state(CASE).unwrap());
    println!("historical_unbound_source decision_count=0 original_loss=explicit artifact_loss=explicit historical_binding=retained incomplete_normative=blocked reads_append=0");
    world.finish();
}

#[test]
fn historical_reconstruction_short_long_measurements() {
    let world = World::new();
    let short = world
        .store
        .get_case_state(CASE)
        .unwrap()
        .unwrap()
        .generation;
    for count in [0, 400] {
        for n in 0..count {
            let state = world.store.get_case_state(CASE).unwrap().unwrap();
            world
                .store
                .commit_secured_transition(
                    &world.owner,
                    TENANT,
                    secured_pending(
                        &format!("transition:historical-age:{n}"),
                        CASE,
                        state.generation,
                        &world.owner.projected_principal_id(),
                        TransitionPayload::ParticipantBound {
                            participant_id: "participant:model".into(),
                            role: format!("historical-role-{n}"),
                        },
                    ),
                    true,
                )
                .unwrap();
        }
        let size = world
            .store
            .get_case_state(CASE)
            .unwrap()
            .unwrap()
            .generation;
        for coordinate in [1, short, size] {
            let start = Instant::now();
            let v = view(&world, coordinate);
            println!("historical_cost case={CASE} history={size} coordinate={coordinate} scanned={} replayed={} items={} elapsed_us={} id={}",
                v.transitions_scanned, v.transitions_replayed, v.semantic_items, start.elapsed().as_micros(), v.view_id);
            assert_eq!(v.transitions_scanned as u64, size);
            assert_eq!(v.transitions_replayed as u64, size + 2 * coordinate);
            assert!(v.semantic_items < 16);
        }
    }
    assert!(world.store.verify_case_state(CASE).unwrap());
    world.finish();
}

#[test]
fn historical_current_participant_scope_filters_preexisting_private_evidence() {
    let world = World::new();
    let op = world.operation("request:private-history", "src/retry.txt");
    world
        .store
        .derive_and_commit_policy_decision(CASE, &op.operation_id)
        .unwrap();
    let admission = world
        .store
        .admit_resource_read_authorized(&world.owner, CASE, &op.operation_id)
        .unwrap();
    world
        .store
        .record_resource_observation_authorized(&world.owner, &admission, world.result(&admission))
        .unwrap();
    let at = world
        .store
        .get_case_state(CASE)
        .unwrap()
        .unwrap()
        .generation;
    world
        .store
        .add_tenant_member(
            &world.owner,
            TENANT,
            &world.outsider.projected_principal_id(),
            2,
        )
        .unwrap();
    let participant = "participant:second-human";
    let principal = world.owner.projected_principal_id();
    let link = crate::transition::PrincipalParticipantLink::new(
        CASE,
        TENANT,
        &world.outsider.projected_principal_id(),
        participant,
        &principal,
        3,
    )
    .unwrap();
    for (n, payload) in [
        TransitionPayload::ParticipantBound {
            participant_id: participant.into(),
            role: "operation-proposer".into(),
        },
        TransitionPayload::ParticipantPrincipalLinked { link: link.clone() },
    ]
    .into_iter()
    .enumerate()
    {
        let state = world.store.get_case_state(CASE).unwrap().unwrap();
        let mut pending = secured_pending(
            &format!("transition:historical-second:{n}"),
            CASE,
            state.generation,
            &principal,
            payload,
        );
        if n == 1 {
            pending.causal_refs = vec![link.principal_id.clone(), participant.into()];
        }
        world
            .store
            .commit_secured_transition(&world.owner, TENANT, pending, true)
            .unwrap();
    }
    let r = HistoricalRequest::inspection(HistoricalCoordinate::Generation(at), participant);
    let other = world
        .store
        .historical_semantic_view_authorized(&world.outsider, CASE, r.clone(), None)
        .unwrap();
    assert!(other.state_then.participant.is_none()); // Participant did not exist then
    assert!(other.state_then.resources.is_empty());
    assert!(other
        .known_by_then
        .iter()
        .all(|e| matches!(e.payload, TransitionPayload::CasePolicyBound { .. })));
    let serialized = serde_json::to_string(&other).unwrap();
    assert!(!serialized.contains(&op.operation_id));
    assert!(!serialized.contains("current retry constraint"));
    assert!(!serialized.contains(RESOURCE));
    assert_ne!(other.view_id, view(&world, at).view_id);
    let scoped = world.store.experience_view_authorized(&world.outsider, CASE, r.clone(), Default::default(), None).unwrap();
    let encoded = serde_json::to_string(&scoped).unwrap();
    assert!(!encoded.contains(&op.operation_id));
    assert!(!encoded.contains(RESOURCE));
    assert!(scoped.episode_slices.is_empty());
    assert!(scoped.relations.is_empty());
    let hidden = crate::graph::experience::ExperienceQuery { from: Some(op.operation_id.clone()),
        to: Some(op.operation_id.clone()), ..Default::default() };
    let absent = crate::graph::experience::ExperienceQuery { from: Some("operation:does-not-exist".into()),
        to: Some("operation:does-not-exist".into()), ..Default::default() };
    assert_eq!(world.store.experience_view_authorized(&world.outsider, CASE, r.clone(), hidden, None).unwrap_err(),
        world.store.experience_view_authorized(&world.outsider, CASE, r.clone(), absent, None).unwrap_err());
    let mut denied = r.clone();
    denied.consumer = "model".into();
    denied.view_kind = "model_context".into();
    assert!(world
        .store
        .historical_semantic_view_authorized(&world.outsider, CASE, denied.clone(), None)
        .is_err());
    let current = world.store.get_case_state(CASE).unwrap().unwrap();
    world
        .store
        .commit_secured_transition(
            &world.owner,
            TENANT,
            secured_pending(
                "transition:historical-current-disclosure",
                CASE,
                current.generation,
                &principal,
                TransitionPayload::ParticipantAdmitted {
                    participant_id: participant.into(),
                    consumer: "model".into(),
                    view_kind: "model_context".into(),
                },
            ),
            true,
        )
        .unwrap();
    let changed = world
        .store
        .historical_semantic_view_authorized(&world.outsider, CASE, denied, None)
        .unwrap();
    assert!(changed
        .known_by_then
        .iter()
        .all(|e| matches!(e.payload, TransitionPayload::CasePolicyBound { .. })));
    assert_ne!(changed.current_scope_digest, other.current_scope_digest);
    assert!(world
        .store
        .historical_semantic_view_authorized(&world.outsider, CASE, request(at), None)
        .is_err());
    println!("historical_disclosure same_tenant=true different_current_principal=true preexisting_private_observation=hidden hidden_resource_identity=absent current_view_change=recompiled historical_permission_restored=false");
    world.finish();
}

#[test]
fn historical_immutable_content_missingness_never_substitutes_live_resource() {
    use crate::conversation::ConversationContentStore;
    let world = World::with_access(AccessKind::Discover);
    assert!(ConversationContentStore::open_existing(&world.path).is_err());
    assert!(!world.path.join("conversation-content-v1").exists());
    let content = ConversationContentStore::open(&world.path).unwrap();
    let record = |id: &str, action| {
        world
            .store
            .record_participant_resource_request(
                &world.owner,
                CASE,
                HUMAN,
                RESOURCE,
                id,
                world
                    .store
                    .get_case_state(CASE)
                    .unwrap()
                    .unwrap()
                    .generation,
                ResourceRequest {
                    schema: RESOURCE_REQUEST_SCHEMA.into(),
                    configuration_digest: world.binding.digest(),
                    action,
                },
            )
            .unwrap()
    };
    let discover = record(
        "request:historical-discover",
        ResourceAction::Discover { path: "src".into() },
    );
    world
        .store
        .derive_and_commit_policy_decision(CASE, &discover.operation_id)
        .unwrap();
    let admitted = world
        .store
        .admit_resource_read_authorized(&world.owner, CASE, &discover.operation_id)
        .unwrap();
    world
        .store
        .record_resource_observation_authorized(
            &world.owner,
            &admitted,
            inspect_confined_tree(world.binding.root().unwrap(), "src", None, 4096, 16).unwrap(),
        )
        .unwrap();
    let op = record(
        "request:historical-content",
        ResourceAction::AdmitContent {
            path: "src/retry.txt".into(),
            candidate_digest: digest_bytes(
                &fs::read(world.path.join("workspace/src/retry.txt")).unwrap(),
            ),
        },
    );
    world
        .store
        .derive_and_commit_policy_decision(CASE, &op.operation_id)
        .unwrap();
    world
        .store
        .admit_discovered_content_authorized(&world.owner, &content, CASE, &op.operation_id)
        .unwrap();
    let state = world.store.get_case_state(CASE).unwrap().unwrap();
    let r = request(state.generation);
    let before = world
        .store
        .historical_semantic_view_authorized(&world.owner, CASE, r.clone(), Some(&content))
        .unwrap();
    let mut recall_request=crate::memory_hierarchy::recall::RecallRequest::new(CASE,state.generation,HUMAN,"exact admitted content");
    recall_request.required_refs=vec![state.admitted_content[0].object.object_id.clone()];
    let recall_before=world.store.recall_trace_authorized(&world.owner,recall_request.clone(),Some(&content)).unwrap().trace;
    assert!(recall_before.closure_complete);
    assert_eq!(before.content_backing.len(), 1);
    assert_eq!(
        before.content_backing[0].posture,
        "exact_original_available"
    );
    fs::write(
        world.path.join("workspace/src/retry.txt"),
        b"live source changed; never historical replacement",
    )
    .unwrap();
    assert_eq!(
        world
            .store
            .historical_semantic_view_authorized(&world.owner, CASE, r.clone(), Some(&content))
            .unwrap(),
        before
    );
    let id = state.admitted_content[0]
        .object
        .object_id
        .strip_prefix("content-object:")
        .unwrap();
    let payload = world
        .path
        .join("conversation-content-v1/objects")
        .join(id)
        .join("payload");
    fs::rename(
        &payload,
        payload.with_file_name("payload-retained-for-recovery"),
    )
    .unwrap();
    let missing = world
        .store
        .historical_semantic_view_authorized(&world.owner, CASE, r, Some(&content))
        .unwrap();
    let recall_missing=world.store.recall_trace_authorized(&world.owner,recall_request,Some(&content)).unwrap().trace;
    assert!(!recall_missing.closure_complete);
    assert_ne!(recall_missing.trace_id,recall_before.trace_id);
    assert!(recall_missing.source_closure.iter().any(|s|s.posture=="original_unavailable_or_integrity_failed"));
    assert_eq!(
        missing.content_backing[0].posture,
        "original_unavailable_or_integrity_failed"
    );
    assert_ne!(before.view_id, missing.view_id);
    assert_eq!(before.state_then, missing.state_then);
    assert_eq!(before.known_by_then, missing.known_by_then);
    assert!(world.store.verify_case_state(CASE).unwrap());
    println!("historical_content admitted_original=exact mutable_source_drift=ignored payload_loss=explicit canonical_metadata=preserved substitute=none readonly_open_creates_nothing=true");
    drop(content);
    world.finish();
}
