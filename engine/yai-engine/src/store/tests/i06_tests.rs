use super::i03_tests::i03_setup_case;
use super::*;
use crate::cognitive::CognitiveCapability;
use crate::conversation::{
    CognitiveCompositionPrerequisite, CognitiveCompositionRequest, ContentModality,
    ContentPartProvenance, ConversationContentObject, ConversationContentPart, ConversationTurn,
};
use crate::transition::TransitionScope;

fn submission(
    owner: &AuthenticatedPrincipal,
    generation: u64,
) -> (PendingTransition, CognitiveCompositionRequest) {
    let object = ConversationContentObject::new(
        "tenant:i06",
        "case:i06",
        ContentModality::Text,
        "text/plain",
        b"explicit conversational input",
    )
    .unwrap();
    let part = ConversationContentPart::build(
        0,
        object,
        ContentPartProvenance::Original {
            imported_by_principal_id: owner.projected_principal_id(),
        },
    )
    .unwrap();
    let turn = ConversationTurn::build(
        "case:i06",
        "tenant:i06",
        "thread:i06",
        "participant:model",
        &owner.projected_principal_id(),
        generation,
        vec![part],
    )
    .unwrap();
    let request = CognitiveCompositionRequest::new(
        &turn,
        "participant:model",
        CognitiveCapability::PrimaryConversation,
        vec![turn.ordered_parts[0].part_id.clone()],
        None,
    )
    .unwrap();
    let pending = PendingTransition {
        transition_id: format!("transition:{}", turn.turn_id),
        case_id: turn.case_id.clone(),
        expected_generation: generation,
        source: TransitionSource {
            component: "i06-fixture".into(),
            participant_id: Some(turn.participant_id.clone()),
            principal_id: Some(owner.projected_principal_id()),
            source_ref: Some(turn.turn_id.clone()),
        },
        scope: Some(TransitionScope {
            case_id: turn.case_id.clone(),
            participant_refs: vec![turn.participant_id.clone()],
            resource_refs: vec![],
            policy_refs: vec![],
        }),
        causal_refs: vec![
            turn.participant_id.clone(),
            turn.ordered_parts[0].object.object_id.clone(),
        ],
        payload: TransitionPayload::ConversationTurnCommitted { turn },
        provenance: vec![],
        summary: None,
    };
    (pending, request)
}

#[test]
fn conversation_intent_atomic_adoption_security_replay_and_compatibility() {
    let path = temp_store_path("i06-intent");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(20601);
    i03_setup_case(&store, &owner, "tenant:i06", "case:i06");
    let prior_history = store.list_case_transitions("case:i06").unwrap();
    let mut txn = store.env.begin_rw_txn().unwrap();
    txn.put(
        store.schema_meta,
        &"meta:canonical_transition_schema",
        &TRANSITION_SCHEMA_V16,
        WriteFlags::empty(),
    )
    .unwrap();
    txn.commit().unwrap();
    drop(store);
    let store = LmdbRecordStore::open(&path).unwrap();
    assert_eq!(
        prior_history,
        store.list_case_transitions("case:i06").unwrap(),
        "metadata upgrade must not rewrite historical transitions"
    );
    let txn = store.env.begin_ro_txn().unwrap();
    assert_eq!(
        txn.get(store.schema_meta, &"meta:canonical_transition_schema")
            .unwrap(),
        TRANSITION_SCHEMA.as_bytes()
    );
    drop(txn);
    let generation = store
        .get_case_state("case:i06")
        .unwrap()
        .unwrap()
        .generation;
    // Force failure in the second canonical append, not merely input validation:
    // occupy the future intent Transition ID with an unrelated valid fact.
    let (pending, request) = submission(&owner, generation + 1);
    let collision = secured_pending(
        &format!("transition:conversation-intent:{}", request.source_turn_id),
        "case:i06",
        generation,
        &owner.projected_principal_id(),
        TransitionPayload::ParticipantBound {
            participant_id: "participant:model".into(),
            role: "model-executor".into(),
        },
    );
    store
        .commit_secured_transition(&owner, "tenant:i06", collision, true)
        .unwrap();
    let before = store.list_case_transitions("case:i06").unwrap();
    assert!(store
        .commit_conversation_submission_authorized(&owner, "tenant:i06", pending, request)
        .is_err());
    assert_eq!(
        store.list_case_transitions("case:i06").unwrap(),
        before,
        "second append failure must roll back Turn too"
    );
    // Advance the history and adopt a distinct Turn.
    store
        .commit_secured_transition(
            &owner,
            "tenant:i06",
            secured_pending(
                "transition:i06-next",
                "case:i06",
                generation + 1,
                &owner.projected_principal_id(),
                TransitionPayload::ParticipantBound {
                    participant_id: "participant:model".into(),
                    role: "model-executor".into(),
                },
            ),
            true,
        )
        .unwrap();
    let (pending, request) = submission(&owner, generation + 2);
    let (turn_commit, intent_commit) = store
        .commit_conversation_submission_authorized(
            &owner,
            "tenant:i06",
            pending.clone(),
            request.clone(),
        )
        .unwrap();
    assert_eq!(
        intent_commit.state.generation,
        turn_commit.state.generation + 1
    );
    assert!(intent_commit.state.last_provider_result.is_none());
    let TransitionPayload::ConversationTurnCommitted { turn } = &turn_commit.transition.payload
    else {
        panic!()
    };
    let alternate = CognitiveCompositionRequest::new(
        turn,
        "participant:model",
        CognitiveCapability::PrimaryConversation,
        request.source_part_ids.clone(),
        Some(CognitiveCompositionPrerequisite {
            capability: CognitiveCapability::SpeechToText,
            source_part_ids: request.source_part_ids.clone(),
        }),
    )
    .unwrap();
    let before = store.list_case_transitions("case:i06").unwrap();
    assert_eq!(
        store
            .record_conversation_execution_intent_authorized(&owner, request.clone())
            .unwrap(),
        request
    );
    assert!(store
        .record_conversation_execution_intent_authorized(&owner, alternate)
        .unwrap_err()
        .contains("immutable"));
    assert!(store
        .commit_conversation_submission_authorized(&owner, "tenant:i06", pending, request.clone())
        .is_err());
    let mut forged = request.clone();
    forged.tenant_id = "tenant:foreign".into();
    assert!(store
        .record_conversation_execution_intent_authorized(&owner, forged)
        .is_err());
    assert!(store
        .record_conversation_execution_intent_authorized(
            &AuthenticatedPrincipal::for_test(20602),
            request.clone()
        )
        .is_err());
    let mut forged = conversation_intent_pending(
        &request,
        intent_commit.state.generation,
        "principal:spoofed",
    );
    forged.transition_id = "transition:i06-forgery".into();
    assert!(store
        .commit_secured_transition(&owner, "tenant:i06", forged, true)
        .is_err());
    assert_eq!(before, store.list_case_transitions("case:i06").unwrap());
    for version in 13..=16 {
        let mut old = turn_commit.transition.clone();
        old.schema = format!("yai.transition.v{version}");
        Transition::from_json(&old.to_json().unwrap()).unwrap();
    }
    let mut old = intent_commit.transition.clone();
    old.schema = "yai.transition.v16".into();
    assert!(old
        .validate()
        .unwrap_err()
        .contains("requires_yai_transition_v17"));
    let state = store.rebuild_case_state("case:i06").unwrap();
    drop(store);
    let reopened = LmdbRecordStore::open(&path).unwrap();
    assert_eq!(reopened.replay_case_state("case:i06").unwrap(), state);
    assert_eq!(reopened.list_case_transitions("case:i06").unwrap(), before);
    println!("i06_intent: atomic_second_append_rollback=true immutable=true spoofed_actor_rejected=true old_turn_readers=true restart_replay=true no_provider=true");
    drop(reopened);
    fs::remove_dir_all(path).unwrap();
}
