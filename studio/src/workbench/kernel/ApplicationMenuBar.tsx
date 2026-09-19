import { useEffect, useRef, useState } from "react";
import type { PlatformServices } from "../../platform/services";
import type { MenuLocation } from "../../platform/menus";

const locations: MenuLocation[] = ["YAI", "File", "Edit", "View", "Go", "Case", "Terminal", "Help"];

export function ApplicationMenuBar({ platform }: { platform: PlatformServices }) {
  const [open, setOpen] = useState<MenuLocation>();
  const [, render] = useState(0);
  const root = useRef<HTMLElement>(null);
  useEffect(() => platform.context.subscribe(() => render((value) => value + 1)).dispose, [platform]);
  useEffect(() => {
    const dismiss = (event: PointerEvent) => { if (!root.current?.contains(event.target as Node)) setOpen(undefined); };
    const keyboard = (event: KeyboardEvent) => { if (event.key === "Escape") setOpen(undefined); };
    window.addEventListener("pointerdown", dismiss); window.addEventListener("keydown", keyboard);
    return () => { window.removeEventListener("pointerdown", dismiss); window.removeEventListener("keydown", keyboard); };
  }, []);
  return <nav ref={root} className="desktop-menu" aria-label="Application menu">
    {locations.map((location) => {
      const items = platform.menus.getMenu(location);
      if (!items.length) return null;
      return <div className="desktop-menu-group" key={location}>
        <button aria-haspopup="menu" aria-expanded={open === location} onClick={() => setOpen(open === location ? undefined : location)} onPointerEnter={() => open && setOpen(location)}>{location}</button>
        {open === location && <div className="desktop-menu-popover" role="menu" aria-label={`${location} menu`}>
          {items.map((item, index) => {
            const previous = items[index - 1];
            const separated = previous && previous.group !== item.group;
            return <div className="registered-menu-item" key={item.id}>{separated && <hr />}<button role="menuitem" disabled={!platform.commands.isEnabled(item.command)} onClick={() => { setOpen(undefined); void platform.commands.executeCommand(item.command); }}><span>{item.checked ? "✓ " : ""}{platform.commands.title(item.command) ?? item.command}</span><kbd>{shortcutFor(item.command)}</kbd></button></div>;
          })}
        </div>}
      </div>;
    })}
  </nav>;
}

function shortcutFor(command: string) {
  const values: Record<string, string> = {
    "studio.file.openCase": "Ctrl O", "studio.view.toggleExplorer": "Ctrl B",
    "studio.view.toggleContext": "Ctrl Shift B", "studio.view.toggleBottomPanel": "Ctrl J",
    "studio.go.back": "Alt Left", "studio.go.forward": "Alt Right",
    "studio.terminal.new": "Ctrl Shift `", "studio.terminal.focus": "Ctrl `",
  };
  return values[command] ?? "";
}
