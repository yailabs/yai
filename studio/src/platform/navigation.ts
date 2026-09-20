export interface NavigationLocation {
  caseRef: string;
  view: string;
  surfaceId: string;
  selection?: string;
  auxiliary?: string;
}

export class NavigationService {
  private entries: NavigationLocation[] = [];
  private index = -1;
  private readonly listeners = new Set<() => void>();

  current() { return this.entries[this.index]; }
  canBack() { return this.index > 0; }
  canForward() { return this.index >= 0 && this.index < this.entries.length - 1; }

  push(location: NavigationLocation) {
    const current = this.current();
    if (current && JSON.stringify(current) === JSON.stringify(location)) return;
    this.entries = [...this.entries.slice(0, this.index + 1), location];
    this.index = this.entries.length - 1;
    this.emit();
  }

  back() { return this.move(-1); }
  forward() { return this.move(1); }

  reset(location?: NavigationLocation) {
    this.entries = location ? [location] : [];
    this.index = location ? 0 : -1;
    this.emit();
  }

  subscribe(listener: () => void) {
    this.listeners.add(listener);
    return { dispose: () => this.listeners.delete(listener) };
  }

  dispose() {
    this.entries = [];
    this.listeners.clear();
  }

  private move(delta: number) {
    const next = this.index + delta;
    if (next < 0 || next >= this.entries.length) return undefined;
    this.index = next;
    this.emit();
    return this.current();
  }

  private emit() { this.listeners.forEach((listener) => listener()); }
}
