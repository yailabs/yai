//! Shared product-process composition for the CLI and native desktop Host.
//!
//! The scheduler and its existing domain adapters remain one implementation.
//! Desktop callers use the typed Host entry, never command parsing or stdout.

mod cli;
mod command_adapters;

/// Canonical CLI registry entry; not a desktop Application transport.
pub fn cli_main(args: Vec<String>) -> i32 {
    cli::run(args)
}

/// Compose the resident Application service with the existing RuntimeInstance.
/// Home identity, leases, recovery and graceful draining stay in their owners.
pub fn serve_application_host(home: &std::path::Path) -> Result<(), String> {
    command_adapters::runtime_instance::serve_application_host(home)
}
