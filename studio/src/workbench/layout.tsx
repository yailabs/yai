import { useEffect, useRef, useState } from "react";

export const clamp = (value: number, min: number, max: number) =>
  Math.max(min, Math.min(max, value));
export function useLayout() {
  const [viewport, setViewport] = useState({
    width: window.innerWidth,
    height: window.innerHeight,
  });
  const [left, setLeft] = useState(238);
  const [right, setRight] = useState(316);
  const [bottom, setBottom] = useState(206);
  const [leftOpen, setLeftOpen] = useState(true);
  const [rightOpen, setRightOpen] = useState(true);
  const [bottomOpen, setBottomOpen] = useState(true);
  useEffect(() => {
    const resize = () =>
      setViewport({ width: window.innerWidth, height: window.innerHeight });
    window.addEventListener("resize", resize);
    const keys = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey) || event.altKey) return;
      const restoreFocus = (panel: string, label: string) => {
        if (document.activeElement?.closest(panel))
          document
            .querySelector<HTMLButtonElement>(`[aria-label="${label}"]`)
            ?.focus();
      };
      if (event.key.toLowerCase() === "j") {
        event.preventDefault();
        restoreFocus("#case-tools", "Toggle bottom panel");
        setBottomOpen((value) => !value);
      }
      if (event.key.toLowerCase() === "b") {
        event.preventDefault();
        restoreFocus(
          event.shiftKey ? "#case-conversation" : "#case-sidebar",
          event.shiftKey ? "Toggle context panel" : "Toggle Case explorer",
        );
        (event.shiftKey ? setRightOpen : setLeftOpen)((value) => !value);
      }
    };
    window.addEventListener("keydown", keys);
    return () => {
      window.removeEventListener("resize", resize);
      window.removeEventListener("keydown", keys);
    };
  }, []);
  const rightMax = Math.max(
    260,
    Math.min(420, viewport.width - (leftOpen ? 345 : 0) - 460),
  );
  const rightSize = clamp(right, 260, rightMax);
  const leftMax = Math.max(
    190,
    Math.min(340, viewport.width - (rightOpen ? rightSize + 5 : 0) - 460),
  );
  const bottomMax = Math.max(140, Math.min(420, viewport.height - 380));
  return {
    left: clamp(left, 190, leftMax),
    right: rightSize,
    bottom: clamp(bottom, 140, bottomMax),
    leftMax,
    rightMax,
    bottomMax,
    setLeft,
    setRight,
    setBottom,
    leftOpen,
    rightOpen,
    bottomOpen,
    setLeftOpen,
    setRightOpen,
    setBottomOpen,
  };
}
export type Layout = ReturnType<typeof useLayout>;

export function Splitter({
  label,
  controls,
  value,
  min,
  max,
  axis,
  reverse = false,
  onChange,
}: {
  label: string;
  controls: string;
  value: number;
  min: number;
  max: number;
  axis: "x" | "y";
  reverse?: boolean;
  onChange: (value: number) => void;
}) {
  const drag = useRef<{ coordinate: number; value: number } | null>(null);
  const direction = reverse ? -1 : 1;
  return (
    <div
      role="separator"
      tabIndex={0}
      className={`splitter ${axis}`}
      aria-label={label}
      aria-controls={controls}
      aria-orientation={axis === "x" ? "vertical" : "horizontal"}
      aria-valuenow={Math.round(value)}
      aria-valuemin={min}
      aria-valuemax={max}
      onPointerDown={(event) => {
        if (event.button !== 0) return;
        event.preventDefault();
        event.currentTarget.focus();
        drag.current = {
          coordinate: axis === "x" ? event.clientX : event.clientY,
          value,
        };
        event.currentTarget.setPointerCapture(event.pointerId);
      }}
      onPointerMove={(event) => {
        if (!drag.current) return;
        const coordinate = axis === "x" ? event.clientX : event.clientY;
        onChange(
          clamp(
            drag.current.value +
              (coordinate - drag.current.coordinate) * direction,
            min,
            max,
          ),
        );
      }}
      onPointerUp={(event) => {
        drag.current = null;
        event.currentTarget.releasePointerCapture(event.pointerId);
      }}
      onLostPointerCapture={() => {
        drag.current = null;
      }}
      onKeyDown={(event) => {
        const delta =
          axis === "x"
            ? { ArrowLeft: -10, ArrowRight: 10 }
            : { ArrowUp: -10, ArrowDown: 10 };
        if (event.key === "Home" || event.key === "End") {
          event.preventDefault();
          onChange(event.key === "Home" ? min : max);
        } else if (event.key in delta) {
          event.preventDefault();
          onChange(
            clamp(
              value + delta[event.key as keyof typeof delta]! * direction,
              min,
              max,
            ),
          );
        }
      }}
    />
  );
}
