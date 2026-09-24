//! YAI executable; command ownership lives in the shared product composition.
fn main() {
    std::process::exit(yai::cli_main(std::env::args().skip(1).collect()));
}
