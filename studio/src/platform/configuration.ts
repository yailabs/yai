import { toDisposable, type Disposable } from "./lifecycle";

export class ConfigurationService implements Disposable {
  private readonly values = new Map<string, unknown>();
  private readonly listeners = new Set<(key: string) => void>();

  constructor(defaults: Record<string, unknown>) {
    Object.entries(defaults).forEach(([key, value]) => this.values.set(key, value));
  }

  get<T>(key: string): T | undefined {
    return this.values.get(key) as T | undefined;
  }

  update<T>(key: string, value: T) {
    this.values.set(key, value);
    this.listeners.forEach((listener) => listener(key));
  }

  subscribe(listener: (key: string) => void): Disposable {
    this.listeners.add(listener);
    return toDisposable(() => this.listeners.delete(listener));
  }

  dispose() {
    this.listeners.clear();
    this.values.clear();
  }
}
