import type { IconName } from "../../components/Icon";
import { toDisposable, type Disposable } from "../../platform/lifecycle";

export type SurfaceRole = "content" | "projection" | "system" | "interactive-case";

// These capabilities describe Workbench interaction only. They never grant
// Case authority or imply that a YAI application mutation exists.
export type SurfaceCapability =
  | "previewable"
  | "pinnable"
  | "singleton"
  | "editable"
  | "dirty-aware"
  | "searchable"
  | "zoomable"
  | "navigable"
  | "selectable";

export type SurfacePosture = "ready" | "read-only" | "unavailable" | "error";

// SurfaceInput is transient Workbench interaction state. It identifies a
// representation; it is never a canonical Case object or authority grant.
export interface SurfaceInput {
  id: string;
  identity: string;
  surfaceType: string;
  title: string;
  icon: IconName;
  pinned: boolean;
  dirty?: boolean;
  posture?: SurfacePosture;
  objectRef?: string;
  viewId?: string;
  metadata?: Readonly<Record<string, string>>;
}

export interface SurfaceGroupSnapshot {
  inputs: readonly SurfaceInput[];
  activeId?: string;
}

export class SurfaceGroupService implements Disposable {
  private inputs: SurfaceInput[] = [];
  private activeId?: string;
  private readonly listeners = new Set<() => void>();

  snapshot(): SurfaceGroupSnapshot {
    return { inputs: this.inputs, activeId: this.activeId };
  }

  open(input: SurfaceInput) {
    // A preview slot is placement, never a second copy of an open material.
    const existing = this.inputs.find((candidate) => candidate.identity === input.identity);
    if (existing) {
      if (input.pinned) this.pin(existing.id);
      this.activate(this.inputs.find((candidate) => candidate.identity === input.identity)!.id);
      return;
    }
    if (!input.pinned) this.inputs = this.inputs.filter((candidate) => candidate.pinned || candidate.dirty);
    this.inputs = [...this.inputs, { ...input, id: input.identity }];
    this.activeId = input.identity;
    this.emit();
  }

  activate(id: string) {
    if (this.activeId === id || !this.inputs.some((input) => input.id === id)) return;
    this.activeId = id;
    this.emit();
  }

  pin(id: string) {
    const current = this.inputs.find((input) => input.id === id);
    if (!current || current.pinned) return;
    const pinnedId = current.identity;
    this.inputs = this.inputs.map((input) => input.id === id
      ? { ...input, id: pinnedId, pinned: true }
      : input);
    if (this.activeId === id) this.activeId = pinnedId;
    this.emit();
  }

  update(id: string, patch: Partial<Omit<SurfaceInput, "id" | "identity">>) {
    const index = this.inputs.findIndex((input) => input.id === id);
    if (index < 0 || Object.entries(patch).every(([key, value]) => Object.is(this.inputs[index][key as keyof SurfaceInput], value))) return;
    // Editing claims the preview so another open cannot hide unsaved work.
    this.inputs = this.inputs.map((input) => input.id === id ? { ...input, ...patch, pinned: input.pinned || Boolean(patch.dirty) } : input);
    this.emit();
  }

  replace(id: string, input: SurfaceInput) {
    const index = this.inputs.findIndex((candidate) => candidate.id === id);
    if (index < 0) { this.open(input); return; }
    this.inputs = this.inputs.map((candidate) => candidate.id === id ? { ...input, id } : candidate);
    this.activeId = id;
    this.emit();
  }

  close(id: string) {
    const index = this.inputs.findIndex((input) => input.id === id);
    if (index < 0) return;
    this.inputs = this.inputs.filter((input) => input.id !== id);
    if (this.activeId === id) {
      this.activeId = this.inputs[Math.min(index, this.inputs.length - 1)]?.id;
    }
    this.emit();
  }

  move(delta: number) {
    const index = this.inputs.findIndex((input) => input.id === this.activeId);
    if (index < 0 || !this.inputs.length) return;
    this.activeId = this.inputs[(index + delta + this.inputs.length) % this.inputs.length].id;
    this.emit();
  }

  subscribe(listener: () => void): Disposable {
    this.listeners.add(listener);
    return toDisposable(() => this.listeners.delete(listener));
  }

  dispose() {
    this.inputs = [];
    this.listeners.clear();
  }

  private emit() { this.listeners.forEach((listener) => listener()); }
}
