# YAI (C)

YAI is the systems core of the YAI platform.

It brings control, execution, and cognition into one governed architecture built for explicit authority, bounded behavior, and proof under operation.

This repository contains the primary implementation of YAI: runtime foundations, execution layers, cognition subsystems, shared protocol and support surfaces, and the program artifacts that govern their evolution.

Each layer carries constraints forward.  
Law defines them. Interfaces expose them. YAI implements them. Operations verify them.

## Design posture

- **Authority is explicit**
- **Behavior is bounded**
- **Execution is governable**
- **Cognition is controlled**
- **Evidence is first-class**
- **Change is deliberate**

## Scope

This repository owns the governed implementation of YAI and the program artifacts required to evolve it under control.

It does not own canonical law (`yai-law`) or shared cross-repo governance tooling (`yai-infra`).

## Build

```bash
make yai
make dist
```

## Test/Verify

```bash
make test
make verify
```

Primary runtime entrypoints:
- `build/bin/yai`

Repository topology is authoritative under:
- `cmd/`
- `include/yai/`
- `lib/`
- `tests/`

## Documentation

- `docs/README.md`

## Dependency discipline

Canonical law is consumed as a pinned dependency through `deps/yai-law/`.  
SDK alignment is tracked through `deps/yai-sdk.ref`.

Divergence from pinned law or aligned interfaces must be corrected in implementation.

## License

Apache-2.0. See `LICENSE`, `NOTICE`, and `THIRD_PARTY_NOTICES.md`.

## Law compatibility declaration

- Human-readable declaration: `LAW_COMPATIBILITY.md`
- Machine-readable declaration: `law-compatibility.v1.json`
