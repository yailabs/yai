import { lazy } from "react";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import { Badge, EmptyState } from "../../components/primitives";
import { Icon } from "../../components/Icon";
import { WorkflowInputAction } from "./applicationActions";
import { WorkflowActions } from "./WorkflowActions";
import { HandoffActions } from "./HandoffActions";
const GraphViewport = lazy(() => import("../../live/GraphViewport").then(module => ({ default: module.GraphViewport })));

export function WorkSurface({ workspace, platform, actions }: Pick<SurfaceRendererProps, "workspace" | "platform" | "actions">) {
  const nodes = workspace.work.nodes;
  const events = workspace.memory.timeline.filter(event => /workflow|handoff|effect|operation|execution|decision/.test(event.kind)).slice(-12).reverse();
  return <article className="live-page work-surface"><header className="operational-heading"><div><h1>Work</h1><p>Structured progression, Case handoffs and committed operational activity.</p></div><Badge>{nodes.length} workflow nodes</Badge></header>
    <section className="work-section"><h2>Workflow</h2><WorkflowActions workspace={workspace} platform={platform} />
      {!nodes.length && <EmptyState title="No bound Workflow" body="Define human checkpoints or bind an existing qualified definition. Work can also leave committed operations and handoffs without a Workflow." />}
      {nodes.length > 1 && <div className="workflow-graph"><GraphViewport nodes={nodes.map(node => ({ id: node.node_id, label: node.node_id, kind: node.node_kind }))} edges={workspace.work.edges} layout="directed" onInspect={actions.inspect} /></div>}
      {nodes.map(node => <div className="workflow-node-row" key={node.node_id}><button className="fact-row" onClick={() => actions.inspect(node.node_id)}><Icon name="work" /><span><strong>{node.node_id}</strong><small>{node.reason}</small></span><Badge tone={node.posture === "satisfied" ? "success" : "neutral"}>{node.posture}</Badge></button>{node.node_kind === "human_input" && node.posture !== "satisfied" && <WorkflowInputAction application={platform.application} workspace={workspace} nodeRef={node.node_id} refresh={() => platform.commands.executeCommand("studio.case.refresh").then(() => undefined)} />}</div>)}
    </section><HandoffActions workspace={workspace} platform={platform} />
    <section className="work-section"><h2>Recent operational history</h2><p>Latest {events.length} matching events from the bounded Case history projection. An event is not a live execution handle.</p>{events.map(event => <button className="fact-row" key={event.id} onClick={() => actions.inspect(event.id)}><Icon name="memory" /><span><strong>{event.kind.replaceAll("_", " ")}</strong><small>{event.component} · generation {event.sequence}</small></span></button>)}</section>
    <section className="work-section"><h2>Executions & effects</h2><p>Reconnect-safe execution submission, result observation, source acquisition and runtime control are not exposed by this published Application contract. Source declarations, reviews and policy changes remain governed actions in their respective perspectives.</p></section>
  </article>;
}
