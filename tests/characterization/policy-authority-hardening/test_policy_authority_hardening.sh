#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
export CARGO_TARGET_DIR="$ROOT/target"

cargo test --manifest-path "$ROOT/engine/yai-engine/Cargo.toml" h10_ -- --nocapture
cargo test --manifest-path "$ROOT/engine/yai-engine/Cargo.toml" \
  wave10_policy_mutation_between_decision_and_grant_fails_in_same_transaction -- --nocapture

printf 'policy_authority_hardening:canonical_write_rederivation ok\n'
printf 'policy_authority_hardening:canonical_evidence_and_review ok\n'
printf 'policy_authority_hardening:grant_adjacency_and_historical_replay ok\n'
