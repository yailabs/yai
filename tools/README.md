# tools

Authority: local repository tooling only.

`checks/` guards layout and documentation. `validation/topology.py` audits the
single `tests/classification.tsv` authority, generates ordinary Make
dependencies and selects exact compiled Rust tests. Assertions remain in
`tests/` or alongside Rust implementation; tooling owns no product semantics,
test database, provider service or persistent PASS cache.
