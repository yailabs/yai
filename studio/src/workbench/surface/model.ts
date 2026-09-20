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
    if (!input.pinned) {
      this.inputs = this.inputs.filter((candidate) => candidate.pinned || candidate.id === input.id);
    }
    const index = this.inputs.findIndex((candidate) => candidate.id === input.id);
    if (index >= 0) this.inputs[index] = input;
    else this.inputs = [...this.inputs, input];
    this.activeId = input.id;
    this.emit();
  }

  activate(id: string) {
    if (!this.inputs.some((input) => input.id === id)) return;
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
