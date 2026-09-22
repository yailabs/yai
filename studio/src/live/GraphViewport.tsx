import { useEffect, useId, useMemo, useRef, useState } from "react";
import type { LiveEdge, LiveNode } from "../clients/live";
import { Button, EmptyState, SearchInput } from "../components/primitives";
import { completeGraph, graphSlice } from "../contrib/case/graph";

export function GraphViewport({ nodes, edges, layout, onInspect }: {
  nodes: LiveNode[]; edges: LiveEdge[]; layout: "relational" | "directed";
  onInspect: (id: string) => void;
}) {
  const canvas = useRef<SVGSVGElement>(null);
  const [size, setSize] = useState({ width: 860, height: 470 });
  useEffect(() => {
    const element = canvas.current; if (!element) return;
    const observer = new ResizeObserver(() => {
      const { width, height } = element.getBoundingClientRect();
      if (width > 0 && height > 0) setSize(previous => previous.width === width && previous.height === height ? previous : { width, height });
    });
    observer.observe(element); return () => observer.disconnect();
  }, [nodes.length > 0 || edges.length > 0]);
  const columns = Math.max(1, Math.min(4, Math.floor(size.width / 210)));
  const marker = useId().replaceAll(":", "");
  const graph = useMemo(() => completeGraph(nodes, edges), [nodes, edges]);
  const [query, setQuery] = useState("");
  const [page, setPage] = useState(0);
  const [focus, setFocus] = useState<string>();
  const [selected, setSelected] = useState<string>();
  const [camera, setCamera] = useState({ zoom: 1, x: 0, y: 0 });
  const [positions, setPositions] = useState<Record<string, { x: number; y: number }>>({});
  const drag = useRef<{ id?: string; x: number; y: number; origin: { x: number; y: number } } | undefined>(undefined);
  const viewport = useMemo(() => graphSlice(graph, query, focus, page, columns * 3), [graph, query, focus, page, columns]);
  const arranged = viewport.nodes.map((node, index) => ({ ...node, ...(positions[node.id] ?? {
    x: 110 + (index % columns) * 210, y: 60 + Math.floor(index / columns) * 92,
  }) }));
  const byId = new Map(arranged.map(node => [node.id, node]));
  const bounds = { left: 0, top: 0, right: Math.min(columns, Math.max(1, arranged.length)) * 210 + 20, bottom: Math.max(130, Math.ceil(arranged.length / columns) * 92 + 15) };
  const fit = Math.min(1, (size.width - 32) / (bounds.right - bounds.left), (size.height - 32) / (bounds.bottom - bounds.top));
  const scale = fit * camera.zoom;
  const offset = { x: (size.width - (bounds.right + bounds.left) * scale) / 2 + camera.x, y: (size.height - (bounds.bottom + bounds.top) * scale) / 2 + camera.y };
  const point = (event: React.PointerEvent<SVGElement>) => {
    const svg = event.currentTarget instanceof SVGSVGElement ? event.currentTarget : event.currentTarget.ownerSVGElement!;
    const matrix = svg.getScreenCTM();
    return matrix ? new DOMPoint(event.clientX, event.clientY).matrixTransform(matrix.inverse()) : new DOMPoint();
  };
  const reset = () => { setCamera({ zoom: 1, x: 0, y: 0 }); setPositions({}); };
  const select = (id: string) => { setSelected(id); onInspect(id); };
  if (!nodes.length && !edges.length) return <EmptyState title="No graph relations" body="YAI has not exposed qualified relations for this Case." />;
  return <section className="graph-viewport" aria-label={`${layout} Case graph`}>
    <div className="graph-tools">
      <SearchInput aria-label="Filter graph nodes" placeholder="Find an object or phrase" value={query} onChange={event => { setQuery(event.target.value); setPage(0); reset(); }} />
      <Button onClick={reset}>Fit</Button>
      <Button aria-label="Zoom out" disabled={camera.zoom <= .5} onClick={() => setCamera(value => ({ ...value, zoom: Math.max(.5, value.zoom / 1.25) }))}>−</Button>
      <span>{Math.round(scale * 100)}%</span>
      <Button aria-label="Zoom in" disabled={camera.zoom >= 8} onClick={() => setCamera(value => ({ ...value, zoom: Math.min(8, value.zoom * 1.25) }))}>+</Button>
    </div>
    <div className="graph-scope"><span>{focus ? `Related to ${graph.nodes.find(node => node.id === focus)?.label ?? focus}` : "All projected objects"}</span>{focus ? <Button onClick={() => { setFocus(undefined); setPage(0); reset(); }}>Show all</Button> : <Button disabled={!selected} onClick={() => { setFocus(selected); setQuery(""); setPage(0); reset(); }}>Show related</Button>}</div>
    <svg ref={canvas} viewBox={`0 0 ${size.width} ${size.height}`} role="group" aria-label="Graph objects and directed relations"
      onPointerDown={event => { if (event.target !== event.currentTarget) return; const p = point(event); drag.current = { x: p.x, y: p.y, origin: camera }; event.currentTarget.setPointerCapture(event.pointerId); }}
      onPointerMove={event => { const current = drag.current; if (!current) return; const p = point(event); const dx = p.x - current.x, dy = p.y - current.y; if (current.id) setPositions(all => ({ ...all, [current.id!]: { x: current.origin.x + dx / scale, y: current.origin.y + dy / scale } })); else setCamera(value => ({ ...value, x: current.origin.x + dx, y: current.origin.y + dy })); }}
      onPointerUp={() => { drag.current = undefined; }} onPointerCancel={() => { drag.current = undefined; }}>
      <defs><marker id={marker} viewBox="0 0 10 10" refX="17" refY="5" markerWidth="5" markerHeight="5" orient="auto-start-reverse"><path d="M0 0L10 5L0 10z" /></marker></defs>
      <g transform={`translate(${offset.x} ${offset.y}) scale(${scale})`}>
        {viewport.edges.map(edge => { const from = byId.get(edge.from)!; const to = byId.get(edge.to)!; const active = !selected || edge.from === selected || edge.to === selected; return <g data-relation-id={edge.id} key={edge.id} className={`edge ${active ? "active" : "muted"}`}><title>{`${edge.kind}: ${from.label} → ${to.label}`}</title><line x1={from.x} y1={from.y} x2={to.x} y2={to.y} markerEnd={`url(#${marker})`} />{selected && active && <text x={(from.x + to.x) / 2} y={(from.y + to.y) / 2 - 6}>{edge.kind}</text>}</g>; })}
        {arranged.map((node, index) => <g key={node.id} data-object-id={node.id} className={`graph-node ${node.id === selected ? "selected" : ""} ${node.referenceOnly ? "reference-only" : ""}`} transform={`translate(${node.x} ${node.y})`} tabIndex={0} role="button" aria-pressed={node.id === selected} aria-label={`${node.label ?? node.id}, ${node.kind ?? "node"}`} onClick={() => select(node.id)} onKeyDown={event => {
          if (event.key === "Enter" || event.key === " ") { event.preventDefault(); select(node.id); }
          const delta = ({ ArrowRight: 1, ArrowLeft: -1, ArrowDown: columns, ArrowUp: -columns } as Record<string, number>)[event.key];
          if (delta) { event.preventDefault(); const target = Math.min(arranged.length - 1, Math.max(0, index + delta)); event.currentTarget.parentElement?.querySelectorAll<SVGGElement>(".graph-node")[target]?.focus(); }
        }} onPointerDown={event => { event.stopPropagation(); const p = point(event); drag.current = { id: node.id, x: p.x, y: p.y, origin: node }; event.currentTarget.setPointerCapture(event.pointerId); }}>
          <title>{`${node.label ?? node.id}\n${node.kind}${node.referenceOnly ? " · reference only" : ""}\n${node.id}`}</title><rect x={-90} y={-20} width={180} height={54} rx={7} /><circle cx={-76} cy={-2} r={4} /><text x={-63} y={2} className="node-label">{(node.label ?? node.id).slice(0, 23)}{(node.label ?? node.id).length > 23 ? "…" : ""}</text><text x={-63} y={20} className="node-kind">{node.kind?.replaceAll("_", " ")}</text>
        </g>)}
      </g>
    </svg>
    {!arranged.length && <EmptyState title="No matching objects" body="Clear the search or return to all projected objects." />}
    <footer className="graph-summary"><span role="status">{viewport.nodes.length} of {graph.nodes.length} objects · {viewport.edges.length} of {graph.edges.length} relations shown</span><span>{viewport.matches} matching objects</span><Button disabled={viewport.page === 0} onClick={() => { setPage(viewport.page - 1); reset(); }}>Previous</Button><span>{viewport.page + 1} / {viewport.pages}</span><Button disabled={viewport.page + 1 === viewport.pages} onClick={() => { setPage(viewport.page + 1); reset(); }}>Next</Button></footer>
    <p className="graph-hint">Select an object to inspect it. Show related isolates its qualified neighbors. Pages and filters hide relations outside the visible objects; they do not change the projection.</p>
  </section>;
}
