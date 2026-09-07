use super::i03_tests::i03_setup_case;
use super::*;
use crate::cognitive::CognitiveCapability;
use crate::conversation::{
    CognitiveCompositionPrerequisite, CognitiveCompositionRequest, ContentModality,
    ContentPartProvenance, ConversationContentObject, ConversationContentPart, ConversationTurn,
};
use crate::transition::{TransitionScope, TRANSITION_SCHEMA_V17};

#[test]
fn delegated_intent_is_exact_scoped_atomic_and_replayable() {
    use crate::conversation::{authorize_turn_execution, DELEGATED_COMPOSITION_REQUEST_SCHEMA};
    let path = temp_store_path("golden-delegated-intent");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(20621);
    i03_setup_case(&store, &owner, "tenant:i06", "case:i06");
    let generation = store
        .get_case_state("case:i06")
        .unwrap()
        .unwrap()
        .generation;
    store
        .commit_secured_transition(
            &owner,
            "tenant:i06",
            secured_pending(
                "transition:executor",
                "case:i06",
                generation,
                &owner.projected_principal_id(),
                TransitionPayload::ParticipantBound {
                    participant_id: "participant:executor".into(),
                    role: "model-executor".into(),
                },
            ),
            true,
        )
        .unwrap();
    let before = store.list_case_transitions("case:i06").unwrap();
    let generation = store
        .get_case_state("case:i06")
        .unwrap()
        .unwrap()
        .generation;
    let (pending, _) = submission(&owner, generation);
    let TransitionPayload::ConversationTurnCommitted { turn } = &pending.payload else {
        panic!()
    };
    let request = CognitiveCompositionRequest::for_executor(
        turn,
        "participant:executor",
        CognitiveCapability::PrimaryConversation,
        vec![turn.ordered_parts[0].part_id.clone()],
        None,
    )
    .unwrap();
    assert_eq!(request.schema, DELEGATED_COMPOSITION_REQUEST_SCHEMA);
    assert!(
        CognitiveCompositionRequest::new(
            turn,
            "participant:executor",
            CognitiveCapability::PrimaryConversation,
            request.source_part_ids.clone(),
            None
        )
        .is_err(),
        "v1 retains its historical author/executor meaning"
    );
    assert!(store
        .commit_conversation_submission_authorized(&owner, "tenant:i06", pending, request)
        .unwrap_err()
        .contains("view_not_admitted"));
    assert_eq!(
        store.list_case_transitions("case:i06").unwrap(),
        before,
        "failed delegation does not expose half a SEND"
    );
    store
        .commit_secured_transition(
            &owner,
            "tenant:i06",
            secured_pending(
                "transition:executor-view",
                "case:i06",
                generation,
                &owner.projected_principal_id(),
                TransitionPayload::ParticipantAdmitted {
                    participant_id: "participant:executor".into(),
                    consumer: "model".into(),
                    view_kind: "model_context".into(),
                },
            ),
            true,
        )
        .unwrap();
    let (pending, _) = submission(&owner, generation + 1);
    let TransitionPayload::ConversationTurnCommitted { turn } = &pending.payload else {
        panic!()
    };
    let turn = turn.clone();
    let request = CognitiveCompositionRequest::for_executor(
        &turn,
        "participant:executor",
        CognitiveCapability::PrimaryConversation,
        vec![turn.ordered_parts[0].part_id.clone()],
        None,
    )
    .unwrap();
    let state = store.get_case_state("case:i06").unwrap().unwrap();
    assert_eq!(
        authorize_turn_execution(
            &state,
            &before,
            &turn,
            "participant:executor",
            &owner.projected_principal_id()
        )
        .unwrap_err(),
        "cognitive_realization_principal_participant_mismatch",
        "without canonical delegation a model view alone cannot authorize execution"
    );
    let (_, committed) = store
        .commit_conversation_submission_authorized(&owner, "tenant:i06", pending, request.clone())
        .unwrap();
    let history = store.list_case_transitions("case:i06").unwrap();
    let state = committed.state;
    authorize_turn_execution(
        &state,
        &history,
        &turn,
        "participant:executor",
        &owner.projected_principal_id(),
    )
    .unwrap();
    assert!(authorize_turn_execution(
        &state,
        &history,
        &turn,
        "participant:executor",
        "principal:spoofed"
    )
    .is_err());
    let mut wrong_case = state.clone();
    wrong_case.case_id = "case:other".into();
    assert!(authorize_turn_execution(
        &wrong_case,
        &history,
        &turn,
        "participant:executor",
        &owner.projected_principal_id()
    )
    .is_err());
    let mut no_disclosure = state.clone();
    no_disclosure
        .participants
        .iter_mut()
        .find(|p| p.participant_id == "participant:executor")
        .unwrap()
        .admitted_views
        .clear();
    assert!(authorize_turn_execution(
        &no_disclosure,
        &history,
        &turn,
        "participant:executor",
        &owner.projected_principal_id()
    )
    .is_err());
    let mut old = committed.transition.clone();
    old.schema = TRANSITION_SCHEMA_V17.into();
    assert!(old
        .validate()
        .unwrap_err()
        .contains("requires_yai_transition_v18"));
    let mut invalid_history = history.clone();
    invalid_history.last_mut().unwrap().source.participant_id = Some("participant:executor".into());
    assert!(crate::transition::replay_case("case:i06", &invalid_history).is_err());
    assert_eq!(store.replay_case_state("case:i06").unwrap(), state);
    drop(store);
    let store = LmdbRecordStore::open(&path).unwrap();
    assert_eq!(store.rebuild_case_state("case:i06").unwrap(), state);
    assert_eq!(
        store
            .record_conversation_execution_intent_authorized(&owner, request.clone())
            .unwrap(),
        request
    );
    assert_eq!(
        store
            .get_case_state("case:i06")
            .unwrap()
            .unwrap()
            .principal_participant_links
            .len(),
        1
    );
    println!("delegated_intent: exact_turn=true no_principal_transfer=true view_required=true atomic_send=true replay=true provider_mode=no_provider");
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

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
fn workflow_host_intent_is_unique_bounded_atomic_and_replayable() {
    use crate::workflow::{
        ModelWorkOutputContract, WorkflowBudgets, WorkflowExecutorBinding, WorkflowNodeKind,
        WorkflowPredicate,
    };
    let path = temp_store_path("golden-workflow-intent");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(20631);
    i03_setup_case(&store, &owner, "tenant:i06", "case:i06");
    let generation = store
        .get_case_state("case:i06")
        .unwrap()
        .unwrap()
        .generation;
    store
        .commit_secured_transition(
            &owner,
            "tenant:i06",
            secured_pending(
                "transition:workflow-model-view",
                "case:i06",
                generation,
                &owner.projected_principal_id(),
                TransitionPayload::ParticipantAdmitted {
                    participant_id: "participant:model".into(),
                    consumer: "model".into(),
                    view_kind: "model_context".into(),
                },
            ),
            true,
        )
        .unwrap();
    let mut input = h15_model_definition("tenant:i06", "golden-workflow-intent");
    let budgets = WorkflowBudgets::default();
    input.nodes[0].kind = WorkflowNodeKind::ModelWork {
        executor_slot: "model".into(),
        task: "explicit conversational input".into(),
        completion: WorkflowPredicate::ExecutionCaseWorkCompleted,
        budgets: budgets.clone(),
        resource_slot: None,
        output_contract: ModelWorkOutputContract::CaseWork,
    };
    let definition = store.define_workflow(&owner, input, 10).unwrap();
    store
        .bind_case_workflow(
            &owner,
            "case:i06",
            &definition.workflow_definition_id,
            vec![WorkflowExecutorBinding {
                slot: "model".into(),
                participant_id: "participant:model".into(),
            }],
            vec![],
            11,
        )
        .unwrap();
    let (execution, _, _) = store
        .start_workflow_model_work_authorized(&owner, "case:i06", "analyze", 12)
        .unwrap();
    let before = store.list_case_transitions("case:i06").unwrap();
    let state = store.get_case_state("case:i06").unwrap().unwrap();
    let (pending, request) = submission(&owner, state.generation);
    let request = request
        .with_work_limits(budgets.case_work_limits().unwrap())
        .unwrap()
        .with_workflow_execution(&execution.execution_id)
        .unwrap();
    // A fresh digest cannot authorize a different budget or node identity.
    for invalid in [
        request
            .clone()
            .with_workflow_execution("workflow-execution:forged")
            .unwrap(),
        {
            let mut limits = budgets.case_work_limits().unwrap();
            limits.invocations += 1;
            request.clone().with_work_limits(limits).unwrap()
        },
    ] {
        assert!(store
            .commit_conversation_submission_authorized(
                &owner,
                "tenant:i06",
                pending.clone(),
                invalid
            )
            .is_err());
        assert_eq!(
            store.list_case_transitions("case:i06").unwrap(),
            before,
            "rejected intent cannot expose half a SEND"
        );
    }
    store
        .commit_conversation_submission_authorized(&owner, "tenant:i06", pending, request.clone())
        .unwrap();
    let accepted = store.list_case_transitions("case:i06").unwrap();
    let state = store.get_case_state("case:i06").unwrap().unwrap();
    let (second, second_request) = submission(&owner, state.generation);
    let second_request = second_request
        .with_work_limits(budgets.case_work_limits().unwrap())
        .unwrap()
        .with_workflow_execution(&execution.execution_id)
        .unwrap();
    assert!(store
        .commit_conversation_submission_authorized(&owner, "tenant:i06", second, second_request)
        .unwrap_err()
        .contains("workflow_execution_intent_already_adopted"));
    assert_eq!(store.list_case_transitions("case:i06").unwrap(), accepted);
    assert_eq!(store.rebuild_case_state("case:i06").unwrap(), state);
    drop(store);
    let store = LmdbRecordStore::open(&path).unwrap();
    store
        .revalidate_conversation_workflow_intent_authorized(&owner, &request)
        .unwrap();
    let (reopened, _, _) = store
        .start_workflow_model_work_authorized(&owner, "case:i06", "analyze", 99)
        .unwrap();
    assert_eq!(reopened, execution);
    assert_eq!(store.list_case_transitions("case:i06").unwrap(), accepted);
    println!("golden_workflow_intent: execution={} request={} half_send=0 duplicate_turn=0 replay_equal=true provider_dispatches=0", execution.execution_id, request.request_id);
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn golden_reopens_workflow_v2_metadata_without_rewriting_definition_or_history() {
    let path = temp_store_path("golden-workflow-v2-home");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(20632);
    i03_setup_case(&store, &owner, "tenant:i06", "case:i06");
    let mut input = h15_model_definition("tenant:i06", "preserved-v2");
    input.schema = crate::workflow::WORKFLOW_DEFINITION_SCHEMA_V2.into();
    let definition = store.define_workflow(&owner, input, 1).unwrap();
    let before = store.list_case_transitions("case:i06").unwrap();
    let mut txn = store.env.begin_rw_txn().unwrap();
    txn.put(
        store.schema_meta,
        &"meta:workflow_definition_schema",
        &crate::workflow::WORKFLOW_DEFINITION_SCHEMA_V2,
        WriteFlags::empty(),
    )
    .unwrap();
    txn.commit().unwrap();
    drop(store);
    let reopened = LmdbRecordStore::open(&path).unwrap();
    assert_eq!(
        reopened
            .get_workflow_definition_authorized(&owner, &definition.workflow_definition_id)
            .unwrap(),
        definition
    );
    assert_eq!(reopened.list_case_transitions("case:i06").unwrap(), before);
    assert!(reopened.verify_case_state("case:i06").unwrap());
    println!("golden_workflow_v2_home: reopened=true definition_rewritten=false history_rewritten=false replay_equal=true");
    drop(reopened);
    fs::remove_dir_all(path).unwrap();
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
        "participant:model",
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
