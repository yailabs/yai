# Secure Peering Cross-Repo Impact (NP-1/NP-4)

## `yai`

- define and enforce ingress role separation
- preserve owner canonical truth for all peer-originated payloads
- keep remote peer ingress source-scoped and trust-gated
- treat secure overlay as deployment precondition for non-local customer-grade use

## `yai-sdk`

- endpoint model must represent local control target vs owner peer target
- errors must distinguish unresolved target, unsupported transport, unreachable peer ingress
- SDK docs must describe overlay endpoint usage as canonical for non-local owner targets

## `yai-cli`

- command/help must state LAN vs secure peering expectations
- source-plane operator flow must not imply Internet-grade security without secure peering
- troubleshooting flow should check overlay reachability before protocol-layer debugging

## `yai-law`

- governance slice must encode trust/provenance assumptions for remote source attachment
- runtime entrypoint/attachability constraints must retain centralized-control semantics
- overlay presence is an infrastructure precondition and does not replace governance trust semantics
