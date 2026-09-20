import type { CommandService } from "./commands";
import type { ContextPredicate, ContextKeyService } from "./context";
import { toDisposable, type Disposable } from "./lifecycle";

export interface Keybinding {
  id: string;
  command: string;
  key: string;
  when?: ContextPredicate;
  allowInTerminal?: boolean;
}

function chord(event: KeyboardEvent) {
  const parts: string[] = [];
  if (event.ctrlKey || event.metaKey) parts.push("Mod");
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");
  const key = event.key === " " ? "Space" : event.key.length === 1 ? event.key.toLowerCase() : event.key;
  parts.push(key);
  return parts.join("+");
}

export class KeybindingService implements Disposable {
  private readonly bindings = new Map<string, Keybinding>();
  private listener?: (event: KeyboardEvent) => void;

  constructor(
    private readonly commands: CommandService,
    private readonly context: ContextKeyService,
  ) {}

  registerKeybinding(binding: Keybinding): Disposable {
    if (this.bindings.has(binding.id)) throw new Error(`Keybinding already registered: ${binding.id}`);
    this.bindings.set(binding.id, binding);
    return toDisposable(() => this.bindings.delete(binding.id));
  }

  shortcutFor(command: string) {
    return [...this.bindings.values()].find((binding) => binding.command === command)?.key;
  }

  attach(target: Window = window): Disposable {
    if (this.listener) throw new Error("Keybinding service is already attached");
    this.listener = (event) => {
      const terminalFocused = Boolean((event.target as Element | null)?.closest?.(".xterm"));
      const binding = [...this.bindings.values()].find((candidate) =>
        candidate.key === chord(event) &&
        this.context.matches(candidate.when) &&
        (!terminalFocused || candidate.allowInTerminal),
      );
      if (!binding) return;
      event.preventDefault();
      void this.commands.executeCommand(binding.command);
    };
    target.addEventListener("keydown", this.listener);
    return toDisposable(() => {
      if (this.listener) target.removeEventListener("keydown", this.listener);
      this.listener = undefined;
    });
  }

  dispose() {
    if (this.listener) window.removeEventListener("keydown", this.listener);
    this.listener = undefined;
    this.bindings.clear();
  }
}
