//! Guided application input over REPLAI. No terminal mechanics, mutable global
//! Case selection, policy defaults or provider-name inference live here.
use super::super::conversation_controller::{
    admit_workbench_participants, scoped_name, ProviderConnection,
};
use super::super::{record_store_path, security};
use super::*;
use std::io::IsTerminal;
use yai_core_engine::store::lmdb::{LmdbRecordStore, RecordStoreStatusKind};
use yai_core_engine::transition::CaseState;

pub(super) fn require_terminal() -> Result<(), String> {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Err(
            "guided_setup_requires_terminal: use explicit plumbing arguments for automation".into(),
        );
    }
    Ok(())
}

pub(super) fn ask(label: &str, default: Option<&str>) -> Result<String, String> {
    require_terminal()?;
    println!(
        "{}{}",
        label,
        default.map(|v| format!(" [{v}]")).unwrap_or_default()
    );
    let mut interaction = Interaction::new(Editor::new(4096, 0));
    interaction
        .open(
            &std::io::stdin(),
            &std::io::stdout(),
            Prompt::new("setup").map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    loop {
        match interaction
            .poll(Duration::from_millis(100))
            .map_err(|e| e.to_string())?
        {
            Some(Event::Submitted(value)) => {
                let value = value.trim();
                if value.chars().any(char::is_control) {
                    return Err("setup_single_value_required".into());
                }
                return Ok(if value.is_empty() {
                    default.unwrap_or("").into()
                } else {
                    value.into()
                });
            }
            Some(Event::Interrupted | Event::EndOfInput) => {
                return Err("setup_cancelled_no_approval".into())
            }
            Some(Event::Rejected(error)) => interaction
                .external_output(Role::Warning, &error.to_string())
                .map_err(|e| e.to_string())?,
            _ => {}
        }
    }
}

fn confirm(label: &str, word: &str) -> Result<(), String> {
    if ask(
        &format!("{label}\nType {word} to confirm; anything else cancels."),
        None,
    )? != word
    {
        return Err("setup_cancelled_no_approval".into());
    }
    Ok(())
}

fn choose(label: &str, choices: &[String]) -> Result<String, String> {
    if choices.is_empty() {
        return Err(format!("setup_no_choices:{label}"));
    }
    if choices.len() == 1 {
        return Ok(choices[0].clone());
    }
    println!("{label}");
    for (i, value) in choices.iter().enumerate() {
        println!("  {}. {}", i + 1, literal_external_text(value));
    }
    let value = ask("Choose an exact name or number", None)?;
    if choices.contains(&value) {
        return Ok(value);
    }
    value
        .parse::<usize>()
        .ok()
        .and_then(|i| i.checked_sub(1))
        .and_then(|i| choices.get(i))
        .cloned()
        .ok_or("setup_selection_not_in_visible_choices".into())
}

pub(crate) fn initialize(
    tenant: Option<&str>,
    organization: Option<&str>,
) -> Result<(String, String), String> {
    require_terminal()?;
    let authenticated = security::authenticate_local()?;
    if tenant.is_none()
        && organization.is_none()
        && LmdbRecordStore::status(record_store_path()).status == RecordStoreStatusKind::Ready
    {
        let store = LmdbRecordStore::open(record_store_path())?;
        let tenants = store.list_principal_tenants(&authenticated)?;
        if !tenants.is_empty() {
            let id = choose(
                "Existing authorized Tenants",
                &tenants
                    .iter()
                    .map(|t| t.tenant.tenant_id.clone())
                    .collect::<Vec<_>>(),
            )?;
            let relation = tenants
                .iter()
                .find(|t| t.tenant.tenant_id == id)
                .expect("selected Tenant");
            println!("YAI is initialized: {}. Existing identity preserved.", id);
            return Ok((id, relation.tenant.organization_ref.clone()));
        }
    }
    let tenant = match tenant {
        Some(v) => v.into(),
        None => ask("Tenant name", Some("local"))?,
    };
    let organization = match organization {
        Some(v) => v.into(),
        None => ask("Organization name", Some("local"))?,
    };
    let tenant = scoped_name("tenant:", &tenant)?;
    let organization = scoped_name("organization:", &organization)?;
    confirm(
        &format!("Initialize {tenant} / {organization} for the authenticated local Principal?"),
        "create",
    )?;
    Ok((tenant, organization))
}

fn model_candidates(state: &CaseState) -> Vec<String> {
    state
        .participants
        .iter()
        .filter(|p| {
            p.admitted_views
                .iter()
                .any(|v| v.consumer == "model" && v.view_kind == "model_context")
                && !state
                    .principal_participant_links
                    .iter()
                    .any(|l| l.participant_id == p.participant_id)
        })
        .map(|p| p.participant_id.clone())
        .collect()
}

pub(crate) fn open(reference: Option<&str>) -> Result<(), String> {
    require_terminal()?;
    if LmdbRecordStore::status(record_store_path()).status != RecordStoreStatusKind::Ready {
        return Err("YAI is not initialized. Run `yai init` first.".into());
    }
    let authenticated = security::authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let cases = store.list_case_states_authorized(&authenticated, None, 256)?;
    if reference.is_none() && cases.len() == 256 {
        return Err("case_choice_bound: open an exact Case name".into());
    }
    let name = match reference {
        Some(name) => scoped_name("case:", name)?,
        None if cases.is_empty() => scoped_name("case:", &ask("New Case name", None)?)?,
        None => choose(
            "Visible Cases",
            &cases.iter().map(|s| s.case_id.clone()).collect::<Vec<_>>(),
        )?,
    };
    let existing = store.get_case_state_authorized(&authenticated, &name);
    if let Err(error) = &existing {
        if error != "case_not_visible" {
            return Err(error.clone());
        }
    }
    let mut state = if let Ok(state) = existing {
        state
    } else {
        let tenants = store.list_principal_tenants(&authenticated)?;
        let tenant = choose(
            "Tenant for the new Case",
            &tenants
                .iter()
                .filter(|t| {
                    store
                        .resolve_security_context(&authenticated, &t.tenant.tenant_id)
                        .is_ok_and(|c| c.require_owner().is_ok())
                })
                .map(|t| t.tenant.tenant_id.clone())
                .collect::<Vec<_>>(),
        )?;
        confirm(
            &format!(
                "Create {name} in {tenant}? No policy, resources or provider trust are granted."
            ),
            "create",
        )?;
        store
            .create_tenant_case(&authenticated, &tenant, &name)?
            .state
    };
    let principal = authenticated.projected_principal_id();
    let linked = state
        .principal_participant_links
        .iter()
        .find(|l| l.principal_id == principal)
        .map(|l| l.participant_id.clone());
    let models = model_candidates(&state);
    let operator = match linked {
        Some(id) => id,
        None => scoped_name(
            "participant:",
            &ask("Operator Participant name", Some("operator"))?,
        )?,
    };
    let executor = if models.is_empty() {
        scoped_name(
            "participant:",
            &ask(
                "Model Participant name (no provider is connected yet)",
                Some("model"),
            )?,
        )?
    } else {
        choose("Cognitive executor Participant", &models)?
    };
    let complete = state
        .principal_participant_links
        .iter()
        .any(|l| l.principal_id == principal && l.participant_id == operator)
        && state.participants.iter().any(|p| {
            p.participant_id == executor
                && p.admitted_views
                    .iter()
                    .any(|v| v.consumer == "model" && v.view_kind == "model_context")
        });
    if !complete {
        println!("Participant setup at {} generation {}:\n  {operator}: operation-proposer, operation-reviewer, workflow-input; linked to your Principal\n  {executor}: model-executor, operation-proposer; scoped model_context; NO human Principal link\nPolicy, review eligibility, Grants and provider trust remain separate.", state.case_id, state.generation);
        confirm("Admit this Participant configuration?", "admit")?;
        state = admit_workbench_participants(&store, &authenticated, &state, &operator, &executor)?;
        println!("participants_ready: generation {}", state.generation);
    }
    // Release the setup store before the ordinary host opens it. No mutable
    // selected-Case file or shell environment is written.
    drop(store);
    super::run(&[
        "--case".into(),
        name,
        "--subject".into(),
        operator,
        "--executor".into(),
        executor,
    ])
}

pub(super) fn connect(controller: &ConversationController, case_work: bool) -> Result<(), String> {
    use yai_core_engine::provider_governance::ProviderLocality;
    println!(
        "Connect profile: {}. Explicit trust and real mechanical qualification are required.",
        if case_work {
            "workbench (text, native functions, JSON)"
        } else {
            "conversation (text only; use /connect workbench for tools/Workflow)"
        }
    );
    let (generation, primary) = controller.provider_connection_state()?;
    let endpoint = ask(
        "Public provider endpoint (catalog discovery only; no secret URL parameters)",
        None,
    )?;
    let locality =
        if let Some(locality) = super::super::provider_transport::endpoint_locality(&endpoint)? {
            locality
        } else {
            match ask("Locality: loopback / private_network / remote", None)?.as_str() {
                "loopback" => ProviderLocality::Loopback,
                "private_network" => ProviderLocality::PrivateNetwork,
                "remote" => ProviderLocality::Remote,
                _ => return Err("connect_locality_invalid".into()),
            }
        };
    println!("Discovering public model catalog; no inference, trust or Case binding yet.");
    let mut credential = "none".to_string();
    let models = match controller.discover_provider_models(&endpoint, &locality, &credential) {
        Err(error) if error == "provider_catalog_auth_required" => {
            credential = ask("Endpoint requires authentication/access approval. Credential reference env:NAME only; never enter a token", None)?;
            controller.discover_provider_models(&endpoint, &locality, &credential)?
        }
        result => result?,
    };
    let model = choose(
        "Provider-exposed models (catalog is not capability evidence)",
        &models,
    )?;
    println!(
        "Selected model: {}{}",
        literal_external_text(&model),
        if models.len() == 1 {
            " (only catalog entry)"
        } else {
            ""
        }
    );
    println!("Target: {} / {}\nAddress locality: {:?}; credential reference: {}\nCase generation: {generation}\nApproval permits synthetic mechanical probes, trust and a pinned PrimaryConversation binding.\nYou attest this target's semantic suitability as operator_attested, NOT automatic semantic qualification; YAI records the exact provenance. No resource authority is granted.", literal_external_text(&endpoint), literal_external_text(&model), locality, literal_external_text(&credential));
    let replace = primary.is_some();
    if let Some(binding) = primary {
        println!("Existing primary binding: {binding}. This approval explicitly replaces its cognitive policy; it does not recreate the Case.");
    }
    confirm(
        "Confirm this exact connection and operator attestation?",
        if replace { "replace" } else { "approve" },
    )?;
    let result = controller.connect_provider(ProviderConnection {
        endpoint: &endpoint,
        model: &model,
        locality,
        credential_ref: &credential,
        trust_approved: true,
        suitability_ref: None,
        replace,
        case_work,
        expected_generation: Some(generation),
    })?;
    println!(
        "{}",
        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())?
    );
    Ok(())
}

pub(super) fn participants(controller: &mut ConversationController) -> Result<(), String> {
    let auth = security::authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let status = controller.inspect_case(CaseInspection::Status)?;
    let state =
        store.get_case_state_authorized(&auth, status["case"].as_str().ok_or("case_missing")?)?;
    let operator = status["operator"].as_str().ok_or("operator_missing")?;
    let executor = scoped_name(
        "participant:",
        &ask(
            "Model Participant",
            status["executor"].as_str().or(Some("model")),
        )?,
    )?;
    println!("Case {} generation {}\n{operator}: operation-proposer, operation-reviewer, workflow-input; your Principal\n{executor}: model-executor, operation-proposer and model_context; no Principal authority", state.case_id, state.generation);
    confirm(
        "Admit these exact Participant facts? Policy and provider trust remain unchanged.",
        "admit",
    )?;
    let state = admit_workbench_participants(&store, &auth, &state, operator, &executor)?;
    controller.select_executor(&executor)?;
    println!(
        "participants_ready: generation {}; executor: {}",
        state.generation, executor
    );
    Ok(())
}

pub(super) fn review(controller: &ConversationController) -> Result<(), String> {
    use yai_core_engine::transition::ReviewActionKind;
    let reviews = controller.inspect_case(CaseInspection::Reviews)?;
    let pending = reviews
        .as_array()
        .ok_or("review_view_invalid")?
        .iter()
        .filter(|r| matches!(r["status"].as_str(), Some("pending" | "deferred")))
        .collect::<Vec<_>>();
    let id = choose(
        "Pending reviews",
        &pending
            .iter()
            .filter_map(|r| r["review_id"].as_str().map(str::to_string))
            .collect::<Vec<_>>(),
    )?;
    let selected = pending
        .iter()
        .find(|r| r["review_id"].as_str() == Some(&id))
        .ok_or("review_missing")?;
    println!(
        "{}",
        literal_external_text(
            &serde_json::to_string_pretty(
                &controller.inspect_operation(
                    selected["operation_id"]
                        .as_str()
                        .ok_or("review_operation_missing")?
                )?
            )
            .map_err(|e| e.to_string())?
        )
    );
    let action = match ask(
        "Review this exact operation: approve / deny / defer (anything else cancels)",
        None,
    )?
    .as_str()
    {
        "approve" => ReviewActionKind::Approve,
        "deny" => ReviewActionKind::Deny,
        "defer" => ReviewActionKind::Defer,
        _ => return Err("setup_cancelled_no_review".into()),
    };
    let reason = ask("Review reason", None)?;
    let status = controller.inspect_case(CaseInspection::Status)?;
    controller.review_action(
        &id,
        status["operator"].as_str().ok_or("operator_missing")?,
        action,
        &reason,
    )
}
