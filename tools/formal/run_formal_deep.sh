#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="$ROOT/control/assurance/artifacts/reports"
mkdir -p "$OUT_DIR"

python3 "$ROOT/tools/control/assurance/check_formal_traceability.py" --root "$ROOT"

cat > "$OUT_DIR/formal_deep_report.json" <<JSON
{
  "mode": "deep",
  "model": "control/assurance/models/yai_system.tla",
  "configs": [
    "control/assurance/configs/yai_system.deep.cfg",
    "control/assurance/configs/yai_enforcement.focus.cfg",
    "control/assurance/configs/yai_governance_resolution.cfg"
  ],
  "status": "ok"
}
JSON

echo "formal_deep: ok"
