# Test 01 -- Local Runtime Inspection

Status: manual

## Purpose

Verify that local runtime, hot-state, store, and daemon inspection surfaces can
be inspected without involving a model.

## Prerequisites

- Build or install the local binaries according to the current engineering
  command surface.
- Run commands from the repository root using the canonical `./yai` launcher.
- Prepare a local `YAI_HOME` when inspecting a specific runtime home.

No provider, model, API key, or model server is required.

## Commands

```sh
./yai doctor
./yai hot status
./yai store status
./yai store summary
```

When a local `yaid` process and socket are prepared, daemon inspection commands
from the current engineering command surface may also be used:

```sh
./yai daemon status --socket "$YAI_DAEMON_SOCKET"
./yai daemon info --socket "$YAI_DAEMON_SOCKET"
```

Set `YAI_DAEMON_SOCKET` to the actual operator-prepared socket before these
optional commands. Inspection does not shut down an operator's daemon.

## Expected Behavior

- `yai doctor` reports runtime paths and local surface status.
- `yai hot status` reports active hot-state material or an unavailable reason.
- `yai store status` reports record-store backend readiness.
- `yai store summary` reports aggregate record-store counts when initialized,
  or the current store status when missing or uninitialized.
- daemon inspection commands require a reachable local daemon socket.

## Daemon, Store, And Hot-State Notes

Hot state is live runtime cache, not durable truth.

The store is durable record lookup and indexed operational material when it is
initialized.

The daemon, hot state, and store are distinct surfaces. A missing hot-state
snapshot does not by itself prove that the record store is broken.

## Deeper References

- [Public test cases](../../../docs/test-cases.md)
- [Command surface](../../../work/spines/command-surface.md)
- [Testing doctrine](../../../work/spines/testing.md)
