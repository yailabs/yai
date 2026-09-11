//! Unified product source frontier over Resource, policy intake and Case history.
use super::*;
use access::ResourceActionOutcome;
use serde::Deserialize;
use serde_json::{json, Value};
use yai_core_engine::effect::access::source::*;
use yai_core_engine::effect::access::{ResourceAction, ResourceRequest, RESOURCE_REQUEST_SCHEMA};
use yai_core_engine::effect::digest_bytes;
use yai_core_engine::security::AuthenticatedPrincipal;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Perimeter {
    schema: String,
    name: String,
    participant: String,
    #[serde(default)]
    resources: Vec<access::ResourceDefinition>,
    sources: Vec<SourceInput>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceInput {
    name: String,
    resource: String,
    roles: Vec<SourceRole>,
    action: ResourceAction,
    media_type: String,
    #[serde(default)]
    bootstrap_policy: bool,
}

pub(crate) fn command(id: &str, args: &[String]) -> Result<Value, String> {
    let case = named_arg(args, "--case")?;
    let auth = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let state = store.get_case_state_authorized(&auth, &case)?;
    store
        .resolve_security_context(
            &auth,
            state.tenant_id.as_deref().ok_or("source_requires_tenant")?,
        )?
        .require_owner()?;
    match id {
        "yai.case.sources.declare" => {
            use std::io::Read;
            let path = named_arg(args, "--file")?;
            let mut options = fs::OpenOptions::new();
            options.read(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.custom_flags(libc::O_NONBLOCK);
            }
            let file = options
                .open(path)
                .map_err(|e| format!("perimeter_input:{e}"))?;
            if !file.metadata().map_err(|e| e.to_string())?.is_file() {
                return Err("perimeter_regular_file_required".into());
            }
            let mut bytes = Vec::new();
            file.take(65537)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            if bytes.len() > 65536 {
                return Err("perimeter_input_bound".into());
            }
            let input: Perimeter =
                serde_json::from_slice(&bytes).map_err(|e| format!("perimeter_input:{e}"))?;
            if input.schema != "yai.source_perimeter.v1"
                || input.sources.is_empty()
                || input.sources.len() > MAX_CASE_SOURCES
                || input.resources.len() > MAX_CASE_SOURCES
                || input.name.is_empty()
                || input.name.len() > 256
            {
                return Err("perimeter_contract_invalid".into());
            }
            let names: std::collections::BTreeSet<_> =
                input.sources.iter().map(|s| &s.name).collect();
            if names.len() != input.sources.len() {
                return Err("perimeter_duplicate_logical_name".into());
            }
            for resource in input.resources {
                access::import_definition(&store, &auth, &case, resource)?;
            }
            for source in input.sources {
                let binding = store
                    .get_local_access_binding(&case, &source.resource)?
                    .ok_or("source_resource_not_attached")?;
                let declaration = CaseSourceDeclaration {
                    schema: SOURCE_DECLARATION_SCHEMA.into(),
                    source_id: String::new(),
                    case_id: case.clone(),
                    perimeter: input.name.clone(),
                    logical_name: source.name,
                    participant_id: input.participant.clone(),
                    declared_by_principal_id: auth.projected_principal_id(),
                    resource_attachment_id: source.resource,
                    configuration_digest: binding.digest(),
                    roles: source.roles,
                    action: source.action,
                    bootstrap_policy: source.bootstrap_policy,
                    media_type: source.media_type,
                }
                .seal()?;
                store.declare_case_source(&auth, declaration)?;
            }
        }
        "yai.case.sources.acquire" | "yai.case.sources.resume" => {
            let limit = optional_arg(args, "--limit")
                .map(|s| {
                    s.parse::<usize>()
                        .map_err(|_| "source_limit_invalid".to_string())
                })
                .transpose()?
                .unwrap_or(MAX_CASE_SOURCES);
            if limit == 0 || limit > MAX_CASE_SOURCES {
                return Err("source_limit_invalid".into());
            }
            let requested = optional_arg(args, "--source");
            let refresh = args.iter().any(|a| a == "--refresh");
            if refresh && requested.is_none() {
                return Err("source_refresh_requires_exact_source".into());
            }
            let mut sources: Vec<_> = state
                .sources
                .into_iter()
                .filter(|s| s.declaration.declared_by_principal_id == auth.projected_principal_id())
                .collect();
            sources.sort_by_key(|s| {
                (
                    !s.declaration.bootstrap_policy,
                    s.declaration.logical_name.clone(),
                )
            });
            if let Some(name) = &requested {
                store.case_source_authorized(&auth, &case, name)?;
            }
            let ready = store.case_policy_status(&case)?.readiness == NormativeReadiness::Ready;
            let mut completed = 0;
            for source in sources {
                if requested.as_ref().is_some_and(|n| {
                    *n != source.declaration.logical_name && *n != source.declaration.source_id
                }) {
                    continue;
                }
                if source.progress.as_ref().is_some_and(|p| {
                    p.phase == SourcePhase::Revoked
                        || (matches!(
                            p.phase,
                            SourcePhase::Acquired
                                | SourcePhase::Denied
                                | SourcePhase::NeedsProcessing
                        ) && !refresh)
                }) {
                    continue;
                }
                if !source.declaration.bootstrap_policy && !ready {
                    continue;
                }
                if completed == limit {
                    break;
                }
                acquire(
                    &store,
                    &auth,
                    &case,
                    &source.declaration.logical_name,
                    refresh,
                )?;
                completed += 1;
            }
        }
        "yai.case.sources.publish" => {
            let name = named_arg(args, "--source")?;
            let reason = named_arg(args, "--reason")?;
            let (state, source) = store.case_source_authorized(&auth, &case, &name)?;
            if !source.declaration.roles.contains(&SourceRole::Policy) {
                return Err("knowledge_source_is_not_policy".into());
            }
            if state.sources.iter().any(|s| {
                s.declaration.bootstrap_policy
                    && !s
                        .progress
                        .as_ref()
                        .is_some_and(|p| p.phase == SourcePhase::Acquired)
            }) {
                return Err(
                    "all_bootstrap_policy_sources_must_be_acquired_before_publication".into(),
                );
            }
            let revision = source
                .progress
                .as_ref()
                .and_then(|p| p.revision.as_ref())
                .ok_or("policy_source_not_acquired")?;
            for item in &revision.items {
                let SourceBacking::Policy { artifact_id, .. } = &item.backing else {
                    return Err("source_has_no_policy_candidate".into());
                };
                store.validate_tenant_policy_artifact(&auth, artifact_id, &reason)?;
                store.publish_tenant_policy_artifact(&auth, artifact_id, &reason)?;
                let current = store.get_case_state_authorized(&auth, &case)?;
                if !current
                    .policy_bindings
                    .iter()
                    .any(|b| b.artifact_id == *artifact_id)
                {
                    store.bind_tenant_case_policy(
                        &auth,
                        &case,
                        artifact_id,
                        current.generation,
                        &reason,
                    )?;
                }
            }
        }
        "yai.case.sources.revoke" => {
            let name = named_arg(args, "--source")?;
            let reason = named_arg(args, "--reason")?;
            let (_, source) = store.case_source_authorized(&auth, &case, &name)?;
            if !source
                .progress
                .as_ref()
                .is_some_and(|p| p.phase == SourcePhase::Revoked)
            {
                progress(
                    &store,
                    &auth,
                    &case,
                    &name,
                    source.progress.as_ref().map_or(1, |p| p.attempt),
                    SourcePhase::Revoked,
                    None,
                    None,
                    &reason,
                )?;
            }
        }
        "yai.case.sources.read" => {
            return read(
                &store,
                &auth,
                &case,
                &named_arg(args, "--source")?,
                optional_arg(args, "--revision").as_deref(),
            )
        }
        "yai.case.sources.inventory" => {}
        _ => return Err("source_operation_unknown".into()),
    }
    inventory(&store, &auth, &case)
}

fn progress(
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

fn acquire(
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    case: &str,
    name: &str,
    refresh: bool,
) -> Result<(), String> {
    let (_, source) = store.case_source_authorized(auth, case, name)?;
    let d = &source.declaration;
    let attempt = source.progress.as_ref().map_or(1, |p| {
        p.attempt
            + u64::from(
                p.phase != SourcePhase::Acquiring
                    && (refresh
                        || matches!(
                            p.phase,
                            SourcePhase::Inaccessible
                                | SourcePhase::NeedsProcessing
                                | SourcePhase::Denied
                        )),
            )
    });
    if !source
        .progress
        .as_ref()
        .is_some_and(|p| p.phase == SourcePhase::Acquiring && p.attempt == attempt)
    {
        progress(
            store,
            auth,
            case,
            name,
            attempt,
            SourcePhase::Acquiring,
            None,
            None,
            "acquisition_started",
        )?;
    }
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
        acquire_governed(store, auth, case, d, attempt)
    };
    match result {
        Ok((phase, revision, decision, detail)) => {
            let detail = match (
                source.progress.as_ref().and_then(|p| p.revision.as_ref()),
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
    access::advance(store, auth, &op)
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
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    case: &str,
    d: &CaseSourceDeclaration,
    attempt: u64,
) -> Result<Acquisition, String> {
    let outcome = request(store, auth, case, d, attempt, d.action.clone())?;
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
                yai_core_engine::conversation::ConversationContentStore::open(&yai_home())?
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

fn inventory(
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    case: &str,
) -> Result<Value, String> {
    let state = store.get_case_state_authorized(auth, case)?;
    let mut items = Vec::new();
    let mut counts = std::collections::BTreeMap::<String, usize>::new();
    for source in &state.sources {
        let d = &source.declaration;
        if d.declared_by_principal_id != auth.projected_principal_id() {
            continue;
        }
        let phase = source
            .progress
            .as_ref()
            .map(|p| serde_json::to_value(&p.phase).unwrap())
            .unwrap_or(json!("discovered"));
        *counts.entry(phase.as_str().unwrap().into()).or_default() += 1;
        let available = source
            .progress
            .as_ref()
            .is_some_and(|p| p.phase == SourcePhase::Acquired)
            && store
                .case_source_permission(auth, case, &d.source_id, None)
                .is_ok_and(|d| d.outcome == DecisionOutcome::Allow);
        items.push(json!({"name":d.logical_name,"source_id":d.source_id,"perimeter":d.perimeter,"roles":d.roles,
            "resource":d.resource_attachment_id,"action":d.action,"bootstrap_policy":d.bootstrap_policy,"media_type":d.media_type,
            "phase":phase,"progress":source.progress,"available_under_current_authority":available}));
    }
    let status = store.case_policy_status(case)?;
    let complete = !items.is_empty() && items.iter().all(|s| s["phase"] == "acquired");
    Ok(
        json!({"schema":"yai.source_inventory.v1","case_id":case,"generation":state.generation,
        "effective_policy":status,"coverage":{"denominator":"explicit_declared_sources_only_not_company_coverage","declared":items.len(),"counts":counts,
        "acquisition_complete":complete,"knowledge_derivation":"not_implemented"},"sources":items}),
    )
}

fn read(
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    case: &str,
    name: &str,
    revision: Option<&str>,
) -> Result<Value, String> {
    let (state, source) = store.case_source_authorized(auth, case, name)?;
    if store
        .case_source_permission(auth, case, name, None)?
        .outcome
        != DecisionOutcome::Allow
    {
        return Err("source_not_available".into());
    }
    let history = store.list_case_transitions(case)?;
    let captured = if let Some(id) = revision {
        history.iter().rev().find_map(|t| match &t.payload {
            TransitionPayload::CaseSourceProgressed { progress }
                if progress.source_id == source.declaration.source_id =>
            {
                progress
                    .revision
                    .as_ref()
                    .filter(|r| r.revision_id == id)
                    .cloned()
            }
            _ => None,
        })
    } else {
        source.progress.and_then(|p| p.revision)
    }
    .ok_or("source_revision_not_available")?;
    let mut output = Vec::new();
    for item in &captured.items {
        let bytes = match &item.backing {
            SourceBacking::Policy { source_id, .. } => store
                .get_policy_source_authorized(auth, state.tenant_id.as_deref().unwrap(), source_id)?
                .original_bytes()
                .to_vec(),
            SourceBacking::Content { admission_id } => {
                if store
                    .case_source_permission(
                        auth,
                        case,
                        name,
                        Some(ResourceAction::ContentRead {
                            admission_id: admission_id.clone(),
                        }),
                    )?
                    .outcome
                    != DecisionOutcome::Allow
                {
                    return Err("source_not_available".into());
                }
                let admission = history
                    .iter()
                    .find_map(|t| match &t.payload {
                        TransitionPayload::CaseContentAdmitted { admission }
                            if admission.admission_id == *admission_id =>
                        {
                            Some(admission)
                        }
                        _ => None,
                    })
                    .ok_or("source_backing_unavailable")?;
                yai_core_engine::conversation::ConversationContentStore::open(&yai_home())?
                    .read_bytes(&admission.object)?
            }
            SourceBacking::Observation { observation_id } => {
                let o = history
                    .iter()
                    .find_map(|t| match &t.payload {
                        TransitionPayload::ResourceObservationRecorded { observation }
                            if observation.observation_id == *observation_id =>
                        {
                            Some(observation)
                        }
                        _ => None,
                    })
                    .ok_or("source_backing_unavailable")?;
                serde_json::to_vec(&o.result).map_err(|e| e.to_string())?
            }
        };
        if digest_bytes(&bytes) != item.digest || bytes.len() as u64 != item.bytes {
            return Err("source_backing_integrity_mismatch".into());
        }
        output.push(json!({"path":item.path,"digest":item.digest,"bytes":item.bytes,"backing":item.backing,"text":std::str::from_utf8(&bytes).ok(),
            "binary_posture":if std::str::from_utf8(&bytes).is_err() { "exact_binary_backing_not_rendered" } else { "text" }}));
    }
    if store.get_case_state_authorized(auth, case)?.generation != state.generation
        || store
            .case_source_permission(auth, case, name, None)?
            .outcome
            != DecisionOutcome::Allow
    {
        return Err("source_visibility_changed_during_read".into());
    }
    Ok(
        json!({"case_id":case,"source_id":source.declaration.source_id,"revision_id":captured.revision_id,"items":output,"authority":"current_policy_only"}),
    )
}

pub(crate) fn render(value: &Value) {
    println!(
        "Case sources — {} @ {}",
        value["case_id"].as_str().unwrap_or(""),
        value["generation"]
    );
    if let Some(sources) = value["sources"].as_array() {
        for s in sources {
            println!(
                "{}  {}  roles={}  current-access={}\n  {}\n  {}",
                s["name"].as_str().unwrap_or(""),
                s["phase"].as_str().unwrap_or(""),
                s["roles"],
                s["available_under_current_authority"],
                s["source_id"].as_str().unwrap_or(""),
                s["progress"]["detail"].as_str().unwrap_or("pending")
            );
        }
        println!("Governance: {}\nCoverage: {}\nAcquired material is not derived knowledge. Use --json for exact revisions and decision references.", value["effective_policy"]["readiness"], value["coverage"]);
    } else {
        println!("{}", serde_json::to_string_pretty(value).unwrap());
    }
}
