use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use yai_application::{LocalApplication, OperationRequest, APPLICATION_PROTOCOL};
use yai_core_engine::security::AuthenticatedPrincipal;
use yai_core_engine::store::lmdb::LmdbRecordStore;
use yai_core_engine::transition::{
    PendingTransition, PrincipalParticipantLink, TransitionPayload, TransitionSource,
};

fn request(operation_ref: &str, input: serde_json::Value) -> OperationRequest {
    OperationRequest {
        protocol: APPLICATION_PROTOCOL.into(),
        operation_ref: operation_ref.into(),
        correlation_ref: format!("test:{operation_ref}"),
        input,
    }
}

#[test]
fn lists_attaches_and_projects_one_real_case_without_fixtures() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let home = PathBuf::from(format!("/tmp/yai-application-live-{stamp}"));
    fs::create_dir_all(&home).unwrap();
    let auth = AuthenticatedPrincipal::authenticate_local().unwrap();
    let store = LmdbRecordStore::open(home.join("store/lmdb")).unwrap();
    store
        .bootstrap_local_security(
            &auth,
            "tenant:studio-live-test",
            "organization:studio-live-test",
            1,
        )
        .unwrap();
    let first = store
        .create_tenant_case(&auth, "tenant:studio-live-test", "case:studio-live-test")
        .unwrap();
    let participant = "participant:studio-operator";
    let participant_commit = store
        .commit_secured_transition(
            &auth,
            "tenant:studio-live-test",
            PendingTransition::new(
                "transition:studio-live-participant",
                "case:studio-live-test",
                first.state.generation,
                TransitionSource {
                    component: "yai.application.test".into(),
                    participant_id: None,
                    principal_id: Some(auth.projected_principal_id()),
                    source_ref: None,
                },
                TransitionPayload::ParticipantBound {
                    participant_id: participant.into(),
                    role: "operator".into(),
                },
            ),
            true,
        )
        .unwrap();
    let app = LocalApplication::from_yai_home(&home);
    let refused = app.call(request(
        "case.open",
        json!({ "case_ref": "case:studio-live-test" }),
    ));
    assert_eq!(
        refused.result_state,
        yai_application::ResultState::Unauthorized
    );
    let principal = auth.projected_principal_id();
    let link = PrincipalParticipantLink::new(
        "case:studio-live-test",
        "tenant:studio-live-test",
        &principal,
        participant,
        &principal,
        2,
    )
    .unwrap();
    let mut pending_link = PendingTransition::new(
        "transition:studio-live-principal-link",
        "case:studio-live-test",
        participant_commit.state.generation,
        TransitionSource {
            component: "yai.application.test".into(),
            participant_id: Some(participant.into()),
            principal_id: Some(principal.clone()),
            source_ref: Some(link.link_id.clone()),
        },
        TransitionPayload::ParticipantPrincipalLinked { link },
    );
    pending_link.causal_refs = vec![principal, participant.into()];
    store
        .commit_secured_transition(&auth, "tenant:studio-live-test", pending_link, true)
        .unwrap();
    let listed = app.call(request("case.list", json!({})));
    assert_eq!(
        listed.data.as_ref().unwrap()["cases"][0]["case_ref"],
        "case:studio-live-test"
    );
    assert_eq!(
        listed.data.as_ref().unwrap()["cases"][0]["display_name"],
        "Studio Live Test"
    );
    let opened = app.call(request(
        "case.open",
        json!({ "case_ref": "case:studio-live-test" }),
    ));
    assert_eq!(
        opened.data.as_ref().unwrap()["participant_ref"],
        participant
    );
    let snapshot = app.call(request(
        "case.summary",
        json!({ "case_ref": "case:studio-live-test" }),
    ));
    let body = snapshot.data.unwrap();
    assert_eq!(body["case"]["case_ref"], "case:studio-live-test");
    assert_eq!(body["case"]["display_name"], "Studio Live Test");
    assert_eq!(body["case"]["generation"], 3);
    assert_eq!(body["knowledge"]["status"], "empty");
    assert_eq!(body["work"]["status"], "empty");
    assert_eq!(body["compute"]["status"], "empty");
    assert_eq!(body["environment"]["files"], json!([]));
    assert!(body["memory"]["timeline"].as_array().unwrap().len() >= 2);
    let heartbeat = app.call(request(
        "events.heartbeat",
        json!({ "case_ref": "case:studio-live-test" }),
    ));
    assert_eq!(heartbeat.data.as_ref().unwrap()["generation"], 3);
    let stale = app.call(request(
        "case.summary",
        json!({ "case_ref": "case:studio-live-test", "expected_generation": 2 }),
    ));
    assert_eq!(stale.result_state, yai_application::ResultState::Stale);
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn typed_product_operations_bootstrap_and_mutate_without_cli() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let home = PathBuf::from(format!("/tmp/yai-application-parity-{stamp}"));
    fs::create_dir_all(&home).unwrap();
    let app = LocalApplication::from_yai_home(&home);

    let bootstrap = app.call(request(
        "identity.bootstrap",
        json!({
            "tenant_id": "tenant:application-parity",
            "organization_ref": "organization:application-parity"
        }),
    ));
    assert_eq!(
        bootstrap.result_state,
        yai_application::ResultState::Success
    );

    let identity = app.call(request("identity.current", json!({})));
    assert_eq!(identity.result_state, yai_application::ResultState::Success);
    assert_eq!(
        identity.data.as_ref().unwrap()["tenants"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let created = app.call(request(
        "case.create",
        json!({
            "tenant_id": "tenant:application-parity",
            "case_ref": "case:application-parity"
        }),
    ));
    assert_eq!(created.result_state, yai_application::ResultState::Success);

    let role = app.call(request(
        "participant.role.add",
        json!({
            "case_ref": "case:application-parity",
            "participant_ref": "participant:operator",
            "role": "operator"
        }),
    ));
    assert_eq!(role.result_state, yai_application::ResultState::Success);

    let linked = app.call(request(
        "participant.principal.link",
        json!({
            "case_ref": "case:application-parity",
            "participant_ref": "participant:operator",
            "principal_ref": "self"
        }),
    ));
    assert_eq!(linked.result_state, yai_application::ResultState::Success);

    let admitted = app.call(request(
        "participant.view.admit",
        json!({
            "case_ref": "case:application-parity",
            "participant_ref": "participant:operator",
            "consumer": "model",
            "view_kind": "model_context"
        }),
    ));
    assert_eq!(admitted.result_state, yai_application::ResultState::Success);

    let opened = app.call(request(
        "case.open",
        json!({ "case_ref": "case:application-parity" }),
    ));
    assert_eq!(opened.result_state, yai_application::ResultState::Success);
    assert_eq!(
        opened.data.as_ref().unwrap()["participant_ref"],
        "participant:operator"
    );

    let malformed = app.call(request(
        "participant.role.add",
        json!({
            "case_ref": "case:application-parity",
            "participant_ref": "participant:operator",
            "role": "not a role"
        }),
    ));
    assert_ne!(
        malformed.result_state,
        yai_application::ResultState::Success
    );

    let cancelled = app.call(request(
        "case.cancel",
        json!({
            "case_ref": "case:application-parity",
            "reason": "bounded application parity test"
        }),
    ));
    assert_eq!(
        cancelled.result_state,
        yai_application::ResultState::Success
    );
    let cancelled_generation = cancelled.data.as_ref().unwrap()["state"]["generation"]
        .as_u64()
        .unwrap();
    assert!(cancelled.data.as_ref().unwrap()["state"]["cancellation"].is_object());

    drop(app);
    let reopened = LocalApplication::from_yai_home(&home);
    let snapshot = reopened.call(request(
        "case.summary",
        json!({ "case_ref": "case:application-parity" }),
    ));
    assert_eq!(snapshot.result_state, yai_application::ResultState::Success);
    assert_eq!(
        snapshot.data.as_ref().unwrap()["case"]["generation"],
        cancelled_generation
    );
    fs::remove_dir_all(home).unwrap();
}
