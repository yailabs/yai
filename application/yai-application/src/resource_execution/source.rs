//! Source acquisition orchestration shared by product clients.
//! Progress, attempts, policy and exact backing remain owned by the engine.
use std::path::Path;
use super::ResourceActionOutcome;
use yai_core_engine::effect::access::source::*;
use yai_core_engine::effect::access::{ResourceAction, ResourceRequest, RESOURCE_REQUEST_SCHEMA};
use yai_core_engine::effect::{digest_bytes, DecisionOutcome};
use yai_core_engine::security::AuthenticatedPrincipal;
use yai_core_engine::store::lmdb::LmdbRecordStore;
use yai_core_engine::transition::TransitionPayload;

/// Operational lifetime only, never permission to acquire or resume. The
/// canonical attempt admission and current Resource fences still decide that.
pub struct AcquisitionCarrier {
    _lock: std::fs::File,
    case: String,
    source: String,
    attempt: u64,
}

fn carrier_path(home: &Path, case: &str, source: &str, attempt: u64) -> std::path::PathBuf {
    home.join("run/source-carriers").join(format!("{}.lock",
        yai_core_engine::context::stable_digest(&format!("{case}\0{source}\0{attempt}"))))
}

pub fn acquire_carrier(
    home: &Path, store: &LmdbRecordStore, auth: &AuthenticatedPrincipal,
    case: &str, source: &str, attempt: u64,
) -> Result<AcquisitionCarrier, String> {
    let (_, current) = store.case_source_authorized(auth, case, source)?;
    let path = carrier_path(home, case, &current.declaration.source_id, attempt);
    // An older uninstrumented attempt cannot acquire a retrospective liveness
    // witness. Absence of the marker is not evidence that its carrier died.
    if current.progress.as_ref().is_some_and(|p| p.attempt == attempt && p.phase == SourcePhase::Acquiring)
        && !path.exists() {
        return Err("source_carrier_lifetime_unknown".into());
    }
    let lock = crate::runtime_execution::acquire_execution_carrier(&path)
        .map_err(|error| error.replace("execution_carrier", "source_carrier"))?;
    Ok(AcquisitionCarrier { _lock: lock, case: case.into(),
        source: current.declaration.source_id, attempt })
}

pub fn execution_posture(home: &Path, case: &str, source: &str, attempt: u64, phase: &SourcePhase)
    -> Result<crate::SourceExecutionPosture, String> {
    use crate::SourceExecutionPosture;
    if phase != &SourcePhase::Acquiring { return Ok(phase.into()); }
    let file = match std::fs::OpenOptions::new().read(true).write(true)
        .open(carrier_path(home, case, source, attempt)) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(SourceExecutionPosture::Unresolved),
        Err(e) => return Err(format!("source_carrier_observation:{e}")),
    };
    match file.try_lock() {
        Ok(()) => Ok(SourceExecutionPosture::DeliveryIndeterminate),
        Err(std::fs::TryLockError::WouldBlock) => Ok(SourceExecutionPosture::Running),
        Err(std::fs::TryLockError::Error(e)) => Err(format!("source_carrier_observation:{e}")),
    }
}

/// Combine canonical progress and ephemeral carrier observation without
/// reporting an old Acquiring cut as indeterminate after a terminal commit.
/// The second authorized read is also the current disclosure fence.
pub fn observe_execution(
    home: &Path, store: &LmdbRecordStore, auth: &AuthenticatedPrincipal,
    case: &str, participant: &str, source: &str, attempt: u64,
) -> Result<crate::SourceExecutionObservation, String> {
    let (generation, progress, current_source_phase) =
        store.observe_source_attempt_authorized(auth, case, participant, source, attempt)?;
    let posture = execution_posture(home, case, source, attempt, &progress.phase)?;
    let (latest_generation, latest, latest_phase) =
        store.observe_source_attempt_authorized(auth, case, participant, source, attempt)?;
    if generation != latest_generation || progress.progress_id != latest.progress_id
        || current_source_phase != latest_phase {
        return Err("source_execution_observation_stale".into());
    }
    Ok(crate::SourceExecutionObservation {
        schema: "yai.source_execution_observation.v1".into(),
        case_ref: case.into(), participant_ref: participant.into(), source_ref: source.into(),
        attempt, progress_ref: progress.progress_id, posture, phase: progress.phase,
        current_source_phase, observed_generation: generation,
    })
}

pub fn progress(
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    case: &str,
    name: &str,
    attempt: u64,
    phase: SourcePhase,
    revision: Option<SourceRevision>,
    decision_ref: Option<String>,
    detail: &str,
) -> Result<(), String> {
    let (_, source) = store.case_source_authorized(auth, case, name)?;
    let p = SourceProgress {
        schema: SOURCE_PROGRESS_SCHEMA.into(),
        progress_id: String::new(),
        source_id: source.declaration.source_id,
        previous_progress_id: source.progress.map(|p| p.progress_id),
        attempt,
        phase,
        revision,
        decision_ref,
        detail: detail.into(),
    }
    .seal()?;
    store.progress_case_source(auth, case, name, p)?;
    Ok(())
}

pub fn acquire(
    home: &Path,
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    case: &str,
    name: &str,
    refresh: bool,
) -> Result<(), String> {
    let (state, source) = store.case_source_authorized(auth, case, name)?;
    let declaration = &source.declaration;
    if source.progress.as_ref().is_some_and(|p| p.phase == SourcePhase::Acquiring) {
        // The CLI is another client, not a recovery authority. It cannot take
        // over an Application-admitted carrier merely because it can see it.
        return Err("source_attempt_requires_observation_or_resume".into());
    }
    if let Some(prior) = source.progress.as_ref().filter(|p|
        p.phase == SourcePhase::AwaitingReview || (!refresh && matches!(p.phase,
            SourcePhase::Inaccessible | SourcePhase::NeedsProcessing))) {
        let carrier = acquire_carrier(home, store, auth, case, &declaration.source_id, prior.attempt)?;
        let admitted = store.resume_source_attempt_authorized(auth, case,
            &declaration.participant_id, &declaration.source_id, prior.attempt,
            state.generation, &prior.progress_id)?;
        if admitted {
            advance_admitted(home, store, auth, case, name, prior.attempt, &carrier)?;
        }
        return Ok(());
    }
    if !refresh && source.progress.as_ref().is_some_and(|p|
        matches!(p.phase, SourcePhase::Acquired | SourcePhase::Denied)) {
        return Ok(());
    }
    let attempt = source.progress.as_ref().map_or(Some(1), |p| p.attempt.checked_add(1))
        .ok_or("source_attempt_exhausted")?;
    let carrier = acquire_carrier(home, store, auth, case, &declaration.source_id, attempt)?;
    if store.begin_source_attempt_authorized(auth, case, &declaration.participant_id,
        &declaration.source_id, attempt, state.generation)? {
        advance_admitted(home, store, auth, case, name, attempt, &carrier)?;
    }
    Ok(())
}

/// Advance an already durably admitted exact attempt. This never chooses a new
/// attempt and must only be called by the successful admission owner.
pub fn advance_admitted(
    home: &Path,
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    case: &str,
    name: &str,
    attempt: u64,
    carrier: &AcquisitionCarrier,
) -> Result<(), String> {
    let (_, source) = store.case_source_authorized(auth, case, name)?;
    if carrier.case != case || carrier.source != source.declaration.source_id || carrier.attempt != attempt {
        return Err("source_carrier_scope_mismatch".into());
    }
    if !source.progress.as_ref().is_some_and(|p| {
        p.attempt == attempt && p.phase == SourcePhase::Acquiring
    }) {
        return Err("source_attempt_no_longer_dispatchable".into());
    }
    let d = &source.declaration;
    let previous_revision = store.list_case_transitions(case)?.into_iter().rev().find_map(|t| {
        match t.payload {
            TransitionPayload::CaseSourceProgressed { progress }
                if progress.source_id == d.source_id => progress.revision,
            _ => None,
        }
    });
    let result = if d.bootstrap_policy {
        store
            .acquire_bootstrap_policy_source(auth, case, name)
            .map(|r| {
                (
                    SourcePhase::Acquired,
                    Some(r),
                    None,
                    "policy_candidate_not_published".to_string(),
                )
            })
    } else {
        acquire_governed(home, store, auth, case, d, attempt)
    };
    match result {
        Ok((phase, revision, decision, detail)) => {
            let detail = match (
                previous_revision.as_ref(),
                revision.as_ref(),
            ) {
                (Some(old), Some(new)) if old.revision_id == new.revision_id => {
                    "unchanged_revision_reused".to_string()
                }
                (Some(_), Some(_)) => "changed_revision_acquired_old_backing_retained".to_string(),
                _ => detail,
            };
            progress(
                store, auth, case, name, attempt, phase, revision, decision, &detail,
            )
        }
        Err(error) => {
            let detail: String = error
                .split(':')
                .next()
                .unwrap_or("acquisition_failed")
                .chars()
                .filter(|c| !c.is_control())
                .take(256)
                .collect();
            let phase = if error.contains("policy_") || error.contains("unsupported") {
                SourcePhase::NeedsProcessing
            } else {
                SourcePhase::Inaccessible
            };
            progress(store, auth, case, name, attempt, phase, None, None, &detail)
        }
    }
}

type Acquisition = (SourcePhase, Option<SourceRevision>, Option<String>, String);
fn request(
    home: &Path,
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    case: &str,
    d: &CaseSourceDeclaration,
    attempt: u64,
    action: ResourceAction,
) -> Result<ResourceActionOutcome, String> {
    let state = store.get_case_state_authorized(auth, case)?;
    let request = ResourceRequest {
        schema: RESOURCE_REQUEST_SCHEMA.into(),
        configuration_digest: d.configuration_digest.clone(),
        action,
    };
    let base = format!(
        "source-request:{}:{attempt}:{}",
        d.source_id,
        request.digest()
    );
    let history = store.list_case_transitions(case)?;
    let prior = history.iter().rev().find_map(|t| match &t.payload {
        TransitionPayload::OperationRecorded { operation } => match &operation.origin {
            yai_core_engine::effect::OperationOrigin::ParticipantRequest { request_id, .. }
                if request_id == &base || request_id.starts_with(&format!("{base}:retry:")) =>
            {
                Some((operation, request_id))
            }
            _ => None,
        },
        _ => None,
    });
    let id = if let Some((old, id)) = prior {
        let completed = history.iter().any(|t| match &t.payload {
            TransitionPayload::CaseContentAdmitted { admission } => {
                admission.operation_id == old.operation_id
            }
            TransitionPayload::ResourceObservationRecorded { observation } => {
                observation.operation_id == old.operation_id
            }
            _ => false,
        });
        if completed
            || state
                .last_operation
                .as_ref()
                .is_some_and(|o| o.operation_id == old.operation_id)
        {
            id.clone()
        } else {
            format!("{base}:retry:{}", state.generation)
        }
    } else {
        base
    };
    let op = store.record_participant_resource_request(
        auth,
        case,
        &d.participant_id,
        &d.resource_attachment_id,
        &id,
        state.generation,
        request,
    )?;
    // Reused canonical outcomes must not restore revoked authority.
    let outcome_exists = store
        .list_case_transitions(case)?
        .iter()
        .any(|t| match &t.payload {
            TransitionPayload::CaseContentAdmitted { admission } => {
                admission.operation_id == op.operation_id
            }
            TransitionPayload::ResourceObservationRecorded { observation } => {
                observation.operation_id == op.operation_id
            }
            _ => false,
        });
    if outcome_exists
        && store
            .case_source_permission(
                auth,
                case,
                &d.source_id,
                op.resource_request.as_ref().map(|r| r.action.clone()),
            )?
            .outcome
            != DecisionOutcome::Allow
    {
        return Err("source_current_authority_refused".into());
    }
    super::advance(home, store, auth, &op)
}

fn refused(outcome: ResourceActionOutcome) -> Result<Acquisition, String> {
    match outcome {
        ResourceActionOutcome::Denied { decision_id, .. } => Ok((
            SourcePhase::Denied,
            None,
            Some(decision_id),
            "current_policy_denied_acquisition".into(),
        )),
        ResourceActionOutcome::AwaitingReview { decision_id, .. } => Ok((
            SourcePhase::AwaitingReview,
            None,
            Some(decision_id),
            "review_required_no_acquisition".into(),
        )),
        ResourceActionOutcome::Unsupported { .. } => Ok((
            SourcePhase::NeedsProcessing,
            None,
            None,
            "resource_request_unsupported".into(),
        )),
        _ => Err("source_unexpected_resource_result".into()),
    }
}

fn acquire_governed(
    home: &Path,
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    case: &str,
    d: &CaseSourceDeclaration,
    attempt: u64,
) -> Result<Acquisition, String> {
    let outcome = request(home, store, auth, case, d, attempt, d.action.clone())?;
    let ResourceActionOutcome::Observed { observation, .. } = outcome else {
        return refused(outcome);
    };
    let mut items = Vec::new();
    if matches!(d.action, ResourceAction::Discover { .. }) {
        for entry in observation.result["entries"]
            .as_array()
            .ok_or("source_discovery_shape")?
        {
            let path = entry["path"].as_str().ok_or("source_discovery_path")?;
            let digest = entry["digest"].as_str().ok_or("source_discovery_digest")?;
            let prior = store
                .list_case_transitions(case)?
                .into_iter()
                .rev()
                .find_map(|t| match t.payload {
                    TransitionPayload::CaseContentAdmitted { admission }
                        if admission.source_resource_id == d.resource_attachment_id
                            && admission.source_configuration_digest == d.configuration_digest
                            && admission.source_path == path
                            && admission.object.content_digest == digest
                            && admission.participant_ids.contains(&d.participant_id) =>
                    {
                        Some(admission)
                    }
                    _ => None,
                });
            if let Some(admission) = prior {
                // New discovery observed this exact revision. Reuse canonical
                // backing only after fresh admission semantics and integrity.
                if store
                    .case_source_permission(
                        auth,
                        case,
                        &d.source_id,
                        Some(ResourceAction::AdmitContent {
                            path: path.into(),
                            candidate_digest: digest.into(),
                        }),
                    )?
                    .outcome
                    != DecisionOutcome::Allow
                {
                    return Err("source_current_authority_refused".into());
                }
                yai_core_engine::conversation::ConversationContentStore::open(home)?
                    .read_bytes(&admission.object)?;
                items.push(SourceRevisionItem {
                    path: path.into(),
                    digest: admission.object.content_digest,
                    bytes: admission.object.byte_length,
                    backing: SourceBacking::Content {
                        admission_id: admission.admission_id,
                    },
                });
                continue;
            }
            // Canonical request IDs provide crash-safe idempotence per item.
            let outcome = request(
                home,
                store,
                auth,
                case,
                d,
                attempt,
                ResourceAction::AdmitContent {
                    path: path.into(),
                    candidate_digest: digest.into(),
                },
            )?;
            let ResourceActionOutcome::ContentAdmitted { admission, .. } = outcome else {
                return refused(outcome);
            };
            items.push(SourceRevisionItem {
                path: path.into(),
                digest: admission.object.content_digest.clone(),
                bytes: admission.object.byte_length,
                backing: SourceBacking::Content {
                    admission_id: admission.admission_id.clone(),
                },
            });
        }
    } else {
        // This is the exact bounded observation/metadata representation, not
        // a claim to have acquired an entire remote database or service.
        let bytes = serde_json::to_vec(&observation.result).map_err(|e| e.to_string())?;
        items.push(SourceRevisionItem {
            path: d.logical_name.clone(),
            digest: digest_bytes(&bytes),
            bytes: bytes.len() as u64,
            backing: SourceBacking::Observation {
                observation_id: observation.observation_id.clone(),
            },
        });
    }
    Ok((
        SourcePhase::Acquired,
        Some(SourceRevision::new(&d.source_id, items)?),
        Some(observation.decision_id.clone()),
        "exact_material_acquired_no_knowledge_derivation".into(),
    ))
}
