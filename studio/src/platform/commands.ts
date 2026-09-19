import type { ContextPredicate, ContextKeyService } from "./context";
import { toDisposable, type Disposable } from "./lifecycle";

export interface CommandDefinition<Args extends unknown[] = unknown[]> {
  id: string;
  title: string;
  handler: (...args: Args) => void | Promise<void>;
  when?: ContextPredicate;
}

export class CommandService implements Disposable {
  private readonly commands = new Map<string, CommandDefinition>();

  constructor(private readonly context: ContextKeyService) {}

  registerCommand<Args extends unknown[]>(definition: CommandDefinition<Args>): Disposable {
    if (this.commands.has(definition.id)) {
      throw new Error(`Command already registered: ${definition.id}`);
    }
    this.commands.set(definition.id, definition as CommandDefinition);
    return toDisposable(() => this.commands.delete(definition.id));
  }

  has(id: string) {
    return this.commands.has(id);
  }

  isEnabled(id: string) {
    const command = this.commands.get(id);
    return Boolean(command && this.context.matches(command.when));
  }

  title(id: string) {
    return this.commands.get(id)?.title;
  }

  async executeCommand(id: string, ...args: unknown[]) {
    const command = this.commands.get(id);
    if (!command) throw new Error(`Unknown command: ${id}`);
    if (!this.context.matches(command.when)) return false;
    await command.handler(...args);
    return true;
  }

  dispose() {
    this.commands.clear();
  }
}
