#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/yai"
TMP_DIR="${TMPDIR:-/tmp}/yai-spine46-$$"
YAI_HOME="$TMP_DIR/home"
FACT_DB="$YAI_HOME/store/facts/yai-facts.duckdb"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

require_line() {
  local haystack="$1"
  local needle="$2"
  grep -Fq -- "$needle" <<<"$haystack"
}

mkdir -p "$YAI_HOME"

status_before="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts status)"
require_line "$status_before" "fact_plane:"
require_line "$status_before" "backend: duckdb"
require_line "$status_before" "status: not_initialized"
require_line "$status_before" "schema: yai.fact.v1"
printf "fact_plane:status ok\n"

schema="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts schema)"
require_line "$schema" "fact_schema: yai.fact.v1"
require_line "$schema" "bitemporal: yes"
require_line "$schema" "fact_receipt"
require_line "$schema" "fact_decision"
require_line "$schema" "fact_projection"
require_line "$schema" "fact_carrier_outcome"
require_line "$schema" "fact_divergence"
require_line "$schema" "fact_replay"
require_line "$schema" "fact_runtime_graph"
require_line "$schema" "fact_model_behavior"
require_line "$schema" "fact_policy_outcome"
require_line "$schema" "fact_memory_quality"
require_line "$schema" "fact_retrieval_quality"
require_line "$schema" "fact_provider_runtime"
printf "fact_plane:schema ok\n"

require_line "$schema" "transaction_time"
require_line "$schema" "valid_time_start"
require_line "$schema" "valid_time_end"
require_line "$schema" "known_at"
require_line "$schema" "revision_of"
require_line "$schema" "superseded_by"
require_line "$schema" "retracted_by"
printf "fact_plane:bitemporal ok\n"

require_line "$schema" "facts_are_truth: false"
require_line "$status_before" "facts_are_truth: false"
require_line "$status_before" "operational_truth: false"
printf "fact_plane:not_truth ok\n"

init="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts init)"
require_line "$init" "facts_init:"
require_line "$init" "backend: duckdb"
require_line "$init" "status: ready"
require_line "$init" "schema: yai.fact.v1"
require_line "$init" "tables_created: 12"
require_line "$init" "facts_extracted: 0"
test -f "$FACT_DB"
printf "fact_plane:init ok\n"

tables="$(duckdb "$FACT_DB" -csv -noheader -c "SELECT count(*) FROM information_schema.tables WHERE table_schema = 'main' AND table_name LIKE 'fact_%';")"
test "$tables" = "12"
printf "fact_plane:tables ok\n"

status_after="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" facts status)"
require_line "$status_after" "status: ready"
require_line "$status_after" "tables: 12"
require_line "$status_after" "facts_extracted: 0"
printf "fact_plane:no_extraction confirmed\n"
