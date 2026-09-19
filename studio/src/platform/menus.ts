import type { ContextPredicate, ContextKeyService } from "./context";
import { toDisposable, type Disposable } from "./lifecycle";

export type MenuLocation =
  | "YAI" | "File" | "Edit" | "View" | "Go" | "Case" | "Terminal" | "Help"
  | "tree/context" | "editor/context" | "graph/node/context"
  | "graph/edge/context" | "inspector/context" | "terminal/context";

export interface MenuContribution {
  id: string;
  location: MenuLocation;
  command: string;
  group?: string;
  order?: number;
  when?: ContextPredicate;
  checkedWhen?: ContextPredicate;
}

export interface ResolvedMenuItem extends MenuContribution {
  enabled: boolean;
  checked: boolean;
}

export class MenuService implements Disposable {
  private readonly items = new Map<string, MenuContribution>();

  constructor(private readonly context: ContextKeyService) {}

  registerMenuItem(item: MenuContribution): Disposable {
    if (this.items.has(item.id)) throw new Error(`Menu item already registered: ${item.id}`);
    this.items.set(item.id, item);
    return toDisposable(() => this.items.delete(item.id));
  }

  getMenu(location: MenuLocation): ResolvedMenuItem[] {
    return [...this.items.values()]
      .filter((item) => item.location === location && this.context.matches(item.when))
      .sort((left, right) => (left.group ?? "").localeCompare(right.group ?? "") || (left.order ?? 0) - (right.order ?? 0))
      .map((item) => ({ ...item, enabled: true, checked: this.context.matches(item.checkedWhen) }));
  }

  dispose() {
    this.items.clear();
  }
}
