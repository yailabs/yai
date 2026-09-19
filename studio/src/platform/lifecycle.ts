export interface Disposable {
  dispose(): void;
}

export function toDisposable(dispose: () => void): Disposable {
  let active = true;
  return {
    dispose() {
      if (!active) return;
      active = false;
      dispose();
    },
  };
}

export class DisposableStore implements Disposable {
  private readonly values = new Set<Disposable>();
  private disposed = false;

  add<T extends Disposable>(value: T): T {
    if (this.disposed) value.dispose();
    else this.values.add(value);
    return value;
  }

  clear() {
    for (const value of this.values) value.dispose();
    this.values.clear();
  }

  dispose() {
    if (this.disposed) return;
    this.disposed = true;
    this.clear();
  }
}
