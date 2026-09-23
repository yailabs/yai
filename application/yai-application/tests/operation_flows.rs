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
fn provider_suitability_records_honest_attestation_and_refuses_invalid_input() {
    use yai_core_engine::{security::AuthenticatedPrincipal, store::lmdb::LmdbRecordStore};
    let f = Fixture::new("provider-suitability");
    let target = f.success("provider.register", json!({"tenant_id":"tenant:audit",
        "provider_key":"attestation", "adapter":"open_ai_compatible", "endpoint":"http://127.0.0.1:1/v1/chat/completions",
        "model_id":"test-only", "credential_ref":"none", "locality":"loopback"}));
    let input = json!({"target_ref":target["target_id"], "capability":"primary_conversation",
        "suite_ref":"suite:operator", "run_ref":"run:attestation", "evidence_refs":["evidence:operator"]});
    let mut missing = input.clone(); missing["target_ref"] = json!("provider-target:absent");
    assert_ne!(f.call("provider.suitability.record", missing).result_state, ResultState::Success);
    let mut forged = input.clone(); forged["principal_ref"] = json!("principal:forged");
    assert_ne!(f.call("provider.suitability.record", forged).result_state, ResultState::Success);
    let mut empty = input.clone(); empty["evidence_refs"] = json!([]);
    assert_ne!(f.call("provider.suitability.record", empty).result_state, ResultState::Success);
    let auth = AuthenticatedPrincipal::authenticate_local().unwrap();
    let store = LmdbRecordStore::open(f.home.join("store/lmdb")).unwrap();
    let target_id = target["target_id"].as_str().unwrap();
    assert!(store.list_semantic_suitability_evidence_authorized(&auth, target_id, None).unwrap().is_empty());
    let generation = f.generation();
    let evidence = f.success("provider.suitability.record", input);
    assert_eq!(evidence["posture"], "operator_attested");
    assert_eq!(evidence["recorded_by_principal_id"], auth.projected_principal_id());
    drop(store);
    let reopened = LmdbRecordStore::open(f.home.join("store/lmdb")).unwrap();
    let retained = reopened.list_semantic_suitability_evidence_authorized(&auth, target_id, None).unwrap();
    assert_eq!(retained.len(), 1);
    assert_eq!(serde_json::to_value(&retained[0]).unwrap(), evidence);
    assert_eq!(f.generation(), generation, "attestation does not execute provider or mutate Case history");
}

#[test]
fn cognitive_composition_admits_once_and_observes_existing_turn_after_reopen() {
    use yai_core_engine::conversation::*;
    use yai_core_engine::security::AuthenticatedPrincipal;
    use yai_core_engine::store::lmdb::LmdbRecordStore;
    let mut f = Fixture::new("composition-submit");
    let auth = AuthenticatedPrincipal::authenticate_local().unwrap();
    let store = LmdbRecordStore::open(f.home.join("store/lmdb")).unwrap();
    let content = ConversationContentStore::open(&f.home).unwrap();
    let mut draft = ConversationDraft { schema: CONVERSATION_DRAFT_SCHEMA.into(),
        draft_id: "composition-draft".into(), case_id: "case:audit".into(), tenant_id: "tenant:audit".into(),
        thread_id: "thread:application".into(), participant_id: "participant:operator".into(),
        principal_id: auth.projected_principal_id(), base_generation: f.generation(), parts: vec![] };
    content.create_draft(&draft).unwrap();
    content.stage_bytes(&mut draft, ContentModality::Text, "text/plain;charset=utf-8", b"exact existing Turn",
        ContentPartProvenance::Original { imported_by_principal_id: auth.projected_principal_id() }).unwrap();
    let committed = yai_application::cognitive_execution::commit_conversation_draft_with_intent(
        &f.home, &auth, &store, &draft, None, None).unwrap();
    let generation = f.generation();
    let input = json!({"case_ref":"case:audit", "participant_ref":"participant:operator",
        "source_turn_ref":committed.turn.turn_id, "source_part_refs":[committed.turn.ordered_parts[0].part_id],
        "prerequisite":null, "expected_generation":generation});
    let mut hidden = input.clone(); hidden["participant_ref"] = json!("participant:hidden");
    assert_ne!(f.call("cognitive.compose", hidden).result_state, ResultState::Success);
    let mut stale = input.clone(); stale["expected_generation"] = json!(generation - 1);
    assert_eq!(f.call("cognitive.compose", stale).result_state, ResultState::Stale);
    assert_eq!(f.generation(), generation);
    let first = f.success("cognitive.compose", input.clone());
    assert_eq!(first["created"], true);
    let observe = json!({"case_ref":"case:audit", "participant_ref":"participant:operator",
        "execution":{"domain":"cognitive_composition", "request_ref":first["execution"]["request_ref"]}});
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        let result = f.success("execution.get", observe.clone());
        if result["posture"] != "running" { assert_eq!(result["posture"], "unresolved"); break }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    f.app = LocalApplication::from_yai_home(&f.home);
    let retry = f.success("cognitive.compose", input);
    assert_eq!(retry["created"], false);
    assert_eq!(retry["execution"]["request_ref"], first["execution"]["request_ref"]);
    let mut wrong_case = observe.clone(); wrong_case["case_ref"] = json!("case:absent");
    assert_ne!(f.call("execution.get", wrong_case).result_state, ResultState::Success);
    let mut hidden = observe; hidden["participant_ref"] = json!("participant:hidden");
    assert_ne!(f.call("execution.get", hidden).result_state, ResultState::Success);
    let history = store.list_case_transitions("case:audit").unwrap();
    assert_eq!(history.iter().filter(|t| matches!(t.payload, yai_core_engine::transition::TransitionPayload::ConversationExecutionIntentRecorded { .. })).count(), 1);
    assert!(!history.iter().any(|t| matches!(t.payload, yai_core_engine::transition::TransitionPayload::ProviderInvocationStarted { .. })));
    assert_eq!(f.generation(), generation + 1);
}

#[test]
fn conversation_submission_is_durable_idempotent_and_observable_without_provider() {
    let mut f = Fixture::new("conversation-submission");
    let generation = f.generation();
    let input = json!({"case_ref":"case:audit", "participant_ref":"participant:operator",
        "thread_ref":"thread:application", "submission_ref":"send:once", "expected_generation":generation,
        "parts":[{"modality":"text", "media_type":"text/plain;charset=utf-8", "bytes":b"bounded conversation".to_vec()}]});
    let mut hidden = input.clone(); hidden["participant_ref"] = json!("participant:hidden");
    assert_ne!(f.call("conversation.send", hidden).result_state, ResultState::Success);
    let mut stale = input.clone(); stale["expected_generation"] = json!(generation - 1);
    assert_eq!(f.call("conversation.send", stale).result_state, ResultState::Stale);
    assert_eq!(f.generation(), generation);
    let acknowledged = f.success("conversation.send", input.clone());
    assert_eq!(acknowledged["created"], true);
    assert_eq!(acknowledged["execution"]["posture"], "admitted");
    let observation = json!({"case_ref":"case:audit", "participant_ref":"participant:operator",
        "execution":{"domain":"conversation", "submission_ref":"send:once"}});
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        let observed = f.success("execution.get", observation.clone());
        if observed["posture"] != "running" { assert_eq!(observed["posture"], "unresolved"); break }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    f.app = LocalApplication::from_yai_home(&f.home);
    let summary = f.success("case.summary", json!({"case_ref":"case:audit"}));
    assert_eq!(summary["conversation"]["turns"][0]["execution_request_ref"], acknowledged["execution"]["request_ref"]);
    assert_eq!(summary["conversation"]["turns"][0]["id"], acknowledged["execution"]["turn_ref"]);
    let retry = f.success("conversation.send", input.clone());
    assert_eq!(retry["created"], false);
    assert_eq!(retry["execution"]["turn_ref"], acknowledged["execution"]["turn_ref"]);
    assert_eq!(retry["execution"]["request_ref"], acknowledged["execution"]["request_ref"]);
    let mut conflict = input; conflict["parts"][0]["bytes"] = json!(b"different".to_vec());
    assert_eq!(f.call("conversation.send", conflict).error.unwrap().code, "conversation_submission_idempotency_conflict");
    let mut hidden = observation; hidden["participant_ref"] = json!("participant:hidden");
    assert_ne!(f.call("execution.get", hidden).result_state, ResultState::Success);
    let store = yai_core_engine::store::lmdb::LmdbRecordStore::open(f.home.join("store/lmdb")).unwrap();
    let history = store.list_case_transitions("case:audit").unwrap();
    assert_eq!(history.iter().filter(|t| matches!(t.payload, yai_core_engine::transition::TransitionPayload::ConversationTurnCommitted { .. })).count(), 1);
    assert_eq!(history.iter().filter(|t| matches!(t.payload, yai_core_engine::transition::TransitionPayload::ConversationExecutionIntentRecorded { .. })).count(), 1);
    assert!(!history.iter().any(|t| matches!(t.payload, yai_core_engine::transition::TransitionPayload::ProviderInvocationStarted { .. })));
    assert_eq!(f.generation(), generation + 2);
}

#[test]
fn canonical_filesystem_effect_application_retry_observes_receipt_without_second_write() {
    canonical_effect_retry(false, false, false, false);
}

#[test]
fn canonical_filesystem_prepare_without_receipt_is_observed_never_redispatched() {
    canonical_effect_retry(true, false, false, false);
}

#[test]
fn canonical_filesystem_policy_revoked_before_submission_cannot_dispatch() {
    canonical_effect_retry(false, true, false, false);
}

#[test]
fn canonical_process_effect_application_retry_observes_receipt_without_second_signal() {
    canonical_effect_retry(false, false, true, false);
}

#[test]
fn canonical_process_prepare_without_receipt_is_observed_never_redispatched() {
    canonical_effect_retry(true, false, true, true);
}

#[test]
fn canonical_filesystem_explicit_no_effect_reconciliation_retries_once() {
    canonical_effect_retry(true, false, false, true);
}

fn canonical_effect_retry(prepared_only: bool, revoked: bool, process: bool, explicit_retry: bool) {
    use yai_core_engine::effect::*;
    use yai_core_engine::effect::access::*;
    use yai_core_engine::store::lmdb::LmdbRecordStore;
    use yai_core_engine::transition::*;
    let mut f = Fixture::new("canonical-filesystem-effect");
    let root = f.home.join("workspace");
    fs::create_dir_all(root.join("allowed")).unwrap();
    struct TestProcess(std::process::Child);
    impl Drop for TestProcess {
        fn drop(&mut self) { let _ = self.0.kill(); let _ = self.0.wait(); }
    }
    let child = process.then(|| TestProcess(std::process::Command::new("/usr/bin/sleep")
        .arg("120").spawn().unwrap()));
    let binding = LocalAccessBinding {
        schema: LOCAL_ACCESS_BINDING_SCHEMA.into(), case_id: "case:audit".into(),
        attachment_id: "workspace".into(), address: ResourceAddress::Filesystem {
            root: LocalFilesystemBinding::new("case:audit", "workspace", &root).unwrap(),
        },
    };
    let access = ResourceAccessContract {
        schema: RESOURCE_ACCESS_SCHEMA.into(), configuration_digest: binding.digest(),
        participant_ids: vec!["participant:operator".into()], operations: vec![AccessKind::FilesystemRead],
        read_prefixes: vec!["allowed".into()], names: vec![], max_output_bytes: 4096, max_items: 16,
    };
    if let Some(child) = &child {
        f.success("resource.attach_process", json!({"case_ref":"case:audit","attachment_ref":"workspace",
            "pid":child.0.id(),"policy_owner_participant_ref":"participant:operator",
            "actions":["suspend"],"review_requirement":"automatic"}));
    } else {
        f.success("resource.attach", json!({"binding":binding,"access":access,
            "policy_owner_participant_ref":"participant:operator", "write_prefix":"allowed",
            "max_write_bytes":4096,"review_requirement":"automatic"}));
    }
    f.success("participant.role.add", json!({"case_ref":"case:audit",
        "participant_ref":"participant:operator","role":"operation-proposer"}));
    let mut policy: Value = serde_json::from_slice(include_bytes!("../../../tests/fixtures/cli-product-policy.json")).unwrap();
    if process {
        for rule in policy["rules"].as_array_mut().unwrap() {
            rule["operation_kind"] = json!("process.signal");
            rule["resource_kind"] = json!("process");
        }
    }
    let artifact = f.success("policy.ingest", json!({"tenant_id":"tenant:audit",
        "source_bytes":serde_json::to_vec(&policy).unwrap()}));
    let artifact = artifact["view"]["artifact"]["artifact_id"].as_str().unwrap();
    for op in ["policy.validate", "policy.publish"] {
        f.success(op, json!({"artifact_ref":artifact,"reason":"filesystem Application qualification"}));
    }
    f.success("policy.case.bind", json!({"case_ref":"case:audit","artifact_ref":artifact,
        "expected_generation":f.generation(),"reason":"filesystem Application qualification"}));
    let store = LmdbRecordStore::open(f.home.join("store/lmdb")).unwrap();
    // Seed exact canonical candidate provenance. No test provider performs the
    // effect: the product Application dispatcher must perform its admission.
    let auth = yai_core_engine::security::AuthenticatedPrincipal::authenticate_local().unwrap();
    let commit = |label: &str, payload: TransitionPayload, refs: Vec<String>, scope| {
        let generation = store.get_case_state("case:audit").unwrap().unwrap().generation;
        let mut pending = PendingTransition::new(format!("transition:fixture:{label}"),
            "case:audit", generation, TransitionSource { component:"application-effect-fixture".into(),
                participant_id:Some("participant:operator".into()), principal_id:Some(auth.projected_principal_id()),
                source_ref:Some(label.into()) }, payload);
        pending.causal_refs = refs;
        pending.scope = scope;
        store.commit_secured_transition(&auth, "tenant:audit", pending, true).unwrap();
    };
    commit("provider", TransitionPayload::ProviderAttached {
        participant_id:"participant:operator".into(), provider_id:"provider:fixture".into(),
        provider_kind:"openai_compatible".into(), base_url:"http://127.0.0.1:1".into(),
        model_id:"model:fixture".into(), credential_ref:"none".into(),
    }, vec![], None);
    let lineage = ProviderInvocationLineage {
        projection_id:"projection:fixture".into(), context_frame_id:"context:fixture".into(),
        case_generation:f.generation(), rendered_input_id:"input:fixture".into(),
        rendered_input_digest:"digest:fixture".into(), output_contract_id:"output:fixture".into(),
        continuation_disposition:"not_provided".into(),
    };
    commit("invocation", TransitionPayload::ProviderInvocationStarted {
        invocation_id:"invocation:fixture".into(), participant_id:"participant:operator".into(),
        provider_id:"provider:fixture".into(),provider_kind:"openai_compatible".into(),
        model_id:"model:fixture".into(),semantic_lineage:Some(lineage.clone()),governance:None,
    }, vec![], None);
    let proposal = if process {
        r#"{"schema":"yai.operation_proposal.process_signal.v1","operation":"process.signal","resource":"workspace","action":"suspend"}"#
    } else {
        r#"{"schema":"yai.operation_proposal.filesystem_write.v1","operation":"filesystem.write","resource":"workspace","path":"allowed/result.txt","content":"one governed write"}"#
    };
    commit("result", TransitionPayload::ProviderResultRecorded {
        result_id:"result:fixture".into(),invocation_id:"invocation:fixture".into(),
        provider_id:"provider:fixture".into(),provider_kind:"openai_compatible".into(),
        model_id:"model:fixture".into(),semantic_lineage:Some(lineage.clone()),output:proposal.into(),
    }, vec!["invocation:fixture".into()], None);
    let state = store.get_case_state("case:audit").unwrap().unwrap();
    let normalization = NormalizationContext {
        case_id:"case:audit",participant_id:"participant:operator",provider_result_id:"result:fixture",
        provider_invocation_id:"invocation:fixture",case_generation:state.generation,resource:&state.resources[0],
    };
    let operation = if process {
        normalize_process_signal_candidate(proposal, &normalization,
            &store.get_local_process_binding("case:audit", "workspace").unwrap().unwrap().process).unwrap()
    } else { normalize_filesystem_write_candidate(proposal, &normalization).unwrap() };
    let propose = json!({"case_ref":"case:audit", "participant_ref":"participant:operator",
        "resource_ref":"workspace", "candidate_ref":"result:fixture", "expected_generation":state.generation});
    let mut wrong = propose.clone(); wrong["participant_ref"] = json!("participant:hidden");
    assert_eq!(f.call("effect.propose", wrong).result_state, ResultState::Unauthorized);
    let mut stale = propose.clone(); stale["expected_generation"] = json!(0);
    assert_eq!(f.call("effect.propose", stale).result_state, ResultState::Stale);
    assert_eq!(f.generation(), state.generation);
    let recorded = f.success("effect.propose", propose.clone());
    assert_eq!(recorded["posture"], "recorded");
    assert_eq!(recorded["operation"], serde_json::to_value(&operation).unwrap());
    let after_propose = f.generation();
    assert_eq!(f.success("effect.propose", propose), recorded);
    assert_eq!(f.generation(), after_propose);
    assert!(store.get_case_state("case:audit").unwrap().unwrap().effects.is_empty());
    let input = json!({"case_ref":"case:audit","participant_ref":"participant:operator",
        "operation_ref":operation.operation_id,"expected_generation":f.generation()});
    let mut wrong = input.clone();
    wrong["participant_ref"] = json!("participant:hidden");
    assert_eq!(f.call("effect.submit", wrong).result_state, ResultState::Unauthorized);
    let mut stale = input.clone();
    stale["expected_generation"] = json!(0);
    assert_eq!(f.call("effect.submit", stale).result_state, ResultState::Stale);
    assert!(!root.join("allowed/result.txt").exists());
    if revoked {
        f.success("policy.revoke", json!({"artifact_ref":artifact,"reason":"revoke before Application dispatch"}));
        let result = f.call("effect.submit", input);
        assert!(result.result_state != ResultState::Success
            || result.data.as_ref().is_some_and(|data| data["progress"]["status"] == "denied"));
        assert!(!root.join("allowed/result.txt").exists());
        assert!(store.get_case_state("case:audit").unwrap().unwrap().effects.is_empty());
        return;
    }
    if prepared_only {
        struct BeforeEffect;
        impl yai_application::resource_execution::ControlledEffectHooks for BeforeEffect {
            fn failpoint(&self) -> Option<&str> { Some("after_prepare_before_effect") }
        }
        assert!(yai_application::resource_execution::advance_controlled_operation(
            &auth, &mut BeforeEffect, &store, &operation).unwrap_err().contains("controlled_effect_interrupted"));
        let generation = f.generation();
        f.app = LocalApplication::from_yai_home(&f.home);
        let result = f.success("effect.submit", input);
        assert_eq!(result["progress"]["status"], "indeterminate");
        assert!(result["progress"]["receipt_id"].is_null());
        assert!(!root.join("allowed/result.txt").exists());
        assert_eq!(f.generation(), generation, "PREPARE retry cannot append or dispatch");
        let reconcile = json!({"case_ref":"case:audit","participant_ref":"participant:operator",
            "operation_ref":operation.operation_id,"effect_ref":result["progress"]["effect_id"],
            "expected_generation":generation,"retry_no_effect":explicit_retry});
        let mut wrong = reconcile.clone();
        wrong["effect_ref"] = json!("effect:hidden");
        assert_eq!(f.call("effect.reconcile", wrong).result_state, ResultState::Unauthorized);
        let mut stale = reconcile.clone();
        stale["expected_generation"] = json!(0);
        assert_eq!(f.call("effect.reconcile", stale).result_state, ResultState::Stale);
        let reconciled = f.success("effect.reconcile", reconcile.clone());
        if process {
            assert_eq!(reconciled["progress"]["status"], "indeterminate");
            let status = fs::read_to_string(format!("/proc/{}/status", child.as_ref().unwrap().0.id())).unwrap();
            assert!(!status.lines().any(|line| line.starts_with("State:") && line.contains("T (")),
                "process reconciliation must not send the missing suspend signal");
        } else {
            assert_eq!(reconciled["progress"]["status"], "finalized");
            assert_eq!(reconciled["progress"]["outcome"], if explicit_retry { "applied" } else { "no_effect" });
            assert_eq!(root.join("allowed/result.txt").exists(), explicit_retry);
            let generation = f.generation();
            assert_eq!(f.success("effect.reconcile", reconcile), reconciled);
            assert_eq!(f.generation(), generation, "lost reconciliation response must not repeat recovery");
        }
        return;
    }
    let submitted = f.success("effect.submit", input.clone());
    assert_eq!(submitted["progress"]["status"], "finalized");
    assert_eq!(submitted["progress"]["outcome"], "applied");
    if !process {
        assert_eq!(fs::read_to_string(root.join("allowed/result.txt")).unwrap(), "one governed write");
    }
    let generation = f.generation();
    f.app = LocalApplication::from_yai_home(&f.home);
    assert_eq!(f.success("effect.submit", input.clone()), submitted);
    let observe = json!({"case_ref":"case:audit","participant_ref":"participant:operator",
        "execution":{"domain":"controlled_effect","operation_ref":operation.operation_id}});
    assert_eq!(f.success("execution.get", observe.clone()), submitted);
    assert_eq!(f.generation(), generation);
    let history = store.list_case_transitions("case:audit").unwrap();
    assert_eq!(history.iter().filter(|t| matches!(t.payload,
        TransitionPayload::EffectPrepared {..} | TransitionPayload::ProcessEffectPrepared {..})).count(), 1);
    assert_eq!(history.iter().filter(|t| matches!(t.payload,
        TransitionPayload::EffectFinalized {..} | TransitionPayload::ProcessEffectFinalized {..})).count(), 1);
    // A later canonical Operation cannot erase observation of this receipt.
    // Golden's multi-step work exposed a resolver wrongly requiring last_operation.
    let later_lineage = ProviderInvocationLineage { case_generation:f.generation(), ..lineage };
    commit("later-invocation", TransitionPayload::ProviderInvocationStarted {
        invocation_id:"invocation:later".into(), participant_id:"participant:operator".into(),
        provider_id:"provider:fixture".into(), provider_kind:"openai_compatible".into(),
        model_id:"model:fixture".into(), semantic_lineage:Some(later_lineage.clone()), governance:None,
    }, vec![], None);
    commit("later-result", TransitionPayload::ProviderResultRecorded {
        result_id:"result:later".into(), invocation_id:"invocation:later".into(),
        provider_id:"provider:fixture".into(), provider_kind:"openai_compatible".into(),
        model_id:"model:fixture".into(), semantic_lineage:Some(later_lineage), output:proposal.into(),
    }, vec!["invocation:later".into()], None);
    let later = f.success("effect.propose", json!({"case_ref":"case:audit", "participant_ref":"participant:operator",
        "resource_ref":"workspace", "candidate_ref":"result:later", "expected_generation":f.generation()}));
    assert_ne!(later["operation"]["operation_id"], operation.operation_id);
    let generation = f.generation();
    let historical = f.success("execution.get", observe);
    assert_eq!(historical["progress"], submitted["progress"]);
    assert_eq!(f.success("effect.submit", input)["progress"], submitted["progress"]);
    assert_eq!(f.generation(), generation, "historical receipt retry must neither append nor dispatch");
    assert!(store.verify_case_state("case:audit").unwrap());
}

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
    // Explicit resume admits a new queue item linked to the exact stopped cut,
    // without resetting the run's consumed counters or delivery lineage.
    store.activate_runtime_instance(&auth, "test:runtime-observation", now + 2, 30000, 0).unwrap();
    store.claim_runtime_work(&auth, "test:runtime-observation", &work[0].work_id, "test:worker", now + 3).unwrap();
    store.update_runtime_work_state(&auth, "test:runtime-observation", &work[0].work_id,
        Some("test:worker"), yai_core_engine::store::lmdb::RuntimeWorkState::Cancelled,
        "operator_stopped", now + 4).unwrap();
    let mut checkpoint = read_checkpoint_at(&path, "case:audit").unwrap();
    checkpoint.status = yai_application::runtime_execution::CaseRuntimeStop::OperatorStopped;
    checkpoint.journal_path = work[0].journal_path.clone();
    checkpoint.task = work[0].task.clone();
    checkpoint.invocations = 1;
    checkpoint.last_provider_result_id = Some("result:retained".into());
    write_checkpoint_at(&path, &checkpoint).unwrap();
    let resume = json!({"case_ref":"case:audit","participant_ref":"participant:operator",
        "previous_submission_ref":submission.submission_ref,"submission_ref":"request:resume",
        "run_ref":checkpoint.run_id,"checkpoint_digest":yai_application::runtime_execution::checkpoint_digest(&checkpoint).unwrap(),
        "budgets":submission.budgets});
    let mut stale = resume.clone(); stale["checkpoint_digest"] = json!("wrong");
    assert_eq!(f.call("case.resume", stale).result_state, ResultState::Stale);
    let mut hidden = resume.clone(); hidden["participant_ref"] = json!("participant:hidden");
    assert_eq!(f.call("case.resume", hidden).result_state, ResultState::Unauthorized);
    let admitted = f.success("case.resume", resume.clone());
    assert_eq!(admitted["created"], true);
    let resumed = store.observe_runtime_submission_authorized(&auth, "case:audit", "participant:operator", "request:resume").unwrap();
    let next = yai_application::runtime_execution::resumed_work_checkpoint(&checkpoint, &resumed).unwrap();
    assert_eq!(next.run_id, checkpoint.run_id);
    assert_eq!(next.invocations, 1);
    assert_eq!(next.last_provider_result_id, checkpoint.last_provider_result_id);
    assert!(!next.stop_requested);
    assert_ne!(next.work_item_id, checkpoint.work_item_id);
    let mut second = resume.clone(); second["submission_ref"] = json!("request:second-resume");
    assert_ne!(f.call("case.resume", second).result_state, ResultState::Success);
    write_checkpoint_at(&path, &next).unwrap();
    f.app = LocalApplication::from_yai_home(&f.home);
    let retry = f.success("case.resume", resume);
    assert_eq!(retry["created"], false);
    assert_eq!(retry["execution"], admitted["execution"]);
    assert_eq!(store.list_runtime_work_authorized(&auth).unwrap().len(), 2);
    assert_eq!(f.generation(), generation);
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
    let summary = f.success("case.summary", json!({"case_ref":"case:audit"}));
    assert_eq!(summary["environment"]["resources"][0]["configuration_digest"], binding.digest());
    assert!(summary["environment"]["sources"][0]["attempt"].is_null());
    assert!(summary["environment"]["sources"][0]["progress_ref"].is_null());
    let hidden = f.call("case.summary", json!({"case_ref":"case:other"}));
    assert_eq!(hidden.result_state, ResultState::Unauthorized);
    assert!(hidden.data.is_none());
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
    let carrier = yai_application::resource_execution::source::acquire_carrier(
        &f.home, &store, &auth, "case:audit", &source_id, 1).unwrap();
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
    assert_eq!(f.success("execution.get", input.clone())["posture"], "running");
    assert!(yai_application::resource_execution::source::acquire_carrier(
        &f.home, &store, &auth, "case:audit", &source_id, 1).is_err(),
        "an existing carrier cannot be replaced by another client");
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
        &f.home, &store, &auth, "case:audit", &source_id, 1, &carrier,
    ).unwrap();
    drop(carrier);
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
    let summary = f.success("case.summary", json!({"case_ref":"case:audit"}));
    let projected = summary["environment"]["sources"].as_array().unwrap().iter()
        .find(|source| source["id"] == source_id).unwrap();
    assert_eq!(projected["attempt"], observed["attempt"]);
    assert_eq!(projected["progress_ref"], observed["progress_ref"]);
    assert_eq!(observed["progress_ref"], first.progress_id);
    assert_eq!(observed["phase"], "denied");
    let second = SourceProgress { schema:SOURCE_PROGRESS_SCHEMA.into(), progress_id:String::new(),
        source_id:source_id.clone(), previous_progress_id:Some(first.progress_id), attempt:2,
        phase:SourcePhase::Acquiring, revision:None, decision_ref:None, detail:"second exact attempt".into()
    }.seal().unwrap();
    store.progress_case_source(&auth, "case:audit", &source_id, second.clone()).unwrap();
    let mut legacy_observation = input.clone();
    legacy_observation["execution"]["attempt"] = json!(2);
    assert_eq!(f.success("execution.get", legacy_observation)["posture"], "unresolved",
        "legacy admission without carrier evidence must not be reported dead or running");
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

#[test]
fn provider_model_discovery_dispatcher_authorizes_before_network_and_preserves_case() {
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    let f = Fixture::new("model-discovery");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let input = json!({"tenant_id":"tenant:audit","endpoint":endpoint,"locality":"loopback","credential_ref":"none"});
    let mut hidden = input.clone(); hidden["tenant_id"] = json!("tenant:hidden");
    assert_ne!(f.call("provider.models", hidden).result_state, ResultState::Success);
    let mut wrong_locality = input.clone(); wrong_locality["locality"] = json!("remote");
    assert_ne!(f.call("provider.models", wrong_locality).result_state, ResultState::Success);
    let generation = f.generation();
    let peer = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream.set_read_timeout(Some(std::time::Duration::from_secs(3))).unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut line = String::new();reader.read_line(&mut line).unwrap();
        assert!(line.starts_with("GET /v1/models "));
        loop { line.clear();reader.read_line(&mut line).unwrap();if line == "\r\n" { break } }
        let body = r#"{"data":[{"id":"qualified-model-a"}]}"#;
        write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body).unwrap();
    });
    let result = f.success("provider.models", input);
    peer.join().unwrap();
    assert_eq!(result["models"],json!(["qualified-model-a"]));
    assert_eq!(result["authority"],"provider_metadata_only");
    assert_eq!(f.generation(),generation);
    assert!(f.success("case.summary",json!({"case_ref":"case:audit"}))["compute"]["targets"].as_array().unwrap().is_empty());
}
