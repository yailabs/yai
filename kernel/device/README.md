# kernel/drivers

Kernel driver substrate.

This area is reserved for device-facing substrate: bus integration, storage, network devices,
console/input primitives, and other low-level hardware-adjacent responsibilities.

It must not absorb UI backends, desktop adapters, shell concerns, or generic service code.
