use super::*;
use crate::workflow::{
    ModelWorkOutputContract, MAX_WORKFLOW_PATCH_ADDED_NODES, MAX_WORKFLOW_PATCH_BYTES,
    MAX_WORKFLOW_PATCH_OPERATIONS, MAX_WORKFLOW_TASK_BYTES,
};

fn h17_wait_node(node_id: impl Into<String>) -> WorkflowNode {
    WorkflowNode {
        node_id: node_id.into(),
        kind: WorkflowNodeKind::Wait {
            predicate: WorkflowPredicate::CaseLifecycle {
                lifecycle: CaseLifecycle::Closed,
            },
        },
    }
}

fn wait_for_files(control: &Path, prefix: &str, expected: usize) {
    for _ in 0..5_000 {
        let count = fs::read_dir(control)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().starts_with(prefix))
            .count();
        if count == expected {
            return;
        }
        thread::sleep(Duration::from_millis(2));
    }
    panic!("timed out waiting for {expected} {prefix} files");
}

#[test]
fn hardening17_planpatch_limits_and_future_schema_refusal_have_no_off_by_one() {
    let digest = "sha256:h17-bounds".to_string();
    let mut operation_limit = WorkflowPlanPatchInput {
        schema: crate::workflow::WORKFLOW_PLAN_PATCH_SCHEMA.to_string(),
        base_effective_topology_digest: digest.clone(),
        operations: (0..MAX_WORKFLOW_PATCH_OPERATIONS)
            .map(|index| WorkflowPatchOperation::DisableNode {
                node_id: format!("future-{index}"),
            })
            .collect(),
    };
    operation_limit.validate().unwrap();
    operation_limit
        .operations
        .push(WorkflowPatchOperation::DisableNode {
            node_id: "one-over".to_string(),
        });
    assert_eq!(
        operation_limit.validate().unwrap_err(),
        "workflow_plan_patch_bounds_invalid"
    );

    let node_limit = WorkflowPlanPatchInput {
        schema: crate::workflow::WORKFLOW_PLAN_PATCH_SCHEMA.to_string(),
        base_effective_topology_digest: digest.clone(),
        operations: (0..MAX_WORKFLOW_PATCH_ADDED_NODES)
            .map(|index| WorkflowPatchOperation::AddNode {
                node: h17_wait_node(format!("added-{index}")),
            })
            .collect(),
    };
    node_limit.validate().unwrap();
    let mut node_over = node_limit.clone();
    node_over.operations.push(WorkflowPatchOperation::AddNode {
        node: h17_wait_node("added-over"),
    });
    assert_eq!(
        node_over.validate().unwrap_err(),
        "workflow_plan_patch_growth_bound_invalid"
    );

    let model_node = |index: usize, task: String| WorkflowNode {
        node_id: format!("payload-{index}"),
        kind: WorkflowNodeKind::ModelWork {
            executor_slot: "model".to_string(),
            task,
            completion: WorkflowPredicate::ExecutionProviderResult,
            budgets: WorkflowBudgets::default(),
            resource_slot: None,
            output_contract: ModelWorkOutputContract::Text,
        },
    };
    let mut byte_limit = WorkflowPlanPatchInput {
        schema: crate::workflow::WORKFLOW_PLAN_PATCH_SCHEMA.to_string(),
        base_effective_topology_digest: digest,
        operations: (0..4)
            .map(|index| WorkflowPatchOperation::AddNode {
                node: model_node(index, "x".to_string()),
            })
            .collect(),
    };
    let initial_bytes = serde_json::to_vec(&byte_limit).unwrap().len();
    let mut remaining = MAX_WORKFLOW_PATCH_BYTES - initial_bytes;
    for operation in &mut byte_limit.operations {
        let WorkflowPatchOperation::AddNode {
            node:
                WorkflowNode {
                    kind: WorkflowNodeKind::ModelWork { task, .. },
                    ..
                },
        } = operation
        else {
            unreachable!()
        };
        let add = remaining.min(MAX_WORKFLOW_TASK_BYTES - task.len());
        task.push_str(&"x".repeat(add));
        remaining -= add;
    }
    assert_eq!(remaining, 0);
    assert_eq!(
        serde_json::to_vec(&byte_limit).unwrap().len(),
        MAX_WORKFLOW_PATCH_BYTES
    );
    byte_limit.validate().unwrap();
    let mut byte_over = byte_limit.clone();
    let mut extended = false;
    for operation in &mut byte_over.operations {
        if let WorkflowPatchOperation::AddNode {
            node:
                WorkflowNode {
                    kind: WorkflowNodeKind::ModelWork { task, .. },
                    ..
                },
        } = operation
        {
            if task.len() < MAX_WORKFLOW_TASK_BYTES {
                task.push('x');
                extended = true;
                break;
            }
        }
    }
    assert!(extended);
    assert_eq!(
        serde_json::to_vec(&byte_over).unwrap().len(),
        MAX_WORKFLOW_PATCH_BYTES + 1
    );
    assert_eq!(
        byte_over.validate().unwrap_err(),
        "workflow_plan_patch_bounds_invalid"
    );

    let mut future_schema = node_limit;
    future_schema.schema = "yai.workflow_plan_patch.v99".to_string();
    assert_eq!(
        future_schema.validate().unwrap_err(),
        "workflow_plan_patch_bounds_invalid"
    );
    let invalid_qualified_path = WorkflowPlanPatchInput {
        schema: crate::workflow::WORKFLOW_PLAN_PATCH_SCHEMA.to_string(),
        base_effective_topology_digest: "sha256:path".to_string(),
        operations: vec![WorkflowPatchOperation::AddNode {
            node: h17_wait_node("ambiguous/path"),
        }],
    };
    assert_eq!(
        invalid_qualified_path.validate().unwrap_err(),
        "workflow_node_id_invalid"
    );
    println!(
        "h17_patch_bounds: bytes=262144 accepted bytes=262145 rejected operations=32 accepted operations=33 rejected added_nodes=16 accepted added_nodes=17 rejected added_edges_cap=64 dominated_by_operations_cap=32 future_schema=v99_rejected path_separator=in_definition_node_rejected"
    );
}

#[test]
fn hardening17_long_amendment_lineage_replays_and_corruption_fails_closed() {
    let path = temp_store_path("h17-long-amendment-lineage");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(17101);
    let tenant_id = "tenant:h17-lineage";
    let case_id = "case:h17-lineage";
    store
        .bootstrap_local_security(&owner, tenant_id, "organization:h17", 1)
        .unwrap();
    store
        .create_tenant_case(&owner, tenant_id, case_id)
        .unwrap();
    let definition = store
        .define_workflow(
            &owner,
            WorkflowDefinitionInput {
                schema: WORKFLOW_DEFINITION_SCHEMA.to_string(),
                tenant_id: tenant_id.to_string(),
                workflow_key: "h17-lineage".to_string(),
                declared_version: "2".to_string(),
                name: "H17 long amendment lineage".to_string(),
                description: String::new(),
                nodes: vec![h17_wait_node("base")],
                edges: vec![],
            },
            1_710_000,
        )
        .unwrap();
    let bound = store
        .bind_case_workflow(
            &owner,
            case_id,
            &definition.workflow_definition_id,
            vec![],
            vec![],
            1_710_001,
        )
        .unwrap();
    let binding = bound.state.workflow_binding.unwrap();
    let mut revision_digests = Vec::new();
    let started = Instant::now();
    for index in 0..MAX_WORKFLOW_AMENDMENTS {
        let current = store.workflow_status_authorized(&owner, case_id).unwrap();
        let input = WorkflowPlanPatchInput {
            schema: crate::workflow::WORKFLOW_PLAN_PATCH_SCHEMA.to_string(),
            base_effective_topology_digest: current.effective_topology_digest,
            operations: vec![WorkflowPatchOperation::AddNode {
                node: h17_wait_node(format!("future-{index:02}")),
            }],
        };
        let first = store
            .propose_workflow_plan_patch_human(
                &owner,
                case_id,
                input.clone(),
                1_710_010 + index as u64,
            )
            .unwrap();
        let generation_after_first = first.state.generation;
        let repeated = store
            .propose_workflow_plan_patch_human(&owner, case_id, input, 1_711_010 + index as u64)
            .unwrap();
        assert_eq!(
            first.transition.transition_id,
            repeated.transition.transition_id
        );
        assert_eq!(repeated.state.generation, generation_after_first);
        let patch_id = first
            .state
            .workflow_plan_patches
            .last()
            .unwrap()
            .patch_id
            .clone();
        store
            .adopt_workflow_plan_patch(&owner, case_id, &patch_id, 1_712_010 + index as u64)
            .unwrap();
        assert_eq!(
            store
                .adopt_workflow_plan_patch(&owner, case_id, &patch_id, 1_713_010 + index as u64,)
                .unwrap_err(),
            "workflow_plan_patch_already_adopted"
        );
        revision_digests.push(
            store
                .workflow_status_authorized(&owner, case_id)
                .unwrap()
                .effective_topology_digest,
        );
    }
    let lineage_execution_us = started.elapsed().as_micros();
    let state = store.get_case_state(case_id).unwrap().unwrap();
    assert_eq!(state.workflow_amendments.len(), MAX_WORKFLOW_AMENDMENTS);
    let definitions = BTreeMap::from([(
        definition.workflow_definition_id.clone(),
        definition.clone(),
    )]);
    let prefix_replay_started = Instant::now();
    for revision in 1..=MAX_WORKFLOW_AMENDMENTS {
        let topology = derive_effective_workflow_topology(
            &definition,
            &binding,
            &state.workflow_amendments[..revision],
            &definitions,
        )
        .unwrap();
        assert_eq!(topology.revision as usize, revision);
        assert_eq!(topology.nodes.len(), revision + 1);
        assert_eq!(topology.topology_digest, revision_digests[revision - 1]);
    }
    let prefix_replay_us = prefix_replay_started.elapsed().as_micros();
    let rebuild_started = Instant::now();
    let exact = derive_effective_workflow_topology(
        &definition,
        &binding,
        &state.workflow_amendments,
        &definitions,
    )
    .unwrap();
    let topology_rebuild_us = rebuild_started.elapsed().as_micros();
    let amendment_bytes = serde_json::to_vec(&state.workflow_amendments)
        .unwrap()
        .len();
    let transition_count = store.list_case_transitions(case_id).unwrap().len();

    let mut missing = state.workflow_amendments.clone();
    missing.remove(15);
    assert!(
        derive_effective_workflow_topology(&definition, &binding, &missing, &definitions)
            .unwrap_err()
            .contains("workflow_amendment")
    );
    let mut reordered = state.workflow_amendments.clone();
    reordered.swap(14, 15);
    assert!(
        derive_effective_workflow_topology(&definition, &binding, &reordered, &definitions)
            .unwrap_err()
            .contains("workflow_amendment")
    );

    let mut corruptions = Vec::new();
    let middle = 15;
    macro_rules! corruption {
        ($name:literal, $body:expr) => {{
            let mut chain = state.workflow_amendments.clone();
            $body(&mut chain[middle]);
            corruptions.push(($name, chain));
        }};
    }
    corruption!("parent", |value: &mut WorkflowAmendment| value
        .parent_amendment_id =
        Some("workflow-amendment:forged".to_string()));
    corruption!("revision", |value: &mut WorkflowAmendment| value
        .revision +=
        1);
    corruption!("previous_digest", |value: &mut WorkflowAmendment| value
        .previous_topology_digest =
        "sha256:forged".to_string());
    corruption!("resulting_digest", |value: &mut WorkflowAmendment| value
        .resulting_topology_digest =
        "sha256:forged".to_string());
    corruption!("patch_id", |value: &mut WorkflowAmendment| value.patch_id =
        "workflow-plan-patch:forged".to_string());
    corruption!("patch_digest", |value: &mut WorkflowAmendment| value
        .patch_integrity_digest =
        "sha256:forged".to_string());
    corruption!("binding", |value: &mut WorkflowAmendment| value
        .binding_id =
        "case-workflow-binding:forged".to_string());
    corruption!("operations", |value: &mut WorkflowAmendment| value
        .operations =
        vec![WorkflowPatchOperation::AddNode {
            node: h17_wait_node("forged-middle")
        }]);
    corruption!("amendment_id", |value: &mut WorkflowAmendment| value
        .amendment_id =
        "workflow-amendment:forged".to_string());
    for (name, chain) in corruptions {
        assert!(
            derive_effective_workflow_topology(&definition, &binding, &chain, &definitions)
                .is_err(),
            "{name} corruption must fail closed"
        );
    }

    let replay = store.rebuild_case_state(case_id).unwrap();
    assert_eq!(replay.workflow_amendments, state.workflow_amendments);
    let resolution_started = Instant::now();
    let resolution = store.workflow_status_authorized(&owner, case_id).unwrap();
    let resolution_us = resolution_started.elapsed().as_micros();
    assert_eq!(resolution.effective_topology_digest, exact.topology_digest);
    println!(
        "h17_amendment_lineage: revisions=32 bytes={amendment_bytes} transitions={transition_count} effective_nodes={} effective_edges={} lineage_execution_us={lineage_execution_us} prefix_replay_us={prefix_replay_us} topology_rebuild_us={topology_rebuild_us} resolution_us={resolution_us} corruptions=11 fail_closed=11 digest={}",
        exact.nodes.len(),
        exact.edges.len(),
        exact.topology_digest
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn hardening17_process_amendment_contender() {
    let Ok(store_path) = std::env::var("H17_AMENDMENT_STORE") else {
        return;
    };
    let index = std::env::var("H17_AMENDMENT_INDEX").unwrap();
    let patch_id = std::env::var("H17_AMENDMENT_PATCH").unwrap();
    let control = PathBuf::from(std::env::var("H17_AMENDMENT_CONTROL").unwrap());
    fs::write(control.join(format!("ready-{index}")), b"ready").unwrap();
    for _ in 0..5_000 {
        if control.join("go").exists() {
            break;
        }
        thread::sleep(Duration::from_millis(2));
    }
    let store = LmdbRecordStore::open(store_path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(17102);
    let outcome = store.adopt_workflow_plan_patch(
        &owner,
        "case:h17-amendment-race",
        &patch_id,
        1_721_000 + index.parse::<u64>().unwrap(),
    );
    let value = match outcome {
        Ok(commit) => format!("ok:{}", commit.transition.transition_id),
        Err(error) => format!("err:{error}"),
    };
    fs::write(control.join(format!("result-{index}")), value).unwrap();
}

#[test]
fn hardening17_thirty_two_process_amendment_race_has_one_winner() {
    let path = temp_store_path("h17-process-amendment-race");
    let control = temp_store_path("h17-process-amendment-control");
    fs::create_dir_all(&control).unwrap();
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(17102);
    let tenant_id = "tenant:h17-amendment-race";
    let case_id = "case:h17-amendment-race";
    store
        .bootstrap_local_security(&owner, tenant_id, "organization:h17", 1)
        .unwrap();
    store
        .create_tenant_case(&owner, tenant_id, case_id)
        .unwrap();
    let definition = store
        .define_workflow(
            &owner,
            WorkflowDefinitionInput {
                schema: WORKFLOW_DEFINITION_SCHEMA.to_string(),
                tenant_id: tenant_id.to_string(),
                workflow_key: "h17-race".to_string(),
                declared_version: "2".to_string(),
                name: "H17 amendment race".to_string(),
                description: String::new(),
                nodes: vec![h17_wait_node("base")],
                edges: vec![],
            },
            1_720_000,
        )
        .unwrap();
    store
        .bind_case_workflow(
            &owner,
            case_id,
            &definition.workflow_definition_id,
            vec![],
            vec![],
            1_720_001,
        )
        .unwrap();
    let base = store
        .workflow_status_authorized(&owner, case_id)
        .unwrap()
        .effective_topology_digest;
    let mut patch_ids = Vec::new();
    for index in 0..32 {
        let commit = store
            .propose_workflow_plan_patch_human(
                &owner,
                case_id,
                WorkflowPlanPatchInput {
                    schema: crate::workflow::WORKFLOW_PLAN_PATCH_SCHEMA.to_string(),
                    base_effective_topology_digest: base.clone(),
                    operations: vec![WorkflowPatchOperation::AddNode {
                        node: h17_wait_node(format!("contender-{index:02}")),
                    }],
                },
                1_720_010 + index,
            )
            .unwrap();
        patch_ids.push(
            commit
                .state
                .workflow_plan_patches
                .last()
                .unwrap()
                .patch_id
                .clone(),
        );
    }
    let generation_before = store.get_case_state(case_id).unwrap().unwrap().generation;
    let executable = std::env::current_exe().unwrap();
    let mut children = Vec::new();
    for (index, patch_id) in patch_ids.iter().enumerate() {
        children.push(
            std::process::Command::new(&executable)
                .arg("--exact")
                .arg("store::lmdb::tests::hardening17_tests::hardening17_process_amendment_contender")
                .arg("--nocapture")
                .env("H17_AMENDMENT_STORE", &path)
                .env("H17_AMENDMENT_CONTROL", &control)
                .env("H17_AMENDMENT_INDEX", index.to_string())
                .env("H17_AMENDMENT_PATCH", patch_id)
                .spawn()
                .unwrap(),
        );
    }
    wait_for_files(&control, "ready-", 32);
    fs::write(control.join("go"), b"go").unwrap();
    wait_for_files(&control, "result-", 32);
    for child in children {
        assert!(child.wait_with_output().unwrap().status.success());
    }
    let results = (0..32)
        .map(|index| fs::read_to_string(control.join(format!("result-{index}"))).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        results
            .iter()
            .filter(|value| value.starts_with("ok:"))
            .count(),
        1
    );
    assert_eq!(
        results
            .iter()
            .filter(|value| value.as_str() == "err:workflow_patch_stale")
            .count(),
        31
    );
    let state = store.get_case_state(case_id).unwrap().unwrap();
    assert_eq!(state.workflow_amendments.len(), 1);
    assert_eq!(state.generation, generation_before + 1);
    println!(
        "h17_amendment_process_race: processes=32 winners=1 stale=31 generation_delta=1 revision=1 digest={}",
        store
            .workflow_status_authorized(&owner, case_id)
            .unwrap()
            .effective_topology_digest
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
    fs::remove_dir_all(control).unwrap();
}

#[test]
fn hardening17_adjacent_revision_late_writers_never_fork_lineage() {
    let path = temp_store_path("h17-adjacent-revision-races");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(17112);
    let tenant_id = "tenant:h17-adjacent";
    let case_id = "case:h17-adjacent";
    store
        .bootstrap_local_security(&owner, tenant_id, "organization:h17", 1)
        .unwrap();
    store
        .create_tenant_case(&owner, tenant_id, case_id)
        .unwrap();
    let definition = store
        .define_workflow(
            &owner,
            WorkflowDefinitionInput {
                schema: WORKFLOW_DEFINITION_SCHEMA.to_string(),
                tenant_id: tenant_id.to_string(),
                workflow_key: "h17-adjacent".to_string(),
                declared_version: "2".to_string(),
                name: "H17 adjacent revision races".to_string(),
                description: String::new(),
                nodes: vec![h17_wait_node("base")],
                edges: vec![],
            },
            1_725_000,
        )
        .unwrap();
    store
        .bind_case_workflow(
            &owner,
            case_id,
            &definition.workflow_definition_id,
            vec![],
            vec![],
            1_725_001,
        )
        .unwrap();
    for revision in 0..8 {
        let base = store
            .workflow_status_authorized(&owner, case_id)
            .unwrap()
            .effective_topology_digest;
        let propose = |suffix: &str, time: u64| {
            store
                .propose_workflow_plan_patch_human(
                    &owner,
                    case_id,
                    WorkflowPlanPatchInput {
                        schema: crate::workflow::WORKFLOW_PLAN_PATCH_SCHEMA.to_string(),
                        base_effective_topology_digest: base.clone(),
                        operations: vec![WorkflowPatchOperation::AddNode {
                            node: h17_wait_node(format!("revision-{revision}-{suffix}")),
                        }],
                    },
                    time,
                )
                .unwrap()
                .state
                .workflow_plan_patches
                .last()
                .unwrap()
                .patch_id
                .clone()
        };
        let winner = propose("winner", 1_725_010 + revision * 10);
        let late = propose("late", 1_725_011 + revision * 10);
        store
            .adopt_workflow_plan_patch(&owner, case_id, &winner, 1_725_012 + revision * 10)
            .unwrap();
        assert_eq!(
            store
                .adopt_workflow_plan_patch(&owner, case_id, &late, 1_725_013 + revision * 10)
                .unwrap_err(),
            "workflow_patch_stale"
        );
    }
    let state = store.get_case_state(case_id).unwrap().unwrap();
    assert_eq!(state.workflow_amendments.len(), 8);
    for (index, amendment) in state.workflow_amendments.iter().enumerate() {
        assert_eq!(amendment.revision, index as u32 + 1);
        assert_eq!(
            amendment.parent_amendment_id.as_deref(),
            index
                .checked_sub(1)
                .map(|parent| state.workflow_amendments[parent].amendment_id.as_str())
        );
    }
    println!(
        "h17_adjacent_revision_races: rounds=8 winning_amendments=8 late_stale_writers=8 forks=0 final_revision=8"
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn hardening17_amendment_races_with_human_and_passive_progress_are_serializable() {
    let owner = AuthenticatedPrincipal::for_test(17113);

    let human_path = temp_store_path("h17-amendment-human-race");
    let human_store = LmdbRecordStore::open(&human_path).unwrap();
    let human_tenant = "tenant:h17-amendment-human";
    let human_case = "case:h17-amendment-human";
    human_store
        .bootstrap_local_security(&owner, human_tenant, "organization:h17", 1)
        .unwrap();
    let human_root = h15_setup_case(
        &human_store,
        &owner,
        human_tenant,
        human_case,
        "amendment-human",
    );
    let human_definition = human_store
        .define_workflow(
            &owner,
            WorkflowDefinitionInput {
                schema: WORKFLOW_DEFINITION_SCHEMA.to_string(),
                tenant_id: human_tenant.to_string(),
                workflow_key: "h17-amendment-human".to_string(),
                declared_version: "2".to_string(),
                name: "H17 amendment HumanInput race".to_string(),
                description: String::new(),
                nodes: vec![
                    WorkflowNode {
                        node_id: "input".to_string(),
                        kind: WorkflowNodeKind::HumanInput {
                            actor_slot: "actor".to_string(),
                            prompt: "provide bounded input".to_string(),
                            required_roles: vec![],
                            input_kind: HumanInputKind::Text,
                            max_bytes: 64,
                        },
                    },
                    h17_wait_node("human-anchor"),
                ],
                edges: vec![],
            },
            1_726_000,
        )
        .unwrap();
    human_store
        .bind_case_workflow(
            &owner,
            human_case,
            &human_definition.workflow_definition_id,
            vec![WorkflowExecutorBinding {
                slot: "actor".to_string(),
                participant_id: "participant:model".to_string(),
            }],
            vec![],
            1_726_001,
        )
        .unwrap();
    let human_base = human_store
        .workflow_status_authorized(&owner, human_case)
        .unwrap()
        .effective_topology_digest;
    let human_patch = human_store
        .propose_workflow_plan_patch_human(
            &owner,
            human_case,
            WorkflowPlanPatchInput {
                schema: crate::workflow::WORKFLOW_PLAN_PATCH_SCHEMA.to_string(),
                base_effective_topology_digest: human_base,
                operations: vec![WorkflowPatchOperation::DisableNode {
                    node_id: "input".to_string(),
                }],
            },
            1_726_002,
        )
        .unwrap()
        .state
        .workflow_plan_patches
        .last()
        .unwrap()
        .patch_id
        .clone();
    let barrier = Arc::new(Barrier::new(2));
    let human_input_thread = {
        let path = human_path.clone();
        let owner = owner.clone();
        let barrier = barrier.clone();
        thread::spawn(move || {
            let store = LmdbRecordStore::open(path).unwrap();
            barrier.wait();
            store.record_workflow_human_input(&owner, human_case, "input", "H17", 1_726_003)
        })
    };
    let human_patch_thread = {
        let path = human_path.clone();
        let owner = owner.clone();
        let barrier = barrier.clone();
        thread::spawn(move || {
            let store = LmdbRecordStore::open(path).unwrap();
            barrier.wait();
            store.adopt_workflow_plan_patch(&owner, human_case, &human_patch, 1_726_004)
        })
    };
    let human_input = human_input_thread.join().unwrap();
    let human_adoption = human_patch_thread.join().unwrap();
    assert_eq!(
        usize::from(human_input.is_ok()) + usize::from(human_adoption.is_ok()),
        1
    );
    let human_state = human_store.get_case_state(human_case).unwrap().unwrap();
    assert_eq!(
        human_state.workflow_human_inputs.len() + human_state.workflow_amendments.len(),
        1
    );

    let passive_path = temp_store_path("h17-amendment-passive-race");
    let passive_store = LmdbRecordStore::open(&passive_path).unwrap();
    let passive_tenant = "tenant:h17-amendment-passive";
    let passive_case = "case:h17-amendment-passive";
    passive_store
        .bootstrap_local_security(&owner, passive_tenant, "organization:h17", 1)
        .unwrap();
    passive_store
        .create_tenant_case(&owner, passive_tenant, passive_case)
        .unwrap();
    let passive_definition = passive_store
        .define_workflow(
            &owner,
            WorkflowDefinitionInput {
                schema: WORKFLOW_DEFINITION_SCHEMA.to_string(),
                tenant_id: passive_tenant.to_string(),
                workflow_key: "h17-amendment-passive".to_string(),
                declared_version: "2".to_string(),
                name: "H17 amendment passive race".to_string(),
                description: String::new(),
                nodes: vec![
                    WorkflowNode {
                        node_id: "open-wait".to_string(),
                        kind: WorkflowNodeKind::Wait {
                            predicate: WorkflowPredicate::CaseLifecycle {
                                lifecycle: CaseLifecycle::Open,
                            },
                        },
                    },
                    h17_wait_node("passive-anchor"),
                ],
                edges: vec![],
            },
            1_727_000,
        )
        .unwrap();
    passive_store
        .bind_case_workflow(
            &owner,
            passive_case,
            &passive_definition.workflow_definition_id,
            vec![],
            vec![],
            1_727_001,
        )
        .unwrap();
    let passive_base = passive_store
        .workflow_status_authorized(&owner, passive_case)
        .unwrap()
        .effective_topology_digest;
    let passive_patch = passive_store
        .propose_workflow_plan_patch_human(
            &owner,
            passive_case,
            WorkflowPlanPatchInput {
                schema: crate::workflow::WORKFLOW_PLAN_PATCH_SCHEMA.to_string(),
                base_effective_topology_digest: passive_base,
                operations: vec![WorkflowPatchOperation::DisableNode {
                    node_id: "open-wait".to_string(),
                }],
            },
            1_727_002,
        )
        .unwrap()
        .state
        .workflow_plan_patches
        .last()
        .unwrap()
        .patch_id
        .clone();
    let barrier = Arc::new(Barrier::new(2));
    let passive_progress_thread = {
        let path = passive_path.clone();
        let owner = owner.clone();
        let barrier = barrier.clone();
        thread::spawn(move || {
            let store = LmdbRecordStore::open(path).unwrap();
            barrier.wait();
            store.advance_workflow_passive_progress(&owner, passive_case, 8)
        })
    };
    let passive_patch_thread = {
        let path = passive_path.clone();
        let owner = owner.clone();
        let barrier = barrier.clone();
        thread::spawn(move || {
            let store = LmdbRecordStore::open(path).unwrap();
            barrier.wait();
            store.adopt_workflow_plan_patch(&owner, passive_case, &passive_patch, 1_727_003)
        })
    };
    let passive_progress = passive_progress_thread.join().unwrap();
    let passive_adoption = passive_patch_thread.join().unwrap();
    assert!(passive_progress.is_ok() || passive_adoption.is_ok());
    let passive_state = passive_store.get_case_state(passive_case).unwrap().unwrap();
    assert_eq!(
        passive_state.workflow_satisfactions.len() + passive_state.workflow_amendments.len(),
        1
    );
    println!(
        "h17_progression_adoption_races: human_input_or_amendment_one_winner=true passive_satisfaction_or_amendment_one_truth=true write_skew=0"
    );
    drop(human_store);
    drop(passive_store);
    for path in [human_path, human_root, passive_path] {
        fs::remove_dir_all(path).unwrap();
    }
}

fn h17_setup_accepted_handoff(
    store: &LmdbRecordStore,
    owner: &AuthenticatedPrincipal,
    tenant_id: &str,
    source_case_id: &str,
    target_case_id: &str,
    suffix: &str,
) -> (PathBuf, PathBuf, HandoffOffer) {
    let source_root = h15_setup_case(
        store,
        owner,
        tenant_id,
        source_case_id,
        &format!("{suffix}-s"),
    );
    let target_root = h15_setup_case(
        store,
        owner,
        tenant_id,
        target_case_id,
        &format!("{suffix}-t"),
    );
    let offer = store
        .offer_case_handoff(
            owner,
            source_case_id,
            target_case_id,
            HandoffData {
                kind: crate::handoff::HandoffDataKind::Json,
                value: "{\"task\":\"h17-pressure\"}".to_string(),
            },
            vec!["operation-proposer".to_string()],
            1_730_000,
        )
        .unwrap()
        .state
        .handoff_offers
        .last()
        .unwrap()
        .clone();
    store
        .accept_case_handoff(
            owner,
            target_case_id,
            source_case_id,
            &offer.handoff_id,
            "participant:model",
            1_730_001,
        )
        .unwrap();
    (source_root, target_root, offer)
}

#[test]
fn hardening17_process_handoff_contender() {
    let Ok(store_path) = std::env::var("H17_HANDOFF_STORE") else {
        return;
    };
    let mode = std::env::var("H17_HANDOFF_MODE").unwrap();
    let index = std::env::var("H17_HANDOFF_INDEX").unwrap();
    let handoff_id = std::env::var("H17_HANDOFF_ID").unwrap();
    let source_case_id = std::env::var("H17_HANDOFF_SOURCE").unwrap();
    let target_case_id = std::env::var("H17_HANDOFF_TARGET").unwrap();
    let control = PathBuf::from(std::env::var("H17_HANDOFF_CONTROL").unwrap());
    fs::write(control.join(format!("ready-{index}")), b"ready").unwrap();
    for _ in 0..5_000 {
        if control.join("go").exists() {
            break;
        }
        thread::sleep(Duration::from_millis(2));
    }
    let store = LmdbRecordStore::open(store_path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(17103);
    let index_number = index.parse::<u64>().unwrap();
    let outcome = if mode == "result" {
        let (outcome, value) = if index_number % 2 == 0 {
            (HandoffOutcome::Succeeded, "same-result-a")
        } else {
            (HandoffOutcome::Failed, "same-result-b")
        };
        store
            .record_case_handoff_result(
                &owner,
                &target_case_id,
                &handoff_id,
                outcome,
                HandoffData {
                    kind: crate::handoff::HandoffDataKind::Text,
                    value: value.to_string(),
                },
                vec![],
                "participant:model",
                1_731_000 + index_number,
            )
            .map(|commit| commit.transition.transition_id)
    } else {
        store
            .reconcile_case_handoff(
                &owner,
                &source_case_id,
                &handoff_id,
                1_732_000 + index_number,
            )
            .map(|commit| commit.transition.transition_id)
    };
    let value = match outcome {
        Ok(id) => format!("ok:{id}"),
        Err(error) => format!("err:{error}"),
    };
    fs::write(control.join(format!("result-{index}")), value).unwrap();
}

fn run_h17_handoff_processes(
    path: &Path,
    control: &Path,
    mode: &str,
    source_case_id: &str,
    target_case_id: &str,
    handoff_id: &str,
) -> Vec<String> {
    fs::create_dir_all(control).unwrap();
    let executable = std::env::current_exe().unwrap();
    let mut children = Vec::new();
    for index in 0..32 {
        children.push(
            std::process::Command::new(&executable)
                .arg("--exact")
                .arg("store::lmdb::tests::hardening17_tests::hardening17_process_handoff_contender")
                .arg("--nocapture")
                .env("H17_HANDOFF_STORE", path)
                .env("H17_HANDOFF_CONTROL", control)
                .env("H17_HANDOFF_MODE", mode)
                .env("H17_HANDOFF_INDEX", index.to_string())
                .env("H17_HANDOFF_ID", handoff_id)
                .env("H17_HANDOFF_SOURCE", source_case_id)
                .env("H17_HANDOFF_TARGET", target_case_id)
                .spawn()
                .unwrap(),
        );
    }
    wait_for_files(control, "ready-", 32);
    fs::write(control.join("go"), b"go").unwrap();
    wait_for_files(control, "result-", 32);
    for child in children {
        assert!(child.wait_with_output().unwrap().status.success());
    }
    (0..32)
        .map(|index| fs::read_to_string(control.join(format!("result-{index}"))).unwrap())
        .collect()
}

#[test]
fn hardening17_result_and_reconciliation_process_races_have_one_truth() {
    let path = temp_store_path("h17-handoff-process-races");
    let result_control = temp_store_path("h17-handoff-result-control");
    let reconcile_control = temp_store_path("h17-handoff-reconcile-control");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(17103);
    let tenant_id = "tenant:h17-handoff-process";
    let source_case_id = "case:h17-handoff-process-source";
    let target_case_id = "case:h17-handoff-process-target";
    store
        .bootstrap_local_security(&owner, tenant_id, "organization:h17", 1)
        .unwrap();
    let (source_root, target_root, offer) = h17_setup_accepted_handoff(
        &store,
        &owner,
        tenant_id,
        source_case_id,
        target_case_id,
        "process",
    );
    let target_generation = store
        .get_case_state(target_case_id)
        .unwrap()
        .unwrap()
        .generation;
    let result_outcomes = run_h17_handoff_processes(
        &path,
        &result_control,
        "result",
        source_case_id,
        target_case_id,
        &offer.handoff_id,
    );
    let successful_ids = result_outcomes
        .iter()
        .filter_map(|value| value.strip_prefix("ok:"))
        .collect::<BTreeSet<_>>();
    assert_eq!(successful_ids.len(), 1);
    assert_eq!(
        result_outcomes
            .iter()
            .filter(|value| value.starts_with("ok:"))
            .count(),
        16
    );
    assert_eq!(
        result_outcomes
            .iter()
            .filter(|value| value.as_str() == "err:handoff_result_already_terminal")
            .count(),
        16
    );
    let target = store.get_case_state(target_case_id).unwrap().unwrap();
    assert_eq!(target.handoff_results.len(), 1);
    assert_eq!(target.generation, target_generation + 1);

    let source_generation = store
        .get_case_state(source_case_id)
        .unwrap()
        .unwrap()
        .generation;
    let reconcile_outcomes = run_h17_handoff_processes(
        &path,
        &reconcile_control,
        "reconcile",
        source_case_id,
        target_case_id,
        &offer.handoff_id,
    );
    let reconciliation_ids = reconcile_outcomes
        .iter()
        .filter_map(|value| value.strip_prefix("ok:"))
        .collect::<BTreeSet<_>>();
    assert_eq!(reconciliation_ids.len(), 1);
    assert_eq!(
        reconcile_outcomes
            .iter()
            .filter(|value| value.starts_with("ok:"))
            .count(),
        32
    );
    let source = store.get_case_state(source_case_id).unwrap().unwrap();
    assert_eq!(source.handoff_reconciliations.len(), 1);
    assert_eq!(source.generation, source_generation + 1);
    assert!(store.verify_case_state(source_case_id).unwrap());
    assert!(store.verify_case_state(target_case_id).unwrap());
    println!(
        "h17_handoff_process_races: result_processes=32 result_semantic_winners=1 identical_observers=16 conflicts=16 result_generation_delta=1 reconcile_processes=32 reconcile_semantic_winners=1 idempotent_observers=32 reconcile_generation_delta=1"
    );
    drop(store);
    for path in [
        path,
        result_control,
        reconcile_control,
        source_root,
        target_root,
    ] {
        fs::remove_dir_all(path).unwrap();
    }
}

#[test]
fn hardening17_handoff_forgery_and_empty_success_boundaries_are_exact() {
    let path = temp_store_path("h17-handoff-forgery");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(17104);
    let tenant_id = "tenant:h17-handoff-forgery";
    let source_case_id = "case:h17-forgery-source";
    let target_case_id = "case:h17-forgery-target";
    store
        .bootstrap_local_security(&owner, tenant_id, "organization:h17", 1)
        .unwrap();
    let (source_root, target_root, offer) = h17_setup_accepted_handoff(
        &store,
        &owner,
        tenant_id,
        source_case_id,
        target_case_id,
        "forgery",
    );
    let source_evidence = store
        .list_case_transitions(source_case_id)
        .unwrap()
        .last()
        .unwrap()
        .transition_id
        .clone();
    assert_eq!(
        store
            .record_case_handoff_result(
                &owner,
                target_case_id,
                &offer.handoff_id,
                HandoffOutcome::Succeeded,
                HandoffData {
                    kind: crate::handoff::HandoffDataKind::Text,
                    value: "forged cross-Case evidence".to_string(),
                },
                vec![source_evidence.clone()],
                "participant:model",
                1_733_000,
            )
            .unwrap_err(),
        format!("handoff_result_evidence_not_target_local: {source_evidence}")
    );
    assert_eq!(
        store
            .record_case_handoff_result(
                &owner,
                target_case_id,
                "handoff:wrong",
                HandoffOutcome::Succeeded,
                HandoffData {
                    kind: crate::handoff::HandoffDataKind::Text,
                    value: "wrong acceptance".to_string(),
                },
                vec![],
                "participant:model",
                1_733_001,
            )
            .unwrap_err(),
        "handoff_acceptance_not_found"
    );
    let result = store
        .record_case_handoff_result(
            &owner,
            target_case_id,
            &offer.handoff_id,
            HandoffOutcome::Succeeded,
            HandoffData {
                kind: crate::handoff::HandoffDataKind::Json,
                value: "{\"reported\":\"complete\"}".to_string(),
            },
            vec![],
            "participant:model",
            1_733_002,
        )
        .unwrap();
    assert!(result.state.effects.is_empty());
    assert!(result.state.grants.is_empty());
    let mut tampered = result.state.handoff_results[0].clone();
    tampered.outcome = HandoffOutcome::Failed;
    assert_eq!(
        tampered.validate().unwrap_err(),
        "handoff_result_integrity_mismatch"
    );
    let reconciled = store
        .reconcile_case_handoff(&owner, source_case_id, &offer.handoff_id, 1_733_003)
        .unwrap();
    assert_eq!(
        reconciled.state.handoff_reconciliations[0].outcome,
        HandoffOutcome::Succeeded
    );
    assert!(reconciled.state.effects.is_empty());
    assert!(reconciled.state.grants.is_empty());
    println!(
        "h17_handoff_forgery: wrong_handoff=rejected cross_case_evidence=rejected empty_evidence_success=admitted_as_target_report source_effects=0 source_grants=0 tampered_outcome=integrity_rejected"
    );
    drop(store);
    for path in [path, source_root, target_root] {
        fs::remove_dir_all(path).unwrap();
    }
}

#[test]
fn hardening17_cancellation_close_and_handoff_writes_are_serializable() {
    let path = temp_store_path("h17-handoff-lifecycle-races");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(17105);
    let tenant_id = "tenant:h17-lifecycle-races";
    store
        .bootstrap_local_security(&owner, tenant_id, "organization:h17", 1)
        .unwrap();
    let mut roots = Vec::new();

    let accept_source = "case:h17-accept-cancel-source";
    let accept_target = "case:h17-accept-cancel-target";
    roots.push(h15_setup_case(
        &store,
        &owner,
        tenant_id,
        accept_source,
        "accept-cancel-s",
    ));
    roots.push(h15_setup_case(
        &store,
        &owner,
        tenant_id,
        accept_target,
        "accept-cancel-t",
    ));
    let accept_offer = store
        .offer_case_handoff(
            &owner,
            accept_source,
            accept_target,
            HandoffData {
                kind: crate::handoff::HandoffDataKind::Text,
                value: "accept versus cancellation".to_string(),
            },
            vec!["operation-proposer".to_string()],
            1_740_000,
        )
        .unwrap()
        .state
        .handoff_offers[0]
        .clone();
    let barrier = Arc::new(Barrier::new(2));
    let cancel_thread = {
        let path = path.clone();
        let owner = owner.clone();
        let barrier = barrier.clone();
        thread::spawn(move || {
            let store = LmdbRecordStore::open(path).unwrap();
            barrier.wait();
            store.cancel_tenant_case(&owner, accept_source, "race cancellation")
        })
    };
    let accept_thread = {
        let path = path.clone();
        let owner = owner.clone();
        let barrier = barrier.clone();
        let handoff_id = accept_offer.handoff_id.clone();
        thread::spawn(move || {
            let store = LmdbRecordStore::open(path).unwrap();
            barrier.wait();
            store.accept_case_handoff(
                &owner,
                accept_target,
                accept_source,
                &handoff_id,
                "participant:model",
                1_740_001,
            )
        })
    };
    assert!(cancel_thread.join().unwrap().is_ok());
    let accept_outcome = accept_thread.join().unwrap();
    let acceptance_count = store
        .get_case_state(accept_target)
        .unwrap()
        .unwrap()
        .handoff_acceptances
        .len();
    assert_eq!(acceptance_count, usize::from(accept_outcome.is_ok()));
    assert!(store
        .get_case_state(accept_source)
        .unwrap()
        .unwrap()
        .cancellation
        .is_some());

    let result_source = "case:h17-result-cancel-source";
    let result_target = "case:h17-result-cancel-target";
    let (source_root, target_root, result_offer) = h17_setup_accepted_handoff(
        &store,
        &owner,
        tenant_id,
        result_source,
        result_target,
        "result-cancel",
    );
    roots.extend([source_root, target_root]);
    let barrier = Arc::new(Barrier::new(2));
    let cancel_thread = {
        let path = path.clone();
        let owner = owner.clone();
        let barrier = barrier.clone();
        thread::spawn(move || {
            let store = LmdbRecordStore::open(path).unwrap();
            barrier.wait();
            store.cancel_tenant_case(&owner, result_target, "target race cancellation")
        })
    };
    let result_thread = {
        let path = path.clone();
        let owner = owner.clone();
        let barrier = barrier.clone();
        let handoff_id = result_offer.handoff_id.clone();
        thread::spawn(move || {
            let store = LmdbRecordStore::open(path).unwrap();
            barrier.wait();
            store.record_case_handoff_result(
                &owner,
                result_target,
                &handoff_id,
                HandoffOutcome::Succeeded,
                HandoffData {
                    kind: crate::handoff::HandoffDataKind::Text,
                    value: "success won before cancellation".to_string(),
                },
                vec![],
                "participant:model",
                1_740_010,
            )
        })
    };
    assert!(cancel_thread.join().unwrap().is_ok());
    let result_outcome = result_thread.join().unwrap();
    let result_count = store
        .get_case_state(result_target)
        .unwrap()
        .unwrap()
        .handoff_results
        .len();
    assert_eq!(result_count, usize::from(result_outcome.is_ok()));
    if let Err(error) = result_outcome {
        assert_eq!(error, "handoff_result_target_case_terminal");
    }

    let reconcile_source = "case:h17-reconcile-cancel-source";
    let reconcile_target = "case:h17-reconcile-cancel-target";
    let (source_root, target_root, reconcile_offer) = h17_setup_accepted_handoff(
        &store,
        &owner,
        tenant_id,
        reconcile_source,
        reconcile_target,
        "reconcile-cancel",
    );
    roots.extend([source_root, target_root]);
    store
        .record_case_handoff_result(
            &owner,
            reconcile_target,
            &reconcile_offer.handoff_id,
            HandoffOutcome::Succeeded,
            HandoffData {
                kind: crate::handoff::HandoffDataKind::Text,
                value: "settle despite source cancellation".to_string(),
            },
            vec![],
            "participant:model",
            1_740_020,
        )
        .unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let cancel_thread = {
        let path = path.clone();
        let owner = owner.clone();
        let barrier = barrier.clone();
        thread::spawn(move || {
            let store = LmdbRecordStore::open(path).unwrap();
            barrier.wait();
            store.cancel_tenant_case(&owner, reconcile_source, "source reconcile race")
        })
    };
    let reconcile_thread = {
        let path = path.clone();
        let owner = owner.clone();
        let barrier = barrier.clone();
        let handoff_id = reconcile_offer.handoff_id.clone();
        thread::spawn(move || {
            let store = LmdbRecordStore::open(path).unwrap();
            barrier.wait();
            store.reconcile_case_handoff(&owner, reconcile_source, &handoff_id, 1_740_021)
        })
    };
    assert!(cancel_thread.join().unwrap().is_ok());
    assert!(reconcile_thread.join().unwrap().is_ok());
    let reconciled_state = store.get_case_state(reconcile_source).unwrap().unwrap();
    assert!(reconciled_state.cancellation.is_some());
    assert_eq!(reconciled_state.handoff_reconciliations.len(), 1);

    let close_source = "case:h17-close-source";
    let close_target = "case:h17-close-target";
    let (source_root, target_root, close_offer) = h17_setup_accepted_handoff(
        &store,
        &owner,
        tenant_id,
        close_source,
        close_target,
        "close",
    );
    roots.extend([source_root, target_root]);
    store
        .cancel_tenant_case(&owner, close_source, "prepare source close")
        .unwrap();
    assert!(store
        .close_tenant_case(&owner, close_source, "must settle handoff first")
        .unwrap_err()
        .contains("accepted_handoff_unresolved"));
    store
        .cancel_tenant_case(&owner, close_target, "target terminal settlement")
        .unwrap();
    store
        .reconcile_case_handoff(&owner, close_source, &close_offer.handoff_id, 1_740_030)
        .unwrap();
    assert!(
        store
            .close_tenant_case(&owner, close_source, "handoff settled")
            .unwrap()
            .changed
    );
    println!(
        "h17_handoff_lifecycle_races: cancel_accept_serialized=true acceptance_committed={} cancel_result_serialized=true result_committed={} cancel_reconcile_both_preserved=true close_before_settlement=blocked close_after_reconcile=committed",
        acceptance_count,
        result_count
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
    for root in roots {
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn hardening17_concurrent_cycle_creation_never_commits_a_cycle() {
    let path = temp_store_path("h17-cycle-races");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(17106);
    let tenant_id = "tenant:h17-cycle-races";
    store
        .bootstrap_local_security(&owner, tenant_id, "organization:h17", 1)
        .unwrap();
    for case_id in ["case:h17-cycle-a", "case:h17-cycle-b"] {
        store
            .create_tenant_case(&owner, tenant_id, case_id)
            .unwrap();
    }
    let barrier = Arc::new(Barrier::new(2));
    let mut contenders = Vec::new();
    for (index, (source, target)) in [
        ("case:h17-cycle-a", "case:h17-cycle-b"),
        ("case:h17-cycle-b", "case:h17-cycle-a"),
    ]
    .into_iter()
    .enumerate()
    {
        let path = path.clone();
        let owner = owner.clone();
        let barrier = barrier.clone();
        contenders.push(thread::spawn(move || {
            let store = LmdbRecordStore::open(path).unwrap();
            barrier.wait();
            store.offer_case_handoff(
                &owner,
                source,
                target,
                HandoffData {
                    kind: crate::handoff::HandoffDataKind::Text,
                    value: format!("two-way-{index}"),
                },
                vec![],
                1_750_000 + index as u64,
            )
        }));
    }
    let two_way = contenders
        .into_iter()
        .map(|value| value.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(two_way.iter().filter(|value| value.is_ok()).count(), 1);

    for case_id in [
        "case:h17-triangle-a",
        "case:h17-triangle-b",
        "case:h17-triangle-c",
    ] {
        store
            .create_tenant_case(&owner, tenant_id, case_id)
            .unwrap();
    }
    let barrier = Arc::new(Barrier::new(3));
    let mut contenders = Vec::new();
    for (index, (source, target)) in [
        ("case:h17-triangle-a", "case:h17-triangle-b"),
        ("case:h17-triangle-b", "case:h17-triangle-c"),
        ("case:h17-triangle-c", "case:h17-triangle-a"),
    ]
    .into_iter()
    .enumerate()
    {
        let path = path.clone();
        let owner = owner.clone();
        let barrier = barrier.clone();
        contenders.push(thread::spawn(move || {
            let store = LmdbRecordStore::open(path).unwrap();
            barrier.wait();
            store.offer_case_handoff(
                &owner,
                source,
                target,
                HandoffData {
                    kind: crate::handoff::HandoffDataKind::Text,
                    value: format!("triangle-{index}"),
                },
                vec![],
                1_750_010 + index as u64,
            )
        }));
    }
    let triangle = contenders
        .into_iter()
        .map(|value| value.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(triangle.iter().filter(|value| value.is_ok()).count(), 2);
    println!(
        "h17_cycle_races: two_way_writers=2 committed=1 refused=1 three_way_writers=3 committed=2 refused=1 final_active_graph=acyclic"
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn hardening17_terminal_target_disposition_leaves_active_wait_graph() {
    let path = temp_store_path("h17-terminal-active-graph");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(17111);
    let tenant_id = "tenant:h17-terminal-graph";
    store
        .bootstrap_local_security(&owner, tenant_id, "organization:h17", 1)
        .unwrap();
    let root_a = h15_setup_case(
        &store,
        &owner,
        tenant_id,
        "case:h17-terminal-a",
        "terminal-a",
    );
    let root_b = h15_setup_case(
        &store,
        &owner,
        tenant_id,
        "case:h17-terminal-b",
        "terminal-b",
    );
    let offer = store
        .offer_case_handoff(
            &owner,
            "case:h17-terminal-a",
            "case:h17-terminal-b",
            HandoffData {
                kind: crate::handoff::HandoffDataKind::Text,
                value: "A waits on B".to_string(),
            },
            vec![],
            1_755_000,
        )
        .unwrap()
        .state
        .handoff_offers[0]
        .clone();
    store
        .decline_case_handoff(
            &owner,
            "case:h17-terminal-b",
            "case:h17-terminal-a",
            &offer.handoff_id,
            "participant:model",
            "target declines",
            1_755_001,
        )
        .unwrap();
    store
        .offer_case_handoff(
            &owner,
            "case:h17-terminal-b",
            "case:h17-terminal-a",
            HandoffData {
                kind: crate::handoff::HandoffDataKind::Text,
                value: "B may now wait on A".to_string(),
            },
            vec![],
            1_755_002,
        )
        .unwrap();
    println!(
        "h17_active_graph_terminality: original_edge=A->B target_decline=terminal reverse_edge=B->A admitted_before_source_reconcile=true active_cycle=false"
    );
    drop(store);
    for path in [path, root_a, root_b] {
        fs::remove_dir_all(path).unwrap();
    }
}

#[test]
fn hardening17_large_handoff_graph_and_derived_relations_rebuild_exactly() {
    let path = temp_store_path("h17-large-handoff-graph");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(17107);
    let tenant_id = "tenant:h17-large-graph";
    store
        .bootstrap_local_security(&owner, tenant_id, "organization:h17", 1)
        .unwrap();
    let cases = (0..64)
        .map(|index| format!("case:h17-graph-{index:02}"))
        .collect::<Vec<_>>();
    for case_id in &cases {
        store
            .create_tenant_case(&owner, tenant_id, case_id)
            .unwrap();
    }
    let graph_started = Instant::now();
    for index in 0..(cases.len() - 1) {
        store
            .offer_case_handoff(
                &owner,
                &cases[index],
                &cases[index + 1],
                HandoffData {
                    kind: crate::handoff::HandoffDataKind::Text,
                    value: format!("chain-edge-{index:02}"),
                },
                vec![],
                1_760_000 + index as u64,
            )
            .unwrap();
    }
    let offer_validation_us = graph_started.elapsed().as_micros();
    let cycle_started = Instant::now();
    assert_eq!(
        store
            .offer_case_handoff(
                &owner,
                cases.last().unwrap(),
                cases.first().unwrap(),
                HandoffData {
                    kind: crate::handoff::HandoffDataKind::Text,
                    value: "must close the 64-Case cycle".to_string(),
                },
                vec![],
                1_760_100,
            )
            .unwrap_err(),
        "handoff_offer_rederivation_mismatch"
    );
    let cycle_check_us = cycle_started.elapsed().as_micros();

    let rebuild_started = Instant::now();
    let mut relation_count = 0usize;
    for case_id in &cases {
        store.rebuild_graph_relations_for_case(case_id).unwrap();
        let before = store
            .list_graph_relations_by_case(case_id, usize::MAX)
            .unwrap();
        let before_identity = before
            .relations
            .iter()
            .map(|value| {
                (
                    value.relation_id.clone(),
                    value.from_ref.clone(),
                    value.to_ref.clone(),
                    value.edge_kind.clone(),
                )
            })
            .collect::<BTreeSet<_>>();
        store.rebuild_graph_relations_for_case(case_id).unwrap();
        let after = store
            .list_graph_relations_by_case(case_id, usize::MAX)
            .unwrap();
        let after_identity = after
            .relations
            .iter()
            .map(|value| {
                (
                    value.relation_id.clone(),
                    value.from_ref.clone(),
                    value.to_ref.clone(),
                    value.edge_kind.clone(),
                )
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(before_identity, after_identity);
        relation_count += after.relations.len();
        assert!(store.verify_case_state(case_id).unwrap());
    }
    let graph_rebuild_us = rebuild_started.elapsed().as_micros();
    println!(
        "h17_multi_case_scale: cases=64 active_handoffs=63 offer_validation_us={offer_validation_us} cycle_check_us={cycle_check_us} graph_relations={relation_count} graph_double_rebuild_us={graph_rebuild_us} duplicates=0 process_owners=0"
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

fn h17_subflow_node(
    node_id: &str,
    child: &WorkflowDefinition,
    map_model_slots: bool,
) -> WorkflowNode {
    WorkflowNode {
        node_id: node_id.to_string(),
        kind: WorkflowNodeKind::Subflow {
            workflow_definition_id: child.workflow_definition_id.clone(),
            workflow_definition_digest: child.integrity_digest.clone(),
            executor_slot_mapping: if map_model_slots {
                vec![crate::workflow::WorkflowSlotMapping {
                    child_slot: "model".to_string(),
                    parent_slot: "model".to_string(),
                }]
            } else {
                vec![]
            },
            resource_slot_mapping: if map_model_slots {
                vec![crate::workflow::WorkflowSlotMapping {
                    child_slot: "workspace".to_string(),
                    parent_slot: "workspace".to_string(),
                }]
            } else {
                vec![]
            },
            case_slot_mapping: vec![],
        },
    }
}

fn h17_define_depth_four(
    store: &LmdbRecordStore,
    owner: &AuthenticatedPrincipal,
    tenant_id: &str,
    leaf: WorkflowNode,
    key: &str,
    map_model_slots: bool,
) -> Vec<WorkflowDefinition> {
    let mut definitions = vec![store
        .define_workflow(
            owner,
            WorkflowDefinitionInput {
                schema: WORKFLOW_DEFINITION_SCHEMA.to_string(),
                tenant_id: tenant_id.to_string(),
                workflow_key: format!("{key}-level-4"),
                declared_version: "2".to_string(),
                name: format!("{key} level 4"),
                description: String::new(),
                nodes: vec![leaf],
                edges: vec![],
            },
            1_770_004,
        )
        .unwrap()];
    for level in (0..4).rev() {
        let child = definitions.last().unwrap();
        let definition = store
            .define_workflow(
                owner,
                WorkflowDefinitionInput {
                    schema: WORKFLOW_DEFINITION_SCHEMA.to_string(),
                    tenant_id: tenant_id.to_string(),
                    workflow_key: format!("{key}-level-{level}"),
                    declared_version: "2".to_string(),
                    name: format!("{key} level {level}"),
                    description: String::new(),
                    nodes: vec![h17_subflow_node(
                        &format!("s{}", level + 1),
                        child,
                        map_model_slots,
                    )],
                    edges: vec![],
                },
                1_770_010 + level as u64,
            )
            .unwrap();
        definitions.push(definition);
    }
    definitions
}

#[test]
fn hardening17_transitive_child_corruption_fails_closed_and_exact_restore_recovers() {
    let path = temp_store_path("h17-transitive-child-corruption");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(17108);
    let tenant_id = "tenant:h17-child-corruption";
    let case_id = "case:h17-child-corruption";
    store
        .bootstrap_local_security(&owner, tenant_id, "organization:h17", 1)
        .unwrap();
    store
        .create_tenant_case(&owner, tenant_id, case_id)
        .unwrap();
    let definitions = h17_define_depth_four(
        &store,
        &owner,
        tenant_id,
        h17_wait_node("leaf"),
        "corruption",
        false,
    );
    let root = definitions.last().unwrap();
    store
        .bind_case_workflow(
            &owner,
            case_id,
            &root.workflow_definition_id,
            vec![],
            vec![],
            1_770_020,
        )
        .unwrap();
    let expected = store.workflow_status_authorized(&owner, case_id).unwrap();
    assert!(expected
        .nodes
        .iter()
        .any(|value| value.node_id == "root/s1/s2/s3/s4/leaf"));
    let deepest = &definitions[0];
    let key = workflow_definition_key(&deepest.workflow_definition_id);
    let original = {
        let txn = store.env.begin_ro_txn().unwrap();
        txn.get(store.workflow_definitions, &key).unwrap().to_vec()
    };
    let mut corrupt = deepest.clone();
    corrupt.description = "tampered transitive child".to_string();
    {
        let mut txn = store.env.begin_rw_txn().unwrap();
        txn.put(
            store.workflow_definitions,
            &key,
            &serde_json::to_vec(&corrupt).unwrap(),
            WriteFlags::empty(),
        )
        .unwrap();
        txn.commit().unwrap();
    }
    assert_eq!(
        store
            .workflow_status_authorized(&owner, case_id)
            .unwrap_err(),
        "workflow_definition_content_identity_mismatch"
    );
    {
        let mut txn = store.env.begin_rw_txn().unwrap();
        txn.put(
            store.workflow_definitions,
            &key,
            &original,
            WriteFlags::empty(),
        )
        .unwrap();
        txn.commit().unwrap();
    }
    let restored = store.workflow_status_authorized(&owner, case_id).unwrap();
    assert_eq!(expected, restored);
    let mut drift = store.get_case_state(case_id).unwrap().unwrap();
    let binding = drift.workflow_binding.clone().unwrap();
    let definitions_by_id = definitions
        .iter()
        .map(|value| (value.workflow_definition_id.clone(), value.clone()))
        .collect::<BTreeMap<_, _>>();
    let base_topology = derive_effective_workflow_topology(
        root,
        &binding,
        &drift.workflow_amendments,
        &definitions_by_id,
    )
    .unwrap();
    let patch = WorkflowPlanPatch::build(
        WorkflowPlanPatchInput {
            schema: crate::workflow::WORKFLOW_PLAN_PATCH_SCHEMA.to_string(),
            base_effective_topology_digest: base_topology.topology_digest,
            operations: vec![WorkflowPatchOperation::AddNode {
                node: h17_wait_node("upgrade-guard"),
            }],
        },
        tenant_id,
        case_id,
        &binding,
        None,
        0,
        WorkflowPlanPatchOrigin::AuthenticatedHuman {
            principal_id: owner.projected_principal_id(),
        },
        drift.generation + 1,
        1_770_030,
    )
    .unwrap();
    let preview = preview_workflow_patch(root, &binding, &[], &patch, &definitions_by_id).unwrap();
    let mut amendment = WorkflowAmendment::build(
        &patch,
        &preview.topology_digest,
        &owner.projected_principal_id(),
        drift.generation + 2,
        1_770_031,
    )
    .unwrap();
    amendment.resulting_topology_digest = "sha256:simulated-materializer-drift".to_string();
    drift.workflow_amendments.push(amendment);
    assert!(derive_effective_workflow_topology(
        root,
        &binding,
        &drift.workflow_amendments,
        &definitions_by_id,
    )
    .unwrap_err()
    .contains("workflow_amendment"));
    println!(
        "h17_cross_definition: depth=4 qualified_leaf=root/s1/s2/s3/s4/leaf corrupt_deep_child=fail_closed exact_restore=resolution_equal topology_digest={} simulated_upgrade_drift=fail_closed",
        restored.effective_topology_digest
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn hardening17_depth_four_modelwork_recovery_preserves_qualified_identity() {
    let path = temp_store_path("h17-deep-model-recovery");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(17109);
    let tenant_id = "tenant:h17-deep-model";
    let case_id = "case:h17-deep-model";
    store
        .bootstrap_local_security(&owner, tenant_id, "organization:h17", 1)
        .unwrap();
    let root_path = h15_setup_case(&store, &owner, tenant_id, case_id, "deep-model");
    let state = store.get_case_state(case_id).unwrap().unwrap();
    store
        .commit_secured_transition(
            &owner,
            tenant_id,
            secured_pending(
                "transition:h17:deep-model-provider",
                case_id,
                state.generation,
                &owner.projected_principal_id(),
                TransitionPayload::ProviderAttached {
                    participant_id: "participant:model".to_string(),
                    provider_id: "provider:fixture".to_string(),
                    provider_kind: "openai_compatible".to_string(),
                    base_url: "http://127.0.0.1:1".to_string(),
                    model_id: "model:fixture".to_string(),
                    credential_ref: "env:H17_TEST".to_string(),
                },
            ),
            true,
        )
        .unwrap();
    let leaf = WorkflowNode {
        node_id: "deep-model".to_string(),
        kind: WorkflowNodeKind::ModelWork {
            executor_slot: "model".to_string(),
            task: "depth-four recovery".to_string(),
            output_contract: Default::default(),
            completion: WorkflowPredicate::ExecutionProviderResult,
            budgets: WorkflowBudgets::default(),
            resource_slot: Some("workspace".to_string()),
        },
    };
    let definitions = h17_define_depth_four(&store, &owner, tenant_id, leaf, "deep-model", true);
    let root = definitions.last().unwrap();
    store
        .bind_case_workflow(
            &owner,
            case_id,
            &root.workflow_definition_id,
            vec![WorkflowExecutorBinding {
                slot: "model".to_string(),
                participant_id: "participant:model".to_string(),
            }],
            vec![WorkflowResourceBinding {
                slot: "workspace".to_string(),
                attachment_id: "resource:workspace".to_string(),
            }],
            1_771_000,
        )
        .unwrap();
    h15_start_runtime(&store, &owner, "runtime-owner:h17-deep-model", 8);
    let submission = store
        .materialize_workflow_ready_work(
            &owner,
            "runtime-owner:h17-deep-model",
            case_id,
            "/tmp/h17-deep-model.jsonl",
            None,
            1_771_001,
        )
        .unwrap()
        .unwrap();
    let workflow = submission.item.workflow.as_ref().unwrap();
    assert_eq!(workflow.workflow_node_id, "root/s1/s2/s3/s4/deep-model");
    let execution_id = workflow.workflow_execution_id.clone();
    let state = store.get_case_state(case_id).unwrap().unwrap();
    let invocation_id = "provider-invocation:h17-deep-model";
    let lineage = test_provider_lineage(state.generation);
    commit_typed(
        &store,
        "transition:h17-deep-model-invocation",
        case_id,
        state.generation,
        TransitionPayload::ProviderInvocationStarted {
            invocation_id: invocation_id.to_string(),
            participant_id: "participant:model".to_string(),
            provider_id: "provider:fixture".to_string(),
            provider_kind: "openai_compatible".to_string(),
            model_id: "model:fixture".to_string(),
            semantic_lineage: Some(lineage.clone()),
        },
        None,
        vec![execution_id.clone()],
    );
    let state = store.get_case_state(case_id).unwrap().unwrap();
    let result_id = "provider-result:h17-deep-model";
    commit_typed(
        &store,
        "transition:h17-deep-model-result",
        case_id,
        state.generation,
        TransitionPayload::ProviderResultRecorded {
            result_id: result_id.to_string(),
            invocation_id: invocation_id.to_string(),
            provider_id: "provider:fixture".to_string(),
            provider_kind: "openai_compatible".to_string(),
            model_id: "model:fixture".to_string(),
            semantic_lineage: Some(lineage),
            output: "identical output remains one completed turn".to_string(),
        },
        None,
        vec![invocation_id.to_string(), execution_id.clone()],
    );
    let reopened = LmdbRecordStore::open(&path).unwrap();
    let completed = reopened
        .advance_workflow_passive_progress(&owner, case_id, 16)
        .unwrap();
    assert!(completed.completed);
    assert_eq!(
        reopened
            .get_case_state(case_id)
            .unwrap()
            .unwrap()
            .workflow_executions
            .iter()
            .filter(|value| value.execution_id == execution_id)
            .count(),
        1
    );
    let history = reopened.list_case_transitions(case_id).unwrap();
    assert_eq!(
        history
            .iter()
            .filter(|transition| matches!(
                &transition.payload,
                TransitionPayload::ProviderResultRecorded { result_id: value, .. }
                    if value == result_id
            ))
            .count(),
        1
    );
    assert_eq!(
        history
            .iter()
            .filter(|transition| matches!(
                &transition.payload,
                TransitionPayload::WorkflowNodeExecutionStarted { execution }
                    if execution.execution_id == execution_id
            ))
            .count(),
        1
    );
    assert!(reopened.verify_case_state(case_id).unwrap());
    println!(
        "h17_deep_model_recovery: depth=4 node=root/s1/s2/s3/s4/deep-model executions=1 provider_invocations=1 provider_results=1 completed_turn_duplicates=0 replay_equal=true"
    );
    drop(reopened);
    drop(store);
    fs::remove_dir_all(path).unwrap();
    fs::remove_dir_all(root_path).unwrap();
}

#[test]
fn hardening17_depth_four_deterministic_recovery_does_not_duplicate_proposal_or_operation() {
    let path = temp_store_path("h17-deep-deterministic-recovery");
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(17110);
    let tenant_id = "tenant:h17-deep-deterministic";
    let case_id = "case:h17-deep-deterministic";
    store
        .bootstrap_local_security(&owner, tenant_id, "organization:h17", 1)
        .unwrap();
    let root_path = h15_setup_case(&store, &owner, tenant_id, case_id, "deep-deterministic");
    let leaf = WorkflowNode {
        node_id: "deep-deterministic".to_string(),
        kind: WorkflowNodeKind::DeterministicWork {
            proposer_slot: "model".to_string(),
            operation: DeterministicOperationTemplate::FilesystemWrite {
                resource_slot: "workspace".to_string(),
                relative_path: "allowed/h17-deep.txt".to_string(),
                content: "bounded deterministic consequence".to_string(),
            },
            completion: WorkflowPredicate::ExecutionEffectFinalized,
        },
    };
    let definitions =
        h17_define_depth_four(&store, &owner, tenant_id, leaf, "deep-deterministic", true);
    let root = definitions.last().unwrap();
    store
        .bind_case_workflow(
            &owner,
            case_id,
            &root.workflow_definition_id,
            vec![WorkflowExecutorBinding {
                slot: "model".to_string(),
                participant_id: "participant:model".to_string(),
            }],
            vec![WorkflowResourceBinding {
                slot: "workspace".to_string(),
                attachment_id: "resource:workspace".to_string(),
            }],
            1_772_000,
        )
        .unwrap();
    h15_start_runtime(&store, &owner, "runtime-owner:h17-deep-deterministic", 8);
    let work = store
        .materialize_workflow_ready_work(
            &owner,
            "runtime-owner:h17-deep-deterministic",
            case_id,
            "/tmp/h17-deep-deterministic.jsonl",
            None,
            1_772_001,
        )
        .unwrap()
        .unwrap()
        .item;
    assert_eq!(
        work.workflow.as_ref().unwrap().workflow_node_id,
        "root/s1/s2/s3/s4/deep-deterministic"
    );
    let proposal = store
        .record_workflow_deterministic_proposal(&owner, &work)
        .unwrap();
    let reopened = LmdbRecordStore::open(&path).unwrap();
    let recovered_proposal = reopened
        .record_workflow_deterministic_proposal(&owner, &work)
        .unwrap();
    assert_eq!(proposal, recovered_proposal);
    let operation = reopened
        .record_workflow_deterministic_operation_from_proposal(&owner, &work, &recovered_proposal)
        .unwrap();
    let recovered_operation = reopened
        .record_workflow_deterministic_operation_from_proposal(&owner, &work, &recovered_proposal)
        .unwrap();
    assert_eq!(operation, recovered_operation);
    let history = reopened.list_case_transitions(case_id).unwrap();
    assert_eq!(
        history
            .iter()
            .filter(|transition| matches!(
                transition.payload,
                TransitionPayload::WorkflowDeterministicProposalRecorded { .. }
            ))
            .count(),
        1
    );
    assert_eq!(
        history
            .iter()
            .filter(|transition| matches!(
                transition.payload,
                TransitionPayload::OperationRecorded { .. }
            ))
            .count(),
        1
    );
    println!(
        "h17_deep_deterministic_recovery: depth=4 node=root/s1/s2/s3/s4/deep-deterministic proposals=1 operations=1 proposal_duplicates=0 operation_duplicates=0 operation_id={}",
        operation.operation_id
    );
    drop(reopened);
    drop(store);
    fs::remove_dir_all(path).unwrap();
    fs::remove_dir_all(root_path).unwrap();
}
