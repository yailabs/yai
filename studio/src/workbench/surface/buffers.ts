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
    if (current?.source.key === source.key || current?.incoming?.source.key === source.key) return current;
    if (current && (current.source.caseRef !== source.caseRef || current.source.objectRef !== source.objectRef || current.source.sourceRef !== source.sourceRef || current.source.path !== source.path)) {
      throw new Error("A Surface buffer cannot change material identity");
    }
    const sameRevision = current && current.source.revisionRef === source.revisionRef && current.source.digest === source.digest;
    if (sameRevision && current.baseline === value && !current.stale) {
      this.buffers.set(identity, { ...current, source });
      this.emit(identity);
      return this.snapshot(identity);
    }
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
    if (current.value === value) return;
    this.buffers.set(identity, { ...current, value, dirty: value !== current.baseline });
    this.emit(identity);
  }

  revert(identity: string) {
    const current = this.buffers.get(identity);
    if (!current) return;
    const baseline = current.incoming ?? { value: current.baseline, source: current.source };
    this.buffers.set(identity, { baseline: baseline.value, value: baseline.value, dirty: false, source: baseline.source, stale: false });
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

  get dirtyCount() { return [...this.buffers.values()].filter((buffer) => buffer.dirty).length; }

  dispose() { this.buffers.clear(); this.listeners.clear(); }
  private emit(identity: string) { this.listeners.get(identity)?.forEach((listener) => listener()); }
}
