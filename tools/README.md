# tools

Authority: local repository tooling only.

`checks/` guards layout and documentation. `validation/topology.py` audits the
single `tests/classification.tsv` authority, generates ordinary Make
dependencies and selects exact compiled Rust tests. Assertions remain in
`tests/` or alongside Rust implementation; tooling owns no product semantics,
test database, provider service or persistent PASS cache.

`shell/yai.sh` is the source template for the repository-local `./yai` launcher.
`make build-rust` installs it at the repository root when absent; that local
copy is ignored by Git. Existing local launchers are preserved.
