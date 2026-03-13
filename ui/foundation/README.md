# Foundation

Defines the canonical UI model, scene graph primitives, input model, layout model, and styling contracts.

This layer does not contain backend code or service calls.

## Boundary
- accepts no direct dependency on `kernel/` services
- no policy/data/graph transport calls to `sys/`
- no operator command orchestration from `user/`
- no sdk client/session wiring from `sdk/`
