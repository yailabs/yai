# Checks

Authority: validation tooling only.

These scripts guard the canonical documentation tree, current source layout,
historical-evidence banners, and reproducible labs. Product behavior belongs to
tests, not grep-based wave freezes. Obsolete wave/doctrine guards were removed
after their properties were covered by smoke or characterization tests.

Guards fail with direct messages and do not mutate repository state.

`check-roadmap.py` reads the sole maturity matrix in `ROADMAP.md`: validates
state vocabulary, stable row IDs, evidence references, generated counts and one
selected temporal boundary matching the snapshot. Other tables never contribute
counts. `--summary` prints the recomputed line without writing or promoting rows.
It rejects competing live roadmap/control files in root/docs, not historical
reports. Evidence links are traceability, not automated maturity certification.
`make check-docs` includes the guard; `make test-roadmap` supplies no-provider
unit regression on drift/refusal and is classified in fast/publication lanes.
