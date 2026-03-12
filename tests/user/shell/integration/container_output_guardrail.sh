#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CLI="$REPO/dist/bin/yai"
WS_ID="ws_ops_review"
TMP="$(mktemp -d /tmp/yai-cli-ws-output-XXXXXX)"
trap 'rm -rf "$TMP"' EXIT

"$CLI" down >/dev/null 2>&1 || true
set +e
"$CLI" up >/dev/null 2>&1
RC_UP=$?
set -e
if [[ "$RC_UP" -ne 0 && "$RC_UP" -ne 40 ]]; then
  echo "expected up rc 0 or 40, got $RC_UP"
  exit 1
fi
if [[ "$RC_UP" -eq 40 ]]; then
  echo "container_output_guardrail: skipped (runtime unavailable)"
  exit 0
fi

"$CLI" ws clear >/dev/null 2>&1 || true
"$CLI" ws create "$WS_ID" >/dev/null
"$CLI" ws set "$WS_ID" >"$TMP/set.txt" 2>&1
rg -n "^Container set$" "$TMP/set.txt" >/dev/null
rg -n "^  Prompt token +.*${WS_ID}$" "$TMP/set.txt" >/dev/null
"$CLI" ws domain set --family economic --specialization payments >/dev/null
"$CLI" ws policy attach customer.default.org-container-contextual-review >"$TMP/attach.txt" 2>&1
rg -n "^Container policy attach$" "$TMP/attach.txt" >/dev/null

set +e
"$CLI" ws graph summary >"$TMP/graph_summary.txt" 2>&1
RC_GRAPH=$?
set -e
if [[ "$RC_GRAPH" -eq 0 ]]; then
  rg -n "^Container graph$|^Container query$" "$TMP/graph_summary.txt" >/dev/null
  rg -n "^Query$|^Read path$" "$TMP/graph_summary.txt" >/dev/null
  if rg -n "^  Summary +ok$" "$TMP/graph_summary.txt" >/dev/null; then
    echo "ws graph summary still collapsed to generic summary"
    cat "$TMP/graph_summary.txt"
    exit 1
  fi
else
  rg -n "^(BAD ARGS|PROTOCOL ERROR|SERVER UNAVAILABLE)$" "$TMP/graph_summary.txt" >/dev/null
fi

set +e
"$CLI" ws db status >"$TMP/db_status.txt" 2>&1
RC_DB_STATUS=$?
set -e
if [[ "$RC_DB_STATUS" -eq 0 ]]; then
  rg -n "^Container status$|^Container inspect$" "$TMP/db_status.txt" >/dev/null
else
  rg -n "^(BAD ARGS|PROTOCOL ERROR|SERVER UNAVAILABLE)$" "$TMP/db_status.txt" >/dev/null
fi

set +e
"$CLI" ws data evidence >"$TMP/data_evidence.txt" 2>&1
RC_DATA_EVIDENCE=$?
set -e
if [[ "$RC_DATA_EVIDENCE" -eq 0 ]]; then
  rg -n "^Container data$|^Container query$" "$TMP/data_evidence.txt" >/dev/null
  rg -n "^Query$|^Read path$" "$TMP/data_evidence.txt" >/dev/null
  if rg -n "^  Summary +ok$" "$TMP/data_evidence.txt" >/dev/null; then
    echo "ws data evidence still collapsed to generic summary"
    cat "$TMP/data_evidence.txt"
    exit 1
  fi
else
  rg -n "^(BAD ARGS|PROTOCOL ERROR|SERVER UNAVAILABLE)$" "$TMP/data_evidence.txt" >/dev/null
fi

set +e
"$CLI" ws cognition transient >"$TMP/cognition_transient.txt" 2>&1
RC_KNOWLEDGE=$?
set -e
if [[ "$RC_KNOWLEDGE" -eq 0 ]]; then
  rg -n "^Container cognition$|^Container query$|^Container inspect$" "$TMP/cognition_transient.txt" >/dev/null
else
  rg -n "^(BAD ARGS|PROTOCOL ERROR|SERVER UNAVAILABLE)$" "$TMP/cognition_transient.txt" >/dev/null
fi

set +e
"$CLI" ws recovery status >"$TMP/recovery_status.txt" 2>&1
RC_RECOVERY=$?
set -e
if [[ "$RC_RECOVERY" -eq 0 ]]; then
  rg -n "^Container status$|Recovery" "$TMP/recovery_status.txt" >/dev/null
else
  rg -n "^(BAD ARGS|PROTOCOL ERROR|SERVER UNAVAILABLE)$" "$TMP/recovery_status.txt" >/dev/null
fi

"$CLI" ws current >"$TMP/current.txt" 2>&1
rg -n "^Container current$" "$TMP/current.txt" >/dev/null
rg -n "^  Prompt token +.*${WS_ID}$" "$TMP/current.txt" >/dev/null

"$CLI" ws status >"$TMP/status_bound.txt" 2>&1
rg -n "^Container status$" "$TMP/status_bound.txt" >/dev/null
rg -n "^Binding$|^Runtime$|^Context$|^Runtime capabilities$" "$TMP/status_bound.txt" >/dev/null
rg -n "^  Active +yes$|^  Binding +active$" "$TMP/status_bound.txt" >/dev/null
rg -n "^  Container selected +yes$|^  Container bound +yes$" "$TMP/status_bound.txt" >/dev/null
rg -n "^  Exec +|^  Data +|^  Graph +|^  Cognition +|^  Recovery state +" "$TMP/status_bound.txt" >/dev/null

set +e
"$CLI" ws inspect >"$TMP/ws_inspect.txt" 2>&1
RC_WS_INSPECT=$?
set -e
if [[ "$RC_WS_INSPECT" -eq 0 ]]; then
  rg -n "^Container inspect$" "$TMP/ws_inspect.txt" >/dev/null
  rg -n "^Identity$|^Session$|^Runtime capabilities$|^Normative context$|^Resolution$|^Governance$|^Digital$|^Scientific$" "$TMP/ws_inspect.txt" >/dev/null
else
  rg -n "^(BAD ARGS|PROTOCOL ERROR|SERVER UNAVAILABLE)$" "$TMP/ws_inspect.txt" >/dev/null
fi

set +e
"$CLI" inspect runtime >"$TMP/inspect_runtime.txt" 2>&1
RC_INSPECT_RUNTIME=$?
set -e
if [[ "$RC_INSPECT_RUNTIME" -eq 0 ]]; then
  rg -n "^Runtime status$" "$TMP/inspect_runtime.txt" >/dev/null
  rg -n "^Baseline$|^Container binding$|^Capability families$" "$TMP/inspect_runtime.txt" >/dev/null
  rg -n "^  Liveness +|^  Readiness +|^  Exec +|^  Data +|^  Graph +|^  Cognition +" "$TMP/inspect_runtime.txt" >/dev/null
else
  rg -n "^(SERVER UNAVAILABLE|PROTOCOL ERROR)$" "$TMP/inspect_runtime.txt" >/dev/null
fi

set +e
"$CLI" ws policy effective >"$TMP/policy.txt" 2>&1
RC_POLICY=$?
set -e
if [[ "$RC_POLICY" -eq 0 ]]; then
  rg -n "^Container policy effective$" "$TMP/policy.txt" >/dev/null
  rg -n "^Effective stack$|^Outcome$|^Governance$|^Scientific$|^Digital$" "$TMP/policy.txt" >/dev/null
else
  rg -n "^(BAD ARGS|PROTOCOL ERROR|SERVER UNAVAILABLE)$" "$TMP/policy.txt" >/dev/null
fi

set +e
"$CLI" ws debug resolution >"$TMP/debug.txt" 2>&1
RC_DEBUG=$?
set -e
if [[ "$RC_DEBUG" -eq 0 ]]; then
  rg -n "^Container debug resolution$" "$TMP/debug.txt" >/dev/null
  rg -n "^Context sources$|^Resolution$|^Scientific$|^Digital$" "$TMP/debug.txt" >/dev/null
else
  rg -n "^(BAD ARGS|PROTOCOL ERROR|SERVER UNAVAILABLE)$" "$TMP/debug.txt" >/dev/null
fi

set +e
"$CLI" ws run payment.authorize provider=bank resource=money-transfer amount=1200 authority=supervisor >"$TMP/run.txt" 2>&1
RC_RUN=$?
set -e
if [[ "$RC_RUN" -eq 0 ]]; then
  rg -n "^Container run$" "$TMP/run.txt" >/dev/null
  rg -n "^Execution$|^Outcome$|^Scientific$|^Digital$" "$TMP/run.txt" >/dev/null
else
  if rg -n "^Container run$" "$TMP/run.txt" >/dev/null; then
    rg -n "^Execution$|^Outcome$" "$TMP/run.txt" >/dev/null
  else
    rg -n "^(BAD ARGS|DENIED|SERVER UNAVAILABLE|PROTOCOL ERROR)$" "$TMP/run.txt" >/dev/null
  fi
fi

"$CLI" ws prompt-token >"$TMP/prompt.txt" 2>&1
if ! rg -n "^◉ ${WS_ID}$|^o ${WS_ID}$" "$TMP/prompt.txt" >/dev/null; then
  echo "unexpected prompt token output"
  cat "$TMP/prompt.txt"
  exit 1
fi
if rg -n "ws:" "$TMP/prompt.txt" >/dev/null; then
  echo "prompt token must not use ws: prefix"
  cat "$TMP/prompt.txt"
  exit 1
fi

# token must depend on binding, not current cwd
(
  cd /tmp
  "$CLI" ws prompt-token >"$TMP/prompt_from_tmp.txt" 2>&1
)
if ! cmp -s "$TMP/prompt.txt" "$TMP/prompt_from_tmp.txt"; then
  echo "prompt token changed with cwd; binding-based token expected"
  echo "--- from repo cwd ---"
  cat "$TMP/prompt.txt"
  echo "--- from /tmp ---"
  cat "$TMP/prompt_from_tmp.txt"
  exit 1
fi

"$CLI" ws clear >"$TMP/clear.txt" 2>&1 || true
rg -n "^Container cleared$" "$TMP/clear.txt" >/dev/null
rg -n "^  Binding +(active|no_active)$" "$TMP/clear.txt" >/dev/null
"$CLI" ws policy detach customer.default.org-container-contextual-review >"$TMP/detach.txt" 2>&1 || true
"$CLI" ws unset >"$TMP/unset.txt" 2>&1 || true
rg -n "^Container unset$" "$TMP/unset.txt" >/dev/null
"$CLI" ws status >"$TMP/status_unbound.txt" 2>&1 || true
rg -n "^Container status$|^SERVER UNAVAILABLE$" "$TMP/status_unbound.txt" >/dev/null
if ! rg -n "^SERVER UNAVAILABLE$" "$TMP/status_unbound.txt" >/dev/null; then
  rg -n "^  Active +no$|^  Container selected +no$|^  Container bound +no$" "$TMP/status_unbound.txt" >/dev/null
fi
"$CLI" ws prompt-token >"$TMP/prompt_after_unset.txt" 2>&1 || true
if [[ -s "$TMP/prompt_after_unset.txt" ]]; then
  echo "prompt token should be empty after ws unset"
  cat "$TMP/prompt_after_unset.txt"
  exit 1
fi
"$CLI" down >/dev/null 2>&1 || true
echo "container_output_guardrail: ok"
