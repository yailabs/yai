import { useEffect, useRef, useState } from "react";
import { getDocument, GlobalWorkerOptions, type PDFDocumentProxy } from "pdfjs-dist";
import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";
import { EmptyState, IconButton } from "../../components/primitives";
import { Icon } from "../../components/Icon";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";
import { MaterialHeader, presentationMaterial } from "./MaterialSurfaces";

GlobalWorkerOptions.workerSrc = workerUrl;

export default function PdfSurface({ workspace, input }: SurfaceRendererProps) {
  const material = presentationMaterial(workspace, input.objectRef);
  const body = material?.body.kind === "pdf" ? material.body : undefined;
  const canvas = useRef<HTMLCanvasElement>(null);
  const stage = useRef<HTMLDivElement>(null);
  const [document, setDocument] = useState<PDFDocumentProxy>();
  const [page, setPage] = useState(1);
  const [scale, setScale] = useState(1.1);
  const [fit, setFit] = useState(true);
  const [query, setQuery] = useState("");
  const [matches, setMatches] = useState<number>();
  const [error, setError] = useState<string>();

  useEffect(() => {
    if (!body?.source) return;
    let active = true;
    const task = getDocument({ url: body.source });
    void task.promise.then((value) => { if (active) { setDocument(value); setPage(1); setError(undefined); } }, (reason) => active && setError(String(reason)));
    return () => { active = false; void task.destroy(); };
  }, [body?.source]);

  useEffect(() => {
    if (!document || !canvas.current) return;
    let cancelled = false;
    let render: { cancel(): void; promise: Promise<unknown> } | undefined;
    void document.getPage(page).then((pdfPage) => {
      if (cancelled || !canvas.current) return;
      const base = pdfPage.getViewport({ scale: 1 });
      const nextScale = fit && stage.current ? Math.max(.4, Math.min(2.5, (stage.current.clientWidth - 56) / base.width)) : scale;
      const viewport = pdfPage.getViewport({ scale: nextScale });
      const ratio = window.devicePixelRatio || 1;
      canvas.current.width = Math.floor(viewport.width * ratio);
      canvas.current.height = Math.floor(viewport.height * ratio);
      canvas.current.style.width = `${Math.floor(viewport.width)}px`;
      canvas.current.style.height = `${Math.floor(viewport.height)}px`;
      const context = canvas.current.getContext("2d");
      if (!context) return;
      context.setTransform(ratio, 0, 0, ratio, 0, 0);
      render = pdfPage.render({ canvas: canvas.current, canvasContext: context, viewport });
      void render.promise.catch((reason) => reason?.name !== "RenderingCancelledException" && setError(String(reason)));
    });
    return () => { cancelled = true; render?.cancel(); };
  }, [document, fit, page, scale]);

  useEffect(() => {
    const target = stage.current;
    if (!target || !fit) return;
    const observer = new ResizeObserver(() => setScale((value) => value + .00001));
    observer.observe(target);
    return () => observer.disconnect();
  }, [fit]);

  const search = async () => {
    if (!document || !query.trim()) { setMatches(undefined); return; }
    const needle = query.toLocaleLowerCase();
    let count = 0;
    for (let index = 1; index <= document.numPages; index += 1) {
      const content = await (await document.getPage(index)).getTextContent();
      const text = content.items.map((item) => "str" in item ? item.str : "").join(" ").toLocaleLowerCase();
      let cursor = text.indexOf(needle);
      while (cursor >= 0) { count += 1; cursor = text.indexOf(needle, cursor + needle.length); }
    }
    setMatches(count);
  };

  return <article className="material-surface pdf-surface" data-surface-type={input.surfaceType}><MaterialHeader input={input} material={material} />{body?.source ? <><div className="surface-toolbar pdf-toolbar" role="toolbar" aria-label="PDF controls"><IconButton aria-label="Previous page" disabled={page <= 1} onClick={() => setPage((value) => value - 1)}><Icon name="back" /></IconButton><label>Page <input type="number" min={1} max={document?.numPages ?? body.pageCount ?? 1} value={page} onChange={(event) => setPage(Math.max(1, Math.min(document?.numPages ?? 1, Number(event.target.value))))} /> of {document?.numPages ?? body.pageCount ?? "…"}</label><IconButton aria-label="Next page" disabled={!document || page >= document.numPages} onClick={() => setPage((value) => value + 1)}><Icon name="forward" /></IconButton><span className="toolbar-separator" /><button aria-pressed={fit} onClick={() => setFit(true)}>Fit width</button><button onClick={() => { setFit(false); setScale((value) => Math.max(.4, value - .15)); }}>−</button><button onClick={() => { setFit(false); setScale((value) => Math.min(3, value + .15)); }}>+</button><span className="toolbar-separator" /><input value={query} onChange={(event) => setQuery(event.target.value)} onKeyDown={(event) => event.key === "Enter" && void search()} placeholder="Search PDF" aria-label="Search PDF" /><button onClick={() => void search()}>Find</button>{matches !== undefined && <span>{matches} matches</span>}</div><div ref={stage} className="pdf-stage">{error ? <EmptyState title="PDF renderer unavailable" body={error} /> : <canvas ref={canvas} aria-label={`${input.title}, page ${page}`} />}</div></> : <EmptyState title="PDF content unavailable" body={body?.unavailableReason ?? "YAI exposes PDF identity, but no qualified document bytes or URL."} />}</article>;
}
