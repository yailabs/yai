import { useRef } from "react";

export function Splitter({ label, axis, reverse = false, value, min, max, set }: { label: string; axis: "x" | "y"; reverse?: boolean; value: number; min: number; max: number; set: (value: number) => void }) {
  const drag = useRef<{ coordinate: number; value: number } | null>(null);
  const clamp = (next: number) => set(Math.max(min, Math.min(max, next)));
  return <div className={`live-splitter ${axis}`} role="separator" aria-label={label} tabIndex={0} aria-orientation={axis === "x" ? "vertical" : "horizontal"} aria-valuemin={min} aria-valuemax={max} aria-valuenow={value}
    onKeyDown={(event) => { const decrease = axis === "x" ? "ArrowLeft" : "ArrowDown"; const increase = axis === "x" ? "ArrowRight" : "ArrowUp"; if (event.key === decrease || event.key === increase) { event.preventDefault(); clamp(value + (event.key === increase ? 16 : -16) * (axis === "x" && reverse ? -1 : 1)); } else if (event.key === "Home") { event.preventDefault(); clamp(min); } else if (event.key === "End") { event.preventDefault(); clamp(max); } }}
    onPointerDown={(event) => { if (event.button !== 0) return; event.preventDefault(); event.currentTarget.setPointerCapture(event.pointerId); drag.current = { coordinate: axis === "x" ? event.clientX : event.clientY, value }; }}
    onPointerMove={(event) => { if (!drag.current) return; const coordinate = axis === "x" ? event.clientX : event.clientY; clamp(drag.current.value + (coordinate - drag.current.coordinate) * (reverse ? -1 : 1)); }}
    onPointerUp={(event) => { drag.current = null; event.currentTarget.releasePointerCapture(event.pointerId); }}
    onLostPointerCapture={() => { drag.current = null; }} />;
}
