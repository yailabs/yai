import { toDisposable, type Disposable } from "./lifecycle";

export type ContextValue = string | number | boolean | undefined;
export type ContextSnapshot = ReadonlyMap<string, ContextValue>;
export type ContextPredicate = (context: ContextSnapshot) => boolean;

export const when = {
  equals: (key: string, expected: ContextValue): ContextPredicate =>
    (context) => context.get(key) === expected,
  truthy: (key: string): ContextPredicate =>
    (context) => Boolean(context.get(key)),
  all: (...predicates: ContextPredicate[]): ContextPredicate =>
    (context) => predicates.every((predicate) => predicate(context)),
  not: (predicate: ContextPredicate): ContextPredicate =>
    (context) => !predicate(context),
};

export class ContextKeyService implements Disposable {
  private readonly values = new Map<string, ContextValue>();
  private readonly listeners = new Set<() => void>();

  snapshot(): ContextSnapshot {
    return new Map(this.values);
  }

  get(key: string): ContextValue {
    return this.values.get(key);
  }

  set(key: string, value: ContextValue): Disposable {
    const previous = this.values.get(key);
    this.values.set(key, value);
    this.emit();
    return toDisposable(() => {
      if (previous === undefined) this.values.delete(key);
      else this.values.set(key, previous);
      this.emit();
    });
  }

  update(key: string, value: ContextValue) {
    if (this.values.get(key) === value) return;
    this.values.set(key, value);
    this.emit();
  }

  matches(predicate?: ContextPredicate): boolean {
    return predicate ? predicate(this.snapshot()) : true;
  }

  subscribe(listener: () => void): Disposable {
    this.listeners.add(listener);
    return toDisposable(() => this.listeners.delete(listener));
  }

  reset() {
    this.values.clear();
    this.emit();
  }

  dispose() {
    this.values.clear();
    this.listeners.clear();
  }

  private emit() {
    for (const listener of this.listeners) listener();
  }
}
