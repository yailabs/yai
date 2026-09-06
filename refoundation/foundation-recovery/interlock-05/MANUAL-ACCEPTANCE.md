# MANUAL ACCEPTANCE — ZERO TO USE CASE

This is local operator acceptance with deterministic HTTP fixtures, **not live
YVEX qualification**. No model service, GPU, credential, historical Case,
ResourceAttachment or policy artifact is required for conversation execution.
Semantic suitability below is explicitly operator-attested fixture evidence.

Use three terminals in the YAI repository root. For this recorded workstation:

```sh
cd /home/mothx/computer-science/projects/YAI/yai
```

Terminal A (leave running; the printed port must be 18351):

```sh
python3 tests/fixtures/provider_governance_server.py --port 18351 --mode full --model i05-first-fixture --requests 128
```

Terminal B (leave running; the printed port must be 18352):

```sh
python3 tests/fixtures/provider_governance_server.py --port 18352 --mode full --model i05-second-fixture --requests 128
```

Terminal C: execute the following blocks in order. `read` means paste the exact
ID just printed and press Enter. It avoids hidden output parsing, guessed IDs
and generated pseudo-commands. Do not paste later commands while `read` is
waiting for an ID. These commands work in Bash and Zsh without strict-mode
shell options.

## Fresh isolated Case

```sh
make build
I05_HOME="$(mktemp -d /tmp/yai-i05-manual.XXXXXX)"
export YAI_HOME="$I05_HOME"
export NO_COLOR=1
./yai init --tenant tenant:i05-manual --organization organization:i05-manual
./yai identity whoami
./yai case create case:i05-manual --tenant tenant:i05-manual
./yai case participant role add case:i05-manual --participant participant:model --role model-executor
./yai case participant link-principal case:i05-manual --principal self --participant participant:model
./yai case participant view admit case:i05-manual --participant participant:model --consumer model --view model_context
```

## Exact provider targets and evidence

Register A, then paste its printed `target_id` into the read:

```sh
./yai provider add --tenant tenant:i05-manual --provider-key i05-first --endpoint http://127.0.0.1:18351 --model i05-first-fixture --locality loopback
read -r I05_TARGET_A
```

Register B and retain its printed ID:

```sh
./yai provider add --tenant tenant:i05-manual --provider-key i05-second --endpoint http://127.0.0.1:18352 --model i05-second-fixture --locality loopback
read -r I05_TARGET_B
```

Mechanical qualification performs real requests to these two fixture processes:

```sh
./yai provider qualify "$I05_TARGET_A" --realization-shape text_to_text
./yai provider trust approve "$I05_TARGET_A"
./yai provider qualify "$I05_TARGET_B" --realization-shape text_to_text
./yai provider trust approve "$I05_TARGET_B"
./yai provider suitability record "$I05_TARGET_A" --capability primary_conversation --suite fixture:i05-manual --run manual:first --evidence-ref fixture:provider_governance_server
read -r I05_EVIDENCE_A
```

Paste A's `evidence_id` above; now record and retain B's:

```sh
./yai provider suitability record "$I05_TARGET_B" --capability primary_conversation --suite fixture:i05-manual --run manual:second --evidence-ref fixture:provider_governance_server
read -r I05_EVIDENCE_B
```

Provider permission and cognitive choice remain separate:

```sh
./yai case provider bind case:i05-manual --participant participant:model --target "$I05_TARGET_A" --target "$I05_TARGET_B" --failover safe_only --max-attempts 2
./yai case cognitive bind case:i05-manual --participant participant:model --role primary --capability primary_conversation --target "$I05_TARGET_A" --evidence "$I05_EVIDENCE_A" --alternative "$I05_TARGET_B=$I05_EVIDENCE_B"
./yai case cognitive show case:i05-manual --participant participant:model --json
```

Inspect `target_policy.kind=ordered_eligible`: A is preference zero, B is one;
neither choice grants provider admission by itself.

## Canonical SEND, arbitration and exact execution

```sh
./yai case conversation draft create case:i05-manual i05-input --participant participant:model
./yai case conversation draft add-text case:i05-manual i05-input --text 'I05 ordered choice: preserve this exact submitted input.'
./yai case conversation draft send case:i05-manual i05-input
read -r I05_TURN
```

Paste the printed `turn_id`. Planning does not invoke the provider; realization
does. Inspect the selected target, candidate exclusions and exact lane:

```sh
./yai case cognitive plan case:i05-manual --participant participant:model --capability primary_conversation --source "$I05_TURN" --shape text_to_text --json
./yai case cognitive realize case:i05-manual --participant participant:model --capability primary_conversation --turn "$I05_TURN" --json
./yai case conversation turn show case:i05-manual "$I05_TURN" --participant participant:model --json
```

A must be selected. Revoke its current trust and explicitly start a new planning
cycle; B must be selected with A's `trust_not_approved` exclusion:

```sh
./yai provider trust deny "$I05_TARGET_A"
./yai case cognitive plan case:i05-manual --participant participant:model --capability primary_conversation --source "$I05_TURN" --shape text_to_text --json
./yai case cognitive realize case:i05-manual --participant participant:model --capability primary_conversation --turn "$I05_TURN" --json
```

This is an explicit new execution after a recorded successful result, not
automatic failover after uncertain delivery. The automated I05 qualification
separately proves that indeterminate delivery forbids cross-target retry.

## Pinned negative path and reinspection

Replace the policy explicitly with pinned B, make A eligible, and deny B:

```sh
./yai case cognitive bind case:i05-manual --participant participant:model --role primary --capability primary_conversation --target "$I05_TARGET_B" --evidence "$I05_EVIDENCE_B" --replace
./yai provider trust approve "$I05_TARGET_A"
./yai provider trust deny "$I05_TARGET_B"
./yai case cognitive plan case:i05-manual --participant participant:model --capability primary_conversation --source "$I05_TURN" --shape text_to_text --json
./yai case cognitive show case:i05-manual --participant participant:model --json
./yai case conversation turn show case:i05-manual "$I05_TURN" --participant participant:model --json
./yai case show case:i05-manual --json
```

The pinned plan must be unresolved, with no selected target and no execution;
eligible A cannot replace B. Each CLI command starts a fresh process and reopens
the canonical store. The Turn and its content identity must remain unchanged.

## Cleanup

Stop the fixture processes in terminals A and B with Ctrl-C. In terminal C,
remove only the isolated home created above:

```sh
printf '%s\n' "$I05_HOME"
rm -r -- "$I05_HOME"
unset YAI_HOME I05_HOME I05_TARGET_A I05_TARGET_B I05_EVIDENCE_A I05_EVIDENCE_B I05_TURN
```

For the complete automated local qualification, including auxiliary composition
and failure/restart evidence, use `make smoke-cognitive-target-arbitration`.
For release use the TEST.TOPOLOGY.0 publication gate, with the pinned terminal
test dependencies installed as documented in tests/README.md. Real YVEX remains
the separate opt-in `make test-external-yvex` lane.
