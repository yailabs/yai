# MANUAL ACCEPTANCE — ZERO TO USE CASE

Run these commands from the YAI repository root. Prerequisites are `bash`,
`python3`, `make`, the repository Rust/C build toolchain, and permission to bind
loopback sockets. This acceptance uses deterministic local provider fixtures
only long enough to establish existing mechanical ProviderQualification; I02
planning itself performs no provider execution. No YVEX, credential, Policy,
Resource, Workflow, or external-provider environment variable is applicable to
this planning-only wave.

Build the canonical repository launcher and create disposable state:

```bash
make build-rust
./yai version

I02_RUN_ROOT=$(mktemp -d /tmp/yai-i02-manual.XXXXXX)
export YAI_HOME="$I02_RUN_ROOT/yai-home"

python3 tests/fixtures/provider_governance_server.py --mode full --model whisper-name-is-not-semantics --requests 16 >"$I02_RUN_ROOT/primary.port" 2>"$I02_RUN_ROOT/primary.err" &
PRIMARY_PID=$!
python3 tests/fixtures/provider_governance_server.py --mode full --model vision-name-is-not-semantics --requests 16 >"$I02_RUN_ROOT/auxiliary.port" 2>"$I02_RUN_ROOT/auxiliary.err" &
AUXILIARY_PID=$!
sleep 1
PRIMARY_PORT=$(sed -n '1p' "$I02_RUN_ROOT/primary.port")
AUXILIARY_PORT=$(sed -n '1p' "$I02_RUN_ROOT/auxiliary.port")
```

Initialize YAI, inspect the authenticated Principal, and create the Tenant,
Case, and Participant:

```bash
./yai init --tenant tenant:i02-manual --organization organization:i02-manual
./yai doctor
./yai identity whoami
./yai case create case:i02-manual --tenant tenant:i02-manual
./yai case participant role add case:i02-manual --participant participant:model --role model-executor
./yai case participant list case:i02-manual
```

Create, mechanically qualify, trust, and admit the two exact targets. Their
deliberately misleading names prove that names do not confer semantic meaning:

```bash
PRIMARY_ADD=$(./yai provider add --tenant tenant:i02-manual --provider-key misleading-whisper-primary --endpoint "http://127.0.0.1:$PRIMARY_PORT" --model whisper-name-is-not-semantics --credential-ref none --locality loopback)
printf '%s\n' "$PRIMARY_ADD"
PRIMARY_TARGET=$(printf '%s\n' "$PRIMARY_ADD" | sed -n 's/^target_id: //p')

AUXILIARY_ADD=$(./yai provider add --tenant tenant:i02-manual --provider-key misleading-vision-auxiliary --endpoint "http://127.0.0.1:$AUXILIARY_PORT" --model vision-name-is-not-semantics --credential-ref none --locality loopback)
printf '%s\n' "$AUXILIARY_ADD"
AUXILIARY_TARGET=$(printf '%s\n' "$AUXILIARY_ADD" | sed -n 's/^target_id: //p')

./yai provider qualify "$PRIMARY_TARGET"
./yai provider qualify "$AUXILIARY_TARGET"
./yai provider trust approve "$PRIMARY_TARGET"
./yai provider trust approve "$AUXILIARY_TARGET"
./yai case provider bind case:i02-manual --participant participant:model --target "$PRIMARY_TARGET" --target "$AUXILIARY_TARGET" --failover safe_only --max-attempts 2
./yai case provider show case:i02-manual
```

Record truthful operator-attested semantic evidence and create the canonical
primary and auxiliary bindings:

```bash
PRIMARY_EVIDENCE_OUTPUT=$(./yai provider suitability record "$PRIMARY_TARGET" --capability primary_conversation --suite manual:i02 --run manual:primary --evidence-ref evidence:primary-conversation)
printf '%s\n' "$PRIMARY_EVIDENCE_OUTPUT"
PRIMARY_EVIDENCE=$(printf '%s\n' "$PRIMARY_EVIDENCE_OUTPUT" | sed -n 's/^evidence_id: //p')

AUXILIARY_EVIDENCE_OUTPUT=$(./yai provider suitability record "$AUXILIARY_TARGET" --capability speech_to_text --suite manual:i02 --run manual:stt --evidence-ref evidence:speech-to-text)
printf '%s\n' "$AUXILIARY_EVIDENCE_OUTPUT"
AUXILIARY_EVIDENCE=$(printf '%s\n' "$AUXILIARY_EVIDENCE_OUTPUT" | sed -n 's/^evidence_id: //p')

./yai provider suitability show "$PRIMARY_TARGET"
./yai provider suitability show "$AUXILIARY_TARGET"
./yai case cognitive bind case:i02-manual --participant participant:model --role primary --capability primary_conversation --target "$PRIMARY_TARGET" --evidence "$PRIMARY_EVIDENCE"
./yai case cognitive bind case:i02-manual --participant participant:model --role auxiliary --capability speech_to_text --target "$AUXILIARY_TARGET" --evidence "$AUXILIARY_EVIDENCE"
./yai case cognitive show case:i02-manual --participant participant:model --json
```

Stop both provider fixtures before planning. Successful plans after these
commands demonstrate that planning does not dispatch or execute a provider:

```bash
kill "$PRIMARY_PID"
kill "$AUXILIARY_PID"
wait "$PRIMARY_PID"
wait "$AUXILIARY_PID"

./yai case cognitive plan case:i02-manual --participant participant:model --capability primary_conversation --source turn:manual-text --json
./yai case cognitive plan case:i02-manual --participant participant:model --capability speech_to_text --source turn:manual-audio --json
```

Important output postures are respectively `route:native` with `role:primary`
and `route:derived` with `role:auxiliary`; both include a stable
`execution_lane_id`, `provider_realization:deferred_to_execution_adapter`, and
`provider_execution:not_performed`.

Exercise the negative paths. Missing image evidence/binding is unresolved, and
a continuation from another lane is rejected without invalidating the plan:

```bash
./yai case cognitive plan case:i02-manual --participant participant:model --capability image_understanding --source turn:manual-image --json
./yai case cognitive plan case:i02-manual --participant participant:model --capability speech_to_text --source turn:manual-audio --continuation-lane cognitive-lane:wrong --continuation-target "$AUXILIARY_TARGET" --continuation-runtime runtime:opaque --continuation-ref provider:opaque --json
```

The important postures are `route:unresolved` with
`unresolved_reason:auxiliary_binding_missing`, followed by
`continuation_posture:rejected_cross_lane`. Neither command contacts the stopped
providers.

Inspect canonical state, then execute a fresh process invocation to prove
restart/replay stability:

```bash
./yai case show case:i02-manual --json
./yai case cognitive show case:i02-manual --participant participant:model --json
./yai doctor
./yai case cognitive plan case:i02-manual --participant participant:model --capability speech_to_text --source turn:manual-audio --json
```

The Case JSON contains no new ProviderSelection, ProviderInvocation, or
ProviderResult from planning. The repeated speech-to-text plan has the same
binding-derived lane identity; its plan identity is also stable while the Case
generation and planning input remain unchanged.

Clean the disposable environment:

```bash
unset YAI_HOME
rm -rf "$I02_RUN_ROOT"
printf 'I02 manual acceptance completed; disposable state removed\n'
```
