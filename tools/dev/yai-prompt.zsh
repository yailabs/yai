# Session-only workspace prompt hook for zsh.
# Usage:
#   source /path/to/yai/tools/dev/yai-prompt.zsh
#   yai_prompt_enable
#   yai_prompt_disable

if [[ -n "${YAI_PROMPT_LOADED:-}" ]]; then
  return 0
fi
typeset -g YAI_PROMPT_LOADED=1

yai_prompt_token_cmd() {
  local canonical="$HOME/Developer/YAI/yai/tools/bin/yai-ws-token"
  if [[ -x "$canonical" ]]; then
    printf '%s\n' "$canonical"
    return 0
  fi

  local script_dir
  script_dir="$(cd -- "$(dirname -- "${(%):-%N}")" && pwd)"
  local fallback="${script_dir}/../bin/yai-ws-token"
  if [[ -x "$fallback" ]]; then
    printf '%s\n' "$fallback"
    return 0
  fi
  if command -v yai-ws-token >/dev/null 2>&1; then
    command -v yai-ws-token
    return 0
  fi
  return 1
}

yai_prompt_segment() {
  local cmd
  cmd="$(yai_prompt_token_cmd)" || return 0
  YAI_WS_BIND_SCOPE=tty "$cmd" 2>/dev/null || true
}

yai_prompt_precmd() {
  local tok base
  tok="$(yai_prompt_segment)"
  base="${YAI_PROMPT_RPROMPT_BASE:-$RPROMPT}"
  if [[ -n "$tok" ]]; then
    RPROMPT="${tok}${base:+ ${base}}"
  else
    RPROMPT="${base}"
  fi
}

yai_prompt_enable() {
  setopt prompt_subst
  if [[ -z "${YAI_PROMPT_RPROMPT_BASE+x}" ]]; then
    typeset -g YAI_PROMPT_RPROMPT_BASE="$RPROMPT"
  fi
  precmd_functions=(${precmd_functions:#yai_prompt_precmd})
  precmd_functions+=(yai_prompt_precmd)
  yai_prompt_precmd
}

yai_prompt_disable() {
  precmd_functions=(${precmd_functions:#yai_prompt_precmd})
  if [[ -n "${YAI_PROMPT_RPROMPT_BASE+x}" ]]; then
    RPROMPT="$YAI_PROMPT_RPROMPT_BASE"
    unset YAI_PROMPT_RPROMPT_BASE
  fi
}
