# K-2 Kernel Primitive Ownership Matrix

Status: canonical ownership lock for Wave 1.
Scope: `boot/`, `kernel/`, `sys/`, `user/`, legacy migration sources.

## Ownership Classes

- `KERNEL_OWNER`: primitive is kernel-grade and belongs to `kernel/`.
- `SYS_OWNER`: primitive belongs to userspace system services under `sys/`.
- `USER_OWNER`: primitive belongs to user tooling/libs.
- `LEGACY_DEMOLISH`: legacy semantic/component; no canonical ownership in target architecture.

## Primitive Families

| Primitive class | Owner layer | Privilege | Allowed dependencies | Forbidden dependencies |
|---|---|---|---|---|
| Boot / handoff primitives | `boot/` + `kernel/` | max | arch, kernel core init | container manager, orchestration, daemon, graph, data, knowledge |
| Authority root primitives | `kernel/` | max | kernel state/registry | orchestration, daemon, provider logic, userspace policy engine |
| Containment primitives | `kernel/` | max | fs, proc, security, mm | container manager userspace, workflow, graph/data logic |
| Process / thread primitives | `kernel/` | max | sched, mm, security | container config logic, orchestration, daemon runtime logic |
| Scheduler primitives | `kernel/` | max | proc, mm | high-level orchestration scheduling |
| Memory primitives | `kernel/` | max | arch, core | graph/data/knowledge logic |
| VFS / mount / namespace primitives | `kernel/` | max | core, security, container kernel primitives | userspace container lifecycle/config, workspace semantics |
| IPC primitives | `kernel/` | max | proc, mm, security | orchestration bridge logic |
| Network stack base primitives | `kernel/` | max | proc, security | mesh/discovery high logic, provider selection |
| Security / capability / audit primitives | `kernel/` | max | proc, fs, ipc, net | governance service logic |
| Container kernel primitives | `kernel/` | max | fs, proc, security, mm | userspace container manager logic |
| Kernel state / registry / lifecycle | `kernel/` | max | kernel core only | container/orchestration/daemon lifecycle logic |

## Forbidden-In-Kernel List (Hard)

The following are explicitly forbidden from kernel ownership:
- container management logic (`sys/container/*`)
- orchestration logic (`sys/orchestration/*`)
- governance service decisions/publication/escalation (`sys/governance/*`)
- rich policy engine (`sys/policy/*`)
- graph/data engines (`sys/graph/*`, `sys/data/*`)
- provider catalog/selection/inference/embedding logic
- daemon runtime/business logic (`sys/daemon/*`)
- workspace-centric semantics (canonical status: deprecated -> migrate or demolish)

## Migration Decision Matrix: Current Sources

### A) `include/yai/runtime/*` and `lib/runtime/*`

#### A.1 Migrate to `kernel/` (`KERNEL_OWNER`)

- `include/yai/runtime/authority.h`
- `include/yai/runtime/containment.h`
- `include/yai/runtime/dispatch.h` (kernel-grade dispatch hooks only)
- `include/yai/runtime/enforcement.h` (enforcement hooks only)
- `include/yai/runtime/grants.h` (privileged grants hooks only)
- `include/yai/runtime/lifecycle.h` (kernel lifecycle core only)
- `include/yai/runtime/session.h` (privileged admission/session core only)
- `include/yai/runtime/vault.h`
- `lib/runtime/policy/authority/*`
- `lib/runtime/containment/*`
- `lib/runtime/dispatch/*` (low-level control transport + command dispatch hooks only)
- `lib/runtime/enforcement/*` (kernel enforcement substrate only)
- `lib/runtime/grants/*`
- `lib/runtime/lifecycle/{preboot.c,bootstrap.c,runtime_capabilities.c}`
- `lib/runtime/session/session.c` (only privileged session gate/core parts)

#### A.2 Migrate to `sys/container/` or `sys/*` (`SYS_OWNER`)

- `include/yai/runtime/local/*`
- `runtime/compatibility/lib/runtime/local/*`
- `lib/runtime/lifecycle/startup_plan.c` (service-level sequencing)
- `lib/runtime/session/session_reply.c`
- `lib/runtime/session/session_utils.c`
- `lib/runtime/session/utils/*`

#### A.3 Legacy / demolition (`LEGACY_DEMOLISH`)

- `include/yai/runtime/workspace.h`
- `runtime/compatibility/lib/runtime/workspace/*`
- any runtime surface where workspace is central semantic owner

#### A.4 Boundary split required before migration

- `include/yai/runtime/policy.h`, `lib/runtime/policy/policy_state.c`
  - keep only kernel hooks in `kernel/security` or `kernel/kernel`
  - move rich policy evaluation/composition to `sys/policy`


### B) `include/yai/platform/*` and `lib/platform/*`

#### B.1 Migrate to `kernel/` (`KERNEL_OWNER`)

- `include/yai/platform/fs.h`, `lib/platform/fs.c`
- `include/yai/platform/os.h`, `lib/platform/os.c` (privileged low-level subset)
- `include/yai/platform/uds.h`, `lib/platform/uds.c` (if adopted as low-level IPC/transport primitive)

#### B.2 Keep outside kernel (`SYS_OWNER` / `USER_OWNER`)

- `include/yai/platform/clock.h`, `lib/platform/clock.c` (unless required for kernel time source core)
- wrappers/helpers above privileged boundary

#### B.3 Transitional/internal

- `lib/platform/internal.h` must be split into kernel-internal vs userspace/platform adapters.


### C) `include/yai/protocol/*` and `lib/protocol/*`

#### C.1 Kernel ABI candidates (`KERNEL_OWNER` for interface boundary only)

- low-level error/types/message base feeding syscall/ABI contracts:
  - `include/yai/protocol/control/errors.h`
  - `include/yai/protocol/message_types.h`
  - `lib/control/protocol/control/message_types.c` (only if used as kernel ABI serialization boundary)

#### C.2 Protocol userspace/system ownership (`SYS_OWNER`)

- control-plane rich contracts and source-plane logic:
  - `include/yai/protocol/control/{auth.h,roles.h,session.h,source_plane.h,control_plane.h,audit.h,ids.h}`
  - `lib/control/protocol/control/source_plane.c`
- rpc/transport runtime contracts and codecs:
  - `include/yai/protocol/rpc/*`
  - `include/yai/protocol/transport/*`
  - `include/yai/protocol/binary/*`
  - `lib/protocol/rpc/*`
  - `lib/protocol/binary/*`

#### C.3 Not kernel by default

- `source_plane_contract`, runtime rpc contracts, transport contracts as high-level service interface.

## Ownership Lock Rules

1. No module enters `kernel/` without a primitive-family mapping in this matrix.
2. If a module mixes kernel-grade and service-grade logic, split first, then migrate.
3. Workspace semantics are never canonical owners in target architecture.
4. `runtime/` is migration source only; canonical ownership is in `kernel/`, `sys/`, `user/`.
5. Any unresolved module is blocked until classified in K-2 matrix updates.

## Next Step

K-3: instantiate kernel tree ownership stubs (headers + source placeholders) and start first controlled migrations for policy/authority/containment/process/security primitives.
