import { ReviewActions } from "./applicationActions";
import type { AuxiliaryViewProps } from "../../workbench/kernel/types";
import { Icon } from "../../components/Icon";
import { Button, PanelHeader } from "../../components/primitives";
import { CollectionList } from "../../components/CollectionList";
import { fileInput, materialInput, resourceInput, sourceInput } from "../surfaces/inputs";
import { factIcon, factKind, factReferences, findFact } from "./facts";

export function InspectorView({ workspace, selection, actions, platform }: AuxiliaryViewProps) {
  if (selection.startsWith("settings:")) return <div className="context-scroll inspector-view"><PanelHeader title="Application preferences" /><p className="surface-note">These preferences belong to Studio. Case objects remain available in the Explorer.</p></div>;
  const fact = findFact(workspace, selection);
  const kind = factKind(workspace, selection);
  const allRelations = [...workspace.memory.relations, ...workspace.knowledge.relations, ...workspace.work.edges];
  const relations = [...new Map(allRelations.filter(edge => edge.from === selection || edge.to === selection).map(edge => [edge.id, edge])).values()];
  const references = factReferences(workspace, selection);
  const file = workspace.environment.files.find(item => item.id === selection);
  const fixture = workspace.presentation.materials?.find(item => item.id === selection);
  return <div className="context-scroll inspector" data-inspected-ref={selection}>
    <PanelHeader title="Inspector" detail="Selected object" />
    <section className={`inspector-hero ${kind.startsWith("knowledge") ? "domain-knowledge" : ""}`}>
      <div><Icon name={factIcon(kind)} size={18} /></div><span>{kind.replace(/^./, char => char.toUpperCase())}</span><h2>{fact.title}</h2>
      {kind !== "knowledge unit" && <p>{fact.detail}</p>}
      {kind === "knowledge unit" && <div className="inspector-excerpt">{fact.detail}</div>}
      {(file || fixture) && <Button onClick={() => actions.openSurface(file ? fileInput(workspace, selection, true) : materialInput(workspace, selection, fixture!.name, true))}>Open file</Button>}
      {kind === "knowledge document" && <Button onClick={() => actions.openSurface(materialInput(workspace, selection, fact.title, true))}>Open material</Button>}
      {kind === "source" && <Button onClick={() => actions.openSurface(sourceInput(workspace, selection, true))}>Open Source</Button>}
      {kind === "resource" && <Button onClick={() => actions.openSurface(resourceInput(workspace, selection, true))}>Open Resource</Button>}
    </section>
    {kind === "review" && <ReviewActions key={selection} application={platform.application} workspace={workspace} reviewRef={selection} refresh={() => { void platform.commands.executeCommand("studio.case.refresh"); }} />}
    {!!references.length && <section className="inspector-group"><h3>Referenced objects <span>{references.length}</span></h3>{references.map(id => {
      const related = findFact(workspace, id); const relatedKind = factKind(workspace, id);
      return relatedKind === "case fact" ? <p key={id} title={id}>{id}<small> · Detail not projected</small></p> : <button key={id} onClick={() => actions.inspect(id)}><Icon name={factIcon(relatedKind)} /><strong>{related.title}</strong><small>{relatedKind}</small></button>;
    })}</section>}
    <details className="inspector-technical"><summary>Technical details</summary><dl>{fact.values.map(([label, value]) => <div key={label}><dt>{label}</dt><dd>{value}</dd></div>)}</dl></details>
    {!!relations.length && <section className="inspector-group"><h3>Qualified relationships <span>{relations.length}</span></h3><CollectionList key={selection} label="Relationships" pageSize={12} items={relations.map(edge => {
      const outgoing = edge.from === selection;
      const related = outgoing ? edge.to : edge.from;
      return { id: edge.id, title: findFact(workspace, related).title, detail: `${outgoing ? "→ Outgoing" : "← Incoming"} · ${edge.kind.replaceAll("_", " ")}`, icon: factIcon(factKind(workspace, related)), searchText: related };
    })} onSelect={id => { const edge = relations.find(item => item.id === id)!; actions.inspect(edge.from === selection ? edge.to : edge.from); }} /></section>}
  </div>;
}
