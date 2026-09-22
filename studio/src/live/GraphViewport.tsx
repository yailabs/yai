import { useEffect, useId, useMemo, useRef, useState } from "react";
import type { LiveEdge, LiveNode } from "../clients/live";
import { Button, EmptyState, SearchInput } from "../components/primitives";
import { arrangeGraph, completeGraph, graphSlice, type GraphLayout } from "../contrib/case/graph";

export function GraphViewport({ nodes, edges, layout, onInspect }: {
  nodes: LiveNode[]; edges: LiveEdge[]; layout: GraphLayout;
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
  const [kind, setKind] = useState("");
  const [relation, setRelation] = useState("");
  const [camera, setCamera] = useState({ zoom: 1, x: 0, y: 0 });
  const [positions, setPositions] = useState<Record<string, { x: number; y: number }>>({});
  const drag = useRef<{ id?: string; x: number; y: number; origin: { x: number; y: number } } | undefined>(undefined);
  const filtered = useMemo(() => {
    const nodes = graph.nodes.filter(node => !kind || node.kind === kind);
    const ids = new Set(nodes.map(node => node.id));
    return { nodes, edges: graph.edges.filter(edge => ids.has(edge.from) && ids.has(edge.to) && (!relation || edge.kind === relation)) };
  }, [graph, kind, relation]);
  const viewport = useMemo(() => graphSlice(filtered, query, focus, page, columns * 3), [filtered, query, focus, page, columns]);
  const initialPositions = useMemo(() => arrangeGraph(viewport.nodes, viewport.edges, layout, columns), [viewport, layout, columns]);
  const arranged = initialPositions.map(node => ({ ...node, ...positions[node.id] }));
  const byId = new Map(arranged.map(node => [node.id, node]));
  const bounds = { left: Math.min(0, ...initialPositions.map(node => node.x - 105)), top: Math.min(0, ...initialPositions.map(node => node.y - 40)), right: Math.max(210, ...initialPositions.map(node => node.x + 105)), bottom: Math.max(130, ...initialPositions.map(node => node.y + 50)) };
  const fit = Math.min(1, (size.width - 32) / (bounds.right - bounds.left), (size.height - 32) / (bounds.bottom - bounds.top));
  const scale = fit * camera.zoom;
  const labelSize = Math.max(12, Math.min(24, 10 / Math.max(.1, scale)));
  const labelLimit = Math.max(6, Math.floor(142 / (labelSize * .6)));
  const offset = { x: (size.width - (bounds.right + bounds.left) * scale) / 2 + camera.x, y: (size.height - (bounds.bottom + bounds.top) * scale) / 2 + camera.y };
  const point = (event: React.PointerEvent<SVGElement>) => {
    const svg = event.currentTarget instanceof SVGSVGElement ? event.currentTarget : event.currentTarget.ownerSVGElement!;
    const matrix = svg.getScreenCTM();
    return matrix ? new DOMPoint(event.clientX, event.clientY).matrixTransform(matrix.inverse()) : new DOMPoint();
  };
  const reset = () => { setCamera({ zoom: 1, x: 0, y: 0 }); setPositions({}); };
  const select = (id: string) => { setSelected(id); onInspect(id); };
  useEffect(() => {
    const element = canvas.current; if (!element) return;
    const wheel = (event: WheelEvent) => { event.preventDefault(); setCamera(value => ({ ...value, zoom: Math.max(.5, Math.min(8, value.zoom * Math.exp(-event.deltaY * .002))) })); };
    element.addEventListener("wheel", wheel, { passive: false });
    return () => element.removeEventListener("wheel", wheel);
  }, [nodes.length > 0 || edges.length > 0]);
  const neighbors = new Set(graph.edges.filter(edge => edge.from === selected || edge.to === selected || edge.id === selected).flatMap(edge => [edge.from, edge.to]));
  if (!nodes.length && !edges.length) return <EmptyState title="No graph relations" body="YAI has not exposed qualified relations for this Case." />;
  return <section className="graph-viewport" aria-label={`${layout} Case graph`}>
    <div className="graph-tools">
      <SearchInput aria-label="Filter graph nodes" placeholder="Find an object or phrase" value={query} onChange={event => { setQuery(event.target.value); setPage(0); reset(); }} />
      <Button onClick={reset}>Fit</Button>
      <Button aria-label="Zoom out" disabled={camera.zoom <= .5} onClick={() => setCamera(value => ({ ...value, zoom: Math.max(.5, value.zoom / 1.25) }))}>−</Button>
      <span>{Math.round(scale * 100)}%</span>
      <Button aria-label="Zoom in" disabled={camera.zoom >= 8} onClick={() => setCamera(value => ({ ...value, zoom: Math.min(8, value.zoom * 1.25) }))}>+</Button>
      {focus ? <Button onClick={() => { setFocus(undefined); setPage(0); reset(); }}>Show all</Button> : <Button disabled={!graph.nodes.some(node => node.id === selected)} onClick={() => { setFocus(selected); setQuery(""); setPage(0); reset(); }}>Show related</Button>}
    </div>
    <div className="graph-filters"><select aria-label="Graph object kind" value={kind} onChange={event => { setKind(event.target.value); setFocus(undefined); setPage(0); reset(); }}><option value="">All object kinds</option>{[...new Set(graph.nodes.map(node => node.kind).filter(Boolean))].sort().map(value => <option key={value}>{value}</option>)}</select><select aria-label="Graph relation kind" value={relation} onChange={event => { setRelation(event.target.value); setFocus(undefined); setPage(0); reset(); }}><option value="">All relation kinds</option>{[...new Set(graph.edges.map(edge => edge.kind))].sort().map(value => <option key={value}>{value}</option>)}</select><span>{graph.nodes.filter(node => node.referenceOnly).length} unresolved references</span></div>

    <svg ref={canvas} viewBox={`0 0 ${size.width} ${size.height}`} role="group" aria-label="Graph objects and directed relations"
      onPointerDown={event => { if (event.target !== event.currentTarget) return; const p = point(event); drag.current = { x: p.x, y: p.y, origin: camera }; event.currentTarget.setPointerCapture(event.pointerId); }}
      onPointerMove={event => { const current = drag.current; if (!current) return; const p = point(event); const dx = p.x - current.x, dy = p.y - current.y; if (current.id) setPositions(all => ({ ...all, [current.id!]: { x: current.origin.x + dx / scale, y: current.origin.y + dy / scale } })); else setCamera(value => ({ ...value, x: current.origin.x + dx, y: current.origin.y + dy })); }}
      onPointerUp={() => { drag.current = undefined; }} onPointerCancel={() => { drag.current = undefined; }}>
      <defs><marker id={marker} viewBox="0 0 10 10" refX="17" refY="5" markerWidth="5" markerHeight="5" orient="auto-start-reverse"><path d="M0 0L10 5L0 10z" /></marker></defs>
      <g transform={`translate(${offset.x} ${offset.y}) scale(${scale})`}>
        {viewport.edges.map(edge => { const from = byId.get(edge.from)!; const to = byId.get(edge.to)!; const active = !selected || edge.id === selected || edge.from === selected || edge.to === selected; return <g data-relation-id={edge.id} key={edge.id} tabIndex={0} role="button" aria-label={`${edge.kind}: ${from.label} → ${to.label}`} aria-pressed={selected === edge.id} onClick={() => select(edge.id)} onKeyDown={event => { if (event.key === "Enter" || event.key === " ") { event.preventDefault(); select(edge.id); } }} className={`edge ${active ? "active" : "muted"} ${selected === edge.id ? "selected" : ""}`}><title>{`${edge.kind}: ${from.label} → ${to.label}`}</title><line className="edge-hit" x1={from.x} y1={from.y} x2={to.x} y2={to.y} /><line x1={from.x} y1={from.y} x2={to.x} y2={to.y} markerEnd={`url(#${marker})`} />{selected && active && (scale >= .7 || selected === edge.id) && <text x={(from.x + to.x) / 2} y={(from.y + to.y) / 2 - 6}>{edge.kind}</text>}</g>; })}
        {arranged.map((node, index) => <g key={node.id} data-object-id={node.id} className={`graph-node ${node.id === selected ? "selected" : ""} ${selected && node.id !== selected && !neighbors.has(node.id) ? "muted" : ""} ${node.referenceOnly ? "reference-only" : ""}`} transform={`translate(${node.x} ${node.y})`} tabIndex={0} role="button" aria-pressed={node.id === selected} aria-label={`${node.label ?? node.id}, ${node.kind ?? "node"}`} onClick={() => select(node.id)} onKeyDown={event => {
          if (event.key === "Enter" || event.key === " ") { event.preventDefault(); select(node.id); }
          const delta = ({ ArrowRight: 1, ArrowLeft: -1, ArrowDown: columns, ArrowUp: -columns } as Record<string, number>)[event.key];
          if (delta) { event.preventDefault(); const target = Math.min(arranged.length - 1, Math.max(0, index + delta)); event.currentTarget.parentElement?.querySelectorAll<SVGGElement>(".graph-node")[target]?.focus(); }
        }} onPointerDown={event => { event.stopPropagation(); const p = point(event); drag.current = { id: node.id, x: p.x, y: p.y, origin: node }; event.currentTarget.setPointerCapture(event.pointerId); }}>
          <title>{`${node.label ?? node.id}\n${node.kind}${node.referenceOnly ? " · reference only" : ""}\n${node.id}`}</title><rect x={-90} y={-20} width={180} height={54} rx={7} /><circle cx={-76} cy={-2} r={4} /><text x={-63} y={scale < .7 ? 12 : 2} className="node-label" style={{ fontSize: labelSize }}>{(node.label ?? node.id).slice(0, labelLimit)}{(node.label ?? node.id).length > labelLimit ? "…" : ""}</text>{scale >= .7 && <text x={-63} y={20} className="node-kind">{node.kind?.replaceAll("_", " ")}</text>}
        </g>)}
      </g>
    </svg>
    {!arranged.length && <EmptyState title="No matching objects" body="Clear the search or return to all projected objects." />}
    <footer className="graph-summary" title={`${viewport.matches} matching objects. Pages and filters hide relations outside the visible objects; they do not change the projection.`}><span role="status">{viewport.nodes.length} of {graph.nodes.length} objects · {viewport.edges.length} of {graph.edges.length} relations shown</span><Button disabled={viewport.page === 0} onClick={() => { setPage(viewport.page - 1); reset(); }}>Previous</Button><span>{viewport.page + 1} / {viewport.pages}</span><Button disabled={viewport.page + 1 === viewport.pages} onClick={() => { setPage(viewport.page + 1); reset(); }}>Next</Button></footer>
  </section>;
}
