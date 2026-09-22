import { WorkflowInputAction } from "./applicationActions";
import type { PlatformServices } from "../../platform/services";
import { lazy } from "react";
import { Icon, type IconName } from "../../components/Icon";
import { Badge, EmptyState, PanelHeader } from "../../components/primitives";
import type { CasePresentation } from "../../clients/dataSource";
import type { SurfaceInput } from "../../workbench/surface/model";
import type { AuxiliaryViewProps, SidebarViewProps, SurfaceRendererProps, SurfaceSearchResult } from "../../workbench/kernel/types";
import { fileIconForMedia, fileInput, graphInput, materialInput, resourceInput, sourceInput, timelineInput } from "../surfaces/inputs";
import { OverviewSurface as Overview } from "./OverviewSurface";
import { AuthoritySurface as Authority } from "./AuthoritySurface";
import { KnowledgeSurface as Knowledge } from "./KnowledgeSurface";
import { knowledgeGraph, relationGraph } from "./graph";
import { buildFileTree, type FileTreeNode } from "./environment";

const GraphViewport = lazy(() => import("../../live/GraphViewport").then((module) => ({ default: module.GraphViewport })));

export const perspectives = ["Overview", "Environment", "Knowledge", "Memory", "Authority", "Work", "Compute"] as const;
export type Perspective = typeof perspectives[number];

export const perspectiveMeta: Record<Perspective, { icon: IconName; meaning: string }> = {
  Overview: { icon: "overview", meaning: "Case identity and current posture" },
  Environment: { icon: "environment", meaning: "Sources, files and resources" },
  Knowledge: { icon: "knowledge", meaning: "Qualified source-grounded derivation" },
  Memory: { icon: "memory", meaning: "Committed history and derived relations" },
  Authority: { icon: "authority", meaning: "Policy, scope, review and decisions" },
  Work: { icon: "work", meaning: "Workflow definition and progression" },
  Compute: { icon: "compute", meaning: "Provider and runtime posture" },
};

export function CaseSidebarView({ workspace, containerId, selection, actions }: SidebarViewProps) {
  const perspective = containerId as Perspective;
  const rows = perspectiveRows(workspace, perspective);
  const openRow = (item: Row, pinned = false) => {
    if (item.surface) {
      actions.openSurface({ ...item.surface, id: pinned ? item.surface.identity : "surface:preview", pinned });
    } else if (item.material) {
      actions.openSurface(materialInput(workspace, item.id, item.label, pinned));
    } else {
      actions.inspect(item.id);
    }
  };
  return <div className="live-sidebar-content" data-view-container={containerId}>
    {perspective === "Overview" && <>
      <section className="sidebar-group"><h2>Case</h2><button title={workspace.case.case_ref} className={selection === workspace.case.case_ref ? "selected" : ""} onClick={() => actions.inspect(workspace.case.case_ref)}><Icon name="case" /><span><strong>{workspace.case.display_name}</strong><small>{workspace.case.case_ref}</small></span></button></section>
      <section className="sidebar-group case-outline"><h2>Perspectives</h2>{perspectives.slice(1).map((item) => <button key={item} onClick={() => actions.openPerspective(item)}><Icon name={perspectiveMeta[item].icon} /><span><strong>{item}</strong><small>{perspectiveMeta[item].meaning}</small></span></button>)}</section>
      <section className="sidebar-group"><h2>Participants <span>{workspace.overview.participants.length}</span></h2>{workspace.overview.participants.map((person) => <button key={person.id} className={selection === person.id ? "selected" : ""} onClick={() => actions.inspect(person.id)}><Icon name="people" /><span><strong>{person.id}</strong><small>{person.roles.join(", ") || "No role exposed"}</small></span>{person.is_current && <Badge tone="info">Current</Badge>}</button>)}</section>
    </>}
    {perspective === "Environment" && <EnvironmentExplorer workspace={workspace} selection={selection} open={actions.openSurface} inspect={actions.inspect} />}
    {perspective !== "Environment" && rows.map((group) => <section className="sidebar-group" key={group.label}><h2>{group.label}<span>{group.items.length}</span></h2>{group.items.map((item) => <button key={item.id} title={`${item.label} · ${item.detail}`} className={selection === item.id ? "selected" : ""} onClick={() => openRow(item)} onDoubleClick={() => (item.material || item.surface) && openRow(item, true)}><Icon name={item.icon} /><span><strong>{item.label}</strong><small>{item.detail}</small></span></button>)}{!group.items.length && <p className="sidebar-empty">No qualified items.</p>}</section>)}
  </div>;
}

function EnvironmentExplorer({ workspace, selection, open, inspect }: { workspace: CasePresentation; selection: string; open: (input: SurfaceInput) => void; inspect: (id: string) => void }) {
  return <>
    <section className="sidebar-group environment-files"><h2>Files <span>{workspace.environment.files.length}</span></h2><FileTree nodes={buildFileTree(workspace.environment.files)} workspace={workspace} selection={selection} open={open} inspect={inspect} depth={0} /></section>
    <section className="sidebar-group"><h2>Sources <span>{workspace.environment.sources.length}</span></h2>{workspace.environment.sources.map((source) => <button key={source.id} className={selection === source.id ? "selected" : ""} onClick={() => { inspect(source.id); open(sourceInput(workspace, source.id)); }} onDoubleClick={() => open(sourceInput(workspace, source.id, true))}><Icon name="source" /><span><strong>{source.label}</strong><small>{humanize(source.kind)} · {source.posture ?? "declared"}</small></span></button>)}</section>
    <section className="sidebar-group"><h2>Resources <span>{workspace.environment.resources.length}</span></h2>{workspace.environment.resources.map((resource) => <button key={resource.id} className={selection === resource.id ? "selected" : ""} onClick={() => { inspect(resource.id); open(resourceInput(workspace, resource.id)); }} onDoubleClick={() => open(resourceInput(workspace, resource.id, true))}><Icon name={resource.kind === "database" ? "database" : "resource"} /><span><strong>{resource.label ?? resourceLabel(resource.id)}</strong><small>{humanize(resource.kind)}</small></span></button>)}</section>
  </>;
}

function FileTree({ nodes, workspace, selection, open, inspect, depth }: { nodes: FileTreeNode[]; workspace: CasePresentation; selection: string; open: (input: SurfaceInput) => void; inspect: (id: string) => void; depth: number }) {
  return <div className="file-tree">{nodes.map((node) => node.fileId ? <button key={node.path} style={{ paddingLeft: `${8 + depth * 14}px` }} className={selection === node.fileId ? "selected" : ""} title={node.path} onClick={() => { inspect(node.fileId!); open(fileInput(workspace, node.fileId!)); }} onDoubleClick={() => open(fileInput(workspace, node.fileId!, true))}><Icon name={fileIconForMedia(workspace.environment.files.find((file) => file.id === node.fileId)?.media_type, node.path)} /><span><strong>{node.name}</strong></span></button> : <details key={node.path} open><summary style={{ paddingLeft: `${6 + depth * 14}px` }}><Icon name="environment" /><strong>{node.name}</strong></summary><FileTree nodes={node.children} workspace={workspace} selection={selection} open={open} inspect={inspect} depth={depth + 1} /></details>)}</div>;
}

export function PerspectiveSurface({ workspace, input, actions, selection, platform }: SurfaceRendererProps) {
  const perspective = (input.viewId ?? "Overview") as Perspective;
  if (perspective === "Overview") return <Overview workspace={workspace} inspect={actions.inspect} navigate={actions.openPerspective} />;
  if (perspective === "Environment") return <Environment workspace={workspace} inspect={actions.inspect} openSurface={actions.openSurface} />;
  if (perspective === "Knowledge") return <Knowledge key={workspace.case.case_ref} selected={selection} workspace={workspace} inspect={actions.inspect} openSurface={actions.openSurface} />;
  if (perspective === "Memory") return <Memory workspace={workspace} openSurface={actions.openSurface} />;
  if (perspective === "Authority") return <Authority selected={selection} workspace={workspace} inspect={actions.inspect} />;
  if (perspective === "Work") return <Work platform={platform} workspace={workspace} inspect={actions.inspect} />;
  return <Compute workspace={workspace} inspect={actions.inspect} />;
}

export function TimelineSurface({ workspace, actions }: SurfaceRendererProps) {
  return <div className="live-page memory-page" data-surface-type="case.timeline"><SurfaceHeader workspace={workspace} title="Case Timeline" body="A temporal presentation of committed Case history. Chronology does not imply causality." /><ol className="real-timeline">{workspace.memory.timeline.slice().reverse().map((entry) => <li key={entry.id}><time>{formatTime(entry.committed_at_unix_ms)}</time><button onClick={() => actions.inspect(entry.id)}><i /><span><strong>{humanize(entry.kind)}</strong><small>{entry.component} · generation {entry.sequence}</small></span></button></li>)}</ol>{!workspace.memory.timeline.length && <EmptyState title="No committed activity" body="No history is exposed for the current Case projection." />}</div>;
}

export function GraphSurface({ workspace, input, actions }: SurfaceRendererProps) {
  const projection = input.metadata?.projection;
  const graph = projection === "knowledge" ? knowledgeGraph(workspace) : relationGraph(workspace);
  return <div className="live-page graph-surface" data-surface-type="case.graph"><SurfaceHeader workspace={workspace} title={input.title} body={projection === "knowledge" ? "Qualified source-grounded relations presented as a navigable graph." : "Derived Case relationships presented without becoming canonical truth."} />{graph.nodes.length ? <GraphViewport key={`${workspace.case.case_ref}:${projection}`} nodes={graph.nodes} edges={graph.edges} layout="relational" onInspect={actions.inspect} /> : <EmptyState title={`No ${input.title}`} body="No qualified relation projection is available for this Case." />}</div>;
}

export function SourceSurface({ workspace, input, actions }: SurfaceRendererProps) {
  const source = workspace.environment.sources.find((item) => item.id === input.objectRef);
  if (!source) return <EmptyState title="Source unavailable" body="This Source is not present in the current Case projection." />;
  const files = workspace.environment.files.filter((file) => file.source_ref === source.id);
  const resource = workspace.environment.resources.find((item) => item.id === source.resource_ref);
  return <article className="live-page object-surface source-surface" data-surface-type="environment.source"><SurfaceHeader workspace={workspace} title={source.label} body="A governed Case Source relationship. Its retained material is navigated separately." /><dl className="object-facts"><div><dt>Kind</dt><dd>{humanize(source.kind)}</dd></div><div><dt>Posture</dt><dd>{source.posture ?? "declared"}</dd></div><div><dt>Roles</dt><dd>{source.roles.join(", ")}</dd></div><div><dt>Perimeter</dt><dd>{source.perimeter}</dd></div><div><dt>Resource</dt><dd><button onClick={() => actions.openSurface(resourceInput(workspace, source.resource_ref))}>{resource?.label ?? resourceLabel(source.resource_ref)}</button></dd></div><div><dt>Revision</dt><dd>{source.revision_ref ?? "Not acquired"}</dd></div></dl><FactSection title="Retained material" empty="No acquired material">{files.map((file) => <button className="fact-row" key={file.id} onClick={() => actions.openSurface(fileInput(workspace, file.id))}><Icon name={fileIconForMedia(file.media_type, file.path)} /><span><strong>{fileName(file.path)}</strong><small>{file.path}</small></span></button>)}</FactSection><p className="surface-note">Knowledge may derive from this Source, but remains a rebuildable projection rather than the Source itself.</p></article>;
}

export function ResourceSurface({ workspace, input, actions }: SurfaceRendererProps) {
  const resource = workspace.environment.resources.find((item) => item.id === input.objectRef);
  if (!resource) return <EmptyState title="Resource unavailable" body="This Resource is not present in the current Case projection." />;
  const sources = workspace.environment.sources.filter((source) => source.resource_ref === resource.id);
  return <article className="live-page object-surface resource-surface" data-surface-type="environment.resource"><SurfaceHeader workspace={workspace} title={resource.label ?? resourceLabel(resource.id)} body={`${humanize(resource.kind)} capability attached to this Case.`} /><dl className="object-facts"><div><dt>Kind</dt><dd>{humanize(resource.kind)}</dd></div><div><dt>Review</dt><dd>{humanize(resource.review_requirement)}</dd></div><div><dt>Policy</dt><dd>{resource.policy_ref}</dd></div><div><dt>Operations</dt><dd>{resource.operations.length ? resource.operations.join(", ") : "No disclosed operations"}</dd></div>{resource.read_prefixes.length > 0 && <div><dt>Read scope</dt><dd>{resource.read_prefixes.join(", ")}</dd></div>}{resource.allowed_write_prefix && <div><dt>Write scope</dt><dd>{resource.allowed_write_prefix}</dd></div>}</dl><FactSection title="Related Sources" empty="No related Sources">{sources.map((source) => <button className="fact-row" key={source.id} onClick={() => actions.openSurface(sourceInput(workspace, source.id))}><Icon name="source" /><span><strong>{source.label}</strong><small>{humanize(source.kind)} · {source.posture ?? "declared"}</small></span></button>)}</FactSection></article>;
}

export function searchTimelineSurface({ workspace }: Pick<SurfaceRendererProps, "workspace">, _input: SurfaceRendererProps["input"], query: string): readonly SurfaceSearchResult[] {
  const needle = query.trim().toLocaleLowerCase();
  if (!needle) return [];
  return workspace.memory.timeline
    .filter((entry) => `${entry.kind} ${entry.component} ${entry.summary ?? ""} ${entry.sequence}`.toLocaleLowerCase().includes(needle))
    .map((entry) => ({ id: entry.id, label: humanize(entry.kind), detail: `${entry.component} · generation ${entry.sequence}`, objectRef: entry.id }));
}

export function searchGraphSurface({ workspace }: Pick<SurfaceRendererProps, "workspace">, input: SurfaceRendererProps["input"], query: string): readonly SurfaceSearchResult[] {
  const needle = query.trim().toLocaleLowerCase();
  if (!needle) return [];
  const graph = input.metadata?.projection === "knowledge" ? knowledgeGraph(workspace) : relationGraph(workspace);
  return graph.nodes
    .filter((node) => `${node.label} ${node.id} ${node.kind}`.toLocaleLowerCase().includes(needle))
    .map((node) => ({ id: node.id, label: node.label ?? node.id, detail: humanize(node.kind ?? "reference"), objectRef: node.id }));
}

export function ConversationView({ workspace }: AuxiliaryViewProps) {
  return <div className="context-scroll conversation-view"><PanelHeader title="Conversation" detail={workspace.conversation.read_only ? "Committed turns · read only" : "Available"} />{workspace.conversation.turns.map((turn) => { const participant = workspace.overview.participants.find((item) => item.id === turn.participant_ref); const tone = participant?.roles.some((role) => /assistant|ai|model/i.test(role)) ? "ai" : turn.participant_ref.startsWith("yai") ? "system" : "human"; return <article className={`real-turn turn-${tone}`} key={turn.id}><header><span className="turn-avatar">{turn.participant_ref.slice(0, 2).toUpperCase()}</span><strong>{turn.participant_ref}</strong><Badge>Generation {turn.generation}</Badge></header>{turn.parts.map((part, index) => <p key={index}>{part.text ?? `[${part.modality} · ${part.media_type}]`}</p>)}</article>; })}{!workspace.conversation.turns.length && <EmptyState title="No committed turns" body="This Case has no conversation Turns visible to the current participant." />}<div className="read-only-draft"><textarea disabled placeholder="Send is not qualified" aria-label="Conversation draft unavailable" /><small>Conversation is part of this Case. Sending is not qualified at this boundary.</small></div></div>;
}

export function ActivityView({ workspace, actions }: AuxiliaryViewProps) {
  return <div className="context-scroll activity-view"><PanelHeader title="Activity" detail={`Latest ${Math.min(20, workspace.memory.timeline.length)} of ${workspace.memory.timeline.length} events`} />{workspace.memory.timeline.slice(-20).reverse().map((entry) => <button className={`context-event event-${activityTone(entry.kind)}`} key={entry.id} onClick={() => actions.inspect(entry.id)}><time>{formatTime(entry.committed_at_unix_ms)}</time><i /><span><strong>{humanize(entry.kind)}</strong><small>{entry.component} · generation {entry.sequence}</small></span></button>)}{workspace.memory.timeline.length > 20 && <button className="ui-button activity-open-timeline" onClick={() => actions.openSurface(timelineInput(true))}>Open full timeline</button>}{!workspace.memory.timeline.length && <EmptyState title="No committed activity" body="No history is exposed for the current Case projection." />}</div>;
}

export { InspectorView } from "./InspectorView";

function SurfaceHeader({ workspace, title, body }: { workspace: CasePresentation; title: string; body: string }) { return <header className="surface-title"><small title={workspace.case.case_ref}>{workspace.case.display_name} · Generation {workspace.case.generation}</small><h1>{title}</h1><p>{body}</p></header>; }

function Environment({ workspace, openSurface }: { workspace: CasePresentation; inspect: (id: string) => void; openSurface: (input: SurfaceInput) => void }) { return <div className="live-page"><SurfaceHeader workspace={workspace} title="Environment" body="The files available to this Case, the governed Sources that define its information perimeter, and the Resources through which it can operate." /><section className="environment-summary"><button onClick={() => workspace.environment.files[0] && openSurface(fileInput(workspace, workspace.environment.files[0].id))}><Icon name="file" size={22} /><strong>{workspace.environment.files.length}</strong><span>Files</span></button><button onClick={() => workspace.environment.sources[0] && openSurface(sourceInput(workspace, workspace.environment.sources[0].id))}><Icon name="sources" size={22} /><strong>{workspace.environment.sources.length}</strong><span>Sources</span></button><button onClick={() => workspace.environment.resources[0] && openSurface(resourceInput(workspace, workspace.environment.resources[0].id))}><Icon name="environment" size={22} /><strong>{workspace.environment.resources.length}</strong><span>Resources</span></button></section><FactSection title="Information perimeter" empty="No Sources attached">{workspace.environment.sources.map((source) => <button className="fact-row" key={source.id} onClick={() => openSurface(sourceInput(workspace, source.id))}><Icon name="source" /><span><strong>{source.label}</strong><small>{humanize(source.kind)} · {source.perimeter}</small></span><Badge>{source.posture ?? "declared"}</Badge></button>)}</FactSection></div>; }

function Memory({ workspace, openSurface }: { workspace: CasePresentation; openSurface: (input: SurfaceInput) => void }) { return <div className="live-page memory-page"><SurfaceHeader workspace={workspace} title="Memory" body="Committed history and derived relations are first-class Case surfaces." /><div className="surface-launch-grid"><button onClick={() => openSurface(timelineInput(true))}><Icon name="memory" size={22} /><span><strong>Case Timeline</strong><small>{workspace.memory.timeline.length} committed events</small></span></button><button onClick={() => openSurface(graphInput("memory", true))}><Icon name="graph" size={22} /><span><strong>Experience Graph</strong><small>{workspace.memory.relations.length} derived relations</small></span></button></div></div>; }

function Work({ workspace, inspect, platform }: { platform: PlatformServices; workspace: CasePresentation; inspect: (id: string) => void }) { const nodes = workspace.work.nodes.map((node) => ({ id: node.node_id, label: compact(node.node_id), kind: node.node_kind })); return <div className="live-page"><SurfaceHeader workspace={workspace} title="Work" body="Workflow definition and current progression remain distinct." />{workspace.work.status === "empty" ? <EmptyState title="No workflow is configured" body={workspace.work.message ?? "No workflow binding is exposed."} /> : <><GraphViewport key={workspace.case.case_ref} nodes={nodes} edges={workspace.work.edges} layout="directed" onInspect={inspect} /><FactSection title="Current work" empty="No work nodes">{workspace.work.nodes.map((node) => <div className="workflow-node-row" key={node.node_id}><button className="fact-row" onClick={() => inspect(node.node_id)}><Icon name="work" /><span><strong>{node.node_id}</strong><small>{node.reason}</small></span><Badge>{node.posture}</Badge></button>{node.node_kind === "human_input" && node.posture !== "satisfied" && <WorkflowInputAction application={platform.application} workspace={workspace} nodeRef={node.node_id} refresh={() => { void platform.commands.executeCommand("studio.case.refresh"); }} />}</div>)}</FactSection></>}</div>; }
function Compute({ workspace, inspect }: { workspace: CasePresentation; inspect: (id: string) => void }) { return <div className="live-page"><SurfaceHeader workspace={workspace} title="Compute" body="Models run through governed provider/runtime targets; deployment identity and health remain distinct from model identity." /><FactSection title="Targets / deployments" empty={workspace.compute.message}>{workspace.compute.targets.map((target) => <button className="provider-row" key={target.id} onClick={() => inspect(target.id)}><Icon name="compute" /><span><strong>{target.provider_key}</strong><small>Model {target.model_id} · runtime adapter {target.adapter}</small></span><dl><dt>Locality</dt><dd>{target.locality}</dd><dt>Endpoint</dt><dd>{target.endpoint}</dd><dt>Management</dt><dd>{target.management}</dd></dl></button>)}</FactSection>{!workspace.compute.targets.length && <p className="surface-note">No model, provider/runtime or target/deployment is bound to this Case. YVEX management is a separate future control plane.</p>}</div>; }
function FactSection({ title, empty, children }: { title: string; empty: string; children: React.ReactNode }) { const values = Array.isArray(children) ? children : [children]; return <section className="live-section"><PanelHeader title={title} />{values.length && values.some(Boolean) ? children : <EmptyState title={empty} body="Studio has not substituted authored product data." />}</section>; }

type Row = { id: string; label: string; detail: string; icon: IconName; material?: boolean; surface?: SurfaceInput };
function perspectiveRows(workspace: CasePresentation, perspective: Perspective): { label: string; items: Row[] }[] {
  // Environment owns a typed explorer rather than flattening Files, Sources and
  // Resources into the generic material-row path.
  if (perspective === "Environment") return [];
  if (perspective === "Knowledge") return [{ label: "Views", items: [{ id: "knowledge-graph", label: "Knowledge Graph", detail: `${workspace.knowledge.relations.length} relations`, icon: "graph", surface: graphInput("knowledge") }] }, { label: "Documents", items: workspace.knowledge.sources.map((item) => ({ id: item.id, label: item.label, detail: `${item.media_type} · ${item.path}`, icon: fileIconForMedia(item.media_type, item.path), material: true })) }, { label: "Entities", items: workspace.knowledge.entities.map((item) => ({ id: item.id, label: compact(item.id), detail: `${item.definitions.length} definitions`, icon: "knowledge" })) }];
  if (perspective === "Memory") return [{ label: "Views", items: [{ id: "memory-timeline", label: "Case Timeline", detail: `${workspace.memory.timeline.length} events`, icon: "memory", surface: timelineInput() }, { id: "memory-graph", label: "Experience Graph", detail: `${workspace.memory.relations.length} relations`, icon: "graph", surface: graphInput("memory") }] }, { label: "History", items: workspace.memory.timeline.slice(-12).reverse().map((item) => ({ id: item.id, label: humanize(item.kind), detail: `Generation ${item.sequence}`, icon: "memory" })) }];
  if (perspective === "Authority") return [{ label: "Policies", items: workspace.authority.policies.map((item) => ({ id: item.id, label: humanize(item.policy_key), detail: `version ${item.version} · bound`, icon: "authority" })) }, { label: "Reviews", items: workspace.authority.reviews.map((item) => ({ id: item.id, label: compact(item.id), detail: item.status, icon: "review" })) }];
  if (perspective === "Work") return [{ label: "Workflow", items: workspace.work.nodes.map((item) => ({ id: item.node_id, label: compact(item.node_id), detail: item.posture, icon: "work" })) }];
  if (perspective === "Compute") return [{ label: "Targets", items: workspace.compute.targets.map((item) => ({ id: item.id, label: item.provider_key, detail: item.model_id, icon: "compute" })) }];
  return [];
}

function compact(value: string) { return value.length > 38 ? `${value.slice(0, 20)}…${value.slice(-12)}` : value; }
function fileName(value: string) { return value.split(/[\\/]/).filter(Boolean).at(-1) ?? value; }
function resourceLabel(value: string) { return value.replace(/^resource:/, "").replaceAll("-", " ").replace(/^./, (char) => char.toUpperCase()); }
function humanize(value: string) { return value.replaceAll("_", " ").replace(/^./, (char) => char.toUpperCase()); }
function formatTime(value: number) { return new Intl.DateTimeFormat(undefined, { hour: "2-digit", minute: "2-digit" }).format(value); }
function activityTone(value: string) { if (/fail|deny|revoke/i.test(value)) return "error"; if (/review|wait|request/i.test(value)) return "warning"; if (/complete|commit|create|admit/i.test(value)) return "success"; return "info"; }
