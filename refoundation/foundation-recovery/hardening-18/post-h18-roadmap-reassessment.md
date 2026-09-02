# Post-H18 roadmap reassessment

H18 closes the currently evidenced single-store provider-governance foundation.
The repository does not justify automatically declaring W19.

Remaining gaps by class:

- Distributed-systems gap: cross-host shared provider governance, remote health
  consensus, network partitions and replicated Case authority are not claimed.
- Deployment gap: production certificate/private-CA provisioning, DNS resolver
  policy, outbound network policy and credential-store integration remain
  deployment-owned.
- Observability gap: operator-facing long-horizon health and selection metrics
  may be useful, but they are projections and do not require a semantic owner.
- Performance gap: the measured 32-target selection scan is bounded but not
  cheap at 1,000 canonical writes; optimize only if a product workload proves
  it material.
- Product/API gap: explicit trust/qualification retention policy and richer
  certificate diagnostics could improve administration without changing
  semantics.
- Business/product concern: cost, latency, model quality and commercial
  provider choice remain intentionally outside mechanical capability routing.
- Hardening gap: real Internet TLS/DNS chaos, credential-store atomic rotation,
  long-lived asynchronous response delivery and extension authenticity need
  deployment-specific evidence before stronger claims.

No remaining item currently demonstrates a missing semantic owner or a new
Foundation Wave. The next work should be selected from observed deployment or
product pressure, not the numbering sequence.
