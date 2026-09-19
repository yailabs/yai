export type StudioCommandId =
  | "studio.file.openCase"
  | "studio.file.settings"
  | "studio.window.close"
  | "studio.edit.cut"
  | "studio.edit.copy"
  | "studio.edit.paste"
  | "studio.view.toggleExplorer"
  | "studio.view.toggleContext"
  | "studio.view.toggleBottomPanel"
  | "studio.view.resetLayout"
  | "studio.go.back"
  | "studio.go.forward"
  | "studio.go.previousTab"
  | "studio.go.nextTab"
  | `studio.case.${string}`
  | "studio.case.refresh"
  | "studio.terminal.new"
  | "studio.terminal.kill"
  | "studio.terminal.clear"
  | "studio.terminal.focus"
  | "studio.help.about";

export interface StudioCommand {
  id: StudioCommandId;
  label: string;
  shortcut?: string;
  enabled?: boolean;
  checked?: boolean;
  run: () => void;
}

export interface StudioMenu {
  label: string;
  items: Array<StudioCommandId | "separator">;
}

export function commandMap(commands: StudioCommand[]) {
  return new Map(commands.map((command) => [command.id, command]));
}

export function runCommand(commands: StudioCommand[], id: StudioCommandId) {
  const command = commands.find((candidate) => candidate.id === id);
  if (command?.enabled !== false) command?.run();
}
