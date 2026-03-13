# YAI Kernel

## Boundary Statement (AL-1)

`kernel/` is the privileged minimal kernel layer of the YAI OS-substrate path.

Kernel ownership is strictly limited to kernel-grade primitives:
- ABI base (`kernel/include/yai/abi/*`)
- privileged kernel state and lifecycle
- privileged session admission
- privileged registry roots
- grants root validity/checks and lifecycle hooks
- containment root and escape/breach control points
- low-level kernel container primitives (namespaces, rootfs, mounts, limits)
- kernel policy hooks (admission/mount/escape/spawn gating)
- kernel trace/audit/metrics hooks

## Explicitly Out Of Kernel

The following are system-level or user-level responsibilities and must not be implemented in `kernel/`:
- orchestration and workflow engines
- governance resolution/review engines
- high-level policy composition/overlay/compliance logic
- graph/data service engines (materialization, retention, archive, store)
- high-level network control plane (discovery/topology/routing/mesh/service policy)
- daemon management business logic
- operator UX surfaces, SDKs, and CLI logic

## Transitional Notes

- `runtime/compatibility/` remains migration source only and is not kernel authority.
- Header namespaces under `kernel/include/yai/{proc,mm,fs,ipc,net,security,trace,drivers}` are currently skeletal in this wave; this is intentional. Their ownership is kernel-side, but implementations are phased.
- Historical kernel migration notes were moved to `docs/transitional/kernel-meta/` and are non-normative for ownership.

## Practical Rule

If a feature requires domain semantics, workflow composition, operator-facing behavior, or service-level orchestration, it belongs outside `kernel/` (typically `sys/` or `user/`).
