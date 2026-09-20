import type { MouseEvent as ReactMouseEvent, PointerEvent as ReactPointerEvent } from "react";

const interactiveSelector = "button, input, select, textarea, a, [role='button'], [role='menu']";

function targetsInteractiveControl(target: EventTarget) {
  return (target as HTMLElement).closest(interactiveSelector) !== null;
}

export function beginDesktopWindowDrag(event: ReactPointerEvent<HTMLElement>) {
  if (!window.__TAURI__ || event.button !== 0 || targetsInteractiveControl(event.target)) return;
  event.preventDefault();
  void window.__TAURI__.core.invoke("desktop_start_dragging");
}

export function toggleDesktopWindowMaximize(event: ReactMouseEvent<HTMLElement>) {
  if (!window.__TAURI__ || targetsInteractiveControl(event.target)) return;
  event.preventDefault();
  void window.__TAURI__.core.invoke("desktop_toggle_maximize");
}
