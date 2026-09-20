import { useMemo, useState } from "react";
import { Badge, Button, EmptyState } from "../../components/primitives";
import type { MaterialBody, MaterialView } from "../../clients/presentation";
import type { SurfaceRendererProps, SurfaceSearchResult } from "../../workbench/kernel/types";

export function presentationMaterial(workspace: SurfaceRendererProps["workspace"], objectRef?: string) {
  return workspace.presentation.materials?.find((item) => item.id === objectRef);
}

export function MaterialHeader({ input, material }: { input: SurfaceRendererProps["input"]; material?: MaterialView }) {
  return <header className="material-header"><nav aria-label="Surface breadcrumb"><span>Case</span><b>›</b><span>Environment</span><b>›</b><strong>{input.title}</strong></nav><h1>{input.title}</h1><p>{material?.path ?? input.objectRef ?? "No object reference"}</p><div><Badge>{input.metadata?.mediaType ?? "media type unavailable"}</Badge><span>{material?.provenance ?? "Qualified Case presentation"}</span></div></header>;
}

export function MarkdownSurface({ workspace, input }: SurfaceRendererProps) {
  const material = presentationMaterial(workspace, input.objectRef);
  if (material?.body.kind === "document") return <article className="material-surface markdown-surface" data-surface-type={input.surfaceType}><MaterialHeader input={input} material={material} /><section className="material-document searchable-content"><h2>{material.body.title}</h2><p>{material.body.intro}</p>{material.body.sections.map((section) => <section key={section.title}><h3>{section.title}</h3><p>{section.body}</p>{section.points && <ul>{section.points.map((point) => <li key={point}>{point}</li>)}</ul>}</section>)}</section></article>;
  return <LiveTextProjection workspace={workspace} input={input} />;
}

export function TextSurface({ workspace, input }: SurfaceRendererProps) {
  const material = presentationMaterial(workspace, input.objectRef);
  const text = material ? materialText(material.body) : undefined;
  if (text !== undefined) return <article className="material-surface text-surface" data-surface-type={input.surfaceType}><MaterialHeader input={input} material={material} /><pre className="text-surface-content searchable-content">{text}</pre></article>;
  return <LiveTextProjection workspace={workspace} input={input} />;
}

export function StructuredTextSurface({ workspace, input }: SurfaceRendererProps) {
  const material = presentationMaterial(workspace, input.objectRef);
  const source = material ? materialText(material.body) : undefined;
  const mediaType = input.metadata?.mediaType ?? material?.mediaType ?? "structured text";
  return <article className="material-surface structured-text-surface" data-surface-type={input.surfaceType}><MaterialHeader input={input} material={material} />{source !== undefined ? <pre className="structured-code searchable-content"><code>{prettyStructured(source, mediaType)}</code></pre> : <EmptyState title="Structured content unavailable" body="YAI exposes this material identity, but no qualified readable representation." />}</article>;
}

function LiveTextProjection({ workspace, input }: Pick<SurfaceRendererProps, "workspace" | "input">) {
  const file = workspace.environment.files.find((item) => item.id === input.objectRef);
  const source = workspace.environment.sources.find((item) => item.id === (file?.source_ref ?? input.objectRef));
  const qualified = workspace.knowledge.sources.find((item) => item.id === input.objectRef || item.source_ref === (file?.source_ref ?? source?.id));
  const sourceKeys = new Set([qualified?.id, qualified?.source_ref, file?.source_ref, source?.id].filter(Boolean));
  const units = workspace.knowledge.units.filter((unit) => sourceKeys.has(unit.source_ref) && (!file || !qualified?.path || qualified.path === file.path));
  return <article className="material-surface text-surface" data-surface-type={input.surfaceType}><MaterialHeader input={input} /><dl className="material-metadata"><div><dt>Case</dt><dd>{workspace.case.case_ref}</dd></div><div><dt>Revision</dt><dd>{file?.revision_ref ?? source?.revision_ref ?? qualified?.revision_ref ?? "Unavailable"}</dd></div></dl>{units.length ? <section className="material-document searchable-content">{units.map((unit) => <div key={unit.id} className={unit.kind === "heading" ? "heading" : "body"}>{unit.text}</div>)}</section> : <EmptyState title="Content unavailable" body="YAI exposes this material identity, but no qualified readable content projection." />}</article>;
}

export function ImageSurface({ workspace, input, actions }: SurfaceRendererProps) {
  const material = presentationMaterial(workspace, input.objectRef);
  const body = material?.body.kind === "image" ? material.body : undefined;
  const [scale, setScale] = useState<"fit" | "actual" | "large">("fit");
  return <article className="material-surface image-surface" data-surface-type={input.surfaceType}><MaterialHeader input={input} material={material} /><div className="surface-toolbar" role="toolbar" aria-label="Image controls"><div className="segmented"><button aria-pressed={scale === "fit"} onClick={() => setScale("fit")}>Fit</button><button aria-pressed={scale === "actual"} onClick={() => setScale("actual")}>100%</button><button aria-pressed={scale === "large"} onClick={() => setScale("large")}>150%</button></div>{body && <Badge>{body.width} × {body.height}</Badge>}</div>{body ? <button className={`image-stage ${scale}`} onClick={() => input.objectRef && actions.inspect(input.objectRef)} aria-label={`Inspect ${input.title}`}><img src={body.source} alt={body.alt} width={body.width} height={body.height} /></button> : <EmptyState title="Image unavailable" body="The Case exposes image media, but no qualified browser-safe image source." />}{body?.caption && <p className="surface-caption">{body.caption}</p>}</article>;
}

export function TableSurface({ workspace, input, actions }: SurfaceRendererProps) {
  const material = presentationMaterial(workspace, input.objectRef);
  const body = material?.body.kind === "table" ? material.body : undefined;
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<{ key: string; direction: 1 | -1 }>();
  const rows = useMemo(() => {
    if (!body) return [];
    const needle = query.toLocaleLowerCase();
    const filtered = body.rows.filter((row) => !needle || Object.values(row.values).some((value) => value.toLocaleLowerCase().includes(needle)));
    return sort ? [...filtered].sort((a, b) => (a.values[sort.key] ?? "").localeCompare(b.values[sort.key] ?? "") * sort.direction) : filtered;
  }, [body, query, sort]);
  return <article className="material-surface table-surface" data-surface-type={input.surfaceType}><MaterialHeader input={input} material={material} />{body ? <><div className="surface-toolbar" role="toolbar" aria-label="Table controls"><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Filter rows" aria-label="Filter table rows" /><span>{rows.length} of {body.rows.length} rows</span></div><div className="table-scroll"><table><thead><tr>{body.columns.map((column) => <th key={column.key} aria-sort={sort?.key === column.key ? sort.direction === 1 ? "ascending" : "descending" : "none"}><button onClick={() => setSort((current) => ({ key: column.key, direction: current?.key === column.key && current.direction === 1 ? -1 : 1 }))}>{column.label}</button></th>)}</tr></thead><tbody>{rows.map((row) => <tr key={row.id} tabIndex={0} onClick={() => actions.inspect(row.id)} onKeyDown={(event) => { if (event.key === "Enter" || event.key === " ") { event.preventDefault(); actions.inspect(row.id); } }}>{body.columns.map((column) => <td key={column.key}>{row.values[column.key] ?? "—"}</td>)}</tr>)}</tbody></table></div></> : <EmptyState title="Table unavailable" body="No qualified structured rows are available." />}</article>;
}

export function AudioSurface(props: SurfaceRendererProps) { return <MediaSurface {...props} kind="audio" />; }
export function VideoSurface(props: SurfaceRendererProps) { return <MediaSurface {...props} kind="video" />; }
function MediaSurface({ workspace, input, kind }: SurfaceRendererProps & { kind: "audio" | "video" }) {
  const material = presentationMaterial(workspace, input.objectRef);
  const body = material?.body.kind === kind ? material.body : undefined;
  return <article className={`material-surface media-surface ${kind}-surface`} data-surface-type={input.surfaceType}><MaterialHeader input={input} material={material} />{body?.source ? kind === "audio" ? <audio controls preload="metadata" src={body.source}>Audio playback is unavailable.</audio> : <video controls preload="metadata" src={body.source}>Video playback is unavailable.</video> : <EmptyState title={`${kind === "audio" ? "Audio" : "Video"} unavailable`} body={body?.unavailableReason ?? "No qualified media source is exposed to Studio."} />}{body?.caption && <p className="surface-caption">{body.caption}</p>}</article>;
}

export function UnavailableMaterialSurface({ workspace, input, actions }: SurfaceRendererProps) {
  const material = presentationMaterial(workspace, input.objectRef);
  return <article className="material-surface unavailable-material" data-surface-type={input.surfaceType}><MaterialHeader input={input} material={material} /><section className="unknown-material"><EmptyState title="No trusted renderer" body={`Studio has no native renderer for ${input.metadata?.mediaType ?? "this qualified media type"}. Binary content has not been coerced into text.`} /><dl><div><dt>Name</dt><dd>{input.title}</dd></div><div><dt>Media type</dt><dd>{input.metadata?.mediaType ?? "Unavailable"}</dd></div><div><dt>Size</dt><dd>{input.metadata?.size ?? "Unavailable"}</dd></div><div><dt>Provenance</dt><dd>{material?.provenance ?? "Qualified identity only"}</dd></div></dl><div className="unknown-actions"><Button onClick={() => input.objectRef && actions.inspect(input.objectRef)}>Inspect metadata</Button><Button disabled title="No qualified local path is exposed">Open externally unavailable</Button></div></section></article>;
}

export function searchMaterialSurface({ workspace }: Pick<SurfaceRendererProps, "workspace">, input: SurfaceRendererProps["input"], query: string): readonly SurfaceSearchResult[] {
  const material = presentationMaterial(workspace, input.objectRef);
  if (!material || !query.trim()) return [];
  const needle = query.toLocaleLowerCase();
  if (material.body.kind === "table") return material.body.rows.filter((row) => Object.values(row.values).some((value) => value.toLocaleLowerCase().includes(needle))).map((row) => ({ id: row.id, label: Object.values(row.values)[0] ?? row.id, detail: Object.values(row.values).slice(1).join(" · "), objectRef: row.id }));
  return materialText(material.body)?.split("\n").map((line, index) => ({ line: line.trim(), index })).filter(({ line }) => line.toLocaleLowerCase().includes(needle)).slice(0, 40).map(({ line, index }) => ({ id: `${material.id}:${index}`, label: line || `Line ${index + 1}`, detail: `${material.name} · line ${index + 1}`, objectRef: material.id })) ?? [];
}

export async function searchPdfSurface({ workspace }: Pick<SurfaceRendererProps, "workspace">, input: SurfaceRendererProps["input"], query: string): Promise<readonly SurfaceSearchResult[]> {
  const material = presentationMaterial(workspace, input.objectRef);
  const source = material?.body.kind === "pdf" ? material.body.source : undefined;
  const materialId = material?.id;
  const needle = query.trim().toLocaleLowerCase();
  if (!source || !materialId || !needle) return [];
  const [{ getDocument, GlobalWorkerOptions }, worker] = await Promise.all([
    import("pdfjs-dist"),
    import("pdfjs-dist/build/pdf.worker.min.mjs?url"),
  ]);
  GlobalWorkerOptions.workerSrc = worker.default;
  const task = getDocument({ url: source });
  const document = await task.promise;
  try {
    const results: SurfaceSearchResult[] = [];
    for (let page = 1; page <= document.numPages; page += 1) {
      const content = await (await document.getPage(page)).getTextContent();
      const text = content.items.map((item) => "str" in item ? item.str : "").join(" ").trim();
      if (text.toLocaleLowerCase().includes(needle)) results.push({ id: `${materialId}:page:${page}`, label: `Page ${page}`, detail: text.slice(0, 140), objectRef: materialId });
    }
    return results;
  } finally {
    await task.destroy();
  }
}

function materialText(body: MaterialBody) {
  if (body.kind === "text") return body.content;
  if (body.kind === "diff") return body.lines.map((line) => `${line.change === "add" ? "+ " : line.change === "remove" ? "- " : "  "}${line.text}`).join("\n");
  if (body.kind === "document") return [body.title, body.intro, ...body.sections.flatMap((section) => [section.title, section.body, ...(section.points ?? [])])].join("\n\n");
  return undefined;
}

function prettyStructured(source: string, mediaType: string) {
  if (mediaType.includes("json")) { try { return JSON.stringify(JSON.parse(source), null, 2); } catch { return source; } }
  return source;
}
