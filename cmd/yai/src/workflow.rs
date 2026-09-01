//! Workflow product surface. Semantic validation and progression stay in the
//! engine workflow owner; this module parses, dispatches, and renders only.

use super::*;
use crate::security::authenticate_local;
use std::time::{SystemTime, UNIX_EPOCH};
use yai_core_engine::workflow::{
    WorkflowDefinitionInput, WorkflowExecutorBinding, WorkflowResourceBinding,
};

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}

fn repeated_slot_bindings(args: &[String], flag: &str) -> Result<Vec<(String, String)>, String> {
    let mut result = Vec::new();
    let mut index = 0usize;
    while index < args.len() {
        if args[index] == flag {
            let value = args
                .get(index + 1)
                .ok_or_else(|| format!("missing value for {flag}"))?;
            let (slot, target) = value
                .split_once('=')
                .ok_or_else(|| format!("{flag} requires <slot>=<id>"))?;
            if slot.is_empty() || target.is_empty() {
                return Err(format!("{flag} requires <slot>=<id>"));
            }
            result.push((slot.to_string(), target.to_string()));
            index += 2;
        } else {
            index += 1;
        }
    }
    Ok(result)
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string_pretty(value)
            .map_err(|error| format!("workflow_json_render_failed: {error}"))?
    );
    Ok(())
}

fn define(args: &[String]) -> Result<(), String> {
    let tenant_id = named_arg(args, "--tenant")?;
    let path = PathBuf::from(named_arg(args, "--file")?);
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let input: WorkflowDefinitionInput = serde_json::from_str(&raw)
        .map_err(|error| format!("invalid WorkflowDefinition JSON: {error}"))?;
    if input.tenant_id != tenant_id {
        return Err("workflow_definition_tenant_argument_mismatch".to_string());
    }
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let definition = store.define_workflow(&authenticated, input, now_unix_ms())?;
    if args.iter().any(|arg| arg == "--json") {
        return print_json(&definition);
    }
    println!("workflow_definition: accepted");
    println!(
        "workflow_definition_id: {}",
        definition.workflow_definition_id
    );
    println!(
        "workflow_definition_digest: {}",
        definition.integrity_digest
    );
    println!("tenant_id: {}", definition.tenant_id);
    println!("workflow_key: {}", definition.workflow_key);
    println!("declared_version: {}", definition.declared_version);
    println!("nodes: {}", definition.nodes.len());
    println!("edges: {}", definition.edges.len());
    Ok(())
}

fn list(args: &[String]) -> Result<(), String> {
    let tenant_id = named_arg(args, "--tenant")?;
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let definitions = store.list_workflow_definitions_authorized(&authenticated, &tenant_id)?;
    if args.iter().any(|arg| arg == "--json") {
        return print_json(&definitions);
    }
    println!("workflow_definitions: {}", definitions.len());
    for definition in definitions {
        println!(
            "workflow: {} key={} version={} nodes={} edges={}",
            definition.workflow_definition_id,
            definition.workflow_key,
            definition.declared_version,
            definition.nodes.len(),
            definition.edges.len()
        );
    }
    Ok(())
}

fn show(args: &[String]) -> Result<(), String> {
    let definition_id = args
        .iter()
        .find(|value| !value.starts_with("--"))
        .ok_or_else(|| "usage: yai workflow show <definition-id> [--json]".to_string())?;
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let definition = store.get_workflow_definition_authorized(&authenticated, definition_id)?;
    print_json(&definition)
}

fn bind(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let definition_id = named_arg(args, "--definition")?;
    let executor_bindings = repeated_slot_bindings(args, "--executor")?
        .into_iter()
        .map(|(slot, participant_id)| WorkflowExecutorBinding {
            slot,
            participant_id,
        })
        .collect();
    let resource_bindings = repeated_slot_bindings(args, "--resource")?
        .into_iter()
        .map(|(slot, attachment_id)| WorkflowResourceBinding {
            slot,
            attachment_id,
        })
        .collect();
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let commit = store.bind_case_workflow(
        &authenticated,
        &case_id,
        &definition_id,
        executor_bindings,
        resource_bindings,
        now_unix_ms(),
    )?;
    let binding = commit
        .state
        .workflow_binding
        .ok_or_else(|| "workflow_binding_missing_after_commit".to_string())?;
    if args.iter().any(|arg| arg == "--json") {
        return print_json(&binding);
    }
    println!("workflow_binding: accepted");
    println!("case_id: {}", binding.case_id);
    println!("workflow_binding_id: {}", binding.binding_id);
    println!("workflow_definition_id: {}", binding.workflow_definition_id);
    println!("case_generation: {}", commit.state.generation);
    Ok(())
}

fn status(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let resolution = store.workflow_status_authorized(&authenticated, &case_id)?;
    if args.iter().any(|arg| arg == "--json") {
        return print_json(&resolution);
    }
    println!(
        "workflow_definition_id: {}",
        resolution.workflow_definition_id
    );
    println!("workflow_binding_id: {}", resolution.workflow_binding_id);
    println!("case_id: {}", resolution.case_id);
    println!("case_generation: {}", resolution.case_generation);
    println!("completed: {}", resolution.completed);
    println!("satisfied: {}", resolution.satisfied_count);
    println!("active: {}", resolution.active_count);
    println!("waiting: {}", resolution.waiting_count);
    println!("skipped: {}", resolution.skipped_count);
    println!("ready: {}", resolution.ready_work.len());
    for node in resolution.nodes {
        println!(
            "node: {} kind={} posture={:?} reason={} execution={}",
            node.node_id,
            node.node_kind,
            node.posture,
            node.reason,
            node.execution_id.as_deref().unwrap_or("none")
        );
    }
    Ok(())
}

fn input(args: &[String]) -> Result<(), String> {
    let case_id = named_arg(args, "--case")?;
    let node_id = named_arg(args, "--node")?;
    let value = named_arg(args, "--value")?;
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let commit = store.record_workflow_human_input(
        &authenticated,
        &case_id,
        &node_id,
        &value,
        now_unix_ms(),
    )?;
    println!("workflow_human_input: accepted");
    println!("case_id: {case_id}");
    println!("node_id: {node_id}");
    println!("case_generation: {}", commit.state.generation);
    println!("review_action_created: false");
    Ok(())
}

pub(super) fn workflow_command(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("define") => define(&args[1..]),
        Some("list") => list(&args[1..]),
        Some("show") => show(&args[1..]),
        Some("bind") => bind(&args[1..]),
        Some("status") => status(&args[1..]),
        Some("input") => input(&args[1..]),
        _ => Err("usage: yai workflow <define|list|show|bind|status|input> ...".to_string()),
    }
}
