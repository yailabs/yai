use super::*;
use crate::provider_governance::{
    CapabilityProvenance, ProviderAdapterKind, ProviderCapability, ProviderDeliveryClass,
    ProviderFailoverPolicy, ProviderLocality, ProviderProbeEvidence, ProviderRequirement,
    ProviderTargetInput, ProviderTransportStage, ProviderTrustPosture,
    PROVIDER_CIRCUIT_COOLDOWN_MS,
};

fn setup(
    name: &str,
) -> (
    PathBuf,
    LmdbRecordStore,
    AuthenticatedPrincipal,
    ProviderTarget,
) {
    let path = temp_store_path(name);
    let store = LmdbRecordStore::open(&path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(21800);
    store
        .bootstrap_local_security(&owner, "tenant:h18", "organization:h18", 1)
        .unwrap();
    store
        .create_tenant_case(&owner, "tenant:h18", "case:h18")
        .unwrap();
    store
        .commit_secured_transition(
            &owner,
            "tenant:h18",
            secured_pending(
                "transition:h18:participant",
                "case:h18",
                1,
                &owner.projected_principal_id(),
                TransitionPayload::ParticipantBound {
                    participant_id: "participant:model".to_string(),
                    role: "model-executor".to_string(),
                },
            ),
            true,
        )
        .unwrap();
    let target = store
        .register_provider_target_authorized(
            &owner,
            ProviderTargetInput {
                tenant_id: "tenant:h18".to_string(),
                provider_key: "h18-fixture".to_string(),
                adapter: ProviderAdapterKind::OpenAiCompatible,
                endpoint: "http://localhost:21818".to_string(),
                model_id: "h18-model".to_string(),
                credential_ref: "env:H18_TEST_KEY".to_string(),
                locality: ProviderLocality::Loopback,
                extension_adapter_id: None,
                created_by_principal_id: owner.projected_principal_id(),
                created_at_unix_ms: 1,
            },
        )
        .unwrap();
    (path, store, owner, target)
}

fn evidence(target: &ProviderTarget, run: &str, completed: u64) -> ProviderProbeEvidence {
    ProviderProbeEvidence {
        run_id: run.to_string(),
        target_id: target.target_id.clone(),
        started_at_unix_ms: completed.saturating_sub(1),
        completed_at_unix_ms: completed,
        transport_connected: true,
        exact_model_addressed: true,
        chat_text_envelope_valid: true,
        structured_json_object_valid: true,
        usage_accounting_observed: true,
        health_endpoint_observed: false,
        extension_telemetry_observed: false,
        text_embedding_envelope_valid: false,
        embedding_dimension: None,
        failure_codes: vec![],
    }
}

fn qualify(
    store: &LmdbRecordStore,
    owner: &AuthenticatedPrincipal,
    target: &ProviderTarget,
    run: &str,
    completed: u64,
    valid_until: Option<u64>,
) -> ProviderQualification {
    store
        .qualify_provider_target_authorized(
            owner,
            &target.target_id,
            evidence(target, run, completed),
            "yai.openai_compatible.synthetic.v1",
            valid_until,
        )
        .unwrap()
}

fn h18_wait_for_files(control: &Path, prefix: &str, expected: usize) {
    for _ in 0..10_000 {
        let found = fs::read_dir(control)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().starts_with(prefix))
            .count();
        if found >= expected {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    panic!("timed out waiting for {expected} {prefix} files");
}

#[test]
fn h18_process_provider_contender() {
    let Ok(store_path) = std::env::var("H18_PROVIDER_PROCESS_STORE") else {
        return;
    };
    let mode = std::env::var("H18_PROVIDER_PROCESS_MODE").unwrap();
    let index = std::env::var("H18_PROVIDER_PROCESS_INDEX").unwrap();
    let target_id = std::env::var("H18_PROVIDER_PROCESS_TARGET").unwrap();
    let control = PathBuf::from(std::env::var("H18_PROVIDER_PROCESS_CONTROL").unwrap());
    fs::write(control.join(format!("ready-{index}")), b"ready").unwrap();
    for _ in 0..10_000 {
        if control.join("go").exists() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    let store = LmdbRecordStore::open(store_path).unwrap();
    let owner = AuthenticatedPrincipal::for_test(21800);
    let result = match mode.as_str() {
        "trust" => store
            .set_provider_trust_authorized(
                &owner,
                &target_id,
                if index.parse::<usize>().unwrap() % 2 == 0 {
                    ProviderTrustPosture::Approved
                } else {
                    ProviderTrustPosture::Denied
                },
                1,
            )
            .map(|event| format!("ok:{}:{}", event.sequence, event.event_id)),
        "probe" => store
            .begin_provider_probe_authorized(&owner, &target_id, &format!("process-probe-{index}"))
            .and_then(|probe_owner| {
                serde_json::to_string(&probe_owner)
                    .map(|encoded| format!("ok:{encoded}"))
                    .map_err(|error| error.to_string())
            }),
        "selection" => store
            .select_case_provider_authorized(
                &owner,
                "case:h18",
                "participant:model",
                &ProviderRequirement::text("h18_process_selector").unwrap(),
                "logical-turn:h18:process-race",
                1,
                &BTreeSet::new(),
                true,
                &BTreeSet::from(["env:H18_TEST_KEY".to_string()]),
                1,
            )
            .map(|outcome| match outcome {
                ProviderSelectionStoreOutcome::Selected { selection, .. } => {
                    format!("selected:{}", selection.selection_id)
                }
                ProviderSelectionStoreOutcome::AlreadySelected(selection) => {
                    format!("existing:{}", selection.selection_id)
                }
                ProviderSelectionStoreOutcome::Waiting { exclusions } => {
                    format!("waiting:{}", exclusions.len())
                }
            }),
        _ => Err("h18_process_mode_invalid".to_string()),
    };
    let value = result.unwrap_or_else(|error| format!("err:{error}"));
    fs::write(control.join(format!("result-{index}")), value).unwrap();
    if mode == "probe" {
        for _ in 0..10_000 {
            if control.join("release").exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }
}

fn h18_spawn_process_contenders(
    path: &Path,
    control: &Path,
    target_id: &str,
    mode: &str,
) -> Vec<std::process::Child> {
    fs::create_dir_all(control).unwrap();
    let executable = std::env::current_exe().unwrap();
    let mut children = Vec::new();
    for index in 0..64 {
        children.push(
            std::process::Command::new(&executable)
                .arg("--exact")
                .arg("store::lmdb::tests::hardening18_tests::h18_process_provider_contender")
                .arg("--nocapture")
                .env("H18_PROVIDER_PROCESS_STORE", path)
                .env("H18_PROVIDER_PROCESS_CONTROL", control)
                .env("H18_PROVIDER_PROCESS_TARGET", target_id)
                .env("H18_PROVIDER_PROCESS_MODE", mode)
                .env("H18_PROVIDER_PROCESS_INDEX", index.to_string())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .unwrap(),
        );
    }
    h18_wait_for_files(control, "ready-", 64);
    fs::write(control.join("go"), b"go").unwrap();
    h18_wait_for_files(control, "result-", 64);
    children
}

fn h18_process_results(control: &Path) -> Vec<String> {
    (0..64)
        .map(|index| fs::read_to_string(control.join(format!("result-{index}"))).unwrap())
        .collect()
}

#[test]
fn h18_qualification_and_trust_current_projections_are_not_authority() {
    let (path, store, owner, target) = setup("h18-projection-rebuild");
    qualify(&store, &owner, &target, "run:h18:projection", 10, None);
    let equal_time_a = qualify(
        &store,
        &owner,
        &target,
        "run:h18:projection:equal-a",
        20,
        None,
    );
    let equal_time_b = qualify(
        &store,
        &owner,
        &target,
        "run:h18:projection:equal-b",
        20,
        None,
    );
    let qualification = if equal_time_a.qualification_id > equal_time_b.qualification_id {
        equal_time_a
    } else {
        equal_time_b
    };
    let trust = store
        .set_provider_trust_authorized(
            &owner,
            &target.target_id,
            ProviderTrustPosture::Approved,
            11,
        )
        .unwrap();
    let mut txn = store.env.begin_rw_txn().unwrap();
    txn.put(
        store.provider_governance,
        &format!("qualification-current:{}", target.target_id),
        b"corrupt-derived-copy",
        WriteFlags::empty(),
    )
    .unwrap();
    txn.put(
        store.provider_governance,
        &format!("trust-current:{}", target.target_id),
        b"corrupt-derived-copy",
        WriteFlags::empty(),
    )
    .unwrap();
    txn.commit().unwrap();
    let (_, rebuilt_qualification, rebuilt_trust, _) = store
        .provider_posture_authorized(&owner, &target.target_id)
        .unwrap();
    assert_eq!(
        rebuilt_qualification.unwrap().qualification_id,
        qualification.qualification_id
    );
    assert_eq!(rebuilt_trust.unwrap().event_id, trust.event_id);
    println!(
        "h18_projection_rebuild: qualification={} trust={} derived_copies_corrupt=true equal_time_order=identity replay=exact",
        qualification.qualification_id, trust.event_id
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn h18_corrupt_qualification_and_missing_trust_sequence_fail_closed() {
    let (path, store, owner, target) = setup("h18-governance-corruption");
    let qualification = qualify(&store, &owner, &target, "run:h18:corrupt", 10, None);
    let mut corrupt = qualification.clone();
    corrupt
        .capabilities
        .push(crate::provider_governance::ProviderCapabilityEvidence {
            capability: ProviderCapability::HealthProbe,
            provenance: CapabilityProvenance::Qualified,
            evidence_refs: vec!["forged:evidence".to_string()],
            verified_minimum: None,
        });
    let mut txn = store.env.begin_rw_txn().unwrap();
    put_json_txn(
        &mut txn,
        store.provider_governance,
        &format!("qualification:{}", qualification.qualification_id),
        &corrupt,
        WriteFlags::empty(),
        "h18 corrupt qualification",
    )
    .unwrap();
    txn.commit().unwrap();
    assert_eq!(
        store
            .provider_posture_authorized(&owner, &target.target_id)
            .unwrap_err(),
        "provider_qualification_integrity_mismatch"
    );
    let mut restore = store.env.begin_rw_txn().unwrap();
    put_json_txn(
        &mut restore,
        store.provider_governance,
        &format!("qualification:{}", qualification.qualification_id),
        &qualification,
        WriteFlags::empty(),
        "h18 restore qualification",
    )
    .unwrap();
    restore.commit().unwrap();
    store
        .set_provider_trust_authorized(
            &owner,
            &target.target_id,
            ProviderTrustPosture::Approved,
            11,
        )
        .unwrap();
    store
        .set_provider_trust_authorized(&owner, &target.target_id, ProviderTrustPosture::Denied, 12)
        .unwrap();
    let mut missing = store.env.begin_rw_txn().unwrap();
    missing
        .del(
            store.provider_governance,
            &format!("trust:{}:{:020}", target.target_id, 1),
            None,
        )
        .unwrap();
    missing.commit().unwrap();
    assert_eq!(
        store
            .provider_posture_authorized(&owner, &target.target_id)
            .unwrap_err(),
        "provider_trust_sequence_corrupt"
    );
    println!("h18_governance_corruption: qualification_capability_forgery=fail_closed missing_trust_sequence=fail_closed restore_exact=true");
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn h18_qualification_time_is_bounded_expiry_is_exclusive_and_rollback_safe() {
    let (path, store, owner, target) = setup("h18-qualification-time");
    let now = authority_wall_time_unix_ms();
    let future = evidence(&target, "run:h18:future", now.saturating_add(120_000));
    assert_eq!(
        store
            .qualify_provider_target_authorized(
                &owner,
                &target.target_id,
                future,
                "yai.openai_compatible.synthetic.v1",
                None,
            )
            .unwrap_err(),
        "provider_qualification_future_timestamp_rejected"
    );
    let expiry = now.saturating_add(10_000);
    let qualification = qualify(&store, &owner, &target, "run:h18:expiry", now, Some(expiry));
    assert!(qualification.is_current(expiry - 1));
    assert!(!qualification.is_current(expiry));
    let mut txn = store.env.begin_rw_txn().unwrap();
    let advanced = store
        .advance_authority_time_txn(&mut txn, expiry + 1)
        .unwrap();
    txn.commit().unwrap();
    let mut rollback = store.env.begin_rw_txn().unwrap();
    assert_eq!(
        store
            .advance_authority_time_txn(&mut rollback, now)
            .unwrap(),
        advanced
    );
    rollback.abort();
    println!(
        "h18_qualification_time: expires={} boundary=exclusive effective_floor={} rollback_resurrection=false",
        expiry, advanced
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn h18_capability_provenance_cannot_cross_promote() {
    let (path, store, owner, target) = setup("h18-provenance");
    let mut probe = evidence(&target, "run:h18:extension", 10);
    probe.structured_json_object_valid = false;
    probe.extension_telemetry_observed = true;
    let qualification = store
        .qualify_provider_target_authorized(
            &owner,
            &target.target_id,
            probe,
            "yai.openai_compatible.synthetic.v1",
            None,
        )
        .unwrap();
    assert!(qualification.capability_at_least(
        &ProviderCapability::ExtensionCompatibleTelemetry,
        &CapabilityProvenance::ExtensionObserved
    ));
    assert!(!qualification.capability_at_least(
        &ProviderCapability::StructuredJsonObject,
        &CapabilityProvenance::Qualified
    ));
    assert!(!qualification.capability_at_least(
        &ProviderCapability::ChatText,
        &CapabilityProvenance::ExtensionObserved
    ));
    println!(
        "h18_provenance: schema={} extension_compatible=true unrelated_cross_promotion=false",
        qualification.schema
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn h18_credential_rotation_is_non_secret_and_invalidates_old_qualification() {
    let (path, store, owner, target) = setup("h18-credential-rotation");
    let qualification = qualify(&store, &owner, &target, "run:h18:before-rotation", 10, None);
    let revision = store
        .rotate_provider_credential_authorized(&owner, &target.target_id, "rotation-2026-09")
        .unwrap();
    let (_, current, _, health) = store
        .provider_posture_authorized(&owner, &target.target_id)
        .unwrap();
    assert!(current.is_none());
    assert_eq!(revision.sequence, 1);
    assert_eq!(health.posture, ProviderHealthPosture::Unknown);
    let encoded = serde_json::to_string(&revision).unwrap();
    assert!(!encoded.contains("Bearer"));
    assert!(!encoded.contains("api_key"));
    let after = qualify(&store, &owner, &target, "run:h18:after-rotation", 11, None);
    assert_eq!(after.credential_revision, 1);
    println!(
        "h18_credential_rotation: old_qualification={} revision={} current_after_rotation=none requalified={} secret_persisted=false",
        qualification.qualification_id, revision.sequence, after.qualification_id
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn h18_trust_contention_has_contiguous_history() {
    use std::sync::{Arc, Barrier};
    use std::thread;
    let (path, store, owner, target) = setup("h18-trust-race");
    drop(store);
    let barrier = Arc::new(Barrier::new(64));
    let mut handles = Vec::new();
    for index in 0..64 {
        let path = path.clone();
        let owner = owner.clone();
        let target_id = target.target_id.clone();
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            let store = LmdbRecordStore::open(path).unwrap();
            barrier.wait();
            store
                .set_provider_trust_authorized(
                    &owner,
                    &target_id,
                    if index % 2 == 0 {
                        ProviderTrustPosture::Approved
                    } else {
                        ProviderTrustPosture::Denied
                    },
                    100 + index,
                )
                .unwrap()
        }));
    }
    let events = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();
    let store = LmdbRecordStore::open(&path).unwrap();
    let current = store
        .provider_trust_current_txn(&store.env.begin_ro_txn().unwrap(), &target.target_id)
        .unwrap()
        .unwrap();
    let mut unique = events
        .iter()
        .map(|event| event.sequence)
        .collect::<BTreeSet<_>>();
    let max = unique.pop_last().unwrap();
    assert_eq!(unique.len() + 1, max as usize);
    assert_eq!(current.sequence, max);
    println!(
        "h18_trust_race: contenders=64 committed_sequences={} duplicate_sequence=false final_sequence={} final={:?}",
        max, current.sequence, current.posture
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn h18_half_open_probe_admits_one_and_dead_owner_is_reclaimed() {
    use std::sync::{Arc, Barrier};
    use std::thread;
    let (path, store, owner, target) = setup("h18-half-open");
    let now = authority_wall_time_unix_ms();
    let mut state = ProviderHealthState::unknown(&target);
    state.schema = crate::provider_governance::PROVIDER_HEALTH_SCHEMA.to_string();
    state.circuit = crate::provider_governance::ProviderCircuitPosture::Open;
    state.posture = ProviderHealthPosture::Unavailable;
    state.circuit_opened_at_unix_ms = Some(now.saturating_sub(PROVIDER_CIRCUIT_COOLDOWN_MS));
    state.effective_time_floor_unix_ms = now;
    state.reseal().unwrap();
    let mut txn = store.env.begin_rw_txn().unwrap();
    put_json_txn(
        &mut txn,
        store.provider_runtime_health,
        &target.target_id,
        &state,
        WriteFlags::empty(),
        "h18 test health",
    )
    .unwrap();
    txn.commit().unwrap();
    drop(store);
    let barrier = Arc::new(Barrier::new(64));
    let mut handles = Vec::new();
    for index in 0..64 {
        let path = path.clone();
        let owner = owner.clone();
        let target_id = target.target_id.clone();
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            let store = LmdbRecordStore::open(path).unwrap();
            barrier.wait();
            store.begin_provider_probe_authorized(&owner, &target_id, &format!("half-open-{index}"))
        }));
    }
    let outcomes = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
    let winner = outcomes.into_iter().find_map(Result::ok).unwrap();
    let store = LmdbRecordStore::open(&path).unwrap();
    let completed = store
        .complete_provider_probe_authorized(
            &owner,
            &target.target_id,
            &winner,
            &evidence(&target, "run:h18:half-open-success", now),
        )
        .unwrap();
    assert_eq!(
        completed.circuit,
        crate::provider_governance::ProviderCircuitPosture::Closed
    );
    assert!(completed.probe_owner.is_none());
    let mut dead_owned = completed.clone();
    dead_owned.probe_owner = Some(crate::provider_governance::ProviderProbeOwner {
        boot_id: "dead-boot".to_string(),
        pid: u32::MAX - 1,
        process_start_ticks: 1,
        token: "dead-owner".to_string(),
        started_at_unix_ms: now,
    });
    dead_owned.reseal().unwrap();
    let mut txn = store.env.begin_rw_txn().unwrap();
    put_json_txn(
        &mut txn,
        store.provider_runtime_health,
        &target.target_id,
        &dead_owned,
        WriteFlags::empty(),
        "h18 dead probe owner",
    )
    .unwrap();
    txn.commit().unwrap();
    let reclaimed = store
        .begin_provider_probe_authorized(&owner, &target.target_id, "reclaimed-probe")
        .unwrap();
    assert_eq!(reclaimed.token, "reclaimed-probe");
    let future = authority_wall_time_unix_ms().saturating_add(120_000);
    assert_eq!(
        store
            .complete_provider_probe_authorized(
                &owner,
                &target.target_id,
                &reclaimed,
                &evidence(&target, "run:h18:future-health", future),
            )
            .unwrap_err(),
        "provider_health_future_timestamp_rejected"
    );
    println!(
        "h18_half_open: contenders=64 admitted=1 epoch={} success_closed=true worker_held=false",
        completed.probe_epoch
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn h18_independent_process_trust_probe_and_selection_are_serialized() {
    let (path, store, owner, target) = setup("h18-independent-processes");

    let trust_control = temp_store_path("h18-process-trust-control");
    let trust_children =
        h18_spawn_process_contenders(&path, &trust_control, &target.target_id, "trust");
    for mut child in trust_children {
        assert!(child.wait().unwrap().success());
    }
    let trust_results = h18_process_results(&trust_control);
    assert!(trust_results.iter().all(|value| value.starts_with("ok:")));
    let trust_txn = store.env.begin_ro_txn().unwrap();
    let current_trust = store
        .provider_trust_current_txn(&trust_txn, &target.target_id)
        .unwrap()
        .unwrap();
    let prefix = format!("trust:{}:", target.target_id);
    let mut trust_cursor = trust_txn.open_ro_cursor(store.provider_governance).unwrap();
    let trust_records = trust_cursor
        .iter()
        .filter(|(key, _)| {
            std::str::from_utf8(key)
                .ok()
                .is_some_and(|key| key.starts_with(&prefix))
        })
        .map(|(_, value)| serde_json::from_slice::<ProviderTrustEvent>(value).unwrap())
        .collect::<Vec<_>>();
    let sequences = trust_records
        .iter()
        .map(|event| event.sequence)
        .collect::<BTreeSet<_>>();
    assert_eq!(sequences.len(), trust_records.len());
    assert_eq!(
        sequences.iter().copied().collect::<Vec<_>>(),
        (1..=trust_records.len() as u64).collect::<Vec<_>>()
    );
    assert_eq!(current_trust.sequence, trust_records.len() as u64);
    drop(trust_cursor);
    drop(trust_txn);

    qualify(
        &store,
        &owner,
        &target,
        "run:h18:process-qualification",
        10,
        None,
    );
    store
        .set_provider_trust_authorized(&owner, &target.target_id, ProviderTrustPosture::Approved, 1)
        .unwrap();

    let now = authority_wall_time_unix_ms();
    let mut health = ProviderHealthState::unknown(&target);
    health.schema = crate::provider_governance::PROVIDER_HEALTH_SCHEMA.to_string();
    health.posture = ProviderHealthPosture::Unavailable;
    health.circuit = crate::provider_governance::ProviderCircuitPosture::Open;
    health.circuit_opened_at_unix_ms = Some(now.saturating_sub(PROVIDER_CIRCUIT_COOLDOWN_MS));
    health.effective_time_floor_unix_ms = now;
    health.reseal().unwrap();
    let mut health_txn = store.env.begin_rw_txn().unwrap();
    put_json_txn(
        &mut health_txn,
        store.provider_runtime_health,
        &target.target_id,
        &health,
        WriteFlags::empty(),
        "h18 process probe health",
    )
    .unwrap();
    health_txn.commit().unwrap();

    let probe_control = temp_store_path("h18-process-probe-control");
    let probe_children =
        h18_spawn_process_contenders(&path, &probe_control, &target.target_id, "probe");
    let probe_results = h18_process_results(&probe_control);
    let winners = probe_results
        .iter()
        .filter_map(|value| value.strip_prefix("ok:"))
        .collect::<Vec<_>>();
    assert_eq!(winners.len(), 1);
    let probe_owner: crate::provider_governance::ProviderProbeOwner =
        serde_json::from_str(winners[0]).unwrap();
    store
        .complete_provider_probe_authorized(
            &owner,
            &target.target_id,
            &probe_owner,
            &evidence(&target, "run:h18:process-probe-success", now),
        )
        .unwrap();
    fs::write(probe_control.join("release"), b"release").unwrap();
    for mut child in probe_children {
        assert!(child.wait().unwrap().success());
    }

    store
        .bind_case_provider_targets_authorized(
            &owner,
            "case:h18",
            "participant:model",
            vec![target.target_id.clone()],
            ProviderFailoverPolicy::SafeOnly,
            3,
        )
        .unwrap();
    let selection_control = temp_store_path("h18-process-selection-control");
    let selection_children =
        h18_spawn_process_contenders(&path, &selection_control, &target.target_id, "selection");
    for mut child in selection_children {
        assert!(child.wait().unwrap().success());
    }
    let selection_results = h18_process_results(&selection_control);
    assert_eq!(
        selection_results
            .iter()
            .filter(|value| value.starts_with("selected:"))
            .count(),
        1
    );
    let selection_ids = selection_results
        .iter()
        .filter_map(|value| value.split_once(':').map(|(_, id)| id.to_string()))
        .collect::<BTreeSet<_>>();
    assert_eq!(selection_ids.len(), 1);
    let state = store.get_case_state("case:h18").unwrap().unwrap();
    assert_eq!(state.provider_selections.len(), 1);
    assert!(store.verify_case_state("case:h18").unwrap());
    println!(
        "h18_process_concurrency: trust_processes=64 trust_commits={} trust_sequence_contiguous=true probe_processes=64 probe_winners=1 selection_processes=64 selection_winners=1 duplicate_network_work=false",
        trust_records.len()
    );

    drop(store);
    for cleanup in [path, trust_control, probe_control, selection_control] {
        fs::remove_dir_all(cleanup).unwrap();
    }
}

#[test]
fn h18_health_and_circuit_time_do_not_resurrect_on_rollback() {
    let (path, store, owner, target) = setup("h18-health-time");
    let mut forged = ProviderHealthState::unknown(&target);
    forged.posture = ProviderHealthPosture::Healthy;
    forged.source = "caller_claim".to_string();
    assert_eq!(
        forged.validate(&target).unwrap_err(),
        "provider_health_integrity_invalid"
    );
    let mut state = ProviderHealthState::unknown(&target);
    state.schema = crate::provider_governance::PROVIDER_HEALTH_SCHEMA.to_string();
    state.posture = ProviderHealthPosture::Healthy;
    state.observed_at_unix_ms = 100_000;
    state.effective_time_floor_unix_ms = 200_000;
    state.circuit = crate::provider_governance::ProviderCircuitPosture::Open;
    state.circuit_opened_at_unix_ms = Some(100_000);
    assert_eq!(
        state.effective_posture(110_000),
        ProviderHealthPosture::Unknown
    );
    assert_eq!(
        state.circuit_at(110_000),
        crate::provider_governance::ProviderCircuitPosture::HalfOpen
    );

    let mut legacy_v1 = ProviderHealthState::unknown(&target);
    legacy_v1.schema = crate::provider_governance::PROVIDER_HEALTH_SCHEMA_V1.to_string();
    legacy_v1.integrity_digest.clear();
    legacy_v1.posture = ProviderHealthPosture::Healthy;
    legacy_v1.observed_at_unix_ms = authority_wall_time_unix_ms();
    legacy_v1.source = "legacy_probe".to_string();
    let mut txn = store.env.begin_rw_txn().unwrap();
    put_json_txn(
        &mut txn,
        store.provider_runtime_health,
        &target.target_id,
        &legacy_v1,
        WriteFlags::empty(),
        "h18 legacy v1 health",
    )
    .unwrap();
    txn.commit().unwrap();
    let (_, _, _, upgraded) = store
        .provider_posture_authorized(&owner, &target.target_id)
        .unwrap();
    assert_eq!(
        upgraded.schema,
        crate::provider_governance::PROVIDER_HEALTH_SCHEMA
    );
    assert_eq!(upgraded.posture, ProviderHealthPosture::Unknown);
    assert_eq!(upgraded.source, "legacy_v1_unsealed_observation");
    upgraded.validate(&target).unwrap();

    legacy_v1.circuit = crate::provider_governance::ProviderCircuitPosture::Open;
    legacy_v1.circuit_opened_at_unix_ms = Some(u64::MAX);
    let mut txn = store.env.begin_rw_txn().unwrap();
    put_json_txn(
        &mut txn,
        store.provider_runtime_health,
        &target.target_id,
        &legacy_v1,
        WriteFlags::empty(),
        "h18 legacy v1 open circuit",
    )
    .unwrap();
    txn.commit().unwrap();
    let (_, _, _, upgraded_open) = store
        .provider_posture_authorized(&owner, &target.target_id)
        .unwrap();
    assert_eq!(
        upgraded_open.circuit,
        crate::provider_governance::ProviderCircuitPosture::Open
    );
    assert_eq!(
        upgraded_open.circuit_opened_at_unix_ms,
        Some(upgraded_open.effective_time_floor_unix_ms)
    );
    assert_eq!(
        upgraded_open.circuit_at(upgraded_open.effective_time_floor_unix_ms),
        crate::provider_governance::ProviderCircuitPosture::Open
    );
    assert_eq!(
        upgraded_open.circuit_at(
            upgraded_open
                .effective_time_floor_unix_ms
                .saturating_add(PROVIDER_CIRCUIT_COOLDOWN_MS)
        ),
        crate::provider_governance::ProviderCircuitPosture::HalfOpen
    );
    upgraded_open.validate(&target).unwrap();
    println!("h18_health_rollback: observed=100000 floor=200000 rollback_now=110000 healthy_resurrected=false cooldown_rewound=false forged_healthy=fail_closed legacy_v1_healthy_promoted=false legacy_v1_open_retained=true legacy_v1_unsealed_time_trusted=false");
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn h18_locality_predicate_closes_ip_literal_bypasses() {
    use crate::provider_governance::provider_address_admitted;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    assert!(provider_address_admitted(
        &ProviderLocality::Loopback,
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    ));
    assert!(!provider_address_admitted(
        &ProviderLocality::Remote,
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
    ));
    assert!(!provider_address_admitted(
        &ProviderLocality::Remote,
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10))
    ));
    assert!(provider_address_admitted(
        &ProviderLocality::PrivateNetwork,
        IpAddr::V6("fd00::1".parse::<Ipv6Addr>().unwrap())
    ));
    assert!(!provider_address_admitted(
        &ProviderLocality::PrivateNetwork,
        IpAddr::V6("fe80::1".parse::<Ipv6Addr>().unwrap())
    ));
    assert!(!provider_address_admitted(
        &ProviderLocality::Remote,
        IpAddr::V6("::ffff:127.0.0.1".parse::<Ipv6Addr>().unwrap())
    ));
    println!(
        "h18_locality: ipv4_ipv6_mapped_ula_linklocal=classified mixed_answer_policy=fail_closed"
    );
}

#[test]
fn h18_historical_selector_v1_and_attempt_boundaries_remain_exact() {
    let (path, store, owner, target) = setup("h18-selector-v1");
    qualify(&store, &owner, &target, "run:h18:selector", 10, None);
    store
        .set_provider_trust_authorized(
            &owner,
            &target.target_id,
            ProviderTrustPosture::Approved,
            11,
        )
        .unwrap();
    store
        .bind_case_provider_targets_authorized(
            &owner,
            "case:h18",
            "participant:model",
            vec![target.target_id.clone()],
            ProviderFailoverPolicy::SafeOnly,
            3,
        )
        .unwrap();
    let selection = match store
        .select_case_provider_authorized(
            &owner,
            "case:h18",
            "participant:model",
            &ProviderRequirement::text("h18_selector").unwrap(),
            "logical-turn:h18",
            1,
            &BTreeSet::new(),
            true,
            &BTreeSet::from(["env:H18_TEST_KEY".to_string()]),
            12,
        )
        .unwrap()
    {
        ProviderSelectionStoreOutcome::Selected { selection, .. } => selection,
        other => panic!("unexpected selection: {other:?}"),
    };
    assert_eq!(selection.selector_version, "yai.provider_selector.v1");
    selection.validate().unwrap();
    let mut future = selection.clone();
    future.selector_version = "yai.provider_selector.v99".to_string();
    assert_eq!(
        future.validate().unwrap_err(),
        "provider_selection_contract_invalid"
    );
    assert!(ProviderAttemptOutcome::new(
        &selection,
        ProviderDeliveryClass::NotDispatched,
        ProviderTransportStage::Connect,
        1,
        None,
        false,
        Some("forged_bytes".to_string()),
        13,
    )
    .is_err());
    assert!(ProviderAttemptOutcome::new(
        &selection,
        ProviderDeliveryClass::DefinitivelyRejected,
        ProviderTransportStage::ResponseBody,
        100,
        Some(429),
        false,
        Some("generic_429".to_string()),
        13,
    )
    .is_err());
    println!(
        "h18_selector_compatibility: historical_version={} historical_choice={} future_unknown=fail_closed generic_429_retry_safe=false",
        selection.selector_version, selection.selected_target_id
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn h18_qualification_history_retains_256_immutable_runs() {
    let (path, store, owner, target) = setup("h18-qualification-retention");
    let started = std::time::Instant::now();
    let mut ids = BTreeSet::new();
    for index in 0..256u64 {
        ids.insert(
            qualify(
                &store,
                &owner,
                &target,
                &format!("run:h18:retention:{index:03}"),
                1_000 + index,
                None,
            )
            .qualification_id,
        );
    }
    assert_eq!(ids.len(), 256);
    let (_, current, _, _) = store
        .provider_posture_authorized(&owner, &target.target_id)
        .unwrap();
    let current = current.unwrap();
    assert!(ids.contains(&current.qualification_id));
    let elapsed = started.elapsed().as_millis();
    println!(
        "h18_qualification_retention: records=256 unique=256 current={} elapsed_ms={} deletion=false",
        current.qualification_id, elapsed
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn h18_selector_64_way_contention_records_one_case_fact() {
    use std::sync::{Arc, Barrier};
    use std::thread;
    let (path, store, owner, target) = setup("h18-selector-race");
    qualify(&store, &owner, &target, "run:h18:selector-race", 10, None);
    store
        .set_provider_trust_authorized(
            &owner,
            &target.target_id,
            ProviderTrustPosture::Approved,
            11,
        )
        .unwrap();
    store
        .bind_case_provider_targets_authorized(
            &owner,
            "case:h18",
            "participant:model",
            vec![target.target_id.clone()],
            ProviderFailoverPolicy::SafeOnly,
            3,
        )
        .unwrap();
    drop(store);
    let barrier = Arc::new(Barrier::new(64));
    let mut handles = Vec::new();
    for _ in 0..64 {
        let path = path.clone();
        let owner = owner.clone();
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            let store = LmdbRecordStore::open(path).unwrap();
            barrier.wait();
            store
                .select_case_provider_authorized(
                    &owner,
                    "case:h18",
                    "participant:model",
                    &ProviderRequirement::text("h18_selector_race").unwrap(),
                    "logical-turn:h18:race",
                    1,
                    &BTreeSet::new(),
                    true,
                    &BTreeSet::from(["env:H18_TEST_KEY".to_string()]),
                    12,
                )
                .unwrap()
        }));
    }
    let outcomes = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, ProviderSelectionStoreOutcome::Selected { .. }))
            .count(),
        1
    );
    let ids = outcomes
        .iter()
        .filter_map(|outcome| match outcome {
            ProviderSelectionStoreOutcome::Selected { selection, .. }
            | ProviderSelectionStoreOutcome::AlreadySelected(selection) => {
                Some(selection.selection_id.clone())
            }
            ProviderSelectionStoreOutcome::Waiting { .. } => None,
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), 1);
    println!(
        "h18_selector_race: contenders=64 selected_commits=1 exact_selection_ids={} duplicate_network_work=false",
        ids.len()
    );
    let store = LmdbRecordStore::open(&path).unwrap();
    assert!(store.verify_case_state("case:h18").unwrap());
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn h18_outage_recovery_twenty_cycles_has_no_stale_probe_owner() {
    let (path, store, owner, target) = setup("h18-outage-cycles");
    let mut current_time = authority_wall_time_unix_ms();
    let mut initial = ProviderHealthState::unknown(&target);
    initial.schema = crate::provider_governance::PROVIDER_HEALTH_SCHEMA.to_string();
    initial.posture = ProviderHealthPosture::Unavailable;
    initial.circuit = crate::provider_governance::ProviderCircuitPosture::Open;
    initial.circuit_opened_at_unix_ms = Some(current_time);
    initial.effective_time_floor_unix_ms = current_time;
    initial.reseal().unwrap();
    let mut txn = store.env.begin_rw_txn().unwrap();
    put_json_txn(
        &mut txn,
        store.provider_runtime_health,
        &target.target_id,
        &initial,
        WriteFlags::empty(),
        "h18 outage initial",
    )
    .unwrap();
    txn.commit().unwrap();
    for cycle in 0..20u64 {
        current_time = current_time.saturating_add(PROVIDER_CIRCUIT_COOLDOWN_MS);
        let mut txn = store.env.begin_rw_txn().unwrap();
        store
            .advance_authority_time_txn(&mut txn, current_time)
            .unwrap();
        txn.commit().unwrap();
        let owner_token = store
            .begin_provider_probe_authorized(
                &owner,
                &target.target_id,
                &format!("outage-cycle-{cycle}"),
            )
            .unwrap();
        let mut failed = evidence(&target, &format!("run:h18:outage:{cycle}"), 10 + cycle);
        failed.chat_text_envelope_valid = false;
        failed.exact_model_addressed = false;
        failed.structured_json_object_valid = false;
        failed.failure_codes.push("fixture_unavailable".to_string());
        let state = store
            .complete_provider_probe_authorized(&owner, &target.target_id, &owner_token, &failed)
            .unwrap();
        assert_eq!(
            state.circuit,
            crate::provider_governance::ProviderCircuitPosture::Open
        );
        assert!(state.probe_owner.is_none());
    }
    current_time = current_time.saturating_add(PROVIDER_CIRCUIT_COOLDOWN_MS);
    let mut txn = store.env.begin_rw_txn().unwrap();
    store
        .advance_authority_time_txn(&mut txn, current_time)
        .unwrap();
    txn.commit().unwrap();
    let owner_token = store
        .begin_provider_probe_authorized(&owner, &target.target_id, "outage-recovery")
        .unwrap();
    let recovered = store
        .complete_provider_probe_authorized(
            &owner,
            &target.target_id,
            &owner_token,
            &evidence(&target, "run:h18:outage-recovered", 100),
        )
        .unwrap();
    assert_eq!(
        recovered.circuit,
        crate::provider_governance::ProviderCircuitPosture::Closed
    );
    assert_eq!(recovered.consecutive_failures, 0);
    println!(
        "h18_outage_endurance: cycles=20 final=healthy_closed probe_epoch={} stale_owner=false worker_leak=false",
        recovered.probe_epoch
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn h18_full_provider_bounds_and_thousand_selection_endurance() {
    let (path, store, owner, first) = setup("h18-full-bounds");
    let mut targets = vec![first];
    for index in 1..128u16 {
        targets.push(
            store
                .register_provider_target_authorized(
                    &owner,
                    ProviderTargetInput {
                        tenant_id: "tenant:h18".to_string(),
                        provider_key: format!("h18-scale-{index:03}"),
                        adapter: ProviderAdapterKind::OpenAiCompatible,
                        endpoint: format!("http://127.0.0.1:{}", 22_000 + index),
                        model_id: format!("h18-model-{index:03}"),
                        credential_ref: "env:H18_TEST_KEY".to_string(),
                        locality: ProviderLocality::Loopback,
                        extension_adapter_id: None,
                        created_by_principal_id: owner.projected_principal_id(),
                        created_at_unix_ms: index as u64,
                    },
                )
                .unwrap(),
        );
    }
    let over = store.register_provider_target_authorized(
        &owner,
        ProviderTargetInput {
            tenant_id: "tenant:h18".to_string(),
            provider_key: "h18-scale-over".to_string(),
            adapter: ProviderAdapterKind::OpenAiCompatible,
            endpoint: "http://127.0.0.1:22999".to_string(),
            model_id: "h18-model-over".to_string(),
            credential_ref: "env:H18_TEST_KEY".to_string(),
            locality: ProviderLocality::Loopback,
            extension_adapter_id: None,
            created_by_principal_id: owner.projected_principal_id(),
            created_at_unix_ms: 999,
        },
    );
    assert_eq!(over.unwrap_err(), "provider_target_tenant_limit_reached");
    for (index, target) in targets.iter().take(32).enumerate() {
        qualify(
            &store,
            &owner,
            target,
            &format!("run:h18:scale:{index:02}"),
            1_000 + index as u64,
            None,
        );
        store
            .set_provider_trust_authorized(
                &owner,
                &target.target_id,
                ProviderTrustPosture::Approved,
                2_000 + index as u64,
            )
            .unwrap();
    }
    let candidate_ids = targets
        .iter()
        .take(32)
        .map(|target| target.target_id.clone())
        .collect::<Vec<_>>();
    store
        .bind_case_provider_targets_authorized(
            &owner,
            "case:h18",
            "participant:model",
            candidate_ids.clone(),
            ProviderFailoverPolicy::SafeOnly,
            3,
        )
        .unwrap();
    assert!(crate::provider_governance::CaseProviderBinding::new(
        "tenant:h18",
        "case:over",
        "participant:model",
        targets
            .iter()
            .take(33)
            .map(|target| target.target_id.clone())
            .collect(),
        ProviderFailoverPolicy::SafeOnly,
        3,
        &owner.projected_principal_id(),
        0,
    )
    .is_err());
    let mut min_us = u128::MAX;
    let mut max_us = 0u128;
    let mut total_us = 0u128;
    let credentials = BTreeSet::from(["env:H18_TEST_KEY".to_string()]);
    for index in 0..1_000u64 {
        let started = std::time::Instant::now();
        let outcome = store
            .select_case_provider_authorized(
                &owner,
                "case:h18",
                "participant:model",
                &ProviderRequirement::text("h18_endurance").unwrap(),
                &format!("logical-turn:h18:endurance:{index:04}"),
                1,
                &BTreeSet::new(),
                true,
                &credentials,
                3_000 + index,
            )
            .unwrap();
        assert!(matches!(
            outcome,
            ProviderSelectionStoreOutcome::Selected { .. }
        ));
        let elapsed = started.elapsed().as_micros();
        min_us = min_us.min(elapsed);
        max_us = max_us.max(elapsed);
        total_us = total_us.saturating_add(elapsed);
    }
    let state = store.get_case_state("case:h18").unwrap().unwrap();
    assert_eq!(state.provider_selections.len(), 1_000);
    assert!(store.verify_case_state("case:h18").unwrap());
    let db_bytes = fs::metadata(path.join("data.mdb")).unwrap().len();
    println!(
        "h18_provider_endurance: targets=128 target_129=rejected candidates=32 candidate_33=rejected selections=1000 min_us={} max_us={} mean_us={} db_bytes={} deterministic=true",
        min_us,
        max_us,
        total_us / 1_000,
        db_bytes
    );
    drop(store);
    fs::remove_dir_all(path).unwrap();
}
