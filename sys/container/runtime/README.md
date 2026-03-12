# Container Runtime Domain Core

`sys/container/runtime/` is the L2 core of the container domain model.

It defines what a live container is in system-service terms and exposes
the canonical runtime views consumed by other L2 and user-plane services.

## Canonical model scope

- `container.c`: container domain root object and top-level operations
- `identity.c`: container identity, class/profile and source metadata
- `config.c`: declarative container configuration contract
- `lifecycle.c`: container lifecycle transitions and state guards
- `state.c`: runtime operational state and status synthesis
- `services.c`: container-local service surface registration/readiness
- `runtime_view.c`: consolidated runtime view of the live domain
- `runtime_surface.c`: externally consumable service/runtime surfaces
- `grants_view.c`: scoped grants posture view for the domain
- `policy_view.c`: scoped policy posture view for the domain
- `model/registry/`: runtime registry of live container domain instances
- `internal/`: internal model helpers and non-public implementation details

## Ownership boundary

- owns container-domain runtime semantics at L2
- does not own privileged containment/admission/grants validity roots
  (kernel-owned)
