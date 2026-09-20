use serde_json::{json, Value};
use std::process::ExitCode;
use yai_application::{LocalApplication, OperationRequest, APPLICATION_PROTOCOL};

fn main() -> ExitCode {
    let Some(case_ref) = std::env::args().nth(1) else {
        eprintln!("usage: case_summary <case:ref>");
        return ExitCode::from(2);
    };
    let result = LocalApplication::default().call(OperationRequest {
        protocol: APPLICATION_PROTOCOL.into(),
        operation_ref: "case.summary".into(),
        correlation_ref: "qualification:case-summary".into(),
        input: json!({ "case_ref": case_ref }),
    });
    let value = serde_json::to_value(&result).unwrap_or_else(|error| {
        json!({
            "result_state": "error",
            "error": { "code": "serialization_failed", "message": error.to_string() }
        })
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
