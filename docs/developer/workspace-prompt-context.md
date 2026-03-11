# Workspace Prompt Context

## Goal

Provide a compact, stable prompt-facing context payload derived from current workspace binding.

This is not full inspect output. It is a lightweight session summary.

## Shell Token (Git-style)

For terminal prompts, use the helper:

- `tools/bin/yai-ws-token`

It emits a compact token based on active session binding only:

- `◉ <alias>` (default)
- no `ws:` prefix
- empty output when no active/valid binding
- interactive shells resolve per-terminal binding first (`~/.yai/session/by-tty/<tty>.json`)
- non-interactive/scripted calls fall back to `~/.yai/session/active_workspace.json`

This keeps shell UX aligned with Git-like context display while preserving the model:

- token represents binding, not cwd path.

## Contract

Runtime command:

- `command_id: yai.workspace.prompt_context`

Payload result type:

- `yai.workspace.prompt_context.v1`

Primary fields:

- `binding_status` (`active|no_active|stale|invalid`)
- `workspace_id` (when active)
- `workspace_alias` (when active)
- `state` (workspace lifecycle summary)
- `declared.family` (if available)
- `declared.specialization` (if available)
- `reason` (for stale/invalid)

## Design constraints

- small payload
- stable keys
- fast derivation from session + workspace metadata
- no heavy resolution trace data

## Activate/current/clear behavior

- `activate`: writes session binding to selected workspace id.
- `current`: resolves active binding and returns active/no_active/stale/invalid.
- `clear`: removes binding and returns `no_active`.

## Edge cases

- missing binding file -> `no_active`
- malformed binding id -> `invalid`
- binding to missing workspace manifest -> `stale`
- env override `YAI_ACTIVE_WORKSPACE` takes precedence over binding file

## zsh / bash snippets

zsh:

```sh
# Session-only (recommended, non-invasive)
source /path/to/yai/tools/dev/yai-prompt.zsh
yai_prompt_enable

# Disable in current shell:
yai_prompt_disable
```

Runtime policy:

- default runtime behavior does **not** modify user shell files (`~/.zshrc`, prompt themes, etc.).
- managed shell bootstrap is available only via explicit opt-in:
  `YAI_SHELL_INTEGRATION_MODE=managed`.

bash:

```sh
yai_ws_token() {
  local t
  t="$("/Users/francescomaiomascio/Developer/YAI/yai/tools/bin/yai-ws-token")"
  [[ -n "$t" ]] && printf " %s" "$t"
}

PS1='\w $(git branch --show-current 2>/dev/null)$(yai_ws_token) \$ '
```

## Workspace root path

To inspect the active workspace root path:

```sh
cd /Users/francescomaiomascio/Developer/YAI/cli
./dist/bin/yai ws inspect
```

Look at the `Root` / `workspace_root` field.

## Next step readiness

This prompt contract is the base for WS-3 inspect/status flows and shell prompt adapters.
