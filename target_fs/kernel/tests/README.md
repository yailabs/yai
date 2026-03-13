# Kernel Tests

Kernel vertical smoke entrypoint:

- `make -C kernel smoke`
- `make kernel-smoke`

The smoke covers a minimal privileged flow:

1. kernel bootstrap and readiness
2. grant issue/activate lifecycle
3. containment request/activate
4. session admission and container bind
5. escape gate behavior before/after revocation
