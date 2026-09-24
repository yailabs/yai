import { Suspense, useCallback, useEffect, useState, useSyncExternalStore } from "react";
import type { PointerEvent, KeyboardEvent } from "react";
import { Icon } from "../../components/Icon";
import { IconButton } from "../../components/primitives";
import { ContextToolLayout, fitTool, type ToolBounds } from "./contextTools";
import type { AuxiliaryViewContribution, WorkbenchRenderContext } from "./types";

export function ContextTools({ layout, views, context, active, select, visible, close }: {
  layout: ContextToolLayout; views: readonly AuxiliaryViewContribution[]; context: WorkbenchRenderContext;
  active: string; select(id: string): void; visible: boolean; close(): void;
}) {
  const states = useSyncExternalStore(useCallback(listener => layout.subscribe(listener).dispose, [layout]), layout.snapshot);
  const [viewport, setViewport] = useState({width: innerWidth, height: innerHeight});
  const [front, setFront] = useState(active);
  const followsSelection = Boolean(views.find(view => view.id === active)?.followsSelection);
  useEffect(() => { const resize = () => setViewport({width: innerWidth, height: innerHeight}); addEventListener("resize", resize); return () => removeEventListener("resize", resize); }, []);
  useEffect(() => { if (visible) layout.update(active, {open: true}); }, [layout, active, visible]);
  useEffect(() => {
    if (visible && followsSelection) layout.update(active, {open: true});
  }, [layout, active, context.selection, visible, followsSelection]);
  const hasFloats = Object.values(states).some(state => state.floating);
  const docked = !layout.get(active).floating;
  const focusSurface = () => requestAnimationFrame(() => document.querySelector<HTMLElement>(".live-surface")?.focus({preventScroll:true}));
  const choose = (id: string) => { layout.update(id, {open:true}); select(id); setFront(id); };
  const initialBounds = (index: number): ToolBounds => ({x: viewport.width - 424 - index * 350, y: 84 + index * 22,
    width: index === 0 ? 400 : 340, height: index === 0 ? 620 : 440});
  return <>
    {visible && hasFloats && <nav className="context-tool-launcher" aria-label="Context tools">
      {views.map(view => <button key={view.id} aria-label={`Open ${view.title} tool`} aria-pressed={Boolean(states[view.id]?.open && (states[view.id]?.floating || active === view.id))} title={view.title} onClick={() => choose(view.id)}><Icon name={view.followsSelection ? "search" : view.id === "Activity" ? "activity" : "message"} size={17}/></button>)}
    </nav>}
    <aside hidden={!visible} className="live-context context-deck" data-docked={docked} id="case-conversation">
      {docked && <header><div className="segmented">{views.map(view => <button key={view.id} aria-pressed={active === view.id} title={view.title} onClick={() => choose(view.id)}><Icon name={view.followsSelection ? "search" : view.id === "Activity" ? "activity" : "message"} size={17}/></button>)}</div><IconButton aria-label="Close context panel" onClick={() => { close(); focusSurface(); }}><Icon name="right" /></IconButton></header>}
      {views.filter(view => view.id === active || states[view.id]).map(view => {
        const state = layout.get(view.id), floating = state.floating, index = views.findIndex(item => item.id === view.id);
        const bounds = fitTool(state.bounds ?? initialBounds(index), viewport);
        const shown = visible && (floating ? state.open : view.id === active);
        const Component = view.component;
        const actions = state.pinnedRef ? {...context.actions, inspect: (ref: string) => {
          // An explicit link inside the card replaces its pinned target; outside
          // selection changes do not. Never leave a navigable relation inert.
          layout.update(view.id, {pinnedRef: ref}); context.actions.inspect(ref);
        }} : context.actions;
        const setBounds = (next: ToolBounds) => layout.update(view.id, {bounds:fitTool(next, viewport)});
        const drag = (event: PointerEvent<HTMLButtonElement>, resize = false) => {
          if (event.button !== 0) return;
          event.preventDefault(); setFront(view.id);
          const target = event.currentTarget, x = event.clientX, y = event.clientY;
          target.setPointerCapture(event.pointerId);
          const move = (next: globalThis.PointerEvent) => setBounds(resize
            ? {...bounds, width:bounds.width + next.clientX - x, height:bounds.height + next.clientY - y}
            : {...bounds, x:bounds.x + next.clientX - x, y:bounds.y + next.clientY - y});
          const end = () => { target.removeEventListener("pointermove", move); target.removeEventListener("lostpointercapture", end); };
          target.addEventListener("pointermove", move); target.addEventListener("lostpointercapture", end);
        };
        const keyboard = (event: KeyboardEvent<HTMLButtonElement>, resize = false) => {
          const delta: Record<string, [number,number]> = {ArrowLeft:[-16,0],ArrowRight:[16,0],ArrowUp:[0,-16],ArrowDown:[0,16]};
          const change = delta[event.key]; if (!change) return;
          event.preventDefault(); event.stopPropagation();
          setBounds(resize ? {...bounds,width:bounds.width+change[0],height:bounds.height+change[1]} : {...bounds,x:bounds.x+change[0],y:bounds.y+change[1]});
        };
        return <section key={`${context.workspace.case.case_ref}:${context.workspace.case.participant_ref}:${view.id}`} hidden={!shown}
          className={`context-tool ${floating ? "context-tool-floating" : "context-tool-docked"}`} aria-label={`${view.title} tool`}
          data-tool={view.id} data-pinned={Boolean(state.pinnedRef)} tabIndex={-1}
          style={floating ? {left:bounds.x,top:bounds.y,width:bounds.width,height:bounds.height,zIndex:front===view.id?42:41} : undefined}
          onFocusCapture={() => setFront(view.id)} onPointerDownCapture={() => setFront(view.id)}
          onKeyDown={event => { if (event.key === "Escape" && event.target === event.currentTarget && floating) { layout.update(view.id,{open:false}); focusSurface(); } }}>
          <header className="context-tool-toolbar">
            {floating ? <button className="context-tool-handle" aria-label={`Move ${view.title}`} title="Drag to move · arrow keys when focused" onPointerDown={event => drag(event)} onKeyDown={event => keyboard(event)}>{view.title}</button> : <span>{view.title}</span>}
            {view.followsSelection && <IconButton aria-label={state.pinnedRef ? "Unpin Inspector" : "Pin Inspector"} aria-pressed={Boolean(state.pinnedRef)} title={state.pinnedRef ? "Follow selection" : "Keep this object while navigating"} onClick={() => layout.update(view.id,{pinnedRef:state.pinnedRef?undefined:context.selection})}><Icon name="pin" size={14}/></IconButton>}
            <IconButton aria-label={`${floating ? "Dock" : "Float"} ${view.title}`} onClick={() => { layout.update(view.id,{floating:!floating,open:true,bounds:state.bounds??bounds}); select(view.id); }}><Icon name={floating?"right":"maximize"} size={14}/></IconButton>
            {floating && <IconButton aria-label={`Close ${view.title} tool`} onClick={() => {layout.update(view.id,{open:false});focusSurface();}}><Icon name="close" size={14}/></IconButton>}
          </header>
          {state.pinnedRef && <div className="context-tool-pinned">Pinned object · <button onClick={() => layout.update(view.id,{pinnedRef:undefined})}>Follow selection</button></div>}
          <div className="context-tool-content"><Suspense fallback={<p className="surface-note">Loading {view.title}…</p>}><Component {...context} actions={actions} selection={state.pinnedRef ?? context.selection}/></Suspense></div>
          {floating && <button className="context-tool-resize" aria-label={`Resize ${view.title}`} title="Drag to resize · arrow keys when focused" onPointerDown={event=>drag(event,true)} onKeyDown={event=>keyboard(event,true)}>⌟</button>}
        </section>;
      })}
    </aside>
  </>;
}
