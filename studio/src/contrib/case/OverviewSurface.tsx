import "./OverviewSurface.css";
import type { CasePresentation } from "../../clients/dataSource";
import { Icon, type IconName } from "../../components/Icon";
import { Badge, Button, EmptyState, PanelHeader } from "../../components/primitives";

export function OverviewSurface({ workspace: w, inspect, navigate }: { workspace: CasePresentation; inspect: (id: string) => void; navigate: (id: string) => void }) {
  const cards: { title: string; icon: IconName; value: string; detail: string; domain: string }[] = [
    { title: "Environment", icon: "environment", value: `${w.environment.files.length} files`, detail: `${w.environment.sources.length} Sources · ${w.environment.resources.length} Resources`, domain: "environment" },
    { title: "Knowledge", icon: "knowledge", value: `${w.knowledge.units.length} units`, detail: `${w.knowledge.sources.length} documents · ${w.knowledge.relations.length} relations`, domain: "knowledge" },
    { title: "Memory", icon: "memory", value: `${w.memory.timeline.length} events`, detail: `${w.memory.relations.length} derived relations`, domain: "memory" },
    { title: "Authority", icon: "authority", value: `${w.authority.policies.length} policies`, detail: `${w.authority.reviews.length} reviews · ${w.authority.grants.length} grants`, domain: "authority" },
    { title: "Work", icon: "work", value: `${w.work.nodes.length} nodes`, detail: w.work.status.replaceAll("_", " "), domain: "work" },
    { title: "Compute", icon: "compute", value: `${w.compute.targets.length} targets`, detail: w.compute.targets.length ? "Governed inference deployments" : "No inference target bound", domain: "compute" },
  ];
  return <div className="live-page case-overview">
    <header className="surface-title"><small>Case overview</small><h1>{w.case.display_name}</h1><div className="overview-identity"><Badge tone={w.case.case_status === "open" ? "success" : "neutral"}>{w.case.case_status}</Badge><span>{w.overview.participants.length} participants</span><Button onClick={() => inspect(w.case.case_ref)}>Case details</Button></div></header>
    <section className="overview-story" aria-label="Case story">
      <PanelHeader title={w.work.definition?.name || "The work in this Case"} detail="Current Workflow" />
      {w.work.definition?.description && <p>{w.work.definition.description}</p>}
      {w.work.nodes.length ? <><p>Follow the work below to understand what each checkpoint requires. Status and explanations come from YAI; a listed checkpoint is not proof that its work has been completed.</p><ol className="overview-story-steps">{w.work.nodes.map(node => {
        const definition = w.work.definition?.nodes?.find((item): item is { node_id: string; prompt?: string } => Boolean(item && typeof item === "object" && "node_id" in item && item.node_id === node.node_id));
        return <li key={node.node_id}><button onClick={() => inspect(node.node_id)}><span><strong>{typeof definition?.prompt === "string" ? definition.prompt : node.node_id}</strong>{node.reason && <small>{node.reason.replaceAll("_", " ")}</small>}</span><Badge tone="neutral">{node.posture.replaceAll("_", " ")}</Badge></button></li>;
      })}</ol><Button onClick={() => navigate("Work")}>Open workflow and dependencies</Button></> : <p>No qualified Workflow is available yet. Define the operational work in Work; Studio will show its checkpoints here.</p>}
      <div className="overview-story-evidence"><h2>Supporting material</h2><p>These are the Sources attached to this Case. Their presence does not establish that their contents have been verified.</p>{w.environment.sources.map(source => <button key={source.id} onClick={() => inspect(source.id)}><Icon name="environment" size={14} /><span>{source.label}</span><small>{source.roles.join(" · ")}</small></button>)}</div>
      {!w.compute.targets.length && <p className="surface-note">Model explanation is unavailable: this Case has no bound inference target. The Workflow and Sources above remain available without a model.</p>}
    </section>
    <nav className="overview-destinations" aria-label="Case perspectives">{cards.map(card => <button key={card.title} data-domain={card.domain} onClick={() => navigate(card.title)}><Icon name={card.icon} size={20} /><span><strong>{card.title}</strong><small>{card.detail}</small></span><b>{card.value}</b><Icon name="chevron" size={13} /></button>)}</nav>
    <section className="live-section overview-attention"><PanelHeader title="Attention" detail="Current Case facts" />{w.overview.attention.map((item, i) => <button className="fact-row" key={`${item.ref ?? item.kind}:${i}`} onClick={() => item.ref ? inspect(item.ref) : navigate(item.kind === "provider" ? "Compute" : item.kind === "review" ? "Authority" : "Work")}><Icon name={item.kind === "review" ? "review" : item.kind === "provider" ? "compute" : "work"} /><span><strong>{item.title}</strong><small>{item.detail}</small></span><Icon name="chevron" size={13} /></button>)}{!w.overview.attention.length && <EmptyState title="Nothing needs attention" body="No pending attention is included in this Case projection." />}</section>
  </div>;
}
