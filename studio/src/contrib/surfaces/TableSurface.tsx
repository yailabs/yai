import { useCallback, useMemo, useRef, useState, useSyncExternalStore } from "react";
import Papa from "papaparse";
import { Button, EmptyState } from "../../components/primitives";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import { MaterialHeader, presentationMaterial } from "./MaterialSurfaces";
import { useReadableMaterial } from "./materialRead";

export default function TableSurface(props: SurfaceRendererProps) {
  const { workspace, input, actions, buffers } = props;
  const material = presentationMaterial(workspace, input.objectRef);
  const authored = material?.body.kind === "table" ? material.body : undefined;
  const loaded = useReadableMaterial(props, "text", Boolean(authored));
  const subscribe = useCallback((listener: () => void) => buffers.subscribe(input.identity, listener).dispose, [buffers, input.identity]);
  const buffer = useSyncExternalStore(subscribe, () => buffers.snapshot(input.identity));
  const text = buffer?.dirty ? buffer.value : loaded.content;
  const [headerRow, setHeaderRow] = useState(true);
  const parsed = useMemo(() => {
    if (authored) return { body: authored };
    if (text === undefined) return {};
    const result = Papa.parse<string[]>(text, { delimiter: input.metadata?.mediaType === "text/tab-separated-values" ? "\t" : ",", skipEmptyLines: "greedy" });
    const [header, ...remaining] = result.data;
    const rows = headerRow ? remaining : result.data;
    if (result.errors.length || !header?.length || rows.some(row => row.length !== header.length)) return { error: "The rows do not form a consistent table. Open with Text Editor to inspect the exact source." };
    if (header.length > 100) return { error: "This table exceeds the bounded 100-column view. Its exact source remains available in Text Editor." };
    return { body: { columns: header.map((label, index) => ({ key: String(index), label: headerRow && label ? label : `Column ${index + 1}` })), rows: rows.map((row, index) => ({ id: String(index + 1), values: Object.fromEntries(row.map((value, column) => [String(column), value])) })) } };
  }, [authored, text, input.metadata?.mediaType, headerRow]);
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<{ key: string; direction: 1 | -1 }>();
  const [page, setPage] = useState(0);
  const [selected, setSelected] = useState<string>();
  const [widths, setWidths] = useState<Record<string, number>>({});
  const drag = useRef<{ key: string; x: number; width: number } | undefined>(undefined);
  const rows = useMemo(() => {
    const needle = query.toLocaleLowerCase();
    const filtered = parsed.body?.rows.filter(row => !needle || Object.values(row.values).some(value => value.toLocaleLowerCase().includes(needle))) ?? [];
    return sort ? [...filtered].sort((a,b) => (a.values[sort.key] ?? "").localeCompare(b.values[sort.key] ?? "", undefined, { numeric: true }) * sort.direction) : filtered;
  }, [parsed.body, query, sort]);
  const pages = Math.max(1, Math.ceil(rows.length / 50));
  const current = Math.min(page, pages - 1);
  const size = (key: string, value: number) => setWidths(before => ({ ...before, [key]: Math.max(80, Math.min(600, value)) }));
  const select = (id: string) => { setSelected(id); if (authored) actions.inspect(id); else if (input.objectRef) actions.inspect(input.objectRef); };
  return <article className="material-surface table-surface" data-surface-type={input.surfaceType}><MaterialHeader input={input} material={material} />{buffer?.dirty && <p className="preview-local-note">{buffer.stale ? "Table preview of stale local draft" : "Table preview of unsaved local changes"}</p>}{buffer?.dirty && loaded.error && <p className="file-read-warning" role="alert">{loaded.error} Only the unsaved local draft is shown.</p>}{parsed.body ? <>
    <div className="surface-toolbar" role="toolbar" aria-label="Table controls"><input value={query} onChange={event => { setQuery(event.target.value); setPage(0); }} placeholder="Filter rows" aria-label="Filter table rows" /><span>{rows.length} of {parsed.body.rows.length} rows</span>{!authored && <label><input type="checkbox" checked={headerRow} onChange={event => { setHeaderRow(event.target.checked); setPage(0); setSelected(undefined); }} /> First row as column names</label>}<span>Read-only view</span></div>
    <div className="table-scroll"><table style={{ tableLayout: "fixed", width: parsed.body.columns.map(column => widths[column.key] ?? 180).reduce((total,width) => total + width, 0) }}><thead><tr>{parsed.body.columns.map(column => <th key={column.key} style={{ width: widths[column.key] ?? 180 }} aria-sort={sort?.key === column.key ? sort.direction === 1 ? "ascending" : "descending" : "none"}><button onClick={() => { setSort(before => ({ key: column.key, direction: before?.key === column.key && before.direction === 1 ? -1 : 1 })); setPage(0); }}>{column.label}</button><span className="table-column-resize" role="separator" tabIndex={0} aria-label={`Resize ${column.label}`} aria-orientation="vertical" aria-valuemin={80} aria-valuemax={600} aria-valuenow={widths[column.key] ?? 180} onKeyDown={event => { if (event.key === "ArrowLeft" || event.key === "ArrowRight") { event.preventDefault(); size(column.key, (widths[column.key] ?? 180) + (event.key === "ArrowLeft" ? -20 : 20)); } }} onPointerDown={event => { drag.current = { key: column.key, x: event.clientX, width: widths[column.key] ?? 180 }; event.currentTarget.setPointerCapture(event.pointerId); }} onPointerMove={event => { if (drag.current?.key === column.key) size(column.key, drag.current.width + event.clientX - drag.current.x); }} onPointerUp={() => { drag.current = undefined; }} onPointerCancel={() => { drag.current = undefined; }} /></th>)}</tr></thead><tbody>{rows.slice(current*50, (current+1)*50).map(row => <tr key={row.id} tabIndex={0} aria-selected={selected === row.id} onClick={() => select(row.id)} onKeyDown={event => { if (event.key === "Enter" || event.key === " ") { event.preventDefault(); select(row.id); } }}>{parsed.body!.columns.map(column => <td key={column.key} title={row.values[column.key]}>{row.values[column.key] ?? "—"}</td>)}</tr>)}</tbody></table></div>
    <footer className="table-pagination"><span>{selected ? `Selected row ${selected}` : "Select a row to inspect its material"}</span><Button disabled={current === 0} onClick={() => setPage(current-1)}>Previous</Button><span>{current+1} / {pages}</span><Button disabled={current+1 >= pages} onClick={() => setPage(current+1)}>Next</Button></footer>
  </> : <EmptyState title={loaded.loading ? "Reading table…" : "Table unavailable"} body={parsed.error ?? loaded.error ?? "No qualified structured rows are available."} />}</article>;
}
