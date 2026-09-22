import { toDisposable, type Disposable } from "./lifecycle";

export const preferenceNumberBounds: Record<string, { min: number; max: number; step: number }> = {
  "workbench.sidebar.width": { min: 170, max: 320, step: 1 },
  "workbench.auxiliary.width": { min: 290, max: 470, step: 1 },
  "workbench.panel.heightRatio": { min: .2, max: .72, step: .01 },
  "terminal.scrollback": { min: 0, max: 50000, step: 100 },
};

export class ConfigurationService implements Disposable {
  private readonly values = new Map<string, unknown>();
  private readonly listeners = new Set<(key: string) => void>();

  private readonly storageKey = "yai.studio.preferences.v1";

  constructor(private readonly defaults: Record<string, unknown>) {
    Object.entries(defaults).forEach(([key, value]) => this.values.set(key, value));
    try {
      const stored = window.localStorage.getItem(this.storageKey);
      const parsed = stored ? JSON.parse(stored) as { version?: number; values?: Record<string, unknown> } : undefined;
      if (parsed?.version === 1 && parsed.values) Object.entries(parsed.values).forEach(([key, value]) => { if (this.valid(key, value)) this.values.set(key, value); });
    } catch { /* storage can be unavailable in hardened webviews */ }
  }

  private valid(key: string, value: unknown) {
    if (!Object.hasOwn(this.defaults, key) || typeof value !== typeof this.defaults[key]) return false;
    if (typeof value !== "number") return true;
    const bounds = preferenceNumberBounds[key];
    return Number.isFinite(value) && (!bounds || (value >= bounds.min && value <= bounds.max && (bounds.step < 1 || Number.isInteger(value))));
  }

  get<T>(key: string): T | undefined {
    return this.values.get(key) as T | undefined;
  }

  update<T>(key: string, value: T) {
    if (!this.valid(key, value) || Object.is(this.values.get(key), value)) return;
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
