import { toDisposable, type Disposable } from "../../platform/lifecycle";

export interface SurfaceBufferSnapshot {
  baseline: string;
  value: string;
  dirty: boolean;
  source: SurfaceBufferSource;
  stale: boolean;
  incoming?: { value: string; source: SurfaceBufferSource };
}

export interface SurfaceBufferSource {
  key: string;
  caseRef: string;
  objectRef: string;
  sourceRef: string;
  revisionRef: string;
  path: string;
  digest: string;
  generation: number;
}

export class SurfaceBufferService implements Disposable {
  private readonly buffers = new Map<string, SurfaceBufferSnapshot>();
  private readonly listeners = new Map<string, Set<() => void>>();

  initialize(identity: string, value: string, source: SurfaceBufferSource) {
    const current = this.buffers.get(identity);
    if (!current) {
      this.buffers.set(identity, { baseline: value, value, dirty: false, source, stale: false });
    } else if (current.source.key !== source.key) {
      this.buffers.set(identity, current.dirty
        ? { ...current, stale: true, incoming: { value, source } }
        : { baseline: value, value, dirty: false, source, stale: false });
    }
    this.emit(identity);
    return this.snapshot(identity);
  }

  snapshot(identity: string) {
    return this.buffers.get(identity);
  }

  update(identity: string, value: string) {
    const current = this.buffers.get(identity);
    if (!current) throw new Error(`surface buffer ${identity} was updated before exact material initialization`);
    this.buffers.set(identity, { ...current, value, dirty: value !== current.baseline });
    this.emit(identity);
  }

  revert(identity: string) {
    const current = this.buffers.get(identity);
    if (!current) return;
    this.buffers.set(identity, { baseline: current.baseline, value: current.baseline, dirty: false, source: current.source, stale: false });
    this.emit(identity);
  }

  reload(identity: string) {
    const current = this.buffers.get(identity);
    if (!current?.incoming) return;
    this.buffers.set(identity, { baseline: current.incoming.value, value: current.incoming.value, dirty: false, source: current.incoming.source, stale: false });
    this.emit(identity);
  }

  discard(identity: string) {
    this.buffers.delete(identity);
    this.emit(identity);
  }

  subscribe(identity: string, listener: () => void): Disposable {
    const listeners = this.listeners.get(identity) ?? new Set();
    listeners.add(listener);
    this.listeners.set(identity, listeners);
    return toDisposable(() => listeners.delete(listener));
  }

  dispose() { this.buffers.clear(); this.listeners.clear(); }
  private emit(identity: string) { this.listeners.get(identity)?.forEach((listener) => listener()); }
}
