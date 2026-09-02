# H18 failure evidence

## Baseline defects reproduced by direct source inspection

These are direct reads of the exact clean H18 baseline, not reconstructed
runtime transcripts.

### F-H18-01 — derived current pointers were read as authority

Command:
`git show 47986c5:engine/yai-engine/src/store/lmdb.rs | rg -n 'qualification-current|trust-current'`

```text
13092:            &format!("qualification-current:{target_id}"),
13105:            &format!("trust-current:{target_id}"),
13278:        let current_key = format!("qualification-current:{target_id}");
13349:            &format!("trust-current:{}", target.target_id),
```

After fix, reads scan and validate immutable history; corrupt current cache
copies are ignored and exact replay succeeds.

### F-H18-02 — universal provenance ranking and overclaim

Command:
`git show 47986c5:engine/yai-engine/src/provider_governance.rs | rg -n 'fn rank|ExtensionAttested|FirstPartyTelemetry'`

```text
297:    FirstPartyTelemetry,
306:    ExtensionAttested,
310:    pub fn rank(&self) -> u8 {
315:            Self::ExtensionAttested => 3,
396:            capability: ProviderCapability::FirstPartyTelemetry,
397:            provenance: CapabilityProvenance::ExtensionAttested,
```

After fix, v2 uses extension-compatible/observed semantics and an explicit
capability/provenance admissibility matrix; unrelated cross-promotion fails.

### F-H18-03 — implicit continuation retry

Command:
`git show 47986c5:cmd/yai/src/provider.rs | rg -n 'invalid_continuation'`

```text
1966:            .contains("invalid_continuation")
```

The baseline retried without continuation by parsing provider body text. H18
removes that implicit second dispatch. Provider/model switching rebuilds from
canonical Case state in a new admitted attempt.

### F-H18-04 — sandbox and harness findings

The first full run failed two socket fixture tests with `Operation not
permitted`; the identical authorized loopback run passed. Classification:
`DEPLOYMENT_LIMITATION`, not product failure. A subsequent hardening smoke
assertion expected human field spelling inside the typed JSON envelope; tracing
showed correct JSON fields and the harness was corrected. Classification:
test-harness defect.

### F-H18-05 — obsolete automatic-continuation-retry expectation

The first complete `make check` after removing body-text retry heuristics ended
at the historical semantic-continuity smoke:

```text
yai: provider_remote_response:409:bytes=3458
make: *** [Makefile:580: smoke-semantic-continuity] Error 2
```

The fixture still required a second automatic request after generic HTTP 409.
That behavior conflicts with the H18 delivery contract: a generic status/body
does not prove non-execution. The characterization now requires zero automatic
retry, then performs a separate explicit invocation with a canonical rebuilt
frame. The identical focused path and complete rerun produced:

```text
semantic_continuity:unsafe_continuation_retry_refused_and_restart ok
```

Classification: superseded test contract, not a regression to unsafe retry.

No secret leak, duplicate result, unsafe failover, trust fork, forged
capability, stale-health resurrection, DNS locality escape, TLS downgrade or
extension-to-authority promotion remained after correction.
