//! Product evidence invokes the Application dispatcher, never the CLI.
use serde_json::{json, Value};
use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};
use yai_application::{LocalApplication, OperationRequest, OperationResult, ResultState, APPLICATION_PROTOCOL};

struct Fixture { home: PathBuf, app: LocalApplication }

impl Fixture {
    fn new(label: &str) -> Self {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let home = std::env::temp_dir().join(format!("yai-application-{label}-{stamp}"));
        fs::create_dir_all(&home).unwrap();
        let f = Self { app: LocalApplication::from_yai_home(&home), home };
        f.success("identity.bootstrap", json!({"tenant_id":"tenant:audit", "organization_ref":"organization:cli-product"}));
        f.success("case.create", json!({"tenant_id":"tenant:audit", "case_ref":"case:audit"}));
        f.success("participant.role.add", json!({"case_ref":"case:audit", "participant_ref":"participant:operator", "role":"operator"}));
        f.success("participant.principal.link", json!({"case_ref":"case:audit", "participant_ref":"participant:operator", "principal_ref":"self"}));
        f.success("participant.view.admit", json!({"case_ref":"case:audit", "participant_ref":"participant:operator", "consumer":"model", "view_kind":"model_context"}));
        f
    }
    fn call(&self, op: &str, input: Value) -> OperationResult {
        self.app.call(OperationRequest { protocol: APPLICATION_PROTOCOL.into(), operation_ref: op.into(), correlation_ref: "test:operation-audit".into(), input })
    }
    fn success(&self, op: &str, input: Value) -> Value {
        let result = self.call(op, input);
        assert_eq!(result.result_state, ResultState::Success, "{op}: {result:?}");
        result.data.unwrap()
    }
    fn generation(&self) -> u64 {
        self.success("case.summary", json!({"case_ref":"case:audit"}))["case"]["generation"].as_u64().unwrap()
    }
}

impl Drop for Fixture { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.home); } }

#[test]
fn resource_effect_receipt_survives_lost_response_and_retry_without_redispatch() {
    resource_receipt_reconnect(16384, "applied", true, false, false);
}

#[test]
fn resource_failed_no_effect_is_not_reported_as_success_after_reconnect() {
    resource_receipt_reconnect(1024, "failed_no_effect", false, false, false);
}

#[test]
fn resource_review_reconnect_requires_real_approval_before_one_dispatch() {
    resource_receipt_reconnect(16384, "applied", true, true, false);
}

#[test]
fn resource_review_revocation_before_dispatch_refuses_current_observation_and_retry() {
    resource_receipt_reconnect(16384, "applied", true, true, true);
}

fn resource_receipt_reconnect(max_output_bytes: usize, outcome: &str, dispatched: bool, review: bool, revoke_pending: bool) {
    use yai_core_engine::effect::{digest_bytes, LocalFilesystemBinding};
    use yai_core_engine::effect::access::*;
    let mut f = Fixture::new("effect-reconnect");
    let root = f.home.join("effect-root");
    fs::create_dir_all(root.join("work")).unwrap();
    let executable = fs::canonicalize("/usr/bin/python3").unwrap();
    let runner = ProcessRunner {
        executable: executable.display().to_string(),
        executable_digest: digest_bytes(&fs::read(&executable).unwrap()),
        argv: vec!["-I".into(), "-B".into(), "-c".into(),
            "import sys; print('bounded verification executed'); sys.exit(7)".into()],
        working_directory: "work".into(), environment: Default::default(), timeout_ms: 2000,
    };
    let binding = LocalAccessBinding {
        schema: LOCAL_ACCESS_BINDING_SCHEMA.into(), case_id:"case:audit".into(), attachment_id:"runner".into(),
        address: ResourceAddress::ProcessRunner {
            root: LocalFilesystemBinding::new("case:audit", "runner", &root).unwrap(),
            runners: [("verify".into(), runner)].into_iter().collect(),
        },
    };
    let access = ResourceAccessContract {
        schema:RESOURCE_ACCESS_SCHEMA.into(), configuration_digest:binding.digest(),
        participant_ids:vec!["participant:operator".into()], operations:vec![AccessKind::ProcessRun],
        read_prefixes:vec![], names:vec!["verify".into()], max_output_bytes, max_items:8,
    };
    f.success("resource.attach", json!({"binding":binding,"access":access,
        "policy_owner_participant_ref":"participant:operator", "review_requirement":"automatic"}));
    let mut policy = json!({"schema":"yai.policy_source_input.v4","policy_key":"application-process",
        "source_version":"1","owner_ref":"organization:cli-product",
        "source_origin":{"source_system":"application-test","source_uri":"test://effect-reconnect"},
        "validity":{"mode":"unbounded"},"rules":[{"kind":"operation_restriction","rule_id":"run",
            "operation_kind":"process.run","resource_kind":"process_runner","effect":"allow","reason":"bounded test runner"}]});
    if review {
        f.success("participant.role.add", json!({"case_ref":"case:audit","participant_ref":"participant:operator","role":"policy-reviewer"}));
        policy["rules"].as_array_mut().unwrap().push(json!({"kind":"review_requirement","rule_id":"review",
            "operation_kind":"process.run","resource_kind":"process_runner","required":true,"reason":"explicit review before dispatch"}));
        policy["rules"].as_array_mut().unwrap().push(json!({"kind":"authority_requirement","rule_id":"reviewer-role",
            "operation_kind":"process.run","resource_kind":"process_runner","subject":"reviewer",
            "required_role":"policy-reviewer","reason":"explicit eligible reviewer"}));
    }
    let ingested = f.success("policy.ingest", json!({"tenant_id":"tenant:audit","source_bytes":serde_json::to_vec(&policy).unwrap()}));
    let artifact = ingested["view"]["artifact"]["artifact_id"].as_str().unwrap();
    for op in ["policy.validate", "policy.publish"] {
        f.success(op, json!({"artifact_ref":artifact,"reason":"bounded effect qualification"}));
    }
    f.success("policy.case.bind", json!({"case_ref":"case:audit","artifact_ref":artifact,
        "expected_generation":f.generation(),"reason":"bounded effect qualification"}));
    let request = json!({"case_ref":"case:audit","participant_ref":"participant:operator",
        "resource_ref":"runner","submission_ref":"request:lost-effect-response","expected_generation":f.generation(),
        "request":ResourceRequest {schema:RESOURCE_REQUEST_SCHEMA.into(), configuration_digest:binding.digest(),
            action:ResourceAction::ProcessRun {name:"verify".into()}}});
    let mut wrong = request.clone();
    wrong["participant_ref"] = json!("participant:hidden");
    assert_ne!(f.call("resource.request", wrong).result_state, ResultState::Success);
    let before = f.generation();
    // Discard the submission response: the reconnecting client only retains
    // its exact submission reference, not an IPC acknowledgement.
    let mut submitted = f.success("resource.request", request.clone());
    if review {
        assert_eq!(submitted["execution"]["posture"]["state"], "waiting_for_review", "{submitted:#}");
        let pending_generation = f.generation();
        let review_ref = submitted["execution"]["posture"]["review_ref"].clone();
        f.app = LocalApplication::from_yai_home(&f.home);
        let waiting = f.success("execution.get", json!({"case_ref":"case:audit","participant_ref":"participant:operator",
            "execution":{"domain":"resource_request","submission_ref":"request:lost-effect-response"}}));
        assert_eq!(waiting, submitted["execution"]);
        assert_eq!(f.success("resource.request", request.clone())["execution"], waiting);
        assert_eq!(f.generation(), pending_generation, "pending retry cannot grant authority or dispatch");
        let refused = f.call("review.approve", json!({"case_ref":"case:audit","review_ref":review_ref,
            "participant_ref":"participant:hidden","reason":"wrong reviewer"}));
        assert_ne!(refused.result_state, ResultState::Success);
        assert_eq!(f.generation(), pending_generation);
        if revoke_pending {
            f.success("policy.revoke", json!({"artifact_ref":artifact,"reason":"revoke before any dispatch"}));
            let revoked_generation = f.generation();
            f.app = LocalApplication::from_yai_home(&f.home);
            let observed = f.call("execution.get", json!({"case_ref":"case:audit","participant_ref":"participant:operator",
                "execution":{"domain":"resource_request","submission_ref":"request:lost-effect-response"}}));
            assert_eq!(observed.result_state, ResultState::Stale, "{observed:?}");
            assert!(observed.data.is_none());
            assert_eq!(f.call("resource.request", request).result_state, ResultState::Stale);
            assert_eq!(f.generation(), revoked_generation, "reconnect cannot approve, prepare or dispatch revoked work");
            let store = yai_core_engine::store::lmdb::LmdbRecordStore::open(f.home.join("store/lmdb")).unwrap();
            assert!(store.get_case_state("case:audit").unwrap().unwrap().effects.is_empty());
            return;
        }
        let approved = f.success("review.approve", json!({"case_ref":"case:audit","review_ref":review_ref,
            "participant_ref":"participant:operator","reason":"bounded process approved"}));
        assert_eq!(approved["external_effect"], false, "approval alone does not execute");
        submitted = f.success("resource.request", request.clone());
    }
    if dispatched {
        assert_eq!(submitted["outcome"]["observation"]["result"]["exit_code"], 7, "{submitted:#}");
    } else {
        assert_eq!(submitted["outcome"]["observation"]["result"]["reason"], "process_result_envelope_too_small");
    }
    assert!(f.generation() > before);
    let generation = f.generation();
    f.app = LocalApplication::from_yai_home(&f.home);
    let observation = f.success("execution.get", json!({"case_ref":"case:audit","participant_ref":"participant:operator",
        "execution":{"domain":"resource_request","submission_ref":"request:lost-effect-response"}}));
    assert_eq!(observation["posture"]["state"], "effect_recorded");
    assert_eq!(observation["posture"]["outcome"], outcome);
    assert_eq!(observation["posture"]["external_execution_started"], dispatched);
    assert!(observation["posture"]["receipt_ref"].as_str().is_some());
    let repeated = f.success("resource.request", request);
    assert_eq!(repeated["execution"], observation);
    assert!(repeated["outcome"].is_null());
    assert_eq!(f.generation(), generation);
    assert_eq!(repeated["execution"]["posture"]["receipt_ref"], submitted["execution"]["posture"]["receipt_ref"],
        "retry reuses the original exact receipt, without another PREPARE or dispatch");
    f.success("policy.revoke", json!({"artifact_ref":artifact,"reason":"current authority negative after completion"}));
    let revoked = f.call("execution.get", json!({"case_ref":"case:audit","participant_ref":"participant:operator",
        "execution":{"domain":"resource_request","submission_ref":"request:lost-effect-response"}}));
    assert_eq!(revoked.result_state, ResultState::Stale, "{revoked:?}");
    assert!(revoked.data.is_none(), "historical receipt is not current permission to disclose the result");
}

#[test]
fn policy_lifecycle_is_executed_and_stale_binding_does_not_mutate_case() {
    let f = Fixture::new("policy");
    let bytes = include_bytes!("../../../tests/fixtures/cli-product-policy.json").to_vec();
    let ingested = f.success("policy.ingest", json!({"tenant_id":"tenant:audit", "source_bytes":bytes}));
    let artifact = ingested["view"]["artifact"]["artifact_id"].as_str().expect("typed artifact identity");
    for op in ["policy.validate", "policy.publish"] {
        f.success(op, json!({"artifact_ref":artifact, "reason":"application lifecycle qualification"}));
    }
    let generation = f.generation();
    let stale = f.call("policy.case.bind", json!({"case_ref":"case:audit", "artifact_ref":artifact, "expected_generation":generation-1, "reason":"stale negative"}));
    assert_eq!(stale.result_state, ResultState::Stale, "{stale:?}");
    assert_eq!(f.generation(), generation);
    let bound = f.success("policy.case.bind", json!({"case_ref":"case:audit", "artifact_ref":artifact, "expected_generation":generation, "reason":"current application bind"}));
    assert_eq!(bound["changed"], true);
    let generation = f.generation();
    assert!(generation > 0);
    let reopened = LocalApplication::from_yai_home(&f.home).call(OperationRequest {
        protocol: APPLICATION_PROTOCOL.into(), operation_ref:"case.summary".into(), correlation_ref:"test:reopen".into(), input:json!({"case_ref":"case:audit"})
    });
    assert_eq!(reopened.result_state, ResultState::Success);
    assert_eq!(reopened.data.unwrap()["case"]["generation"], generation);
    let missing = f.call("policy.case.bind", json!({"case_ref":"case:missing", "artifact_ref":artifact, "expected_generation":1, "reason":"wrong Case"}));
    assert_ne!(missing.result_state, ResultState::Success);
}

#[test]
fn source_resume_after_backing_failure_retains_attempt_and_exact_retry() {
    use yai_core_engine::effect::LocalFilesystemBinding;
    use yai_core_engine::effect::access::*;
    let mut f = Fixture::new("source-real-resume");
    let root = f.home.join("source-root");
    fs::create_dir_all(&root).unwrap();
    let binding = LocalAccessBinding {
        schema: LOCAL_ACCESS_BINDING_SCHEMA.into(), case_id:"case:audit".into(), attachment_id:"bootstrap".into(),
        address:ResourceAddress::Discovery {root:LocalFilesystemBinding::new("case:audit", "bootstrap", &root).unwrap()},
    };
    let access = ResourceAccessContract {
        schema:RESOURCE_ACCESS_SCHEMA.into(), configuration_digest:binding.digest(),
        participant_ids:vec!["participant:operator".into()], operations:vec![AccessKind::Discover],
        read_prefixes:vec!["policy.json".into()], names:vec![], max_output_bytes:65536, max_items:8,
    };
    f.success("resource.attach", json!({"binding":binding,"access":access,
        "policy_owner_participant_ref":"participant:operator","review_requirement":"automatic"}));
    let declared = f.success("source.declare", json!({"case_ref":"case:audit","perimeter":"test",
        "logical_name":"bootstrap","participant_ref":"participant:operator","resource_ref":"bootstrap",
        "roles":["policy"],"action":ResourceAction::Discover {path:"policy.json".into()},
        "bootstrap_policy":true,"media_type":"application/json"}));
    let source = declared["state"]["sources"][0]["declaration"]["source_id"].as_str().unwrap();
    let acquire = json!({"case_ref":"case:audit","participant_ref":"participant:operator",
        "source_ref":source,"attempt":1,"expected_generation":f.generation()});
    // No authored progress: the real backing is absent when acquisition runs.
    let failed = f.success("source.acquire", acquire.clone());
    assert_eq!(failed["created"], true);
    assert!(matches!(failed["execution"]["phase"].as_str(), Some("inaccessible" | "needs_processing")), "{failed:#}");
    assert_eq!(failed["execution"]["posture"], "interrupted");
    let generation = f.generation();
    f.app = LocalApplication::from_yai_home(&f.home);
    let repeated = f.success("source.acquire", acquire.clone());
    assert_eq!(repeated["created"], false);
    assert_eq!(repeated["execution"], failed["execution"]);
    assert_eq!(f.generation(), generation);
    fs::write(root.join("policy.json"), include_bytes!("../../../tests/fixtures/cli-product-policy.json")).unwrap();
    let resume = json!({"case_ref":"case:audit","participant_ref":"participant:operator",
        "source_ref":source,"attempt":1,"expected_generation":generation,
        "previous_progress_ref":failed["execution"]["progress_ref"]});
    let mut wrong = resume.clone();
    wrong["participant_ref"] = json!("participant:hidden");
    assert_eq!(f.call("source.resume", wrong).result_state, ResultState::Unauthorized);
    let mut stale = resume.clone();
    stale["expected_generation"] = json!(0);
    assert_eq!(f.call("source.resume", stale).result_state, ResultState::Stale);
    assert_eq!(f.generation(), generation);
    let resumed = f.success("source.resume", resume.clone());
    assert_eq!(resumed["created"], true);
    assert_eq!(resumed["execution"]["phase"], "acquired");
    assert_eq!(resumed["execution"]["posture"], "completed");
    assert_eq!(resumed["execution"]["attempt"], 1);
    let settled_generation = f.generation();
    assert_eq!(settled_generation, generation + 2, "only resume admission and completion append progress");
    f.app = LocalApplication::from_yai_home(&f.home);
    let repeated = f.success("source.resume", resume);
    assert_eq!(repeated["created"], false);
    assert_eq!(repeated["execution"], resumed["execution"]);
    assert_eq!(f.generation(), settled_generation);
    let observed = f.success("execution.get", json!({"case_ref":"case:audit","participant_ref":"participant:operator",
        "execution":{"domain":"source_acquisition","source_ref":source,"attempt":1}}));
    assert_eq!(observed, resumed["execution"]);
    let retry_initial = f.success("source.acquire", acquire);
    assert_eq!(retry_initial["created"], false);
    assert_eq!(retry_initial["execution"], observed);
    assert_eq!(f.generation(), settled_generation, "lost initial acknowledgement cannot recreate the attempt");
}

#[test]
fn knowledge_queries_resolve_exact_current_sources_and_hide_revoked_units() {
    use yai_core_engine::effect::LocalFilesystemBinding;
    use yai_core_engine::effect::access::*;
    use yai_core_engine::memory_hierarchy::knowledge::KnowledgeRequest;
    let f = Fixture::new("knowledge-query");
    let root = f.home.join("source-root");
    fs::create_dir_all(&root).unwrap();
    let mut document: Value = serde_json::from_slice(include_bytes!("../../../tests/fixtures/cli-product-policy.json")).unwrap();
    document["rules"].as_array_mut().unwrap().push(json!({"kind":"operation_restriction","rule_id":"discovery",
        "operation_kind":"discovery.enumerate","resource_kind":"discovery","effect":"allow","reason":"documentary inspection"}));
    fs::write(root.join("handbook.md"), format!("# Filesystem handbook\nDocumentary filesystem evidence, not operational authority.\n```yai-policy-json\n{}\n```\n", serde_json::to_string(&document).unwrap())).unwrap();
    let binding = LocalAccessBinding {
        schema:LOCAL_ACCESS_BINDING_SCHEMA.into(), case_id:"case:audit".into(), attachment_id:"document".into(),
        address:ResourceAddress::Discovery {root:LocalFilesystemBinding::new("case:audit", "document", &root).unwrap()},
    };
    let access = ResourceAccessContract {
        schema:RESOURCE_ACCESS_SCHEMA.into(), configuration_digest:binding.digest(),
        participant_ids:vec!["participant:operator".into()], operations:vec![AccessKind::Discover],
        read_prefixes:vec!["handbook.md".into()], names:vec![], max_output_bytes:65536, max_items:8,
    };
    f.success("resource.attach", json!({"binding":binding,"access":access,
        "policy_owner_participant_ref":"participant:operator","review_requirement":"automatic"}));
    let declared = f.success("source.declare", json!({"case_ref":"case:audit","perimeter":"test","logical_name":"manual",
        "participant_ref":"participant:operator","resource_ref":"document","roles":["policy","knowledge"],
        "action":ResourceAction::Discover {path:"handbook.md".into()},"bootstrap_policy":true,"media_type":"text/markdown;profile=yai-mixed-v1"}));
    let source_ref = declared["state"]["sources"][0]["declaration"]["source_id"].as_str().unwrap();
    let acquired = f.success("source.acquire", json!({"case_ref":"case:audit","participant_ref":"participant:operator",
        "source_ref":source_ref,"attempt":1,"expected_generation":f.generation()}));
    assert_eq!(acquired["execution"]["phase"], "acquired");
    f.success("source.publish", json!({"case_ref":"case:audit","source_ref":source_ref,"reason":"publish exact source authority"}));
    let generation = f.generation();
    let request = KnowledgeRequest::new("case:audit");
    let view = f.success("knowledge.inspect", json!({"request":request}));
    assert!(!view["units"].as_array().unwrap().is_empty(), "{view:#}");
    assert!(view["relations"].is_array(), "graph comes from the qualified domain view");
    let found = f.success("knowledge.search", json!({"request":request,"query":"filesystem","limit":8}));
    let unit_ref = found["hits"][0]["document_id"].as_str().expect("fixture document contains filesystem evidence");
    let resolved = f.success("knowledge.resolve", json!({"request":request,"unit_ref":unit_ref}));
    assert_eq!(resolved["unit"]["id"], unit_ref);
    assert_eq!(resolved["view"], view);
    assert!(view["sources"].as_array().unwrap().iter().any(|source| source["id"] == resolved["unit"]["source"]));
    let navigation = f.success("knowledge.navigation", json!({"request":request}));
    assert_eq!(navigation["view_ref"], view["id"]);
    assert!(navigation["navigation"].as_str().unwrap().contains("Case knowledge"));
    assert_eq!(f.generation(), generation, "Knowledge queries are derived, not canonical mutations");
    let invalid = f.call("knowledge.search", json!({"request":request,"query":"filesystem","limit":0}));
    assert_ne!(invalid.result_state, ResultState::Success);
    let wrong = f.call("knowledge.inspect", json!({"request":KnowledgeRequest::new("case:absent")}));
    assert_eq!(wrong.result_state, ResultState::Unauthorized);
    f.success("source.revoke", json!({"case_ref":"case:audit","source_ref":source_ref,"reason":"current disclosure test"}));
    let hidden = f.call("knowledge.resolve", json!({"request":request,"unit_ref":unit_ref}));
    let absent = f.call("knowledge.resolve", json!({"request":request,"unit_ref":"knowledge-unit:absent"}));
    assert_ne!(hidden.result_state, ResultState::Success);
    assert_eq!(hidden.result_state, absent.result_state);
    assert_eq!(hidden.error.unwrap().code, absent.error.unwrap().code);
    assert!(hidden.data.is_none());
    let found = f.success("knowledge.search", json!({"request":request,"query":"filesystem","limit":8}));
    assert!(found["hits"].as_array().unwrap().is_empty());
    assert!(found["view"]["units"].as_array().unwrap().is_empty());
}

#[test]
fn recall_and_working_state_are_real_qualified_results_and_reject_stale_inputs() {
    use yai_core_engine::memory_hierarchy::recall::RecallRequest;
    use yai_core_engine::semantic_state::{CompilationRequest, SemanticPurpose, SemanticScope, WorkingStateRequest};
    let f = Fixture::new("semantic");
    let generation = f.generation();
    let recall = RecallRequest::integrated("case:audit", generation, "participant:operator", "current Case control");
    let result = f.success("semantic.recall", json!({"request":recall}));
    assert!(result.is_object());
    let w = WorkingStateRequest {
        case_id:"case:audit".into(), expected_generation:generation,
        compilation:CompilationRequest {
            scope:SemanticScope::model("participant:operator", SemanticPurpose::Inspection),
            intent:"inspect current Case control".into(),
            output_contract_id:yai_core_engine::context::InvocationOutputContract::NaturalLanguage.contract_id(),
            max_semantic_units:131072, max_derived_items:8, resource_refs:vec![],
            required_refs:vec!["participant:operator".into()], previous_item_ids:vec![], view_selection_id:None,
        },
        recall_query:None, at:None, recall_required_refs:vec![], recall_bounds:Default::default(), max_output_bytes:1024*1024,
    };
    let compiled = f.success("semantic.working_state.compile", json!({"request":w}));
    assert_eq!(compiled["working_state"]["case_id"], "case:audit");
    assert_eq!(compiled["working_state"]["case_generation"], generation);
    assert!(compiled["working_state"]["working_state_id"].as_str().unwrap().starts_with("working-state:"));
    f.success("participant.role.add", json!({"case_ref":"case:audit", "participant_ref":"participant:operator", "role":"reviewer"}));
    let stale = f.call("semantic.working_state.compile", json!({"request":w}));
    assert_eq!(stale.result_state, ResultState::Stale, "{stale:?}");
    let mut hidden = recall;
    hidden.expected_generation = f.generation();
    hidden.participant_id = "participant:missing".into();
    let hidden = f.call("semantic.recall", json!({"request":hidden}));
    assert_ne!(hidden.result_state, ResultState::Success);
    assert!(hidden.data.is_none());
}

#[test]
fn lost_runtime_acknowledgement_is_observable_after_stop_and_reopen_without_redispatch() {
    use yai_core_engine::effect::LocalFilesystemBinding;
    use yai_core_engine::security::AuthenticatedPrincipal;
    use yai_core_engine::store::lmdb::{LmdbRecordStore, RuntimeCaseBudgets, RuntimeInstanceAcquireRequest, RuntimeInstanceConfig};
    use yai_application::CaseRunInput;
    use yai_core_engine::transition::{PendingTransition, ResourceAttachmentState, ResourceKind, ReviewRequirement, TransitionPayload, TransitionSource};
    let mut f = Fixture::new("execution-observation");
    let auth = AuthenticatedPrincipal::authenticate_local().unwrap();
    let store = LmdbRecordStore::open(f.home.join("store/lmdb")).unwrap();
    let root = f.home.join("workspace");
    fs::create_dir_all(&root).unwrap();
    let attachment = ResourceAttachmentState {
        attachment_id:"workspace".into(), kind:ResourceKind::Filesystem,
        allowed_write_prefix:"allowed".into(), max_write_bytes:1024,
        policy_id:"policy:envelope".into(), policy_owner_participant_id:"participant:operator".into(),
        review_requirement:ReviewRequirement::RequireReview, process_signal_actions:vec![], access:None,
    };
    let mut pending = PendingTransition::new("transition:runtime-test-resource", "case:audit", f.generation(),
        TransitionSource { component:"application-test".into(), participant_id:None, principal_id:Some(auth.projected_principal_id()), source_ref:None },
        TransitionPayload::ResourceAttached { attachment });
    pending.causal_refs = vec!["participant:operator".into()];
    store.commit_tenant_resource_attachment(&auth, "tenant:audit", pending,
        &LocalFilesystemBinding::new("case:audit", "workspace", &root).unwrap()).unwrap();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
    store.acquire_runtime_instance(&auth, &RuntimeInstanceAcquireRequest {
        owner_pid:std::process::id(), owner_token:"test:runtime-observation".into(), now_unix_ms:now, lease_duration_ms:30000,
        config:RuntimeInstanceConfig { workers:1, max_active_per_tenant:1, max_queued_per_tenant:4, max_queued_total:4 },
    }, false).unwrap();
    store.activate_runtime_instance(&auth, "test:runtime-observation", now, 30000, 0).unwrap();
    let input = json!({"case_ref":"case:audit", "participant_ref":"participant:operator", "execution":{"domain":"runtime_work", "submission_ref":"request:lost-response"}});
    let absent = f.call("execution.get", input.clone());
    assert_eq!(absent.result_state, ResultState::Unauthorized);
    let generation = f.generation();
    let submission = CaseRunInput {
        submission_ref:"request:lost-response".into(), case_ref:"case:audit".into(), participant_ref:"participant:operator".into(), resource_ref:"workspace".into(),
        task:"bounded test work".into(),
        budgets:RuntimeCaseBudgets { max_invocations:1, max_operations:1, max_semantic_units:1024, max_resident_items:16, max_estimated_input_units:4096, max_provider_retries:0, max_runtime_ms:Some(1000), stop_on_deny:true, continue_after_malformed:false },
    };
    // The domain commit survives even when the client loses its acknowledgement.
    let lost = f.success("case.run", json!(submission));
    assert_eq!(lost["created"], true);
    let repeated = f.success("case.run", json!(submission));
    assert_eq!(repeated["created"], false);
    let visible = f.success("execution.get", input.clone());
    assert_eq!(visible["execution_ref"], repeated["execution"]["execution_ref"]);
    assert_eq!(visible["state"], "queued");
    assert_eq!(visible["attempt_count"], 0);
    let mut conflict = submission.clone();
    conflict.task = "different work".into();
    assert_eq!(f.call("case.run", json!(conflict)).error.unwrap().code, "runtime_work_idempotency_conflict");
    store.stop_runtime_instance(&auth, "test:runtime-observation", now + 1).unwrap();
    drop(store);
    f.app = LocalApplication::from_yai_home(&f.home);
    assert_eq!(f.success("execution.get", input.clone()), visible);
    let recovered_ack = f.success("case.run", json!(submission));
    assert_eq!(recovered_ack["created"], false);
    assert_eq!(recovered_ack["execution"], visible);
    let mut new_submission = submission.clone();
    new_submission.submission_ref = "request:new-while-stopped".into();
    assert_eq!(f.call("case.run", json!(new_submission)).result_state, ResultState::CorePending);
    assert_eq!(f.call("case.run", json!(conflict)).error.unwrap().code, "runtime_work_idempotency_conflict");
    let mut wrong_submission = submission.clone();
    wrong_submission.participant_ref = "participant:other".into();
    assert_ne!(f.call("case.run", json!(wrong_submission)).result_state, ResultState::Success);
    assert_eq!(f.generation(), generation, "submission and observation create no Case Transition");
    for field in ["owner_token", "runtime_owner_token", "journal_path", "task"] { assert!(visible.get(field).is_none()); }
    let mut wrong = input.clone();
    wrong["participant_ref"] = json!("participant:other");
    let wrong = f.call("execution.get", wrong);
    assert_eq!(wrong.result_state, ResultState::Unauthorized);
    assert_eq!(wrong.error.unwrap().code, absent.error.unwrap().code);
    let mut other_case = input;
    other_case["case_ref"] = json!("case:other");
    assert_eq!(f.call("execution.get", other_case).result_state, ResultState::Unauthorized);
    let store = LmdbRecordStore::open(f.home.join("store/lmdb")).unwrap();
    let work = store.list_runtime_work_authorized(&auth).unwrap();
    assert_eq!(work.len(), 1);
    assert_eq!(work[0].attempt_count, 0);
    // A runner checkpoint fixture qualifies the Application control contract,
    // not actual scheduler dispatch (covered separately by the Host lane).
    use yai_application::runtime_execution::{CaseRuntimeCheckpoint, checkpoint_path_for, write_checkpoint_at, read_checkpoint_at};
    let checkpoint: CaseRuntimeCheckpoint = serde_json::from_value(json!({
        "schema":"yai.case_runtime_checkpoint.v3", "run_id":"run:application-stop",
        "runtime_instance_id":work[0].runtime_instance_id,"work_item_id":work[0].work_id,
        "case_id":"case:audit","participant_id":"participant:operator","attachment_id":"workspace",
        "journal_path":"private-journal","task":"private-task","status":"running","stop_detail":"",
        "stop_requested":false,"invocations":0,"operations":0,"provider_failures":0,
        "cumulative_estimated_input_units":0,"cumulative_provider_latency_ms":0,
        "max_invocations":1,"max_operations":1,"max_semantic_units":1024,"max_resident_items":16,
        "max_cumulative_estimated_input_units":4096,"max_provider_retries":0,
        "stop_on_deny":true,"continue_after_malformed":false,"previous_item_ids":[],
        "last_projection_selected_items":0,"last_projection_omitted_items":0,"last_semantic_units":0
    })).unwrap();
    let path = checkpoint_path_for(&f.home, "case:audit");
    write_checkpoint_at(&path, &checkpoint).unwrap();
    let stop = json!({"case_ref":"case:audit","participant_ref":"participant:operator",
        "submission_ref":submission.submission_ref,"run_ref":checkpoint.run_id});
    let mut stale = stop.clone();
    stale["run_ref"] = json!("run:other");
    assert_eq!(f.call("case.stop", stale).result_state, ResultState::Stale);
    let mut hidden = stop.clone();
    hidden["participant_ref"] = json!("participant:hidden");
    assert_eq!(f.call("case.stop", hidden).result_state, ResultState::Unauthorized);
    assert!(!read_checkpoint_at(&path, "case:audit").unwrap().stop_requested);
    let stopped = f.success("case.stop", stop.clone());
    assert_eq!(stopped["runner"]["run_ref"], "run:application-stop");
    assert_eq!(stopped["runner"]["stop_requested"], true);
    f.app = LocalApplication::from_yai_home(&f.home);
    assert_eq!(f.success("case.stop", stop), stopped);
    assert_eq!(f.generation(), generation, "stop is not Case cancellation or a Transition");
    assert_eq!(store.list_runtime_work_authorized(&auth).unwrap()[0].attempt_count, 0);
}

#[test]
fn source_attempt_observation_retains_exact_history_and_current_revoke_after_reopen() {
    use yai_core_engine::effect::LocalFilesystemBinding;
    use yai_core_engine::effect::access::*;
    use yai_core_engine::effect::access::source::*;
    use yai_core_engine::security::AuthenticatedPrincipal;
    use yai_core_engine::store::lmdb::LmdbRecordStore;
    let mut f = Fixture::new("source-attempt-observation");
    let root = f.home.join("source-root");
    fs::create_dir_all(root.join("docs")).unwrap();
    let binding = LocalAccessBinding {
        schema: LOCAL_ACCESS_BINDING_SCHEMA.into(), case_id: "case:audit".into(),
        attachment_id: "documents".into(),
        address: ResourceAddress::Discovery {
            root: LocalFilesystemBinding::new("case:audit", "documents", &root).unwrap(),
        },
    };
    let access = ResourceAccessContract {
        schema: RESOURCE_ACCESS_SCHEMA.into(), configuration_digest: binding.digest(),
        participant_ids: vec!["participant:operator".into()],
        operations: vec![AccessKind::Discover], read_prefixes: vec!["docs".into()],
        names: vec![], max_output_bytes: 1024, max_items: 8,
    };
    f.success("resource.attach", json!({"binding":binding,"access":access,
        "policy_owner_participant_ref":"participant:operator", "review_requirement":"require_review"}));
    f.success("source.declare", json!({"case_ref":"case:audit", "perimeter":"test",
        "logical_name":"manual", "participant_ref":"participant:operator", "resource_ref":"documents",
        "roles":["knowledge"], "action":ResourceAction::Discover { path:"docs".into() }, "media_type":"text/plain"}));
    let auth = AuthenticatedPrincipal::authenticate_local().unwrap();
    let store = LmdbRecordStore::open(f.home.join("store/lmdb")).unwrap();
    assert_eq!(store.get_case_state("case:audit").unwrap().unwrap().resources[0].review_requirement,
        yai_core_engine::transition::ReviewRequirement::RequireReview);
    let (_, source) = store.case_source_authorized(&auth, "case:audit", "manual").unwrap();
    let source_id = source.declaration.source_id.clone();
    let input = json!({"case_ref":"case:audit", "participant_ref":"participant:operator",
        "execution":{"domain":"source_acquisition","source_ref":source_id,"attempt":1}});
    assert_eq!(f.call("execution.get", input.clone()).result_state, ResultState::Unauthorized);
    let ingested = f.success("policy.ingest", json!({"tenant_id":"tenant:audit",
        "source_bytes":include_bytes!("../../../tests/fixtures/cli-product-policy.json").to_vec()}));
    let artifact = ingested["view"]["artifact"]["artifact_id"].as_str().unwrap();
    f.success("policy.validate", json!({"artifact_ref":artifact,"reason":"source refusal qualification"}));
    f.success("policy.publish", json!({"artifact_ref":artifact,"reason":"source refusal qualification"}));
    f.success("policy.case.bind", json!({"case_ref":"case:audit","artifact_ref":artifact,
        "expected_generation":f.generation(),"reason":"source refusal qualification"}));
    let resource_submission = json!({"case_ref":"case:audit", "participant_ref":"participant:operator",
        "resource_ref":"documents", "submission_ref":"request:application-resource",
        "expected_generation":f.generation(), "request":ResourceRequest {
            schema: RESOURCE_REQUEST_SCHEMA.into(), configuration_digest:binding.digest(),
            action:ResourceAction::Discover { path:"docs".into() },
        }});
    let resource_result = f.success("resource.request", resource_submission.clone());
    assert_eq!(resource_result["execution"]["posture"]["state"], "refused");
    let resource_generation = f.generation();
    let resource_observe = json!({"case_ref":"case:audit","participant_ref":"participant:operator",
        "execution":{"domain":"resource_request","submission_ref":"request:application-resource"}});
    let observed = f.success("execution.get", resource_observe.clone());
    assert_eq!(observed["operation_ref"], resource_result["execution"]["operation_ref"]);
    let mut wrong_observe = resource_observe;
    wrong_observe["participant_ref"] = json!("participant:hidden");
    assert_eq!(f.call("execution.get", wrong_observe).result_state, ResultState::Unauthorized);
    let duplicate = f.success("resource.request", resource_submission);
    assert_eq!(duplicate["execution"]["operation_ref"], observed["operation_ref"]);
    assert_eq!(f.generation(), resource_generation, "duplicate refused submission cannot redispatch or append history");
    // Use the same acquisition orchestration as the product CLI. Observation
    // must describe a real attempted acquisition, not authored progress.
    let submitted_generation = f.generation();
    let barrier = std::sync::Barrier::new(2);
    let claims = std::thread::scope(|scope| {
        let submit = || {
            barrier.wait();
            store.begin_source_attempt_authorized(
                &auth, "case:audit", "participant:operator", &source_id, 1, submitted_generation,
            ).unwrap()
        };
        let first = scope.spawn(submit);
        let second = scope.spawn(submit);
        [first.join().unwrap(), second.join().unwrap()]
    });
    assert_eq!(claims.iter().filter(|created| **created).count(), 1,
        "only one concurrent client may advance the newly admitted attempt");
    let admitted_generation = f.generation();
    assert_eq!(admitted_generation, submitted_generation + 1);
    assert!(!store.begin_source_attempt_authorized(
        &auth, "case:audit", "participant:operator", &source_id, 1, submitted_generation,
    ).unwrap(), "lost acknowledgement cannot admit a second dispatcher");
    assert_eq!(f.generation(), admitted_generation);
    assert!(store.begin_source_attempt_authorized(
        &auth, "case:audit", "participant:operator", &source_id, 2, admitted_generation,
    ).is_err(), "pending acquisition must not be bypassed with a new attempt");
    assert_eq!(yai_application::resource_execution::source::acquire(
        &f.home, &store, &auth, "case:audit", &source_id, false,
    ).unwrap_err(), "source_attempt_requires_observation_or_resume",
        "another CLI client cannot take over the admitted attempt");
    assert_eq!(f.generation(), admitted_generation);
    yai_application::resource_execution::source::advance_admitted(
        &f.home, &store, &auth, "case:audit", &source_id, 1,
    ).unwrap();
    let (_, acquired) = store.case_source_authorized(&auth, "case:audit", &source_id).unwrap();
    let first = acquired.progress.unwrap();
    assert_eq!(first.attempt, 1);
    assert_eq!(first.phase, SourcePhase::Denied);
    assert!(first.decision_ref.is_some(), "refusal cites the canonical policy Decision");
    assert!(first.revision.is_none(), "denied acquisition cannot invent retained material");
    let completed_generation = f.generation();
    assert!(!store.begin_source_attempt_authorized(
        &auth, "case:audit", "participant:operator", &source_id, 1, submitted_generation,
    ).unwrap(), "terminal refusal is observed, never retried implicitly");
    assert_eq!(f.generation(), completed_generation);
    f.success("source.declare", json!({"case_ref":"case:audit", "perimeter":"test",
        "logical_name":"application-manual", "participant_ref":"participant:operator", "resource_ref":"documents",
        "roles":["knowledge"], "action":ResourceAction::Discover { path:"docs".into() }, "media_type":"text/plain"}));
    let (_, app_source) = store.case_source_authorized(&auth, "case:audit", "application-manual").unwrap();
    let submission = json!({"case_ref":"case:audit", "participant_ref":"participant:operator",
        "source_ref":app_source.declaration.source_id,"attempt":1,"expected_generation":f.generation()});
    let mut stale = submission.clone();
    stale["expected_generation"] = json!(0);
    assert_eq!(f.call("source.acquire", stale).result_state, ResultState::Stale);
    let mut wrong = submission.clone();
    wrong["participant_ref"] = json!("participant:hidden");
    assert_eq!(f.call("source.acquire", wrong).result_state, ResultState::Unauthorized);
    let submitted = f.success("source.acquire", submission.clone());
    assert_eq!(submitted["created"], true);
    assert_eq!(submitted["execution"]["phase"], "denied");
    assert_eq!(submitted["execution"]["posture"], "refused");
    assert_eq!(submitted["advancement"], "returned");
    let retry_generation = f.generation();
    // Simulate response loss and a newly attached application client.
    f.app = LocalApplication::from_yai_home(&f.home);
    let retry = f.success("source.acquire", submission);
    assert_eq!(retry["created"], false);
    assert_eq!(retry["advancement"], "existing_attempt");
    assert_eq!(retry["execution"], submitted["execution"]);
    assert_eq!(f.generation(), retry_generation, "retry appends no second attempt or Decision");
    let observed = f.success("execution.get", input.clone());
    assert_eq!(observed["progress_ref"], first.progress_id);
    assert_eq!(observed["phase"], "denied");
    let second = SourceProgress { schema:SOURCE_PROGRESS_SCHEMA.into(), progress_id:String::new(),
        source_id:source_id.clone(), previous_progress_id:Some(first.progress_id), attempt:2,
        phase:SourcePhase::Acquiring, revision:None, decision_ref:None, detail:"second exact attempt".into()
    }.seal().unwrap();
    store.progress_case_source(&auth, "case:audit", &source_id, second.clone()).unwrap();
    let older = f.success("execution.get", input.clone());
    assert_eq!(older["phase"], "denied");
    assert_eq!(older["current_source_phase"], "acquiring");
    let blocked_resume = json!({"case_ref":"case:audit","participant_ref":"participant:operator",
        "source_ref":source_id,"attempt":2,"expected_generation":f.generation(),
        "previous_progress_ref":second.progress_id});
    let before_resume = f.generation();
    assert_ne!(f.call("source.resume", blocked_resume).result_state, ResultState::Success,
        "an unresolved carrier cannot be assumed dead from client disconnection");
    assert_eq!(f.generation(), before_resume);
    // A settled interruption fixture is distinct from crash-recovery evidence.
    let interrupted = SourceProgress { schema:SOURCE_PROGRESS_SCHEMA.into(), progress_id:String::new(),
        source_id:source_id.clone(), previous_progress_id:Some(second.progress_id), attempt:2,
        phase:SourcePhase::Inaccessible, revision:None, decision_ref:None,
        detail:"settled interruption fixture".into() }.seal().unwrap();
    store.progress_case_source(&auth, "case:audit", &source_id, interrupted.clone()).unwrap();
    let resume = json!({"case_ref":"case:audit","participant_ref":"participant:operator",
        "source_ref":source_id,"attempt":2,"expected_generation":f.generation(),
        "previous_progress_ref":interrupted.progress_id});
    let resumed = f.success("source.resume", resume.clone());
    assert_eq!(resumed["created"], true);
    assert_eq!(resumed["execution"]["attempt"], 2);
    assert_eq!(resumed["execution"]["phase"], "denied");
    let resumed_generation = f.generation();
    let repeated = f.success("source.resume", resume);
    assert_eq!(repeated["created"], false);
    assert_eq!(repeated["execution"], resumed["execution"]);
    assert_eq!(f.generation(), resumed_generation);
    f.success("source.revoke", json!({"case_ref":"case:audit","source_ref":source_id,"reason":"test revoke"}));
    drop(store);
    f.app = LocalApplication::from_yai_home(&f.home);
    let generation = f.generation();
    let revoked = f.success("execution.get", input.clone());
    assert_eq!(revoked["phase"], "denied");
    assert_eq!(revoked["current_source_phase"], "revoked");
    assert!(revoked.get("detail").is_none());
    for wrong in [
        json!({"case_ref":"case:other", "participant_ref":"participant:operator", "execution":input["execution"]}),
        json!({"case_ref":"case:audit", "participant_ref":"participant:other", "execution":input["execution"]}),
        json!({"case_ref":"case:audit", "participant_ref":"participant:operator", "execution":{"domain":"source_acquisition","source_ref":source_id,"attempt":99}}),
    ] {
        let result = f.call("execution.get", wrong);
        assert_eq!(result.result_state, ResultState::Unauthorized);
        assert!(result.data.is_none());
    }
    assert_eq!(f.generation(), generation, "observing cannot progress or reacquire a source");
}
