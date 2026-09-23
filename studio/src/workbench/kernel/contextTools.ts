import { toDisposable } from "../../platform/lifecycle";

export interface ToolBounds { x: number; y: number; width: number; height: number }
export interface ContextToolState { open: boolean; floating: boolean; bounds?: ToolBounds; pinnedRef?: string }
const initial: ContextToolState = { open: false, floating: false };

/** Window-local composition inside an existing Case/Participant session. */
export class ContextToolLayout {
  private value: Readonly<Record<string, ContextToolState>> = {};
  private readonly listeners = new Set<() => void>();
  readonly snapshot = () => this.value;
  readonly subscribe = (listener: () => void) => { this.listeners.add(listener); return toDisposable(() => this.listeners.delete(listener)); };
  get(id: string) { return this.value[id] ?? initial; }
  update(id: string, patch: Partial<ContextToolState>) {
    const next = { ...this.get(id), ...patch };
    if (JSON.stringify(this.value[id]) === JSON.stringify(next)) return;
    this.value = { ...this.value, [id]: next }; this.emit();
  }
  private emit() { for (const listener of this.listeners) listener(); }
  reset() { this.value = {}; this.emit(); }
  dispose() { this.listeners.clear(); }
}

/** Viewport changes constrain presentation, never rewrite Case or draft state. */
export function fitTool(bounds: ToolBounds, viewport: {width: number; height: number}): ToolBounds {
  const width = Math.min(Math.max(280, bounds.width), Math.max(280, viewport.width - 24));
  const height = Math.min(Math.max(220, bounds.height), Math.max(220, viewport.height - 104));
  return { width, height, x: Math.max(12, Math.min(bounds.x, viewport.width - width - 12)),
    y: Math.max(72, Math.min(bounds.y, viewport.height - height - 32)) };
}
