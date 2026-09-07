# MANUAL ACCEPTANCE — ZERO TO USE CASE

Run from the repository root in two terminals. These are ordinary commands, not
a shell assertion harness. No `set -euo pipefail`, Python assertions, internal
database edits, ResourceAttachment or YVEX deployment is required.

This local provider is explicitly a deterministic HTTP fixture. It proves YAI
routing and provenance, not intelligence, transcription quality or YVEX
interoperability. Suitability below is an explicit fixture/operator attestation,
separate from the mechanical provider probes.

## Terminal A — disposable loopback provider

Keep this command running. If port 18106 is already occupied, stop here; do not
reuse an unidentified service.

```sh
python3 tests/fixtures/provider_governance_server.py --port 18106 --model i06-manual-fixture --mode full --requests 64
```

## Terminal B — fresh Case and governed cognitive admission

The two short output captures retain generated target/evidence IDs; all actions
remain ordinary `./yai` commands.

```sh
make build-rust
I06_RUN="$(mktemp -d /tmp/yai-i06-manual.XXXXXX)"
export YAI_HOME="$I06_RUN/home"
./yai init --tenant tenant:i06-manual --organization organization:i06
./yai case create case:i06-manual --tenant tenant:i06-manual
./yai case participant role add case:i06-manual --participant participant:model --role model-executor
./yai case participant link-principal case:i06-manual --principal self --participant participant:model
./yai case participant view admit case:i06-manual --participant participant:model --consumer model --view model_context
I06_TARGET="$(./yai provider add --tenant tenant:i06-manual --provider-key i06-fixture --endpoint http://127.0.0.1:18106 --model i06-manual-fixture --locality loopback | sed -n 's/^target_id: //p')"
./yai provider qualify "$I06_TARGET" --realization-shape text_to_text
./yai provider trust approve "$I06_TARGET"
./yai case provider bind case:i06-manual --participant participant:model --target "$I06_TARGET" --failover safe_only --max-attempts 1
I06_EVIDENCE="$(./yai provider suitability record "$I06_TARGET" --capability primary_conversation --suite fixture:i06-manual --run manual-local --evidence-ref evidence:i06-deterministic-fixture | sed -n 's/^evidence_id: //p')"
./yai case cognitive bind case:i06-manual --participant participant:model --role primary --capability primary_conversation --target "$I06_TARGET" --evidence "$I06_EVIDENCE"
./yai prompt --case case:i06-manual --subject participant:model
```

Inside the prompt, type the following, pressing Enter after each line:

```text
Saluta e descrivi il contesto della conversazione.
/thread status
/exit
```

The fixture returns its fixed text. Inspect `conversation_turn`,
`conversation_generation`, `conversation_cognition` and
`conversation_execution: "completed"`. The cognition line names the adopted
intent, exact target, execution/validation plans, lane, selection, invocation,
ProviderResult and Projection/ContextFrame. No operation is proposed/executed.

## Inspect and reopen

```sh
./yai case conversation turn list case:i06-manual --participant participant:model --json
./yai case conversation turn show case:i06-manual latest --participant participant:model --json
./yai case cognitive show case:i06-manual --participant participant:model --json
./yai case provider show case:i06-manual
./yai store status
./yai prompt --case case:i06-manual --subject participant:model
```

The same Turn/intent remain inspectable across processes. Inside the reopened
prompt, use `/retry` followed by the actual Turn ID printed by SEND if you want
to inspect result reuse. It returns the same ProviderResult with
`recovered: true`; it does not add a user Turn. Exit with `/exit`.

Typed direct/composed host behavior, actual process-restart recovery, two-target
arbitration, pinning, failure and /retry are executable qualification, not a
claim of manual human acceptance:

```sh
make smoke-conversation-cognitive-host
PATH="$PWD/build/r4-venv/bin:$PATH" make smoke-replai-terminal
```

The PTY command uses the existing pinned terminal-test environment documented in
[the terminal contract](../../../docs/replai-terminal.md). No REPLAI pin advance.

## Cleanup

Exit the prompt; stop the provider in Terminal A with Ctrl-C. In Terminal B:

```sh
rm -r -- "$I06_RUN"
unset YAI_HOME I06_RUN I06_TARGET I06_EVIDENCE
```

Only the newly created disposable state is removed. Existing YAI Homes and
provider deployments are not touched.
