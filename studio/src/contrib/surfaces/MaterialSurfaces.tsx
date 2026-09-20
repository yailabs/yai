import { useState } from "react";
import { Badge, Button, EmptyState } from "../../components/primitives";
import type { MaterialView } from "../../clients/presentation";
import type { SurfaceRendererProps } from "../../workbench/kernel/types";

function fixtureMaterial(workspace: SurfaceRendererProps["workspace"], objectRef?: string) {
  return workspace.presentation.materials?.find((item) => item.id === objectRef);
}

function MaterialHeader({ input, material }: { input: SurfaceRendererProps["input"]; material?: MaterialView }) {
  return <header className="material-header">
    <span>{material?.provenance ?? "Qualified Case presentation"}</span>
    <h1>{input.title}</h1>
    <p>{material?.path ?? input.objectRef ?? "No object reference"} · {input.metadata?.mediaType ?? "media type unavailable"}</p>
  </header>;
}

export function MarkdownSurface({ workspace, input }: SurfaceRendererProps) {
  const material = fixtureMaterial(workspace, input.objectRef);
  if (material?.body.kind === "document") return <article className="material-surface markdown-surface" data-surface-type={input.surfaceType}>
    <MaterialHeader input={input} material={material} />
    <section className="material-document"><h2>{material.body.title}</h2><p>{material.body.intro}</p>{material.body.sections.map((section) => <section key={section.title}><h3>{section.title}</h3><p>{section.body}</p>{section.points && <ul>{section.points.map((point) => <li key={point}>{point}</li>)}</ul>}</section>)}</section>
  </article>;
  return <LiveTextProjection workspace={workspace} input={input} />;
}

export function TextSurface({ workspace, input }: SurfaceRendererProps) {
  const material = fixtureMaterial(workspace, input.objectRef);
  if (material?.body.kind === "diff") return <article className="material-surface text-surface" data-surface-type={input.surfaceType}>
    <MaterialHeader input={input} material={material} />
    <pre className="text-surface-content">{material.body.lines.map((line) => `${line.change === "add" ? "+ " : line.change === "remove" ? "- " : "  "}${line.text}`).join("\n")}</pre>
  </article>;
  if (material?.body.kind === "document") return <article className="material-surface text-surface" data-surface-type={input.surfaceType}><MaterialHeader input={input} material={material} /><pre className="text-surface-content">{[material.body.title, material.body.intro, ...material.body.sections.flatMap((section) => [section.title, section.body, ...(section.points ?? [])])].join("\n\n")}</pre></article>;
  return <LiveTextProjection workspace={workspace} input={input} />;
}

function LiveTextProjection({ workspace, input }: Pick<SurfaceRendererProps, "workspace" | "input">) {
  const file = workspace.environment.files.find((item) => item.id === input.objectRef);
  const source = workspace.environment.sources.find((item) => item.id === (file?.source_ref ?? input.objectRef));
  const qualified = workspace.knowledge.sources.find((item) => item.id === input.objectRef || item.source_ref === (file?.source_ref ?? source?.id));
  const sourceKeys = new Set([qualified?.id, qualified?.source_ref, file?.source_ref, source?.id].filter(Boolean));
  const units = workspace.knowledge.units.filter((unit) => sourceKeys.has(unit.source_ref) && (!file || !qualified?.path || qualified.path === file.path));
  return <article className="material-surface text-surface" data-surface-type={input.surfaceType}><MaterialHeader input={input} /><dl className="material-metadata"><div><dt>Case</dt><dd>{workspace.case.case_ref}</dd></div><div><dt>Revision</dt><dd>{file?.revision_ref ?? source?.revision_ref ?? qualified?.revision_ref ?? "Unavailable"}</dd></div></dl>{units.length ? <section className="material-document">{units.map((unit) => <div key={unit.id} className={unit.kind === "heading" ? "heading" : "body"}>{unit.text}</div>)}</section> : <EmptyState title="Content unavailable" body="YAI exposes this material identity, but no qualified readable content projection." />}</article>;
}

export function ImageSurface({ workspace, input, actions }: SurfaceRendererProps) {
  const material = fixtureMaterial(workspace, input.objectRef);
  const body = material?.body.kind === "image" ? material.body : undefined;
  const [naturalSize, setNaturalSize] = useState(false);
  return <article className="material-surface image-surface" data-surface-type={input.surfaceType}>
    <MaterialHeader input={input} material={material} />
    {body ? <><div className="surface-toolbar"><Button onClick={() => setNaturalSize((value) => !value)}>{naturalSize ? "Fit" : "Natural size"}</Button><Badge>{body.width} × {body.height}</Badge></div><button className={`image-stage ${naturalSize ? "natural" : "fit"}`} onClick={() => input.objectRef && actions.inspect(input.objectRef)} aria-label={`Inspect ${input.title}`}><img src={body.source} alt={body.alt} width={body.width} height={body.height} /></button>{body.caption && <p className="surface-caption">{body.caption}</p>}</> : <EmptyState title="Image unavailable" body="The Case exposes image media, but no qualified image bytes or URL." />}
  </article>;
}

export function PdfSurface({ workspace, input }: SurfaceRendererProps) {
  const material = fixtureMaterial(workspace, input.objectRef);
  const body = material?.body.kind === "pdf" ? material.body : undefined;
  return <article className="material-surface pdf-surface" data-surface-type={input.surfaceType}><MaterialHeader input={input} material={material} />{body?.source ? <object data={body.source} type="application/pdf" aria-label={input.title}><EmptyState title="PDF renderer unavailable" body="The desktop webview could not render this qualified PDF source." /></object> : <EmptyState title="PDF content unavailable" body={body?.unavailableReason ?? "YAI exposes PDF identity, but no qualified document bytes or URL."} />}{body?.pageCount && <p className="surface-caption">{body.pageCount} pages reported by the authored fixture.</p>}</article>;
}

export function TableSurface({ workspace, input, actions }: SurfaceRendererProps) {
  const material = fixtureMaterial(workspace, input.objectRef);
  const body = material?.body.kind === "table" ? material.body : undefined;
  return <article className="material-surface table-surface" data-surface-type={input.surfaceType}><MaterialHeader input={input} material={material} />{body ? <div className="table-scroll"><table><thead><tr>{body.columns.map((column) => <th key={column.key}>{column.label}</th>)}</tr></thead><tbody>{body.rows.map((row) => <tr key={row.id} tabIndex={0} onClick={() => actions.inspect(row.id)}>{body.columns.map((column) => <td key={column.key}>{row.values[column.key] ?? "—"}</td>)}</tr>)}</tbody></table></div> : <EmptyState title="Table unavailable" body="No qualified structured rows are available." />}</article>;
}

export function UnavailableMaterialSurface({ workspace, input }: SurfaceRendererProps) {
  const material = fixtureMaterial(workspace, input.objectRef);
  return <article className="material-surface unavailable-material" data-surface-type={input.surfaceType}><MaterialHeader input={input} material={material} /><EmptyState title="No trusted renderer" body={`Studio has no trusted Surface renderer for ${input.metadata?.mediaType ?? "this qualified media type"}. The material has not been coerced into text.`} /></article>;
}
