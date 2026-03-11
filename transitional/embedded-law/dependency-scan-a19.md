# A19 Embedded-Law Dependency Scan

This marker captures active non-doc references still tied to `embedded/law`.

## Runtime/tooling references (representative)

- `Makefile`: `LAW_COMPAT_ROOT` fallback to `embedded/law`
- `lib/governance/loader/law_loader.c`: candidate roots include `embedded/law`
- `lib/governance/discovery/domain_discovery.c`: reads classification and specialization indexes from `embedded/law`
- `lib/runtime/session/utils/session_utils_surface_core.inc.c`: embedded-law candidate roots
- `tools/dev/resolve-law-embed.sh`: resolves `embedded/law`
- `tools/dev/resolve-law-compat.sh`: resolves `embedded/law`
- `tools/dev/check-generated.sh`: expects `embedded/law`
- `tools/bin/yai-law-embed-sync`: default target `embedded/law`
- `tools/bin/yai-law-compat-check`: validates `embedded/law`

## Migration direction

- target authoritative governance content: `governance/`
- keep `embedded/law` only as transitional package/export surface
- cut runtime read paths from embedded roots as Block B progresses
