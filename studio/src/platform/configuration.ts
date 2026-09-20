import { toDisposable, type Disposable } from "./lifecycle";

export class ConfigurationService implements Disposable {
  private readonly values = new Map<string, unknown>();
  private readonly listeners = new Set<(key: string) => void>();

  private readonly storageKey = "yai.studio.preferences.v1";

  constructor(defaults: Record<string, unknown>) {
    Object.entries(defaults).forEach(([key, value]) => this.values.set(key, value));
    try {
      const stored = window.localStorage.getItem(this.storageKey);
      const parsed = stored ? JSON.parse(stored) as { version?: number; values?: Record<string, unknown> } : undefined;
      if (parsed?.version === 1 && parsed.values) Object.entries(parsed.values).forEach(([key, value]) => this.values.set(key, value));
    } catch { /* storage can be unavailable in hardened webviews */ }
  }

  get<T>(key: string): T | undefined {
    return this.values.get(key) as T | undefined;
  }

  update<T>(key: string, value: T) {
    this.values.set(key, value);
    try {
      window.localStorage.setItem(this.storageKey, JSON.stringify({ version: 1, values: Object.fromEntries(this.values) }));
    } catch { /* keep the preference for the current renderer lifecycle */ }
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
