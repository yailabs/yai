# Runtime

Provides compositor, renderer, session state, render loop, and asset lifecycle.

Runtime binds protocol contracts to a selected backend and shell surfaces.

## Boundary
- no backend-specific code outside `backends/`
- no panel business logic from `shell/`
- no app composition logic from `apps/`
- integrates with `sys/` through typed contracts, not ad-hoc UI hooks
