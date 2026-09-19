import { toDisposable, type Disposable } from "../../platform/lifecycle";

export interface EditorInput {
  id: string;
  type: "perspective" | "material" | "settings";
  title: string;
  pinned: boolean;
  resourceRef?: string;
  perspective?: string;
}

export interface EditorGroupSnapshot {
  inputs: readonly EditorInput[];
  activeId?: string;
}

export class EditorGroupService implements Disposable {
  private inputs: EditorInput[] = [];
  private activeId?: string;
  private readonly listeners = new Set<() => void>();

  snapshot(): EditorGroupSnapshot {
    return { inputs: this.inputs, activeId: this.activeId };
  }

  open(input: EditorInput) {
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
    const pinnedId = current.type === "material" && current.resourceRef
      ? `material:${current.resourceRef}`
      : current.id;
    this.inputs = this.inputs.map((input) => input.id === id ? { ...input, id: pinnedId, pinned: true } : input);
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
