import type { CasePresentation } from "../../clients/dataSource";
import type { IconName } from "../../components/Icon";
import type { SurfaceInput } from "../../workbench/surface/model";
import { isPolicySource } from "../case/environment";

export const surfaceTypes = {
  perspective: "case.perspective",
  source: "environment.source",
  resource: "environment.resource",
  textEditor: "material.text-editor",
  markdown: "material.markdown",
  text: "material.text",
  structuredText: "material.structured-text",
  image: "material.image",
  pdf: "material.pdf",
  audio: "material.audio",
  video: "material.video",
  table: "data.table",
  timeline: "case.timeline",
  graph: "case.graph",
  recall: "memory.recall",
  workingState: "memory.working-state",
  settings: "studio.settings",
  unavailable: "material.unavailable",
} as const;

export function perspectiveInput(viewId: string, title: string, icon: IconName): SurfaceInput {
  const identity = `perspective:${viewId}`;
  return { id: identity, identity, surfaceType: surfaceTypes.perspective, title, icon, pinned: true, viewId };
}

export function settingsInput(): SurfaceInput {
  return { id: "settings", identity: "settings", surfaceType: surfaceTypes.settings, title: "Settings", icon: "settings", pinned: true };
}

export function timelineInput(pinned = false): SurfaceInput {
  const identity = "case-view:memory-timeline";
  return { id: pinned ? identity : "surface:preview", identity, surfaceType: surfaceTypes.timeline, title: "Case Timeline", icon: "memory", pinned, viewId: "Memory", metadata: { projection: "memory.timeline" } };
}

export function memoryInput(kind: "recall" | "workingState", pinned = true): SurfaceInput {
  const identity = `case-view:${kind}`;
  return { id: identity, identity, surfaceType: surfaceTypes[kind], title: kind === "recall" ? "Recall" : "Working State", icon: "memory", pinned, viewId: "Memory" };
}

export function graphInput(projection: "memory" | "knowledge", pinned = false): SurfaceInput {
  const identity = `case-view:${projection}-graph`;
  return { id: pinned ? identity : "surface:preview", identity, surfaceType: surfaceTypes.graph, title: projection === "memory" ? "Experience Graph" : "Knowledge Graph", icon: "graph", pinned, viewId: projection === "memory" ? "Memory" : "Knowledge", metadata: { projection } };
}

export function materialInput(
  workspace: CasePresentation,
  objectRef: string,
  title: string,
  pinned = false,
): SurfaceInput {
  const material = workspace.presentation.materials?.find((item) => item.id === objectRef);
  const file = workspace.environment.files.find((item) => item.id === objectRef);
  if (file) return fileInput(workspace, file.id, pinned);
  const document = workspace.knowledge.sources.find((item) => item.id === objectRef);
  const qualifiedFile = workspace.environment.files.find((item) =>
    document ? item.source_ref === document.source_ref && item.revision_ref === document.revision_ref && item.path === document.path && item.digest === document.digest
      : material && item.path === material.path);
  if (qualifiedFile) return fileInput(workspace, qualifiedFile.id, pinned);
  const sourceRef = document?.source_ref ?? objectRef;
  const source = workspace.environment.sources.find((item) => item.id === sourceRef);
  const qualified = workspace.knowledge.sources.find((item) => item.id === objectRef || item.source_ref === sourceRef);
  const mediaType = material?.mediaType ?? qualified?.media_type ?? source?.media_type;
  const identity = `material:${objectRef}`;
  return {
    id: pinned ? identity : "surface:preview",
    identity,
    surfaceType: resolveMaterialSurfaceType(mediaType),
    title,
    icon: fileIconForMedia(mediaType, material?.path),
    pinned,
    objectRef,
    metadata: mediaType ? { mediaType } : undefined,
  };
}

export function fileInput(
  workspace: CasePresentation,
  objectRef: string,
  pinned = false,
  surfaceType?: string,
): SurfaceInput {
  const file = workspace.environment.files.find((item) => item.id === objectRef);
  if (!file) return materialInput(workspace, objectRef, objectRef, pinned);
  const identity = `material:${objectRef}`;
  const resolved = surfaceType ?? resolveFileSurfaceType(file.media_type);
  return {
    id: pinned ? identity : "surface:preview",
    identity,
    surfaceType: resolved,
    title: file.path.split("/").at(-1) ?? file.path,
    icon: fileIconForMedia(file.media_type, file.path),
    pinned,
    posture: "ready",
    objectRef,
    metadata: {
      mediaType: file.media_type,
      path: file.path,
      sourceRef: file.source_ref,
      revisionRef: file.revision_ref,
      digest: file.digest,
      size: String(file.bytes),
    },
  };
}

export function sourceInput(workspace: CasePresentation, objectRef: string, pinned = false): SurfaceInput {
  const source = workspace.environment.sources.find((item) => item.id === objectRef);
  const identity = `source:${objectRef}`;
  const policy = source && isPolicySource(source);
  return { id: pinned ? identity : "surface:preview", identity, surfaceType: surfaceTypes.source, title: source?.label ?? objectRef, icon: policy ? "authority" : "source", pinned, objectRef, viewId: policy ? "Authority" : "Environment" };
}

export function resourceInput(workspace: CasePresentation, objectRef: string, pinned = false): SurfaceInput {
  const resource = workspace.environment.resources.find((item) => item.id === objectRef);
  const identity = `resource:${objectRef}`;
  return { id: pinned ? identity : "surface:preview", identity, surfaceType: surfaceTypes.resource, title: resource?.label ?? resource?.id ?? objectRef, icon: resource?.kind === "database" ? "database" : "resource", pinned, objectRef, viewId: "Environment" };
}

export function fileIconForMedia(mediaType?: string, path?: string): IconName {
  const type = mediaType?.split(";")[0];
  const name = path?.toLocaleLowerCase() ?? "";
  if (type === "text/markdown" || /\.(md|markdown)$/.test(name)) return "markdownFile";
  if (type === "application/json" || type === "application/ld+json" || /\.jsonc?$/.test(name)) return "jsonFile";
  if (type === "application/toml" || type === "application/yaml" || type === "application/x-yaml" || /\.(toml|ya?ml|xml)$/.test(name)) return "configFile";
  if (type === "application/pdf") return "pdf";
  if (type?.startsWith("image/")) return "image";
  if (type?.startsWith("audio/")) return "audio";
  if (type?.startsWith("video/")) return "video";
  if (/\.(rs|tsx?|jsx?|py|sh|bash|zsh|c|cc|cpp|cxx|h|hpp|css|html?)$/.test(name)) return "codeFile";
  if (type?.startsWith("text/") || structuredTextTypes.has(type ?? "")) return "textFile";
  return type === "application/octet-stream" ? "binaryFile" : "file";
}

export interface RendererChoice { type: string; title: string }
export function rendererChoices(mediaType?: string): readonly RendererChoice[] {
  const type = mediaType?.split(";")[0];
  if (type === "text/markdown") return [{ type: surfaceTypes.textEditor, title: "Text Editor" }, { type: surfaceTypes.markdown, title: "Markdown Preview" }];
  if (type === "image/svg+xml") return [{ type: surfaceTypes.textEditor, title: "SVG/Text Editor" }, { type: surfaceTypes.image, title: "Image Preview" }];
  if (type === "text/csv" || type === "text/tab-separated-values") return [{ type: surfaceTypes.textEditor, title: "Text Editor" }, { type: surfaceTypes.table, title: "Table" }];
  if (type?.startsWith("text/") || structuredTextTypes.has(type ?? "")) return [{ type: surfaceTypes.textEditor, title: "Text Editor" }];
  return [{ type: resolveMaterialSurfaceType(mediaType), title: rendererTitle(resolveMaterialSurfaceType(mediaType)) }];
}

function resolveFileSurfaceType(mediaType?: string) {
  const type = mediaType?.split(";")[0];
  return type?.startsWith("text/") || structuredTextTypes.has(type ?? "") ? surfaceTypes.textEditor : resolveMaterialSurfaceType(mediaType);
}

function rendererTitle(type: string) {
  return ({ [surfaceTypes.image]: "Image Preview", [surfaceTypes.pdf]: "PDF Viewer", [surfaceTypes.audio]: "Audio Player", [surfaceTypes.video]: "Video Player", [surfaceTypes.table]: "Table", [surfaceTypes.unavailable]: "Metadata" } as Record<string, string>)[type] ?? "Viewer";
}

export function resolveMaterialSurfaceType(mediaType?: string): string {
  if (mediaType === "text/markdown") return surfaceTypes.markdown;
  if (mediaType && structuredTextTypes.has(mediaType.split(";")[0])) return surfaceTypes.structuredText;
  if (mediaType?.startsWith("text/")) return surfaceTypes.text;
  if (mediaType && safeImageTypes.has(mediaType.split(";")[0])) return surfaceTypes.image;
  if (mediaType === "application/pdf") return surfaceTypes.pdf;
  if (mediaType?.startsWith("audio/")) return surfaceTypes.audio;
  if (mediaType?.startsWith("video/")) return surfaceTypes.video;
  if (mediaType === "application/vnd.yai.table+json") return surfaceTypes.table;
  return surfaceTypes.unavailable;
}

const structuredTextTypes = new Set([
  "application/json", "application/ld+json", "application/yaml", "application/x-yaml",
  "application/toml", "application/xml", "text/xml", "text/csv", "text/tab-separated-values",
]);

const safeImageTypes = new Set(["image/png", "image/jpeg", "image/webp", "image/gif", "image/svg+xml"]);
