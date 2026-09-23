---
name: yai-capability-surface
description: Classify and qualify an added or materially changed executable YAI capability across Application, CLI and Studio.
---

# Product capability surface

Use for executable capability additions or material semantic changes, not an
internal helper touched incidentally. The code-owned
[catalog](../../application/yai-application/src/capabilities.rs) is the source
of product metadata; regenerate its human matrix rather than editing the
[matrix](../../docs/reference/application-capabilities.md) independently.

For every supported semantic action, identify the engine owner, stable
capability ID, typed Application operation or exact deferred/internal reason,
CLI posture, Studio posture, authority and state impact, diagnostics and product
tests. Do not create a second registry or present target-only mechanics as
executable. Application metadata and client discovery grant no authority.

An executable PRODUCT family can remain Application-deferred only for a
code-owned, test-enforced missing semantic contract. CLI placement, wrapper
effort and missing Studio UI are not blockers. List partial operations without
promoting the entire family. Keep Application orchestration frontend-independent;
CLI and Studio must not independently recreate domain admission.

Qualification is per action, not by catalog count. Exercise the actual typed
dispatcher with a positive case and the relevant hidden, stale or unauthorized
negative; assert resulting domain facts or their absence. For effectful
submissions also prove durable exact identity, lost-response retry,
current-authority observation and no duplicate dispatch. A `UiAlreadyConsumed`
claim requires a typed Studio client, authored interaction and product test;
an available Application method alone is `ApplicationReady`.

At closure distinguish engine execution, Application operations, CLI commands
and Studio interactions. Report every current action and exact gap rather than
inferring family parity from one summary operation. Use the catalog and
[validation doctrine](../../docs/test-cases.md) for existing guards and lanes.
