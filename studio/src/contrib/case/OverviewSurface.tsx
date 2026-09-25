import "./OverviewSurface.css";
import { OverviewNarrative } from "./OverviewNarrative";
import type { PlatformServices } from "../../platform/services";
import type { CasePresentation } from "../../clients/dataSource";
import { Icon, type IconName } from "../../components/Icon";
import { Badge, Button, EmptyState, PanelHeader } from "../../components/primitives";

export function OverviewSurface({ workspace: w, inspect, navigate, platform }: { platform?: PlatformServices; workspace: CasePresentation; inspect: (id: string) => void; navigate: (id: string) => void }) {
  const cards: { title: string; icon: IconName; value: string; detail: string; domain: string }[] = [
    { title: "Environment", icon: "environment", value: `${w.environment.files.length} files`, detail: `${w.environment.sources.length} Sources · ${w.environment.resources.length} Resources`, domain: "environment" },
    { title: "Knowledge", icon: "knowledge", value: `${w.knowledge.units.length} units`, detail: `${w.knowledge.sources.length} documents · ${w.knowledge.relations.length} relations`, domain: "knowledge" },
    { title: "Memory", icon: "memory", value: `${w.memory.timeline.length} events`, detail: `${w.memory.relations.length} derived relations`, domain: "memory" },
    { title: "Authority", icon: "authority", value: `${w.authority.policies.length} policies`, detail: `${w.authority.reviews.length} reviews · ${w.authority.grants.length} grants`, domain: "authority" },
    { title: "Work", icon: "work", value: `${w.work.nodes.length} nodes`, detail: w.work.status.replaceAll("_", " "), domain: "work" },
    { title: "Compute", icon: "compute", value: `${w.compute.targets.length} targets`, detail: w.compute.targets.length ? "Governed inference deployments" : "No inference target bound", domain: "compute" },
  ];
  const acquired = w.environment.sources.filter(source => source.posture === "acquired").length;
  const completed = w.work.nodes.filter(node => node.posture === "satisfied").length;
  const events = [...w.memory.timeline].sort((a, b) => b.sequence - a.sequence).slice(0, 6);
  const updated = w.case.updated_at_unix_ms;
  const time = (value: number) => new Date(value).toLocaleString();
  return <div className="live-page case-overview">
    <header className="surface-title"><small>Case overview</small><h1>{w.case.display_name}</h1><div className="overview-identity"><Badge tone={w.case.case_status === "open" ? "success" : "neutral"}>{w.case.case_status}</Badge><span>{w.overview.participants.length} participants</span><Button onClick={() => inspect(w.case.case_ref)}>Case details</Button></div></header>
    <nav className="overview-sections" aria-label="Overview sections"><a href="#overview-state">Situation</a><a href="#overview-work">Workflow</a><a href="#overview-world">Sources</a><a href="#overview-history">Recent changes</a></nav>
    <section id="overview-state" className="overview-state" aria-label="Current Case situation">
      <header><h2>Current situation</h2>{updated ? <time dateTime={new Date(updated).toISOString()}>Case updated {time(updated)}</time> : <small>Last update time not projected</small>}</header>
      <dl className="overview-signals">
        <div><dt>Source acquisition</dt><dd>{acquired} / {w.environment.sources.length} acquired</dd></div>
        <div><dt>Workflow</dt><dd>{completed} / {w.work.nodes.length} satisfied</dd></div>
        <div><dt>Review states</dt><dd>{w.authority.reviews.length ? [...new Set(w.authority.reviews.map(review => review.status))].join(" · ") : "No projected reviews"}</dd></div>
        <div><dt>Inference</dt><dd>{w.compute.targets.length ? `${w.compute.targets.length} bound targets` : "No target bound"}</dd></div>
      </dl>
      <p className="surface-note">Current projected facts. Resource attachment is not live availability; a model binding is not proof of readiness.</p>
      {w.overview.attention.map((item, index) => <button className="overview-attention-row" key={`${item.ref ?? item.kind}:${index}`} onClick={() => item.ref ? inspect(item.ref) : navigate(item.kind === "provider" ? "Compute" : item.kind === "review" ? "Authority" : "Work")}><Icon name="warning" size={16}/><span><strong>{item.title}</strong><small>{item.detail}</small></span><Icon name="chevron" size={13}/></button>)}
    </section>
    {platform && <OverviewNarrative workspace={w} platform={platform} inspect={inspect} configure={() => navigate("Compute")}/>}
    <section id="overview-work" className="overview-story" aria-label="Case story">
      <PanelHeader title={w.work.definition?.name || "The work in this Case"} detail="Current Workflow" />
      {w.work.definition?.description && <p>{w.work.definition.description}</p>}
      {w.work.nodes.length ? <><p>Follow the work below to understand what each checkpoint requires. Status and explanations come from YAI; a listed checkpoint is not proof that its work has been completed.</p><ol className="overview-story-steps">{w.work.nodes.map(node => {
        const definition = w.work.effective_nodes ? w.work.effective_nodes.find(item => item.node_id === node.node_id)?.node : w.work.definition?.nodes?.find((item): item is { node_id: string; prompt?: string } => Boolean(item && typeof item === "object" && "node_id" in item && item.node_id === node.node_id));
        return <li key={node.node_id}><button onClick={() => inspect(node.node_id)}><span><strong>{typeof definition?.prompt === "string" ? definition.prompt : node.node_id}</strong>{node.reason && <small>{node.reason.replaceAll("_", " ")}</small>}</span><Badge tone="neutral">{node.posture.replaceAll("_", " ")}</Badge></button></li>;
      })}</ol><Button onClick={() => { platform?.context.update(`studio.work.section:${JSON.stringify([w.case.case_ref, w.case.participant_ref])}`, "Workflow"); navigate("Work"); }}>Open workflow and dependencies</Button></> : <p>No qualified Workflow is available yet. Define the operational work in Work; Studio will show its checkpoints here.</p>}
      <div id="overview-world" className="overview-story-evidence"><h2>Supporting material</h2><p>These are the Sources attached to this Case. Their presence does not establish that their contents have been verified.</p>{w.environment.sources.map(source => <button key={source.id} onClick={() => inspect(source.id)}><Icon name="environment" size={14} /><span>{source.label}</span><small>{source.kind.replaceAll("_", " ")} · {source.posture?.replaceAll("_", " ") ?? "Posture not projected"}</small></button>)}</div>
      {!w.compute.targets.length && <p className="surface-note">Model explanation is unavailable: this Case has no bound inference target. The Workflow and Sources above remain available without a model.</p>}
    </section>
    <nav className="overview-destinations" aria-label="Case perspectives">{cards.map(card => <button key={card.title} data-domain={card.domain} onClick={() => navigate(card.title)}><Icon name={card.icon} size={20} /><span><strong>{card.title}</strong><small>{card.detail}</small></span><b>{card.value}</b><Icon name="chevron" size={13} /></button>)}</nav>
    <section id="overview-history" className="overview-history" aria-label="Recent Case changes"><header><h2>Recent changes</h2><Button onClick={() => navigate("Memory")}>Open Memory</Button></header><p className="surface-note">Latest {events.length} events from the projected history. This is committed activity, not a live execution queue.</p>{events.map(event => <button key={event.id} onClick={() => inspect(event.id)}><Icon name="memory" size={15}/><span><strong>{event.summary || event.kind.replaceAll("_", " ")}</strong><small>{event.component}{event.participant_ref ? ` · ${event.participant_ref}` : ""}</small></span><time dateTime={new Date(event.committed_at_unix_ms).toISOString()}>{time(event.committed_at_unix_ms)}</time></button>)}{!events.length && <EmptyState title="No history projected" body="No committed events are included in the current projection."/>}</section>
  </div>;
}
