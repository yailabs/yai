#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
OUT_DIR="$ROOT/build/test/integration_governance_resolution"
mkdir -p "$OUT_DIR"
export YAI_GOVERNANCE_CANONICAL_ONLY=1

LAW_SRCS=$(find "$ROOT/lib/governance" -type f -name '*.c' | sort)
CFLAGS='-Wall -Wextra -std=c11 -O2'
INCLUDES="-I$ROOT/include -I$ROOT/include/yai"

for t in test_d1_resolution test_d8_resolution test_economic_resolution test_overlay_resolution test_cross_layer_mixed_resolution test_cross_domain_resolution; do
  cc $CFLAGS $INCLUDES "$ROOT/tests/integration/governance/${t}.c" $LAW_SRCS -o "$OUT_DIR/$t"
  "$OUT_DIR/$t"
done

"$ROOT/tests/integration/governance/test_ingestion_pipeline.sh"

echo "governance_resolution_smoke: ok"
