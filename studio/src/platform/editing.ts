import type { ContextKeyService } from "./context";
import { toDisposable, type Disposable } from "./lifecycle";

export type EditCommand = "undo" | "redo" | "cut" | "copy" | "paste" | "selectAll";
export interface EditSelection {
  text: string;
  /** False after a different document, selection, revision or terminal becomes current. */
  current(): boolean;
  replace(text: string): void;
}
export interface EditTarget {
  element: HTMLElement;
  enabled(command: EditCommand): boolean;
  selection(): EditSelection;
  execute(command: "undo" | "redo" | "selectAll"): void;
  focus(): void;
}
const commands: EditCommand[] = ["undo", "redo", "cut", "copy", "paste", "selectAll"];

/** Focused editor, text field and PTY operations; no Case mutation or global edit shortcuts. */
export class EditingService implements Disposable {
  private targets = new Set<EditTarget>();
  private focused?: HTMLElement;
  private listeners = new Set<() => void>();
  private notice?: string;
  private detach?: () => void;
  constructor(private context: ContextKeyService) {}

  snapshot = () => this.notice;
  subscribe = (listener: () => void) => { this.listeners.add(listener); return toDisposable(() => this.listeners.delete(listener)); };
  dismiss() { this.report(undefined); }
  private report(message?: string) { this.notice = message; this.listeners.forEach(listener => listener()); }

  register(target: EditTarget) {
    this.targets.add(target); this.changed();
    return toDisposable(() => { this.targets.delete(target); this.changed(); });
  }
  attach() {
    this.detach?.();
    const focus = () => {
      const element = document.activeElement;
      // Menus and the command palette operate on the control that opened them.
      if (element instanceof HTMLElement && !element.closest('.desktop-menu, .workbench-search')) this.focused = element;
      this.changed();
    };
    focus();
    document.addEventListener("focusin", focus);
    document.addEventListener("selectionchange", this.changed);
    document.addEventListener("input", this.changed);
    const detach = () => {
      document.removeEventListener("focusin", focus);
      document.removeEventListener("selectionchange", this.changed);
      document.removeEventListener("input", this.changed);
    };
    this.detach = detach;
    return toDisposable(detach);
  }
  changed = () => {
    const target = this.target();
    for (const command of commands) this.context.update(`edit.${command}`, target?.enabled(command) ?? (command === "copy" && Boolean(globalThis.getSelection?.()?.toString())));
  };
  private target(): EditTarget | undefined {
    const element = this.focused;
    if (!element?.isConnected) return undefined;
    const target = [...this.targets].find(value => value.element.contains(element));
    if (target) return target;
    if (element.closest(".xterm")) return undefined;
    if (!(element instanceof HTMLTextAreaElement || element instanceof HTMLInputElement) || element.selectionStart === null || element.disabled) return undefined;
    return {
      element, focus: () => element.focus(),
      enabled: command => command === "selectAll" || (command === "copy" ? element.selectionStart !== element.selectionEnd : !element.readOnly && (command === "paste" || command === "cut" && element.selectionStart !== element.selectionEnd || (command === "undo" || command === "redo") && document.queryCommandEnabled(command))),
      execute: command => { if (command === "selectAll") element.select(); else document.execCommand(command); },
      selection: () => {
        const value = element.value, start = element.selectionStart!, end = element.selectionEnd!;
        return { text: value.slice(start, end), current: () => element.value === value && element.selectionStart === start && element.selectionEnd === end,
          replace: text => {
            // Native insertion keeps browser undo and React's normal input event path.
            if (!document.execCommand("insertText", false, text)) throw new Error("native_insert_unavailable");
          } };
      },
    };
  }
  async execute(command: EditCommand) {
    this.dismiss();
    const target = this.target(), focused = this.focused;
    if (!target) {
      if (command === "copy" && globalThis.getSelection?.()?.toString()) {
        try { if (!document.execCommand("copy")) await navigator.clipboard.writeText(getSelection()!.toString()); }
        catch { this.report("Clipboard access was refused. Use the system Copy shortcut on the selected text."); }
      }
      return;
    }
    if (!target.enabled(command)) return;
    target.focus();
    if (command === "undo" || command === "redo" || command === "selectAll") { target.execute(command); this.changed(); return; }
    const selected = target.selection();
    const current = () => target.element.isConnected && this.focused === focused && selected.current();
    try {
      if (command === "paste") {
        const text = await navigator.clipboard.readText();
        if (!current()) { this.report("Paste cancelled because the active content or selection changed. Paste again at the intended location."); return; }
        selected.replace(text);
      } else {
        // Synchronous browser copy/cut preserves the user gesture on native WebViews.
        if (document.execCommand(command)) return;
        await navigator.clipboard.writeText(selected.text);
        if (command === "cut") {
          if (!current()) { this.report("Text was copied, but not removed because the content or selection changed."); return; }
          selected.replace("");
        }
      }
    } catch {
      this.report(`Clipboard ${command} is unavailable or permission was refused. Use the system keyboard shortcut in the focused control.`);
    } finally { this.changed(); }
  }
  dispose() { this.detach?.(); this.targets.clear(); this.listeners.clear(); this.focused = undefined; }
}
