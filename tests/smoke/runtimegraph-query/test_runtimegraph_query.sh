#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
YAI_BIN="$ROOT/target/debug/yai"
TMP_DIR="${TMPDIR:-/tmp}/yai-spine44-$$"
YAI_HOME="$TMP_DIR/home"
JOURNAL="$TMP_DIR/journal.jsonl"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

mkdir -p "$TMP_DIR" "$YAI_HOME"

cat > "$JOURNAL" <<'JSONL'
{"schema":"yai.store.record.v0","record_id":"rec:spine44-attempt","case_ref":"case:spine44","record_kind":"attempt","subject_ref":"subject:filesystem","attempt_id":"attempt:spine44-write","decision_id":"","receipt_id":"","summary":"attempt targets filesystem subject"}
{"schema":"yai.store.record.v0","record_id":"rec:spine44-decision","case_ref":"case:spine44","record_kind":"decision","subject_ref":"subject:filesystem","attempt_id":"attempt:spine44-write","decision_id":"decision:spine44-allow","receipt_id":"","summary":"decision controls attempt"}
{"schema":"yai.store.record.v0","record_id":"rec:spine44-receipt","case_ref":"case:spine44","record_kind":"effect_receipt","subject_ref":"subject:filesystem","attempt_id":"attempt:spine44-write","decision_id":"decision:spine44-allow","receipt_id":"receipt:spine44-write","summary":"receipt records effect"}
{"schema":"yai.store.record.v0","record_id":"rec:spine44-policy","case_ref":"case:spine44","record_kind":"policy_rule","subject_ref":"subject:filesystem","attempt_id":"","decision_id":"","receipt_id":"","summary":"policy constrains subject"}
JSONL

YAI_HOME="$YAI_HOME" "$YAI_BIN" security bootstrap-local \
  --tenant tenant:spine44 --organization organization:characterization >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" case create \
  --case case:spine44 --tenant tenant:spine44 >/dev/null
YAI_HOME="$YAI_HOME" "$YAI_BIN" case create \
  --case case:missing --tenant tenant:spine44 >/dev/null

YAI_HOME="$YAI_HOME" "$YAI_BIN" graph rebuild --case case:spine44 --from journal --path "$JOURNAL" >/dev/null

summary="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph runtime-summary --case case:spine44)"
grep -q "runtime_graph_summary:" <<<"$summary"
grep -Eq "edges_total: [1-9][0-9]*" <<<"$summary"

fanout="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph fanout --case case:spine44 --node decision:spine44-allow --limit 10)"
grep -q "graph_fanout:" <<<"$fanout"
grep -q "edge_kind: decision_controls_attempt" <<<"$fanout"
grep -q "from_ref: decision:spine44-allow" <<<"$fanout"
grep -q "to_ref: attempt:spine44-write" <<<"$fanout"
printf "runtime_graph_query:fanout ok\n"

fanin="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph fanin --case case:spine44 --node attempt:spine44-write --limit 10)"
grep -q "graph_fanin:" <<<"$fanin"
grep -q "edge_kind: decision_controls_attempt" <<<"$fanin"
grep -q "from_ref: decision:spine44-allow" <<<"$fanin"
grep -q "to_ref: attempt:spine44-write" <<<"$fanin"
printf "runtime_graph_query:fanin ok\n"

neighborhood="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph neighborhood --case case:spine44 --node decision:spine44-allow --depth 1 --limit 20)"
grep -q "graph_neighborhood:" <<<"$neighborhood"
grep -q "depth: 1" <<<"$neighborhood"
grep -Eq "nodes_total: [1-9][0-9]*" <<<"$neighborhood"
grep -Eq "edges_total: [1-9][0-9]*" <<<"$neighborhood"
printf "runtime_graph_query:neighborhood ok\n"

path_found="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph path --case case:spine44 --from decision:spine44-allow --to attempt:spine44-write --max-depth 4)"
grep -q "graph_path:" <<<"$path_found"
grep -q "path_status: found" <<<"$path_found"
grep -q "hops: 1" <<<"$path_found"
printf "runtime_graph_query:path_found ok\n"

path_missing="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph path --case case:spine44 --from decision:spine44-allow --to receipt:spine44-write --max-depth 4)"
grep -q "path_status: not_found" <<<"$path_missing"
printf "runtime_graph_query:path_not_found ok\n"

edge_filter="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph fanout --case case:spine44 --node decision:spine44-allow --edge-kind receipt_records_effect --limit 10)"
grep -q "edge_kind_filter: receipt_records_effect" <<<"$edge_filter"
grep -q "edges_total: 0" <<<"$edge_filter"
printf "runtime_graph_query:edge_filter ok\n"

bounded="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph fanout --case case:spine44 --node decision:spine44-allow --limit 999)"
grep -q "limit: 200" <<<"$bounded"
grep -q "limit_clamped: yes" <<<"$bounded"
depth_bounded="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph path --case case:spine44 --from decision:spine44-allow --to attempt:spine44-write --max-depth 99)"
grep -q "max_depth: 6" <<<"$depth_bounded"
grep -q "max_depth_clamped: yes" <<<"$depth_bounded"
printf "runtime_graph_query:bounded ok\n"

empty="$(YAI_HOME="$YAI_HOME" "$YAI_BIN" graph fanout --case case:missing --node decision:missing --limit 10)"
grep -q "graph_fanout:" <<<"$empty"
grep -q "edges_total: 0" <<<"$empty"
