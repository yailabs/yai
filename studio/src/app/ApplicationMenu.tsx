import { useEffect, useRef, useState } from "react";
import type { ScenarioId } from "../clients/presentation";
import type { Layout } from "../workbench/layout";

type MenuName = "File" | "View";

export function ApplicationMenu({
  view,
  cases,
  currentCase,
  layout,
  openCase,
  newCase,
}: {
  view: "start" | "new" | "case" | "unknown";
  cases: readonly { id: ScenarioId; label: string }[];
  currentCase: ScenarioId | null;
  layout: Layout;
  openCase: (id: ScenarioId) => void;
  newCase: (fromSource: boolean) => void;
}) {
  const [open, setOpen] = useState<MenuName | null>(null);
  const root = useRef<HTMLElement>(null);
  const act = (action: () => void) => {
    setOpen(null);
    action();
  };

  useEffect(() => {
    const close = (event: PointerEvent) => {
      if (!root.current?.contains(event.target as Node)) setOpen(null);
    };
    const keys = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(null);
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "n") {
        event.preventDefault();
        setOpen(null);
        newCase(false);
      }
    };
    window.addEventListener("pointerdown", close);
    window.addEventListener("keydown", keys);
    return () => {
      window.removeEventListener("pointerdown", close);
      window.removeEventListener("keydown", keys);
    };
  }, [newCase]);

  const toggle = (name: MenuName) =>
    setOpen((value) => (value === name ? null : name));
  const focusMain = () =>
    document
      .querySelector<HTMLElement>(
        view === "case"
          ? ".work-surface [role='tab'][aria-selected='true']"
          : view === "new"
            ? ".case-composer input"
            : ".start-center button",
      )
      ?.focus();

  return (
    <nav className="application-menu" aria-label="Application menu" ref={root}>
      <span className="application-name">YAI Studio</span>
      <div className="menu-entry">
        <button
          aria-haspopup="menu"
          aria-expanded={open === "File"}
          onClick={() => toggle("File")}
          onKeyDown={(event) => {
            if (event.key === "ArrowDown") {
              event.preventDefault();
              setOpen("File");
              requestAnimationFrame(() =>
                root.current
                  ?.querySelector<HTMLButtonElement>(
                    ".menu-entry:first-of-type [role='menuitem']",
                  )
                  ?.focus(),
              );
            }
          }}
        >
          File
        </button>
        {open === "File" && (
          <div className="application-popover" role="menu" aria-label="File">
            <button role="menuitem" onClick={() => act(() => newCase(false))}>
              <span>New Case…</span>
              <kbd>Ctrl N</kbd>
            </button>
            <button role="menuitem" onClick={() => act(() => newCase(true))}>
              <span>New Case from Source…</span>
            </button>
            <div className="menu-rule" />
            <span className="menu-label">OPEN FIXTURE CASE</span>
            {cases.map((item) => (
              <button
                role="menuitem"
                key={item.id}
                onClick={() => act(() => openCase(item.id))}
              >
                <span>{item.label}</span>
                {currentCase === item.id && (
                  <span aria-label="Current Case">✓</span>
                )}
              </button>
            ))}
          </div>
        )}
      </div>
      <div className="menu-entry">
        <button
          aria-haspopup="menu"
          aria-expanded={open === "View"}
          onClick={() => toggle("View")}
        >
          View
        </button>
        {open === "View" && (
          <div className="application-popover" role="menu" aria-label="View">
            <button role="menuitem" onClick={() => act(focusMain)}>
              <span>Focus Content</span>
            </button>
            {view === "case" && (
              <>
                <div className="menu-rule" />
                <button
                  role="menuitemcheckbox"
                  aria-checked={layout.leftOpen}
                  onClick={() =>
                    act(() => layout.setLeftOpen((value) => !value))
                  }
                >
                  <span>Case Explorer</span>
                  <kbd>Ctrl B</kbd>
                </button>
                <button
                  role="menuitemcheckbox"
                  aria-checked={layout.bottomOpen}
                  onClick={() =>
                    act(() => layout.setBottomOpen((value) => !value))
                  }
                >
                  <span>Bottom Panel</span>
                  <kbd>Ctrl J</kbd>
                </button>
                <button
                  role="menuitemcheckbox"
                  aria-checked={layout.rightOpen}
                  onClick={() =>
                    act(() => layout.setRightOpen((value) => !value))
                  }
                >
                  <span>Context Panel</span>
                  <kbd>Ctrl ⇧ B</kbd>
                </button>
              </>
            )}
          </div>
        )}
      </div>
      <span className="menu-context">
        {view === "case"
          ? "Case Workbench"
          : view === "new"
            ? "Case composition"
            : "Start Center"}
      </span>
    </nav>
  );
}
