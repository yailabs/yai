# include/

Canonical header topology for YAI OS substrate.

Rules:

- `include/yai/` is the canonical YAI namespace surface.
- `include/uapi/` is the user-space ABI surface.
- `include/generated/` is generated-only.
- `include/asm-generic/` is cross-arch low-level shared surface.

Top-level peer buckets under `include/` model OS responsibilities
(e.g. `net`, `trace`, `drm`, `sound`, `soc`, `media`, `crypto`, ...).

These peer buckets may begin as stubs before implementation exists.
They are topology-first, not completeness-first.

Do not collapse OS responsibility buckets back into app/framework groupings.
