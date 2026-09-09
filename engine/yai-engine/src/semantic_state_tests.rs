use super::*;
use crate::conversation::{
    ContentModality, ContentPartProvenance, ConversationContentObject, ConversationContentPart,
    ConversationTurn,
};
use crate::transition::{replay_case, TransitionScope, TransitionSource, TRANSITION_SCHEMA};

fn push(history: &mut Vec<Transition>, case: &str, payload: TransitionPayload) {
    let sequence = history.len() as u64 + 1;
    let mut t = Transition {
        schema: TRANSITION_SCHEMA.into(),
        transition_id: format!("transition:{case}:{sequence}"),
        case_id: case.into(),
        sequence,
        committed_at_unix_ms: sequence,
        source: TransitionSource {
            component: "semantic-state-qualification".into(),
            principal_id: Some("principal:operator".into()),
            participant_id: None,
            source_ref: None,
        },
        scope: None,
        causal_refs: vec![],
        payload,
        provenance: vec![],
        summary: None,
    };
    if let TransitionPayload::ConversationTurnCommitted { turn } = &t.payload {
        t.source.participant_id = Some(turn.participant_id.clone());
        t.source.principal_id = Some(turn.submitted_by_principal_id.clone());
        t.scope = Some(TransitionScope {
            case_id: case.into(),
            participant_refs: vec![turn.participant_id.clone()],
            resource_refs: vec![],
            policy_refs: vec![],
        });
        t.causal_refs.push(turn.participant_id.clone());
        t.causal_refs.extend(
            turn.ordered_parts
                .iter()
                .map(|p| p.object.object_id.clone()),
        );
    }
    match &t.payload {
        TransitionPayload::ParticipantPrincipalLinked { link } => {
            t.causal_refs
                .extend([link.principal_id.clone(), link.participant_id.clone()]);
        }
        TransitionPayload::ConversationExecutionIntentRecorded { request } => {
            let author = h_turn(history, &request.source_turn_id)
                .participant_id
                .clone();
            t.source.participant_id = Some(author.clone());
            t.source.source_ref = Some(request.request_id.clone());
            t.scope = Some(TransitionScope {
                case_id: case.into(),
                participant_refs: vec![request.participant_id.clone(), author.clone()],
                resource_refs: vec![],
                policy_refs: vec![],
            });
            t.causal_refs.extend([
                author,
                request.request_id.clone(),
                request.source_turn_id.clone(),
                request.participant_id.clone(),
            ]);
            t.causal_refs.extend(request.source_part_ids.clone());
        }
        TransitionPayload::CasePolicyBound { binding }
        | TransitionPayload::CasePolicyReplaced { binding, .. } => {
            t.causal_refs.extend([
                binding.artifact_id.clone(),
                binding.publication_event_id.clone(),
            ]);
            t.causal_refs.extend(binding.replaces_binding_id.clone());
        }
        TransitionPayload::CasePolicyUnbound { binding_id, .. } => {
            t.causal_refs.push(binding_id.clone())
        }
        _ => {}
    }
    t.validate().unwrap();
    history.push(t);
}

fn history(case: &str) -> Vec<Transition> {
    let mut h = vec![];
    push(
        &mut h,
        case,
        TransitionPayload::TenantCaseOpened {
            lifecycle: CaseLifecycle::Open,
            tenant_id: "tenant:semantic".into(),
            principal_id: "principal:operator".into(),
        },
    );
    for participant in ["participant:model", "participant:private"] {
        push(
            &mut h,
            case,
            TransitionPayload::ParticipantBound {
                participant_id: participant.into(),
                role: "model-executor".into(),
            },
        );
        push(
            &mut h,
            case,
            TransitionPayload::ParticipantAdmitted {
                participant_id: participant.into(),
                consumer: "model".into(),
                view_kind: "model_context".into(),
            },
        );
    }
    h
}

fn turn(h: &mut Vec<Transition>, case: &str, participant: &str, text: &str) -> String {
    let object = ConversationContentObject::new(
        "tenant:semantic",
        case,
        ContentModality::Text,
        "text/plain;charset=utf-8",
        text.as_bytes(),
    )
    .unwrap();
    let part = ConversationContentPart::build(
        0,
        object,
        ContentPartProvenance::Original {
            imported_by_principal_id: "principal:operator".into(),
        },
    )
    .unwrap();
    let t = ConversationTurn::build(
        case,
        "tenant:semantic",
        "thread:main",
        participant,
        "principal:operator",
        h.len() as u64,
        vec![part],
    )
    .unwrap();
    let id = t.turn_id.clone();
    push(
        h,
        case,
        TransitionPayload::ConversationTurnCommitted { turn: t },
    );
    id
}

fn source(case: &str, history: &[Transition]) -> SemanticState {
    SemanticState::compose(&replay_case(case, history).unwrap(), history).unwrap()
}

fn h_turn<'a>(h: &'a [Transition], id: &str) -> &'a ConversationTurn {
    h.iter()
        .find_map(|t| match &t.payload {
            TransitionPayload::ConversationTurnCommitted { turn } if turn.turn_id == id => {
                Some(turn)
            }
            _ => None,
        })
        .unwrap()
}

fn request() -> CompilationRequest {
    CompilationRequest {
        scope: ProjectionRequest::model("participant:model", SemanticPurpose::Conversation),
        intent: "Investigate release constraint".into(),
        output_contract_id: crate::context::InvocationOutputContract::NaturalLanguage.contract_id(),
        max_semantic_units: 8192,
        max_derived_items: 8,
        resource_refs: vec![],
        required_refs: vec![],
        previous_item_ids: vec![],
        view_selection_id: None,
    }
}

#[test]
fn qualified_state_refuses_materialization_history_or_case_drift() {
    let h = history("case:s");
    let mut state = replay_case("case:s", &h).unwrap();
    state.participants[0]
        .roles
        .push("operation-reviewer".into());
    assert_eq!(
        SemanticState::compose(&state, &h).unwrap_err(),
        "semantic_state_materialization_replay_mismatch"
    );
    let mut broken = h.clone();
    broken[2].case_id = "case:other".into();
    assert!(SemanticState::compose(&state, &broken).is_err());
    let mut broken = h.clone();
    broken[2].sequence += 1;
    assert!(SemanticState::compose(&state, &broken).is_err());
}

#[test]
fn case_age_locality_and_explicit_requirement_survive_task_switch() {
    let mut results = vec![];
    for (case, count) in [("case:young", 2), ("case:old", 1200)] {
        let mut h = history(case);
        let constraint = turn(&mut h, case, "participant:model", "Release constraint: retry delay must never exceed 200ms; submitted requirement, not policy authority.");
        for n in 0..count {
            turn(
                &mut h,
                case,
                "participant:model",
                &format!("irrelevant past local note {n}"),
            );
        }
        let s = source(case, &h);
        let mut r = request();
        r.scope.max_interaction_turns = 2;
        r.required_refs.push(constraint.clone());
        let w = s.compile(&r).unwrap();
        assert!(w.entries.iter().any(|e| e.entry_id == constraint));
        assert_eq!(
            w.entries
                .iter()
                .filter(|e| matches!(e.value, SemanticValue::ConversationTurn { .. }))
                .count(),
            3
        );
        let mut changed_task = r.clone();
        changed_task.intent =
            "Validate different subgoal against the same release constraint".into();
        let next = s.compile(&changed_task).unwrap();
        assert_ne!(w.id(), next.id());
        assert!(next.entries.iter().any(|e| e.entry_id == constraint));
        assert!(matches!(
            next.entries
                .iter()
                .find(|e| e.entry_id == constraint)
                .unwrap()
                .posture,
            AuthorityPosture::CommittedApplicationContent
        ));
        println!("locality case={case} transitions={} selected={} semantic_units={} omitted={} required_retained=true", h.len(), w.bounds.selected_items, w.bounds.selected_semantic_units, w.bounds.omitted_items);
        results.push((w.bounds.selected_items, w.bounds.selected_semantic_units));
    }
    assert_eq!(results[0].0, results[1].0);
    assert!(results[1].1.abs_diff(results[0].1) < 64);
}

#[test]
fn participant_disclosure_precedes_relevance_and_rendering() {
    let mut h = history("case:scope");
    let secret = turn(
        &mut h,
        "case:scope",
        "participant:private",
        "private source cannot be recalled by model",
    );
    let s = source("case:scope", &h);
    let r = request();
    let model = s.compile(&r).unwrap();
    assert!(!serde_json::to_string(&model)
        .unwrap()
        .contains("private source"));
    let mut denied = r.clone();
    denied.required_refs.push(secret.clone());
    assert_eq!(
        s.compile(&denied).unwrap_err(),
        "semantic_required_source_unavailable"
    );
    denied.required_refs = vec!["unknown:source".into()];
    assert_eq!(
        s.compile(&denied).unwrap_err(),
        "semantic_required_source_unavailable"
    );
    let mut private = r.clone();
    private.scope.participant_id = "participant:private".into();
    assert!(s
        .compile(&private)
        .unwrap()
        .entries
        .iter()
        .any(|e| e.entry_id == secret));
    assert!(model.lower_context(&s, &private).is_err());
    let mut wrong_view = r.clone();
    wrong_view.scope.view_kind = "unadmitted".into();
    assert!(s.compile(&wrong_view).is_err());
}

#[test]
fn delta_equivalence_rebuild_and_stale_request_refusal() {
    let mut h = history("case:delta");
    turn(&mut h, "case:delta", "participant:model", "first task");
    let before = source("case:delta", &h);
    let r = request();
    let w = before.compile(&r).unwrap();
    turn(&mut h, "case:delta", "participant:model", "new local task");
    push(
        &mut h,
        "case:delta",
        TransitionPayload::ParticipantBound {
            participant_id: "participant:model".into(),
            role: "operation-proposer".into(),
        },
    );
    let after = source("case:delta", &h);
    let delta = derive_delta(&before, &after, &r).unwrap();
    assert!(delta
        .changes
        .iter()
        .any(|c| matches!(c, SemanticChange::Added { .. })));
    assert!(delta
        .changes
        .iter()
        .any(|c| matches!(c, SemanticChange::Replaced { .. })));
    let (incremental, mode) = compile_delta(&w, &before, &after, &r, &delta).unwrap();
    assert_eq!(mode, DeltaCompilationMode::FullRecompilation);
    assert_eq!(incremental, after.compile(&r).unwrap());
    let restored_history =
        serde_json::from_str::<Vec<Transition>>(&serde_json::to_string(&h).unwrap()).unwrap();
    assert_eq!(
        incremental,
        source("case:delta", &restored_history).compile(&r).unwrap()
    );
    assert!(w.lower_context(&after, &r).is_err());
    let mut corrupted = delta.clone();
    corrupted.changes.clear();
    assert!(compile_delta(&w, &before, &after, &r, &corrupted).is_err());
    let mut changed = r.clone();
    changed.max_semantic_units += 1;
    assert!(compile_delta(&w, &before, &after, &changed, &delta).is_err());
    assert!(derive_delta(&after, &before, &r).is_err());
    println!("semantic_delta full_equals_recompiled=true stale_delta_refused=true restart_reconstruction=true mode={mode:?}");
}

#[test]
fn mandatory_budget_and_serialized_working_state_tamper_fail_closed() {
    let h = history("case:budget");
    let s = source("case:budget", &h);
    let mut r = request();
    r.max_semantic_units = 1;
    assert!(s
        .compile(&r)
        .unwrap_err()
        .contains("budget_below_mandatory"));
    r = request();
    let w = s.compile(&r).unwrap();
    let mut encoded = serde_json::to_value(&w).unwrap();
    encoded["entries"] = serde_json::json!([]);
    let forged: SemanticWorkingState = serde_json::from_value(encoded).unwrap();
    assert!(forged.lower_context(&s, &r).is_err());
    let mut encoded = serde_json::to_value(&w).unwrap();
    encoded["compiler"] = serde_json::json!("yai.state_compiler.v99");
    let forged: SemanticWorkingState = serde_json::from_value(encoded).unwrap();
    assert!(forged.lower_context(&s, &r).is_err());
}

#[test]
fn compatibility_lowering_is_target_independent_and_provenance_exact() {
    let mut h = history("case:lower");
    turn(&mut h, "case:lower", "participant:model", "hello");
    let s = source("case:lower", &h);
    let r = request();
    let w = s.compile(&r).unwrap();
    let p = w.lower_context(&s, &r).unwrap();
    assert_eq!(p.bounds.working_state_id.as_deref(), Some(w.id()));
    let frame = crate::context::build_context_frame(
        &p,
        &r.intent,
        crate::context::InvocationOutputContract::NaturalLanguage,
    )
    .unwrap();
    assert_eq!(frame.entries, w.entries);
    let mut renders = vec![];
    for name in ["ordinary-target", "Super-Whisper-GPT-DeepSeek"] {
        renders.push(
            crate::context::render_openai_compatible(
                &frame,
                &crate::context::ProviderModelProfile {
                    provider_id: name.into(),
                    provider_kind: "openai_compatible".into(),
                    model_id: name.into(),
                    structured_output_supported: false,
                    continuation_supported: false,
                },
                "auto",
            )
            .unwrap(),
        );
    }
    assert_eq!(renders[0].user_content, renders[1].user_content);
    assert_eq!(renders[0].system_content, renders[1].system_content);
    let left = w
        .residency_report(&p, "provider:a".into(), "model:a".into())
        .unwrap();
    let right = w
        .residency_report(&p, "provider:b".into(), "model:b".into())
        .unwrap();
    assert_eq!(left.selected_item_ids, right.selected_item_ids);
    assert_ne!(left.plan_id, right.plan_id);
}

#[test]
fn canonical_executor_delegation_is_exact_not_ambient_visibility() {
    let case = "case:delegation";
    let mut h = history(case);
    let id = turn(
        &mut h,
        case,
        "participant:private",
        "explicit operator SEND to model",
    );
    let secret = turn(
        &mut h,
        case,
        "participant:private",
        "different private work never delegated",
    );
    let mut r = request();
    r.required_refs.push(id.clone());
    assert!(source(case, &h).compile(&r).is_err());
    let link = crate::transition::PrincipalParticipantLink::new(
        case,
        "tenant:semantic",
        "principal:operator",
        "participant:private",
        "principal:operator",
        1,
    )
    .unwrap();
    push(
        &mut h,
        case,
        TransitionPayload::ParticipantPrincipalLinked { link },
    );
    let t = h_turn(&h, &id);
    let intent = crate::conversation::CognitiveCompositionRequest::for_executor(
        t,
        "participant:model",
        crate::cognitive::CognitiveCapability::PrimaryConversation,
        t.ordered_parts.iter().map(|p| p.part_id.clone()).collect(),
        None,
    )
    .unwrap();
    push(
        &mut h,
        case,
        TransitionPayload::ConversationExecutionIntentRecorded { request: intent },
    );
    let s = source(case, &h);
    let w = s.compile(&r).unwrap();
    assert!(w.entries.iter().any(|e| e.entry_id == id));
    assert!(!w.entries.iter().any(|e| e.entry_id == secret));
    assert!(!serde_json::to_string(&w.entries)
        .unwrap()
        .contains("different private work"));
    r.required_refs = vec![secret];
    assert_eq!(
        s.compile(&r).unwrap_err(),
        "semantic_required_source_unavailable"
    );
}

#[test]
fn policy_big_picture_supersession_and_removal_match_full_delta() {
    let case = "case:policy";
    let mut h = history(case);
    let r = request();
    let mut prior: Option<crate::case_policy::CasePolicyBinding> = None;
    for version in ["1", "2"] {
        let raw = serde_json::to_vec(&serde_json::json!({
            "schema": crate::governance::POLICY_SOURCE_INPUT_SCHEMA,
            "policy_key":"release", "source_version":version, "owner_ref":"organization:semantic",
            "source_origin":{"source_system":"component-test","source_uri":format!("test://release/{version}")},
            "validity":{"mode":"unbounded"},
            "rules":[{"kind":"operation_restriction","rule_id":"protect-source","operation_kind":"filesystem.write","resource_kind":"filesystem","effect":"deny","reason":"protected source denied"}]
        })).unwrap();
        let artifact = crate::governance::compile_policy_source(&raw)
            .unwrap()
            .artifact;
        let next = crate::case_policy::build_case_policy_binding(
            case,
            &artifact,
            &format!("policy-event:{version}"),
            version.parse().unwrap(),
            h.len() as u64 + 1,
            "participant:private",
            "qualified policy source",
            prior.as_ref().map(|b| b.binding_id.clone()),
        )
        .unwrap();
        let before = source(case, &h);
        let w = before.compile(&r).unwrap();
        let payload = match &prior {
            None => TransitionPayload::CasePolicyBound {
                binding: next.clone(),
            },
            Some(p) => TransitionPayload::CasePolicyReplaced {
                prior_binding_id: p.binding_id.clone(),
                binding: next.clone(),
            },
        };
        push(&mut h, case, payload);
        for n in 0..40 {
            turn(
                &mut h,
                case,
                "participant:model",
                &format!("irrelevant task {version} {n}"),
            );
        }
        let after = source(case, &h);
        let full = after.compile(&r).unwrap();
        let policy_id = format!("policy:{}", next.binding_id);
        assert!(full
            .entries
            .iter()
            .any(|e| e.entry_id == policy_id && e.posture == AuthorityPosture::ControlState));
        let mut another_task = r.clone();
        another_task.intent = "Inspect completely different local subgoal".into();
        assert!(after
            .compile(&another_task)
            .unwrap()
            .entries
            .iter()
            .any(|e| e.entry_id == policy_id));
        if let Some(p) = prior {
            assert!(!full
                .entries
                .iter()
                .any(|e| e.entry_id == format!("policy:{}", p.binding_id)));
            let mut stale_required = r.clone();
            stale_required
                .required_refs
                .push(format!("policy:{}", p.binding_id));
            assert_eq!(
                after.compile(&stale_required).unwrap_err(),
                "semantic_required_source_unavailable"
            );
        }
        let delta = derive_delta(&before, &after, &r).unwrap();
        assert_eq!(
            compile_delta(&w, &before, &after, &r, &delta).unwrap().0,
            full
        );
        prior = Some(next);
    }
    let prior = prior.unwrap();
    let before = source(case, &h);
    let w = before.compile(&r).unwrap();
    push(
        &mut h,
        case,
        TransitionPayload::CasePolicyUnbound {
            binding_id: prior.binding_id.clone(),
            lineage_id: prior.lineage_id,
            actor_ref: "participant:private".into(),
            reason: "explicit unbind".into(),
        },
    );
    let after = source(case, &h);
    let delta = derive_delta(&before, &after, &r).unwrap();
    assert!(delta.changes.iter().any(|c| matches!(c, SemanticChange::Removed { entry_id, .. } if entry_id == &format!("policy:{}", prior.binding_id))));
    assert_eq!(
        compile_delta(&w, &before, &after, &r, &delta).unwrap().0,
        after.compile(&r).unwrap()
    );
    assert!(w.lower_context(&after, &r).is_err());
    println!("policy current_binding_retained=true task_switch_preserved=true superseded_ref_refused=true removal_delta_equals_full=true old_working_state_stale=true");
}
