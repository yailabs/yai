# Compatibility

This policy defines compatibility guarantees for the unified single-repository YAI system.

## Source-of-truth model

Compatibility is governance-driven from canonical in-repo roots:

- protocol contracts/schema: `lib/control/contracts/model/schema/`
- protocol headers/ABI: `include/yai/control/contracts/`
- schemas: `governance/model/schema/`
- manifests: `governance/model/manifests/`
- model/registry/grammar: `governance/model/registry/`, `governance/grammar/`

External split-repo compatibility assumptions are sunset.

## Contract compatibility baseline

| YAI line | Contract baseline | Guarantee |
|---|---|---|
| `v1.0.x` | canonical `governance/` | contract/model/schema/manifest conformance is mandatory |

If implementation and governance artifacts diverge, implementation is defective.

## Platform support

| Platform | Support level | CI coverage |
|---|---|---|
| Ubuntu latest/LTS | Supported | build + validators + gates |
| macOS latest | Supported | build + validators + gates |
| Windows | Not supported | no compatibility guarantee |

## Toolchain baseline

| Tool | Requirement |
|---|---|
| C compiler | `gcc` or `clang` |
| Build | `make` |
| Python | Python 3 |
| Rust | required only for `mind/` build paths |

## Compatibility rules

1. Canonical governance roots are authoritative.
2. Runtime/tooling must consume canonical roots directly.
3. Generated/export artifacts are derived, never normative.
4. Breaking changes require explicit changelog/versioning handling.

## License

Apache-2.0.
