# Foundation Hardening 14 report

State: semantic implementation and publication-bound qualification complete at
`0b48edee499f7b74b3a529f728af7912a24d0e5a`; remote push verification is the
only remaining publication step.

Baseline: `bdda5a707e1286c4586f3e3ce2b3ef315342c6b0`.

## Verdict

H14 closes the local Linux YAI-mediated shared-resource contract without adding
a semantic owner. `resource_control.rs` remains the sole Case-independent
resource authority owner. Integrity-valid fence bytes are insufficient: the
carrier requires the exact current canonical fence, epoch, resource,
Case/Operation/Grant/Effect binding and executing process birth identity.

Resource event/state schemas advance to v2. Events now contain the full
ResourceIdentity, full fence and exact predecessor ID/digest; a bounded FSM
rebuild rejects gaps, duplicates, regressions, impossible acquire/reclaim/
release order and identity switches. Historical v1 records remain readable but
are explicitly not standalone-rebuildable because they did not persist enough
information. New v2 history reproduces current authority exactly without
touching Case history.

The Tenant-scoped filesystem carrier is mechanically confined on Linux. It
verifies the root device/inode, resolves parent components with `openat2`
`RESOLVE_BENEATH|RESOLVE_NO_MAGICLINKS|RESOLVE_NO_SYMLINKS`, and performs the
atomic replacement relative to a trusted directory descriptor. Final and
intermediate symlinks, root replacement, traversal and parent rename/symlink
swaps fail closed or remain bound to the already opened directory. Atomic
replacement avoids mutating a pre-existing outside hard-link inode, but YAI
does not claim exclusive inode ownership or protection from arbitrary non-YAI
host writers. Strong TOCTOU qualification is Linux-specific; unsupported
platforms fail conservatively for the new scoped carrier.

The terminal Case transition and resource release remain one LMDB transaction.
Before commit, the effect remains unresolved and its lease active; after
commit, the effect is terminal and the resource released. Repeated recovery is
idempotent. An eight-process run admitted one acquisition, rejected seven,
advanced epochs 1→2→3→4→5 across reclaim/release, and observed one physical
mutation per effect.

Process signals are never blindly retried after a crash following `kill(2)`.
Terminate is unsafe/ambiguous to repeat; suspend and resume use
observation-only recovery. Syscall acceptance, later observed state and causal
conclusion remain distinct. A PID with another start discriminator receives no
signal. Unprovable truth becomes/stays Indeterminate and retains the resource.

`resource_temporarily_owned` now maps RuntimeWorkItem to nonterminal `Blocked`,
releases its worker and waits for a real resource-state change. After terminal
release, the same WorkItem resumes. An exact still-issued adjacent Grant may be
reused only after the canonical writer rechecks policy/time/generation; a stale
Grant is invalidated and a later retry rederives authority. This is physical
admission, not policy DENY, Review wait or effect uncertainty.

## External provider

The permanent harness is black-box: endpoint plus provider-exposed model ID,
through the generic `openai_compatible` path. It does not inspect a YVEX
repository or invoke YVEX CLI/plumbing. Generic response parsing now reports an
optional response model ID and tolerates extensions. No live endpoint or model
was supplied/reachable, so the state is honestly
`blocked_external_dependency`. There is no YVEX-side defect and no X/Y/Z model
result at this checkpoint.

## Ownership and footprint

- semantic owners added: 0
- LMDB databases: 34/40 before and after; no store added
- `main.rs`: 2009 lines before and after; unchanged
- primary owners changed: existing `resource_control.rs`, `effect.rs`,
  `store/lmdb.rs`, `controlled_effect.rs`, `runtime_instance.rs`, and generic
  provider surface
- schemas: ResourceControlState/Event v2; LocalFilesystemBinding v2; old v1
  readers retained

## Qualification

`make check`, `make characterization`, the explicit W10/H10 through W14/H14
smoke matrix and the governed endurance target pass. Current Rust totals are
130 engine and 11 CLI tests. The 26-turn governed runtime, both bounded
128-iteration bodies, R1-R6/A1-A6 recovery paths, Tenant isolation,
RuntimeInstance panic/restart semantics and prior fencing/carrier flows remain
green. Formatting, layout/docs and `git diff --check` pass. Clippy under the
repository contract exits 0; its pre-existing baseline warnings remain, and
the one H14-introduced tuple-complexity warning was removed before publication.

## W15 entry

The nine entry questions are YES for the stated local Linux/YAI-mediated and
generic-provider boundaries. W15 still must implement WorkflowDefinition,
exact Case/Tenant adoption, completion predicates, deterministic resolver,
ReadyWorkSet, ModelWork, deterministic work, HumanInput, Condition/Wait,
EffectGoal and derived progress/replay. H14 implements none of them.
