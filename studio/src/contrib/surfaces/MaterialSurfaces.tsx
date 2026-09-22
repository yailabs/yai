import { lazy, Suspense, useCallback, useEffect, useMemo, useState, useSyncExternalStore } from "react";
import { Badge, Button, EmptyState } from "../../components/primitives";
import type { MaterialView } from "../../clients/presentation";
import type { SurfaceRendererProps, SurfaceSearchResult, WorkbenchRenderContext } from "../../workbench/kernel/types";
import { expectedMaterialIdentity, materialDataUrl, materialText, useReadableMaterial } from "./materialRead";
import { validateMaterialContent, validateMaterialRead } from "./materialIdentity";

const MarkdownPreview = lazy(() => import("./MarkdownPreview"));
const CodeEditorSurface = lazy(() => import("./CodeEditorSurface"));

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

export function TextEditorSurface(props: SurfaceRendererProps) {
  const { input, actions, buffers } = props;
  const loaded = useReadableMaterial(props);
  const subscribe = useCallback((listener: () => void) => buffers.subscribe(input.identity, listener).dispose, [buffers, input.identity]);
  const snapshot = useSyncExternalStore(subscribe, () => buffers.snapshot(input.identity));
  useEffect(() => {
    if (loaded.content !== undefined && loaded.source) {
      buffers.initialize(input.identity, loaded.content, loaded.source);
    }
  }, [buffers, input.identity, loaded.content, loaded.source]);
  const registerCommands = useCallback((editorCommand: (name: string) => void) => {
    const command = (event: Event) => {
      const name = (event as CustomEvent<{ name: string }>).detail?.name;
      if (name === "revert") buffers.revert(input.identity);
      else editorCommand(name);
    };
    window.addEventListener("yai:surface-command", command);
    return () => window.removeEventListener("yai:surface-command", command);
  }, [buffers, input.identity]);
  useEffect(() => { actions.updateSurface(input.id, { dirty: snapshot?.dirty }); }, [actions.updateSurface, input.id, snapshot?.dirty]);
  if (loaded.loading && !snapshot) return <article className="material-surface text-editor-surface"><MaterialHeader input={input} /><div className="surface-loading">Reading exact retained revision…</div></article>;
  if ((loaded.error && !snapshot?.dirty) || !snapshot) return <article className="material-surface text-editor-surface"><MaterialHeader input={input} /><EmptyState title="File content unavailable" body={loaded.error ?? "No exact content is available."} /></article>;
  return <article className="material-surface text-editor-surface" data-surface-type={input.surfaceType} data-material-key={snapshot.source.key}><MaterialHeader input={input} />{loaded.error && <p className="file-read-warning" role="alert">{loaded.error} Local edits are retained and have not been saved.</p>}<div className="surface-toolbar file-toolbar" role="toolbar"><span>{loaded.loading ? "Checking current revision…" : snapshot.stale ? "A newer revision is available" : snapshot.dirty ? "Local changes" : "Exact retained revision"}</span><Badge tone={snapshot.stale || snapshot.dirty ? "warning" : "success"}>{snapshot.stale ? "Stale" : snapshot.dirty ? "Unsaved" : "Current"}</Badge>{snapshot.stale && <button onClick={() => buffers.reload(input.identity)}>Reload</button>}<button onClick={() => window.dispatchEvent(new CustomEvent("yai:surface-command", { detail: { name: "find" } }))}>Find</button><button onClick={() => window.dispatchEvent(new CustomEvent("yai:surface-command", { detail: { name: "replace" } }))}>Replace</button><button disabled title="YAI has no qualified participant-origin filesystem mutation contract">Save unavailable</button></div><Suspense fallback={<div className="surface-loading">Loading syntax-aware editor…</div>}><CodeEditorSurface editing={props.platform.editing} buffers={buffers} identity={input.identity} title={input.title} path={input.metadata?.path} mediaType={input.metadata?.mediaType} snapshot={snapshot} readOnly={input.posture === "read-only"} onChange={(value) => buffers.update(input.identity, value)} onCommand={registerCommands} /></Suspense></article>;
}

export function MarkdownSurface(props: SurfaceRendererProps) {
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
  const subscribe = useCallback((listener: () => void) => props.buffers.subscribe(input.identity, listener).dispose, [props.buffers, input.identity]);
  const buffer = useSyncExternalStore(subscribe, () => props.buffers.snapshot(input.identity));
  const content = buffer?.dirty ? buffer.value : loaded.content;
  return <article className={`material-surface ${mode}-surface`} data-surface-type={input.surfaceType}><MaterialHeader input={input} />{buffer?.dirty && <p className="preview-local-note">{buffer.stale ? "Stale local draft" : "Preview of unsaved local changes"} · not committed to YAI</p>}{buffer?.dirty && loaded.error && <p className="file-read-warning" role="alert">{loaded.error} Only the unsaved local draft is shown.</p>}{content !== undefined ? mode === "markdown" ? <Suspense fallback={<p className="surface-loading">Loading Markdown preview…</p>}><MarkdownPreview {...props} content={content} /></Suspense> : <pre className="text-surface-content searchable-content">{mode === "structured" ? prettyStructured(content, input.metadata?.mediaType ?? "") : content}</pre> : loaded.loading ? <div className="surface-loading">Reading exact retained revision…</div> : <EmptyState title="Content unavailable" body={loaded.error ?? "No qualified readable content projection."} />}</article>;
}

export function ImageSurface(props: SurfaceRendererProps) {
  const { workspace, input, actions } = props;
  const material = presentationMaterial(workspace, input.objectRef);
  const body = material?.body.kind === "image" ? material.body : undefined;
  const loaded = useReadableMaterial(props, "bytes", Boolean(body));
  const subscribe = useCallback((listener: () => void) => props.buffers.subscribe(input.identity, listener).dispose, [props.buffers, input.identity]);
  const buffer = useSyncExternalStore(subscribe, () => props.buffers.snapshot(input.identity));
  const draft = input.metadata?.mediaType === "image/svg+xml" && buffer?.dirty ? buffer.value : undefined;
  const source = useMemo(() => draft !== undefined ? `data:image/svg+xml;charset=utf-8,${encodeURIComponent(draft)}` : body?.source ?? (loaded.data ? materialDataUrl(loaded.data) : undefined), [draft, body?.source, loaded.data]);
  const [scale, setScale] = useState<"fit" | "actual" | "large">("fit");
  const [failed, setFailed] = useState<string>();
  return <article className="material-surface image-surface" data-surface-type={input.surfaceType}><MaterialHeader input={input} material={material} />{draft !== undefined && <p className="preview-local-note">{buffer?.stale ? "Stale local SVG draft" : "Preview of unsaved local SVG"} · inert image</p>}{draft !== undefined && loaded.error && <p className="file-read-warning" role="alert">{loaded.error} Only the unsaved local draft is shown.</p>}<div className="surface-toolbar" role="toolbar" aria-label="Image controls"><div className="segmented"><button aria-pressed={scale === "fit"} onClick={() => setScale("fit")}>Fit</button><button aria-pressed={scale === "actual"} onClick={() => setScale("actual")}>100%</button><button aria-pressed={scale === "large"} onClick={() => setScale("large")}>150%</button></div>{body && <Badge>{body.width} × {body.height}</Badge>}</div>{source && failed !== source ? <button className={`image-stage ${scale}`} onClick={() => input.objectRef && actions.inspect(input.objectRef)} aria-label={`Inspect ${input.title}`}><img key={source} src={source} alt={body?.alt ?? input.title} width={body?.width} height={body?.height} onError={() => setFailed(source)} /></button> : <EmptyState title="Image unavailable" body={loaded.loading ? "Reading exact retained image…" : loaded.error ?? (failed ? "The WebView could not decode this exact image." : "No qualified image source is exposed.")} />}{body?.caption && <p className="surface-caption">{body.caption}</p>}</article>;
}

export function AudioSurface(props: SurfaceRendererProps) { return <MediaSurface {...props} kind="audio" />; }
export function VideoSurface(props: SurfaceRendererProps) { return <MediaSurface {...props} kind="video" />; }
function MediaSurface(props: SurfaceRendererProps & { kind: "audio" | "video" }) {
  const { workspace, input, kind } = props;
  const material = presentationMaterial(workspace, input.objectRef);
  const body = material?.body.kind === kind ? material.body : undefined;
  const loaded = useReadableMaterial(props, "bytes", Boolean(body));
  const source = useMemo(() => body?.source ?? (loaded.data ? materialDataUrl(loaded.data) : undefined), [body?.source, loaded.data]);
  const [failed, setFailed] = useState<string>();
  return <article className={`material-surface media-surface ${kind}-surface`} data-surface-type={input.surfaceType}><MaterialHeader input={input} material={material} />{source && failed !== source ? kind === "audio" ? <audio key={source} onError={() => setFailed(source)} controls preload="metadata" src={source}>Audio playback is unavailable.</audio> : <video key={source} onError={() => setFailed(source)} controls preload="metadata" src={source}>Video playback is unavailable.</video> : <EmptyState title={`${kind === "audio" ? "Audio" : "Video"} unavailable`} body={loaded.loading ? "Reading exact retained media…" : loaded.error ?? (failed ? "This WebView cannot play the qualified media format." : body?.unavailableReason ?? "No qualified media source is exposed to Studio.")} />}{body?.caption && <p className="surface-caption">{body.caption}</p>}</article>;
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
      const expected = expectedMaterialIdentity({ workspace, input });
      if (result.result_state === "success" && result.data?.encoding === "utf-8" && expected && !validateMaterialRead(expected, result.data) && !await validateMaterialContent(result.data)) text = result.data.content;
    }
  }
  return text?.split("\n").map((line, index) => ({ line: line.trim(), index })).filter(({ line }) => line.toLocaleLowerCase().includes(needle)).slice(0, 40).map(({ line, index }) => ({ id: `${input.identity}:${index}`, label: line || `Line ${index + 1}`, detail: `${input.title} · line ${index + 1}`, objectRef: input.objectRef })) ?? [];
}

export async function searchPdfSurface({ workspace, readMaterial }: Pick<SurfaceRendererProps, "workspace" | "readMaterial">, input: SurfaceRendererProps["input"], query: string): Promise<readonly SurfaceSearchResult[]> {
  const material = presentationMaterial(workspace, input.objectRef);
  const source = material?.body.kind === "pdf" ? material.body.source : undefined;
  const materialId = material?.id ?? input.objectRef;
  const needle = query.trim().toLocaleLowerCase();
  if (!materialId || !needle) return [];
  let data: Uint8Array<ArrayBuffer> | undefined;
  if (!source) {
    const expected = expectedMaterialIdentity({ workspace, input });
    if (!expected) return [];
    const result = await readMaterial({ case_ref: expected.caseRef, source_ref: expected.sourceRef, revision_ref: expected.revisionRef, path: expected.path, expected_generation: expected.generation });
    if (result.result_state !== "success" || !result.data || validateMaterialRead(expected, result.data) || await validateMaterialContent(result.data)) return [];
    data = result.data.encoding === "utf-8" ? new TextEncoder().encode(result.data.content) : Uint8Array.from(atob(result.data.content), char => char.charCodeAt(0));
  }
  const [{ getDocument, GlobalWorkerOptions }, worker] = await Promise.all([
    import("pdfjs-dist"),
    import("pdfjs-dist/build/pdf.worker.min.mjs?url"),
  ]);
  GlobalWorkerOptions.workerSrc = worker.default;
  const task = getDocument(data ? { data } : { url: source });
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

function prettyStructured(source: string, mediaType: string) {
  if (mediaType.includes("json")) { try { return JSON.stringify(JSON.parse(source), null, 2); } catch { return source; } }
  return source;
}
