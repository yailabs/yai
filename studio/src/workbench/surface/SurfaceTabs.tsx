import { useLayoutEffect, useRef } from "react";
import { Icon } from "../../components/Icon";
import type { SurfaceInput, SurfaceRole } from "./model";

export function SurfaceTabs({ inputs, activeId, describe, activate, pin, close }: {
  inputs: readonly SurfaceInput[];
  activeId?: string;
  describe(input: SurfaceInput): { role: SurfaceRole; pinnable: boolean; archetype?: string };
  activate(input: SurfaceInput): void;
  pin(id: string): void;
  close(id: string): void;
}) {
  const strip = useRef<HTMLDivElement>(null);
  useLayoutEffect(() => {
    const element = strip.current;
    if (!element) return;
    const reveal = () => {
      const tab = element.querySelector<HTMLElement>('[data-active="true"]');
      if (!tab) return;
      const bounds = element.getBoundingClientRect();
      const active = tab.getBoundingClientRect();
      if (active.right > bounds.right) element.scrollLeft += active.right - bounds.right;
      if (active.left < bounds.left) element.scrollLeft -= bounds.left - active.left;
    };
    reveal();
    const observer = new ResizeObserver(reveal);
    observer.observe(element);
    return () => observer.disconnect();
  }, [activeId, inputs]);
  return <div className="surface-tabbar"><div ref={strip} className="surface-tabs" role="tablist" aria-label="Open work surfaces" onKeyDown={(event) => {
    if (!(event.target instanceof HTMLElement) || event.target.getAttribute("role") !== "tab") return;
    if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
    const index = inputs.findIndex((input) => input.id === activeId);
    const next = event.key === "Home" ? 0 : event.key === "End" ? inputs.length - 1
      : (index + (event.key === "ArrowRight" ? 1 : -1) + inputs.length) % inputs.length;
    if (!inputs[next]) return;
    event.preventDefault(); activate(inputs[next]);
    event.currentTarget.querySelectorAll<HTMLButtonElement>('[role="tab"]')[next]?.focus();
  }}>{inputs.map((input) => {
    const descriptor = describe(input);
    return <div key={input.id} className={`surface-tab surface-role-${descriptor.role} surface-posture-${input.posture ?? "ready"}`} data-active={activeId === input.id} data-archetype={descriptor.archetype ?? "product"}>
      <button role="tab" tabIndex={activeId === input.id ? 0 : -1} aria-selected={activeId === input.id}
        title={`${input.metadata?.path ?? input.title}${input.pinned ? "" : " · Preview"}${input.posture === "read-only" ? " · Read only" : ""}`}
        onClick={() => activate(input)} onDoubleClick={() => descriptor.pinnable && pin(input.id)}>
        <Icon name={input.icon} size={14} /><span className="surface-tab-title">{input.title}</span>
        {input.dirty ? <i className="dirty-mark" aria-label="Unsaved changes">●</i> : !input.pinned ? <i className="preview-mark">Preview</i> : null}
      </button>
      <button className="surface-tab-close" aria-label={`Close ${input.title}`} onClick={() => close(input.id)}><Icon name="close" size={12} /></button>
    </div>;
  })}</div><select className="surface-switcher" aria-label="Switch open Surface" value={activeId ?? ""} onChange={event => { const next = inputs.find(input => input.id === event.target.value); if (next) activate(next); }}>
    {!inputs.length && <option value="">No open Surfaces</option>}
    {inputs.map(input => <option key={input.id} value={input.id}>{input.title}{input.dirty ? " · Unsaved" : ""} — {describe(input).archetype ?? "product"}</option>)}
  </select></div>;
}
