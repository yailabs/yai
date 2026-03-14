# Providers Public Headers

Canonical provider infrastructure headers:

- `catalog.h` (provider catalog + registration surface)
- `registry.h`
- `policy.h`
- `selection.h`
- `embedding.h`
- `inference.h`
- `mocks.h`

Providers are controlled internal runtime infrastructure.

Boundary contract:

- provider APIs expose registry, selection and adapter surfaces
- protocol contracts remain in `include/yai/protocol/`
- orchestration and agents consume providers, they do not define provider model
