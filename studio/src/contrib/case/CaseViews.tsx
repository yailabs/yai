import { ResourceRequestability } from "./ResourceRequestability";
import { DeclareSourceAction, RevokeSourceAction, PublishSourcePolicyAction } from "./SourceActions";
import type { PlatformServices } from "../../platform/services";
import { lazy, useEffect, useState, type ReactNode } from "react";
import { Icon, type IconName } from "../../components/Icon";
import { Badge, EmptyState, PanelHeader, SearchInput } from "../../components/primitives";
import type { CasePresentation } from "../../clients/dataSource";
import type { SurfaceInput } from "../../workbench/surface/model";
import type { AuxiliaryViewProps, SidebarViewProps, SurfaceRendererProps, SurfaceSearchResult } from "../../workbench/kernel/types";
import { fileIconForMedia, fileInput, graphInput, memoryInput, materialInput, resourceInput, sourceInput, timelineInput } from "../surfaces/inputs";
import { OverviewSurface as Overview } from "./OverviewSurface";
import { AuthoritySurface as Authority } from "./AuthoritySurface";
import { KnowledgeSurface as Knowledge } from "./KnowledgeSurface";
import { knowledgeGraph, relationGraph } from "./graph";
import { buildFileTree, environmentMaterials, environmentKindIcon, isPolicySource, type FileTreeNode } from "./environment";

const AcquireSourceAction = lazy(() => import("./ExecutionActions").then(module => ({ default: module.AcquireSourceAction })));
const RequestResourceAction = lazy(() => import("./ExecutionActions").then(module => ({ default: module.RequestResourceAction })));
const AttachResourceAction = lazy(() => import("./ResourceSetup").then(module => ({ default: module.AttachResourceAction })));
const AttachProcessAction = lazy(() => import("./ExecutionActions").then(module => ({ default: module.AttachProcessAction })));

const WorkSurface = lazy(() => import("./WorkSurface").then(module => ({ default: module.WorkSurface })));
const ComputeSurface = lazy(() => import("./ComputeSurface").then(module => ({ default: module.ComputeSurface })));

const GraphViewport = lazy(() => import("../../live/GraphViewport").then((module) => ({ default: module.GraphViewport })));

export const perspectives = ["Overview", "Environment", "Knowledge", "Memory", "Authority", "Work", "Compute"] as const;
export type Perspective = typeof perspectives[number];

export const perspectiveMeta: Record<Perspective, { icon: IconName; meaning: string }> = {
  Overview: { icon: "overview", meaning: "Case identity and current posture" },
  Environment: { icon: "environment", meaning: "Sources, files and resources" },
  Knowledge: { icon: "knowledge", meaning: "Qualified source-grounded derivation" },
  Memory: { icon: "memory", meaning: "Committed history and derived relations" },
  Authority: { icon: "authority", meaning: "Policy, scope, review and decisions" },
  Work: { icon: "work", meaning: "Workflow, handoffs and operational history" },
  Compute: { icon: "compute", meaning: "Provider and runtime posture" },
};

export function CaseSidebarView({ workspace, containerId, selection, actions, platform }: SidebarViewProps) {
  const [query, setQuery] = useState("");
  useEffect(() => setQuery(""), [containerId, workspace.case.case_ref]);
  const needle = query.trim().toLocaleLowerCase();
  const perspective = containerId as Perspective;
  const rows = perspectiveRows(workspace, perspective).map(group => ({ ...group, total: group.items.length, items: group.items.filter(item => `${item.label} ${item.detail}`.toLocaleLowerCase().includes(needle)) }));
  const openRow = (item: Row, pinned = false) => {
    if (perspective === "Work") {
      platform.context.update(`studio.work.section:${JSON.stringify([workspace.case.case_ref, workspace.case.participant_ref])}`, "Workflow");
      actions.openPerspective("Work");
    }
    if (item.surface) {
      actions.openSurface({ ...item.surface, id: pinned ? item.surface.identity : "surface:preview", pinned });
    } else if (item.material) {
      actions.openSurface(materialInput(workspace, item.id, item.label, pinned));
    } else {
      actions.inspect(item.id);
    }
  };
  return <div className="live-sidebar-content" data-view-container={containerId}>
    {perspective !== "Overview" && <SearchInput className="sidebar-filter" aria-label={`Filter ${perspective}`} placeholder={`Filter ${perspective.toLowerCase()}`} value={query} onChange={event => setQuery(event.target.value)} />}
    {perspective === "Overview" && <>
      <section className="sidebar-group"><h2>Case</h2><button title={workspace.case.case_ref} className={selection === workspace.case.case_ref ? "selected" : ""} onClick={() => actions.inspect(workspace.case.case_ref)}><Icon name="case" /><span><strong>{workspace.case.display_name}</strong><small>{workspace.case.case_ref}</small></span></button></section>
      <section className="sidebar-group case-outline"><h2>Perspectives</h2>{perspectives.slice(1).map((item) => <button key={item} onClick={() => actions.openPerspective(item)}><Icon name={perspectiveMeta[item].icon} /><span><strong>{item}</strong><small>{perspectiveMeta[item].meaning}</small></span></button>)}</section>
      <section className="sidebar-group"><h2>Participants <span>{workspace.overview.participants.length}</span></h2>{workspace.overview.participants.map((person) => <button key={person.id} className={selection === person.id ? "selected" : ""} onClick={() => actions.inspect(person.id)}><Icon name="people" /><span><strong>{person.id}</strong><small>{person.roles.join(", ") || "No role exposed"}</small></span>{person.is_current && <Badge tone="info">Current</Badge>}</button>)}</section>
    </>}
    {perspective === "Environment" && <EnvironmentExplorer query={needle} workspace={workspace} selection={selection} open={actions.openSurface} inspect={actions.inspect} />}
    {perspective !== "Environment" && rows.map((group) => <SidebarSection key={group.label} title={group.label} count={group.items.length} total={group.total}>{group.items.map((item) => <button key={item.id} title={`${item.label} · ${item.detail}`} className={selection === item.id ? "selected" : ""} onClick={() => openRow(item)} onDoubleClick={() => (item.material || item.surface) && openRow(item, true)}><Icon name={item.icon} /><span><strong>{item.label}</strong><small>{item.detail}</small></span></button>)}{!group.items.length && <p className="sidebar-empty">{query ? "No matching items." : "No qualified items."}</p>}</SidebarSection>)}
  </div>;
}

function SidebarSection({ title, count, total = count, children, className = "" }: { title: string; count: number; total?: number; children: ReactNode; className?: string }) { return <details className={`sidebar-group ${className}`} open><summary><strong>{title}</strong><span>{count === total ? count : `${count}/${total}`}</span></summary>{children}</details>; }

function EnvironmentExplorer({ workspace, selection, open, inspect, query }: { query: string; workspace: CasePresentation; selection: string; open: (input: SurfaceInput) => void; inspect: (id: string) => void }) {
  const material = environmentMaterials(workspace);
  const files = material.files.filter(file => file.path.toLocaleLowerCase().includes(query));
  const sources = material.sources.filter(source => `${source.label} ${source.kind} ${source.posture}`.toLocaleLowerCase().includes(query));
  const resources = workspace.environment.resources.filter(resource => `${resource.label ?? resource.id} ${resource.kind}`.toLocaleLowerCase().includes(query));
  return <>
    <SidebarSection className="environment-files" title="Files" count={files.length} total={material.files.length}><FileTree nodes={buildFileTree(files)} workspace={workspace} selection={selection} open={open} inspect={inspect} depth={0} /></SidebarSection>
    {material.policyOnlyCount > 0 && <p className="sidebar-empty">{material.policyOnlyCount} policy-only Source(s) in Authority.</p>}
    <SidebarSection title="Sources" count={sources.length} total={material.sources.length}>{sources.map((source) => <button key={source.id} className={selection === source.id ? "selected" : ""} onClick={() => open(sourceInput(workspace, source.id))} onDoubleClick={() => open(sourceInput(workspace, source.id, true))}><Icon name={isPolicySource(source) ? "authority" : environmentKindIcon(source.kind)} /><span><strong>{source.label}</strong><small>{humanize(source.kind)} · {source.posture ?? "declared"}</small></span></button>)}</SidebarSection>
    <SidebarSection title="Resources" count={resources.length} total={workspace.environment.resources.length}>{resources.map((resource) => <button key={resource.id} className={selection === resource.id ? "selected" : ""} onClick={() => open(resourceInput(workspace, resource.id))} onDoubleClick={() => open(resourceInput(workspace, resource.id, true))}><Icon name={environmentKindIcon(resource.kind)} /><span><strong>{resource.label ?? resourceLabel(resource.id)}</strong><small>{humanize(resource.kind)}</small></span></button>)}</SidebarSection>
  </>;
}

function FileTree({ nodes, workspace, selection, open, inspect, depth }: { nodes: FileTreeNode[]; workspace: CasePresentation; selection: string; open: (input: SurfaceInput) => void; inspect: (id: string) => void; depth: number }) {
  return <div className="file-tree">{nodes.map((node) => node.fileId ? <button key={node.path} style={{ paddingLeft: `${8 + depth * 14}px` }} className={selection === node.fileId ? "selected" : ""} title={node.path} onClick={() => open(fileInput(workspace, node.fileId!))} onDoubleClick={() => open(fileInput(workspace, node.fileId!, true))}><Icon name={fileIconForMedia(workspace.environment.files.find((file) => file.id === node.fileId)?.media_type, node.path)} /><span><strong>{node.name}</strong></span></button> : <details key={node.path} open><summary style={{ paddingLeft: `${6 + depth * 14}px` }}><Icon name="environment" /><strong>{node.name}</strong></summary><FileTree nodes={node.children} workspace={workspace} selection={selection} open={open} inspect={inspect} depth={depth + 1} /></details>)}</div>;
}

export function PerspectiveSurface({ workspace, input, actions, selection, platform }: SurfaceRendererProps) {
  const perspective = (input.viewId ?? "Overview") as Perspective;
  if (perspective === "Overview") return <Overview workspace={workspace} platform={platform} inspect={actions.inspect} navigate={actions.openPerspective} openWorkingState={() => actions.openSurface(memoryInput("workingState"))} />;
  if (perspective === "Environment") return <Environment platform={platform} workspace={workspace} inspect={actions.inspect} openSurface={actions.openSurface} />;
  if (perspective === "Knowledge") return <Knowledge platform={platform} actions={actions} key={workspace.case.case_ref} selected={selection} workspace={workspace} inspect={actions.inspect} openSurface={actions.openSurface} />;
  if (perspective === "Memory") return <Memory workspace={workspace} openSurface={actions.openSurface} />;
  if (perspective === "Authority") return <Authority platform={platform} selected={selection} workspace={workspace} inspect={actions.inspect} openSurface={actions.openSurface} />;
  if (perspective === "Work") return <WorkSurface key={workspace.case.case_ref} platform={platform} workspace={workspace} actions={actions} />;
  return <ComputeSurface key={workspace.case.case_ref} workspace={workspace} actions={actions} platform={platform} />;
}

export function TimelineSurface({ workspace, actions }: SurfaceRendererProps) {
  return <div className="live-page memory-page" data-surface-type="case.timeline"><SurfaceHeader workspace={workspace} title="Case Timeline" body="A temporal presentation of committed Case history. Chronology does not imply causality." /><ol className="real-timeline">{workspace.memory.timeline.slice().reverse().map((entry) => <li key={entry.id}><time>{formatTime(entry.committed_at_unix_ms)}</time><button onClick={() => actions.inspect(entry.id)}><i /><span><strong>{humanize(entry.kind)}</strong><small>{entry.component}</small></span></button></li>)}</ol>{!workspace.memory.timeline.length && <EmptyState title="No committed activity" body="No history is exposed for the current Case projection." />}</div>;
}

export function GraphSurface({ workspace, input, actions }: SurfaceRendererProps) {
  const projection = input.metadata?.projection;
  const graph = projection === "knowledge" ? knowledgeGraph(workspace) : relationGraph(workspace);
  return <div className="graph-surface" data-surface-type="case.graph"><header className="graph-surface-title"><strong>{input.title}</strong><span>{projection === "knowledge" ? "Qualified source relations" : "Temporal presentation · chronology does not imply causality"}</span></header>{graph.nodes.length ? <GraphViewport key={`${workspace.case.case_ref}:${projection}`} nodes={graph.nodes} edges={graph.edges} layout={projection === "knowledge" ? "relational" : "temporal"} onInspect={actions.inspect} /> : <EmptyState title={`No ${input.title}`} body="No qualified relation projection is available for this Case." />}</div>;
}

export function SourceSurface({ workspace, input, actions, platform }: SurfaceRendererProps) {
  const source = workspace.environment.sources.find((item) => item.id === input.objectRef);
  if (!source) return <EmptyState title="Source unavailable" body="This Source is not present in the current Case projection." />;
  const files = workspace.environment.files.filter((file) => file.source_ref === source.id);
  const resource = workspace.environment.resources.find((item) => item.id === source.resource_ref);
  return <article className="live-page object-surface source-surface" data-surface-type="environment.source"><SurfaceHeader workspace={workspace} title={source.label} body="A governed Case Source relationship. Its retained material is navigated separately." /><AcquireSourceAction workspace={workspace} platform={platform} sourceRef={source.id} /><RevokeSourceAction application={platform.application} workspace={workspace} sourceRef={source.id} refresh={() => platform.commands.executeCommand("studio.case.refresh").then(() => undefined)} /><PublishSourcePolicyAction application={platform.application} workspace={workspace} sourceRef={source.id} refresh={() => platform.commands.executeCommand("studio.case.refresh").then(() => undefined)} /><dl className="object-facts"><div><dt>Kind</dt><dd>{humanize(source.kind)}</dd></div><div><dt>Posture</dt><dd>{source.posture ?? "declared"}</dd></div><div><dt>Roles</dt><dd>{source.roles.join(", ")}</dd></div><div><dt>Perimeter</dt><dd>{source.perimeter}</dd></div><div><dt>Resource</dt><dd><button onClick={() => actions.openSurface(resourceInput(workspace, source.resource_ref))}>{resource?.label ?? resourceLabel(source.resource_ref)}</button></dd></div><div><dt>Revision</dt><dd>{source.revision_ref ?? "Not acquired"}</dd></div></dl><FactSection title="Retained material" empty="No acquired material">{files.map((file) => <button className="fact-row" key={file.id} onClick={() => actions.openSurface(fileInput(workspace, file.id))}><Icon name={fileIconForMedia(file.media_type, file.path)} /><span><strong>{fileName(file.path)}</strong><small>{file.path}</small></span></button>)}</FactSection><p className="surface-note">Knowledge may derive from this Source, but remains a rebuildable projection rather than the Source itself.</p></article>;
}

export function ResourceSurface({ workspace, input, actions, platform }: SurfaceRendererProps) {
  const resource = workspace.environment.resources.find((item) => item.id === input.objectRef);
  if (!resource) return <EmptyState title="Resource unavailable" body="This Resource is not present in the current Case projection." />;
  const sources = workspace.environment.sources.filter((source) => source.resource_ref === resource.id);
  return <article className="live-page object-surface resource-surface" data-surface-type="environment.resource"><SurfaceHeader workspace={workspace} title={resource.label ?? resourceLabel(resource.id)} body={`${humanize(resource.kind)} capability attached to this Case.`} /><DeclareSourceAction application={platform.application} workspace={workspace} resourceRef={resource.id} refresh={() => platform.commands.executeCommand("studio.case.refresh").then(() => undefined)} /><dl className="object-facts"><div><dt>Kind</dt><dd>{humanize(resource.kind)}</dd></div><div><dt>Review</dt><dd>{humanize(resource.review_requirement)}</dd></div><div><dt>Policy</dt><dd>{resource.policy_ref}</dd></div><div><dt>Operations</dt><dd>{resource.operations.length ? resource.operations.join(", ") : "No disclosed operations"}</dd></div>{resource.read_prefixes.length > 0 && <div><dt>Read scope</dt><dd>{resource.read_prefixes.join(", ")}</dd></div>}{resource.allowed_write_prefix && <div><dt>Write scope</dt><dd>{resource.allowed_write_prefix}</dd></div>}</dl><RequestResourceAction workspace={workspace} platform={platform} resourceRef={resource.id} /><ResourceRequestability workspace={workspace} resourceRef={resource.id} application={platform.application} /><FactSection title="Related Sources" empty="No related Sources">{sources.map((source) => <button className="fact-row" key={source.id} onClick={() => actions.openSurface(sourceInput(workspace, source.id))}><Icon name={environmentKindIcon(source.kind)} /><span><strong>{source.label}</strong><small>{humanize(source.kind)} · {source.posture ?? "declared"}</small></span></button>)}</FactSection></article>;
}

export function searchTimelineSurface({ workspace }: Pick<SurfaceRendererProps, "workspace">, _input: SurfaceRendererProps["input"], query: string): readonly SurfaceSearchResult[] {
  const needle = query.trim().toLocaleLowerCase();
  if (!needle) return [];
  return workspace.memory.timeline
    .filter((entry) => `${entry.kind} ${entry.component} ${entry.summary ?? ""} ${entry.sequence}`.toLocaleLowerCase().includes(needle))
    .map((entry) => ({ id: entry.id, label: humanize(entry.kind), detail: `${entry.component}`, objectRef: entry.id }));
}

export function searchGraphSurface({ workspace }: Pick<SurfaceRendererProps, "workspace">, input: SurfaceRendererProps["input"], query: string): readonly SurfaceSearchResult[] {
  const needle = query.trim().toLocaleLowerCase();
  if (!needle) return [];
  const graph = input.metadata?.projection === "knowledge" ? knowledgeGraph(workspace) : relationGraph(workspace);
  return graph.nodes
    .filter((node) => `${node.label} ${node.id} ${node.kind}`.toLocaleLowerCase().includes(needle))
    .map((node) => ({ id: node.id, label: node.label ?? node.id, detail: humanize(node.kind ?? "reference"), objectRef: node.id }));
}

export { ConversationView } from "./ConversationView";

export function ActivityView({ workspace, actions, selection }: AuxiliaryViewProps) {
  return <div className="context-scroll activity-view"><PanelHeader title="Activity" detail={`Latest ${Math.min(20, workspace.memory.timeline.length)} of ${workspace.memory.timeline.length} events`} />{workspace.memory.timeline.slice(-20).reverse().map((entry) => <button className={`context-event event-${activityTone(entry.kind)}${selection === entry.id ? " selected" : ""}`} key={entry.id} onClick={() => actions.inspect(entry.id)}><time>{formatTime(entry.committed_at_unix_ms)}</time><i /><span><strong>{humanize(entry.kind)}</strong><small>{entry.component}</small></span></button>)}{workspace.memory.timeline.length > 20 && <button className="ui-button activity-open-timeline" onClick={() => actions.openSurface(timelineInput(true))}>Open projected timeline</button>}{!workspace.memory.timeline.length && <EmptyState title="No committed activity" body="No history is exposed for the current Case projection." />}</div>;
}

export { InspectorView } from "./InspectorView";

function SurfaceHeader({ workspace, title, body }: { workspace: CasePresentation; title: string; body: string }) { return <header className="surface-title"><small title={workspace.case.case_ref}>{workspace.case.display_name}</small><h1>{title}</h1><p>{body}</p></header>; }

function Environment({ workspace, openSurface, platform }: { platform: PlatformServices; workspace: CasePresentation; inspect: (id: string) => void; openSurface: (input: SurfaceInput) => void }) { const material = environmentMaterials(workspace); return <div className="live-page"><SurfaceHeader workspace={workspace} title="Environment" body="The files available to this Case, the governed Sources that define its information perimeter, and the Resources through which it can operate." /><AttachResourceAction workspace={workspace} platform={platform} /><AttachProcessAction workspace={workspace} platform={platform} /><DeclareSourceAction application={platform.application} workspace={workspace} refresh={() => platform.commands.executeCommand("studio.case.refresh").then(() => undefined)} /><section className="environment-summary"><button onClick={() => material.files[0] && openSurface(fileInput(workspace, material.files[0].id))}><Icon name="file" size={22} /><strong>{material.files.length}</strong><span>Files</span></button><button onClick={() => material.sources[0] && openSurface(sourceInput(workspace, material.sources[0].id))}><Icon name="sources" size={22} /><strong>{material.sources.length}</strong><span>Sources</span></button><button onClick={() => workspace.environment.resources[0] && openSurface(resourceInput(workspace, workspace.environment.resources[0].id))}><Icon name="environment" size={22} /><strong>{workspace.environment.resources.length}</strong><span>Resources</span></button></section><FactSection title="Information perimeter" empty="No Sources attached">{material.sources.map((source) => <button className="fact-row" key={source.id} onClick={() => openSurface(sourceInput(workspace, source.id))}><Icon name={environmentKindIcon(source.kind)} /><span><strong>{source.label}</strong><small>{humanize(source.kind)} · {source.perimeter}</small></span><Badge>{source.posture ?? "declared"}</Badge></button>)}</FactSection></div>; }

function Memory({ workspace, openSurface }: { workspace: CasePresentation; openSurface: (input: SurfaceInput) => void }) { return <div className="live-page memory-page"><SurfaceHeader workspace={workspace} title="Memory" body="Committed history and derived relations are first-class Case surfaces." /><div className="surface-launch-grid"><button onClick={() => openSurface(memoryInput("recall"))}><Icon name="search" /><span><strong>Recall</strong><small>Qualified evidence for an explicit task and cut</small></span></button><button onClick={() => openSurface(memoryInput("workingState"))}><Icon name="memory" /><span><strong>Working State</strong><small>Current control and explicitly pageable evidence</small></span></button><button onClick={() => openSurface(timelineInput(true))}><Icon name="memory" size={22} /><span><strong>Case Timeline</strong><small>{workspace.memory.timeline.length} committed events</small></span></button><button onClick={() => openSurface(graphInput("memory", true))}><Icon name="graph" size={22} /><span><strong>Experience Graph</strong><small>{workspace.memory.relations.length} derived relations</small></span></button></div></div>; }



function FactSection({ title, empty, children }: { title: string; empty: string; children: React.ReactNode }) { const values = Array.isArray(children) ? children : [children]; return <section className="live-section"><PanelHeader title={title} />{values.length && values.some(Boolean) ? children : <EmptyState title={empty} body="Studio has not substituted authored product data." />}</section>; }

type Row = { id: string; label: string; detail: string; icon: IconName; material?: boolean; surface?: SurfaceInput };
function perspectiveRows(workspace: CasePresentation, perspective: Perspective): { label: string; items: Row[] }[] {
  // Environment owns a typed explorer rather than flattening Files, Sources and
  // Resources into the generic material-row path.
  if (perspective === "Environment") return [];
  if (perspective === "Knowledge") return [{ label: "Views", items: [{ id: "knowledge-graph", label: "Knowledge Graph", detail: `${workspace.knowledge.relations.length} relations`, icon: "graph", surface: graphInput("knowledge") }] }, { label: "Documents", items: workspace.knowledge.sources.map((item) => ({ id: item.id, label: item.label, detail: `${item.media_type} · ${item.path}`, icon: fileIconForMedia(item.media_type, item.path), material: true })) }, { label: "Entities", items: workspace.knowledge.entities.map((item) => ({ id: item.id, label: compact(item.id), detail: `${item.definitions.length} definitions`, icon: "knowledge" })) }];
  if (perspective === "Memory") return [{ label: "Views", items: [{ id: "memory-recall", label: "Recall", detail: "Task-qualified evidence", icon: "search", surface: memoryInput("recall") }, { id: "memory-working", label: "Working State", detail: "Current control and evidence", icon: "memory", surface: memoryInput("workingState") }, { id: "memory-timeline", label: "Case Timeline", detail: `${workspace.memory.timeline.length} events`, icon: "memory", surface: timelineInput() }, { id: "memory-graph", label: "Experience Graph", detail: `${workspace.memory.relations.length} relations`, icon: "graph", surface: graphInput("memory") }] }, { label: "History", items: workspace.memory.timeline.slice(-12).reverse().map((item) => ({ id: item.id, label: humanize(item.kind), detail: `State version ${item.sequence}`, icon: "memory" })) }];
  if (perspective === "Authority") return [{ label: "Policy Sources", items: workspace.environment.sources.filter(isPolicySource).map(source => ({ id: source.id, label: source.label, detail: source.roles.join(" · "), icon: "authority", surface: sourceInput(workspace, source.id) })) }, { label: "Policies", items: workspace.authority.policies.map((item) => ({ id: item.id, label: humanize(item.policy_key), detail: `version ${item.version} · bound`, icon: "authority" })) }, { label: "Reviews", items: workspace.authority.reviews.map((item) => ({ id: item.id, label: compact(item.id), detail: item.status, icon: "review" })) }];
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
