use super::*;
use crate::effect::access::*;
use crate::effect::{DecisionOutcome, OperationKind};
use crate::transition::EffectLifecycle;

const CASE: &str = "case:resource-contract";
const TENANT: &str = "tenant:resource-contract";
const HUMAN: &str = "participant:human";
const RESOURCE: &str = "resource:workspace";

struct World {
    path: PathBuf,
    store: LmdbRecordStore,
    owner: AuthenticatedPrincipal,
    outsider: AuthenticatedPrincipal,
    binding: LocalAccessBinding,
    artifact_id: String,
}

impl World {
    fn new() -> Self {
        Self::with_process(false)
    }

    fn with_process(process: bool) -> Self {
        Self::with_access(if process {
            AccessKind::ProcessRun
        } else {
            AccessKind::FilesystemRead
        })
    }

    fn with_access(kind: AccessKind) -> Self {
        let process = kind == AccessKind::ProcessRun;
        let path = temp_store_path("resource-access");
        fs::create_dir_all(path.join("workspace/src")).unwrap();
        fs::create_dir_all(path.join("workspace/protected")).unwrap();
        fs::write(
            path.join("workspace/src/retry.txt"),
            b"current retry constraint",
        )
        .unwrap();
        fs::write(
            path.join("workspace/protected/secret.txt"),
            b"not disclosed",
        )
        .unwrap();
        let store = LmdbRecordStore::open(path.join("store")).unwrap();
        let owner = AuthenticatedPrincipal::for_test(29001);
        let outsider = AuthenticatedPrincipal::for_test(29002);
        store
            .bootstrap_local_security(&owner, TENANT, "organization:resource-contract", 1)
            .unwrap();
        store
            .bootstrap_local_security(&outsider, "tenant:outsider", "organization:other", 1)
            .unwrap();
        let principal = owner.projected_principal_id();
        let mut state = store
            .create_tenant_case(&owner, TENANT, CASE)
            .unwrap()
            .state;
        for (participant, role) in [
            (HUMAN, "operation-proposer"),
            ("participant:model", "model-executor"),
        ] {
            state = store
                .commit_secured_transition(
                    &owner,
                    TENANT,
                    secured_pending(
                        &format!("transition:bind:{participant}"),
                        CASE,
                        state.generation,
                        &principal,
                        TransitionPayload::ParticipantBound {
                            participant_id: participant.into(),
                            role: role.into(),
                        },
                    ),
                    true,
                )
                .unwrap()
                .state;
        }
        let link = crate::transition::PrincipalParticipantLink::new(
            CASE, TENANT, &principal, HUMAN, &principal, 2,
        )
        .unwrap();
        let mut pending = secured_pending(
            "transition:human-link",
            CASE,
            state.generation,
            &principal,
            TransitionPayload::ParticipantPrincipalLinked { link },
        );
        pending.causal_refs = vec![principal.clone(), HUMAN.into()];
        state = store
            .commit_secured_transition(&owner, TENANT, pending, true)
            .unwrap()
            .state;
        let root = LocalFilesystemBinding::new(CASE, RESOURCE, &path.join("workspace")).unwrap();
        for participant in [HUMAN, "participant:model"] {
            state = store
                .commit_secured_transition(
                    &owner,
                    TENANT,
                    secured_pending(
                        &format!("transition:view:{participant}"),
                        CASE,
                        state.generation,
                        &principal,
                        TransitionPayload::ParticipantAdmitted {
                            participant_id: participant.into(),
                            consumer: "model".into(),
                            view_kind: "model_context".into(),
                        },
                    ),
                    true,
                )
                .unwrap()
                .state;
        }
        let address = if process {
            let executable = fs::canonicalize("/usr/bin/python3").unwrap();
            let runner = ProcessRunner {
                executable: executable.to_str().unwrap().into(),
                executable_digest: digest_bytes(&fs::read(executable).unwrap()),
                argv: vec![
                    "-I".into(),
                    "-B".into(),
                    "-c".into(),
                    "import sys; print('real governed test outcome'); sys.exit(7)".into(),
                ],
                working_directory: "src".into(),
                environment: Default::default(),
                timeout_ms: 1000,
            };
            ResourceAddress::ProcessRunner {
                root,
                runners: [("test".into(), runner)].into(),
            }
        } else if kind == AccessKind::Discover {
            ResourceAddress::Discovery { root }
        } else {
            ResourceAddress::Filesystem { root }
        };
        let binding = LocalAccessBinding {
            schema: LOCAL_ACCESS_BINDING_SCHEMA.into(),
            case_id: CASE.into(),
            attachment_id: RESOURCE.into(),
            address,
        };
        let attachment = ResourceAttachmentState {
            attachment_id: RESOURCE.into(),
            kind: binding.kind(),
            allowed_write_prefix: String::new(),
            max_write_bytes: 0,
            policy_id: "policy:resource-envelope".into(),
            policy_owner_participant_id: HUMAN.into(),
            review_requirement: ReviewRequirement::Automatic,
            process_signal_actions: vec![],
            access: Some(ResourceAccessContract {
                schema: RESOURCE_ACCESS_SCHEMA.into(),
                configuration_digest: binding.digest(),
                participant_ids: vec![HUMAN.into(), "participant:model".into()],
                operations: if kind == AccessKind::Discover {
                    vec![kind, AccessKind::AdmitContent]
                } else {
                    vec![kind]
                },
                read_prefixes: vec!["src".into()],
                names: if process { vec!["test".into()] } else { vec![] },
                max_output_bytes: 4096,
                max_items: 16,
            }),
        };
        let mut pending = secured_pending(
            "transition:workspace",
            CASE,
            state.generation,
            &principal,
            TransitionPayload::ResourceAttached { attachment },
        );
        pending.scope = Some(TransitionScope {
            case_id: CASE.into(),
            participant_refs: vec![HUMAN.into()],
            resource_refs: vec![RESOURCE.into()],
            policy_refs: vec!["policy:resource-envelope".into()],
        });
        pending.causal_refs = vec![HUMAN.into()];
        state = store
            .commit_tenant_access_attachment(&owner, TENANT, pending, &binding)
            .unwrap()
            .state;
        let bytes = serde_json::to_vec(&serde_json::json!({
            "schema": POLICY_SOURCE_INPUT_SCHEMA, "policy_key":"resources.read", "source_version":"1",
            "owner_ref":"organization:resource-contract", "source_origin": {"source_system":"resource-contract-test","source_uri":"test://resource-contract/policy"},
            "validity":{"mode":"unbounded"}, "rules":[
                {"kind":"operation_restriction","rule_id":"read","operation_kind":kind.operation_name(),"resource_kind":kind.resource_name(),"effect":"allow","reason":"bounded resource action"},
                {"kind":"authority_requirement","rule_id":"proposer","operation_kind":kind.operation_name(),"resource_kind":kind.resource_name(),"subject":"proposer","required_role":"operation-proposer","reason":"explicit proposer role"},
                {"kind":"evidence_obligation","rule_id":"origin","operation_kind":kind.operation_name(),"resource_kind":kind.resource_name(),"obligation":"source_provenance","reason":"authenticated request"},
                {"kind":"operation_restriction","rule_id":"content-admit","operation_kind":"content.admit","resource_kind":"discovery","effect":"allow","reason":"separate immutable import authority"}
            ]
        })).unwrap();
        let compilation = scope_policy_compilation(
            &compile_policy_source(&bytes).unwrap(),
            TENANT,
            "organization:resource-contract",
        )
        .unwrap();
        let artifact_id = compilation.artifact.artifact_id.clone();
        store
            .ingest_tenant_policy_compilation(&owner, TENANT, &compilation)
            .unwrap();
        store
            .validate_tenant_policy_artifact(&owner, &artifact_id, "validate contract")
            .unwrap();
        store
            .publish_tenant_policy_artifact(&owner, &artifact_id, "publish contract")
            .unwrap();
        store
            .bind_tenant_case_policy(
                &owner,
                CASE,
                &artifact_id,
                state.generation,
                "admit bounded resource reads",
            )
            .unwrap();
        Self {
            path,
            store,
            owner,
            outsider,
            binding,
            artifact_id,
        }
    }

    fn request(&self, path: &str) -> ResourceRequest {
        ResourceRequest {
            schema: RESOURCE_REQUEST_SCHEMA.into(),
            configuration_digest: self.binding.digest(),
            action: ResourceAction::FilesystemRead { path: path.into() },
        }
    }

    fn operation(&self, id: &str, path: &str) -> Operation {
        self.store
            .record_participant_resource_request(
                &self.owner,
                CASE,
                HUMAN,
                RESOURCE,
                id,
                self.store.get_case_state(CASE).unwrap().unwrap().generation,
                self.request(path),
            )
            .unwrap()
    }

    fn result(&self, admission: &ResourceReadAdmission) -> serde_json::Value {
        let path = admission
            .operation
            .resource_request
            .as_ref()
            .unwrap()
            .action
            .path()
            .unwrap();
        let bytes = read_confined_file(
            admission.binding.root().unwrap(),
            path,
            admission.access.max_output_bytes,
        )
        .unwrap();
        serde_json::json!({"posture":"read", "digest":digest_bytes(&bytes), "text":String::from_utf8(bytes).unwrap()})
    }

    fn finish(self) {
        let path = self.path;
        drop(self.store);
        fs::remove_dir_all(path).unwrap();
    }
}

#[test]
fn capability_view_is_current_scoped_policy_constrained_and_not_authority() {
    let world = World::new();
    let history = world.store.list_case_transitions(CASE).unwrap();
    let human = world
        .store
        .case_capability_view_authorized(&world.owner, CASE, HUMAN)
        .unwrap();
    assert_eq!(human.entries.len(), 1);
    assert_eq!(
        human.entries[0].operation_kind,
        OperationKind::ResourceAccess(AccessKind::FilesystemRead)
    );
    assert!(human.entries[0].requires_current_decision);
    assert!(!human.entries[0].policy_constraints.is_empty());
    assert_eq!(
        world
            .store
            .case_capability_view_authorized(&world.owner, CASE, HUMAN)
            .unwrap(),
        human
    );
    let model = world
        .store
        .case_capability_view_authorized(&world.owner, CASE, "participant:model")
        .unwrap();
    assert!(model.entries.is_empty());
    assert_eq!(model.exclusions[0].reason, "proposer_role_missing");
    assert!(world
        .store
        .case_capability_view_authorized(&world.outsider, CASE, HUMAN)
        .is_err());
    assert_eq!(world.store.list_case_transitions(CASE).unwrap(), history);
    let other = "case:resource-isolation";
    let mut other_state = world
        .store
        .create_tenant_case(&world.owner, TENANT, other)
        .unwrap()
        .state;
    assert!(world
        .store
        .case_capability_view_authorized(&world.owner, other, HUMAN)
        .is_err());
    other_state = world
        .store
        .commit_secured_transition(
            &world.owner,
            TENANT,
            secured_pending(
                "transition:other-human",
                other,
                other_state.generation,
                &world.owner.projected_principal_id(),
                TransitionPayload::ParticipantBound {
                    participant_id: HUMAN.into(),
                    role: "operation-proposer".into(),
                },
            ),
            true,
        )
        .unwrap()
        .state;
    world
        .store
        .bind_tenant_case_policy(
            &world.owner,
            other,
            &world.artifact_id,
            other_state.generation,
            "explicit policy, no inherited resources",
        )
        .unwrap();
    let other_view = world
        .store
        .case_capability_view_authorized(&world.owner, other, HUMAN)
        .unwrap();
    assert!(other_view.entries.is_empty() && other_view.exclusions.is_empty());
    let state = world.store.get_case_state(CASE).unwrap().unwrap();
    world
        .store
        .commit_secured_transition(
            &world.owner,
            TENANT,
            secured_pending(
                "transition:model-proposer",
                CASE,
                state.generation,
                &world.owner.projected_principal_id(),
                TransitionPayload::ParticipantBound {
                    participant_id: "participant:model".into(),
                    role: "operation-proposer".into(),
                },
            ),
            true,
        )
        .unwrap();
    let admitted = world
        .store
        .case_capability_view_authorized(&world.owner, CASE, "participant:model")
        .unwrap();
    assert_eq!(admitted.entries.len(), 1);
    assert_ne!(admitted.view_id, model.view_id);
    assert!(admitted.entries[0].requires_current_decision);
    // Requestability cannot authorize a protected concrete argument.
    let operation = world.operation("request:protected-after-view", "protected/secret.txt");
    let (decision, _) = world
        .store
        .derive_and_commit_policy_decision(CASE, &operation.operation_id)
        .unwrap();
    assert_eq!(decision.outcome, DecisionOutcome::Deny);
    assert!(world
        .store
        .get_case_state(CASE)
        .unwrap()
        .unwrap()
        .grants
        .is_empty());
    world
        .store
        .revoke_tenant_policy_artifact(
            &world.owner,
            &world.artifact_id,
            "revoked views cannot authorize fresh work",
        )
        .unwrap();
    assert!(world
        .store
        .case_capability_view_authorized(&world.owner, CASE, HUMAN)
        .is_err());
    println!("capability_view: derived=true view_created_decisions=0 role_constraint=exact cross_case=empty cross_tenant=refused protected_request=deny revoked_policy=refused provider=no_provider");
    world.finish();
}

#[test]
fn discovered_content_is_admitted_exactly_without_turn_or_external_authority() {
    use crate::conversation::ConversationContentStore;
    let world = World::with_access(AccessKind::Discover);
    let content_store = ConversationContentStore::open(&world.path).unwrap();
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
        "request:discover",
        ResourceAction::Discover { path: "src".into() },
    );
    world
        .store
        .derive_and_commit_policy_decision(CASE, &discover.operation_id)
        .unwrap();
    let read = world
        .store
        .admit_resource_read_authorized(&world.owner, CASE, &discover.operation_id)
        .unwrap();
    let result =
        inspect_confined_tree(world.binding.root().unwrap(), "src", None, 4096, 16).unwrap();
    let discovery = world
        .store
        .record_resource_observation_authorized(&world.owner, &read, result)
        .unwrap();
    let bytes = fs::read(world.path.join("workspace/src/retry.txt")).unwrap();
    let digest = digest_bytes(&bytes);
    let operation = record(
        "request:admit",
        ResourceAction::AdmitContent {
            path: "src/retry.txt".into(),
            candidate_digest: digest.clone(),
        },
    );
    let (decision, _) = world
        .store
        .derive_and_commit_policy_decision(CASE, &operation.operation_id)
        .unwrap();
    assert_eq!(decision.outcome, DecisionOutcome::Allow);
    assert!(world
        .store
        .admit_resource_read_authorized(&world.owner, CASE, &operation.operation_id)
        .is_err());
    assert!(world
        .store
        .admit_discovered_content_authorized(
            &world.outsider,
            &content_store,
            CASE,
            &operation.operation_id
        )
        .is_err());
    fs::write(
        world.path.join("workspace/src/retry.txt"),
        b"changed candidate",
    )
    .unwrap();
    assert!(world
        .store
        .admit_discovered_content_authorized(
            &world.owner,
            &content_store,
            CASE,
            &operation.operation_id
        )
        .unwrap_err()
        .contains("candidate_drift"));
    assert!(world
        .store
        .get_case_state(CASE)
        .unwrap()
        .unwrap()
        .admitted_content
        .is_empty());
    fs::write(world.path.join("workspace/src/retry.txt"), &bytes).unwrap();
    let object = content_store
        .publish_owned_file_bytes(TENANT, CASE, &bytes)
        .unwrap();
    let admission = CaseContentAdmission::new(
        &operation,
        &decision,
        &discovery,
        &world.owner.projected_principal_id(),
        vec![HUMAN.into(), "participant:model".into()],
        object.clone(),
    )
    .unwrap();
    let mut pending = secured_pending(
        "transition:content-admission-test",
        CASE,
        world
            .store
            .get_case_state(CASE)
            .unwrap()
            .unwrap()
            .generation,
        &world.owner.projected_principal_id(),
        TransitionPayload::CaseContentAdmitted {
            admission: admission.clone(),
        },
    );
    pending.source.participant_id = Some(HUMAN.into());
    pending.scope = Some(TransitionScope {
        case_id: CASE.into(),
        participant_refs: admission.participant_ids.clone(),
        resource_refs: vec![RESOURCE.into()],
        policy_refs: operation.scope.policy_refs.clone(),
    });
    pending.causal_refs = vec![
        operation.operation_id.clone(),
        decision.decision_id.clone(),
        discovery.observation_id.clone(),
        RESOURCE.into(),
    ];
    // A raw canonical writer cannot assert that a payload has been verified.
    assert!(world
        .store
        .commit_secured_transition(&world.owner, TENANT, pending.clone(), false)
        .unwrap_err()
        .contains("verified_owned_bytes"));
    let before = world.store.list_case_transitions(CASE).unwrap();
    let mut txn = world.store.env.begin_rw_txn().unwrap();
    let context = world
        .store
        .resolve_security_context_txn(&txn, &world.owner, TENANT)
        .unwrap();
    assert!(world
        .store
        .commit_transition_txn_with_owned_content(
            &mut txn,
            pending.clone(),
            true,
            None,
            Some(&context),
            None,
            Some(&content_store)
        )
        .unwrap_err()
        .contains("injected_failure"));
    drop(txn);
    assert_eq!(world.store.list_case_transitions(CASE).unwrap(), before);
    assert_eq!(content_store.read_bytes(&object).unwrap(), bytes); // complete orphan is permitted, no half relation
    let mut stale = pending.clone();
    stale.expected_generation -= 1;
    let mut txn = world.store.env.begin_rw_txn().unwrap();
    let context = world
        .store
        .resolve_security_context_txn(&txn, &world.owner, TENANT)
        .unwrap();
    assert!(world
        .store
        .commit_transition_txn_with_owned_content(
            &mut txn,
            stale,
            false,
            None,
            Some(&context),
            None,
            Some(&content_store)
        )
        .unwrap_err()
        .contains("stale_case_generation"));
    drop(txn);
    let actual = world
        .store
        .admit_discovered_content_authorized(
            &world.owner,
            &content_store,
            CASE,
            &operation.operation_id,
        )
        .unwrap();
    assert_eq!(actual, admission);
    assert!(world
        .store
        .admit_discovered_content_authorized(
            &world.owner,
            &content_store,
            CASE,
            &operation.operation_id
        )
        .is_err());
    let history = world.store.list_case_transitions(CASE).unwrap();
    assert!(!history.iter().any(|t| matches!(
        &t.payload,
        TransitionPayload::ConversationTurnCommitted { .. }
            | TransitionPayload::ExecutionGrantIssued { .. }
    )));
    let mut downgraded = history.last().unwrap().clone();
    downgraded.schema = crate::transition::TRANSITION_SCHEMA_V17.into();
    assert!(downgraded.validate().is_err());
    fs::write(
        world.path.join("workspace/src/retry.txt"),
        b"source removed or changed after admission",
    )
    .unwrap();
    let World {
        store, path, owner, ..
    } = world;
    drop(store);
    drop(content_store);
    let store = LmdbRecordStore::open(path.join("store")).unwrap();
    let objects = ConversationContentStore::open(&path).unwrap();
    assert_eq!(
        store
            .get_case_state_authorized(&owner, CASE)
            .unwrap()
            .admitted_content,
        vec![actual.clone()]
    );
    assert_eq!(objects.read_bytes(&actual.object).unwrap(), bytes);
    assert!(store.verify_case_state(CASE).unwrap());
    assert_eq!(
        store.rebuild_case_state(CASE).unwrap().admitted_content,
        vec![actual]
    );
    assert_eq!(store.list_case_transitions(CASE).unwrap(), history);
    println!("discovery_admission: drift=refused outsider=refused unverified_writer=refused failed_commit=atomic stale_generation=refused owned_bytes=exact replay=equivalent turns=0 grants=0");
    drop(store);
    drop(objects);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn published_i06_store_markers_upgrade_without_history_rewrite() {
    let path = temp_store_path("golden-i06-marker-upgrade");
    let store = LmdbRecordStore::open(&path).unwrap();
    let mut txn = store.env.begin_rw_txn().unwrap();
    txn.put(
        store.schema_meta,
        &"meta:canonical_transition_schema",
        &crate::transition::TRANSITION_SCHEMA_V17,
        WriteFlags::empty(),
    )
    .unwrap();
    txn.put(
        store.schema_meta,
        &"meta:case_state_schema",
        &crate::transition::CASE_STATE_SCHEMA_V14,
        WriteFlags::empty(),
    )
    .unwrap();
    txn.commit().unwrap();
    drop(store);
    let store = LmdbRecordStore::open(&path).unwrap();
    let txn = store.env.begin_ro_txn().unwrap();
    assert_eq!(
        txn.get(store.schema_meta, &"meta:canonical_transition_schema")
            .unwrap(),
        crate::transition::TRANSITION_SCHEMA.as_bytes()
    );
    assert_eq!(
        txn.get(store.schema_meta, &"meta:case_state_schema")
            .unwrap(),
        crate::transition::CASE_STATE_SCHEMA.as_bytes()
    );
    drop(txn);
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn resource_read_real_filesystem_policy_lineage_replay_and_restart() {
    let world = World::new();
    let operation = world.operation("request:first", "src/retry.txt");
    let (decision, _) = world
        .store
        .derive_and_commit_policy_decision(CASE, &operation.operation_id)
        .unwrap();
    assert_eq!(decision.outcome, DecisionOutcome::Allow);
    let admission = world
        .store
        .admit_resource_read_authorized(&world.owner, CASE, &operation.operation_id)
        .unwrap();
    let observation = world
        .store
        .record_resource_observation_authorized(&world.owner, &admission, world.result(&admission))
        .unwrap();
    assert_eq!(observation.operation_id, operation.operation_id);
    assert_eq!(observation.decision_id, decision.decision_id);
    let current = world.store.get_case_state(CASE).unwrap().unwrap();
    let history = world.store.list_case_transitions(CASE).unwrap();
    let projected = crate::context::compile_projection(
        &current,
        &history,
        &crate::context::ProjectionRequest::model(
            HUMAN,
            crate::context::ProjectionPurpose::Conversation,
        ),
        &Default::default(),
    )
    .unwrap();
    assert!(projected.entries.iter().any(|entry| matches!(&entry.value, crate::context::ProjectedValue::ResourceObservation { observation_id, preview, truncated, .. }
        if observation_id == &observation.observation_id && preview.contains("current retry constraint") && !truncated)));
    let other = crate::context::compile_projection(
        &current,
        &history,
        &crate::context::ProjectionRequest::model(
            "participant:model",
            crate::context::ProjectionPurpose::Conversation,
        ),
        &Default::default(),
    )
    .unwrap();
    assert!(!other.entries.iter().any(|entry| matches!(
        entry.value,
        crate::context::ProjectedValue::ResourceObservation { .. }
    )));
    assert!(
        issue_policy_execution_grant(&operation, &decision, admission.case_generation).is_err()
    );
    let history = world.store.list_case_transitions(CASE).unwrap();
    assert!(!history.iter().any(|transition| matches!(
        transition.payload,
        TransitionPayload::ExecutionGrantIssued { .. } | TransitionPayload::EffectPrepared { .. }
    )));
    assert!(world.store.verify_case_state(CASE).unwrap());
    let state = world.store.get_case_state(CASE).unwrap().unwrap();
    // Read-only attachment changes are not smuggled into old serialized history.
    let attached = history
        .iter()
        .find(|transition| {
            matches!(
                transition.payload,
                TransitionPayload::ResourceAttached { .. }
            )
        })
        .unwrap();
    let mut downgraded = attached.clone();
    downgraded.schema = crate::transition::TRANSITION_SCHEMA_V17.into();
    assert!(downgraded.validate().is_err());
    world
        .store
        .materialize_graph_relations_for_case(CASE)
        .unwrap();
    assert_eq!(world.store.replay_case_state(CASE).unwrap(), state);
    let path = world.path.clone();
    drop(world.store);
    let reopened = LmdbRecordStore::open(path.join("store")).unwrap();
    assert_eq!(reopened.replay_case_state(CASE).unwrap(), state);
    assert_eq!(reopened.list_case_transitions(CASE).unwrap(), history);
    assert_eq!(
        reopened.get_local_access_binding(CASE, RESOURCE).unwrap(),
        Some(world.binding)
    );
    println!("resource_read: operation={} decision={} observation={} provider=no_provider effect_grant=none replay=equal restart=equal", operation.operation_id, decision.decision_id, observation.observation_id);
    drop(reopened);
    fs::remove_dir_all(path).unwrap();
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn resource_process_canonical_prepare_fence_real_exit_terminal_and_replay() {
    let world = World::with_process(true);
    let operation = world
        .store
        .record_participant_resource_request(
            &world.owner,
            CASE,
            HUMAN,
            RESOURCE,
            "request:process",
            world
                .store
                .get_case_state(CASE)
                .unwrap()
                .unwrap()
                .generation,
            ResourceRequest {
                schema: RESOURCE_REQUEST_SCHEMA.into(),
                configuration_digest: world.binding.digest(),
                action: ResourceAction::ProcessRun {
                    name: "test".into(),
                },
            },
        )
        .unwrap();
    let (decision, _) = world
        .store
        .derive_and_commit_policy_decision(CASE, &operation.operation_id)
        .unwrap();
    assert_eq!(decision.outcome, DecisionOutcome::Allow);
    let generation = world
        .store
        .get_case_state(CASE)
        .unwrap()
        .unwrap()
        .generation;
    let grant = issue_policy_execution_grant(&operation, &decision, generation).unwrap();
    assert_eq!(grant.schema, crate::effect::RESOURCE_EXECUTION_GRANT_SCHEMA);
    let mut pending = PendingTransition::new(
        "transition:process-grant",
        CASE,
        generation,
        TransitionSource::component("resource-contract"),
        TransitionPayload::ExecutionGrantIssued {
            grant: grant.clone(),
        },
    );
    pending.causal_refs = vec![
        operation.operation_id.clone(),
        decision.decision_id.clone(),
        grant.decision_basis_id.clone().unwrap(),
        grant.effective_policy_id.clone().unwrap(),
    ];
    world.store.commit_transition(pending).unwrap();
    let pre = ResourceObservation::new(&operation, &decision,
        serde_json::json!({"posture":"configured_exact_process", "configuration_digest":world.binding.digest()}), authority_wall_time_unix_ms()).unwrap();
    let prepared = PreparedResourceEffect::new(&operation, &decision, &grant, pre).unwrap();
    let mut pending = PendingTransition::new(
        "transition:resource-prepare",
        CASE,
        generation + 1,
        TransitionSource {
            component: "resource-contract".into(),
            participant_id: Some(HUMAN.into()),
            principal_id: Some(world.owner.projected_principal_id()),
            source_ref: Some(operation.operation_id.clone()),
        },
        TransitionPayload::ResourceEffectPrepared { prepared },
    );
    let TransitionPayload::ResourceEffectPrepared { prepared } = &pending.payload else {
        unreachable!()
    };
    pending.causal_refs = vec![
        operation.operation_id.clone(),
        decision.decision_id.clone(),
        grant.grant_id.clone(),
        prepared.expected_pre_observation.observation_id.clone(),
    ];
    pending.scope = Some(operation.scope.clone());
    // No raw Transition can publish half a PREPARE or forge its lease.
    assert!(world
        .store
        .commit_transition(pending.clone())
        .unwrap_err()
        .contains("atomic_fence"));
    assert!(world
        .store
        .commit_fenced_resource_effect_prepared(&world.outsider, pending.clone())
        .is_err());
    let mut forged = pending.clone();
    if let TransitionPayload::ResourceEffectPrepared { prepared } = &mut forged.payload {
        prepared.effect_id.push_str("forged");
    }
    assert!(world
        .store
        .commit_fenced_resource_effect_prepared(&world.owner, forged)
        .unwrap_err()
        .contains("exact_canonical"));
    let PreparedCommitOutcome::Prepared(commit) = world
        .store
        .commit_fenced_resource_effect_prepared(&world.owner, pending.clone())
        .unwrap()
    else {
        panic!("valid Grant")
    };
    let TransitionPayload::ResourceEffectPrepared { prepared } = commit.transition.payload else {
        unreachable!()
    };
    let fence = prepared.resource_fence.as_ref().unwrap();
    assert_eq!(
        world
            .store
            .get_resource_control_state(&fence.resource_id)
            .unwrap()
            .unwrap()
            .active_lease
            .unwrap()
            .fence,
        *fence
    );
    assert!(world
        .store
        .commit_fenced_resource_effect_prepared(&world.owner, pending)
        .is_err());
    let result = world
        .store
        .execute_prepared_resource_process(&world.owner, &prepared)
        .unwrap();
    assert_eq!(result["exit_code"], 7);
    assert_eq!(result["stdout"], "real governed test outcome\n");
    // Revocation after the external outcome prevents fresh dispatch, but must
    // not erase the outcome or prevent truthful terminal recording.
    world
        .store
        .revoke_tenant_policy_artifact(
            &world.owner,
            &world.artifact_id,
            "revoke after process outcome",
        )
        .unwrap();
    assert!(world
        .store
        .validate_resource_effect_dispatch_authorized(&world.owner, &prepared)
        .is_err());
    let post =
        ResourceObservation::new(&operation, &decision, result, authority_wall_time_unix_ms())
            .unwrap();
    let receipt = ResourceEffectReceipt::new(
        &prepared,
        &post,
        crate::effect::EffectOutcome::Applied,
        true,
    )
    .unwrap();
    let mut pending = PendingTransition::new(
        "transition:resource-terminal",
        CASE,
        commit.state.generation,
        TransitionSource::component("resource-contract"),
        TransitionPayload::ResourceEffectFinalized {
            effect_id: prepared.effect_id.clone(),
            observation: post.clone(),
            receipt: receipt.clone(),
        },
    );
    pending.scope = Some(operation.scope.clone());
    pending.causal_refs = vec![
        prepared.effect_id.clone(),
        operation.operation_id.clone(),
        grant.grant_id.clone(),
        prepared.expected_pre_observation.observation_id.clone(),
    ];
    assert!(world.store.commit_transition(pending.clone()).is_err());
    let mut forged = pending.clone();
    if let TransitionPayload::ResourceEffectFinalized { receipt, .. } = &mut forged.payload {
        receipt.grant_id = "grant:unrelated".into();
    }
    assert!(world
        .store
        .commit_fenced_effect_terminal(forged, fence)
        .is_err());
    assert!(world
        .store
        .get_resource_control_state(&fence.resource_id)
        .unwrap()
        .unwrap()
        .active_lease
        .is_some());
    world
        .store
        .commit_fenced_effect_terminal(pending, fence)
        .unwrap();
    assert!(world
        .store
        .get_resource_control_state(&fence.resource_id)
        .unwrap()
        .unwrap()
        .active_lease
        .is_none());
    assert!(world
        .store
        .execute_prepared_resource_process(&world.owner, &prepared)
        .is_err());
    let state = world.store.get_case_state(CASE).unwrap().unwrap();
    assert_eq!(state.effects[0].status, EffectLifecycle::Finalized);
    assert_eq!(world.store.replay_case_state(CASE).unwrap(), state);
    let path = world.path.clone();
    drop(world.store);
    let reopened = LmdbRecordStore::open(path.join("store")).unwrap();
    assert_eq!(reopened.replay_case_state(CASE).unwrap(), state);
    assert!(reopened
        .execute_prepared_resource_process(&world.owner, &prepared)
        .is_err());
    println!("resource_process: operation={} grant={} effect={} receipt={} exit=7 replay=equal restart=no_redispatch provider=no_provider", operation.operation_id, grant.grant_id, prepared.effect_id, receipt.receipt_id);
    drop(reopened);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn resource_read_denial_stale_policy_scope_and_request_identity_fail_closed() {
    let world = World::new();
    let denied = world.operation("request:protected", "protected/secret.txt");
    let (decision, _) = world
        .store
        .derive_and_commit_policy_decision(CASE, &denied.operation_id)
        .unwrap();
    assert_eq!(decision.outcome, DecisionOutcome::Deny);
    assert!(world
        .store
        .admit_resource_read_authorized(&world.owner, CASE, &denied.operation_id)
        .is_err());
    let operation = world.operation("request:read", "src/retry.txt");
    world
        .store
        .derive_and_commit_policy_decision(CASE, &operation.operation_id)
        .unwrap();
    let admission = world
        .store
        .admit_resource_read_authorized(&world.owner, CASE, &operation.operation_id)
        .unwrap();
    assert!(world
        .store
        .admit_resource_read_authorized(&world.outsider, CASE, &operation.operation_id)
        .is_err());
    assert_eq!(
        world
            .store
            .record_participant_resource_request(
                &world.owner,
                CASE,
                HUMAN,
                RESOURCE,
                "request:read",
                0,
                world.request("src/retry.txt")
            )
            .unwrap(),
        operation
    );
    assert!(world
        .store
        .record_participant_resource_request(
            &world.owner,
            CASE,
            HUMAN,
            RESOURCE,
            "request:read",
            0,
            world.request("protected/secret.txt")
        )
        .unwrap_err()
        .contains("identity_collision"));
    assert!(world
        .store
        .record_participant_resource_request(
            &world.owner,
            CASE,
            "participant:model",
            RESOURCE,
            "request:spoof",
            admission.case_generation,
            world.request("src/retry.txt")
        )
        .unwrap_err()
        .contains("principal_participant_link"));
    assert!(world
        .store
        .record_participant_resource_request(
            &world.owner,
            CASE,
            HUMAN,
            RESOURCE,
            "request:stale",
            0,
            world.request("src/retry.txt")
        )
        .unwrap_err()
        .contains("stale_case_generation"));
    let before = world.store.list_case_transitions(CASE).unwrap();
    world
        .store
        .revoke_tenant_policy_artifact(
            &world.owner,
            &world.artifact_id,
            "revoked between read and publication",
        )
        .unwrap();
    assert!(world
        .store
        .admit_resource_read_authorized(&world.owner, CASE, &operation.operation_id)
        .is_err());
    assert!(world
        .store
        .record_resource_observation_authorized(&world.owner, &admission, world.result(&admission))
        .is_err());
    assert_eq!(world.store.list_case_transitions(CASE).unwrap(), before);
    assert!(world.store.verify_case_state(CASE).unwrap());
    println!("resource_read_denials: protected_path=deny cross_tenant=deny principal_spoof=deny stale_generation=deny revoked_policy=deny observations=0");
    world.finish();
}

#[test]
fn resource_observation_commit_failure_is_atomic_and_cannot_be_replayed_twice() {
    let world = World::new();
    let operation = world.operation("request:atomic", "src/retry.txt");
    world
        .store
        .derive_and_commit_policy_decision(CASE, &operation.operation_id)
        .unwrap();
    let admission = world
        .store
        .admit_resource_read_authorized(&world.owner, CASE, &operation.operation_id)
        .unwrap();
    let observation = ResourceObservation::new(
        &admission.operation,
        &admission.decision,
        world.result(&admission),
        authority_wall_time_unix_ms(),
    )
    .unwrap();
    let mut pending = secured_pending(
        &format!("transition:{}", observation.observation_id),
        CASE,
        admission.case_generation,
        &world.owner.projected_principal_id(),
        TransitionPayload::ResourceObservationRecorded { observation },
    );
    pending.source.participant_id = Some(HUMAN.into());
    pending.scope = Some(operation.scope.clone());
    pending.causal_refs = vec![
        operation.operation_id.clone(),
        admission.decision.decision_id.clone(),
        RESOURCE.into(),
    ];
    let before = world.store.list_case_transitions(CASE).unwrap();
    let mut txn = world.store.env.begin_rw_txn().unwrap();
    let context = world
        .store
        .resolve_security_context_txn(&txn, &world.owner, TENANT)
        .unwrap();
    assert!(world
        .store
        .commit_transition_txn_at(&mut txn, pending.clone(), true, None, Some(&context))
        .unwrap_err()
        .contains("injected_failure"));
    drop(txn);
    assert_eq!(world.store.list_case_transitions(CASE).unwrap(), before);
    assert!(world
        .store
        .commit_transition(pending.clone())
        .unwrap_err()
        .contains("authenticated_resource_observation"));
    world
        .store
        .commit_secured_transition(&world.owner, TENANT, pending.clone(), false)
        .unwrap();
    pending.transition_id.push_str(":forged-duplicate");
    pending.expected_generation += 1;
    assert!(world
        .store
        .commit_secured_transition(&world.owner, TENANT, pending, false)
        .is_err());
    assert_eq!(
        world.store.list_case_transitions(CASE).unwrap().len(),
        before.len() + 1
    );
    assert!(world.store.verify_case_state(CASE).unwrap());
    world.finish();
}
