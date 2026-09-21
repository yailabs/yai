export type TerminalCommand = "new" | "kill" | "clear" | "focus";
export const terminalEventName = "yai:terminal-command";

export function dispatchTerminalCommand(command: TerminalCommand) {
  window.dispatchEvent(new CustomEvent(terminalEventName, { detail: command }));
}
