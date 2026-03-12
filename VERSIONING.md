# VERSIONING

Versioning policy for the unified single-repository YAI system.

Public baseline for unified convergence: `v1.0.0`.

## Scope

This policy applies to:

- runtime surfaces (`cmd/`, `lib/`, `include/`)
- governance surfaces (`governance/`)
- tooling/validation/release surfaces (`tools/`, `tests/`)
- canonical docs (`docs/`)

## Scheme

Semantic Versioning: `MAJOR.MINOR.PATCH`.

- `MAJOR`: breaking compatibility
- `MINOR`: backward-compatible features
- `PATCH`: backward-compatible fixes/hardening/docs/tooling

## Unified release rule

Each release is single-repo and must publish:

- YAI version
- governance baseline (`governance/` at release commit)
- compatibility impact summary
- operator migration notes when applicable

No split-repository pin/pairing model is canonical.

## Breaking-change criteria

A change is breaking when it affects required consumer/operator behavior, including:

- protocol/control semantics
- workspace/runtime contract behavior
- governance contract/model/schema/manifest interpretation
- output formats consumed by canonical tooling

Breaking changes require:

- explicit `CHANGELOG.md` entry
- version bump according to SemVer
- validation evidence

## Compatibility linkage

- compatibility guarantees: `COMPATIBILITY.md`
- release history: `CHANGELOG.md`

## License

Apache-2.0.
