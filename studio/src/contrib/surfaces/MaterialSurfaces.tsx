import { useEffect, useMemo, useRef, useState } from "react";
import { Badge, Button, EmptyState } from "../../components/primitives";
import type { MaterialBody, MaterialView } from "../../clients/presentation";
import type { SurfaceRendererProps, SurfaceSearchResult, WorkbenchRenderContext } from "../../workbench/kernel/types";

export function presentationMaterial(workspace: SurfaceRendererProps["workspace"], objectRef?: string) {
  const direct = workspace.presentation.materials?.find((item) => item.id === objectRef);
  if (direct) return direct;
  const file = workspace.environment.files.find((item) => item.id === objectRef);
  return workspace.presentation.materials?.find((item) => item.path === file?.path);
}

export function MaterialHeader({ input, material }: { input: SurfaceRendererProps["input"]; material?: MaterialView }) {
  const path = material?.path ?? input.metadata?.path;
  const parts = path?.split("/").filter(Boolean) ?? [input.title];
  return <header className="material-header minimal"><nav aria-label="File breadcrumb">{parts.map((part, index) => <span key={`${part}:${index}`}>{index > 0 && <b>›</b>}{part}</span>)}</nav></header>;
}

function useReadableMaterial({ workspace, input, readMaterial }: SurfaceRendererProps) {
  const fixture = presentationMaterial(workspace, input.objectRef);
  const authored = fixture ? materialText(fixture.body) : undefined;
  const [state, setState] = useState<{ content?: string; generation?: number; error?: string; loading: boolean }>({ content: authored, generation: authored === undefined ? undefined : workspace.case.generation, loading: authored === undefined });
  useEffect(() => {
    if (authored !== undefined) { setState({ content: authored, generation: workspace.case.generation, loading: false }); return; }
    const file = workspace.environment.files.find((item) => item.id === input.objectRef);
    if (!file) { setState({ error: "No qualified file revision is attached to this Surface.", loading: false }); return; }
    let active = true;
    setState({ loading: true });
    void readMaterial({ case_ref: workspace.case.case_ref, source_ref: file.source_ref, revision_ref: file.revision_ref, path: file.path, expected_generation: workspace.case.generation }).then((result) => {
      if (!active) return;
      if (result.result_state !== "success" || !result.data) { setState({ error: result.error?.safe_message ?? `Material read ${result.result_state}.`, loading: false }); return; }
      if (result.data.encoding !== "utf-8") { setState({ error: "This exact revision is binary and requires a trusted binary renderer.", loading: false }); return; }
      setState({ content: result.data.content, generation: result.data.generation, loading: false });
    });
    return () => { active = false; };
  }, [authored, input.objectRef, readMaterial, workspace.case.case_ref, workspace.case.generation, workspace.environment.files]);
  return state;
}

export function TextEditorSurface(props: SurfaceRendererProps) {
  const { input, actions, buffers } = props;
  const loaded = useReadableMaterial(props);
  const textarea = useRef<HTMLTextAreaElement>(null);
  const [snapshot, setSnapshot] = useState(() => buffers.snapshot(input.identity));
  const [findOpen, setFindOpen] = useState(false);
  const [find, setFind] = useState("");
  const [replace, setReplace] = useState("");
  useEffect(() => buffers.subscribe(input.identity, () => setSnapshot(buffers.snapshot(input.identity))).dispose, [buffers, input.identity]);
  useEffect(() => { if (loaded.content !== undefined) setSnapshot(buffers.initialize(input.identity, loaded.content, loaded.generation)); }, [buffers, input.identity, loaded.content, loaded.generation]);
  useEffect(() => {
    const command = (event: Event) => {
      const name = (event as CustomEvent<{ name: string }>).detail?.name;
      if (name === "revert") buffers.revert(input.identity);
      if (name === "replace") setFindOpen(true);
      if (name === "selectAll") { textarea.current?.focus(); textarea.current?.select(); }
      if (name === "undo" || name === "redo") { textarea.current?.focus(); document.execCommand(name); }
    };
    window.addEventListener("yai:surface-command", command);
    return () => window.removeEventListener("yai:surface-command", command);
  }, [buffers, input.identity]);
  useEffect(() => { actions.updateSurface(input.id, { dirty: snapshot?.dirty }); }, [actions, input.id, snapshot?.dirty]);
  if (loaded.loading) return <article className="material-surface text-editor-surface"><MaterialHeader input={input} /><div className="surface-loading">Reading exact retained revision…</div></article>;
  if (loaded.error || !snapshot) return <article className="material-surface text-editor-surface"><MaterialHeader input={input} /><EmptyState title="File content unavailable" body={loaded.error ?? "No exact content is available."} /></article>;
  const lines = Math.max(1, snapshot.value.split("\n").length);
  const replaceNext = () => {
    if (!find) return;
    const start = snapshot.value.indexOf(find, textarea.current?.selectionEnd ?? 0);
    const index = start >= 0 ? start : snapshot.value.indexOf(find);
    if (index < 0) return;
    const next = snapshot.value.slice(0, index) + replace + snapshot.value.slice(index + find.length);
    buffers.update(input.identity, next);
    requestAnimationFrame(() => { textarea.current?.focus(); textarea.current?.setSelectionRange(index, index + replace.length); });
  };
  return <article className="material-surface text-editor-surface" data-surface-type={input.surfaceType}><MaterialHeader input={input} /><div className="surface-toolbar file-toolbar" role="toolbar"><span>{snapshot.stale ? "Case changed since this buffer opened" : snapshot.dirty ? "Local changes" : "Exact retained revision"}</span><Badge tone={snapshot.stale || snapshot.dirty ? "warning" : "success"}>{snapshot.stale ? "Stale" : snapshot.dirty ? "Unsaved" : "Current"}</Badge>{snapshot.stale && <button onClick={() => buffers.reload(input.identity)}>Reload</button>}<button onClick={() => setFindOpen((value) => !value)}>Find / Replace</button><button disabled title="YAI has no qualified participant-origin filesystem mutation contract">Save unavailable</button></div>{findOpen && <div className="editor-find"><input value={find} onChange={(event) => setFind(event.target.value)} placeholder="Find" aria-label="Find" /><input value={replace} onChange={(event) => setReplace(event.target.value)} placeholder="Replace" aria-label="Replace" /><button onClick={replaceNext}>Replace next</button><button onClick={() => setFindOpen(false)}>Close</button></div>}<div className="text-editor"><pre aria-hidden="true" className="line-numbers">{Array.from({ length: lines }, (_, index) => index + 1).join("\n")}</pre><textarea ref={textarea} spellCheck={false} value={snapshot.value} onChange={(event) => buffers.update(input.identity, event.target.value)} aria-label={`Edit ${input.title}`} /></div></article>;
}

export function MarkdownSurface(props: SurfaceRendererProps) {
  const { workspace, input } = props;
  const material = presentationMaterial(workspace, input.objectRef);
  if (material?.body.kind === "document") return <article className="material-surface markdown-surface" data-surface-type={input.surfaceType}><MaterialHeader input={input} material={material} /><section className="material-document searchable-content"><h2>{material.body.title}</h2><p>{material.body.intro}</p>{material.body.sections.map((section) => <section key={section.title}><h3>{section.title}</h3><p>{section.body}</p>{section.points && <ul>{section.points.map((point) => <li key={point}>{point}</li>)}</ul>}</section>)}</section></article>;
  return <ReadableTextPreview {...props} mode="markdown" />;
}

export function TextSurface(props: SurfaceRendererProps) {
  const { workspace, input } = props;
  const material = presentationMaterial(workspace, input.objectRef);
  const text = material ? materialText(material.body) : undefined;
  if (text !== undefined) return <article className="material-surface text-surface" data-surface-type={input.surfaceType}><MaterialHeader input={input} material={material} /><pre className="text-surface-content searchable-content">{text}</pre></article>;
  return <ReadableTextPreview {...props} mode="text" />;
}

export function StructuredTextSurface(props: SurfaceRendererProps) {
  const { workspace, input } = props;
  const material = presentationMaterial(workspace, input.objectRef);
  const source = material ? materialText(material.body) : undefined;
  const mediaType = input.metadata?.mediaType ?? material?.mediaType ?? "structured text";
  if (source === undefined) return <ReadableTextPreview {...props} mode="structured" />;
  return <article className="material-surface structured-text-surface" data-surface-type={input.surfaceType}><MaterialHeader input={input} material={material} /><pre className="structured-code searchable-content"><code>{prettyStructured(source, mediaType)}</code></pre></article>;
}

function ReadableTextPreview(props: SurfaceRendererProps & { mode: "markdown" | "text" | "structured" }) {
  const { input, mode } = props;
  const loaded = useReadableMaterial(props);
  return <article className={`material-surface ${mode}-surface`} data-surface-type={input.surfaceType}><MaterialHeader input={input} />{loaded.loading ? <div className="surface-loading">Reading exact retained revision…</div> : loaded.content !== undefined ? <pre className="text-surface-content searchable-content">{mode === "structured" ? prettyStructured(loaded.content, input.metadata?.mediaType ?? "") : loaded.content}</pre> : <EmptyState title="Content unavailable" body={loaded.error ?? "No qualified readable content projection."} />}</article>;
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

export async function searchMaterialSurface({ workspace, readMaterial, buffers }: WorkbenchRenderContext, input: SurfaceRendererProps["input"], query: string): Promise<readonly SurfaceSearchResult[]> {
  const material = presentationMaterial(workspace, input.objectRef);
  if (!query.trim()) return [];
  const needle = query.toLocaleLowerCase();
  if (material?.body.kind === "table") return material.body.rows.filter((row) => Object.values(row.values).some((value) => value.toLocaleLowerCase().includes(needle))).map((row) => ({ id: row.id, label: Object.values(row.values)[0] ?? row.id, detail: Object.values(row.values).slice(1).join(" · "), objectRef: row.id }));
  let text = buffers.snapshot(input.identity)?.value ?? (material ? materialText(material.body) : undefined);
  if (text === undefined) {
    const file = workspace.environment.files.find((item) => item.id === input.objectRef);
    if (file) {
      const result = await readMaterial({ case_ref: workspace.case.case_ref, source_ref: file.source_ref, revision_ref: file.revision_ref, path: file.path, expected_generation: workspace.case.generation });
      if (result.result_state === "success" && result.data?.encoding === "utf-8") text = result.data.content;
    }
  }
  return text?.split("\n").map((line, index) => ({ line: line.trim(), index })).filter(({ line }) => line.toLocaleLowerCase().includes(needle)).slice(0, 40).map(({ line, index }) => ({ id: `${input.identity}:${index}`, label: line || `Line ${index + 1}`, detail: `${input.title} · line ${index + 1}`, objectRef: input.objectRef })) ?? [];
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
