//! Unified product source frontier over Resource, policy intake and Case history.
use super::*;
use yai_application::resource_execution::source::{acquire, progress};
use serde::Deserialize;
use serde_json::{json, Value};
use yai_core_engine::effect::access::source::*;
use yai_core_engine::effect::access::ResourceAction;
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
                    &yai_home(),
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
            store.publish_case_source_policy_authorized(
                &auth,
                &case,
                &named_arg(args, "--source")?,
                &named_arg(args, "--reason")?,
            )?;
        }
        "yai.case.sources.routes" => {
            let view = store.case_source_routing_authorized(
                &auth,
                &case,
                &named_arg(args, "--source")?,
                optional_arg(args, "--revision").as_deref(),
                &yai_core_engine::conversation::ConversationContentStore::open(&yai_home())?,
            )?;
            return serde_json::to_value(view).map_err(|e| e.to_string());
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
        let material_revision_id = source.progress.as_ref().and_then(|progress| {
            progress
                .revision
                .as_ref()
                .map(SourceRevision::material_revision_id)
        });
        items.push(json!({"name":d.logical_name,"source_id":d.source_id,"perimeter":d.perimeter,"roles":d.roles,
            "resource":d.resource_attachment_id,"action":d.action,"bootstrap_policy":d.bootstrap_policy,"media_type":d.media_type,
            "phase":phase,"progress":source.progress,"material_revision_id":material_revision_id,
            "available_under_current_authority":available}));
    }
    let status = store.case_policy_status(case)?;
    let complete = !items.is_empty() && items.iter().all(|s| s["phase"] == "acquired");
    Ok(
        json!({"schema":"yai.source_inventory.v2","case_id":case,"generation":state.generation,
        "effective_policy":status,"coverage":{"denominator":"explicit_declared_sources_only_not_company_coverage","declared":items.len(),"counts":counts,
        "acquisition_complete":complete,"knowledge_derivation":"not_performed_by_acquisition"},"sources":items}),
    )
}

fn read(
    store: &LmdbRecordStore,
    auth: &AuthenticatedPrincipal,
    case: &str,
    name: &str,
    revision: Option<&str>,
) -> Result<Value, String> {
    let resolved = store.resolve_case_source_authorized(
        auth,
        case,
        name,
        revision,
        &yai_core_engine::conversation::ConversationContentStore::open(&yai_home())?,
    )?;
    let output: Vec<_> = resolved.items.iter().map(|(item, bytes)| json!({
        "path":item.path,"digest":item.digest,"bytes":item.bytes,"backing":item.backing,
        "text":std::str::from_utf8(bytes).ok(),
        "binary_posture":if std::str::from_utf8(bytes).is_err() {"exact_binary_backing_not_rendered"} else {"text"}
    })).collect();
    Ok(
        json!({"schema":"yai.source_read.v2","case_id":case,"source_id":resolved.declaration.source_id,
        "revision_id":resolved.revision.revision_id,
        "material_revision_id":resolved.revision.material_revision_id(),
        "items":output,"authority":"current_policy_only"}),
    )
}

pub(crate) fn render(value: &Value) {
    if value["schema"] == "yai.source_routing.v2" {
        println!(
            "Case source routing — {}\n{}\n{}",
            value["case_id"], value["source_id"], value["revision_id"]
        );
        for item in value["items"].as_array().into_iter().flatten() {
            println!("{} roles={}", item[0]["path"], item[1]["roles"]);
            for region in item[1]["regions"].as_array().into_iter().flatten() {
                println!(
                    "  {}  {}  {}",
                    region["location"], region["routes"], region["id"]
                );
            }
        }
        println!("Routes are derived eligibility, never publication or current authority.");
        return;
    }
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
