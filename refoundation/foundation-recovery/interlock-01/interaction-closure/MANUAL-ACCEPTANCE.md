# MANUAL ACCEPTANCE — ZERO TO USE CASE

This acceptance uses ordinary commands in execution order. It contains no
`set -e`, `test`, Python extraction, hidden Case state, or internal file edits.
The low-level draft commands are intentionally Advanced qualification plumbing,
not the final human chat journey.

Prerequisites: Linux, Rust/C build dependencies already required by YAI, and a
checkout at the repository root. The focused host smoke starts only a local
fixture provider and sends no data externally.

```sh
make build-rust
make smoke-conversation-interaction-host

export YAI_HOME="$(mktemp -d /tmp/yai-post-i01-acceptance.XXXXXX)"

./target/debug/yai init --tenant tenant:post-i01-acceptance --organization organization:cli-product
./target/debug/yai doctor

./target/debug/yai case create case:post-i01-acceptance --tenant tenant:post-i01-acceptance
./target/debug/yai case participant role add case:post-i01-acceptance --participant participant:chat --role model-executor
./target/debug/yai case participant link-principal case:post-i01-acceptance --principal self --participant participant:chat
./target/debug/yai case participant view admit case:post-i01-acceptance --participant participant:chat --consumer model --view model_context

./target/debug/yai case conversation draft create case:post-i01-acceptance first-turn --participant participant:chat --thread thread:manual-one
./target/debug/yai case conversation draft add-text case:post-i01-acceptance first-turn --text "Primo Turn canonico, indipendente dal provider."
./target/debug/yai case conversation draft send case:post-i01-acceptance first-turn

./target/debug/yai case conversation draft create case:post-i01-acceptance second-turn --participant participant:chat --thread thread:manual-two
./target/debug/yai case conversation draft add-text case:post-i01-acceptance second-turn --text "Secondo Turn in una chat distinta ma nello stesso Case."
./target/debug/yai case conversation draft send case:post-i01-acceptance second-turn

./target/debug/yai case conversation turn list case:post-i01-acceptance --participant participant:chat
./target/debug/yai case conversation turn show case:post-i01-acceptance latest --participant participant:chat
./target/debug/yai case show case:post-i01-acceptance --json

./target/debug/yai help --advanced
./target/debug/yai prompt --help

rm -rf "$YAI_HOME"
unset YAI_HOME
```

Important expected postures:

- both SEND commands print `canonical: yes` and
  `provider_execution_started: no`;
- Turn list shows two committed thread identities derived from Turn history;
- Turn show reports `content_integrity: verified`;
- `make smoke-conversation-interaction-host` reports two passing focused tests,
  including provider failure preservation, retry without duplicate Turn, and
  execution with no ResourceAttachment;
- Advanced help retains the draft plumbing and frozen `yai prompt`;
- normal Product help does not claim a temporary `yai chat`.

Negative path: the focused host smoke commits a Turn with no provider binding
and verifies the explicit `provider_unavailable` posture while the Turn remains
canonical. It also submits typed media and verifies
`typed_media_adapter_pending` without flattening or losing the content.

Interactive terminal editing/history/paste/key/redraw acceptance is not in
scope and remains `awaiting_replia_integration`. No private YAI substitute is
claimed.
