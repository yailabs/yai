# Checks

Authority: validation tooling only.

These scripts guard the canonical documentation tree, current source layout,
historical-evidence banners, and reproducible labs. Product behavior belongs to
tests, not grep-based wave freezes. Obsolete wave/doctrine guards were removed
after their properties were covered by smoke or characterization tests.

Guards fail with direct messages and do not mutate repository state.
