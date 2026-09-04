//! Invocation-scoped local authentication and Tenant security commands.
//!
//! The CLI observes kernel credentials on every invocation. It never accepts a
//! Principal ID, role, username, or environment variable as authentication.

use super::*;
use std::time::{SystemTime, UNIX_EPOCH};
use yai_core_engine::security::AuthenticatedPrincipal;
use yai_core_engine::transition::{CaseState, PrincipalParticipantLink};

pub(super) fn authenticate_local() -> Result<AuthenticatedPrincipal, String> {
    AuthenticatedPrincipal::authenticate_local()
}

pub(super) fn reject_spoofed_as(
    args: &[String],
    authenticated_principal_id: &str,
) -> Result<(), String> {
    if let Some(claim) = optional_arg(args, "--as") {
        if claim != authenticated_principal_id {
            return Err("caller_supplied_as_cannot_authenticate_principal".to_string());
        }
    }
    Ok(())
}

/// Compatibility reads may inspect an unscoped historical Case. Every scoped
/// Case read resolves the current kernel Principal and Tenant membership before
/// returning any Case-derived body.
pub(super) fn authorize_case_read_if_scoped(
    store: &LmdbRecordStore,
    case_id: &str,
) -> Result<CaseState, String> {
    let state = store
        .get_case_state(case_id)?
        .ok_or_else(|| "case_not_visible".to_string())?;
    if state.tenant_id.is_some() {
        let authenticated = authenticate_local()?;
        store.get_case_state_authorized(&authenticated, case_id)
    } else {
        Ok(state)
    }
}

fn now_unix_ms() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis() as u64)
        .map_err(|error| format!("system_clock_before_unix_epoch: {error}"))
}

pub(super) fn security_command(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("bootstrap-local") => {
            let tenant_id = named_arg(&args[1..], "--tenant")?;
            let organization_ref = named_arg(&args[1..], "--organization")?;
            let authenticated = authenticate_local()?;
            let store = LmdbRecordStore::open(record_store_path())?;
            let outcome = store.bootstrap_local_security(
                &authenticated,
                &tenant_id,
                &organization_ref,
                now_unix_ms()?,
            )?;
            println!("security_bootstrap: {}", if outcome.created { "created" } else { "already_exists" });
            println!("authenticated: true");
            println!("authentication_kind: {}", authenticated.authentication_method());
            println!("real_uid: {}", authenticated.binding().real_uid);
            println!("effective_uid: {}", authenticated.binding().effective_uid);
            println!("real_gid: {}", authenticated.binding().real_gid);
            println!("effective_gid: {}", authenticated.binding().effective_gid);
            println!("principal_id: {}", outcome.principal.principal_id);
            println!("tenant_id: {}", outcome.tenant.tenant_id);
            println!("organization_ref: {}", outcome.tenant.organization_ref);
            println!("membership: owner");
            println!("security_events: {}", outcome.events.len());
            Ok(())
        }
        _ => Err("usage: yai security bootstrap-local --tenant <tenant:id> --organization <organization:id>".to_string()),
    }
}

pub(super) fn identity_command(args: &[String]) -> Result<(), String> {
    if args.first().map(String::as_str) != Some("whoami") {
        return Err("usage: yai identity whoami".to_string());
    }
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let principal = store.enrolled_principal(&authenticated)?;
    let relations = store.list_principal_tenants(&authenticated)?;
    println!("authenticated: true");
    println!("principal_id: {}", principal.principal_id);
    println!("authn_method: {}", principal.authentication_method);
    println!("binding_ref: {}", authenticated.binding().binding_ref);
    println!("real_uid: {}", authenticated.binding().real_uid);
    println!("effective_uid: {}", authenticated.binding().effective_uid);
    println!("real_gid: {}", authenticated.binding().real_gid);
    println!("effective_gid: {}", authenticated.binding().effective_gid);
    println!(
        "credential_mismatch: {}",
        authenticated.binding().real_uid != authenticated.binding().effective_uid
            || authenticated.binding().real_gid != authenticated.binding().effective_gid
    );
    println!("tenant_relations: {}", relations.len());
    for relation in relations {
        println!(
            "tenant: {} membership={:?} organization_ref={}",
            relation.tenant.tenant_id, relation.membership, relation.tenant.organization_ref
        );
    }
    Ok(())
}

pub(super) fn tenant_command(args: &[String]) -> Result<(), String> {
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    match args.first().map(String::as_str) {
        Some("list") => {
            let relations = store.list_principal_tenants(&authenticated)?;
            println!("tenants_total: {}", relations.len());
            for relation in relations {
                println!("tenant_id: {}", relation.tenant.tenant_id);
                println!("organization_ref: {}", relation.tenant.organization_ref);
                println!("membership: {:?}", relation.membership);
            }
            Ok(())
        }
        Some("status") => {
            let tenant_id = named_arg(&args[1..], "--tenant")?;
            let context = store.resolve_security_context(&authenticated, &tenant_id)?;
            let tenant = store.get_tenant(&tenant_id)?.ok_or_else(|| "tenant_not_visible".to_string())?;
            println!("tenant_id: {}", tenant.tenant_id);
            println!("organization_ref: {}", tenant.organization_ref);
            println!("owner_principal_id: {}", tenant.owner_principal_id);
            println!("current_principal_id: {}", context.principal_id());
            println!("membership: {:?}", context.membership());
            Ok(())
        }
        Some("add-member") => {
            let tenant_id = named_arg(&args[1..], "--tenant")?;
            let principal_id = named_arg(&args[1..], "--principal")?;
            let event = store.add_tenant_member(
                &authenticated,
                &tenant_id,
                &principal_id,
                now_unix_ms()?,
            )?;
            println!("tenant_member_add: committed");
            println!("tenant_id: {tenant_id}");
            println!("principal_id: {principal_id}");
            println!("membership: member");
            println!("security_event_id: {}", event.event_id);
            Ok(())
        }
        _ => Err("usage: yai tenant list | yai tenant status --tenant <tenant:id> | yai tenant add-member --tenant <tenant:id> --principal <principal:id>".to_string()),
    }
}

pub(super) fn case_security_command(args: &[String]) -> Result<(), String> {
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    match args.first().map(String::as_str) {
        Some("create") => {
            let case_id = named_arg(&args[1..], "--case")?;
            let tenant_id = named_arg(&args[1..], "--tenant")?;
            let commit = store.create_tenant_case(&authenticated, &tenant_id, &case_id)?;
            println!("case_created: true");
            println!("case_id: {}", commit.state.case_id);
            println!("tenant_id: {}", commit.state.tenant_id.as_deref().unwrap_or("none"));
            println!("case_generation: {}", commit.state.generation);
            println!("lifecycle: open");
            println!("principal_id: {}", authenticated.projected_principal_id());
            Ok(())
        }
        Some("principal") if args.get(1).map(String::as_str) == Some("link") => {
            let case_id = named_arg(&args[2..], "--case")?;
            let requested_principal_id = named_arg(&args[2..], "--principal")?;
            let principal_id = if requested_principal_id == "self" {
                authenticated.projected_principal_id()
            } else {
                requested_principal_id
            };
            let participant_id = named_arg(&args[2..], "--participant")?;
            let state = store.get_case_state_authorized(&authenticated, &case_id)?;
            let tenant_id = state.tenant_id.clone().ok_or_else(|| "legacy_unscoped_case_cannot_accept_principal_link".to_string())?;
            let creator = authenticated.projected_principal_id();
            let link = PrincipalParticipantLink::new(
                &case_id,
                &tenant_id,
                &principal_id,
                &participant_id,
                &creator,
                now_unix_ms()?,
            )?;
            let mut pending = PendingTransition::new(
                format!("transition:{}", link.link_id),
                &case_id,
                state.generation,
                TransitionSource {
                    component: "yai.case_security".to_string(),
                    participant_id: None,
                    principal_id: Some(creator.clone()),
                    source_ref: Some(link.link_id.clone()),
                },
                TransitionPayload::ParticipantPrincipalLinked { link: link.clone() },
            );
            pending.causal_refs = vec![link.principal_id.clone(), link.participant_id.clone()];
            let commit = store.commit_secured_transition(
                &authenticated,
                &tenant_id,
                pending,
                true,
            )?;
            println!("principal_participant_linked: true");
            println!("case_id: {case_id}");
            println!("tenant_id: {tenant_id}");
            println!("principal_id: {principal_id}");
            println!("participant_id: {participant_id}");
            println!("link_id: {}", link.link_id);
            println!("case_generation: {}", commit.state.generation);
            Ok(())
        }
        _ => Err("usage: yai case create --case <case:id> --tenant <tenant:id> | yai case principal link --case <case:id> --principal <principal:id> --participant <participant:id>".to_string()),
    }
}
