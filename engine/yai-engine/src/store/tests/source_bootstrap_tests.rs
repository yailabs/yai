use super::*;
use crate::effect::access::source::*;

fn declaration(w: &World) -> CaseSourceDeclaration {
    CaseSourceDeclaration {
        schema: SOURCE_DECLARATION_SCHEMA.into(),
        source_id: String::new(),
        case_id: CASE.into(),
        perimeter: "company".into(),
        logical_name: "source".into(),
        participant_id: HUMAN.into(),
        declared_by_principal_id: w.owner.projected_principal_id(),
        resource_attachment_id: RESOURCE.into(),
        configuration_digest: w.binding.digest(),
        roles: vec![SourceRole::Knowledge],
        action: ResourceAction::Discover {
            path: "src/retry.txt".into(),
        },
        bootstrap_policy: false,
        media_type: "text/plain".into(),
    }
    .seal()
    .unwrap()
}

#[test]
fn source_declaration_scope_bootstrap_reentry_and_forged_coverage_refuse() {
    let w = World::with_access(AccessKind::Discover);
    let d = declaration(&w);
    let before = w.store.list_case_transitions(CASE).unwrap();
    assert!(w.store.declare_case_source(&w.outsider, d.clone()).is_err());
    let mut hidden = d.clone();
    hidden.participant_id = "participant:model".into();
    assert!(w
        .store
        .declare_case_source(&w.owner, hidden.seal().unwrap())
        .is_err());
    let mut sibling = d.clone();
    sibling.action = ResourceAction::Discover {
        path: "protected/secret.txt".into(),
    };
    assert!(w
        .store
        .declare_case_source(&w.owner, sibling.seal().unwrap())
        .is_err());
    let mut setup = d.clone();
    setup.bootstrap_policy = true;
    setup.roles = vec![SourceRole::Policy];
    assert!(w
        .store
        .declare_case_source(&w.owner, setup.seal().unwrap())
        .unwrap_err()
        .contains("bootstrap_authority_unavailable"));
    assert_eq!(w.store.list_case_transitions(CASE).unwrap(), before);
    let state = w.store.declare_case_source(&w.owner, d.clone()).unwrap();
    assert_eq!(
        w.store.declare_case_source(&w.owner, d.clone()).unwrap(),
        state
    );
    assert_eq!(
        w.store
            .case_source_permission(&w.owner, CASE, "source", None)
            .unwrap()
            .outcome,
        DecisionOutcome::Deny,
        "unrecorded request has no source-provenance evidence"
    );
    let op = w
        .store
        .record_participant_resource_request(
            &w.owner,
            CASE,
            HUMAN,
            RESOURCE,
            "source-discovery",
            state.generation,
            d.request(),
        )
        .unwrap();
    w.store
        .derive_and_commit_policy_decision(CASE, &op.operation_id)
        .unwrap();
    let admitted = w
        .store
        .admit_resource_read_authorized(&w.owner, CASE, &op.operation_id)
        .unwrap();
    let observed =
        inspect_confined_tree(w.binding.root().unwrap(), "src/retry.txt", None, 4096, 16).unwrap();
    let observed = w
        .store
        .record_resource_observation_authorized(&w.owner, &admitted, observed)
        .unwrap();
    let forged = SourceProgress {
        schema: SOURCE_PROGRESS_SCHEMA.into(),
        progress_id: String::new(),
        source_id: d.source_id.clone(),
        previous_progress_id: None,
        attempt: 1,
        phase: SourcePhase::Acquired,
        revision: Some(SourceRevision::new(&d.source_id, vec![]).unwrap()),
        decision_ref: Some(observed.decision_id),
        detail: "forged complete".into(),
    }
    .seal()
    .unwrap();
    let error = w
        .store
        .progress_case_source(&w.owner, CASE, "source", forged)
        .unwrap_err();
    assert!(error.contains("coverage_mismatch"), "{error}");
    let p = SourceProgress {
        schema: SOURCE_PROGRESS_SCHEMA.into(),
        progress_id: String::new(),
        source_id: d.source_id.clone(),
        previous_progress_id: None,
        attempt: 1,
        phase: SourcePhase::Acquiring,
        revision: None,
        decision_ref: None,
        detail: "bounded attempt".into(),
    }
    .seal()
    .unwrap();
    let acquiring = w
        .store
        .progress_case_source(&w.owner, CASE, "source", p.clone())
        .unwrap();
    assert_eq!(
        w.store
            .progress_case_source(&w.owner, CASE, "source", p.clone())
            .unwrap(),
        acquiring
    );
    let mut stale = p;
    stale.detail = "different stale update".into();
    assert!(w
        .store
        .progress_case_source(&w.owner, CASE, "source", stale.seal().unwrap())
        .is_err());
    assert!(w
        .store
        .case_source_permission(&w.owner, CASE, "source", None)
        .is_ok());
    w.store
        .revoke_tenant_policy_artifact(&w.owner, &w.artifact_id, "source visibility revoke")
        .unwrap();
    assert!(w
        .store
        .case_source_permission(&w.owner, CASE, "source", None)
        .is_err());
    assert!(w.store.verify_case_state(CASE).unwrap());
    println!("source_admission outsider=refused unlinked_participant=refused sibling=refused bootstrap_reentry=refused forged_coverage=refused stale_progress=refused revoke_without_case_generation=refused");
    w.finish();
}

#[test]
fn source_revision_identity_schema_readers_restart_and_cache_amnesia() {
    let w = World::with_access(AccessKind::Discover);
    let d = declaration(&w);
    let state = w.store.declare_case_source(&w.owner, d.clone()).unwrap();
    let item = SourceRevisionItem {
        path: "src/retry.txt".into(),
        digest: digest_bytes(b"one"),
        bytes: 3,
        backing: SourceBacking::Content {
            admission_id: "case-content:first".into(),
        },
    };
    let r1 = SourceRevision::new(&d.source_id, vec![item.clone()]).unwrap();
    let mut same = item.clone();
    same.backing = SourceBacking::Content {
        admission_id: "case-content:retry".into(),
    };
    assert_eq!(
        r1.revision_id,
        SourceRevision::new(&d.source_id, vec![same])
            .unwrap()
            .revision_id
    );
    assert_ne!(
        r1.revision_id,
        SourceRevision::new("case-source:other", vec![item.clone()])
            .unwrap()
            .revision_id
    );
    let mut changed = item;
    changed.digest = digest_bytes(b"two");
    assert_ne!(
        r1.revision_id,
        SourceRevision::new(&d.source_id, vec![changed])
            .unwrap()
            .revision_id
    );
    let history = w.store.list_case_transitions(CASE).unwrap();
    let mut old = history.last().unwrap().clone();
    old.schema = crate::transition::TRANSITION_SCHEMA_V18.into();
    assert!(old
        .validate()
        .unwrap_err()
        .contains("requires_yai_transition_v19"));
    let mut value = serde_json::to_value(&state).unwrap();
    value["schema"] = crate::transition::CASE_STATE_SCHEMA_V15.into();
    assert!(CaseState::from_json(&value.to_string()).is_err());
    value["sources"] = serde_json::json!([]);
    assert!(CaseState::from_json(&value.to_string()).is_ok());
    w.store.clear_case_operational_memory(CASE).unwrap();
    w.store.clear_semantic_context_artifacts().unwrap();
    w.store.drop_effective_policy(CASE).unwrap();
    w.store.rebuild_graph_relations_for_case(CASE).unwrap();
    w.store.rebuild_effective_policy(CASE).unwrap();
    assert_eq!(w.store.get_case_state(CASE).unwrap().unwrap(), state);
    assert_eq!(w.store.list_case_transitions(CASE).unwrap(), history);
    let path = w.path.clone();
    let owner = w.owner.clone();
    drop(w.store);
    let reopened = LmdbRecordStore::open(path.join("store")).unwrap();
    assert_eq!(
        reopened
            .case_source_authorized(&owner, CASE, "source")
            .unwrap()
            .1
            .declaration,
        d
    );
    assert_eq!(reopened.rebuild_case_state(CASE).unwrap(), state);
    assert!(reopened.verify_case_state(CASE).unwrap());
    println!("source_identity same_material_same_revision=true distinct_logical_sources=true changed_revision=true previous_schema_forgery=refused restart=true replay=true cache_amnesia=true read_transitions=0");
    drop(reopened);
    fs::remove_dir_all(path).unwrap();
}
