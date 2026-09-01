#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
YAI_HOME=${YAI_HOME:-"/tmp/yai-spine29-home-$$"}
export YAI_HOME

YAI_BIN="$ROOT/target/debug/yai"
RECORD_STORE="$YAI_HOME/store/lmdb"

cleanup() {
  rm -rf "$YAI_HOME"
}
trap cleanup EXIT INT TERM

require_line() {
  output="$1"
  expected="$2"
  if ! printf '%s\n' "$output" | grep -Fq "$expected"; then
    printf 'missing expected line: %s\noutput:\n%s\n' "$expected" "$output" >&2
    exit 1
  fi
}

reject_line() {
  output="$1"
  rejected="$2"
  if printf '%s\n' "$output" | grep -Fq "$rejected"; then
    printf 'unexpected line: %s\noutput:\n%s\n' "$rejected" "$output" >&2
    exit 1
  fi
}

rm -rf "$YAI_HOME"

missing=$("$YAI_BIN" store status)
require_line "$missing" "record_store_backend: lmdb"
require_line "$missing" "record_store_status: missing"
require_line "$missing" "record_store_path: $RECORD_STORE"
require_line "$missing" "schema: yai.record.v1"
reject_line "$missing" "record_store_status: ready"

doctor_missing=$("$YAI_BIN" doctor)
require_line "$doctor_missing" "Posture            NOT_INITIALIZED"
require_line "$doctor_missing" "Storage         missing"
require_line "$doctor_missing" "Storage backend lmdb"

mkdir -p "$RECORD_STORE"

not_initialized=$("$YAI_BIN" store status)
require_line "$not_initialized" "record_store_status: not_initialized"
reject_line "$not_initialized" "record_store_status: ready"

doctor_not_initialized=$("$YAI_BIN" doctor)
require_line "$doctor_not_initialized" "Posture            NOT_INITIALIZED"
require_line "$doctor_not_initialized" "Storage         not_initialized"
require_line "$doctor_not_initialized" "Storage backend lmdb"

printf 'record_store_cli:spine29 ok\n'
