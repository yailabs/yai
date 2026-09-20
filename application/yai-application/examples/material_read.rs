use serde_json::{json, Value};
use std::process::ExitCode;
use yai_application::{LocalApplication, OperationRequest, APPLICATION_PROTOCOL};

fn main() -> ExitCode {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments.len() != 4 {
        eprintln!("usage: material_read <case:ref> <source:ref> <source-revision:ref> <path>");
        return ExitCode::from(2);
    }
    let result = LocalApplication::default().call(OperationRequest {
        protocol: APPLICATION_PROTOCOL.into(),
        operation_ref: "material.read".into(),
        correlation_ref: "qualification:material-read".into(),
        input: json!({
            "case_ref": arguments[0],
            "source_ref": arguments[1],
            "revision_ref": arguments[2],
            "path": arguments[3]
        }),
    });
    let value = serde_json::to_value(&result).unwrap_or_else(|error| {
        json!({ "result_state": "error", "error": { "code": "serialization_failed", "message": error.to_string() } })
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| Value::Null.to_string())
    );
    if result.result_state == yai_application::ResultState::Success {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
