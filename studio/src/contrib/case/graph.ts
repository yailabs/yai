import type { LiveEdge, LiveNode, LiveWorkspace } from "../../clients/live";

export interface GraphNode extends LiveNode { referenceOnly?: boolean }
export interface CaseGraph { nodes: GraphNode[]; edges: LiveEdge[] }
export const textLabel = (text: string, fallback: string) => text.trim().split("\n").find(Boolean)?.slice(0, 90) || fallback;

/** Presentation index only: an endpoint is never dropped or promoted to a new Case object. */
export function completeGraph(nodes: GraphNode[], edges: LiveEdge[]): CaseGraph {
  const index = new Map(nodes.map(node => [node.id, node]));
  for (const edge of edges) {
    for (const [id, kind] of [[edge.from, edge.from_kind], [edge.to, edge.to_kind]]) {
      if (id && !index.has(id)) index.set(id, { id, label: id, kind: kind ?? "reference", referenceOnly: true, detail: "Referenced by a qualified relation; object detail is not projected." });
    }
  }
  return { nodes: [...index.values()], edges };
}

export function knowledgeGraph(workspace: LiveWorkspace): CaseGraph {
  const k = workspace.knowledge;
  const candidates: GraphNode[] = [
    ...k.sources.map(item => ({ id: item.id, label: item.path || item.label, kind: "document", detail: item.status })),
    ...k.units.map(item => ({ id: item.id, label: textLabel(item.text, item.kind), kind: item.kind, detail: item.text })),
    ...k.entities.map(item => ({ id: item.id, label: item.id, kind: "entity" })),
    ...k.contradictions.map(item => ({ id: item.id, label: `${item.entity}: ${item.predicate}`, kind: "contradiction" })),
    ...workspace.environment.resources.map(item => ({ id: item.id, label: item.label ?? item.id, kind: "resource" })),
    ...workspace.environment.sources.map(item => ({ id: item.id, label: item.label, kind: "source" })),
    ...workspace.environment.files.map(item => ({ id: item.id, label: item.path, kind: "file" })),
  ];
  const referenced = new Set(k.relations.flatMap(edge => [edge.from, edge.to]));
  const knowledge = new Set([...k.units, ...k.sources, ...k.entities, ...k.contradictions].map(item => item.id));
  return completeGraph(candidates.filter(item => knowledge.has(item.id) || referenced.has(item.id)), k.relations);
}

export function relationGraph(workspace: LiveWorkspace): CaseGraph {
  const events = workspace.memory.timeline.map(item => ({ id: item.id, label: item.kind.replaceAll("_", " "), kind: "event" }));
  const refs = new Set(workspace.memory.relations.flatMap(edge => [edge.from, edge.to]));
  return completeGraph(events.filter(item => refs.has(item.id)), workspace.memory.relations);
}

export function graphSlice(graph: CaseGraph, query: string, focus: string | undefined, page = 0, pageSize = 20) {
  const neighbors = new Set(focus ? [focus] : []);
  if (focus) for (const edge of graph.edges) {
    if (edge.from === focus) neighbors.add(edge.to);
    if (edge.to === focus) neighbors.add(edge.from);
  }
  const terms = query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
  const matches = graph.nodes.filter(node => (!focus || neighbors.has(node.id)) && terms.every(term => `${node.label} ${node.kind} ${node.detail ?? ""} ${node.id}`.toLocaleLowerCase().includes(term)));
  // Keep the focused object visible across pages of its neighborhood.
  const anchor = focus ? matches.find(node => node.id === focus) : undefined;
  const rest = anchor ? matches.filter(node => node !== anchor) : matches;
  const size = pageSize - (anchor ? 1 : 0);
  const pages = Math.max(1, Math.ceil(rest.length / size));
  const current = Math.min(Math.max(0, page), pages - 1);
  const nodes = [...(anchor ? [anchor] : []), ...rest.slice(current * size, (current + 1) * size)];
  const visible = new Set(nodes.map(node => node.id));
  return { nodes, edges: graph.edges.filter(edge => visible.has(edge.from) && visible.has(edge.to)), matches: matches.length, pages, page: current };
}
