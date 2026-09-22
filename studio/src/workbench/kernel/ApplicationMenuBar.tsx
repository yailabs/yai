import { useEffect, useRef, useState } from "react";
import type { PlatformServices } from "../../platform/services";
import type { MenuLocation } from "../../platform/menus";

const locations: MenuLocation[] = ["YAI", "File", "Edit", "View", "Go", "Case", "Terminal", "Help"];

export function ApplicationMenuBar({ platform }: { platform: PlatformServices }) {
  const [open, setOpen] = useState<MenuLocation>();
  const [, render] = useState(0);
  const root = useRef<HTMLElement>(null);
  const returnFocus = useRef<HTMLElement | null>(null);
  const selectedText = useRef<Range | undefined>(undefined);
  const rememberFocus = (element: EventTarget | null) => {
    if (!(element instanceof HTMLElement) || root.current?.contains(element)) return;
    returnFocus.current = element;
    const selection = window.getSelection();
    selectedText.current = selection?.rangeCount ? selection.getRangeAt(0).cloneRange() : undefined;
  };
  const restoreFocus = () => {
    if (!returnFocus.current?.isConnected) return;
    returnFocus.current.focus({ preventScroll: true });
    if (selectedText.current?.commonAncestorContainer.isConnected && !returnFocus.current.closest(".cm-editor, input, textarea")) {
      const selection = window.getSelection(); selection?.removeAllRanges(); selection?.addRange(selectedText.current);
    }
  };
  useEffect(() => platform.context.subscribe(() => render((value) => value + 1)).dispose, [platform]);
  useEffect(() => {
    const dismiss = (event: PointerEvent) => { if (!root.current?.contains(event.target as Node)) setOpen(undefined); };
    const keyboard = (event: KeyboardEvent) => { if (event.key === "Escape") setOpen(undefined); };
    window.addEventListener("pointerdown", dismiss); window.addEventListener("keydown", keyboard);
    return () => { window.removeEventListener("pointerdown", dismiss); window.removeEventListener("keydown", keyboard); };
  }, []);
  return <nav ref={root} className="desktop-menu" aria-label="Application menu" onPointerDownCapture={() => rememberFocus(document.activeElement)} onFocusCapture={event => rememberFocus(event.relatedTarget)} onKeyDown={(event) => {
      const target = event.target as HTMLElement;
      const triggers = [...root.current!.querySelectorAll<HTMLButtonElement>('[aria-haspopup="menu"]')];
      if (event.key === "ArrowLeft" || event.key === "ArrowRight") {
        event.preventDefault();
        const index = triggers.findIndex((button) => button.parentElement?.contains(target));
        const next = triggers[(index + (event.key === "ArrowRight" ? 1 : -1) + triggers.length) % triggers.length];
        next?.focus(); if (open) setOpen(next?.textContent as MenuLocation);
      }
      if (event.key === "ArrowDown" || event.key === "ArrowUp") {
        event.preventDefault();
        if (target.getAttribute("aria-haspopup") === "menu") {
          setOpen(target.textContent as MenuLocation);
          requestAnimationFrame(() => target.parentElement?.querySelector<HTMLButtonElement>('[role="menuitem"]:not(:disabled)')?.focus());
        } else {
          const buttons = [...target.closest('[role="menu"]')?.querySelectorAll<HTMLButtonElement>('button:not(:disabled)') ?? []];
          const index = buttons.indexOf(target as HTMLButtonElement);
          buttons[(index + (event.key === "ArrowDown" ? 1 : -1) + buttons.length) % buttons.length]?.focus();
        }
      }
      if (event.key === "Escape") { event.preventDefault(); target.closest('.desktop-menu-group')?.querySelector<HTMLButtonElement>('[aria-haspopup]')?.focus(); setOpen(undefined); }
    }}>
    {locations.map((location) => {
      const items = platform.menus.getMenu(location);
      if (!items.length) return null;
      return <div className="desktop-menu-group" key={location}>
        <button aria-haspopup="menu" aria-expanded={open === location} onClick={() => setOpen(open === location ? undefined : location)} onPointerEnter={() => open && setOpen(location)}>{location}</button>
        {open === location && <div className="desktop-menu-popover" role="menu" aria-label={`${location} menu`}>
          {items.map((item, index) => {
            const previous = items[index - 1];
            const separated = previous && previous.group !== item.group;
            return <div className="registered-menu-item" key={item.id}>{separated && <hr />}<button role="menuitem" disabled={!platform.commands.isEnabled(item.command)} onClick={() => { setOpen(undefined); restoreFocus(); void platform.commands.executeCommand(item.command); }}><span>{item.checked ? "✓ " : ""}{platform.commands.title(item.command) ?? item.command}</span><kbd>{platform.keybindings.shortcutFor(item.command)?.replace("Mod", navigator.platform.toLowerCase().includes("mac") ? "⌘" : "Ctrl").replaceAll("+", " ")}</kbd></button></div>;
          })}
        </div>}
      </div>;
    })}
  </nav>;
}
