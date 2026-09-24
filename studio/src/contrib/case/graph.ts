import type { LiveEdge, LiveNode, LiveWorkspace } from "../../clients/live";

export interface GraphNode extends LiveNode { referenceOnly?: boolean }
export interface CaseGraph { nodes: GraphNode[]; edges: LiveEdge[] }
export type GraphLayout = "relational" | "directed" | "temporal";

/** Local geometry only. It neither infers nor appends a relation. */
export function arrangeGraph(nodes: GraphNode[], edges: LiveEdge[], layout: GraphLayout, columns: number) {
  if (layout === "temporal") return nodes.map((node, index) => ({ ...node, x: 115 + (index % columns) * 220, y: 55 + Math.floor(index / columns) * 110 }));
  if (layout === "directed") {
    const levels = new Map<string, number>();
    const remaining = new Set(nodes.map(node => node.id));
    let level = 0;
    while (remaining.size) {
      const roots = [...remaining].filter(id => !edges.some(edge => edge.to === id && edge.from !== id && remaining.has(edge.from)));
      // Cycles are retained and placed together; layout does not claim a DAG.
      const next = roots.length ? roots : [...remaining];
      next.forEach(id => { levels.set(id, level); remaining.delete(id); }); level++;
    }
    const rows = new Map<number, number>();
    return nodes.map(node => { const column = levels.get(node.id)!; const row = rows.get(column) ?? 0; rows.set(column, row + 1); return { ...node, x: 115 + column * 235, y: 60 + row * 100 }; });
  }
  // Deterministic bounded force layout for the currently displayed projection.
  const positions = nodes.map((node, index) => ({ ...node, x: Math.cos(index * 2 * Math.PI / Math.max(1, nodes.length)) * Math.max(180, nodes.length * 24), y: Math.sin(index * 2 * Math.PI / Math.max(1, nodes.length)) * Math.max(120, nodes.length * 16) }));
  const index = new Map(positions.map((node, i) => [node.id, i]));
  for (let step = 0; step < 70; step++) {
    const forces = positions.map(() => ({ x: 0, y: 0 }));
    for (let a = 0; a < positions.length; a++) for (let b = a + 1; b < positions.length; b++) {
      const dx = positions[a].x - positions[b].x || .1, dy = positions[a].y - positions[b].y || .1;
      const squared = Math.max(400, dx * dx + dy * dy);
      const strength = 900 / squared;
      forces[a].x += dx * strength; forces[a].y += dy * strength;
      forces[b].x -= dx * strength; forces[b].y -= dy * strength;
    }
    for (const edge of edges) {
      const a = index.get(edge.from), b = index.get(edge.to); if (a === undefined || b === undefined || a === b) continue;
      const dx = positions[b].x - positions[a].x, dy = positions[b].y - positions[a].y;
      const strength = .07 * Math.max(0, (Math.hypot(dx, dy) - 230) / 230);
      forces[a].x += dx * strength; forces[a].y += dy * strength; forces[b].x -= dx * strength; forces[b].y -= dy * strength;
    }
    positions.forEach((node, i) => { node.x += Math.max(-12, Math.min(12, forces[i].x - node.x * .035)); node.y += Math.max(-12, Math.min(12, forces[i].y - node.y * .035)); });
  }
  return positions;
}
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

/** Camera geometry only; fitting must never rewrite dragged node positions. */
export function graphBounds(nodes: Array<{ x: number; y: number }>) {
  return { left: Math.min(0, ...nodes.map(node => node.x - 105)), top: Math.min(0, ...nodes.map(node => node.y - 40)), right: Math.max(210, ...nodes.map(node => node.x + 105)), bottom: Math.max(130, ...nodes.map(node => node.y + 50)) };
}
export function fitGraphCamera(nodes: Array<{ x: number; y: number }>, initial: Array<{ x: number; y: number }>, size: { width: number; height: number }) {
  const bounds = graphBounds(nodes), basis = graphBounds(initial);
  const fittedScale = (b: ReturnType<typeof graphBounds>) => Math.max(.001, Math.min(1, (size.width - 32) / (b.right - b.left), (size.height - 32) / (b.bottom - b.top)));
  const scale = fittedScale(bounds), base = fittedScale(basis);
  return { zoom: scale / base, x: ((basis.right + basis.left) - (bounds.right + bounds.left)) * scale / 2, y: ((basis.bottom + basis.top) - (bounds.bottom + bounds.top)) * scale / 2 };
}
