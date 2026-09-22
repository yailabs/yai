//! CLI syntax only; reusable Resource orchestration belongs to Application.
use super::*;
use serde_json::{json, Value};
pub(crate) use yai_application::resource_execution::{import_definition, ResourceDefinition, ResourceActionOutcome};

#[cfg(test)]
pub(crate) fn attach(
    store: &LmdbRecordStore,
    authenticated: &yai_core_engine::security::AuthenticatedPrincipal,
    binding: &yai_core_engine::effect::access::LocalAccessBinding,
    access: yai_core_engine::effect::access::ResourceAccessContract,
    policy_owner: &str,
    write_envelope: Option<(&str, usize)>,
) -> Result<CaseState, String> {
    yai_application::resource_execution::attach(
        store, authenticated, binding, access, policy_owner, write_envelope,
        ReviewRequirement::Automatic,
    ).map(|result| result.state)
}

pub(crate) fn advance(
    store: &LmdbRecordStore,
    authenticated: &yai_core_engine::security::AuthenticatedPrincipal,
    operation: &Operation,
) -> Result<ResourceActionOutcome, String> {
    yai_application::resource_execution::advance(&yai_home(), store, authenticated, operation)
}

pub(crate) fn command(operation_id: &str, args: &[String]) -> Result<Value, String> {
    use std::io::Read;
    let case_id = named_arg(args, "--case")?;
    let store = LmdbRecordStore::open(record_store_path())?;
    let authenticated = authenticate_local()?;
    let state = store.get_case_state_authorized(&authenticated, &case_id)?;
    let input: Value = {
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
            .map_err(|e| format!("resource_input_open:{e}"))?;
        if !file.metadata().map_err(|e| e.to_string())?.is_file() {
            return Err("resource_input_regular_file_required".into());
        }
        let mut bytes = Vec::new();
        file.take(65_537)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 65_536 {
            return Err("resource_input_bound_exceeded".into());
        }
        serde_json::from_slice(&bytes).map_err(|e| format!("resource_input_json:{e}"))?
    };
    match operation_id {
        "yai.case.resource.import" => {
            let definition =
                serde_json::from_value(input).map_err(|e| format!("resource_definition:{e}"))?;
            let state = import_definition(&store, &authenticated, &case_id, definition)?;
            Ok(
                json!({"case_id":case_id,"generation":state.generation,"resources":state.resources,
                "authority":"attachment_is_not_operation_permission"}),
            )
        }
        "yai.case.resource.request" => {
            let request =
                serde_json::from_value(input).map_err(|e| format!("resource_request:{e}"))?;
            let operation = store.record_participant_resource_request(
                &authenticated,
                &case_id,
                &named_arg(args, "--participant")?,
                &named_arg(args, "--resource")?,
                &named_arg(args, "--request-id")?,
                optional_arg(args, "--generation")
                    .map(|value| value.parse::<u64>().map_err(|e| e.to_string()))
                    .transpose()?
                    .unwrap_or(state.generation),
                request,
            )?;
            serde_json::to_value(advance(&store, &authenticated, &operation)?)
                .map_err(|e| e.to_string())
        }
        _ => Err("resource_application_operation_unknown".into()),
    }
}
