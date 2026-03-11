# Runtime Public Headers

Public runtime nucleus headers for:

- authority, dispatch, enforcement, lifecycle, session, workspace
- runtime-side policy, grants, containment
- event taxonomy and vault runtime state

Boundary notes:

- governance resolves policy content; runtime applies policy state
- orchestration governs flow control; runtime owns executable core state
- vault wire ABI and protocol contracts stay under `protocol/`; `runtime/vault.h`
  is the runtime-side state/use surface
