import { useState } from "react";
import type { CasePresentation } from "../../clients/dataSource";
import type { SurfaceInput } from "../../workbench/surface/model";
import { CollectionList, type CollectionItem } from "../../components/CollectionList";
import { Button, EmptyState } from "../../components/primitives";
import { Icon } from "../../components/Icon";
import { fileIconForMedia, graphInput, materialInput } from "../surfaces/inputs";
import { textLabel } from "./graph";

export function KnowledgeSurface({ workspace, inspect, openSurface, selected }: {
  workspace: CasePresentation; selected?: string; inspect: (id: string) => void; openSurface: (input: SurfaceInput) => void;
}) {
  const [category, setCategory] = useState("Documents");
  const [topic, setTopic] = useState<string>();
  const knowledge = workspace.knowledge;
  const sections: Record<string, CollectionItem[]> = {
    Documents: knowledge.sources.map(item => ({ id: item.id, title: item.path || item.label, detail: item.label, icon: fileIconForMedia(item.media_type, item.path), meta: item.status })),
    Units: knowledge.units.map(item => ({ id: item.id, title: textLabel(item.text, item.kind), detail: knowledge.sources.find(source => source.id === item.source_ref)?.path, icon: "knowledge", meta: item.kind.replaceAll("_", " "), searchText: `${item.id} ${item.text}` })),
    Entities: knowledge.entities.map(item => ({ id: item.id, title: item.id, meta: `${item.definitions.length} definitions`, icon: "knowledge" })),
    Topics: knowledge.topics.map(item => ({ id: item.name, title: item.name, meta: `${item.units.length} units`, icon: "knowledge" })),
    Contradictions: knowledge.contradictions.map(item => ({ id: item.id, title: `${item.entity} · ${item.predicate}`, meta: item.posture, detail: `${item.members.length} members`, icon: "review" })),
  };
  const topicUnits = topic ? new Set(knowledge.topics.find(item => item.name === topic)?.units ?? []) : undefined;
  const items = topicUnits ? sections.Units.filter(item => topicUnits.has(item.id)) : sections[category];
  const select = (id: string) => {
    if (topic) { inspect(id); return; }
    if (category === "Topics") { setTopic(id); return; }
    if (category === "Documents") {
      const document = knowledge.sources.find(item => item.id === id)!;
      openSurface(materialInput(workspace, id, document.path || document.label));
    } else inspect(id);
  };
  return <div className="live-page knowledge-page">
    <header className="surface-title"><small>{workspace.case.display_name}</small><h1>Knowledge</h1><p>Documents, extracted units and relationships grounded in acquired Sources.</p></header>
    <section className="surface-launch"><Icon name="graph" size={22} /><span><strong>Knowledge Graph</strong><small>{knowledge.relations.length} qualified relations · {knowledge.units.length} units</small></span><Button onClick={() => openSurface(graphInput("knowledge", true))}>Open graph</Button></section>
    {knowledge.status === "empty" ? <EmptyState title="No qualified Knowledge derivation" body={knowledge.message} /> : <>
      <div className="collection-categories" role="group" aria-label="Knowledge categories">{Object.entries(sections).map(([name, rows]) => <button key={name} aria-pressed={category === name} onClick={() => { setCategory(name); setTopic(undefined); }}>{name}<span>{rows.length}</span></button>)}</div>
      {topic && <div className="collection-topic"><Button onClick={() => setTopic(undefined)}>All topics</Button><strong>{topic}</strong></div>}
      <CollectionList key={`${category}:${topic ?? ""}`} label={topic ? "Topic units" : category} items={items} selected={selected} onSelect={select} />
      <p className="surface-note">Derived Knowledge is rebuildable. Counts describe this qualified projection; they do not imply inferred claims or entities.</p>
    </>}
  </div>;
}
