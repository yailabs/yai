mod help;
mod output;
mod parser;
mod product;
mod registry;

use output::{render_error, render_result, CliError};
use parser::{parse, ParseOutcome};

pub(crate) fn run(args: Vec<String>) -> i32 {
    match parse(&args) {
        Ok(ParseOutcome::Help(request)) => {
            help::render(&request);
            0
        }
        Ok(ParseOutcome::Invoke(invocation)) => {
            let operation_id = invocation.descriptor.operation_id;
            match product::execute(&invocation) {
                Ok(result) => {
                    render_result(operation_id, invocation.json, result);
                    0
                }
                Err(error) => {
                    let exit_code =
                        if invocation.descriptor.visibility == registry::Visibility::Product {
                            error.exit_code()
                        } else {
                            // Engineering/compatibility probes historically use
                            // exit 2 for an unsuccessful finite operation. Keep
                            // that documented qualification contract; porcelain
                            // distinguishes syntax (2) from domain failure (3).
                            2
                        };
                    render_error(Some(operation_id), invocation.json, error);
                    exit_code
                }
            }
        }
        Err(error) => {
            let exit_code = error.exit_code();
            render_error(None, args.iter().any(|arg| arg == "--json"), error);
            exit_code
        }
    }
}

pub(crate) fn syntax_error(message: impl Into<String>) -> CliError {
    CliError::usage(message)
}
