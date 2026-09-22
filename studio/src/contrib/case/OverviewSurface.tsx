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
    <header className="surface-title"><small>Case overview</small><h1>{w.case.display_name}</h1><div className="overview-identity"><Badge tone={w.case.case_status === "open" ? "success" : "neutral"}>{w.case.case_status}</Badge><span>Generation {w.case.generation}</span><span>{w.overview.participants.length} participants</span><Button onClick={() => inspect(w.case.case_ref)}>Case details</Button></div></header>
    <nav className="overview-destinations" aria-label="Case perspectives">{cards.map(card => <button key={card.title} data-domain={card.domain} onClick={() => navigate(card.title)}><Icon name={card.icon} size={20} /><span><strong>{card.title}</strong><small>{card.detail}</small></span><b>{card.value}</b><Icon name="chevron" size={13} /></button>)}</nav>
    <section className="live-section overview-attention"><PanelHeader title="Attention" detail="Current Case facts" />{w.overview.attention.map((item, i) => <button className="fact-row" key={`${item.ref ?? item.kind}:${i}`} onClick={() => item.ref ? inspect(item.ref) : navigate(item.kind === "provider" ? "Compute" : item.kind === "review" ? "Authority" : "Work")}><Icon name={item.kind === "review" ? "review" : item.kind === "provider" ? "compute" : "work"} /><span><strong>{item.title}</strong><small>{item.detail}</small></span><Icon name="chevron" size={13} /></button>)}{!w.overview.attention.length && <EmptyState title="Nothing needs attention" body="No pending attention is included in this Case projection." />}</section>
  </div>;
}
