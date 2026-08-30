#!/bin/sh
set -eu

run_dir="/tmp/yai-daemon-core-loop-$$"
socket_path="$run_dir/yaid.sock"
log_path="$run_dir/yaid.log"

mkdir -p "$run_dir"
YAI_HOME="$run_dir/home"
export YAI_HOME
rm -f "$socket_path"

build/yaid --socket "$socket_path" --foreground >"$log_path" 2>&1 &
daemon_pid=$!

cleanup() {
  if kill -0 "$daemon_pid" 2>/dev/null; then
    kill "$daemon_pid" 2>/dev/null || true
    wait "$daemon_pid" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

tries=0
while [ ! -S "$socket_path" ]; do
  tries=$((tries + 1))
  if [ "$tries" -gt 50 ]; then
    cat "$log_path" >&2 || true
    exit 1
  fi
  sleep 0.1
done

status_output=$(target/debug/yai daemon status --socket "$socket_path")
minimum_output=$(target/debug/yai daemon run-minimum-loop --socket "$socket_path")
minimum_journal=$(printf '%s\n' "$minimum_output" | sed -n 's/.*"journal_path":"\([^"]*\)".*/\1/p')

if [ -z "$minimum_journal" ]; then
  printf '%s\n' "$minimum_output" >&2
  exit 1
fi

journal_output=$(target/debug/yai daemon journal-summary --socket "$socket_path" --journal "$minimum_journal")
projection_output=$(target/debug/yai daemon projection-summary --socket "$socket_path" --journal "$minimum_journal")
filesystem_output=$(target/debug/yai daemon run-filesystem-loop --socket "$socket_path")
filesystem_output=$(target/debug/yai daemon run-filesystem-loop --socket "$socket_path")
filesystem_journal=$(printf '%s\n' "$filesystem_output" | sed -n 's/.*"journal_path":"\([^"]*\)".*/\1/p')

if [ -z "$filesystem_journal" ]; then
  printf '%s\n' "$filesystem_output" >&2
  exit 1
fi

filesystem_summary=$(target/debug/yai daemon journal-summary --socket "$socket_path" --journal "$filesystem_journal")
filesystem_projection=$(target/debug/yai projection inspect --journal "$filesystem_journal")
security_bootstrap=$(target/debug/yai security bootstrap-local \
  --tenant tenant:daemon-core-loop --organization organization:characterization)
tenant_case=$(target/debug/yai case create \
  --case case:new12-filesystem --tenant tenant:daemon-core-loop)
filesystem_case_entry=$(YAI_JOURNAL="$filesystem_journal" target/debug/yai case enter --case case:new12-filesystem --subject subject:llm-provider)
filesystem_case_shell=$(YAI_JOURNAL="$filesystem_journal" target/debug/yai case enter --case case:new12-filesystem --subject subject:llm-provider --shell zsh)
filesystem_provider_attach=$(YAI_JOURNAL="$filesystem_journal" target/debug/yai case attach-provider --case case:new12-filesystem --subject subject:llm-provider --base-url http://127.0.0.1:43117/v1/chat/completions --model qwen-local)
filesystem_provider_shell=$(YAI_JOURNAL="$filesystem_journal" target/debug/yai case attach-provider --case case:new12-filesystem --subject subject:llm-provider --base-url http://127.0.0.1:43117/v1/chat/completions --model qwen-local --shell zsh)
filesystem_transcript_on=$(YAI_JOURNAL="$filesystem_journal" YAI_CASE_REF=case:new12-filesystem YAI_SUBJECT_REF=subject:llm-provider YAI_PROVIDER_BASE_URL=http://127.0.0.1:43117/v1/chat/completions YAI_PROVIDER_MODEL=qwen-local target/debug/yai prompt --once "/transcript on")
filesystem_transcript_status=$(YAI_JOURNAL="$filesystem_journal" YAI_CASE_REF=case:new12-filesystem YAI_SUBJECT_REF=subject:llm-provider YAI_PROVIDER_BASE_URL=http://127.0.0.1:43117/v1/chat/completions YAI_PROVIDER_MODEL=qwen-local target/debug/yai prompt --once "/transcript status")
filesystem_thread_status=$(YAI_JOURNAL="$filesystem_journal" YAI_CASE_REF=case:new12-filesystem YAI_SUBJECT_REF=subject:llm-provider YAI_PROVIDER_BASE_URL=http://127.0.0.1:43117/v1/chat/completions YAI_PROVIDER_MODEL=qwen-local target/debug/yai prompt --once "/thread status")
filesystem_thread_new=$(YAI_JOURNAL="$filesystem_journal" YAI_CASE_REF=case:new12-filesystem YAI_SUBJECT_REF=subject:llm-provider YAI_PROVIDER_BASE_URL=http://127.0.0.1:43117/v1/chat/completions YAI_PROVIDER_MODEL=qwen-local target/debug/yai prompt --once "/thread new clean")
filesystem_thread_use_default=$(YAI_JOURNAL="$filesystem_journal" YAI_CASE_REF=case:new12-filesystem YAI_SUBJECT_REF=subject:llm-provider YAI_PROVIDER_BASE_URL=http://127.0.0.1:43117/v1/chat/completions YAI_PROVIDER_MODEL=qwen-local target/debug/yai prompt --once "/thread use thread:default")
filesystem_memory_propose=$(YAI_JOURNAL="$filesystem_journal" YAI_CASE_REF=case:new12-filesystem YAI_SUBJECT_REF=subject:llm-provider YAI_PROVIDER_BASE_URL=http://127.0.0.1:43117/v1/chat/completions YAI_PROVIDER_MODEL=qwen-local target/debug/yai prompt --once "/memory propose smoke boundary residue")
filesystem_prompt_dry_run=$(YAI_JOURNAL="$filesystem_journal" YAI_CASE_REF=case:new12-filesystem YAI_SUBJECT_REF=subject:llm-provider YAI_PROVIDER_BASE_URL=http://127.0.0.1:43117/v1/chat/completions YAI_PROVIDER_MODEL=qwen-local target/debug/yai prompt --dry-run --once "What subjects are bound to this case?")
shutdown_output=$(target/debug/yai daemon shutdown --socket "$socket_path")

printf '%s\n' "$status_output" | grep '"status":"ok"' >/dev/null
printf '%s\n' "$minimum_output" | grep '"status":"completed"' >/dev/null
printf '%s\n' "$minimum_output" | grep '"receipt_status":"blocked"' >/dev/null
printf '%s\n' "$journal_output" | grep '"status":"ok"' >/dev/null
printf '%s\n' "$projection_output" | grep '"status":"ok"' >/dev/null
printf '%s\n' "$filesystem_output" | grep '"status":"completed"' >/dev/null
printf '%s\n' "$filesystem_output" | grep '"record_count":28' >/dev/null
printf '%s\n' "$filesystem_output" | grep '"fs_write_blocked":"blocked"' >/dev/null
printf '%s\n' "$filesystem_output" | grep '"fs_write_allowed":"fixture_no_effect"' >/dev/null
printf '%s\n' "$filesystem_output" | grep '"decision_outcome":"fixture_descriptor_only"' >/dev/null
printf '%s\n' "$filesystem_output" | grep '"case_session":"active"' >/dev/null
printf '%s\n' "$filesystem_output" | grep '"case_world":"loaded"' >/dev/null
printf '%s\n' "$filesystem_output" | grep '"case_context":"active"' >/dev/null
printf '%s\n' "$filesystem_summary" | grep '"status":"ok"' >/dev/null
printf '%s\n' "$shutdown_output" | grep '"status":"ok"' >/dev/null
printf '%s\n' "$filesystem_projection" | grep 'case_domains: 1' >/dev/null
printf '%s\n' "$filesystem_projection" | grep 'case_attachments: 1' >/dev/null
printf '%s\n' "$filesystem_projection" | grep 'case_bindings: 1' >/dev/null
printf '%s\n' "$filesystem_projection" | grep 'projection_results: 2' >/dev/null
printf '%s\n' "$filesystem_projection" | grep 'projection_rules: 1' >/dev/null
printf '%s\n' "$filesystem_projection" | grep 'authority_scopes: 3' >/dev/null
printf '%s\n' "$filesystem_projection" | grep 'model_interpretations: 0' >/dev/null
printf '%s\n' "$filesystem_projection" | grep 'operator: 1' >/dev/null
printf '%s\n' "$filesystem_projection" | grep 'model: 1' >/dev/null
printf '%s\n' "$filesystem_projection" | grep 'redacted_or_limited: 1' >/dev/null
printf '%s\n' "$filesystem_projection" | grep 'projection_freshness: fresh' >/dev/null
printf '%s\n' "$filesystem_projection" | grep 'freshness_policy: usable' >/dev/null
printf '%s\n' "$filesystem_projection" | grep 'source: hot_state' >/dev/null
printf '%s\n' "$security_bootstrap" | grep 'authentication_kind: local_posix_effective_credential' >/dev/null
printf '%s\n' "$security_bootstrap" | grep 'tenant_id: tenant:daemon-core-loop' >/dev/null
printf '%s\n' "$tenant_case" | grep 'case_created: true' >/dev/null
printf '%s\n' "$tenant_case" | grep 'tenant_id: tenant:daemon-core-loop' >/dev/null
printf '%s\n' "$filesystem_case_entry" | grep 'case_entry: accepted' >/dev/null
printf '%s\n' "$filesystem_case_entry" | grep 'subject_ref: subject:llm-provider' >/dev/null
printf '%s\n' "$filesystem_case_entry" | grep 'case_session: active' >/dev/null
printf '%s\n' "$filesystem_case_entry" | grep 'case_world: materialized' >/dev/null
printf '%s\n' "$filesystem_case_entry" | grep 'case_context: active' >/dev/null
printf '%s\n' "$filesystem_case_entry" | grep 'participant_view: model_context' >/dev/null
printf '%s\n' "$filesystem_case_entry" | grep 'raw_journal_access: not_provided' >/dev/null
printf '%s\n' "$filesystem_case_shell" | grep 'export YAI_CASE_REF=' >/dev/null
printf '%s\n' "$filesystem_case_shell" | grep 'export YAI_CASE_PROMPT_FLAG=' >/dev/null
printf '%s\n' "$filesystem_case_shell" | grep 'export PROMPT=' >/dev/null
printf '%s\n' "$filesystem_case_shell" | grep 'export RPROMPT="$YAI_RPROMPT_BASE"' >/dev/null
printf '%s\n' "$filesystem_provider_attach" | grep 'provider_attachment: accepted' >/dev/null
printf '%s\n' "$filesystem_provider_attach" | grep 'provider_attachment_status: attached' >/dev/null
printf '%s\n' "$filesystem_provider_attach" | grep 'case_session: active' >/dev/null
printf '%s\n' "$filesystem_provider_attach" | grep 'case_context: active' >/dev/null
printf '%s\n' "$filesystem_provider_attach" | grep 'authority_scope: model interpretation_only' >/dev/null
printf '%s\n' "$filesystem_provider_shell" | grep 'export YAI_PROVIDER_BASE_URL=' >/dev/null
printf '%s\n' "$filesystem_provider_shell" | grep 'export YAI_PROVIDER_MODEL=' >/dev/null
printf '%s\n' "$filesystem_transcript_on" | grep 'prompt_transcript_retention: enabled' >/dev/null
printf '%s\n' "$filesystem_transcript_on" | grep 'transcript_retention: full_redacted_case_local' >/dev/null
printf '%s\n' "$filesystem_transcript_status" | grep 'prompt_transcript_retention: enabled' >/dev/null
printf '%s\n' "$filesystem_transcript_status" | grep 'memory_candidate: derived_not_raw_chat' >/dev/null
printf '%s\n' "$filesystem_thread_status" | grep 'interaction_thread: thread:default' >/dev/null
printf '%s\n' "$filesystem_thread_status" | grep 'journal_role: replay_audit_not_chat_memory' >/dev/null
printf '%s\n' "$filesystem_thread_new" | grep 'interaction_thread: new active' >/dev/null
printf '%s\n' "$filesystem_thread_new" | grep 'semantic_context: new_thread_projection_rebuild_required' >/dev/null
printf '%s\n' "$filesystem_thread_new" | grep 'journal:audit retained' >/dev/null
printf '%s\n' "$filesystem_thread_use_default" | grep 'interaction_thread: restored previous' >/dev/null
printf '%s\n' "$filesystem_thread_use_default" | grep 'thread_id: thread:default' >/dev/null
printf '%s\n' "$filesystem_memory_propose" | grep 'memory_proposal: retired' >/dev/null
printf '%s\n' "$filesystem_memory_propose" | grep 'canonical_history_mutated: no' >/dev/null
printf '%s\n' "$filesystem_prompt_dry_run" | grep 'model_prompt: dry_run' >/dev/null
printf '%s\n' "$filesystem_prompt_dry_run" | grep 'case_session: active' >/dev/null
printf '%s\n' "$filesystem_prompt_dry_run" | grep 'case_context: active' >/dev/null
printf '%s\n' "$filesystem_prompt_dry_run" | grep 'interaction_thread: thread:default' >/dev/null
printf '%s\n' "$filesystem_prompt_dry_run" | grep 'projection_freshness: fresh' >/dev/null
printf '%s\n' "$filesystem_prompt_dry_run" | grep 'freshness_policy: usable' >/dev/null
printf '%s\n' "$filesystem_prompt_dry_run" | grep 'freshness_source: hot_state' >/dev/null
printf '%s\n' "$filesystem_prompt_dry_run" | grep 'context_source: typed_projection_plus_context_frame' >/dev/null
printf '%s\n' "$filesystem_prompt_dry_run" | grep 'transcript_retention: full_redacted_case_local' >/dev/null
printf '%s\n' "$filesystem_prompt_dry_run" | grep 'raw_journal_access: not_provided' >/dev/null
printf '%s\n' "$filesystem_prompt_dry_run" | grep 'decision_authority: not_provided' >/dev/null
printf '%s\n' "$filesystem_prompt_dry_run" | grep 'receipt_authority: not_provided' >/dev/null

wait "$daemon_pid"
trap - EXIT INT TERM

printf 'daemon:started\n'
printf 'ipc:status ok\n'
printf 'daemon-loop:minimum completed\n'
printf 'daemon-loop:filesystem completed\n'
printf 'journal:summary ok\n'
printf 'projection:summary ok\n'
printf 'security:tenant case bootstrapped\n'
printf 'case:enter ok\n'
printf 'case:attach-provider ok\n'
printf 'case:transcript retention ok\n'
printf 'case:interaction thread ok\n'
printf 'case:legacy memory proposal retired ok\n'
printf 'case:prompt dry-run ok\n'
printf 'case:context active ok\n'
printf 'daemon:shutdown ok\n'
