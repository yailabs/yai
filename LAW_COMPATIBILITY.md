# Law Compatibility Declaration

- normative authority repo: `law`
- compatibility mode: `embedded-surface-primary`
- embedded source path: `embedded/law`
- supported embedded law version(s):
  - `0.1.0`
- tested embedded law version: `0.1.0`
- notes:
  - `yai` is the integration/runtime authority.
  - Canonical law remains outside this repository (`law`).
  - Runtime-facing law is consumed from exported embedded surface (`embedded/law/`).
  - Legacy `deps/law/` bridge has been retired from active runtime and tooling paths.
