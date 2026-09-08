# Tests and executable evidence

Tests are classified by **the property they prove**, not domain nouns or their
directory. [classification.tsv](classification.tsv) is the single executable
classification authority; [validation guide](../docs/test-cases.md) owns current
procedures. Historical work records are not current gates.

## Choose a gate

| Command | Claim and environment |
|---|---|
| `make test-fast` | guards, pure Rust tests, bounded C components, topology assertions; no provider |
| `make test-local` | deterministic local proof, real loopback transport, product flows and recovery; excludes separately classified endurance |
| `make check` / `make test-release` | publication union: all previous check/characterization proof, including formerly default bounded endurance; no external provider |
| `make characterization` | behavior-freezing posture across proof classes; not another test level |
| `make check characterization` | shared Make leaves execute once in this invocation |
| `make test-endurance` | explicit scale lane, including previously ignored W19/H19/W20 characterizations |
| `make test-external-yvex` | opt-in real-provider text interoperability; absence is nonzero; no fixture fallback |
| `make qualification-provider-connection` | opt-in real public provider through guided REPLAI `/connect` and cognitive SEND; exact operator endpoint/model; not full Golden tools |
| `make test-manual` | list operator procedures only; NOT an acceptance run or PASS |
| `make test-golden-local` | explicit cumulative reference free/Workflow lifecycle; real Product, persistence and resource peers, loopback model; separate from `check` |
| `make test-golden-external-yvex` | same reference product lifecycle using the operator's real endpoint/model; no model fixture; absent deployment refuses |

Independent diagnostic lanes: `test-unit`, `test-component`, `test-contract`,
`test-product`, `test-recovery`. `build` and `build-rust` compile, **not test**.
`smoke` remains the named smoke-leaf union; individual historical commands stay
available. They are not replacements for the complete publication union.

The native terminal product lane also requires the pinned Python packages in
[requirements-terminal.txt](requirements-terminal.txt). See
[native REPLAI reproduction](../docs/replai-terminal.md#reproduction-and-evidence)
for the isolated virtualenv setup before running the publication union.

`smoke-provider-connection` exercises the sole `/connect` through real PTYs with
strict string-only, fully capable and malformed loopback providers: text must
qualify, tools/JSON must be reported exactly, then canonical cognitive SEND and
replay must succeed. No connection profile is exposed or grants missing shapes.
Catalog scenarios additionally prove singleton/multiple
selection, auth challenge, malformed/empty/duplicate refusal, catalog/Case drift,
cancelled consent and explicit replacement. No hardcoded model entry is needed
for a singleton. Its external counterpart requires
`YAI_EXTERNAL_PROVIDER_BASE_URL` and `YAI_EXTERNAL_PROVIDER_MODEL`; the supplied
endpoint must be a credential-free loopback endpoint (including an operator-owned
SSH forward). Absence exits 3, with no fixture fallback. The driver's explicit
`--external --qualification-only` mode tests synthetic native function/result
and JSON contracts only, not a user SEND or the complete Golden lifecycle.

## Orthogonal evidence axes

Each row binds a stable ID, source paths, exact selector/executor, property,
proof class, posture, provider mode, network, mutation, cadence, entrypoint and
reachability. Compiled Rust identities are explicit: new/removed tests require
catalog reconciliation.

- Proof: `unit`, `component`, `contract`, `product`, `recovery`, `endurance`,
  `external`, `manual`; `historical` explicitly means non-executable evidence.
- Posture: `regression`, `characterization`, `qualification`; `support` and
  `historical` rows are not standalone passes.
- Provider: `no_provider`, `loopback_fixture`, `external_yvex`, `external_other`.
  Provider-shaped records constructed in-process are not provider execution.
  Local IPC is not provider HTTP.
- Cadence: fast, publication, explicit golden/endurance/external/manual, or support.

Mixed verticals use their strongest/principal property and declare dependencies.
A C component plus CLI compatibility assertion is not a governed product effect.
Bounded scale checks embedded in verticals remain there. Separately classified
endurance is independently selectable; formerly default tests remain in release.

Local execution requires installed toolchain/native/Cargo dependencies, not a
deployed model or internet service. Validation exports one absolute Cargo target
directory and offline Cargo mode. This is **not an OS network sandbox** or a
timing reproducibility guarantee: Linux facilities, sockets, processes, native
libraries, temporary files and build outputs are real dependencies/mutations.
`no_provider` does not mean `pure`, `read_only` or `product_reachable`.

## Auditing and reporting

```sh
make check-validation-topology
python3 tools/validation/topology.py list --lane contract
python3 tools/validation/topology.py list --lane recovery
python3 tools/validation/topology.py list --lane external
```

The catalog compiles to an ordinary Make include under `build/`. The audit
checks assertion files, Make leaves, exact compiled Rust tests, old coverage,
actual dry-run reachability/duplication, external/local contradictions and the
`./yai` launcher in shell procedures. Empty/stale Rust selections cannot pass.
Rust partitions run actual libtest binaries, not replacement assertions.

Make leaves emit `validation_entry`; Rust partitions emit exact commands,
selected count, provider modes, exit and elapsed time. Wave reports retain the
outer run ID, baseline/pre-state, cwd/environment, command, exit and bounded raw
output. Metadata alone never proves PASS; grand test totals are insufficient.

There is no persistent PASS cache. Separate gate invocations deliberately rerun
overlapping proof. Some vertical scripts re-run Rust assertions **and verify
diagnostic output**; these repetitions remain pending safe factoring. Full Rust
suites hidden inside build and recursive characterization build are removed.

## Corpus and claim limits

`characterization/` holds behavior-freezing verticals and topology assertions;
`smoke/` retains product, fixture and C components; `fixtures/` holds peers/data;
`cases/` holds procedures. Rust tests remain beside implementation. No directory
shuffle, provider/test service, database or runtime owner is introduced.

W19/H19/W20 and I01–I06 remain publication-reachable. The I06 host recovery
leaf is `smoke-conversation-cognitive-host`; `smoke-replai-terminal` exercises
real PTY SEND/arbitration/pinning/retry. Canonical intent/adoption/replay runs in
the no-provider Rust recovery lane. Loopback proves YAI
semantics and real generic adapter transport, **not live YVEX multipart
compatibility or model quality**. External YVEX text qualification has a separate
scope; public typed-media external qualification remains integration work.

`smoke-guided-case-setup` separately proves the short `init`/`open` Product
journey with a real REPLAI PTY and **no provider**: explicit consent,
cancellation, existing Case preservation and selection. The Golden local free
and Workflow runs use guided `/connect`, `/review`, resource/policy setup and
bare `/retry` over the real controller and loopback provider transport. Neither
lane authorizes a human PASS or live YVEX claim.
