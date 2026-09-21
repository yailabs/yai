import { useEffect, useRef } from "react";

const focusable = 'button:not(:disabled), input:not(:disabled), select:not(:disabled), textarea:not(:disabled), a[href], [tabindex="0"]';

/** Shared modal focus boundary. Nested popovers restore their owning control. */
export function useModalFocus(close: () => void) {
  const root = useRef<HTMLElement>(null);
  const closeRef = useRef(close);
  closeRef.current = close;
  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    const keyboard = (event: KeyboardEvent) => {
      if (!root.current) return;
      if (event.key === "Escape") { event.preventDefault(); event.stopPropagation(); closeRef.current(); }
      if (event.key !== "Tab") return;
      const elements = [...root.current.querySelectorAll<HTMLElement>(focusable)].filter((element) => element.getClientRects().length);
      const first = elements[0], last = elements.at(-1);
      if (!first) { event.preventDefault(); root.current.focus(); return; }
      if (!root.current.contains(document.activeElement) || (!event.shiftKey && document.activeElement === last)) { event.preventDefault(); first.focus(); }
      else if (event.shiftKey && (document.activeElement === first || document.activeElement === root.current)) { event.preventDefault(); last?.focus(); }
    };
    document.addEventListener("keydown", keyboard, true);
    root.current?.querySelector<HTMLElement>(focusable)?.focus();
    return () => { document.removeEventListener("keydown", keyboard, true); if (previous?.isConnected) previous.focus(); };
  }, []);
  return root;
}
