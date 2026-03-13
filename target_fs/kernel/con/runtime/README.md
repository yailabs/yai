# kernel/container/runtime

Container runtime operational surface.

This area owns runtime-facing container views and operational access surfaces.

Canonical responsibilities:
- runtime view materialization
- runtime-facing container surface access
- workspace runtime support under `kernel/container/workspace/`

Non-responsibilities:
- kernel core policy/grants/registry primitives
- generic session core lifecycle
