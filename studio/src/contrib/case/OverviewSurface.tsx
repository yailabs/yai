import "./OverviewSurface.css";
import { useCallback, useId, useState, useSyncExternalStore } from "react";
import { OverviewNarrative } from "./OverviewNarrative";
import type { PlatformServices } from "../../platform/services";
import type { CasePresentation } from "../../clients/dataSource";
import { Icon } from "../../components/Icon";
import { Badge, Button, EmptyState, PanelHeader } from "../../components/primitives";

export function OverviewSurface({ workspace: w, inspect, navigate, openWorkingState, platform }: { platform?: PlatformServices; workspace: CasePresentation; inspect: (id: string) => void; navigate: (id: string) => void; openWorkingState: () => void }) {
  const identity = useId();
  const sections = ["Situation", "Workflow", "Sources", "Recent changes"] as const;
  type Section = typeof sections[number];
  const sectionKey = `studio.overview.section:${JSON.stringify([w.case.case_ref, w.case.participant_ref])}`;
  const [fallback, setFallback] = useState<Section>("Situation");
  const subscribe = useCallback((listener: () => void) => platform?.context.subscribe(listener).dispose ?? (() => {}), [platform]);
  const section = useSyncExternalStore(subscribe, () => {
    const value = (platform ? platform.context.get(sectionKey) : fallback) as Section;
    return sections.includes(value) ? value : "Situation";
  });
  const select = (value: Section) => platform ? platform.context.update(sectionKey, value) : setFallback(value);
  const panel = (value: Section) => ({ role: "tabpanel" as const, id: `${identity}-${value.replaceAll(" ", "-")}-panel`, "aria-labelledby": `${identity}-${value.replaceAll(" ", "-")}-tab`, hidden: section !== value, tabIndex: 0 });
  const acquired = w.environment.sources.filter(source => source.posture === "acquired").length;
  const completed = w.work.nodes.filter(node => node.posture === "satisfied").length;
  const events = [...w.memory.timeline].sort((a, b) => b.sequence - a.sequence).slice(0, 6);
  const updated = w.case.updated_at_unix_ms;
  const time = (value: number) => new Date(value).toLocaleString();
  return <div className="case-overview">
    <header className="surface-title"><div><small>Case overview</small><h1>{w.case.display_name}</h1></div><div className="overview-identity"><Badge tone={w.case.case_status === "open" ? "success" : "neutral"}>{w.case.case_status}</Badge><span>{w.overview.participants.length} participants</span><Button onClick={() => inspect(w.case.case_ref)}>Case details</Button></div></header>
    <nav className="overview-sections" role="tablist" aria-label="Overview sections" onKeyDown={event => {
      const index = sections.indexOf(section);
      const next = event.key === "ArrowRight" ? (index + 1) % sections.length : event.key === "ArrowLeft" ? (index + sections.length - 1) % sections.length : event.key === "Home" ? 0 : event.key === "End" ? sections.length - 1 : undefined;
      if (next === undefined) return;
      event.preventDefault(); select(sections[next]);
      event.currentTarget.querySelector<HTMLButtonElement>(`[data-overview-section="${sections[next]}"]`)?.focus();
    }}>{sections.map(value => <button key={value} role="tab" id={`${identity}-${value.replaceAll(" ", "-")}-tab`} data-overview-section={value} aria-controls={`${identity}-${value.replaceAll(" ", "-")}-panel`} aria-selected={section === value} tabIndex={section === value ? 0 : -1} onClick={() => select(value)}>{value}</button>)}</nav>
    <div className="overview-content">
    <section {...panel("Situation")}>
    <section id="overview-state" className="overview-state" aria-label="Current Case situation">
      <header><h2>Current situation</h2>{updated ? <time dateTime={new Date(updated).toISOString()}>Case updated {time(updated)}</time> : <small>Last update time not projected</small>}</header>
      <dl className="overview-signals">
        <div><dt>Source acquisition</dt><dd>{acquired} / {w.environment.sources.length} acquired</dd></div>
        <div><dt>Workflow</dt><dd>{completed} / {w.work.nodes.length} satisfied</dd></div>
        <div><dt>Review states</dt><dd>{w.authority.reviews.length ? [...new Set(w.authority.reviews.map(review => review.status))].join(" · ") : "No projected reviews"}</dd></div>
        <div><dt>Inference</dt><dd>{w.compute.targets.length ? `${w.compute.targets.length} bound targets` : "No target bound"}</dd></div>
      </dl>
      <p className="surface-note">Current projected facts. Resource attachment is not live availability; a model binding is not proof of readiness.</p>
    </section>
    {platform && <OverviewNarrative workspace={w} platform={platform} inspect={inspect} configure={() => navigate("Compute")} openWorkingState={openWorkingState}/>}
    {w.overview.attention.length > 0 && <section className="overview-attention" aria-label="Needs attention"><h2>Needs attention</h2>
      {w.overview.attention.map((item, index) => <button className="overview-attention-row" key={`${item.ref ?? item.kind}:${index}`} onClick={() => item.ref ? inspect(item.ref) : navigate(item.kind === "provider" ? "Compute" : item.kind === "review" ? "Authority" : "Work")}><Icon name="warning" size={16}/><span><strong>{item.title}</strong><small>{item.detail}</small></span><Icon name="chevron" size={13}/></button>)}
    </section>}
    </section>
    <section {...panel("Workflow")}>
    <section id="overview-work" className="overview-story" aria-label="Case story">
      <PanelHeader title={w.work.definition?.name || "The work in this Case"} detail="Current Workflow" />
      {w.work.definition?.description && <p>{w.work.definition.description}</p>}
      {w.work.nodes.length ? <><p>Follow the work below to understand what each checkpoint requires. Status and explanations come from YAI; a listed checkpoint is not proof that its work has been completed.</p><ol className="overview-story-steps">{w.work.nodes.map(node => {
        const definition = w.work.effective_nodes ? w.work.effective_nodes.find(item => item.node_id === node.node_id)?.node : w.work.definition?.nodes?.find((item): item is { node_id: string; prompt?: string } => Boolean(item && typeof item === "object" && "node_id" in item && item.node_id === node.node_id));
        return <li key={node.node_id}><button onClick={() => inspect(node.node_id)}><span><strong>{typeof definition?.prompt === "string" ? definition.prompt : node.node_id}</strong>{node.reason && <small>{node.reason.replaceAll("_", " ")}</small>}</span><Badge tone="neutral">{node.posture.replaceAll("_", " ")}</Badge></button></li>;
      })}</ol><Button onClick={() => { platform?.context.update(`studio.work.section:${JSON.stringify([w.case.case_ref, w.case.participant_ref])}`, "Workflow"); navigate("Work"); }}>Open workflow and dependencies</Button></> : <p>No qualified Workflow is available yet. Define the operational work in Work; Studio will show its checkpoints here.</p>}
    </section>
    </section>
    <section {...panel("Sources")}>
      <div id="overview-world" className="overview-story-evidence"><h2>Supporting material</h2><p>These are the Sources attached to this Case. Their presence does not establish that their contents have been verified.</p>{w.environment.sources.map(source => <button key={source.id} onClick={() => inspect(source.id)}><Icon name="environment" size={14} /><span>{source.label}</span><small>{source.kind.replaceAll("_", " ")} · {source.posture?.replaceAll("_", " ") ?? "Posture not projected"}</small></button>)}</div>
      {!w.compute.targets.length && <p className="surface-note">Model explanation is unavailable: this Case has no bound inference target. The Workflow and Sources remain available without a model.</p>}
    </section>
    <section {...panel("Recent changes")}>
    <section id="overview-history" className="overview-history" aria-label="Recent Case changes"><header><h2>Recent changes</h2><Button onClick={() => navigate("Memory")}>Open Memory</Button></header><p className="surface-note">Latest {events.length} events from the projected history. This is committed activity, not a live execution queue.</p>{events.map(event => <button key={event.id} onClick={() => inspect(event.id)}><Icon name="memory" size={15}/><span><strong>{event.summary || event.kind.replaceAll("_", " ")}</strong><small>{event.component}{event.participant_ref ? ` · ${event.participant_ref}` : ""}</small></span><time dateTime={new Date(event.committed_at_unix_ms).toISOString()}>{time(event.committed_at_unix_ms)}</time></button>)}{!events.length && <EmptyState title="No history projected" body="No committed events are included in the current projection."/>}</section>
    </section>
    </div>
  </div>;
}
