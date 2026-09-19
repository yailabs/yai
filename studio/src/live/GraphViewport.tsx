import { useMemo, useRef, useState } from "react";
import type { LiveEdge, LiveNode } from "../clients/live";
import { Button, EmptyState, SearchInput } from "../components/primitives";

interface Positioned extends LiveNode { x: number; y: number }

export function GraphViewport({ nodes, edges, layout, onInspect }: {
  nodes: LiveNode[]; edges: LiveEdge[]; layout: "relational" | "directed";
  onInspect: (id: string) => void;
}) {
  const [query, setQuery] = useState("");
  const [scale, setScale] = useState(1);
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const [selected, setSelected] = useState<string>();
  const drag = useRef<{ kind: "canvas" | "node"; id?: string; x: number; y: number } | undefined>(undefined);
  const [positions, setPositions] = useState<Record<string, { x: number; y: number }>>({});
  const arranged = useMemo<Positioned[]>(() => nodes.map((node, index) => {
    const directedX = 110 + (index % 4) * 210;
    const directedY = 90 + Math.floor(index / 4) * 150;
    const angle = (index / Math.max(nodes.length, 1)) * Math.PI * 2;
    const initial = layout === "directed" ? { x: directedX, y: directedY } : { x: 430 + Math.cos(angle) * (150 + (index % 3) * 28), y: 235 + Math.sin(angle) * (135 + (index % 2) * 25) };
    return { ...node, ...(positions[node.id] ?? initial) };
  }), [nodes, layout, positions]);
  const byId = new Map(arranged.map((node) => [node.id, node]));
  const neighbors = new Set(edges.flatMap((edge) => edge.from === selected ? [edge.to] : edge.to === selected ? [edge.from] : []));
  const matched = new Set(arranged.filter((node) => `${node.label ?? node.id} ${node.kind ?? ""}`.toLowerCase().includes(query.toLowerCase())).map((node) => node.id));
  if (!nodes.length) return <EmptyState title="No graph relations" body="YAI has not exposed qualified relations for this Case." />;
  return <section className="graph-viewport" aria-label={`${layout} Case graph`}>
    <div className="graph-tools">
      <SearchInput aria-label="Filter graph nodes" placeholder="Find node" value={query} onChange={(event) => setQuery(event.target.value)} />
      <Button onClick={() => { setScale(1); setOffset({ x: 0, y: 0 }); }}>Fit</Button>
      <Button aria-label="Zoom out" onClick={() => setScale((value) => Math.max(.55, value - .15))}>−</Button>
      <span>{Math.round(scale * 100)}%</span>
      <Button aria-label="Zoom in" onClick={() => setScale((value) => Math.min(1.8, value + .15))}>+</Button>
    </div>
    <svg viewBox="0 0 860 470" role="img"
      onPointerDown={(event) => { if (event.target === event.currentTarget) drag.current = { kind: "canvas", x: event.clientX - offset.x, y: event.clientY - offset.y }; }}
      onPointerMove={(event) => { const current = drag.current; if (!current) return; if (current.kind === "canvas") setOffset({ x: event.clientX - current.x, y: event.clientY - current.y }); else if (current.id) setPositions((all) => ({ ...all, [current.id!]: { x: (event.nativeEvent.offsetX - offset.x) / scale, y: (event.nativeEvent.offsetY - offset.y) / scale } })); }}
      onPointerUp={() => { drag.current = undefined; }} onPointerLeave={() => { drag.current = undefined; }}>
      <g transform={`translate(${offset.x} ${offset.y}) scale(${scale})`}>
        {edges.map((edge) => { const from = byId.get(edge.from); const to = byId.get(edge.to); if (!from || !to) return null; const active = !selected || edge.from === selected || edge.to === selected; return <g key={edge.id} className={active ? "edge active" : "edge muted"}><line x1={from.x} y1={from.y} x2={to.x} y2={to.y} /><text x={(from.x + to.x) / 2} y={(from.y + to.y) / 2 - 6}>{edge.kind}</text></g>; })}
        {arranged.map((node) => { const active = !selected || node.id === selected || neighbors.has(node.id); const hidden = query && !matched.has(node.id); return <g key={node.id} className={`graph-node ${active ? "active" : "muted"} ${hidden ? "filtered" : ""}`} transform={`translate(${node.x} ${node.y})`} tabIndex={0} role="button" aria-label={`${node.label ?? node.id}, ${node.kind ?? "node"}`} onDoubleClick={() => onInspect(node.id)} onClick={() => { setSelected(node.id); onInspect(node.id); }} onPointerDown={(event) => { event.stopPropagation(); drag.current = { kind: "node", id: node.id, x: event.clientX, y: event.clientY }; }}><circle r={node.id === selected ? 12 : 9} /><text y={24}>{node.label ?? compact(node.id)}</text></g>; })}
      </g>
    </svg>
  </section>;
}

function compact(value: string) { return value.length > 28 ? `${value.slice(0, 14)}…${value.slice(-9)}` : value; }
