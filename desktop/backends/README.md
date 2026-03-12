# Backends

Backend implementations provide native window/input/render integration.

Backends must implement the desktop backend contract from `include/yai/desktop/backend.h`.

## Boundary
- no shell panel logic
- no command parsing
- no protocol schema mutation
- only runtime-facing input/present lifecycle
