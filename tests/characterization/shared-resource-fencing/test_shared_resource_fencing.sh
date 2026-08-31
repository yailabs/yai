#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"

output=$(cd "$ROOT/engine" && cargo test -p yai-engine \
  wave14_shared_resource_epoch_blocks_competitor_and_stale_carrier \
  -- --nocapture 2>&1)
grep -F "wave14_fencing:" <<<"$output"
grep -F "resource_temporarily_owned" <<<"$output"
grep -F "stale_resource_fence" <<<"$output"
printf 'shared_resource_fencing_characterization: pass\n'
"$ROOT/tests/characterization/shared-resource-fencing/test_cross_process_fencing.sh"
