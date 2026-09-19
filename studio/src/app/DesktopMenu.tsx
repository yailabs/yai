import { useEffect, useMemo, useRef, useState } from "react";
import { commandMap } from "./commands";
import type { StudioCommand, StudioMenu } from "./commands";

export function DesktopMenu({ commands, menus }: { commands: StudioCommand[]; menus: StudioMenu[] }) {
  const [open, setOpen] = useState<string>();
  const root = useRef<HTMLElement>(null);
  const byId = useMemo(() => commandMap(commands), [commands]);

  useEffect(() => {
    const dismiss = (event: PointerEvent) => {
      if (!root.current?.contains(event.target as Node)) setOpen(undefined);
    };
    const keyboard = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(undefined);
    };
    window.addEventListener("pointerdown", dismiss);
    window.addEventListener("keydown", keyboard);
    return () => { window.removeEventListener("pointerdown", dismiss); window.removeEventListener("keydown", keyboard); };
  }, []);

  return <nav ref={root} className="desktop-menu" aria-label="Application menu">
    {menus.map((menu) => <div className="desktop-menu-group" key={menu.label}>
      <button aria-haspopup="menu" aria-expanded={open === menu.label} onClick={() => setOpen(open === menu.label ? undefined : menu.label)} onPointerEnter={() => open && setOpen(menu.label)}>{menu.label}</button>
      {open === menu.label && <div className="desktop-menu-popover" role="menu" aria-label={`${menu.label} menu`}>
        {menu.items.map((item, index) => {
          if (item === "separator") return <hr key={`separator:${index}`} />;
          const command = byId.get(item);
          if (!command) return null;
          return <button key={command.id} role="menuitem" disabled={command.enabled === false} onClick={() => { setOpen(undefined); command.run(); }}><span>{command.checked ? "✓ " : ""}{command.label}</span>{command.shortcut && <kbd>{command.shortcut}</kbd>}</button>;
        })}
      </div>}
    </div>)}
  </nav>;
}
