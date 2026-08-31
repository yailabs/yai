#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"

output=$(cd "$ROOT/engine" && cargo test -p yai-engine \
  wave14_process_signal_uses_same_authority_spine_and_exact_birth_fence \
  -- --nocapture 2>&1)
grep -F "wave14_process_carrier:" <<<"$output"
grep -F "syscall_accepted=true" <<<"$output"
grep -F "finalized=true" <<<"$output"
printf 'second_carrier_characterization: pass\n'
