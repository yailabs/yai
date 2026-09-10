use super::*;
use crate::graph::experience::{ExperienceQuery, ExperienceView, RelationKind as K};

fn derive(h: &HistoricalSemanticView, q: ExperienceQuery) -> Result<ExperienceView, String> {
    crate::graph::experience::derive(h, q, "test:fixed-qualified-disclosure")
}

fn experience(w: &World, at: u64, q: ExperienceQuery) -> ExperienceView {
    let mut r = request(at);
    r.max_items = 4096;
    r.max_bytes = 16_777_216;
    w.store
        .experience_view_authorized(&w.owner, CASE, r, q, None)
        .unwrap()
}

fn path(from: &str, to: &str) -> ExperienceQuery {
    ExperienceQuery {
        from: Some(from.into()),
        to: Some(to.into()),
        ..Default::default()
    }
}

#[test]
fn experience_committed_cross_episode_origin_and_current_scope_identity() {
    let w = World::new();
    let commit = |id: &str,
                  payload: TransitionPayload,
                  refs: Vec<String>,
                  scope: Option<TransitionScope>| {
        let s = w.store.get_case_state(CASE).unwrap().unwrap();
        let mut p = secured_pending(
            id,
            CASE,
            s.generation,
            &w.owner.projected_principal_id(),
            payload,
        );
        p.causal_refs = refs;
        p.scope = scope;
        w.store
            .commit_secured_transition(&w.owner, TENANT, p, true)
            .unwrap()
    };
    commit(
        "transition:experience:provider",
        TransitionPayload::ProviderAttached {
            participant_id: HUMAN.into(),
            provider_id: "provider:experience".into(),
            provider_kind: "openai_compatible".into(),
            base_url: "http://127.0.0.1:1".into(),
            model_id: "contract-no-inference".into(),
            credential_ref: "none".into(),
        },
        vec![],
        None,
    );
    let s = w.store.get_case_state(CASE).unwrap().unwrap();
    let lineage = test_provider_lineage(s.generation);
    commit(
        "transition:experience:invocation",
        TransitionPayload::ProviderInvocationStarted {
            invocation_id: "invocation:experience".into(),
            participant_id: HUMAN.into(),
            provider_id: "provider:experience".into(),
            provider_kind: "openai_compatible".into(),
            model_id: "contract-no-inference".into(),
            semantic_lineage: Some(lineage.clone()),
            governance: None,
        },
        vec![],
        None,
    );
    let candidate = r#"{"schema":"yai.operation_proposal.filesystem_write.v1","operation":"filesystem.write","resource":"resource:experience-write","path":"src/result","content":"bounded"}"#;
    commit(
        "transition:experience:result",
        TransitionPayload::ProviderResultRecorded {
            invocation_id: "invocation:experience".into(),
            result_id: "provider-result:experience".into(),
            provider_id: "provider:experience".into(),
            provider_kind: "openai_compatible".into(),
            model_id: "contract-no-inference".into(),
            semantic_lineage: Some(lineage),
            output: candidate.into(),
        },
        vec!["invocation:experience".into()],
        None,
    );
    // Existing typed filesystem-proposal history, not a forged native
    // capability result (that public producer contract requires its own frame).
    let resource = ResourceAttachmentState {
        attachment_id: "resource:experience-write".into(),
        kind: ResourceKind::Filesystem,
        allowed_write_prefix: "src".into(),
        max_write_bytes: 128,
        policy_id: "policy:inert-envelope".into(),
        policy_owner_participant_id: HUMAN.into(),
        review_requirement: ReviewRequirement::Automatic,
        process_signal_actions: vec![],
        access: None,
    };
    commit(
        "transition:experience:write-resource",
        TransitionPayload::ResourceAttached {
            attachment: resource.clone(),
        },
        vec![HUMAN.into()],
        None,
    );
    let s = w.store.get_case_state(CASE).unwrap().unwrap();
    let op = normalize_filesystem_write_candidate(
        candidate,
        &NormalizationContext {
            case_id: CASE,
            participant_id: HUMAN,
            provider_result_id: "provider-result:experience",
            provider_invocation_id: "invocation:experience",
            case_generation: s.generation,
            resource: &resource,
        },
    )
    .unwrap();
    commit(
        "transition:experience:operation",
        TransitionPayload::OperationRecorded {
            operation: op.clone(),
        },
        op.origin.causal_refs(),
        Some(op.scope.clone()),
    );
    let (d, c) = w
        .store
        .derive_and_commit_policy_decision(CASE, &op.operation_id)
        .unwrap();
    assert_eq!(d.outcome, DecisionOutcome::Deny); // no write rule: origin confers no permission
    let q = path("provider-result:experience", &d.decision_id);
    let v = experience(&w, c.state.generation, q.clone());
    assert_eq!(v.result, "qualified_path");
    assert_eq!(
        v.events.first().unwrap().posture,
        crate::semantic_state::AuthorityPosture::ProviderClaim
    );
    // A direct evidence-support edge is also legitimate. Inspect the full graph
    // to prove the separate origin/lifecycle links, not just shortest-path shape.
    let all = experience(&w, c.state.generation, Default::default());
    assert!(all.relations.iter().any(|r| r.kind == K::ProposalOrigin));
    let operation_path = experience(
        &w,
        c.state.generation,
        path("provider-result:experience", &op.operation_id),
    );
    assert_eq!(operation_path.episode_slices.len(), 2);
    let history = w.store.list_case_transitions(CASE).unwrap();
    let episodes = crate::memory_hierarchy::derive_episodes(CASE, &history).unwrap();
    let result_episode = episodes
        .iter()
        .find(|e| {
            e.transition_ids
                .contains(&"transition:experience:result".into())
        })
        .unwrap();
    let operation_episode = episodes
        .iter()
        .find(|e| {
            e.transition_ids
                .contains(&"transition:experience:operation".into())
        })
        .unwrap();
    assert_ne!(result_episode.episode_id, operation_episode.episode_id);
    commit(
        "transition:experience:hidden-scope-change",
        TransitionPayload::ParticipantBound {
            participant_id: "participant:model".into(),
            role: "private-scope-change".into(),
        },
        vec![],
        None,
    );
    assert_eq!(v, experience(&w, c.state.generation, q.clone()));
    commit(
        "transition:experience:own-scope-change",
        TransitionPayload::ParticipantBound {
            participant_id: HUMAN.into(),
            role: "qualified-extra-role".into(),
        },
        vec![],
        None,
    );
    assert_ne!(
        v.current_disclosure_digest,
        experience(&w, c.state.generation, q).current_disclosure_digest
    );
    assert!(w.store.verify_case_state(CASE).unwrap());
    println!("experience_committed_cross_episode provider_result=provider-result:experience operation={} decision={} w20_result_episode={} w20_operation_episode={} relation=ProposalOrigin model_claim_not_fact=true other_scope_change=noninterference own_scope_change=reidentified provider_calls=0", op.operation_id, d.decision_id, result_episode.episode_id, operation_episode.episode_id);
    w.finish();
}

#[test]
fn experience_policy_support_supersession_late_evidence_and_no_chronological_cause() {
    let w = World::new();
    let p1 = w
        .store
        .get_case_state(CASE)
        .unwrap()
        .unwrap()
        .policy_bindings[0]
        .binding_id
        .clone();
    let op = w.operation("request:experience:1", "src/retry.txt");
    let (d1, cut) = w
        .store
        .derive_and_commit_policy_decision(CASE, &op.operation_id)
        .unwrap();
    let read = w
        .store
        .admit_resource_read_authorized(&w.owner, CASE, &op.operation_id)
        .unwrap();
    let mut result = w.result(&read);
    result["occurred_at_unix_ms"] = 1.into();
    result["causal_refs"] = serde_json::json!(["fabricated:causal-source"]);
    result["operation_id"] = "operation:injected-neighbor".into();
    let observed = w
        .store
        .record_resource_observation_authorized(&w.owner, &read, result)
        .unwrap();
    let (_, at) = replace(&w, "2", "deny", false);
    let p2 = w
        .store
        .get_case_state(CASE)
        .unwrap()
        .unwrap()
        .policy_bindings[0]
        .binding_id
        .clone();
    let other = w.operation("request:experience:unrelated", "src/retry.txt");
    let (d2, current) = w
        .store
        .derive_and_commit_policy_decision(CASE, &other.operation_id)
        .unwrap();
    assert_eq!(d1.outcome, DecisionOutcome::Allow);
    assert_eq!(d2.outcome, DecisionOutcome::Deny);
    let h = w.store.list_case_transitions(CASE).unwrap();
    let v = experience(&w, current.state.generation, Default::default());
    for (a, b, kind) in [
        (&p1, &d1.decision_id, K::PolicyBasis),
        (&p2, &d2.decision_id, K::PolicyBasis),
        (&p1, &p2, K::PolicyReplacement),
        (&op.operation_id, &d1.decision_id, K::OperationDecision),
        (
            &d1.decision_id,
            &observed.observation_id,
            K::DecisionObservation,
        ),
    ] {
        let p = experience(&w, current.state.generation, path(a, b));
        assert_eq!(p.result, "qualified_path");
        assert!(p.relations.iter().any(|r| r.kind == kind), "{p:?}");
        assert!(p.relations.iter().all(|r| r.sources.len() == 2));
    }
    let old = experience(&w, cut.state.generation, Default::default());
    assert!(!old
        .events
        .iter()
        .any(|e| e.object_refs.contains(&observed.observation_id)));
    let late = v
        .events
        .iter()
        .find(|e| e.object_refs.contains(&observed.observation_id))
        .unwrap();
    assert_eq!(late.occurred_at_unix_ms, None);
    assert_eq!(late.observed_at_unix_ms, Some(observed.observed_at_unix_ms));
    assert!(late.recorded_generation > cut.state.generation);
    assert!(!serde_json::to_string(&v)
        .unwrap()
        .contains("fabricated:causal-source"));
    assert!(!serde_json::to_string(&v)
        .unwrap()
        .contains("operation:injected-neighbor"));
    // Same resource and chronology do not connect two independent operations.
    assert!(experience(
        &w,
        current.state.generation,
        path(&op.operation_id, &other.operation_id)
    )
    .relations
    .is_empty());
    assert!(
        experience(&w, current.state.generation, path(&p2, &d1.decision_id))
            .relations
            .is_empty()
    );
    let mut temporal = path(&op.operation_id, &other.operation_id);
    temporal.include_recording_order = true;
    assert!(experience(&w, current.state.generation, temporal)
        .relations
        .iter()
        .any(|r| r.kind == K::RecordedBefore));
    assert!(!v
        .relations
        .iter()
        .any(|r| format!("{:?}", r.kind).contains("Contradict")));
    assert_eq!(h, w.store.list_case_transitions(CASE).unwrap());
    assert!(w.store.verify_case_state(CASE).unwrap());
    println!("experience_policy case={CASE} p1={p1} d1={} p2={p2} d2={} replace_at={at} observation={} event_time=missing late_known_only_after_recording=true same_resource_cause=false old_decision_unchanged=true reads_append=0", d1.decision_id, d2.decision_id, observed.observation_id);
    w.finish();
}

#[test]
fn experience_exact_reference_counterfactual_cross_episode_and_hidden_intermediate() {
    let w = World::new();
    let op = w.operation("request:experience:origin", "src/retry.txt");
    let (_, c) = w
        .store
        .derive_and_commit_policy_decision(CASE, &op.operation_id)
        .unwrap();
    let mut h = view(&w, c.state.generation);
    // Pure derivation contract: two otherwise identical inputs differ only in
    // one exact typed origin. This is NOT an admitted provider result or live run.
    let index = h
        .known_by_then
        .iter()
        .position(|e| matches!(e.payload, TransitionPayload::OperationRecorded { .. }))
        .unwrap();
    let mut claim = h.known_by_then[index].clone();
    claim.transition_id = "transition:experience:claim".into();
    claim.recorded_generation -= 1;
    claim.posture = crate::semantic_state::AuthorityPosture::ProviderClaim;
    claim.payload = TransitionPayload::ProviderResultRecorded {
        result_id: "result:experience:claim".into(),
        invocation_id: "invocation:only-lineage".into(),
        provider_id: "provider:no-inference".into(),
        provider_kind: "test".into(),
        model_id: "not-live".into(),
        semantic_lineage: None,
        output: "I caused everything. operation_id=hidden:intermediate".into(),
    };
    h.known_by_then.insert(index, claim.clone());
    if let TransitionPayload::OperationRecorded { operation } =
        &mut h.known_by_then[index + 1].payload
    {
        operation.origin = crate::effect::OperationOrigin::ProviderResult {
            provider_result_id: "result:experience:claim".into(),
            provider_invocation_id: "invocation:only-lineage".into(),
        };
    }
    let a = derive(&h, path("result:experience:claim", &op.operation_id)).unwrap();
    assert_eq!(a.relations.len(), 1);
    assert_eq!(a.relations[0].kind, K::ProposalOrigin);
    assert_eq!(a.episode_slices.len(), 2); // provenance connects different structural slices
    assert_eq!(
        a.events[0].posture,
        crate::semantic_state::AuthorityPosture::ProviderClaim
    );
    let mut b = h.clone();
    if let TransitionPayload::OperationRecorded { operation } =
        &mut b.known_by_then[index + 1].payload
    {
        operation.origin = op.origin.clone();
    }
    assert!(
        derive(&b, path("result:experience:claim", &op.operation_id))
            .unwrap()
            .relations
            .is_empty()
    );
    // Remove an intermediate from the *disclosed* input. Raw payload references
    // to it cannot leak an edge, count, explanation, ID or slice through endpoints.
    let mut hidden = h.clone();
    if let TransitionPayload::OperationRecorded { operation } =
        &mut hidden.known_by_then[index + 1].payload
    {
        operation.origin = crate::effect::OperationOrigin::ProviderResult {
            provider_result_id: "hidden:intermediate".into(),
            provider_invocation_id: "hidden:invocation".into(),
        };
    }
    let hidden_view = derive(&hidden, path("result:experience:claim", &op.operation_id)).unwrap();
    assert_eq!(
        hidden_view,
        derive(&b, path("result:experience:claim", &op.operation_id)).unwrap()
    );
    assert!(!serde_json::to_string(&hidden_view)
        .unwrap()
        .contains("hidden:"));
    println!("experience_counterfactual typed_origin=edge no_origin=no_edge model_claim=claim cross_episode_slices=2 hidden_intermediate=noninterference raw_prose_ignored=true provider=no_provider");
    w.finish();
}

#[test]
fn experience_recovery_source_loss_bounds_and_zero_mutation() {
    let w = World::new();
    let op = w.operation("request:experience:recovery", "src/retry.txt");
    let (_, c) = w
        .store
        .derive_and_commit_policy_decision(CASE, &op.operation_id)
        .unwrap();
    let before = experience(&w, c.state.generation, Default::default());
    let history = w.store.list_case_transitions(CASE).unwrap();
    let memory = derive_operational_memory(CASE, &history).unwrap();
    w.store.replace_case_operational_memory(&memory).unwrap();
    w.store.rebuild_graph_relations_for_case(CASE).unwrap();
    w.store.clear_case_operational_memory(CASE).unwrap();
    w.store.clear_graph_relations_for_case(CASE).unwrap();
    w.store.clear_semantic_context_artifacts().unwrap();
    w.store.drop_effective_policy(CASE).unwrap();
    assert_eq!(
        before,
        experience(&w, c.state.generation, Default::default())
    );
    w.store.rebuild_graph_relations_for_case(CASE).unwrap();
    w.store.replace_case_operational_memory(&memory).unwrap();
    w.store.rebuild_effective_policy(CASE).unwrap();
    let World {
        path,
        store,
        owner,
        outsider,
        binding,
        artifact_id,
    } = w;
    drop(store);
    let w = World {
        store: LmdbRecordStore::open(path.join("store")).unwrap(),
        path,
        owner,
        outsider,
        binding,
        artifact_id,
    };
    assert_eq!(
        before,
        experience(&w, c.state.generation, Default::default())
    );
    let mut tiny = ExperienceQuery::default();
    tiny.max_events = 1;
    assert!(w
        .store
        .experience_view_authorized(&w.owner, CASE, request(c.state.generation), tiny, None)
        .unwrap_err()
        .contains("budget"));
    let mut tiny = ExperienceQuery::default();
    tiny.max_bytes = 1;
    assert!(w
        .store
        .experience_view_authorized(&w.owner, CASE, request(c.state.generation), tiny, None)
        .unwrap_err()
        .contains("budget"));
    assert!(w
        .store
        .experience_view_authorized(
            &w.outsider,
            CASE,
            request(c.state.generation),
            Default::default(),
            None
        )
        .is_err());
    w.store
        .discard_policy_artifact_for_test(&w.artifact_id)
        .unwrap();
    let lost = experience(&w, c.state.generation, Default::default());
    assert!(lost.events.iter().any(|e| !e.source_closed));
    assert!(!lost.relations.iter().any(|r| r.kind == K::PolicyBasis));
    assert_ne!(lost.view_id, before.view_id);
    let missing_anchor = lost.events.iter().find(|e| !e.source_closed).unwrap();
    assert_eq!(
        w.store
            .experience_view_authorized(
                &w.owner,
                CASE,
                request(c.state.generation),
                self::path(&missing_anchor.transition_id, &missing_anchor.transition_id),
                None
            )
            .unwrap_err(),
        "experience_anchor_backing_unavailable"
    );
    assert_eq!(history, w.store.list_case_transitions(CASE).unwrap());
    assert_eq!(c.state, w.store.replay_case_state(CASE).unwrap());
    println!("experience_recovery id={} restart=equal cache_drop=equal rebuild=equal missing_artifact=unclosed invalid_edge_removed=true budgets=refuse scope=refuse transitions_added=0", before.view_id);
    w.finish();
}

#[test]
fn experience_short_long_cost_characterization() {
    let w = World::new();
    let op = w.operation("request:experience:cost", "src/retry.txt");
    let (d, _) = w
        .store
        .derive_and_commit_policy_decision(CASE, &op.operation_id)
        .unwrap();
    for count in [0, 400] {
        for n in 0..count {
            let s = w.store.get_case_state(CASE).unwrap().unwrap();
            let mut p = secured_pending(
                &format!("transition:experience:noise:{n}"),
                CASE,
                s.generation,
                &w.owner.projected_principal_id(),
                TransitionPayload::ParticipantBound {
                    participant_id: "participant:model".into(),
                    role: format!("noise-{n}"),
                },
            );
            // Extra caller-supplied causal_refs are lineage text, not typed proof.
            p.causal_refs = vec![op.operation_id.clone(), d.decision_id.clone()];
            w.store
                .commit_secured_transition(&w.owner, TENANT, p, true)
                .unwrap();
        }
        let at = w.store.get_case_state(CASE).unwrap().unwrap().generation;
        let start = Instant::now();
        let h = view(&w, at);
        let reconstruction = start.elapsed().as_micros();
        let start = Instant::now();
        let all = derive(&h, Default::default()).unwrap();
        let build = start.elapsed().as_micros();
        let start = Instant::now();
        let p = derive(&h, path(&op.operation_id, &d.decision_id)).unwrap();
        let path_build = start.elapsed().as_micros();
        assert_eq!(p.relations.len(), 1);
        println!("experience_cost history={} inspected={} visible_events={} episode_slices={} relations={} reconstruct_us={} relation_build_us={} build_plus_path_us={} output_bytes={} path_events={} path_edges={}", at, h.transitions_scanned, all.events.len(), all.episode_slices.len(), all.relations.len(), reconstruction, build, path_build, serde_json::to_vec(&p).unwrap().len(), p.events.len(), p.relations.len());
    }
    assert!(w.store.verify_case_state(CASE).unwrap());
    w.finish();
}
