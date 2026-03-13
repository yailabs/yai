#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI="$ROOT/build/bin/yai"
SOCK="${YAI_RUNTIME_INGRESS:-$HOME/.yai/run/control.sock}"

if [[ ! -x "$YAI" ]]; then
  make -C "$ROOT" yai >/dev/null
fi

RUNTIME_PID=""
cleanup() {
  if [[ -n "$RUNTIME_PID" ]] && kill -0 "$RUNTIME_PID" 2>/dev/null; then
    kill "$RUNTIME_PID" >/dev/null 2>&1 || true
    wait "$RUNTIME_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

"$YAI" >/tmp/yai_orchestration_smoke_up.log 2>&1 &
RUNTIME_PID=$!

for _ in $(seq 1 50); do
  if [[ -S "$SOCK" ]]; then
    break
  fi
  sleep 0.1
done

if [[ ! -S "$SOCK" ]]; then
  echo "orchestration_smoke: FAIL (missing ingress socket $SOCK)"
  exit 1
fi

python3 "$ROOT/tests/legacy/runtime/integration/test_handshake.py"
echo "orchestration_smoke: ok"
