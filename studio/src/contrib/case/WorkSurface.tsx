import { DecisionHistory } from "./DecisionHistory";
import { RunCaseAction, ExecutionHistory } from "./ExecutionActions";
import { lazy, useCallback, useId, useSyncExternalStore } from "react";
import "./WorkSurface.css";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import { Badge, EmptyState } from "../../components/primitives";
import { Icon } from "../../components/Icon";
import { WorkflowInputAction } from "./applicationActions";
import { WorkflowActions } from "./WorkflowActions";
import { HandoffActions } from "./HandoffActions";
const GraphViewport = lazy(() => import("../../live/GraphViewport").then(module => ({ default: module.GraphViewport })));

const sections = ["Workflow", "Executions", "Handoffs", "Decisions", "Activity"] as const;
type Section = typeof sections[number];

export function WorkSurface({ workspace, platform, actions }: Pick<SurfaceRendererProps, "workspace" | "platform" | "actions">) {
  const identity = useId();
  const sectionKey = `studio.work.section:${JSON.stringify([workspace.case.case_ref, workspace.case.participant_ref])}`;
  const subscribe = useCallback((listener: () => void) => platform.context.subscribe(listener).dispose, [platform]);
  const section = useSyncExternalStore(subscribe, () => {
    const value = platform.context.get(sectionKey) as Section;
    return sections.includes(value) ? value : "Workflow";
  });
  const select = (value: Section) => platform.context.update(sectionKey, value);
  const panel = (value: Section) => ({ role: "tabpanel" as const, id: `${identity}-${value}-panel`, "aria-labelledby": `${identity}-${value}-tab`, hidden: section !== value, tabIndex: 0 });
  const nodes = workspace.work.nodes;
  const events = workspace.memory.timeline.filter(event => /workflow|handoff|effect|operation|execution|decision/.test(event.kind)).slice(-12).reverse();
  return <article className="work-surface">
    <header className="operational-heading"><div><h1>Work</h1><span>Case operations</span></div><RunCaseAction workspace={workspace} platform={platform} /></header>
    <nav className="work-navigation" role="tablist" aria-label="Work sections" onKeyDown={event => {
      const index = sections.indexOf(section);
      const next = event.key === "ArrowRight" ? (index + 1) % sections.length : event.key === "ArrowLeft" ? (index + sections.length - 1) % sections.length : event.key === "Home" ? 0 : event.key === "End" ? sections.length - 1 : undefined;
      if (next === undefined) return;
      event.preventDefault(); select(sections[next]);
      event.currentTarget.querySelector<HTMLButtonElement>(`[data-work-section="${sections[next]}"]`)?.focus();
    }}>{sections.map(value => <button key={value} role="tab" id={`${identity}-${value}-tab`} data-work-section={value} aria-controls={`${identity}-${value}-panel`} aria-selected={section === value} tabIndex={section === value ? 0 : -1} onClick={() => select(value)}>{value}</button>)}</nav>
    <div className="work-content">
    <section {...panel("Workflow")}><header className="work-section-heading"><h2>Workflow</h2><Badge>{nodes.length} nodes</Badge></header><WorkflowActions workspace={workspace} platform={platform} />
      {!nodes.length && <EmptyState title="No bound Workflow" body="Define human checkpoints or bind an existing qualified definition. Work can also leave committed operations and handoffs without a Workflow." />}
      {nodes.length > 1 && <div className="workflow-graph"><GraphViewport nodes={nodes.map(node => ({ id: node.node_id, label: node.node_id, kind: node.node_kind }))} edges={workspace.work.edges} layout="directed" onInspect={actions.inspect} /></div>}
      {nodes.map(node => <div className="workflow-node-row" key={node.node_id}><button className="fact-row" onClick={() => actions.inspect(node.node_id)}><Icon name="work" /><span><strong>{node.node_id}</strong><small>{node.reason}</small></span><Badge tone={node.posture === "satisfied" ? "success" : "neutral"}>{node.posture}</Badge></button>{node.node_kind === "human_input" && node.posture !== "satisfied" && <WorkflowInputAction application={platform.application} workspace={workspace} nodeRef={node.node_id} refresh={() => platform.commands.executeCommand("studio.case.refresh").then(() => undefined)} />}</div>)}
    </section>
    <section {...panel("Executions")}><ExecutionHistory workspace={workspace} platform={platform} /></section>
    <section {...panel("Decisions")}><DecisionHistory key={`${workspace.case.case_ref}:${workspace.case.participant_ref}:${workspace.case.generation}`} workspace={workspace} platform={platform} actions={actions} /></section>
    <section {...panel("Handoffs")}><HandoffActions workspace={workspace} platform={platform} /></section>
    <section {...panel("Activity")} className="work-section"><h2>Recent operational history</h2><p>Latest {events.length} matching events from the bounded Case history projection. An event is not a live execution handle.</p>{events.map(event => <button className="fact-row" key={event.id} onClick={() => actions.inspect(event.id)}><Icon name="memory" /><span><strong>{event.kind.replaceAll("_", " ")}</strong><small>{event.component}</small></span></button>)}</section>
    </div>
  </article>;
}
