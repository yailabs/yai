//! Workflow product surface. Semantic validation and progression stay in the
//! engine workflow owner; this module parses, dispatches, and renders only.

use super::*;
use crate::command_adapters::security::authenticate_local;
use std::time::{SystemTime, UNIX_EPOCH};
use yai_core_engine::workflow::{
    WorkflowCaseBinding, WorkflowDefinitionInput, WorkflowExecutorBinding, WorkflowPlanPatchInput,
    WorkflowResourceBinding,
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
    let case_bindings = repeated_slot_bindings(args, "--case-slot")?
        .into_iter()
        .map(|(slot, case_id)| WorkflowCaseBinding { slot, case_id })
        .collect();
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let commit = store.bind_case_workflow_composed(
        &authenticated,
        &case_id,
        &definition_id,
        executor_bindings,
        resource_bindings,
        case_bindings,
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
    println!("effective_revision: {}", resolution.effective_revision);
    println!(
        "effective_topology_digest: {}",
        resolution.effective_topology_digest
    );
    println!("amendments: {}", resolution.amendment_ids.len());
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

fn patch_command(args: &[String]) -> Result<(), String> {
    let operation = args.first().map(String::as_str).ok_or_else(|| {
        "usage: yai workflow patch <propose|list|show|validate|adopt>".to_string()
    })?;
    let args = &args[1..];
    let case_id = named_arg(args, "--case")?;
    let authenticated = authenticate_local()?;
    let store = LmdbRecordStore::open(record_store_path())?;
    match operation {
        "propose" => {
            let path = PathBuf::from(named_arg(args, "--file")?);
            let raw = fs::read_to_string(&path)
                .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
            let input: WorkflowPlanPatchInput = serde_json::from_str(&raw)
                .map_err(|error| format!("invalid WorkflowPlanPatch JSON: {error}"))?;
            let commit = store.propose_workflow_plan_patch_human(
                &authenticated,
                &case_id,
                input,
                now_unix_ms(),
            )?;
            let patch = commit
                .state
                .workflow_plan_patches
                .last()
                .ok_or_else(|| "workflow_plan_patch_missing_after_commit".to_string())?;
            if args.iter().any(|arg| arg == "--json") {
                print_json(patch)
            } else {
                println!("workflow_plan_patch: proposed");
                println!("patch_id: {}", patch.patch_id);
                println!("base_revision: {}", patch.base_revision);
                println!(
                    "base_topology_digest: {}",
                    patch.base_effective_topology_digest
                );
                println!("operations: {}", patch.operations.len());
                Ok(())
            }
        }
        "propose-model" => {
            let provider_result_id = named_arg(args, "--provider-result")?;
            let commit = store.propose_workflow_plan_patch_from_provider_result(
                &authenticated,
                &case_id,
                &provider_result_id,
                now_unix_ms(),
            )?;
            let patch = commit
                .state
                .workflow_plan_patches
                .last()
                .ok_or_else(|| "workflow_plan_patch_missing_after_commit".to_string())?;
            if args.iter().any(|arg| arg == "--json") {
                print_json(patch)
            } else {
                println!("workflow_plan_patch: proposed_by_model_result");
                println!("patch_id: {}", patch.patch_id);
                println!("provider_result_id: {provider_result_id}");
                println!("adopted: false");
                Ok(())
            }
        }
        "list" => {
            let state = store.get_case_state_authorized(&authenticated, &case_id)?;
            if args.iter().any(|arg| arg == "--json") {
                return print_json(&state.workflow_plan_patches);
            }
            println!(
                "workflow_plan_patches: {}",
                state.workflow_plan_patches.len()
            );
            for patch in state.workflow_plan_patches {
                let adopted = state
                    .workflow_amendments
                    .iter()
                    .any(|amendment| amendment.patch_id == patch.patch_id);
                println!(
                    "patch: {} base_revision={} operations={} adopted={}",
                    patch.patch_id,
                    patch.base_revision,
                    patch.operations.len(),
                    adopted
                );
            }
            Ok(())
        }
        "show" => {
            let patch_id = named_arg(args, "--patch")?;
            let state = store.get_case_state_authorized(&authenticated, &case_id)?;
            let patch = state
                .workflow_plan_patches
                .iter()
                .find(|patch| patch.patch_id == patch_id)
                .ok_or_else(|| "workflow_plan_patch_not_found".to_string())?;
            print_json(patch)
        }
        "validate" => {
            let patch_id = named_arg(args, "--patch")?;
            let topology = store.validate_workflow_plan_patch_authorized(
                &authenticated,
                &case_id,
                &patch_id,
            )?;
            if args.iter().any(|arg| arg == "--json") {
                print_json(&topology)
            } else {
                println!("workflow_plan_patch: valid");
                println!("patch_id: {patch_id}");
                println!("resulting_revision: {}", topology.revision);
                println!("resulting_topology_digest: {}", topology.topology_digest);
                Ok(())
            }
        }
        "adopt" => {
            let patch_id = named_arg(args, "--patch")?;
            let commit = store.adopt_workflow_plan_patch(
                &authenticated,
                &case_id,
                &patch_id,
                now_unix_ms(),
            )?;
            let amendment = commit
                .state
                .workflow_amendments
                .last()
                .ok_or_else(|| "workflow_amendment_missing_after_commit".to_string())?;
            if args.iter().any(|arg| arg == "--json") {
                print_json(amendment)
            } else {
                println!("workflow_amendment: adopted");
                println!("amendment_id: {}", amendment.amendment_id);
                println!("patch_id: {}", amendment.patch_id);
                println!("revision: {}", amendment.revision);
                println!("topology_digest: {}", amendment.resulting_topology_digest);
                Ok(())
            }
        }
        _ => Err(
            "usage: yai workflow patch <propose|propose-model|list|show|validate|adopt>"
                .to_string(),
        ),
    }
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
        Some("patch") => patch_command(&args[1..]),
        _ => Err("usage: yai workflow <define|list|show|bind|status|input|patch> ...".to_string()),
    }
}
