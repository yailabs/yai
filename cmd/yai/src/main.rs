//! YAI product command entry point.
//!
//! Command metadata, parsing, help, lane selection, and output projection live
//! in `cli`. The historical command implementations remain isolated behind
//! `command_adapters` until their domain owners expose typed operations.

mod cli;
mod command_adapters;

fn main() {
    std::process::exit(cli::run(std::env::args().skip(1).collect()));
}
