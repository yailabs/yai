# kernel/core

Kernel entry nucleus.

This area must stay intentionally small.
It hosts only top-level kernel bootstrap/orchestration primitives and must not become
a generic dump for helpers, policies, registries, or subsystem-local mechanics.
