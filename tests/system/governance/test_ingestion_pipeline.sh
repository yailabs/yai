#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
CLI="$ROOT/tools/bin/yai-govern"
SRC_ID="src.sample.digital-outbound"
NORM_ID="norm.src-sample-digital-outbound"
CID="enterprise.sample.src-sample-digital-outbound.candidate.v1"

"$CLI" source inspect "$SRC_ID" >/dev/null
"$CLI" parse "$SRC_ID" >/dev/null
"$CLI" parsed inspect "$SRC_ID" >/dev/null
"$CLI" normalize "$SRC_ID" >/dev/null
"$CLI" normalized inspect "$NORM_ID" >/dev/null
"$CLI" build "$NORM_ID" >/dev/null
"$CLI" candidate inspect "$CID" >/dev/null
"$CLI" validate "$CID" >/dev/null
"$CLI" review status "$CID" >/dev/null
"$CLI" status "$CID" >/dev/null

echo "integration_ingestion_pipeline: ok"
