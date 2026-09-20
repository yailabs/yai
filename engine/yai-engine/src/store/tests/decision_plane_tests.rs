use super::*;
use crate::cognitive::{
    CognitiveDecisionBudget, CognitiveDecisionCandidate, CognitiveDecisionCandidateOrigin,
    CognitiveDecisionDistribution, CognitiveDecisionFrontier,
    CognitiveDecisionFrontierCompleteness, CognitiveDecisionFrontierRequest,
    CognitiveDecisionOutput, CognitiveDecisionProducerIdentity, CognitiveDecisionScore,
    CognitiveScoreDirection, CognitiveScoreSemantics, CognitiveUncertaintyMetric,
    COGNITIVE_DECISION_OUTPUT_SCHEMA,
};
use crate::semantic_state::{
    CompilationRequest, SemanticPurpose, SemanticScope, WorkingStateRequest,
};
use std::time::Instant;

use crate::workflow::{
    ModelWorkOutputContract, WorkflowBudgets, WorkflowDefinitionInput, WorkflowEdge,
    WorkflowEdgeKind, WorkflowExecutorBinding, WorkflowNode, WorkflowNodeKind, WorkflowPredicate,
    WorkflowResourceBinding, WORKFLOW_DEFINITION_SCHEMA,
};

fn candidate(
    candidate_id: &str,
    candidate_kind: &str,
    description: &str,
    semantic_refs: &[&str],
) -> CognitiveDecisionCandidate {
    CognitiveDecisionCandidate {
        candidate_id: candidate_id.into(),
        candidate_kind: candidate_kind.into(),
        description: description.into(),
        semantic_refs: semantic_refs.iter().map(|value| (*value).into()).collect(),
    }
}

fn budget(max_candidates: usize) -> CognitiveDecisionBudget {
    CognitiveDecisionBudget {
        max_candidates,
        max_result_bytes: 64 * 1024,
        max_compute_millis: 1_000,
    }
}

fn producer() -> CognitiveDecisionProducerIdentity {
    CognitiveDecisionProducerIdentity {
        capability_id: "fixture:finite-candidate-scoring".into(),
        producer_id: "fixture:deterministic-decision-oracle".into(),
        producer_version: "v1".into(),
        model_id: None,
        composition_id: None,
        profile_id: None,
        state_generation: None,
        evidence_refs: vec!["test:cognitive-decision-plane".into()],
    }
}

fn output(request_id: &str, scores: Vec<(&str, i64)>) -> CognitiveDecisionOutput {
    CognitiveDecisionOutput {
        schema: COGNITIVE_DECISION_OUTPUT_SCHEMA.into(),
        request_id: request_id.into(),
        producer: producer(),
        score_semantics: CognitiveScoreSemantics::RelativeCandidateProbability {
            normalization: "fixture:fixed-point-softmax".into(),
        },
        score_direction: CognitiveScoreDirection::HigherIsPreferred,
        score_scale: 1_000_000,
        scores: scores
            .into_iter()
            .map(|(candidate_id, value)| CognitiveDecisionScore {
                candidate_id: candidate_id.into(),
                value,
            })
            .collect(),
        uncertainty: Some(CognitiveUncertaintyMetric {
            metric_id: "fixture:normalized-entropy".into(),
            value: 270_000,
            scale: 1_000_000,
            interpretation: "Fixture metric; not calibrated correctness confidence".into(),
        }),
    }
}

fn working_request(world: &World) -> WorkingStateRequest {
    let state = world.store.get_case_state(CASE).unwrap().unwrap();
    WorkingStateRequest {
        case_id: CASE.into(),
        expected_generation: state.generation,
        compilation: CompilationRequest {
            scope: SemanticScope::model(HUMAN, SemanticPurpose::Inspection),
            intent: "choose the next bounded verification step".into(),
            output_contract_id: crate::context::InvocationOutputContract::NaturalLanguage
                .contract_id(),
            max_semantic_units: 131_072,
            max_derived_items: 8,
            resource_refs: vec![RESOURCE.into()],
            required_refs: vec![HUMAN.into(), RESOURCE.into()],
            previous_item_ids: vec![],
            view_selection_id: None,
        },
        recall_query: Some("current resource verification constraints".into()),
        at: None,
        recall_required_refs: vec![],
        recall_bounds: Default::default(),
        max_output_bytes: 1024 * 1024,
    }
}

fn bind_frontier_workflow(world: &World) {
    let definition = world
        .store
        .define_workflow(
            &world.owner,
            WorkflowDefinitionInput {
                schema: WORKFLOW_DEFINITION_SCHEMA.into(),
                tenant_id: TENANT.into(),
                workflow_key: "cognitive-frontier".into(),
                declared_version: "1".into(),
                name: "Cognitive frontier fixture".into(),
                description: "Typed passive progression followed by exact work".into(),
                nodes: vec![
                    WorkflowNode {
                        node_id: "current-case-open".into(),
                        kind: WorkflowNodeKind::Condition {
                            predicate: WorkflowPredicate::CaseLifecycle {
                                lifecycle: CaseLifecycle::Open,
                            },
                        },
                    },
                    WorkflowNode {
                        node_id: "verify-resource".into(),
                        kind: WorkflowNodeKind::ModelWork {
                            executor_slot: "operator".into(),
                            task: "Verify the exact task-bound Resource".into(),
                            completion: WorkflowPredicate::ExecutionProviderResult,
                            budgets: WorkflowBudgets::default(),
                            resource_slot: Some("workspace".into()),
                            output_contract: ModelWorkOutputContract::Text,
                        },
                    },
                ],
                edges: vec![WorkflowEdge {
                    from: "current-case-open".into(),
                    to: "verify-resource".into(),
                    kind: WorkflowEdgeKind::OnTrue,
                }],
            },
            1_790_001,
        )
        .unwrap();
    world
        .store
        .bind_case_workflow(
            &world.owner,
            CASE,
            &definition.workflow_definition_id,
            vec![WorkflowExecutorBinding {
                slot: "operator".into(),
                participant_id: HUMAN.into(),
            }],
            vec![WorkflowResourceBinding {
                slot: "workspace".into(),
                attachment_id: RESOURCE.into(),
            }],
            1_790_002,
        )
        .unwrap();
}

fn attach_frontier_resources(world: &World, count: usize) -> Vec<String> {
    let template = world.store.get_case_state(CASE).unwrap().unwrap().resources[0].clone();
    let mut ids = Vec::new();
    for index in 0..count {
        let resource_id = format!("resource:frontier-{index:02}");
        let mut attachment = template.clone();
        attachment.attachment_id = resource_id.clone();
        attachment.policy_id = format!("policy:frontier-{index:02}");
        if let Some(access) = &mut attachment.access {
            access.configuration_digest = digest_bytes(resource_id.as_bytes());
        }
        let state = world.store.get_case_state(CASE).unwrap().unwrap();
        let mut pending = secured_pending(
            &format!("transition:frontier-resource:{index:02}"),
            CASE,
            state.generation,
            &world.owner.projected_principal_id(),
            TransitionPayload::ResourceAttached { attachment },
        );
        pending.scope = Some(TransitionScope {
            case_id: CASE.into(),
            participant_refs: vec![HUMAN.into()],
            resource_refs: vec![resource_id.clone()],
            policy_refs: vec![format!("policy:frontier-{index:02}")],
        });
        pending.causal_refs = vec![HUMAN.into()];
        world
            .store
            .commit_secured_transition(&world.owner, TENANT, pending, true)
            .unwrap();
        ids.push(resource_id);
    }
    ids
}

#[test]
fn cognitive_decision_plane_is_w_bound_non_authoritative_and_current() {
    let world = World::new();
    let working_request = working_request(&world);
    let working = world
        .store
        .compile_working_state_authorized(&world.owner, working_request.clone(), None)
        .unwrap()
        .working_state;
    let alternatives = vec![
        candidate(
            "candidate:inspect-more",
            "yai.next-step.inspect-more.v1",
            "Inspect more admitted evidence",
            &[RESOURCE],
        ),
        candidate(
            "candidate:verify",
            "yai.next-step.verify.v1",
            "Verify the current resource observation",
            &[RESOURCE],
        ),
        candidate(
            "candidate:escalate",
            "yai.next-step.escalate-generation.v1",
            "Escalate to generative reasoning",
            &[],
        ),
        candidate(
            "candidate:stop",
            "yai.next-step.stop.v1",
            "Stop this bounded cognitive step",
            &[],
        ),
    ];

    let before_history = world.store.list_case_transitions(CASE).unwrap();
    let before_state = world.store.get_case_state(CASE).unwrap().unwrap();
    let prepare_started = Instant::now();
    let request = world
        .store
        .prepare_cognitive_decision_request_authorized(
            &world.owner,
            &working,
            "yai.next-cognitive-step.v1",
            alternatives.clone(),
            budget(8),
            None,
        )
        .unwrap();
    let prepare_us = prepare_started.elapsed().as_micros();
    assert_eq!(request.working_state_id, working.id());
    assert_eq!(request.task_id, working.semantic_task_id().unwrap());
    assert_eq!(request.case_id, CASE);
    assert_eq!(request.participant_id, HUMAN);

    let mut permuted = alternatives.clone();
    permuted.reverse();
    assert_eq!(
        request,
        world
            .store
            .prepare_cognitive_decision_request_authorized(
                &world.owner,
                &working,
                "yai.next-cognitive-step.v1",
                permuted,
                budget(8),
                None,
            )
            .unwrap(),
        "candidate order is not candidate meaning"
    );

    for reference in ["resource:hidden", "resource:foreign-case"] {
        let error = world
            .store
            .prepare_cognitive_decision_request_authorized(
                &world.owner,
                &working,
                "yai.next-cognitive-step.v1",
                vec![
                    candidate(
                        "candidate:foreign",
                        "yai.next-step.inspect-more.v1",
                        "Foreign or hidden candidate",
                        &[reference],
                    ),
                    candidate("candidate:stop", "yai.next-step.stop.v1", "Stop", &[]),
                ],
                budget(8),
                None,
            )
            .unwrap_err();
        assert_eq!(error, "cognitive_decision_candidate_reference_unavailable");
    }
    assert!(world
        .store
        .prepare_cognitive_decision_request_authorized(
            &world.outsider,
            &working,
            "yai.next-cognitive-step.v1",
            alternatives.clone(),
            budget(8),
            None,
        )
        .is_err());
    let duplicate = vec![alternatives[0].clone(), alternatives[0].clone()];
    assert_eq!(
        world
            .store
            .prepare_cognitive_decision_request_authorized(
                &world.owner,
                &working,
                "yai.next-cognitive-step.v1",
                duplicate,
                budget(8),
                None,
            )
            .unwrap_err(),
        "cognitive_decision_candidate_id_duplicate"
    );

    let fixture_output = output(
        &request.request_id,
        vec![
            ("candidate:verify", 700_000),
            ("candidate:stop", 100_000),
            ("candidate:inspect-more", 150_000),
            ("candidate:escalate", 50_000),
        ],
    );
    let qualification = world
        .store
        .qualify_cognitive_decision_distribution_authorized(
            &world.owner,
            &working,
            &request,
            fixture_output.clone(),
            None,
        )
        .unwrap();
    assert_eq!(
        qualification.distribution.preferred_candidate_id().unwrap(),
        "candidate:verify"
    );
    assert!(matches!(
        qualification.distribution.score_semantics,
        CognitiveScoreSemantics::RelativeCandidateProbability { .. }
    ));
    assert!(qualification.distribution.uncertainty.is_some());
    assert_eq!(
        world.store.list_case_transitions(CASE).unwrap(),
        before_history
    );
    let after_state = world.store.get_case_state(CASE).unwrap().unwrap();
    assert_eq!(after_state.last_operation, before_state.last_operation);
    assert_eq!(after_state.last_decision, before_state.last_decision);
    assert_eq!(after_state.grants, before_state.grants);
    assert_eq!(after_state.effects, before_state.effects);

    let mut score_permuted = fixture_output.clone();
    score_permuted.scores.reverse();
    assert_eq!(
        qualification.distribution,
        CognitiveDecisionDistribution::qualify(&request, score_permuted).unwrap()
    );
    let mut wrong_sum = fixture_output.clone();
    wrong_sum.scores[0].value += 1;
    assert_eq!(
        CognitiveDecisionDistribution::qualify(&request, wrong_sum).unwrap_err(),
        "cognitive_decision_probability_distribution_invalid"
    );
    let mut false_calibration = fixture_output.clone();
    false_calibration.score_semantics = CognitiveScoreSemantics::CalibratedProbability {
        calibration_artifact_id: "calibration:unproven".into(),
        calibration_scope: "fixture:test".into(),
    };
    assert_eq!(
        CognitiveDecisionDistribution::qualify(&request, false_calibration).unwrap_err(),
        "cognitive_decision_calibration_evidence_missing"
    );
    let mut raw = fixture_output.clone();
    raw.score_semantics = CognitiveScoreSemantics::RawScore {
        measure: "fixture:utility-logit".into(),
    };
    raw.scores[0].value = -9;
    assert!(CognitiveDecisionDistribution::qualify(&request, raw).is_ok());
    let mut invalid_uncertainty = fixture_output.clone();
    invalid_uncertainty.uncertainty.as_mut().unwrap().value = 1_000_001;
    assert_eq!(
        CognitiveDecisionDistribution::qualify(&request, invalid_uncertainty).unwrap_err(),
        "cognitive_decision_uncertainty_value_invalid"
    );
    let mut replaced_task = request.clone();
    replaced_task.decision_kind = "yai.silent-task-replacement.v1".into();
    assert_eq!(
        world
            .store
            .qualify_cognitive_decision_distribution_authorized(
                &world.owner,
                &working,
                &replaced_task,
                fixture_output.clone(),
                None,
            )
            .unwrap_err(),
        "cognitive_decision_request_integrity_mismatch"
    );

    // A distribution ID is not an Operation ID and cannot enter canonical
    // authority as though a high score had admitted it.
    assert_eq!(
        world
            .store
            .derive_policy_decision(CASE, &qualification.distribution.distribution_id)
            .unwrap_err(),
        "authority_operation_not_current"
    );
    assert_eq!(
        world.store.list_case_transitions(CASE).unwrap(),
        before_history
    );

    // Normal admission remains the only path to canonical Decision material.
    let operation = world.operation("request:cognitive-decision:verify", "src/retry.txt");
    let (decision, _) = world
        .store
        .derive_and_commit_policy_decision(CASE, &operation.operation_id)
        .unwrap();
    assert_ne!(
        decision.decision_id,
        qualification.distribution.distribution_id
    );
    assert_eq!(decision.operation_id, operation.operation_id);
    assert!(world
        .store
        .get_case_state(CASE)
        .unwrap()
        .unwrap()
        .effects
        .is_empty());

    // Rebuild/disposable derived structures and reopen the LMDB environment.
    world.store.clear_semantic_context_artifacts().unwrap();
    world.store.clear_case_operational_memory(CASE).unwrap();
    world.store.rebuild_graph_relations_for_case(CASE).unwrap();
    let mut current_request = working_request.clone();
    current_request.expected_generation = world
        .store
        .get_case_state(CASE)
        .unwrap()
        .unwrap()
        .generation;
    let current_working = world
        .store
        .compile_working_state_authorized(&world.owner, current_request.clone(), None)
        .unwrap()
        .working_state;
    let current_decision_request = world
        .store
        .prepare_cognitive_decision_request_authorized(
            &world.owner,
            &current_working,
            "yai.next-cognitive-step.v1",
            alternatives.clone(),
            budget(8),
            None,
        )
        .unwrap();
    let reopened = LmdbRecordStore::open(world.path.join("store")).unwrap();
    let reopened_working = reopened
        .compile_working_state_authorized(&world.owner, current_request, None)
        .unwrap()
        .working_state;
    assert_eq!(reopened_working, current_working);
    assert_eq!(
        reopened
            .prepare_cognitive_decision_request_authorized(
                &world.owner,
                &reopened_working,
                "yai.next-cognitive-step.v1",
                alternatives.clone(),
                budget(8),
                None,
            )
            .unwrap(),
        current_decision_request
    );
    drop(reopened);

    // Bounded candidate pressure: no provider is called and one exact vector is
    // interpreted without creating a canonical queue or transition.
    let large_candidates = (0..32)
        .map(|index| {
            candidate(
                &format!("candidate:bounded-{index:02}"),
                "yai.next-step.bounded-fixture.v1",
                &format!("Bounded fixture alternative {index}"),
                &[],
            )
        })
        .collect::<Vec<_>>();
    let large_started = Instant::now();
    let large_request = world
        .store
        .prepare_cognitive_decision_request_authorized(
            &world.owner,
            &current_working,
            "yai.bounded-pressure.v1",
            large_candidates,
            budget(32),
            None,
        )
        .unwrap();
    let large_prepare_us = large_started.elapsed().as_micros();
    let large_output = CognitiveDecisionOutput {
        schema: COGNITIVE_DECISION_OUTPUT_SCHEMA.into(),
        request_id: large_request.request_id.clone(),
        producer: producer(),
        score_semantics: CognitiveScoreSemantics::RelativeCandidateProbability {
            normalization: "fixture:uniform".into(),
        },
        score_direction: CognitiveScoreDirection::HigherIsPreferred,
        score_scale: 3_200,
        scores: large_request
            .candidates
            .iter()
            .map(|item| CognitiveDecisionScore {
                candidate_id: item.candidate_id.clone(),
                value: 100,
            })
            .collect(),
        uncertainty: None,
    };
    let large_qualification = world
        .store
        .qualify_cognitive_decision_distribution_authorized(
            &world.owner,
            &current_working,
            &large_request,
            large_output,
            None,
        )
        .unwrap();
    assert_eq!(large_qualification.distribution.scores.len(), 32);
    assert_eq!(
        world
            .store
            .get_case_state(CASE)
            .unwrap()
            .unwrap()
            .effects
            .len(),
        0
    );

    // Same-generation authority change invalidates W before any later score
    // preparation/acceptance. The old request is not an authorization lease.
    let generation_before_revoke = world
        .store
        .get_case_state(CASE)
        .unwrap()
        .unwrap()
        .generation;
    world
        .store
        .revoke_tenant_policy_artifact(
            &world.owner,
            &world.artifact_id,
            "cognitive decision current authority revoked",
        )
        .unwrap();
    assert_eq!(
        world
            .store
            .get_case_state(CASE)
            .unwrap()
            .unwrap()
            .generation,
        generation_before_revoke
    );
    assert_eq!(
        world
            .store
            .prepare_cognitive_decision_request_authorized(
                &world.owner,
                &current_working,
                "yai.next-cognitive-step.v1",
                alternatives,
                budget(8),
                None,
            )
            .unwrap_err(),
        "stale_semantic_working_state"
    );

    println!(
        "cognitive_decision_plane schema=v1 candidates=4 preferred=candidate:verify score_semantics=relative_candidate_probability calibrated=false uncertainty_separate=true hidden=absent wrong_case=absent permutation=stable score_authority=false transitions=0 effects=0 provider_calls=0 restart=equal same_generation_revoke=stale bounded_candidates=32 prepare_us={} result_requalification_us={} output_bytes={} large_prepare_us={} large_requalification_us={} large_output_bytes={}",
        prepare_us,
        qualification.current_requalification_us,
        qualification.output_bytes,
        large_prepare_us,
        large_qualification.current_requalification_us,
        large_qualification.output_bytes,
    );
    world.finish();
}

#[test]
fn cognitive_decision_frontier_is_typed_bounded_and_evolves_without_scoring() {
    let world = World::new();
    bind_frontier_workflow(&world);
    let mut request = working_request(&world);
    request.compilation.resource_refs = vec![RESOURCE.into(), RESOURCE.into()];
    request.compilation.scope.max_items = 128;
    request.expected_generation = world
        .store
        .get_case_state(CASE)
        .unwrap()
        .unwrap()
        .generation;
    let working_a = world
        .store
        .compile_working_state_authorized(&world.owner, request.clone(), None)
        .unwrap()
        .working_state;
    let frontier_request_a = CognitiveDecisionFrontierRequest::new(&working_a, 8).unwrap();
    let transitions_before_frontier = world.store.list_case_transitions(CASE).unwrap();
    let started = Instant::now();
    let qualified_a = world
        .store
        .derive_cognitive_decision_frontier_authorized(
            &world.owner,
            &working_a,
            frontier_request_a.clone(),
            None,
        )
        .unwrap();
    let ordinary_wall_us = started.elapsed().as_micros();
    assert_eq!(qualified_a.frontier.candidates.len(), 2);
    assert_eq!(qualified_a.frontier.visible_candidate_count, 2);
    assert_eq!(qualified_a.frontier.visible_origin_count, 2);
    assert_eq!(
        qualified_a.frontier.completeness,
        CognitiveDecisionFrontierCompleteness::CompleteAtTypedProfile
    );
    assert!(qualified_a.frontier.candidates.iter().any(|candidate| {
        candidate.origins.iter().any(|origin| {
            matches!(
                origin,
                CognitiveDecisionCandidateOrigin::WorkflowResolvableProgress { node_id, .. }
                    if node_id == "current-case-open"
            )
        })
    }));
    assert!(qualified_a.frontier.candidates.iter().any(|candidate| {
        candidate.origins.iter().any(|origin| matches!(
            origin,
            CognitiveDecisionCandidateOrigin::TaskResource { resource_ref, exact_required: true }
                if resource_ref == RESOURCE
        ))
    }));
    assert_eq!(
        world.store.list_case_transitions(CASE).unwrap(),
        transitions_before_frontier,
        "frontier construction is derived"
    );

    // Duplicate enumeration of one typed origin does not create duplicate
    // semantic candidates; origin order is representational only.
    let mut workflow_origins = qualified_a
        .frontier
        .candidates
        .iter()
        .flat_map(|candidate| candidate.origins.iter())
        .filter(|origin| {
            matches!(
                origin,
                CognitiveDecisionCandidateOrigin::WorkflowResolvableProgress { .. }
                    | CognitiveDecisionCandidateOrigin::WorkflowReadyWork { .. }
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    workflow_origins.extend(workflow_origins.clone());
    workflow_origins.reverse();
    assert_eq!(
        CognitiveDecisionFrontier::derive(&frontier_request_a, &working_a, workflow_origins)
            .unwrap(),
        qualified_a.frontier
    );

    let multi_origin = CognitiveDecisionFrontier::derive(
        &frontier_request_a,
        &working_a,
        vec![
            qualified_a
                .frontier
                .candidates
                .iter()
                .flat_map(|candidate| candidate.origins.iter())
                .find(|origin| {
                    matches!(
                        origin,
                        CognitiveDecisionCandidateOrigin::WorkflowResolvableProgress { .. }
                    )
                })
                .unwrap()
                .clone(),
            CognitiveDecisionCandidateOrigin::TaskResource {
                resource_ref: RESOURCE.into(),
                exact_required: false,
            },
        ],
    )
    .unwrap();
    let merged_resource = multi_origin
        .candidates
        .iter()
        .find(|candidate| candidate.candidate.semantic_refs == vec![RESOURCE.to_string()])
        .unwrap();
    assert_eq!(merged_resource.origins.len(), 2);
    assert_eq!(
        merged_resource.requirement,
        crate::cognitive::CognitiveDecisionCandidateRequirement::Required
    );

    let decision_request = world
        .store
        .prepare_cognitive_decision_request_from_frontier_authorized(
            &world.owner,
            &working_a,
            &qualified_a.frontier,
            "yai.next-cognitive-step.v1",
            budget(8),
            None,
        )
        .unwrap();
    assert_eq!(
        decision_request.candidates,
        qualified_a.frontier.decision_candidates()
    );
    assert_eq!(
        world
            .store
            .derive_policy_decision(CASE, &qualified_a.frontier.frontier_id)
            .unwrap_err(),
        "authority_operation_not_current"
    );
    assert!(world
        .store
        .derive_cognitive_decision_frontier_authorized(
            &world.outsider,
            &working_a,
            frontier_request_a.clone(),
            None,
        )
        .is_err());
    let mut cross_case_request = frontier_request_a.clone();
    cross_case_request.case_id = "case:foreign".into();
    assert_eq!(
        world
            .store
            .derive_cognitive_decision_frontier_authorized(
                &world.owner,
                &working_a,
                cross_case_request,
                None,
            )
            .unwrap_err(),
        "cognitive_decision_frontier_request_integrity_mismatch"
    );
    let visible_frontier = serde_json::to_string(&qualified_a.frontier).unwrap();
    assert!(!visible_frontier.contains("resource:hidden"));
    assert!(!visible_frontier.contains("case:foreign"));

    // A normal canonical Workflow consequence changes the next exact frontier
    // without any model choosing or generating alternatives.
    let advanced = world
        .store
        .advance_workflow_passive_progress(&world.owner, CASE, 4)
        .unwrap();
    assert!(advanced
        .ready_work
        .iter()
        .any(|work| work.node_id == "verify-resource"));
    request.expected_generation = advanced.case_generation;
    let working_b = world
        .store
        .compile_working_state_authorized(&world.owner, request.clone(), None)
        .unwrap()
        .working_state;
    let qualified_b = world
        .store
        .derive_cognitive_decision_frontier_authorized(
            &world.owner,
            &working_b,
            CognitiveDecisionFrontierRequest::new(&working_b, 8).unwrap(),
            None,
        )
        .unwrap();
    assert_ne!(
        qualified_a.frontier.frontier_id,
        qualified_b.frontier.frontier_id
    );
    assert!(!qualified_b.frontier.candidates.iter().any(|candidate| {
        candidate.origins.iter().any(|origin| {
            matches!(
                origin,
                CognitiveDecisionCandidateOrigin::WorkflowResolvableProgress { node_id, .. }
                    if node_id == "current-case-open"
            )
        })
    }));
    assert!(qualified_b.frontier.candidates.iter().any(|candidate| {
        candidate.origins.iter().any(|origin| {
            matches!(
                origin,
                CognitiveDecisionCandidateOrigin::WorkflowReadyWork { node_id, .. }
                    if node_id == "verify-resource"
            )
        })
    }));
    assert!(world
        .store
        .prepare_cognitive_decision_request_from_frontier_authorized(
            &world.owner,
            &working_a,
            &qualified_a.frontier,
            "yai.next-cognitive-step.v1",
            budget(8),
            None,
        )
        .is_err());

    let reopened = LmdbRecordStore::open(world.path.join("store")).unwrap();
    let reopened_working = reopened
        .compile_working_state_authorized(&world.owner, request.clone(), None)
        .unwrap()
        .working_state;
    let reopened_frontier = reopened
        .derive_cognitive_decision_frontier_authorized(
            &world.owner,
            &reopened_working,
            CognitiveDecisionFrontierRequest::new(&reopened_working, 8).unwrap(),
            None,
        )
        .unwrap();
    assert_eq!(reopened_working, working_b);
    assert_eq!(reopened_frontier.frontier, qualified_b.frontier);
    drop(reopened);

    // Typed task-Resource pressure supplies optional opportunities. Required
    // Workflow obligations are retained first; optional omission is explicit.
    let resource_ids = attach_frontier_resources(&world, 40);
    let mut pressure_request = working_request(&world);
    pressure_request.expected_generation = world
        .store
        .get_case_state(CASE)
        .unwrap()
        .unwrap()
        .generation;
    pressure_request.compilation.scope.max_items = 128;
    pressure_request.compilation.resource_refs = resource_ids.clone();
    pressure_request.compilation.required_refs = vec![HUMAN.into()];
    let pressure_working = world
        .store
        .compile_working_state_authorized(&world.owner, pressure_request.clone(), None)
        .unwrap()
        .working_state;
    let pressure_started = Instant::now();
    let pressure = world
        .store
        .derive_cognitive_decision_frontier_authorized(
            &world.owner,
            &pressure_working,
            CognitiveDecisionFrontierRequest::new(&pressure_working, 32).unwrap(),
            None,
        )
        .unwrap();
    let pressure_wall_us = pressure_started.elapsed().as_micros();
    assert_eq!(pressure.frontier.candidates.len(), 32);
    assert_eq!(pressure.frontier.required_candidate_count, 1);
    assert_eq!(pressure.frontier.optional_candidate_count, 40);
    assert_eq!(pressure.frontier.omitted_optional_candidates, 9);
    assert_eq!(pressure.frontier.visible_candidate_count, 41);
    assert_eq!(
        pressure.frontier.completeness,
        CognitiveDecisionFrontierCompleteness::OptionalCandidatesOmitted { omitted_count: 9 }
    );

    let mut mandatory_request = pressure_request;
    mandatory_request.compilation.resource_refs = resource_ids[..33].to_vec();
    mandatory_request.compilation.required_refs = std::iter::once(HUMAN.to_string())
        .chain(resource_ids[..33].iter().cloned())
        .collect();
    let mandatory_working = world
        .store
        .compile_working_state_authorized(&world.owner, mandatory_request, None)
        .unwrap()
        .working_state;
    assert_eq!(
        world
            .store
            .derive_cognitive_decision_frontier_authorized(
                &world.owner,
                &mandatory_working,
                CognitiveDecisionFrontierRequest::new(&mandatory_working, 32).unwrap(),
                None,
            )
            .unwrap_err(),
        "cognitive_decision_frontier_required_bound_exceeded"
    );

    let generation_before_revoke = world
        .store
        .get_case_state(CASE)
        .unwrap()
        .unwrap()
        .generation;
    world
        .store
        .revoke_tenant_policy_artifact(
            &world.owner,
            &world.artifact_id,
            "frontier current authority revoked",
        )
        .unwrap();
    assert_eq!(
        world
            .store
            .get_case_state(CASE)
            .unwrap()
            .unwrap()
            .generation,
        generation_before_revoke
    );
    assert_eq!(
        world
            .store
            .prepare_cognitive_decision_request_from_frontier_authorized(
                &world.owner,
                &pressure_working,
                &pressure.frontier,
                "yai.next-cognitive-step.v1",
                budget(32),
                None,
            )
            .unwrap_err(),
        "stale_semantic_working_state"
    );

    let final_state = world.store.get_case_state(CASE).unwrap().unwrap();
    assert!(final_state.last_operation.is_none());
    assert_eq!(final_state.effects.len(), 0);
    println!(
        "cognitive_decision_frontier schema=v1 ordinary_candidates=2 ordinary_origins={} ordinary_requalification_us={} ordinary_generation_us={} ordinary_wall_us={} ordinary_output_bytes={} workflow_state_evolution=true duplicate_origin_dedup=true multi_origin_closure=true hidden=absent cross_case=absent decision_request_v1=unchanged scoring_calls=0 model_calls=0 transitions_from_frontier=0 operations=0 effects=0 restart=equal optional_visible=41 optional_selected=31 optional_omitted=9 pressure_requalification_us={} pressure_generation_us={} pressure_wall_us={} pressure_output_bytes={} mandatory_overflow=refused same_generation_revoke=stale",
        qualified_a.origin_count,
        qualified_a.current_requalification_us,
        qualified_a.candidate_generation_us,
        ordinary_wall_us,
        qualified_a.output_bytes,
        pressure.current_requalification_us,
        pressure.candidate_generation_us,
        pressure_wall_us,
        pressure.output_bytes,
    );
    world.finish();
}
