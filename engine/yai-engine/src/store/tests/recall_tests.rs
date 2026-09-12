use super::*;
use crate::graph::experience::RelationKind as K;
use crate::memory_hierarchy::recall::{RecallRequest, RecallResult};

fn commit(w: &World, id: &str, payload: TransitionPayload) -> u64 {
    let s = w.store.get_case_state(CASE).unwrap().unwrap();
    let mut p = secured_pending(
        id,
        CASE,
        s.generation,
        &w.owner.projected_principal_id(),
        payload,
    );
    if let TransitionPayload::ProviderResultRecorded { invocation_id, .. } = &p.payload {
        p.causal_refs = vec![invocation_id.clone()];
    }
    w.store
        .commit_secured_transition(&w.owner, TENANT, p, true)
        .unwrap()
        .state
        .generation
}

fn claim(w: &World, name: &str, provider: &str, output: &str) -> (String, u64) {
    if !w
        .store
        .get_case_state(CASE)
        .unwrap()
        .unwrap()
        .provider
        .iter()
        .any(|p| p.provider_id == provider)
    {
        commit(
            w,
            &format!("transition:recall:attach:{name}"),
            TransitionPayload::ProviderAttached {
                participant_id: HUMAN.into(),
                provider_id: provider.into(),
                provider_kind: "openai_compatible".into(),
                base_url: "http://127.0.0.1:1".into(),
                model_id: "no-inference-contract".into(),
                credential_ref: "none".into(),
            },
        );
    }
    let invocation_id = format!("invocation:recall:{name}");
    let result_id = format!("result:recall:{name}");
    let lineage = test_provider_lineage(w.store.get_case_state(CASE).unwrap().unwrap().generation);
    commit(
        w,
        &format!("transition:recall:invoke:{name}"),
        TransitionPayload::ProviderInvocationStarted {
            invocation_id: invocation_id.clone(),
            participant_id: HUMAN.into(),
            provider_id: provider.into(),
            provider_kind: "openai_compatible".into(),
            model_id: "no-inference-contract".into(),
            semantic_lineage: Some(lineage.clone()),
            governance: None,
        },
    );
    let at = commit(
        w,
        &format!("transition:recall:result:{name}"),
        TransitionPayload::ProviderResultRecorded {
            invocation_id,
            result_id: result_id.clone(),
            provider_id: provider.into(),
            provider_kind: "openai_compatible".into(),
            model_id: "no-inference-contract".into(),
            semantic_lineage: Some(lineage),
            output: output.into(),
        },
    );
    (result_id, at)
}

fn recall(w: &World, query: &str, refs: &[&str], at: Option<u64>) -> Result<RecallResult, String> {
    let s = w.store.get_case_state(CASE)?.unwrap();
    let mut r = RecallRequest::new(CASE, s.generation, HUMAN, query);
    r.required_refs = refs.iter().map(|id| id.to_string()).collect();
    if let Some(at) = at {
        r.at = HistoricalCoordinate::Generation(at);
    }
    w.store.recall_trace_authorized(&w.owner, r, None)
}

#[test]
fn working_recall_policy_current_asof_freshness_tamper_and_atomic_budget() {
    use crate::semantic_state::{CompilationRequest, SemanticScope, SemanticPurpose, SemanticValue,
        WorkingStateRequest, DeltaCompilationMode, SemanticWorkingState};
    let w = World::new();
    let op = w.operation("request:working:allow", "src/retry.txt");
    let (d1, cut) = w.store.derive_and_commit_policy_decision(CASE, &op.operation_id).unwrap();
    let old_policy = d1.decision_basis.as_ref().unwrap().effective_policy_id.clone();
    replace(&w, "2", "deny", false);
    let state = w.store.get_case_state(CASE).unwrap().unwrap();
    let history = w.store.list_case_transitions(CASE).unwrap();
    let request = WorkingStateRequest {
        case_id: CASE.into(), expected_generation: state.generation,
        compilation: CompilationRequest {
            scope: SemanticScope::model(HUMAN, SemanticPurpose::Inspection),
            intent: "explain historical filesystem policy decision".into(),
            output_contract_id: crate::context::InvocationOutputContract::NaturalLanguage.contract_id(),
            max_semantic_units: 131072, max_derived_items: 16,
            resource_refs: vec![], required_refs: vec![HUMAN.into()],
            previous_item_ids: vec![], view_selection_id: None,
        }, at: Some(HistoricalCoordinate::Generation(cut.state.generation)),
        recall_required_refs: vec![d1.decision_id.clone()],
        recall_bounds: Default::default(), max_output_bytes: 1024 * 1024,
    };
    let result = w.store.compile_working_state_authorized(&w.owner, request.clone(), None).unwrap();
    let working = &result.working_state;
    assert_eq!(result.compilation_mode, DeltaCompilationMode::FullRecompilation);
    assert!(working.entries().iter().any(|e| matches!(&e.value,
        SemanticValue::EffectiveAuthority { effective_policy_id: Some(id), .. } if *id != old_policy)));
    assert!(working.entries().iter().any(|e| matches!(&e.value,
        SemanticValue::RecalledEvidence { evidence } if evidence.events.iter().any(|e|
            e.event.object_refs.contains(&d1.decision_id))
            && evidence.events.iter().all(|e| e.event.recorded_generation <= cut.state.generation))));
    assert_eq!(result.lower_context().unwrap().entries, working.entries());
    let projection = result.lower_context().unwrap();
    assert_eq!(projection.schema, crate::context::PROJECTION_SCHEMA_V11);
    let frame = crate::context::build_context_frame(&projection, &request.compilation.intent,
        crate::context::InvocationOutputContract::NaturalLanguage).unwrap();
    assert_eq!(frame.schema, crate::context::CONTEXT_FRAME_SCHEMA_V11);
    assert_eq!(frame.entries, working.entries());
    assert!(crate::context::build_context_frame(&projection, "silently replace the task",
        crate::context::InvocationOutputContract::NaturalLanguage).is_err());
    let mut no_optional = request.clone(); no_optional.compilation.max_derived_items = 0;
    let exact_only = w.store.compile_working_state_authorized(&w.owner, no_optional, None).unwrap();
    assert_eq!(exact_only.working_state.entries().iter().filter(|e| matches!(e.value, SemanticValue::RecalledEvidence { .. })).count(), 1);
    assert!(serde_json::to_string(&exact_only.working_state).unwrap().contains(&d1.decision_id));
    w.store.validate_working_state_authorized(&w.owner, working, None).unwrap();
    let mut forged = serde_json::to_value(working).unwrap();
    forged["entries"] = serde_json::json!([]);
    let forged: SemanticWorkingState = serde_json::from_value(forged).unwrap();
    assert!(w.store.validate_working_state_authorized(&w.owner, &forged, None).is_err());
    let mut tiny = request.clone(); tiny.compilation.max_semantic_units = 1;
    assert!(w.store.compile_working_state_authorized(&w.owner, tiny, None).is_err());
    let mut missing = request.clone(); missing.recall_required_refs = vec!["decision:absent".into()];
    assert!(w.store.compile_working_state_authorized(&w.owner, missing, None).is_err());
    assert!(w.store.compile_working_state_authorized(&w.outsider, request.clone(), None).is_err());
    w.store.clear_semantic_context_artifacts().unwrap();
    w.store.clear_case_operational_memory(CASE).unwrap();
    w.store.rebuild_graph_relations_for_case(CASE).unwrap();
    assert_eq!(*working, w.store.compile_working_state_authorized(&w.owner, request.clone(), None).unwrap().working_state);
    let current_artifact = state.policy_bindings[0].artifact_id.clone();
    w.store.revoke_tenant_policy_artifact(&w.owner, &current_artifact, "global current authority revoke").unwrap();
    assert_eq!(w.store.get_case_state(CASE).unwrap().unwrap(), state);
    assert!(w.store.validate_working_state_authorized(&w.owner, working, None).is_err());
    let refreshed = w.store.compile_working_state_authorized(&w.owner, request, None).unwrap();
    assert_ne!(refreshed.working_state.id(), working.id());
    assert_eq!(w.store.list_case_transitions(CASE).unwrap(), history);
    println!("working_policy asof_basis={} current_policy_not_rewound=true current_revoke_same_generation_invalidates=true tamper=refused exact_anchor_budget=refused outsider=refused cache_rebuild=equal lower_entries=identical models=0 transitions=0", d1.decision_basis.unwrap().basis_id);
    w.finish();
}

#[test]
fn recall_policy_lineage_discontinuous_current_asof_and_readonly() {
    let w = World::new();
    let before = w.store.get_case_state(CASE).unwrap().unwrap();
    let p1 = before.policy_bindings[0].binding_id.clone();
    let op = w.operation("request:recall:allow", "src/retry.txt");
    let (d1, cut) = w
        .store
        .derive_and_commit_policy_decision(CASE, &op.operation_id)
        .unwrap();
    replace(&w, "2", "deny", false);
    let now = w.store.get_case_state(CASE).unwrap().unwrap();
    let p2 = now.policy_bindings[0].binding_id.clone();
    let history = w.store.list_case_transitions(CASE).unwrap();
    let current = recall(&w, "policy", &[&d1.decision_id], None).unwrap();
    assert!(current
        .trace
        .events
        .iter()
        .any(|e| e.event.object_refs.contains(&p1)
            && e.validity_at_cut == "historical_binding_not_current_at_cut"));
    assert!(current
        .trace
        .events
        .iter()
        .any(|e| e.event.object_refs.contains(&p2)
            && e.validity_at_cut == "bound_at_cut_not_execution_permission"));
    assert!(current
        .trace
        .relations
        .iter()
        .any(|r| r.kind == crate::graph::experience::RelationKind::PolicyReplacement));
    let old = recall(&w, "policy", &[&d1.decision_id], Some(cut.state.generation)).unwrap();
    assert!(!serde_json::to_string(&old.trace).unwrap().contains(&p2));
    assert!(old
        .trace
        .events
        .iter()
        .all(|e| e.event.recorded_generation <= cut.state.generation));
    assert_eq!(
        current.trace,
        recall(&w, "policy", &[&d1.decision_id], None)
            .unwrap()
            .trace
    );
    assert_eq!(w.store.list_case_transitions(CASE).unwrap(), history);
    assert!(w.store.verify_case_state(CASE).unwrap());
    assert!(
        current.trace.closure_complete,
        "{:?}",
        current.trace.source_closure
    );
    println!("recall_policy current={} asof={} p1={} p2={} segments={} events={} relations={} bytes={} source_closed={} no_mutation=true", current.trace.trace_id, old.trace.trace_id, p1, p2, current.trace.segments.len(), current.trace.events.len(), current.trace.relations.len(), current.measurements.output_bytes, current.trace.closure_complete);
    w.finish();
}

#[test]
fn recall_claim_conflict_wrong_memory_late_evidence_and_episode_anchor() {
    use crate::memory_hierarchy::{EpistemicClass, SemanticLifecycle};
    let w = World::new();
    let (c1, cut) = claim(
        &w,
        "obsolete",
        "provider:recall",
        "resilience resilience retry always succeeds",
    );
    let (c2, _) = claim(
        &w,
        "contrary",
        "provider:recall",
        "resilience retry fails under concurrency",
    );
    let op = w.operation("request:recall:late", "src/retry.txt");
    w.store
        .derive_and_commit_policy_decision(CASE, &op.operation_id)
        .unwrap();
    let admission = w
        .store
        .admit_resource_read_authorized(&w.owner, CASE, &op.operation_id)
        .unwrap();
    let before_observation = w.store.get_case_state(CASE).unwrap().unwrap().generation;
    let mut value = w.result(&admission);
    value["occurred_at_unix_ms"] = 1.into(); // data, not a producer temporal contract
    let observation = w
        .store
        .record_resource_observation_authorized(&w.owner, &admission, value)
        .unwrap();
    let mut request = RecallRequest::new(
        CASE,
        w.store.get_case_state(CASE).unwrap().unwrap().generation,
        HUMAN,
        "resilience",
    );
    request.required_refs = vec![observation.observation_id.clone()];
    request.bounds.candidates = 1;
    let result = w
        .store
        .recall_trace_authorized(&w.owner, request, None)
        .unwrap();
    assert_eq!(result.trace.candidates.len(), 1);
    assert!(result
        .trace
        .assertions
        .iter()
        .any(
            |a| a.assertion.assertion_id == result.trace.candidates[0].source_ref
                && a.source_events
                    .contains(&"transition:recall:result:obsolete".into())
        ));
    assert_eq!(
        result
            .trace
            .assertions
            .iter()
            .filter(|a| a.assertion.predicate == "provider.claim")
            .count(),
        2
    );
    assert_eq!(result.trace.contradictions.len(), 1);
    assert_eq!(
        result.trace.contradictions[0].resolution_posture,
        "unresolved"
    );
    assert!(result
        .trace
        .assertions
        .iter()
        .filter(|a| a.assertion.predicate == "provider.claim")
        .all(
            |a| a.assertion.epistemic_class == EpistemicClass::ProviderOriginatedClaim
                && a.assertion.lifecycle == SemanticLifecycle::Contradicted
                && a.assertion.supersession_refs.is_empty()
        ));
    assert!(result
        .trace
        .events
        .iter()
        .any(|e| e.event.object_refs.contains(&c2)));
    assert!(result
        .trace
        .candidates
        .iter()
        .any(|c| c.plane == "lexical_bm25"));
    let late = result
        .trace
        .events
        .iter()
        .find(|e| e.event.object_refs.contains(&observation.observation_id))
        .unwrap();
    assert_eq!(
        late.event.observed_at_unix_ms,
        Some(observation.observed_at_unix_ms)
    );
    assert_eq!(late.event.occurred_at_unix_ms, None);
    assert!(recall(
        &w,
        "resilience",
        &[&observation.observation_id],
        Some(before_observation)
    )
    .unwrap_err()
    .contains("anchor_unavailable"));
    let old = recall(&w, "resilience", &[&c1], Some(cut)).unwrap();
    assert!(old.trace.contradictions.is_empty());
    assert!(!old
        .trace
        .events
        .iter()
        .any(|e| e.event.object_refs.contains(&c2)));
    let episodes = crate::memory_hierarchy::derive_episodes(
        CASE,
        &w.store.list_case_transitions(CASE).unwrap(),
    )
    .unwrap();
    let episode = episodes
        .iter()
        .find(|e| {
            e.transition_ids
                .contains(&"transition:recall:result:obsolete".into())
        })
        .unwrap();
    let exact = recall(&w, "unmatched-token", &[&episode.episode_id], None).unwrap();
    assert!(exact
        .trace
        .events
        .iter()
        .any(|e| e.event.object_refs.contains(&c1)));
    assert!(!result
        .trace
        .relations
        .iter()
        .any(|r| r.kind != K::RecordedBefore
            && (r.from_event == "transition:recall:result:obsolete"
                || r.from_event == "transition:recall:result:contrary")));
    println!("recall_claims c1={} c2={} contradiction=unresolved recency_not_supersession=true wrong_memory_not_fact=true episode={} late_observation={} occurred_time=missing asof_excludes_later=true segments={} source_closed={} obsolete_candidate_rank=1 claim_exact_anchor=false", c1,c2,episode.episode_id,observation.observation_id,result.trace.segments.len(),result.trace.closure_complete);
    w.finish();
}

#[test]
fn recall_recovery_missingness_budgets_scope_and_stale_request() {
    let w = World::new();
    let op = w.operation("request:recall:recover", "src/retry.txt");
    let (_, c) = w
        .store
        .derive_and_commit_policy_decision(CASE, &op.operation_id)
        .unwrap();
    let r = RecallRequest::new(CASE, c.state.generation, HUMAN, "policy");
    let first = w
        .store
        .recall_trace_authorized(&w.owner, r.clone(), None)
        .unwrap()
        .trace;
    let history = w.store.list_case_transitions(CASE).unwrap();
    let memory = derive_operational_memory(CASE, &history).unwrap();
    // Publish and drop an actual W20 episodic/semantic index bundle through the
    // existing sidecar lifecycle. Fixture vectors qualify mechanics, not an encoder.
    use crate::memory_index::*;
    let hierarchy =
        crate::memory_hierarchy::build_memory_hierarchy(&c.state, &history, &memory).unwrap();
    let corpus = derive_hierarchy_representation_corpus(&memory, &hierarchy).unwrap();
    let profile = MemoryRepresentationProfile::new_v2(
        TENANT,
        "encoder:cache-test",
        "fixture",
        "exact-test-revision",
        2,
    )
    .unwrap();
    let vectors = corpus
        .documents
        .iter()
        .map(|d| (d.document_id.clone(), vec![1.0, 0.0]))
        .collect();
    let bundle = MemoryIndexBundle::build(corpus, profile.clone(), &vectors).unwrap();
    let index_root = w.path.join("memory-index");
    let lock =
        acquire_memory_index_build_lock(&index_root, TENANT, CASE, &profile.profile_id).unwrap();
    publish_memory_index_locked(&lock, &bundle, false).unwrap();
    assert_eq!(
        first,
        w.store
            .recall_trace_authorized(&w.owner, r.clone(), None)
            .unwrap()
            .trace
    );
    assert!(drop_memory_index_locked(&lock).unwrap());
    assert_eq!(
        first,
        w.store
            .recall_trace_authorized(&w.owner, r.clone(), None)
            .unwrap()
            .trace
    );
    publish_memory_index_locked(&lock, &bundle, false).unwrap();
    drop(lock);
    w.store.replace_case_operational_memory(&memory).unwrap();
    w.store.rebuild_graph_relations_for_case(CASE).unwrap();
    w.store.clear_case_operational_memory(CASE).unwrap();
    w.store.clear_graph_relations_for_case(CASE).unwrap();
    w.store.drop_effective_policy(CASE).unwrap();
    w.store.clear_semantic_context_artifacts().unwrap();
    assert_eq!(
        first,
        w.store
            .recall_trace_authorized(&w.owner, r.clone(), None)
            .unwrap()
            .trace
    );
    w.store.replace_case_operational_memory(&memory).unwrap();
    w.store.rebuild_graph_relations_for_case(CASE).unwrap();
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
        first,
        w.store
            .recall_trace_authorized(&w.owner, r.clone(), None)
            .unwrap()
            .trace
    );
    let mut bad = r.clone();
    bad.expected_generation += 1;
    assert!(w
        .store
        .recall_trace_authorized(&w.owner, bad, None)
        .unwrap_err()
        .contains("scope_mismatch"));
    let mut bad = r.clone();
    bad.at = HistoricalCoordinate::Generation(c.state.generation + 1);
    assert!(w
        .store
        .recall_trace_authorized(&w.owner, bad, None)
        .is_err());
    let mut tiny = r.clone();
    tiny.bounds.bytes = 1;
    assert!(w
        .store
        .recall_trace_authorized(&w.owner, tiny, None)
        .unwrap_err()
        .contains("budget"));
    let mut tiny = r.clone();
    tiny.required_refs = vec![op.operation_id.clone()];
    tiny.bounds.events = 1;
    assert!(w
        .store
        .recall_trace_authorized(&w.owner, tiny, None)
        .unwrap_err()
        .contains("budget"));
    assert!(w
        .store
        .recall_trace_authorized(&w.outsider, r.clone(), None)
        .is_err());
    let mut hidden = r.clone();
    hidden.required_refs = vec!["hidden:never-disclosed".into()];
    let mut absent = r.clone();
    absent.required_refs = vec!["absent:not-present".into()];
    assert_eq!(
        w.store
            .recall_trace_authorized(&w.owner, hidden, None)
            .unwrap_err(),
        w.store
            .recall_trace_authorized(&w.owner, absent, None)
            .unwrap_err()
    );
    w.store
        .discard_policy_artifact_for_test(&w.artifact_id)
        .unwrap();
    let incomplete = w
        .store
        .recall_trace_authorized(&w.owner, r, None)
        .unwrap()
        .trace;
    assert!(!incomplete.closure_complete);
    assert_ne!(first.trace_id, incomplete.trace_id);
    assert!(!incomplete
        .relations
        .iter()
        .any(|r| r.kind == K::PolicyBasis));
    assert!(incomplete
        .source_closure
        .iter()
        .any(|s| s.posture.contains("unavailable")));
    assert_eq!(history, w.store.list_case_transitions(CASE).unwrap());
    assert!(w.store.verify_case_state(CASE).unwrap());
    println!("recall_recovery restart=equal graph_memory_policy_drop_rebuild=equal episodic_semantic_index_publish_drop_rebuild=equal stale=refused bounds=refused scope=refused absent_hidden=same missing_artifact=explicit_unclosed invalid_support_removed=true reads_append=0");
    w.finish();
}

#[test]
fn recall_optional_groups_omit_without_evicting_required_anchor() {
    let w = World::new();
    let (a, _) = claim(&w, "one", "provider:one", "zebra bounded experience");
    let (b, _) = claim(&w, "two", "provider:two", "zebra bounded experience");
    let at = w.store.get_case_state(CASE).unwrap().unwrap().generation;
    let mut r = RecallRequest::new(CASE, at, HUMAN, "zebra");
    r.required_refs = vec![a.clone()];
    r.bounds.events = 1;
    let trace = w
        .store
        .recall_trace_authorized(&w.owner, r, None)
        .unwrap()
        .trace;
    assert_eq!(trace.events.len(), 1);
    assert!(trace.events[0].event.object_refs.contains(&a));
    assert!(!trace.events[0].event.object_refs.contains(&b));
    assert!(trace.omitted_candidates >= 1);
    println!(
        "recall_budget required={} retained=true other={} omitted={} source_closed={}",
        a, b, trace.omitted_candidates, trace.closure_complete
    );
    w.finish();
}

#[test]
fn recall_optional_vectors_revalidate_sources_query_profile_and_hidden_inputs() {
    use crate::memory_hierarchy::recall::RecallVectorInput;
    use crate::memory_index::{
        MemoryRepresentationDocument, MemoryRepresentationProfile, MemoryVectorIndex,
        RetrievalQueryDocument,
    };
    let w = World::new();
    let (id, _) = claim(
        &w,
        "vector",
        "provider:vector",
        "resilience qualified candidate",
    );
    let baseline = recall(&w, "resilience", &[], None).unwrap();
    let documents: Vec<_> = baseline
        .trace
        .assertions
        .iter()
        .map(|a| MemoryRepresentationDocument::from_semantic_assertion(&a.assertion).unwrap())
        .collect();
    assert_eq!(documents.len(), 1);
    let profile = MemoryRepresentationProfile::new_v2(
        TENANT,
        "encoder:test",
        "not-external",
        "fixture-revision",
        2,
    )
    .unwrap();
    let values = documents
        .iter()
        .map(|d| (d.document_id.clone(), vec![1.0, 0.0]))
        .collect();
    let index = MemoryVectorIndex::build(&documents, &profile, &values).unwrap();
    let r = RecallRequest::new(
        CASE,
        baseline.trace.request.expected_generation,
        HUMAN,
        "unmatched-token",
    );
    let v = RecallVectorInput {
        profile,
        index,
        query: RetrievalQueryDocument::new_v2(&r.query).unwrap(),
        query_vector: vec![1.0, 0.0],
    };
    let trace = w
        .store
        .recall_trace_with_vectors_authorized(&w.owner, r.clone(), None, Some(&v))
        .unwrap()
        .trace;
    assert!(trace
        .candidates
        .iter()
        .any(|c| c.plane == "exact_vector_candidate"));
    assert!(trace
        .events
        .iter()
        .any(|e| e.event.object_refs.contains(&id)));
    let mut hidden = v.clone();
    let mut extra = hidden.index.embeddings[0].clone();
    extra.representation_document_id = "hidden:document".into();
    extra.values = vec![f32::NAN];
    hidden.index.embeddings.push(extra);
    assert_eq!(
        trace,
        w.store
            .recall_trace_with_vectors_authorized(&w.owner, r.clone(), None, Some(&hidden))
            .unwrap()
            .trace
    );
    let mut bad = v.clone();
    bad.query = RetrievalQueryDocument::new_v2("different").unwrap();
    assert!(w
        .store
        .recall_trace_with_vectors_authorized(&w.owner, r.clone(), None, Some(&bad))
        .is_err());
    let mut bad = v.clone();
    bad.index.embeddings[0].representation_document_digest = "tampered".into();
    assert!(w
        .store
        .recall_trace_with_vectors_authorized(&w.owner, r.clone(), None, Some(&bad))
        .is_err());
    let mut bad = v.clone();
    bad.query_vector = vec![0.0];
    assert!(w
        .store
        .recall_trace_with_vectors_authorized(&w.owner, r, None, Some(&bad))
        .is_err());
    println!("recall_vectors mode=explicit_preencoded candidates=1 same_query_profile=true stale_source=refused hidden_embedding=noninterference model_calls=0 encoder_semantic_qualification=NOT_CLAIMED");
    w.finish();
}

#[test]
fn recall_discontinuous_short_long_characterization() {
    let mut selected_counts = Vec::new();
    for gap in [0, 10_000] {
        let w = World::new();
        let p1 = w
            .store
            .get_case_state(CASE)
            .unwrap()
            .unwrap()
            .policy_bindings[0]
            .binding_id
            .clone();
        let old = w.operation("request:recall:original", "src/retry.txt");
        let (d1, _) = w
            .store
            .derive_and_commit_policy_decision(CASE, &old.operation_id)
            .unwrap();
        let a = w
            .store
            .admit_resource_read_authorized(&w.owner, CASE, &old.operation_id)
            .unwrap();
        let observation = w
            .store
            .record_resource_observation_authorized(&w.owner, &a, w.result(&a))
            .unwrap();
        // Similar file names and chronology are neither exact subject identity
        // nor qualified causal support. They remain raw discoverable experience.
        let mut distractors = Vec::new();
        for i in 0..64 {
            let o = w.operation(
                &format!("request:recall:distractor:{i}"),
                &format!("src/retrying-{i}.txt"),
            );
            distractors.push(o.operation_id);
        }
        for n in 0..gap {
            commit(
                &w,
                &format!("transition:recall:gap-a:{n}"),
                TransitionPayload::ParticipantBound {
                    participant_id: "participant:model".into(),
                    role: "irrelevant-other-task".into(),
                },
            );
        }
        replace(&w, "2", "deny", false);
        let p2 = w
            .store
            .get_case_state(CASE)
            .unwrap()
            .unwrap()
            .policy_bindings[0]
            .binding_id
            .clone();
        let current = w.operation("request:recall:current", "src/retry.txt");
        let (d2, _) = w
            .store
            .derive_and_commit_policy_decision(CASE, &current.operation_id)
            .unwrap();
        assert_eq!(d2.outcome, DecisionOutcome::Deny);
        for n in 0..gap {
            commit(
                &w,
                &format!("transition:recall:gap-b:{n}"),
                TransitionPayload::ParticipantBound {
                    participant_id: "participant:model".into(),
                    role: "irrelevant-other-task".into(),
                },
            );
        }
        let (claim_id, _) = claim(
            &w,
            "late",
            "provider:late",
            "retry design needs independent verification",
        );
        let generation = w.store.get_case_state(CASE).unwrap().unwrap().generation;
        let history_before = w.store.list_case_transitions(CASE).unwrap();
        let start = Instant::now();
        let r = recall(&w, "retry", &[&d2.decision_id], None).unwrap();
        let total_us = start.elapsed().as_micros();
        for id in [
            &p1,
            &p2,
            &old.operation_id,
            &current.operation_id,
            &d1.decision_id,
            &d2.decision_id,
            &observation.observation_id,
            &claim_id,
        ] {
            assert!(
                r.trace
                    .events
                    .iter()
                    .any(|e| e.event.object_refs.contains(id)),
                "missing {id}"
            );
        }
        assert!(r.trace.events.iter().all(|e| !e
            .event
            .object_refs
            .iter()
            .any(|id| distractors.contains(id))));
        assert!(r.trace.segments.len() >= 5);
        assert!(r
            .trace
            .relations
            .iter()
            .any(|r| r.kind == K::PolicyReplacement));
        assert!(r
            .trace
            .relations
            .iter()
            .any(|r| r.kind == K::DecisionObservation));
        assert!(r
            .trace
            .events
            .iter()
            .any(|e| e.event.object_refs.contains(&p1)
                && e.validity_at_cut == "historical_binding_not_current_at_cut"));
        assert!(r.trace.closure_complete);
        assert!(r.trace.events.len() < 20);
        assert_eq!(history_before, w.store.list_case_transitions(CASE).unwrap());
        assert!(w.store.verify_case_state(CASE).unwrap());
        selected_counts.push((
            r.trace.events.len(),
            r.trace.relations.len(),
            r.trace.segments.len(),
        ));
        println!("recall_characterization history={} gap_each={} source_visible_events={} candidate_count={} candidate_discovery_us={} relation_build_us={} qualification_us={} closure_us={} end_to_end_us={} selected_events={} selected_relations={} segments={} units={} bytes={} zero_transitions=true",generation,gap,r.measurements.qualified_events,r.trace.candidates.len(),r.measurements.discovery_us,r.measurements.relation_build_us,r.measurements.qualification_us,r.measurements.source_closure_us,total_us,r.trace.events.len(),r.trace.relations.len(),r.trace.segments.len(),r.measurements.semantic_units,r.measurements.output_bytes);
        println!(
            "recall_raw_candidates={}",
            serde_json::to_string(&r.trace.candidates).unwrap()
        );
        println!(
            "recall_qualified_trajectory={}",
            serde_json::to_string(
                &r.trace
                    .events
                    .iter()
                    .map(|e| (&e.event.object_refs, &e.validity_at_cut, &e.reasons))
                    .collect::<Vec<_>>()
            )
            .unwrap()
        );
        w.finish();
    }
    assert_eq!(selected_counts[0], selected_counts[1]);
}

#[test]
fn recall_current_disclosure_hides_private_events_counts_reasons_and_anchors() {
    let w = World::new();
    let op = w.operation("request:recall:private", "src/retry.txt");
    w.store
        .derive_and_commit_policy_decision(CASE, &op.operation_id)
        .unwrap();
    let a = w
        .store
        .admit_resource_read_authorized(&w.owner, CASE, &op.operation_id)
        .unwrap();
    let observed = w
        .store
        .record_resource_observation_authorized(&w.owner, &a, w.result(&a))
        .unwrap();
    let cut = w.store.get_case_state(CASE).unwrap().unwrap().generation;
    w.store
        .add_tenant_member(&w.owner, TENANT, &w.outsider.projected_principal_id(), 2)
        .unwrap();
    let participant = "participant:recall-other";
    commit(
        &w,
        "transition:recall:other-role",
        TransitionPayload::ParticipantBound {
            participant_id: participant.into(),
            role: "operation-proposer".into(),
        },
    );
    let state = w.store.get_case_state(CASE).unwrap().unwrap();
    let link = crate::transition::PrincipalParticipantLink::new(
        CASE,
        TENANT,
        &w.outsider.projected_principal_id(),
        participant,
        &w.owner.projected_principal_id(),
        3,
    )
    .unwrap();
    let mut pending = secured_pending(
        "transition:recall:other-link",
        CASE,
        state.generation,
        &w.owner.projected_principal_id(),
        TransitionPayload::ParticipantPrincipalLinked { link: link.clone() },
    );
    pending.causal_refs = vec![link.principal_id, participant.into()];
    let state = w
        .store
        .commit_secured_transition(&w.owner, TENANT, pending, true)
        .unwrap()
        .state;
    let mut r = RecallRequest::new(CASE, state.generation, participant, "retry");
    r.at = HistoricalCoordinate::Generation(cut);
    let trace = w
        .store
        .recall_trace_authorized(&w.outsider, r.clone(), None)
        .unwrap()
        .trace;
    assert!(trace.events.is_empty() && trace.candidates.is_empty() && trace.relations.is_empty());
    let mut integrated = r.clone();
    integrated.schema = crate::memory_hierarchy::recall::RECALL_REQUEST_V2.into();
    let v2 = w.store.recall_trace_authorized(&w.outsider, integrated.clone(), None).unwrap().trace;
    assert_eq!(v2.events, trace.events);
    assert_eq!(v2.candidates, trace.candidates);
    assert!(v2.documentary.as_ref().unwrap().sources.is_empty(), "Member H/S inspection survives the separate Owner-only D gate");
    integrated.required_refs = vec![op.operation_id.clone()];
    let denied = w.store.recall_trace_authorized(&w.outsider, integrated.clone(), None).unwrap_err();
    integrated.required_refs = vec!["operation:absent".into()];
    assert_eq!(denied, w.store.recall_trace_authorized(&w.outsider, integrated, None).unwrap_err());
    let output = serde_json::to_string(&trace).unwrap();
    for hidden in [
        &op.operation_id,
        &observed.observation_id,
        RESOURCE,
        "current retry constraint",
    ] {
        assert!(!output.contains(hidden));
    }
    let mut absent = r.clone();
    absent.required_refs = vec!["operation:absent".into()];
    let mut hidden = r.clone();
    hidden.required_refs = vec![op.operation_id.clone()];
    assert_eq!(
        w.store
            .recall_trace_authorized(&w.outsider, absent, None)
            .unwrap_err(),
        w.store
            .recall_trace_authorized(&w.outsider, hidden, None)
            .unwrap_err()
    );
    r.participant_id = HUMAN.into();
    assert!(w
        .store
        .recall_trace_authorized(&w.outsider, r, None)
        .is_err());
    println!("recall_disclosure same_tenant_different_principal=true historical_private_resource=hidden candidate_count=0 event_count=0 path_count=0 required_hidden_absent=same identity_impersonation=refused");
    w.finish();
}

#[test]
fn recall_relation_counterfactual_and_hidden_intermediate_do_not_become_memory() {
    let w = World::new();
    let op = w.operation("request:recall:counterfactual", "src/retry.txt");
    let (d, c) = w
        .store
        .derive_and_commit_policy_decision(CASE, &op.operation_id)
        .unwrap();
    let h = view(&w, c.state.generation);
    let history = w.store.list_case_transitions(CASE).unwrap();
    let mut r = RecallRequest::new(CASE, c.state.generation, HUMAN, "no-lexical-hit");
    r.required_refs = vec![d.decision_id.clone()];
    let a = crate::memory_hierarchy::recall::compile(&h, "fixed-scope", &history, r.clone(), None)
        .unwrap()
        .trace;
    assert!(a.relations.iter().any(|r| r.kind == K::PolicyBasis));
    // Resolver-only counterfactual, not a fabricated admitted Decision: qualified
    // input has no normative link. Same chronology/text cannot reconstruct it.
    let mut no_basis = h.clone();
    for e in &mut no_basis.known_by_then {
        if let TransitionPayload::DecisionRecorded { decision } = &mut e.payload {
            decision.decision_basis = None;
        }
    }
    let b = crate::memory_hierarchy::recall::compile(
        &no_basis,
        "fixed-scope",
        &history,
        r.clone(),
        None,
    )
    .unwrap()
    .trace;
    assert!(!b.relations.iter().any(|r| r.kind == K::PolicyBasis));
    // Changing an undisclosed typed intermediate must not change visible
    // topology, reasons, labels, counts or the public source digest.
    let mut hidden = no_basis.clone();
    for e in &mut hidden.known_by_then {
        if let TransitionPayload::OperationRecorded { operation } = &mut e.payload {
            operation.origin = crate::effect::OperationOrigin::ProviderResult {
                provider_result_id: "hidden:result".into(),
                provider_invocation_id: "hidden:invocation".into(),
            };
        }
    }
    let x = crate::memory_hierarchy::recall::compile(&hidden, "fixed-scope", &history, r, None)
        .unwrap()
        .trace;
    assert_eq!(b, x);
    assert!(!serde_json::to_string(&x).unwrap().contains("hidden:"));
    assert_eq!(history, w.store.list_case_transitions(CASE).unwrap());
    println!("recall_counterfactual scope=resolver_contract normative_link=present_vs_absent chronology=unchanged missing_link_not_invented=true hidden_intermediate=semantic_identity_equal canonical_mutation=0");
    w.finish();
}
